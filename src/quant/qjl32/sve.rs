use super::BLOCK_WIDTH;
use crate::quant::isa::Isa;
use crate::quant::prod::{PreparedQuery, ProdQuantizer};

pub(super) fn score_block32_sve(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    super::scalar::score_block32_scalar(quantizer, prepared, codes, gammas, out_scores)
}
