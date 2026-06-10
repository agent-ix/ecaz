use super::{neon, BLOCK_WIDTH};
use crate::quant::isa::Isa;

/// SVE backend placeholder. The real vector-length-agnostic SVE/SVE2 kernel
/// is a Graviton-lane deliverable; until it lands and is validated on SVE
/// hardware, SVE-capable hosts route through the NEON backend (every SVE
/// implementation includes NEON). `Isa::Sve`/`Isa::Sve2` are never reported
/// until a real SVE kernel runs.
pub(super) fn score_block32_sve(
    query_words: &[u64],
    candidates: &[&[u64]; BLOCK_WIDTH],
    out_distances: &mut [u32],
) -> Isa {
    neon::score_block32_neon(query_words, candidates, out_distances)
}
