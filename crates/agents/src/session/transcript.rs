//! Transcript Management
//!
//! Handles JSONL transcript files for session recording.

use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Transcript entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub timestamp: u64,
    pub role: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

/// Session transcript
pub struct Transcript {
    entries: Vec<TranscriptEntry>,
    file_path: Option<std::path::PathBuf>,
}

impl Transcript {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            file_path: None,
        }
    }

    pub fn with_file(path: impl AsRef<Path>) -> Self {
        Self {
            entries: Vec::new(),
            file_path: Some(path.as_ref().to_path_buf()),
        }
    }

    pub fn add_entry(&mut self, role: impl Into<String>, content: impl Into<String>) {
        let entry = TranscriptEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            role: role.into(),
            content: content.into(),
            metadata: None,
        };
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[TranscriptEntry] {
        &self.entries
    }

    /// Save transcript to JSONL file
    pub async fn save(&self) -> Result<(), TranscriptError> {
        if let Some(path) = &self.file_path {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await
                .map_err(|e| TranscriptError::IoError(e.to_string()))?;

            for entry in &self.entries {
                let line = serde_json::to_string(entry)
                    .map_err(|e| TranscriptError::SerializationError(e.to_string()))?;
                file.write_all(line.as_bytes())
                    .await
                    .map_err(|e| TranscriptError::IoError(e.to_string()))?;
                file.write_all(b"\n")
                    .await
                    .map_err(|e| TranscriptError::IoError(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Load transcript from JSONL file
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, TranscriptError> {
        let file = File::open(path.as_ref())
            .await
            .map_err(|e| TranscriptError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut entries = Vec::new();
        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| TranscriptError::IoError(e.to_string()))?
        {
            let entry: TranscriptEntry = serde_json::from_str(&line)
                .map_err(|e| TranscriptError::SerializationError(e.to_string()))?;
            entries.push(entry);
        }

        Ok(Self {
            entries,
            file_path: Some(path.as_ref().to_path_buf()),
        })
    }

    /// Get last N entries
    pub fn last_n(&self, n: usize) -> &[TranscriptEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }
}

impl Default for Transcript {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum TranscriptError {
    IoError(String),
    SerializationError(String),
}

impl std::fmt::Display for TranscriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptError::IoError(s) => write!(f, "IO error: {}", s),
            TranscriptError::SerializationError(s) => write!(f, "Serialization error: {}", s),
        }
    }
}

impl std::error::Error for TranscriptError {}
