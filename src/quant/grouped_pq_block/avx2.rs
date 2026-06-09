use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::is_x86_feature_detected;

pub(super) fn score_block32_avx2(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: runtime feature detection above guarantees AVX2 support,
            // and callers validate LUT/code/output shapes before dispatch.
            return unsafe { score_block32_avx2_impl(lut, group_count, codes, out_scores) };
        }
    }

    scalar::score_block32_scalar(lut, group_count, codes, out_scores)
}

#[cfg(test)]
pub(super) fn score_block32_avx2_for_test(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: runtime feature detection above guarantees AVX2 support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe { score_block32_avx2_impl(lut, group_count, codes, out_scores) });
        }
    }

    let _ = (lut, group_count, codes, out_scores);
    None
}

#[cfg(target_arch = "x86")]
use std::arch::x86::{
    __m256i, _mm256_add_ps, _mm256_i32gather_ps, _mm256_loadu_si256, _mm256_setzero_ps,
    _mm256_storeu_ps,
};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256i, _mm256_add_ps, _mm256_i32gather_ps, _mm256_loadu_si256, _mm256_setzero_ps,
    _mm256_storeu_ps,
};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_block32_avx2_impl(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    let centroid_count = crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS;
    for lane_base in (0..BLOCK_WIDTH).step_by(8) {
        let mut acc = _mm256_setzero_ps();
        for group_index in 0..group_count {
            let lut_offset = group_index * centroid_count;
            let byte_index = group_index / 2;
            let indexes = [
                lut_index(lut_offset, codes[lane_base], byte_index, group_index),
                lut_index(lut_offset, codes[lane_base + 1], byte_index, group_index),
                lut_index(lut_offset, codes[lane_base + 2], byte_index, group_index),
                lut_index(lut_offset, codes[lane_base + 3], byte_index, group_index),
                lut_index(lut_offset, codes[lane_base + 4], byte_index, group_index),
                lut_index(lut_offset, codes[lane_base + 5], byte_index, group_index),
                lut_index(lut_offset, codes[lane_base + 6], byte_index, group_index),
                lut_index(lut_offset, codes[lane_base + 7], byte_index, group_index),
            ];
            let index_vector = _mm256_loadu_si256(indexes.as_ptr().cast::<__m256i>());
            let values = _mm256_i32gather_ps(lut.as_ptr(), index_vector, 4);
            acc = _mm256_add_ps(acc, values);
        }
        _mm256_storeu_ps(out_scores.as_mut_ptr().add(lane_base), acc);
    }

    Isa::Avx2
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
fn lut_index(lut_offset: usize, code: &[u8], byte_index: usize, group_index: usize) -> i32 {
    let packed = code[byte_index];
    let centroid = if group_index & 1 == 0 {
        usize::from(packed & 0x0F)
    } else {
        usize::from(packed >> 4)
    };
    i32::try_from(lut_offset + centroid).expect("grouped-PQ LUT index should fit in i32")
}
