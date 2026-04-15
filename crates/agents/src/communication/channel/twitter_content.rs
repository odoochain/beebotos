//! Twitter/X Message Content Parser
//!
//! Provides parsing and handling of various Twitter/X message content types
//! including tweets, direct messages, media, polls, and cards.

use serde::{Deserialize, Serialize};

use crate::communication::channel::content::{
    ContentType as UnifiedContentType, MediaContent, PlatformContent,
};
use crate::error::{AgentError, Result};

/// Twitter message content types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum TwitterContent {
    /// Plain text tweet
    #[serde(rename = "text")]
    Text(TwitterTextContent),
    /// Tweet with media (images, videos, GIFs)
    #[serde(rename = "media")]
    Media(TwitterMediaContent),
    /// Tweet with poll
    #[serde(rename = "poll")]
    Poll(TwitterPollContent),
    /// Tweet with card (link preview)
    #[serde(rename = "card")]
    Card(TwitterCardContent),
    /// Quote tweet
    #[serde(rename = "quote")]
    Quote(TwitterQuoteContent),
    /// Reply tweet
    #[serde(rename = "reply")]
    Reply(TwitterReplyContent),
    /// Retweet
    #[serde(rename = "retweet")]
    Retweet(TwitterRetweetContent),
    /// Thread (series of connected tweets)
    #[serde(rename = "thread")]
    Thread(TwitterThreadContent),
    /// Direct message
    #[serde(rename = "dm")]
    DirectMessage(TwitterDMContent),
}

/// Text content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwitterTextContent {
    /// The text content (max 280 chars for standard tweets, 4000 for Twitter
    /// Blue)
    pub text: String,
    /// Whether this is a long tweet (Twitter Blue)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_long_tweet: Option<bool>,
    /// Language code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

/// Media content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwitterMediaContent {
    /// The text content
    pub text: String,
    /// Media items
    pub media: Vec<TwitterMediaItem>,
    /// Alt text for accessibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
}

/// Twitter media item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwitterMediaItem {
    /// Media ID
    pub id: String,
    /// Media type: photo, video, animated_gif
    #[serde(rename = "type")]
    pub media_type: String,
    /// Media URL
    pub url: String,
    /// Display URL (shortened)
    #[serde(rename = "display_url")]
    pub display_url: String,
    /// Expanded URL
    #[serde(rename = "expanded_url")]
    pub expanded_url: String,
    /// Media key for API v2
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "media_key")]
    pub media_key: Option<String>,
    /// Width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// Height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Duration in milliseconds (for video)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i32>,
    /// Variants for video (different qualities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<TwitterMediaVariant>>,
}

/// Twitter media variant (for videos)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwitterMediaVariant {
    /// Bitrate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i32>,
    /// Content type
    #[serde(rename = "content_type")]
    pub content_type: String,
    /// URL
    pub url: String,
}

/// Poll content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwitterPollContent {
    /// The text content
    pub text: String,
    /// Poll options
    pub options: Vec<TwitterPollOption>,
    /// Duration in minutes
    #[serde(rename = "duration_minutes")]
    pub duration_minutes: i32,
    /// End datetime
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "end_datetime")]
    pub end_datetime: Option<String>,
    /// Voting status
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "voting_status")]
    pub voting_status: Option<String>,
}

/// Twitter poll option
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwitterPollOption {
    /// Option position (1-4)
    pub position: i32,
    /// Option label
    pub label: String,
    /// Vote count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub votes: Option<i32>,
}

/// Card content structure (link preview)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwitterCardContent {
    /// The text content
    pub text: String,
    /// Card URL
    pub url: String,
    /// Card title
    pub title: String,
    /// Card description
    pub description: String,
    /// Card image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Card type: summary, summary_large_image, app, player
    #[serde(rename = "card_type")]
    pub card_type: String,
    /// Site name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
}

/// Quote tweet content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwitterQuoteContent {
    /// The text content
    pub text: String,
    /// Quoted tweet ID
    #[serde(rename = "quoted_tweet_id")]
    pub quoted_tweet_id: String,
    /// Quoted tweet text
    #[serde(rename = "quoted_text")]
    pub quoted_text: String,
    /// Quoted tweet author
    #[serde(rename = "quoted_author")]
    pub quoted_author: String,
    /// Quoted tweet author ID
    #[serde(rename = "quoted_author_id")]
    pub quoted_author_id: String,
}

/// Reply tweet content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwitterReplyContent {
    /// The text content
    pub text: String,
    /// Reply to tweet ID
    #[serde(rename = "in_reply_to_tweet_id")]
    pub in_reply_to_tweet_id: String,
    /// Reply to user ID
    #[serde(rename = "in_reply_to_user_id")]
    pub in_reply_to_user_id: String,
    /// Reply to username
    #[serde(rename = "in_reply_to_username")]
    pub in_reply_to_username: String,
}

