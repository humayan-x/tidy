//! Atomic file mover with non-destructive collision renaming and staged cross-device fallback.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::common::error::{Result, TidyError};
use crate::common::paths::ensure_parent_dir;
use crate::safety::collision::resolve_collision_with_claimed;
use std::collections::HashSet;

/// The detailed record of an executed or simulated file move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveOutcome {
    /// Original file path.
    pub source: PathBuf,
    /// Final destination path (including collision numbering if applicable).
    pub destination: PathBuf,
    /// File size in bytes.
    pub file_size: u64,
    /// Whether a collision occurred that required renaming.
    pub was_collision: bool,
    /// Whether the file was moved across different filesystems/mounts via copy-fallback.
    pub is_cross_device: bool,
    /// Whether the moved item was a symbolic link.
    pub is_symlink: bool,
}

/// Executes a safe, non-destructive file move from `source` to `proposed_dest`.
pub fn safe_move(source: &Path, proposed_dest: &Path, dry_run: bool) -> Result<MoveOutcome> {
    safe_move_with_claimed(source, proposed_dest, dry_run, None)
}

/// Executes a safe, non-destructive file move, optionally checking and updating an in-memory set
/// of already claimed destination paths (useful for batch collision projection in `--dry-run`).
pub fn safe_move_with_claimed(
    source: &Path,
    proposed_dest: &Path,
    dry_run: bool,
    claimed_paths: Option<&mut HashSet<PathBuf>>,
) -> Result<MoveOutcome> {
    let sym_meta = fs::symlink_metadata(source).map_err(|e| TidyError::FileOperation {
        path: source.to_path_buf(),
        message: format!("Failed to read source file metadata: {}", e),
    })?;

    let is_symlink = sym_meta.file_type().is_symlink();
    let file_size = if is_symlink { 0 } else { sym_meta.len() };

    // 1. Resolve collision non-destructively
    let collision_res = resolve_collision_with_claimed(proposed_dest, claimed_paths);
    let destination = collision_res.resolved_path;
    let was_collision = collision_res.had_collision;

    // 2. If dry-run, exit early before touching disk
    if dry_run {
        return Ok(MoveOutcome {
            source: source.to_path_buf(),
            destination,
            file_size,
            was_collision,
            is_cross_device: false,
            is_symlink,
        });
    }

    // 3. Ensure destination parent directory exists
    ensure_parent_dir(&destination)?;

    // 4. Attempt atomic move
    let mut is_cross_device = false;
    match fs::rename(source, &destination) {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::CrossesDevices || err.raw_os_error() == Some(18) => {
            // Cross-device link error (EXDEV): fall back to staged copy-then-delete
            is_cross_device = true;
            cross_device_move(source, &destination, is_symlink, file_size)?;
        }
        Err(err) => {
            return Err(TidyError::FileOperation {
                path: source.to_path_buf(),
                message: format!(
                    "Failed to move file to '{}': {}",
                    destination.display(),
                    err
                ),
            });
        }
    }

    Ok(MoveOutcome {
        source: source.to_path_buf(),
        destination,
        file_size,
        was_collision,
        is_cross_device,
        is_symlink,
    })
}

