//! Context Management Module
//!
//! Provides functionality for assembling and managing conversation context
//! for LLM interactions, including token estimation, context window management,
//! and historical message retrieval.

pub mod assembler;

pub use assembler::{
    AssembledContext, AssemblerConfig, AssemblyStrategy, ContextAssembler, ContextMessage,
    ContextMetadata,
};

/// Context manager for handling conversation state
#[derive(Debug, Clone)]
pub struct ContextManager {
    assembler: ContextAssembler,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self {
            assembler: ContextAssembler::new(),
        }
    }

    /// Create with custom assembler configuration
    pub fn with_config(config: AssemblerConfig) -> Self {
        Self {
            assembler: ContextAssembler::with_config(config),
        }
    }

    /// Get the underlying assembler
    pub fn assembler(&self) -> &ContextAssembler {
        &self.assembler
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}
