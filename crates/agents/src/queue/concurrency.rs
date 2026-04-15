//! Concurrency Queue

use crate::error::Result;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Concurrent task
#[derive(Debug)]
pub struct ConcurrentTask {
    pub id: Uuid,
    pub semaphore: Arc<Semaphore>,
    pub task: Box<dyn FnOnce() -> Result<()> + Send>,
}

/// Concurrency-limited queue
pub struct ConcurrencyQueue {
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
}

impl ConcurrencyQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    pub async fn execute<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let _permit = self.semaphore.acquire().await
            .map_err(|e| crate::error::AgentError::runtime(format!("Semaphore error: {}", e)))?;
        
        tokio::task::spawn_blocking(f).await
            .map_err(|e| crate::error::AgentError::runtime(format!("Task error: {}", e)))?
    }

    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}
