//! Metrics Exporters

use super::Result;

/// Prometheus exporter
pub struct PrometheusExporter {
    endpoint: String,
}

impl PrometheusExporter {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    pub async fn export(&self, _data: &str) -> Result<()> {
        tracing::info!("Exporting to Prometheus at {}", self.endpoint);
        Ok(())
    }
}

/// OpenTelemetry exporter
pub struct OpenTelemetryExporter {
    endpoint: String,
}

impl OpenTelemetryExporter {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    pub async fn export(&self, _data: &str) -> Result<()> {
        tracing::info!("Exporting to OpenTelemetry at {}", self.endpoint);
        Ok(())
    }
}
