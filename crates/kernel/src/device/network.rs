//! Network Device
//!
//! Network interface for packet I/O.

use crate::error::Result;

/// Network device
pub struct NetworkDevice {
    /// MAC address
    mac: [u8; 6],
    /// IP address - reserved for network stack implementation
    #[allow(dead_code)]
    _ip: [u8; 4],
}

impl NetworkDevice {
    /// Create new network device with MAC address
    pub fn new(mac: [u8; 6]) -> Self {
        Self {
            mac,
            _ip: [0, 0, 0, 0],
        }
    }

    /// Initialize network device
    pub fn init(&mut self) -> Result<()> {
        tracing::info!("Initializing network device {:02x?}", self.mac);
        Ok(())
    }

    /// Send data packet
    pub fn send(&self, data: &[u8]) -> Result<usize> {
        tracing::info!("Sending {} bytes", data.len());
        Ok(data.len())
    }

    /// Receive data into buffer
    pub fn receive(&self, buf: &mut [u8]) -> Result<usize> {
        tracing::info!("Receiving up to {} bytes", buf.len());
        Ok(0)
    }
}
