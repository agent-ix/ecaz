//! 32-candidate blocked LUT scoring for TurboQuant no-QJL 4-bit codes.
//!
//! Block scorers return the ISA actually implemented by the selected backend.
//! Counter call sites must use that returned value for kernel rows; scalar
//! tails are still reported separately under `Isa::Scalar`. Fallback backend
//! stubs that delegate to the scalar implementation return `Isa::Scalar` until
//! replaced by real ISA kernels.

mod avx2;
mod neon;
mod scalar;
mod sve;

pub(crate) const BLOCK_WIDTH: usize = 32;

pub(crate) fn expected_mse_code_len(original_dim: usize) -> usize {
    original_dim.div_ceil(2)
}

pub(crate) fn validate_lut_shape(lut: &[f32], original_dim: usize) -> Result<(), String> {
    if lut.len() != original_dim * 16 {
        return Err(format!(
            "lut32 LUT length mismatch: got {}, expected {}",
            lut.len(),
            original_dim * 16
        ));
    }
    Ok(())
}

pub(crate) fn validate_mse_code_shape(
    index: usize,
    original_dim: usize,
    code: &[u8],
) -> Result<(), String> {
    let expected_mse_len = expected_mse_code_len(original_dim);
    if code.len() < expected_mse_len {
        return Err(format!(
            "lut32 code {index} too short: got {}, expected at least {expected_mse_len}",
            code.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn score_lut_no_qjl_4bit_batch(
    lut: &[f32],
    original_dim: usize,
    mse_codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Result<(), String> {
    if mse_codes.len() != out_scores.len() {
        return Err(format!(
            "lut32 score output count {} does not match candidate count {}",
            out_scores.len(),
            mse_codes.len()
        ));
    }
    validate_lut_shape(lut, original_dim)?;
    for (index, code) in mse_codes.iter().enumerate() {
        validate_mse_code_shape(index, original_dim, code)?;
    }

    let mut block_start = 0usize;
    while block_start + BLOCK_WIDTH <= mse_codes.len() {
        let _ = score_lut_no_qjl_4bit_block32(
            lut,
            original_dim,
            mse_codes[block_start..block_start + BLOCK_WIDTH]
                .try_into()
                .expect("slice length is exactly one block"),
            &mut out_scores[block_start..block_start + BLOCK_WIDTH],
        );
        block_start += BLOCK_WIDTH;
    }

    for (code, out_score) in mse_codes[block_start..]
        .iter()
        .zip(out_scores[block_start..].iter_mut())
    {
        *out_score = score_lut_no_qjl_4bit_scalar(lut, original_dim, code);
    }

    Ok(())
}

pub(crate) fn score_lut_no_qjl_4bit_block32(
    lut: &[f32],
    original_dim: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Avx2 => {
            avx2::score_block32_avx2(lut, original_dim, &codes, out_scores)
        }
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve => {
            sve::score_block32_sve(lut, original_dim, &codes, out_scores)
        }
        crate::quant::isa::Isa::Neon => {
            neon::score_block32_neon(lut, original_dim, &codes, out_scores)
        }
        crate::quant::isa::Isa::Scalar => {
            scalar::score_block32_scalar(lut, original_dim, &codes, out_scores)
        }
    }
}

pub(crate) fn score_lut_no_qjl_4bit_scalar(lut: &[f32], original_dim: usize, code: &[u8]) -> f32 {
    scalar::score_scalar_tail(lut, original_dim, code)
}

#[cfg(test)]
mod tests {
    use super::{
        score_lut_no_qjl_4bit_batch, score_lut_no_qjl_4bit_block32, score_lut_no_qjl_4bit_scalar,
        BLOCK_WIDTH,
    };

    fn lut(dim: usize) -> Vec<f32> {
        (0..dim * 16)
            .map(|index| ((index as i32 % 29) - 14) as f32 * 0.125)
            .collect()
    }

    fn code(dim: usize, seed: u8) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(dim.div_ceil(2));
        for byte_index in 0..dim.div_ceil(2) {
            let low = seed.wrapping_add((byte_index as u8).wrapping_mul(3)) & 0x0F;
            let high = seed.wrapping_add((byte_index as u8).wrapping_mul(5)) & 0x0F;
            bytes.push(low | (high << 4));
        }
        bytes
    }

    #[test]
    fn lut32_batch_under_block_width_matches_scalar_tail_bits() {
        let dim = 1536;
        let lut = lut(dim);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH - 1)
            .map(|seed| code(dim, seed as u8))
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; code_refs.len()];

        score_lut_no_qjl_4bit_batch(&lut, dim, &code_refs, &mut scores).unwrap();

        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(
                score.to_bits(),
                score_lut_no_qjl_4bit_scalar(&lut, dim, code).to_bits()
            );
        }
    }

    #[test]
    fn lut32_batch_with_blocks_and_tail_matches_scalar_tail_bits() {
        let dim = 1536;
        let lut = lut(dim);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH + 7)
            .map(|seed| code(dim, seed as u8))
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; code_refs.len()];

        score_lut_no_qjl_4bit_batch(&lut, dim, &code_refs, &mut scores).unwrap();

        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(
                score.to_bits(),
                score_lut_no_qjl_4bit_scalar(&lut, dim, code).to_bits()
            );
        }
    }

    #[test]
    fn lut32_block32_matches_scalar_tail_bits() {
        let dim = 1536;
        let lut = lut(dim);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let isa = score_lut_no_qjl_4bit_block32(
            &lut,
            dim,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            &mut scores,
        );

        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(
                score.to_bits(),
                score_lut_no_qjl_4bit_scalar(&lut, dim, code).to_bits()
            );
        }
        assert_eq!(isa, crate::quant::isa::Isa::Scalar);
    }

    #[test]
    fn lut32_rejects_shape_mismatch() {
        let lut = lut(8);
        let code = code(8, 1);
        let codes = vec![code.as_slice()];
        let mut scores = vec![0.0, 0.0];

        assert!(score_lut_no_qjl_4bit_batch(&lut, 8, &codes, &mut scores).is_err());
    }
}
