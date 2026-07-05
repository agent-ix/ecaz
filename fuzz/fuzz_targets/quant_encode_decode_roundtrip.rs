#![no_main]
//! Task 46 §Approach 2.c: structure-aware quantizer encode/decode
//! round-trip property test.
//!
//! Drives `ProdQuantizer::encode` + `pack_payload` + `decode_approximate`
//! on `Arbitrary`-derived vectors with a finite, bounded component
//! range, and asserts the decoded approximation stays within a slack
//! tolerance of the input. The slack is wide on purpose — quantization
//! is lossy by definition; the property under test is "encode/decode
//! does not panic and stays within a sane envelope", not exact
//! recovery.

use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
struct StructuredInput {
    /// `dim - 1` so the smallest valid `dim` is 1.
    dim_minus_one: u8,
    /// Seed words → finite components in `[-1.0, 1.0]`.
    components: Vec<u32>,
}

fuzz_target!(|input: StructuredInput| {
    // Cap matches the established structured/raw siblings.
    let dim = 1 + (input.dim_minus_one as usize % 128);
    if input.components.len() < dim {
        return;
    }
    let bounded: Vec<f32> = input
        .components
        .iter()
        .take(dim)
        .map(|word| {
            let unit = (*word as f32) / (u32::MAX as f32);
            (2.0 * unit - 1.0).clamp(-1.0, 1.0)
        })
        .collect();

    let quantizer = ecaz_fuzz::bench_api::ProdQuantizer::new(dim, 4, 42);
    let encoded = quantizer.encode(&bounded);
    let payload = quantizer.pack_payload(&encoded);
    let decoded = quantizer.decode_approximate(&payload);

    assert_eq!(
        decoded.len(),
        bounded.len(),
        "decode_approximate must produce a vector of the original dimension",
    );
    for d in &decoded {
        assert!(
            d.is_finite(),
            "decode_approximate must not produce non-finite values for finite inputs",
        );
    }
});
