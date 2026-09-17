//! Undo & Rollback engine for safely reversing operations from the ledger.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::error::{Result, TidyError};
use crate::safety::ledger::Ledger;
use crate::safety::mover::safe_move;

/// Individual outcome of attempting to undo a file move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoOutcome {
    /// File was successfully restored to the source path (or non-colliding fallback).
    Restored { from: PathBuf, to: PathBuf },
    /// Destination file was missing from disk (e.g., deleted or moved manually).
    SkippedMissing { destination: PathBuf },
    /// Reversal failed with an error.
    Failed { destination: PathBuf, error: String },
}

/// Comprehensive report of the rollback execution.
#[derive(Debug, Clone)]
pub struct UndoReport {
    /// The UUID of the run that was rolled back.
    pub run_uuid: String,
    /// Number of files successfully moved back.
    pub restored_count: usize,
    /// Number of files that could not be restored because they no longer existed.
    pub skipped_count: usize,
    /// Category directories that became empty and were pruned.
    pub pruned_dirs: Vec<PathBuf>,
    /// Detailed breakdown of each operation's outcome.
    pub details: Vec<UndoOutcome>,
    /// Whether this was executed as a dry-run preview.
    pub is_dry_run: bool,
}

/// Rolls back an entire run's operations in reverse chronological order.
pub fn execute_undo(ledger: &mut Ledger, target_run_uuid: Option<&str>) -> Result<UndoReport> {
    execute_undo_opt(ledger, target_run_uuid, false)
}

/// Rolls back an entire run's operations with optional dry-run preview mode.
pub fn execute_undo_opt(
    ledger: &mut Ledger,
    target_run_uuid: Option<&str>,
    dry_run: bool,
) -> Result<UndoReport> {
    let run = match target_run_uuid {
        Some(uuid) => ledger
            .get_run_by_uuid(uuid)?
            .ok_or_else(|| TidyError::Database(rusqlite::Error::QueryReturnedNoRows))?,
        None => ledger.get_latest_completed_run()?.ok_or_else(|| {
            TidyError::Aborted("No completed runs available to undo in history ledger".into())
        })?,
    };

    let operations = ledger.get_operations_for_run(run.id)?;
    let mut details = Vec::new();
    let mut restored_count = 0;
    let mut skipped_count = 0;
    let mut candidate_prune_dirs: HashSet<PathBuf> = HashSet::new();

    for op in operations {
        if !op.destination_path.exists() {
            details.push(UndoOutcome::SkippedMissing {
                destination: op.destination_path,
            });
            skipped_count += 1;
            continue;
        }

        // Track the parent category folder to prune later if left empty
        if let Some(parent) = op.destination_path.parent() {
            candidate_prune_dirs.insert(parent.to_path_buf());
        }

        // Restore file from destination back to source (using safe_move to prevent overwriting)
        match safe_move(&op.destination_path, &op.source_path, dry_run) {
            Ok(outcome) => {
                restored_count += 1;
                details.push(UndoOutcome::Restored {
                    from: outcome.source,
                    to: outcome.destination,
                });
            }
            Err(e) => {
                details.push(UndoOutcome::Failed {
                    destination: op.destination_path,
                    error: e.to_string(),
                });
            }
        }
    }

    let mut pruned_dirs = Vec::new();

    if !dry_run {
        // Prune empty category directories left behind
        // Sort directories by depth descending so deepest child directories are pruned before their parents
        let mut sorted_dirs: Vec<PathBuf> = candidate_prune_dirs.into_iter().collect();
        sorted_dirs.sort_by_key(|a| std::cmp::Reverse(a.components().count()));

        let root_path = PathBuf::from(&run.root_path);
        let canonical_root = fs::canonicalize(&root_path).unwrap_or_else(|_| root_path.clone());

        for dir in sorted_dirs {
            let mut current = dir;
            while current != root_path {
                let canonical_current =
                    fs::canonicalize(&current).unwrap_or_else(|_| current.clone());
                if !canonical_current.starts_with(&canonical_root)
                    || canonical_current == canonical_root
                {
                    break;
                }

                if is_dir_empty(&current) && fs::remove_dir(&current).is_ok() {
                    if !pruned_dirs.contains(&current) {
                        pruned_dirs.push(current.clone());
                    }
                    if let Some(parent) = current.parent() {
                        current = parent.to_path_buf();
                        continue;
                    }
                }
                break;
            }
        }

        // Mark the run as REVERTED in SQLite
        ledger.mark_run_status(run.id, "REVERTED")?;
    }

    Ok(UndoReport {
        run_uuid: run.uuid,
        restored_count,
        skipped_count,
        pruned_dirs,
        details,
        is_dry_run: dry_run,
    })
}

