use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;
use crate::quant::prod::{PreparedQuery, ProdQuantizer};

#[cfg(target_arch = "aarch64")]
use std::arch::is_aarch64_feature_detected;

const OCTET_WIDTH: usize = 8;

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
            unsafe {
                score_block32_candidate_parallel_neon(
                    quantizer, prepared, codes, gammas, out_scores,
                )
            };
            return Isa::Neon;
        }
    }

    scalar::score_block32_scalar(quantizer, prepared, codes, gammas, out_scores)
}

/// NEON octet entry for the 8-31-candidate remainder band of the width
/// cascade — the aarch64 counterpart of `avx2::score_octet8_avx2`. Without
/// it the remainder fell back to scalar on Apple silicon (Task 104 packet
/// 007 reviewer finding: 8-15/16-31-wide flushes carried the bulk of the
/// HNSW/SPIRE QJL kernel-on scalar share).
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
            // and callers validate qjl32 shapes before dispatch.
            unsafe {
                score_octet8_candidate_parallel_neon(quantizer, prepared, codes, gammas, out_scores)
            };
            return Some(Isa::Neon);
        }
    }

    let _ = (quantizer, prepared, codes, gammas, out_scores);
    None
}

#[cfg(test)]
pub(super) fn score_block32_neon_for_test(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support;
            // test fixtures use the same validated shapes as the public block path.
            unsafe {
                score_block32_candidate_parallel_neon(
                    quantizer, prepared, codes, gammas, out_scores,
                )
            };
            return Some(Isa::Neon);
        }
    }

    let _ = (quantizer, prepared, codes, gammas, out_scores);
    None
}

