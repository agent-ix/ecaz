use super::{scalar, BLOCK_WIDTH};

pub(super) fn score_block32_sve(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) {
    scalar::score_block32_scalar(lut, original_dim, codes, out_scores);
}
