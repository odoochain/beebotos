//! Kernel scheduler benchmarks

use beebotos_kernel::capabilities::{CapabilityLevel, CapabilitySet};
use beebotos_kernel::scheduler::queue::{SchedulingAlgorithm, TaskQueue};
use beebotos_kernel::scheduler::{Priority, Task};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_task_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_creation");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                Task::new(rand::random::<u64>(), "bench_task").with_priority(Priority::Normal)
            });
        });
    }

    group.finish();
}

fn bench_priority_comparison(c: &mut Criterion) {
    c.bench_function("priority_comparison", |b| {
        let p1 = Priority::High;
        let p2 = Priority::Normal;

        b.iter(|| {
            black_box(p1 < p2); // Lower value = higher priority
        });
    });
}

fn bench_capability_verification(c: &mut Criterion) {
    let caps = CapabilitySet::standard();

    c.bench_function("capability_verify_l5", |b| {
        b.iter(|| {
            let _ = black_box(caps.verify(CapabilityLevel::L5SpawnLimited));
        });
    });
}

fn bench_queue_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("queue_operations");

    group.bench_function("cfs_enqueue", |b| {
        let queue = TaskQueue::new(SchedulingAlgorithm::Cfs);
        let _task = create_test_task(1, Priority::Normal);

        b.iter(|| {
            // Note: Would need async runtime for actual benchmark
            black_box(&queue);
        });
    });

    group.bench_function("priority_enqueue", |b| {
        let queue = TaskQueue::new(SchedulingAlgorithm::Priority);

        b.iter(|| {
            black_box(&queue);
        });
    });

    group.finish();
}

fn create_test_task(id: u64, priority: Priority) -> Task {
    Task::new(id, format!("task_{}", id)).with_priority(priority)
}

criterion_group!(
    benches,
    bench_task_creation,
    bench_priority_comparison,
    bench_capability_verification,
    bench_queue_operations,
);
criterion_main!(benches);
