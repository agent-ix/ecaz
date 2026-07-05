//! Microbenchmarks for tqvector text I/O (parse and format).

#[path = "../helpers.rs"]
mod helpers;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ecaz::bench_api::{format_text, parse_text, ProdQuantizer};

fn bench_parse_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_io/parse");
    for &(dim, bits) in &[(256, 4u8), (1536, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let encoded = quantizer.encode(&helpers::random_unit_vector(dim, 99));
        let mut code = encoded.mse_packed.clone();
        code.extend_from_slice(&encoded.qjl_packed);
        let text = format_text(dim as u16, bits, 42, encoded.gamma, &code);

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            b.iter(|| parse_text(&text));
        });
    }
    group.finish();
}

fn bench_format_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_io/format");
    for &(dim, bits) in &[(256, 4u8), (1536, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let encoded = quantizer.encode(&helpers::random_unit_vector(dim, 99));
        let mut code = encoded.mse_packed.clone();
        code.extend_from_slice(&encoded.qjl_packed);

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            b.iter(|| format_text(dim as u16, bits, 42, encoded.gamma, &code));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parse_text, bench_format_text);
criterion_main!(benches);
