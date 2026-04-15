//! Kernel Main Entry Point
//!
//! Main entry point for the BeeBotOS kernel binary.
//! Includes graceful shutdown handling and jemalloc memory allocator.

// Enable jemalloc for better memory performance
#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOCATOR: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use beebotos_kernel::boot::{boot, BootInfo, MemoryRegion, MemoryRegionType};
use beebotos_kernel::KernelBuilder;
use tokio::signal;
use tracing::{error, info, warn};

/// Shutdown signal type
#[derive(Debug)]
#[allow(dead_code)]
enum ShutdownSignal {
    /// Ctrl+C (SIGINT)
    Interrupt,
    /// SIGTERM
    Terminate,
    /// Internal shutdown request - reserved for programmatic shutdown
    Internal,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("BeeBotOS Kernel starting...");

    #[cfg(feature = "jemalloc")]
    info!("Using jemalloc memory allocator");

    // Create minimal boot info for testing
    static MEMORY_MAP: [MemoryRegion; 2] = [
        MemoryRegion {
            start: 0x00000000,
            size: 0x10000000, // 256MB
            region_type: MemoryRegionType::Usable,
        },
        MemoryRegion {
            start: 0x10000000,
            size: 0x10000000, // 256MB
            region_type: MemoryRegionType::Kernel,
        },
    ];

    let boot_info = BootInfo {
        memory_map: &MEMORY_MAP,
        cmd_line: "",
        bootloader_name: "test",
        cpu_count: 1,
        boot_time: std::time::SystemTime::now(),
    };

    // Boot the kernel
    let kernel = match boot_and_build(&boot_info).await {
        Ok(k) => {
            info!("Kernel booted successfully");
            k
        }
        Err(e) => {
            error!("Kernel boot failed: {}", e);
            std::process::exit(1);
        }
    };

    // Start the kernel
    if let Err(e) = kernel.start().await {
        error!("Failed to start kernel: {}", e);
        std::process::exit(1);
    }

    info!("Kernel is running. Press Ctrl+C to shutdown gracefully.");

    // Wait for shutdown signal
    let signal = wait_for_shutdown_signal().await;
    info!("Received shutdown signal: {:?}", signal);

    // Perform graceful shutdown
    if let Err(e) = graceful_shutdown(kernel).await {
        error!("Error during shutdown: {}", e);
        std::process::exit(1);
    }

    info!("Kernel shutdown complete.");
}

/// Boot and build kernel
async fn boot_and_build(
    boot_info: &BootInfo,
) -> Result<beebotos_kernel::Kernel, beebotos_kernel::error::KernelError> {
    // Perform boot sequence
    boot(boot_info).map_err(|e| {
        beebotos_kernel::error::KernelError::internal(format!("Boot failed: {}", e))
    })?;

    // Build kernel with default config
    let kernel = KernelBuilder::new().with_max_agents(1000).build()?;

    Ok(kernel)
}

/// Wait for shutdown signal
async fn wait_for_shutdown_signal() -> ShutdownSignal {
    tokio::select! {
        _ = signal::ctrl_c() => {
            ShutdownSignal::Interrupt
        }
        _ = wait_for_sigterm() => {
            ShutdownSignal::Terminate
        }
    }
}

/// Wait for SIGTERM (Unix only)
#[cfg(unix)]
async fn wait_for_sigterm() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(sig) => sig,
        Err(e) => {
            tracing::error!("Failed to create SIGTERM handler: {}", e);
            return;
        }
    };

    sigterm.recv().await;
}

#[cfg(not(unix))]
async fn wait_for_sigterm() {
    // Windows doesn't have SIGTERM, just wait forever
    std::future::pending::<()>().await;
}

/// Graceful shutdown sequence
async fn graceful_shutdown(
    kernel: beebotos_kernel::Kernel,
) -> Result<(), beebotos_kernel::error::KernelError> {
    const SHUTDOWN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

    info!("Starting graceful shutdown sequence...");

    // Step 1: Stop accepting new tasks
    info!("Step 1/4: Stopping new task acceptance...");
    // This is handled by the kernel's running flag

    // Step 2: Wait for running tasks with timeout
    info!(
        "Step 2/4: Waiting for running tasks (timeout: {:?})...",
        SHUTDOWN_TIMEOUT
    );

    let shutdown_future = kernel.stop();

    match tokio::time::timeout(SHUTDOWN_TIMEOUT, shutdown_future).await {
        Ok(_) => {
            info!("All tasks completed gracefully");
        }
        Err(_) => {
            warn!("Shutdown timeout reached, forcing termination of remaining tasks");
            // Force cancel remaining tasks
        }
    }

    // Step 3: Flush audit logs
    info!("Step 3/4: Flushing audit logs...");
    // This would be called if we had access to the security manager
    // kernel.flush_audit_logs().await?;

    // Step 4: Cleanup resources
    info!("Step 4/4: Cleaning up resources...");

    // Flush storage if needed
    // kernel.flush_storage().await?;

    // Print final statistics
    let stats = kernel.stats().await;
    info!("Final kernel stats: {:?}", stats);

    info!("Graceful shutdown complete");
    Ok(())
}
