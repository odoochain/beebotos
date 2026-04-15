//! Queue Management Module
//!
//! Multi-queue concurrency system:
//! - Main queue: Sequential execution
//! - Cron queue: Scheduled tasks (using `crate::scheduling::cron`)
//! - Subagent queue: Parallel execution (max 5)
//! - Nested queue: Recursion prevention
//! - 🟢 P0 FIX: DAG Scheduler for explicit task dependencies

pub mod dead_letter;
pub mod main_queue;
pub mod manager;
pub mod nested;
pub mod subagent;

// RELIABILITY FIX: Worker task functions for panic recovery
pub mod worker_tasks;

// 🟢 P0 FIX: DAG Task Scheduler
pub mod dag_scheduler;

pub use manager::{
    Priority, QueueError, QueueManager, QueueStats, QueueTask, TaskProcessor, TaskResult, TaskType,
};

// ARCHITECTURE FIX: Re-export dead letter queue types
pub use dead_letter::{
    DeadLetterEntry, DeadLetterQueue, DLQConfig, DLQStats, DLQTaskProcessor,
};

// 🟢 P0 FIX: Re-export DAG scheduler types
pub use dag_scheduler::{
    DagScheduler, DagWorkflow, DagWorkflowBuilder, DagTask,
    TaskExecutionStatus, WorkflowStatus, WorkflowInstance,
    TaskExecutor, TaskExecutionRequest,
    SchedulerConfig, SchedulerEvent, SchedulerMetrics,
    TaskPriority, ResourceRequirements,
    WorkflowConfig, TaskRetryPolicy,
    SchedulerError,
};

// 🟢 P1 FIX: Re-export cron types from scheduling module
pub use crate::scheduling::cron::{
    CronScheduler, CronJob, JobId, ScheduleType, ContextMode,
    CronPersistence, CronError,
};

// Example module (optional, for documentation)
#[cfg(feature = "examples")]
pub mod example;
