use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use beebotos_social_brain::neat::{Genome, Config, Population};

fn genome_creation_benchmark(c: &mut Criterion) {
    let config = Config::default();
    
    c.bench_function("genome_create", |b| {
        b.iter(|| {
            Genome::new(black_box(&config))
        })
    });
}

fn genome_activation_benchmark(c: &mut Criterion) {
    let config = Config::default();
    let genome = Genome::new(&config);
    let inputs = vec![1.0, 0.5, -0.5, 0.0];
    
    c.bench_function("genome_activate", |b| {
        b.iter(|| {
            genome.activate(black_box(&inputs))
        })
    });
}

fn mutation_benchmark(c: &mut Criterion) {
    let config = Config::default();
    let mut genome = Genome::new(&config);
    
    c.bench_function("genome_mutate", |b| {
        b.iter(|| {
            genome.mutate(black_box(&config));
        })
    });
}

fn crossover_benchmark(c: &mut Criterion) {
    let config = Config::default();
    let genome1 = Genome::new(&config);
    let genome2 = Genome::new(&config);
    
    c.bench_function("genome_crossover", |b| {
        b.iter(|| {
            Genome::crossover(black_box(&genome1), black_box(&genome2));
        })
    });
}

fn population_evolution_benchmark(c: &mut Criterion) {
    let config = Config {
        population_size: 100,
        ..Default::default()
    };
    
    let mut group = c.benchmark_group("population");
    
    for size in [50, 100, 200].iter() {
        let mut config = config.clone();
        config.population_size = *size;
        let mut population = Population::new(config);
        
        group.bench_with_input(
            BenchmarkId::new("evolve", size),
            size,
            |b, _| {
                b.iter(|| {
                    population.evolve_generation();
                })
            }
        );
    }
    
    group.finish();
}

fn speciation_benchmark(c: &mut Criterion) {
    let config = Config {
        population_size: 150,
        compatibility_threshold: 3.0,
        ..Default::default()
    };
    let population = Population::new(config);
    
    c.bench_function("speciate", |b| {
        b.iter(|| {
            population.speciate();
        })
    });
}

criterion_group!(
    neat_benches,
    genome_creation_benchmark,
    genome_activation_benchmark,
    mutation_benchmark,
    crossover_benchmark,
    population_evolution_benchmark,
    speciation_benchmark
);

criterion_main!(neat_benches);
