// file: benches/executor_benchmark.rs
// version: 1.0.0
// guid: 9da31367-a838-412e-8a0e-218c2cd6406e

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use copilot_agent_util::{config::Config, executor::Executor};
use tokio::runtime::Runtime;

fn benchmark_executor_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("executor creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = Config::default();
                black_box(Executor::new(config).await.unwrap())
            })
        })
    });
}

criterion_group!(benches, benchmark_executor_creation);
criterion_main!(benches);
