//! Event processing and safe move dispatch loop.
//!
//! Coordinates the debounced filesystem events, download/temporary guard,
//! write-completion detector, classifier, safe mover, and SQLite ledger.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::Arc;
use std::time::Duration;

use crate::common::error::Result;
use crate::core::classifier::{ClassificationResult, Classifier};
use crate::core::config::Config;
use crate::safety::ledger::Ledger;
use crate::safety::mover::{safe_move, MoveOutcome};
use crate::watch::detector::PendingFileTracker;
use crate::watch::engine::WatcherEngine;
use crate::watch::guard::{GuardDecision, WatchGuard};

/// Configuration options for the watcher daemon.
#[derive(Debug, Clone)]
pub struct WatchConfig {
    pub target_dir: PathBuf,
    pub recursive: bool,
    pub debounce_duration: Duration,
    pub stability_tick: Duration,
    pub required_stable_ticks: u32,
    pub custom_db_path: Option<PathBuf>,
}

impl WatchConfig {
    pub fn new(
        target_dir: PathBuf,
        recursive: bool,
        debounce_ms: u64,
        stability_tick_ms: u64,
    ) -> Self {
        Self {
            target_dir,
            recursive,
            debounce_duration: Duration::from_millis(debounce_ms),
            stability_tick: Duration::from_millis(stability_tick_ms),
            required_stable_ticks: 2,
            custom_db_path: None,
        }
    }

    pub fn with_custom_db(mut self, db_path: PathBuf) -> Self {
        self.custom_db_path = Some(db_path);
        self
    }
}

/// The main dispatcher coordinating file ingestion, classification, and safe movement.
pub struct WatchDispatcher {
    config: WatchConfig,
    classifier: Classifier,
    active_destinations: HashSet<String>,
    tracker: PendingFileTracker,
    ledger: Ledger,
}

impl WatchDispatcher {
    /// Creates a new dispatcher instance with default ledger resolution.
    pub fn new(config: WatchConfig, tidy_config: &Config) -> Result<Self> {
        let ledger = match &config.custom_db_path {
            Some(path) => Ledger::open_at(path)?,
            None => Ledger::open_default()?,
        };
        Self::with_ledger(config, tidy_config, ledger)
    }

    /// Creates a new dispatcher instance with an explicit ledger.
    pub fn with_ledger(config: WatchConfig, tidy_config: &Config, ledger: Ledger) -> Result<Self> {
        let rules = tidy_config.compile()?;
        let grace_duration = Duration::from_secs(rules.grace_period_secs);
        let classifier = Classifier::new(rules);
        let active_destinations = classifier.active_destinations().clone();
        let tracker = PendingFileTracker::new_with_grace_period(
            config.required_stable_ticks,
            Duration::from_secs(600),
            grace_duration,
        );

        Ok(Self {
            config,
            classifier,
            active_destinations,
            tracker,
            ledger,
        })
    }

    /// Processes a list of settled file paths, executing safe moves and committing to the ledger.
    pub fn process_settled_files(&mut self, settled_paths: &[PathBuf]) -> Result<Vec<MoveOutcome>> {
        if settled_paths.is_empty() {
            return Ok(Vec::new());
        }

        let guard = WatchGuard::new(
            self.classifier.rules(),
            &self.active_destinations,
            &self.config.target_dir,
        );

        let mut outcomes = Vec::new();

        for path in settled_paths {
            // Re-evaluate guard in case file was modified, moved, or deleted
            if !guard.evaluate(path).is_accepted() {
                continue;
            }

            let classification = self.classifier.classify(path, &self.config.target_dir);

            if let ClassificationResult::Move {
                category,
                target_path,
                ..
            } = classification
            {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                match safe_move(path, &target_path, false) {
                    Ok(outcome) => {
                        let dest_rel = outcome
                            .destination
                            .strip_prefix(&self.config.target_dir)
                            .map(|r| r.to_string_lossy().to_string())
                            .unwrap_or_else(|_| outcome.destination.to_string_lossy().to_string());

                        println!(
                            "  [tidy] Organized: {} -> {} ({})",
                            file_name, dest_rel, category
                        );
                        tracing::info!(
                            "Organized '{}' -> '{}' [category: {}, collision: {}]",
                            path.display(),
                            outcome.destination.display(),
                            category,
                            outcome.was_collision
                        );
                        outcomes.push(outcome);
                    }
                    Err(err) => {
                        tracing::error!(
                            "Failed to move '{}' to '{}': {}",
                            path.display(),
                            target_path.display(),
                            err
                        );
                        eprintln!("  [tidy] Error organizing '{}': {}", file_name, err);
                    }
                }
            }
        }

        // Commit batch to SQLite ledger
        if !outcomes.is_empty() {
            if let Err(e) = self
                .ledger
                .record_run("tidy watch", &self.config.target_dir, &outcomes)
            {
                tracing::error!("Failed to record watch batch to ledger: {}", e);
            }
        }

        Ok(outcomes)
    }

    /// Starts the main event listening and dispatch loop.
    pub fn run(&mut self, shutdown_signal: Arc<AtomicBool>) -> Result<()> {
        let engine = WatcherEngine::new(
            &self.config.target_dir,
            self.config.recursive,
            self.config.debounce_duration,
        )?;

        let rx = engine.receiver();

        println!(
            "  [tidy] Watching '{}' (recursive: {}, debounce: {:?})...",
            self.config.target_dir.display(),
            self.config.recursive,
            self.config.debounce_duration
        );
        tracing::info!(
            "Started watcher on '{}' (recursive: {})",
            self.config.target_dir.display(),
            self.config.recursive
        );

        while !shutdown_signal.load(Ordering::SeqCst) {
            // Receive debounced events or timeout to advance the stability tracker
            match rx.recv_timeout(self.config.stability_tick) {
                Ok(event_res) => match event_res {
                    Ok(events) => {
                        let guard = WatchGuard::new(
                            self.classifier.rules(),
                            &self.active_destinations,
                            &self.config.target_dir,
                        );
                        for event in events {
                            let path = event.path;
                            let decision = guard.evaluate(&path);
                            if decision == GuardDecision::Accept {
                                tracing::debug!(
                                    "Registering candidate for stability check: {}",
                                    path.display()
                                );
                                self.tracker.register(path);
                            } else {
                                tracing::trace!(
                                    "Ignored event for '{}' [reason: {:?}]",
                                    path.display(),
                                    decision
                                );
                            }
                        }
                    }
                    Err(err) => {
                        tracing::warn!("Watcher event error: {}", err);
                    }
                },
                Err(RecvTimeoutError::Timeout) => {
                    // Normal tick interval: proceed to check tracker
                }
                Err(RecvTimeoutError::Disconnected) => {
                    tracing::warn!("Watcher event channel disconnected; exiting loop");
                    break;
                }
            }

            // Advance stability tracker and process settled files
            let settled = self.tracker.tick();
            if !settled.is_empty() {
                let _ = self.process_settled_files(&settled);
            }
        }

        // Graceful shutdown: process any remaining settled files
        let final_settled = self.tracker.tick();
        if !final_settled.is_empty() {
            let _ = self.process_settled_files(&final_settled);
        }

        println!("  [tidy] Watcher stopped cleanly.");
        tracing::info!("Watcher stopped cleanly.");

        Ok(())
    }
}
