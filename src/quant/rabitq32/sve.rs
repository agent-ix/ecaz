use super::{neon, PreparedBits1, BLOCK_WIDTH};
use crate::quant::isa::Isa;

/// SVE backend placeholder. The real vector-length-agnostic SVE/SVE2 kernel
/// is a Phase C (Graviton 4) deliverable; until it lands and is validated on
/// SVE hardware, SVE-capable hosts route through the NEON backend (every SVE
/// implementation includes NEON) so dispatch never degrades to forced-scalar
/// scoring. The returned ISA is whatever the NEON backend actually used —
/// `Isa::Sve`/`Isa::Sve2` are never reported until a real SVE kernel runs.
pub(super) fn score_block32_sve(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    neon::score_block32_neon(prepared, codes, out_scores)
}
