use super::BLOCK_WIDTH;
use crate::quant::isa::Isa;

pub(super) fn score_block32_scalar(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);

    let mut sums = [0.0_f32; BLOCK_WIDTH];
    for dim_index in 0..original_dim {
        let lut_offset = dim_index * 16;
        let byte_index = dim_index / 2;
        if dim_index & 1 == 0 {
            for lane in 0..BLOCK_WIDTH {
                let centroid = (codes[lane][byte_index] & 0x0F) as usize;
                sums[lane] += lut[lut_offset + centroid];
            }
        } else {
            for lane in 0..BLOCK_WIDTH {
                let centroid = (codes[lane][byte_index] >> 4) as usize;
                sums[lane] += lut[lut_offset + centroid];
            }
        }
    }
    out_scores.copy_from_slice(&sums);
    Isa::Scalar
}

#[allow(dead_code)]
pub(super) fn score_scalar_tail(lut: &[f32], original_dim: usize, code: &[u8]) -> f32 {
    let mut sum = 0.0_f32;
    let mut dim_index = 0usize;

    for &packed in code {
        if dim_index >= original_dim {
            break;
        }

        let low_nibble = (packed & 0x0F) as usize;
        sum += lut[dim_index * 16 + low_nibble];
        dim_index += 1;

        if dim_index >= original_dim {
            break;
        }

        let high_nibble = (packed >> 4) as usize;
        sum += lut[dim_index * 16 + high_nibble];
        dim_index += 1;
    }

    sum
}
