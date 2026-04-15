//! GPU Acceleration Module
//!
//! Provides GPU acceleration for neural network operations and batch processing.
//! Supports CUDA (NVIDIA) and ROCm (AMD) backends.

pub mod cuda;
pub mod rocm;
pub mod ops;
pub mod buffer;

pub use cuda::{CudaBackend, CudaError};
pub use rocm::{RocmBackend, RocmError};
pub use ops::{
    GpuOps, MatrixOp, VectorOp, ActivationOp,
};
pub use buffer::{GpuBuffer, BufferType};

use serde::{Deserialize, Serialize};

/// GPU backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuBackendType {
    /// NVIDIA CUDA
    Cuda,
    /// AMD ROCm
    Rocm,
    /// Apple Metal (future)
    Metal,
    /// WebGPU (future)
    WebGpu,
}

/// GPU configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// Backend type
    pub backend: GpuBackendType,
    /// Device ID (0 = default)
    pub device_id: i32,
    /// Enable memory pooling
    pub enable_memory_pool: bool,
    /// Memory pool size (MB)
    pub memory_pool_size_mb: usize,
    /// Batch size for operations
    pub batch_size: usize,
    /// Stream count for parallel execution
    pub stream_count: usize,
}

impl GpuConfig {
    /// Create CUDA configuration
    pub fn cuda(device_id: i32) -> Self {
        Self {
            backend: GpuBackendType::Cuda,
            device_id,
            enable_memory_pool: true,
            memory_pool_size_mb: 512,
            batch_size: 1024,
            stream_count: 4,
        }
    }

    /// Create ROCm configuration
    pub fn rocm(device_id: i32) -> Self {
        Self {
            backend: GpuBackendType::Rocm,
            device_id,
            enable_memory_pool: true,
            memory_pool_size_mb: 512,
            batch_size: 1024,
            stream_count: 4,
        }
    }
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self::cuda(0)
    }
}

/// GPU device information
#[derive(Debug, Clone)]
pub struct GpuDeviceInfo {
    /// Device name
    pub name: String,
    /// Total memory (bytes)
    pub total_memory: usize,
    /// Compute capability
    pub compute_capability: Option<(i32, i32)>,
}
