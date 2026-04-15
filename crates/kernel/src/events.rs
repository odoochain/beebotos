//! Kernel event types for SystemEventBus integration

use beebotos_core::event_bus::SystemEvent;
use chrono::{DateTime, Utc};

/// Kernel task events
#[derive(Debug, Clone)]
pub enum KernelTaskEvent {
    /// Task spawned
    Spawned {
        /// Task ID
        task_id: u64,
        /// Task name
        name: String,
        /// Task priority level
        priority: String,
    },
    /// Task started execution
    Started {
        /// Task ID
        task_id: u64,
        /// Worker ID that started the task
        worker_id: usize,
    },
    /// Task completed successfully
    Completed {
        /// Task ID
        task_id: u64,
        /// Execution duration in milliseconds
        duration_ms: u64,
    },
    /// Task failed
    Failed {
        /// Task ID
        task_id: u64,
        /// Error message
        error: String,
    },
    /// Task cancelled
    Cancelled {
        /// Task ID
        task_id: u64,
        /// Cancellation reason
        reason: String,
    },
    /// Task timed out
    TimedOut {
        /// Task ID
        task_id: u64,
        /// Timeout duration in seconds
        timeout_secs: u64,
    },
}

impl SystemEvent for KernelTaskEvent {
    fn event_type(&self) -> &'static str {
        "kernel.task"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Kernel resource events
#[derive(Debug, Clone)]
pub enum KernelResourceEvent {
    /// Memory pressure detected
    MemoryPressure {
        /// Used memory in bytes
        used_bytes: u64,
        /// Total memory in bytes
        total_bytes: u64,
        /// Current pressure level
        pressure_level: PressureLevel,
    },
    /// CPU throttling
    CpuThrottling {
        /// Throttling percentage
        throttled_percent: f64,
    },
    /// Sandbox violation
    SandboxViolation {
        /// Task ID that caused the violation
        task_id: u64,
        /// Type of violation
        violation_type: String,
    },
}

/// Memory pressure level
#[derive(Debug, Clone, Copy)]
pub enum PressureLevel {
    /// Low memory pressure
    Low,
    /// Medium memory pressure
    Medium,
    /// High memory pressure
    High,
    /// Critical memory pressure
    Critical,
}

impl SystemEvent for KernelResourceEvent {
    fn event_type(&self) -> &'static str {
        "kernel.resource"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
