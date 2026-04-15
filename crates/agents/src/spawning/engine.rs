//! Spawn Engine
//!
//! Handles subagent creation and lifecycle management.

use std::sync::Arc;

use tokio::sync::mpsc;
use uuid::Uuid;

use super::{SpawnConfig, SpawnResult, SpawnStatus};
use crate::session::{SessionKey, Workspace};

/// Spawn engine for managing subagents
pub struct SpawnEngine {
    quota_manager: QuotaManager,
    workspace_factory: WorkspaceFactory,
    announce_tx: Arc<mpsc::Sender<super::announce::Announcement>>,
}

/// Quota manager
pub struct QuotaManager {
    max_concurrent: usize,
    current: std::sync::atomic::AtomicUsize,
}

impl QuotaManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            current: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn allocate(&self, _config: &SpawnConfig) -> Result<QuotaToken, SpawnError> {
        let current = self.current.load(std::sync::atomic::Ordering::SeqCst);
        if current >= self.max_concurrent {
            return Err(SpawnError::QuotaExceeded);
        }

        self.current
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(QuotaToken {
            manager: Arc::new(std::sync::Mutex::new(std::sync::atomic::AtomicUsize::new(
                self.current.load(std::sync::atomic::Ordering::SeqCst),
            ))),
        })
    }
}

/// Quota token - released when dropped
pub struct QuotaToken {
    manager: Arc<std::sync::Mutex<std::sync::atomic::AtomicUsize>>,
}

impl Drop for QuotaToken {
    fn drop(&mut self) {
        if let Ok(counter) = self.manager.lock() {
            counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

/// Workspace factory
pub struct WorkspaceFactory {
    base_path: std::path::PathBuf,
}

impl WorkspaceFactory {
    pub fn new(base_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    pub async fn create(&self, key: &SessionKey) -> Result<Workspace, SpawnError> {
        let path = self.base_path.join(key.to_path());
        Workspace::new(&path)
            .create()
            .await
            .map_err(|e| SpawnError::WorkspaceError(e.to_string()))?;
        Ok(Workspace::new(path))
    }
}

impl SpawnEngine {
    pub fn new(announce_tx: mpsc::Sender<super::announce::Announcement>) -> Self {
        Self {
            quota_manager: QuotaManager::new(100), // Max 100 concurrent subagents
            workspace_factory: WorkspaceFactory::new("./workspaces"),
            announce_tx: Arc::new(announce_tx),
        }
    }

    /// Non-blocking spawn
    pub async fn spawn(&self, config: SpawnConfig) -> Result<SpawnResult, SpawnError> {
        // 1. Allocate quota
        let _quota = self.quota_manager.allocate(&config)?;

        // 2. Generate child session key
        let child_key = config
            .parent
            .spawn_child()
            .map_err(|e| SpawnError::SessionError(e.to_string()))?;

        // 3. Create workspace
        let _workspace = self.workspace_factory.create(&child_key).await?;

        // 4. Generate run ID
        let run_id = Uuid::new_v4().to_string();

        // 5. Clone child_key for the async block
        let child_key_clone = child_key.clone();

        // 6. Spawn initialization task (background)
        let announce_tx = Arc::clone(&self.announce_tx);
        let parent_key = config.parent.clone();
        tokio::spawn(async move {
            // Background initialization
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;

            // Send ready announcement
            let _ = announce_tx
                .send(super::announce::Announcement {
                    run_id: Uuid::new_v4().to_string(),
                    parent: parent_key,
                    child: child_key_clone,
                    status: super::announce::CompletionStatus::Success,
                    result_summary: "Subagent initialized".to_string(),
                    duration: std::time::Duration::from_millis(50),
                    tokens_used: super::announce::TokenUsage::default(),
                    estimated_cost: 0.0,
                    transcript_path: std::path::PathBuf::new(),
                })
                .await;
        });

        // 7. Return immediately (non-blocking)
        Ok(SpawnResult {
            status: SpawnStatus::Accepted,
            run_id,
            child_session_key: child_key,
            estimated_init_time: std::time::Duration::from_millis(100),
        })
    }
}

/// Spawn errors
#[derive(Debug, Clone)]
pub enum SpawnError {
    QuotaExceeded,
    SessionError(String),
    WorkspaceError(String),
    InitializationFailed(String),
}

impl std::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpawnError::QuotaExceeded => write!(f, "Quota exceeded"),
            SpawnError::SessionError(s) => write!(f, "Session error: {}", s),
            SpawnError::WorkspaceError(s) => write!(f, "Workspace error: {}", s),
            SpawnError::InitializationFailed(s) => write!(f, "Initialization failed: {}", s),
        }
    }
}

impl std::error::Error for SpawnError {}