/// Retweet content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwitterRetweetContent {
    /// Original tweet ID
    #[serde(rename = "retweeted_tweet_id")]
    pub retweeted_tweet_id: String,
    /// Original tweet text
    #[serde(rename = "retweeted_text")]
    pub retweeted_text: String,
    /// Original tweet author
    #[serde(rename = "retweeted_author")]
    pub retweeted_author: String,
    /// Original tweet author ID
    #[serde(rename = "retweeted_author_id")]
    pub retweeted_author_id: String,
    /// Comment added when retweeting (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Thread content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TwitterThreadContent {
    /// Thread tweets
    pub tweets: Vec<TwitterThreadTweet>,
    /// Thread author
    pub author: String,
    /// Thread author ID
    #[serde(rename = "author_id")]
    pub author_id: String,
}

/// Thread tweet
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwitterThreadTweet {
    /// Tweet ID
    pub id: String,
    /// Tweet text
    pub text: String,
    /// Position in thread
    pub position: i32,
}

/// Direct message content structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TwitterDMContent {
    /// Message text
    pub text: String,
    /// Sender ID
    #[serde(rename = "sender_id")]
    pub sender_id: String,
    /// Recipient ID
    #[serde(rename = "recipient_id")]
    pub recipient_id: String,
    /// Attachments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<TwitterDMAttachment>>,
}

/// DM attachment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwitterDMAttachment {
    /// Attachment type: media, location
    #[serde(rename = "type")]
    pub attachment_type: String,
    /// Media info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<TwitterMediaItem>,
    /// Location info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<TwitterLocation>,
}

/// Twitter location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwitterLocation {
    /// Location type
    #[serde(rename = "type")]
    pub location_type: String,
    /// Coordinates
    pub coordinates: Vec<f64>,
    /// Place name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Twitter content parser
#[derive(Debug, Clone, Default)]
pub struct TwitterContentParser;

impl TwitterContentParser {
    /// Create a new content parser
    pub fn new() -> Self {
        Self
    }

