use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

#[cfg(target_arch = "aarch64")]
use std::arch::is_aarch64_feature_detected;

pub(super) fn score_block32_neon(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support,
            // and callers validate LUT/code/output shapes before dispatch.
            return unsafe { score_block32_neon_impl(lut, original_dim, codes, out_scores) };
        }
    }

    scalar::score_block32_scalar(lut, original_dim, codes, out_scores)
}

#[cfg(test)]
pub(super) fn score_block32_neon_for_test(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe { score_block32_neon_impl(lut, original_dim, codes, out_scores) });
        }
    }

    let _ = (lut, original_dim, codes, out_scores);
    None
}

#[cfg(target_arch = "aarch64")]
const QUAD_COUNT: usize = BLOCK_WIDTH / 4;

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_block32_neon_impl(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    use std::arch::aarch64::{
        uint8x16x4_t, vaddq_f32, vaddq_u32, vdupq_n_f32, vdupq_n_u32, vld1q_u32, vld1q_u8,
        vmulq_u32, vqtbl4q_u8, vreinterpretq_f32_u8, vreinterpretq_u8_u32, vst1q_f32,
    };

    // Register-resident f32 LUT select, the vqtbl sibling of the AVX2
    // permute/blend shape: per dim the 16-entry LUT (64 bytes) is held as a
    // four-register byte table, and each lane's entry is selected by a
    // vqtbl4q_u8 over byte indexes derived from the lane's nibble
    // (nibble*4 replicated into four bytes plus the 0..3 byte offsets).
    // Accumulation stays per-lane in dim order, so scores are bit-exact
    // against the scalar block reference.
    let mut acc = [vdupq_n_f32(0.0); QUAD_COUNT];
    let byte_offsets = vdupq_n_u32(0x0302_0100);
    let byte_replicate = vdupq_n_u32(0x0101_0101);
    for dim_index in 0..original_dim {
        let lut_bytes = lut.as_ptr().add(dim_index * 16).cast::<u8>();
        let table = uint8x16x4_t(
            vld1q_u8(lut_bytes),
            vld1q_u8(lut_bytes.add(16)),
            vld1q_u8(lut_bytes.add(32)),
            vld1q_u8(lut_bytes.add(48)),
        );
        let byte_index = dim_index / 2;
        for (quad, acc_slot) in acc.iter_mut().enumerate() {
            let lane_base = quad * 4;
            let scaled_indexes = [
                nibble_index(codes[lane_base], byte_index, dim_index) * 4,
                nibble_index(codes[lane_base + 1], byte_index, dim_index) * 4,
                nibble_index(codes[lane_base + 2], byte_index, dim_index) * 4,
                nibble_index(codes[lane_base + 3], byte_index, dim_index) * 4,
            ];
            let byte_bases = vmulq_u32(vld1q_u32(scaled_indexes.as_ptr()), byte_replicate);
            let byte_indexes = vreinterpretq_u8_u32(vaddq_u32(byte_bases, byte_offsets));
            let values = vreinterpretq_f32_u8(vqtbl4q_u8(table, byte_indexes));
            *acc_slot = vaddq_f32(*acc_slot, values);
        }
    }
    for (quad, acc_slot) in acc.iter().enumerate() {
        vst1q_f32(out_scores.as_mut_ptr().add(quad * 4), *acc_slot);
    }

    Isa::Neon
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn nibble_index(code: &[u8], byte_index: usize, dim_index: usize) -> u32 {
    let packed = code[byte_index];
    let nibble = if dim_index & 1 == 0 {
        packed & 0x0F
    } else {
        packed >> 4
    };
    u32::from(nibble)
}
