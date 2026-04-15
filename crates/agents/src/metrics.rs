//! Metrics Module
//!
//! 🟢 P2 FIX: Comprehensive Prometheus-style metrics for observability.
//!
//! This module provides production-ready metrics collection and export
//! in Prometheus format. It includes:
//! - Counters, Gauges, and Histograms
//! - Labels/dimensions support
//! - Metric collectors for different subsystems
//! - Prometheus text format export
//!
//! # Usage
//!
//! ```ignore
//! use beebotos_agents::metrics::{AgentMetrics, MetricsCollector};
//!
//! let metrics = MetricsCollector::new();
//! metrics.record_task_started("agent-1", "llm_chat");
//! metrics.record_task_completed("agent-1", "llm_chat", 150);
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::debug;

// Import the labels macro from crate root (it's #[macro_export])
use crate::labels;

// =============================================================================
// Core Metric Types
// =============================================================================

/// Counter metric - monotonically increasing value
#[derive(Debug)]
pub struct Counter {
    value: AtomicU64,
    labels: HashMap<String, String>,
}

impl Counter {
    /// Create a new counter
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
            labels: HashMap::new(),
        }
    }

    /// Create a counter with labels
    pub fn with_labels(labels: HashMap<String, String>) -> Self {
        Self {
            value: AtomicU64::new(0),
            labels,
        }
    }

    /// Increment by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment by delta
    pub fn add(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    /// Get current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get labels
    pub fn labels(&self) -> &HashMap<String, String> {
        &self.labels
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Gauge metric - can go up and down
#[derive(Debug)]
pub struct Gauge {
    value: AtomicU64,
    labels: HashMap<String, String>,
}

impl Gauge {
    /// Create a new gauge
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
            labels: HashMap::new(),
        }
    }

    /// Create a gauge with labels
    pub fn with_labels(labels: HashMap<String, String>) -> Self {
        Self {
            value: AtomicU64::new(0),
            labels,
        }
    }

    /// Set value
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// Increment by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement by 1
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// Add delta
    pub fn add(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    /// Subtract delta
    pub fn sub(&self, delta: u64) {
        self.value.fetch_sub(delta, Ordering::Relaxed);
    }

    /// Get current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get labels
    pub fn labels(&self) -> &HashMap<String, String> {
        &self.labels
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

/// Histogram bucket for latency distribution
#[derive(Debug, Clone)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Histogram metric for latency tracking
#[derive(Debug)]
pub struct Histogram {
    buckets: RwLock<Vec<HistogramBucket>>,
    sum: AtomicU64, // Stored as nanoseconds * 1000 for precision
    count: AtomicU64,
}

impl Histogram {
    /// Create histogram with default buckets (in milliseconds)
    pub fn new() -> Self {
        Self::with_buckets(&[
            1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0,
        ])
    }

    /// Create histogram with custom buckets (in milliseconds)
    pub fn with_buckets(buckets: &[f64]) -> Self {
        let buckets = buckets
            .iter()
            .map(|&upper_bound| HistogramBucket {
                upper_bound,
                count: 0,
            })
            .collect();

        Self {
            buckets: RwLock::new(buckets),
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    /// Record a value (in milliseconds)
    pub fn observe(&self, value_ms: u64) {
        let value_f64 = value_ms as f64;

        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value_ms, Ordering::Relaxed);

        let mut buckets = self.buckets.write();
        for bucket in buckets.iter_mut() {
            if value_f64 <= bucket.upper_bound {
                bucket.count += 1;
            }
        }
    }

    /// Get bucket counts
    pub fn buckets(&self) -> Vec<HistogramBucket> {
        self.buckets.read().clone()
    }

    /// Get total count
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get sum (in milliseconds)
    pub fn sum(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Metrics Registry
// =============================================================================

/// Metrics registry with label support
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    counters: RwLock<HashMap<String, Vec<Arc<Counter>>>>,
    gauges: RwLock<HashMap<String, Vec<Arc<Gauge>>>>,
    histograms: RwLock<HashMap<String, Arc<Histogram>>>,
}

impl MetricsRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
        }
    }

    /// Register or get a counter
    pub fn counter(&self, name: impl Into<String>) -> Arc<Counter> {
        let name = name.into();
        let counter = Arc::new(Counter::new());

        let mut counters = self.counters.write();
        counters.entry(name).or_default().push(counter.clone());
        counter
    }

    /// Register a counter with specific labels
    pub fn counter_with_labels(
        &self,
        name: impl Into<String>,
        labels: HashMap<String, String>,
    ) -> Arc<Counter> {
        let name = name.into();
        let counter = Arc::new(Counter::with_labels(labels));

        let mut counters = self.counters.write();
        counters.entry(name).or_default().push(counter.clone());
        counter
    }

    /// Register or get a gauge
    pub fn gauge(&self, name: impl Into<String>) -> Arc<Gauge> {
        let name = name.into();
        let gauge = Arc::new(Gauge::new());

        let mut gauges = self.gauges.write();
        gauges.entry(name).or_default().push(gauge.clone());
        gauge
    }

    /// Register a gauge with specific labels
    pub fn gauge_with_labels(
        &self,
        name: impl Into<String>,
        labels: HashMap<String, String>,
    ) -> Arc<Gauge> {
        let name = name.into();
        let gauge = Arc::new(Gauge::with_labels(labels));

        let mut gauges = self.gauges.write();
        gauges.entry(name).or_default().push(gauge.clone());
        gauge
    }

    /// Register a histogram
    pub fn histogram(&self, name: impl Into<String>) -> Arc<Histogram> {
        let name = name.into();
        let histogram = Arc::new(Histogram::new());

        let mut histograms = self.histograms.write();
        histograms.entry(name).or_insert_with(|| histogram.clone());
        histogram
    }

    /// Register a histogram with custom buckets
    pub fn histogram_with_buckets(
        &self,
        name: impl Into<String>,
        buckets: &[f64],
    ) -> Arc<Histogram> {
        let name = name.into();
        let histogram = Arc::new(Histogram::with_buckets(buckets));

        let mut histograms = self.histograms.write();
        histograms.entry(name).or_insert_with(|| histogram.clone());
        histogram
    }

    /// Export metrics in Prometheus text format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // Export counters
        let counters = self.counters.read();
        for (name, instances) in counters.iter() {
            output.push_str(&format!("# TYPE {} counter\n", name));
            for counter in instances {
                let labels = format_labels(counter.labels());
                output.push_str(&format!("{}{} {}\n", name, labels, counter.get()));
            }
        }

        // Export gauges
        let gauges = self.gauges.read();
        for (name, instances) in gauges.iter() {
            output.push_str(&format!("# TYPE {} gauge\n", name));
            for gauge in instances {
                let labels = format_labels(gauge.labels());
                output.push_str(&format!("{}{} {}\n", name, labels, gauge.get()));
            }
        }

        // Export histograms
        let histograms = self.histograms.read();
        for (name, histogram) in histograms.iter() {
            output.push_str(&format!("# TYPE {} histogram\n", name));

            let buckets = histogram.buckets();
            for bucket in &buckets {
                output.push_str(&format!(
                    "{}_bucket{{le=\"{}\"}} {}\n",
                    name, bucket.upper_bound, bucket.count
                ));
            }
            // +Inf bucket
            output.push_str(&format!(
                "{}_bucket{{le=\"+Inf\"}} {}\n",
                name,
                histogram.count()
            ));

            output.push_str(&format!("{}_sum {}\n", name, histogram.sum()));
            output.push_str(&format!("{}_count {}\n", name, histogram.count()));
        }

        output
    }

    /// Get all metric names
    pub fn metric_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.counters.read().keys().cloned());
        names.extend(self.gauges.read().keys().cloned());
        names.extend(self.histograms.read().keys().cloned());
        names
    }
}

