//! Media Understanding Module
//!
//! Provides functionality for understanding and processing various media types
//! including images, documents, and audio using LLM capabilities.

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{AgentError, Result};
use crate::media::attachment::{AttachmentType, ParsedAttachment};

/// Media understanding capability trait
///
/// This trait defines the interface for media understanding capabilities,
/// allowing different implementations (local models, cloud APIs, etc.)
#[async_trait]
pub trait MediaUnderstanding: Send + Sync {
    /// Check if this understanding capability is available
    async fn is_available(&self) -> bool;

    /// Get the name of the understanding provider
    fn name(&self) -> &str;

    /// Get supported media types
    fn supported_types(&self) -> Vec<AttachmentType>;
}

/// Image understanding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUnderstanding {
    /// Description of the image
    pub description: String,
    /// Detected objects in the image
    pub objects: Vec<DetectedObject>,
    /// Extracted text from image (OCR)
    pub extracted_text: Option<String>,
    /// Image dimensions
    pub dimensions: Option<ImageDimensions>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Detected object in an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedObject {
    /// Object label/class
    pub label: String,
    /// Confidence score
    pub confidence: f32,
    /// Bounding box coordinates (x, y, width, height)
    pub bbox: Option<(f32, f32, f32, f32)>,
}

/// Image dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ImageDimensions {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// Document understanding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUnderstanding {
    /// Document title
    pub title: Option<String>,
    /// Document summary
    pub summary: String,
    /// Extracted text content
    pub content: String,
    /// Key sections identified
    pub sections: Vec<DocumentSection>,
    /// Key entities extracted
    pub entities: Vec<ExtractedEntity>,
    /// Document type
    pub document_type: DocumentType,
    /// Page count (if applicable)
    pub page_count: Option<u32>,
    /// Confidence score
    pub confidence: f32,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Document section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    /// Section title
    pub title: String,
    /// Section content
    pub content: String,
    /// Section level (1 = top level)
    pub level: u32,
}

/// Extracted entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// Entity text
    pub text: String,
    /// Entity type (person, organization, location, etc.)
    pub entity_type: String,
    /// Confidence score
    pub confidence: f32,
}

/// Document type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    /// Plain text document
    Text,
    /// PDF document
    Pdf,
    /// Word document
    Word,
    /// Spreadsheet
    Spreadsheet,
    /// Presentation
    Presentation,
    /// Code file
    Code,
    /// Markdown
    Markdown,
    /// HTML
    Html,
    /// Unknown type
    Unknown,
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Text => write!(f, "text"),
            DocumentType::Pdf => write!(f, "pdf"),
            DocumentType::Word => write!(f, "word"),
            DocumentType::Spreadsheet => write!(f, "spreadsheet"),
            DocumentType::Presentation => write!(f, "presentation"),
            DocumentType::Code => write!(f, "code"),
            DocumentType::Markdown => write!(f, "markdown"),
            DocumentType::Html => write!(f, "html"),
            DocumentType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Audio transcription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscription {
    /// Transcribed text
    pub text: String,
    /// Language detected
    pub language: Option<String>,
    /// Confidence score
    pub confidence: f32,
    /// Duration in seconds
    pub duration: Option<f64>,
    /// Word-level timestamps (if available)
    pub word_timestamps: Option<Vec<WordTimestamp>>,
    /// Speaker segments (if diarization available)
    pub speaker_segments: Option<Vec<SpeakerSegment>>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Word timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTimestamp {
    /// Word text
    pub word: String,
    /// Start time in seconds
    pub start: f64,
    /// End time in seconds
    pub end: f64,
    /// Confidence score
    pub confidence: f32,
}

/// Speaker segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerSegment {
    /// Speaker identifier
    pub speaker: String,
    /// Segment text
    pub text: String,
    /// Start time in seconds
    pub start: f64,
    /// End time in seconds
    pub end: f64,
}

/// LLM-based media understanding implementation
#[derive(Debug, Clone)]
pub struct LLMMediaUnderstanding {
    /// Provider name
    provider: String,
    /// Model name
    model: String,
    /// Configuration
    config: UnderstandingConfig,
}

