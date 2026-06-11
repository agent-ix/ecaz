use super::{split_qjl_code_bytes, BLOCK_WIDTH};
use crate::quant::isa::Isa;
use crate::quant::prod::{PreparedQuery, ProdQuantizer};

pub(super) fn score_block32_scalar(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    debug_assert_eq!(out_scores.len(), BLOCK_WIDTH);
    for lane in 0..BLOCK_WIDTH {
        out_scores[lane] = score_scalar_tail(quantizer, prepared, codes[lane], gammas[lane]);
    }
    Isa::Scalar
}

pub(super) fn score_scalar_tail(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    code: &[u8],
    gamma: f32,
) -> f32 {
    let (mse_packed, qjl_packed) = split_qjl_code_bytes(quantizer.original_dim, code);
    let mut mse_sum = 0.0_f32;
    let mut qjl_sum = 0.0_f32;
    let mut dim_index = 0usize;

    while dim_index + 8 <= quantizer.original_dim {
        let indices = decode_eight_3bit_aligned(mse_packed, dim_index);
        let sign_lanes = qjl_sign_lanes(qjl_packed[dim_index / 8]);
        for lane in 0..8 {
            let absolute = dim_index + lane;
            mse_sum += quantizer.codebook[indices[lane]] * prepared.rotated[absolute];
            qjl_sum += prepared.sq[absolute] * sign_lanes[lane];
        }
        dim_index += 8;
    }

    while dim_index < quantizer.original_dim {
        let centroid_index = mse_index_at_3bit(mse_packed, dim_index);
        mse_sum += quantizer.codebook[centroid_index] * prepared.rotated[dim_index];
        qjl_sum += if qjl_sign_at(qjl_packed, dim_index) {
            prepared.sq[dim_index]
        } else {
            -prepared.sq[dim_index]
        };
        dim_index += 1;
    }

    mse_sum + gamma * prepared.qjl_scale * qjl_sum
}

fn decode_eight_3bit_aligned(packed: &[u8], dim_index: usize) -> [usize; 8] {
    debug_assert_eq!(dim_index % 8, 0);
    let byte_index = (dim_index / 8) * 3;
    let word = u32::from_le_bytes([
        packed[byte_index],
        packed[byte_index + 1],
        packed[byte_index + 2],
        0,
    ]);
    [
        (word & 0x7) as usize,
        ((word >> 3) & 0x7) as usize,
        ((word >> 6) & 0x7) as usize,
        ((word >> 9) & 0x7) as usize,
        ((word >> 12) & 0x7) as usize,
        ((word >> 15) & 0x7) as usize,
        ((word >> 18) & 0x7) as usize,
        ((word >> 21) & 0x7) as usize,
    ]
}

pub(super) fn mse_index_at_3bit(packed: &[u8], dim_index: usize) -> usize {
    let bit_offset = dim_index * 3;
    let byte_index = bit_offset / 8;
    let bit_shift = bit_offset % 8;
    let mut word = packed[byte_index] as u32;
    if byte_index + 1 < packed.len() {
        word |= (packed[byte_index + 1] as u32) << 8;
    }
    ((word >> bit_shift) & 0x7) as usize
}

pub(super) fn qjl_sign_at(packed: &[u8], dim_index: usize) -> bool {
    (packed[dim_index / 8] >> (dim_index % 8)) & 1 == 1
}

fn qjl_sign_lanes(byte: u8) -> &'static [f32; 8] {
    static LUT: [[f32; 8]; 256] = build_qjl_sign_lut();
    &LUT[byte as usize]
}

const fn build_qjl_sign_lut() -> [[f32; 8]; 256] {
    let mut lut = [[0.0; 8]; 256];
    let mut byte = 0usize;
    while byte < 256 {
        let mut lane = 0usize;
        while lane < 8 {
            lut[byte][lane] = if ((byte >> lane) & 1) == 1 { 1.0 } else { -1.0 };
            lane += 1;
        }
        byte += 1;
    }
    lut
}
