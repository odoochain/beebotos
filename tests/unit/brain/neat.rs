//! NEAT Unit Tests
//!
//! Tests for NeuroEvolution of Augmenting Topologies.

use social_brain::neat::*;

#[test]
fn test_genome_creation() {
    let mut genome = Genome::new();
    
    // Add inputs and outputs
    let input1 = genome.add_node(NodeType::Input);
    let input2 = genome.add_node(NodeType::Input);
    let output = genome.add_node(NodeType::Output);
    
    assert_eq!(genome.node_genes.len(), 3);
    
    // Add connection
    let conn = genome.add_connection(input1, output, 0.5, true);
    assert!(conn.is_some());
    
    // Try duplicate connection (should fail)
    let duplicate = genome.add_connection(input1, output, 0.3, true);
    assert!(duplicate.is_none());
}

#[test]
fn test_innovation_tracker() {
    let mut tracker = InnovationTracker::new();
    
    // First innovation
    let innov1 = tracker.get_connection_innovation(0, 1);
    assert_eq!(innov1, 0);
    
    // Same connection should return same innovation
    let innov2 = tracker.get_connection_innovation(0, 1);
    assert_eq!(innov2, innov1);
    
    // Different connection should get new innovation
    let innov3 = tracker.get_connection_innovation(1, 2);
    assert_eq!(innov3, 1);
}

#[test]
fn test_compatibility_distance() {
    let mut genome1 = Genome::new();
    let n1 = genome1.add_node(NodeType::Input);
    let n2 = genome1.add_node(NodeType::Output);
    genome1.add_connection(n1, n2, 1.0, true);
    
    let mut genome2 = Genome::new();
    let n3 = genome2.add_node(NodeType::Input);
    let n4 = genome2.add_node(NodeType::Output);
    genome2.add_connection(n3, n4, 1.5, true);
    
    let distance = genome1.compatibility_distance(&genome2, 1.0, 1.0, 0.4);
    assert!(distance >= 0.0);
}

#[test]
fn test_crossover() {
    let mut parent1 = Genome::new();
    let n1 = parent1.add_node(NodeType::Input);
    let n2 = parent1.add_node(NodeType::Output);
    parent1.add_connection(n1, n2, 1.0, true);
    parent1.fitness = Some(10.0);
    
    let mut parent2 = Genome::new();
    let n3 = parent2.add_node(NodeType::Input);
    let n4 = parent2.add_node(NodeType::Output);
    parent2.add_connection(n3, n4, 0.5, true);
    parent2.fitness = Some(5.0);
    
    let child = parent1.crossover(&parent2);
    
    // Child should have genes from both parents
    assert!(!child.connection_genes.is_empty());
}

#[test]
fn test_mutation_add_connection() {
    let mut genome = Genome::new();
    let input = genome.add_node(NodeType::Input);
    let output = genome.add_node(NodeType::Output);
    
    // Should be able to add connection
    let added = genome.mutate_add_connection(&mut InnovationTracker::new());
    assert!(added);
    
    assert_eq!(genome.connection_genes.len(), 1);
}

#[test]
fn test_mutation_add_node() {
    let mut genome = Genome::new();
    let input = genome.add_node(NodeType::Input);
    let output = genome.add_node(NodeType::Output);
    genome.add_connection(input, output, 1.0, true);
    
    let mut tracker = InnovationTracker::new();
    let added = genome.mutate_add_node(&mut tracker);
    assert!(added);
    
    // Should have 3 nodes now (input, hidden, output)
    assert_eq!(genome.node_genes.len(), 3);
    
    // Should have 2 connections (old one disabled + 2 new)
    assert_eq!(genome.connection_genes.len(), 3);
}

#[test]
fn test_neural_network_activation() {
    // Create simple network: input -> output
    let mut genome = Genome::new();
    let input = genome.add_node(NodeType::Input);
    let output = genome.add_node(NodeType::Output);
    genome.add_connection(input, output, 1.0, true);
    
    let network = NeuralNetwork::from_genome(&genome);
    
    // Activate with input
    let inputs = vec![1.0];
    let outputs = network.activate(&inputs);
    
    assert_eq!(outputs.len(), 1);
    // Sigmoid(1.0 * 1.0) ≈ 0.731
    assert!(outputs[0] > 0.7 && outputs[0] < 0.8);
}

#[test]
fn test_species_assignment() {
    let mut population = Population::new(10);
    let config = NeatConfig::default();
    
    // Initialize population
    for _ in 0..10 {
        let mut genome = Genome::new();
        // Add minimal structure
        let input = genome.add_node(NodeType::Input);
        let output = genome.add_node(NodeType::Output);
        genome.add_connection(input, output, 1.0, true);
        population.add_genome(genome);
    }
    
    // Speciate
    population.speciate(&config);
    
    // Should have at least one species
    assert!(!population.species.is_empty());
}
