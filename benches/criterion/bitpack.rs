//! Microbenchmarks for MSE/QJL bit-packing and unpacking.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ecaz::bench_api::{
    mse_code_len, pack_mse_indices, pack_qjl_signs, qjl_code_len, unpack_mse_indices,
    unpack_qjl_signs,
};

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn gen_mse_indices(dim: usize, bits_per_index: u8, seed: u64) -> Vec<u16> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let max_val = 1u16 << bits_per_index;
    (0..dim).map(|_| rng.gen_range(0..max_val)).collect()
}

fn gen_qjl_signs(dim: usize, seed: u64) -> Vec<bool> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..dim).map(|_| rng.gen::<bool>()).collect()
}

fn bench_pack_mse(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitpack/pack_mse");
    for &(dim, bits) in &[(256, 3u8), (1536, 3), (1536, 4), (1536, 7), (3072, 3)] {
        let indices = gen_mse_indices(dim, bits, 42);

        group.throughput(Throughput::Bytes(mse_code_len(dim, bits + 1) as u64));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            b.iter(|| pack_mse_indices(&indices, bits));
        });
    }
    group.finish();
}

fn bench_unpack_mse(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitpack/unpack_mse");
    for &(dim, bits) in &[(256, 3u8), (1536, 3), (1536, 4), (1536, 7), (3072, 3)] {
        let indices = gen_mse_indices(dim, bits, 42);
        let packed = pack_mse_indices(&indices, bits);

        group.throughput(Throughput::Elements(dim as u64));
        group.bench_function(BenchmarkId::new(format!("d{dim}_b{bits}"), dim), |b| {
            b.iter(|| unpack_mse_indices(&packed, dim, bits));
        });
    }
    group.finish();
}

fn bench_pack_qjl(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitpack/pack_qjl");
    for &dim in &[256, 1536, 3072] {
        let signs = gen_qjl_signs(dim, 42);

        group.throughput(Throughput::Bytes(qjl_code_len(dim) as u64));
        group.bench_function(BenchmarkId::from_parameter(dim), |b| {
            b.iter(|| pack_qjl_signs(&signs));
        });
    }
    group.finish();
}

fn bench_unpack_qjl(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitpack/unpack_qjl");
    for &dim in &[256, 1536, 3072] {
        let signs = gen_qjl_signs(dim, 42);
        let packed = pack_qjl_signs(&signs);

        group.throughput(Throughput::Elements(dim as u64));
        group.bench_function(BenchmarkId::from_parameter(dim), |b| {
            b.iter(|| unpack_qjl_signs(&packed, dim));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_pack_mse,
    bench_unpack_mse,
    bench_pack_qjl,
    bench_unpack_qjl
);
criterion_main!(benches);
