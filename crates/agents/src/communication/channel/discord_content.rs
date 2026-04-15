//! Discord Message Content Parser
//!
//! Provides parsing and handling of various Discord message content types
//! including text, embeds, files, images, videos, stickers, and reactions.

use serde::{Deserialize, Serialize};

use crate::communication::channel::content::{
    ContentType as UnifiedContentType, MediaContent, PlatformContent,
};
use crate::error::{AgentError, Result};

/// Discord message content types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum DiscordContent {
    /// Plain text message
    #[serde(rename = "text")]
    Text(DiscordTextContent),
    /// Rich embed message
    #[serde(rename = "embed")]
    Embed(DiscordEmbedContent),
    /// File attachment
    #[serde(rename = "file")]
    File(DiscordFileContent),
    /// Image attachment
    #[serde(rename = "image")]
    Image(DiscordImageContent),
    /// Video attachment
    #[serde(rename = "video")]
    Video(DiscordVideoContent),
    /// Sticker
    #[serde(rename = "sticker")]
    Sticker(DiscordStickerContent),
    /// Reaction
    #[serde(rename = "reaction")]
    Reaction(DiscordReactionContent),
}

/// Text content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DiscordTextContent {
    /// The text content
    pub text: String,
    /// Whether this message uses text-to-speech
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
    /// Allowed mentions configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mentions: Option<AllowedMentions>,
    /// Message reference (for replies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
}

/// Allowed mentions configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AllowedMentions {
    /// An array of allowed mention types to parse from the content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse: Option<Vec<String>>,
    /// Array of role IDs to mention
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    /// Array of user IDs to mention
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<String>>,
    /// For replies, whether to mention the author of the message being replied
    /// to
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "replied_user")]
    pub replied_user: Option<bool>,
}

/// Message reference for replies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageReference {
    /// ID of the originating message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// ID of the originating message's channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// ID of the originating message's guild
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    /// When sending, whether to error if the referenced message doesn't exist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_if_not_exists: Option<bool>,
}

/// Embed content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DiscordEmbedContent {
    /// Title of embed (up to 256 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Type of embed (always "rich" for webhook embeds)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub embed_type: Option<String>,
    /// Description of embed (up to 4096 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// URL of embed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Timestamp of embed content (ISO8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Color code of the embed (integer representation of hex color)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<i32>,
    /// Footer information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<EmbedFooter>,
    /// Image information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<EmbedImage>,
    /// Thumbnail information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<EmbedThumbnail>,
    /// Video information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<EmbedVideo>,
    /// Provider information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<EmbedProvider>,
    /// Author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<EmbedAuthor>,
    /// Fields information (up to 25 fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<EmbedField>>,
}

/// Embed footer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedFooter {
    /// Footer text (up to 2048 characters)
    pub text: String,
    /// URL of footer icon (only supports http(s) and attachments)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "icon_url")]
    pub icon_url: Option<String>,
    /// Proxied URL of footer icon
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "proxy_icon_url")]
    pub proxy_icon_url: Option<String>,
}

/// Embed image
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedImage {
    /// Source URL of image (only supports http(s) and attachments)
    pub url: String,
    /// Proxied URL of image
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "proxy_url")]
    pub proxy_url: Option<String>,
    /// Height of image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Width of image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
}

/// Embed thumbnail
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedThumbnail {
    /// Source URL of thumbnail (only supports http(s) and attachments)
    pub url: String,
    /// Proxied URL of thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "proxy_url")]
    pub proxy_url: Option<String>,
    /// Height of thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Width of thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
}

/// Embed video
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedVideo {
    /// Source URL of video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Proxied URL of video
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "proxy_url")]
    pub proxy_url: Option<String>,
    /// Height of video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Width of video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
}

/// Embed provider
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedProvider {
    /// Name of provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL of provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Embed author
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedAuthor {
    /// Name of author (up to 256 characters)
    pub name: String,
    /// URL of author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// URL of author icon (only supports http(s) and attachments)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "icon_url")]
    pub icon_url: Option<String>,
    /// Proxied URL of author icon
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "proxy_icon_url")]
    pub proxy_icon_url: Option<String>,
}

