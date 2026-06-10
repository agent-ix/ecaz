use super::{PreparedBits1, BLOCK_WIDTH};
use crate::quant::isa::Isa;

pub(super) fn score_block32_scalar(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);
    for lane in 0..BLOCK_WIDTH {
        out_scores[lane] = score_scalar_tail(prepared, codes[lane]);
    }
    Isa::Scalar
}

pub(super) fn score_partial_scalar(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), codes.len());
    for (code, out_score) in codes.iter().zip(out_scores.iter_mut()) {
        *out_score = score_scalar_tail(prepared, code);
    }
    Isa::Scalar
}

pub(super) fn score_scalar_tail(prepared: PreparedBits1<'_>, code: &[u8]) -> f32 {
    let sum_q_dequant = sum_query_dequant_bits1_byte_lut_scalar(prepared, code);
    finish_scalar_only_estimate(prepared, sum_q_dequant, code)
}

#[allow(dead_code)]
pub(super) fn raw_popcount_bits1(code: &[u8], dimensions: usize) -> u32 {
    let full_bytes = dimensions / 8;
    let tail_bits = dimensions % 8;
    let mut count = code[..full_bytes]
        .iter()
        .map(|byte| byte.count_ones())
        .sum::<u32>();
    if tail_bits > 0 {
        let mask = (1_u8 << tail_bits) - 1;
        count += (code[full_bytes] & mask).count_ones();
    }
    count
}

#[inline]
fn sum_query_dequant_bits1_byte_lut_scalar(prepared: PreparedBits1<'_>, code: &[u8]) -> f32 {
    let mut sum = 0.0_f32;
    let mut dim_index = 0_usize;

    while dim_index + 8 <= prepared.dimensions {
        let row = &prepared.bits1_byte_lut[code[dim_index / 8] as usize];
        sum += prepared.query_rotated[dim_index] * row[0]
            + prepared.query_rotated[dim_index + 1] * row[1]
            + prepared.query_rotated[dim_index + 2] * row[2]
            + prepared.query_rotated[dim_index + 3] * row[3]
            + prepared.query_rotated[dim_index + 4] * row[4]
            + prepared.query_rotated[dim_index + 5] * row[5]
            + prepared.query_rotated[dim_index + 6] * row[6]
            + prepared.query_rotated[dim_index + 7] * row[7];
        dim_index += 8;
    }

    while dim_index < prepared.dimensions {
        let bit = (code[dim_index / 8] >> (dim_index % 8)) & 1;
        sum += prepared.query_rotated[dim_index] * prepared.bits1_byte_lut[usize::from(bit)][0];
        dim_index += 1;
    }

    sum
}

#[inline]
pub(super) fn finish_scalar_only_estimate(
    prepared: PreparedBits1<'_>,
    sum_q_dequant: f32,
    code: &[u8],
) -> f32 {
    let packed_bytes = prepared.dimensions.div_ceil(8);
    debug_assert!(code.len() >= packed_bytes + crate::quant::rabitq::RABITQ_SCALAR_LEN);
    let candidate_norm = f32::from_le_bytes(
        code[packed_bytes..packed_bytes + crate::quant::rabitq::RABITQ_NORM_LEN]
            .try_into()
            .expect("norm slice is always 4 bytes"),
    );
    let candidate_o_dot = f32::from_le_bytes(
        code[packed_bytes + crate::quant::rabitq::RABITQ_NORM_LEN
            ..packed_bytes
                + crate::quant::rabitq::RABITQ_NORM_LEN
                + crate::quant::rabitq::RABITQ_UNIT_DOT_LEN]
            .try_into()
            .expect("o_dot slice is always 4 bytes"),
    );
    let candidate_x_norm = f32::from_le_bytes(
        code[packed_bytes
            + crate::quant::rabitq::RABITQ_NORM_LEN
            + crate::quant::rabitq::RABITQ_UNIT_DOT_LEN
            ..packed_bytes + crate::quant::rabitq::RABITQ_SCALAR_LEN]
            .try_into()
            .expect("x_norm slice is always 4 bytes"),
    );

    const O_DOT_FLOOR: f32 = 1e-6;
    if candidate_o_dot.abs() < O_DOT_FLOOR
        || !candidate_o_dot.is_finite()
        || candidate_x_norm <= 0.0
        || !candidate_x_norm.is_finite()
    {
        return 0.0;
    }
    candidate_norm * sum_q_dequant / (candidate_o_dot * candidate_x_norm)
}
