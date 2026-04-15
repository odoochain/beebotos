//! Reasoning Engine Benchmarks
//!
//! Performance benchmarks for reasoning and inference operations.

use beebotos_social_brain::knowledge::*;
use beebotos_social_brain::reasoning::deductive::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Benchmark knowledge base operations
fn knowledge_base_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("knowledge_base");

    group.bench_function("add_fact", |b| {
        let mut kb = KnowledgeBase::new();
        let mut counter = 0usize;
        b.iter(|| {
            let fact = Fact::new(format!("predicate{}", counter))
                .with_arg(Term::const_(format!("arg{}", counter)));
            counter += 1;
            black_box(&mut kb).add_fact(fact);
        })
    });

    group.bench_function("add_rule", |b| {
        let mut kb = KnowledgeBase::new();
        let mut counter = 0usize;
        b.iter(|| {
            let rule = Rule::new(Atom::new(format!("head{}", counter)))
                .if_(Atom::new(format!("body{}", counter)));
            counter += 1;
            black_box(&mut kb).add_rule(rule);
        })
    });

    group.bench_function("query", |b| {
        let mut kb = KnowledgeBase::new();
        for i in 0..100 {
            kb.add_fact(Fact::new("test_predicate").with_arg(Term::const_(format!("arg{}", i))));
        }
        b.iter(|| {
            let results = black_box(&kb).query(black_box("test_predicate"));
            black_box(results);
        })
    });

    group.finish();
}

/// Benchmark forward chaining inference
fn forward_chaining_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("forward_chaining");

    group.bench_function("small_kb", |b| {
        let mut kb = KnowledgeBase::new();

        // Add base facts
        kb.add_fact(Fact::new("human").with_arg(Term::const_("socrates")));
        kb.add_fact(Fact::new("human").with_arg(Term::const_("plato")));

        // Add rule: human(X) => mortal(X)
        kb.add_rule(
            Rule::new(Atom::new("mortal").arg(Term::var("X")))
                .if_(Atom::new("human").arg(Term::var("X"))),
        );

        b.iter(|| {
            let mut kb_clone = black_box(kb.clone());
            let new_facts = kb_clone.forward_chain(black_box(10));
            black_box(new_facts);
        })
    });

    group.bench_function("medium_kb", |b| {
        let mut kb = KnowledgeBase::new();

        // Add multiple facts and rules
        for i in 0..50 {
            kb.add_fact(Fact::new("category_a").with_arg(Term::const_(format!("item{}", i))));
        }

        for i in 0..10 {
            kb.add_rule(
                Rule::new(Atom::new(format!("derived{}", i)).arg(Term::var("X")))
                    .if_(Atom::new("category_a").arg(Term::var("X"))),
            );
        }

        b.iter(|| {
            let mut kb_clone = black_box(kb.clone());
            let new_facts = kb_clone.forward_chain(black_box(10));
            black_box(new_facts);
        })
    });

    group.finish();
}

/// Benchmark backward chaining inference
fn backward_chaining_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("backward_chaining");

    group.bench_function("prove_fact", |b| {
        let mut kb = KnowledgeBase::new();
        kb.add_fact(Fact::new("mortal").with_arg(Term::const_("socrates")));

        let goal = Atom::new("mortal").arg(Term::const_("socrates"));

        b.iter(|| {
            let result = black_box(&kb).prove(black_box(&goal));
            black_box(result);
        })
    });

    group.bench_function("prove_with_rules", |b| {
        let mut kb = KnowledgeBase::new();
        kb.add_fact(Fact::new("human").with_arg(Term::const_("socrates")));
        kb.add_rule(
            Rule::new(Atom::new("mortal").arg(Term::var("X")))
                .if_(Atom::new("human").arg(Term::var("X"))),
        );

        let goal = Atom::new("mortal").arg(Term::const_("socrates"));

        b.iter(|| {
            let result = black_box(&kb).prove(black_box(&goal));
            black_box(result);
        })
    });

    group.finish();
}

