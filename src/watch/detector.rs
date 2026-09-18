//! Write-completion detector and file stability tracker.
//!
//! Debounces rapid filesystem bursts and verifies that candidate files have settled
//! (unchanging file size and modified timestamp across consecutive poll intervals)
//! and that external write handles have closed before initiating file organization.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// Tracks the stability progress of an individual file undergoing writes or downloads.
#[derive(Debug, Clone)]
pub struct PendingFile {
    #[allow(dead_code)]
    pub path: PathBuf,
    pub last_size: u64,
    pub last_mtime: Option<SystemTime>,
    pub consecutive_stable_ticks: u32,
    pub first_seen: Instant,
}

/// Manages a collection of candidate files and emits them once write activity has settled.
pub struct PendingFileTracker {
    pending: HashMap<PathBuf, PendingFile>,
    required_stable_ticks: u32,
    max_tracking_duration: Duration,
    grace_period: Duration,
}

impl PendingFileTracker {
    /// Creates a new tracker with specified stability criteria and zero grace period.
    pub fn new(required_stable_ticks: u32, max_tracking_duration: Duration) -> Self {
        Self::new_with_grace_period(required_stable_ticks, max_tracking_duration, Duration::from_secs(0))
    }

    /// Creates a new tracker with specified stability criteria and grace period.
    pub fn new_with_grace_period(
        required_stable_ticks: u32,
        max_tracking_duration: Duration,
        grace_period: Duration,
    ) -> Self {
        Self {
            pending: HashMap::new(),
            required_stable_ticks,
            max_tracking_duration,
            grace_period,
        }
    }

    /// Default configuration: 2 stable checks and 10 minutes maximum tracking window.
    #[allow(dead_code)]
    pub fn default_config() -> Self {
        Self::new_with_grace_period(2, Duration::from_secs(600), Duration::from_secs(3))
    }

    /// Registers a newly discovered or modified file path.
    pub fn register(&mut self, path: PathBuf) {
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            let size = metadata.len();
            let mtime = metadata.modified().ok();

            self.pending
                .entry(path.clone())
                .and_modify(|entry| {
                    if entry.last_size != size || entry.last_mtime != mtime {
                        entry.last_size = size;
                        entry.last_mtime = mtime;
                        entry.consecutive_stable_ticks = 0;
                    }
                })
                .or_insert_with(|| PendingFile {
                    path,
                    last_size: size,
                    last_mtime: mtime,
                    consecutive_stable_ticks: 0,
                    first_seen: Instant::now(),
                });
        }
    }

    /// Advances the stability tracker by one tick, returning paths that have fully settled.
    pub fn tick(&mut self) -> Vec<PathBuf> {
        let mut settled = Vec::new();
        let mut to_remove = Vec::new();

        for (path, entry) in self.pending.iter_mut() {
            // If the file was deleted/moved in the meantime, discard it
            let metadata = match fs::symlink_metadata(path) {
                Ok(m) => m,
                Err(_) => {
                    to_remove.push(path.clone());
                    continue;
                }
            };

            // If tracking has exceeded the maximum duration, discard to prevent unbounded memory growth
            if entry.first_seen.elapsed() > self.max_tracking_duration {
                tracing::warn!(
                    "File '{}' did not settle within {:?}, dropping from tracker",
                    path.display(),
                    self.max_tracking_duration
                );
                to_remove.push(path.clone());
                continue;
            }

            let current_size = metadata.len();
            let current_mtime = metadata.modified().ok();

            // Check if file is still growing or being written
            if current_size != entry.last_size || current_mtime != entry.last_mtime {
                entry.last_size = current_size;
                entry.last_mtime = current_mtime;
                entry.consecutive_stable_ticks = 0;
                continue;
            }

            // Zero-byte guard: if a file has 0 bytes, wait at least 1.5 seconds before considering
            // it stable, to allow applications or downloads time to write their initial chunks.
            if current_size == 0 && entry.first_seen.elapsed() < Duration::from_millis(1500) {
                continue;
            }

            // Grace period guard: if file was touched recently, wait until grace period has elapsed
            if let Some(mtime) = current_mtime {
                if let Ok(elapsed) = mtime.elapsed() {
                    if elapsed < self.grace_period {
                        continue;
                    }
                }
            }

            // Size and mtime are identical to last tick. Verify file access readiness.
            if is_file_ready_for_access(path) {
                entry.consecutive_stable_ticks += 1;

                if entry.consecutive_stable_ticks >= self.required_stable_ticks {
                    settled.push(path.clone());
                    to_remove.push(path.clone());
                }
            } else {
                // File is busy or locked by another write process
                entry.consecutive_stable_ticks = 0;
            }
        }

        for path in to_remove {
            self.pending.remove(&path);
        }

        settled
    }

    /// Returns the number of currently tracked pending files.
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Checks if there are any pending files.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Manually removes a path from the tracker.
    #[allow(dead_code)]
    pub fn remove(&mut self, path: &Path) {
        self.pending.remove(path);
    }
}

