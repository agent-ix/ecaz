use super::BLOCK_WIDTH;
use crate::quant::isa::Isa;
use crate::quant::prod::Int8ApproxNoQjl4BitQuery;

pub(super) fn score_block32_scalar(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);
    for (code, out) in codes.iter().zip(out_scores.iter_mut()) {
        *out = score_candidate(prepared, original_dim, code);
    }
    Isa::Scalar
}

pub(super) fn score_partial_scalar(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), codes.len());
    for (code, out) in codes.iter().zip(out_scores.iter_mut()) {
        *out = score_candidate(prepared, original_dim, code);
    }
    Isa::Scalar
}

/// Mirrors `score_ip_from_split_parts_int8_approx_no_qjl_4bit` exactly:
/// i32 accumulation of `codebook[nibble] * rotated[dim]`, scaled once.
pub(super) fn score_candidate(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    mse_packed: &[u8],
) -> f32 {
    if prepared.score_scale == 0.0 {
        return 0.0;
    }

    let mut sum = 0_i32;
    let mut dim_index = 0usize;

    for &packed in mse_packed {
        if dim_index >= original_dim {
            break;
        }

        let low_nibble = (packed & 0x0F) as usize;
        sum += prepared.codebook[low_nibble] as i32 * prepared.rotated[dim_index] as i32;
        dim_index += 1;

        if dim_index >= original_dim {
            break;
        }

        let high_nibble = (packed >> 4) as usize;
        sum += prepared.codebook[high_nibble] as i32 * prepared.rotated[dim_index] as i32;
        dim_index += 1;
    }

    sum as f32 * prepared.score_scale
}
