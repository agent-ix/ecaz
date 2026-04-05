//! Microbenchmarks for Lloyd-Max codebook generation.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use tqvector::bench_api::lloyd_max;

fn bench_lloyd_max(c: &mut Criterion) {
    let mut group = c.benchmark_group("codebook/lloyd_max");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);

    for &(b, dim, points) in &[
        (1usize, 256, 5_000usize),
        (3, 1536, 20_000),
        (5, 1536, 20_000),
        (7, 1536, 20_000),
    ] {
        group.bench_function(BenchmarkId::new(format!("b{b}_d{dim}"), points), |bench| {
            bench.iter(|| lloyd_max(b, dim, points))
        });
    }
    group.finish();
}

fn bench_lloyd_max_dimension_sensitivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("codebook/lloyd_max_dim");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);

    for &dim in &[64, 256, 768, 1536, 3072] {
        group.bench_function(BenchmarkId::new("b3", dim), |bench| {
            bench.iter(|| lloyd_max(3, dim, 20_000));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_lloyd_max,
    bench_lloyd_max_dimension_sensitivity
);
criterion_main!(benches);
