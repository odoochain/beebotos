//! Subagent Queue
//!
//! Parallel execution queue (max 5 concurrent).

use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::queue::QueueError;

/// Subagent task
#[derive(Debug)]
pub struct SubagentTask {
    pub id: Uuid,
    pub parent_agent: Uuid,
    pub description: String,
    pub priority: u8,
}

/// Subagent queue
pub struct SubagentQueue {
    queue: VecDeque<SubagentTask>,
}

impl SubagentQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, task: SubagentTask) {
        self.queue.push_back(task);
    }

    pub fn dequeue(&mut self) -> Option<SubagentTask> {
        self.queue.pop_front()
    }

    pub fn pending_for(&self, agent_id: Uuid) -> Vec<&SubagentTask> {
        self.queue
            .iter()
            .filter(|t| t.parent_agent == agent_id)
            .collect()
    }
}

impl Default for SubagentQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Subagent queue with concurrency limit
pub struct SubagentQueueState {
    semaphore: Arc<Semaphore>,
    #[allow(dead_code)]
    max_concurrent: usize,
}

impl SubagentQueueState {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    pub async fn acquire_permit(
        &self,
    ) -> std::result::Result<tokio::sync::OwnedSemaphorePermit, QueueError> {
        self.semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| QueueError::QueueFull)
    }

    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}
