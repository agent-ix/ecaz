use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::is_x86_feature_detected;

pub(super) fn score_block32_avx2(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: runtime feature detection above guarantees AVX2 support,
            // and callers validate LUT/code/output shapes before dispatch.
            return unsafe { score_block32_avx2_impl(lut, original_dim, codes, out_scores) };
        }
    }

    scalar::score_block32_scalar(lut, original_dim, codes, out_scores)
}

#[cfg(test)]
pub(super) fn score_block32_avx2_for_test(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: runtime feature detection above guarantees AVX2 support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe { score_block32_avx2_impl(lut, original_dim, codes, out_scores) });
        }
    }

    let _ = (lut, original_dim, codes, out_scores);
    None
}

#[cfg(target_arch = "x86")]
use std::arch::x86::{
    __m256, __m256i, _mm256_add_ps, _mm256_and_si256, _mm256_blendv_ps, _mm256_castsi256_ps,
    _mm256_cmpgt_epi32, _mm256_loadu_ps, _mm256_loadu_si256, _mm256_permutevar8x32_ps,
    _mm256_set1_epi32, _mm256_setzero_ps, _mm256_storeu_ps,
};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256, __m256i, _mm256_add_ps, _mm256_and_si256, _mm256_blendv_ps, _mm256_castsi256_ps,
    _mm256_cmpgt_epi32, _mm256_loadu_ps, _mm256_loadu_si256, _mm256_permutevar8x32_ps,
    _mm256_set1_epi32, _mm256_setzero_ps, _mm256_storeu_ps,
};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const OCTET_COUNT: usize = BLOCK_WIDTH / 8;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_block32_avx2_impl(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    // Register-resident f32 LUT select, sharing the grouped-PQ F8 shape:
    // per dim the 16-entry LUT is held as two 8-float halves and each lane's
    // entry is selected with a permute over each half plus a blend on the
    // index high bit. Accumulation stays per-lane in dim order, so scores
    // are bit-exact against the scalar block reference. Dims are the outer
    // loop so each LUT half is loaded once per dim instead of once per octet.
    let mut acc: [__m256; OCTET_COUNT] = [_mm256_setzero_ps(); OCTET_COUNT];
    let seven = _mm256_set1_epi32(7);
    for dim_index in 0..original_dim {
        let lut_offset = dim_index * 16;
        let byte_index = dim_index / 2;
        let low_lut = _mm256_loadu_ps(lut.as_ptr().add(lut_offset));
        let high_lut = _mm256_loadu_ps(lut.as_ptr().add(lut_offset + 8));
        for (octet, acc_slot) in acc.iter_mut().enumerate() {
            let lane_base = octet * 8;
            let indexes = [
                nibble_index(codes[lane_base], byte_index, dim_index),
                nibble_index(codes[lane_base + 1], byte_index, dim_index),
                nibble_index(codes[lane_base + 2], byte_index, dim_index),
                nibble_index(codes[lane_base + 3], byte_index, dim_index),
                nibble_index(codes[lane_base + 4], byte_index, dim_index),
                nibble_index(codes[lane_base + 5], byte_index, dim_index),
                nibble_index(codes[lane_base + 6], byte_index, dim_index),
                nibble_index(codes[lane_base + 7], byte_index, dim_index),
            ];
            let index_vector = _mm256_loadu_si256(indexes.as_ptr().cast::<__m256i>());
            let low_indexes = _mm256_and_si256(index_vector, seven);
            let low_values = _mm256_permutevar8x32_ps(low_lut, low_indexes);
            let high_values = _mm256_permutevar8x32_ps(high_lut, low_indexes);
            let high_mask = _mm256_castsi256_ps(_mm256_cmpgt_epi32(index_vector, seven));
            let values = _mm256_blendv_ps(low_values, high_values, high_mask);
            *acc_slot = _mm256_add_ps(*acc_slot, values);
        }
    }
    for (octet, acc_slot) in acc.iter().enumerate() {
        _mm256_storeu_ps(out_scores.as_mut_ptr().add(octet * 8), *acc_slot);
    }

    Isa::Avx2
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
fn nibble_index(code: &[u8], byte_index: usize, dim_index: usize) -> i32 {
    let packed = code[byte_index];
    let nibble = if dim_index & 1 == 0 {
        packed & 0x0F
    } else {
        packed >> 4
    };
    i32::from(nibble)
}
