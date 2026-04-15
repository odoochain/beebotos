//! Attachment Parser
//!
//! Provides parsing of attachments from various messaging platforms,
//! with special focus on Lark/Feishu attachments including images,
/// files, and embedded images in rich text.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::communication::channel::lark_content::{
    LarkContent, LarkContentParser, LarkFileContent, LarkImageContent,
};
use crate::communication::channel::matrix_content::{
    MatrixContent, MatrixFileContent, MatrixImageContent,
};
use crate::communication::channel::telegram_content::{
    TelegramAudioContent, TelegramContent, TelegramDocumentContent, TelegramPhotoContent,
    TelegramVideoContent, TelegramVoiceContent,
};
use crate::error::{AgentError, Result};
use crate::media::downloader::MediaType;

/// Parsed attachment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedAttachment {
    /// Unique identifier for the attachment
    pub id: String,
    /// Attachment type
    pub attachment_type: AttachmentType,
    /// File key or URL for downloading
    pub file_key: Option<String>,
    /// Original URL if available
    pub url: Option<String>,
    /// Original filename
    pub file_name: Option<String>,
    /// File size in bytes (if known)
    pub file_size: Option<u64>,
    /// MIME type (if known)
    pub mime_type: Option<String>,
    /// File extension
    pub extension: Option<String>,
    /// Source platform
    pub source_platform: PlatformSource,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Attachment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentType {
    /// Image attachment
    Image,
    /// Generic file attachment
    File,
    /// Audio/voice attachment
    Audio,
    /// Video attachment
    Video,
    /// Sticker/image attachment
    Sticker,
    /// Embedded image in rich text
    EmbeddedImage,
    /// Document attachment
    Document,
    /// Archive attachment
    Archive,
    /// Unknown type
    Unknown,
}

impl AttachmentType {
    /// Convert to media type for downloader
    pub fn to_media_type(&self) -> MediaType {
        match self {
            AttachmentType::Image | AttachmentType::EmbeddedImage => MediaType::Image,
            AttachmentType::File | AttachmentType::Document | AttachmentType::Archive => {
                MediaType::File
            }
            AttachmentType::Audio => MediaType::Voice,
            AttachmentType::Video => MediaType::Video,
            AttachmentType::Sticker => MediaType::Sticker,
            AttachmentType::Unknown => MediaType::File,
        }
    }

    /// Get from MIME type
    pub fn from_mime_type(mime_type: &str) -> Self {
        let mime = mime_type.to_lowercase();
        if mime.starts_with("image/") {
            AttachmentType::Image
        } else if mime.starts_with("audio/") {
            AttachmentType::Audio
        } else if mime.starts_with("video/") {
            AttachmentType::Video
        } else if mime.contains("pdf")
            || mime.contains("word")
            || mime.contains("excel")
            || mime.contains("powerpoint")
            || mime.contains("text/")
        {
            AttachmentType::Document
        } else if mime.contains("zip")
            || mime.contains("rar")
            || mime.contains("7z")
            || mime.contains("tar")
            || mime.contains("gzip")
        {
            AttachmentType::Archive
        } else {
            AttachmentType::File
        }
    }

    /// Get from file extension
    pub fn from_extension(extension: &str) -> Self {
        let ext = extension.trim_start_matches('.').to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "svg" | "heic" | "avif"
            | "jxl" => AttachmentType::Image,
            "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac" | "wma" => AttachmentType::Audio,
            "mp4" | "webm" | "avi" | "mov" | "mkv" | "flv" | "wmv" | "mpeg" => {
                AttachmentType::Video
            }
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "csv" | "rtf"
            | "odt" | "ods" | "odp" | "md" => AttachmentType::Document,
            "zip" | "rar" | "7z" | "gz" | "tar" | "bz2" | "xz" => AttachmentType::Archive,
            _ => AttachmentType::File,
        }
    }
}

impl std::fmt::Display for AttachmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttachmentType::Image => write!(f, "image"),
            AttachmentType::File => write!(f, "file"),
            AttachmentType::Audio => write!(f, "audio"),
            AttachmentType::Video => write!(f, "video"),
            AttachmentType::Sticker => write!(f, "sticker"),
            AttachmentType::EmbeddedImage => write!(f, "embedded_image"),
            AttachmentType::Document => write!(f, "document"),
            AttachmentType::Archive => write!(f, "archive"),
            AttachmentType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Platform source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformSource {
    /// Lark/Feishu platform
    Lark,
    /// DingTalk platform
    DingTalk,
    /// Slack platform
    Slack,
    /// Discord platform
    Discord,
    /// Telegram platform
    Telegram,
    /// Matrix platform
    Matrix,
    /// Microsoft Teams platform
    Teams,
    /// Generic/unknown platform
    Generic,
}

