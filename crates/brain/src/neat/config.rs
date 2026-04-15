//! NEAT Configuration
//!
//! Hyperparameters for NEAT algorithm.

use serde::{Deserialize, Serialize};

/// NEAT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeatConfig {
    pub population_size: usize,
    pub mutation_rate: f32,
    pub structural_mutation_rate: f32,
    pub crossover_rate: f32,
    pub compatibility_threshold: f32,
    pub elitism: usize,
    pub min_species_size: usize,
    pub max_stagnation: usize,
    pub weight_perturbation: f32,
    pub add_node_probability: f32,
    pub add_connection_probability: f32,
    pub excess_coefficient: f32,
    pub disjoint_coefficient: f32,
    pub weight_coefficient: f32,
    pub weight_mutation_rate: f32,
}

impl NeatConfig {
    /// Standard configuration
    pub fn standard() -> Self {
        Self {
            population_size: 50,
            mutation_rate: 0.1,
            structural_mutation_rate: 0.05,
            crossover_rate: 0.75,
            compatibility_threshold: 3.0,
            elitism: 2,
            min_species_size: 5,
            max_stagnation: 15,
            weight_perturbation: 0.1,
            add_node_probability: 0.03,
            add_connection_probability: 0.05,
            excess_coefficient: 1.0,
            disjoint_coefficient: 1.0,
            weight_coefficient: 0.4,
            weight_mutation_rate: 0.8,
        }
    }

    /// Conservative configuration (slower evolution)
    pub fn conservative() -> Self {
        Self {
            population_size: 100,
            mutation_rate: 0.05,
            structural_mutation_rate: 0.02,
            crossover_rate: 0.8,
            compatibility_threshold: 4.0,
            elitism: 5,
            min_species_size: 10,
            max_stagnation: 20,
            weight_perturbation: 0.05,
            add_node_probability: 0.01,
            add_connection_probability: 0.02,
            excess_coefficient: 1.0,
            disjoint_coefficient: 1.0,
            weight_coefficient: 0.4,
            weight_mutation_rate: 0.8,
        }
    }

    /// Aggressive configuration (faster evolution)
    pub fn aggressive() -> Self {
        Self {
            population_size: 30,
            mutation_rate: 0.2,
            structural_mutation_rate: 0.1,
            crossover_rate: 0.6,
            compatibility_threshold: 2.0,
            elitism: 1,
            min_species_size: 3,
            max_stagnation: 10,
            weight_perturbation: 0.2,
            add_node_probability: 0.05,
            add_connection_probability: 0.1,
            excess_coefficient: 1.0,
            disjoint_coefficient: 1.0,
            weight_coefficient: 0.4,
            weight_mutation_rate: 0.8,
        }
    }
}

impl Default for NeatConfig {
    fn default() -> Self {
        Self::standard()
    }
}
