use super::{neon, BLOCK_WIDTH};
use crate::quant::isa::Isa;
use crate::quant::prod::Int8ApproxNoQjl4BitQuery;

/// SVE backend placeholder: SVE hosts route through the NEON backend until
/// a real SVE kernel lands on the Graviton lane (rabitq32/hamming32
/// policy). `Isa::Sve`/`Sve2` are never reported until then.
pub(super) fn score_block32_sve(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    neon::score_block32_neon(prepared, original_dim, codes, out_scores)
}
