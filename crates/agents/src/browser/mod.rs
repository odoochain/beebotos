//! Browser Module
//!
//! Browser automation and control.

pub mod controller;
pub mod automation;
pub mod cdp;

pub use controller::BrowserController;

use crate::error::Result;

/// Browser configuration
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub headless: bool,
    pub viewport: Viewport,
}

#[derive(Debug, Clone)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            viewport: Viewport { width: 1920, height: 1080 },
        }
    }
}
