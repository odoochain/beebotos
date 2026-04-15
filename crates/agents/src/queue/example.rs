//! Queue Manager Usage Example
//!
//! Example of how to use the production-ready QueueManager.

use std::sync::Arc;

use ::tracing::info;
use async_trait::async_trait;

use crate::queue::{Priority, QueueManager, QueueTask, TaskProcessor, TaskResult, TaskType};
use crate::session::{SessionKey, SessionType};

/// Example task processor
pub struct ExampleProcessor;

#[async_trait]
impl TaskProcessor for ExampleProcessor {
    async fn process(&self, task: QueueTask) -> TaskResult {
        info!("Processing task: {} - {:?}", task.id, task.task_type);

        // Simulate work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        TaskResult {
            task_id: task.id.clone(),
            success: true,
            output: format!("Processed: {:?}", task.task_type),
        }
    }
}

/// Example usage
pub async fn example_usage() {
    // Create queue manager
    let manager = Arc::new(QueueManager::new());

    // Create processor
    let processor = Arc::new(ExampleProcessor);

    // Spawn workers
    manager.spawn_workers(processor).await;

    // Create a session
    let session = SessionKey::new("agent-1", SessionType::Session);

    // Submit tasks to different queues

    // Main queue - sequential execution
    manager
        .submit_main(QueueTask {
            id: "task-1".to_string(),
            session_key: session.clone(),
            task_type: TaskType::ExecuteCommand("ls -la".to_string()),
            priority: Priority::Normal,
        })
        .unwrap();

    // Subagent queue - parallel execution (max 5 concurrent)
    manager
        .submit_subagent(QueueTask {
            id: "task-2".to_string(),
            session_key: session.clone(),
            task_type: TaskType::SpawnSubagent("sub-task".to_string()),
            priority: Priority::High,
        })
        .await
        .unwrap();

    // Cron queue - scheduled tasks
    manager
        .submit_cron(QueueTask {
            id: "task-3".to_string(),
            session_key: session.clone(),
            task_type: TaskType::CronJob("daily-backup".to_string()),
            priority: Priority::Low,
        })
        .unwrap();

    // Nested queue - with recursion prevention
    manager
        .submit_nested(QueueTask {
            id: "task-4".to_string(),
            session_key: session.clone(),
            task_type: TaskType::ExecuteCommand("nested-call".to_string()),
            priority: Priority::Normal,
        })
        .await
        .unwrap();

    // Wait for tasks to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Get statistics
    let stats = manager.stats().await;
    info!("Queue stats: {:?}", stats);

    // Graceful shutdown
    manager.shutdown().await;
}
