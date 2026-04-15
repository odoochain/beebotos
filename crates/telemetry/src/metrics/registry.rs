//! Metrics Registry

use std::collections::HashMap;

use super::collectors::{Counter, Gauge};

/// Metrics registry
pub struct MetricsRegistry {
    counters: HashMap<String, Counter>,
    gauges: HashMap<String, Gauge>,
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
        }
    }

    pub fn counter(&mut self, name: impl Into<String>) -> &Counter {
        self.counters.entry(name.into()).or_default()
    }

    pub fn gauge(&mut self, name: impl Into<String>) -> &Gauge {
        self.gauges.entry(name.into()).or_default()
    }
}
