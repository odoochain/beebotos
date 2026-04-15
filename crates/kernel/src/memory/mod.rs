//! Memory Management
//!
//! Provides kernel memory management including:
//! - Global memory allocator with statistics
//! - Heap management
//! - Slab allocator for fixed-size objects
//! - Virtual memory management
//! - Memory pools for efficient allocation

pub mod allocator;
pub mod heap;
pub mod isolation;
pub mod mmap;
pub mod paging;
pub mod safety;
pub mod slab;
pub mod vm;

pub use allocator::{
    format_stats, get_memory_limit, init, set_memory_limit, KernelAllocator, MemoryManager,
    MemoryPool, MemoryStats, MemoryTracker, PoolStats, MEMORY_STATS, MEMORY_TRACKER,
};
pub use heap::KernelHeap;
pub use isolation::{
    global as global_isolation, IsolationStats, MemoryIsolation, MemoryPermissions,
    ProcessMemorySpace, UserMemoryRegion,
};
pub use paging::{PageFlags, PageTable, PageTableEntry, PAGE_SIZE};
pub use safety::{
    global_tracker, init_global_tracker, AccessCheck, AllocationCheck, CanaryGuard, MemoryGuard,
    MemorySafetyTracker, SafetyStats,
};
pub use slab::SlabAllocator;
pub use vm::{RegionFlags, VirtualMemory, VirtualRegion};

use crate::error::Result;

/// Global memory manager instance
static MEMORY_MANAGER: std::sync::OnceLock<MemoryManager> = std::sync::OnceLock::new();

/// Get the global memory manager
pub fn manager() -> &'static MemoryManager {
    MEMORY_MANAGER.get_or_init(MemoryManager::new)
}

/// Initialize memory management subsystem
///
/// This should be called once during kernel boot.
pub fn init_subsystem() -> Result<()> {
    allocator::init()?;
    heap::init()?;
    slab::init()?;
    vm::init()?;
    isolation::init();

    tracing::info!("Memory management subsystem initialized");
    Ok(())
}

/// Memory statistics snapshot
#[derive(Debug, Clone, Copy)]
pub struct MemorySnapshot {
    /// Currently used memory in bytes
    pub current_used_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_used_bytes: usize,
    /// Total bytes allocated
    pub total_allocated_bytes: usize,
    /// Total bytes freed
    pub total_freed_bytes: usize,
    /// Number of allocation operations
    pub allocation_count: usize,
    /// Number of deallocation operations
    pub deallocation_count: usize,
}

impl MemorySnapshot {
    /// Capture current memory statistics
    ///
    /// Uses SeqCst ordering to ensure consistent snapshot across all counters.
    /// This is slightly slower than Relaxed but ensures we get a coherent view
    /// of memory state, which is important for OOM decisions.
    pub fn capture() -> Self {
        // Use SeqCst for all atomic operations to ensure a consistent snapshot
        Self {
            current_used_bytes: MEMORY_STATS.current_used(),
            peak_used_bytes: MEMORY_STATS.peak_used(),
            total_allocated_bytes: MEMORY_STATS.total_allocated(),
            total_freed_bytes: MEMORY_STATS.total_freed(),
            allocation_count: MEMORY_STATS.allocation_count(),
            deallocation_count: MEMORY_STATS.deallocation_count(),
        }
    }

    /// Calculate fragmentation ratio
    pub fn fragmentation_ratio(&self) -> f64 {
        if self.total_allocated_bytes == 0 {
            return 0.0;
        }
        let actual = self.total_allocated_bytes - self.total_freed_bytes;
        let tracked = self.current_used_bytes;
        if actual == 0 {
            return 0.0;
        }
        (tracked as f64 - actual as f64) / actual as f64
    }

    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Memory: {} MB current / {} MB peak | Allocs: {}/{} | Fragmentation: {:.2}%",
            self.current_used_bytes / (1024 * 1024),
            self.peak_used_bytes / (1024 * 1024),
            self.allocation_count,
            self.deallocation_count,
            self.fragmentation_ratio() * 100.0
        )
    }
}

/// Memory pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    /// Normal memory usage
    Normal,
    /// Memory usage is elevated
    Elevated,
    /// Memory is under pressure, should free resources
    Critical,
    /// Out of memory
    OutOfMemory,
}

/// Check current memory pressure level
pub fn check_pressure() -> MemoryPressure {
    let current = MEMORY_STATS.current_used();
    let limit = allocator::get_memory_limit();

    if limit == 0 {
        // No limit set, use system memory
        let sys = sysinfo::System::new_all();
        let total = sys.total_memory() * 1024; // Convert KB to bytes
        let used_percent = (current as f64 / total as f64) * 100.0;

        match used_percent {
            p if p > 95.0 => MemoryPressure::OutOfMemory,
            p if p > 85.0 => MemoryPressure::Critical,
            p if p > 70.0 => MemoryPressure::Elevated,
            _ => MemoryPressure::Normal,
        }
    } else {
        // Use configured limit
        let used_percent = (current as f64 / limit as f64) * 100.0;

        match used_percent {
            p if p > 100.0 => MemoryPressure::OutOfMemory,
            p if p > 90.0 => MemoryPressure::Critical,
            p if p > 75.0 => MemoryPressure::Elevated,
            _ => MemoryPressure::Normal,
        }
    }
}

/// Memory region types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    /// Available RAM
    Usable,
    /// Reserved (unusable)
    Reserved,
    /// Kernel code/data
    Kernel,
    /// Device memory
    Device,
    /// ACPI reclaimable
    AcpiReclaimable,
}

/// Memory region descriptor
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Start address
    pub start: u64,
    /// Size in bytes
    pub size: u64,
    /// Region type
    pub region_type: MemoryRegionType,
}

/// Memory configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Maximum memory size in bytes
    pub max_memory_size: usize,
    /// Initial heap size in bytes
    pub initial_heap_size: usize,
    /// Enable huge pages
    pub huge_pages: bool,
    /// Memory limit for WASM
    pub wasm_memory_limit: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_memory_size: 1024 * 1024 * 1024, // 1GB
            initial_heap_size: 64 * 1024 * 1024, // 64MB
            huge_pages: false,
            wasm_memory_limit: 128 * 1024 * 1024, // 128MB
        }
    }
}

/// Print memory map (for debugging)
pub fn print_memory_map(regions: &[MemoryRegion]) {
    tracing::info!("Memory Map:");
    for (i, region) in regions.iter().enumerate() {
        let size_mb = region.size / (1024 * 1024);
        tracing::info!(
            "  [{}] {:016x} - {:016x} ({} MB): {:?}",
            i,
            region.start,
            region.start + region.size,
            size_mb,
            region.region_type
        );
    }
}
