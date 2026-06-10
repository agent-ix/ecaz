use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;
use crate::quant::prod::{PreparedQuery, ProdQuantizer};

#[cfg(target_arch = "aarch64")]
use std::arch::is_aarch64_feature_detected;

pub(super) fn score_block32_neon(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support,
            // and callers validate qjl32 shapes before dispatch.
            for lane in 0..BLOCK_WIDTH {
                out_scores[lane] =
                    unsafe { score_candidate_neon(quantizer, prepared, codes[lane], gammas[lane]) };
            }
            return Isa::Neon;
        }
    }

    scalar::score_block32_scalar(quantizer, prepared, codes, gammas, out_scores)
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_candidate_neon(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    code: &[u8],
    gamma: f32,
) -> f32 {
    let (mse_packed, qjl_packed) = super::split_qjl_code_bytes(quantizer.original_dim, code);
    let shifts = [0_u32, 3, 6, 9, 12, 15, 18, 21];
    let mut products = [0.0_f32; 4];
    let mut mse_sum = 0.0_f32;
    let mut qjl_sum = 0.0_f32;
    let mut dim_index = 0usize;

    while dim_index + 8 <= quantizer.original_dim {
        let word = decode_eight_3bit_aligned_word(mse_packed, dim_index);
        let indices = [
            ((word >> shifts[0]) & 0x7) as usize,
            ((word >> shifts[1]) & 0x7) as usize,
            ((word >> shifts[2]) & 0x7) as usize,
            ((word >> shifts[3]) & 0x7) as usize,
            ((word >> shifts[4]) & 0x7) as usize,
            ((word >> shifts[5]) & 0x7) as usize,
            ((word >> shifts[6]) & 0x7) as usize,
            ((word >> shifts[7]) & 0x7) as usize,
        ];
        let signs = qjl_sign_lanes(qjl_packed[dim_index / 8]);

        accumulate_four_neon(
            &mut mse_sum,
            [
                quantizer.codebook[indices[0]],
                quantizer.codebook[indices[1]],
                quantizer.codebook[indices[2]],
                quantizer.codebook[indices[3]],
            ],
            &prepared.rotated[dim_index..dim_index + 4],
            &mut products,
        );
        accumulate_four_neon(
            &mut qjl_sum,
            [signs[0], signs[1], signs[2], signs[3]],
            &prepared.sq[dim_index..dim_index + 4],
            &mut products,
        );
        accumulate_four_neon(
            &mut mse_sum,
            [
                quantizer.codebook[indices[4]],
                quantizer.codebook[indices[5]],
                quantizer.codebook[indices[6]],
                quantizer.codebook[indices[7]],
            ],
            &prepared.rotated[dim_index + 4..dim_index + 8],
            &mut products,
        );
        accumulate_four_neon(
            &mut qjl_sum,
            [signs[4], signs[5], signs[6], signs[7]],
            &prepared.sq[dim_index + 4..dim_index + 8],
            &mut products,
        );

        dim_index += 8;
    }

    while dim_index < quantizer.original_dim {
        let centroid_index = scalar::mse_index_at_3bit(mse_packed, dim_index);
        mse_sum += quantizer.codebook[centroid_index] * prepared.rotated[dim_index];
        qjl_sum += if scalar::qjl_sign_at(qjl_packed, dim_index) {
            prepared.sq[dim_index]
        } else {
            -prepared.sq[dim_index]
        };
        dim_index += 1;
    }

    mse_sum + gamma * prepared.qjl_scale * qjl_sum
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn accumulate_four_neon(
    sum: &mut f32,
    left: [f32; 4],
    right: &[f32],
    products: &mut [f32; 4],
) {
    use std::arch::aarch64::{vld1q_f32, vmulq_f32, vst1q_f32};

    let left = vld1q_f32(left.as_ptr());
    let right = vld1q_f32(right.as_ptr());
    vst1q_f32(products.as_mut_ptr(), vmulq_f32(left, right));
    for product in products {
        *sum += *product;
    }
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn decode_eight_3bit_aligned_word(packed: &[u8], dim_index: usize) -> u32 {
    debug_assert_eq!(dim_index % 8, 0);
    let byte_index = (dim_index / 8) * 3;
    u32::from_le_bytes([
        packed[byte_index],
        packed[byte_index + 1],
        packed[byte_index + 2],
        0,
    ])
}

#[cfg(target_arch = "aarch64")]
#[inline]
fn qjl_sign_lanes(byte: u8) -> &'static [f32; 8] {
    static LUT: [[f32; 8]; 256] = build_qjl_sign_lut();
    &LUT[byte as usize]
}

#[cfg(target_arch = "aarch64")]
const fn build_qjl_sign_lut() -> [[f32; 8]; 256] {
    let mut lut = [[0.0; 8]; 256];
    let mut byte = 0usize;
    while byte < 256 {
        let mut lane = 0usize;
        while lane < 8 {
            lut[byte][lane] = if ((byte >> lane) & 1) == 1 { 1.0 } else { -1.0 };
            lane += 1;
        }
        byte += 1;
    }
    lut
}
