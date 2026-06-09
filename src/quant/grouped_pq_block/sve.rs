use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

#[cfg(target_arch = "aarch64")]
use std::arch::is_aarch64_feature_detected;

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    r#"
    .text
    .arch armv8.2-a+sve

    .global ecaz_grouped_pq_sve_accumulate_f32
    .hidden ecaz_grouped_pq_sve_accumulate_f32
    .type ecaz_grouped_pq_sve_accumulate_f32, %function
ecaz_grouped_pq_sve_accumulate_f32:
    mov x3, #0
1:
    whilelt p0.s, x3, x2
    b.none 2f
    ld1w z0.s, p0/z, [x0, x3, lsl #2]
    ld1w z1.s, p0/z, [x1, x3, lsl #2]
    fadd z0.s, z0.s, z1.s
    st1w z0.s, p0, [x0, x3, lsl #2]
    incw x3
    b 1b
2:
    ret

    .global ecaz_grouped_pq_sve_cntw
    .hidden ecaz_grouped_pq_sve_cntw
    .type ecaz_grouped_pq_sve_cntw, %function
ecaz_grouped_pq_sve_cntw:
    cntw x0
    ret
"#
);

#[cfg(target_arch = "aarch64")]
extern "C" {
    fn ecaz_grouped_pq_sve_accumulate_f32(out: *mut f32, values: *const f32, count: usize);
}

#[cfg(all(test, target_arch = "aarch64"))]
extern "C" {
    fn ecaz_grouped_pq_sve_cntw() -> usize;
}

pub(super) fn score_block32_sve(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("sve2") {
            // SAFETY: runtime feature detection above guarantees SVE2/SVE
            // support, and callers validate LUT/code/output shapes before dispatch.
            return unsafe {
                score_block32_sve_impl(lut, group_count, codes, out_scores, Isa::Sve2)
            };
        }
        if is_aarch64_feature_detected!("sve") {
            // SAFETY: runtime feature detection above guarantees SVE support,
            // and callers validate LUT/code/output shapes before dispatch.
            return unsafe {
                score_block32_sve_impl(lut, group_count, codes, out_scores, Isa::Sve)
            };
        }
    }

    scalar::score_block32_scalar(lut, group_count, codes, out_scores)
}

#[cfg(test)]
pub(super) fn score_block32_sve_for_test(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("sve2") {
            // SAFETY: runtime feature detection above guarantees SVE2/SVE support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe {
                score_block32_sve_impl(lut, group_count, codes, out_scores, Isa::Sve2)
            });
        }
        if is_aarch64_feature_detected!("sve") {
            // SAFETY: runtime feature detection above guarantees SVE support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe {
                score_block32_sve_impl(lut, group_count, codes, out_scores, Isa::Sve)
            });
        }
    }

    let _ = (lut, group_count, codes, out_scores);
    None
}

#[cfg(test)]
pub(super) fn runtime_vector_lanes_for_test() -> Option<usize> {
    runtime_vector_lanes()
}

#[cfg(test)]
fn runtime_vector_lanes() -> Option<usize> {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("sve") {
            // SAFETY: runtime feature detection above guarantees SVE support.
            return Some(unsafe { ecaz_grouped_pq_sve_cntw() });
        }
    }
    None
}

#[cfg(target_arch = "aarch64")]
unsafe fn score_block32_sve_impl(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
    isa: Isa,
) -> Isa {
    debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);

    out_scores[..BLOCK_WIDTH].fill(0.0);
    let mut values = [0.0_f32; BLOCK_WIDTH];
    let centroid_count = crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS;
    for group_index in 0..group_count {
        let lut_offset = group_index * centroid_count;
        let byte_index = group_index / 2;
        for lane in 0..BLOCK_WIDTH {
            values[lane] = lut[lut_offset + centroid_index(codes[lane], byte_index, group_index)];
        }
        ecaz_grouped_pq_sve_accumulate_f32(out_scores.as_mut_ptr(), values.as_ptr(), BLOCK_WIDTH);
    }

    isa
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
