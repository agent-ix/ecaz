use super::{scalar, PreparedBits1, BLOCK_WIDTH};
use crate::quant::isa::Isa;

/// AVX2 backend for the 32-candidate RaBitQ bits=1 block.
///
/// Reuses the production AVX2 byte-LUT pair primitive
/// (`rabitq::sum_query_dequant_avx2_bits1_pair`), so the kernel's
/// floating-point operation order is identical to the production
/// `estimate_ip_bits1_batch` AVX2 path. The per-candidate finish step stays
/// lane-local through the shared scalar helper.
#[cfg(target_arch = "x86_64")]
pub(super) fn score_block32_avx2(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    if std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma") {
        // SAFETY: runtime feature detection above proves AVX2+FMA. The batch
        // wrapper validated `prepared` (`query_rotated.len() == dimensions`)
        // and every code's length before dispatch, and
        // `out_scores.len() == BLOCK_WIDTH` is asserted inside the impl,
        // which covers the pair primitive's invariants.
        unsafe { score_block32_avx2_impl(prepared, codes, out_scores) };
        return Isa::Avx2;
    }
    scalar::score_block32_scalar(prepared, codes, out_scores)
}

#[cfg(not(target_arch = "x86_64"))]
pub(super) fn score_block32_avx2(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    scalar::score_block32_scalar(prepared, codes, out_scores)
}

/// # Safety
///
/// Caller must confirm AVX2+FMA availability and that `prepared` passed
/// `PreparedBits1::validate` plus per-code `validate_code_shape`: every code
/// holds at least `ceil(dimensions / 8)` packed bytes followed by the RaBitQ
/// scalar metadata, and `query_rotated.len() == dimensions`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn score_block32_avx2_impl(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) {
    debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);
    for pair_index in 0..BLOCK_WIDTH / 2 {
        let lane = pair_index * 2;
        let code0 = codes[lane];
        let code1 = codes[lane + 1];
        // SAFETY: AVX2+FMA availability and code/query shape invariants are
        // this function's own safety contract, upheld by the caller.
        let (sum0, sum1) = unsafe {
            crate::quant::rabitq::sum_query_dequant_avx2_bits1_pair(
                prepared.query_rotated,
                prepared.dimensions,
                prepared.bits1_byte_lut,
                prepared.dequant_lut,
                code0,
                code1,
            )
        };
        out_scores[lane] = scalar::finish_scalar_only_estimate(prepared, sum0, code0);
        out_scores[lane + 1] = scalar::finish_scalar_only_estimate(prepared, sum1, code1);
    }
}

/// Partial-batch (1..=31 candidates) AVX2 scoring: full pairs through the
/// production pair primitive, an odd trailing candidate through the
/// single-candidate primitive. Same operation orders as the production
/// `estimate_ip_bits1_batch` AVX2 path.
#[cfg(target_arch = "x86_64")]
pub(super) fn score_partial_avx2(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    if std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma") {
        // SAFETY: runtime feature detection above proves AVX2+FMA. The batch
        // wrapper validated `prepared` and every code's length before
        // dispatch; the impl asserts matching code/score counts, which
        // covers the primitives' invariants.
        unsafe { score_partial_avx2_impl(prepared, codes, out_scores) };
        return Isa::Avx2;
    }
    scalar::score_partial_scalar(prepared, codes, out_scores)
}

#[cfg(not(target_arch = "x86_64"))]
pub(super) fn score_partial_avx2(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    scalar::score_partial_scalar(prepared, codes, out_scores)
}

/// # Safety
///
/// Caller must confirm AVX2+FMA availability and that `prepared` passed
/// `PreparedBits1::validate` plus per-code `validate_code_shape`, with
/// `codes.len() == out_scores.len()`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn score_partial_avx2_impl(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) {
    debug_assert_eq!(out_scores.len(), codes.len());
    let mut code_pairs = codes.chunks_exact(2);
    let mut out_pairs = out_scores.chunks_exact_mut(2);
    for (pair, out_pair) in (&mut code_pairs).zip(&mut out_pairs) {
        // SAFETY: same contract as this function.
        let (sum0, sum1) = unsafe {
            crate::quant::rabitq::sum_query_dequant_avx2_bits1_pair(
                prepared.query_rotated,
                prepared.dimensions,
                prepared.bits1_byte_lut,
                prepared.dequant_lut,
                pair[0],
                pair[1],
            )
        };
        out_pair[0] = scalar::finish_scalar_only_estimate(prepared, sum0, pair[0]);
        out_pair[1] = scalar::finish_scalar_only_estimate(prepared, sum1, pair[1]);
    }
    if let (Some(code), Some(out)) = (
        code_pairs.remainder().first(),
        out_pairs.into_remainder().first_mut(),
    ) {
        // SAFETY: same contract as this function.
        let sum = unsafe {
            crate::quant::rabitq::sum_query_dequant_avx2_bits1(
                prepared.query_rotated,
                prepared.dimensions,
                prepared.bits1_byte_lut,
                prepared.dequant_lut,
                code,
            )
        };
        *out = scalar::finish_scalar_only_estimate(prepared, sum, code);
    }
}
