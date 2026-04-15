//! Unified Message Processing Pipeline
//!
//! This module provides a unified message processing pipeline for all channels.
//! It handles:
//! 1. Receiving messages from various channels (WebSocket/Webhook/Polling)
//! 2. Multimodal preprocessing (text, image, file, rich media)
//! 3. LLM-compatible format conversion
//! 4. LLM processing
//! 5. Response formatting for channel-specific output
//! 6. Sending replies back to channels

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, info, warn};

use crate::communication::channel::ContentType;
use crate::communication::{Message, PlatformType};
use crate::error::Result;
use crate::media::{
    AttachmentType, AudioTranscriptionService, DocumentUnderstandingService, FormattedMessage,
    ImageUnderstandingService, LLMFormat, MediaDownloader, MessageFormatter, MessageRole,
    ParsedAttachment,
};
use crate::models::router::ModelRouter;

/// Message processing context
#[derive(Debug, Clone)]
pub struct MessageContext {
    /// Platform type
    pub platform: PlatformType,
    /// Channel ID (chat_id, open_id, etc.)
    pub channel_id: String,
    /// Sender ID
    pub sender_id: String,
    /// Message ID
    pub message_id: String,
    /// Thread ID for conversation tracking
    pub thread_id: Option<String>,
    /// Raw message data
    pub raw_data: serde_json::Value,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Processed message ready for LLM
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    /// Original context
    pub context: MessageContext,
    /// Formatted message for LLM
    pub formatted: FormattedMessage,
    /// Parsed attachments
    pub attachments: Vec<ParsedAttachment>,
    /// Media understanding results
    pub media_understanding: Vec<MediaUnderstandingResult>,
}

/// Media understanding result
#[derive(Debug, Clone)]
pub enum MediaUnderstandingResult {
    Image(crate::media::ImageUnderstanding),
    Document(crate::media::DocumentUnderstanding),
    Audio(crate::media::AudioTranscription),
}

/// Channel response message
#[derive(Debug, Clone)]
pub struct ChannelResponse {
    /// Response text
    pub text: String,
    /// Response attachments (images, files, etc.)
    pub attachments: Vec<ResponseAttachment>,
    /// Response type
    pub response_type: ResponseType,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Response attachment
#[derive(Debug, Clone)]
pub struct ResponseAttachment {
    pub attachment_type: AttachmentType,
    pub content: Vec<u8>,
    pub mime_type: String,
    pub file_name: Option<String>,
}

/// Response type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    Text,
    Image,
    File,
    RichMedia,
    Card,
}

/// Message processor trait
///
/// Each channel implements this trait to handle platform-specific message
/// processing
#[async_trait]
pub trait MessageProcessor: Send + Sync {
    /// Get supported platform
    fn platform(&self) -> PlatformType;

    /// Get supported content types
    fn supported_content_types(&self) -> Vec<ContentType>;

    /// Parse raw message into MessageContext
    async fn parse_message(&self, raw_data: serde_json::Value)
        -> Result<(MessageContext, Message)>;

    /// Extract attachments from message
    async fn extract_attachments(
        &self,
        context: &MessageContext,
        raw_data: &serde_json::Value,
    ) -> Result<Vec<ParsedAttachment>>;

    /// Download attachment content
    async fn download_attachment(&self, attachment: &ParsedAttachment) -> Result<Vec<u8>>;

    /// Format response for channel
    async fn format_response(
        &self,
        llm_response: &str,
        response_type: ResponseType,
    ) -> Result<ChannelResponse>;

    /// Send response to channel
    async fn send_response(&self, channel_id: &str, response: &ChannelResponse) -> Result<()>;
}

/// Unified message pipeline
pub struct MessagePipeline {
    /// Message formatter for LLM
    formatter: MessageFormatter,
    /// Media downloader
    #[allow(dead_code)]
    downloader: MediaDownloader,
    /// Image understanding service
    image_service: Option<ImageUnderstandingService>,
    /// Document understanding service
    document_service: Option<DocumentUnderstandingService>,
    /// Audio transcription service
    audio_service: Option<AudioTranscriptionService>,
    /// Model router
    #[allow(dead_code)]
    model_router: Arc<ModelRouter>,
    /// LLM format type
    llm_format: LLMFormat,
}

impl MessagePipeline {
    /// Create new message pipeline
    pub fn new(model_router: Arc<ModelRouter>) -> Self {
        Self {
            formatter: MessageFormatter::new(),
            downloader: MediaDownloader::default().expect("Failed to create MediaDownloader"),
            image_service: None,
            document_service: None,
            audio_service: None,
            model_router,
            llm_format: LLMFormat::OpenAI,
        }
    }

    /// Create with media understanding services
    pub fn with_media_services(
        mut self,
        image_service: ImageUnderstandingService,
        document_service: DocumentUnderstandingService,
        audio_service: AudioTranscriptionService,
    ) -> Self {
        self.image_service = Some(image_service);
        self.document_service = Some(document_service);
        self.audio_service = Some(audio_service);
        self
    }

    /// Set LLM format
    pub fn with_llm_format(mut self, format: LLMFormat) -> Self {
        self.llm_format = format;
        self
    }

