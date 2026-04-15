//! Storage Device
//!
//! Block storage device interface.

use crate::error::Result;

/// Block storage device
pub struct StorageDevice {
    /// Block size in bytes
    block_size: usize,
    /// Number of blocks
    num_blocks: u64,
}

impl StorageDevice {
    /// Create new storage device
    pub fn new(block_size: usize, num_blocks: u64) -> Self {
        Self {
            block_size,
            num_blocks,
        }
    }

    /// Initialize storage device
    pub fn init(&mut self) -> Result<()> {
        tracing::info!(
            "Initializing storage: {} blocks of {} bytes",
            self.num_blocks,
            self.block_size
        );
        Ok(())
    }

    /// Read block from storage
    pub fn read(&self, block: u64, _buf: &mut [u8]) -> Result<()> {
        tracing::info!("Reading block {}", block);
        Ok(())
    }

    /// Write block to storage
    pub fn write(&self, block: u64, _buf: &[u8]) -> Result<()> {
        tracing::info!("Writing block {}", block);
        Ok(())
    }

    /// Get total storage size in bytes
    pub fn size(&self) -> u64 {
        self.block_size as u64 * self.num_blocks
    }
}