impl std::fmt::Display for PlatformSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformSource::Lark => write!(f, "lark"),
            PlatformSource::DingTalk => write!(f, "dingtalk"),
            PlatformSource::Slack => write!(f, "slack"),
            PlatformSource::Discord => write!(f, "discord"),
            PlatformSource::Telegram => write!(f, "telegram"),
            PlatformSource::Matrix => write!(f, "matrix"),
            PlatformSource::Teams => write!(f, "teams"),
            PlatformSource::Generic => write!(f, "generic"),
        }
    }
}

/// Attachment parser for extracting attachments from messages
#[derive(Debug, Clone, Default)]
pub struct AttachmentParser;

impl AttachmentParser {
    /// Create a new attachment parser
    pub fn new() -> Self {
        Self
    }

    /// Parse Lark attachment from message content
    ///
    /// # Arguments
    /// * `content` - LarkContent to parse
    ///
    /// # Returns
    /// List of parsed attachments
    pub fn parse_lark_attachment(content: &LarkContent) -> Vec<ParsedAttachment> {
        let mut attachments = Vec::new();

        match content {
            LarkContent::Image(image) => {
                if let Some(attachment) = Self::parse_lark_image(image) {
                    attachments.push(attachment);
                }
            }
            LarkContent::File(file) => {
                if let Some(attachment) = Self::parse_lark_file(file) {
                    attachments.push(attachment);
                }
            }
            LarkContent::Audio(audio) => {
                let attachment = ParsedAttachment {
                    id: format!("lark_audio_{}", audio.file_key),
                    attachment_type: AttachmentType::Audio,
                    file_key: Some(audio.file_key.clone()),
                    url: None,
                    file_name: Some(format!("audio_{}.mp3", audio.file_key)),
                    file_size: None,
                    mime_type: Some("audio/mpeg".to_string()),
                    extension: Some("mp3".to_string()),
                    source_platform: PlatformSource::Lark,
                    metadata: {
                        let mut m = HashMap::new();
                        if let Some(duration) = audio.duration {
                            m.insert("duration_ms".to_string(), duration.to_string());
                        }
                        m
                    },
                };
                attachments.push(attachment);
            }
            LarkContent::Sticker(sticker) => {
                let attachment = ParsedAttachment {
                    id: format!("lark_sticker_{}", sticker.file_key),
                    attachment_type: AttachmentType::Sticker,
                    file_key: Some(sticker.file_key.clone()),
                    url: None,
                    file_name: Some(format!("sticker_{}.webp", sticker.file_key)),
                    file_size: None,
                    mime_type: Some("image/webp".to_string()),
                    extension: Some("webp".to_string()),
                    source_platform: PlatformSource::Lark,
                    metadata: HashMap::new(),
                };
                attachments.push(attachment);
            }
            LarkContent::Post(post) => {
                // Parse embedded images from post content
                let embedded = Self::parse_post_embedded_images(post);
                attachments.extend(embedded);
            }
            _ => {
                // Other content types don't have attachments
                debug!("Content type has no attachments");
            }
        }

        attachments
    }

    /// Parse Lark image content
    fn parse_lark_image(image: &LarkImageContent) -> Option<ParsedAttachment> {
        if image.image_key.is_empty() {
            warn!("Lark image has empty image_key");
            return None;
        }

        Some(ParsedAttachment {
            id: format!("lark_img_{}", image.image_key),
            attachment_type: AttachmentType::Image,
            file_key: Some(image.image_key.clone()),
            url: None,
            file_name: Some(format!("image_{}.jpg", image.image_key)),
            file_size: None,
            mime_type: Some("image/jpeg".to_string()),
            extension: Some("jpg".to_string()),
            source_platform: PlatformSource::Lark,
            metadata: HashMap::new(),
        })
    }

    /// Parse Lark file content
    fn parse_lark_file(file: &LarkFileContent) -> Option<ParsedAttachment> {
        if file.file_key.is_empty() {
            warn!("Lark file has empty file_key");
            return None;
        }

        let file_name = file
            .file_name
            .clone()
            .unwrap_or_else(|| format!("file_{}", file.file_key));

        let extension = Self::extract_extension(&file_name);
        let attachment_type = AttachmentType::from_extension(&extension);
        let mime_type = Self::guess_mime_type(&extension);

        Some(ParsedAttachment {
            id: format!("lark_file_{}", file.file_key),
            attachment_type,
            file_key: Some(file.file_key.clone()),
            url: None,
            file_name: Some(file_name),
            file_size: None,
            mime_type: Some(mime_type),
            extension: Some(extension),
            source_platform: PlatformSource::Lark,
            metadata: HashMap::new(),
        })
    }

