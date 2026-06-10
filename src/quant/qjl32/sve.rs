use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;
use crate::quant::prod::{PreparedQuery, ProdQuantizer};

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
use std::arch::is_aarch64_feature_detected;

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
core::arch::global_asm!(
    r#"
    .text
    .arch armv8.2-a+sve

    .global ecaz_qjl32_sve_mul_f32
    .hidden ecaz_qjl32_sve_mul_f32
    .type ecaz_qjl32_sve_mul_f32, %function
ecaz_qjl32_sve_mul_f32:
    mov x4, #0
1:
    whilelt p0.s, x4, x3
    b.none 2f
    ld1w z0.s, p0/z, [x1, x4, lsl #2]
    ld1w z1.s, p0/z, [x2, x4, lsl #2]
    fmul z0.s, z0.s, z1.s
    st1w z0.s, p0, [x0, x4, lsl #2]
    incw x4
    b 1b
2:
    ret

    .global ecaz_qjl32_sve_cntw
    .hidden ecaz_qjl32_sve_cntw
    .type ecaz_qjl32_sve_cntw, %function
ecaz_qjl32_sve_cntw:
    cntw x0
    ret
"#
);

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
extern "C" {
    fn ecaz_qjl32_sve_mul_f32(out: *mut f32, left: *const f32, right: *const f32, count: usize);
}

#[cfg(all(test, target_arch = "aarch64", not(target_vendor = "apple")))]
extern "C" {
    fn ecaz_qjl32_sve_cntw() -> usize;
}

pub(super) fn score_block32_sve(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
    {
        if is_aarch64_feature_detected!("sve2") {
            // SAFETY: runtime feature detection above guarantees SVE2/SVE
            // support, and callers validate qjl32 shapes before dispatch.
            return unsafe {
                score_block32_sve_impl(quantizer, prepared, codes, gammas, out_scores, Isa::Sve2)
            };
        }
        if is_aarch64_feature_detected!("sve") {
            // SAFETY: runtime feature detection above guarantees SVE support,
            // and callers validate qjl32 shapes before dispatch.
            return unsafe {
                score_block32_sve_impl(quantizer, prepared, codes, gammas, out_scores, Isa::Sve)
            };
        }
    }

    scalar::score_block32_scalar(quantizer, prepared, codes, gammas, out_scores)
}

#[cfg(test)]
pub(super) fn score_block32_sve_for_test(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
    {
        if is_aarch64_feature_detected!("sve2") {
            // SAFETY: runtime feature detection above guarantees SVE2/SVE support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe {
                score_block32_sve_impl(quantizer, prepared, codes, gammas, out_scores, Isa::Sve2)
            });
        }
        if is_aarch64_feature_detected!("sve") {
            // SAFETY: runtime feature detection above guarantees SVE support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe {
                score_block32_sve_impl(quantizer, prepared, codes, gammas, out_scores, Isa::Sve)
            });
        }
    }

    let _ = (quantizer, prepared, codes, gammas, out_scores);
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
            return Some(unsafe { ecaz_qjl32_sve_cntw() });
        }
    }
    None
}

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
unsafe fn score_block32_sve_impl(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
    isa: Isa,
) -> Isa {
    debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);

    for lane in 0..BLOCK_WIDTH {
        out_scores[lane] = score_candidate_sve(quantizer, prepared, codes[lane], gammas[lane]);
    }
    isa
}

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
unsafe fn score_candidate_sve(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    code: &[u8],
    gamma: f32,
) -> f32 {
    let (mse_packed, qjl_packed) = super::split_qjl_code_bytes(quantizer.original_dim, code);
    let mut mse_values = vec![0.0_f32; quantizer.original_dim];
    let mut qjl_signs = vec![0.0_f32; quantizer.original_dim];

    for dim_index in 0..quantizer.original_dim {
        let centroid_index = scalar::mse_index_at_3bit(mse_packed, dim_index);
        mse_values[dim_index] = quantizer.codebook[centroid_index];
        qjl_signs[dim_index] = if scalar::qjl_sign_at(qjl_packed, dim_index) {
            1.0
        } else {
            -1.0
        };
    }

    let mse_sum = mul_sum_scalar_order_sve(&mse_values, &prepared.rotated);
    let qjl_sum = mul_sum_scalar_order_sve(&qjl_signs, &prepared.sq);
    mse_sum + gamma * prepared.qjl_scale * qjl_sum
}

#[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
unsafe fn mul_sum_scalar_order_sve(left: &[f32], right: &[f32]) -> f32 {
    debug_assert_eq!(left.len(), right.len());
    let mut products = vec![0.0_f32; left.len()];
    ecaz_qjl32_sve_mul_f32(
        products.as_mut_ptr(),
        left.as_ptr(),
        right.as_ptr(),
        left.len(),
    );
    products.into_iter().sum()
}
