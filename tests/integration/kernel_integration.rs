//! Kernel integration tests

use beebot_kernel::{Kernel, KernelBuilder};
use beebot_kernel::scheduler::{Priority, SchedulerConfig};
use beebot_kernel::capabilities::CapabilitySet;

#[tokio::test]
async fn test_kernel_lifecycle() {
    let kernel = KernelBuilder::new()
        .build()
        .expect("Failed to build kernel");
    
    // Start kernel
    kernel.start().await.expect("Failed to start kernel");
    assert!(kernel.is_running());
    
    // Stop kernel
    kernel.stop().await;
    assert!(!kernel.is_running());
}

#[tokio::test]
async fn test_task_spawning() {
    let kernel = KernelBuilder::new()
        .build()
        .expect("Failed to build kernel");
    
    kernel.start().await.unwrap();
    
    let task_id = kernel.spawn_task(
        "test_task",
        Priority::NORMAL,
        CapabilitySet::standard(),
        async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok(())
        }
    ).await.expect("Failed to spawn task");
    
    assert!(task_id.0 > 0);
    
    kernel.stop().await;
}

#[tokio::test]
async fn test_multiple_tasks() {
    let kernel = KernelBuilder::new()
        .build()
        .expect("Failed to build kernel");
    
    kernel.start().await.unwrap();
    
    let mut task_ids = vec![];
    
    for i in 0..10 {
        let id = kernel.spawn_task(
            format!("task_{}", i),
            Priority::NORMAL,
            CapabilitySet::standard(),
            async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                Ok(())
            }
        ).await.expect("Failed to spawn task");
        
        task_ids.push(id);
    }
    
    assert_eq!(task_ids.len(), 10);
    
    // Let tasks complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let stats = kernel.scheduler_stats().await;
    assert!(stats.tasks_created >= 10);
    
    kernel.stop().await;
}

#[tokio::test]
async fn test_priority_scheduling() {
    let kernel = KernelBuilder::new()
        .with_scheduler(SchedulerConfig {
            time_quantum_ms: 10,
            ..Default::default()
        })
        .build()
        .expect("Failed to build kernel");
    
    kernel.start().await.unwrap();
    
    // Spawn low priority task
    let low_id = kernel.spawn_task(
        "low_priority",
        Priority::LOW,
        CapabilitySet::standard(),
        async {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            Ok(())
        }
    ).await.unwrap();
    
    // Spawn high priority task
    let high_id = kernel.spawn_task(
        "high_priority",
        Priority::HIGH,
        CapabilitySet::standard(),
        async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok(())
        }
    ).await.unwrap();
    
    assert!(high_id.0 > 0);
    assert!(low_id.0 > 0);
    
    kernel.stop().await;
}
