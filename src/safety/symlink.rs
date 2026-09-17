//! Symlink safety guard to prevent infinite traversal loops and directory escapes.

use crate::common::error::{Result, TidyError};
use std::fs;
use std::path::{Path, PathBuf};

/// Categorization of a path based on its symlink nature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymlinkType {
    /// The path is a normal file or directory, not a symbolic link.
    NotSymlink,
    /// The path is a symlink pointing to a regular file.
    FileSymlink(PathBuf),
    /// The path is a symlink pointing to a directory.
    DirectorySymlink(PathBuf),
    /// The path is a symlink whose target does not exist.
    DanglingSymlink(PathBuf),
}

/// Inspects the given path using `fs::symlink_metadata` to determine if it is a symlink
/// and what kind of target it references.
pub fn inspect_symlink(path: &Path) -> Result<SymlinkType> {
    let metadata = fs::symlink_metadata(path).map_err(|e| TidyError::FileOperation {
        path: path.to_path_buf(),
        message: format!("Failed to read symlink metadata: {}", e),
    })?;

    if !metadata.file_type().is_symlink() {
        return Ok(SymlinkType::NotSymlink);
    }

    let target = fs::read_link(path).map_err(|e| TidyError::FileOperation {
        path: path.to_path_buf(),
        message: format!("Failed to read symlink target: {}", e),
    })?;

    // Determine target validity relative to the link's directory
    let resolved_target = if target.is_relative() {
        path.parent().unwrap_or_else(|| Path::new("")).join(&target)
    } else {
        target.clone()
    };

    if !resolved_target.exists() {
        return Ok(SymlinkType::DanglingSymlink(target));
    }

    if resolved_target.is_dir() {
        Ok(SymlinkType::DirectorySymlink(target))
    } else {
        Ok(SymlinkType::FileSymlink(target))
    }
}

/// Checks if a file or directory should be traversed during recursive scanning.
///
/// Under tidy's safety rules:
/// - Directory symlinks are **never** followed to prevent escaping the root or infinite loops.
/// - File symlinks can be processed as individual items.
pub fn should_traverse_dir(path: &Path) -> bool {
    match inspect_symlink(path) {
        Ok(SymlinkType::DirectorySymlink(_)) => false,
        _ => path.is_dir(),
    }
}

/// Helper to check if a path is a symbolic link pointing to a directory.
pub fn is_directory_symlink(path: &Path) -> bool {
    matches!(inspect_symlink(path), Ok(SymlinkType::DirectorySymlink(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::os::unix::fs::symlink;

    #[test]
    fn test_regular_file_is_not_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("normal.txt");
        File::create(&file_path).unwrap();

        let sym_type = inspect_symlink(&file_path).unwrap();
        assert_eq!(sym_type, SymlinkType::NotSymlink);
        assert!(!should_traverse_dir(&file_path));
    }

    #[test]
    fn test_file_symlink_detection() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target.txt");
        File::create(&target).unwrap();

        let link = temp.path().join("link.txt");
        symlink(&target, &link).unwrap();

        let sym_type = inspect_symlink(&link).unwrap();
        assert_eq!(sym_type, SymlinkType::FileSymlink(target));
        assert!(!should_traverse_dir(&link));
    }

    #[test]
    fn test_directory_symlink_is_not_traversed() {
        let temp = tempfile::tempdir().unwrap();
        let target_dir = temp.path().join("target_dir");
        fs::create_dir(&target_dir).unwrap();

        let link_dir = temp.path().join("link_dir");
        symlink(&target_dir, &link_dir).unwrap();

        let sym_type = inspect_symlink(&link_dir).unwrap();
        assert_eq!(sym_type, SymlinkType::DirectorySymlink(target_dir));
        // Crucial safety check: directory symlinks must NOT be traversed!
        assert!(!should_traverse_dir(&link_dir));
    }

    #[test]
    fn test_dangling_symlink_detection() {
        let temp = tempfile::tempdir().unwrap();
        let non_existent = temp.path().join("does_not_exist.txt");
        let link = temp.path().join("dangling.txt");
        symlink(&non_existent, &link).unwrap();

        let sym_type = inspect_symlink(&link).unwrap();
        assert_eq!(sym_type, SymlinkType::DanglingSymlink(non_existent));
    }
}
