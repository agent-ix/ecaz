use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::is_x86_feature_detected;

pub(super) fn score_block32_avx2(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: runtime feature detection above guarantees AVX2 support,
            // and callers validate LUT/code/output shapes before dispatch.
            return unsafe { score_octets_avx2_impl(lut, original_dim, codes, out_scores) };
        }
    }

    scalar::score_block32_scalar(lut, original_dim, codes, out_scores)
}

/// Octet-granular entry for sub-block tails: `codes.len()` must be a
/// nonzero multiple of 8 and at most `BLOCK_WIDTH`. Returns `None` when
/// AVX2 is unavailable so the caller can fall back.
pub(super) fn score_octets_avx2(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        debug_assert!(!codes.is_empty() && codes.len() % 8 == 0 && codes.len() <= BLOCK_WIDTH);
        if is_x86_feature_detected!("avx2") {
            // SAFETY: runtime feature detection above guarantees AVX2 support,
            // and callers validate LUT/code/output shapes before dispatch.
            return Some(unsafe { score_octets_avx2_impl(lut, original_dim, codes, out_scores) });
        }
    }

    let _ = (lut, original_dim, codes, out_scores);
    None
}

#[cfg(test)]
pub(super) fn score_block32_avx2_for_test(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: runtime feature detection above guarantees AVX2 support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe { score_octets_avx2_impl(lut, original_dim, codes, out_scores) });
        }
    }

    let _ = (lut, original_dim, codes, out_scores);
    None
}

#[cfg(target_arch = "x86")]
use std::arch::x86 as arch;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64 as arch;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use arch::{
    __m128i, __m256, __m256i, _mm256_add_ps, _mm256_and_si256, _mm256_blendv_ps,
    _mm256_castsi256_ps, _mm256_cmpgt_epi32, _mm256_cvtepu8_epi32, _mm256_loadu_ps,
    _mm256_permutevar8x32_ps, _mm256_set1_epi32, _mm256_setzero_ps, _mm256_srli_epi32,
    _mm256_storeu_ps, _mm_loadu_si128, _mm_setzero_si128, _mm_srli_si128, _mm_unpackhi_epi16,
    _mm_unpackhi_epi32, _mm_unpackhi_epi8, _mm_unpacklo_epi16, _mm_unpacklo_epi32,
    _mm_unpacklo_epi8,
};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const OCTET_COUNT: usize = BLOCK_WIDTH / 8;

/// Code bytes consumed per lane per repack chunk (each byte packs two dims).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const CHUNK_BYTES: usize = 16;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_octets_avx2_impl(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    // Shuffle-repack register-LUT scoring (the Task 94 F8 approach taken one
    // step further): code bytes are transposed from per-candidate rows into
    // per-dim columns with an unpack network once per 16-byte chunk, so the
    // scoring loop is pure SIMD — no per-dim scalar nibble extraction and no
    // scalar-to-vector store round trips. Each transposed byte feeds two
    // dims (low then high nibble); each dim's 16-entry f32 LUT is selected
    // with a permute over its two 8-float halves plus a blend on the index
    // high bit. Per-lane accumulation stays in dim order, so scores remain
    // bit-exact against the scalar block reference. Lane count is octet
    // granular (8..=32 in steps of 8) so sub-block tails only pay for the
    // octets they occupy.
    let octet_count = codes.len() / 8;
    debug_assert!(octet_count >= 1 && octet_count <= OCTET_COUNT);
    debug_assert_eq!(codes.len(), octet_count * 8);
    debug_assert_eq!(out_scores.len(), codes.len());
    let mut acc: [__m256; OCTET_COUNT] = [_mm256_setzero_ps(); OCTET_COUNT];
    let seven = _mm256_set1_epi32(7);
    let low_mask = _mm256_set1_epi32(0x0F);
    let total_bytes = super::expected_mse_code_len(original_dim);

    let mut byte_base = 0usize;
    while byte_base < total_bytes {
        let chunk_len = CHUNK_BYTES.min(total_bytes - byte_base);
        for (octet, acc_slot) in acc.iter_mut().enumerate().take(octet_count) {
            let lane_base = octet * 8;
            let mut rows = [_mm_setzero_si128(); 8];
            if chunk_len == CHUNK_BYTES {
                for (row, code) in rows.iter_mut().zip(&codes[lane_base..lane_base + 8]) {
                    *row = _mm_loadu_si128(code.as_ptr().add(byte_base).cast::<__m128i>());
                }
            } else {
                for (row, code) in rows.iter_mut().zip(&codes[lane_base..lane_base + 8]) {
                    let mut padded = [0u8; CHUNK_BYTES];
                    padded[..chunk_len].copy_from_slice(&code[byte_base..byte_base + chunk_len]);
                    *row = _mm_loadu_si128(padded.as_ptr().cast::<__m128i>());
                }
            }
            let cols = transpose_8x16(rows);

            for b in 0..chunk_len {
                let pair = cols[b / 2];
                let col = if b & 1 == 0 {
                    pair
                } else {
                    _mm_srli_si128(pair, 8)
                };
                let packed = _mm256_cvtepu8_epi32(col);
                let dim_low = (byte_base + b) * 2;
                let low_indexes = _mm256_and_si256(packed, low_mask);
                // Accumulate one dim at a time: float addition is not
                // associative, and bit-exactness requires the scalar
                // reference's per-lane dim-order sums.
                *acc_slot = _mm256_add_ps(
                    *acc_slot,
                    select_lut_entries(lut, dim_low, low_indexes, seven),
                );
                let dim_high = dim_low + 1;
                if dim_high < original_dim {
                    let high_indexes = _mm256_and_si256(_mm256_srli_epi32(packed, 4), low_mask);
                    *acc_slot = _mm256_add_ps(
                        *acc_slot,
                        select_lut_entries(lut, dim_high, high_indexes, seven),
                    );
                }
            }
        }
        byte_base += chunk_len;
    }

    for (octet, acc_slot) in acc.iter().enumerate().take(octet_count) {
        _mm256_storeu_ps(out_scores.as_mut_ptr().add(octet * 8), *acc_slot);
    }

    Isa::Avx2
}

