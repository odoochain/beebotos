//! Signal Handling

use std::collections::HashMap;

use crate::error::Result;

/// Signal types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signal {
    Interrupt,
    Terminate,
    Reload,
    Custom(u32),
}

/// Signal handler
pub type SignalHandler = Box<dyn Fn(Signal) -> Result<()> + Send + Sync>;

/// Signal manager
pub struct SignalManager {
    handlers: HashMap<Signal, Vec<SignalHandler>>,
}

impl SignalManager {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, signal: Signal, handler: SignalHandler) {
        self.handlers.entry(signal).or_default().push(handler);
    }

    pub fn emit(&self, signal: Signal) -> Result<()> {
        if let Some(handlers) = self.handlers.get(&signal) {
            for handler in handlers {
                handler(signal)?;
            }
        }
        Ok(())
    }
}

impl Default for SignalManager {
    fn default() -> Self {
        Self::new()
    }
}
