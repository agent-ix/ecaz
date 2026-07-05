//! Instruction-count benchmarks for FWHT (iai-callgrind).

use ecaz::bench_api::fwht_in_place;
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

#[library_benchmark]
fn fwht_2048() {
    let mut data: Vec<f32> = (0..2048).map(|i| (i as f32) * 0.001).collect();
    fwht_in_place(black_box(&mut data));
}

#[library_benchmark]
fn fwht_4096() {
    let mut data: Vec<f32> = (0..4096).map(|i| (i as f32) * 0.001).collect();
    fwht_in_place(black_box(&mut data));
}

library_benchmark_group!(
    name = hadamard;
    benchmarks = fwht_2048, fwht_4096
);
main!(library_benchmark_groups = hadamard);
