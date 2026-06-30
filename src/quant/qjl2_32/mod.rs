//! 32-candidate blocked TurboQuant2 QJL scoring.
//!
//! TurboQuant2 uses one MSE bit plus one QJL sign bit per dimension. The
//! candidate-parallel NEON kernel keeps candidates in vector lanes while
//! walking dimensions in scalar order for each lane.

mod neon;

use crate::quant::prod::{
    mse_code_len, qjl_code_len, ExactScoreMode, PreparedQuery, ProdQuantizer,
};

pub(crate) const BLOCK_WIDTH: usize = 32;
pub(crate) const OCTET_WIDTH: usize = 8;

pub(crate) fn expected_code_len(original_dim: usize) -> usize {
    mse_code_len(original_dim, 2) + qjl_code_len(original_dim)
}

pub(crate) fn validate_qjl2_shape(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
) -> Result<(), String> {
    if quantizer.bits != 2 || quantizer.exact_score_mode() != ExactScoreMode::MseLutQjl {
        return Err(format!(
            "qjl2_32 requires QJL-active TurboQuant2 scoring, got mode {}",
            quantizer.exact_score_mode_name()
        ));
    }
    if prepared.lut.len() != quantizer.original_dim * 2 {
        return Err(format!(
            "qjl2_32 LUT length mismatch: got {}, expected {}",
            prepared.lut.len(),
            quantizer.original_dim * 2
        ));
    }
    if prepared.sq.len() != quantizer.original_dim {
        return Err(format!(
            "qjl2_32 QJL projection length mismatch: got {}, expected {}",
            prepared.sq.len(),
            quantizer.original_dim
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
            "qjl2_32 code {index} length mismatch: got {}, expected {expected_len}",
            code.len()
        ));
    }
    Ok(())
}

