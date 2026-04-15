//! WASM Fuel Metering
//!
//! Resource accounting for WASM execution:
//! - Fuel consumption tracking
//! - Cost models for operations
//! - Limit enforcement

// tracing::trace; // Currently unused

/// Fuel limit configuration
#[derive(Debug, Clone, Copy)]
pub enum FuelLimit {
    /// No fuel limit (unlimited execution)
    Infinite,
    /// Limited fuel amount
    Limited(u64),
}

impl FuelLimit {
    /// Get fuel amount, returns max for infinite
    pub fn amount(&self) -> u64 {
        match self {
            FuelLimit::Infinite => u64::MAX,
            FuelLimit::Limited(n) => *n,
        }
    }

    /// Check if fuel is effectively unlimited
    pub fn is_unlimited(&self) -> bool {
        matches!(self, FuelLimit::Infinite)
    }
}

impl Default for FuelLimit {
    fn default() -> Self {
        FuelLimit::Limited(10_000_000)
    }
}

/// Fuel cost model
///
/// Defines how many fuel units each operation costs.
/// Based on wasmtime's fuel consumption model.
#[derive(Debug, Clone)]
pub struct CostModel {
    /// Base cost for any instruction
    pub base_cost: u64,
    /// Memory load cost per byte
    pub memory_load_cost: u64,
    /// Memory store cost per byte
    pub memory_store_cost: u64,
    /// Branch cost
    pub branch_cost: u64,
    /// Call cost
    pub call_cost: u64,
    /// Memory grow cost per page
    pub memory_grow_cost: u64,
    /// Table access cost
    pub table_access_cost: u64,
    /// Host call cost (base)
    pub host_call_base: u64,
    /// Host call cost per byte of data
    pub host_call_per_byte: u64,
}

impl CostModel {
    /// Create a new cost model with default values
    pub fn new() -> Self {
        Self {
            base_cost: 1,
            memory_load_cost: 1,
            memory_store_cost: 1,
            branch_cost: 1,
            call_cost: 1,
            memory_grow_cost: 10_000, // Expensive!
            table_access_cost: 1,
            host_call_base: 100,
            host_call_per_byte: 1,
        }
    }

    /// Conservative cost model (higher costs)
    pub fn conservative() -> Self {
        Self {
            base_cost: 1,
            memory_load_cost: 2,
            memory_store_cost: 2,
            branch_cost: 2,
            call_cost: 2,
            memory_grow_cost: 100_000,
            table_access_cost: 2,
            host_call_base: 500,
            host_call_per_byte: 2,
        }
    }

    /// Relaxed cost model (lower costs)
    pub fn relaxed() -> Self {
        Self {
            base_cost: 1,
            memory_load_cost: 1,
            memory_store_cost: 1,
            branch_cost: 1,
            call_cost: 1,
            memory_grow_cost: 1_000,
            table_access_cost: 1,
            host_call_base: 50,
            host_call_per_byte: 1,
        }
    }

    /// Calculate cost for memory operation
    pub fn memory_cost(&self, load: bool, bytes: usize) -> u64 {
        let per_byte = if load {
            self.memory_load_cost
        } else {
            self.memory_store_cost
        };
        self.base_cost + (per_byte * bytes as u64)
    }

    /// Calculate cost for host call
    pub fn host_call_cost(&self, data_bytes: usize) -> u64 {
        self.host_call_base + (self.host_call_per_byte * data_bytes as u64)
    }
}

impl Default for CostModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Fuel consumption tracker
pub struct FuelTracker {
    /// Cost model
    model: CostModel,
    /// Fuel consumed
    consumed: u64,
    /// Fuel limit
    limit: FuelLimit,
}

impl FuelTracker {
    /// Create new fuel tracker
    pub fn new(model: CostModel, limit: FuelLimit) -> Self {
        Self {
            model,
            consumed: 0,
            limit,
        }
    }

    /// Consume fuel
    pub fn consume(&mut self, amount: u64) -> bool {
        if let FuelLimit::Limited(limit) = self.limit {
            if self.consumed + amount > limit {
                self.consumed = limit;
                return false; // Out of fuel
            }
        }
        self.consumed += amount;
        true
    }

