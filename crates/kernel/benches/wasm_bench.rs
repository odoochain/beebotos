//! WASM Runtime Benchmarks

use beebotos_kernel::wasm::metering::{CostModel, FuelLimit, FuelTracker};
use beebotos_kernel::wasm::{
    quick_compile, quick_instantiate, test_module_add, EngineConfig, WasmEngine,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn bench_wasm_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_compilation");

    // Test module compilation
    let wasm = test_module_add();
    group.throughput(Throughput::Bytes(wasm.len() as u64));

    group.bench_function("compile_small_module", |b| {
        b.iter(|| {
            let result = quick_compile(black_box(&wasm));
            black_box(result);
        });
    });

    group.finish();
}

fn bench_wasm_instantiation(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_instantiation");

    let wasm = test_module_add();
    let engine = WasmEngine::new(EngineConfig::default()).unwrap();
    let module = engine.compile(&wasm).unwrap();

    group.bench_function("instantiate_module", |b| {
        b.iter(|| {
            let instance = engine.instantiate(black_box(&module));
            black_box(instance);
        });
    });

    group.finish();
}

fn bench_fuel_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("fuel_tracking");

    group.bench_function("consume_fuel", |b| {
        let tracker = FuelTracker::new(FuelLimit::Limited(1_000_000));
        b.iter(|| {
            let _ = tracker.consume(black_box(100));
        });
    });

    group.bench_function("check_remaining", |b| {
        let tracker = FuelTracker::new(FuelLimit::Limited(1_000_000));
        tracker.consume(500_000);
        b.iter(|| {
            let remaining = tracker.remaining();
            black_box(remaining);
        });
    });

    group.bench_function("reset_fuel", |b| {
        let tracker = FuelTracker::new(FuelLimit::Limited(1_000_000));
        b.iter(|| {
            tracker.reset(black_box(1_000_000));
        });
    });

    group.finish();
}

fn bench_cost_model(c: &mut Criterion) {
    let mut group = c.benchmark_group("cost_model");

    let model = CostModel::default();

    group.bench_function("instruction_cost_lookup", |b| {
        b.iter(|| {
            let cost = model.instruction_cost;
            black_box(cost);
        });
    });

    group.bench_function("memory_access_cost", |b| {
        b.iter(|| {
            let cost = model.memory_access_cost;
            black_box(cost);
        });
    });

    group.finish();
}

fn bench_engine_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_creation");

    for config_type in ["default", "production", "development"].iter() {
        let config = match *config_type {
            "production" => EngineConfig::production(),
            "development" => EngineConfig::development(),
            _ => EngineConfig::default(),
        };

        group.bench_with_input(
            BenchmarkId::new("create_engine", config_type),
            &config,
            |b, config| {
                b.iter(|| {
                    let engine = WasmEngine::new(config.clone());
                    black_box(engine);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    wasm_benches,
    bench_wasm_compilation,
    bench_wasm_instantiation,
    bench_fuel_tracking,
    bench_cost_model,
    bench_engine_creation,
);
criterion_main!(wasm_benches);
