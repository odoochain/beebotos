//! Kernel Demo Example
//!
//! Demonstrates basic kernel usage: build, start, spawn task, shutdown.

use beebotos_kernel::capabilities::CapabilitySet;
use beebotos_kernel::scheduler::{Priority, SchedulerConfig};
use beebotos_kernel::KernelBuilder;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("=== BeeBotOS Kernel Demo ===");

    // 构建 Kernel
    let kernel = KernelBuilder::new()
        .with_max_agents(100)
        .with_scheduler(SchedulerConfig::development())
        .build()?;

    // 启动
    kernel.start().await?;
    info!("Kernel started successfully!");

    // 创建示例任务
    let task_id = kernel
        .spawn_task(
            "hello-task",
            Priority::Normal,
            CapabilitySet::empty(),
            async {
                info!("Hello from kernel task!");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                info!("Task completed!");
                Ok(())
            },
        )
        .await?;

    info!("Spawned task with ID: {:?}", task_id);

    // 等待几秒让任务执行
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 获取统计信息
    let stats = kernel.stats().await;
    info!("Kernel stats: {:?}", stats);

    // 优雅停止
    kernel.stop().await;
    info!("Kernel stopped. Demo complete!");

    Ok(())
}
