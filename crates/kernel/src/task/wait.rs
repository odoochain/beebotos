//! Wait

use super::process::ProcessState;
use super::Process;
use crate::error::Result;

/// Wait for child process
pub fn waitpid(pid: u64) -> Result<Option<(u64, i32)>> {
    tracing::info!("Waiting for process {}", pid);
    // Simplified - would actually wait
    Ok(Some((pid, 0)))
}

/// Wait for any child
pub fn wait_any() -> Result<Option<(u64, i32)>> {
    tracing::info!("Waiting for any child process");
    Ok(None)
}

/// Check if child has exited
pub fn check_child(process: &Process) -> bool {
    matches!(
        process.state,
        ProcessState::Terminated | ProcessState::Zombie
    )
}
