#![no_main]
//! Structure-aware sibling of `fuzz_vector_normalize`.
//!
//! The raw target parses raw bytes as a packed `[f32]` and rejects on
//! any non-finite value. `Arbitrary` lets us draw a `Vec<u32>` and map
//! every word to a clamped, finite `f32` in `[-1.0, 1.0]` so every
//! iteration drives `ProdQuantizer::encode` against a valid input
//! rather than spending its budget on rejection.

use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
struct StructuredInput {
    /// `dim - 1` so the smallest valid `dim` is 1.
    dim_minus_one: u8,
    /// Raw seed words — every word maps to one clamped, finite `f32`
    /// component of the input vector.
    components: Vec<u32>,
}

fuzz_target!(|input: StructuredInput| {
    // ProdQuantizer requires `2..=8` bits and `1..=u16::MAX` dim. The
    // raw target capped at 128; keep the same cap for apples-to-apples.
    let dim_cap = 128;
    let dim = 1 + (input.dim_minus_one as usize % dim_cap);
    if input.components.len() < dim {
        return;
    }
    let bounded: Vec<f32> = input
        .components
        .iter()
        .take(dim)
        .map(|word| {
            // Map every word to a deterministic finite value in [-1.0, 1.0].
            // `(word as f32) / (u32::MAX as f32)` is always in [0.0, 1.0]
            // and `2.0 * v - 1.0` puts it into [-1.0, 1.0]. Skips the
            // NaN/inf path that the raw target rejects.
            let unit = (*word as f32) / (u32::MAX as f32);
            (2.0 * unit - 1.0).clamp(-1.0, 1.0)
        })
        .collect();
    let quantizer = ecaz_fuzz::bench_api::ProdQuantizer::new(bounded.len(), 4, 42);
    let _ = quantizer.encode(&bounded);
});
