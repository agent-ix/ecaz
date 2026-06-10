use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;
use crate::quant::prod::Int8ApproxNoQjl4BitQuery;

/// AVX2 backend placeholder pending the Intel-lane slice (`vpmaddubsw`-style
/// i8 dot products are the candidate strategy). x86_64 hosts use the scalar
/// path and counter rows report `isa=scalar` truthfully until it lands.
pub(super) fn score_block32_avx2(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    scalar::score_block32_scalar(prepared, original_dim, codes, out_scores)
}

pub(super) fn score_partial_avx2(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    scalar::score_partial_scalar(prepared, original_dim, codes, out_scores)
}
