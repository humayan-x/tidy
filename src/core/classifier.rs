//! File classification logic, matching rules, and directory recursion guards.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::core::config::CompiledRules;
use crate::core::normalizer::{parse_file_name, ExtInfo};
use crate::core::sniffer::sniff_file;

/// The reason a file or directory was ignored during scanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IgnoreReason {
    /// File or directory starts with a dot (hidden).
    Hidden,
    /// Path matches a configured ignore pattern (e.g. `.crdownload`, `.tmp`).
    Pattern,
    /// Directory is one of the destination category folders (prevents recursive loop).
    ActiveDestination,
    /// Path is not a regular file or recognized symlink.
    NotRegularFile,
}

/// The outcome of evaluating a file against the rule engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassificationResult {
    /// The file matches a category and should be moved.
    Move {
        category: String,
        target_subfolder: String,
        target_path: PathBuf,
        stem: String,
        extension: Option<String>,
        is_compound: bool,
    },
    /// The file was deliberately skipped.
    Ignore { reason: IgnoreReason },
    /// The file does not match any known category and was left untouched.
    Unmatched,
}

/// High-level classifier orchestrating rule evaluation, extension normalization, and sniffing.
pub struct Classifier {
    rules: CompiledRules,
    active_destinations: HashSet<String>,
}

impl Classifier {
    pub fn new(rules: CompiledRules) -> Self {
        let mut active_destinations = HashSet::new();
        for category in rules.extension_to_category.values() {
            active_destinations.insert(rules.get_destination_folder(category));
        }
        for dest in rules.destinations.values() {
            active_destinations.insert(dest.clone());
        }

        Self {
            rules,
            active_destinations,
        }
    }

    /// Returns a reference to the underlying compiled rules.
    #[allow(dead_code)]
    pub fn rules(&self) -> &CompiledRules {
        &self.rules
    }

    /// Returns a reference to the active destination folder names.
    pub fn active_destinations(&self) -> &HashSet<String> {
        &self.active_destinations
    }

    /// Evaluates a file path located under `root_dir`.
    ///
    /// * `file_path`: Absolute or relative path to the candidate file.
    /// * `root_dir`: The root organizing directory being scanned.
    pub fn classify(&self, file_path: &Path, root_dir: &Path) -> ClassificationResult {
        let file_name = match file_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => {
                return ClassificationResult::Ignore {
                    reason: IgnoreReason::NotRegularFile,
                }
            }
        };

        // 1. Check if the path is an active destination category folder under root_dir to prevent recursion
        if file_path.parent() == Some(root_dir)
            && self.active_destinations.contains(file_name)
            && !file_path.is_file()
        {
            return ClassificationResult::Ignore {
                reason: IgnoreReason::ActiveDestination,
            };
        }

        // 2. Check if the parent of this file is already one of the active destination folders
        //    (e.g., do not re-organize files already inside root_dir/Images)
        if let Ok(relative) = file_path.strip_prefix(root_dir) {
            if relative.components().count() > 1 {
                if let Some(first_component) = relative.components().next() {
                    let comp_str = first_component.as_os_str().to_string_lossy();
                    if self.active_destinations.contains(comp_str.as_ref()) {
                        return ClassificationResult::Ignore {
                            reason: IgnoreReason::ActiveDestination,
                        };
                    }
                }
            }
        }

        // 3. Check ignore rules (hidden files and glob patterns)
        if self.rules.ignore_hidden && file_name.starts_with('.') {
            return ClassificationResult::Ignore {
                reason: IgnoreReason::Hidden,
            };
        }

        let rel_str = file_path
            .strip_prefix(root_dir)
            .ok()
            .and_then(|p| p.to_str());

        if self.rules.is_path_ignored(file_name, rel_str) {
            return ClassificationResult::Ignore {
                reason: IgnoreReason::Pattern,
            };
        }

        // 4. Parse file name and detect normalized extension
        let ExtInfo {
            stem,
            extension,
            is_compound,
        } = parse_file_name(file_name);

        // 5. Match normalized extension against category tables
        if let Some(ref ext) = extension {
            if let Some(category) = self.rules.extension_to_category.get(ext) {
                let target_subfolder = self.rules.resolve_target_subfolder(category, ext);
                let target_path = root_dir.join(&target_subfolder).join(file_name);
                return ClassificationResult::Move {
                    category: category.clone(),
                    target_subfolder,
                    target_path,
                    stem,
                    extension: Some(ext.clone()),
                    is_compound,
                };
            }
        }

