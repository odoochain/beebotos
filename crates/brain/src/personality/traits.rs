//! Personality Traits

use serde::{Deserialize, Serialize};

/// Individual trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    pub name: String,
    pub value: f64,
    pub heritability: f64,
}

/// Trait collection
#[derive(Debug, Default)]
pub struct TraitCollection {
    traits: Vec<Trait>,
}

impl TraitCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, trait_item: Trait) {
        self.traits.push(trait_item);
    }
}
