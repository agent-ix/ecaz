use super::{scalar, BLOCK_WIDTH};
use crate::quant::isa::Isa;

#[cfg(target_arch = "aarch64")]
use std::arch::is_aarch64_feature_detected;

pub(super) fn score_block32_neon(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support,
            // and callers validate LUT/code/output shapes before dispatch.
            return unsafe { score_octets_neon_impl(lut, original_dim, codes, out_scores) };
        }
    }

    scalar::score_block32_scalar(lut, original_dim, codes, out_scores)
}

/// Octet-granular entry for sub-block tails: `codes.len()` must be a
/// nonzero multiple of 8 and at most `BLOCK_WIDTH`. Returns `None` when
/// NEON is unavailable so the caller can fall back.
pub(super) fn score_octets_neon(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(target_arch = "aarch64")]
    {
        debug_assert!(!codes.is_empty() && codes.len() % 8 == 0 && codes.len() <= BLOCK_WIDTH);
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support,
            // and callers validate LUT/code/output shapes before dispatch.
            return Some(unsafe { score_octets_neon_impl(lut, original_dim, codes, out_scores) });
        }
    }

    let _ = (lut, original_dim, codes, out_scores);
    None
}

#[cfg(test)]
pub(super) fn score_block32_neon_for_test(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<Isa> {
    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            // SAFETY: runtime feature detection above guarantees NEON support;
            // test fixtures use the same validated shapes as the public block path.
            return Some(unsafe { score_octets_neon_impl(lut, original_dim, codes, out_scores) });
        }
    }

    let _ = (lut, original_dim, codes, out_scores);
    None
}

#[cfg(target_arch = "aarch64")]
const OCTET_COUNT: usize = BLOCK_WIDTH / 8;