        // 6. Sniffer fallback: If extension is missing or unrecognized, inspect magic bytes
        if let Some(sniffed) = sniff_file(file_path) {
            // First check if the sniffed extension matches a category
            let matched_category = self
                .rules
                .extension_to_category
                .get(&sniffed.extension)
                .cloned()
                .or_else(|| sniffed.suggested_category.map(String::from));

            if let Some(category) = matched_category {
                let target_subfolder = self
                    .rules
                    .resolve_target_subfolder(&category, &sniffed.extension);
                let target_path = root_dir.join(&target_subfolder).join(file_name);
                return ClassificationResult::Move {
                    category,
                    target_subfolder,
                    target_path,
                    stem,
                    extension: Some(sniffed.extension),
                    is_compound: false,
                };
            }
        }

        // 7. Non-destructive guarantee: Unmatched files remain untouched
        ClassificationResult::Unmatched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use std::fs::File;
    use std::io::Write;

    fn setup_classifier() -> Classifier {
        let config = Config::default();
        let rules = config.compile().unwrap();
        Classifier::new(rules)
    }

    #[test]
    fn test_classify_standard_files() {
        let classifier = setup_classifier();
        let root = Path::new("/downloads");

        let res = classifier.classify(Path::new("/downloads/photo.jpg"), root);
        match res {
            ClassificationResult::Move {
                category,
                target_subfolder,
                target_path,
                stem,
                ..
            } => {
                assert_eq!(category, "Images");
                assert_eq!(target_subfolder, "Images/JPG");
                assert_eq!(target_path, Path::new("/downloads/Images/JPG/photo.jpg"));
                assert_eq!(stem, "photo");
            }
            _ => panic!("Expected Move for photo.jpg"),
        }

        let res = classifier.classify(Path::new("/downloads/backup.tar.gz"), root);
        match res {
            ClassificationResult::Move {
                category,
                target_subfolder,
                target_path,
                is_compound,
                ..
            } => {
                assert_eq!(category, "Archives");
                assert_eq!(target_subfolder, "Archives/TAR.GZ");
                assert_eq!(
                    target_path,
                    Path::new("/downloads/Archives/TAR.GZ/backup.tar.gz")
                );
                assert!(is_compound);
            }
            _ => panic!("Expected Move for backup.tar.gz"),
        }
    }

    #[test]
    fn test_classify_ignored_files() {
        let classifier = setup_classifier();
        let root = Path::new("/downloads");

        let res = classifier.classify(Path::new("/downloads/.DS_Store"), root);
        assert_eq!(
            res,
            ClassificationResult::Ignore {
                reason: IgnoreReason::Hidden
            }
        );

        let res = classifier.classify(Path::new("/downloads/movie.mp4.crdownload"), root);
        assert_eq!(
            res,
            ClassificationResult::Ignore {
                reason: IgnoreReason::Pattern
            }
        );
    }

    #[test]
    fn test_classify_prevents_active_dir_recursion() {
        let classifier = setup_classifier();
        let root = Path::new("/downloads");

        // Direct destination directory
        let res = classifier.classify(Path::new("/downloads/Images"), root);
        assert_eq!(
            res,
            ClassificationResult::Ignore {
                reason: IgnoreReason::ActiveDestination
            }
        );

        // Nested file already inside organized category folder
        let res = classifier.classify(Path::new("/downloads/Images/existing.png"), root);
        assert_eq!(
            res,
            ClassificationResult::Ignore {
                reason: IgnoreReason::ActiveDestination
            }
        );
    }

    #[test]
    fn test_classify_root_file_named_as_category() {
        let classifier = setup_classifier();
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();

        // Create a regular file directly in root named "Images" with PNG magic bytes
        let file_path = root.join("Images");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
            .unwrap();

        let res = classifier.classify(&file_path, root);
        match res {
            ClassificationResult::Move { category, .. } => {
                assert_eq!(category, "Images");
            }
            other => panic!("Expected Move for file named Images, got {:?}", other),
        }
    }

    #[test]
    fn test_classify_sniffing_fallback() {
        let classifier = setup_classifier();
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();

        // Create a file without extension containing PNG magic bytes
        let file_path = root.join("my_image");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
            .unwrap();

        let res = classifier.classify(&file_path, root);
        match res {
            ClassificationResult::Move {
                category,
                target_subfolder,
                extension,
                ..
            } => {
                assert_eq!(category, "Images");
                assert_eq!(target_subfolder, "Images/PNG");
                assert_eq!(extension.as_deref(), Some("png"));
            }
            _ => panic!("Expected sniffing to classify image without extension"),
        }
    }

