//! Media Store
//!
//! Provides persistent storage for media files with organized directory
//! structure. Files are organized by date and use UUID-based filenames for
//! uniqueness.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Local, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{AgentError, Result};

/// Media store configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaStoreConfig {
    /// Base directory for storing media files
    pub base_dir: PathBuf,
    /// Whether to organize files by date (YYYY/MM/DD)
    pub organize_by_date: bool,
    /// Whether to use UUID as filename (otherwise preserve original name)
    pub use_uuid_filename: bool,
    /// Maximum file size in bytes (default: 100MB)
    pub max_file_size: usize,
    /// Allowed file extensions (empty means all allowed)
    pub allowed_extensions: Vec<String>,
}

impl Default for MediaStoreConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("./media"),
            organize_by_date: true,
            use_uuid_filename: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_extensions: vec![],
        }
    }
}

/// Media file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    /// Unique identifier (UUID)
    pub id: String,
    /// Original filename
    pub original_name: String,
    /// Stored filename
    pub stored_name: String,
    /// File path relative to base directory
    pub relative_path: PathBuf,
    /// Absolute file path
    pub absolute_path: PathBuf,
    /// File size in bytes
    pub file_size: u64,
    /// MIME type
    pub mime_type: String,
    /// File extension
    pub extension: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<Utc>,
    /// Last access timestamp
    pub last_accessed_at: Option<chrono::DateTime<Utc>>,
    /// Additional metadata
    pub extra: HashMap<String, String>,
}

/// Media store for file operations
#[derive(Debug, Clone)]
pub struct MediaStore {
    config: MediaStoreConfig,
}

impl MediaStore {
    /// Create a new media store with configuration
    ///
    /// # Arguments
    /// * `config` - Store configuration
    ///
    /// # Returns
    /// New MediaStore instance
    pub fn new(config: MediaStoreConfig) -> Self {
        Self { config }
    }

    /// Create a new media store with default configuration
    ///
    /// # Returns
    /// New MediaStore instance with default config
    pub fn default() -> Self {
        Self::new(MediaStoreConfig::default())
    }

