//! Signals
//!
//! Signal handling and delivery for inter-process communication.

// use crate::error::KernelResult; // Currently unused

/// POSIX signal numbers
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Interrupt signal (Ctrl+C)
    SigInt = 2,
    /// Quit signal
    SigQuit = 3,
    /// Abort signal
    SigAbort = 6,
    /// Kill signal
    SigKill = 9,
    /// Segmentation fault
    SigSegv = 11,
    /// Termination signal
    SigTerm = 15,
}

/// Signal handler function type
pub type SignalHandler = fn(Signal);

/// Signal delivery mechanism
pub struct SignalDelivery {
    /// Pending signals to deliver
    pending: Vec<Signal>,
    /// Registered signal handlers
    handlers: [Option<SignalHandler>; 32],
}

impl SignalDelivery {
    /// Create new signal delivery
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            handlers: [None; 32],
        }
    }

    /// Send signal to be delivered
    pub fn send(&mut self, signal: Signal) {
        self.pending.push(signal);
    }

    /// Set handler for specific signal
    pub fn set_handler(&mut self, signal: Signal, handler: SignalHandler) {
        self.handlers[signal as usize] = Some(handler);
    }

    /// Deliver all pending signals
    pub fn deliver_pending(&mut self) {
        for signal in self.pending.drain(..) {
            if let Some(handler) = self.handlers[signal as usize] {
                handler(signal);
            }
        }
    }
}

impl Default for SignalDelivery {
    fn default() -> Self {
        Self::new()
    }
}
