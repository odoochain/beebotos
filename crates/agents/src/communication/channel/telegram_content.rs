//! Telegram Message Content Parser
//!
//! Provides parsing and handling of various Telegram message content types
//! including text, photo, video, audio, voice, document, location, contact, and
//! sticker.

use serde::{Deserialize, Serialize};

use crate::communication::channel::content::{
    ContentType as UnifiedContentType, MediaContent, PlatformContent,
};
use crate::error::{AgentError, Result};

/// Telegram message content types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TelegramContent {
    /// Plain text message
    Text(TelegramTextContent),
    /// Photo message
    Photo(TelegramPhotoContent),
    /// Video message
    Video(TelegramVideoContent),
    /// Audio/music message
    Audio(TelegramAudioContent),
    /// Voice message
    Voice(TelegramVoiceContent),
    /// Document/file message
    Document(TelegramDocumentContent),
    /// Location message
    Location(TelegramLocationContent),
    /// Contact message
    Contact(TelegramContactContent),
    /// Sticker message
    Sticker(TelegramStickerContent),
    /// Animation (GIF) message
    Animation(TelegramAnimationContent),
    /// Video note (round video) message
    VideoNote(TelegramVideoNoteContent),
}

/// Text content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramTextContent {
    /// The text content
    pub text: String,
    /// Entities in the message text (mentions, hashtags, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<TelegramMessageEntity>>,
}

/// Message entity (mention, hashtag, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramMessageEntity {
    /// Type of the entity
    #[serde(rename = "type")]
    pub entity_type: String,
    /// Offset in UTF-16 code units to the start of the entity
    pub offset: i32,
    /// Length of the entity in UTF-16 code units
    pub length: i32,
    /// For "text_link" only, URL that will be opened after user taps on the
    /// text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// For "text_mention" only, the mentioned user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<TelegramUser>,
    /// For "pre" only, the programming language of the entity text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// User information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramUser {
    /// Unique identifier for this user or bot
    pub id: i64,
    /// True, if this user is a bot
    #[serde(rename = "is_bot")]
    pub is_bot: bool,
    /// User's or bot's first name
    #[serde(rename = "first_name")]
    pub first_name: String,
    /// User's or bot's last name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "last_name")]
    pub last_name: Option<String>,
    /// User's or bot's username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// IETF language tag of the user's language
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "language_code")]
    pub language_code: Option<String>,
}

/// Photo content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramPhotoContent {
    /// Array of available photo sizes
    pub photos: Vec<TelegramPhotoSize>,
    /// Caption for the photo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// Caption entities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<TelegramMessageEntity>>,
}

/// Photo size
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelegramPhotoSize {
    /// Identifier for this file
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// Unique identifier for this file
    #[serde(rename = "file_unique_id")]
    pub file_unique_id: String,
    /// Photo width
    pub width: i32,
    /// Photo height
    pub height: i32,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_size")]
    pub file_size: Option<i64>,
}

/// Video content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramVideoContent {
    /// Video file information
    pub video: TelegramVideoFile,
    /// Caption for the video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// Caption entities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<TelegramMessageEntity>>,
}

/// Video file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramVideoFile {
    /// Identifier for this file
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// Unique identifier for this file
    #[serde(rename = "file_unique_id")]
    pub file_unique_id: String,
    /// Video width
    pub width: i32,
    /// Video height
    pub height: i32,
    /// Duration of the video in seconds
    pub duration: i32,
    /// Video thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<TelegramPhotoSize>,
    /// Original filename
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_name")]
    pub file_name: Option<String>,
    /// MIME type of the file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime_type")]
    pub mime_type: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_size")]
    pub file_size: Option<i64>,
}

/// Audio content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramAudioContent {
    /// Audio file information
    pub audio: TelegramAudioFile,
    /// Caption for the audio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

/// Audio file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramAudioFile {
    /// Identifier for this file
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// Unique identifier for this file
    #[serde(rename = "file_unique_id")]
    pub file_unique_id: String,
    /// Duration of the audio in seconds
    pub duration: i32,
    /// Performer of the audio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performer: Option<String>,
    /// Title of the audio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Original filename
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_name")]
    pub file_name: Option<String>,
    /// MIME type of the file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime_type")]
    pub mime_type: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_size")]
    pub file_size: Option<i64>,
    /// Thumbnail of the album cover
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<TelegramPhotoSize>,
}

/// Voice content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramVoiceContent {
    /// Voice file information
    pub voice: TelegramVoiceFile,
    /// Caption for the voice message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