/// Candidate-parallel qjl32 block kernel (Task 104). Mirrors the AVX2
/// octet design: candidates occupy vector lanes and dimensions iterate
/// sequentially, so each candidate keeps the scalar accumulation order
/// (separate multiply and add roundings, one accumulator per lane) and
/// stays inside the family's 4-ulp pre-slice tolerance contract. The
/// 8-entry 3-bit codebook lives in a 32-byte tbl register pair, so the
/// per-dimension codebook gather is a `vqtbl2q_u8` byte shuffle (the
/// NEON analogue of `_mm256_permutevar8x32_ps`) instead of scalar loads.
/// Load the eight-entry 3-bit codebook into the 32-byte tbl register pair.
///
/// # Safety
/// Requires NEON and a codebook of exactly eight f32s.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn load_codebook_table(quantizer: &ProdQuantizer) -> std::arch::aarch64::uint8x16x2_t {
    use std::arch::aarch64::{uint8x16x2_t, vld1q_u8};

    debug_assert_eq!(quantizer.codebook.len(), 8);
    // SAFETY: the 3-bit lane codebook is exactly eight contiguous f32s,
    // reinterpreted as the 32-byte tbl source.
    let table_bytes = std::slice::from_raw_parts(quantizer.codebook.as_ptr().cast::<u8>(), 32);
    uint8x16x2_t(
        vld1q_u8(table_bytes.as_ptr()),
        vld1q_u8(table_bytes.as_ptr().add(16)),
    )
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
    let cb_table = load_codebook_table(quantizer);
    score_octet_candidate_parallel_neon(
        quantizer,
        prepared,
        codes.as_slice(),
        gammas.as_slice(),
        out_scores,
        0,
        cb_table,
    );
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
    let cb_table = load_codebook_table(quantizer);

    let mut block_lane = 0usize;
    while block_lane < BLOCK_WIDTH {
        score_octet_candidate_parallel_neon(
            quantizer,
            prepared,
            codes.as_slice(),
            gammas.as_slice(),
            out_scores,
            block_lane,
            cb_table,
        );
        block_lane += OCTET_WIDTH;
    }
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
    cb_table: std::arch::aarch64::uint8x16x2_t,
) {
    use std::arch::aarch64::{
        vaddq_f32, vaddq_u32, vandq_u32, vbslq_f32, vceqq_u32, vdupq_n_f32, vdupq_n_s32,
        vdupq_n_u32, vld1q_f32, vld1q_u32, vmulq_f32, vmulq_u32, vqtbl2q_u8,
        vreinterpretq_f32_u8, vreinterpretq_u8_u32, vshlq_n_u32, vshlq_u32, vst1q_f32,
    };

    let mut mse_codes = [&[][..]; OCTET_WIDTH];
    let mut qjl_codes = [&[][..]; OCTET_WIDTH];
    for lane in 0..OCTET_WIDTH {
        let (mse_packed, qjl_packed) =
            super::split_qjl_code_bytes(quantizer.original_dim, codes[block_lane + lane]);
        mse_codes[lane] = mse_packed;
        qjl_codes[lane] = qjl_packed;
    }

    let mask7 = vdupq_n_u32(0x7);
    let bit1 = vdupq_n_u32(1);
    let byte_spread = vdupq_n_u32(0x0101_0101);
    let byte_offsets = vdupq_n_u32(0x0302_0100);
    let one = vdupq_n_f32(1.0);
    let neg_one = vdupq_n_f32(-1.0);

    let mut mse_acc0 = vdupq_n_f32(0.0);
    let mut mse_acc1 = vdupq_n_f32(0.0);
    let mut qjl_acc0 = vdupq_n_f32(0.0);
    let mut qjl_acc1 = vdupq_n_f32(0.0);

    let mut dim_index = 0usize;
    while dim_index + 8 <= quantizer.original_dim {
        let mut words = [0_u32; OCTET_WIDTH];
        let mut sign_bytes = [0_u32; OCTET_WIDTH];
        for lane in 0..OCTET_WIDTH {
            words[lane] = decode_eight_3bit_aligned_word(mse_codes[lane], dim_index);
            sign_bytes[lane] = u32::from(qjl_codes[lane][dim_index / 8]);
        }
        let words_lo = vld1q_u32(words.as_ptr());
        let words_hi = vld1q_u32(words.as_ptr().add(4));
        let signs_lo = vld1q_u32(sign_bytes.as_ptr());
        let signs_hi = vld1q_u32(sign_bytes.as_ptr().add(4));

        for subdim in 0..8 {
            let index_shift = vdupq_n_s32(-((subdim * 3) as i32));
            let sign_shift = vdupq_n_s32(-(subdim as i32));

            let idx_lo = vandq_u32(vshlq_u32(words_lo, index_shift), mask7);
            let idx_hi = vandq_u32(vshlq_u32(words_hi, index_shift), mask7);
            let bytes_lo = vaddq_u32(
                vmulq_u32(vshlq_n_u32::<2>(idx_lo), byte_spread),
                byte_offsets,
            );
            let bytes_hi = vaddq_u32(
                vmulq_u32(vshlq_n_u32::<2>(idx_hi), byte_spread),
                byte_offsets,
            );
            let cb_lo =
                vreinterpretq_f32_u8(vqtbl2q_u8(cb_table, vreinterpretq_u8_u32(bytes_lo)));
            let cb_hi =
                vreinterpretq_f32_u8(vqtbl2q_u8(cb_table, vreinterpretq_u8_u32(bytes_hi)));

            let absolute = dim_index + subdim;
            let rotated = vdupq_n_f32(*prepared.rotated.get_unchecked(absolute));
            mse_acc0 = vaddq_f32(mse_acc0, vmulq_f32(cb_lo, rotated));
            mse_acc1 = vaddq_f32(mse_acc1, vmulq_f32(cb_hi, rotated));

            let bit_lo = vandq_u32(vshlq_u32(signs_lo, sign_shift), bit1);
            let bit_hi = vandq_u32(vshlq_u32(signs_hi, sign_shift), bit1);
            let sign_lo = vbslq_f32(vceqq_u32(bit_lo, bit1), one, neg_one);
            let sign_hi = vbslq_f32(vceqq_u32(bit_hi, bit1), one, neg_one);
            let sq = vdupq_n_f32(*prepared.sq.get_unchecked(absolute));
            qjl_acc0 = vaddq_f32(qjl_acc0, vmulq_f32(sign_lo, sq));
            qjl_acc1 = vaddq_f32(qjl_acc1, vmulq_f32(sign_hi, sq));
        }

        dim_index += 8;
    }

    while dim_index < quantizer.original_dim {
        let mut cb_values = [0.0_f32; OCTET_WIDTH];
        let mut sign_values = [0.0_f32; OCTET_WIDTH];
        for lane in 0..OCTET_WIDTH {
            cb_values[lane] =
                quantizer.codebook[scalar::mse_index_at_3bit(mse_codes[lane], dim_index)];
            sign_values[lane] = if scalar::qjl_sign_at(qjl_codes[lane], dim_index) {
                1.0
            } else {
                -1.0
            };
        }
        let rotated = vdupq_n_f32(prepared.rotated[dim_index]);
        mse_acc0 = vaddq_f32(mse_acc0, vmulq_f32(vld1q_f32(cb_values.as_ptr()), rotated));
        mse_acc1 = vaddq_f32(
            mse_acc1,
            vmulq_f32(vld1q_f32(cb_values.as_ptr().add(4)), rotated),
        );
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