    /// Consume base operation cost
    pub fn consume_base(&mut self) -> bool {
        self.consume(self.model.base_cost)
    }

    /// Consume memory operation cost
    pub fn consume_memory(&mut self, load: bool, bytes: usize) -> bool {
        let cost = self.model.memory_cost(load, bytes);
        self.consume(cost)
    }

    /// Consume host call cost
    pub fn consume_host_call(&mut self, data_bytes: usize) -> bool {
        let cost = self.model.host_call_cost(data_bytes);
        self.consume(cost)
    }

    /// Get consumed fuel
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Get remaining fuel
    pub fn remaining(&self) -> u64 {
        match self.limit {
            FuelLimit::Infinite => u64::MAX,
            FuelLimit::Limited(limit) => limit.saturating_sub(self.consumed),
        }
    }

    /// Check if has fuel remaining
    pub fn has_fuel(&self) -> bool {
        self.remaining() > 0
    }

    /// Get utilization ratio
    pub fn utilization(&self) -> f64 {
        match self.limit {
            FuelLimit::Infinite => 0.0,
            FuelLimit::Limited(limit) => {
                if limit == 0 {
                    return 0.0;
                }
                self.consumed as f64 / limit as f64
            }
        }
    }

    /// Set new limit
    pub fn set_limit(&mut self, limit: FuelLimit) {
        self.limit = limit;
    }

    /// Get current limit
    pub fn limit(&self) -> FuelLimit {
        self.limit
    }

    /// Get cost model
    pub fn model(&self) -> &CostModel {
        &self.model
    }

    /// Reset consumption counter
    pub fn reset(&mut self) {
        self.consumed = 0;
    }
}

/// Resource limits for a WASM instance
#[derive(Debug, Clone)]
pub struct WasmResourceLimits {
    /// Maximum memory in bytes
    pub max_memory: usize,
    /// Maximum fuel
    pub max_fuel: FuelLimit,
    /// Maximum execution time (ms)
    pub max_execution_time_ms: u64,
    /// Maximum call stack depth
    pub max_call_stack: usize,
    /// Maximum host call depth
    pub max_host_call_depth: usize,
}

impl WasmResourceLimits {
    /// Conservative limits for untrusted code
    pub fn conservative() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64MB
            max_fuel: FuelLimit::Limited(1_000_000),
            max_execution_time_ms: 1000, // 1 second
            max_call_stack: 100,
            max_host_call_depth: 10,
        }
    }

    /// Standard limits
    pub fn standard() -> Self {
        Self {
            max_memory: 128 * 1024 * 1024, // 128MB
            max_fuel: FuelLimit::Limited(10_000_000),
            max_execution_time_ms: 30000, // 30 seconds
            max_call_stack: 500,
            max_host_call_depth: 50,
        }
    }

    /// Relaxed limits for trusted code
    pub fn relaxed() -> Self {
        Self {
            max_memory: 512 * 1024 * 1024, // 512MB
            max_fuel: FuelLimit::Limited(100_000_000),
            max_execution_time_ms: 300000, // 5 minutes
            max_call_stack: 1000,
            max_host_call_depth: 100,
        }
    }

    /// Unlimited (use with caution!)
    pub fn unlimited() -> Self {
        Self {
            max_memory: usize::MAX,
            max_fuel: FuelLimit::Infinite,
            max_execution_time_ms: u64::MAX,
            max_call_stack: usize::MAX,
            max_host_call_depth: usize::MAX,
        }
    }
}

impl Default for WasmResourceLimits {
    fn default() -> Self {
        Self::standard()
    }
}

/// Resource usage snapshot
#[derive(Debug, Clone, Copy)]
pub struct ResourceUsage {
    /// Memory used in bytes
    pub memory_bytes: usize,
    /// Fuel consumed
    pub fuel_consumed: u64,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Call stack depth
    pub call_stack_depth: usize,
}

