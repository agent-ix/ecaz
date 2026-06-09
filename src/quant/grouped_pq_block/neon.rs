use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

pub(super) fn score_block32_neon(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    scalar::score_block32_scalar(lut, group_count, codes, out_scores)
}
