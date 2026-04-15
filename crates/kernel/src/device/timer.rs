//! Timer Device
//!
//! Hardware timer for scheduling and timekeeping.

use crate::error::Result;

/// Hardware timer
pub struct Timer {
    /// Timer frequency in Hz
    frequency: u64,
    /// Tick counter
    ticks: u64,
}

impl Timer {
    /// Create new timer with specified frequency
    pub fn new(frequency: u64) -> Self {
        Self {
            frequency,
            ticks: 0,
        }
    }

    /// Initialize timer hardware
    pub fn init(&mut self) -> Result<()> {
        tracing::info!("Initializing timer at {} Hz", self.frequency);
        Ok(())
    }

    /// Increment tick counter
    pub fn tick(&mut self) {
        self.ticks += 1;
    }

    /// Busy-wait sleep for milliseconds
    pub fn sleep_ms(&self, ms: u64) {
        // Busy wait for simplicity
        let target = self.ticks + (ms * self.frequency / 1000);
        while self.ticks < target {
            core::hint::spin_loop();
        }
    }
}
