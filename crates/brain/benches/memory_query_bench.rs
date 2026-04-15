//! Memory System Benchmarks
//!
//! Performance benchmarks for memory operations and queries.

use beebotos_social_brain::memory::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Benchmark short-term memory operations
fn stm_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("short_term_memory");

    group.bench_function("push", |b| {
        let mut stm = ShortTermMemory::new();
        b.iter(|| {
            let mut s = black_box(stm.clone());
            s.push(black_box("test content"));
            black_box(s);
        })
    });

    group.bench_function("push_with_priority", |b| {
        let mut stm = ShortTermMemory::new();
        b.iter(|| {
            let mut s = black_box(stm.clone());
            s.push_with_priority(black_box("important content"), Priority::High);
            black_box(s);
        })
    });

    group.bench_function("retrieve", |b| {
        let mut stm = ShortTermMemory::new();
        for i in 0..50 {
            stm.push(format!("content with keyword test {}", i));
        }
        b.iter(|| {
            let results = black_box(&stm).retrieve(black_box("keyword"));
            black_box(results);
        })
    });

    group.bench_function("rehearse", |b| {
        let mut stm = ShortTermMemory::new();
        stm.push("test");
        let id = stm.items()[0].id.clone();
        b.iter(|| {
            let mut s = black_box(stm.clone());
            let _ = s.rehearse(black_box(&id));
            black_box(s);
        })
    });

    group.finish();
}

/// Benchmark episodic memory operations
fn episodic_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("episodic_memory");

    // Setup: Create memory with episodes
    let mut em = EpisodicMemory::new();
    for i in 0..100 {
        em.encode(
            format!("Event number {} happened today", i),
            i as u64 * 1000,
            Some(Location {
                name: format!("Location{}", i % 10),
                coordinates: Some((40.0 + i as f64 * 0.1, -74.0 + i as f64 * 0.1)),
            }),
        );
    }

    group.bench_function("encode", |b| {
        let mut em = EpisodicMemory::new();
        let mut counter = 0u64;
        b.iter(|| {
            let id = black_box(&mut em).encode(black_box("Test event"), black_box(counter), None);
            counter += 1;
            black_box(id);
        })
    });

    group.bench_function("query_time_range", |b| {
        b.iter(|| {
            let results = black_box(&em).query_time_range(black_box(10000), black_box(50000));
            black_box(results);
        })
    });

    group.bench_function("query_location", |b| {
        b.iter(|| {
            let results = black_box(&em).query_location(black_box("Location5"));
            black_box(results);
        })
    });

    group.bench_function("search", |b| {
        b.iter(|| {
            let results = black_box(&em).search(black_box("happened"));
            black_box(results);
        })
    });

    group.finish();
}

/// Benchmark semantic memory operations
fn semantic_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("semantic_memory");

    // Setup: Create memory with concepts
    let mut sm = SemanticMemory::new();
    let mut ids = vec![];
    for i in 0..100 {
        let id = sm.learn_concept(
            format!("Concept{}", i),
            format!("Definition of concept {}", i),
            format!("Category{}", i % 10),
        );
        ids.push(id);
    }

    // Add some relations
    for i in 0..50 {
        let _ = sm.add_relation(&ids[i], &ids[i + 1], RelationType::RelatedTo, 0.8);
    }

    group.bench_function("learn_concept", |b| {
        let mut sm = SemanticMemory::new();
        let mut counter = 0usize;
        b.iter(|| {
            let id = black_box(&mut sm).learn_concept(
                black_box(format!("NewConcept{}", counter)),
                black_box("Definition"),
                black_box("Category"),
            );
            counter += 1;
            black_box(id);
        })
    });

    group.bench_function("find_by_name", |b| {
        b.iter(|| {
            let result = black_box(&sm).find_by_name(black_box("Concept50"));
            black_box(result);
        })
    });

    group.bench_function("find_similar", |b| {
        b.iter(|| {
            let results = black_box(&sm).find_similar(black_box(&ids[50]), 0.5);
            black_box(results);
        })
    });

    group.bench_function("by_category", |b| {
        b.iter(|| {
            let results = black_box(&sm).by_category(black_box("Category5"));
            black_box(results);
        })
    });

    group.finish();
}

