//! System Calls
//!
//! Low-level system call interface for task operations.

use crate::error::Result;

/// System call numbers
#[repr(u64)]
#[derive(Debug, Clone, Copy)]
pub enum SyscallNumber {
    /// Exit process
    Exit = 0,
    /// Fork process
    Fork = 1,
    /// Read from file descriptor
    Read = 2,
    /// Write to file descriptor
    Write = 3,
    /// Open file
    Open = 4,
    /// Close file descriptor
    Close = 5,
    /// Map memory
    Mmap = 6,
    /// Unmap memory
    Munmap = 7,
}

/// System call handler
pub struct SyscallHandler;

impl SyscallHandler {
    /// Create new syscall handler
    pub fn new() -> Self {
        Self
    }

    /// Handle system call by number
    pub fn handle(&self, num: u64, args: &[u64]) -> Result<i64> {
        match num {
            0 => self.sys_exit(args[0]),
            2 => self.sys_read(args[0], args[1], args[2]),
            3 => self.sys_write(args[0], args[1], args[2]),
            _ => Err(crate::error::KernelError::InvalidSyscall(num)),
        }
    }

    fn sys_exit(&self, code: u64) -> Result<i64> {
        tracing::info!("Process exit with code {}", code);
        Ok(0)
    }

    fn sys_read(&self, fd: u64, buf: u64, count: u64) -> Result<i64> {
        tracing::info!("Read {} bytes from fd {} to {:#x}", count, fd, buf);
        Ok(0)
    }

    fn sys_write(&self, fd: u64, buf: u64, count: u64) -> Result<i64> {
        tracing::info!("Write {} bytes to fd {} from {:#x}", count, fd, buf);
        Ok(count as i64)
    }
}

impl Default for SyscallHandler {
    fn default() -> Self {
        Self::new()
    }
}
