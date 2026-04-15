//! NEAT evolutionary engine

use super::{
    genome::Genome,
    network::Network,
    species::Species,
    crossover::Crossover,
    mutation::Mutation,
};

pub struct EvolutionEngine {
    population: Vec<Genome>,
    species: Vec<Species>,
    generation: u32,
    config: EvolutionConfig,
    innovation_counter: u64,
}

#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    pub population_size: usize,
    pub compatibility_threshold: f64,
    pub survival_rate: f64,
    pub elite_percentage: f64,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            population_size: 150,
            compatibility_threshold: 3.0,
            survival_rate: 0.2,
            elite_percentage: 0.05,
        }
    }
}

impl EvolutionEngine {
    pub fn new(config: EvolutionConfig) -> Self {
        Self {
            population: Vec::new(),
            species: Vec::new(),
            generation: 0,
            config,
            innovation_counter: 0,
        }
    }

    pub fn initialize(&mut self, inputs: usize, outputs: usize) {
        for _ in 0..self.config.population_size {
            let genome = Genome::new(inputs, outputs);
            self.population.push(genome);
        }
    }

    pub fn evolve_generation<F>(&mut self, fitness_fn: F) -> (Genome, f64)
    where
        F: Fn(&Network) -> f64,
    {
        self.evaluate_fitness(&fitness_fn);
        self.speciate();
        self.create_next_generation();
        self.generation += 1;
        self.get_best()
    }

    fn evaluate_fitness<F>(&mut self, fitness_fn: &F)
    where
        F: Fn(&Network) -> f64,
    {
        for genome in &mut self.population {
            let network = Network::from_genome(genome);
            genome.fitness = fitness_fn(&network);
        }
    }

    fn speciate(&mut self) {
        // Simplified speciation
        self.species.clear();
        let mut new_species = Species::new(0);
        if let Some(first) = self.population.first() {
            new_species.representative = first.clone();
            new_species.members = self.population.clone();
            self.species.push(new_species);
        }
    }

    fn create_next_generation(&mut self) {
        // Elitism + mutation
        let mut new_population: Vec<Genome> = Vec::new();
        
        if let Some(best) = self.population.iter().max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap()) {
            new_population.push(best.clone());
        }
        
        while new_population.len() < self.config.population_size {
            if let Some(parent) = self.population.iter().max_by(|a, b| crate::utils::compare_f32(&a.fitness, &b.fitness)) {
                let mut child = parent.clone();
                Mutation::mutate_genome(&mut child);
                new_population.push(child);
            }
        }
        
        self.population = new_population;
    }

    fn get_best(&self) -> (Genome, f64) {
        self.population.iter()
            .max_by(|a, b| crate::utils::compare_f32(&a.fitness, &b.fitness))
            .map(|g| (g.clone(), g.fitness))
            .unwrap_or_else(|| (Genome::new(0, 0), 0.0))
    }

    pub fn generation(&self) -> u32 { self.generation }
    pub fn average_fitness(&self) -> f64 {
        if self.population.is_empty() { 0.0 } 
        else { self.population.iter().map(|g| g.fitness).sum::<f64>() / self.population.len() as f64 }
    }
}
