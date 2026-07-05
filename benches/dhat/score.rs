//! dhat heap profiling: scoring pipeline.
//! Run with: cargo run --release --features dhat-heap --bin dhat_score
//! Verifies zero-allocation claim for per-candidate scoring.

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
    let dim = 1536;
    let bits = 4u8;
    let seed = 42u64;

    let quantizer = ecaz::bench_api::ProdQuantizer::new(dim, bits, seed);
    let query = random_unit_vector(dim, 1);
    let prepared = quantizer.prepare_ip_query(&query);

    // Pre-encode 100 payloads (allocation here is expected)
    let payloads: Vec<Vec<u8>> = (0..100)
        .map(|i| quantizer.pack_payload(&quantizer.encode(&random_unit_vector(dim, i + 100))))
        .collect();

    // Now profile only the scoring loop — should be zero-allocation
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    for _ in 0..10_000 {
        for payload in &payloads {
            let _ = quantizer.score_ip_encoded(&prepared, payload);
        }
    }

    #[cfg(feature = "dhat-heap")]
    drop(_profiler);
}