/// Understanding configuration
#[derive(Debug, Clone)]
pub struct UnderstandingConfig {
    /// Maximum image size in bytes
    pub max_image_size: usize,
    /// Maximum document size in bytes
    pub max_document_size: usize,
    /// Maximum audio duration in seconds
    pub max_audio_duration: u64,
    /// Whether to perform OCR on images
    pub enable_ocr: bool,
    /// Whether to detect objects in images
    pub enable_object_detection: bool,
    /// Whether to perform speaker diarization
    pub enable_diarization: bool,
    /// Timeout for understanding operations
    pub timeout_seconds: u64,
}

impl Default for UnderstandingConfig {
    fn default() -> Self {
        Self {
            max_image_size: 20 * 1024 * 1024,     // 20MB
            max_document_size: 100 * 1024 * 1024, // 100MB
            max_audio_duration: 7200,             // 2 hours
            enable_ocr: true,
            enable_object_detection: true,
            enable_diarization: false,
            timeout_seconds: 120,
        }
    }
}

impl LLMMediaUnderstanding {
    /// Create a new LLM media understanding instance
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            config: UnderstandingConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        provider: impl Into<String>,
        model: impl Into<String>,
        config: UnderstandingConfig,
    ) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            config,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &UnderstandingConfig {
        &self.config
    }

    /// Understand an image
    ///
    /// # Arguments
    /// * `attachment` - The image attachment to understand
    /// * `prompt` - Optional prompt to guide understanding
    ///
    /// # Returns
    /// Image understanding result
    pub async fn understand_image(
        &self,
        attachment: &ParsedAttachment,
        prompt: Option<&str>,
    ) -> Result<ImageUnderstanding> {
        // Validate attachment type
        if !matches!(
            attachment.attachment_type,
            AttachmentType::Image | AttachmentType::EmbeddedImage | AttachmentType::Sticker
        ) {
            return Err(AgentError::InvalidConfig(format!(
                "Expected image attachment, got {:?}",
                attachment.attachment_type
            )));
        }

        // Check file size
        if let Some(size) = attachment.file_size {
            if size > self.config.max_image_size as u64 {
                return Err(AgentError::InvalidConfig(format!(
                    "Image too large: {} bytes (max: {})",
                    size, self.config.max_image_size
                )));
            }
        }

        tracing::info!(
            "Understanding image {} with provider {}",
            attachment.id,
            self.provider
        );

        // This is a placeholder implementation
        // In a real implementation, this would:
        // 1. Download/load the image
        // 2. Send to LLM vision API
        // 3. Parse the response

        let default_prompt = prompt.unwrap_or("Describe this image in detail.");

        // Placeholder result
        Ok(ImageUnderstanding {
            description: format!("Image understanding with prompt: {}", default_prompt),
            objects: vec![],
            extracted_text: if self.config.enable_ocr {
                Some("OCR not yet implemented".to_string())
            } else {
                None
            },
            dimensions: None,
            confidence: 0.95,
            metadata: {
                let mut m = HashMap::new();
                m.insert("provider".to_string(), self.provider.clone());
                m.insert("model".to_string(), self.model.clone());
                m.insert("prompt".to_string(), default_prompt.to_string());
                m
            },
        })
    }

    /// Understand a document
    ///
    /// # Arguments
    /// * `attachment` - The document attachment to understand
    /// * `extract_full_content` - Whether to extract full text content
    ///
    /// # Returns
    /// Document understanding result
    pub async fn understand_document(
        &self,
        attachment: &ParsedAttachment,
        extract_full_content: bool,
    ) -> Result<DocumentUnderstanding> {
        // Validate attachment type
        if !matches!(
            attachment.attachment_type,
            AttachmentType::Document | AttachmentType::File | AttachmentType::Archive
        ) {
            return Err(AgentError::InvalidConfig(format!(
                "Expected document attachment, got {:?}",
                attachment.attachment_type
            )));
        }

        // Check file size
        if let Some(size) = attachment.file_size {
            if size > self.config.max_document_size as u64 {
                return Err(AgentError::InvalidConfig(format!(
                    "Document too large: {} bytes (max: {})",
                    size, self.config.max_document_size
                )));
            }
        }

        tracing::info!(
            "Understanding document {} with provider {}",
            attachment.id,
            self.provider
        );

        // Determine document type
        let doc_type = self.infer_document_type(attachment);

        // Placeholder result
        Ok(DocumentUnderstanding {
            title: attachment.file_name.clone(),
            summary: "Document understanding not yet fully implemented".to_string(),
            content: if extract_full_content {
                "Full content extraction not yet implemented".to_string()
            } else {
                String::new()
            },
            sections: vec![],
            entities: vec![],
            document_type: doc_type,
            page_count: None,
            confidence: 0.90,
            metadata: {
                let mut m = HashMap::new();
                m.insert("provider".to_string(), self.provider.clone());
                m.insert("model".to_string(), self.model.clone());
                m.insert(
                    "file_name".to_string(),
                    attachment.file_name.clone().unwrap_or_default(),
                );
                m
            },
        })
    }

    /// Transcribe audio
    ///
    /// # Arguments
    /// * `attachment` - The audio attachment to transcribe
    /// * `language` - Optional language hint (e.g., "en", "zh")
    ///
    /// # Returns
    /// Audio transcription result
    pub async fn transcribe_audio(
        &self,
        attachment: &ParsedAttachment,
        language: Option<&str>,
    ) -> Result<AudioTranscription> {
        // Validate attachment type
        if !matches!(attachment.attachment_type, AttachmentType::Audio) {
            return Err(AgentError::InvalidConfig(format!(
                "Expected audio attachment, got {:?}",
                attachment.attachment_type
            )));
        }

        tracing::info!(
            "Transcribing audio {} with provider {}",
            attachment.id,
            self.provider
        );

        // Placeholder result
        Ok(AudioTranscription {
            text: "Audio transcription not yet fully implemented".to_string(),
            language: language.map(|s| s.to_string()),
            confidence: 0.85,
            duration: attachment
                .metadata
                .get("duration_ms")
                .and_then(|d| d.parse::<f64>().ok().map(|ms| ms / 1000.0)),
            word_timestamps: None,
            speaker_segments: if self.config.enable_diarization {
                Some(vec![])
            } else {
                None
            },
            metadata: {
                let mut m = HashMap::new();
                m.insert("provider".to_string(), self.provider.clone());
                m.insert("model".to_string(), self.model.clone());
                m
            },
        })
    }

    /// Infer document type from attachment
    fn infer_document_type(&self, attachment: &ParsedAttachment) -> DocumentType {
        let mime = attachment.mime_type.as_deref().unwrap_or("");
        let ext = attachment.extension.as_deref().unwrap_or("");

        if mime.contains("pdf") || ext == "pdf" {
            DocumentType::Pdf
        } else if mime.contains("word") || ext == "doc" || ext == "docx" {
            DocumentType::Word
        } else if mime.contains("excel") || mime.contains("sheet") || ext == "xls" || ext == "xlsx"
        {
            DocumentType::Spreadsheet
        } else if mime.contains("powerpoint")
            || mime.contains("presentation")
            || ext == "ppt"
            || ext == "pptx"
        {
            DocumentType::Presentation
        } else if mime.contains("markdown") || ext == "md" {
            DocumentType::Markdown
        } else if mime.contains("html") || ext == "html" || ext == "htm" {
            DocumentType::Html
        } else if ext == "txt" || ext == "csv" || ext == "json" || ext == "xml" {
            DocumentType::Text
        } else if ext == "rs"
            || ext == "py"
            || ext == "js"
            || ext == "ts"
            || ext == "java"
            || ext == "cpp"
            || ext == "c"
            || ext == "go"
        {
            DocumentType::Code
        } else {
            DocumentType::Unknown
        }
    }
}

