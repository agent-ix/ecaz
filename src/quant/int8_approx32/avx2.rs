use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;
use crate::quant::prod::Int8ApproxNoQjl4BitQuery;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::is_x86_feature_detected;

/// AVX2 backend. Per 32 packed bytes (64 dims): nibble-split, codebook
/// dequant via `vpshufb` (16-entry i8 table broadcast to both 128-bit
/// lanes), byte-interleave of the even/odd dequant vectors back into
/// natural dim order so the rotated query loads stay contiguous, then
/// sign-extension to i16 and `vpmaddwd` pair-sums into i32 lanes.
///
/// `vpmaddubsw` (the instruction named in the Task 98 deferral) is NOT
/// used: it saturates the i16 pair sums, and with full-range i8 operands
/// the ±128 corner (128 * 128 * 2 = 32768 > i16::MAX) would silently break
/// the family's integer-exact contract. The widen-then-`vpmaddwd` pair sums
/// are exact for all i8 inputs, and the i32 accumulation is
/// order-independent, so results are bit-identical to the scalar reference.
pub(super) fn score_block32_avx2(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);
            for (code, out) in codes.iter().zip(out_scores.iter_mut()) {
                // SAFETY: AVX2 detected above; the batch wrapper validated
                // code lengths (>= ceil(original_dim / 2) packed bytes) and
                // the prepared query carries exactly original_dim rotated
                // values.
                *out = unsafe { score_candidate_avx2(prepared, original_dim, code) };
            }
            return Isa::Avx2;
        }
    }

    scalar::score_block32_scalar(prepared, original_dim, codes, out_scores)
}

pub(super) fn score_partial_avx2(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            debug_assert_eq!(out_scores.len(), codes.len());
            for (code, out) in codes.iter().zip(out_scores.iter_mut()) {
                // SAFETY: as in score_block32_avx2.
                *out = unsafe { score_candidate_avx2(prepared, original_dim, code) };
            }
            return Isa::Avx2;
        }
    }

    scalar::score_partial_scalar(prepared, original_dim, codes, out_scores)
}

#[cfg(target_arch = "x86")]
use std::arch::x86 as arch;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64 as arch;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use arch::{
    __m128i, __m256i, _mm256_add_epi32, _mm256_and_si256, _mm256_broadcastsi128_si256,
    _mm256_castsi256_si128, _mm256_cvtepi8_epi16, _mm256_extracti128_si256, _mm256_loadu_si256,
    _mm256_madd_epi16, _mm256_permute2x128_si256, _mm256_set1_epi8, _mm256_setzero_si256,
    _mm256_shuffle_epi8, _mm256_srli_epi16, _mm256_unpackhi_epi8, _mm256_unpacklo_epi8,
    _mm_add_epi32, _mm_cvtsi128_si32, _mm_loadu_si128, _mm_shuffle_epi32, _mm_unpackhi_epi64,
};

/// # Safety
///
/// Caller must confirm AVX2 availability, `mse_packed.len() >=
/// ceil(original_dim / 2)`, and `prepared.rotated.len() == original_dim`.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn score_candidate_avx2(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    mse_packed: &[u8],
) -> f32 {
    if prepared.score_scale == 0.0 {
        return 0.0;
    }

    let codebook = _mm256_broadcastsi128_si256(_mm_loadu_si128(
        prepared.codebook.as_ptr() as *const __m128i
    ));
    let nibble_mask = _mm256_set1_epi8(0x0F);
    let mut acc = _mm256_setzero_si256();

    let full_chunks = original_dim / 64;
    let mut dim_index = 0usize;
    for chunk in 0..full_chunks {
        let byte_base = chunk * 32;
        // SAFETY: byte_base + 32 <= ceil(original_dim / 2) within full
        // 64-dim chunks; dim_index + 64 <= original_dim for rotated.
        let packed = _mm256_loadu_si256(mse_packed.as_ptr().add(byte_base) as *const __m256i);
        let low_indices = _mm256_and_si256(packed, nibble_mask);
        let high_indices = _mm256_and_si256(_mm256_srli_epi16::<4>(packed), nibble_mask);
        // Low nibbles dequantize even dims, high nibbles odd dims. Indexes
        // are masked to 0..15, so vpshufb's sign-bit zeroing never fires.
        let dequant_even = _mm256_shuffle_epi8(codebook, low_indices);
        let dequant_odd = _mm256_shuffle_epi8(codebook, high_indices);
        // Re-interleave even/odd back into natural dim order. Byte unpack
        // works per 128-bit lane: lo = (dims 0..16 | 32..48),
        // hi = (dims 16..32 | 48..64); the cross-lane permutes restore
        // contiguous halves so plain rotated loads line up.
        let inter_lo = _mm256_unpacklo_epi8(dequant_even, dequant_odd);
        let inter_hi = _mm256_unpackhi_epi8(dequant_even, dequant_odd);
        let dequant_first = _mm256_permute2x128_si256::<0x20>(inter_lo, inter_hi);
        let dequant_second = _mm256_permute2x128_si256::<0x31>(inter_lo, inter_hi);
        let rotated_first =
            _mm256_loadu_si256(prepared.rotated.as_ptr().add(dim_index) as *const __m256i);
        let rotated_second =
            _mm256_loadu_si256(prepared.rotated.as_ptr().add(dim_index + 32) as *const __m256i);
        acc = _mm256_add_epi32(acc, madd_i8_vectors(dequant_first, rotated_first));
        acc = _mm256_add_epi32(acc, madd_i8_vectors(dequant_second, rotated_second));
        dim_index += 64;
    }

    let mut sum = horizontal_sum_epi32(acc);

    // Scalar tail for the trailing < 64 dims, identical to the reference.
    while dim_index < original_dim {
        let packed = mse_packed[dim_index / 2];
        let nibble = if dim_index % 2 == 0 {
            packed & 0x0F
        } else {
            packed >> 4
        } as usize;
        sum += prepared.codebook[nibble] as i32 * prepared.rotated[dim_index] as i32;
        dim_index += 1;
    }

    sum as f32 * prepared.score_scale
}

/// Exact i8×i8 dot-product partial sums: sign-extend both operands to i16
/// per 128-bit half, `vpmaddwd` the pairs (|sum| <= 2 * 128 * 128, exact in
/// i32), and add the halves. Each output i32 lane holds four products.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn madd_i8_vectors(a: __m256i, b: __m256i) -> __m256i {
    let a_lo = _mm256_cvtepi8_epi16(_mm256_castsi256_si128(a));
    let a_hi = _mm256_cvtepi8_epi16(_mm256_extracti128_si256::<1>(a));
    let b_lo = _mm256_cvtepi8_epi16(_mm256_castsi256_si128(b));
    let b_hi = _mm256_cvtepi8_epi16(_mm256_extracti128_si256::<1>(b));
    _mm256_add_epi32(
        _mm256_madd_epi16(a_lo, b_lo),
        _mm256_madd_epi16(a_hi, b_hi),
    )
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn horizontal_sum_epi32(acc: __m256i) -> i32 {
    let sum128 = _mm_add_epi32(
        _mm256_castsi256_si128(acc),
        _mm256_extracti128_si256::<1>(acc),
    );
    let sum64 = _mm_add_epi32(sum128, _mm_unpackhi_epi64(sum128, sum128));
    let sum32 = _mm_add_epi32(sum64, _mm_shuffle_epi32::<0b01>(sum64));
    _mm_cvtsi128_si32(sum32)
}