    #[test]
    fn test_classify_flat_mode_when_disabled() {
        let mut config = Config::default();
        config.settings.nest_by_extension = false;
        let rules = config.compile().unwrap();
        let classifier = Classifier::new(rules);
        let root = Path::new("/downloads");

        let res = classifier.classify(Path::new("/downloads/photo.jpg"), root);
        match res {
            ClassificationResult::Move {
                target_subfolder,
                target_path,
                ..
            } => {
                assert_eq!(target_subfolder, "Images");
                assert_eq!(target_path, Path::new("/downloads/Images/photo.jpg"));
            }
            _ => panic!("Expected Move for photo.jpg"),
        }
    }

    #[test]
    fn test_unmatched_files_left_untouched() {
        let classifier = setup_classifier();
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();

        let file_path = root.join("random_unmatched.xyz999");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"unknown binary data").unwrap();

        let res = classifier.classify(&file_path, root);
        assert_eq!(res, ClassificationResult::Unmatched);
    }

    #[test]
    fn test_classify_human_readable_subfolders() {
        let classifier = setup_classifier();
        let root = Path::new("/downloads");

        // PDFs
        let res = classifier.classify(Path::new("/downloads/contract.pdf"), root);
        match res {
            ClassificationResult::Move {
                category,
                target_subfolder,
                ..
            } => {
                assert_eq!(category, "Documents");
                assert_eq!(target_subfolder, "Documents/PDFs");
            }
            _ => panic!("Expected Move for contract.pdf"),
        }

        // Word (.doc and .docx)
        for name in &["memo.doc", "report.docx"] {
            let res = classifier.classify(Path::new(&format!("/downloads/{}", name)), root);
            match res {
                ClassificationResult::Move {
                    category,
                    target_subfolder,
                    ..
                } => {
                    assert_eq!(category, "Documents");
                    assert_eq!(target_subfolder, "Documents/Word");
                }
                _ => panic!("Expected Move for {}", name),
            }
        }

        // Excel (.xls, .xlsx, .ods)
        for name in &["budget.xls", "sales.xlsx", "calc.ods"] {
            let res = classifier.classify(Path::new(&format!("/downloads/{}", name)), root);
            match res {
                ClassificationResult::Move {
                    category,
                    target_subfolder,
                    ..
                } => {
                    assert_eq!(category, "Documents");
                    assert_eq!(target_subfolder, "Documents/Excel");
                }
                _ => panic!("Expected Move for {}", name),
            }
        }

        // PowerPoint (.ppt, .pptx)
        for name in &["slides.ppt", "pitch.pptx"] {
            let res = classifier.classify(Path::new(&format!("/downloads/{}", name)), root);
            match res {
                ClassificationResult::Move {
                    category,
                    target_subfolder,
                    ..
                } => {
                    assert_eq!(category, "Documents");
                    assert_eq!(target_subfolder, "Documents/PowerPoint");
                }
                _ => panic!("Expected Move for {}", name),
            }
        }
    }

    #[test]
    fn test_classify_unmapped_formats_fallback_to_uppercase() {
        let classifier = setup_classifier();
        let root = Path::new("/downloads");

        let cases = [
            ("notes.txt", "Documents", "Documents/TXT"),
            ("document.odt", "Documents", "Documents/ODT"),
            ("slides.odp", "Documents", "Documents/ODP"),
            ("readme.md", "Documents", "Documents/MD"),
            ("book.epub", "Documents", "Documents/EPUB"),
            ("data.csv", "Documents", "Documents/CSV"),
            ("photo.jpg", "Images", "Images/JPG"),
            ("song.mp3", "Audio", "Audio/MP3"),
            ("archive.tar.gz", "Archives", "Archives/TAR.GZ"),
            ("script.py", "Code", "Code/PY"),
        ];

        for (filename, expected_cat, expected_subfolder) in cases {
            let res = classifier.classify(Path::new(&format!("/downloads/{}", filename)), root);
            match res {
                ClassificationResult::Move {
                    category,
                    target_subfolder,
                    ..
                } => {
                    assert_eq!(category, expected_cat);
                    assert_eq!(target_subfolder, expected_subfolder);
                }
                _ => panic!("Expected Move for {}", filename),
            }
        }
    }

    #[test]
    fn test_classify_sniffed_pdf_uses_readable_folder() {
        let classifier = setup_classifier();
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();

        // Create an extensionless file with PDF magic bytes (%PDF-1.5)
        let file_path = root.join("my_document");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"%PDF-1.5\nfake pdf content").unwrap();

        let res = classifier.classify(&file_path, root);
        match res {
            ClassificationResult::Move {
                category,
                target_subfolder,
                extension,
                ..
            } => {
                assert_eq!(category, "Documents");
                assert_eq!(target_subfolder, "Documents/PDFs");
                assert_eq!(extension.as_deref(), Some("pdf"));
            }
            _ => panic!("Expected sniffing to classify PDF into Documents/PDFs"),
        }
    }
}