/// Voice file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramVoiceFile {
    /// Identifier for this file
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// Unique identifier for this file
    #[serde(rename = "file_unique_id")]
    pub file_unique_id: String,
    /// Duration of the audio in seconds
    pub duration: i32,
    /// MIME type of the file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime_type")]
    pub mime_type: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_size")]
    pub file_size: Option<i64>,
}

/// Document content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramDocumentContent {
    /// Document file information
    pub document: TelegramDocumentFile,
    /// Caption for the document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// Caption entities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<TelegramMessageEntity>>,
}

/// Document file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramDocumentFile {
    /// Identifier for this file
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// Unique identifier for this file
    #[serde(rename = "file_unique_id")]
    pub file_unique_id: String,
    /// Document thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<TelegramPhotoSize>,
    /// Original filename
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_name")]
    pub file_name: Option<String>,
    /// MIME type of the file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime_type")]
    pub mime_type: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_size")]
    pub file_size: Option<i64>,
}

/// Location content structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TelegramLocationContent {
    /// Longitude
    pub longitude: f64,
    /// Latitude
    pub latitude: f64,
    /// The radius of uncertainty for the location, measured in meters
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "horizontal_accuracy")]
    pub horizontal_accuracy: Option<f64>,
    /// Time relative to the message sending date, during which the location can
    /// be updated
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "live_period")]
    pub live_period: Option<i32>,
}

/// Contact content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramContactContent {
    /// Contact's phone number
    #[serde(rename = "phone_number")]
    pub phone_number: String,
    /// Contact's first name
    #[serde(rename = "first_name")]
    pub first_name: String,
    /// Contact's last name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "last_name")]
    pub last_name: Option<String>,
    /// Contact's user identifier in Telegram
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "user_id")]
    pub user_id: Option<i64>,
    /// Additional data about the contact in vCard format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcard: Option<String>,
}

/// Sticker content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramStickerContent {
    /// Sticker file information
    pub sticker: TelegramStickerFile,
}

/// Sticker file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramStickerFile {
    /// Identifier for this file
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// Unique identifier for this file
    #[serde(rename = "file_unique_id")]
    pub file_unique_id: String,
    /// Type of the sticker
    #[serde(rename = "type")]
    pub sticker_type: String,
    /// Sticker width
    pub width: i32,
    /// Sticker height
    pub height: i32,
    /// True, if the sticker is animated
    #[serde(rename = "is_animated")]
    pub is_animated: bool,
    /// True, if the sticker is a video sticker
    #[serde(rename = "is_video")]
    pub is_video: bool,
    /// Sticker thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<TelegramPhotoSize>,
    /// Emoji associated with the sticker
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    /// Name of the sticker set
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "set_name")]
    pub set_name: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_size")]
    pub file_size: Option<i64>,
}

/// Animation (GIF) content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramAnimationContent {
    /// Animation file information
    pub animation: TelegramAnimationFile,
    /// Caption for the animation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

/// Animation file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramAnimationFile {
    /// Identifier for this file
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// Unique identifier for this file
    #[serde(rename = "file_unique_id")]
    pub file_unique_id: String,
    /// Video width
    pub width: i32,
    /// Video height
    pub height: i32,
    /// Duration of the video in seconds
    pub duration: i32,
    /// Animation thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<TelegramPhotoSize>,
    /// Original filename
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_name")]
    pub file_name: Option<String>,
    /// MIME type of the file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime_type")]
    pub mime_type: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_size")]
    pub file_size: Option<i64>,
}

/// Video note (round video) content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramVideoNoteContent {
    /// Video note file information
    pub video_note: TelegramVideoNoteFile,
}

/// Video note file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelegramVideoNoteFile {
    /// Identifier for this file
    #[serde(rename = "file_id")]
    pub file_id: String,
    /// Unique identifier for this file
    #[serde(rename = "file_unique_id")]
    pub file_unique_id: String,
    /// Video width and height (diameter of the video message)
    pub length: i32,
    /// Duration of the video in seconds
    pub duration: i32,
    /// Video thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<TelegramPhotoSize>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "file_size")]
    pub file_size: Option<i64>,
}

/// Telegram content parser
#[derive(Debug, Clone, Default)]
pub struct TelegramContentParser;

impl TelegramContentParser {
    /// Create a new content parser
    pub fn new() -> Self {
        Self
    }

