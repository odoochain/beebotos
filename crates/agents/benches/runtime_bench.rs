//! Agent runtime benchmarks (Stub)

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_stub(c: &mut Criterion) {
    c.bench_function("stub", |b| b.iter(|| {}));
}

criterion_group!(benches, bench_stub);
criterion_main!(benches);