/// Selects `lut[dim_index * 16 + index]` for eight dword lane indexes
/// (each 0..=15) from the dim's register-resident 16-entry f32 LUT.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn select_lut_entries(
    lut: &[f32],
    dim_index: usize,
    indexes: __m256i,
    seven: __m256i,
) -> __m256 {
    let lut_ptr = lut.as_ptr().add(dim_index * 16);
    let low_lut = _mm256_loadu_ps(lut_ptr);
    let high_lut = _mm256_loadu_ps(lut_ptr.add(8));
    let low_values = _mm256_permutevar8x32_ps(low_lut, indexes);
    let high_values = _mm256_permutevar8x32_ps(high_lut, indexes);
    let high_select = _mm256_castsi256_ps(_mm256_cmpgt_epi32(indexes, seven));
    _mm256_blendv_ps(low_values, high_values, high_select)
}

/// Transposes eight 16-byte candidate rows into byte columns. Output
/// element `i` packs column `2i` in its low 8 bytes and column `2i + 1`
/// in its high 8 bytes, where column `c` holds byte `c` of all eight rows.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn transpose_8x16(rows: [__m128i; 8]) -> [__m128i; 8] {
    // Pass 1 (bytes): pair rows (0,1), (2,3), (4,5), (6,7).
    let p0 = _mm_unpacklo_epi8(rows[0], rows[1]);
    let p1 = _mm_unpacklo_epi8(rows[2], rows[3]);
    let p2 = _mm_unpacklo_epi8(rows[4], rows[5]);
    let p3 = _mm_unpacklo_epi8(rows[6], rows[7]);
    let q0 = _mm_unpackhi_epi8(rows[0], rows[1]);
    let q1 = _mm_unpackhi_epi8(rows[2], rows[3]);
    let q2 = _mm_unpackhi_epi8(rows[4], rows[5]);
    let q3 = _mm_unpackhi_epi8(rows[6], rows[7]);
    // Pass 2 (words): gather four-row groups per column pair.
    let u0 = _mm_unpacklo_epi16(p0, p1);
    let u1 = _mm_unpackhi_epi16(p0, p1);
    let u2 = _mm_unpacklo_epi16(p2, p3);
    let u3 = _mm_unpackhi_epi16(p2, p3);
    let w0 = _mm_unpacklo_epi16(q0, q1);
    let w1 = _mm_unpackhi_epi16(q0, q1);
    let w2 = _mm_unpacklo_epi16(q2, q3);
    let w3 = _mm_unpackhi_epi16(q2, q3);
    // Pass 3 (dwords): stitch all eight rows; each output packs two columns.
    [
        _mm_unpacklo_epi32(u0, u2), // columns 0, 1
        _mm_unpackhi_epi32(u0, u2), // columns 2, 3
        _mm_unpacklo_epi32(u1, u3), // columns 4, 5
        _mm_unpackhi_epi32(u1, u3), // columns 6, 7
        _mm_unpacklo_epi32(w0, w2), // columns 8, 9
        _mm_unpackhi_epi32(w0, w2), // columns 10, 11
        _mm_unpacklo_epi32(w1, w3), // columns 12, 13
        _mm_unpackhi_epi32(w1, w3), // columns 14, 15
    ]
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))]
mod tests {
    use super::*;

    #[test]
    fn transpose_8x16_yields_byte_columns() {
        if !is_x86_feature_detected!("avx2") {
            return;
        }
        let mut input = [[0u8; 16]; 8];
        for (row, bytes) in input.iter_mut().enumerate() {
            for (col, byte) in bytes.iter_mut().enumerate() {
                *byte = (row * 16 + col) as u8;
            }
        }
        // SAFETY: feature detection above; plain integer shuffles on stack data.
        let cols = unsafe {
            let rows = input.map(|bytes| _mm_loadu_si128(bytes.as_ptr().cast::<__m128i>()));
            transpose_8x16(rows)
        };
        let mut out = [[0u8; 16]; 8];
        for (slot, col) in cols.iter().enumerate() {
            // SAFETY: writing 16 bytes from an xmm register to a 16-byte array.
            unsafe {
                arch::_mm_storeu_si128(out[slot].as_mut_ptr().cast::<__m128i>(), *col);
            }
        }
        for (slot, slot_bytes) in out.iter().enumerate() {
            for half in 0..2 {
                let col = slot * 2 + half;
                for row in 0..8 {
                    assert_eq!(
                        slot_bytes[half * 8 + row],
                        input[row][col],
                        "column {col} row {row}"
                    );
                }
            }
        }
    }
}
