//! Thread management
//!
//! Provides thread control block and state management.

use super::TaskId;
// use crate::error::KernelResult; // Currently unused

/// Thread execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    /// Thread is ready to run
    Ready,
    /// Thread is currently executing
    Running,
    /// Thread is blocked waiting for resource
    Blocked,
    /// Thread has terminated
    Terminated,
}

/// Thread control block
#[derive(Debug)]
pub struct Thread {
    /// Thread ID
    pub tid: TaskId,
    /// Parent process ID
    pub pid: TaskId,
    /// Current execution state
    pub state: ThreadState,
    /// Stack pointer address
    pub stack_pointer: u64,
}

impl Thread {
    /// Create a new thread
    pub fn new(tid: TaskId, pid: TaskId) -> Self {
        Self {
            tid,
            pid,
            state: ThreadState::Ready,
            stack_pointer: 0,
        }
    }

    /// Perform context switch to next thread
    pub fn context_switch(&mut self, _next: &mut Thread) {
        // Save current context, restore next context
    }
}