/// Embed field
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedField {
    /// Name of the field (up to 256 characters)
    pub name: String,
    /// Value of the field (up to 1024 characters)
    pub value: String,
    /// Whether this field should display inline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<bool>,
}

/// File content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DiscordFileContent {
    /// File name
    pub filename: String,
    /// File description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Content type (MIME type)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "content_type")]
    pub content_type: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// File URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Whether this file is a spoiler
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spoiler: Option<bool>,
}

/// Image content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DiscordImageContent {
    /// Image URL
    pub url: String,
    /// Image width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// Image height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Image size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// Content type
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "content_type")]
    pub content_type: Option<String>,
    /// Caption/alt text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

/// Video content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DiscordVideoContent {
    /// Video URL
    pub url: String,
    /// Video width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// Video height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Video duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    /// Video size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// Content type
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "content_type")]
    pub content_type: Option<String>,
    /// Caption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

/// Sticker content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DiscordStickerContent {
    /// Sticker ID
    pub id: String,
    /// Sticker name
    pub name: String,
    /// Sticker format type
    #[serde(rename = "format_type")]
    pub format_type: i32,
    /// Associated emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    /// Sticker URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Reaction content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DiscordReactionContent {
    /// Emoji information
    pub emoji: ReactionEmoji,
    /// Count of reactions
    pub count: i32,
    /// Whether the current user reacted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub me: Option<bool>,
    /// Message ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Channel ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
}

/// Reaction emoji
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReactionEmoji {
    /// Emoji ID (null for Unicode emojis)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Emoji name
    pub name: String,
    /// Whether this emoji is animated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animated: Option<bool>,
}

/// Discord content parser
#[derive(Debug, Clone, Default)]
pub struct DiscordContentParser;

impl DiscordContentParser {
    /// Create a new content parser
    pub fn new() -> Self {
        Self
    }