/// Checks if a directory exists and contains zero entries.
fn is_dir_empty(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_execute_undo_full_cycle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        let db_path = root.join("history.db");
        let mut ledger = Ledger::open_at(&db_path).unwrap();

        // 1. Setup original files
        let src1 = root.join("photo.jpg");
        let src2 = root.join("notes.txt");
        File::create(&src1)
            .unwrap()
            .write_all(b"image data")
            .unwrap();
        File::create(&src2)
            .unwrap()
            .write_all(b"text data")
            .unwrap();

        // 2. Perform moves to category folders
        let dest1 = root.join("Images").join("photo.jpg");
        let dest2 = root.join("Documents").join("notes.txt");
        let m1 = safe_move(&src1, &dest1, false).unwrap();
        let m2 = safe_move(&src2, &dest2, false).unwrap();

        // 3. Record in ledger
        let moves = vec![m1, m2];
        let run = ledger.record_run("tidy run", root, &moves).unwrap();
        assert_eq!(run.status, "COMPLETED");

        // Verify files are at destinations and original paths are gone
        assert!(!src1.exists());
        assert!(!src2.exists());
        assert!(dest1.exists());
        assert!(dest2.exists());

        // 4. Execute undo
        let report = execute_undo(&mut ledger, None).unwrap();
        assert_eq!(report.run_uuid, run.uuid);
        assert_eq!(report.restored_count, 2);
        assert_eq!(report.skipped_count, 0);

        // Verify files are restored to their exact original locations
        assert!(src1.exists());
        assert_eq!(fs::read(&src1).unwrap(), b"image data");
        assert!(src2.exists());
        assert_eq!(fs::read(&src2).unwrap(), b"text data");

        // Verify empty category directories were pruned
        assert!(!dest1.parent().unwrap().exists());
        assert!(!dest2.parent().unwrap().exists());
    }

    #[test]
    fn test_undo_with_source_collision() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        let db_path = root.join("history.db");
        let mut ledger = Ledger::open_at(&db_path).unwrap();

        let src = root.join("photo.jpg");
        File::create(&src).unwrap().write_all(b"old photo").unwrap();

        let dest = root.join("Images").join("photo.jpg");
        let m = safe_move(&src, &dest, false).unwrap();
        ledger.record_run("tidy run", root, &[m]).unwrap();

        // User placed a NEW photo.jpg at the original source path before running undo!
        File::create(&src).unwrap().write_all(b"new photo").unwrap();

        // Undo must not overwrite the new photo!
        let report = execute_undo(&mut ledger, None).unwrap();
        assert_eq!(report.restored_count, 1);

        // Original new photo stays intact
        assert_eq!(fs::read(&src).unwrap(), b"new photo");
        // Restored file gets non-colliding name: photo (1).jpg
        let restored_alt = root.join("photo (1).jpg");
        assert!(restored_alt.exists());
        assert_eq!(fs::read(&restored_alt).unwrap(), b"old photo");
    }

    #[test]
    fn test_undo_with_missing_destination_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        let db_path = root.join("history.db");
        let mut ledger = Ledger::open_at(&db_path).unwrap();

        let src = root.join("deleted_later.txt");
        File::create(&src).unwrap();

        let dest = root.join("Docs").join("deleted_later.txt");
        let m = safe_move(&src, &dest, false).unwrap();
        ledger.record_run("tidy run", root, &[m]).unwrap();

        // File was deleted manually by user
        fs::remove_file(&dest).unwrap();

        // Undo should gracefully skip
        let report = execute_undo(&mut ledger, None).unwrap();
        assert_eq!(report.restored_count, 0);
        assert_eq!(report.skipped_count, 1);
    }

    #[test]
    fn test_multi_run_undo_sequential() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        let db_path = root.join("history.db");
        let mut ledger = Ledger::open_at(&db_path).unwrap();

        // Run 1
        let f1 = root.join("file1.jpg");
        File::create(&f1).unwrap().write_all(b"1").unwrap();
        let d1 = root.join("Images").join("file1.jpg");
        let m1 = safe_move(&f1, &d1, false).unwrap();
        ledger.record_run("tidy run", root, &[m1]).unwrap();

        // Run 2
        let f2 = root.join("file2.txt");
        File::create(&f2).unwrap().write_all(b"2").unwrap();
        let d2 = root.join("Documents").join("file2.txt");
        let m2 = safe_move(&f2, &d2, false).unwrap();
        ledger.record_run("tidy run", root, &[m2]).unwrap();

        // Undo 2 runs sequentially
        let r2 = execute_undo(&mut ledger, None).unwrap();
        assert_eq!(r2.restored_count, 1);
        assert!(f2.exists());
        assert!(!d2.exists());

        let r1 = execute_undo(&mut ledger, None).unwrap();
        assert_eq!(r1.restored_count, 1);
        assert!(f1.exists());
        assert!(!d1.exists());
    }
}
