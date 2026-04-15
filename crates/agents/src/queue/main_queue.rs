//! Main Queue
//!
//! Sequential task execution queue with priority support.

use std::collections::VecDeque;

use crate::queue::QueueTask;

/// Main queue - processes tasks sequentially
pub struct MainQueueState {
    queue: VecDeque<QueueTask>,
    current: Option<QueueTask>,
}

impl MainQueueState {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
        }
    }

    pub fn enqueue(&mut self, task: QueueTask) {
        // Insert based on priority
        let pos = self
            .queue
            .iter()
            .position(|t| t.priority < task.priority)
            .unwrap_or(self.queue.len());
        self.queue.insert(pos, task);
    }

    pub fn dequeue(&mut self) -> Option<QueueTask> {
        if self.current.is_none() {
            self.current = self.queue.pop_front();
        }
        self.current.clone()
    }

    pub fn complete_current(&mut self) {
        self.current = None;
    }

    pub fn len(&self) -> usize {
        self.queue.len() + if self.current.is_some() { 1 } else { 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for MainQueueState {
    fn default() -> Self {
        Self::new()
    }
}
