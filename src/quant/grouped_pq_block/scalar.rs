use super::BLOCK_WIDTH;
use crate::quant::isa::Isa;

pub(super) fn score_block32_scalar(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);

    for lane in 0..BLOCK_WIDTH {
        out_scores[lane] = score_scalar_tail(lut, group_count, codes[lane]);
    }
    Isa::Scalar
}

pub(super) fn score_scalar_tail(lut: &[f32], group_count: usize, code: &[u8]) -> f32 {
    let mut sum = 0.0_f32;
    for group_index in 0..group_count {
        let packed = code[group_index / 2];
        let centroid = if group_index & 1 == 0 {
            packed & 0x0F
        } else {
            packed >> 4
        } as usize;
        sum += lut[group_index * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS + centroid];
    }
    sum
}