/// Helper performing staged copy-then-remove when moving across filesystem boundaries.
fn cross_device_move(
    source: &Path,
    destination: &Path,
    is_symlink: bool,
    expected_size: u64,
) -> Result<()> {
    #[cfg(unix)]
    if is_symlink {
        let link_target = fs::read_link(source).map_err(|e| TidyError::FileOperation {
            path: source.to_path_buf(),
            message: format!(
                "Failed to read symlink target during cross-device move: {}",
                e
            ),
        })?;

        std::os::unix::fs::symlink(&link_target, destination).map_err(|e| {
            TidyError::FileOperation {
                path: destination.to_path_buf(),
                message: format!("Failed to recreate symlink during cross-device move: {}", e),
            }
        })?;

        let _ = fs::remove_file(source);
        return Ok(());
    }

    // Staged copy strategy for regular files:
    // Write to hidden temp file in the destination folder first to prevent partial files on crash
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let temp_name = format!(
        ".tidy_tmp_{}_{}",
        uuid::Uuid::new_v4(),
        destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    let temp_dest = parent.join(temp_name);

    let copy_result = fs::copy(source, &temp_dest);
    if let Err(e) = copy_result {
        // Clean up temporary staged file if copy failed
        let _ = fs::remove_file(&temp_dest);
        return Err(TidyError::FileOperation {
            path: source.to_path_buf(),
            message: format!("Cross-device copy failed: {}", e),
        });
    }

    // Verify copied file size matches source
    let copied_size = fs::metadata(&temp_dest).map(|m| m.len()).unwrap_or(0);
    if copied_size != expected_size {
        let _ = fs::remove_file(&temp_dest);
        return Err(TidyError::FileOperation {
            path: source.to_path_buf(),
            message: format!(
                "Cross-device copy size mismatch: expected {} bytes, wrote {} bytes",
                expected_size, copied_size
            ),
        });
    }

    // Preserve modification and access timestamps from source file first (while temp_dest is still writable)
    if let Ok(src_meta) = fs::metadata(source) {
        let mut times = std::fs::FileTimes::new();
        let mut has_time = false;
        if let Ok(mtime) = src_meta.modified() {
            times = times.set_modified(mtime);
            has_time = true;
        }
        if let Ok(atime) = src_meta.accessed() {
            times = times.set_accessed(atime);
            has_time = true;
        }
        if has_time {
            if let Ok(f) = std::fs::OpenOptions::new().write(true).open(&temp_dest) {
                let _ = f.set_times(times);
            }
        }
    }

    // Preserve permissions from source file (done after timestamps so read-only permissions don't block writing timestamps)
    if let Ok(perms) = fs::metadata(source).map(|m| m.permissions()) {
        let _ = fs::set_permissions(&temp_dest, perms);
    }

    // Atomically rename staged temp file to final destination (same filesystem -> atomic)
    if let Err(e) = fs::rename(&temp_dest, destination) {
        let _ = fs::remove_file(&temp_dest);
        return Err(TidyError::FileOperation {
            path: temp_dest,
            message: format!("Failed to finalize cross-device move: {}", e),
        });
    }

    // Only upon confirmed destination placement, remove original source file
    if let Err(e) = fs::remove_file(source) {
        return Err(TidyError::FileOperation {
            path: source.to_path_buf(),
            message: format!(
                "File copied to '{}', but failed to delete original source: {}",
                destination.display(),
                e
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_safe_move_standard() {
        let temp_dir = tempfile::tempdir().unwrap();
        let src = temp_dir.path().join("source.txt");
        let mut file = File::create(&src).unwrap();
        file.write_all(b"hello world").unwrap();

        let dest = temp_dir.path().join("sub").join("dest.txt");

        let outcome = safe_move(&src, &dest, false).unwrap();
        assert_eq!(outcome.destination, dest);
        assert_eq!(outcome.file_size, 11);
        assert!(!outcome.was_collision);
        assert!(!src.exists());
        assert!(dest.exists());
    }

    #[test]
    fn test_safe_move_with_collision() {
        let temp_dir = tempfile::tempdir().unwrap();
        let src = temp_dir.path().join("photo.jpg");
        let mut f1 = File::create(&src).unwrap();
        f1.write_all(b"new photo").unwrap();

        let dest_dir = temp_dir.path().join("Images");
        fs::create_dir(&dest_dir).unwrap();
        let existing = dest_dir.join("photo.jpg");
        let mut f2 = File::create(&existing).unwrap();
        f2.write_all(b"existing photo").unwrap();

        let outcome = safe_move(&src, &existing, false).unwrap();
        assert!(outcome.was_collision);
        assert_eq!(outcome.destination, dest_dir.join("photo (1).jpg"));
        assert!(existing.exists());
        assert_eq!(fs::read(&existing).unwrap(), b"existing photo");
        assert!(dest_dir.join("photo (1).jpg").exists());
        assert_eq!(
            fs::read(dest_dir.join("photo (1).jpg")).unwrap(),
            b"new photo"
        );
    }

    #[test]
    fn test_safe_move_dry_run_does_not_modify_disk() {
        let temp_dir = tempfile::tempdir().unwrap();
        let src = temp_dir.path().join("document.pdf");
        File::create(&src).unwrap();
        let dest = temp_dir.path().join("Docs").join("document.pdf");

        let outcome = safe_move(&src, &dest, true).unwrap();
        assert_eq!(outcome.destination, dest);
        assert!(src.exists());
        assert!(!dest.exists());
        assert!(!dest.parent().unwrap().exists());
    }

    #[test]
    fn test_cross_device_fallback_logic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let src = temp_dir.path().join("cross_src.txt");
        let mut file = File::create(&src).unwrap();
        file.write_all(b"cross device data").unwrap();

        let dest = temp_dir.path().join("target_dir").join("cross_dest.txt");
        fs::create_dir(dest.parent().unwrap()).unwrap();

        // Directly invoke cross_device_move to verify staged copy & delete
        cross_device_move(&src, &dest, false, 17).unwrap();

        assert!(!src.exists());
        assert!(dest.exists());
        assert_eq!(fs::read(&dest).unwrap(), b"cross device data");
    }
}
