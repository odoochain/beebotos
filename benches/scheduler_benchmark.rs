use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use beebotos_kernel::scheduler::{Scheduler, Task, Priority};
use std::time::Duration;

fn scheduler_creation_benchmark(c: &mut Criterion) {
    c.bench_function("scheduler_create", |b| {
        b.iter(|| {
            Scheduler::new(black_box(1024))
        })
    });
}

fn task_spawn_benchmark(c: &mut Criterion) {
    let mut scheduler = Scheduler::new(1024);
    let task = Task::new(Priority::Normal, Duration::from_millis(100));
    
    c.bench_function("task_spawn", |b| {
        b.iter(|| {
            scheduler.spawn(black_box(task.clone()))
        })
    });
}

fn task_spawn_many_benchmark(c: &mut Criterion) {
    let priorities = [Priority::Low, Priority::Normal, Priority::High, Priority::Realtime];
    
    c.bench_function("task_spawn_1000", |b| {
        b.iter_batched(
            || Scheduler::new(1024),
            |mut scheduler| {
                for i in 0..1000 {
                    let priority = priorities[i % priorities.len()];
                    let task = Task::new(priority, Duration::from_millis(100));
                    scheduler.spawn(task);
                }
            },
            BatchSize::SmallInput
        )
    });
}

fn schedule_cycle_benchmark(c: &mut Criterion) {
    let priorities = [Priority::Low, Priority::Normal, Priority::High];
    
    c.bench_function("schedule_cycle", |b| {
        b.iter_batched(
            || {
                let mut scheduler = Scheduler::new(1024);
                for i in 0..100 {
                    let priority = priorities[i % priorities.len()];
                    let task = Task::new(priority, Duration::from_millis(100));
                    scheduler.spawn(task);
                }
                scheduler
            },
            |mut scheduler| {
                for _ in 0..100 {
                    scheduler.tick();
                }
            },
            BatchSize::SmallInput
        )
    });
}

fn context_switch_benchmark(c: &mut Criterion) {
    let mut scheduler = Scheduler::new(1024);
    let task1 = Task::new(Priority::Normal, Duration::from_millis(10));
    let task2 = Task::new(Priority::Normal, Duration::from_millis(10));
    
    scheduler.spawn(task1);
    scheduler.spawn(task2);
    
    c.bench_function("context_switch", |b| {
        b.iter(|| {
            scheduler.switch_context();
        })
    });
}

fn priority_inheritance_benchmark(c: &mut Criterion) {
    c.bench_function("priority_inheritance", |b| {
        b.iter_batched(
            || {
                let mut scheduler = Scheduler::new(1024);
                let low_task = Task::new(Priority::Low, Duration::from_millis(100));
                let high_task = Task::new(Priority::High, Duration::from_millis(50));
                scheduler.spawn(low_task);
                scheduler.spawn(high_task);
                scheduler
            },
            |mut scheduler| {
                scheduler.resolve_priority_inversion();
            },
            BatchSize::SmallInput
        )
    });
}

criterion_group!(
    scheduler_benches,
    scheduler_creation_benchmark,
    task_spawn_benchmark,
    task_spawn_many_benchmark,
    schedule_cycle_benchmark,
    context_switch_benchmark,
    priority_inheritance_benchmark
);

criterion_main!(scheduler_benches);