    /// Parse content from JSON value
    ///
    /// # Arguments
    /// * `content_type` - The content type (text, media, poll, card, quote,
    ///   reply, retweet, thread, dm)
    /// * `content` - The JSON content to parse
    ///
    /// # Returns
    /// Parsed TwitterContent enum variant
    pub fn parse(content_type: &str, content: serde_json::Value) -> Result<TwitterContent> {
        match content_type {
            "text" => {
                let text_content: TwitterTextContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse text content: {}", e))
                    })?;
                Ok(TwitterContent::Text(text_content))
            }
            "media" => {
                let media_content: TwitterMediaContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse media content: {}", e))
                    })?;
                Ok(TwitterContent::Media(media_content))
            }
            "poll" => {
                let poll_content: TwitterPollContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse poll content: {}", e))
                    })?;
                Ok(TwitterContent::Poll(poll_content))
            }
            "card" => {
                let card_content: TwitterCardContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse card content: {}", e))
                    })?;
                Ok(TwitterContent::Card(card_content))
            }
            "quote" => {
                let quote_content: TwitterQuoteContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse quote content: {}", e))
                    })?;
                Ok(TwitterContent::Quote(quote_content))
            }
            "reply" => {
                let reply_content: TwitterReplyContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse reply content: {}", e))
                    })?;
                Ok(TwitterContent::Reply(reply_content))
            }
            "retweet" => {
                let retweet_content: TwitterRetweetContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse retweet content: {}", e))
                    })?;
                Ok(TwitterContent::Retweet(retweet_content))
            }
            "thread" => {
                let thread_content: TwitterThreadContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse thread content: {}", e))
                    })?;
                Ok(TwitterContent::Thread(thread_content))
            }
            "dm" => {
                let dm_content: TwitterDMContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse DM content: {}", e))
                    })?;
                Ok(TwitterContent::DirectMessage(dm_content))
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
    /// Parsed TwitterContent enum variant
    pub fn parse_str(content_type: &str, content_json: &str) -> Result<TwitterContent> {
        let content: serde_json::Value = serde_json::from_str(content_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON: {}", e)))?;
        Self::parse(content_type, content)
    }

    /// Extract plain text from any content type
    ///
    /// # Arguments
    /// * `content` - The TwitterContent to extract text from
    ///
    /// # Returns
    /// Extracted plain text string
    pub fn extract_text(content: &TwitterContent) -> String {
        match content {
            TwitterContent::Text(text) => text.text.clone(),
            TwitterContent::Media(media) => {
                let media_desc = media
                    .media
                    .iter()
                    .map(|m| format!("[{}]", m.media_type))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{} {}", media.text, media_desc)
            }
            TwitterContent::Poll(poll) => {
                let options = poll
                    .options
                    .iter()
                    .map(|o| format!("{}. {}", o.position, o.label))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{}\n\nPoll:\n{}", poll.text, options)
            }
            TwitterContent::Card(card) => {
                format!("{}\n\n[Card: {} - {}]", card.text, card.title, card.url)
            }
            TwitterContent::Quote(quote) => {
                format!(
                    "{}\n\n[Quoted @{}: {}]",
                    quote.text, quote.quoted_author, quote.quoted_text
                )
            }
            TwitterContent::Reply(reply) => reply.text.clone(),
            TwitterContent::Retweet(retweet) => {
                if let Some(comment) = &retweet.comment {
                    format!(
                        "{}\n\n[RT @{}: {}]",
                        comment, retweet.retweeted_author, retweet.retweeted_text
                    )
                } else {
                    format!(
                        "[RT @{}: {}]",
                        retweet.retweeted_author, retweet.retweeted_text
                    )
                }
            }
            TwitterContent::Thread(thread) => {
                let tweets_text = thread
                    .tweets
                    .iter()
                    .map(|t| format!("{}. {}", t.position, t.text))
                    .collect::<Vec<_>>()
                    .join("\n\n");
                format!("[Thread by @{}]\n\n{}", thread.author, tweets_text)
            }
            TwitterContent::DirectMessage(dm) => {
                format!("[DM] {}", dm.text)
            }
        }
    }

    /// Get content type string
    ///
    /// # Arguments
    /// * `content` - The TwitterContent
    ///
    /// # Returns
    /// Content type string
    pub fn get_content_type(content: &TwitterContent) -> &'static str {
        match content {
            TwitterContent::Text(_) => "text",
            TwitterContent::Media(_) => "media",
            TwitterContent::Poll(_) => "poll",
            TwitterContent::Card(_) => "card",
            TwitterContent::Quote(_) => "quote",
            TwitterContent::Reply(_) => "reply",
            TwitterContent::Retweet(_) => "retweet",
            TwitterContent::Thread(_) => "thread",
            TwitterContent::DirectMessage(_) => "dm",
        }
    }

    /// Create text content
    ///
    /// # Arguments
    /// * `text` - The text content
    ///
    /// # Returns
    /// TwitterContent::Text variant
    pub fn create_text(text: impl Into<String>) -> TwitterContent {
        TwitterContent::Text(TwitterTextContent {
            text: text.into(),
            is_long_tweet: None,
            lang: None,
        })
    }

    /// Create media content
    ///
    /// # Arguments
    /// * `text` - The text content
    /// * `media` - Media items
    ///
    /// # Returns
    /// TwitterContent::Media variant
    pub fn create_media(text: impl Into<String>, media: Vec<TwitterMediaItem>) -> TwitterContent {
        TwitterContent::Media(TwitterMediaContent {
            text: text.into(),
            media,
            alt_text: None,
        })
    }

    /// Create reply content
    ///
    /// # Arguments
    /// * `text` - The text content
    /// * `reply_to_tweet_id` - Tweet ID to reply to
    /// * `reply_to_username` - Username to reply to
    ///
    /// # Returns
    /// TwitterContent::Reply variant
    pub fn create_reply(
        text: impl Into<String>,
        reply_to_tweet_id: impl Into<String>,
        reply_to_username: impl Into<String>,
    ) -> TwitterContent {
        TwitterContent::Reply(TwitterReplyContent {
            text: text.into(),
            in_reply_to_tweet_id: reply_to_tweet_id.into(),
            in_reply_to_user_id: String::new(),
            in_reply_to_username: reply_to_username.into(),
        })
    }

    /// Create quote tweet content
    ///
    /// # Arguments
    /// * `text` - The text content
    /// * `quoted_tweet_id` - Tweet ID being quoted
    /// * `quoted_author` - Author of quoted tweet
    /// * `quoted_text` - Text of quoted tweet
    ///
    /// # Returns
    /// TwitterContent::Quote variant
    pub fn create_quote(
        text: impl Into<String>,
        quoted_tweet_id: impl Into<String>,
        quoted_author: impl Into<String>,
        quoted_text: impl Into<String>,
    ) -> TwitterContent {
        TwitterContent::Quote(TwitterQuoteContent {
            text: text.into(),
            quoted_tweet_id: quoted_tweet_id.into(),
            quoted_text: quoted_text.into(),
            quoted_author: quoted_author.into(),
            quoted_author_id: String::new(),
        })
    }

    /// Create DM content
    ///
    /// # Arguments
    /// * `text` - The text content
    /// * `sender_id` - Sender user ID
    /// * `recipient_id` - Recipient user ID
    ///
    /// # Returns
    /// TwitterContent::DirectMessage variant
    pub fn create_dm(
        text: impl Into<String>,
        sender_id: impl Into<String>,
        recipient_id: impl Into<String>,
    ) -> TwitterContent {
        TwitterContent::DirectMessage(TwitterDMContent {
            text: text.into(),
            sender_id: sender_id.into(),
            recipient_id: recipient_id.into(),
            attachments: None,
        })
    }

    /// Serialize content to JSON string
    ///
    /// # Arguments
    /// * `content` - The TwitterContent to serialize
    ///
    /// # Returns
    /// JSON string representation
    pub fn to_json(content: &TwitterContent) -> Result<String> {
        serde_json::to_string(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Serialize content to JSON value
    ///
    /// # Arguments
    /// * `content` - The TwitterContent to serialize
    ///
    /// # Returns
    /// JSON value representation
    pub fn to_json_value(content: &TwitterContent) -> Result<serde_json::Value> {
        serde_json::to_value(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Check if content has media
    pub fn has_media(content: &TwitterContent) -> bool {
        matches!(content, TwitterContent::Media(_))
    }

    /// Check if content is a reply
    pub fn is_reply(content: &TwitterContent) -> bool {
        matches!(content, TwitterContent::Reply(_))
    }

    /// Check if content is a retweet
    pub fn is_retweet(content: &TwitterContent) -> bool {
        matches!(content, TwitterContent::Retweet(_))
    }

    /// Get media URLs from content
    pub fn get_media_urls(content: &TwitterContent) -> Vec<String> {
        match content {
            TwitterContent::Media(media) => media.media.iter().map(|m| m.url.clone()).collect(),
            _ => Vec::new(),
        }
    }

    /// Truncate text to Twitter's character limit
    pub fn truncate_to_limit(text: &str, limit: usize) -> String {
        if text.len() <= limit {
            text.to_string()
        } else {
            let truncated = &text[..limit];
            format!("{}...", truncated)
        }
    }
}

/// Tweet builder for easy tweet creation
pub struct TweetBuilder {
    text: String,
    reply_to_tweet_id: Option<String>,
    media_keys: Vec<String>,
    poll_options: Vec<String>,
    poll_duration: Option<i32>,
}

impl TweetBuilder {
    /// Create a new tweet builder
    pub fn new() -> Self {
        Self {
            text: String::new(),
            reply_to_tweet_id: None,
            media_keys: Vec::new(),
            poll_options: Vec::new(),
            poll_duration: None,
        }
    }

    /// Set the text
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Set reply to tweet ID
    pub fn reply_to(mut self, tweet_id: impl Into<String>) -> Self {
        self.reply_to_tweet_id = Some(tweet_id.into());
        self
    }

    /// Add media key
    pub fn media(mut self, media_key: impl Into<String>) -> Self {
        self.media_keys.push(media_key.into());
        self
    }

    /// Add poll option
    pub fn poll_option(mut self, option: impl Into<String>) -> Self {
        self.poll_options.push(option.into());
        self
    }

    /// Set poll duration in minutes
    pub fn poll_duration(mut self, minutes: i32) -> Self {
        self.poll_duration = Some(minutes);
        self
    }

    /// Build the tweet JSON for API v2
    pub fn build(self) -> serde_json::Value {
        let mut tweet = serde_json::json!({
            "text": self.text,
        });

        if let Some(reply_id) = self.reply_to_tweet_id {
            tweet["reply"] = serde_json::json!({
                "in_reply_to_tweet_id": reply_id,
            });
        }

        if !self.media_keys.is_empty() {
            tweet["media"] = serde_json::json!({
                "media_keys": self.media_keys,
            });
        }

        if !self.poll_options.is_empty() && self.poll_duration.is_some() {
            tweet["poll"] = serde_json::json!({
                "options": self.poll_options.iter().enumerate().map(|(i, opt)| {
                    serde_json::json!({
                        "position": i + 1,
                        "label": opt,
                    })
                }).collect::<Vec<_>>(),
                "duration_minutes": self.poll_duration.unwrap(),
            });
        }

        tweet
    }
}

impl Default for TweetBuilder {
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
            "text": "Hello, Twitter!"
        });
        let content = TwitterContentParser::parse("text", json).unwrap();
        assert!(matches!(content, TwitterContent::Text(_)));
        assert_eq!(
            TwitterContentParser::extract_text(&content),
            "Hello, Twitter!"
        );
    }

    #[test]
    fn test_parse_media_content() {
        let json = serde_json::json!({
            "text": "Check out this photo!",
            "media": [
                {
                    "id": "123456",
                    "type": "photo",
                    "url": "https://pbs.twimg.com/media/...",
                    "display_url": "pic.twitter.com/...",
                    "expanded_url": "https://twitter.com/..."
                }
            ]
        });
        let content = TwitterContentParser::parse("media", json).unwrap();
        assert!(matches!(content, TwitterContent::Media(_)));
        let text = TwitterContentParser::extract_text(&content);
        assert!(text.contains("Check out this photo!"));
        assert!(text.contains("[photo]"));
    }

    #[test]
    fn test_parse_quote_content() {
        let json = serde_json::json!({
            "text": "This is interesting!",
            "quoted_tweet_id": "789",
            "quoted_text": "Original tweet",
            "quoted_author": "originaluser",
            "quoted_author_id": "123"
        });
        let content = TwitterContentParser::parse("quote", json).unwrap();
        assert!(matches!(content, TwitterContent::Quote(_)));
        let text = TwitterContentParser::extract_text(&content);
        assert!(text.contains("This is interesting!"));
        assert!(text.contains("@originaluser"));
    }

    #[test]
    fn test_create_text() {
        let content = TwitterContentParser::create_text("Test tweet");
        assert!(
            matches!(content, TwitterContent::Text(TwitterTextContent { text, .. }) if text == "Test tweet")
        );
    }

    #[test]
    fn test_create_reply() {
        let content = TwitterContentParser::create_reply("Reply text", "123456", "user");
        assert!(
            matches!(content, TwitterContent::Reply(TwitterReplyContent { 
            text, 
            in_reply_to_tweet_id, 
            in_reply_to_username,
            ..
        }) if text == "Reply text" && in_reply_to_tweet_id == "123456" && in_reply_to_username == "user")
        );
    }

    #[test]
    fn test_get_content_type() {
        let text = TwitterContent::Text(TwitterTextContent::default());
        assert_eq!(TwitterContentParser::get_content_type(&text), "text");

        let media = TwitterContent::Media(TwitterMediaContent::default());
        assert_eq!(TwitterContentParser::get_content_type(&media), "media");
    }

    #[test]
    fn test_tweet_builder() {
        let tweet = TweetBuilder::new()
            .text("Hello World")
            .reply_to("123456")
            .media("media_key_1")
            .poll_option("Yes")
            .poll_option("No")
            .poll_duration(1440)
            .build();

        assert_eq!(tweet["text"], "Hello World");
        assert_eq!(tweet["reply"]["in_reply_to_tweet_id"], "123456");
        assert!(tweet["media"]["media_keys"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("media_key_1")));
        assert!(tweet["poll"].is_object());
    }

    #[test]
    fn test_truncate_to_limit() {
        let long_text = "a".repeat(300);
        let truncated = TwitterContentParser::truncate_to_limit(&long_text, 280);
        assert!(truncated.len() <= 283); // 280 + "..."

        let short_text = "Short text";
        let not_truncated = TwitterContentParser::truncate_to_limit(short_text, 280);
        assert_eq!(not_truncated, short_text);
    }
}

// =============================================================================
// 🟢 P0 FIX: PlatformContent trait implementation for unified content framework
// =============================================================================

impl PlatformContent for TwitterContent {
    fn content_type(&self) -> UnifiedContentType {
        match self {
            TwitterContent::Text(_) => UnifiedContentType::Text,
            TwitterContent::Media(_) => UnifiedContentType::Image,
            TwitterContent::Poll(_) => UnifiedContentType::Card,
            TwitterContent::Card(_) => UnifiedContentType::Card,
            TwitterContent::Quote(_) => UnifiedContentType::Rich,
            TwitterContent::Reply(_) => UnifiedContentType::Text,
            TwitterContent::Retweet(_) => UnifiedContentType::Rich,
            TwitterContent::Thread(_) => UnifiedContentType::Rich,
            TwitterContent::DirectMessage(_) => UnifiedContentType::Text,
        }
    }

    fn extract_text(&self) -> String {
        TwitterContentParser::extract_text(self)
    }
}

impl TwitterMediaItem {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.url.clone(),
            width: self.width,
            height: self.height,
            duration: self.duration_ms,
            mime_type: None,
            filename: None,
            caption: None,
            size: None,
            thumbnail: None,
        }
    }
}
