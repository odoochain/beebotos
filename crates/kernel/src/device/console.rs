//! Console Device
//!
//! Console input/output interface.

use crate::error::Result;

/// Console output
pub struct Console;

impl Console {
    /// Create new console
    pub fn new() -> Self {
        Self
    }

    /// Initialize console
    pub fn init(&mut self) -> Result<()> {
        tracing::info!("Initializing console");
        Ok(())
    }

    /// Write string to console
    pub fn write(&self, s: &str) {
        // Output to serial port or VGA buffer
        tracing::info!("CONSOLE: {}", s);
    }

    /// Read byte from console input
    pub fn read(&self) -> Option<u8> {
        // Read from keyboard
        None
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}
