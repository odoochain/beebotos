//! MIME Type Detection
//!
//! Provides MIME type detection from file magic numbers and file extensions.
//! Supports common image, document, audio, and video formats.

use std::collections::HashMap;
use std::path::Path;

use once_cell::sync::Lazy;

use crate::error::{AgentError, Result};

/// Magic number patterns for file type detection
#[derive(Debug, Clone)]
struct MagicPattern {
    /// Magic bytes to match
    bytes: Vec<u8>,
    /// Offset in file where magic bytes start
    offset: usize,
    /// MIME type for this pattern
    mime_type: &'static str,
    /// File extension for this pattern
    extension: &'static str,
}

/// Known magic number patterns
static MAGIC_PATTERNS: Lazy<Vec<MagicPattern>> = Lazy::new(|| {
    vec![
        // Images
        MagicPattern {
            bytes: vec![0xFF, 0xD8, 0xFF],
            offset: 0,
            mime_type: "image/jpeg",
            extension: "jpg",
        },
        MagicPattern {
            bytes: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            offset: 0,
            mime_type: "image/png",
            extension: "png",
        },
        MagicPattern {
            bytes: vec![0x47, 0x49, 0x46, 0x38],
            offset: 0,
            mime_type: "image/gif",
            extension: "gif",
        },
        MagicPattern {
            bytes: vec![0x52, 0x49, 0x46, 0x46],
            offset: 0,
            mime_type: "image/webp",
            extension: "webp",
        },
        MagicPattern {
            bytes: vec![0x42, 0x4D],
            offset: 0,
            mime_type: "image/bmp",
            extension: "bmp",
        },
        MagicPattern {
            bytes: vec![0x49, 0x49, 0x2A, 0x00],
            offset: 0,
            mime_type: "image/tiff",
            extension: "tiff",
        },
        MagicPattern {
            bytes: vec![0x4D, 0x4D, 0x00, 0x2A],
            offset: 0,
            mime_type: "image/tiff",
            extension: "tiff",
        },
        MagicPattern {
            bytes: vec![0x38, 0x42, 0x50, 0x53],
            offset: 0,
            mime_type: "image/vnd.adobe.photoshop",
            extension: "psd",
        },
        MagicPattern {
            bytes: vec![0xFF, 0x0A],
            offset: 0,
            mime_type: "image/jxl",
            extension: "jxl",
        },
        // SVG (XML-based, check for <svg)
        MagicPattern {
            bytes: b"<?xml".to_vec(),
            offset: 0,
            mime_type: "image/svg+xml",
            extension: "svg",
        },
        // ICO
        MagicPattern {
            bytes: vec![0x00, 0x00, 0x01, 0x00],
            offset: 0,
            mime_type: "image/x-icon",
            extension: "ico",
        },
        // HEIC/HEIF
        MagicPattern {
            bytes: vec![0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x63],
            offset: 4,
            mime_type: "image/heic",
            extension: "heic",
        },
        MagicPattern {
            bytes: vec![0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x66],
            offset: 4,
            mime_type: "image/heif",
            extension: "heif",
        },
        // Audio
        MagicPattern {
            bytes: vec![0x49, 0x44, 0x33],
            offset: 0,
            mime_type: "audio/mpeg",
            extension: "mp3",
        },
        MagicPattern {
            bytes: vec![0xFF, 0xFB],
            offset: 0,
            mime_type: "audio/mpeg",
            extension: "mp3",
        },
        MagicPattern {
            bytes: vec![0xFF, 0xF3],
            offset: 0,
            mime_type: "audio/mpeg",
            extension: "mp3",
        },
        MagicPattern {
            bytes: vec![0xFF, 0xF2],
            offset: 0,
            mime_type: "audio/mpeg",
            extension: "mp3",
        },
        MagicPattern {
            bytes: b"RIFF".to_vec(),
            offset: 0,
            mime_type: "audio/wav",
            extension: "wav",
        },
        MagicPattern {
            bytes: b"OggS".to_vec(),
            offset: 0,
            mime_type: "audio/ogg",
            extension: "ogg",
        },
        MagicPattern {
            bytes: b"fLaC".to_vec(),
            offset: 0,
            mime_type: "audio/flac",
            extension: "flac",
        },
        MagicPattern {
            bytes: b"MThd".to_vec(),
            offset: 0,
            mime_type: "audio/midi",
            extension: "mid",
        },
        MagicPattern {
            bytes: b"FORM".to_vec(),
            offset: 0,
            mime_type: "audio/aiff",
            extension: "aiff",
        },
        // Video
        MagicPattern {
            bytes: b"ftypisom".to_vec(),
            offset: 4,
            mime_type: "video/mp4",
            extension: "mp4",
        },
        MagicPattern {
            bytes: b"ftypmp42".to_vec(),
            offset: 4,
            mime_type: "video/mp4",
            extension: "mp4",
        },
        MagicPattern {
            bytes: b"ftypMSNV".to_vec(),
            offset: 4,
            mime_type: "video/mp4",
            extension: "mp4",
        },
        MagicPattern {
            bytes: b"ftypM4V ".to_vec(),
            offset: 4,
            mime_type: "video/mp4",
            extension: "mp4",
        },
        MagicPattern {
            bytes: b"ftypavc1".to_vec(),
            offset: 4,
            mime_type: "video/mp4",
            extension: "mp4",
        },
        MagicPattern {
            bytes: vec![0x1A, 0x45, 0xDF, 0xA3],
            offset: 0,
            mime_type: "video/webm",
            extension: "webm",
        },
        MagicPattern {
            bytes: vec![0x1A, 0x45, 0xDF, 0xA3],
            offset: 0,
            mime_type: "video/webm",
            extension: "webm",
        },
        MagicPattern {
            bytes: b"RIFF".to_vec(),
            offset: 0,
            mime_type: "video/avi",
            extension: "avi",
        },
        MagicPattern {
            bytes: b"FLV".to_vec(),
            offset: 0,
            mime_type: "video/x-flv",
            extension: "flv",
        },
        // Documents
        MagicPattern {
            bytes: b"%PDF".to_vec(),
            offset: 0,
            mime_type: "application/pdf",
            extension: "pdf",
        },
        MagicPattern {
            bytes: b"PK\x03\x04".to_vec(),
            offset: 0,
            mime_type: "application/zip",
            extension: "zip",
        },
        MagicPattern {
            bytes: b"PK\x05\x06".to_vec(),
            offset: 0,
            mime_type: "application/zip",
            extension: "zip",
        },
        MagicPattern {
            bytes: b"PK\x07\x08".to_vec(),
            offset: 0,
            mime_type: "application/zip",
            extension: "zip",
        },
        MagicPattern {
            bytes: b"Rar!".to_vec(),
            offset: 0,
            mime_type: "application/x-rar-compressed",
            extension: "rar",
        },
        MagicPattern {
            bytes: b"7z\xBC\xAF\x27\x1C".to_vec(),
            offset: 0,
            mime_type: "application/x-7z-compressed",
            extension: "7z",
        },
        MagicPattern {
            bytes: b"\x1F\x8B".to_vec(),
            offset: 0,
            mime_type: "application/gzip",
            extension: "gz",
        },
        MagicPattern {
            bytes: b"BZ".to_vec(),
            offset: 0,
            mime_type: "application/x-bzip2",
            extension: "bz2",
        },
        MagicPattern {
            bytes: b"ustar\x00".to_vec(),
            offset: 257,
            mime_type: "application/x-tar",
            extension: "tar",
        },
        MagicPattern {
            bytes: b"ustar  ".to_vec(),
            offset: 257,
            mime_type: "application/x-tar",
            extension: "tar",
        },
        MagicPattern {
            bytes: b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1".to_vec(),
            offset: 0,
            mime_type: "application/msword",
            extension: "doc",
        },
        // Office Open XML (docx, xlsx, pptx)
        MagicPattern {
            bytes: b"PK\x03\x04".to_vec(),
            offset: 0,
            mime_type: "application/vnd.openxmlformats",
            extension: "docx",
        },
        // Text
        MagicPattern {
            bytes: b"\xEF\xBB\xBF".to_vec(),
            offset: 0,
            mime_type: "text/plain",
            extension: "txt",
        },
        MagicPattern {
            bytes: b"\xFF\xFE".to_vec(),
            offset: 0,
            mime_type: "text/plain",
            extension: "txt",
        },
        MagicPattern {
            bytes: b"\xFE\xFF".to_vec(),
            offset: 0,
            mime_type: "text/plain",
            extension: "txt",
        },
        // JSON
        MagicPattern {
            bytes: b"{".to_vec(),
            offset: 0,
            mime_type: "application/json",
            extension: "json",
        },
        // XML
        MagicPattern {
            bytes: b"<?xml ".to_vec(),
            offset: 0,
            mime_type: "application/xml",
            extension: "xml",
        },
        // HTML
        MagicPattern {
            bytes: b"<!DOCTYPE html".to_vec(),
            offset: 0,
            mime_type: "text/html",
            extension: "html",
        },
        MagicPattern {
            bytes: b"<!doctype html".to_vec(),
            offset: 0,
            mime_type: "text/html",
            extension: "html",
        },
        MagicPattern {
            bytes: b"<html".to_vec(),
            offset: 0,
            mime_type: "text/html",
            extension: "html",
        },
    ]
});

