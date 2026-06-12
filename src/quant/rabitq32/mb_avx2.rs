//! AVX2 backend for the multi-bit (bits=2/4) RaBitQ block kernel.
//!
//! 8-wide candidate transpose: each 256-bit accumulator carries eight
//! candidates' running `Σ_d query_rotated[d] * dequant_lut[level_d]`, advanced
//! one dimension at a time with an FMA. The ≤16-entry dequant LUT is gathered
//! with `permutevar8x32` (low-8 / high-8 split blended on `index > 7`), the
//! same shuffle shape as the grouped-PQ AVX2 kernel; the 16-entry blend also
//! serves bits=2 (indices < 4 select the low half). The per-candidate
//! norm/o_dot/x_norm correction is applied scalar after each 8-wide sum.
//!
//! Graded under the rabitq32 family envelope (ADR-076): the FMA reorders the
//! accumulation versus the forced-scalar anchor.
//!
//! NOTE: validated by compile-check on x86_64 and benched on the Intel lane;
//! the M5 dev host cannot execute this path.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use super::read_level_for;
use super::{PreparedBitsN, BLOCK_WIDTH};
use crate::quant::isa::Isa;

#[cfg(target_arch = "x86")]
use std::arch::is_x86_feature_detected;

#[cfg(target_arch = "x86")]
use std::arch::x86::{
    __m256i, _mm256_and_si256, _mm256_blendv_ps, _mm256_castsi256_ps, _mm256_cmpgt_epi32,
    _mm256_fmadd_ps, _mm256_loadu_ps, _mm256_loadu_si256, _mm256_permutevar8x32_ps,
    _mm256_set1_epi32, _mm256_set1_ps, _mm256_setzero_ps, _mm256_storeu_ps,
};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256i, _mm256_and_si256, _mm256_blendv_ps, _mm256_castsi256_ps, _mm256_cmpgt_epi32,
    _mm256_fmadd_ps, _mm256_loadu_ps, _mm256_loadu_si256, _mm256_permutevar8x32_ps,
    _mm256_set1_epi32, _mm256_set1_ps, _mm256_setzero_ps, _mm256_storeu_ps,
};

pub(super) fn score_block32_avx2(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: AVX2+FMA confirmed at runtime; callers validate shapes.
            unsafe { score_block32_avx2_impl(prepared, codes, out_scores) };
            return Isa::Avx2;
        }
    }
    super::mb_scalar::score_block32_scalar(prepared, codes, out_scores)
}

pub(super) fn score_partial_avx2(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(codes.len(), out_scores.len());
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: AVX2+FMA confirmed at runtime; callers validate shapes.
            unsafe { score_partial_avx2_impl(prepared, codes, out_scores) };
            return Isa::Avx2;
        }
    }
    super::mb_scalar::score_partial_scalar(prepared, codes, out_scores)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn score_block32_avx2_impl(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) {
    let mut lane_base = 0;
    while lane_base + 8 <= BLOCK_WIDTH {
        score_oct_avx2(
            prepared,
            &codes[lane_base..lane_base + 8],
            &mut out_scores[lane_base..],
        );
        lane_base += 8;
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn score_partial_avx2_impl(
    prepared: PreparedBitsN<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) {
    let n = codes.len();
    let mut lane_base = 0;
    while lane_base + 8 <= n {
        score_oct_avx2(
            prepared,
            &codes[lane_base..lane_base + 8],
            &mut out_scores[lane_base..],
        );
        lane_base += 8;
    }
    while lane_base < n {
        out_scores[lane_base] = super::mb_scalar::score_scalar_tail(prepared, codes[lane_base]);
        lane_base += 1;
    }
}

/// Score eight candidates in parallel: sum `dequant_lut[level] * query[d]`
/// 8-wide across dimensions, then apply the scalar correction per lane.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2,fma")]
#[inline]
unsafe fn score_oct_avx2(prepared: PreparedBitsN<'_>, codes: &[&[u8]], out_scores: &mut [f32]) {
    let bits = prepared.bits as usize;
    let lut_ptr = prepared.dequant_lut.as_ptr();
    // The dequant LUT is query-independent; hoist the two halves once.
    let low_lut = _mm256_loadu_ps(lut_ptr);
    let high_lut = _mm256_loadu_ps(lut_ptr.add(8));
    let seven = _mm256_set1_epi32(7);

    let mut acc = _mm256_setzero_ps();
    for (dim_index, &query_value) in prepared
        .query_rotated
        .iter()
        .enumerate()
        .take(prepared.dimensions)
    {
        let indexes = [
            read_level_for(codes[0], dim_index, bits) as i32,
            read_level_for(codes[1], dim_index, bits) as i32,
            read_level_for(codes[2], dim_index, bits) as i32,
            read_level_for(codes[3], dim_index, bits) as i32,
            read_level_for(codes[4], dim_index, bits) as i32,
            read_level_for(codes[5], dim_index, bits) as i32,
            read_level_for(codes[6], dim_index, bits) as i32,
            read_level_for(codes[7], dim_index, bits) as i32,
        ];
        let index_vector = _mm256_loadu_si256(indexes.as_ptr().cast::<__m256i>());
        let low_indexes = _mm256_and_si256(index_vector, seven);
        let low_values = _mm256_permutevar8x32_ps(low_lut, low_indexes);
        let high_values = _mm256_permutevar8x32_ps(high_lut, low_indexes);
        let high_mask = _mm256_castsi256_ps(_mm256_cmpgt_epi32(index_vector, seven));
        let dequant = _mm256_blendv_ps(low_values, high_values, high_mask);
        acc = _mm256_fmadd_ps(dequant, _mm256_set1_ps(query_value), acc);
    }

    let mut sums = [0.0_f32; 8];
    _mm256_storeu_ps(sums.as_mut_ptr(), acc);
    for lane in 0..8 {
        out_scores[lane] = super::finish_multibit_estimate(prepared, sums[lane], codes[lane]);
    }
}
