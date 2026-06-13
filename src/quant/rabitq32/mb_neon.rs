//! NEON backend for the multi-bit (bits=2/4) RaBitQ block kernel.
//!
//! 4-wide candidate transpose: each 128-bit accumulator carries four
//! candidates' running `Σ_d query_rotated[d] * dequant_lut[level_d]`, advanced
//! one dimension at a time with an FMA. The dequant LUT (≤16 f32) stays in
//! registers and the rotated query streams, so the working set is independent
//! of candidate count and dimension count (unlike a folded per-dim LUT, which
//! would blow L1 at high dimensions). The per-candidate norm/o_dot/x_norm
//! correction is applied scalar after each 4-wide sum.
//!
//! Graded under the rabitq32 family envelope (ADR-076): the FMA reorders the
//! accumulation versus the forced-scalar anchor, so this is not bit-exact.

#[cfg(target_arch = "aarch64")]
use super::{finish_multibit_estimate, read_level_for};
use super::{PreparedBitsN, BLOCK_WIDTH};
use crate::quant::isa::Isa;

pub(super) fn score_block32_neon(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: NEON confirmed at runtime; callers validate code/output
            // shapes before dispatch.
            unsafe { score_block32_neon_impl(prepared, codes, out_scores) };
            return Isa::Neon;
        }
    }
    super::mb_scalar::score_block32_scalar(prepared, codes, out_scores)
}

pub(super) fn score_partial_neon(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(codes.len(), out_scores.len());
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: NEON confirmed at runtime; callers validate shapes.
            unsafe { score_partial_neon_impl(prepared, codes, out_scores) };
            return Isa::Neon;
        }
    }
    super::mb_scalar::score_partial_scalar(prepared, codes, out_scores)
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_block32_neon_impl(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) {
    let mut lane_base = 0;
    while lane_base + 4 <= BLOCK_WIDTH {
        score_quad_neon(
            prepared,
            &codes[lane_base..lane_base + 4],
            &mut out_scores[lane_base..],
        );
        lane_base += 4;
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_partial_neon_impl(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) {
    let n = codes.len();
    let mut lane_base = 0;
    while lane_base + 4 <= n {
        score_quad_neon(
            prepared,
            &codes[lane_base..lane_base + 4],
            &mut out_scores[lane_base..],
        );
        lane_base += 4;
    }
    while lane_base < n {
        out_scores[lane_base] = super::mb_scalar::score_scalar_tail(prepared, codes[lane_base]);
        lane_base += 1;
    }
}

/// Score four candidates in parallel: sum `dequant_lut[level] * query[d]`
/// 4-wide across dimensions, then apply the scalar correction per lane.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
#[inline]
unsafe fn score_quad_neon(prepared: PreparedBitsN<'_>, codes: &[&[u8]], out_scores: &mut [f32]) {
    use std::arch::aarch64::{vdupq_n_f32, vfmaq_f32, vld1q_f32, vst1q_f32};

    let bits = prepared.bits as usize;
    let lut = prepared.dequant_lut;
    let mut acc = vdupq_n_f32(0.0);
    for (dim_index, &query_value) in prepared
        .query_rotated
        .iter()
        .enumerate()
        .take(prepared.dimensions)
    {
        let dequant = [
            lut[read_level_for(codes[0], dim_index, bits) as usize],
            lut[read_level_for(codes[1], dim_index, bits) as usize],
            lut[read_level_for(codes[2], dim_index, bits) as usize],
            lut[read_level_for(codes[3], dim_index, bits) as usize],
        ];
        acc = vfmaq_f32(acc, vld1q_f32(dequant.as_ptr()), vdupq_n_f32(query_value));
    }

    let mut sums = [0.0_f32; 4];
    vst1q_f32(sums.as_mut_ptr(), acc);
    for lane in 0..4 {
        out_scores[lane] = finish_multibit_estimate(prepared, sums[lane], codes[lane]);
    }
}
