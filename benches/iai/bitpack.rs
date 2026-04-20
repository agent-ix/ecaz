//! Instruction-count benchmarks for bit-packing (iai-callgrind).

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;
use ecaz::bench_api::{pack_mse_indices, pack_qjl_signs, unpack_mse_indices};

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn gen_indices(dim: usize, bits: u8, seed: u64) -> Vec<u16> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let max_val = 1u16 << bits;
    (0..dim).map(|_| rng.gen_range(0..max_val)).collect()
}

fn gen_signs(dim: usize, seed: u64) -> Vec<bool> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..dim).map(|_| rng.gen::<bool>()).collect()
}

#[library_benchmark]
fn pack_mse_1536_3bit() {
    let indices = gen_indices(1536, 3, 42);
    black_box(pack_mse_indices(&indices, 3));
}

#[library_benchmark]
fn unpack_mse_1536_3bit() {
    let indices = gen_indices(1536, 3, 42);
    let packed = pack_mse_indices(&indices, 3);
    black_box(unpack_mse_indices(&packed, 1536, 3));
}

#[library_benchmark]
fn pack_qjl_1536() {
    let signs = gen_signs(1536, 42);
    black_box(pack_qjl_signs(&signs));
}

library_benchmark_group!(
    name = bitpack;
    benchmarks = pack_mse_1536_3bit, unpack_mse_1536_3bit, pack_qjl_1536
);
main!(library_benchmark_groups = bitpack);