/// Benchmark memory query builder
fn memory_query_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_query");

    group.bench_function("query_build", |b| {
        b.iter(|| {
            let query = MemoryQuery::new(black_box("search term"))
                .with_types(black_box(vec![MemoryType::Episodic, MemoryType::Semantic]))
                .with_min_importance(black_box(0.5))
                .with_limit(black_box(10));
            black_box(query);
        })
    });

    group.bench_function("query_with_time_range", |b| {
        b.iter(|| {
            let query = MemoryQuery::new(black_box("search"))
                .with_time_range(black_box(0), black_box(1000000));
            black_box(query);
        })
    });

    group.finish();
}

/// Benchmark memory consolidation
fn consolidation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("consolidation");

    group.bench_function("consolidate_small", |b| {
        let mut memory = UnifiedMemory::new();
        // Add items to STM
        for i in 0..10 {
            memory.short_term.push(format!("Item {}", i));
            let id = memory.short_term.items()[i].id.clone();
            for _ in 0..5 {
                let _ = memory.short_term.rehearse(&id);
            }
        }
        b.iter(|| {
            let mut m = black_box(memory.clone());
            let count = m.consolidate();
            black_box(count);
        })
    });

    group.bench_function("consolidate_medium", |b| {
        let mut memory = UnifiedMemory::new();
        // Add items to STM
        for i in 0..50 {
            memory.short_term.push(format!("Item {}", i));
            if i < memory.short_term.len() {
                let id = memory.short_term.items()[i].id.clone();
                for _ in 0..5 {
                    let _ = memory.short_term.rehearse(&id);
                }
            }
        }
        b.iter(|| {
            let mut m = black_box(memory.clone());
            let count = m.consolidate();
            black_box(count);
        })
    });

    group.finish();
}

/// Benchmark procedural memory
fn procedural_memory_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("procedural_memory");

    // Setup
    let mut pm = ProceduralMemory::new();
    for i in 0..50 {
        let steps = vec![
            Step {
                id: "1".to_string(),
                action: format!("Action {}A", i),
                expected_outcome: "Success".to_string(),
                on_success: Some("2".to_string()),
                on_failure: None,
            },
            Step {
                id: "2".to_string(),
                action: format!("Action {}B", i),
                expected_outcome: "Complete".to_string(),
                on_success: None,
                on_failure: None,
            },
        ];
        pm.learn(format!("Procedure{}", i), steps);
    }

    group.bench_function("learn_procedure", |b| {
        let mut pm = ProceduralMemory::new();
        let steps = vec![Step {
            id: "1".to_string(),
            action: "Test".to_string(),
            expected_outcome: "Done".to_string(),
            on_success: None,
            on_failure: None,
        }];
        let mut counter = 0usize;
        b.iter(|| {
            let id = black_box(&mut pm).learn(
                black_box(format!("NewProc{}", counter)),
                black_box(steps.clone()),
            );
            counter += 1;
            black_box(id);
        })
    });

    group.bench_function("search", |b| {
        b.iter(|| {
            let results = black_box(&pm).search(black_box("Procedure25"));
            black_box(results);
        })
    });

    group.bench_function("record_execution", |b| {
        let proc_id = pm.list_all()[0].id.clone();
        b.iter(|| {
            let mut p = black_box(pm.clone());
            p.record_execution(black_box(&proc_id), black_box(true));
            black_box(p);
        })
    });

    group.finish();
}

criterion_group!(
    memory_benches,
    stm_operations_benchmark,
    episodic_operations_benchmark,
    semantic_operations_benchmark,
    memory_query_benchmark,
    consolidation_benchmark,
    procedural_memory_benchmark
);

criterion_main!(memory_benches);
