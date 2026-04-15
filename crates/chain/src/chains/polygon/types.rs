//! Polygon-specific types

use serde::{Deserialize, Serialize};

/// Polygon Block statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolygonBlockStats {
    pub last_block_time_ms: f64,
    pub avg_block_time_ms: f64,
    pub min_block_time_ms: f64,
    pub max_block_time_ms: f64,
}

impl PolygonBlockStats {
    pub fn update(&mut self, block_time_ms: f64) {
        self.last_block_time_ms = block_time_ms;

        if self.avg_block_time_ms == 0.0 {
            self.avg_block_time_ms = block_time_ms;
            self.min_block_time_ms = block_time_ms;
            self.max_block_time_ms = block_time_ms;
        } else {
            self.avg_block_time_ms = self.avg_block_time_ms * 0.9 + block_time_ms * 0.1;
            self.min_block_time_ms = self.min_block_time_ms.min(block_time_ms);
            self.max_block_time_ms = self.max_block_time_ms.max(block_time_ms);
        }
    }
}