#[async_trait]
impl MediaUnderstanding for LLMMediaUnderstanding {
    async fn is_available(&self) -> bool {
        // Placeholder: check if the LLM provider is available
        // In a real implementation, this would make a health check request
        true
    }

    fn name(&self) -> &str {
        &self.provider
    }

    fn supported_types(&self) -> Vec<AttachmentType> {
        vec![
            AttachmentType::Image,
            AttachmentType::EmbeddedImage,
            AttachmentType::Sticker,
            AttachmentType::Document,
            AttachmentType::File,
            AttachmentType::Audio,
        ]
    }
}

/// Image understanding specialized struct
#[derive(Debug, Clone)]
pub struct ImageUnderstandingService {
    inner: LLMMediaUnderstanding,
}

impl ImageUnderstandingService {
    /// Create a new image understanding service
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            inner: LLMMediaUnderstanding::new(provider, model),
        }
    }

    /// Understand an image with detailed analysis
    pub async fn analyze(
        &self,
        attachment: &ParsedAttachment,
        analysis_type: ImageAnalysisType,
    ) -> Result<ImageUnderstanding> {
        let prompt = match analysis_type {
            ImageAnalysisType::General => "Describe this image in detail.",
            ImageAnalysisType::OcrOnly => "Extract all text from this image.",
            ImageAnalysisType::Objects => "Identify all objects in this image.",
            ImageAnalysisType::Scene => "Describe the scene and setting of this image.",
            ImageAnalysisType::Faces => "Describe any people and their expressions in this image.",
        };

        self.inner.understand_image(attachment, Some(prompt)).await
    }
}

