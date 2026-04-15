//! PAD (Pleasure-Arousal-Dominance) Benchmarks
//!
//! Performance benchmarks for PAD emotional model operations.

use beebotos_social_brain::pad::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Benchmark PAD creation
fn pad_creation_benchmark(c: &mut Criterion) {
    c.bench_function("pad_creation", |b| {
        b.iter(|| {
            let pad = Pad::new(black_box(0.5), black_box(0.6), black_box(0.7));
            black_box(pad);
        })
    });
}

/// Benchmark PAD operations
fn pad_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("pad_operations");

    let pad1 = Pad::new(0.5, 0.6, 0.7);
    let pad2 = Pad::new(-0.3, 0.4, 0.5);

    group.bench_function("addition", |b| {
        b.iter(|| {
            let result = black_box(pad1) + black_box(pad2);
            black_box(result);
        })
    });

    group.bench_function("multiplication", |b| {
        b.iter(|| {
            let result = black_box(pad1) * black_box(0.5);
            black_box(result);
        })
    });

    group.bench_function("distance", |b| {
        b.iter(|| {
            let dist = black_box(pad1).distance(&black_box(pad2));
            black_box(dist);
        })
    });

    group.bench_function("blend", |b| {
        b.iter(|| {
            let blended = black_box(pad1).blend(&black_box(pad2), black_box(0.5));
            black_box(blended);
        })
    });

    group.finish();
}

/// Benchmark emotion conversion
fn emotion_conversion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("emotion_conversion");

    group.bench_function("pad_to_basic_emotion", |b| {
        let pad = Pad::new(0.8, 0.6, 0.5);
        b.iter(|| {
            let emotion = black_box(pad).to_basic_emotion();
            black_box(emotion);
        })
    });

    group.bench_function("basic_emotion_to_pad", |b| {
        b.iter(|| {
            let pad = Pad::from_basic_emotion(black_box(BasicEmotion::Happy));
            black_box(pad);
        })
    });

    group.finish();
}

/// Benchmark emotional intelligence operations
fn emotional_intelligence_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("emotional_intelligence");

    group.bench_function("create_ei", |b| {
        b.iter(|| {
            let ei = EmotionalIntelligence::new();
            black_box(ei);
        })
    });

    group.bench_function("update_emotion", |b| {
        let mut ei = EmotionalIntelligence::new();
        let event = EmotionalEvent {
            description: "Test event".to_string(),
            pleasure_impact: 0.3,
            arousal_impact: 0.2,
            dominance_impact: 0.1,
        };
        b.iter(|| {
            let mut ei_clone = ei.clone();
            ei_clone.update(&event);
            black_box(ei_clone);
        })
    });

    group.bench_function("tick_decay", |b| {
        let mut ei = EmotionalIntelligence::new();
        // Set up with some emotion
        ei.update(&EmotionalEvent {
            description: "Exciting".to_string(),
            pleasure_impact: 0.8,
            arousal_impact: 0.7,
            dominance_impact: 0.5,
        });

        b.iter(|| {
            let mut ei_clone = ei.clone();
            ei_clone.tick();
            black_box(ei_clone);
        })
    });

    group.bench_function("empathize", |b| {
        let mut ei = EmotionalIntelligence::new();
        let other_emotion = Pad::new(0.6, 0.5, 0.4);

        b.iter(|| {
            let mut ei_clone = ei.clone();
            ei_clone.empathize(&other_emotion);
            black_box(ei_clone);
        })
    });

    group.finish();
}

/// Benchmark batch operations
fn batch_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    // Generate test data
    let pads: Vec<Pad> = (0..1000)
        .map(|i| {
            Pad::new(
                (i as f32 % 100.0) / 100.0 * 2.0 - 1.0,
                (i as f32 % 50.0) / 50.0,
                0.5,
            )
        })
        .collect();

    group.bench_function("batch_distance_calculation", |b| {
        b.iter(|| {
            let reference = Pad::neutral();
            let distances: Vec<f32> = pads
                .iter()
                .map(|p| black_box(p).distance(&black_box(reference)))
                .collect();
            black_box(distances);
        })
    });

    group.bench_function("batch_blend_operations", |b| {
        let target = Pad::new(0.5, 0.5, 0.5);
        b.iter(|| {
            let blended: Vec<Pad> = pads
                .iter()
                .map(|p| black_box(p).blend(&black_box(target), 0.3))
                .collect();
            black_box(blended);
        })
    });

    group.finish();
}

criterion_group!(
    pad_benches,
    pad_creation_benchmark,
    pad_operations_benchmark,
    emotion_conversion_benchmark,
    emotional_intelligence_benchmark,
    batch_operations_benchmark
);

criterion_main!(pad_benches);
