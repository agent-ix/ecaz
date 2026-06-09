use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

pub(super) fn score_block32_avx2(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    scalar::score_block32_scalar(lut, original_dim, codes, out_scores)
}
