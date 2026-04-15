//! NEAT Benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use beebot_social_brain::neat::*;

fn bench_genome_creation(c: &mut Criterion) {
    c.bench_function("genome_new_minimal_10x5", |b| {
        let mut innovations = InnovationTracker::new();
        
        b.iter(|| {
            black_box(Genome::new_minimal(10, 5, &mut innovations));
        });
    });
}

fn bench_network_activation(c: &mut Criterion) {
    let mut innovations = InnovationTracker::new();
    let genome = Genome::new_minimal(10, 5, &mut innovations);
    let mut network = NeuralNetwork::from_genome(&genome);
    let inputs = vec![0.5; 10];
    
    c.bench_function("network_activate_10x5", |b| {
        b.iter(|| {
            black_box(network.activate(&inputs));
        });
    });
}

fn bench_compatibility_distance(c: &mut Criterion) {
    let config = NeatConfig::standard();
    let mut innovations = InnovationTracker::new();
    let genome1 = Genome::new_minimal(5, 3, &mut innovations);
    let genome2 = Genome::new_minimal(5, 3, &mut innovations);
    
    c.bench_function("compatibility_distance", |b| {
        b.iter(|| {
            black_box(genome1.compatibility_distance(&genome2, &config));
        });
    });
}

fn bench_crossover(c: &mut Criterion) {
    let mut innovations = InnovationTracker::new();
    let mut genome1 = Genome::new_minimal(5, 3, &mut innovations);
    let mut genome2 = Genome::new_minimal(5, 3, &mut innovations);
    genome1.fitness = 10.0;
    genome2.fitness = 5.0;
    
    c.bench_function("genome_crossover", |b| {
        b.iter(|| {
            black_box(Genome::crossover(&genome1, &genome2));
        });
    });
}

criterion_group!(
    benches,
    bench_genome_creation,
    bench_network_activation,
    bench_compatibility_distance,
    bench_crossover
);
criterion_main!(benches);
