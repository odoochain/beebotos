//! Counterfactual Reasoning
#![allow(dead_code)]

/// Counterfactual scenario
#[derive(Debug)]
pub struct Counterfactual {
    pub premise: String,
    pub alternative: String,
    pub consequences: Vec<String>,
}

/// Counterfactual reasoner
#[derive(Debug, Default)]
pub struct CounterfactualReasoner;

impl CounterfactualReasoner {
    pub fn new() -> Self {
        Self
    }

    pub fn what_if(&self, actual: &str, alternative: &str) -> Counterfactual {
        Counterfactual {
            premise: actual.to_string(),
            alternative: alternative.to_string(),
            consequences: vec!["Different outcome".to_string()],
        }
    }

    pub fn regret_analysis(&self, choices: &[String], outcome: &str) -> Vec<String> {
        // Analyze what would have been better
        choices.iter()
            .filter(|c| !outcome.contains(*c))
            .cloned()
            .collect()
    }
}
