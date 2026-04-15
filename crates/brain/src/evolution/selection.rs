//! Selection algorithms for NEAT

use super::genome::Genome;
use rand::Rng;

/// Selection strategy trait
pub trait SelectionStrategy {
    fn select<'a>(&self, population: &'a [Genome]) -> &'a Genome;
}

/// Tournament selection
pub struct TournamentSelection {
    tournament_size: usize,
}

impl TournamentSelection {
    pub fn new(tournament_size: usize) -> Self {
        Self { tournament_size }
    }
}

impl SelectionStrategy for TournamentSelection {
    fn select<'a>(&self, population: &'a [Genome]) -> &'a Genome {
        let mut rng = rand::thread_rng();
        let mut best = &population[rng.gen_range(0..population.len())];
        
        for _ in 1..self.tournament_size {
            let contender = &population[rng.gen_range(0..population.len())];
            if contender.fitness > best.fitness {
                best = contender;
            }
        }
        
        best
    }
}

/// Roulette wheel selection (fitness proportionate)
pub struct RouletteSelection;

impl SelectionStrategy for RouletteSelection {
    fn select<'a>(&self, population: &'a [Genome]) -> &'a Genome {
        let total_fitness: f64 = population.iter().map(|g| g.fitness.max(0.0)).sum();
        let mut rng = rand::thread_rng();
        let threshold = rng.gen::<f64>() * total_fitness;
        
        let mut accumulated = 0.0;
        for genome in population {
            accumulated += genome.fitness.max(0.0);
            if accumulated >= threshold {
                return genome;
            }
        }
        
        &population[population.len() - 1]
    }
}

/// Rank-based selection
pub struct RankSelection;

impl SelectionStrategy for RankSelection {
    fn select<'a>(&self, population: &'a [Genome]) -> &'a Genome {
        let mut indexed: Vec<(usize, &Genome)> = population.iter().enumerate().collect();
        indexed.sort_by(|a, b| crate::utils::compare_f32(&a.1.fitness, &b.1.fitness).reverse());
        
        let total_ranks: usize = (1..=indexed.len()).sum();
        let mut rng = rand::thread_rng();
        let threshold = rng.gen::<f64>() * total_ranks as f64;
        
        let mut accumulated = 0.0;
        for (rank, (_, genome)) in indexed.iter().enumerate() {
            accumulated += (indexed.len() - rank) as f64;
            if accumulated >= threshold {
                return genome;
            }
        }
        
        indexed[indexed.len() - 1].1
    }
}