/// Format labels for Prometheus output
fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let label_strs: Vec<String> = labels
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, v))
        .collect();

    format!("{{{}}}", label_strs.join(","))
}

// =============================================================================
// 🟢 P2 FIX: Comprehensive Agent Metrics
// =============================================================================

/// Comprehensive metrics collector for Agent operations
pub struct MetricsCollector {
    registry: Arc<MetricsRegistry>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            registry: Arc::new(MetricsRegistry::new()),
        }
    }

    /// Get the underlying registry
    pub fn registry(&self) -> &Arc<MetricsRegistry> {
        &self.registry
    }

    // Task metrics

    /// Record task started
    pub fn record_task_started(&self, agent_id: &str, task_type: &str) {
        let labels = labels!("agent_id" => agent_id, "task_type" => task_type);
        let counter = self
            .registry
            .counter_with_labels("agent_tasks_started_total", labels);
        counter.inc();
        debug!(
            "Metrics: task started for agent {} (type: {})",
            agent_id, task_type
        );
    }

    /// Record task completed with duration
    pub fn record_task_completed(&self, agent_id: &str, task_type: &str, duration_ms: u64) {
        // Success counter
        let labels =
            labels!("agent_id" => agent_id, "task_type" => task_type, "status" => "success");
        let counter = self
            .registry
            .counter_with_labels("agent_tasks_completed_total", labels);
        counter.inc();

        // Duration histogram
        let hist = self.registry.histogram("agent_task_duration_ms");
        hist.observe(duration_ms);

        debug!(
            "Metrics: task completed for agent {} (type: {}) in {}ms",
            agent_id, task_type, duration_ms
        );
    }

    /// Record task failed
    pub fn record_task_failed(&self, agent_id: &str, task_type: &str, error_type: &str) {
        let labels = labels!(
            "agent_id" => agent_id,
            "task_type" => task_type,
            "error_type" => error_type
        );
        let counter = self
            .registry
            .counter_with_labels("agent_tasks_failed_total", labels);
        counter.inc();
        debug!(
            "Metrics: task failed for agent {} (type: {}, error: {})",
            agent_id, task_type, error_type
        );
    }

    // Message metrics

    /// Record message sent
    pub fn record_message_sent(&self, agent_id: &str, platform: &str) {
        let labels = labels!("agent_id" => agent_id, "platform" => platform);
        let counter = self
            .registry
            .counter_with_labels("agent_messages_sent_total", labels);
        counter.inc();
    }

    /// Record message received
    pub fn record_message_received(&self, agent_id: &str, platform: &str) {
        let labels = labels!("agent_id" => agent_id, "platform" => platform);
        let counter = self
            .registry
            .counter_with_labels("agent_messages_received_total", labels);
        counter.inc();
    }

    // Session metrics

    /// Record session started
    pub fn record_session_started(&self, agent_id: &str, session_type: &str) {
        let labels = labels!("agent_id" => agent_id, "session_type" => session_type);
        let counter = self
            .registry
            .counter_with_labels("agent_sessions_started_total", labels);
        counter.inc();

        // Also increment active sessions gauge
        let gauge = self.registry.gauge("agent_active_sessions");
        gauge.inc();
    }

    /// Record session ended
    pub fn record_session_ended(&self, agent_id: &str, duration_ms: u64) {
        let labels = labels!("agent_id" => agent_id);
        let counter = self
            .registry
            .counter_with_labels("agent_sessions_ended_total", labels);
        counter.inc();

        // Decrement active sessions
        let gauge = self.registry.gauge("agent_active_sessions");
        gauge.dec();

        // Record session duration
        let hist = self.registry.histogram("agent_session_duration_ms");
        hist.observe(duration_ms);
    }

    // Chain transaction metrics

    /// Record chain transaction submitted
    pub fn record_chain_tx_submitted(&self, agent_id: &str, chain_id: u64) {
        let labels = labels!(
            "agent_id" => agent_id,
            "chain_id" => &chain_id.to_string()
        );
        let counter = self
            .registry
            .counter_with_labels("agent_chain_tx_submitted_total", labels);
        counter.inc();
    }

    /// Record chain transaction confirmed
    pub fn record_chain_tx_confirmed(&self, agent_id: &str, chain_id: u64, confirm_time_ms: u64) {
        let labels = labels!(
            "agent_id" => agent_id,
            "chain_id" => &chain_id.to_string()
        );
        let counter = self
            .registry
            .counter_with_labels("agent_chain_tx_confirmed_total", labels);
        counter.inc();

        let hist = self.registry.histogram("agent_chain_tx_confirm_time_ms");
        hist.observe(confirm_time_ms);
    }

    /// Record chain transaction failed
    pub fn record_chain_tx_failed(&self, agent_id: &str, chain_id: u64, reason: &str) {
        let labels = labels!(
            "agent_id" => agent_id,
            "chain_id" => &chain_id.to_string(),
            "reason" => reason
        );
        let counter = self
            .registry
            .counter_with_labels("agent_chain_tx_failed_total", labels);
        counter.inc();
    }

    // WASM execution metrics

    /// Record WASM execution
    pub fn record_wasm_execution(
        &self,
        agent_id: &str,
        skill_name: &str,
        duration_ms: u64,
        success: bool,
    ) {
        let status = if success { "success" } else { "failure" };
        let labels = labels!(
            "agent_id" => agent_id,
            "skill_name" => skill_name,
            "status" => status
        );
        let counter = self
            .registry
            .counter_with_labels("agent_wasm_executions_total", labels);
        counter.inc();

        let hist = self.registry.histogram("agent_wasm_execution_duration_ms");
        hist.observe(duration_ms);
    }

    // Export metrics

    /// Export in Prometheus format
    pub fn export_prometheus(&self) -> String {
        self.registry.export_prometheus()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper Macros
// =============================================================================

/// Helper macro to create label HashMap
#[macro_export]
macro_rules! labels {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut labels = ::std::collections::HashMap::new();
            $(
                labels.insert($key.to_string(), $value.to_string());
            )*
            labels
        }
    };
}

