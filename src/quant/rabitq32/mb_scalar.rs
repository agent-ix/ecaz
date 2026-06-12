//! Forced-scalar reference for the multi-bit (bits=2/4) RaBitQ block kernel.
//!
//! This is the strict parity anchor for the multi-bit lane: it reproduces the
//! production scalar estimate (`rabitq::sum_query_dequant_scalar` +
//! `finish_scalar_only_estimate`) in dim order, so SIMD backends grade against
//! it under the ADR-076 family envelope while the scalar path stays bit-exact
//! by construction.

use super::{finish_multibit_estimate, read_level_for, PreparedBitsN, BLOCK_WIDTH};
use crate::quant::isa::Isa;

pub(super) fn score_block32_scalar(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);
    for lane in 0..BLOCK_WIDTH {
        out_scores[lane] = score_scalar_tail(prepared, codes[lane]);
    }
    Isa::Scalar
}

pub(super) fn score_partial_scalar(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), codes.len());
    for (code, out_score) in codes.iter().zip(out_scores.iter_mut()) {
        *out_score = score_scalar_tail(prepared, code);
    }
    Isa::Scalar
}

pub(super) fn score_scalar_tail(prepared: PreparedBitsN<'_>, code: &[u8]) -> f32 {
    let sum_q_dequant = sum_query_dequant_scalar(prepared, code);
    finish_multibit_estimate(prepared, sum_q_dequant, code)
}

/// Bit-exact mirror of `rabitq::sum_query_dequant_scalar`: per-dim
/// `query_rotated[d] * dequant_lut[level_d]`, accumulated in dimension order.
#[inline]
fn sum_query_dequant_scalar(prepared: PreparedBitsN<'_>, code: &[u8]) -> f32 {
    let bits = prepared.bits as usize;
    let mut sum = 0.0_f32;
    for (dim_index, &query_value) in prepared
        .query_rotated
        .iter()
        .enumerate()
        .take(prepared.dimensions)
    {
        let level = read_level_for(code, dim_index, bits) as usize;
        sum += query_value * prepared.dequant_lut[level];
    }
    sum
}
