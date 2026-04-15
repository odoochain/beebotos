//! Core types benchmarks

use beebotos_core::types::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_agent_id_creation(c: &mut Criterion) {
    c.bench_function("agent_id_new", |b| {
        b.iter(|| {
            black_box(AgentId::new());
        });
    });
}

fn bench_agent_id_to_string(c: &mut Criterion) {
    let id = AgentId::new();
    c.bench_function("agent_id_display", |b| {
        b.iter(|| {
            black_box(id.to_string());
        });
    });
}

fn bench_capability_level_comparison(c: &mut Criterion) {
    c.bench_function("capability_level_cmp", |b| {
        b.iter(|| {
            black_box(CapabilityLevel::L10 > CapabilityLevel::L0);
        });
    });
}

fn bench_session_id_generation(c: &mut Criterion) {
    c.bench_function("session_id_generate", |b| {
        b.iter(|| {
            black_box(SessionId::new());
        });
    });
}

criterion_group!(
    benches,
    bench_agent_id_creation,
    bench_agent_id_to_string,
    bench_capability_level_comparison,
    bench_session_id_generation
);
criterion_main!(benches);