/// Verifies that a file can be accessed and is not exclusively locked by an active writer.
pub fn is_file_ready_for_access(path: &Path) -> bool {
    #[cfg(unix)]
    use std::os::unix::io::AsRawFd;

    let file = match fs::OpenOptions::new().read(true).open(path) {
        Ok(f) => f,
        Err(err) => {
            tracing::trace!(
                "File '{}' not ready for read access: {}",
                path.display(),
                err
            );
            return false;
        }
    };

    #[cfg(unix)]
    {
        let fd = file.as_raw_fd();
        // Check for active exclusive locks via non-blocking shared flock
        let lock_res = unsafe { libc::flock(fd, libc::LOCK_SH | libc::LOCK_NB) };
        if lock_res != 0 {
            let err = std::io::Error::last_os_error();
            tracing::trace!(
                "File '{}' is exclusively locked by another process: {}",
                path.display(),
                err
            );
            return false;
        }
        let _ = unsafe { libc::flock(fd, libc::LOCK_UN) };
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_tracker_settles_stable_file() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("stable.txt");
        File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let mut tracker = PendingFileTracker::new(2, Duration::from_secs(60));
        tracker.register(file_path.clone());

        // First tick: 1 stable tick -> not settled yet
        let settled = tracker.tick();
        assert!(settled.is_empty());
        assert_eq!(tracker.len(), 1);

        // Second tick: 2 stable ticks -> settled!
        let settled = tracker.tick();
        assert_eq!(settled.len(), 1);
        assert_eq!(settled[0], file_path);
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_tracker_resets_on_file_growth() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("growing.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"chunk1").unwrap();
        file.flush().unwrap();

        let mut tracker = PendingFileTracker::new(2, Duration::from_secs(60));
        tracker.register(file_path.clone());

        // First tick
        assert!(tracker.tick().is_empty());

        // Write additional data to simulate download in progress
        file.write_all(b"chunk2").unwrap();
        file.flush().unwrap();

        // Second tick: file size changed -> resets stable tick count
        assert!(tracker.tick().is_empty());
        assert_eq!(tracker.len(), 1);

        // Third tick: size unchanged (1 stable tick)
        assert!(tracker.tick().is_empty());

        // Fourth tick: size unchanged (2 stable ticks) -> now settled!
        let settled = tracker.tick();
        assert_eq!(settled.len(), 1);
        assert_eq!(settled[0], file_path);
    }

    #[test]
    fn test_tracker_discards_vanished_file() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("deleted.txt");
        File::create(&file_path)
            .unwrap()
            .write_all(b"temp")
            .unwrap();

        let mut tracker = PendingFileTracker::new(2, Duration::from_secs(60));
        tracker.register(file_path.clone());
        assert_eq!(tracker.len(), 1);

        // Simulate file being deleted
        fs::remove_file(&file_path).unwrap();

        // Tick detects disappearance and removes entry cleanly
        let settled = tracker.tick();
        assert!(settled.is_empty());
        assert!(tracker.is_empty());
    }
}
