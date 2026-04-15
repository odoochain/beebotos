//! Cost Tracking

use std::collections::HashMap;

/// Cost tracker for model usage
#[derive(Debug, Default)]
pub struct CostTracker {
    by_provider: HashMap<String, f64>,
    by_model: HashMap<String, f64>,
    total: f64,
}

impl CostTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, provider: &str, model: &str, cost: f64) {
        *self.by_provider.entry(provider.to_string()).or_default() += cost;
        *self.by_model.entry(model.to_string()).or_default() += cost;
        self.total += cost;
    }

    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn by_provider(&self, provider: &str) -> f64 {
        self.by_provider.get(provider).copied().unwrap_or(0.0)
    }

    pub fn by_model(&self, model: &str) -> f64 {
        self.by_model.get(model).copied().unwrap_or(0.0)
    }

    pub fn report(&self) -> String {
        format!("Total cost: ${:.4}", self.total)
    }
}
