//! Paging
//!
//! Page table management and virtual memory mapping.

use crate::error::Result;

/// Page size (4KB)
pub const PAGE_SIZE: usize = 4096;

/// Page table entry
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry {
    /// Physical address
    pub physical_addr: u64,
    /// Page flags
    pub flags: PageFlags,
}

/// Page flags
#[derive(Debug, Clone, Copy)]
pub struct PageFlags {
    /// Page is present in memory
    pub present: bool,
    /// Page is writable
    pub writable: bool,
    /// Page accessible from user mode
    pub user_accessible: bool,
}

/// Page table
pub struct PageTable {
    entries: [Option<PageTableEntry>; 512],
}

impl PageTable {
    /// Create new page table
    pub fn new() -> Self {
        Self {
            entries: [None; 512],
        }
    }

    /// Map virtual address to physical address
    pub fn map(&mut self, virtual_addr: u64, physical_addr: u64, flags: PageFlags) -> Result<()> {
        let index = (virtual_addr / PAGE_SIZE as u64) as usize % 512;
        self.entries[index] = Some(PageTableEntry {
            physical_addr,
            flags,
        });
        Ok(())
    }

    /// Unmap virtual address
    pub fn unmap(&mut self, virtual_addr: u64) -> Result<()> {
        let index = (virtual_addr / PAGE_SIZE as u64) as usize % 512;
        self.entries[index] = None;
        Ok(())
    }

    /// Lookup page table entry
    pub fn lookup(&self, virtual_addr: u64) -> Option<&PageTableEntry> {
        let index = (virtual_addr / PAGE_SIZE as u64) as usize % 512;
        self.entries[index].as_ref()
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}