    /// Parse content from JSON value
    ///
    /// # Arguments
    /// * `content_type` - The content type (text, embed, file, image, video,
    ///   sticker, reaction)
    /// * `content` - The JSON content to parse
    ///
    /// # Returns
    /// Parsed DiscordContent enum variant
    pub fn parse(content_type: &str, content: serde_json::Value) -> Result<DiscordContent> {
        match content_type {
            "text" => {
                let text_content: DiscordTextContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse text content: {}", e))
                    })?;
                Ok(DiscordContent::Text(text_content))
            }
            "embed" => {
                let embed_content: DiscordEmbedContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse embed content: {}", e))
                    })?;
                Ok(DiscordContent::Embed(embed_content))
            }
            "file" => {
                let file_content: DiscordFileContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse file content: {}", e))
                    })?;
                Ok(DiscordContent::File(file_content))
            }
            "image" => {
                let image_content: DiscordImageContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse image content: {}", e))
                    })?;
                Ok(DiscordContent::Image(image_content))
            }
            "video" => {
                let video_content: DiscordVideoContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse video content: {}", e))
                    })?;
                Ok(DiscordContent::Video(video_content))
            }
            "sticker" => {
                let sticker_content: DiscordStickerContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse sticker content: {}", e))
                    })?;
                Ok(DiscordContent::Sticker(sticker_content))
            }
            "reaction" => {
                let reaction_content: DiscordReactionContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse reaction content: {}", e))
                    })?;
                Ok(DiscordContent::Reaction(reaction_content))
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
    /// Parsed DiscordContent enum variant
    pub fn parse_str(content_type: &str, content_json: &str) -> Result<DiscordContent> {
        let content: serde_json::Value = serde_json::from_str(content_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON: {}", e)))?;
        Self::parse(content_type, content)
    }

    /// Extract plain text from any content type
    ///
    /// # Arguments
    /// * `content` - The DiscordContent to extract text from
    ///
    /// # Returns
    /// Extracted plain text string
    pub fn extract_text(content: &DiscordContent) -> String {
        match content {
            DiscordContent::Text(text) => text.text.clone(),
            DiscordContent::Embed(embed) => {
                let mut texts = Vec::new();
                if let Some(title) = &embed.title {
                    texts.push(title.clone());
                }
                if let Some(description) = &embed.description {
                    texts.push(description.clone());
                }
                if let Some(fields) = &embed.fields {
                    for field in fields {
                        texts.push(format!("{}: {}", field.name, field.value));
                    }
                }
                texts.join("\n")
            }
            DiscordContent::File(file) => {
                format!("[File: {}]", file.filename)
            }
            DiscordContent::Image(image) => {
                format!(
                    "[Image] {}",
                    image.caption.as_deref().unwrap_or("(no caption)")
                )
            }
            DiscordContent::Video(video) => {
                format!(
                    "[Video: {}s] {}",
                    video
                        .duration
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "?".to_string()),
                    video.caption.as_deref().unwrap_or("(no caption)")
                )
            }
            DiscordContent::Sticker(sticker) => {
                format!("[Sticker: {}]", sticker.name)
            }
            DiscordContent::Reaction(reaction) => {
                format!("[Reaction: {}] x{}", reaction.emoji.name, reaction.count)
            }
        }
    }

    /// Get content type string
    ///
    /// # Arguments
    /// * `content` - The DiscordContent
    ///
    /// # Returns
    /// Content type string
    pub fn get_content_type(content: &DiscordContent) -> &'static str {
        match content {
            DiscordContent::Text(_) => "text",
            DiscordContent::Embed(_) => "embed",
            DiscordContent::File(_) => "file",
            DiscordContent::Image(_) => "image",
            DiscordContent::Video(_) => "video",
            DiscordContent::Sticker(_) => "sticker",
            DiscordContent::Reaction(_) => "reaction",
        }
    }

    /// Create text content
    ///
    /// # Arguments
    /// * `text` - The text content
    ///
    /// # Returns
    /// DiscordContent::Text variant
    pub fn create_text(text: impl Into<String>) -> DiscordContent {
        DiscordContent::Text(DiscordTextContent {
            text: text.into(),
            tts: None,
            allowed_mentions: None,
            message_reference: None,
        })
    }

    /// Create embed content
    ///
    /// # Arguments
    /// * `embed` - The embed content
    ///
    /// # Returns
    /// DiscordContent::Embed variant
    pub fn create_embed(embed: DiscordEmbedContent) -> DiscordContent {
        DiscordContent::Embed(embed)
    }

    /// Create file content
    ///
    /// # Arguments
    /// * `filename` - The file name
    /// * `url` - Optional file URL
    ///
    /// # Returns
    /// DiscordContent::File variant
    pub fn create_file(filename: impl Into<String>, url: Option<String>) -> DiscordContent {
        DiscordContent::File(DiscordFileContent {
            filename: filename.into(),
            description: None,
            content_type: None,
            size: None,
            url,
            spoiler: None,
        })
    }

    /// Create image content
    ///
    /// # Arguments
    /// * `url` - The image URL
    ///
    /// # Returns
    /// DiscordContent::Image variant
    pub fn create_image(url: impl Into<String>) -> DiscordContent {
        DiscordContent::Image(DiscordImageContent {
            url: url.into(),
            width: None,
            height: None,
            size: None,
            content_type: None,
            caption: None,
        })
    }

    /// Create video content
    ///
    /// # Arguments
    /// * `url` - The video URL
    ///
    /// # Returns
    /// DiscordContent::Video variant
    pub fn create_video(url: impl Into<String>) -> DiscordContent {
        DiscordContent::Video(DiscordVideoContent {
            url: url.into(),
            width: None,
            height: None,
            duration: None,
            size: None,
            content_type: None,
            caption: None,
        })
    }

    /// Create sticker content
    ///
    /// # Arguments
    /// * `id` - Sticker ID
    /// * `name` - Sticker name
    ///
    /// # Returns
    /// DiscordContent::Sticker variant
    pub fn create_sticker(id: impl Into<String>, name: impl Into<String>) -> DiscordContent {
        DiscordContent::Sticker(DiscordStickerContent {
            id: id.into(),
            name: name.into(),
            format_type: 1, // PNG
            emoji: None,
            url: None,
        })
    }

    /// Create reaction content
    ///
    /// # Arguments
    /// * `emoji_name` - Emoji name
    /// * `count` - Reaction count
    ///
    /// # Returns
    /// DiscordContent::Reaction variant
    pub fn create_reaction(emoji_name: impl Into<String>, count: i32) -> DiscordContent {
        DiscordContent::Reaction(DiscordReactionContent {
            emoji: ReactionEmoji {
                id: None,
                name: emoji_name.into(),
                animated: None,
            },
            count,
            me: None,
            message_id: None,
            channel_id: None,
        })
    }

    /// Serialize content to JSON string
    ///
    /// # Arguments
    /// * `content` - The DiscordContent to serialize
    ///
    /// # Returns
    /// JSON string representation
    pub fn to_json(content: &DiscordContent) -> Result<String> {
        serde_json::to_string(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Serialize content to JSON value
    ///
    /// # Arguments
    /// * `content` - The DiscordContent to serialize
    ///
    /// # Returns
    /// JSON value representation
    pub fn to_json_value(content: &DiscordContent) -> Result<serde_json::Value> {
        serde_json::to_value(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }
}

/// Embed builder for easy embed creation
pub struct EmbedBuilder {
    embed: DiscordEmbedContent,
}

impl EmbedBuilder {
    /// Create a new embed builder
    pub fn new() -> Self {
        Self {
            embed: DiscordEmbedContent {
                embed_type: Some("rich".to_string()),
                ..Default::default()
            },
        }
    }

    /// Set the title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.embed.title = Some(title.into());
        self
    }

    /// Set the description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.embed.description = Some(description.into());
        self
    }

    /// Set the URL
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.embed.url = Some(url.into());
        self
    }

    /// Set the color (hex color code)
    pub fn color(mut self, color: i32) -> Self {
        self.embed.color = Some(color);
        self
    }

    /// Set the timestamp
    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.embed.timestamp = Some(timestamp.into());
        self
    }

    /// Set the timestamp to current time
    pub fn timestamp_now(mut self) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        self.embed.timestamp = Some(now);
        self
    }

    /// Set the footer
    pub fn footer(mut self, text: impl Into<String>, icon_url: Option<String>) -> Self {
        self.embed.footer = Some(EmbedFooter {
            text: text.into(),
            icon_url,
            proxy_icon_url: None,
        });
        self
    }

    /// Set the image
    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.embed.image = Some(EmbedImage {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        });
        self
    }

    /// Set the thumbnail
    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.embed.thumbnail = Some(EmbedThumbnail {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
        });
        self
    }

    /// Set the author
    pub fn author(
        mut self,
        name: impl Into<String>,
        url: Option<String>,
        icon_url: Option<String>,
    ) -> Self {
        self.embed.author = Some(EmbedAuthor {
            name: name.into(),
            url,
            icon_url,
            proxy_icon_url: None,
        });
        self
    }

    /// Add a field
    pub fn field(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
        inline: bool,
    ) -> Self {
        let field = EmbedField {
            name: name.into(),
            value: value.into(),
            inline: Some(inline),
        };

        if let Some(fields) = &mut self.embed.fields {
            fields.push(field);
        } else {
            self.embed.fields = Some(vec![field]);
        }
        self
    }

    /// Build the embed
    pub fn build(self) -> DiscordEmbedContent {
        self.embed
    }

    /// Build and wrap in DiscordContent
    pub fn build_content(self) -> DiscordContent {
        DiscordContent::Embed(self.embed)
    }
}

