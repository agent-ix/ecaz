//! 32-candidate blocked TurboQuant QJL scoring for canonical 4-bit codes.
//!
//! Task 97 covers the currently implemented QJL-active TurboQuant lane:
//! `bits=4` at a non-tiled dimension. The packed code is `[mse_packed][qjl_packed]`
//! where the MSE stage is 3 bits per dimension and QJL contributes one sign bit
//! per dimension plus per-candidate gamma metadata.

mod avx2;
mod neon;
mod scalar;
mod sve;

use crate::quant::prod::{
    mse_code_len, qjl_code_len, ExactScoreMode, PreparedQuery, ProdQuantizer,
};

pub(crate) const BLOCK_WIDTH: usize = 32;
pub(crate) const OCTET_WIDTH: usize = 8;

pub(crate) fn expected_code_len(original_dim: usize) -> usize {
    mse_code_len(original_dim, 4) + qjl_code_len(original_dim)
}

pub(crate) fn validate_qjl_shape(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
) -> Result<(), String> {
    if quantizer.bits != 4 || quantizer.exact_score_mode() != ExactScoreMode::MseLutQjl {
        return Err(format!(
            "qjl32 requires QJL-active TurboQuant 4-bit scoring, got mode {}",
            quantizer.exact_score_mode_name()
        ));
    }
    if prepared.rotated.len() != quantizer.original_dim {
        return Err(format!(
            "qjl32 rotated query length mismatch: got {}, expected {}",
            prepared.rotated.len(),
            quantizer.original_dim
        ));
    }
    if prepared.sq.len() != quantizer.original_dim {
        return Err(format!(
            "qjl32 QJL projection length mismatch: got {}, expected {}",
            prepared.sq.len(),
            quantizer.original_dim
        ));
    }
    if quantizer.codebook.len() < 8 {
        return Err(format!(
            "qjl32 codebook too short: got {}, expected at least 8",
            quantizer.codebook.len()
        ));
    }
    Ok(())
}

