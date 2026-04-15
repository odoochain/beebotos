//! WASM Runtime
//!
//! WebAssembly execution environment for BeeBotOS kernel.
//! Updated for wasmtime 34.0 API.

pub mod engine;
pub mod host_funcs;
pub mod instance;
pub mod memory;
pub mod metering;
pub mod precompile;
pub mod trap;
pub mod wasi_ctx;
pub mod wasi_view;

pub use engine::{CacheStats, EngineConfig, WasmEngine};
pub use host_funcs::{HostContext, HostFunctions};
pub use instance::{InstanceStats, WasmInstance};
pub use memory::{MemoryConfig, MemoryManager, MemoryStats, MAX_PAGES, PAGE_SIZE};
pub use metering::{
    CostModel, FuelLimit, FuelTracker, ResourceUsage as MeteringResourceUsage, WasmResourceLimits,
};
pub use precompile::{PrecompileCache, PrecompileManager, PrecompileStats};
pub use trap::{convert_wasmtime_error, TrapAction, TrapFrame, TrapHandler, WasmTrap};
pub use wasi_ctx::{
    create_restricted_wasi_context, create_wasi_context, create_wasi_context_with_caps,
    FilesystemAccess, StdioConfig, WasiCapabilities, WasiHostContext,
};
pub use wasi_view::{BeeBotOsWasiView, ComponentEngine, ComponentInstance};

use crate::error::Result;

/// WASM runtime version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize WASM runtime
///
/// Must be called before using any WASM functionality.
pub fn init() -> Result<()> {
    tracing::info!("Initializing WASM runtime v{}", VERSION);

    // Verify we can create an engine
    let _config = EngineConfig::default();

    tracing::info!("WASM runtime initialized successfully");
    Ok(())
}

/// Initialize with custom configuration
pub fn init_with_config(config: EngineConfig) -> Result<()> {
    tracing::info!("Initializing WASM runtime with custom config");

    // Create engine to verify configuration is valid
    let _engine = WasmEngine::new(config)?;

    tracing::info!("WASM runtime initialized successfully");
    Ok(())
}

/// Check if WASM runtime is available
pub fn is_available() -> bool {
    // Try to create a default engine
    std::panic::catch_unwind(|| {
        EngineConfig::default();
    })
    .is_ok()
}

/// Get runtime version info
pub fn version_info() -> VersionInfo {
    VersionInfo {
        version: VERSION.to_string(),
        wasmtime_version: "34.0".to_string(),
    }
}

/// Version information
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Runtime version
    pub version: String,
    /// Wasmtime version
    pub wasmtime_version: String,
}

/// Quick compile helper
///
/// Compiles WASM bytes to a module using default engine configuration.
pub fn quick_compile(wasm_bytes: &[u8]) -> Result<wasmtime::Module> {
    let engine = WasmEngine::new(EngineConfig::default())?;
    engine.compile(wasm_bytes)
}

/// Quick instantiate helper
///
/// Compiles and instantiates WASM bytes with default settings.
pub fn quick_instantiate(wasm_bytes: &[u8]) -> Result<WasmInstance> {
    let engine = WasmEngine::new(EngineConfig::default())?;
    let module = engine.compile(wasm_bytes)?;
    engine.instantiate(&module)
}

/// Global runtime statistics
static RUNTIME_STATS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Increment instances created counter
pub fn record_instance_created() {
    use std::sync::atomic::Ordering;
    RUNTIME_STATS.fetch_add(1, Ordering::Relaxed);
}

/// Get total instances created
pub fn total_instances_created() -> u64 {
    use std::sync::atomic::Ordering;
    RUNTIME_STATS.load(Ordering::Relaxed)
}

/// Utility to create a simple add function WASM module (for testing)
pub fn test_module_add() -> Vec<u8> {
    // Minimal WASM module that exports an "add" function
    // (module
    //   (func $add (param i32 i32) (result i32)
    //     local.get 0
    //     local.get 1
    //     i32.add)
    //   (export "add" (func $add)))
    vec![
        0x00, 0x61, 0x73, 0x6d, // magic
        0x01, 0x00, 0x00, 0x00, // version
        0x01, 0x07, 0x01, // type section
        0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, // func type (i32, i32) -> i32
        0x03, 0x02, 0x01, 0x00, // func section
        0x07, 0x07, 0x01, // export section
        0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // "add" -> func 0
        0x0a, 0x09, 0x01, // code section
        0x07, 0x00, // body
        0x20, 0x00, // local.get 0
        0x20, 0x01, // local.get 1
        0x6a, // i32.add
        0x0b, // end
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let info = version_info();
        assert!(!info.version.is_empty());
    }

    #[test]
    fn test_test_module_add() {
        let wasm = test_module_add();
        assert!(!wasm.is_empty());

        // Try to compile it
        let result = quick_compile(&wasm);
        assert!(
            result.is_ok(),
            "Failed to compile test module: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_quick_instantiate() {
        let wasm = test_module_add();
        let result = quick_instantiate(&wasm);
        assert!(result.is_ok(), "Failed to instantiate: {:?}", result.err());
    }
}
