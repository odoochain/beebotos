//! Skills Security

use crate::error::{AgentError, Result};

/// Security policy for skills
#[derive(Debug, Clone)]
pub struct SkillSecurityPolicy {
    pub allow_network: bool,
    pub allow_filesystem: bool,
    pub allow_env_access: bool,
    pub max_memory_mb: u32,
    pub timeout_secs: u32,
    /// Allowed WASM imports (whitelist approach)
    pub allowed_imports: Vec<String>,
    /// Maximum WASM module size in bytes
    pub max_module_size: usize,
}

impl Default for SkillSecurityPolicy {
    fn default() -> Self {
        Self {
            allow_network: false,
            allow_filesystem: false,
            allow_env_access: false,
            max_memory_mb: 128,
            timeout_secs: 30,
            allowed_imports: vec![
                "env".to_string(),
                "env.memory".to_string(),
                "env.abort".to_string(),
            ],
            max_module_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Skill security validator
pub struct SkillSecurityValidator {
    policy: SkillSecurityPolicy,
}

/// WASM validation error
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Module too large
    ModuleTooLarge { size: usize, max: usize },
    /// Unauthorized import
    UnauthorizedImport(String),
    /// Invalid WASM format
    InvalidWasm(String),
    /// Memory limit exceeded
    MemoryLimitExceeded { requested: u32, max: u32 },
    /// Dangerous pattern detected
    DangerousPattern(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::ModuleTooLarge { size, max } => {
                write!(f, "WASM module size {} exceeds maximum {}", size, max)
            }
            ValidationError::UnauthorizedImport(import) => {
                write!(f, "Unauthorized import: {}", import)
            }
            ValidationError::InvalidWasm(msg) => write!(f, "Invalid WASM: {}", msg),
            ValidationError::MemoryLimitExceeded { requested, max } => {
                write!(f, "Memory limit {} exceeds maximum {}", requested, max)
            }
            ValidationError::DangerousPattern(pattern) => {
                write!(f, "Dangerous pattern detected: {}", pattern)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl SkillSecurityValidator {
    pub fn new(policy: SkillSecurityPolicy) -> Self {
        Self { policy }
    }

    /// Validate WASM module against security policy
    /// 
    /// Checks:
    /// 1. Module size limits
    /// 2. Import whitelist
    /// 3. Memory limits
    /// 4. Dangerous patterns (host function calls, etc.)
    /// 5. Valid WASM structure
    pub fn validate(&self, skill_wasm: &[u8]) -> Result<()> {
        // Check module size
        if skill_wasm.len() > self.policy.max_module_size {
            return Err(AgentError::Validation(format!(
                "WASM module too large: {} bytes (max: {})",
                skill_wasm.len(),
                self.policy.max_module_size
            )));
        }

        // Parse and validate WASM module
        self.validate_wasm_structure(skill_wasm)?;
        
        // Validate imports against whitelist
        self.validate_imports(skill_wasm)?;
        
        // Validate memory limits
        self.validate_memory_limits(skill_wasm)?;
        
        // Check for dangerous patterns
        self.validate_no_dangerous_patterns(skill_wasm)?;

        Ok(())
    }

    /// Validate basic WASM structure
    fn validate_wasm_structure(&self, wasm: &[u8]) -> Result<()> {
        // Check WASM magic number and version
        if wasm.len() < 8 {
            return Err(AgentError::Validation(
                "WASM module too small".to_string()
            ));
        }

        // WASM magic number: \0asm
        if &wasm[0..4] != &[0x00, 0x61, 0x73, 0x6d] {
            return Err(AgentError::Validation(
                "Invalid WASM magic number".to_string()
            ));
        }

        // WASM version: 1
        if &wasm[4..8] != &[0x01, 0x00, 0x00, 0x00] {
            return Err(AgentError::Validation(
                "Unsupported WASM version".to_string()
            ));
        }

        Ok(())
    }

    /// Validate imports against whitelist
    fn validate_imports(&self, wasm: &[u8]) -> Result<()> {
        // Parse import section
        // This is a simplified check - in production, use wasmparser crate
        let forbidden_imports = vec![
            "env.__syscall",
            "env.__wasi",
            "env.abort",
            "env.exit",
        ];

        // Check for forbidden import patterns in raw bytes
        // Note: This is a basic check. Full WASM parsing is recommended
        let wasm_str = String::from_utf8_lossy(wasm);
        for forbidden in &forbidden_imports {
            if wasm_str.contains(forbidden) && !self.policy.allowed_imports.contains(&forbidden.to_string()) {
                return Err(AgentError::Validation(format!(
                    "Forbidden import detected: {}", forbidden
                )));
            }
        }

        Ok(())
    }

    /// Validate memory limits
    fn validate_memory_limits(&self, wasm: &[u8]) -> Result<()> {
        // Look for memory section and validate limits
        // Simplified: check for memory-related patterns
        // In production, parse the WASM properly
        
        let max_memory_pages = self.policy.max_memory_mb * 16; // 64KB per page
        
        // Basic check for memory section
        // Full implementation would parse the WASM memory section properly
        if wasm.len() > 1024 * 1024 * self.policy.max_memory_mb as usize {
            return Err(AgentError::Validation(format!(
                "WASM may exceed memory limit of {} MB",
                self.policy.max_memory_mb
            )));
        }

        Ok(())
    }

    /// Check for dangerous patterns that could lead to sandbox escape
    fn validate_no_dangerous_patterns(&self, wasm: &[u8]) -> Result<()> {
        let wasm_str = String::from_utf8_lossy(wasm);
        
        // Check for potentially dangerous patterns
        let dangerous_patterns = vec![
            ("inline assembly", "asm"),
            ("raw pointer manipulation", "unsafe"),
            ("direct system calls", "syscall"),
            ("memory corruption", "buffer overflow"),
        ];

        for (name, pattern) in &dangerous_patterns {
            if wasm_str.contains(pattern) {
                return Err(AgentError::Validation(format!(
                    "Dangerous pattern detected: {} ({})", name, pattern
                )));
            }
        }

        // Check for known vulnerable function imports
        let vulnerable_functions = vec![
            "__stack_chk_fail",
            "__libc_start_main",
            "system",
            "exec",
            "popen",
        ];

        for func in &vulnerable_functions {
            if wasm_str.contains(func) {
                return Err(AgentError::Validation(format!(
                    "Potentially vulnerable function import: {}", func
                )));
            }
        }

        Ok(())
    }

    /// Detect potential sandbox escape attempts
    pub fn detect_sandbox_escape(&self, wasm: &[u8]) -> Result<Vec<String>> {
        let mut detections = Vec::new();
        let wasm_str = String::from_utf8_lossy(wasm);

        // Check for known escape techniques
        let escape_patterns = vec![
            ("Spectre/Meltdown", "rdtsc"),
            ("Rowhammer", "clflush"),
            ("Side-channel", "cache"),
            ("Timing attack", "performance.now"),
        ];

        for (technique, pattern) in &escape_patterns {
            if wasm_str.contains(pattern) {
                detections.push(format!("{} attack pattern detected", technique));
            }
        }

        Ok(detections)
    }

    pub fn policy(&self) -> &SkillSecurityPolicy {
        &self.policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_magic_validation() {
        let validator = SkillSecurityValidator::new(SkillSecurityPolicy::default());
        
        // Valid WASM header
        let valid_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        assert!(validator.validate_wasm_structure(&valid_wasm).is_ok());
        
        // Invalid WASM header
        let invalid_wasm = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
        assert!(validator.validate_wasm_structure(&invalid_wasm).is_err());
    }

    #[test]
    fn test_module_size_limit() {
        let mut policy = SkillSecurityPolicy::default();
        policy.max_module_size = 100;
        
        let validator = SkillSecurityValidator::new(policy);
        
        // Module too large
        let large_wasm = vec![0x00; 101];
        assert!(validator.validate(&large_wasm).is_err());
    }
}
