use super::{scalar, PreparedBits1, BLOCK_WIDTH};
use crate::quant::isa::Isa;

pub(super) fn score_block32_sve(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    scalar::score_block32_scalar(prepared, codes, out_scores)
}
