//! NEAT (NeuroEvolution of Augmenting Topologies) Tests
//!
//! Comprehensive tests for the NEAT algorithm implementation.

use beebotos_brain::{
    AgentBrain, FitnessResult, Genome, InnovationTracker, NeatConfig, NeuralNetwork, Population,
};

/// Test genome creation with minimal structure
#[test]
fn test_genome_creation() {
    let genome = Genome::new_minimal(1, 3, 2);

    assert_eq!(genome.id, 1);
    assert_eq!(genome.layers.len(), 2); // Input + Output
    assert_eq!(genome.node_count(), 5); // 3 input + 2 output
    assert!(!genome.connections.is_empty());
}

/// Test genome crossover between two parents
#[test]
fn test_genome_crossover() {
    let config = NeatConfig::standard();
    let parent1 = Genome::new_minimal(1, 2, 1);
    let mut parent2 = Genome::new_minimal(2, 2, 1);

    // Give parent1 higher fitness
    parent2.fitness = 0.5;

    let child = Genome::crossover(&parent1, &parent2);

    assert_eq!(child.layers.len(), parent1.layers.len());
    assert!(child.fitness == 0.0); // Child has no fitness yet
    assert_ne!(child.id, parent1.id);
    assert_ne!(child.id, parent2.id);
}

/// Test weight mutation
#[test]
fn test_genome_weight_mutation() {
    let config = NeatConfig::standard();
    let mut genome = Genome::new_minimal(1, 2, 1);

    let original_weights: Vec<f32> = genome.connections.iter().map(|c| c.weight).collect();

    genome.mutate_weights(&config);

    // Check that some weights changed
    let mutated = genome
        .connections
        .iter()
        .zip(original_weights.iter())
        .any(|(c, orig)| (c.weight - orig).abs() > 0.001);

    assert!(mutated, "Some weights should have mutated");
}

/// Test structural mutation - add node
#[test]
fn test_structural_mutation_add_node() {
    let config = NeatConfig {
        add_node_probability: 1.0, // Force mutation
        ..NeatConfig::standard()
    };

    let mut genome = Genome::new_minimal(1, 2, 1);
    let original_connections = genome.connections.len();

    let mut innovations = InnovationTracker::new();
    genome.mutate(&config, &mut innovations);

    // Adding a node should add 2 connections and disable 1
    assert!(
        genome.connections.len() >= original_connections + 1,
        "Structural mutation should add connections"
    );
}

/// Test structural mutation - add connection
#[test]
fn test_structural_mutation_add_connection() {
    let config = NeatConfig {
        add_connection_probability: 1.0, // Force mutation
        ..NeatConfig::standard()
    };

    let mut genome = Genome::new_minimal(1, 3, 2);
    let original_count = genome.connections.len();

    let mut innovations = InnovationTracker::new();
    genome.mutate(&config, &mut innovations);

    assert!(
        genome.connections.len() >= original_count,
        "Should have same or more connections"
    );
}

/// Test neural network activation
#[test]
fn test_neural_network_activation() {
    let genome = Genome::new_minimal(1, 3, 2);
    let mut network = NeuralNetwork::from_genome(&genome);

    let inputs = vec![0.5, 0.3, 0.2];
    let outputs = network.activate(&inputs);

    assert_eq!(outputs.len(), 2);
    // Outputs should be between 0 and 1 (sigmoid)
    assert!(outputs.iter().all(|&o| o >= 0.0 && o <= 1.0));
}

/// Test neural network forward pass matches predict
#[test]
fn test_network_forward_predict_consistency() {
    let genome = Genome::new_minimal(1, 2, 1);
    let mut network = NeuralNetwork::from_genome(&genome);

    let inputs = vec![0.5, 0.5];
    let outputs1 = network.activate(&inputs);
    let outputs2 = network.predict(&inputs);

    assert_eq!(outputs1, outputs2);
}

/// Test population creation
#[test]
fn test_population_creation() {
    let config = NeatConfig::standard();
    let population = Population::new(50, 3, 2, &config);

    assert_eq!(population.size, 50);
    assert_eq!(population.genomes.len(), 50);
    assert_eq!(population.generation, 0);
}

