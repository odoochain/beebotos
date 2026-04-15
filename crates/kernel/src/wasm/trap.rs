//! WASM Trap Handling
//!
//! Handles WASM runtime traps and errors for wasmtime 34.0

use crate::error::KernelError;

/// WASM trap types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmTrap {
    /// Stack overflow
    StackOverflow,
    /// Memory out of bounds
    MemoryOutOfBounds,
    /// Heap misaligned
    HeapMisaligned,
    /// Indirect call type mismatch
    IndirectCallTypeMismatch,
    /// Table out of bounds
    TableOutOfBounds,
    /// Integer division by zero
    IntegerDivisionByZero,
    /// Integer overflow
    IntegerOverflow,
    /// Bad conversion to integer
    BadConversionToInteger,
    /// Unreachable code reached
    UnreachableCodeReached,
    /// Interrupt
    Interrupt,
    /// Out of fuel (resource exhaustion)
    OutOfFuel,
    /// User-defined trap
    User(u8),
    /// Host error
    HostError,
    /// Unknown trap
    Unknown,
}

impl WasmTrap {
    /// Get trap description
    pub fn description(&self) -> &'static str {
        match self {
            WasmTrap::StackOverflow => "stack overflow",
            WasmTrap::MemoryOutOfBounds => "memory out of bounds",
            WasmTrap::HeapMisaligned => "heap misaligned",
            WasmTrap::IndirectCallTypeMismatch => "indirect call type mismatch",
            WasmTrap::TableOutOfBounds => "table out of bounds",
            WasmTrap::IntegerDivisionByZero => "integer divide by zero",
            WasmTrap::IntegerOverflow => "integer overflow",
            WasmTrap::BadConversionToInteger => "invalid conversion to integer",
            WasmTrap::UnreachableCodeReached => "unreachable code executed",
            WasmTrap::Interrupt => "interrupted",
            WasmTrap::OutOfFuel => "out of fuel",
            WasmTrap::User(_) => "user-defined trap",
            WasmTrap::HostError => "host function error",
            WasmTrap::Unknown => "unknown trap",
        }
    }

    /// Check if trap is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self, WasmTrap::OutOfFuel | WasmTrap::Interrupt)
    }

    /// Check if trap indicates resource exhaustion
    pub fn is_resource_exhaustion(&self) -> bool {
        matches!(
            self,
            WasmTrap::StackOverflow | WasmTrap::MemoryOutOfBounds | WasmTrap::OutOfFuel
        )
    }
}

impl std::fmt::Display for WasmTrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::error::Error for WasmTrap {}

impl From<WasmTrap> for KernelError {
    fn from(trap: WasmTrap) -> Self {
        match trap {
            WasmTrap::OutOfFuel => KernelError::resource_exhausted("WASM out of fuel"),
            WasmTrap::StackOverflow => KernelError::resource_exhausted("WASM stack overflow"),
            WasmTrap::MemoryOutOfBounds => KernelError::Memory("WASM memory out of bounds".into()),
            _ => KernelError::internal(format!("WASM trap: {}", trap.description())),
        }
    }
}

/// Convert wasmtime error to our error type
///
/// wasmtime 34.0: TrapCode is exposed through the error downcasting.
/// We infer trap types from error messages for compatibility.
pub fn convert_wasmtime_error(e: wasmtime::Error) -> KernelError {
    let error_str = e.to_string();

    // Check for specific trap types in error message
    if error_str.contains("out of fuel") {
        return WasmTrap::OutOfFuel.into();
    }

    if error_str.contains("memory") && error_str.contains("out of bounds") {
        return WasmTrap::MemoryOutOfBounds.into();
    }

    if error_str.contains("stack") && error_str.contains("overflow") {
        return WasmTrap::StackOverflow.into();
    }

    if error_str.contains("unreachable") {
        return WasmTrap::UnreachableCodeReached.into();
    }

    if error_str.contains("interrupt") {
        return WasmTrap::Interrupt.into();
    }

    // Generic WASM error
    KernelError::internal(format!("WASM error: {}", e))
}

/// Trap handler configuration
#[derive(Debug, Clone)]
pub struct TrapHandler {
    /// Maximum trap count before giving up
    pub max_trap_count: u32,
    /// Enable automatic retry for recoverable traps
    pub retry_recoverable: bool,
    /// Number of retry attempts
    pub max_retries: u32,
}

impl Default for TrapHandler {
    fn default() -> Self {
        Self {
            max_trap_count: 100,
            retry_recoverable: true,
            max_retries: 3,
        }
    }
}

impl TrapHandler {
    /// Create new trap handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle a trap
    pub fn handle(&self, trap: WasmTrap, context: &str) -> TrapAction {
        tracing::error!("WASM trap in {}: {}", context, trap);

        if trap.is_recoverable() && self.retry_recoverable {
            TrapAction::Retry
        } else {
            TrapAction::Propagate
        }
    }
}

/// Action to take after handling a trap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapAction {
    /// Retry the operation
    Retry,
    /// Propagate the error
    Propagate,
    /// Terminate the instance
    Terminate,
}

/// Trap frame information
///
/// wasmtime 34.0: FrameInfo is available through the error downcast
#[derive(Debug, Clone)]
pub struct TrapFrame {
    /// Function name
    pub function_name: Option<String>,
    /// Module name
    pub module_name: Option<String>,
    /// Source location
    pub source_location: Option<String>,
}

impl TrapFrame {
    /// Create from wasmtime frame
    ///
    /// Note: wasmtime 34.0 FrameInfo API is stable
    #[allow(dead_code)]
    pub fn from_wasmtime_frame(frame: &wasmtime::FrameInfo) -> Self {
        Self {
            function_name: frame.func_name().map(|s| s.to_string()),
            module_name: None,
            source_location: None,
        }
    }
}

/// Extract trap frames from an error
///
/// Note: wasmtime 34.0 provides backtrace through error downcasting
/// This is a simplified implementation
pub fn extract_trap_frames(_e: &wasmtime::Error) -> Vec<TrapFrame> {
    // wasmtime 34.0 backtrace API is available through error downcasting
    // For now, return empty vector
    Vec::new()
}
