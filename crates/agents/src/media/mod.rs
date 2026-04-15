//! Media Handling Module
//!
//! Provides functionality for downloading and processing media files
//! from various platforms including Lark/Feishu.

pub mod attachment;
pub mod downloader;
pub mod formatter;
pub mod mime;
pub mod multimodal;
pub mod store;
pub mod understanding;

pub use attachment::{AttachmentParser, AttachmentType, ParsedAttachment, PlatformSource};
pub use downloader::{
    DownloadConfig, LazyUrl, MediaCache, MediaDownloadResult, MediaDownloader, MediaType,
    PlatformMediaDownloader,
};
pub use formatter::{
    AttachmentContent, FormattedMessage, LLMFormat, MessageAttachment, MessageFormatter,
    MessageRole,
};
pub use mime::MimeDetector;
pub use multimodal::{
    ImageDownloader, ImageFormat, LarkImageDownloader, MultimodalContent, MultimodalProcessor,
    ProcessedImage,
};
pub use store::{MediaMetadata, MediaStore, MediaStoreConfig};
pub use understanding::{
    AudioTranscription, AudioTranscriptionService, DocumentSection, DocumentType,
    DocumentUnderstanding, DocumentUnderstandingService, ExtractedEntity, ImageAnalysisType,
    ImageDimensions, ImageUnderstanding, ImageUnderstandingService, LLMMediaUnderstanding,
    MediaUnderstanding, SpeakerSegment, UnderstandingConfig, WordTimestamp,
};
