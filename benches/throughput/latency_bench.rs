//! Latency and throughput benchmarks

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

fn bench_message_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_latency");
    group.measurement_time(Duration::from_secs(10));

    for size in [64, 256, 1024, 4096, 16384].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                // Simulate message processing
                std::thread::sleep(Duration::from_micros(10));
            });
        });
    }

    group.finish();
}

fn bench_task_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_throughput");
    group.measurement_time(Duration::from_secs(10));

    for count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                for _ in 0..count {
                    std::hint::black_box(0);
                }
            });
        });
    }

    group.finish();
}

fn bench_agent_spawn_rate(c: &mut Criterion) {
    c.bench_function("agent_spawn_rate", |b| {
        b.iter(|| {
            // Simulate agent spawn
            std::thread::sleep(Duration::from_millis(1));
        });
    });
}

criterion_group!(
    benches,
    bench_message_latency,
    bench_task_throughput,
    bench_agent_spawn_rate
);
criterion_main!(benches);