/// Code bytes consumed per lane per repack chunk (each byte packs two dims).
#[cfg(target_arch = "aarch64")]
const CHUNK_BYTES: usize = 16;

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn score_octets_neon_impl(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Isa {
    use std::arch::aarch64::{
        vaddq_f32, vand_u8, vdup_n_u8, vdupq_n_f32, vdupq_n_u8, vget_high_u16, vget_high_u8,
        vget_low_u16, vget_low_u8, vld1q_u8, vmovl_u16, vmovl_u8, vshr_n_u8, vst1q_f32,
    };

    // Shuffle-repack register-LUT scoring, the NEON sibling of the AVX2
    // kernel: code bytes are transposed from per-candidate rows into per-dim
    // columns with a 3-pass vzip network once per 16-byte chunk, so the
    // scoring loop is pure SIMD — no per-dim scalar nibble extraction and no
    // scalar-to-vector store round trips. Each transposed byte feeds two
    // dims (low then high nibble); each dim's 16-entry f32 LUT (64 bytes) is
    // held as a four-register byte table and selected per quad with
    // vqtbl4q_u8 over in-vector byte indexes. Per-lane accumulation stays in
    // dim order, so scores remain bit-exact against the scalar block
    // reference. Lane count is octet granular (8..=32 in steps of 8) so
    // sub-block tails only pay for the octets they occupy.
    let octet_count = codes.len() / 8;
    debug_assert!(octet_count >= 1 && octet_count <= OCTET_COUNT);
    debug_assert_eq!(codes.len(), octet_count * 8);
    debug_assert_eq!(out_scores.len(), codes.len());

    // Two f32 quads per octet: acc pair = lanes 0..4 and 4..8 of the octet.
    let mut acc = [vdupq_n_f32(0.0); 2 * OCTET_COUNT];
    let nibble_mask = vdup_n_u8(0x0F);
    let total_bytes = super::expected_mse_code_len(original_dim);

    let mut byte_base = 0usize;
    while byte_base < total_bytes {
        let chunk_len = CHUNK_BYTES.min(total_bytes - byte_base);
        for (octet, acc_pair) in acc.chunks_exact_mut(2).enumerate().take(octet_count) {
            let lane_base = octet * 8;
            let mut rows = [vdupq_n_u8(0); 8];
            if chunk_len == CHUNK_BYTES {
                for (row, code) in rows.iter_mut().zip(&codes[lane_base..lane_base + 8]) {
                    *row = vld1q_u8(code.as_ptr().add(byte_base));
                }
            } else {
                for (row, code) in rows.iter_mut().zip(&codes[lane_base..lane_base + 8]) {
                    let mut padded = [0u8; CHUNK_BYTES];
                    padded[..chunk_len].copy_from_slice(&code[byte_base..byte_base + chunk_len]);
                    *row = vld1q_u8(padded.as_ptr());
                }
            }
            let cols = transpose_8x16(rows);

            for b in 0..chunk_len {
                let pair = cols[b / 2];
                let col = if b & 1 == 0 {
                    vget_low_u8(pair)
                } else {
                    vget_high_u8(pair)
                };
                let dim_low = (byte_base + b) * 2;
                let low_table = load_dim_table(lut, dim_low);
                // Accumulate one dim at a time: float addition is not
                // associative, and bit-exactness requires the scalar
                // reference's per-lane dim-order sums.
                let low_wide = vmovl_u8(vand_u8(col, nibble_mask));
                acc_pair[0] = vaddq_f32(
                    acc_pair[0],
                    select_lut_entries(low_table, vmovl_u16(vget_low_u16(low_wide))),
                );
                acc_pair[1] = vaddq_f32(
                    acc_pair[1],
                    select_lut_entries(low_table, vmovl_u16(vget_high_u16(low_wide))),
                );
                let dim_high = dim_low + 1;
                if dim_high < original_dim {
                    let high_table = load_dim_table(lut, dim_high);
                    let high_wide = vmovl_u8(vshr_n_u8::<4>(col));
                    acc_pair[0] = vaddq_f32(
                        acc_pair[0],
                        select_lut_entries(high_table, vmovl_u16(vget_low_u16(high_wide))),
                    );
                    acc_pair[1] = vaddq_f32(
                        acc_pair[1],
                        select_lut_entries(high_table, vmovl_u16(vget_high_u16(high_wide))),
                    );
                }
            }
        }
        byte_base += chunk_len;
    }

    for (octet, acc_pair) in acc.chunks_exact(2).enumerate().take(octet_count) {
        vst1q_f32(out_scores.as_mut_ptr().add(octet * 8), acc_pair[0]);
        vst1q_f32(out_scores.as_mut_ptr().add(octet * 8 + 4), acc_pair[1]);
    }

    Isa::Neon
}

/// Selects `lut_table[index]` f32 entries for the four dword lane indexes
/// (each 0..=15) from the dim's register-resident 64-byte table: byte
/// indexes are `nibble * 4` replicated into four bytes plus the 0..3
/// offsets, then one `vqtbl4q_u8` per quad.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn select_lut_entries(
    table: std::arch::aarch64::uint8x16x4_t,
    nibbles: std::arch::aarch64::uint32x4_t,
) -> std::arch::aarch64::float32x4_t {
    use std::arch::aarch64::{
        vaddq_u32, vdupq_n_u32, vmulq_u32, vqtbl4q_u8, vreinterpretq_f32_u8, vreinterpretq_u8_u32,
        vshlq_n_u32,
    };
    let byte_bases = vmulq_u32(vshlq_n_u32::<2>(nibbles), vdupq_n_u32(0x0101_0101));
    let byte_indexes = vreinterpretq_u8_u32(vaddq_u32(byte_bases, vdupq_n_u32(0x0302_0100)));
    vreinterpretq_f32_u8(vqtbl4q_u8(table, byte_indexes))
}

/// Loads one dim's 16-entry f32 LUT (64 bytes) as a four-register byte table.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn load_dim_table(lut: &[f32], dim_index: usize) -> std::arch::aarch64::uint8x16x4_t {
    use std::arch::aarch64::{uint8x16x4_t, vld1q_u8};
    let lut_bytes = lut.as_ptr().add(dim_index * 16).cast::<u8>();
    uint8x16x4_t(
        vld1q_u8(lut_bytes),
        vld1q_u8(lut_bytes.add(16)),
        vld1q_u8(lut_bytes.add(32)),
        vld1q_u8(lut_bytes.add(48)),
    )
}