/// Metered execution guard
///
/// Automatically tracks resource usage during execution
pub struct MeteredExecution {
    /// Fuel tracker
    tracker: FuelTracker,
    /// Resource limits
    limits: WasmResourceLimits,
    /// Start timestamp
    start_time: std::time::Instant,
}

impl MeteredExecution {
    /// Start metered execution
    pub fn start(limits: WasmResourceLimits) -> Self {
        Self {
            tracker: FuelTracker::new(CostModel::new(), limits.max_fuel.clone()),
            limits,
            start_time: std::time::Instant::now(),
        }
    }

    /// Check if execution should continue
    pub fn check(&self) -> Result<(), MeteringError> {
        // Check fuel
        if !self.tracker.has_fuel() {
            return Err(MeteringError::OutOfFuel);
        }

        // Check time
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        if elapsed > self.limits.max_execution_time_ms {
            return Err(MeteringError::Timeout);
        }

        Ok(())
    }

    /// Get current usage
    pub fn usage(&self) -> ResourceUsage {
        ResourceUsage {
            memory_bytes: 0, // Would need to track separately
            fuel_consumed: self.tracker.consumed(),
            execution_time_ms: self.start_time.elapsed().as_millis() as u64,
            call_stack_depth: 0, // Would need to track separately
        }
    }

    /// Get tracker reference
    pub fn tracker(&self) -> &FuelTracker {
        &self.tracker
    }

    /// Get tracker mutable reference
    pub fn tracker_mut(&mut self) -> &mut FuelTracker {
        &mut self.tracker
    }
}

/// Metering error
#[derive(Debug, Clone)]
pub enum MeteringError {
    /// Out of fuel
    OutOfFuel,
    /// Execution timeout
    Timeout,
    /// Stack overflow
    StackOverflow,
    /// Memory limit exceeded
    MemoryExceeded,
}

impl std::fmt::Display for MeteringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeteringError::OutOfFuel => write!(f, "Out of fuel"),
            MeteringError::Timeout => write!(f, "Execution timeout"),
            MeteringError::StackOverflow => write!(f, "Stack overflow"),
            MeteringError::MemoryExceeded => write!(f, "Memory limit exceeded"),
        }
    }
}

impl std::error::Error for MeteringError {}

/// Convert metering error to kernel error
impl From<MeteringError> for crate::error::KernelError {
    fn from(e: MeteringError) -> Self {
        match e {
            MeteringError::OutOfFuel => {
                crate::error::KernelError::resource_exhausted("Out of fuel")
            }
            MeteringError::Timeout => crate::error::KernelError::Timeout,
            MeteringError::StackOverflow => {
                crate::error::KernelError::resource_exhausted("Stack overflow")
            }
            MeteringError::MemoryExceeded => {
                crate::error::KernelError::Memory("Memory limit exceeded".into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuel_limit() {
        let limit = FuelLimit::Limited(1000);
        assert_eq!(limit.amount(), 1000);
        assert!(!limit.is_unlimited());

        let infinite = FuelLimit::Infinite;
        assert!(infinite.is_unlimited());
    }

    #[test]
    fn test_fuel_tracker() {
        let model = CostModel::new();
        let limit = FuelLimit::Limited(100);
        let mut tracker = FuelTracker::new(model, limit);

        assert!(tracker.consume(50));
        assert_eq!(tracker.consumed(), 50);
        assert_eq!(tracker.remaining(), 50);

        assert!(!tracker.consume(60)); // Would exceed limit
        assert!(tracker.remaining() <= 50);
    }

    #[test]
    fn test_cost_model() {
        let model = CostModel::conservative();

        let mem_load = model.memory_cost(true, 1024);
        let mem_store = model.memory_cost(false, 1024);

        assert!(mem_load > 0);
        assert!(mem_store >= mem_load);
    }

    #[test]
    fn test_resource_limits() {
        let conservative = WasmResourceLimits::conservative();
        assert!(conservative.max_memory < WasmResourceLimits::relaxed().max_memory);

        let standard = WasmResourceLimits::standard();
        assert!(!standard.max_fuel.is_unlimited());
    }
}
