//! Fitness evaluation utilities

use super::network::Network;

/// Fitness function trait
pub trait FitnessFunction: Send + Sync {
    fn evaluate(&self, network: &Network) -> f64;
}

/// XOR fitness function (classic NEAT benchmark)
pub struct XORFitness;

impl FitnessFunction for XORFitness {
    fn evaluate(&self, network: &Network) -> f64 {
        let inputs = vec![
            vec![0.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 0.0],
            vec![1.0, 1.0],
        ];
        let expected = vec![0.0, 1.0, 1.0, 0.0];
        
        let mut error = 0.0;
        
        for (input, exp) in inputs.iter().zip(expected.iter()) {
            let output = network.activate(input);
            let diff = output[0] - exp;
            error += diff * diff;
        }
        
        // Fitness is inverse of error
        let fitness = 4.0 - error;
        fitness.max(0.0)
    }
}

/// Multi-objective fitness
pub struct MultiObjectiveFitness {
    objectives: Vec<Box<dyn FitnessFunction>>,
    weights: Vec<f64>,
}

impl MultiObjectiveFitness {
    pub fn new(objectives: Vec<Box<dyn FitnessFunction>>, weights: Vec<f64>) -> Self {
        Self { objectives, weights }
    }
    
    pub fn evaluate_weighted(&self, network: &Network) -> f64 {
        let mut total = 0.0;
        let mut total_weight = 0.0;
        
        for (obj, weight) in self.objectives.iter().zip(self.weights.iter()) {
            total += obj.evaluate(network) * weight;
            total_weight += weight;
        }
        
        if total_weight > 0.0 {
            total / total_weight
        } else {
            0.0
        }
    }
}

/// Novelty search fitness
pub struct NoveltyFitness {
    archive: Vec<Vec<f64>>,
    k: usize,
    threshold: f64,
}

impl NoveltyFitness {
    pub fn new(k: usize, threshold: f64) -> Self {
        Self {
            archive: Vec::new(),
            k,
            threshold,
        }
    }
    
    pub fn calculate_novelty(&self, behavior: &[f64]) -> f64 {
        if self.archive.is_empty() {
            return 1.0;
        }
        
        // Calculate distances to all archived behaviors
        let mut distances: Vec<f64> = self.archive.iter()
            .map(|archived| Self::distance(behavior, archived))
            .collect();
        
        // Sort and take k nearest
        distances.sort_by(|a, b| crate::utils::compare_f32(a, b));
        
        let k_nearest: f64 = distances.iter().take(self.k).sum();
        k_nearest / self.k as f64
    }
    
    pub fn add_to_archive(&mut self, behavior: Vec<f64>) {
        self.archive.push(behavior);
    }
    
    fn distance(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}
