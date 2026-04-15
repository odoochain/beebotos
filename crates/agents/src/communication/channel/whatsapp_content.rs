//! WhatsApp Message Content Parser
//!
//! Provides parsing and handling of various WhatsApp message content types
//! including text, media, documents, locations, contacts, and interactive
//! messages. This version is designed to work with the Baileys bridge.

use serde::{Deserialize, Serialize};

use crate::communication::channel::content::{
    ContentType as UnifiedContentType, MediaContent, PlatformContent,
};
use crate::error::{AgentError, Result};

/// WhatsApp message content types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum WhatsAppContent {
    /// Plain text message
    #[serde(rename = "text")]
    Text(WhatsAppTextContent),
    /// Image message
    #[serde(rename = "image")]
    Image(WhatsAppImageContent),
    /// Video message
    #[serde(rename = "video")]
    Video(WhatsAppVideoContent),
    /// Audio/voice message
    #[serde(rename = "audio")]
    Audio(WhatsAppAudioContent),
    /// Voice message (PTT - Push To Talk)
    #[serde(rename = "voice")]
    Voice(WhatsAppVoiceContent),
    /// Document message
    #[serde(rename = "document")]
    Document(WhatsAppDocumentContent),
    /// Location message
    #[serde(rename = "location")]
    Location(WhatsAppLocationContent),
    /// Contact message
    #[serde(rename = "contacts")]
    Contacts(WhatsAppContactsContent),
    /// Sticker message
    #[serde(rename = "sticker")]
    Sticker(WhatsAppStickerContent),
    /// Interactive message (buttons, lists)
    #[serde(rename = "interactive")]
    Interactive(WhatsAppInteractiveContent),
    /// Template message
    #[serde(rename = "template")]
    Template(WhatsAppTemplateContent),
    /// Reaction message
    #[serde(rename = "reaction")]
    Reaction(WhatsAppReactionContent),
}

/// Text content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WhatsAppTextContent {
    /// The text content (max 4096 characters for Baileys)
    pub body: String,
    /// Whether this message is a preview URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<bool>,
    /// Quoted message info (for replies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quoted_message: Option<Box<WhatsAppTextContent>>,
}

/// Image content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WhatsAppImageContent {
    /// Media file path (local path after download)
    pub file_path: Option<String>,
    /// Media URL (if available)
    pub url: Option<String>,
    /// Image caption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Image width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// Image height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
}

/// Video content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WhatsAppVideoContent {
    /// Media file path (local path after download)
    pub file_path: Option<String>,
    /// Media URL (if available)
    pub url: Option<String>,
    /// Video caption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Video width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// Video height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
}

/// Audio content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WhatsAppAudioContent {
    /// Media file path (local path after download)
    pub file_path: Option<String>,
    /// Media URL (if available)
    pub url: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
}

/// Voice content structure (PTT - Push To Talk)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WhatsAppVoiceContent {
    /// Media file path (local path after download)
    pub file_path: Option<String>,
    /// Media URL (if available)
    pub url: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    /// Waveform data (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waveform: Option<Vec<u8>>,
}

/// Document content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WhatsAppDocumentContent {
    /// Media file path (local path after download)
    pub file_path: Option<String>,
    /// Media URL (if available)
    pub url: Option<String>,
    /// Document caption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// File name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
}

/// Location content structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WhatsAppLocationContent {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Location name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Location address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// URL for the location (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Contacts content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WhatsAppContactsContent {
    pub contacts: Vec<WhatsAppContact>,
}

/// WhatsApp contact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppContact {
    /// Contact display name
    pub display_name: String,
    /// vCard data
    pub vcard: String,
    /// Phone number (extracted from vCard)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// Organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
}

/// Sticker content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WhatsAppStickerContent {
    /// Media file path (local path after download)
    pub file_path: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    /// Animated sticker flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animated: Option<bool>,
    /// Sticker emoji (if associated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
}

/// Interactive content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppInteractiveContent {
    /// Interactive type: button, list, product, product_list
    #[serde(rename = "type")]
    pub interactive_type: String,
    /// Header (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<WhatsAppInteractiveHeader>,
    /// Body
    pub body: WhatsAppInteractiveBody,
    /// Footer (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<WhatsAppInteractiveFooter>,
    /// Action
    pub action: WhatsAppInteractiveAction,
}