// =============================================================================
// Legacy Types (for backward compatibility)
// =============================================================================

/// Global agent metrics (legacy)
pub struct AgentMetrics {
    pub tasks_started: Arc<Counter>,
    pub tasks_completed: Arc<Counter>,
    pub tasks_failed: Arc<Counter>,
    pub messages_sent: Arc<Counter>,
    pub messages_received: Arc<Counter>,
    pub active_sessions: Arc<Gauge>,
}

impl AgentMetrics {
    pub fn new(registry: &mut MetricsRegistry) -> Self {
        Self {
            tasks_started: registry.counter("agent_tasks_started"),
            tasks_completed: registry.counter("agent_tasks_completed"),
            tasks_failed: registry.counter("agent_tasks_failed"),
            messages_sent: registry.counter("agent_messages_sent"),
            messages_received: registry.counter("agent_messages_received"),
            active_sessions: registry.gauge("agent_active_sessions"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new();
        counter.inc();
        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new();
        gauge.set(100);
        gauge.inc();
        gauge.dec();
        assert_eq!(gauge.get(), 100);
    }

    #[test]
    fn test_histogram() {
        let hist = Histogram::new();
        hist.observe(50);
        hist.observe(150);
        hist.observe(250);

        assert_eq!(hist.count(), 3);
        assert_eq!(hist.sum(), 450);

        let buckets = hist.buckets();
        assert!(buckets
            .iter()
            .any(|b| b.upper_bound == 100.0 && b.count >= 1));
    }

    #[test]
    fn test_metrics_collector() {
        let metrics = MetricsCollector::new();
        metrics.record_task_started("agent-1", "llm_chat");
        metrics.record_task_completed("agent-1", "llm_chat", 150);

        let output = metrics.export_prometheus();
        assert!(output.contains("agent_tasks_started_total"));
        assert!(output.contains("agent_tasks_completed_total"));
    }

    #[test]
    fn test_labels_macro() {
        let labels = labels!(
            "agent_id" => "test",
            "task_type" => "chat"
        );
        assert_eq!(labels.get("agent_id"), Some(&"test".to_string()));
    }
}