/// Extension to MIME type mapping
static EXT_TO_MIME: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Images
    map.insert("jpg", "image/jpeg");
    map.insert("jpeg", "image/jpeg");
    map.insert("png", "image/png");
    map.insert("gif", "image/gif");
    map.insert("webp", "image/webp");
    map.insert("bmp", "image/bmp");
    map.insert("tiff", "image/tiff");
    map.insert("tif", "image/tiff");
    map.insert("svg", "image/svg+xml");
    map.insert("ico", "image/x-icon");
    map.insert("heic", "image/heic");
    map.insert("heif", "image/heif");
    map.insert("avif", "image/avif");
    map.insert("jxl", "image/jxl");
    map.insert("psd", "image/vnd.adobe.photoshop");
    map.insert("raw", "image/x-raw");
    map.insert("cr2", "image/x-canon-cr2");
    map.insert("nef", "image/x-nikon-nef");

    // Audio
    map.insert("mp3", "audio/mpeg");
    map.insert("wav", "audio/wav");
    map.insert("ogg", "audio/ogg");
    map.insert("oga", "audio/ogg");
    map.insert("flac", "audio/flac");
    map.insert("m4a", "audio/mp4");
    map.insert("aac", "audio/aac");
    map.insert("wma", "audio/x-ms-wma");
    map.insert("mid", "audio/midi");
    map.insert("midi", "audio/midi");
    map.insert("aiff", "audio/aiff");
    map.insert("au", "audio/basic");

    // Video
    map.insert("mp4", "video/mp4");
    map.insert("m4v", "video/mp4");
    map.insert("webm", "video/webm");
    map.insert("avi", "video/x-msvideo");
    map.insert("mov", "video/quicktime");
    map.insert("mkv", "video/x-matroska");
    map.insert("flv", "video/x-flv");
    map.insert("wmv", "video/x-ms-wmv");
    map.insert("mpeg", "video/mpeg");
    map.insert("mpg", "video/mpeg");
    map.insert("3gp", "video/3gpp");
    map.insert("ts", "video/mp2t");

    // Documents
    map.insert("pdf", "application/pdf");
    map.insert("doc", "application/msword");
    map.insert(
        "docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    );
    map.insert("xls", "application/vnd.ms-excel");
    map.insert(
        "xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    );
    map.insert("ppt", "application/vnd.ms-powerpoint");
    map.insert(
        "pptx",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    );
    map.insert("odt", "application/vnd.oasis.opendocument.text");
    map.insert("ods", "application/vnd.oasis.opendocument.spreadsheet");
    map.insert("odp", "application/vnd.oasis.opendocument.presentation");
    map.insert("rtf", "application/rtf");
    map.insert("epub", "application/epub+zip");
    map.insert("tex", "application/x-tex");

    // Archives
    map.insert("zip", "application/zip");
    map.insert("rar", "application/x-rar-compressed");
    map.insert("7z", "application/x-7z-compressed");
    map.insert("gz", "application/gzip");
    map.insert("tar", "application/x-tar");
    map.insert("bz2", "application/x-bzip2");
    map.insert("xz", "application/x-xz");

    // Code/Text
    map.insert("txt", "text/plain");
    map.insert("csv", "text/csv");
    map.insert("html", "text/html");
    map.insert("htm", "text/html");
    map.insert("css", "text/css");
    map.insert("js", "text/javascript");
    map.insert("json", "application/json");
    map.insert("xml", "application/xml");
    map.insert("yaml", "application/yaml");
    map.insert("yml", "application/yaml");
    map.insert("md", "text/markdown");
    map.insert("py", "text/x-python");
    map.insert("rs", "text/x-rust");
    map.insert("java", "text/x-java");
    map.insert("c", "text/x-c");
    map.insert("cpp", "text/x-c++");
    map.insert("h", "text/x-c");
    map.insert("go", "text/x-go");
    map.insert("rb", "text/x-ruby");
    map.insert("php", "text/x-php");
    map.insert("sh", "text/x-shellscript");

    // Binary/Other
    map.insert("bin", "application/octet-stream");
    map.insert("exe", "application/octet-stream");
    map.insert("dll", "application/octet-stream");
    map.insert("so", "application/octet-stream");
    map.insert("dmg", "application/x-apple-diskimage");
    map.insert("iso", "application/x-iso9660-image");

    map
});