/// Interactive header
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppInteractiveHeader {
    /// Header type: text, video, image, document
    #[serde(rename = "type")]
    pub header_type: String,
    /// Text content (if type is text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Media content (if type is media)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<WhatsAppVideoContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<WhatsAppImageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<WhatsAppDocumentContent>,
}

/// Interactive body
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppInteractiveBody {
    /// Body text
    pub text: String,
}

/// Interactive footer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppInteractiveFooter {
    /// Footer text
    pub text: String,
}

/// Interactive action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppInteractiveAction {
    /// Button text (for button messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button: Option<String>,
    /// Button sections (for list messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<WhatsAppSection>>,
    /// Buttons (for button messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<WhatsAppButton>>,
}

/// Section for list messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppSection {
    /// Section title
    pub title: String,
    /// Section rows
    pub rows: Vec<WhatsAppSectionRow>,
}

/// Section row
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppSectionRow {
    /// Row ID
    pub id: String,
    /// Row title
    pub title: String,
    /// Row description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Button
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppButton {
    /// Button type: reply
    #[serde(rename = "type")]
    pub button_type: String,
    /// Reply button
    pub reply: WhatsAppReplyButton,
}

/// Reply button
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppReplyButton {
    /// Button ID
    pub id: String,
    /// Button title
    pub title: String,
}

/// Template content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppTemplateContent {
    /// Template name
    pub name: String,
    /// Template language
    pub language: WhatsAppTemplateLanguage,
    /// Template components
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<WhatsAppTemplateComponent>>,
}

/// Template language
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppTemplateLanguage {
    /// Language code (e.g., "en", "en_US")
    pub code: String,
    /// Policy: deterministic or fallback
    pub policy: String,
}

/// Template component
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppTemplateComponent {
    /// Component type: header, body, button
    #[serde(rename = "type")]
    pub component_type: String,
    /// Component parameters
    pub parameters: Vec<WhatsAppTemplateParameter>,
}

/// Template parameter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppTemplateParameter {
    /// Parameter type: text, currency, date_time, image, document, video
    #[serde(rename = "type")]
    pub param_type: String,
    /// Text value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Currency value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<WhatsAppCurrency>,
    /// DateTime value
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "date_time")]
    pub date_time: Option<WhatsAppDateTime>,
    /// Image value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<WhatsAppImageContent>,
    /// Document value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<WhatsAppDocumentContent>,
    /// Video value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<WhatsAppVideoContent>,
}

/// Currency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppCurrency {
    /// Currency code (ISO 4217)
    #[serde(rename = "currency_code")]
    pub currency_code: String,
    /// Amount multiplied by 1000
    #[serde(rename = "amount_1000")]
    pub amount_1000: i64,
}

/// DateTime
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppDateTime {
    /// Fallback text
    #[serde(rename = "fallback_value")]
    pub fallback_value: String,
}

/// Reaction content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhatsAppReactionContent {
    /// Message ID to react to
    #[serde(rename = "message_id")]
    pub message_id: String,
    /// Emoji reaction
    pub emoji: String,
}

/// WhatsApp content parser
#[derive(Debug, Clone, Default)]
pub struct WhatsAppContentParser;

impl WhatsAppContentParser {
    /// Create a new content parser
    pub fn new() -> Self {
        Self
    }

