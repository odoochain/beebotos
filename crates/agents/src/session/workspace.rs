//! Workspace Management
//!
//! Manages working directories for agent sessions.

use std::path::{Path, PathBuf};

use tokio::fs;

/// Workspace for agent session
#[derive(Debug, Clone)]
pub struct Workspace {
    pub path: PathBuf,
}

impl Workspace {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub async fn create(&self) -> Result<(), WorkspaceError> {
        fs::create_dir_all(&self.path)
            .await
            .map_err(|e| WorkspaceError::CreateFailed(e.to_string()))?;
        Ok(())
    }

    pub fn file_path(&self, filename: impl AsRef<Path>) -> PathBuf {
        self.path.join(filename)
    }

    pub async fn write_file(
        &self,
        filename: impl AsRef<Path>,
        content: impl AsRef<[u8]>,
    ) -> Result<(), WorkspaceError> {
        let path = self.file_path(filename);
        fs::write(path, content)
            .await
            .map_err(|e| WorkspaceError::WriteFailed(e.to_string()))
    }

    pub async fn read_file(&self, filename: impl AsRef<Path>) -> Result<Vec<u8>, WorkspaceError> {
        let path = self.file_path(filename);
        fs::read(path)
            .await
            .map_err(|e| WorkspaceError::ReadFailed(e.to_string()))
    }
}

#[derive(Debug, Clone)]
pub enum WorkspaceError {
    CreateFailed(String),
    WriteFailed(String),
    ReadFailed(String),
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceError::CreateFailed(s) => write!(f, "Create failed: {}", s),
            WorkspaceError::WriteFailed(s) => write!(f, "Write failed: {}", s),
            WorkspaceError::ReadFailed(s) => write!(f, "Read failed: {}", s),
        }
    }
}

impl std::error::Error for WorkspaceError {}