impl Default for EmbedBuilder {
    fn default() -> Self {
        Self::new()
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
        let content = DiscordContentParser::parse("text", json).unwrap();
        assert!(matches!(content, DiscordContent::Text(_)));
        assert_eq!(
            DiscordContentParser::extract_text(&content),
            "Hello, World!"
        );
    }

    #[test]
    fn test_parse_embed_content() {
        let json = serde_json::json!({
            "title": "Test Embed",
            "description": "This is a test embed",
            "color": 0x00FF00,
            "fields": [
                {
                    "name": "Field 1",
                    "value": "Value 1",
                    "inline": true
                }
            ]
        });
        let content = DiscordContentParser::parse("embed", json).unwrap();
        assert!(matches!(content, DiscordContent::Embed(_)));
        let text = DiscordContentParser::extract_text(&content);
        assert!(text.contains("Test Embed"));
        assert!(text.contains("Field 1"));
    }

    #[test]
    fn test_parse_file_content() {
        let json = serde_json::json!({
            "filename": "document.pdf",
            "size": 1024,
            "url": "https://cdn.discordapp.com/attachments/..."
        });
        let content = DiscordContentParser::parse("file", json).unwrap();
        assert!(matches!(content, DiscordContent::File(_)));
        let text = DiscordContentParser::extract_text(&content);
        assert!(text.contains("document.pdf"));
    }

