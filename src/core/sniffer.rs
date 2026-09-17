//! Magic-byte sniffing fallback using `infer` for extension-less or unrecognized files.

use std::path::Path;

/// Information retrieved from inspecting a file's magic header bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SniffedType {
    /// Inferred standard extension (e.g. `jpg`, `png`, `zip`, `pdf`).
    pub extension: String,
    /// Detected MIME type string (e.g. `image/jpeg`).
    pub mime_type: String,
    /// Fallback category suggested by the MIME classification.
    pub suggested_category: Option<&'static str>,
}

/// Inspects header bytes from the file at `path` to identify its type.
pub fn sniff_file(path: &Path) -> Option<SniffedType> {
    let kind = infer::get_from_path(path).ok()??;
    let mime = kind.mime_type();
    let ext = kind.extension().to_lowercase();
    let category = mime_to_category(mime);

    Some(SniffedType {
        extension: ext,
        mime_type: mime.to_string(),
        suggested_category: category,
    })
}

/// Inspects a byte buffer directly to identify its type (useful for testing or stream buffers).
pub fn sniff_bytes(bytes: &[u8]) -> Option<SniffedType> {
    let kind = infer::get(bytes)?;
    let mime = kind.mime_type();
    let ext = kind.extension().to_lowercase();
    let category = mime_to_category(mime);

    Some(SniffedType {
        extension: ext,
        mime_type: mime.to_string(),
        suggested_category: category,
    })
}

/// Maps standard MIME types to tidy default category groups.
pub fn mime_to_category(mime: &str) -> Option<&'static str> {
    if mime.starts_with("image/") {
        Some("Images")
    } else if mime.starts_with("video/") {
        Some("Video")
    } else if mime.starts_with("audio/") {
        Some("Audio")
    } else if mime == "application/pdf"
        || mime.starts_with("text/")
        || mime.contains("document")
        || mime.contains("sheet")
        || mime.contains("presentation")
        || mime == "application/epub+zip"
    {
        Some("Documents")
    } else if mime == "application/zip"
        || mime == "application/x-tar"
        || mime == "application/gzip"
        || mime == "application/x-bzip2"
        || mime == "application/x-xz"
        || mime == "application/x-7z-compressed"
        || mime == "application/vnd.rar"
    {
        Some("Archives")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sniff_png_bytes() {
        // PNG magic bytes: 89 50 4E 47 0D 0A 1A 0A
        let png_header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let sniffed = sniff_bytes(&png_header).expect("should identify PNG");
        assert_eq!(sniffed.extension, "png");
        assert_eq!(sniffed.mime_type, "image/png");
        assert_eq!(sniffed.suggested_category, Some("Images"));
    }

    #[test]
    fn test_sniff_pdf_bytes() {
        // PDF magic bytes: %PDF- (25 50 44 46 2D)
        let pdf_header = b"%PDF-1.5 test content";
        let sniffed = sniff_bytes(pdf_header).expect("should identify PDF");
        assert_eq!(sniffed.extension, "pdf");
        assert_eq!(sniffed.mime_type, "application/pdf");
        assert_eq!(sniffed.suggested_category, Some("Documents"));
    }

    #[test]
    fn test_sniff_zip_bytes() {
        // ZIP magic bytes: PK\x03\x04
        let zip_header: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
        let sniffed = sniff_bytes(&zip_header).expect("should identify ZIP");
        assert_eq!(sniffed.extension, "zip");
        assert_eq!(sniffed.suggested_category, Some("Archives"));
    }

    #[test]
    fn test_sniff_unknown_bytes() {
        let unknown = [0x00, 0x01, 0x02, 0x03];
        let sniffed = sniff_bytes(&unknown);
        assert!(sniffed.is_none());
    }
}
