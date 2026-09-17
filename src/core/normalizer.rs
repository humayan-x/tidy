//! File extension normalization, compound archive detection, and collision formatting.

use std::path::Path;

/// Known compound archive extensions that must be treated as atomic units.
pub const COMPOUND_EXTENSIONS: &[&str] = &[
    "tar.gz", "tar.bz2", "tar.xz", "tar.zst", "tar.br", "tar.lz4", "tar.lz", "tar.sz",
];

/// Parsed breakdown of a filename into stem and normalized extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtInfo {
    /// The filename stem without the categorized extension.
    pub stem: String,
    /// The normalized lowercase extension, if any.
    pub extension: Option<String>,
    /// Whether the extension was identified as a compound format (e.g. `.tar.gz`).
    pub is_compound: bool,
}

/// Parses a filename or path and extracts the stem and normalized lowercase extension.
///
/// If a known compound extension is present (e.g. `archive.tar.gz`), it is extracted as
/// an atomic extension (`tar.gz`) rather than splitting on the last dot (`gz`).
pub fn parse_file_name(filename: &str) -> ExtInfo {
    // Check compound extensions first
    for &compound in COMPOUND_EXTENSIONS {
        let suffix = format!(".{}", compound);
        if filename.len() > suffix.len() {
            let split_pos = filename.len() - suffix.len();
            if filename.is_char_boundary(split_pos)
                && filename[split_pos..].eq_ignore_ascii_case(&suffix)
            {
                let stem = filename[..split_pos].to_string();
                return ExtInfo {
                    stem,
                    extension: Some(compound.to_string()),
                    is_compound: true,
                };
            }
        }
    }

    // Fall back to standard single-extension splitting
    let path = Path::new(filename);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    let stem = match &extension {
        Some(_) => path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename)
            .to_string(),
        None => filename.to_string(),
    };

    ExtInfo {
        stem,
        extension,
        is_compound: false,
    }
}

/// Formats a non-destructive collision filename with an incremental counter.
///
/// Examples:
/// - `photo.jpg` -> `photo (1).jpg`
/// - `backup.tar.gz` -> `backup (1).tar.gz`
/// - `README` -> `README (1)`
pub fn format_collision_name(stem: &str, extension: Option<&str>, counter: usize) -> String {
    match extension {
        Some(ext) => format!("{} ({}).{}", stem, counter, ext),
        None => format!("{} ({})", stem, counter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_extensions() {
        let info = parse_file_name("document.pdf");
        assert_eq!(info.stem, "document");
        assert_eq!(info.extension.as_deref(), Some("pdf"));
        assert!(!info.is_compound);

        let info = parse_file_name("PHOTO.JPEG");
        assert_eq!(info.stem, "PHOTO");
        assert_eq!(info.extension.as_deref(), Some("jpeg"));
        assert!(!info.is_compound);
    }

    #[test]
    fn test_compound_archive_extensions() {
        let info = parse_file_name("release-v1.0.tar.gz");
        assert_eq!(info.stem, "release-v1.0");
        assert_eq!(info.extension.as_deref(), Some("tar.gz"));
        assert!(info.is_compound);

        let info = parse_file_name("backup.tar.xz");
        assert_eq!(info.stem, "backup");
        assert_eq!(info.extension.as_deref(), Some("tar.xz"));
        assert!(info.is_compound);

        let info = parse_file_name("source.TAR.BZ2");
        assert_eq!(info.stem, "source");
        assert_eq!(info.extension.as_deref(), Some("tar.bz2"));
        assert!(info.is_compound);
    }

    #[test]
    fn test_extension_less_files() {
        let info = parse_file_name("Makefile");
        assert_eq!(info.stem, "Makefile");
        assert_eq!(info.extension, None);
        assert!(!info.is_compound);

        let info = parse_file_name("LICENSE");
        assert_eq!(info.stem, "LICENSE");
        assert_eq!(info.extension, None);
    }

    #[test]
    fn test_multiple_dots_non_compound() {
        let info = parse_file_name("my.file.with.dots.txt");
        assert_eq!(info.stem, "my.file.with.dots");
        assert_eq!(info.extension.as_deref(), Some("txt"));
        assert!(!info.is_compound);
    }

    #[test]
    fn test_collision_name_formatting() {
        assert_eq!(
            format_collision_name("report", Some("pdf"), 1),
            "report (1).pdf"
        );
        assert_eq!(
            format_collision_name("backup", Some("tar.gz"), 2),
            "backup (2).tar.gz"
        );
        assert_eq!(format_collision_name("Makefile", None, 3), "Makefile (3)");
    }

    #[test]
    fn test_unicode_compound_archive_extensions() {
        // Turkish capital I with dot and German capital sharp S
        let info = parse_file_name("İSTANBUL_ẞ_backup.tar.gz");
        assert_eq!(info.stem, "İSTANBUL_ẞ_backup");
        assert_eq!(info.extension.as_deref(), Some("tar.gz"));
        assert!(info.is_compound);
    }
}