pub(crate) fn validate_code_shape(
    index: usize,
    original_dim: usize,
    code: &[u8],
) -> Result<(), String> {
    let expected_len = expected_code_len(original_dim);
    if code.len() != expected_len {
        return Err(format!(
            "qjl32 code {index} length mismatch: got {}, expected {expected_len}",
            code.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn score_turboquant_qjl_batch(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]],
    gammas: &[f32],
    out_scores: &mut [f32],
) -> Result<(), String> {
    if codes.len() != gammas.len() || codes.len() != out_scores.len() {
        return Err(format!(
            "qjl32 score output count {} does not match code count {} and gamma count {}",
            out_scores.len(),
            codes.len(),
            gammas.len()
        ));
    }
    validate_qjl_shape(quantizer, prepared)?;
    for (index, code) in codes.iter().enumerate() {
        validate_code_shape(index, quantizer.original_dim, code)?;
    }

    let mut block_start = 0usize;
    while block_start + BLOCK_WIDTH <= codes.len() {
        let _ = score_turboquant_qjl_block32(
            quantizer,
            prepared,
            codes[block_start..block_start + BLOCK_WIDTH]
                .try_into()
                .expect("slice length is exactly one block"),
            gammas[block_start..block_start + BLOCK_WIDTH]
                .try_into()
                .expect("slice length is exactly one block"),
            &mut out_scores[block_start..block_start + BLOCK_WIDTH],
        );
        block_start += BLOCK_WIDTH;
    }

    for ((code, gamma), out_score) in codes[block_start..]
        .iter()
        .zip(gammas[block_start..].iter())
        .zip(out_scores[block_start..].iter_mut())
    {
        *out_score = score_turboquant_qjl_scalar(quantizer, prepared, code, *gamma);
    }
    Ok(())
}

pub(crate) fn score_turboquant_qjl_block32(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: [&[u8]; BLOCK_WIDTH],
    gammas: [f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Avx2 => {
            avx2::score_block32_avx2(quantizer, prepared, &codes, &gammas, out_scores)
        }
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve => {
            sve::score_block32_sve(quantizer, prepared, &codes, &gammas, out_scores)
        }
        crate::quant::isa::Isa::Neon => {
            neon::score_block32_neon(quantizer, prepared, &codes, &gammas, out_scores)
        }
        crate::quant::isa::Isa::Scalar => {
            scalar::score_block32_scalar(quantizer, prepared, &codes, &gammas, out_scores)
        }
    }
}

pub(crate) fn score_turboquant_qjl_octet8_avx2(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: [&[u8]; OCTET_WIDTH],
    gammas: [f32; OCTET_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    avx2::score_octet8_avx2(quantizer, prepared, &codes, &gammas, out_scores)
}

pub(crate) fn score_turboquant_qjl_scalar(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    code: &[u8],
    gamma: f32,
) -> f32 {
    scalar::score_scalar_tail(quantizer, prepared, code, gamma)
}

#[cfg(test)]
pub(crate) fn score_turboquant_qjl_block32_neon_for_test(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: [&[u8]; BLOCK_WIDTH],
    gammas: [f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    neon::score_block32_neon_for_test(quantizer, prepared, &codes, &gammas, out_scores)
}

#[cfg(test)]
pub(crate) fn score_turboquant_qjl_block32_sve_for_test(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: [&[u8]; BLOCK_WIDTH],
    gammas: [f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    sve::score_block32_sve_for_test(quantizer, prepared, &codes, &gammas, out_scores)
}

#[cfg(test)]
pub(crate) fn runtime_sve_vector_lanes_for_test() -> Option<usize> {
    sve::runtime_vector_lanes_for_test()
}

pub(super) fn split_qjl_code_bytes<'a>(
    original_dim: usize,
    code: &'a [u8],
) -> (&'a [u8], &'a [u8]) {
    let mse_len = mse_code_len(original_dim, 4);
    let qjl_len = qjl_code_len(original_dim);
    debug_assert_eq!(code.len(), mse_len + qjl_len);
    (&code[..mse_len], &code[mse_len..mse_len + qjl_len])
}

#[cfg(test)]
mod tests {
    use super::{
        runtime_sve_vector_lanes_for_test, score_turboquant_qjl_batch,
        score_turboquant_qjl_block32, score_turboquant_qjl_block32_neon_for_test,
        score_turboquant_qjl_block32_sve_for_test, score_turboquant_qjl_octet8_avx2,
        score_turboquant_qjl_scalar, validate_qjl_shape, BLOCK_WIDTH, OCTET_WIDTH,
    };
    use crate::quant::isa::Isa;

    #[test]
    fn qjl32_scalar_matches_pre_slice_scorer_bits() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1024, 4, 42);
        let query = random_unit_vector(1024, 71);
        let prepared = quantizer.prepare_ip_query(&query);
        validate_qjl_shape(&quantizer, &prepared).unwrap();
        let encoded: Vec<_> = (0..BLOCK_WIDTH + 7)
            .map(|seed| quantizer.encode(&random_unit_vector(1024, seed as u64 + 200)))
            .collect();

        for encoded in &encoded {
            let mut code = Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
            code.extend_from_slice(&encoded.mse_packed);
            code.extend_from_slice(&encoded.qjl_packed);
            let scalar = score_turboquant_qjl_scalar(&quantizer, &prepared, &code, encoded.gamma);
            let pre_slice =
                quantizer.score_ip_from_parts_scalar_reference(&prepared, encoded.gamma, &code);
            assert_eq!(scalar.to_bits(), pre_slice.to_bits());
        }
    }

    #[test]
    fn qjl32_batch_with_blocks_and_tail_matches_pre_slice_scorer_bits() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1024, 4, 42);
        let query = random_unit_vector(1024, 91);
        let prepared = quantizer.prepare_ip_query(&query);
        let encoded: Vec<_> = (0..BLOCK_WIDTH + 7)
            .map(|seed| quantizer.encode(&random_unit_vector(1024, seed as u64 + 300)))
            .collect();
        let codes: Vec<Vec<u8>> = encoded
            .iter()
            .map(|encoded| {
                let mut code =
                    Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
                code.extend_from_slice(&encoded.mse_packed);
                code.extend_from_slice(&encoded.qjl_packed);
                code
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let gammas: Vec<f32> = encoded.iter().map(|encoded| encoded.gamma).collect();
        let mut scores = vec![0.0; code_refs.len()];

        score_turboquant_qjl_batch(&quantizer, &prepared, &code_refs, &gammas, &mut scores)
            .unwrap();

        for ((code, gamma), score) in code_refs.iter().zip(gammas.iter()).zip(scores.iter()) {
            let pre_slice = quantizer.score_ip_from_parts_scalar_reference(&prepared, *gamma, code);
            assert_eq!(score.to_bits(), pre_slice.to_bits());
        }
    }

    #[test]
    fn qjl32_block32_matches_pre_slice_scorer_bits() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1024, 4, 42);
        let query = random_unit_vector(1024, 111);
        let prepared = quantizer.prepare_ip_query(&query);
        let encoded: Vec<_> = (0..BLOCK_WIDTH)
            .map(|seed| quantizer.encode(&random_unit_vector(1024, seed as u64 + 400)))
            .collect();
        let codes: Vec<Vec<u8>> = encoded
            .iter()
            .map(|encoded| {
                let mut code =
                    Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
                code.extend_from_slice(&encoded.mse_packed);
                code.extend_from_slice(&encoded.qjl_packed);
                code
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let gammas: [f32; BLOCK_WIDTH] = encoded
            .iter()
            .map(|encoded| encoded.gamma)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let isa = score_turboquant_qjl_block32(
            &quantizer,
            &prepared,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            gammas,
            &mut scores,
        );

        for ((code, gamma), score) in code_refs.iter().zip(gammas.iter()).zip(scores.iter()) {
            let pre_slice = quantizer.score_ip_from_parts_scalar_reference(&prepared, *gamma, code);
            if isa == Isa::Scalar {
                assert_eq!(score.to_bits(), pre_slice.to_bits());
            } else {
                assert_close(*score, pre_slice, 4);
            }
        }
        assert!(matches!(isa, Isa::Scalar | Isa::Avx2));
    }

    #[test]
    fn qjl32_avx2_block32_matches_pre_slice_scorer_tolerance() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1024, 4, 42);
        let query = random_unit_vector(1024, 211);
        let prepared = quantizer.prepare_ip_query(&query);
        let encoded: Vec<_> = (0..BLOCK_WIDTH)
            .map(|seed| quantizer.encode(&random_unit_vector(1024, seed as u64 + 600)))
            .collect();
        let codes: Vec<Vec<u8>> = encoded
            .iter()
            .map(|encoded| {
                let mut code =
                    Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
                code.extend_from_slice(&encoded.mse_packed);
                code.extend_from_slice(&encoded.qjl_packed);
                code
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let gammas: [f32; BLOCK_WIDTH] = encoded
            .iter()
            .map(|encoded| encoded.gamma)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let Some(isa) = super::avx2::score_block32_avx2_for_test(
            &quantizer,
            &prepared,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            &gammas,
            &mut scores,
        ) else {
            return;
        };

        assert_eq!(isa, Isa::Avx2);
        for ((code, gamma), score) in code_refs.iter().zip(gammas.iter()).zip(scores.iter()) {
            let pre_slice = quantizer.score_ip_from_parts_scalar_reference(&prepared, *gamma, code);
            assert_close(*score, pre_slice, 4);
        }
    }

    #[test]
    fn qjl32_avx2_octet8_matches_pre_slice_scorer_tolerance() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1024, 4, 42);
        let query = random_unit_vector(1024, 213);
        let prepared = quantizer.prepare_ip_query(&query);
        let encoded: Vec<_> = (0..OCTET_WIDTH)
            .map(|seed| quantizer.encode(&random_unit_vector(1024, seed as u64 + 625)))
            .collect();
        let codes: Vec<Vec<u8>> = encoded
            .iter()
            .map(|encoded| {
                let mut code =
                    Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
                code.extend_from_slice(&encoded.mse_packed);
                code.extend_from_slice(&encoded.qjl_packed);
                code
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let gammas: [f32; OCTET_WIDTH] = encoded
            .iter()
            .map(|encoded| encoded.gamma)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut scores = vec![0.0; OCTET_WIDTH];

        let Some(isa) = score_turboquant_qjl_octet8_avx2(
            &quantizer,
            &prepared,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one octet"),
            gammas,
            &mut scores,
        ) else {
            return;
        };

        assert_eq!(isa, Isa::Avx2);
        for ((code, gamma), score) in code_refs.iter().zip(gammas.iter()).zip(scores.iter()) {
            let pre_slice = quantizer.score_ip_from_parts_scalar_reference(&prepared, *gamma, code);
            assert_close(*score, pre_slice, 4);
        }
    }

    #[test]
    fn qjl32_neon_block32_matches_pre_slice_scorer_tolerance_when_available() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1024, 4, 42);
        let query = random_unit_vector(1024, 215);
        let prepared = quantizer.prepare_ip_query(&query);
        let encoded: Vec<_> = (0..BLOCK_WIDTH)
            .map(|seed| quantizer.encode(&random_unit_vector(1024, seed as u64 + 640)))
            .collect();
        let codes: Vec<Vec<u8>> = encoded
            .iter()
            .map(|encoded| {
                let mut code =
                    Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
                code.extend_from_slice(&encoded.mse_packed);
                code.extend_from_slice(&encoded.qjl_packed);
                code
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let gammas: [f32; BLOCK_WIDTH] = encoded
            .iter()
            .map(|encoded| encoded.gamma)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let Some(isa) = score_turboquant_qjl_block32_neon_for_test(
            &quantizer,
            &prepared,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            gammas,
            &mut scores,
        ) else {
            return;
        };

        assert_eq!(isa, Isa::Neon);
        for ((code, gamma), score) in code_refs.iter().zip(gammas.iter()).zip(scores.iter()) {
            let pre_slice = quantizer.score_ip_from_parts_scalar_reference(&prepared, *gamma, code);
            assert_close(*score, pre_slice, 4);
        }
    }

    #[test]
    fn qjl32_sve_block32_matches_pre_slice_scorer_tolerance_when_available() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1024, 4, 42);
        let query = random_unit_vector(1024, 216);
        let prepared = quantizer.prepare_ip_query(&query);
        let encoded: Vec<_> = (0..BLOCK_WIDTH)
            .map(|seed| quantizer.encode(&random_unit_vector(1024, seed as u64 + 650)))
            .collect();
        let codes: Vec<Vec<u8>> = encoded
            .iter()
            .map(|encoded| {
                let mut code =
                    Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
                code.extend_from_slice(&encoded.mse_packed);
                code.extend_from_slice(&encoded.qjl_packed);
                code
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let gammas: [f32; BLOCK_WIDTH] = encoded
            .iter()
            .map(|encoded| encoded.gamma)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let Some(isa) = score_turboquant_qjl_block32_sve_for_test(
            &quantizer,
            &prepared,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            gammas,
            &mut scores,
        ) else {
            return;
        };

        assert!(matches!(isa, Isa::Sve | Isa::Sve2));
        assert!(runtime_sve_vector_lanes_for_test().is_some());
        for ((code, gamma), score) in code_refs.iter().zip(gammas.iter()).zip(scores.iter()) {
            let pre_slice = quantizer.score_ip_from_parts_scalar_reference(&prepared, *gamma, code);
            assert_close(*score, pre_slice, 4);
        }
    }

    #[test]
    fn qjl32_scalar_reference_matches_production_dispatch_tolerance() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1024, 4, 42);
        let query = random_unit_vector(1024, 231);
        let prepared = quantizer.prepare_ip_query(&query);
        let encoded: Vec<_> = (0..BLOCK_WIDTH + 7)
            .map(|seed| quantizer.encode(&random_unit_vector(1024, seed as u64 + 700)))
            .collect();

        for encoded in &encoded {
            let mut code = Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
            code.extend_from_slice(&encoded.mse_packed);
            code.extend_from_slice(&encoded.qjl_packed);
            let scalar = score_turboquant_qjl_scalar(&quantizer, &prepared, &code, encoded.gamma);
            let production = quantizer.score_ip_from_parts(&prepared, encoded.gamma, &code);
            assert_close(scalar, production, 4);
        }
    }

    #[test]
    fn qjl32_block32_matches_production_dispatch_tolerance() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1024, 4, 42);
        let query = random_unit_vector(1024, 251);
        let prepared = quantizer.prepare_ip_query(&query);
        let encoded: Vec<_> = (0..BLOCK_WIDTH)
            .map(|seed| quantizer.encode(&random_unit_vector(1024, seed as u64 + 800)))
            .collect();
        let codes: Vec<Vec<u8>> = encoded
            .iter()
            .map(|encoded| {
                let mut code =
                    Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
                code.extend_from_slice(&encoded.mse_packed);
                code.extend_from_slice(&encoded.qjl_packed);
                code
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let gammas: [f32; BLOCK_WIDTH] = encoded
            .iter()
            .map(|encoded| encoded.gamma)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let _isa = score_turboquant_qjl_block32(
            &quantizer,
            &prepared,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            gammas,
            &mut scores,
        );

        for ((code, gamma), score) in code_refs.iter().zip(gammas.iter()).zip(scores.iter()) {
            let production = quantizer.score_ip_from_parts(&prepared, *gamma, code);
            assert_close(*score, production, 4);
        }
    }

    #[test]
    fn qjl32_rejects_no_qjl_1536_lane() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 131);
        let prepared = quantizer.prepare_ip_query(&query);

        let err = validate_qjl_shape(&quantizer, &prepared).unwrap_err();

        assert!(err.contains("requires QJL-active TurboQuant 4-bit scoring"));
        assert!(err.contains("mse_no_qjl_4bit"));
    }

    fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
        let mut state = seed ^ 0xA076_1D64_78BD_642F;
        let mut values = Vec::with_capacity(dim);
        let mut norm_sq = 0.0_f32;
        for _ in 0..dim {
            state = state
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(0xBF58_476D_1CE4_E5B9);
            let raw = ((state >> 32) as u32) as f32 / (u32::MAX as f32);
            let value = raw * 2.0 - 1.0;
            values.push(value);
            norm_sq += value * value;
        }
        let norm = norm_sq.sqrt().max(f32::MIN_POSITIVE);
        for value in &mut values {
            *value /= norm;
        }
        values
    }

    fn assert_close(actual: f32, expected: f32, max_ulp: u32) {
        let ulp = ulp_distance(actual, expected);
        let rel = ((actual - expected).abs() / expected.abs().max(1.0e-12)).abs();
        assert!(
            ulp <= max_ulp || rel <= 1.0e-6,
            "actual={actual:?} expected={expected:?} ulp={ulp} rel={rel:?}"
        );
    }

    fn ulp_distance(lhs: f32, rhs: f32) -> u32 {
        fn ordered(bits: u32) -> i32 {
            let signed = bits as i32;
            if signed < 0 {
                i32::MIN - signed
            } else {
                signed
            }
        }
        ordered(lhs.to_bits()).abs_diff(ordered(rhs.to_bits()))
    }
}