    /// Parse embedded images from post content
    fn parse_post_embedded_images(
        post: &crate::communication::channel::lark_content::LarkPostContent,
    ) -> Vec<ParsedAttachment> {
        let mut attachments = Vec::new();

        // Parse post content structure
        if let Some(content) = post.content.get("content") {
            if let Some(lines) = content.as_array() {
                for (line_idx, line) in lines.iter().enumerate() {
                    if let Some(elements) = line.as_array() {
                        for (elem_idx, elem) in elements.iter().enumerate() {
                            if let Some(tag) = elem.get("tag").and_then(|t| t.as_str()) {
                                if tag == "img" {
                                    if let Some(image_key) =
                                        elem.get("image_key").and_then(|k| k.as_str())
                                    {
                                        let width = elem
                                            .get("width")
                                            .and_then(|w| w.as_i64())
                                            .map(|w| w as i32);
                                        let height = elem
                                            .get("height")
                                            .and_then(|h| h.as_i64())
                                            .map(|h| h as i32);

                                        let mut metadata = HashMap::new();
                                        if let Some(w) = width {
                                            metadata.insert("width".to_string(), w.to_string());
                                        }
                                        if let Some(h) = height {
                                            metadata.insert("height".to_string(), h.to_string());
                                        }
                                        metadata.insert("line".to_string(), line_idx.to_string());
                                        metadata
                                            .insert("position".to_string(), elem_idx.to_string());

                                        let attachment = ParsedAttachment {
                                            id: format!(
                                                "lark_post_img_{}_{}_{}",
                                                image_key, line_idx, elem_idx
                                            ),
                                            attachment_type: AttachmentType::EmbeddedImage,
                                            file_key: Some(image_key.to_string()),
                                            url: None,
                                            file_name: Some(format!(
                                                "embedded_image_{}.jpg",
                                                image_key
                                            )),
                                            file_size: None,
                                            mime_type: Some("image/jpeg".to_string()),
                                            extension: Some("jpg".to_string()),
                                            source_platform: PlatformSource::Lark,
                                            metadata,
                                        };
                                        attachments.push(attachment);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !attachments.is_empty() {
            info!(
                "Found {} embedded images in post content",
                attachments.len()
            );
        }

        attachments
    }

    /// Parse attachments from raw Lark message JSON
    ///
    /// # Arguments
    /// * `msg_type` - Message type string
    /// * `content_json` - Content JSON string
    ///
    /// # Returns
    /// List of parsed attachments
    pub fn parse_lark_message(msg_type: &str, content_json: &str) -> Result<Vec<ParsedAttachment>> {
        let content = LarkContentParser::parse_str(msg_type, content_json)
            .map_err(|e| AgentError::platform(format!("Failed to parse Lark content: {}", e)))?;

        Ok(Self::parse_lark_attachment(&content))
    }

    /// Parse generic attachment from URL
    ///
    /// # Arguments
    /// * `url` - Attachment URL
    /// * `file_name` - Optional filename
    ///
    /// # Returns
    /// Parsed attachment
    pub fn parse_from_url(url: impl Into<String>, file_name: Option<String>) -> ParsedAttachment {
        let url = url.into();
        let extracted_name = file_name.or_else(|| Self::extract_filename_from_url(&url));
        let extension = extracted_name
            .as_ref()
            .map(|n| Self::extract_extension(n))
            .unwrap_or_default();

        let attachment_type = AttachmentType::from_extension(&extension);
        let mime_type = Self::guess_mime_type(&extension);

        ParsedAttachment {
            id: format!("url_{}", uuid::Uuid::new_v4().simple()),
            attachment_type,
            file_key: None,
            url: Some(url),
            file_name: extracted_name,
            file_size: None,
            mime_type: Some(mime_type),
            extension: Some(extension),
            source_platform: PlatformSource::Generic,
            metadata: HashMap::new(),
        }
    }

    /// Batch parse multiple attachments
    ///
    /// # Arguments
    /// * `attachments` - List of (content, msg_type) tuples
    ///
    /// # Returns
    /// List of parsed attachments for each input
    pub fn batch_parse_lark(
        attachments: &[(String, String)],
    ) -> Vec<Result<Vec<ParsedAttachment>>> {
        attachments
            .iter()
            .map(|(msg_type, content)| Self::parse_lark_message(msg_type, content))
            .collect()
    }

    /// Filter attachments by type
    ///
    /// # Arguments
    /// * `attachments` - List of attachments
    /// * `attachment_type` - Type to filter by
    ///
    /// # Returns
    /// Filtered list
    pub fn filter_by_type(
        attachments: &[ParsedAttachment],
        attachment_type: AttachmentType,
    ) -> Vec<ParsedAttachment> {
        attachments
            .iter()
            .filter(|a| a.attachment_type == attachment_type)
            .cloned()
            .collect()
    }

    /// Get all image attachments (including embedded)
    ///
    /// # Arguments
    /// * `attachments` - List of attachments
    ///
    /// # Returns
    /// List of image attachments
    pub fn get_all_images(attachments: &[ParsedAttachment]) -> Vec<ParsedAttachment> {
        attachments
            .iter()
            .filter(|a| {
                a.attachment_type == AttachmentType::Image
                    || a.attachment_type == AttachmentType::EmbeddedImage
                    || a.attachment_type == AttachmentType::Sticker
            })
            .cloned()
            .collect()
    }

    /// Check if content has any attachments
    ///
    /// # Arguments
    /// * `content` - LarkContent to check
    ///
    /// # Returns
    /// True if has attachments
    pub fn has_attachments(content: &LarkContent) -> bool {
        !Self::parse_lark_attachment(content).is_empty()
    }

    /// Count attachments in content
    ///
    /// # Arguments
    /// * `content` - LarkContent to count
    ///
    /// # Returns
    /// Number of attachments
    pub fn count_attachments(content: &LarkContent) -> usize {
        Self::parse_lark_attachment(content).len()
    }

    /// Parse Matrix attachment from message content
    ///
    /// # Arguments
    /// * `content` - MatrixContent to parse
    ///
    /// # Returns
    /// List of parsed attachments
    pub fn parse_matrix_attachment(content: &MatrixContent) -> Vec<ParsedAttachment> {
        let mut attachments = Vec::new();

        match content {
            MatrixContent::Image(image) => {
                if let Some(attachment) = Self::parse_matrix_image(image) {
                    attachments.push(attachment);
                }
            }
            MatrixContent::File(file) => {
                if let Some(attachment) = Self::parse_matrix_file(file) {
                    attachments.push(attachment);
                }
            }
            MatrixContent::Audio(audio) => {
                if let Some(attachment) = Self::parse_matrix_audio(audio) {
                    attachments.push(attachment);
                }
            }
            MatrixContent::Video(video) => {
                if let Some(attachment) = Self::parse_matrix_video(video) {
                    attachments.push(attachment);
                }
            }
            _ => {
                debug!("Matrix content type has no attachments");
            }
        }

        attachments
    }

    /// Parse Matrix image content
    fn parse_matrix_image(image: &MatrixImageContent) -> Option<ParsedAttachment> {
        if image.url.is_empty() {
            warn!("Matrix image has empty URL");
            return None;
        }

        let mime_type = image.info.as_ref().and_then(|i| i.mimetype.clone());
        let file_size = image.info.as_ref().and_then(|i| i.size);

        Some(ParsedAttachment {
            id: format!(
                "matrix_img_{}",
                image.url.replace("mxc://", "").replace("/", "_")
            ),
            attachment_type: AttachmentType::Image,
            file_key: Some(image.url.clone()),
            url: None,
            file_name: Some(format!(
                "image_{}.jpg",
                image.url.split('/').last().unwrap_or("unknown")
            )),
            file_size,
            mime_type: mime_type.or_else(|| Some("image/jpeg".to_string())),
            extension: Some("jpg".to_string()),
            source_platform: PlatformSource::Matrix,
            metadata: {
                let mut m = HashMap::new();
                if let Some(info) = &image.info {
                    if let Some(w) = info.w {
                        m.insert("width".to_string(), w.to_string());
                    }
                    if let Some(h) = info.h {
                        m.insert("height".to_string(), h.to_string());
                    }
                }
                m
            },
        })
    }

    /// Parse Matrix file content
    fn parse_matrix_file(file: &MatrixFileContent) -> Option<ParsedAttachment> {
        if file.url.is_empty() {
            warn!("Matrix file has empty URL");
            return None;
        }

        let file_name = file
            .filename
            .clone()
            .unwrap_or_else(|| format!("file_{}", file.url.split('/').last().unwrap_or("unknown")));

        let extension = Self::extract_extension(&file_name);
        let attachment_type = AttachmentType::from_extension(&extension);
        let mime_type = file
            .info
            .as_ref()
            .and_then(|i| i.mimetype.clone())
            .unwrap_or_else(|| Self::guess_mime_type(&extension));

        Some(ParsedAttachment {
            id: format!(
                "matrix_file_{}",
                file.url.replace("mxc://", "").replace("/", "_")
            ),
            attachment_type,
            file_key: Some(file.url.clone()),
            url: None,
            file_name: Some(file_name),
            file_size: file.info.as_ref().and_then(|i| i.size),
            mime_type: Some(mime_type),
            extension: Some(extension),
            source_platform: PlatformSource::Matrix,
            metadata: HashMap::new(),
        })
    }

    /// Parse Matrix audio content
    fn parse_matrix_audio(
        audio: &crate::communication::channel::matrix_content::MatrixAudioContent,
    ) -> Option<ParsedAttachment> {
        if audio.url.is_empty() {
            warn!("Matrix audio has empty URL");
            return None;
        }

        Some(ParsedAttachment {
            id: format!(
                "matrix_audio_{}",
                audio.url.replace("mxc://", "").replace("/", "_")
            ),
            attachment_type: AttachmentType::Audio,
            file_key: Some(audio.url.clone()),
            url: None,
            file_name: Some(format!(
                "audio_{}.ogg",
                audio.url.split('/').last().unwrap_or("unknown")
            )),
            file_size: audio.info.as_ref().and_then(|i| i.size),
            mime_type: audio
                .info
                .as_ref()
                .and_then(|i| i.mimetype.clone())
                .or_else(|| Some("audio/ogg".to_string())),
            extension: Some("ogg".to_string()),
            source_platform: PlatformSource::Matrix,
            metadata: {
                let mut m = HashMap::new();
                if let Some(info) = &audio.info {
                    if let Some(duration) = info.duration {
                        m.insert("duration_ms".to_string(), duration.to_string());
                    }
                }
                m
            },
        })
    }

    /// Parse Matrix video content
    fn parse_matrix_video(
        video: &crate::communication::channel::matrix_content::MatrixVideoContent,
    ) -> Option<ParsedAttachment> {
        if video.url.is_empty() {
            warn!("Matrix video has empty URL");
            return None;
        }

        Some(ParsedAttachment {
            id: format!(
                "matrix_video_{}",
                video.url.replace("mxc://", "").replace("/", "_")
            ),
            attachment_type: AttachmentType::Video,
            file_key: Some(video.url.clone()),
            url: None,
            file_name: Some(format!(
                "video_{}.mp4",
                video.url.split('/').last().unwrap_or("unknown")
            )),
            file_size: video.info.as_ref().and_then(|i| i.size),
            mime_type: video
                .info
                .as_ref()
                .and_then(|i| i.mimetype.clone())
                .or_else(|| Some("video/mp4".to_string())),
            extension: Some("mp4".to_string()),
            source_platform: PlatformSource::Matrix,
            metadata: {
                let mut m = HashMap::new();
                if let Some(info) = &video.info {
                    if let Some(duration) = info.duration {
                        m.insert("duration_ms".to_string(), duration.to_string());
                    }
                    if let Some(w) = info.w {
                        m.insert("width".to_string(), w.to_string());
                    }
                    if let Some(h) = info.h {
                        m.insert("height".to_string(), h.to_string());
                    }
                }
                m
            },
        })
    }

    /// Parse Telegram attachment from message content
    ///
    /// # Arguments
    /// * `content` - TelegramContent to parse
    ///
    /// # Returns
    /// List of parsed attachments
    pub fn parse_telegram_attachment(content: &TelegramContent) -> Vec<ParsedAttachment> {
        let mut attachments = Vec::new();

        match content {
            TelegramContent::Photo(photo) => {
                if let Some(attachment) = Self::parse_telegram_photo(photo) {
                    attachments.push(attachment);
                }
            }
            TelegramContent::Video(video) => {
                if let Some(attachment) = Self::parse_telegram_video(video) {
                    attachments.push(attachment);
                }
            }
            TelegramContent::Audio(audio) => {
                if let Some(attachment) = Self::parse_telegram_audio(audio) {
                    attachments.push(attachment);
                }
            }
            TelegramContent::Voice(voice) => {
                if let Some(attachment) = Self::parse_telegram_voice(voice) {
                    attachments.push(attachment);
                }
            }
            TelegramContent::Document(document) => {
                if let Some(attachment) = Self::parse_telegram_document(document) {
                    attachments.push(attachment);
                }
            }
            _ => {
                debug!("Telegram content type has no attachments");
            }
        }

        attachments
    }

    /// Parse Telegram photo content
    fn parse_telegram_photo(photo: &TelegramPhotoContent) -> Option<ParsedAttachment> {
        // Get the largest photo size
        let largest = photo
            .photos
            .iter()
            .max_by_key(|p| p.file_size.unwrap_or(0))?;

        Some(ParsedAttachment {
            id: format!("tg_photo_{}", largest.file_unique_id),
            attachment_type: AttachmentType::Image,
            file_key: Some(largest.file_id.clone()),
            url: None,
            file_name: Some(format!("photo_{}.jpg", largest.file_unique_id)),
            file_size: largest.file_size.map(|s| s as u64),
            mime_type: Some("image/jpeg".to_string()),
            extension: Some("jpg".to_string()),
            source_platform: PlatformSource::Telegram,
            metadata: {
                let mut m = HashMap::new();
                m.insert("width".to_string(), largest.width.to_string());
                m.insert("height".to_string(), largest.height.to_string());
                if let Some(caption) = &photo.caption {
                    m.insert("caption".to_string(), caption.clone());
                }
                m
            },
        })
    }

    /// Parse Telegram video content
    fn parse_telegram_video(video: &TelegramVideoContent) -> Option<ParsedAttachment> {
        Some(ParsedAttachment {
            id: format!("tg_video_{}", video.video.file_unique_id),
            attachment_type: AttachmentType::Video,
            file_key: Some(video.video.file_id.clone()),
            url: None,
            file_name: video
                .video
                .file_name
                .clone()
                .or_else(|| Some(format!("video_{}.mp4", video.video.file_unique_id))),
            file_size: video.video.file_size.map(|s| s as u64),
            mime_type: video
                .video
                .mime_type
                .clone()
                .or_else(|| Some("video/mp4".to_string())),
            extension: Some("mp4".to_string()),
            source_platform: PlatformSource::Telegram,
            metadata: {
                let mut m = HashMap::new();
                m.insert("duration".to_string(), video.video.duration.to_string());
                m.insert("width".to_string(), video.video.width.to_string());
                m.insert("height".to_string(), video.video.height.to_string());
                if let Some(caption) = &video.caption {
                    m.insert("caption".to_string(), caption.clone());
                }
                m
            },
        })
    }

    /// Parse Telegram audio content
    fn parse_telegram_audio(audio: &TelegramAudioContent) -> Option<ParsedAttachment> {
        Some(ParsedAttachment {
            id: format!("tg_audio_{}", audio.audio.file_unique_id),
            attachment_type: AttachmentType::Audio,
            file_key: Some(audio.audio.file_id.clone()),
            url: None,
            file_name: audio.audio.file_name.clone().or_else(|| {
                let performer = audio.audio.performer.as_deref().unwrap_or("Unknown");
                let title = audio.audio.title.as_deref().unwrap_or("Unknown");
                Some(format!("{} - {}.mp3", performer, title))
            }),
            file_size: audio.audio.file_size.map(|s| s as u64),
            mime_type: audio
                .audio
                .mime_type
                .clone()
                .or_else(|| Some("audio/mpeg".to_string())),
            extension: Some("mp3".to_string()),
            source_platform: PlatformSource::Telegram,
            metadata: {
                let mut m = HashMap::new();
                m.insert("duration".to_string(), audio.audio.duration.to_string());
                if let Some(performer) = &audio.audio.performer {
                    m.insert("performer".to_string(), performer.clone());
                }
                if let Some(title) = &audio.audio.title {
                    m.insert("title".to_string(), title.clone());
                }
                if let Some(caption) = &audio.caption {
                    m.insert("caption".to_string(), caption.clone());
                }
                m
            },
        })
    }

    /// Parse Telegram voice content
    fn parse_telegram_voice(voice: &TelegramVoiceContent) -> Option<ParsedAttachment> {
        Some(ParsedAttachment {
            id: format!("tg_voice_{}", voice.voice.file_unique_id),
            attachment_type: AttachmentType::Audio,
            file_key: Some(voice.voice.file_id.clone()),
            url: None,
            file_name: Some(format!("voice_{}.ogg", voice.voice.file_unique_id)),
            file_size: voice.voice.file_size.map(|s| s as u64),
            mime_type: voice
                .voice
                .mime_type
                .clone()
                .or_else(|| Some("audio/ogg".to_string())),
            extension: Some("ogg".to_string()),
            source_platform: PlatformSource::Telegram,
            metadata: {
                let mut m = HashMap::new();
                m.insert("duration".to_string(), voice.voice.duration.to_string());
                if let Some(caption) = &voice.caption {
                    m.insert("caption".to_string(), caption.clone());
                }
                m
            },
        })
    }

    /// Parse Telegram document content
    fn parse_telegram_document(document: &TelegramDocumentContent) -> Option<ParsedAttachment> {
        let file_name = document
            .document
            .file_name
            .clone()
            .unwrap_or_else(|| format!("file_{}", document.document.file_unique_id));

        let extension = Self::extract_extension(&file_name);
        let attachment_type = AttachmentType::from_extension(&extension);
        let mime_type = document
            .document
            .mime_type
            .clone()
            .unwrap_or_else(|| Self::guess_mime_type(&extension));

        Some(ParsedAttachment {
            id: format!("tg_doc_{}", document.document.file_unique_id),
            attachment_type,
            file_key: Some(document.document.file_id.clone()),
            url: None,
            file_name: Some(file_name),
            file_size: document.document.file_size.map(|s| s as u64),
            mime_type: Some(mime_type),
            extension: Some(extension),
            source_platform: PlatformSource::Telegram,
            metadata: {
                let mut m = HashMap::new();
                if let Some(caption) = &document.caption {
                    m.insert("caption".to_string(), caption.clone());
                }
                m
            },
        })
    }

    /// Extract extension from filename
    fn extract_extension(filename: &str) -> String {
        std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default()
    }

    /// Extract filename from URL
    fn extract_filename_from_url(url: &str) -> Option<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(segments) = parsed.path_segments() {
                if let Some(last) = segments.last() {
                    if !last.is_empty() {
                        return Some(last.to_string());
                    }
                }
            }
        }
        None
    }

    /// Guess MIME type from extension
    fn guess_mime_type(extension: &str) -> String {
        let ext = extension.trim_start_matches('.').to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "ppt" => "application/vnd.ms-powerpoint",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "txt" => "text/plain",
            "csv" => "text/csv",
            "zip" => "application/zip",
            "mp3" => "audio/mpeg",
            "mp4" => "video/mp4",
            _ => "application/octet-stream",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication::channel::lark_content::{
        LarkAudioContent, LarkFileContent, LarkImageContent, LarkPostContent, LarkStickerContent,
    };

    #[test]
    fn test_parse_lark_image() {
        let image = LarkImageContent {
            image_key: "img_12345".to_string(),
        };
        let content = LarkContent::Image(image);
        let attachments = AttachmentParser::parse_lark_attachment(&content);

        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].attachment_type, AttachmentType::Image);
        assert_eq!(attachments[0].file_key, Some("img_12345".to_string()));
    }

    #[test]
    fn test_parse_lark_file() {
        let file = LarkFileContent {
            file_key: "file_67890".to_string(),
            file_name: Some("document.pdf".to_string()),
        };
        let content = LarkContent::File(file);
        let attachments = AttachmentParser::parse_lark_attachment(&content);

        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].attachment_type, AttachmentType::Document);
        assert_eq!(attachments[0].extension, Some("pdf".to_string()));
    }

    #[test]
    fn test_parse_lark_audio() {
        let audio = LarkAudioContent {
            file_key: "audio_111".to_string(),
            duration: Some(5000),
        };
        let content = LarkContent::Audio(audio);
        let attachments = AttachmentParser::parse_lark_attachment(&content);

        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].attachment_type, AttachmentType::Audio);
        assert_eq!(
            attachments[0].metadata.get("duration_ms"),
            Some(&"5000".to_string())
        );
    }

    #[test]
    fn test_parse_lark_sticker() {
        let sticker = LarkStickerContent {
            file_key: "sticker_222".to_string(),
        };
        let content = LarkContent::Sticker(sticker);
        let attachments = AttachmentParser::parse_lark_attachment(&content);

        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].attachment_type, AttachmentType::Sticker);
    }

    #[test]
    fn test_parse_post_embedded_images() {
        let post = LarkPostContent {
            content: serde_json::json!({
                "content": [
                    [
                        {"tag": "text", "text": "Hello"},
                        {"tag": "img", "image_key": "img_embed_1", "width": 100, "height": 200}
                    ],
                    [
                        {"tag": "img", "image_key": "img_embed_2"}
                    ]
                ]
            }),
        };
        let content = LarkContent::Post(post);
        let attachments = AttachmentParser::parse_lark_attachment(&content);

        assert_eq!(attachments.len(), 2);
        assert_eq!(
            attachments[0].attachment_type,
            AttachmentType::EmbeddedImage
        );
        assert_eq!(attachments[0].file_key, Some("img_embed_1".to_string()));
        assert_eq!(
            attachments[0].metadata.get("width"),
            Some(&"100".to_string())
        );
    }

    #[test]
    fn test_parse_from_url() {
        let attachment = AttachmentParser::parse_from_url("https://example.com/file.pdf", None);

        assert_eq!(attachment.attachment_type, AttachmentType::Document);
        assert_eq!(attachment.file_name, Some("file.pdf".to_string()));
        assert_eq!(attachment.extension, Some("pdf".to_string()));
    }

    #[test]
    fn test_attachment_type_from_extension() {
        assert_eq!(AttachmentType::from_extension("jpg"), AttachmentType::Image);
        assert_eq!(
            AttachmentType::from_extension(".png"),
            AttachmentType::Image
        );
        assert_eq!(AttachmentType::from_extension("mp3"), AttachmentType::Audio);
        assert_eq!(
            AttachmentType::from_extension("pdf"),
            AttachmentType::Document
        );
        assert_eq!(
            AttachmentType::from_extension("zip"),
            AttachmentType::Archive
        );
    }

    #[test]
    fn test_attachment_type_from_mime() {
        assert_eq!(
            AttachmentType::from_mime_type("image/jpeg"),
            AttachmentType::Image
        );
        assert_eq!(
            AttachmentType::from_mime_type("audio/mpeg"),
            AttachmentType::Audio
        );
        assert_eq!(
            AttachmentType::from_mime_type("application/pdf"),
            AttachmentType::Document
        );
    }

    #[test]
    fn test_filter_by_type() {
        let attachments = vec![
            ParsedAttachment {
                id: "1".to_string(),
                attachment_type: AttachmentType::Image,
                file_key: None,
                url: None,
                file_name: None,
                file_size: None,
                mime_type: None,
                extension: None,
                source_platform: PlatformSource::Generic,
                metadata: HashMap::new(),
            },
            ParsedAttachment {
                id: "2".to_string(),
                attachment_type: AttachmentType::File,
                file_key: None,
                url: None,
                file_name: None,
                file_size: None,
                mime_type: None,
                extension: None,
                source_platform: PlatformSource::Generic,
                metadata: HashMap::new(),
            },
            ParsedAttachment {
                id: "3".to_string(),
                attachment_type: AttachmentType::Image,
                file_key: None,
                url: None,
                file_name: None,
                file_size: None,
                mime_type: None,
                extension: None,
                source_platform: PlatformSource::Generic,
                metadata: HashMap::new(),
            },
        ];

        let images = AttachmentParser::filter_by_type(&attachments, AttachmentType::Image);
        assert_eq!(images.len(), 2);
    }

    #[test]
    fn test_get_all_images() {
        let attachments = vec![
            ParsedAttachment {
                id: "1".to_string(),
                attachment_type: AttachmentType::Image,
                file_key: None,
                url: None,
                file_name: None,
                file_size: None,
                mime_type: None,
                extension: None,
                source_platform: PlatformSource::Generic,
                metadata: HashMap::new(),
            },
            ParsedAttachment {
                id: "2".to_string(),
                attachment_type: AttachmentType::EmbeddedImage,
                file_key: None,
                url: None,
                file_name: None,
                file_size: None,
                mime_type: None,
                extension: None,
                source_platform: PlatformSource::Generic,
                metadata: HashMap::new(),
            },
            ParsedAttachment {
                id: "3".to_string(),
                attachment_type: AttachmentType::File,
                file_key: None,
                url: None,
                file_name: None,
                file_size: None,
                mime_type: None,
                extension: None,
                source_platform: PlatformSource::Generic,
                metadata: HashMap::new(),
            },
        ];

        let images = AttachmentParser::get_all_images(&attachments);
        assert_eq!(images.len(), 2);
    }

    #[test]
    fn test_has_attachments() {
        let image = LarkImageContent {
            image_key: "img_123".to_string(),
        };
        let content = LarkContent::Image(image);
        assert!(AttachmentParser::has_attachments(&content));

        let text = crate::communication::channel::lark_content::LarkTextContent {
            text: "Hello".to_string(),
        };
        let content = LarkContent::Text(text);
        assert!(!AttachmentParser::has_attachments(&content));
    }

    #[test]
    fn test_extract_filename_from_url() {
        assert_eq!(
            AttachmentParser::extract_filename_from_url("https://example.com/file.pdf"),
            Some("file.pdf".to_string())
        );
        assert_eq!(
            AttachmentParser::extract_filename_from_url("https://example.com/path/to/image.png"),
            Some("image.png".to_string())
        );
        assert_eq!(
            AttachmentParser::extract_filename_from_url("https://example.com/"),
            None
        );
    }
}
