//! Microbenchmarks for the scoring hot loop — the innermost path called per candidate.

#[path = "../helpers.rs"]
mod helpers;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ecaz::bench_api::ProdQuantizer;

fn bench_score_ip_encoded(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_encoded");
    for &(dim, bits) in &[
        (256, 4u8),
        (768, 4),
        (1536, 3),
        (1536, 4),
        (1536, 6),
        (3072, 4),
    ] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared = quantizer.prepare_ip_query(&helpers::random_unit_vector(dim, 1));
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                quantizer
                    .pack_payload(&quantizer.encode(&helpers::random_unit_vector(dim, i + 100)))
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let score = quantizer.score_ip_encoded(&prepared, &payloads[idx % 1000]);
                idx += 1;
                score
            });
        });
    }
    group.finish();
}

fn bench_score_ip_codes_lite(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_codes_lite");
    for &(dim, bits) in &[(256, 4u8), (1536, 4), (3072, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let codes: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                let enc = quantizer.encode(&helpers::random_unit_vector(dim, i + 200));
                let mut code = enc.mse_packed;
                code.extend_from_slice(&enc.qjl_packed);
                code
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let a = &codes[idx % 1000];
                let b_code = &codes[(idx + 1) % 1000];
                idx += 1;
                quantizer.score_ip_codes_lite(a, b_code)
            });
        });
    }
    group.finish();
}

fn bench_score_ip_from_parts(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_from_parts");
    for &(dim, bits) in &[(256, 4u8), (768, 4), (1536, 4), (3072, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared = quantizer.prepare_ip_query(&helpers::random_unit_vector(dim, 1));
        let candidates: Vec<(f32, Vec<u8>)> = (0..1000)
            .map(|i| {
                let enc = quantizer.encode(&helpers::random_unit_vector(dim, i + 300));
                let mut code = enc.mse_packed;
                code.extend_from_slice(&enc.qjl_packed);
                (enc.gamma, code)
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let (gamma, code_bytes) = &candidates[idx % 1000];
                idx += 1;
                quantizer.score_ip_from_parts(&prepared, *gamma, code_bytes)
            });
        });
    }
    group.finish();
}

fn bench_score_ip_encoded_lite(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_encoded_lite");
    for &(dim, bits) in &[(256, 4u8), (1536, 4), (3072, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                quantizer
                    .pack_payload(&quantizer.encode(&helpers::random_unit_vector(dim, i + 400)))
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let a = &payloads[idx % 1000];
                let b_payload = &payloads[(idx + 1) % 1000];
                idx += 1;
                quantizer.score_ip_encoded_lite(a, b_payload)
            });
        });
    }
    group.finish();
}

fn bench_decode_approximate(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/decode_approximate");
    for &(dim, bits) in &[(1536, 4u8), (3072, 4)] {
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                quantizer
                    .pack_payload(&quantizer.encode(&helpers::random_unit_vector(dim, i + 500)))
            })
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let payload = &payloads[idx % 1000];
                idx += 1;
                quantizer.decode_approximate(payload)
            });
        });
    }
    group.finish();
}

/// pq_fastscan-flavor scoring path (storage_format='pq_fastscan' on ec_ivf).
/// Uses a precomputed lookup table over packed 4-bit mse codes, with no QJL
/// rotation. This is the hot path inside `IvfQuantizerProfile::PqFastScan`
/// and the dominant kernel for the pq_fastscan storage format.
fn bench_score_ip_from_parts_lut_no_qjl_4bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_from_parts_lut_no_qjl_4bit");
    for &dim in &[256usize, 768, 1536, 3072] {
        let bits = 4u8; // lut_no_qjl_4bit is 4-bit only by construction
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(
            &helpers::random_unit_vector(dim, 1),
        );
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| quantizer.encode(&helpers::random_unit_vector(dim, i + 600)).mse_packed)
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let mse_packed = &payloads[idx % 1000];
                idx += 1;
                quantizer.score_ip_from_parts_lut_no_qjl_4bit(&prepared, mse_packed)
            });
        });
    }
    group.finish();
}

/// Tiled variant of the pq_fastscan path. Same query/code shape but with the
/// LUT tiled to fit in cache for large dim; used at high dimensionalities.
fn bench_score_ip_from_parts_tiled_lut_no_qjl_4bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_from_parts_tiled_lut_no_qjl_4bit");
    let tile_size = 512;
    for &dim in &[1536usize, 3072] {
        let bits = 4u8;
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared = quantizer.prepare_ip_query_tiled_lut_no_qjl_4bit(
            &helpers::random_unit_vector(dim, 1),
            tile_size,
        );
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| quantizer.encode(&helpers::random_unit_vector(dim, i + 700)).mse_packed)
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}_t{tile_size}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let mse_packed = &payloads[idx % 1000];
                idx += 1;
                quantizer.score_ip_from_parts_tiled_lut_no_qjl_4bit(&prepared, mse_packed)
            });
        });
    }
    group.finish();
}

/// int8-approx variant of the pq_fastscan path: same code-set, quantized
/// LUT to int8 for further throughput at marginal recall cost.
fn bench_score_ip_from_parts_int8_approx_no_qjl_4bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_ip_from_parts_int8_approx_no_qjl_4bit");
    for &dim in &[256usize, 768, 1536, 3072] {
        let bits = 4u8;
        let quantizer = ProdQuantizer::new(dim, bits, 42);
        let prepared = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(
            &helpers::random_unit_vector(dim, 1),
        );
        let payloads: Vec<Vec<u8>> = (0..1000)
            .map(|i| quantizer.encode(&helpers::random_unit_vector(dim, i + 800)).mse_packed)
            .collect();

        group.throughput(Throughput::Elements(1));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            let mut idx = 0usize;
            b.iter(|| {
                let mse_packed = &payloads[idx % 1000];
                idx += 1;
                quantizer.score_ip_from_parts_int8_approx_no_qjl_4bit(&prepared, mse_packed)
            });
        });
    }
    group.finish();
}

fn bench_score_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("quant/score_throughput");
    let dim = 1536;
    let bits = 4u8;
    let quantizer = ProdQuantizer::new(dim, bits, 42);
    let prepared = quantizer.prepare_ip_query(&helpers::random_unit_vector(dim, 1));
    let payloads: Vec<Vec<u8>> = (0..1000)
        .map(|i| {
            quantizer.pack_payload(&quantizer.encode(&helpers::random_unit_vector(dim, i + 100)))
        })
        .collect();

    group.throughput(Throughput::Elements(1000));
    group.bench_function("d1536_b4_batch1000", |b| {
        b.iter(|| {
            let mut sum = 0.0f32;
            for payload in &payloads {
                sum += quantizer.score_ip_encoded(&prepared, payload);
            }
            sum
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_score_ip_encoded,
    bench_score_ip_codes_lite,
    bench_score_ip_from_parts,
    bench_score_ip_encoded_lite,
    bench_score_ip_from_parts_lut_no_qjl_4bit,
    bench_score_ip_from_parts_tiled_lut_no_qjl_4bit,
    bench_score_ip_from_parts_int8_approx_no_qjl_4bit,
    bench_decode_approximate,
    bench_score_throughput
);
criterion_main!(benches);
