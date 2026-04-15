//! Microsoft Teams Message Content Parser
//!
//! Provides parsing and handling of various Teams message content types
//! including text, HTML, Adaptive Cards, Hero Cards, and attachments.

use serde::{Deserialize, Serialize};

use crate::communication::channel::content::{
    ContentType as UnifiedContentType, MediaContent, PlatformContent,
};
use crate::error::{AgentError, Result};

/// Teams message content types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum TeamsContent {
    /// Plain text message
    #[serde(rename = "text")]
    Text(TeamsTextContent),
    /// HTML formatted message
    #[serde(rename = "html")]
    Html(TeamsHtmlContent),
    /// Adaptive Card
    #[serde(rename = "adaptive_card")]
    AdaptiveCard(TeamsAdaptiveCardContent),
    /// Hero Card
    #[serde(rename = "hero_card")]
    HeroCard(TeamsHeroCardContent),
    /// Thumbnail Card
    #[serde(rename = "thumbnail_card")]
    ThumbnailCard(TeamsThumbnailCardContent),
    /// File attachment
    #[serde(rename = "file")]
    File(TeamsFileContent),
    /// Image attachment
    #[serde(rename = "image")]
    Image(TeamsImageContent),
    /// Video attachment
    #[serde(rename = "video")]
    Video(TeamsVideoContent),
    /// Audio attachment
    #[serde(rename = "audio")]
    Audio(TeamsAudioContent),
    /// Rich text with mentions
    #[serde(rename = "rich_text")]
    RichText(TeamsRichTextContent),
}

/// Text content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsTextContent {
    /// The text content (supports markdown)
    pub text: String,
    /// Text format
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "textFormat")]
    pub text_format: Option<String>,
}

/// HTML content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsHtmlContent {
    /// The HTML content
    pub html: String,
    /// Plain text version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Adaptive Card content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsAdaptiveCardContent {
    /// Adaptive Card version
    pub version: String,
    /// Card body elements
    pub body: Vec<serde_json::Value>,
    /// Card actions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<serde_json::Value>>,
    /// Card schema
    #[serde(rename = "$schema")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Fallback text
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fallbackText")]
    pub fallback_text: Option<String>,
    /// Speak text (for accessibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speak: Option<String>,
    /// Background image
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "backgroundImage")]
    pub background_image: Option<String>,
    /// Min height
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "minHeight")]
    pub min_height: Option<String>,
    /// RTL support
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "rtl")]
    pub rtl: Option<bool>,
}

/// Hero Card content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsHeroCardContent {
    /// Card title
    pub title: String,
    /// Card subtitle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// Card text
    pub text: String,
    /// Card images
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<TeamsCardImage>>,
    /// Card buttons
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<TeamsCardAction>>,
    /// Tap action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tap: Option<TeamsCardAction>,
}

/// Thumbnail Card content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsThumbnailCardContent {
    /// Card title
    pub title: String,
    /// Card subtitle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// Card text
    pub text: String,
    /// Card images
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<TeamsCardImage>>,
    /// Card buttons
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<TeamsCardAction>>,
    /// Tap action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tap: Option<TeamsCardAction>,
}

/// Card image
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamsCardImage {
    /// Image URL
    pub url: String,
    /// Alt text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    /// Tap action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tap: Option<TeamsCardAction>,
}

/// Card action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamsCardAction {
    /// Action type
    #[serde(rename = "type")]
    pub action_type: String,
    /// Action title
    pub title: String,
    /// Action value
    pub value: String,
    /// Image URL (for image buttons)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Display text
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "displayText")]
    pub display_text: Option<String>,
}

/// File content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsFileContent {
    /// File name
    pub name: String,
    /// Content URL
    #[serde(rename = "contentUrl")]
    pub content_url: String,
    /// Content type
    #[serde(rename = "contentType")]
    pub content_type: String,
    /// File size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// Thumbnail URL
    #[serde(rename = "thumbnailUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
}

