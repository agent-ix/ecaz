//! 32-candidate blocked TiledLut scoring for the HNSW TurboQuant no-QJL
//! 4-bit exact-score mode (Task 98).
//!
//! Same algebra as the FullLut family (per-dim 16-entry f32 LUT sums) but
//! accumulated tile-by-tile for cache locality, matching the production
//! `score_ip_from_split_parts_tiled_lut_no_qjl_4bit` operation order. The
//! scalar backend is the strict `to_bits()` parity anchor; SIMD backends
//! (when they land) follow the rabitq32 envelope contract. Blocking scores
//! tile-outer/candidate-inner so a hot LUT tile is reused across the whole
//! block before moving on.
//!
//! The HNSW routing slice consumes these entry points; the allow is
//! removed when it lands.
#![allow(dead_code)]

mod scalar;

pub(crate) const BLOCK_WIDTH: usize = 32;

pub(crate) fn validate_code_shape(
    index: usize,
    original_dim: usize,
    mse_packed: &[u8],
) -> Result<(), String> {
    let expected = original_dim.div_ceil(2);
    if mse_packed.len() < expected {
        return Err(format!(
            "tiled_lut32 candidate {index} packed length {} is shorter than expected {expected}",
            mse_packed.len()
        ));
    }
    Ok(())
}

/// Scores up to one block (1..=32 candidates) tile-outer/candidate-inner.
/// All backends currently resolve to the scalar tile walk; SIMD variants
/// are a follow-up slice gated on the Phase A width distribution.
pub(crate) fn score_tiled_lut_run(
    lut: &[f32],
    tile_size: usize,
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    debug_assert!(!codes.is_empty() && codes.len() <= BLOCK_WIDTH);
    debug_assert_eq!(codes.len(), out_scores.len());
    scalar::score_run_scalar(lut, tile_size, original_dim, codes, out_scores)
}

/// Forced-scalar single-candidate reference; the strict parity anchor.
pub(crate) fn score_tiled_lut_scalar(
    lut: &[f32],
    tile_size: usize,
    original_dim: usize,
    mse_packed: &[u8],
) -> f32 {
    scalar::score_candidate(lut, tile_size, original_dim, mse_packed)
}

#[cfg(test)]
mod tests {
    use super::{score_tiled_lut_run, score_tiled_lut_scalar, BLOCK_WIDTH};
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
    fn run_is_bit_equal_with_production_tiled_scorer() {
        let dimensions = 1536;
        let quantizer = ProdQuantizer::cached(dimensions, 4, 42);
        let query = vector(dimensions, 7);
        let prepared = quantizer.prepare_ip_query_tiled_lut_no_qjl_4bit(&query, 128);
        let encoded: Vec<_> = (0..BLOCK_WIDTH + 5)
            .map(|seed| quantizer.encode(&vector(dimensions, seed as u64 + 100)))
            .collect();

        for window in [&encoded[..BLOCK_WIDTH], &encoded[BLOCK_WIDTH..]] {
            let codes: Vec<&[u8]> = window
                .iter()
                .map(|encoded| encoded.mse_packed.as_slice())
                .collect();
            let mut scores = vec![0.0; codes.len()];
            let isa = score_tiled_lut_run(
                &prepared.lut,
                prepared.tile_size,
                dimensions,
                &codes,
                &mut scores,
            );
            assert_eq!(isa, crate::quant::isa::Isa::Scalar);
            for (score, encoded) in scores.iter().zip(window.iter()) {
                let mut full = encoded.mse_packed.clone();
                full.extend_from_slice(&encoded.qjl_packed);
                let production =
                    quantizer.score_ip_from_parts_tiled_lut_no_qjl_4bit(&prepared, &full);
                assert_eq!(score.to_bits(), production.to_bits());
                assert_eq!(
                    score_tiled_lut_scalar(
                        &prepared.lut,
                        prepared.tile_size,
                        dimensions,
                        &encoded.mse_packed,
                    )
                    .to_bits(),
                    production.to_bits()
                );
            }
        }
    }
}
