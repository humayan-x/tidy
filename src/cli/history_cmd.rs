//! Implementation of the `tidy history` command for inspecting and pruning the transaction ledger.

use console::style;
use std::fs;

use crate::cli::format::human_bytes;
use crate::cli::HistoryArgs;
use crate::common::error::Result;
use crate::common::paths::get_history_db_path;
use crate::safety::ledger::Ledger;

/// Executes the history command.
pub fn execute_history_cmd(args: &HistoryArgs) -> Result<()> {
    match &args.action {
        crate::cli::HistoryAction::Info => {
            let db_path = get_history_db_path()?;
            if !db_path.exists() {
                println!("  No history ledger found at '{}'.", db_path.display());
                return Ok(());
            }

            let ledger = Ledger::open_default()?;
            let stats = ledger.get_history_stats()?;
            let db_size = fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

            println!();
            println!("{}", style("  TIDY TRANSACTION LEDGER").bold().underlined());
            println!();
            println!("  {:<12} {}", style("Database:").cyan(), db_path.display());
            println!("  {:<12} {}", style("Size:").cyan(), human_bytes(db_size));
            println!("  {:<12} {}", style("Total Runs:").cyan(), stats.total_runs);
            println!(
                "  {:<12} {}",
                style("Completed:").green(),
                stats.completed_runs
            );
            println!(
                "  {:<12} {}",
                style("Reverted:").yellow(),
                stats.reverted_runs
            );
            println!(
                "  {:<12} {}",
                style("Operations:").cyan(),
                stats.total_operations
            );
            if let Some(oldest) = stats.oldest_run {
                println!("  {:<12} {}", style("Oldest Run:").dim(), oldest);
            }
            if let Some(newest) = stats.newest_run {
                println!("  {:<12} {}", style("Newest Run:").dim(), newest);
            }
            println!();
        }
        crate::cli::HistoryAction::Prune { days } => {
            let mut ledger = Ledger::open_default()?;
            let deleted = ledger.prune_older_than(*days)?;
            println!();
            if deleted == 0 {
                println!("  No runs older than {} days found to prune.", days);
            } else {
                println!(
                    "  {} Successfully pruned {} run(s) older than {} days and reclaimed database space.",
                    style("✓").green().bold(),
                    deleted,
                    days
                );
            }
            println!();
        }
    }
    Ok(())
}
