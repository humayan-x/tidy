//! CLI execution handler for `tidy watch`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use directories::UserDirs;

use crate::cli::WatchArgs;
use crate::common::error::{Result, TidyError};
use crate::core::config::Config;
use crate::watch::dispatcher::{WatchConfig, WatchDispatcher};

/// Executes the `tidy watch` daemon command.
pub fn execute_watch(args: &WatchArgs, custom_config: Option<&Path>) -> Result<()> {
    // 1. Load configuration
    let config = Config::load_or_default(custom_config)?;

    // 2. Resolve watch directory: CLI arg -> config.settings.watch_dir -> User Downloads folder -> Current directory
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
        None => {
            if let Some(ref configured_dir) = config.settings.watch_dir {
                if configured_dir.exists() {
                    fs::canonicalize(configured_dir)?
                } else {
                    return Err(TidyError::PathResolution(format!(
                        "Configured watch directory does not exist: {}",
                        configured_dir.display()
                    )));
                }
            } else {
                let default_dir = UserDirs::new()
                    .and_then(|u| u.download_dir().map(Path::to_path_buf))
                    .filter(|p| p.exists())
                    .unwrap_or_else(|| {
                        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                    });
                fs::canonicalize(default_dir)?
            }
        }
    };

    // 3. Resolve debounce and stability parameters
    let debounce_ms = args.debounce.unwrap_or(config.settings.debounce_ms);
    let stability_tick_ms = config.settings.stability_tick_ms;

    let watch_config = WatchConfig::new(
        target_dir.clone(),
        args.recursive,
        debounce_ms,
        stability_tick_ms,
    );

    // 4. If --initial-scan requested, run single-pass organization of existing files first
    if args.initial_scan {
        println!(
            "  [tidy] Running initial single-pass scan of '{}'...",
            target_dir.display()
        );
        let run_args = crate::cli::RunArgs {
            path: Some(target_dir.clone()),
            dry_run: false,
            recursive: args.recursive,
            flat: args.flat,
            exclude: args.exclude.clone(),
        };
        crate::cli::run::execute_run(&run_args, custom_config)?;
    }

    // Apply CLI overrides to config
    let mut config = config;
    if args.flat {
        config.settings.nest_by_extension = false;
    }
    if let Some(gp) = args.grace_period {
        config.settings.grace_period_secs = gp;
    }
    config.settings.exclude.extend(args.exclude.clone());

    // 5. Register graceful termination signal handler (SIGINT / SIGTERM / SIGHUP)
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let sig_clone = Arc::clone(&shutdown_signal);

    if let Err(e) = ctrlc::set_handler(move || {
        println!("\n  [tidy] Shutdown signal received. Finishing in-flight operations...");
        sig_clone.store(true, Ordering::SeqCst);
    }) {
        tracing::debug!("Signal handler notice: {}", e);
    }

    // 6. Initialize and run dispatcher
    let mut dispatcher = WatchDispatcher::new(watch_config, &config)?;
    dispatcher.run(shutdown_signal)?;

    Ok(())
}
