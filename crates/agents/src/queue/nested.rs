//! Nested Queue
//!
//! Prevents infinite recursion by tracking nesting depth.

use std::collections::HashMap;

use uuid::Uuid;

use crate::error::Result;
use crate::queue::QueueError;

/// Nested task
pub struct NestedTask {
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub level: u32,
    pub task: Box<dyn FnOnce() -> Result<()> + Send>,
}

impl std::fmt::Debug for NestedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NestedTask")
            .field("id", &self.id)
            .field("parent", &self.parent)
            .field("level", &self.level)
            .field("task", &"<closure>")
            .finish()
    }
}

/// Nested queue for hierarchical tasks
pub struct NestedQueue {
    tasks: Vec<NestedTask>,
}

impl NestedQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add(&mut self, task: NestedTask) {
        self.tasks.push(task);
    }

    pub fn get_ready(&mut self) -> Vec<NestedTask> {
        let mut ready = Vec::new();
        let mut i = 0;
        while i < self.tasks.len() {
            let parent = self.tasks[i].parent;
            let is_ready = parent.is_none() || !self.tasks.iter().any(|t| Some(t.id) == parent);
            if is_ready {
                ready.push(self.tasks.swap_remove(i));
            } else {
                i += 1;
            }
        }
        ready
    }

    #[allow(dead_code)]
    fn is_parent_done(&self, parent_id: Option<Uuid>) -> bool {
        parent_id.map_or(true, |id| !self.tasks.iter().any(|t| t.id == id))
    }
}

impl Default for NestedQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Nested queue with depth tracking
pub struct NestedQueueState {
    max_depth: usize,
    active_depths: HashMap<String, usize>,
}

impl NestedQueueState {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            active_depths: HashMap::new(),
        }
    }

    /// Try to enter nested execution
    pub fn enter(&mut self, session_id: &str) -> std::result::Result<usize, QueueError> {
        let current = self.active_depths.get(session_id).copied().unwrap_or(0);

        if current >= self.max_depth {
            return Err(QueueError::NestedLimitExceeded);
        }

        let new_depth = current + 1;
        self.active_depths.insert(session_id.to_string(), new_depth);
        Ok(new_depth)
    }

    /// Exit nested execution
    pub fn exit(&mut self, session_id: &str) {
        if let Some(current) = self.active_depths.get_mut(session_id) {
            if *current > 1 {
                *current -= 1;
            } else {
                self.active_depths.remove(session_id);
            }
        }
    }

    /// Get current depth
    pub fn get_depth(&self, session_id: &str) -> usize {
        self.active_depths.get(session_id).copied().unwrap_or(0)
    }
}

impl Default for NestedQueueState {
    fn default() -> Self {
        Self::new(5) // Default max depth of 5
    }
}
