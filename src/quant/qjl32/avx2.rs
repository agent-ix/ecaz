use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;
use crate::quant::prod::{PreparedQuery, ProdQuantizer};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::is_x86_feature_detected;

pub(super) fn score_block32_avx2(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: runtime feature detection above guarantees AVX2 support,
            // and callers validate qjl32 shapes before dispatch.
            unsafe {
                score_block32_candidate_parallel_avx2(
                    quantizer, prepared, codes, gammas, out_scores,
                )
            };
            return Isa::Avx2;
        }
    }

    scalar::score_block32_scalar(quantizer, prepared, codes, gammas, out_scores)
}

#[cfg(test)]
pub(super) fn score_block32_avx2_for_test(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: runtime feature detection above guarantees AVX2 support;
            // test fixtures use the same validated shapes as the public block path.
            unsafe {
                score_block32_candidate_parallel_avx2(
                    quantizer, prepared, codes, gammas, out_scores,
                )
            };
            return Some(Isa::Avx2);
        }
    }

    let _ = (quantizer, prepared, codes, gammas, out_scores);
    None
}

#[cfg(target_arch = "x86")]
use std::arch::x86::{
    __m256, __m256i, _mm256_add_ps, _mm256_loadu_ps, _mm256_loadu_si256, _mm256_mul_ps,
    _mm256_permutevar8x32_ps, _mm256_set1_ps, _mm256_setzero_ps, _mm256_storeu_ps,
};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256, __m256i, _mm256_add_ps, _mm256_loadu_ps, _mm256_loadu_si256, _mm256_mul_ps,
    _mm256_permutevar8x32_ps, _mm256_set1_ps, _mm256_setzero_ps, _mm256_storeu_ps,
};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_block32_candidate_parallel_avx2(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) {
    let codebook = _mm256_loadu_ps(quantizer.codebook.as_ptr());
    let qjl_scale = _mm256_set1_ps(prepared.qjl_scale);

    let mut block_lane = 0usize;
    while block_lane < BLOCK_WIDTH {
        score_candidate_octet_avx2(
            quantizer, prepared, codes, gammas, out_scores, block_lane, codebook, qjl_scale,
        );
        block_lane += 8;
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_candidate_octet_avx2(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
    block_lane: usize,
    codebook: __m256,
    qjl_scale: __m256,
) {
    let mut mse_codes = [&[][..]; 8];
    let mut qjl_codes = [&[][..]; 8];
    for lane in 0..8 {
        let (mse_packed, qjl_packed) =
            super::split_qjl_code_bytes(quantizer.original_dim, codes[block_lane + lane]);
        mse_codes[lane] = mse_packed;
        qjl_codes[lane] = qjl_packed;
    }

    let mut mse_acc = _mm256_setzero_ps();
    let mut qjl_acc = _mm256_setzero_ps();
    let mut dim_index = 0usize;
    while dim_index + 8 <= quantizer.original_dim {
        let mut mse_words = [0_u32; 8];
        let mut qjl_bytes = [0_u8; 8];
        let mut lane = 0usize;
        while lane < 8 {
            mse_words[lane] = decode_eight_3bit_aligned_word(mse_codes[lane], dim_index);
            qjl_bytes[lane] = qjl_codes[lane][dim_index / 8];
            lane += 1;
        }

        let shifts = [0_u32, 3, 6, 9, 12, 15, 18, 21];
        let mut subdim = 0usize;
        while subdim < 8 {
            let mut mse_indices = [0_i32; 8];
            let mut qjl_signs = [0.0_f32; 8];
            let mut lane = 0usize;
            while lane < 8 {
                mse_indices[lane] = ((mse_words[lane] >> shifts[subdim]) & 0x7) as i32;
                qjl_signs[lane] = if ((qjl_bytes[lane] >> subdim) & 1) == 1 {
                    1.0
                } else {
                    -1.0
                };
                lane += 1;
            }

            let index_vector = _mm256_loadu_si256(mse_indices.as_ptr().cast::<__m256i>());
            let codebook_values = _mm256_permutevar8x32_ps(codebook, index_vector);
            let absolute = dim_index + subdim;
            let rotated = _mm256_set1_ps(prepared.rotated[absolute]);
            mse_acc = _mm256_add_ps(mse_acc, _mm256_mul_ps(codebook_values, rotated));

            let sign_values = _mm256_loadu_ps(qjl_signs.as_ptr());
            let sq = _mm256_set1_ps(prepared.sq[absolute]);
            qjl_acc = _mm256_add_ps(qjl_acc, _mm256_mul_ps(sign_values, sq));
            subdim += 1;
        }

        dim_index += 8;
    }

    while dim_index < quantizer.original_dim {
        let mut mse_indices = [0_i32; 8];
        let mut qjl_signs = [0.0_f32; 8];
        let mut lane = 0usize;
        while lane < 8 {
            mse_indices[lane] = scalar::mse_index_at_3bit(mse_codes[lane], dim_index) as i32;
            qjl_signs[lane] = if scalar::qjl_sign_at(qjl_codes[lane], dim_index) {
                1.0
            } else {
                -1.0
            };
            lane += 1;
        }

        let index_vector = _mm256_loadu_si256(mse_indices.as_ptr().cast::<__m256i>());
        let codebook_values = _mm256_permutevar8x32_ps(codebook, index_vector);
        let rotated = _mm256_set1_ps(prepared.rotated[dim_index]);
        mse_acc = _mm256_add_ps(mse_acc, _mm256_mul_ps(codebook_values, rotated));

        let sign_values = _mm256_loadu_ps(qjl_signs.as_ptr());
        let sq = _mm256_set1_ps(prepared.sq[dim_index]);
        qjl_acc = _mm256_add_ps(qjl_acc, _mm256_mul_ps(sign_values, sq));
        dim_index += 1;
    }

    let gamma = _mm256_loadu_ps(gammas.as_ptr().add(block_lane));
    let qjl_weight = _mm256_mul_ps(gamma, qjl_scale);
    let scores = _mm256_add_ps(mse_acc, _mm256_mul_ps(qjl_weight, qjl_acc));
    _mm256_storeu_ps(out_scores.as_mut_ptr().add(block_lane), scores);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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
