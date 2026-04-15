//! Runtime Scheduler

use std::collections::VecDeque;

use uuid::Uuid;

use crate::error::Result;

/// Scheduled task for runtime scheduler
pub struct ScheduledTask {
    pub id: Uuid,
    pub priority: TaskPriority,
    pub handler: Box<dyn FnOnce() -> Result<()> + Send>,
}

/// Type alias for backward compatibility
pub type Task = ScheduledTask;

impl std::fmt::Debug for ScheduledTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScheduledTask")
            .field("id", &self.id)
            .field("priority", &self.priority)
            .field("handler", &"<FnOnce>")
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task scheduler
pub struct Scheduler {
    queues: [VecDeque<ScheduledTask>; 4],
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            queues: [
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
        }
    }

    pub fn schedule(&mut self, task: Task) {
        let idx = task.priority as usize;
        self.queues[idx].push_back(task);
    }

    pub fn next(&mut self) -> Option<Task> {
        for queue in self.queues.iter_mut().rev() {
            if let Some(task) = queue.pop_front() {
                return Some(task);
            }
        }
        None
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
