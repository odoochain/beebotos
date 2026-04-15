//! Anomaly Detection

use crate::error::Result;

/// Anomaly detector
pub struct AnomalyDetector {
    threshold: f64,
    baseline: Vec<f64>,
}

impl AnomalyDetector {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            baseline: Vec::new(),
        }
    }

    pub fn record(&mut self, value: f64) {
        self.baseline.push(value);
        if self.baseline.len() > 100 {
            self.baseline.remove(0);
        }
    }

    pub fn detect(&self, value: f64) -> bool {
        if self.baseline.is_empty() {
            return false;
        }
        
        let mean: f64 = self.baseline.iter().sum::<f64>() / self.baseline.len() as f64;
        let diff = (value - mean).abs();
        
        diff > self.threshold
    }
}
