//! Microbenchmarks for ProdQuantizer::prepare_ip_query() — once-per-query cost.

#[path = "../helpers.rs"]
mod helpers;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ecaz::bench_api::ProdQuantizer;

fn bench_prepare_ip_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/prepare_ip_query");
    for &(dim, bits) in &[
        (256, 4u8),
        (768, 4),
        (1536, 3),
        (1536, 4),
        (1536, 6),
        (3072, 4),
    ] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let query = helpers::random_unit_vector(dim, 1);

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            b.iter(|| quantizer.prepare_ip_query(&query));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_prepare_ip_query);
criterion_main!(benches);
