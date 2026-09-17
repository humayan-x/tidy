//! Implementation of the `tidy status` command.

use console::style;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::format::human_bytes;
use crate::common::error::Result;
use crate::common::paths::{get_config_file_path, get_history_db_path};
use crate::core::config::Config;
use crate::safety::ledger::Ledger;

/// Executes the `tidy status` command, reporting configuration, database, and recent run metrics.
pub fn execute_status_cmd(custom_config: Option<&Path>) -> Result<()> {
    println!();
    println!("{}", style("  TIDY SYSTEM STATUS").bold().underlined());
    println!();

    // 1. Configuration status
    let config_path = match custom_config {
        Some(p) => p.to_path_buf(),
        None => get_config_file_path()?,
    };

    let config_exists = config_path.exists();
    let config = Config::load_or_default(custom_config)?;
    let rules = config.compile()?;

    println!("  {}", style("Configuration:").cyan().bold());
    println!("    Path:    {}", config_path.display());
    if config_exists {
        println!(
            "    Status:  {}",
            style("Active (loaded from disk)").green()
        );
    } else {
        println!(
            "    Status:  {}",
            style("Active (using zero-config defaults)").yellow()
        );
    }
    println!(
        "    Rules:   {} active categories, {} custom destinations",
        config.categories.len(),
        config.destinations.len()
    );

    let cat_names: Vec<&str> = rules
        .extension_to_category
        .values()
        .map(String::as_str)
        .collect();
    let mut unique_cats = cat_names;
    unique_cats.sort();
    unique_cats.dedup();
    println!("    Categories: {}", unique_cats.join(", "));
    println!();

    // 2. Database & State Ledger
    let db_path = get_history_db_path()?;
    let db_exists = db_path.exists();
    let db_size = if db_exists {
        let mut total = fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
        let wal_path = PathBuf::from(format!("{}-wal", db_path.display()));
        if let Ok(m) = fs::metadata(&wal_path) {
            total += m.len();
        }
        let shm_path = PathBuf::from(format!("{}-shm", db_path.display()));
        if let Ok(m) = fs::metadata(&shm_path) {
            total += m.len();
        }
        total
    } else {
        0
    };

    println!("  {}", style("State & Ledger:").cyan().bold());
    println!("    Database: {}", db_path.display());
    if db_exists {
        println!("    Size:     {}", human_bytes(db_size));
    } else {
        println!("    Status:   {}", style("Not initialized yet").dim());
    }

    // 3. Recent Activity from Ledger
    if db_exists {
        if let Ok(ledger) = Ledger::open_default() {
            let recent_runs = ledger.list_recent_runs(1)?;
            if let Some(latest) = recent_runs.first() {
                let ops = ledger.get_operations_for_run(latest.id).unwrap_or_default();
                println!();
                println!("  {}", style("Latest Activity:").cyan().bold());
                println!("    Run ID:    {}", latest.uuid);
                println!("    Timestamp: {}", latest.timestamp);
                println!("    Command:   {}", latest.command);
                println!("    Root:      {}", latest.root_path);
                let status_styled = if latest.status == "COMPLETED" {
                    style(&latest.status).green().bold()
                } else if latest.status == "REVERTED" {
                    style(&latest.status).yellow().bold()
                } else {
                    style(&latest.status).red().bold()
                };
                println!("    Status:    {}", status_styled);
                println!("    Files:     {} file operations recorded", ops.len());
            } else {
                println!();
                println!("  {}", style("Latest Activity:").cyan().bold());
                println!("    {}", style("No recorded runs yet").dim());
            }
        }
    }

    // 4. Background Service / Daemon Status
    println!();
    println!("  {}", style("Background Service:").cyan().bold());
    let service_desc = crate::service::check_service_status();
    println!("    Status:    {}", service_desc);
    #[cfg(target_os = "linux")]
    {
        if let Ok(unit_path) = crate::service::linux::get_systemd_unit_path() {
            println!("    Unit Path: {}", unit_path.display());
        }
    }

    println!();
    Ok(())
}
