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
            for lane in 0..BLOCK_WIDTH {
                out_scores[lane] =
                    unsafe { score_candidate_avx2(quantizer, prepared, codes[lane], gammas[lane]) };
            }
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
            for lane in 0..BLOCK_WIDTH {
                out_scores[lane] =
                    unsafe { score_candidate_avx2(quantizer, prepared, codes[lane], gammas[lane]) };
            }
            return Some(Isa::Avx2);
        }
    }

    let _ = (quantizer, prepared, codes, gammas, out_scores);
    None
}

#[cfg(target_arch = "x86")]
use std::arch::x86::{
    __m256i, _mm256_loadu_ps, _mm256_loadu_si256, _mm256_mul_ps, _mm256_permutevar8x32_ps,
    _mm256_storeu_ps,
};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256i, _mm256_loadu_ps, _mm256_loadu_si256, _mm256_mul_ps, _mm256_permutevar8x32_ps,
    _mm256_storeu_ps,
};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_candidate_avx2(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    code: &[u8],
    gamma: f32,
) -> f32 {
    let (mse_packed, qjl_packed) = super::split_qjl_code_bytes(quantizer.original_dim, code);
    let codebook = _mm256_loadu_ps(quantizer.codebook.as_ptr());
    let shifts = [0_i32, 3, 6, 9, 12, 15, 18, 21];
    let mut mse_terms = [0.0_f32; 8];
    let mut qjl_terms = [0.0_f32; 8];
    let mut mse_sum = 0.0_f32;
    let mut qjl_sum = 0.0_f32;
    let mut dim_index = 0usize;

    while dim_index + 8 <= quantizer.original_dim {
        let word = decode_eight_3bit_aligned_word(mse_packed, dim_index);
        let indices = [
            ((word >> shifts[0]) & 0x7) as i32,
            ((word >> shifts[1]) & 0x7) as i32,
            ((word >> shifts[2]) & 0x7) as i32,
            ((word >> shifts[3]) & 0x7) as i32,
            ((word >> shifts[4]) & 0x7) as i32,
            ((word >> shifts[5]) & 0x7) as i32,
            ((word >> shifts[6]) & 0x7) as i32,
            ((word >> shifts[7]) & 0x7) as i32,
        ];
        let index_vector = _mm256_loadu_si256(indices.as_ptr().cast::<__m256i>());
        let codebook_values = _mm256_permutevar8x32_ps(codebook, index_vector);
        let rotated = _mm256_loadu_ps(prepared.rotated.as_ptr().add(dim_index));
        _mm256_storeu_ps(
            mse_terms.as_mut_ptr(),
            _mm256_mul_ps(codebook_values, rotated),
        );

        let sign_values = _mm256_loadu_ps(qjl_sign_lanes(qjl_packed[dim_index / 8]).as_ptr());
        let sq = _mm256_loadu_ps(prepared.sq.as_ptr().add(dim_index));
        _mm256_storeu_ps(qjl_terms.as_mut_ptr(), _mm256_mul_ps(sign_values, sq));

        for lane in 0..8 {
            mse_sum += mse_terms[lane];
            qjl_sum += qjl_terms[lane];
        }
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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
fn qjl_sign_lanes(byte: u8) -> &'static [f32; 8] {
    static LUT: [[f32; 8]; 256] = build_qjl_sign_lut();
    &LUT[byte as usize]
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
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
