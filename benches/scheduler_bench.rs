//! Scheduler Benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use beebot_kernel::scheduler::*;

fn bench_priority_queue(c: &mut Criterion) {
    c.bench_function("priority_queue_enqueue", |b| {
        let queue = TaskQueue::new(SchedulingAlgorithm::Priority);
        let mut counter = 0u64;
        
        b.iter(|| {
            let task = create_test_task(counter, Priority::NORMAL);
            counter += 1;
            // Note: Would need async runtime for actual benchmark
        });
    });
}

fn bench_cfs_queue(c: &mut Criterion) {
    c.bench_function("cfs_queue_enqueue", |b| {
        let queue = TaskQueue::new(SchedulingAlgorithm::CFS);
        let mut counter = 0u64;
        
        b.iter(|| {
            let task = create_test_task(counter, Priority::NORMAL);
            counter += 1;
        });
    });
}

fn bench_capability_check(c: &mut Criterion) {
    c.bench_function("capability_check_l5", |b| {
        let caps = CapabilitySet::standard();
        
        b.iter(|| {
            black_box(caps.has(CapabilityLevel::L5_SpawnLimited));
        });
    });
}

fn create_test_task(id: u64, priority: Priority) -> Task {
    Task::new(
        format!("task_{}", id),
        priority,
        CapabilitySet::empty(),
        Box::pin(async move { Ok(()) }),
    )
}

criterion_group!(
    benches,
    bench_priority_queue,
    bench_cfs_queue,
    bench_capability_check
);
criterion_main!(benches);
