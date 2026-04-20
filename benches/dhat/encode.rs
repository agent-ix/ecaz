//! dhat heap profiling: encode pipeline.
//! Run with: cargo run --release --features dhat-heap --bin dhat_encode
//! Then open dhat-heap.json at https://nnethercote.github.io/dh_view/dh_view.html

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
    for value in &mut values {
        *value /= norm.max(f32::EPSILON);
    }
    values
}

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let dim = 1536;
    let bits = 4u8;
    let seed = 42u64;

    // Build quantizer once (not profiled for per-encode cost)
    let quantizer = ecaz::bench_api::ProdQuantizer::new(dim, bits, seed);
    let vector = random_unit_vector(dim, 99);

    // Profile 1000 encode iterations
    for _ in 0..1000 {
        let encoded = quantizer.encode(&vector);
        let _ = quantizer.pack_payload(&encoded);
    }

    #[cfg(feature = "dhat-heap")]
    drop(_profiler);
}
