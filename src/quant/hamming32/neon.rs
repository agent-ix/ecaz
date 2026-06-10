use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

/// NEON backend: `veorq_u8` XOR + `vcntq_u8` per-byte popcount with a u8
/// lane accumulator, reduced once per candidate via `vaddlvq_u8`-style
/// widening. Integer-exact: produces identical distances to the scalar
/// reference by construction.
#[cfg(target_arch = "aarch64")]
pub(super) fn score_block32_neon(
    query_words: &[u64],
    candidates: &[&[u64]; BLOCK_WIDTH],
    out_distances: &mut [u32],
) -> Isa {
    if std::arch::is_aarch64_feature_detected!("neon") {
        // SAFETY: runtime feature detection above proves NEON; callers
        // validated that every candidate has exactly `query_words.len()`
        // words, and the impl asserts output length.
        unsafe { score_run_neon(query_words, candidates.as_slice(), out_distances) };
        return Isa::Neon;
    }
    scalar::score_block32_scalar(query_words, candidates, out_distances)
}

#[cfg(not(target_arch = "aarch64"))]
pub(super) fn score_block32_neon(
    query_words: &[u64],
    candidates: &[&[u64]; BLOCK_WIDTH],
    out_distances: &mut [u32],
) -> Isa {
    scalar::score_block32_scalar(query_words, candidates, out_distances)
}

#[cfg(target_arch = "aarch64")]
pub(super) fn score_partial_neon(
    query_words: &[u64],
    candidates: &[&[u64]],
    out_distances: &mut [u32],
) -> Isa {
    if std::arch::is_aarch64_feature_detected!("neon") {
        // SAFETY: as for the block path; shape invariants validated by the
        // caller, output length asserted inside.
        unsafe { score_run_neon(query_words, candidates, out_distances) };
        return Isa::Neon;
    }
    scalar::score_partial_scalar(query_words, candidates, out_distances)
}

#[cfg(not(target_arch = "aarch64"))]
pub(super) fn score_partial_neon(
    query_words: &[u64],
    candidates: &[&[u64]],
    out_distances: &mut [u32],
) -> Isa {
    scalar::score_partial_scalar(query_words, candidates, out_distances)
}

/// # Safety
///
/// Caller must confirm NEON availability, `candidates[i].len() ==
/// query_words.len()` for every candidate, and `out_distances.len() ==
/// candidates.len()`.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_run_neon(query_words: &[u64], candidates: &[&[u64]], out_distances: &mut [u32]) {
    use std::arch::aarch64::{
        vaddlvq_u8, vaddq_u8, vcntq_u8, vdupq_n_u8, veorq_u8, vld1q_u64, vreinterpretq_u8_u64,
    };

    debug_assert_eq!(out_distances.len(), candidates.len());
    let word_count = query_words.len();
    let pair_words = word_count / 2 * 2;

    for (candidate, out) in candidates.iter().zip(out_distances.iter_mut()) {
        debug_assert_eq!(candidate.len(), word_count);
        // vcntq_u8 yields per-byte counts <= 8; a u8 lane accumulator is
        // safe for up to 31 packed 16-byte chunks (31 * 8 = 248 < 256).
        // Reduce every 31 chunks to stay exact for any word count.
        let mut total: u32 = 0;
        let mut acc = vdupq_n_u8(0);
        let mut chunks_in_acc = 0u32;
        let mut word_index = 0usize;
        while word_index < pair_words {
            // SAFETY: word_index + 2 <= word_count for both slices.
            let query = vreinterpretq_u8_u64(vld1q_u64(query_words.as_ptr().add(word_index)));
            let code = vreinterpretq_u8_u64(vld1q_u64(candidate.as_ptr().add(word_index)));
            acc = vaddq_u8(acc, vcntq_u8(veorq_u8(query, code)));
            chunks_in_acc += 1;
            if chunks_in_acc == 31 {
                total += u32::from(vaddlvq_u8(acc));
                acc = vdupq_n_u8(0);
                chunks_in_acc = 0;
            }
            word_index += 2;
        }
        if chunks_in_acc > 0 {
            total += u32::from(vaddlvq_u8(acc));
        }
        if word_index < word_count {
            total += (query_words[word_index] ^ candidate[word_index]).count_ones();
        }
        *out = total;
    }
}