/// Image content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsImageContent {
    /// Image URL
    pub url: String,
    /// Alt text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    /// Image width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// Image height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Image size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
}

/// Video content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsVideoContent {
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
    /// Thumbnail URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    /// Title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Audio content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsAudioContent {
    /// Audio URL
    pub url: String,
    /// Audio duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    /// Title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Rich text content structure (with mentions)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TeamsRichTextContent {
    /// Rich text content
    pub text: String,
    /// Mentions in the text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<TeamsMention>>,
    /// Text format
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "textFormat")]
    pub text_format: Option<String>,
}

/// Teams mention
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamsMention {
    /// Mention type (person, channel, team)
    #[serde(rename = "type")]
    pub mention_type: String,
    /// Mentioned item ID
    pub id: String,
    /// Mentioned item name
    pub name: String,
    /// Text that represents the mention in the message
    pub text: String,
}

/// Teams content parser
#[derive(Debug, Clone, Default)]
pub struct TeamsContentParser;

impl TeamsContentParser {
    /// Create a new content parser
    pub fn new() -> Self {
        Self
    }

    /// Parse content from JSON value
    ///
    /// # Arguments
    /// * `content_type` - The content type (text, html, adaptive_card,
    ///   hero_card, etc.)
    /// * `content` - The JSON content to parse
    ///
    /// # Returns
    /// Parsed TeamsContent enum variant
    pub fn parse(content_type: &str, content: serde_json::Value) -> Result<TeamsContent> {
        match content_type {
            "text" => {
                let text_content: TeamsTextContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse text content: {}", e))
                    })?;
                Ok(TeamsContent::Text(text_content))
            }
            "html" => {
                let html_content: TeamsHtmlContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse HTML content: {}", e))
                    })?;
                Ok(TeamsContent::Html(html_content))
            }
            "adaptive_card" => {
                let card_content: TeamsAdaptiveCardContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!(
                            "Failed to parse Adaptive Card content: {}",
                            e
                        ))
                    })?;
                Ok(TeamsContent::AdaptiveCard(card_content))
            }
            "hero_card" => {
                let hero_content: TeamsHeroCardContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse Hero Card content: {}", e))
                    })?;
                Ok(TeamsContent::HeroCard(hero_content))
            }
            "thumbnail_card" => {
                let thumb_content: TeamsThumbnailCardContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!(
                            "Failed to parse Thumbnail Card content: {}",
                            e
                        ))
                    })?;
                Ok(TeamsContent::ThumbnailCard(thumb_content))
            }
            "file" => {
                let file_content: TeamsFileContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse file content: {}", e))
                    })?;
                Ok(TeamsContent::File(file_content))
            }
            "image" => {
                let image_content: TeamsImageContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse image content: {}", e))
                    })?;
                Ok(TeamsContent::Image(image_content))
            }
            "video" => {
                let video_content: TeamsVideoContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse video content: {}", e))
                    })?;
                Ok(TeamsContent::Video(video_content))
            }
            "audio" => {
                let audio_content: TeamsAudioContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse audio content: {}", e))
                    })?;
                Ok(TeamsContent::Audio(audio_content))
            }
            "rich_text" => {
                let rich_content: TeamsRichTextContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse rich text content: {}", e))
                    })?;
                Ok(TeamsContent::RichText(rich_content))
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
    /// Parsed TeamsContent enum variant
    pub fn parse_str(content_type: &str, content_json: &str) -> Result<TeamsContent> {
        let content: serde_json::Value = serde_json::from_str(content_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON: {}", e)))?;
        Self::parse(content_type, content)
    }

    /// Extract plain text from any content type
    ///
    /// # Arguments
    /// * `content` - The TeamsContent to extract text from
    ///
    /// # Returns
    /// Extracted plain text string
    pub fn extract_text(content: &TeamsContent) -> String {
        match content {
            TeamsContent::Text(text) => text.text.clone(),
            TeamsContent::Html(html) => {
                // Strip HTML tags
                Self::strip_html(&html.html)
            }
            TeamsContent::AdaptiveCard(card) => {
                // Extract text from card body
                let mut texts = Vec::new();
                for element in &card.body {
                    if let Some(text) = element.get("text").and_then(|t| t.as_str()) {
                        texts.push(text.to_string());
                    }
                }
                texts.join(" ")
            }
            TeamsContent::HeroCard(hero) => {
                let mut texts = vec![hero.title.clone(), hero.text.clone()];
                if let Some(sub) = &hero.subtitle {
                    texts.push(sub.clone());
                }
                texts.join(" - ")
            }
            TeamsContent::ThumbnailCard(thumb) => {
                let mut texts = vec![thumb.title.clone(), thumb.text.clone()];
                if let Some(sub) = &thumb.subtitle {
                    texts.push(sub.clone());
                }
                texts.join(" - ")
            }
            TeamsContent::File(file) => {
                format!("[File: {}]", file.name)
            }
            TeamsContent::Image(image) => {
                format!(
                    "[Image] {}",
                    image.alt.as_deref().unwrap_or("(no alt text)")
                )
            }
            TeamsContent::Video(video) => {
                format!(
                    "[Video: {}s] {}",
                    video
                        .duration
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "?".to_string()),
                    video.title.as_deref().unwrap_or("(no title)")
                )
            }
            TeamsContent::Audio(audio) => {
                format!(
                    "[Audio: {}s] {}",
                    audio
                        .duration
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "?".to_string()),
                    audio.title.as_deref().unwrap_or("(no title)")
                )
            }
            TeamsContent::RichText(rich) => rich.text.clone(),
        }
    }

    /// Get content type string
    ///
    /// # Arguments
    /// * `content` - The TeamsContent
    ///
    /// # Returns
    /// Content type string
    pub fn get_content_type(content: &TeamsContent) -> &'static str {
        match content {
            TeamsContent::Text(_) => "text",
            TeamsContent::Html(_) => "html",
            TeamsContent::AdaptiveCard(_) => "adaptive_card",
            TeamsContent::HeroCard(_) => "hero_card",
            TeamsContent::ThumbnailCard(_) => "thumbnail_card",
            TeamsContent::File(_) => "file",
            TeamsContent::Image(_) => "image",
            TeamsContent::Video(_) => "video",
            TeamsContent::Audio(_) => "audio",
            TeamsContent::RichText(_) => "rich_text",
        }
    }

    /// Create text content
    ///
    /// # Arguments
    /// * `text` - The text content
    ///
    /// # Returns
    /// TeamsContent::Text variant
    pub fn create_text(text: impl Into<String>) -> TeamsContent {
        TeamsContent::Text(TeamsTextContent {
            text: text.into(),
            text_format: Some("markdown".to_string()),
        })
    }

    /// Create HTML content
    ///
    /// # Arguments
    /// * `html` - The HTML content
    ///
    /// # Returns
    /// TeamsContent::Html variant
    pub fn create_html(html: impl Into<String>) -> TeamsContent {
        TeamsContent::Html(TeamsHtmlContent {
            html: html.into(),
            text: None,
        })
    }

    /// Create Adaptive Card content
    ///
    /// # Arguments
    /// * `body` - Card body elements
    ///
    /// # Returns
    /// TeamsContent::AdaptiveCard variant
    pub fn create_adaptive_card(body: Vec<serde_json::Value>) -> TeamsContent {
        TeamsContent::AdaptiveCard(TeamsAdaptiveCardContent {
            version: "1.4".to_string(),
            body,
            actions: None,
            schema: Some("http://adaptivecards.io/schemas/adaptive-card.json".to_string()),
            fallback_text: None,
            speak: None,
            background_image: None,
            min_height: None,
            rtl: None,
        })
    }

    /// Create Hero Card content
    ///
    /// # Arguments
    /// * `title` - Card title
    /// * `text` - Card text
    ///
    /// # Returns
    /// TeamsContent::HeroCard variant
    pub fn create_hero_card(title: impl Into<String>, text: impl Into<String>) -> TeamsContent {
        TeamsContent::HeroCard(TeamsHeroCardContent {
            title: title.into(),
            subtitle: None,
            text: text.into(),
            images: None,
            buttons: None,
            tap: None,
        })
    }

    /// Create file content
    ///
    /// # Arguments
    /// * `name` - File name
    /// * `url` - Content URL
    /// * `content_type` - MIME type
    ///
    /// # Returns
    /// TeamsContent::File variant
    pub fn create_file(
        name: impl Into<String>,
        url: impl Into<String>,
        content_type: impl Into<String>,
    ) -> TeamsContent {
        TeamsContent::File(TeamsFileContent {
            name: name.into(),
            content_url: url.into(),
            content_type: content_type.into(),
            size: None,
            thumbnail_url: None,
        })
    }

    /// Create image content
    ///
    /// # Arguments
    /// * `url` - Image URL
    ///
    /// # Returns
    /// TeamsContent::Image variant
    pub fn create_image(url: impl Into<String>) -> TeamsContent {
        TeamsContent::Image(TeamsImageContent {
            url: url.into(),
            alt: None,
            width: None,
            height: None,
            size: None,
        })
    }

    /// Create video content
    ///
    /// # Arguments
    /// * `url` - Video URL
    ///
    /// # Returns
    /// TeamsContent::Video variant
    pub fn create_video(url: impl Into<String>) -> TeamsContent {
        TeamsContent::Video(TeamsVideoContent {
            url: url.into(),
            width: None,
            height: None,
            duration: None,
            thumbnail: None,
            title: None,
        })
    }

    /// Create audio content
    ///
    /// # Arguments
    /// * `url` - Audio URL
    ///
    /// # Returns
    /// TeamsContent::Audio variant
    pub fn create_audio(url: impl Into<String>) -> TeamsContent {
        TeamsContent::Audio(TeamsAudioContent {
            url: url.into(),
            duration: None,
            title: None,
        })
    }

    /// Create rich text content
    ///
    /// # Arguments
    /// * `text` - Rich text content
    /// * `mentions` - Optional mentions
    ///
    /// # Returns
    /// TeamsContent::RichText variant
    pub fn create_rich_text(
        text: impl Into<String>,
        mentions: Option<Vec<TeamsMention>>,
    ) -> TeamsContent {
        TeamsContent::RichText(TeamsRichTextContent {
            text: text.into(),
            mentions,
            text_format: Some("xml".to_string()),
        })
    }

    /// Serialize content to JSON string
    ///
    /// # Arguments
    /// * `content` - The TeamsContent to serialize
    ///
    /// # Returns
    /// JSON string representation
    pub fn to_json(content: &TeamsContent) -> Result<String> {
        serde_json::to_string(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Serialize content to JSON value
    ///
    /// # Arguments
    /// * `content` - The TeamsContent to serialize
    ///
    /// # Returns
    /// JSON value representation
    pub fn to_json_value(content: &TeamsContent) -> Result<serde_json::Value> {
        serde_json::to_value(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Strip HTML tags from text
    fn strip_html(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;

        for c in html.chars() {
            match c {
                '<' => in_tag = true,
                '>' => {
                    in_tag = false;
                    result.push(' '); // Add space between tags
                }
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }

        // Clean up extra spaces
        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Convert markdown to HTML
    ///
    /// # Arguments
    /// * `markdown` - Markdown text
    ///
    /// # Returns
    /// HTML text
    pub fn markdown_to_html(markdown: &str) -> String {
        let mut html = markdown.to_string();

        // Bold
        html = html.replace("**", "<strong>");
        // This is a simple replacement, in production use a proper markdown parser
        // For now, just return the text with basic formatting

        // Code blocks
        if html.starts_with("```") && html.ends_with("```") {
            html = format!("<pre><code>{}</code></pre>", &html[3..html.len() - 3]);
        }

        // Inline code
        html = html.replace("`", "<code>");

        html
    }

    /// Escape special HTML characters
    ///
    /// # Arguments
    /// * `text` - Plain text
    ///
    /// # Returns
    /// Escaped HTML text
    pub fn escape_html(text: &str) -> String {
        text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#x27;")
    }
}

/// Adaptive Card builder for easy card creation
pub struct AdaptiveCardBuilder;

impl AdaptiveCardBuilder {
    /// Create a new Adaptive Card
    pub fn new() -> serde_json::Value {
        serde_json::json!({
            "type": "AdaptiveCard",
            "version": "1.4",
            "body": [],
            "actions": [],
        })
    }

    /// Add a text block
    pub fn add_text_block(
        card: &mut serde_json::Value,
        text: impl Into<String>,
        size: &str,
        weight: &str,
    ) {
        let body = card["body"].as_array_mut().unwrap();
        body.push(serde_json::json!({
            "type": "TextBlock",
            "text": text.into(),
            "size": size,
            "weight": weight,
        }));
    }

    /// Add a container
    pub fn add_container(card: &mut serde_json::Value, items: Vec<serde_json::Value>) {
        let body = card["body"].as_array_mut().unwrap();
        body.push(serde_json::json!({
            "type": "Container",
            "items": items,
        }));
    }

    /// Add an action button
    pub fn add_action(
        card: &mut serde_json::Value,
        title: impl Into<String>,
        action_type: &str,
        data: serde_json::Value,
    ) {
        let actions = card["actions"].as_array_mut().unwrap();
        actions.push(serde_json::json!({
            "type": action_type,
            "title": title.into(),
            "data": data,
        }));
    }

    /// Add an image
    pub fn add_image(
        card: &mut serde_json::Value,
        url: impl Into<String>,
        alt_text: impl Into<String>,
    ) {
        let body = card["body"].as_array_mut().unwrap();
        body.push(serde_json::json!({
            "type": "Image",
            "url": url.into(),
            "altText": alt_text.into(),
        }));
    }

    /// Add a fact set
    pub fn add_fact_set(card: &mut serde_json::Value, facts: Vec<(String, String)>) {
        let facts_json: Vec<serde_json::Value> = facts
            .into_iter()
            .map(|(title, value)| {
                serde_json::json!({
                    "title": title,
                    "value": value,
                })
            })
            .collect();

        let body = card["body"].as_array_mut().unwrap();
        body.push(serde_json::json!({
            "type": "FactSet",
            "facts": facts_json,
        }));
    }

    /// Add an input field
    pub fn add_input_text(
        card: &mut serde_json::Value,
        id: impl Into<String>,
        placeholder: impl Into<String>,
        is_multiline: bool,
    ) {
        let body = card["body"].as_array_mut().unwrap();
        body.push(serde_json::json!({
            "type": "Input.Text",
            "id": id.into(),
            "placeholder": placeholder.into(),
            "isMultiline": is_multiline,
        }));
    }

    /// Add a column set
    pub fn add_column_set(card: &mut serde_json::Value, columns: Vec<Vec<serde_json::Value>>) {
        let columns_json: Vec<serde_json::Value> = columns
            .into_iter()
            .map(|items| {
                serde_json::json!({
                    "type": "Column",
                    "items": items,
                })
            })
            .collect();

        let body = card["body"].as_array_mut().unwrap();
        body.push(serde_json::json!({
            "type": "ColumnSet",
            "columns": columns_json,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teams_content_parser() {
        let content = TeamsContentParser::create_text("Hello World");
        match content {
            TeamsContent::Text(text) => {
                assert_eq!(text.text, "Hello World");
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn test_extract_text() {
        let text_content = TeamsContent::Text(TeamsTextContent {
            text: "Hello".to_string(),
            text_format: None,
        });
        assert_eq!(TeamsContentParser::extract_text(&text_content), "Hello");

        let html_content = TeamsContent::Html(TeamsHtmlContent {
            html: "<p>Hello <strong>World</strong></p>".to_string(),
            text: None,
        });
        assert_eq!(
            TeamsContentParser::extract_text(&html_content),
            "Hello World"
        );
    }

    #[test]
    fn test_adaptive_card_builder() {
        let mut card = AdaptiveCardBuilder::new();
        AdaptiveCardBuilder::add_text_block(&mut card, "Title", "large", "bolder");
        AdaptiveCardBuilder::add_text_block(&mut card, "Body text", "medium", "default");

        assert_eq!(card["type"], "AdaptiveCard");
        assert_eq!(card["body"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_create_hero_card() {
        let content = TeamsContentParser::create_hero_card("Title", "Body text");
        match content {
            TeamsContent::HeroCard(hero) => {
                assert_eq!(hero.title, "Title");
                assert_eq!(hero.text, "Body text");
            }
            _ => panic!("Expected HeroCard content"),
        }
    }

    #[test]
    fn test_strip_html() {
        let html = "<p>Hello <strong>World</strong></p>";
        assert_eq!(TeamsContentParser::strip_html(html), "Hello World");

        let html2 = "<div><span>Test</span> content</div>";
        assert_eq!(TeamsContentParser::strip_html(html2), "Test content");
    }

    #[test]
    fn test_escape_html() {
        let text = "<script>alert('xss')</script>";
        let escaped = TeamsContentParser::escape_html(text);
        assert_eq!(
            escaped,
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_parse_adaptive_card() {
        let json = r#"{
            "version": "1.4",
            "body": [
                {"type": "TextBlock", "text": "Hello"},
                {"type": "Image", "url": "https://example.com/image.png"}
            ],
            "actions": [
                {"type": "Action.Submit", "title": "Submit"}
            ]
        }"#;

        let content = TeamsContentParser::parse_str("adaptive_card", json).unwrap();
        match content {
            TeamsContent::AdaptiveCard(card) => {
                assert_eq!(card.version, "1.4");
                assert_eq!(card.body.len(), 2);
            }
            _ => panic!("Expected AdaptiveCard content"),
        }
    }
}

// =============================================================================
// 🟢 P0 FIX: PlatformContent trait implementation for unified content framework
// =============================================================================

impl PlatformContent for TeamsContent {
    fn content_type(&self) -> UnifiedContentType {
        match self {
            TeamsContent::Text(_) => UnifiedContentType::Text,
            TeamsContent::Html(_) => UnifiedContentType::Rich,
            TeamsContent::AdaptiveCard(_) => UnifiedContentType::Card,
            TeamsContent::HeroCard(_) => UnifiedContentType::Card,
            TeamsContent::ThumbnailCard(_) => UnifiedContentType::Card,
            TeamsContent::File(_) => UnifiedContentType::File,
            TeamsContent::Image(_) => UnifiedContentType::Image,
            TeamsContent::Video(_) => UnifiedContentType::Video,
            TeamsContent::Audio(_) => UnifiedContentType::Audio,
            TeamsContent::RichText(_) => UnifiedContentType::Rich,
        }
    }

    fn extract_text(&self) -> String {
        TeamsContentParser::extract_text(self)
    }
}

impl TeamsImageContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.url.clone(),
            width: self.width,
            height: self.height,
            size: self.size,
            mime_type: None,
            filename: None,
            caption: self.alt.clone(),
            thumbnail: None,
            duration: None,
        }
    }
}

impl TeamsVideoContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.url.clone(),
            width: self.width,
            height: self.height,
            duration: self.duration,
            mime_type: None,
            filename: self.title.clone(),
            caption: None,
            size: None,
            thumbnail: self.thumbnail.clone(),
        }
    }
}

impl TeamsFileContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.content_url.clone(),
            mime_type: Some(self.content_type.clone()),
            filename: Some(self.name.clone()),
            size: self.size,
            width: None,
            height: None,
            duration: None,
            caption: None,
            thumbnail: self.thumbnail_url.clone(),
        }
    }
}
