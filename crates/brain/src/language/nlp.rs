//! NLP Module
//!
//! Natural language processing utilities.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// NLP processor
pub struct NlpProcessor;

impl NlpProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Tokenize text into words
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_lowercase()).collect()
    }

    /// Extract keywords
    pub fn extract_keywords(&self, text: &str) -> Vec<String> {
        self.tokenize(text)
            .into_iter()
            .filter(|w| w.len() > 4)
            .take(5)
            .collect()
    }
}

impl Default for NlpProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Named entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedEntity {
    pub text: String,
    pub entity_type: EntityType,
}

/// Entity types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
}
