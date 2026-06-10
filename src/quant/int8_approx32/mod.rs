//! 32-candidate blocked Int8Approx scoring for the HNSW TurboQuant no-QJL
//! 4-bit exact-score mode (Task 98).
//!
//! Per candidate: `sum_i32(codebook[i8 nibble] * rotated[i8 dim])` scaled
//! once by `score_scale`. The i32 accumulation is order-independent, so
//! every ISA backend must produce identical sums and therefore identical
//! final scores — parity tests assert strict `to_bits()` equality, with no
//! tolerance framing (the hamming32 contract).
//!
//! The HNSW routing slice consumes these entry points; the allow is
//! removed when it lands.
#![allow(dead_code)]

mod avx2;
mod neon;
mod scalar;
mod sve;

use crate::quant::prod::Int8ApproxNoQjl4BitQuery;

pub(crate) const BLOCK_WIDTH: usize = 32;

pub(crate) fn validate_code_shape(
    index: usize,
    original_dim: usize,
    mse_packed: &[u8],
) -> Result<(), String> {
    let expected = original_dim.div_ceil(2);
    if mse_packed.len() < expected {
        return Err(format!(
            "int8_approx32 candidate {index} packed length {} is shorter than expected {expected}",
            mse_packed.len()
        ));
    }
    Ok(())
}

pub(crate) fn score_int8_approx_block32(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Avx2 => {
            avx2::score_block32_avx2(prepared, original_dim, codes, out_scores)
        }
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve => {
            sve::score_block32_sve(prepared, original_dim, codes, out_scores)
        }
        crate::quant::isa::Isa::Neon => {
            neon::score_block32_neon(prepared, original_dim, codes, out_scores)
        }
        crate::quant::isa::Isa::Scalar => {
            scalar::score_block32_scalar(prepared, original_dim, codes, out_scores)
        }
    }
}

/// Partial-batch (1..=31) scoring per the partial-width convention.
pub(crate) fn score_int8_approx_partial(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    debug_assert!(!codes.is_empty() && codes.len() < BLOCK_WIDTH);
    debug_assert_eq!(codes.len(), out_scores.len());
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Neon
        | crate::quant::isa::Isa::Sve2
        | crate::quant::isa::Isa::Sve => {
            neon::score_partial_neon(prepared, original_dim, codes, out_scores)
        }
        crate::quant::isa::Isa::Avx2 => {
            avx2::score_partial_avx2(prepared, original_dim, codes, out_scores)
        }
        crate::quant::isa::Isa::Scalar => {
            scalar::score_partial_scalar(prepared, original_dim, codes, out_scores)
        }
    }
}

/// Forced-scalar single-candidate reference; the strict parity anchor.
#[allow(dead_code)]
pub(crate) fn score_int8_approx_scalar(
    prepared: &Int8ApproxNoQjl4BitQuery,
    original_dim: usize,
    mse_packed: &[u8],
) -> f32 {
    scalar::score_candidate(prepared, original_dim, mse_packed)
}

#[cfg(test)]
mod tests {
    use super::{
        score_int8_approx_block32, score_int8_approx_partial, score_int8_approx_scalar, BLOCK_WIDTH,
    };
    use crate::quant::prod::ProdQuantizer;

    fn vector(dimensions: usize, seed: u64) -> Vec<f32> {
        let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
        (0..dimensions)
            .map(|index| {
                state = state
                    .wrapping_mul(0xA076_1D64_78BD_642F)
                    .wrapping_add(0xE703_7ED1_A0B4_28DB);
                let raw = ((state >> 32) as u32) as f32 / (u32::MAX as f32);
                (raw * 2.0 - 1.0) + (index as f32 * 0.000_1)
            })
            .collect()
    }

    #[test]
    fn block32_and_partial_are_bit_equal_with_scalar_reference() {
        let dimensions = 1536;
        let quantizer = ProdQuantizer::cached(dimensions, 4, 42);
        let query = vector(dimensions, 7);
        let prepared = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(&query);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH + 7)
            .map(|seed| {
                quantizer
                    .encode(&vector(dimensions, seed as u64 + 100))
                    .mse_packed
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();

        let block: &[&[u8]; BLOCK_WIDTH] = code_refs[..BLOCK_WIDTH].try_into().unwrap();
        let mut block_scores = vec![0.0; BLOCK_WIDTH];
        score_int8_approx_block32(&prepared, dimensions, block, &mut block_scores);
        for (score, code) in block_scores.iter().zip(code_refs[..BLOCK_WIDTH].iter()) {
            let reference = score_int8_approx_scalar(&prepared, dimensions, code);
            assert_eq!(score.to_bits(), reference.to_bits());
        }

        let mut partial_scores = vec![0.0; 7];
        score_int8_approx_partial(
            &prepared,
            dimensions,
            &code_refs[BLOCK_WIDTH..],
            &mut partial_scores,
        );
        for (score, code) in partial_scores.iter().zip(code_refs[BLOCK_WIDTH..].iter()) {
            let reference = score_int8_approx_scalar(&prepared, dimensions, code);
            assert_eq!(score.to_bits(), reference.to_bits());
        }
    }

    #[test]
    fn scalar_reference_matches_production_scorer_bits() {
        let dimensions = 1536;
        let quantizer = ProdQuantizer::cached(dimensions, 4, 42);
        let query = vector(dimensions, 11);
        let prepared = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(&query);
        for seed in 0..8u64 {
            let encoded = quantizer.encode(&vector(dimensions, seed + 300));
            let kernel = score_int8_approx_scalar(&prepared, dimensions, &encoded.mse_packed);
            let production = quantizer.score_ip_from_parts_int8_approx_no_qjl_4bit(&prepared, &{
                let mut full = encoded.mse_packed.clone();
                full.extend_from_slice(&encoded.qjl_packed);
                full
            });
            assert_eq!(kernel.to_bits(), production.to_bits());
        }
    }
}