/// MIME type to extension mapping
static MIME_TO_EXT: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Images
    map.insert("image/jpeg", "jpg");
    map.insert("image/png", "png");
    map.insert("image/gif", "gif");
    map.insert("image/webp", "webp");
    map.insert("image/bmp", "bmp");
    map.insert("image/tiff", "tiff");
    map.insert("image/svg+xml", "svg");
    map.insert("image/x-icon", "ico");
    map.insert("image/heic", "heic");
    map.insert("image/heif", "heif");
    map.insert("image/avif", "avif");
    map.insert("image/jxl", "jxl");
    map.insert("image/vnd.adobe.photoshop", "psd");

    // Audio
    map.insert("audio/mpeg", "mp3");
    map.insert("audio/wav", "wav");
    map.insert("audio/ogg", "ogg");
    map.insert("audio/flac", "flac");
    map.insert("audio/mp4", "m4a");
    map.insert("audio/aac", "aac");
    map.insert("audio/midi", "mid");
    map.insert("audio/aiff", "aiff");

    // Video
    map.insert("video/mp4", "mp4");
    map.insert("video/webm", "webm");
    map.insert("video/x-msvideo", "avi");
    map.insert("video/quicktime", "mov");
    map.insert("video/x-matroska", "mkv");
    map.insert("video/x-flv", "flv");
    map.insert("video/mpeg", "mpeg");
    map.insert("video/3gpp", "3gp");

    // Documents
    map.insert("application/pdf", "pdf");
    map.insert("application/msword", "doc");
    map.insert(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "docx",
    );
    map.insert("application/vnd.ms-excel", "xls");
    map.insert(
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xlsx",
    );
    map.insert("application/vnd.ms-powerpoint", "ppt");
    map.insert(
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "pptx",
    );
    map.insert("application/vnd.oasis.opendocument.text", "odt");
    map.insert("application/vnd.oasis.opendocument.spreadsheet", "ods");
    map.insert("application/vnd.oasis.opendocument.presentation", "odp");
    map.insert("application/rtf", "rtf");
    map.insert("application/epub+zip", "epub");

    // Archives
    map.insert("application/zip", "zip");
    map.insert("application/x-rar-compressed", "rar");
    map.insert("application/x-7z-compressed", "7z");
    map.insert("application/gzip", "gz");
    map.insert("application/x-tar", "tar");
    map.insert("application/x-bzip2", "bz2");

    // Text/Code
    map.insert("text/plain", "txt");
    map.insert("text/csv", "csv");
    map.insert("text/html", "html");
    map.insert("text/css", "css");
    map.insert("text/javascript", "js");
    map.insert("application/json", "json");
    map.insert("application/xml", "xml");
    map.insert("application/yaml", "yaml");
    map.insert("text/markdown", "md");

    map.insert("application/octet-stream", "bin");

    map
});

