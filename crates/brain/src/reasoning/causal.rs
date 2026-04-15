//! Causal Reasoning
#![allow(dead_code)]

use std::collections::HashMap;

/// Causal model
#[derive(Debug, Default)]
pub struct CausalModel {
    relationships: HashMap<String, Vec<String>>,
}

impl CausalModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_cause(&mut self, cause: impl Into<String>, effect: impl Into<String>) {
        self.relationships
            .entry(cause.into())
            .or_default()
            .push(effect.into());
    }

    pub fn effects(&self, cause: &str) -> Vec<&String> {
        self.relationships.get(cause).map(|v| v.iter().collect()).unwrap_or_default()
    }
}