/// Image analysis types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageAnalysisType {
    /// General description
    General,
    /// OCR only
    OcrOnly,
    /// Object detection
    Objects,
    /// Scene understanding
    Scene,
    /// Face analysis
    Faces,
}

/// Document understanding specialized struct
#[derive(Debug, Clone)]
pub struct DocumentUnderstandingService {
    inner: LLMMediaUnderstanding,
}

impl DocumentUnderstandingService {
    /// Create a new document understanding service
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            inner: LLMMediaUnderstanding::new(provider, model),
        }
    }

    /// Extract text from document
    pub async fn extract_text(&self, attachment: &ParsedAttachment) -> Result<String> {
        let result = self.inner.understand_document(attachment, true).await?;
        Ok(result.content)
    }

    /// Summarize document
    pub async fn summarize(&self, attachment: &ParsedAttachment) -> Result<String> {
        let result = self.inner.understand_document(attachment, false).await?;
        Ok(result.summary)
    }

    /// Extract entities from document
    pub async fn extract_entities(
        &self,
        attachment: &ParsedAttachment,
    ) -> Result<Vec<ExtractedEntity>> {
        let result = self.inner.understand_document(attachment, true).await?;
        Ok(result.entities)
    }
}

/// Audio transcription specialized struct
#[derive(Debug, Clone)]
pub struct AudioTranscriptionService {
    inner: LLMMediaUnderstanding,
}