/// MIME type detector
#[derive(Debug, Clone, Default)]
pub struct MimeDetector;

impl MimeDetector {
    /// Create a new MIME detector
    pub fn new() -> Self {
        Self
    }

    /// Detect MIME type from file magic bytes
    ///
    /// # Arguments
    /// * `data` - File data bytes (at least first few bytes needed)
    ///
    /// # Returns
    /// (MIME type, extension) tuple if detected, None otherwise
    pub fn detect(data: &[u8]) -> Option<(&'static str, &'static str)> {
        if data.is_empty() {
            return None;
        }

        for pattern in MAGIC_PATTERNS.iter() {
            if Self::matches_pattern(data, pattern) {
                return Some((pattern.mime_type, pattern.extension));
            }
        }

        // Special handling for SVG (check content)
        if Self::is_svg(data) {
            return Some(("image/svg+xml", "svg"));
        }

        // Special handling for text files
        if Self::is_text(data) {
            return Some(("text/plain", "txt"));
        }

        None
    }

    /// Detect MIME type from file path by reading first bytes
    ///
    /// # Arguments
    /// * `path` - File path
    ///
    /// # Returns
    /// (MIME type, extension) tuple if detected
    pub async fn detect_from_file(path: impl AsRef<Path>) -> Result<Option<(String, String)>> {
        let path = path.as_ref();

        // Try to read first 8192 bytes for magic detection
        let data = match tokio::fs::read(path).await {
            Ok(data) => data,
            Err(e) => {
                return Err(AgentError::platform(format!("Failed to read file: {}", e)).into())
            }
        };

        // Try magic detection first
        if let Some((mime, ext)) = Self::detect(&data) {
            return Ok(Some((mime.to_string(), ext.to_string())));
        }

        // Fall back to extension-based detection
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(mime) = Self::from_extension(ext) {
                return Ok(Some((mime.to_string(), ext.to_string())));
            }
        }