/// Test speciation
#[test]
fn test_population_speciation() {
    let config = NeatConfig::standard();
    let mut population = Population::new(50, 3, 2, &config);

    population.speciate(&config);

    assert!(
        !population.species.is_empty(),
        "Should have at least one species"
    );

    // All genomes should be assigned to a species
    let assigned_count: usize = population.species.iter().map(|s| s.members.len()).sum();

    assert_eq!(assigned_count, 50);
}

/// Test population evolution
#[test]
fn test_population_evolution() {
    let config = NeatConfig::standard();
    let mut population = Population::new(30, 2, 1, &config);

    // Create fitness results
    let fitness_results: Vec<FitnessResult> = population
        .genomes
        .iter()
        .enumerate()
        .map(|(i, g)| FitnessResult {
            agent_id: beebotos_core::AgentId::new(),
            fitness: 0.5 + (i as f32 * 0.01), // Varying fitness
            generation: 0,
            metrics: std::collections::HashMap::new(),
        })
        .collect();

    let initial_best = population.best_fitness;

    population.evolve(&fitness_results, &config);

    assert_eq!(population.generation, 1);
    // Best fitness should be tracked
    assert!(population.best_fitness >= initial_best);
}

/// Test population statistics
#[test]
fn test_population_stats() {
    let config = NeatConfig::standard();
    let mut population = Population::new(50, 3, 2, &config);

    // Assign some fitness values
    for (i, genome) in population.genomes.iter_mut().enumerate() {
        genome.fitness = i as f32 * 0.1;
    }

    let stats = population.stats();

    assert_eq!(stats.population_size, 50);
    assert!(stats.avg_fitness > 0.0);
    assert!(stats.max_fitness >= stats.avg_fitness);
    assert!(stats.min_fitness <= stats.avg_fitness);
}

/// Test AgentBrain
#[test]
fn test_agent_brain() {
    let genome = Genome::new_minimal(1, 2, 1);
    let mut brain = AgentBrain::from_genome(genome);

    let inputs = vec![0.5, 0.5];
    let outputs = brain.think(&inputs);

    assert_eq!(outputs.len(), 1);

    brain.update_fitness(1.0);
    assert_eq!(brain.fitness, 1.0);
}

/// Test innovation tracker
#[test]
fn test_innovation_tracker() {
    let mut tracker = InnovationTracker::new();

    let innov1 = tracker.get_connection_innovation(0, 1);
    let innov2 = tracker.get_connection_innovation(0, 1);
    let innov3 = tracker.get_connection_innovation(1, 2);

    // Same connection should get same innovation number
    assert_eq!(innov1, innov2);
    // Different connection should get different number
    assert_ne!(innov1, innov3);

    // Node innovation should be different
    let node_innov = tracker.get_node_innovation(innov1);
    assert!(node_innov > innov1);
}

/// Test genome compatibility distance
#[test]
fn test_compatibility_distance() {
    let config = NeatConfig::standard();
    let genome1 = Genome::new_minimal(1, 2, 1);
    let mut genome2 = Genome::new_minimal(2, 2, 1);

    // Mutate genome2 to create differences
    genome2.mutate_weights(&config);

    let distance = genome1.compatibility_distance(&genome2, &config);

    assert!(distance >= 0.0);
    // Same structure should have relatively small distance
    assert!(distance < config.compatibility_threshold * 2.0);
}

/// Test NEAT configuration presets
#[test]
fn test_neat_config_presets() {
    let standard = NeatConfig::standard();
    let conservative = NeatConfig::conservative();
    let aggressive = NeatConfig::aggressive();

    // Conservative should have lower mutation rates
    assert!(conservative.mutation_rate < standard.mutation_rate);

    // Aggressive should have higher mutation rates
    assert!(aggressive.mutation_rate > standard.mutation_rate);

    // Conservative should have larger population
    assert!(conservative.population_size > aggressive.population_size);
}
