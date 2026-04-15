//! Metrics types

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Metric label
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
    pub value: String,
}

impl Label {
    /// Create new label
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Metric with name and value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub labels: Vec<Label>,
    pub timestamp: u64,
}

impl Metric {
    /// Create new metric
    pub fn new(name: impl Into<String>, value: MetricValue) -> Self {
        Self {
            name: name.into(),
            value,
            labels: Vec::new(),
            timestamp: now(),
        }
    }

    /// Add label
    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Metric unit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricUnit {
    Bytes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Percent,
    Count,
    /// Custom unit
    Other(String),
}

impl MetricUnit {
    /// Get unit name
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bytes => "bytes",
            Self::Seconds => "s",
            Self::Milliseconds => "ms",
            Self::Microseconds => "us",
            Self::Nanoseconds => "ns",
            Self::Percent => "%",
            Self::Count => "1",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// Metric descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDescriptor {
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub unit: MetricUnit,
    pub labels: Vec<String>,
}

impl MetricDescriptor {
    /// Create new descriptor
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        metric_type: MetricType,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            metric_type,
            unit: MetricUnit::Count,
            labels: Vec::new(),
        }
    }

    /// Set unit
    pub fn with_unit(mut self, unit: MetricUnit) -> Self {
        self.unit = unit;
        self
    }

    /// Add label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<Bucket>),
}

/// Histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Labeled metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledMetric {
    pub descriptor: MetricDescriptor,
    pub labels: HashMap<String, String>,
    pub value: MetricValue,
    pub timestamp: u64,
}

/// Predefined agent metrics
pub struct AgentMetrics;

impl AgentMetrics {
    /// Agent spawn counter
    pub const AGENT_SPAWNS: MetricDescriptor = MetricDescriptor {
        name: String::new(), // Would be &'static str in real impl
        description: String::new(),
        metric_type: MetricType::Counter,
        unit: MetricUnit::Count,
        labels: Vec::new(),
    };

    /// Agent memory usage
    pub const AGENT_MEMORY: MetricDescriptor = MetricDescriptor {
        name: String::new(),
        description: String::new(),
        metric_type: MetricType::Gauge,
        unit: MetricUnit::Bytes,
        labels: Vec::new(),
    };

    /// Task execution duration
    pub const TASK_DURATION: MetricDescriptor = MetricDescriptor {
        name: String::new(),
        description: String::new(),
        metric_type: MetricType::Histogram,
        unit: MetricUnit::Milliseconds,
        labels: Vec::new(),
    };
}

/// Metric filter
#[derive(Debug, Clone, Default)]
pub struct MetricFilter {
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl MetricFilter {
    /// Create new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Include pattern
    pub fn include(mut self, pattern: impl Into<String>) -> Self {
        self.include_patterns.push(pattern.into());
        self
    }

    /// Exclude pattern
    pub fn exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Check if name matches filter
    pub fn matches(&self, name: &str) -> bool {
        // Check exclusions first
        for pattern in &self.exclude_patterns {
            if Self::glob_match(pattern, name) {
                return false;
            }
        }

        // Check inclusions
        if self.include_patterns.is_empty() {
            return true;
        }

        for pattern in &self.include_patterns {
            if Self::glob_match(pattern, name) {
                return true;
            }
        }

        false
    }

    /// Simple glob matching
    fn glob_match(pattern: &str, text: &str) -> bool {
        // Simplified glob matching
        if pattern == "*" {
            return true;
        }
        if let Some(prefix) = pattern.strip_suffix('*') {
            return text.starts_with(prefix);
        }
        if let Some(suffix) = pattern.strip_prefix('*') {
            return text.ends_with(suffix);
        }
        pattern == text
    }
}

/// Aggregation window
#[derive(Debug, Clone, Copy)]
pub struct AggregationWindow {
    pub duration_secs: u64,
    pub step_secs: u64,
}

impl AggregationWindow {
    /// Create 1 minute window
    pub fn one_minute() -> Self {
        Self {
            duration_secs: 60,
            step_secs: 10,
        }
    }

    /// Create 5 minute window
    pub fn five_minutes() -> Self {
        Self {
            duration_secs: 300,
            step_secs: 30,
        }
    }

    /// Create 1 hour window
    pub fn one_hour() -> Self {
        Self {
            duration_secs: 3600,
            step_secs: 60,
        }
    }
}