    /// Parse content from JSON value
    pub fn parse(content_type: &str, content: serde_json::Value) -> Result<WhatsAppContent> {
        match content_type {
            "text" => {
                let text_content: WhatsAppTextContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse text content: {}", e))
                    })?;
                Ok(WhatsAppContent::Text(text_content))
            }
            "image" => {
                let image_content: WhatsAppImageContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse image content: {}", e))
                    })?;
                Ok(WhatsAppContent::Image(image_content))
            }
            "video" => {
                let video_content: WhatsAppVideoContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse video content: {}", e))
                    })?;
                Ok(WhatsAppContent::Video(video_content))
            }
            "audio" => {
                let audio_content: WhatsAppAudioContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse audio content: {}", e))
                    })?;
                Ok(WhatsAppContent::Audio(audio_content))
            }
            "voice" => {
                let voice_content: WhatsAppVoiceContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse voice content: {}", e))
                    })?;
                Ok(WhatsAppContent::Voice(voice_content))
            }
            "document" => {
                let document_content: WhatsAppDocumentContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse document content: {}", e))
                    })?;
                Ok(WhatsAppContent::Document(document_content))
            }
            "location" => {
                let location_content: WhatsAppLocationContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse location content: {}", e))
                    })?;
                Ok(WhatsAppContent::Location(location_content))
            }
            "contacts" => {
                let contacts_content: WhatsAppContactsContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse contacts content: {}", e))
                    })?;
                Ok(WhatsAppContent::Contacts(contacts_content))
            }
            "sticker" => {
                let sticker_content: WhatsAppStickerContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse sticker content: {}", e))
                    })?;
                Ok(WhatsAppContent::Sticker(sticker_content))
            }
            "interactive" => {
                let interactive_content: WhatsAppInteractiveContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse interactive content: {}", e))
                    })?;
                Ok(WhatsAppContent::Interactive(interactive_content))
            }
            "template" => {
                let template_content: WhatsAppTemplateContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse template content: {}", e))
                    })?;
                Ok(WhatsAppContent::Template(template_content))
            }
            "reaction" => {
                let reaction_content: WhatsAppReactionContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse reaction content: {}", e))
                    })?;
                Ok(WhatsAppContent::Reaction(reaction_content))
            }
            _ => {
                Err(AgentError::platform(format!("Unknown content type: {}", content_type)).into())
            }
        }
    }

    /// Parse content from JSON string
    pub fn parse_str(content_type: &str, content_json: &str) -> Result<WhatsAppContent> {
        let content: serde_json::Value = serde_json::from_str(content_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON: {}", e)))?;
        Self::parse(content_type, content)
    }

    /// Extract plain text from any content type
    pub fn extract_text(content: &WhatsAppContent) -> String {
        match content {
            WhatsAppContent::Text(text) => text.body.clone(),
            WhatsAppContent::Image(image) => {
                format!("[Image] {}", image.caption.as_deref().unwrap_or(""))
            }
            WhatsAppContent::Video(video) => {
                format!("[Video] {}", video.caption.as_deref().unwrap_or(""))
            }
            WhatsAppContent::Audio(_) => "[Audio]".to_string(),
            WhatsAppContent::Voice(voice) => {
                format!(
                    "[Voice message, {}s]",
                    voice.duration.map(|d| d.to_string()).unwrap_or_default()
                )
            }
            WhatsAppContent::Document(doc) => {
                format!(
                    "[Document: {}] {}",
                    doc.filename.as_deref().unwrap_or("unnamed"),
                    doc.caption.as_deref().unwrap_or("")
                )
            }
            WhatsAppContent::Location(loc) => {
                format!("[Location: {}, {}]", loc.latitude, loc.longitude)
            }
            WhatsAppContent::Contacts(contacts) => {
                let names: Vec<String> = contacts
                    .contacts
                    .iter()
                    .map(|c| c.display_name.clone())
                    .collect();
                format!("[Contacts: {}]", names.join(", "))
            }
            WhatsAppContent::Sticker(_) => "[Sticker]".to_string(),
            WhatsAppContent::Interactive(_) => "[Interactive message]".to_string(),
            WhatsAppContent::Template(template) => {
                format!("[Template: {}]", template.name)
            }
            WhatsAppContent::Reaction(reaction) => {
                format!("[Reaction: {}]", reaction.emoji)
            }
        }
    }

    /// Get content type string
    pub fn get_content_type(content: &WhatsAppContent) -> &'static str {
        match content {
            WhatsAppContent::Text(_) => "text",
            WhatsAppContent::Image(_) => "image",
            WhatsAppContent::Video(_) => "video",
            WhatsAppContent::Audio(_) => "audio",
            WhatsAppContent::Voice(_) => "voice",
            WhatsAppContent::Document(_) => "document",
            WhatsAppContent::Location(_) => "location",
            WhatsAppContent::Contacts(_) => "contacts",
            WhatsAppContent::Sticker(_) => "sticker",
            WhatsAppContent::Interactive(_) => "interactive",
            WhatsAppContent::Template(_) => "template",
            WhatsAppContent::Reaction(_) => "reaction",
        }
    }

    /// Create text content
    pub fn create_text(body: impl Into<String>) -> WhatsAppContent {
        WhatsAppContent::Text(WhatsAppTextContent {
            body: body.into(),
            preview_url: None,
            quoted_message: None,
        })
    }

    /// Create image content
    pub fn create_image(file_path: impl Into<String>, caption: Option<String>) -> WhatsAppContent {
        WhatsAppContent::Image(WhatsAppImageContent {
            file_path: Some(file_path.into()),
            url: None,
            caption,
            mime_type: None,
            width: None,
            height: None,
            file_size: None,
        })
    }

    /// Create video content
    pub fn create_video(file_path: impl Into<String>, caption: Option<String>) -> WhatsAppContent {
        WhatsAppContent::Video(WhatsAppVideoContent {
            file_path: Some(file_path.into()),
            url: None,
            caption,
            mime_type: None,
            width: None,
            height: None,
            duration: None,
            file_size: None,
        })
    }

    /// Create document content
    pub fn create_document(
        file_path: impl Into<String>,
        filename: Option<String>,
        caption: Option<String>,
    ) -> WhatsAppContent {
        WhatsAppContent::Document(WhatsAppDocumentContent {
            file_path: Some(file_path.into()),
            url: None,
            caption,
            filename,
            mime_type: None,
            file_size: None,
        })
    }

    /// Create location content
    pub fn create_location(
        latitude: f64,
        longitude: f64,
        name: Option<String>,
        address: Option<String>,
    ) -> WhatsAppContent {
        WhatsAppContent::Location(WhatsAppLocationContent {
            latitude,
            longitude,
            name,
            address,
            url: None,
        })
    }

    /// Serialize content to JSON string
    pub fn to_json(content: &WhatsAppContent) -> Result<String> {
        serde_json::to_string(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Serialize content to JSON value
    pub fn to_json_value(content: &WhatsAppContent) -> Result<serde_json::Value> {
        serde_json::to_value(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Check if content has media
    pub fn has_media(content: &WhatsAppContent) -> bool {
        matches!(
            content,
            WhatsAppContent::Image(_)
                | WhatsAppContent::Video(_)
                | WhatsAppContent::Audio(_)
                | WhatsAppContent::Voice(_)
                | WhatsAppContent::Document(_)
                | WhatsAppContent::Sticker(_)
        )
    }

    /// Get media file path if available
    pub fn get_media_path(content: &WhatsAppContent) -> Option<&str> {
        match content {
            WhatsAppContent::Image(img) => img.file_path.as_deref(),
            WhatsAppContent::Video(vid) => vid.file_path.as_deref(),
            WhatsAppContent::Audio(aud) => aud.file_path.as_deref(),
            WhatsAppContent::Voice(voice) => voice.file_path.as_deref(),
            WhatsAppContent::Document(doc) => doc.file_path.as_deref(),
            WhatsAppContent::Sticker(sticker) => sticker.file_path.as_deref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_content() {
        let json = serde_json::json!({
            "body": "Hello, WhatsApp!"
        });
        let content = WhatsAppContentParser::parse("text", json).unwrap();
        assert!(matches!(content, WhatsAppContent::Text(_)));
        assert_eq!(
            WhatsAppContentParser::extract_text(&content),
            "Hello, WhatsApp!"
        );
    }

    #[test]
    fn test_parse_image_content() {
        let json = serde_json::json!({
            "file_path": "/path/to/image.jpg",
            "caption": "My photo"
        });
        let content = WhatsAppContentParser::parse("image", json).unwrap();
        assert!(matches!(content, WhatsAppContent::Image(_)));
        let text = WhatsAppContentParser::extract_text(&content);
        assert!(text.contains("[Image]"));
        assert!(text.contains("My photo"));
    }

    #[test]
    fn test_parse_location_content() {
        let json = serde_json::json!({
            "latitude": 37.7749,
            "longitude": -122.4194,
            "name": "San Francisco"
        });
        let content = WhatsAppContentParser::parse("location", json).unwrap();
        assert!(matches!(content, WhatsAppContent::Location(_)));
        let text = WhatsAppContentParser::extract_text(&content);
        assert!(text.contains("37.7749"));
        assert!(text.contains("-122.4194"));
    }

    #[test]
    fn test_create_text() {
        let content = WhatsAppContentParser::create_text("Test message");
        assert!(
            matches!(content, WhatsAppContent::Text(WhatsAppTextContent { body, .. }) if body == "Test message")
        );
    }

    #[test]
    fn test_create_location() {
        let content = WhatsAppContentParser::create_location(
            37.7749,
            -122.4194,
            Some("SF".to_string()),
            None,
        );
        assert!(matches!(content, WhatsAppContent::Location(_)));
        if let WhatsAppContent::Location(loc) = content {
            assert_eq!(loc.latitude, 37.7749);
            assert_eq!(loc.longitude, -122.4194);
            assert_eq!(loc.name, Some("SF".to_string()));
        }
    }

    #[test]
    fn test_get_content_type() {
        let text = WhatsAppContent::Text(WhatsAppTextContent::default());
        assert_eq!(WhatsAppContentParser::get_content_type(&text), "text");

        let image = WhatsAppContent::Image(WhatsAppImageContent::default());
        assert_eq!(WhatsAppContentParser::get_content_type(&image), "image");

        let voice = WhatsAppContent::Voice(WhatsAppVoiceContent::default());
        assert_eq!(WhatsAppContentParser::get_content_type(&voice), "voice");
    }

    #[test]
    fn test_has_media() {
        let text = WhatsAppContent::Text(WhatsAppTextContent::default());
        assert!(!WhatsAppContentParser::has_media(&text));

        let image = WhatsAppContent::Image(WhatsAppImageContent::default());
        assert!(WhatsAppContentParser::has_media(&image));

        let voice = WhatsAppContent::Voice(WhatsAppVoiceContent::default());
        assert!(WhatsAppContentParser::has_media(&voice));
    }
}

// =============================================================================
// 🟢 P0 FIX: PlatformContent trait implementation for unified content framework
// =============================================================================

impl PlatformContent for WhatsAppContent {
    fn content_type(&self) -> UnifiedContentType {
        match self {
            WhatsAppContent::Text(_) => UnifiedContentType::Text,
            WhatsAppContent::Image(_) => UnifiedContentType::Image,
            WhatsAppContent::Video(_) => UnifiedContentType::Video,
            WhatsAppContent::Audio(_) => UnifiedContentType::Audio,
            WhatsAppContent::Voice(_) => UnifiedContentType::Audio,
            WhatsAppContent::Document(_) => UnifiedContentType::File,
            WhatsAppContent::Location(_) => UnifiedContentType::Location,
            WhatsAppContent::Contacts(_) => UnifiedContentType::Contact,
            WhatsAppContent::Sticker(_) => UnifiedContentType::Sticker,
            WhatsAppContent::Interactive(_) => UnifiedContentType::Card,
            WhatsAppContent::Template(_) => UnifiedContentType::Card,
            WhatsAppContent::Reaction(_) => UnifiedContentType::System,
        }
    }

    fn extract_text(&self) -> String {
        WhatsAppContentParser::extract_text(self)
    }
}

impl WhatsAppImageContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.url.clone().unwrap_or_default(),
            width: self.width,
            height: self.height,
            size: self.file_size,
            mime_type: self.mime_type.clone(),
            filename: None,
            caption: self.caption.clone(),
            thumbnail: None,
            duration: None,
        }
    }
}

impl WhatsAppVideoContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.url.clone().unwrap_or_default(),
            width: self.width,
            height: self.height,
            duration: self.duration,
            mime_type: self.mime_type.clone(),
            filename: None,
            caption: self.caption.clone(),
            size: self.file_size,
            thumbnail: None,
        }
    }
}

impl WhatsAppDocumentContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.url.clone().unwrap_or_default(),
            mime_type: self.mime_type.clone(),
            filename: self.filename.clone(),
            size: self.file_size,
            width: None,
            height: None,
            duration: None,
            caption: self.caption.clone(),
            thumbnail: None,
        }
    }
}
