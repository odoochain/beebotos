//! NEAT Evolution Benchmarks

use beebotos_brain::neat::{
    AgentBrain, FitnessResult, Genome, InnovationTracker, NeatConfig, Population,
};
use beebotos_core::AgentId;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Benchmark genome operations
fn bench_genome_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("genome");

    // Benchmark minimal genome creation
    group.bench_function("new_minimal", |b| {
        b.iter(|| Genome::new_minimal(black_box(0), 4, 2));
    });

    // Benchmark genome cloning
    group.bench_function("clone", |b| {
        let genome = Genome::new_minimal(0, 4, 2);
        b.iter(|| {
            black_box(genome.clone());
        });
    });

    // Benchmark weight mutation
    group.bench_function("mutate_weights", |b| {
        let mut genome = Genome::new_minimal(0, 4, 2);
        let config = NeatConfig::standard();
        b.iter(|| {
            genome.mutate_weights(black_box(&config));
        });
    });

    // Benchmark crossover
    group.bench_function("crossover", |b| {
        let parent1 = Genome::new_minimal(0, 4, 2);
        let parent2 = Genome::new_minimal(1, 4, 2);
        b.iter(|| {
            Genome::crossover(black_box(&parent1), black_box(&parent2));
        });
    });

    // Benchmark full mutation with different complexities
    for (nodes, connections) in [(4, 6), (10, 20), (50, 100)].iter() {
        group.bench_with_input(
            BenchmarkId::new("full_mutation", format!("{}n_{}c", nodes, connections)),
            &(*nodes, *connections),
            |b, (nodes, connections)| {
                let mut genome = create_genome_with_complexity(*nodes, *connections);
                let mut innovations = InnovationTracker::new();
                let config = NeatConfig::standard();

                b.iter(|| {
                    genome.mutate(black_box(&config), black_box(&mut innovations));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark neural network operations
fn bench_neural_network(c: &mut Criterion) {
    let mut group = c.benchmark_group("neural_network");

    // Benchmark network creation from genome
    for size in [4, 10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("from_genome", size), size, |b, &size| {
            let genome = create_genome_with_complexity(size, size * 2);
            b.iter(|| {
                beebotos_brain::neat::NeuralNetwork::from_genome(black_box(&genome));
            });
        });
    }

    // Benchmark forward pass
    for (inputs, hidden, outputs) in [(4, 5, 2), (10, 20, 5), (50, 100, 10)].iter() {
        group.bench_with_input(
            BenchmarkId::new(
                "forward_pass",
                format!("{}i_{}h_{}o", inputs, hidden, outputs),
            ),
            &(*inputs, *hidden, *outputs),
            |b, (inputs, hidden, outputs)| {
                let genome =
                    create_genome_with_complexity(*inputs + *hidden + *outputs, *inputs * *hidden);
                let network = beebotos_brain::neat::NeuralNetwork::from_genome(&genome);
                let input_vec = vec![0.5; *inputs];

                b.iter(|| {
                    network.activate(black_box(&input_vec));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark population evolution
fn bench_population_evolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("population");

    // Benchmark population creation
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("new", size), size, |b, &size| {
            let config = NeatConfig::standard();
            b.iter(|| {
                Population::new(black_box(size), 4, 2, black_box(&config));
            });
        });
    }

    // Benchmark speciation
    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("speciate", size), size, |b, &size| {
            let config = NeatConfig::standard();
            let mut population = Population::new(size, 4, 2, &config);

            b.iter(|| {
                population.speciate(black_box(&config));
            });
        });
    }

    // Benchmark evolution step
    for size in [10, 50].iter() {
        group.bench_with_input(BenchmarkId::new("evolve", size), size, |b, &size| {
            let config = NeatConfig::standard();
            let mut population = Population::new(size, 4, 2, &config);

            // Create dummy fitness results
            let fitness_results: Vec<FitnessResult> = (0..size)
                .map(|i| FitnessResult {
                    agent_id: AgentId::new(format!("agent_{}", i)),
                    fitness: rand::random::<f32>(),
                    generation: 0,
                    metrics: std::collections::HashMap::new(),
                })
                .collect();

            b.iter(|| {
                population.evolve(black_box(&fitness_results), black_box(&config));
            });
        });
    }

    group.finish();
}

/// Benchmark agent brain operations
fn bench_agent_brain(c: &mut Criterion) {
    let mut group = c.benchmark_group("agent_brain");

    // Benchmark agent brain creation
    group.bench_function("from_genome", |b| {
        let genome = Genome::new_minimal(0, 4, 2);
        b.iter(|| {
            AgentBrain::from_genome(black_box(genome.clone()));
        });
    });

    // Benchmark thinking (forward pass + state update)
    for (inputs, hidden, outputs) in [(4, 5, 2), (10, 20, 5)].iter() {
        group.bench_with_input(
            BenchmarkId::new("think", format!("{}i_{}h_{}o", inputs, hidden, outputs)),
            &(*inputs, *hidden, *outputs),
            |b, (inputs, hidden, outputs)| {
                let genome =
                    create_genome_with_complexity(*inputs + *hidden + *outputs, *inputs * *hidden);
                let mut brain = AgentBrain::from_genome(genome);
                let input_vec = vec![0.5; *inputs];

                b.iter(|| {
                    brain.think(black_box(&input_vec));
                });
            },
        );
    }

    // Benchmark fitness update
    group.bench_function("update_fitness", |b| {
        let genome = Genome::new_minimal(0, 4, 2);
        let mut brain = AgentBrain::from_genome(genome);
        let mut delta = 0.0;

        b.iter(|| {
            brain.update_fitness(black_box(delta));
            delta += 0.1;
        });
    });

    group.finish();
}

/// Benchmark innovation tracking
fn bench_innovation_tracker(c: &mut Criterion) {
    let mut group = c.benchmark_group("innovation");

    // Benchmark connection innovation tracking
    group.bench_function("connection_innovation", |b| {
        let mut tracker = InnovationTracker::new();
        let mut from = 0;
        let mut to = 1;

        b.iter(|| {
            tracker.get_connection_innovation(black_box(from), black_box(to));
            from += 1;
            to += 1;
        });
    });

    // Benchmark node innovation tracking
    group.bench_function("node_innovation", |b| {
        let mut tracker = InnovationTracker::new();
        let mut conn = 0;

        b.iter(|| {
            tracker.get_node_innovation(black_box(conn));
            conn += 1;
        });
    });

    group.finish();
}

/// Helper function to create genomes with specific complexity
fn create_genome_with_complexity(nodes: usize, connections: usize) -> Genome {
    let mut genome = Genome::new_minimal(0, 4, 2);

    // Add connections to reach desired complexity
    // This is a simplified version - real implementation would
    // add proper connections between nodes
    for _ in 0..connections {
        // In real implementation, add actual connections
    }

    genome
}

criterion_group!(
    neat_benches,
    bench_genome_operations,
    bench_neural_network,
    bench_population_evolution,
    bench_agent_brain,
    bench_innovation_tracker
);
criterion_main!(neat_benches);