    /// Process incoming message through the pipeline
    pub async fn process_incoming(
        &self,
        processor: &dyn MessageProcessor,
        raw_data: serde_json::Value,
    ) -> Result<ChannelResponse> {
        // Step 1: Parse message
        let (context, message) = processor.parse_message(raw_data.clone()).await?;
        info!(
            "📨 Received message from {:?}: {}",
            context.platform, message.content
        );

        // Step 2: Extract attachments
        let attachments = processor.extract_attachments(&context, &raw_data).await?;
        debug!("Found {} attachments", attachments.len());

        // Step 3: Process media (download and understand)
        let media_results = self.process_media(&attachments).await?;

        // Step 4: Build multimodal message for LLM
        let processed = self
            .build_processed_message(context, message, attachments, media_results)
            .await?;

        // Step 5: Call LLM
        let llm_response = self.call_llm(&processed).await?;

        // Step 6: Format response for channel
        let response = processor
            .format_response(&llm_response, ResponseType::Text)
            .await?;

        Ok(response)
    }

    /// Process media attachments
    async fn process_media(
        &self,
        attachments: &[ParsedAttachment],
    ) -> Result<Vec<MediaUnderstandingResult>> {
        let mut results = Vec::new();

        for attachment in attachments {
            match attachment.attachment_type {
                AttachmentType::Image | AttachmentType::EmbeddedImage => {
                    if let Some(ref service) = self.image_service {
                        match service
                            .analyze(attachment, crate::media::ImageAnalysisType::General)
                            .await
                        {
                            Ok(result) => results.push(MediaUnderstandingResult::Image(result)),
                            Err(e) => warn!("Failed to analyze image: {}", e),
                        }
                    }
                }
                AttachmentType::Document | AttachmentType::File => {
                    if let Some(ref service) = self.document_service {
                        match service.summarize(attachment).await {
                            Ok(summary) => {
                                let doc_result = crate::media::DocumentUnderstanding {
                                    title: attachment.file_name.clone(),
                                    summary,
                                    content: String::new(),
                                    sections: vec![],
                                    entities: vec![],
                                    document_type: crate::media::DocumentType::Unknown,
                                    page_count: None,
                                    confidence: 0.9,
                                    metadata: HashMap::new(),
                                };
                                results.push(MediaUnderstandingResult::Document(doc_result));
                            }
                            Err(e) => warn!("Failed to summarize document: {}", e),
                        }
                    }
                }
                AttachmentType::Audio => {
                    if let Some(ref service) = self.audio_service {
                        match service.transcribe(attachment, None).await {
                            Ok(result) => results.push(MediaUnderstandingResult::Audio(result)),
                            Err(e) => warn!("Failed to transcribe audio: {}", e),
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(results)
    }

    /// Build processed message with media context
    async fn build_processed_message(
        &self,
        context: MessageContext,
        message: Message,
        attachments: Vec<ParsedAttachment>,
        media_results: Vec<MediaUnderstandingResult>,
    ) -> Result<ProcessedMessage> {
        // Build text content with media descriptions
        let mut text = message.content.clone();

        for result in &media_results {
            match result {
                MediaUnderstandingResult::Image(img) => {
                    text.push_str(&format!("\n\n[Image: {}]", img.description));
                }
                MediaUnderstandingResult::Document(doc) => {
                    text.push_str(&format!(
                        "\n\n[Document: {} - {}]",
                        doc.title.as_deref().unwrap_or("Unknown"),
                        doc.summary
                    ));
                }
                MediaUnderstandingResult::Audio(audio) => {
                    text.push_str(&format!("\n\n[Audio Transcription: {}]", audio.text));
                }
            }
        }

        // Build formatted message
        let formatted = self
            .formatter
            .build_multimodal_message(text, attachments.clone(), MessageRole::User)
            .await?;

        Ok(ProcessedMessage {
            context,
            formatted,
            attachments,
            media_understanding: media_results,
        })
    }

    /// Call LLM with processed message
    async fn call_llm(&self, processed: &ProcessedMessage) -> Result<String> {
        // Convert to LLM format
        let _llm_message = self
            .formatter
            .format_for_llm(&processed.formatted, self.llm_format);

        // Call model router
        // TODO: Implement actual LLM call through model router
        // For now, return a placeholder response
        Ok(format!(
            "I received your message: {}",
            processed.formatted.text
        ))
    }

    /// Send response through channel
    pub async fn send_response(
        &self,
        processor: &dyn MessageProcessor,
        channel_id: &str,
        response: &ChannelResponse,
    ) -> Result<()> {
        processor.send_response(channel_id, response).await
    }
}

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Enable image understanding
    pub enable_image_understanding: bool,
    /// Enable document understanding
    pub enable_document_understanding: bool,
    /// Enable audio transcription
    pub enable_audio_transcription: bool,
    /// LLM provider for media understanding
    pub media_llm_provider: String,
    /// LLM model for media understanding
    pub media_llm_model: String,
    /// Maximum attachment size
    pub max_attachment_size: usize,
    /// Supported content types
    pub supported_content_types: Vec<ContentType>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_image_understanding: true,
            enable_document_understanding: true,
            enable_audio_transcription: true,
            media_llm_provider: "openai".to_string(),
            media_llm_model: "gpt-4o".to_string(),
            max_attachment_size: 100 * 1024 * 1024, // 100MB
            supported_content_types: vec![
                ContentType::Text,
                ContentType::Image,
                ContentType::File,
                ContentType::Audio,
                ContentType::Video,
                ContentType::Rich,
                ContentType::Card,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert!(config.enable_image_understanding);
        assert!(config.enable_document_understanding);
        assert!(config.enable_audio_transcription);
        assert_eq!(config.media_llm_provider, "openai");
        assert_eq!(config.media_llm_model, "gpt-4o");
    }

    #[test]
    fn test_response_type() {
        assert_eq!(ResponseType::Text as i32, 0);
        assert_eq!(ResponseType::Image as i32, 1);
    }
}
