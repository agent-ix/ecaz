//! Microbenchmarks for FWHT and SRHT transforms.

#[path = "../helpers.rs"]
mod helpers;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ecaz::bench_api::{
    fwht_in_place, inverse_srht, orthonormal_fwht_in_place, pad_input, sign_vector, srht,
    transform_dim,
};

fn bench_fwht_in_place(c: &mut Criterion) {
    let mut group = c.benchmark_group("hadamard/fwht_in_place");
    for &size in &[64, 256, 1024, 2048, 4096] {
        let template: Vec<f32> = (0..size).map(|i| (i as f32) * 0.001).collect();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let mut data = template.clone();
            b.iter(|| {
                data.copy_from_slice(&template);
                fwht_in_place(&mut data);
            });
        });
    }
    group.finish();
}

fn bench_orthonormal_fwht(c: &mut Criterion) {
    let mut group = c.benchmark_group("hadamard/orthonormal_fwht");
    for &size in &[2048, 4096] {
        let template: Vec<f32> = (0..size).map(|i| (i as f32) * 0.001).collect();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::from_parameter(size), |b| {
            let mut data = template.clone();
            b.iter(|| {
                data.copy_from_slice(&template);
                orthonormal_fwht_in_place(&mut data);
            });
        });
    }
    group.finish();
}

fn bench_srht(c: &mut Criterion) {
    let mut group = c.benchmark_group("hadamard/srht");
    for &dim in &[256, 768, 1536, 3072] {
        let td = transform_dim(dim);
        let padded = pad_input(&helpers::random_unit_vector(dim, 42), td);
        let signs = sign_vector(td, 42);

        group.throughput(Throughput::Elements(td as u64));
        group.bench_function(BenchmarkId::new("forward", dim), |b| {
            b.iter(|| srht(&padded, &signs));
        });
    }
    group.finish();
}

fn bench_inverse_srht(c: &mut Criterion) {
    let mut group = c.benchmark_group("hadamard/inverse_srht");
    for &dim in &[768, 1536, 3072] {
        let td = transform_dim(dim);
        let padded = pad_input(&helpers::random_unit_vector(dim, 42), td);
        let signs = sign_vector(td, 42);
        let rotated = srht(&padded, &signs);

        group.throughput(Throughput::Elements(td as u64));
        group.bench_function(BenchmarkId::new("inverse", dim), |b| {
            b.iter(|| inverse_srht(&rotated, &signs));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_fwht_in_place,
    bench_orthonormal_fwht,
    bench_srht,
    bench_inverse_srht
);
criterion_main!(benches);
