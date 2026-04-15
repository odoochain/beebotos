//! BeeBotOS Metrics Collection and Export
//!
//! This module provides metrics collection, aggregation, and export
//! capabilities for the BeeBotOS platform.

pub mod collectors;
pub mod exporters;
pub mod registry;
pub mod types;

use std::sync::Arc;

pub use collectors::{Counter, Gauge, Histogram, Meter};
pub use exporters::{OpenTelemetryExporter, PrometheusExporter};
pub use registry::MetricsRegistry;
use thiserror::Error;
pub use types::{Label, Metric, MetricType, MetricValue};

/// Metrics collection error types
#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("metric not found: {0}")]
    NotFound(String),
    #[error("invalid metric value: {0}")]
    InvalidValue(String),
    #[error("exporter error: {0}")]
    ExporterError(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for metrics operations
pub type Result<T> = std::result::Result<T, MetricsError>;

/// Main metrics system configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Export interval in seconds
    pub export_interval_secs: u64,
    /// Enable Prometheus endpoint
    pub enable_prometheus: bool,
    /// Prometheus bind address
    pub prometheus_bind: String,
    /// Enable OpenTelemetry export
    pub enable_otel: bool,
    /// OpenTelemetry endpoint
    pub otel_endpoint: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            export_interval_secs: 15,
            enable_prometheus: true,
            prometheus_bind: "0.0.0.0:9090".to_string(),
            enable_otel: false,
            otel_endpoint: "http://localhost:4317".to_string(),
        }
    }
}

/// Main metrics system handle
pub struct MetricsSystem {
    registry: Arc<MetricsRegistry>,
    #[allow(dead_code)]
    config: MetricsConfig,
}

impl MetricsSystem {
    /// Create a new metrics system
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            registry: Arc::new(MetricsRegistry::new()),
            config,
        }
    }

    /// Get the metrics registry
    pub fn registry(&self) -> Arc<MetricsRegistry> {
        self.registry.clone()
    }

    /// Start metrics collection and export
    pub async fn start(&self) -> Result<()> {
        // Implementation would start background tasks for export
        Ok(())
    }

    /// Stop the metrics system
    pub async fn stop(&self) -> Result<()> {
        // Implementation would stop background tasks
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert_eq!(config.export_interval_secs, 15);
        assert!(config.enable_prometheus);
        assert!(!config.enable_otel);
    }

    #[test]
    fn test_metrics_system_creation() {
        let config = MetricsConfig::default();
        let system = MetricsSystem::new(config);
        let registry = system.registry();
        assert!(Arc::strong_count(&registry) == 2);
    }
}
