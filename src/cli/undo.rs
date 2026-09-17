//! Implementation of the `tidy undo` command for rolling back operations from the ledger.

use crate::cli::format::print_undo_summary;
use crate::cli::UndoArgs;
use crate::common::error::{Result, TidyError};
use crate::safety::ledger::Ledger;
use crate::safety::undo::execute_undo_opt;

/// Executes the undo rollback command from the CLI.
pub fn execute_undo_cmd(args: &UndoArgs) -> Result<()> {
    let mut ledger = Ledger::open_default()?;

    if let Some(run_id) = &args.run_id {
        let report = execute_undo_opt(&mut ledger, Some(run_id), args.dry_run)?;
        print_undo_summary(&report);
        return Ok(());
    }

    if args.all {
        let mut count = 0;
        loop {
            match execute_undo_opt(&mut ledger, None, args.dry_run) {
                Ok(report) => {
                    count += 1;
                    print_undo_summary(&report);
                    if args.dry_run {
                        println!("  [dry-run] Tested rollback of most recent run.");
                        break;
                    }
                }
                Err(TidyError::Aborted(_)) => {
                    if count == 0 {
                        println!("  No completed runs available to undo in history ledger.");
                    } else {
                        println!("  All available runs successfully rolled back.");
                    }
                    break;
                }
                Err(e) => return Err(e),
            }
        }
        return Ok(());
    }

    let runs_to_undo = args.last.unwrap_or(1);
    for i in 0..runs_to_undo {
        match execute_undo_opt(&mut ledger, None, args.dry_run) {
            Ok(report) => {
                print_undo_summary(&report);
                if args.dry_run && runs_to_undo > 1 {
                    println!("  [dry-run] Tested rollback of most recent run.");
                    break;
                }
            }
            Err(TidyError::Aborted(_)) if i > 0 => {
                println!("  No more completed runs left to undo.");
                break;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}
