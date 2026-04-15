//! Fork

use super::{Process, TaskId};
use crate::error::Result;

/// Fork a new process
pub fn fork(parent: &Process) -> Result<Process> {
    let child_pid = TaskId::new(parent.pid.as_u64() + 1);
    let mut child = Process::new(child_pid);
    child.parent = Some(parent.pid);

    tracing::info!(
        "Forked process {} from {}",
        child_pid.as_u64(),
        parent.pid.as_u64()
    );
    Ok(child)
}

/// Clone (Linux-style)
pub fn clone(flags: u64, stack: u64) -> Result<TaskId> {
    tracing::info!("Clone with flags {:#x}, stack {:#x}", flags, stack);
    Ok(TaskId::new(1))
}
