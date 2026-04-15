//! Simple scheduler example

use beebotos_kernel::capabilities::CapabilitySet;
use beebotos_kernel::scheduler::{Priority, SchedulerConfig};
use beebotos_kernel::KernelBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Simple Scheduler Example\n");

    // Create kernel with custom config
    let kernel = KernelBuilder::new()
        .with_scheduler(SchedulerConfig {
            time_slice_ms: 100,
            ..Default::default()
        })
        .build()?;

    // Start kernel
    kernel.start().await?;
    println!("✅ Kernel started\n");

    // Spawn some tasks
    for i in 0..5 {
        let task_id = kernel
            .spawn_task(
                format!("task_{}", i),
                Priority::Normal,
                CapabilitySet::standard(),
                async move {
                    println!("  Task {} is running", i);
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    println!("  Task {} completed", i);
                    Ok(())
                },
            )
            .await?;

        println!("✅ Spawned task {} with ID {:?}", i, task_id);
    }

    // Wait for tasks to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Show stats
    let stats = kernel.scheduler_stats().await;
    println!("\n📊 Scheduler Stats:");
    println!("  Tasks submitted: {}", stats.tasks_submitted);
    println!("  Tasks completed: {}", stats.tasks_completed);

    // Stop kernel
    kernel.stop().await;
    println!("\n✅ Kernel stopped");

    Ok(())
}