impl AudioTranscriptionService {
    /// Create a new audio transcription service
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            inner: LLMMediaUnderstanding::new(provider, model),
        }
    }

    /// Transcribe audio to text
    pub async fn transcribe(
        &self,
        attachment: &ParsedAttachment,
        language: Option<&str>,
    ) -> Result<AudioTranscription> {
        self.inner.transcribe_audio(attachment, language).await
    }

    /// Transcribe with speaker diarization
    pub async fn transcribe_with_diarization(
        &self,
        attachment: &ParsedAttachment,
        language: Option<&str>,
    ) -> Result<AudioTranscription> {
        // Note: In a real implementation, this would use a config with
        // enable_diarization = true
        self.inner.transcribe_audio(attachment, language).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::attachment::PlatformSource;

    fn create_test_attachment(attachment_type: AttachmentType) -> ParsedAttachment {
        ParsedAttachment {
            id: uuid::Uuid::new_v4().to_string(),
            attachment_type,
            file_key: Some("test_key".to_string()),
            url: None,
            file_name: Some("test_file.jpg".to_string()),
            file_size: Some(1024),
            mime_type: Some("image/jpeg".to_string()),
            extension: Some("jpg".to_string()),
            source_platform: PlatformSource::Generic,
            metadata: HashMap::new(),
        }
    }

    fn create_test_audio_attachment() -> ParsedAttachment {
        ParsedAttachment {
            id: uuid::Uuid::new_v4().to_string(),
            attachment_type: AttachmentType::Audio,
            file_key: Some("audio_key".to_string()),
            url: None,
            file_name: Some("test_audio.mp3".to_string()),
            file_size: Some(1024 * 1024),
            mime_type: Some("audio/mpeg".to_string()),
            extension: Some("mp3".to_string()),
            source_platform: PlatformSource::Generic,
            metadata: {
                let mut m = HashMap::new();
                m.insert("duration_ms".to_string(), "30000".to_string());
                m
            },
        }
    }

    #[tokio::test]
    async fn test_llm_media_understanding_new() {
        let understanding = LLMMediaUnderstanding::new("openai", "gpt-4o");
        assert_eq!(understanding.name(), "openai");
        assert!(understanding.is_available().await);
    }

    #[tokio::test]
    async fn test_understand_image() {
        let understanding = LLMMediaUnderstanding::new("openai", "gpt-4o");
        let attachment = create_test_attachment(AttachmentType::Image);

        let result = understanding
            .understand_image(&attachment, None)
            .await
            .unwrap();
        assert!(!result.description.is_empty());
        assert!(result.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_understand_document() {
        let understanding = LLMMediaUnderstanding::new("openai", "gpt-4o");
        let mut attachment = create_test_attachment(AttachmentType::Document);
        attachment.mime_type = Some("application/pdf".to_string());
        attachment.extension = Some("pdf".to_string());
        attachment.file_name = Some("document.pdf".to_string());

        let result = understanding
            .understand_document(&attachment, true)
            .await
            .unwrap();
        assert_eq!(result.document_type, DocumentType::Pdf);
        assert!(result.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_transcribe_audio() {
        let understanding = LLMMediaUnderstanding::new("openai", "whisper-1");
        let attachment = create_test_audio_attachment();

        let result = understanding
            .transcribe_audio(&attachment, Some("en"))
            .await
            .unwrap();
        assert!(!result.text.is_empty());
        assert_eq!(result.language, Some("en".to_string()));
        assert_eq!(result.duration, Some(30.0));
    }

    #[test]
    fn test_infer_document_type() {
        let understanding = LLMMediaUnderstanding::new("openai", "gpt-4o");

        let test_cases = vec![
            ("pdf", "application/pdf", DocumentType::Pdf),
            (
                "docx",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                DocumentType::Word,
            ),
            ("txt", "text/plain", DocumentType::Text),
            ("md", "text/markdown", DocumentType::Markdown),
            ("html", "text/html", DocumentType::Html),
            ("rs", "text/plain", DocumentType::Code),
        ];

        for (ext, mime, expected) in test_cases {
            let attachment = ParsedAttachment {
                id: "test".to_string(),
                attachment_type: AttachmentType::Document,
                file_key: None,
                url: None,
                file_name: Some(format!("test.{}", ext)),
                file_size: None,
                mime_type: Some(mime.to_string()),
                extension: Some(ext.to_string()),
                source_platform: PlatformSource::Generic,
                metadata: HashMap::new(),
            };

            let doc_type = understanding.infer_document_type(&attachment);
            assert_eq!(doc_type, expected, "Failed for extension: {}", ext);
        }
    }

    #[test]
    fn test_document_type_display() {
        assert_eq!(DocumentType::Pdf.to_string(), "pdf");
        assert_eq!(DocumentType::Text.to_string(), "text");
        assert_eq!(DocumentType::Code.to_string(), "code");
    }

    #[test]
    fn test_supported_types() {
        let understanding = LLMMediaUnderstanding::new("openai", "gpt-4o");
        let types = understanding.supported_types();
        assert!(types.contains(&AttachmentType::Image));
        assert!(types.contains(&AttachmentType::Document));
        assert!(types.contains(&AttachmentType::Audio));
    }

    #[tokio::test]
    async fn test_image_understanding_service() {
        let service = ImageUnderstandingService::new("openai", "gpt-4o");
        let attachment = create_test_attachment(AttachmentType::Image);

        let result = service
            .analyze(&attachment, ImageAnalysisType::General)
            .await
            .unwrap();
        assert!(!result.description.is_empty());
    }

    #[tokio::test]
    async fn test_document_understanding_service() {
        let service = DocumentUnderstandingService::new("openai", "gpt-4o");
        let mut attachment = create_test_attachment(AttachmentType::Document);
        attachment.extension = Some("txt".to_string());

        let result = service.summarize(&attachment).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_audio_transcription_service() {
        let service = AudioTranscriptionService::new("openai", "whisper-1");
        let attachment = create_test_audio_attachment();

        let result = service.transcribe(&attachment, Some("zh")).await.unwrap();
        assert_eq!(result.language, Some("zh".to_string()));
    }
}
