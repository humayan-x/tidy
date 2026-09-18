//! Default category taxonomy and standard ignore rules for zero-config operation.

use std::collections::BTreeMap;

/// Returns the standard default categories and their associated file extensions.
///
/// Categories strictly match the blueprint:
/// - Images
/// - Documents
/// - Audio
/// - Video
/// - Archives (including compound extensions like tar.gz)
/// - Code/Data
pub fn default_categories() -> BTreeMap<String, Vec<String>> {
    let mut categories = BTreeMap::new();

    categories.insert(
        "Images".to_string(),
        vec![
            "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "tiff", "tif", "heic", "raw",
            "psd", "ai", "eps", "avif",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    categories.insert(
        "Documents".to_string(),
        vec![
            "pdf", "epub", "mobi", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods",
            "odp", "txt", "md", "rtf", "csv", "tsv", "pages", "numbers", "key",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    categories.insert(
        "Audio".to_string(),
        vec![
            "mp3", "flac", "wav", "aac", "ogg", "m4a", "wma", "aiff", "aif", "opus", "mid", "midi",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    categories.insert(
        "Video".to_string(),
        vec![
            "mp4", "mkv", "mov", "avi", "webm", "wmv", "flv", "m4v", "3gp", "ts",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    categories.insert(
        "Archives".to_string(),
        vec![
            "zip", "rar", "7z", "gz", "bz2", "xz", "zst", "tar", "iso", "dmg", "pkg", "deb", "rpm",
            "tar.gz", "tar.bz2", "tar.xz", "tar.zst", "tar.br", "tar.lz4",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    categories.insert(
        "Code".to_string(),
        vec![
            "rs", "py", "js", "ts", "jsx", "tsx", "json", "yaml", "yml", "toml", "sql", "sh",
            "bash", "zsh", "html", "htm", "css", "scss", "sass", "c", "cpp", "h", "hpp", "go",
            "java", "kt", "swift", "rb", "php", "lua", "vim", "xml", "diff", "patch",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    );

    categories
}

/// Returns the default human-readable folder name mapping for file extensions.
///
/// Groups related extensions into clean, readable directory names:
/// - `.pdf` -> `PDFs`
/// - `.doc`, `.docx` -> `Word`
/// - `.xls`, `.xlsx`, `.ods` -> `Excel`
/// - `.ppt`, `.pptx` -> `PowerPoint`
///
/// Extensions not present in this map fall back to their normalized uppercase extension.
pub fn default_extension_subfolders() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    map.insert("pdf".to_string(), "PDFs".to_string());
    map.insert("doc".to_string(), "Word".to_string());
    map.insert("docx".to_string(), "Word".to_string());
    map.insert("xls".to_string(), "Excel".to_string());
    map.insert("xlsx".to_string(), "Excel".to_string());
    map.insert("ods".to_string(), "Excel".to_string());
    map.insert("ppt".to_string(), "PowerPoint".to_string());
    map.insert("pptx".to_string(), "PowerPoint".to_string());
    map
}

/// Returns the standard list of glob patterns to ignore.
///
/// Filters active browser downloads, partial files, OS indexing files, and editor swap files.
pub fn default_ignore_patterns() -> Vec<String> {
    vec![
        // Browser and torrent download in-progress guards
        "*.crdownload".to_string(),
        "*.part".to_string(),
        "*.download".to_string(),
        "*.aria2".to_string(),
        "*.tmp".to_string(),
        "*.partial".to_string(),
        "*.downloading".to_string(),
        "*.!qb".to_string(),
        "*.!ut".to_string(),
        // macOS system files
        ".DS_Store".to_string(),
        "._*".to_string(),
        ".Spotlight-V100".to_string(),
        ".Trashes".to_string(),
        // Windows system files
        "Thumbs.db".to_string(),
        "desktop.ini".to_string(),
        // Editor swap and temp files
        "*.swp".to_string(),
        "*~".to_string(),
        "*.bak".to_string(),
    ]
}

/// Quick check whether a filename represents an in-progress or temporary download.
pub fn is_download_in_progress(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower.ends_with(".crdownload")
        || lower.ends_with(".part")
        || lower.ends_with(".download")
        || lower.ends_with(".aria2")
        || lower.ends_with(".tmp")
        || lower.ends_with(".partial")
        || lower.ends_with(".downloading")
        || lower.ends_with(".!qb")
        || lower.ends_with(".!ut")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_download_in_progress() {
        assert!(is_download_in_progress("movie.mp4.crdownload"));
        assert!(is_download_in_progress("archive.zip.part"));
        assert!(is_download_in_progress("file.download"));
        assert!(is_download_in_progress("dataset.csv.aria2"));
        assert!(is_download_in_progress("file.tmp"));
        assert!(is_download_in_progress("ubuntu.iso.!qB"));
        assert!(is_download_in_progress("game.zip.!ut"));
        assert!(!is_download_in_progress("movie.mp4"));
        assert!(!is_download_in_progress("archive.zip"));
    }

    #[test]
    fn test_default_categories_contain_essentials() {
        let cats = default_categories();
        assert!(cats.contains_key("Images"));
        assert!(cats.contains_key("Documents"));
        assert!(cats.contains_key("Audio"));
        assert!(cats.contains_key("Video"));
        assert!(cats.contains_key("Archives"));
        assert!(cats.contains_key("Code"));

        let archives = cats.get("Archives").unwrap();
        assert!(archives.contains(&"tar.gz".to_string()));
        assert!(archives.contains(&"zip".to_string()));
    }

    #[test]
    fn test_default_ignore_patterns_contain_downloads() {
        let patterns = default_ignore_patterns();
        assert!(patterns.contains(&"*.crdownload".to_string()));
        assert!(patterns.contains(&"*.part".to_string()));
        assert!(patterns.contains(&"*.tmp".to_string()));
        assert!(patterns.contains(&".DS_Store".to_string()));
    }

    #[test]
    fn test_default_extension_subfolders() {
        let subfolders = default_extension_subfolders();
        assert_eq!(subfolders.get("pdf"), Some(&"PDFs".to_string()));
        assert_eq!(subfolders.get("doc"), Some(&"Word".to_string()));
        assert_eq!(subfolders.get("docx"), Some(&"Word".to_string()));
        assert_eq!(subfolders.get("xls"), Some(&"Excel".to_string()));
        assert_eq!(subfolders.get("xlsx"), Some(&"Excel".to_string()));
        assert_eq!(subfolders.get("ods"), Some(&"Excel".to_string()));
        assert_eq!(subfolders.get("ppt"), Some(&"PowerPoint".to_string()));
        assert_eq!(subfolders.get("pptx"), Some(&"PowerPoint".to_string()));

        // Ensure other extensions are unmapped so they fall back to uppercase
        assert_eq!(subfolders.get("odt"), None);
        assert_eq!(subfolders.get("txt"), None);
        assert_eq!(subfolders.get("epub"), None);
        assert_eq!(subfolders.get("png"), None);
    }
}
