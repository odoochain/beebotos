//! Species management for NEAT

use serde::{Deserialize, Serialize};

use super::{Genome, NeatConfig};

/// Species - group of compatible genomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    pub id: usize,
    pub representative: Genome,
    pub members: Vec<Genome>,
    pub age: usize,
    pub stagnation: usize,
    pub offspring_count: usize,
    pub best_fitness: f32,
    pub avg_fitness: f32,
}

impl Species {
    /// Create new species
    pub fn new(id: usize, representative: Genome) -> Self {
        Self {
            id,
            representative,
            members: Vec::new(),
            age: 0,
            stagnation: 0,
            offspring_count: 0,
            best_fitness: f32::NEG_INFINITY,
            avg_fitness: 0.0,
        }
    }

    /// Check if genome is compatible with this species
    pub fn is_compatible(&self, genome: &Genome, config: &NeatConfig) -> bool {
        let distance = self.representative.compatibility_distance(genome, config);
        distance < config.compatibility_threshold
    }

    /// Update representative (usually best or random member)
    pub fn update_representative(&mut self) {
        if self.members.is_empty() {
            return;
        }

        // Update to best member
        if let Some(best) = self
            .members
            .iter()
            .max_by(|a, b| crate::utils::compare_f32(&a.fitness, &b.fitness))
        {
            if best.fitness > self.best_fitness {
                self.best_fitness = best.fitness;
                self.representative = best.clone();
                self.stagnation = 0;
            } else {
                self.stagnation += 1;
            }
        }

        // Calculate average fitness
        self.avg_fitness =
            self.members.iter().map(|m| m.fitness).sum::<f32>() / self.members.len() as f32;
    }

    /// Age the species
    pub fn age(&mut self) {
        self.age += 1;
    }

    /// Check if species is stagnant
    pub fn is_stagnant(&self, max_stagnation: usize) -> bool {
        self.stagnation >= max_stagnation
    }

    /// Get fittest member
    pub fn fittest(&self) -> Option<&Genome> {
        self.members
            .iter()
            .max_by(|a, b| crate::utils::compare_f32(&a.fitness, &b.fitness))
    }

    /// Calculate adjusted fitness sum
    pub fn adjusted_fitness_sum(&self) -> f32 {
        self.members.iter().map(|m| m.adjusted_fitness).sum()
    }

    /// Get size
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Cull to top fraction
    pub fn cull(&mut self, survival_rate: f64) {
        let keep_count = (self.members.len() as f64 * survival_rate).ceil() as usize;

        // Sort by fitness descending
        self.members
            .sort_by(|a, b| crate::utils::compare_f32(&b.fitness, &a.fitness));

        // Keep only top members
        self.members.truncate(keep_count.max(1));
    }
}

/// Species statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesStats {
    pub id: usize,
    pub size: usize,
    pub age: usize,
    pub stagnation: usize,
    pub best_fitness: f32,
    pub avg_fitness: f32,
    pub offspring_count: usize,
}

impl From<&Species> for SpeciesStats {
    fn from(s: &Species) -> Self {
        Self {
            id: s.id,
            size: s.size(),
            age: s.age,
            stagnation: s.stagnation,
            best_fitness: s.best_fitness,
            avg_fitness: s.avg_fitness,
            offspring_count: s.offspring_count,
        }
    }
}
