//! 32-candidate blocked Hamming (binary fingerprint) scoring.
//!
//! Task 95: `popcount(query_words XOR candidate_words)` over `u64` sidecar
//! words. All arithmetic is integer-exact, so every ISA backend must produce
//! identical distances — parity tests use strict equality, with no ADR-076
//! tolerance machinery.

mod avx2;
mod neon;
mod scalar;
mod sve;

pub(crate) const BLOCK_WIDTH: usize = 32;

pub(crate) fn validate_word_shape(
    index: usize,
    query_word_count: usize,
    candidate_words: &[u64],
) -> Result<(), String> {
    if candidate_words.len() != query_word_count {
        return Err(format!(
            "hamming32 candidate {index} word count {} does not match query word count {}",
            candidate_words.len(),
            query_word_count
        ));
    }
    Ok(())
}

pub(crate) fn score_hamming_block32(
    query_words: &[u64],
    candidates: &[&[u64]; BLOCK_WIDTH],
    out_distances: &mut [u32],
) -> crate::quant::isa::Isa {
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Avx2 => {
            avx2::score_block32_avx2(query_words, candidates, out_distances)
        }
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve => {
            // Real SVE kernel is a Graviton-lane deliverable; SVE hosts use
            // the validated NEON backend until it lands (same policy as
            // rabitq32).
            sve::score_block32_sve(query_words, candidates, out_distances)
        }
        crate::quant::isa::Isa::Neon => {
            neon::score_block32_neon(query_words, candidates, out_distances)
        }
        crate::quant::isa::Isa::Scalar => {
            scalar::score_block32_scalar(query_words, candidates, out_distances)
        }
    }
}

/// Scores a partial batch (1..=31 candidates) through the best available
/// backend. Graph-AM batches are bounded by degree/survivor budgets and
/// rarely reach the 32-wide block (Task 93 packet 004 finding), so the
/// partial path must not fall back to scalar on SIMD hosts.
pub(crate) fn score_hamming_partial(
    query_words: &[u64],
    candidates: &[&[u64]],
    out_distances: &mut [u32],
) -> crate::quant::isa::Isa {
    debug_assert!(!candidates.is_empty() && candidates.len() < BLOCK_WIDTH);
    debug_assert_eq!(candidates.len(), out_distances.len());
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Neon
        | crate::quant::isa::Isa::Sve2
        | crate::quant::isa::Isa::Sve => {
            neon::score_partial_neon(query_words, candidates, out_distances)
        }
        crate::quant::isa::Isa::Avx2 => {
            avx2::score_partial_avx2(query_words, candidates, out_distances)
        }
        crate::quant::isa::Isa::Scalar => {
            scalar::score_partial_scalar(query_words, candidates, out_distances)
        }
    }
}

/// Forced-scalar single-candidate reference; the strict parity anchor.
#[allow(dead_code)]
pub(crate) fn hamming_distance_scalar(query_words: &[u64], candidate_words: &[u64]) -> u32 {
    scalar::hamming_distance(query_words, candidate_words)
}

/// Forced-scalar block entry for off-path scoring-share measurement.
#[cfg(feature = "bench")]
pub(crate) fn score_hamming_block32_scalar_reference(
    query_words: &[u64],
    candidates: &[&[u64]; BLOCK_WIDTH],
    out_distances: &mut [u32],
) {
    scalar::score_block32_scalar(query_words, candidates, out_distances);
}

#[cfg(test)]
mod tests {
    use super::{
        hamming_distance_scalar, score_hamming_block32, score_hamming_partial, BLOCK_WIDTH,
    };

    fn words(count: usize, seed: u64) -> Vec<u64> {
        let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
        (0..count)
            .map(|_| {
                state = state
                    .wrapping_mul(0xA076_1D64_78BD_642F)
                    .wrapping_add(0xE703_7ED1_A0B4_28DB);
                state
            })
            .collect()
    }

    fn host_expected_simd_isa() -> crate::quant::isa::Isa {
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                return crate::quant::isa::Isa::Neon;
            }
        }
        crate::quant::isa::Isa::Scalar
    }

    #[test]
    fn block32_is_integer_exact_with_scalar_reference() {
        for word_count in [1usize, 3, 12, 24, 25] {
            let query = words(word_count, 7);
            let candidates: Vec<Vec<u64>> = (0..BLOCK_WIDTH)
                .map(|seed| words(word_count, seed as u64 + 100))
                .collect();
            let candidate_refs: Vec<&[u64]> = candidates.iter().map(Vec::as_slice).collect();
            let mut distances = vec![0u32; BLOCK_WIDTH];

            let isa = score_hamming_block32(
                &query,
                candidate_refs
                    .as_slice()
                    .try_into()
                    .expect("test fixture is exactly one block"),
                &mut distances,
            );

            for (distance, candidate) in distances.iter().zip(candidate_refs.iter()) {
                assert_eq!(*distance, hamming_distance_scalar(&query, candidate));
            }
            assert_eq!(isa, host_expected_simd_isa());
        }
    }

    #[test]
    fn partial_is_integer_exact_with_scalar_reference() {
        for count in [1usize, 2, 7, 22, 31] {
            let word_count = 24;
            let query = words(word_count, 11);
            let candidates: Vec<Vec<u64>> = (0..count)
                .map(|seed| words(word_count, seed as u64 + 300))
                .collect();
            let candidate_refs: Vec<&[u64]> = candidates.iter().map(Vec::as_slice).collect();
            let mut distances = vec![0u32; count];

            score_hamming_partial(&query, &candidate_refs, &mut distances);

            for (distance, candidate) in distances.iter().zip(candidate_refs.iter()) {
                assert_eq!(*distance, hamming_distance_scalar(&query, candidate));
            }
        }
    }

    #[test]
    fn scalar_reference_is_xor_popcount() {
        let query = words(24, 31);
        let candidate = words(24, 41);
        let expected: u32 = query
            .iter()
            .zip(candidate.iter())
            .map(|(q, c)| (q ^ c).count_ones())
            .sum();
        assert_eq!(hamming_distance_scalar(&query, &candidate), expected);
        assert_eq!(hamming_distance_scalar(&query, &query), 0);
    }
}