    #[test]
    fn test_embed_builder() {
        let embed = EmbedBuilder::new()
            .title("Test Title")
            .description("Test Description")
            .color(0xFF0000)
            .field("Field 1", "Value 1", true)
            .field("Field 2", "Value 2", false)
            .footer("Footer text", None)
            .build();

        assert_eq!(embed.title, Some("Test Title".to_string()));
        assert_eq!(embed.description, Some("Test Description".to_string()));
        assert_eq!(embed.color, Some(0xFF0000));
        assert_eq!(embed.fields.as_ref().map(|f| f.len()), Some(2));
        assert!(embed.footer.is_some());
    }

    #[test]
    fn test_get_content_type() {
        let text = DiscordContent::Text(DiscordTextContent::default());
        assert_eq!(DiscordContentParser::get_content_type(&text), "text");

        let embed = DiscordContent::Embed(DiscordEmbedContent::default());
        assert_eq!(DiscordContentParser::get_content_type(&embed), "embed");

        let file = DiscordContent::File(DiscordFileContent::default());
        assert_eq!(DiscordContentParser::get_content_type(&file), "file");
    }

    #[test]
    fn test_create_text() {
        let content = DiscordContentParser::create_text("Test message");
        assert!(
            matches!(content, DiscordContent::Text(DiscordTextContent { text, .. }) if text == "Test message")
        );
    }

    #[test]
    fn test_create_image() {
        let content = DiscordContentParser::create_image("https://example.com/image.png");
        assert!(
            matches!(content, DiscordContent::Image(DiscordImageContent { url, .. }) if url == "https://example.com/image.png")
        );
    }

    #[test]
    fn test_serialize_content() {
        let content = DiscordContentParser::create_text("Test");
        let json = DiscordContentParser::to_json(&content).unwrap();
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_allowed_mentions() {
        let mentions = AllowedMentions {
            parse: Some(vec!["users".to_string(), "roles".to_string()]),
            roles: None,
            users: None,
            replied_user: Some(true),
        };

        let json = serde_json::to_string(&mentions).unwrap();
        assert!(json.contains("users"));
        assert!(json.contains("roles"));
    }
}

// =============================================================================
// 🟢 P0 FIX: PlatformContent trait implementation for unified content framework
// =============================================================================

impl PlatformContent for DiscordContent {
    fn content_type(&self) -> UnifiedContentType {
        match self {
            DiscordContent::Text(_) => UnifiedContentType::Text,
            DiscordContent::Embed(_) => UnifiedContentType::Rich,
            DiscordContent::File(_) => UnifiedContentType::File,
            DiscordContent::Image(_) => UnifiedContentType::Image,
            DiscordContent::Video(_) => UnifiedContentType::Video,
            DiscordContent::Sticker(_) => UnifiedContentType::Sticker,
            DiscordContent::Reaction(_) => UnifiedContentType::System,
        }
    }

    fn extract_text(&self) -> String {
        DiscordContentParser::extract_text(self)
    }
}

impl DiscordImageContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.url.clone(),
            width: self.width,
            height: self.height,
            size: self.size,
            mime_type: self.content_type.clone(),
            filename: None,
            caption: self.caption.clone(),
            thumbnail: None,
            duration: None,
        }
    }
}

impl DiscordVideoContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.url.clone(),
            width: self.width,
            height: self.height,
            duration: self.duration,
            mime_type: self.content_type.clone(),
            filename: None,
            caption: self.caption.clone(),
            size: self.size,
            thumbnail: None,
        }
    }
}

impl DiscordFileContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.url.clone().unwrap_or_default(),
            mime_type: self.content_type.clone(),
            filename: Some(self.filename.clone()),
            size: self.size,
            width: None,
            height: None,
            duration: None,
            caption: self.description.clone(),
            thumbnail: None,
        }
    }
}