/// Transposes eight 16-byte candidate rows into byte columns. Output
/// element `i` packs column `2i` in its low 8 bytes and column `2i + 1`
/// in its high 8 bytes, where column `c` holds byte `c` of all eight rows.
/// Same network and output mapping as the AVX2 `transpose_8x16`
/// (`vzip1q` = `unpacklo`, `vzip2q` = `unpackhi`).
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn transpose_8x16(
    rows: [std::arch::aarch64::uint8x16_t; 8],
) -> [std::arch::aarch64::uint8x16_t; 8] {
    use std::arch::aarch64::{
        vreinterpretq_u16_u8, vreinterpretq_u32_u16, vreinterpretq_u8_u32, vzip1q_u16, vzip1q_u32,
        vzip1q_u8, vzip2q_u16, vzip2q_u32, vzip2q_u8,
    };

    // Pass 1 (bytes): pair rows (0,1), (2,3), (4,5), (6,7).
    let p0 = vreinterpretq_u16_u8(vzip1q_u8(rows[0], rows[1]));
    let p1 = vreinterpretq_u16_u8(vzip1q_u8(rows[2], rows[3]));
    let p2 = vreinterpretq_u16_u8(vzip1q_u8(rows[4], rows[5]));
    let p3 = vreinterpretq_u16_u8(vzip1q_u8(rows[6], rows[7]));
    let q0 = vreinterpretq_u16_u8(vzip2q_u8(rows[0], rows[1]));
    let q1 = vreinterpretq_u16_u8(vzip2q_u8(rows[2], rows[3]));
    let q2 = vreinterpretq_u16_u8(vzip2q_u8(rows[4], rows[5]));
    let q3 = vreinterpretq_u16_u8(vzip2q_u8(rows[6], rows[7]));
    // Pass 2 (halfwords): gather four-row groups per column pair.
    let u0 = vreinterpretq_u32_u16(vzip1q_u16(p0, p1));
    let u1 = vreinterpretq_u32_u16(vzip2q_u16(p0, p1));
    let u2 = vreinterpretq_u32_u16(vzip1q_u16(p2, p3));
    let u3 = vreinterpretq_u32_u16(vzip2q_u16(p2, p3));
    let w0 = vreinterpretq_u32_u16(vzip1q_u16(q0, q1));
    let w1 = vreinterpretq_u32_u16(vzip2q_u16(q0, q1));
    let w2 = vreinterpretq_u32_u16(vzip1q_u16(q2, q3));
    let w3 = vreinterpretq_u32_u16(vzip2q_u16(q2, q3));
    // Pass 3 (words): stitch all eight rows; each output packs two columns.
    [
        vreinterpretq_u8_u32(vzip1q_u32(u0, u2)), // columns 0, 1
        vreinterpretq_u8_u32(vzip2q_u32(u0, u2)), // columns 2, 3
        vreinterpretq_u8_u32(vzip1q_u32(u1, u3)), // columns 4, 5
        vreinterpretq_u8_u32(vzip2q_u32(u1, u3)), // columns 6, 7
        vreinterpretq_u8_u32(vzip1q_u32(w0, w2)), // columns 8, 9
        vreinterpretq_u8_u32(vzip2q_u32(w0, w2)), // columns 10, 11
        vreinterpretq_u8_u32(vzip1q_u32(w1, w3)), // columns 12, 13
        vreinterpretq_u8_u32(vzip2q_u32(w1, w3)), // columns 14, 15
    ]
}

#[cfg(all(test, target_arch = "aarch64"))]
mod tests {
    use super::*;

    #[test]
    fn transpose_8x16_yields_byte_columns() {
        if !is_aarch64_feature_detected!("neon") {
            return;
        }
        let mut input = [[0u8; 16]; 8];
        for (row, bytes) in input.iter_mut().enumerate() {
            for (col, byte) in bytes.iter_mut().enumerate() {
                *byte = (row * 16 + col) as u8;
            }
        }
        // SAFETY: feature detection above; plain integer shuffles on stack data.
        let cols = unsafe {
            let rows = input.map(|bytes| std::arch::aarch64::vld1q_u8(bytes.as_ptr()));
            transpose_8x16(rows)
        };
        let mut out = [[0u8; 16]; 8];
        for (slot, col) in cols.iter().enumerate() {
            // SAFETY: writing 16 bytes from a vector register to a 16-byte array.
            unsafe {
                std::arch::aarch64::vst1q_u8(out[slot].as_mut_ptr(), *col);
            }
        }
        for (slot, slot_bytes) in out.iter().enumerate() {
            for half in 0..2 {
                let col = slot * 2 + half;
                for row in 0..8 {
                    assert_eq!(
                        slot_bytes[half * 8 + row],
                        input[row][col],
                        "column {col} row {row}"
                    );
                }
            }
        }
    }
}
