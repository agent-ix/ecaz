use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
use std::arch::is_aarch64_feature_detected;

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
core::arch::global_asm!(
    r#"
    .text
    .arch armv8.2-a+sve

    // Per-dim LUT select + accumulate, VL-agnostic:
    //   x0 = accumulator base (f32, count elements)
    //   x1 = LUT base for this dim (16 f32 entries)
    //   x2 = per-lane u32 nibble indexes (count elements, each 0..=15)
    //   x3 = count
    // Loads the per-lane LUT entries with a vector gather instead of the
    // grouped-PQ scalar value-staging loop, then accumulates in place.
    .global ecaz_lut32_sve_gather_accumulate_f32
    .hidden ecaz_lut32_sve_gather_accumulate_f32
    .type ecaz_lut32_sve_gather_accumulate_f32, %function
ecaz_lut32_sve_gather_accumulate_f32:
    mov x4, #0
1:
    whilelt p0.s, x4, x3
    b.none 2f
    ld1w z0.s, p0/z, [x2, x4, lsl #2]
    ld1w z1.s, p0/z, [x1, z0.s, uxtw #2]
    ld1w z2.s, p0/z, [x0, x4, lsl #2]
    fadd z2.s, z2.s, z1.s
    st1w z2.s, p0, [x0, x4, lsl #2]
    incw x4
    b 1b
2:
    ret

    .global ecaz_lut32_sve_cntw
    .hidden ecaz_lut32_sve_cntw
    .type ecaz_lut32_sve_cntw, %function
ecaz_lut32_sve_cntw:
    cntw x0
    ret
"#
);

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
extern "C" {
    fn ecaz_lut32_sve_gather_accumulate_f32(
        acc: *mut f32,
        lut_dim_base: *const f32,
        indexes: *const u32,
        count: usize,
    );
}

#[cfg(all(test, target_arch = "aarch64", not(target_vendor = "apple")))]
extern "C" {
    fn ecaz_lut32_sve_cntw() -> usize;
}

pub(super) fn score_block32_sve(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
    {
        if is_aarch64_feature_detected!("sve2") {
            // SAFETY: runtime feature detection above guarantees SVE2/SVE
            // support, and callers validate LUT/code/output shapes before dispatch.
            return unsafe {
                score_block32_sve_impl(lut, original_dim, codes, out_scores, Isa::Sve2)
            };
        }
        if is_aarch64_feature_detected!("sve") {
            // SAFETY: runtime feature detection above guarantees SVE support,
            // and callers validate LUT/code/output shapes before dispatch.
            return unsafe {
                score_block32_sve_impl(lut, original_dim, codes, out_scores, Isa::Sve)
            };
        }
    }

    scalar::score_block32_scalar(lut, original_dim, codes, out_scores)
}

#[cfg(test)]
pub(super) fn score_block32_sve_for_test(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
    {
        if is_aarch64_feature_detected!("sve2") {
            // SAFETY: runtime feature detection above guarantees SVE2/SVE support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe {
                score_block32_sve_impl(lut, original_dim, codes, out_scores, Isa::Sve2)
            });
        }
        if is_aarch64_feature_detected!("sve") {
            // SAFETY: runtime feature detection above guarantees SVE support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe {
                score_block32_sve_impl(lut, original_dim, codes, out_scores, Isa::Sve)
            });
        }
    }

    let _ = (lut, original_dim, codes, out_scores);
    None
}

/// Live-width entry for sub-block tails: scores exactly `codes.len()`
/// candidates (1..BLOCK_WIDTH) with no padding — the VL-agnostic `whilelt`
/// loop in the gather helper predicates the tail natively. Returns `None`
/// when SVE is unavailable so the caller can fall back.
pub(super) fn score_partial_sve(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
    {
        debug_assert!(!codes.is_empty() && codes.len() < BLOCK_WIDTH);
        if is_aarch64_feature_detected!("sve2") {
            // SAFETY: runtime feature detection above guarantees SVE2/SVE
            // support, and callers validate LUT/code/output shapes before dispatch.
            return Some(unsafe { score_sve_impl(lut, original_dim, codes, out_scores, Isa::Sve2) });
        }
        if is_aarch64_feature_detected!("sve") {
            // SAFETY: runtime feature detection above guarantees SVE support,
            // and callers validate LUT/code/output shapes before dispatch.
            return Some(unsafe { score_sve_impl(lut, original_dim, codes, out_scores, Isa::Sve) });
        }
    }

    let _ = (lut, original_dim, codes, out_scores);
    None
}

#[cfg(test)]
pub(super) fn runtime_vector_lanes_for_test() -> Option<usize> {
    runtime_vector_lanes()
}

#[cfg(test)]
fn runtime_vector_lanes() -> Option<usize> {
    #[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
    {
        if is_aarch64_feature_detected!("sve") {
            // SAFETY: runtime feature detection above guarantees SVE support.
            return Some(unsafe { ecaz_lut32_sve_cntw() });
        }
    }
    None
}

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
unsafe fn score_block32_sve_impl(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
    isa: Isa,
) -> Isa {
    score_sve_impl(lut, original_dim, codes, out_scores, isa)
}

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
unsafe fn score_sve_impl(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
    isa: Isa,
) -> Isa {
    debug_assert!(!codes.is_empty() && codes.len() <= BLOCK_WIDTH);
    debug_assert_eq!(out_scores.len(), codes.len());

    let lane_count = codes.len();
    out_scores[..lane_count].fill(0.0);
    let mut indexes = [0_u32; BLOCK_WIDTH];
    for dim_index in 0..original_dim {
        let byte_index = dim_index / 2;
        for (lane, index) in indexes.iter_mut().enumerate().take(lane_count) {
            *index = nibble_index(codes[lane], byte_index, dim_index);
        }
        ecaz_lut32_sve_gather_accumulate_f32(
            out_scores.as_mut_ptr(),
            lut.as_ptr().add(dim_index * 16),
            indexes.as_ptr(),
            lane_count,
        );
    }

    isa
}

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
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