    /// Initialize the store by creating base directory
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.config.base_dir)
            .await
            .map_err(|e| {
                AgentError::platform(format!("Failed to create media directory: {}", e))
            })?;
        info!("Media store initialized at: {:?}", self.config.base_dir);
        Ok(())
    }

    /// Save media file to store
    ///
    /// # Arguments
    /// * `data` - File data as bytes
    /// * `original_name` - Original filename
    /// * `mime_type` - MIME type of the file
    /// * `extra` - Additional metadata
    ///
    /// # Returns
    /// MediaMetadata for the saved file
    pub async fn save(
        &self,
        data: &[u8],
        original_name: impl Into<String>,
        mime_type: impl Into<String>,
        extra: Option<HashMap<String, String>>,
    ) -> Result<MediaMetadata> {
        let original_name = original_name.into();
        let mime_type = mime_type.into();

        // Check file size
        if data.len() > self.config.max_file_size {
            return Err(AgentError::platform(format!(
                "File too large: {} bytes (max: {})",
                data.len(),
                self.config.max_file_size
            ))
            .into());
        }

        // Validate extension if restrictions are set
        let extension = Self::extract_extension(&original_name);
        if !self.config.allowed_extensions.is_empty() {
            let ext_lower = extension.to_lowercase();
            if !self
                .config
                .allowed_extensions
                .iter()
                .any(|e| e.to_lowercase() == ext_lower)
            {
                return Err(AgentError::platform(format!(
                    "File extension '{}' not allowed",
                    extension
                ))
                .into());
            }
        }

        // Generate unique ID and filename
        let id = Uuid::new_v4().to_string();
        let stored_name = if self.config.use_uuid_filename {
            format!("{}.{}", id, extension.trim_start_matches('.'))
        } else {
            Self::sanitize_filename(&original_name)
        };

        // Build directory path
        let dir_path = self.build_directory_path();
        let relative_path = dir_path.join(&stored_name);
        let absolute_path = self.config.base_dir.join(&relative_path);

        // Create directory if needed
        let parent_dir = absolute_path
            .parent()
            .ok_or_else(|| AgentError::platform("Invalid file path"))?;
        fs::create_dir_all(parent_dir)
            .await
            .map_err(|e| AgentError::platform(format!("Failed to create directory: {}", e)))?;

        // Write file
        let mut file = fs::File::create(&absolute_path)
            .await
            .map_err(|e| AgentError::platform(format!("Failed to create file: {}", e)))?;

        file.write_all(data)
            .await
            .map_err(|e| AgentError::platform(format!("Failed to write file: {}", e)))?;

        file.flush()
            .await
            .map_err(|e| AgentError::platform(format!("Failed to flush file: {}", e)))?;

        let metadata = MediaMetadata {
            id,
            original_name: original_name.clone(),
            stored_name,
            relative_path,
            absolute_path,
            file_size: data.len() as u64,
            mime_type,
            extension,
            created_at: Utc::now(),
            last_accessed_at: None,
            extra: extra.unwrap_or_default(),
        };

        info!(
            "Saved media file: {} ({} bytes)",
            metadata.original_name, metadata.file_size
        );

        Ok(metadata)
    }

    /// Save media file from stream
    ///
    /// # Arguments
    /// * `reader` - Async reader for file data
    /// * `original_name` - Original filename
    /// * `mime_type` - MIME type of the file
    /// * `extra` - Additional metadata
    ///
    /// # Returns
    /// MediaMetadata for the saved file
    pub async fn save_from_stream<R>(
        &self,
        mut reader: R,
        original_name: impl Into<String>,
        mime_type: impl Into<String>,
        extra: Option<HashMap<String, String>>,
    ) -> Result<MediaMetadata>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        let original_name = original_name.into();
        let mime_type = mime_type.into();

        // Read all data
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .await
            .map_err(|e| AgentError::platform(format!("Failed to read stream: {}", e)))?;

        // Use regular save method
        self.save(&data, original_name, mime_type, extra).await
    }

    /// Get media file by ID
    ///
    /// # Arguments
    /// * `id` - File ID (UUID)
    ///
    /// # Returns
    /// File data as bytes if found
    pub async fn get(&self, id: &str) -> Result<Option<(Vec<u8>, MediaMetadata)>> {
        // Find file by ID - need to search in directory structure
        let file_path = self.find_file_by_id(id).await?;

        match file_path {
            Some(path) => {
                let mut file = fs::File::open(&path)
                    .await
                    .map_err(|e| AgentError::platform(format!("Failed to open file: {}", e)))?;

                let mut data = Vec::new();
                file.read_to_end(&mut data)
                    .await
                    .map_err(|e| AgentError::platform(format!("Failed to read file: {}", e)))?;

                // Build metadata from file info
                let metadata = self.build_metadata_from_path(&path, id).await?;

                debug!("Retrieved media file: {} ({} bytes)", id, data.len());

                Ok(Some((data, metadata)))
            }
            None => {
                warn!("Media file not found: {}", id);
                Ok(None)
            }
        }
    }

    /// Get file path by ID
    ///
    /// # Arguments
    /// * `id` - File ID (UUID)
    ///
    /// # Returns
    /// Absolute path to file if found
    pub async fn get_path(&self, id: &str) -> Result<Option<PathBuf>> {
        self.find_file_by_id(id).await
    }

    /// Delete media file by ID
    ///
    /// # Arguments
    /// * `id` - File ID (UUID)
    ///
    /// # Returns
    /// True if deleted, false if not found
    pub async fn delete(&self, id: &str) -> Result<bool> {
        match self.find_file_by_id(id).await? {
            Some(path) => {
                fs::remove_file(&path)
                    .await
                    .map_err(|e| AgentError::platform(format!("Failed to delete file: {}", e)))?;

                info!("Deleted media file: {}", id);

                // Try to clean up empty parent directories
                self.cleanup_empty_dirs(&path).await?;

                Ok(true)
            }
            None => {
                warn!("Cannot delete - media file not found: {}", id);
                Ok(false)
            }
        }
    }

    /// Check if file exists
    ///
    /// # Arguments
    /// * `id` - File ID (UUID)
    ///
    /// # Returns
    /// True if file exists
    pub async fn exists(&self, id: &str) -> Result<bool> {
        let path = self.find_file_by_id(id).await?;
        Ok(path.is_some())
    }

    /// List all media files
    ///
    /// # Returns
    /// List of media metadata
    pub async fn list_all(&self) -> Result<Vec<MediaMetadata>> {
        let mut files = Vec::new();
        self.collect_files_recursive(&self.config.base_dir, &mut files)
            .await?;
        Ok(files)
    }

    /// List files by date
    ///
    /// # Arguments
    /// * `year` - Year (e.g., 2024)
    /// * `month` - Month (1-12, optional)
    /// * `day` - Day (1-31, optional)
    ///
    /// # Returns
    /// List of media metadata matching the date criteria
    pub async fn list_by_date(
        &self,
        year: i32,
        month: Option<u32>,
        day: Option<u32>,
    ) -> Result<Vec<MediaMetadata>> {
        let mut path = self.config.base_dir.join(year.to_string());

        if let Some(m) = month {
            path = path.join(format!("{:02}", m));
            if let Some(d) = day {
                path = path.join(format!("{:02}", d));
            }
        }

        if !path.exists() {
            return Ok(vec![]);
        }

        let mut files = Vec::new();
        self.collect_files_recursive(&path, &mut files).await?;
        Ok(files)
    }

    /// Get storage statistics
    ///
    /// # Returns
    /// (total_files, total_bytes)
    pub async fn get_stats(&self) -> Result<(usize, u64)> {
        let files = self.list_all().await?;
        let total_bytes: u64 = files.iter().map(|f| f.file_size).sum();
        Ok((files.len(), total_bytes))
    }

    /// Clean up old files
    ///
    /// # Arguments
    /// * `max_age_days` - Maximum age in days
    ///
    /// # Returns
    /// Number of files deleted
    pub async fn cleanup_old_files(&self, max_age_days: i64) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(max_age_days);
        let files = self.list_all().await?;
        let mut deleted = 0;

        for file in files {
            if file.created_at < cutoff {
                if self.delete(&file.id).await? {
                    deleted += 1;
                }
            }
        }

        info!(
            "Cleaned up {} old files (older than {} days)",
            deleted, max_age_days
        );
        Ok(deleted)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: MediaStoreConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &MediaStoreConfig {
        &self.config
    }

    /// Build directory path based on configuration
    fn build_directory_path(&self) -> PathBuf {
        if self.config.organize_by_date {
            let now = Local::now();
            PathBuf::from(format!(
                "{:04}/{:02}/{:02}",
                now.year(),
                now.month(),
                now.day()
            ))
        } else {
            PathBuf::from("")
        }
    }

    /// Extract file extension from filename
    fn extract_extension(filename: &str) -> String {
        Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_lowercase()))
            .unwrap_or_default()
    }

    /// Sanitize filename for safe storage
    fn sanitize_filename(filename: &str) -> String {
        // Remove path separators and other dangerous characters
        filename
            .replace(['/', '\\', '\0'], "_")
            .replace("..", "_")
            .trim_start_matches('.')
            .to_string()
    }

    /// Find file by ID (UUID)
    async fn find_file_by_id(&self, id: &str) -> Result<Option<PathBuf>> {
        // Search recursively for file starting with the UUID
        let mut result = None;
        self.search_recursive(&self.config.base_dir, id, &mut result)
            .await?;
        Ok(result)
    }

    /// Recursive search for file
    async fn search_recursive(
        &self,
        dir: &Path,
        id: &str,
        result: &mut Option<PathBuf>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(dir)
            .await
            .map_err(|e| AgentError::platform(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| AgentError::platform(format!("Failed to read entry: {}", e)))?
        {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .await
                .map_err(|e| AgentError::platform(format!("Failed to get file type: {}", e)))?;

            if file_type.is_dir() {
                // Recurse into subdirectory
                Box::pin(self.search_recursive(&path, id, result)).await?;
                if result.is_some() {
                    return Ok(());
                }
            } else if file_type.is_file() {
                // Check if filename starts with the ID
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    if name.starts_with(id) {
                        *result = Some(path);
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }

    /// Collect all files recursively
    async fn collect_files_recursive(
        &self,
        dir: &Path,
        files: &mut Vec<MediaMetadata>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(dir)
            .await
            .map_err(|e| AgentError::platform(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| AgentError::platform(format!("Failed to read entry: {}", e)))?
        {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .await
                .map_err(|e| AgentError::platform(format!("Failed to get file type: {}", e)))?;

            if file_type.is_dir() {
                Box::pin(self.collect_files_recursive(&path, files)).await?;
            } else if file_type.is_file() {
                // Extract ID from filename (assuming UUID format)
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    if let Ok(metadata) = self.build_metadata_from_path(&path, name).await {
                        files.push(metadata);
                    }
                }
            }
        }

        Ok(())
    }

    /// Build metadata from file path
    async fn build_metadata_from_path(&self, path: &Path, id: &str) -> Result<MediaMetadata> {
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| AgentError::platform(format!("Failed to get file metadata: {}", e)))?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let relative_path = path
            .strip_prefix(&self.config.base_dir)
            .unwrap_or(path)
            .to_path_buf();

        let extension = Self::extract_extension(&file_name);

        // Guess MIME type from extension
        let mime_type = mime_guess_from_extension(&extension);

        Ok(MediaMetadata {
            id: id.to_string(),
            original_name: file_name.clone(),
            stored_name: file_name,
            relative_path,
            absolute_path: path.to_path_buf(),
            file_size: metadata.len(),
            mime_type,
            extension,
            created_at: Utc::now(), // Could use file creation time if available
            last_accessed_at: Some(Utc::now()),
            extra: HashMap::new(),
        })
    }

    /// Clean up empty parent directories
    async fn cleanup_empty_dirs(&self, file_path: &Path) -> Result<()> {
        if let Some(parent) = file_path.parent() {
            // Only clean up directories within our base directory
            if parent.starts_with(&self.config.base_dir) && parent != self.config.base_dir {
                if let Ok(mut entries) = fs::read_dir(parent).await {
                    if entries.next_entry().await.is_ok_and(|e| e.is_none()) {
                        // Directory is empty, remove it
                        let _ = fs::remove_dir(parent).await;
                        // Recursively clean up parent
                        Box::pin(self.cleanup_empty_dirs(parent)).await?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Guess MIME type from file extension
fn mime_guess_from_extension(ext: &str) -> String {
    let ext = ext.trim_start_matches('.').to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "json" => "application/json",
        "xml" => "application/xml",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "gzip" => "application/gzip",
        _ => "application/octet-stream",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    async fn create_test_store() -> (MediaStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = MediaStoreConfig {
            base_dir: temp_dir.path().to_path_buf(),
            organize_by_date: false,
            use_uuid_filename: true,
            max_file_size: 10 * 1024 * 1024,
            allowed_extensions: vec![],
        };
        let store = MediaStore::new(config);
        store.init().await.unwrap();
        (store, temp_dir)
    }

    #[tokio::test]
    async fn test_save_and_get() {
        let (store, _temp) = create_test_store().await;

        let data = b"Hello, World!";
        let metadata = store
            .save(data, "test.txt", "text/plain", None)
            .await
            .unwrap();

        assert!(!metadata.id.is_empty());
        assert_eq!(metadata.file_size, 13);
        assert_eq!(metadata.mime_type, "text/plain");

        let (retrieved_data, _) = store.get(&metadata.id).await.unwrap().unwrap();
        assert_eq!(retrieved_data, data);
    }

    #[tokio::test]
    async fn test_delete() {
        let (store, _temp) = create_test_store().await;

        let data = b"Test data";
        let metadata = store
            .save(data, "test.txt", "text/plain", None)
            .await
            .unwrap();

        assert!(store.exists(&metadata.id).await.unwrap());

        let deleted = store.delete(&metadata.id).await.unwrap();
        assert!(deleted);

        assert!(!store.exists(&metadata.id).await.unwrap());
    }

    #[tokio::test]
    async fn test_file_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let config = MediaStoreConfig {
            base_dir: temp_dir.path().to_path_buf(),
            organize_by_date: false,
            use_uuid_filename: true,
            max_file_size: 10, // 10 bytes max
            allowed_extensions: vec![],
        };
        let store = MediaStore::new(config);
        store.init().await.unwrap();

        let data = b"This is more than 10 bytes";
        let result = store.save(data, "test.txt", "text/plain", None).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_extension() {
        assert_eq!(MediaStore::extract_extension("file.txt"), ".txt");
        assert_eq!(MediaStore::extract_extension("file.PNG"), ".png");
        assert_eq!(MediaStore::extract_extension("file"), "");
        assert_eq!(MediaStore::extract_extension("path/to/file.jpg"), ".jpg");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(MediaStore::sanitize_filename("file.txt"), "file.txt");
        // Path separators and path traversal are replaced with _
        assert_eq!(
            MediaStore::sanitize_filename("../etc/passwd"),
            "__etc_passwd"
        );
        assert_eq!(MediaStore::sanitize_filename("file/name"), "file_name");
        assert_eq!(MediaStore::sanitize_filename(".hidden"), "hidden");
    }

    #[test]
    fn test_mime_guess() {
        assert_eq!(mime_guess_from_extension(".jpg"), "image/jpeg");
        assert_eq!(mime_guess_from_extension(".png"), "image/png");
        assert_eq!(mime_guess_from_extension(".pdf"), "application/pdf");
        assert_eq!(
            mime_guess_from_extension(".unknown"),
            "application/octet-stream"
        );
    }
}
