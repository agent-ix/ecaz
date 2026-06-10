use super::BLOCK_WIDTH;
use crate::quant::isa::Isa;

pub(super) fn score_block32_scalar(
    query_words: &[u64],
    candidates: &[&[u64]; BLOCK_WIDTH],
    out_distances: &mut [u32],
) -> Isa {
    debug_assert_eq!(out_distances.len(), BLOCK_WIDTH);
    for (candidate, out) in candidates.iter().zip(out_distances.iter_mut()) {
        *out = hamming_distance(query_words, candidate);
    }
    Isa::Scalar
}

pub(super) fn score_partial_scalar(
    query_words: &[u64],
    candidates: &[&[u64]],
    out_distances: &mut [u32],
) -> Isa {
    debug_assert_eq!(out_distances.len(), candidates.len());
    for (candidate, out) in candidates.iter().zip(out_distances.iter_mut()) {
        *out = hamming_distance(query_words, candidate);
    }
    Isa::Scalar
}

pub(super) fn hamming_distance(query_words: &[u64], candidate_words: &[u64]) -> u32 {
    debug_assert_eq!(query_words.len(), candidate_words.len());
    query_words
        .iter()
        .zip(candidate_words.iter())
        .map(|(query, candidate)| (query ^ candidate).count_ones())
        .sum()
}
