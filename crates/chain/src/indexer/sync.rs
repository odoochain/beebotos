//! Indexer Sync

use crate::compat::B256;

/// Sync status
#[derive(Debug, Clone)]
pub enum SyncStatus {
    Syncing { current: u64, target: u64 },
    Synced { head: u64 },
    Error(String),
}

/// Sync manager
pub struct SyncManager {
    target_block: u64,
}

impl SyncManager {
    pub fn new() -> Self {
        Self { target_block: 0 }
    }
    
    pub fn set_target(&mut self, block: u64) {
        self.target_block = block;
    }
    
    pub fn status(&self, current: u64) -> SyncStatus {
        if current >= self.target_block {
            SyncStatus::Synced { head: current }
        } else {
            SyncStatus::Syncing {
                current,
                target: self.target_block,
            }
        }
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}
