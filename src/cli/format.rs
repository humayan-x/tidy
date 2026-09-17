//! Terminal styling, color diffs, and formatting helpers using `console`.

use console::style;
use std::path::Path;

use crate::safety::undo::{UndoOutcome, UndoReport};

/// Converts a byte count into a human-readable string (e.g. `124 B`, `15.2 KB`, `3.4 MB`).
pub fn human_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Information about a planned move for dry-run preview display.
#[derive(Debug, Clone)]
pub struct PlannedMovePreview {
    pub source_name: String,
    pub dest_display: String,
    pub category: String,
    pub file_size: u64,
    pub was_collision: bool,
}

/// Displays a color-coded table of proposed file moves for `--dry-run`.
pub fn print_dry_run_table(root_dir: &Path, items: &[PlannedMovePreview]) {
    println!();
    println!(
        "{}",
        style(format!(
            "  DRY RUN PREVIEW (Target: {})",
            root_dir.display()
        ))
        .bold()
        .underlined()
    );
    println!();

    for item in items {
        let cat_badge = style(format!("[{:<9}]", item.category)).cyan().bold();
        let src = style(&item.source_name).yellow();
        let arrow = style("->").dim();
        let dest = style(&item.dest_display).green();
        let size = style(format!("({})", human_bytes(item.file_size))).dim();

        if item.was_collision {
            let collision_tag = style("[COLLISION RENAMED]").magenta().bold();
            println!(
                "  {} {} {} {} {} {}",
                cat_badge, src, arrow, dest, size, collision_tag
            );
        } else {
            println!("  {} {} {} {} {}", cat_badge, src, arrow, dest, size);
        }
    }

    println!();
    let total_bytes: u64 = items.iter().map(|i| i.file_size).sum();
    println!(
        "  {}",
        style(format!(
            "Dry run complete: {} files would be moved ({}), 0 disk modifications made.",
            items.len(),
            human_bytes(total_bytes)
        ))
        .yellow()
        .bold()
    );
    println!();
}

/// Displays execution summary after a live `tidy run`.
pub fn print_run_summary(elapsed_ms: u128, files_moved: usize, total_bytes: u64, run_uuid: &str) {
    println!();
    println!(
        "  {}",
        style(format!(
            "✓ Successfully organized {} files ({}) in {}ms",
            files_moved,
            human_bytes(total_bytes),
            elapsed_ms
        ))
        .green()
        .bold()
    );
    println!(
        "    {}",
        style(format!("Run ID: {} (use 'tidy undo' to revert)", run_uuid)).dim()
    );
    println!();
}

/// Displays execution summary after a `tidy undo`.
pub fn print_undo_summary(report: &UndoReport) {
    println!();
    let title = if report.is_dry_run {
        format!("  UNDO DRY RUN PREVIEW (Run ID: {})", report.run_uuid)
    } else {
        format!("  UNDO REPORT (Run ID: {})", report.run_uuid)
    };
    println!("{}", style(title).bold().underlined());
    println!();

    for detail in &report.details {
        match detail {
            UndoOutcome::Restored { from, to } => {
                let from_str = from.file_name().unwrap_or_default().to_string_lossy();
                let to_str = to.display().to_string();
                let tag = if report.is_dry_run {
                    style("[WOULD RESTORE]").cyan().bold()
                } else {
                    style("[RESTORED]").green().bold()
                };
                println!(
                    "  {} {} {} {}",
                    tag,
                    style(from_str).yellow(),
                    style("->").dim(),
                    style(to_str).cyan()
                );
            }
            UndoOutcome::SkippedMissing { destination } => {
                println!(
                    "  {} {} (file not found on disk)",
                    style("[SKIPPED]").dim(),
                    style(destination.display()).dim()
                );
            }
            UndoOutcome::Failed { destination, error } => {
                println!(
                    "  {} {}: {}",
                    style("[FAILED]").red().bold(),
                    destination.display(),
                    error
                );
            }
        }
    }

    if !report.pruned_dirs.is_empty() {
        println!();
        for dir in &report.pruned_dirs {
            println!(
                "  {} Cleaned empty folder {}",
                style("[PRUNED]").blue().bold(),
                dir.display()
            );
        }
    }

    println!();
    if report.is_dry_run {
        println!(
            "  {}",
            style(format!(
                "Dry run complete: {} files would be restored, {} skipped, 0 disk modifications made.",
                report.restored_count,
                report.skipped_count,
            ))
            .yellow()
            .bold()
        );
    } else {
        println!(
            "  {}",
            style(format!(
                "✓ Undo complete: {} files restored, {} skipped, {} empty folders pruned.",
                report.restored_count,
                report.skipped_count,
                report.pruned_dirs.len()
            ))
            .green()
            .bold()
        );
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_bytes() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1024), "1.0 KB");
        assert_eq!(human_bytes(1536), "1.5 KB");
        assert_eq!(human_bytes(1048576), "1.0 MB");
        assert_eq!(human_bytes(1073741824), "1.0 GB");
    }
}
