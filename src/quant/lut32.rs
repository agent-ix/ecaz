//! 32-candidate blocked LUT scoring for TurboQuant no-QJL 4-bit codes.

pub(crate) const BLOCK_WIDTH: usize = 32;

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
    if lut.len() != original_dim * 16 {
        return Err(format!(
            "lut32 LUT length mismatch: got {}, expected {}",
            lut.len(),
            original_dim * 16
        ));
    }
    let expected_mse_len = original_dim.div_ceil(2);
    for (index, code) in mse_codes.iter().enumerate() {
        if code.len() < expected_mse_len {
            return Err(format!(
                "lut32 code {index} too short: got {}, expected at least {expected_mse_len}",
                code.len()
            ));
        }
    }

    let mut block_start = 0usize;
    while block_start + BLOCK_WIDTH <= mse_codes.len() {
        score_block32(
            lut,
            original_dim,
            &mse_codes[block_start..block_start + BLOCK_WIDTH],
            &mut out_scores[block_start..block_start + BLOCK_WIDTH],
        );
        block_start += BLOCK_WIDTH;
    }

    for (code, out_score) in mse_codes[block_start..]
        .iter()
        .zip(out_scores[block_start..].iter_mut())
    {
        *out_score = score_scalar(lut, original_dim, code);
    }

    Ok(())
}

fn score_block32(lut: &[f32], original_dim: usize, codes: &[&[u8]], out_scores: &mut [f32]) {
    debug_assert_eq!(codes.len(), BLOCK_WIDTH);
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
}

fn score_scalar(lut: &[f32], original_dim: usize, code: &[u8]) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::{score_lut_no_qjl_4bit_batch, score_scalar, BLOCK_WIDTH};

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
    fn lut32_matches_scalar_for_blocks_and_tail() {
        let dim = 1536;
        let lut = lut(dim);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH + 7)
            .map(|seed| code(dim, seed as u8))
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; code_refs.len()];

        score_lut_no_qjl_4bit_batch(&lut, dim, &code_refs, &mut scores).unwrap();

        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(score.to_bits(), score_scalar(&lut, dim, code).to_bits());
        }
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
