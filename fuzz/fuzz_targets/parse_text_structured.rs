#![no_main]
//! Structure-aware sibling of `fuzz_parse_text`.
//!
//! Task 46 §Why names `parse_text.rs` explicitly as the canonical
//! example of structural waste: the raw target has to satisfy seven
//! layered gates (valid UTF-8 → `[…]` brackets → comma-separated
//! `key=value` header → numeric `dim`/`bits` → `bits == 4` and
//! `seed == 42` canonical defaults → valid hex body → body length
//! equal to `payload_len(dim, bits) - 4`) before reaching the
//! actually-interesting parse arithmetic. Almost every mutated input
//! falls out at one of those gates and the fuzzer's budget is wasted.
//!
//! This target encodes the gate contract into the Arbitrary input so
//! every iteration produces a well-formed canonical string and
//! exercises the parser's success path. The property is round-trip:
//! `parse_text(format(dim, gamma, codes))` returns
//! `(dim, DEFAULT_QUANT_BITS, DEFAULT_QUANT_SEED, gamma, codes)`.

use std::fmt::Write;

use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
struct StructuredInput {
    /// Maps to `dim` in `1..=1024` — bounded so the formatted code
    /// payload stays comfortably under the libFuzzer 4096-byte input
    /// cap (`payload_len(1024, 4) - 4` = 516 bytes → 1032 hex chars +
    /// ~32 header bytes ≈ 1.1 KiB).
    dim_minus_one: u16,
    /// Free-form 32 bits to seed `gamma`. We map this via
    /// `f32::from_bits` and reject NaN inputs after the round-trip
    /// fails — `f32::parse` does not preserve NaN bit patterns, and
    /// reasoning about that is out of scope for this target.
    gamma_bits: u32,
    /// Raw byte stream used to populate the code body. We take exactly
    /// `payload_len(dim, bits) - 4` bytes off this stream.
    code_seed: Vec<u8>,
}

fuzz_target!(|input: StructuredInput| {
    let dim_usize = 1 + (input.dim_minus_one as usize % 1024);
    let dim = dim_usize as u16;
    let bits = ecaz_fuzz::bench_api::DEFAULT_QUANT_BITS;
    let seed = ecaz_fuzz::bench_api::DEFAULT_QUANT_SEED;
    let gamma = f32::from_bits(input.gamma_bits);

    // `f32::parse` does not round-trip NaN bit patterns; non-finite
    // gammas would also flunk the asserted equality below. Filter
    // them out here so the target stays a clean success-path property.
    if !gamma.is_finite() {
        return;
    }

    let expected_code_len = ecaz_fuzz::bench_api::payload_len(dim_usize, bits) - 4;
    if input.code_seed.len() < expected_code_len {
        // Insufficient entropy for the chosen dim; rerun with more bytes.
        return;
    }
    let codes: Vec<u8> = input.code_seed.iter().copied().take(expected_code_len).collect();

    let mut text = String::with_capacity(64 + codes.len() * 2);
    write!(
        &mut text,
        "[dim={dim},bits={bits},seed={seed},gamma={gamma}]:{hex}",
        hex = hex::encode(&codes),
    )
    .expect("write to String never fails");

    match ecaz_fuzz::bench_api::parse_text(&text) {
        Ok((out_dim, out_bits, out_seed, out_gamma, out_codes)) => {
            assert_eq!(out_dim, dim, "parsed dim should round-trip");
            assert_eq!(out_bits, bits, "parsed bits should round-trip");
            assert_eq!(out_seed, seed, "parsed seed should round-trip");
            assert!(
                (out_gamma - gamma).abs() <= 1e-3 * gamma.abs().max(1.0),
                "parsed gamma should round-trip within f32 text-format precision: \
                 got {out_gamma}, expected {gamma}",
            );
            assert_eq!(out_codes, codes, "parsed code bytes should round-trip");
        }
        Err(err) => {
            panic!("structured parse_text input should always parse, got {err}: input={text}");
        }
    }
});
