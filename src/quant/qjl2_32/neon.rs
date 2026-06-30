use super::{
    bit_at, score_turboquant_qjl2_scalar, split_qjl2_code_bytes, BLOCK_WIDTH, OCTET_WIDTH,
};
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
            // and callers validate qjl2_32 shapes before dispatch.
            unsafe {
                score_block32_candidate_parallel_neon(
                    quantizer, prepared, codes, gammas, out_scores,
                )
            };
            return Isa::Neon;
        }
    }

    for lane in 0..BLOCK_WIDTH {
        out_scores[lane] =
            score_turboquant_qjl2_scalar(quantizer, prepared, codes[lane], gammas[lane]);
    }
    Isa::Scalar
}

pub(super) fn score_octet8_neon(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; OCTET_WIDTH],
    gammas: &[f32; OCTET_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support,
            // and callers validate qjl2_32 shapes before dispatch.
            unsafe {
                score_octet8_candidate_parallel_neon(quantizer, prepared, codes, gammas, out_scores)
            };
            return Some(Isa::Neon);
        }
    }

    let _ = (quantizer, prepared, codes, gammas, out_scores);
    None
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_block32_candidate_parallel_neon(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) {
    let mut block_lane = 0usize;
    while block_lane < BLOCK_WIDTH {
        score_octet_candidate_parallel_neon(
            quantizer,
            prepared,
            codes.as_slice(),
            gammas.as_slice(),
            out_scores,
            block_lane,
        );
        block_lane += OCTET_WIDTH;
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_octet8_candidate_parallel_neon(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; OCTET_WIDTH],
    gammas: &[f32; OCTET_WIDTH],
    out_scores: &mut [f32],
) {
    score_octet_candidate_parallel_neon(
        quantizer,
        prepared,
        codes.as_slice(),
        gammas.as_slice(),
        out_scores,
        0,
    );
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_octet_candidate_parallel_neon(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]],
    gammas: &[f32],
    out_scores: &mut [f32],
    block_lane: usize,
) {
    use std::arch::aarch64::{
        vaddq_f32, vandq_u32, vbslq_f32, vceqq_u32, vdupq_n_f32, vdupq_n_s32, vdupq_n_u32,
        vld1q_f32, vld1q_u32, vmulq_f32, vshlq_u32, vst1q_f32,
    };

    let mut mse_codes = [&[][..]; OCTET_WIDTH];
    let mut qjl_codes = [&[][..]; OCTET_WIDTH];
    for lane in 0..OCTET_WIDTH {
        let (mse_packed, qjl_packed) =
            split_qjl2_code_bytes(quantizer.original_dim, codes[block_lane + lane]);
        mse_codes[lane] = mse_packed;
        qjl_codes[lane] = qjl_packed;
    }

    let bit1 = vdupq_n_u32(1);
    let one = vdupq_n_f32(1.0);
    let neg_one = vdupq_n_f32(-1.0);
    let mut mse_acc0 = vdupq_n_f32(0.0);
    let mut mse_acc1 = vdupq_n_f32(0.0);
    let mut qjl_acc0 = vdupq_n_f32(0.0);
    let mut qjl_acc1 = vdupq_n_f32(0.0);

    let mut dim_index = 0usize;
    while dim_index + 8 <= quantizer.original_dim {
        let mut mse_bytes = [0_u32; OCTET_WIDTH];
        let mut qjl_bytes = [0_u32; OCTET_WIDTH];
        for lane in 0..OCTET_WIDTH {
            mse_bytes[lane] = u32::from(mse_codes[lane][dim_index / 8]);
            qjl_bytes[lane] = u32::from(qjl_codes[lane][dim_index / 8]);
        }
        let mse_lo = vld1q_u32(mse_bytes.as_ptr());
        let mse_hi = vld1q_u32(mse_bytes.as_ptr().add(4));
        let qjl_lo = vld1q_u32(qjl_bytes.as_ptr());
        let qjl_hi = vld1q_u32(qjl_bytes.as_ptr().add(4));

        for subdim in 0..8 {
            let shift = vdupq_n_s32(-(subdim as i32));
            let absolute = dim_index + subdim;

            let mse_bit_lo = vandq_u32(vshlq_u32(mse_lo, shift), bit1);
            let mse_bit_hi = vandq_u32(vshlq_u32(mse_hi, shift), bit1);
            let lut0 = vdupq_n_f32(*prepared.lut.get_unchecked(absolute * 2));
            let lut1 = vdupq_n_f32(*prepared.lut.get_unchecked(absolute * 2 + 1));
            mse_acc0 = vaddq_f32(mse_acc0, vbslq_f32(vceqq_u32(mse_bit_lo, bit1), lut1, lut0));
            mse_acc1 = vaddq_f32(mse_acc1, vbslq_f32(vceqq_u32(mse_bit_hi, bit1), lut1, lut0));

            let sign_bit_lo = vandq_u32(vshlq_u32(qjl_lo, shift), bit1);
            let sign_bit_hi = vandq_u32(vshlq_u32(qjl_hi, shift), bit1);
            let sq = vdupq_n_f32(*prepared.sq.get_unchecked(absolute));
            qjl_acc0 = vaddq_f32(
                qjl_acc0,
                vmulq_f32(vbslq_f32(vceqq_u32(sign_bit_lo, bit1), one, neg_one), sq),
            );
            qjl_acc1 = vaddq_f32(
                qjl_acc1,
                vmulq_f32(vbslq_f32(vceqq_u32(sign_bit_hi, bit1), one, neg_one), sq),
            );
        }

        dim_index += 8;
    }

    while dim_index < quantizer.original_dim {
        let mut mse_values = [0.0_f32; OCTET_WIDTH];
        let mut sign_values = [0.0_f32; OCTET_WIDTH];
        for lane in 0..OCTET_WIDTH {
            mse_values[lane] =
                prepared.lut[dim_index * 2 + usize::from(bit_at(mse_codes[lane], dim_index))];
            sign_values[lane] = if bit_at(qjl_codes[lane], dim_index) {
                1.0
            } else {
                -1.0
            };
        }
        mse_acc0 = vaddq_f32(mse_acc0, vld1q_f32(mse_values.as_ptr()));
        mse_acc1 = vaddq_f32(mse_acc1, vld1q_f32(mse_values.as_ptr().add(4)));
        let sq = vdupq_n_f32(prepared.sq[dim_index]);
        qjl_acc0 = vaddq_f32(qjl_acc0, vmulq_f32(vld1q_f32(sign_values.as_ptr()), sq));
        qjl_acc1 = vaddq_f32(
            qjl_acc1,
            vmulq_f32(vld1q_f32(sign_values.as_ptr().add(4)), sq),
        );
        dim_index += 1;
    }

    let qjl_scale = vdupq_n_f32(prepared.qjl_scale);
    let gamma_lo = vld1q_f32(gammas.as_ptr().add(block_lane));
    let gamma_hi = vld1q_f32(gammas.as_ptr().add(block_lane + 4));
    let weight_lo = vmulq_f32(gamma_lo, qjl_scale);
    let weight_hi = vmulq_f32(gamma_hi, qjl_scale);
    vst1q_f32(
        out_scores.as_mut_ptr().add(block_lane),
        vaddq_f32(mse_acc0, vmulq_f32(weight_lo, qjl_acc0)),
    );
    vst1q_f32(
        out_scores.as_mut_ptr().add(block_lane + 4),
        vaddq_f32(mse_acc1, vmulq_f32(weight_hi, qjl_acc1)),
    );
}
