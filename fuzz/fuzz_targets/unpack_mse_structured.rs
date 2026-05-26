#![no_main]
//! Structure-aware fuzz target for the MSE index pack/unpack round-trip.
//!
//! Sibling of the raw-byte `fuzz_unpack_mse` target. Where the raw-byte
//! target spends most of its budget producing inputs whose packed length
//! does not match the declared `(dim, bits_per_index)` and is rejected at
//! the first length check, this target uses `arbitrary` to always produce
//! a valid-shape input: a positive dimension, a bits-per-index in
//! `2..=7`, and exactly `dim` indices clamped to the corresponding bit
//! width. The fuzzer then mutates within the shape rather than against
//! the structural gate.
//!
//! Property: for every `(indices, bits)` triple drawn from the structured
//! input, `unpack(pack(indices), dim, bits) == indices`. Round-trip is
//! the cleanest decoder/encoder invariant; if structured fuzzing ever
//! finds a divergence it is by construction a real bug, not a length
//! mismatch.

use libfuzzer_sys::fuzz_target;

#[derive(Debug, arbitrary::Arbitrary)]
struct StructuredInput {
    /// Maps to `dim` in `1..=2048` to bound corpus growth without
    /// changing the shape of the test.
    dim_minus_one: u16,
    /// Maps to `bits_per_index` in `2..=7` (the supported MSE range).
    bits_selector: u8,
    /// Raw seed bytes; we take exactly `dim` indices from this stream,
    /// masking each one down to `bits_per_index` so packing always sees
    /// in-range values.
    raw_indices: Vec<u16>,
}

fuzz_target!(|input: StructuredInput| {
    let dim = 1 + (input.dim_minus_one as usize % 2048);
    let bits = 2 + (input.bits_selector % 6);
    let mask: u16 = (1u16 << bits) - 1;

    if input.raw_indices.len() < dim {
        // Insufficient entropy for the chosen dimension. Skipping
        // preserves valid-shape only inputs and lets the engine
        // synthesize more bytes on the next iteration.
        return;
    }

    let indices: Vec<u16> = input
        .raw_indices
        .iter()
        .take(dim)
        .map(|value| value & mask)
        .collect();

    let packed = ecaz_fuzz::bench_api::pack_mse_indices(&indices, bits);
    let expected_packed_len = (dim * bits as usize).div_ceil(8);
    assert_eq!(
        packed.len(),
        expected_packed_len,
        "pack output length must match the declared (dim={dim}, bits={bits}) shape",
    );

    let unpacked = ecaz_fuzz::bench_api::unpack_mse_indices(&packed, dim, bits);
    assert_eq!(
        unpacked, indices,
        "MSE pack/unpack must round-trip exactly for dim={dim} bits={bits}",
    );
});
