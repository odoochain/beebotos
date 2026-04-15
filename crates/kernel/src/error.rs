//! Kernel Error Types

use thiserror::Error;

/// Kernel result type
pub type Result<T> = std::result::Result<T, KernelError>;

/// Boot errors
#[derive(Error, Debug, Clone)]
pub enum BootError {
    /// Boot process failed
    #[error("Boot failed: {0}")]
    Failed(String),

    /// Invalid boot parameters
    #[error("Invalid boot parameters")]
    InvalidParameters,

    /// Hardware initialization failed
    #[error("Hardware initialization failed: {0}")]
    HardwareInitFailed(String),

    /// Memory initialization failed
    #[error("Memory initialization failed")]
    MemoryInitFailed,

    /// Interrupt setup failed
    #[error("Interrupt setup failed")]
    InterruptSetupFailed,

    /// Scheduler initialization failed
    #[error("Scheduler initialization failed")]
    SchedulerInitFailed,
}

/// Kernel error types
#[derive(Error, Debug, Clone)]
pub enum KernelError {
    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Agent already exists
    #[error("Agent already exists: {0}")]
    AgentExists(String),

    /// Insufficient capabilities
    #[error("Insufficient capabilities: required {required:?}, have {current:?}")]
    InsufficientCapability {
        /// Required capability level
        required: crate::capabilities::CapabilityLevel,
        /// Current capability level
        current: crate::capabilities::CapabilityLevel,
    },

    /// Security violation
    #[error("Security error: {0}")]
    Security(String),

    /// Invalid capability
    #[error("Invalid capability")]
    InvalidCapability,

    /// Capability has expired
    #[error("Capability expired")]
    CapabilityExpired,

    /// Resource limit exceeded
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Invalid syscall number
    #[error("Invalid syscall: {0}")]
    InvalidSyscall(u64),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Operation timed out
    #[error("Timeout")]
    Timeout,

    /// Operation would block
    #[error("Would block")]
    WouldBlock,

    /// Scheduler error
    #[error("Scheduler error: {0}")]
    Scheduler(String),

    /// Memory error
    #[error("Memory error: {0}")]
    Memory(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Feature not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Out of memory
    #[error("Out of memory")]
    OutOfMemory,

    /// Invalid memory address
    #[error("Invalid memory address")]
    InvalidAddress,
}

impl KernelError {
    /// Create out of memory error
    pub fn out_of_memory() -> Self {
        KernelError::OutOfMemory
    }

    /// Create invalid address error
    pub fn invalid_address() -> Self {
        KernelError::InvalidAddress
    }

    /// Create resource exhausted error
    pub fn resource_exhausted(msg: impl Into<String>) -> Self {
        KernelError::ResourceExhausted(msg.into())
    }

    /// Create internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        KernelError::Internal(msg.into())
    }

    /// Create invalid argument error
    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        KernelError::InvalidArgument(msg.into())
    }

    /// Create not implemented error
    pub fn not_implemented(msg: impl Into<String>) -> Self {
        KernelError::NotImplemented(msg.into())
    }

    /// Create IO error
    pub fn io(msg: impl Into<String>) -> Self {
        KernelError::Io(msg.into())
    }

    /// Create memory error
    pub fn memory(msg: impl Into<String>) -> Self {
        KernelError::Memory(msg.into())
    }

    /// Create TEE error
    pub fn tee_error(msg: impl Into<String>) -> Self {
        KernelError::Security(format!("TEE: {}", msg.into()))
    }
}

/// Security errors
#[derive(Error, Debug, Clone)]
pub enum SecurityError {
    /// Capability has expired
    #[error("Capability expired")]
    CapabilityExpired,

    /// Insufficient capability level
    #[error("Insufficient capability: required {required:?}, current {current:?}")]
    InsufficientCapability {
        /// Required capability level
        required: crate::capabilities::CapabilityLevel,
        /// Current capability level
        current: crate::capabilities::CapabilityLevel,
    },

    /// No capabilities assigned
    #[error("No capabilities assigned")]
    NoCapabilities,

    /// Capability not delegable
    #[error("Not delegable")]
    NotDelegable,

    /// Invalid security token
    #[error("Invalid token")]
    InvalidToken,

    /// Sandbox security violation
    #[error("Sandbox violation")]
    SandboxViolation,

    /// TEE not available
    #[error("TEE unavailable")]
    TeeUnavailable,
}

impl From<SecurityError> for KernelError {
    fn from(e: SecurityError) -> Self {
        KernelError::Security(e.to_string())
    }
}

impl From<crate::scheduler::SchedulerError> for KernelError {
    fn from(e: crate::scheduler::SchedulerError) -> Self {
        KernelError::Scheduler(e.to_string())
    }
}
