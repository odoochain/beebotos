//! Device Management
//!
//! Hardware device drivers and management.

pub mod console;
pub mod network;
pub mod pci;
pub mod storage;
pub mod timer;

pub use network::NetworkDevice;
pub use timer::Timer;

use crate::error::Result;

/// Device trait for hardware abstraction
pub trait Device {
    /// Initialize device hardware
    fn init(&mut self) -> Result<()>;
    /// Shutdown device
    fn shutdown(&mut self) -> Result<()>;
}
