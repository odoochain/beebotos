//! Debug utilities for kernel

use std::fmt;
use std::time::Instant;

/// Debug tracer
pub struct Tracer {
    /// Tracer name
    name: String,
    /// Start timestamp
    start: Instant,
}

impl Tracer {
    /// Create new tracer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }

    /// Print trace
    pub fn trace(&self, msg: impl fmt::Display) {
        let elapsed = self.start.elapsed();
        log::debug!("[{}] {} ({}µs)", self.name, msg, elapsed.as_micros());
    }
}

impl Drop for Tracer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        log::debug!("[{}] completed in {}µs", self.name, elapsed.as_micros());
    }
}

/// Assert condition in debug builds
#[macro_export]
macro_rules! debug_assert {
    ($cond:expr, $msg:expr) => {
        if cfg!(debug_assertions) && !$cond {
            panic!("Debug assertion failed: {}", $msg);
        }
    };
}

/// Debug print macro
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
}

/// Memory debugger
pub struct MemoryDebugger;

impl MemoryDebugger {
    /// Print memory stats
    pub fn print_stats() {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("Vm") {
                        log::debug!("{}", line);
                    }
                }
            }
        }
    }

    /// Get current RSS
    pub fn current_rss() -> Option<usize> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(status) = fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        let parts: Vec<_> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return parts[1].parse().ok();
                        }
                    }
                }
            }
        }
        None
    }
}

/// State dump for debugging
pub trait StateDump {
    fn dump_state(&self) -> serde_json::Value;
}

/// Debug breakpoint (no-op in release)
#[inline]
pub fn breakpoint() {
    #[cfg(debug_assertions)]
    {
        log::debug!("Breakpoint hit");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
