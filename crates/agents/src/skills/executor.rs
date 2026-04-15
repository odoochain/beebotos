//! Skill Executor
//!
//! Executes WASM skills in sandboxed environment using kernel's WASM runtime.

use beebotos_kernel::error::KernelError;
use beebotos_kernel::wasm::{EngineConfig, WasmEngine};

use crate::skills::loader::LoadedSkill;

/// Skill executor using kernel's WASM runtime
pub struct SkillExecutor {
    engine: WasmEngine,
}

/// Execution context for skills
#[derive(Debug, Clone)]
pub struct SkillContext {
    pub input: String,
    pub parameters: std::collections::HashMap<String, String>,
}

/// Skill execution result
#[derive(Debug, Clone)]
pub struct SkillExecutionResult {
    pub task_id: String,
    pub success: bool,
    pub output: String,
    pub execution_time_ms: u64,
}

impl SkillExecutor {
    /// Create a new skill executor with default configuration
    pub fn new() -> Result<Self, SkillExecutionError> {
        let config = EngineConfig::default();
        let engine =
            WasmEngine::new(config).map_err(|e| SkillExecutionError::EngineError(e.to_string()))?;

        Ok(Self { engine })
    }

    /// Create with custom engine configuration
    pub fn with_config(config: EngineConfig) -> Result<Self, SkillExecutionError> {
        let engine =
            WasmEngine::new(config).map_err(|e| SkillExecutionError::EngineError(e.to_string()))?;

        Ok(Self { engine })
    }

    /// Execute a skill
    pub async fn execute(
        &self,
        skill: &LoadedSkill,
        context: SkillContext,
    ) -> Result<SkillExecutionResult, SkillExecutionError> {
        // Read WASM bytes
        let wasm_bytes = tokio::fs::read(&skill.wasm_path)
            .await
            .map_err(|e| SkillExecutionError::IoError(e.to_string()))?;

        // Compile and cache module using kernel's engine
        let module = self
            .engine
            .compile_cached(&skill.id, &wasm_bytes)
            .map_err(|e| SkillExecutionError::InvalidWasm(e.to_string()))?;

        // Instantiate with host functions
        let mut instance = self
            .engine
            .instantiate_with_host(&module, &skill.id)
            .map_err(|e| SkillExecutionError::InstantiationError(e.to_string()))?;

        // Get entry function and execute
        let start_time = std::time::Instant::now();

        // Serialize input to memory
        let input_bytes = context.input.as_bytes();
        let input_ptr = 0; // Simplified - in production use proper memory allocation

        instance
            .write_memory(input_ptr, input_bytes)
            .map_err(|e| SkillExecutionError::ExecutionFailed(e.to_string()))?;

        // Call entry point
        let result = instance.call_typed::<(i32, i32), i32>(
            &skill.manifest.entry_point,
            (input_ptr as i32, input_bytes.len() as i32),
        );

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(_) => Ok(SkillExecutionResult {
                task_id: skill.id.clone(),
                success: true,
                output: "Skill executed successfully".to_string(),
                execution_time_ms,
            }),
            Err(e) => Err(SkillExecutionError::ExecutionFailed(e.to_string())),
        }
    }

    /// Precompile skill for faster loading
    pub async fn precompile(&self, skill: &LoadedSkill) -> Result<Vec<u8>, SkillExecutionError> {
        let wasm_bytes = tokio::fs::read(&skill.wasm_path)
            .await
            .map_err(|e| SkillExecutionError::IoError(e.to_string()))?;

        self.engine
            .precompile(&wasm_bytes)
            .map_err(|e| SkillExecutionError::CompilationError(e.to_string()))
    }

    /// Get engine cache statistics
    pub fn cache_stats(&self) -> beebotos_kernel::wasm::CacheStats {
        self.engine.cache_stats()
    }
}

impl Default for SkillExecutor {
    fn default() -> Self {
        // Note: This may panic if engine creation fails.
        // In production, prefer using SkillExecutor::new() and handling the error.
        match Self::new() {
            Ok(executor) => executor,
            Err(e) => {
                tracing::error!("Failed to create SkillExecutor in Default: {}", e);
                panic!("Failed to create SkillExecutor: {}", e)
            }
        }
    }
}

/// Skill execution errors
#[derive(Debug, Clone)]
pub enum SkillExecutionError {
    IoError(String),
    EngineError(String),
    InvalidWasm(String),
    InstantiationError(String),
    EntryPointNotFound(String),
    ExecutionFailed(String),
    CompilationError(String),
}

impl std::fmt::Display for SkillExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillExecutionError::IoError(s) => write!(f, "IO error: {}", s),
            SkillExecutionError::EngineError(s) => write!(f, "Engine error: {}", s),
            SkillExecutionError::InvalidWasm(s) => write!(f, "Invalid WASM: {}", s),
            SkillExecutionError::InstantiationError(s) => write!(f, "Instantiation error: {}", s),
            SkillExecutionError::EntryPointNotFound(s) => write!(f, "Entry point not found: {}", s),
            SkillExecutionError::ExecutionFailed(s) => write!(f, "Execution failed: {}", s),
            SkillExecutionError::CompilationError(s) => write!(f, "Compilation error: {}", s),
        }
    }
}

impl std::error::Error for SkillExecutionError {}

impl From<KernelError> for SkillExecutionError {
    fn from(e: KernelError) -> Self {
        SkillExecutionError::EngineError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_executor_creation() {
        let executor = SkillExecutor::new();
        assert!(executor.is_ok());
    }
}