    /// Parse content from JSON value
    ///
    /// # Arguments
    /// * `content_type` - The content type (text, photo, video, etc.)
    /// * `content` - The JSON content to parse
    ///
    /// # Returns
    /// Parsed TelegramContent enum variant
    pub fn parse(content_type: &str, content: serde_json::Value) -> Result<TelegramContent> {
        match content_type {
            "text" => {
                let text_content: TelegramTextContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse text content: {}", e))
                    })?;
                Ok(TelegramContent::Text(text_content))
            }
            "photo" => {
                let photo_content: TelegramPhotoContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse photo content: {}", e))
                    })?;
                Ok(TelegramContent::Photo(photo_content))
            }
            "video" => {
                let video_content: TelegramVideoContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse video content: {}", e))
                    })?;
                Ok(TelegramContent::Video(video_content))
            }
            "audio" => {
                let audio_content: TelegramAudioContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse audio content: {}", e))
                    })?;
                Ok(TelegramContent::Audio(audio_content))
            }
            "voice" => {
                let voice_content: TelegramVoiceContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse voice content: {}", e))
                    })?;
                Ok(TelegramContent::Voice(voice_content))
            }
            "document" => {
                let document_content: TelegramDocumentContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse document content: {}", e))
                    })?;
                Ok(TelegramContent::Document(document_content))
            }
            "location" => {
                let location_content: TelegramLocationContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse location content: {}", e))
                    })?;
                Ok(TelegramContent::Location(location_content))
            }
            "contact" => {
                let contact_content: TelegramContactContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse contact content: {}", e))
                    })?;
                Ok(TelegramContent::Contact(contact_content))
            }
            "sticker" => {
                let sticker_content: TelegramStickerContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse sticker content: {}", e))
                    })?;
                Ok(TelegramContent::Sticker(sticker_content))
            }
            "animation" => {
                let animation_content: TelegramAnimationContent = serde_json::from_value(content)
                    .map_err(|e| {
                    AgentError::platform(format!("Failed to parse animation content: {}", e))
                })?;
                Ok(TelegramContent::Animation(animation_content))
            }
            "video_note" => {
                let video_note_content: TelegramVideoNoteContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse video note content: {}", e))
                    })?;
                Ok(TelegramContent::VideoNote(video_note_content))
            }
            _ => {
                Err(AgentError::platform(format!("Unknown content type: {}", content_type)).into())
            }
        }
    }

    /// Parse content from JSON string
    ///
    /// # Arguments
    /// * `content_type` - The content type
    /// * `content_json` - JSON string to parse
    ///
    /// # Returns
    /// Parsed TelegramContent enum variant
    pub fn parse_str(content_type: &str, content_json: &str) -> Result<TelegramContent> {
        let content: serde_json::Value = serde_json::from_str(content_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON: {}", e)))?;
        Self::parse(content_type, content)
    }

    /// Extract plain text from any content type
    ///
    /// # Arguments
    /// * `content` - The TelegramContent to extract text from
    ///
    /// # Returns
    /// Extracted plain text string
    pub fn extract_text(content: &TelegramContent) -> String {
        match content {
            TelegramContent::Text(text) => text.text.clone(),
            TelegramContent::Photo(photo) => {
                format!(
                    "[Photo] {}",
                    photo.caption.as_deref().unwrap_or("(no caption)")
                )
            }
            TelegramContent::Video(video) => {
                format!(
                    "[Video: {}s] {}",
                    video.video.duration,
                    video.caption.as_deref().unwrap_or("(no caption)")
                )
            }
            TelegramContent::Audio(audio) => {
                format!(
                    "[Audio: {} - {}] {}",
                    audio.audio.performer.as_deref().unwrap_or("Unknown"),
                    audio.audio.title.as_deref().unwrap_or("Unknown"),
                    audio.caption.as_deref().unwrap_or("(no caption)")
                )
            }
            TelegramContent::Voice(voice) => {
                format!(
                    "[Voice: {}s] {}",
                    voice.voice.duration,
                    voice.caption.as_deref().unwrap_or("(no caption)")
                )
            }
            TelegramContent::Document(doc) => {
                format!(
                    "[File: {}] {}",
                    doc.document.file_name.as_deref().unwrap_or("unnamed"),
                    doc.caption.as_deref().unwrap_or("(no caption)")
                )
            }
            TelegramContent::Location(loc) => {
                format!("[Location: {}, {}]", loc.latitude, loc.longitude)
            }
            TelegramContent::Contact(contact) => {
                format!(
                    "[Contact: {} {} - {}]",
                    contact.first_name,
                    contact.last_name.as_deref().unwrap_or(""),
                    contact.phone_number
                )
            }
            TelegramContent::Sticker(sticker) => {
                format!("[Sticker: {:?}]", sticker.sticker.emoji)
            }
            TelegramContent::Animation(anim) => {
                format!(
                    "[Animation: {}s] {}",
                    anim.animation.duration,
                    anim.caption.as_deref().unwrap_or("(no caption)")
                )
            }
            TelegramContent::VideoNote(note) => {
                format!("[Video Note: {}s]", note.video_note.duration)
            }
        }
    }

    /// Get file ID from content (for media types)
    ///
    /// # Arguments
    /// * `content` - The TelegramContent
    ///
    /// # Returns
    /// File ID if available
    pub fn get_file_id(content: &TelegramContent) -> Option<&str> {
        match content {
            TelegramContent::Photo(photo) => photo
                .photos
                .iter()
                .max_by_key(|p| p.file_size.unwrap_or(0))
                .map(|p| p.file_id.as_str()),
            TelegramContent::Video(video) => Some(&video.video.file_id),
            TelegramContent::Audio(audio) => Some(&audio.audio.file_id),
            TelegramContent::Voice(voice) => Some(&voice.voice.file_id),
            TelegramContent::Document(doc) => Some(&doc.document.file_id),
            TelegramContent::Sticker(sticker) => Some(&sticker.sticker.file_id),
            TelegramContent::Animation(anim) => Some(&anim.animation.file_id),
            TelegramContent::VideoNote(note) => Some(&note.video_note.file_id),
            _ => None,
        }
    }

    /// Get file size from content (for media types)
    ///
    /// # Arguments
    /// * `content` - The TelegramContent
    ///
    /// # Returns
    /// File size in bytes if available
    pub fn get_file_size(content: &TelegramContent) -> Option<i64> {
        match content {
            TelegramContent::Photo(photo) => photo
                .photos
                .iter()
                .max_by_key(|p| p.file_size.unwrap_or(0))
                .and_then(|p| p.file_size),
            TelegramContent::Video(video) => video.video.file_size,
            TelegramContent::Audio(audio) => audio.audio.file_size,
            TelegramContent::Voice(voice) => voice.voice.file_size,
            TelegramContent::Document(doc) => doc.document.file_size,
            TelegramContent::Sticker(sticker) => sticker.sticker.file_size,
            TelegramContent::Animation(anim) => anim.animation.file_size,
            TelegramContent::VideoNote(note) => note.video_note.file_size,
            _ => None,
        }
    }

    /// Get MIME type from content
    ///
    /// # Arguments
    /// * `content` - The TelegramContent
    ///
    /// # Returns
    /// MIME type if available
    pub fn get_mime_type(content: &TelegramContent) -> Option<&str> {
        match content {
            TelegramContent::Video(video) => video.video.mime_type.as_deref(),
            TelegramContent::Audio(audio) => audio.audio.mime_type.as_deref(),
            TelegramContent::Voice(voice) => voice.voice.mime_type.as_deref(),
            TelegramContent::Document(doc) => doc.document.mime_type.as_deref(),
            TelegramContent::Animation(anim) => anim.animation.mime_type.as_deref(),
            _ => None,
        }
    }

    /// Get content type string
    ///
    /// # Arguments
    /// * `content` - The TelegramContent
    ///
    /// # Returns
    /// Content type string
    pub fn get_content_type(content: &TelegramContent) -> &'static str {
        match content {
            TelegramContent::Text(_) => "text",
            TelegramContent::Photo(_) => "photo",
            TelegramContent::Video(_) => "video",
            TelegramContent::Audio(_) => "audio",
            TelegramContent::Voice(_) => "voice",
            TelegramContent::Document(_) => "document",
            TelegramContent::Location(_) => "location",
            TelegramContent::Contact(_) => "contact",
            TelegramContent::Sticker(_) => "sticker",
            TelegramContent::Animation(_) => "animation",
            TelegramContent::VideoNote(_) => "video_note",
        }
    }

    /// Create text content
    pub fn create_text(text: impl Into<String>) -> TelegramContent {
        TelegramContent::Text(TelegramTextContent {
            text: text.into(),
            entities: None,
        })
    }

    /// Create photo content
    pub fn create_photo(
        photos: Vec<TelegramPhotoSize>,
        caption: Option<String>,
    ) -> TelegramContent {
        TelegramContent::Photo(TelegramPhotoContent {
            photos,
            caption,
            caption_entities: None,
        })
    }

    /// Create video content
    pub fn create_video(video: TelegramVideoFile, caption: Option<String>) -> TelegramContent {
        TelegramContent::Video(TelegramVideoContent {
            video,
            caption,
            caption_entities: None,
        })
    }

    /// Create audio content
    pub fn create_audio(audio: TelegramAudioFile, caption: Option<String>) -> TelegramContent {
        TelegramContent::Audio(TelegramAudioContent { audio, caption })
    }

    /// Create voice content
    pub fn create_voice(voice: TelegramVoiceFile, caption: Option<String>) -> TelegramContent {
        TelegramContent::Voice(TelegramVoiceContent { voice, caption })
    }

    /// Create document content
    pub fn create_document(
        document: TelegramDocumentFile,
        caption: Option<String>,
    ) -> TelegramContent {
        TelegramContent::Document(TelegramDocumentContent {
            document,
            caption,
            caption_entities: None,
        })
    }

    /// Create location content
    pub fn create_location(latitude: f64, longitude: f64) -> TelegramContent {
        TelegramContent::Location(TelegramLocationContent {
            latitude,
            longitude,
            horizontal_accuracy: None,
            live_period: None,
        })
    }

    /// Create contact content
    pub fn create_contact(
        phone_number: impl Into<String>,
        first_name: impl Into<String>,
    ) -> TelegramContent {
        TelegramContent::Contact(TelegramContactContent {
            phone_number: phone_number.into(),
            first_name: first_name.into(),
            last_name: None,
            user_id: None,
            vcard: None,
        })
    }

    /// Create sticker content
    pub fn create_sticker(sticker: TelegramStickerFile) -> TelegramContent {
        TelegramContent::Sticker(TelegramStickerContent { sticker })
    }

    /// Serialize content to JSON string
    pub fn to_json(content: &TelegramContent) -> Result<String> {
        serde_json::to_string(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Serialize content to JSON value
    pub fn to_json_value(content: &TelegramContent) -> Result<serde_json::Value> {
        serde_json::to_value(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_content() {
        let json = serde_json::json!({
            "text": "Hello, World!"
        });
        let content = TelegramContentParser::parse("text", json).unwrap();
        assert!(matches!(content, TelegramContent::Text(_)));
        assert_eq!(
            TelegramContentParser::extract_text(&content),
            "Hello, World!"
        );
    }

    #[test]
    fn test_parse_photo_content() {
        let json = serde_json::json!({
            "photos": [
                {
                    "file_id": "photo123",
                    "file_unique_id": "unique123",
                    "width": 100,
                    "height": 100,
                    "file_size": 1024
                }
            ],
            "caption": "My photo"
        });
        let content = TelegramContentParser::parse("photo", json).unwrap();
        assert!(matches!(content, TelegramContent::Photo(_)));
        let text = TelegramContentParser::extract_text(&content);
        assert!(text.contains("Photo"));
        assert!(text.contains("My photo"));
    }

    #[test]
    fn test_parse_video_content() {
        let json = serde_json::json!({
            "video": {
                "file_id": "video123",
                "file_unique_id": "unique123",
                "width": 1920,
                "height": 1080,
                "duration": 60
            },
            "caption": "My video"
        });
        let content = TelegramContentParser::parse("video", json).unwrap();
        assert!(matches!(content, TelegramContent::Video(_)));
        let text = TelegramContentParser::extract_text(&content);
        assert!(text.contains("Video"));
        assert!(text.contains("60s"));
    }

    #[test]
    fn test_parse_location_content() {
        let json = serde_json::json!({
            "latitude": 37.7749,
            "longitude": -122.4194
        });
        let content = TelegramContentParser::parse("location", json).unwrap();
        assert!(matches!(content, TelegramContent::Location(_)));
        let text = TelegramContentParser::extract_text(&content);
        assert!(text.contains("37.7749"));
        assert!(text.contains("-122.4194"));
    }

    #[test]
    fn test_parse_contact_content() {
        let json = serde_json::json!({
            "phone_number": "+1234567890",
            "first_name": "John",
            "last_name": "Doe"
        });
        let content = TelegramContentParser::parse("contact", json).unwrap();
        assert!(matches!(content, TelegramContent::Contact(_)));
        let text = TelegramContentParser::extract_text(&content);
        assert!(text.contains("John"));
        assert!(text.contains("Doe"));
        assert!(text.contains("+1234567890"));
    }

    #[test]
    fn test_get_file_id() {
        let photo = TelegramContent::Photo(TelegramPhotoContent {
            photos: vec![TelegramPhotoSize {
                file_id: "photo123".to_string(),
                file_unique_id: "unique123".to_string(),
                width: 100,
                height: 100,
                file_size: Some(1024),
            }],
            caption: None,
            caption_entities: None,
        });
        assert_eq!(TelegramContentParser::get_file_id(&photo), Some("photo123"));

        let text = TelegramContent::Text(TelegramTextContent::default());
        assert_eq!(TelegramContentParser::get_file_id(&text), None);
    }

    #[test]
    fn test_get_content_type() {
        let text = TelegramContent::Text(TelegramTextContent::default());
        assert_eq!(TelegramContentParser::get_content_type(&text), "text");

        let photo = TelegramContent::Photo(TelegramPhotoContent::default());
        assert_eq!(TelegramContentParser::get_content_type(&photo), "photo");

        let video = TelegramContent::Video(TelegramVideoContent::default());
        assert_eq!(TelegramContentParser::get_content_type(&video), "video");
    }

    #[test]
    fn test_create_text() {
        let content = TelegramContentParser::create_text("Test message");
        assert!(
            matches!(content, TelegramContent::Text(TelegramTextContent { text, .. }) if text == "Test message")
        );
    }

    #[test]
    fn test_create_location() {
        let content = TelegramContentParser::create_location(37.7749, -122.4194);
        assert!(matches!(content, TelegramContent::Location(_)));
        if let TelegramContent::Location(loc) = content {
            assert_eq!(loc.latitude, 37.7749);
            assert_eq!(loc.longitude, -122.4194);
        }
    }

    #[test]
    fn test_serialize_content() {
        let content = TelegramContentParser::create_text("Test");
        let json = TelegramContentParser::to_json(&content).unwrap();
        assert!(json.contains("Test"));
    }
}

// =============================================================================
// 🟢 P0 FIX: PlatformContent trait implementation for unified content framework
// =============================================================================

impl PlatformContent for TelegramContent {
    fn content_type(&self) -> UnifiedContentType {
        match self {
            TelegramContent::Text(_) => UnifiedContentType::Text,
            TelegramContent::Photo(_) => UnifiedContentType::Image,
            TelegramContent::Video(_) => UnifiedContentType::Video,
            TelegramContent::Audio(_) => UnifiedContentType::Audio,
            TelegramContent::Voice(_) => UnifiedContentType::Audio,
            TelegramContent::Document(_) => UnifiedContentType::File,
            TelegramContent::Location(_) => UnifiedContentType::Location,
            TelegramContent::Contact(_) => UnifiedContentType::Contact,
            TelegramContent::Sticker(_) => UnifiedContentType::Sticker,
            TelegramContent::Animation(_) => UnifiedContentType::Video,
            TelegramContent::VideoNote(_) => UnifiedContentType::Video,
        }
    }

    fn extract_text(&self) -> String {
        TelegramContentParser::extract_text(self)
    }
}

impl TelegramPhotoContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        // Use the largest photo size available
        let photo = self.photos.iter().max_by_key(|p| p.file_size.unwrap_or(0));
        MediaContent {
            url: photo.map(|p| p.file_id.clone()).unwrap_or_default(),
            width: photo.map(|p| p.width),
            height: photo.map(|p| p.height),
            size: photo.and_then(|p| p.file_size),
            mime_type: None,
            filename: None,
            caption: self.caption.clone(),
            thumbnail: None,
            duration: None,
        }
    }
}

impl TelegramVideoContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.video.file_id.clone(),
            width: Some(self.video.width),
            height: Some(self.video.height),
            duration: Some(self.video.duration),
            mime_type: self.video.mime_type.clone(),
            filename: self.video.file_name.clone(),
            caption: self.caption.clone(),
            size: self.video.file_size,
            thumbnail: self.video.thumbnail.as_ref().map(|t| t.file_id.clone()),
        }
    }
}

impl TelegramDocumentContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.document.file_id.clone(),
            mime_type: self.document.mime_type.clone(),
            filename: self.document.file_name.clone(),
            size: self.document.file_size,
            width: None,
            height: None,
            duration: None,
            caption: self.caption.clone(),
            thumbnail: self.document.thumbnail.as_ref().map(|t| t.file_id.clone()),
        }
    }
}
