use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

#[cfg(target_arch = "aarch64")]
use std::arch::is_aarch64_feature_detected;

pub(super) fn score_block32_neon(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support,
            // and callers validate LUT/code/output shapes before dispatch.
            return unsafe { score_block32_neon_impl(lut, group_count, codes, out_scores) };
        }
    }

    scalar::score_block32_scalar(lut, group_count, codes, out_scores)
}

#[cfg(test)]
pub(super) fn score_block32_neon_for_test(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe { score_block32_neon_impl(lut, group_count, codes, out_scores) });
        }
    }

    let _ = (lut, group_count, codes, out_scores);
    None
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_block32_neon_impl(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    use std::arch::aarch64::{vaddq_f32, vdupq_n_f32, vld1q_f32, vst1q_f32};

    let centroid_count = crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS;
    for lane_base in (0..BLOCK_WIDTH).step_by(4) {
        let mut acc = vdupq_n_f32(0.0);
        for group_index in 0..group_count {
            let lut_offset = group_index * centroid_count;
            let byte_index = group_index / 2;
            let values = [
                lut[lut_offset + centroid_index(codes[lane_base], byte_index, group_index)],
                lut[lut_offset + centroid_index(codes[lane_base + 1], byte_index, group_index)],
                lut[lut_offset + centroid_index(codes[lane_base + 2], byte_index, group_index)],
                lut[lut_offset + centroid_index(codes[lane_base + 3], byte_index, group_index)],
            ];
            acc = vaddq_f32(acc, vld1q_f32(values.as_ptr()));
        }
        vst1q_f32(out_scores.as_mut_ptr().add(lane_base), acc);
    }

    Isa::Neon
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn centroid_index(code: &[u8], byte_index: usize, group_index: usize) -> usize {
    let packed = code[byte_index];
    if group_index & 1 == 0 {
        usize::from(packed & 0x0F)
    } else {
        usize::from(packed >> 4)
    }
}
