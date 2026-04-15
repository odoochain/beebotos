//! Scheduler tests

use beebotos_kernel::capabilities::CapabilitySet;
use beebotos_kernel::scheduler::queue::{SchedulingAlgorithm, TaskQueue};
use beebotos_kernel::scheduler::{Priority, Scheduler, SchedulerConfig, Task};

#[tokio::test]
async fn test_scheduler_creation() {
    let config = SchedulerConfig::default();
    let scheduler = Scheduler::new(config);

    assert_eq!(scheduler.queue_length().await, 0);
    assert_eq!(scheduler.running_count().await, 0);
}

#[tokio::test]
async fn test_task_spawn() {
    let config = SchedulerConfig::default();
    let scheduler = Scheduler::new(config);

    // Start the scheduler first
    scheduler.start().await.unwrap();

    let task_id = scheduler
        .spawn(
            "test_task",
            Priority::Normal,
            CapabilitySet::empty(),
            async { Ok(()) },
        )
        .await
        .unwrap();

    assert!(task_id > beebotos_kernel::TaskId(0));

    // Clean up
    scheduler.stop().await;
}

#[test]
fn test_task_creation() {
    let task = Task::new(beebotos_kernel::TaskId(42), "my_task").with_priority(Priority::High);

    assert_eq!(task.id, beebotos_kernel::TaskId(42));
}

#[test]
fn test_priority_ordering() {
    // Lower value = higher priority (RealTime=0, High=1, Normal=2, Low=3, Idle=4)
    assert!(Priority::High < Priority::Normal);
    assert!(Priority::Normal < Priority::Low);
    assert!(Priority::RealTime < Priority::High);
    assert!(Priority::Low < Priority::Idle);
}

#[tokio::test]
async fn test_task_queue_operations() {
    let queue = TaskQueue::new(SchedulingAlgorithm::Priority);

    let task1 = Task::new(beebotos_kernel::TaskId(1), "task1").with_priority(Priority::Normal);
    let task2 = Task::new(beebotos_kernel::TaskId(2), "task2").with_priority(Priority::High);

    queue.enqueue(task1).await;
    queue.enqueue(task2).await;

    assert_eq!(queue.len().await, 2);

    // Should dequeue higher priority task first
    let dequeued = queue.dequeue().await;
    assert!(dequeued.is_some());
}

#[tokio::test]
async fn test_scheduler_stats() {
    let config = SchedulerConfig::default();
    let scheduler = Scheduler::new(config);

    let stats = scheduler.stats().await;
    assert_eq!(stats.tasks_submitted, 0);
    assert_eq!(stats.tasks_completed, 0);
}