        Ok(None)
    }

    /// Get MIME type from file extension
    ///
    /// # Arguments
    /// * `extension` - File extension (with or without dot)
    ///
    /// # Returns
    /// MIME type string if known
    pub fn from_extension(extension: impl AsRef<str>) -> Option<&'static str> {
        let ext = extension.as_ref().trim_start_matches('.').to_lowercase();
        EXT_TO_MIME.get(ext.as_str()).copied()
    }

    /// Get file extension from MIME type
    ///
    /// # Arguments
    /// * `mime_type` - MIME type string
    ///
    /// # Returns
    /// File extension if known
    pub fn to_extension(mime_type: impl AsRef<str>) -> Option<&'static str> {
        let mime = mime_type.as_ref().to_lowercase();

        // Handle MIME types with parameters (e.g., "text/plain; charset=utf-8")
        let mime = mime.split(';').next().unwrap_or(&mime).trim();

        MIME_TO_EXT.get(mime).copied()
    }

    /// Check if MIME type is an image
    ///
    /// # Arguments
    /// * `mime_type` - MIME type to check
    ///
    /// # Returns
    /// True if image type
    pub fn is_image(mime_type: impl AsRef<str>) -> bool {
        mime_type.as_ref().to_lowercase().starts_with("image/")
    }

    /// Check if MIME type is audio
    ///
    /// # Arguments
    /// * `mime_type` - MIME type to check
    ///
    /// # Returns
    /// True if audio type
    pub fn is_audio(mime_type: impl AsRef<str>) -> bool {
        mime_type.as_ref().to_lowercase().starts_with("audio/")
    }

    /// Check if MIME type is video
    ///
    /// # Arguments
    /// * `mime_type` - MIME type to check
    ///
    /// # Returns
    /// True if video type
    pub fn is_video(mime_type: impl AsRef<str>) -> bool {
        mime_type.as_ref().to_lowercase().starts_with("video/")
    }

    /// Check if MIME type is a document
    ///
    /// # Arguments
    /// * `mime_type` - MIME type to check
    ///
    /// # Returns
    /// True if document type
    pub fn is_document(mime_type: impl AsRef<str>) -> bool {
        let mime = mime_type.as_ref().to_lowercase();
        mime.starts_with("application/pdf")
            || mime.starts_with("application/msword")
            || mime.starts_with("application/vnd.openxmlformats-officedocument")
            || mime.starts_with("application/vnd.oasis.opendocument")
            || mime == "application/rtf"
            // Note: text/plain, text/html, text/markdown are handled as "text" category
            // but still considered documents for type checking purposes
            || mime == "text/plain"
            || mime == "text/html"
            || mime == "text/markdown"
    }

    /// Check if MIME type is an archive
    ///
    /// # Arguments
    /// * `mime_type` - MIME type to check
    ///
    /// # Returns
    /// True if archive type
    pub fn is_archive(mime_type: impl AsRef<str>) -> bool {
        let mime = mime_type.as_ref().to_lowercase();
        mime.starts_with("application/zip")
            || mime.starts_with("application/x-rar")
            || mime.starts_with("application/x-7z")
            || mime.starts_with("application/gzip")
            || mime.starts_with("application/x-tar")
            || mime.starts_with("application/x-bzip")
    }

    /// Get category for MIME type
    ///
    /// # Arguments
    /// * `mime_type` - MIME type
    ///
    /// # Returns
    /// Category string (image, audio, video, document, archive, text, other)
    pub fn get_category(mime_type: impl AsRef<str>) -> &'static str {
        let mime = mime_type.as_ref();
        if Self::is_image(mime) {
            "image"
        } else if Self::is_audio(mime) {
            "audio"
        } else if Self::is_video(mime) {
            "video"
        } else if mime.starts_with("text/") {
            // Check text/ before is_document to ensure text/* types are categorized as text
            "text"
        } else if Self::is_document(mime) {
            "document"
        } else if Self::is_archive(mime) {
            "archive"
        } else {
            "other"
        }
    }

    /// Check if data matches a magic pattern
    fn matches_pattern(data: &[u8], pattern: &MagicPattern) -> bool {
        let start = pattern.offset;
        let end = start + pattern.bytes.len();

        if data.len() < end {
            return false;
        }

        &data[start..end] == pattern.bytes.as_slice()
    }

    /// Check if data is likely an SVG
    fn is_svg(data: &[u8]) -> bool {
        // Check if it starts with XML declaration or <svg
        if data.starts_with(b"<?xml") || data.starts_with(b"<svg") {
            // Check if content contains SVG elements
            if let Ok(text) = std::str::from_utf8(&data[..data.len().min(4096)]) {
                return text.contains("<svg") || text.contains("xmlns=");
            }
        }
        false
    }

    /// Check if data is likely text
    fn is_text(data: &[u8]) -> bool {
        // Check for null bytes (binary files usually contain them)
        if data.contains(&0) {
            return false;
        }

        // Check if valid UTF-8
        std::str::from_utf8(data).is_ok()
    }

    /// Get all supported extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        EXT_TO_MIME.keys().copied().collect()
    }

    /// Get all supported MIME types
    pub fn supported_mime_types() -> Vec<&'static str> {
        MIME_TO_EXT.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_jpeg() {
        let data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        let result = MimeDetector::detect(&data);
        assert_eq!(result, Some(("image/jpeg", "jpg")));
    }

    #[test]
    fn test_detect_png() {
        let data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let result = MimeDetector::detect(&data);
        assert_eq!(result, Some(("image/png", "png")));
    }

    #[test]
    fn test_detect_gif() {
        let data = b"GIF89ahello";
        let result = MimeDetector::detect(&data[..]);
        assert_eq!(result, Some(("image/gif", "gif")));
    }

    #[test]
    fn test_detect_pdf() {
        let data = b"%PDF-1.4";
        let result = MimeDetector::detect(&data[..]);
        assert_eq!(result, Some(("application/pdf", "pdf")));
    }

    #[test]
    fn test_detect_zip() {
        let data = vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00];
        let result = MimeDetector::detect(&data);
        assert_eq!(result, Some(("application/zip", "zip")));
    }

    #[test]
    fn test_detect_mp3() {
        let data = vec![0x49, 0x44, 0x33, 0x04];
        let result = MimeDetector::detect(&data);
        assert_eq!(result, Some(("audio/mpeg", "mp3")));
    }

    #[test]
    fn test_from_extension() {
        assert_eq!(MimeDetector::from_extension("jpg"), Some("image/jpeg"));
        assert_eq!(MimeDetector::from_extension(".png"), Some("image/png"));
        assert_eq!(MimeDetector::from_extension("PDF"), Some("application/pdf"));
        assert_eq!(MimeDetector::from_extension("unknown"), None);
    }

    #[test]
    fn test_to_extension() {
        assert_eq!(MimeDetector::to_extension("image/jpeg"), Some("jpg"));
        assert_eq!(MimeDetector::to_extension("image/png"), Some("png"));
        assert_eq!(MimeDetector::to_extension("application/pdf"), Some("pdf"));
        assert_eq!(MimeDetector::to_extension("unknown/type"), None);
    }

    #[test]
    fn test_to_extension_with_params() {
        assert_eq!(
            MimeDetector::to_extension("text/plain; charset=utf-8"),
            Some("txt")
        );
    }

    #[test]
    fn test_is_image() {
        assert!(MimeDetector::is_image("image/jpeg"));
        assert!(MimeDetector::is_image("IMAGE/PNG"));
        assert!(!MimeDetector::is_image("application/pdf"));
    }

    #[test]
    fn test_is_audio() {
        assert!(MimeDetector::is_audio("audio/mpeg"));
        assert!(!MimeDetector::is_audio("video/mp4"));
    }

    #[test]
    fn test_is_video() {
        assert!(MimeDetector::is_video("video/mp4"));
        assert!(!MimeDetector::is_video("audio/mpeg"));
    }

    #[test]
    fn test_is_document() {
        assert!(MimeDetector::is_document("application/pdf"));
        assert!(MimeDetector::is_document("text/plain"));
        assert!(!MimeDetector::is_document("image/jpeg"));
    }

    #[test]
    fn test_is_archive() {
        assert!(MimeDetector::is_archive("application/zip"));
        assert!(MimeDetector::is_archive("application/x-rar-compressed"));
        assert!(!MimeDetector::is_archive("image/jpeg"));
    }

    #[test]
    fn test_get_category() {
        assert_eq!(MimeDetector::get_category("image/jpeg"), "image");
        assert_eq!(MimeDetector::get_category("audio/mpeg"), "audio");
        assert_eq!(MimeDetector::get_category("video/mp4"), "video");
        assert_eq!(MimeDetector::get_category("application/pdf"), "document");
        assert_eq!(MimeDetector::get_category("application/zip"), "archive");
        assert_eq!(MimeDetector::get_category("text/plain"), "text");
        assert_eq!(MimeDetector::get_category("unknown/type"), "other");
    }

    #[test]
    fn test_detect_empty() {
        let data: Vec<u8> = vec![];
        assert_eq!(MimeDetector::detect(&data), None);
    }

    #[test]
    fn test_detect_text() {
        let data = b"Hello, World! This is plain text.";
        let result = MimeDetector::detect(&data[..]);
        assert_eq!(result, Some(("text/plain", "txt")));
    }

    #[test]
    fn test_detect_json() {
        let data = b"{\"key\": \"value\"}";
        let result = MimeDetector::detect(&data[..]);
        assert_eq!(result, Some(("application/json", "json")));
    }

    #[test]
    fn test_is_svg() {
        let data = b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\">";
        assert!(MimeDetector::is_svg(data));
    }
}
