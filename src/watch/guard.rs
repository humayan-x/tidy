//! Ingestion and safety guard for the filesystem watcher.
//!
//! Filters browser download artifacts (.crdownload, .part, etc.), temporary/swap files,
//! active destination folders, and user-configured ignore patterns.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::core::config::CompiledRules;
use crate::core::taxonomy::is_download_in_progress;
use crate::safety::symlink::is_directory_symlink;

/// Evaluates whether an incoming filesystem path should be processed or ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardDecision {
    /// The path is a valid candidate for ingestion and stability tracking.
    Accept,
    /// The path is an in-progress download artifact (.crdownload, .part, etc.).
    DownloadInProgress,
    /// The path is a temporary or swap file (e.g. .swp, *~, .DS_Store).
    TemporaryOrSystem,
    /// The path is within an active destination folder (prevents recursive processing).
    ActiveDestination,
    /// The path matches an explicit user ignore pattern.
    UserIgnored,
    /// The path is a directory or directory symlink (not an organizing candidate).
    DirectoryOrNonRegular,
    /// The path no longer exists on disk (e.g. temporary file deleted or renamed).
    DoesNotExist,
}

impl GuardDecision {
    pub fn is_accepted(&self) -> bool {
        matches!(self, GuardDecision::Accept)
    }
}

/// Guard instance holding compiled rules and active destination directories.
pub struct WatchGuard<'a> {
    rules: &'a CompiledRules,
    active_destinations: &'a HashSet<String>,
    root_dir: &'a Path,
}

impl<'a> WatchGuard<'a> {
    pub fn new(
        rules: &'a CompiledRules,
        active_destinations: &'a HashSet<String>,
        root_dir: &'a Path,
    ) -> Self {
        Self {
            rules,
            active_destinations,
            root_dir,
        }
    }

    /// Inspects a candidate path and determines if it should be ingested.
    pub fn evaluate(&self, path: &Path) -> GuardDecision {
        // 1. Check if path still exists on disk
        let metadata = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_) => return GuardDecision::DoesNotExist,
        };

        // 2. Ignore directories and directory symlinks
        if metadata.is_dir() || is_directory_symlink(path) {
            return GuardDecision::DirectoryOrNonRegular;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => return GuardDecision::DirectoryOrNonRegular,
        };

        // 3. Check if path resides inside an active category destination directory
        if self.is_in_active_destination(path) {
            return GuardDecision::ActiveDestination;
        }

        // 4. Check for in-progress browser download extensions
        if is_download_in_progress(file_name) {
            return GuardDecision::DownloadInProgress;
        }

        // 5. Check for editor temporary and swap files
        if is_editor_temp_file(file_name) {
            return GuardDecision::TemporaryOrSystem;
        }

        // 6. Check user-configured ignore patterns and hidden files
        if self.rules.is_ignored(file_name) {
            return GuardDecision::UserIgnored;
        }

        GuardDecision::Accept
    }

    /// Checks if a file path is located directly within one of the target category subfolders.
    fn is_in_active_destination(&self, path: &Path) -> bool {
        if let Ok(relative) = path.strip_prefix(self.root_dir) {
            if relative.components().count() > 1 {
                if let Some(first_component) = relative.components().next() {
                    let comp_str = first_component.as_os_str().to_string_lossy();
                    if self.active_destinations.contains(comp_str.as_ref()) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Checks if a filename matches standard editor temporary/swap file patterns.
pub fn is_editor_temp_file(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower.ends_with(".swp")
        || lower.ends_with(".swo")
        || lower.ends_with('~')
        || lower.ends_with(".bak")
        || lower.starts_with(".#")
        || lower.starts_with("#")
        || file_name == ".DS_Store"
        || file_name == "Thumbs.db"
        || file_name == "desktop.ini"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use std::fs::File;
    use std::io::Write;

    fn setup_test_guard<'a>(
        rules: &'a CompiledRules,
        active_dest: &'a HashSet<String>,
        root: &'a Path,
    ) -> WatchGuard<'a> {
        WatchGuard::new(rules, active_dest, root)
    }

    #[test]
    fn test_guard_filters_downloads() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let crdownload = root.join("movie.mp4.crdownload");
        File::create(&crdownload)
            .unwrap()
            .write_all(b"partial")
            .unwrap();

        let config = Config::default();
        let rules = config.compile().unwrap();
        let mut active = HashSet::new();
        active.insert("Images".to_string());
        active.insert("Video".to_string());

        let guard = setup_test_guard(&rules, &active, root);
        assert_eq!(
            guard.evaluate(&crdownload),
            GuardDecision::DownloadInProgress
        );
    }

    #[test]
    fn test_guard_filters_active_destinations() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let img_dir = root.join("Images");
        fs::create_dir(&img_dir).unwrap();
        let img_file = img_dir.join("photo.jpg");
        File::create(&img_file).unwrap().write_all(b"data").unwrap();

        let config = Config::default();
        let rules = config.compile().unwrap();
        let mut active = HashSet::new();
        active.insert("Images".to_string());

        let guard = setup_test_guard(&rules, &active, root);
        assert_eq!(guard.evaluate(&img_file), GuardDecision::ActiveDestination);
    }

    #[test]
    fn test_guard_filters_editor_temp_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let swp_file = root.join(".notes.txt.swp");
        File::create(&swp_file).unwrap().write_all(b"vim").unwrap();

        let config = Config::default();
        let rules = config.compile().unwrap();
        let active = HashSet::new();

        let guard = setup_test_guard(&rules, &active, root);
        assert_eq!(guard.evaluate(&swp_file), GuardDecision::TemporaryOrSystem);
    }

    #[test]
    fn test_guard_accepts_valid_file() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let valid_file = root.join("invoice.pdf");
        File::create(&valid_file)
            .unwrap()
            .write_all(b"%PDF-1.4")
            .unwrap();

        let config = Config::default();
        let rules = config.compile().unwrap();
        let active = HashSet::new();

        let guard = setup_test_guard(&rules, &active, root);
        assert_eq!(guard.evaluate(&valid_file), GuardDecision::Accept);
    }
}