/// Benchmark knowledge graph operations
fn knowledge_graph_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("knowledge_graph");

    group.bench_function("add_node", |b| {
        let mut graph = KnowledgeGraph::new();
        let mut counter = 0usize;
        b.iter(|| {
            let node = Node {
                id: format!("node{}", counter),
                label: format!("Node {}", counter),
                properties: std::collections::HashMap::new(),
            };
            counter += 1;
            black_box(&mut graph).add_node(node);
        })
    });

    group.bench_function("add_edge", |b| {
        let mut graph = KnowledgeGraph::new();
        let node1 = Node {
            id: "n1".to_string(),
            label: "Node1".to_string(),
            properties: std::collections::HashMap::new(),
        };
        let node2 = Node {
            id: "n2".to_string(),
            label: "Node2".to_string(),
            properties: std::collections::HashMap::new(),
        };
        graph.add_node(node1);
        graph.add_node(node2);

        let mut counter = 0usize;
        b.iter(|| {
            let edge = Edge {
                source: "n1".to_string(),
                target: "n2".to_string(),
                relation: format!("rel{}", counter),
                weight: 1.0,
            };
            counter += 1;
            black_box(&mut graph).add_edge(edge);
        })
    });

    group.bench_function("node_count", |b| {
        let mut graph = KnowledgeGraph::new();
        for i in 0..100 {
            graph.add_node(Node {
                id: format!("node{}", i),
                label: format!("Node {}", i),
                properties: std::collections::HashMap::new(),
            });
        }
        b.iter(|| {
            let count = black_box(&graph).node_count();
            black_box(count);
        })
    });

    group.finish();
}

/// Benchmark inference engine
fn inference_engine_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("inference_engine");

    group.bench_function("infer", |b| {
        let engine = InferenceEngine::new();
        let graph = KnowledgeGraph::new();

        b.iter(|| {
            let results =
                black_box(&engine).infer(black_box(&graph), black_box("test is related to query"));
            black_box(results);
        })
    });

    group.bench_function("check_consistency", |b| {
        let engine = InferenceEngine::new();
        let graph = KnowledgeGraph::new();

        b.iter(|| {
            let report = black_box(&engine).check_consistency(black_box(&graph));
            black_box(report);
        })
    });

    group.finish();
}

/// Benchmark ontology operations
fn ontology_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ontology");

    group.bench_function("add_concept", |b| {
        let mut ontology = Ontology::new();
        let mut counter = 0usize;
        b.iter(|| {
            let concept = Concept {
                id: format!("concept{}", counter),
                name: format!("Concept {}", counter),
                description: Some(format!("Description {}", counter)),
                parent: None,
                properties: std::collections::HashMap::new(),
            };
            counter += 1;
            black_box(&mut ontology).add_concept(concept);
        })
    });

    group.bench_function("get_concept", |b| {
        let mut ontology = Ontology::new();
        for i in 0..100 {
            ontology.add_concept(Concept {
                id: format!("concept{}", i),
                name: format!("Concept {}", i),
                description: None,
                parent: None,
                properties: std::collections::HashMap::new(),
            });
        }
        b.iter(|| {
            let result = black_box(&ontology).get_concept(black_box("concept50"));
            black_box(result);
        })
    });

    group.finish();
}

/// Benchmark complex reasoning scenarios
fn complex_reasoning_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_reasoning");

    group.bench_function("chain_inference", |b| {
        let mut kb = KnowledgeBase::new();

        // Build a chain: A => B => C => D
        kb.add_fact(Fact::new("fact_a"));
        kb.add_rule(Rule::new(Atom::new("fact_b")).if_(Atom::new("fact_a")));
        kb.add_rule(Rule::new(Atom::new("fact_c")).if_(Atom::new("fact_b")));
        kb.add_rule(Rule::new(Atom::new("fact_d")).if_(Atom::new("fact_c")));

        b.iter(|| {
            let mut kb_clone = black_box(kb.clone());
            let results = kb_clone.forward_chain(black_box(5));
            black_box(results);
        })
    });

    group.bench_function("branching_inference", |b| {
        let mut kb = KnowledgeBase::new();

        // One fact leads to multiple conclusions
        kb.add_fact(Fact::new("base_fact"));
        for i in 0..10 {
            kb.add_rule(
                Rule::new(Atom::new(format!("conclusion{}", i))).if_(Atom::new("base_fact")),
            );
        }

        b.iter(|| {
            let mut kb_clone = black_box(kb.clone());
            let results = kb_clone.forward_chain(black_box(10));
            black_box(results);
        })
    });

    group.finish();
}

criterion_group!(
    reasoning_benches,
    knowledge_base_benchmark,
    forward_chaining_benchmark,
    backward_chaining_benchmark,
    knowledge_graph_benchmark,
    inference_engine_benchmark,
    ontology_benchmark,
    complex_reasoning_benchmark
);

criterion_main!(reasoning_benches);
