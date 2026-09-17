//! Filesystem watcher engine abstracting OS kernel events.
//!
//! Uses notify-debouncer-mini to bind inotify (Linux) and FSEvents (macOS)
//! with configurable debounce coalescing.

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use crate::common::error::{Result, TidyError};

/// Wrapper around `notify-debouncer-mini` providing debounced filesystem events.
pub struct WatcherEngine {
    _debouncer: Debouncer<RecommendedWatcher>,
    rx: Receiver<DebounceEventResult>,
}

impl WatcherEngine {
    /// Initializes a debounced watcher on `watch_path`.
    ///
    /// * `watch_path`: Target directory to monitor.
    /// * `recursive`: Whether to watch subdirectories recursively.
    /// * `debounce_duration`: Time window to coalesce rapid filesystem events.
    pub fn new(watch_path: &Path, recursive: bool, debounce_duration: Duration) -> Result<Self> {
        let (tx, rx) = channel();

        let mut debouncer = new_debouncer(debounce_duration, tx)
            .map_err(|e| TidyError::Watcher(format!("Failed to initialize debouncer: {}", e)))?;

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        debouncer
            .watcher()
            .watch(watch_path, mode)
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("No space left on device") || err_str.contains("os error 28") {
                    TidyError::Watcher(format!(
                        "Inotify watch limit reached (ENOSPC) while watching '{}'. \
                        Increase system inotify limit: sudo sysctl fs.inotify.max_user_watches=524288",
                        watch_path.display()
                    ))
                } else {
                    TidyError::Watcher(format!(
                        "Failed to watch directory '{}': {}",
                        watch_path.display(),
                        e
                    ))
                }
            })?;

        Ok(Self {
            _debouncer: debouncer,
            rx,
        })
    }

    /// Returns a reference to the event receiver channel.
    pub fn receiver(&self) -> &Receiver<DebounceEventResult> {
        &self.rx
    }
}
