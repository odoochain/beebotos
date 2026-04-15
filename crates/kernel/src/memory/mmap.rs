//! Memory Mapping
//!
//! Virtual memory mapping operations.

use crate::error::Result;

/// Memory map handle
#[derive(Debug)]
pub struct MemoryMap {
    /// Mapped address
    pub addr: u64,
    /// Mapped size
    pub size: usize,
    /// Mapping flags
    pub flags: MapFlags,
}

/// Memory mapping flags
#[derive(Debug, Clone, Copy)]
pub struct MapFlags {
    /// Readable mapping
    pub read: bool,
    /// Writable mapping
    pub write: bool,
    /// Executable mapping
    pub exec: bool,
    /// Shared mapping
    pub shared: bool,
}

/// Memory mapper
pub struct MemoryMapper;

impl MemoryMapper {
    /// Create new memory mapper
    pub fn new() -> Self {
        Self
    }

    /// Map memory region
    pub fn mmap(&self, addr: Option<u64>, size: usize, flags: MapFlags) -> Result<MemoryMap> {
        Ok(MemoryMap {
            addr: addr.unwrap_or(0),
            size,
            flags,
        })
    }

    /// Unmap memory region
    pub fn munmap(&self, map: MemoryMap) -> Result<()> {
        tracing::info!("Unmapping {} bytes at {:#x}", map.size, map.addr);
        Ok(())
    }
}

impl Default for MemoryMapper {
    fn default() -> Self {
        Self::new()
    }
}
