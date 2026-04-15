//! Process
//!
//! Process management and control block.

use super::TaskId;
use crate::error::Result;

/// Process lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Process created but not yet running
    New,
    /// Process is actively running
    Running,
    /// Process is sleeping/waiting
    Sleeping,
    /// Process has exited but not reaped
    Zombie,
    /// Process has terminated
    Terminated,
}

/// Process control block
#[derive(Debug)]
pub struct Process {
    /// Process ID
    pub pid: TaskId,
    /// Current process state
    pub state: ProcessState,
    /// Parent process ID
    pub parent: Option<TaskId>,
    /// Child process IDs
    pub children: Vec<TaskId>,
}

impl Process {
    /// Create new process with given PID
    pub fn new(pid: TaskId) -> Self {
        Self {
            pid,
            state: ProcessState::New,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Fork this process into a child
    pub fn fork(&self) -> Result<Self> {
        let child = Self::new(TaskId::new(self.pid.as_u64() + 1));
        Ok(child)
    }

    /// Terminate this process
    pub fn terminate(&mut self) {
        self.state = ProcessState::Terminated;
    }
}
