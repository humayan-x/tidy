//! Implementation of the `tidy run` command for single-pass scanning and organizing.

use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

use crate::cli::format::{print_dry_run_table, print_run_summary, PlannedMovePreview};
use crate::cli::RunArgs;
use crate::common::error::{Result, TidyError};
use crate::core::classifier::{ClassificationResult, Classifier};
use crate::core::config::Config;
use crate::safety::ledger::Ledger;
use crate::safety::mover::{safe_move, safe_move_with_claimed};
use crate::safety::symlink::should_traverse_dir;

/// A planned file move discovered during directory scanning.
#[derive(Debug, Clone)]
pub struct PlannedMove {
    pub source: PathBuf,
    pub proposed_dest: PathBuf,
    pub category: String,
    pub file_size: u64,
}

/// Executes a single-pass scan and organize cycle.
pub fn execute_run(args: &RunArgs, custom_config: Option<&Path>) -> Result<()> {
    let target_dir = match &args.path {
        Some(p) => {
            if !p.exists() {
                return Err(TidyError::PathResolution(format!(
                    "Target path does not exist: {}",
                    p.display()
                )));
            }
            fs::canonicalize(p)?
        }
        None => fs::canonicalize(std::env::current_dir()?)?,
    };

    let config = Config::load_or_default(custom_config)?;
    let rules = config.compile()?;
    let classifier = Classifier::new(rules);
    let planned_moves = scan_directory(&target_dir, args.recursive, &classifier)?;

    if planned_moves.is_empty() {
        println!("No files to organize in '{}'.", target_dir.display());
        return Ok(());
    }

    if args.dry_run {
        let mut previews = Vec::with_capacity(planned_moves.len());
        let mut claimed_paths = HashSet::new();

        for p in &planned_moves {
            // Simulate collision resolution with batch claimed tracking without modifying disk
            let outcome = safe_move_with_claimed(
                &p.source,
                &p.proposed_dest,
                true,
                Some(&mut claimed_paths),
            )?;
            let source_name = p
                .source
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let dest_rel = outcome
                .destination
                .strip_prefix(&target_dir)
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| outcome.destination.to_string_lossy().to_string());

            previews.push(PlannedMovePreview {
                source_name,
                dest_display: dest_rel,
                category: p.category.clone(),
                file_size: p.file_size,
                was_collision: outcome.was_collision,
            });
        }

        print_dry_run_table(&target_dir, &previews);
        return Ok(());
    }

    // Live Execution
    let start_time = Instant::now();
    let pb = ProgressBar::new(planned_moves.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .map_err(|e| TidyError::Other(anyhow::anyhow!(e)))?
            .progress_chars("━╾─"),
    );

    let mut outcomes = Vec::with_capacity(planned_moves.len());
    let mut total_bytes = 0;
    let mut move_error = None;

    for m in &planned_moves {
        let file_name = m.source.file_name().unwrap_or_default().to_string_lossy();
        pb.set_message(format!("Moving {}", file_name));

        match safe_move(&m.source, &m.proposed_dest, false) {
            Ok(outcome) => {
                total_bytes += outcome.file_size;
                outcomes.push(outcome);
                pb.inc(1);
            }
            Err(err) => {
                move_error = Some(err);
                break;
            }
        }
    }

    pb.finish_and_clear();

    if let Some(err) = move_error {
        if !outcomes.is_empty() {
            let mut ledger = Ledger::open_default()?;
            let run = ledger.record_run_with_status(
                "tidy run",
                &target_dir,
                &outcomes,
                "PARTIAL_FAILURE",
            )?;
            eprintln!(
                "  ⚠ Run halted due to error. {} of {} files were moved before failure.",
                outcomes.len(),
                planned_moves.len()
            );
            eprintln!(
                "    Run ID: {} (use 'tidy undo' to roll back moved files)",
                run.uuid
            );
        }
        return Err(err);
    }

    // Commit run batch to SQLite ledger
    let mut ledger = Ledger::open_default()?;
    let run = ledger.record_run("tidy run", &target_dir, &outcomes)?;

    print_run_summary(
        start_time.elapsed().as_millis(),
        outcomes.len(),
        total_bytes,
        &run.uuid,
    );

    Ok(())
}

/// Scans the target directory and gathers all candidate moves according to the classifier.
pub fn scan_directory(
    target_dir: &Path,
    recursive: bool,
    classifier: &Classifier,
) -> Result<Vec<PlannedMove>> {
    let mut planned = Vec::new();
    let active_output_folders = classifier.active_destinations();

    let mut walker = WalkDir::new(target_dir);
    if !recursive {
        walker = walker.max_depth(1);
    }

    for entry in walker.into_iter().filter_entry(|e| {
        // Always include the root directory
        if e.path() == target_dir {
            return true;
        }

        let file_name = match e.file_name().to_str() {
            Some(n) => n,
            None => return false,
        };

        // If directory, apply directory-specific traversal checks
        if e.file_type().is_dir() {
            // Don't traverse active destination folders at root level (Images/, Documents/, etc.)
            if e.depth() == 1 && active_output_folders.contains(file_name) {
                return false;
            }

            // Check symlink safety for directories: do not follow directory symlinks
            if !should_traverse_dir(e.path()) {
                return false;
            }

            // Filter hidden directories if configured
            if classifier.rules().ignore_hidden && file_name.starts_with('.') {
                return false;
            }
        }

        true
    }) {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!("Skipping unreadable entry during scan: {}", err);
                continue;
            }
        };

        // Process only files (or file symlinks)
        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        let classification = classifier.classify(path, target_dir);

        if let ClassificationResult::Move {
            category,
            target_path,
            ..
        } = classification
        {
            let file_size = fs::symlink_metadata(path).map(|m| m.len()).unwrap_or(0);
            planned.push(PlannedMove {
                source: path.to_path_buf(),
                proposed_dest: target_path,
                category,
                file_size,
            });
        }
    }

    Ok(planned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_scan_directory_non_recursive() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let f1 = root.join("pic.png");
        let f2 = root.join("notes.txt");
        let f3 = root.join("ignored.crdownload");
        File::create(&f1).unwrap().write_all(b"png").unwrap();
        File::create(&f2).unwrap().write_all(b"txt").unwrap();
        File::create(&f3).unwrap().write_all(b"partial").unwrap();

        let config = Config::default();
        let rules = config.compile().unwrap();
        let classifier = Classifier::new(rules);

        let planned = scan_directory(root, false, &classifier).unwrap();
        assert_eq!(planned.len(), 2);
        let categories: Vec<&str> = planned.iter().map(|p| p.category.as_str()).collect();
        assert!(categories.contains(&"Images"));
        assert!(categories.contains(&"Documents"));
    }
}
