//! Implementation of `tidy init` for scaffolding a default configuration file.

use console::style;
use std::fs;
use std::path::Path;

use crate::common::error::Result;
use crate::common::paths::{ensure_parent_dir, get_config_file_path};
use crate::core::config::generate_default_config_template;

/// Scaffolds a default `config.toml` file.
pub fn execute_init(custom_path: Option<&Path>, force: bool) -> Result<()> {
    let config_path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => get_config_file_path()?,
    };

    if config_path.exists() && !force {
        println!();
        println!(
            "  {} Configuration file already exists at '{}'",
            style("Notice:").yellow().bold(),
            config_path.display()
        );
        println!("  Use 'tidy init --force' to overwrite with defaults.");
        println!();
        return Ok(());
    }

    ensure_parent_dir(&config_path)?;
    let template = generate_default_config_template();
    fs::write(&config_path, template)?;

    println!();
    println!(
        "  {} Initialized configuration file at '{}'",
        style("✓").green().bold(),
        config_path.display()
    );
    println!("  You can now customize categories, ignore patterns, and destinations.");
    println!();

    Ok(())
}
