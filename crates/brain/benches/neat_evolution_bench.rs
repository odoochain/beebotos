//! NEAT Evolution Benchmarks
//!
//! Performance benchmarks for NEAT algorithm operations.

use beebotos_social_brain::neat::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Benchmark genome creation
fn genome_creation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("genome_creation");

    for size in [5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}", size, size)),
            &size,
            |b, &size| {
                b.iter(|| {
                    let genome =
                        Genome::new_minimal(black_box(1), black_box(size), black_box(size));
                    black_box(genome);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark neural network activation
fn network_activation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_activation");

    for (inputs, outputs) in [(5, 3), (10, 5), (20, 10)] {
        let genome = Genome::new_minimal(1, inputs, outputs);
        let mut network = NeuralNetwork::from_genome(&genome);
        let input_vec = vec![0.5; inputs];

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}->{}", inputs, outputs)),
            &input_vec,
            |b, inputs| {
                b.iter(|| {
                    let result = black_box(&mut network).activate(black_box(inputs));
                    black_box(result);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark genome mutation
fn genome_mutation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("genome_mutation");

    let config = NeatConfig::standard();

    group.bench_function("weight_mutation", |b| {
        let mut genome = Genome::new_minimal(1, 10, 5);
        b.iter(|| {
            let mut g = black_box(genome.clone());
            g.mutate_weights(black_box(&config));
            black_box(g);
        })
    });

    group.bench_function("structural_mutation_add_node", |b| {
        let mut genome = Genome::new_minimal(1, 5, 3);
        let mut innovations = InnovationTracker::new();
        b.iter(|| {
            let mut g = black_box(genome.clone());
            let mut i = black_box(innovations.clone());
            g.mutate(black_box(&config), black_box(&mut i));
            black_box(g);
        })
    });

    group.finish();
}

/// Benchmark genome crossover
fn genome_crossover_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("genome_crossover");

    for size in [5, 10, 20] {
        let parent1 = Genome::new_minimal(1, size, size);
        let parent2 = Genome::new_minimal(2, size, size);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}", size, size)),
            &(parent1, parent2),
            |b, (p1, p2)| {
                b.iter(|| {
                    let child = Genome::crossover(black_box(p1), black_box(p2));
                    black_box(child);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark compatibility distance calculation
fn compatibility_distance_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compatibility_distance");

    let config = NeatConfig::standard();
    let genome1 = Genome::new_minimal(1, 10, 5);
    let mut genome2 = Genome::new_minimal(2, 10, 5);

    // Add some differences
    let mut innovations = InnovationTracker::new();
    genome2.mutate(&config, &mut innovations);

    group.bench_function("distance_calculation", |b| {
        b.iter(|| {
            let dist =
                black_box(&genome1).compatibility_distance(black_box(&genome2), black_box(&config));
            black_box(dist);
        })
    });

    group.finish();
}

/// Benchmark population operations
fn population_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("population_operations");

    let config = NeatConfig::standard();

    group.bench_function("population_creation_50", |b| {
        b.iter(|| {
            let pop = Population::new(
                black_box(50),
                black_box(5),
                black_box(3),
                black_box(&config),
            );
            black_box(pop);
        })
    });

    group.bench_function("population_creation_100", |b| {
        b.iter(|| {
            let pop = Population::new(
                black_box(100),
                black_box(5),
                black_box(3),
                black_box(&config),
            );
            black_box(pop);
        })
    });

    group.bench_function("speciation", |b| {
        let mut pop = Population::new(50, 5, 3, &config);
        b.iter(|| {
            let mut p = black_box(pop.clone());
            p.speciate(black_box(&config));
            black_box(p);
        })
    });

    group.bench_function("evolution_step", |b| {
        let mut pop = Population::new(30, 5, 3, &config);
        let fitness_results: Vec<FitnessResult> = pop
            .genomes
            .iter()
            .map(|g| FitnessResult {
                agent_id: beebotos_core::AgentId::new(&format!("agent_{}", g.id)),
                fitness: rand::random::<f32>(),
                generation: 0,
                metrics: std::collections::HashMap::new(),
            })
            .collect();

        b.iter(|| {
            let mut p = black_box(pop.clone());
            p.evolve(black_box(&fitness_results), black_box(&config));
            black_box(p);
        })
    });

    group.finish();
}

/// Benchmark innovation tracking
fn innovation_tracking_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("innovation_tracking");

    group.bench_function("connection_innovation", |b| {
        let mut tracker = InnovationTracker::new();
        let mut counter = 0usize;
        b.iter(|| {
            let innov = tracker
                .get_connection_innovation(black_box(counter % 10), black_box((counter + 1) % 10));
            counter += 1;
            black_box(innov);
        })
    });

    group.bench_function("node_innovation", |b| {
        let mut tracker = InnovationTracker::new();
        let mut counter = 0usize;
        b.iter(|| {
            let innov = tracker.get_node_innovation(black_box(counter));
            counter += 1;
            black_box(innov);
        })
    });

    group.finish();
}

criterion_group!(
    neat_benches,
    genome_creation_benchmark,
    network_activation_benchmark,
    genome_mutation_benchmark,
    genome_crossover_benchmark,
    compatibility_distance_benchmark,
    population_operations_benchmark,
    innovation_tracking_benchmark
);

criterion_main!(neat_benches);
