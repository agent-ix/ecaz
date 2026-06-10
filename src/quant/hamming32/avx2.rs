use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

/// AVX2 backend placeholder. x86_64 hardware `POPCNT` over `u64` words (the
/// scalar path) is already a hardware popcount; whether a nibble-LUT
/// `vpshufb` + `_mm256_sad_epu8` AVX2 kernel beats it for sidecar word
/// counts is a Phase D (Intel lane) measurement question. Until that lands,
/// x86_64 hosts use the scalar path and counter rows report `isa=scalar`.
pub(super) fn score_block32_avx2(
    query_words: &[u64],
    candidates: &[&[u64]; BLOCK_WIDTH],
    out_distances: &mut [u32],
) -> Isa {
    scalar::score_block32_scalar(query_words, candidates, out_distances)
}

pub(super) fn score_partial_avx2(
    query_words: &[u64],
    candidates: &[&[u64]],
    out_distances: &mut [u32],
) -> Isa {
    scalar::score_partial_scalar(query_words, candidates, out_distances)
}
