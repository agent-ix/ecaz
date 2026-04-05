//! Microbenchmarks for the full ProdQuantizer::encode() pipeline.

#[path = "../helpers.rs"]
mod helpers;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tqvector::bench_api::ProdQuantizer;

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/encode");
    for &(dim, bits) in &[
        (64, 4u8),
        (256, 4),
        (768, 4),
        (1536, 2),
        (1536, 3),
        (1536, 4),
        (1536, 6),
        (1536, 8),
        (3072, 4),
    ] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let vector = helpers::random_unit_vector(dim, 99);

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            b.iter(|| quantizer.encode(&vector));
        });
    }
    group.finish();
}

fn bench_encode_pack(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/encode_pack");
    for &(dim, bits) in &[(768, 4u8), (1536, 4), (3072, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let vector = helpers::random_unit_vector(dim, 99);

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            b.iter(|| {
                let encoded = quantizer.encode(&vector);
                quantizer.pack_payload(&encoded)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_encode, bench_encode_pack);
criterion_main!(benches);
