//! Predictable, non-destructive filename collision resolution.

use crate::core::normalizer::{format_collision_name, parse_file_name, ExtInfo};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Result of collision detection and path resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionResolution {
    /// The final safe path on disk that does not conflict with existing files.
    pub resolved_path: PathBuf,
    /// Whether an existing file collided with the originally proposed path.
    pub had_collision: bool,
    /// If a collision occurred, the increment counter assigned (1, 2, 3...).
    pub counter: usize,
}

/// Checks whether `proposed_path` exists on disk. If so, computes an incremental non-destructive
/// alternative (`file (1).ext`, `archive (1).tar.gz`, etc.) until a free path is found.
pub fn resolve_collision(proposed_path: &Path) -> CollisionResolution {
    resolve_collision_with_claimed(proposed_path, None)
}

/// Checks whether `proposed_path` exists on disk or has already been claimed in this batch.
pub fn resolve_collision_with_claimed(
    proposed_path: &Path,
    mut claimed_paths: Option<&mut HashSet<PathBuf>>,
) -> CollisionResolution {
    let parent = proposed_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let is_occupied = |path: &Path, claimed: &Option<&mut HashSet<PathBuf>>| -> bool {
        path.exists() || claimed.as_ref().map(|c| c.contains(path)).unwrap_or(false)
    };

    if !is_occupied(proposed_path, &claimed_paths) {
        if let Some(ref mut c) = claimed_paths {
            c.insert(proposed_path.to_path_buf());
        }
        return CollisionResolution {
            resolved_path: proposed_path.to_path_buf(),
            had_collision: false,
            counter: 0,
        };
    }

    let file_name = match proposed_path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => {
            return CollisionResolution {
                resolved_path: proposed_path.to_path_buf(),
                had_collision: false,
                counter: 0,
            };
        }
    };

    let ExtInfo {
        stem,
        extension,
        is_compound: _,
    } = parse_file_name(file_name);

    let mut counter = 1;
    loop {
        let new_name = format_collision_name(&stem, extension.as_deref(), counter);
        let candidate_path = parent.join(&new_name);

        if !is_occupied(&candidate_path, &claimed_paths) {
            if let Some(ref mut c) = claimed_paths {
                c.insert(candidate_path.clone());
            }
            return CollisionResolution {
                resolved_path: candidate_path,
                had_collision: true,
                counter,
            };
        }

        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_no_collision_when_destination_is_free() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("unique_file.txt");

        let resolution = resolve_collision(&target);
        assert_eq!(resolution.resolved_path, target);
        assert!(!resolution.had_collision);
        assert_eq!(resolution.counter, 0);
    }

    #[test]
    fn test_single_collision_increments_counter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("photo.jpg");
        File::create(&target).unwrap();

        let resolution = resolve_collision(&target);
        assert_eq!(
            resolution.resolved_path,
            temp_dir.path().join("photo (1).jpg")
        );
        assert!(resolution.had_collision);
        assert_eq!(resolution.counter, 1);
    }

    #[test]
    fn test_multiple_collisions_increment_sequentially() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("report.pdf");
        File::create(&target).unwrap();
        File::create(temp_dir.path().join("report (1).pdf")).unwrap();
        File::create(temp_dir.path().join("report (2).pdf")).unwrap();

        let resolution = resolve_collision(&target);
        assert_eq!(
            resolution.resolved_path,
            temp_dir.path().join("report (3).pdf")
        );
        assert!(resolution.had_collision);
        assert_eq!(resolution.counter, 3);
    }

    #[test]
    fn test_compound_extension_collision() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("backup.tar.gz");
        File::create(&target).unwrap();

        let resolution = resolve_collision(&target);
        assert_eq!(
            resolution.resolved_path,
            temp_dir.path().join("backup (1).tar.gz")
        );
        assert!(resolution.had_collision);
        assert_eq!(resolution.counter, 1);
    }

    #[test]
    fn test_extension_less_file_collision() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("Makefile");
        File::create(&target).unwrap();

        let resolution = resolve_collision(&target);
        assert_eq!(
            resolution.resolved_path,
            temp_dir.path().join("Makefile (1)")
        );
        assert!(resolution.had_collision);
        assert_eq!(resolution.counter, 1);
    }

    #[test]
    fn test_batch_claimed_collisions() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target = temp_dir.path().join("photo.jpg");
        // Target does NOT exist on disk yet
        let mut claimed = HashSet::new();

        // First item claims photo.jpg
        let res1 = resolve_collision_with_claimed(&target, Some(&mut claimed));
        assert_eq!(res1.resolved_path, target);
        assert!(!res1.had_collision);

        // Second item with same target is projected as photo (1).jpg
        let res2 = resolve_collision_with_claimed(&target, Some(&mut claimed));
        assert_eq!(res2.resolved_path, temp_dir.path().join("photo (1).jpg"));
        assert!(res2.had_collision);
        assert_eq!(res2.counter, 1);

        // Third item is projected as photo (2).jpg
        let res3 = resolve_collision_with_claimed(&target, Some(&mut claimed));
        assert_eq!(res3.resolved_path, temp_dir.path().join("photo (2).jpg"));
        assert!(res3.had_collision);
        assert_eq!(res3.counter, 2);
    }
}