pub(crate) fn score_turboquant_qjl2_block32(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: [&[u8]; BLOCK_WIDTH],
    gammas: [f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    match crate::quant::isa::current_isa() {
        crate::quant::isa::Isa::Neon
        | crate::quant::isa::Isa::Sve
        | crate::quant::isa::Isa::Sve2 => {
            neon::score_block32_neon(quantizer, prepared, &codes, &gammas, out_scores)
        }
        crate::quant::isa::Isa::Avx2 | crate::quant::isa::Isa::Scalar => {
            score_block32_scalar(quantizer, prepared, &codes, &gammas, out_scores)
        }
    }
}

pub(crate) fn score_turboquant_qjl2_octet8(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: [&[u8]; OCTET_WIDTH],
    gammas: [f32; OCTET_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    match crate::quant::isa::current_isa() {
        crate::quant::isa::Isa::Neon
        | crate::quant::isa::Isa::Sve
        | crate::quant::isa::Isa::Sve2 => {
            neon::score_octet8_neon(quantizer, prepared, &codes, &gammas, out_scores)
        }
        crate::quant::isa::Isa::Avx2 | crate::quant::isa::Isa::Scalar => None,
    }
}

pub(crate) fn score_turboquant_qjl2_scalar(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    code: &[u8],
    gamma: f32,
) -> f32 {
    let (mse_packed, qjl_packed) = split_qjl2_code_bytes(quantizer.original_dim, code);
    let mut mse_sum = 0.0_f32;
    let mut qjl_sum = 0.0_f32;
    for dim_index in 0..quantizer.original_dim {
        let lut_offset = dim_index * 2 + usize::from(bit_at(mse_packed, dim_index));
        mse_sum += prepared.lut[lut_offset];
        qjl_sum += if bit_at(qjl_packed, dim_index) {
            prepared.sq[dim_index]
        } else {
            -prepared.sq[dim_index]
        };
    }
    mse_sum + gamma * prepared.qjl_scale * qjl_sum
}

fn score_block32_scalar(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: &[&[u8]; BLOCK_WIDTH],
    gammas: &[f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    for lane in 0..BLOCK_WIDTH {
        out_scores[lane] =
            score_turboquant_qjl2_scalar(quantizer, prepared, codes[lane], gammas[lane]);
    }
    crate::quant::isa::Isa::Scalar
}

pub(super) fn split_qjl2_code_bytes<'a>(
    original_dim: usize,
    code: &'a [u8],
) -> (&'a [u8], &'a [u8]) {
    let mse_len = mse_code_len(original_dim, 2);
    let qjl_len = qjl_code_len(original_dim);
    debug_assert_eq!(code.len(), mse_len + qjl_len);
    (&code[..mse_len], &code[mse_len..mse_len + qjl_len])
}

pub(super) fn bit_at(packed: &[u8], dim_index: usize) -> bool {
    (packed[dim_index / 8] >> (dim_index % 8)) & 1 == 1
}

#[cfg(test)]
mod tests {
    use super::{
        score_turboquant_qjl2_block32, score_turboquant_qjl2_octet8, score_turboquant_qjl2_scalar,
        validate_code_shape, validate_qjl2_shape, BLOCK_WIDTH, OCTET_WIDTH,
    };

    #[test]
    fn qjl2_scalar_matches_pre_slice_scorer_bits() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1536, 2, 42);
        let query = random_unit_vector(1536, 71);
        let prepared = quantizer.prepare_ip_query(&query);
        validate_qjl2_shape(&quantizer, &prepared).unwrap();
        let encoded: Vec<_> = (0..BLOCK_WIDTH + 7)
            .map(|seed| quantizer.encode(&random_unit_vector(1536, seed as u64 + 200)))
            .collect();

        for encoded in &encoded {
            let mut code = Vec::with_capacity(encoded.mse_packed.len() + encoded.qjl_packed.len());
            code.extend_from_slice(&encoded.mse_packed);
            code.extend_from_slice(&encoded.qjl_packed);
            validate_code_shape(0, quantizer.original_dim, &code).unwrap();
            let scalar = score_turboquant_qjl2_scalar(&quantizer, &prepared, &code, encoded.gamma);
            let pre_slice =
                quantizer.score_ip_from_parts_scalar_reference(&prepared, encoded.gamma, &code);
            assert_eq!(scalar.to_bits(), pre_slice.to_bits());
        }
    }

    #[test]
    fn qjl2_block32_matches_production_dispatch_tolerance() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1536, 2, 42);
        let query = random_unit_vector(1536, 81);
        let prepared = quantizer.prepare_ip_query(&query);
        validate_qjl2_shape(&quantizer, &prepared).unwrap();
        let encoded: Vec<_> = (0..BLOCK_WIDTH)
            .map(|seed| quantizer.encode(&random_unit_vector(1536, seed as u64 + 300)))
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
        let code_refs: [&[u8]; BLOCK_WIDTH] = std::array::from_fn(|index| codes[index].as_slice());
        let gammas: [f32; BLOCK_WIDTH] = std::array::from_fn(|index| encoded[index].gamma);
        let mut out = vec![0.0_f32; BLOCK_WIDTH];

        let _ = score_turboquant_qjl2_block32(&quantizer, &prepared, code_refs, gammas, &mut out);

        for ((code, gamma), score) in codes.iter().zip(gammas.iter()).zip(out.iter()) {
            let production = quantizer.score_ip_from_parts(&prepared, *gamma, code);
            assert!((score - production).abs() <= 1.0e-4);
        }
    }

    #[test]
    fn qjl2_octet8_matches_production_dispatch_tolerance() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1536, 2, 42);
        let query = random_unit_vector(1536, 91);
        let prepared = quantizer.prepare_ip_query(&query);
        validate_qjl2_shape(&quantizer, &prepared).unwrap();
        let encoded: Vec<_> = (0..OCTET_WIDTH)
            .map(|seed| quantizer.encode(&random_unit_vector(1536, seed as u64 + 400)))
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
        let code_refs: [&[u8]; OCTET_WIDTH] = std::array::from_fn(|index| codes[index].as_slice());
        let gammas: [f32; OCTET_WIDTH] = std::array::from_fn(|index| encoded[index].gamma);
        let mut out = vec![0.0_f32; OCTET_WIDTH];

        if score_turboquant_qjl2_octet8(&quantizer, &prepared, code_refs, gammas, &mut out)
            .is_none()
        {
            return;
        }

        for ((code, gamma), score) in codes.iter().zip(gammas.iter()).zip(out.iter()) {
            let production = quantizer.score_ip_from_parts(&prepared, *gamma, code);
            assert!((score - production).abs() <= 1.0e-4);
        }
    }

    fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(seed);
        let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
        for value in &mut values {
            *value /= norm;
        }
        values
    }
}
