//! 32-candidate blocked grouped-PQ scoring.
//!
//! This module is the ADR-076 block-kernel surface for PqFastScan /
//! grouped-PQ search codes. Scalar scoring is the bit-exact reference;
//! ISA-specific modules return the ISA they actually implement.

mod avx2;
mod neon;
mod scalar;
mod sve;

pub(crate) const BLOCK_WIDTH: usize = 32;

pub(crate) fn expected_code_len(group_count: usize) -> usize {
    group_count.div_ceil(2)
}

pub(crate) fn validate_lut_shape(lut: &[f32], group_count: usize) -> Result<(), String> {
    let expected = group_count * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS;
    if lut.len() != expected {
        return Err(format!(
            "grouped_pq_block LUT length mismatch: got {}, expected {}",
            lut.len(),
            expected
        ));
    }
    Ok(())
}

pub(crate) fn validate_code_shape(
    index: usize,
    group_count: usize,
    code: &[u8],
) -> Result<(), String> {
    let expected = expected_code_len(group_count);
    if code.len() < expected {
        return Err(format!(
            "grouped_pq_block code {index} too short: got {}, expected at least {expected}",
            code.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn score_grouped_pq_batch(
    lut: &[f32],
    group_count: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> Result<(), String> {
    if codes.len() != out_scores.len() {
        return Err(format!(
            "grouped_pq_block score output count {} does not match candidate count {}",
            out_scores.len(),
            codes.len()
        ));
    }
    validate_lut_shape(lut, group_count)?;
    for (index, code) in codes.iter().enumerate() {
        validate_code_shape(index, group_count, code)?;
    }

    let mut block_start = 0usize;
    while block_start + BLOCK_WIDTH <= codes.len() {
        let _ = score_grouped_pq_block32(
            lut,
            group_count,
            codes[block_start..block_start + BLOCK_WIDTH]
                .try_into()
                .expect("slice length is exactly one block"),
            &mut out_scores[block_start..block_start + BLOCK_WIDTH],
        );
        block_start += BLOCK_WIDTH;
    }

    for (code, out_score) in codes[block_start..]
        .iter()
        .zip(out_scores[block_start..].iter_mut())
    {
        *out_score = score_grouped_pq_scalar(lut, group_count, code);
    }

    Ok(())
}

pub(crate) fn score_grouped_pq_block32(
    lut: &[f32],
    group_count: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Avx2 => {
            avx2::score_block32_avx2(lut, group_count, &codes, out_scores)
        }
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve => {
            sve::score_block32_sve(lut, group_count, &codes, out_scores)
        }
        crate::quant::isa::Isa::Neon => {
            neon::score_block32_neon(lut, group_count, &codes, out_scores)
        }
        crate::quant::isa::Isa::Scalar => {
            scalar::score_block32_scalar(lut, group_count, &codes, out_scores)
        }
    }
}

pub(crate) fn score_grouped_pq_scalar(lut: &[f32], group_count: usize, code: &[u8]) -> f32 {
    scalar::score_scalar_tail(lut, group_count, code)
}

#[cfg(test)]
pub(crate) fn score_grouped_pq_block32_neon_for_test(
    lut: &[f32],
    group_count: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    neon::score_block32_neon_for_test(lut, group_count, &codes, out_scores)
}

#[cfg(test)]
pub(crate) fn score_grouped_pq_block32_sve_for_test(
    lut: &[f32],
    group_count: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    sve::score_block32_sve_for_test(lut, group_count, &codes, out_scores)
}

#[cfg(test)]
pub(crate) fn runtime_sve_vector_lanes_for_test() -> Option<usize> {
    sve::runtime_vector_lanes_for_test()
}

#[cfg(test)]
mod tests {
    use super::{
        runtime_sve_vector_lanes_for_test, score_grouped_pq_batch, score_grouped_pq_block32,
        score_grouped_pq_block32_neon_for_test, score_grouped_pq_block32_sve_for_test,
        score_grouped_pq_scalar, BLOCK_WIDTH,
    };
    use crate::quant::grouped_pq::{grouped_pq_score_f32, pack_grouped_pq_nibbles};

    fn lut(group_count: usize) -> Vec<f32> {
        let mut state = 0x9E37_79B9_7F4A_7C15_u64 ^ group_count as u64;
        (0..group_count * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS)
            .map(|_| {
                state = state
                    .wrapping_mul(0xBF58_476D_1CE4_E5B9)
                    .wrapping_add(0x94D0_49BB_1331_11EB);
                let raw = ((state >> 24) & 0xFFFF) as i32 - 32731;
                (raw as f32 * 0.000_37) + 0.000_13
            })
            .collect()
    }

    fn code(group_count: usize, seed: u8) -> Vec<u8> {
        let indices: Vec<u8> = (0..group_count)
            .map(|group| {
                seed.wrapping_add((group as u8).wrapping_mul(7))
                    .wrapping_add((group as u8) >> 1)
                    & 0x0F
            })
            .collect();
        pack_grouped_pq_nibbles(&indices)
    }

    #[test]
    fn grouped_pq_block32_matches_scalar_reference_bits_across_shapes() {
        for group_count in [7usize, 8, 16, 32] {
            let lut = lut(group_count);
            let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH)
                .map(|seed| code(group_count, seed as u8))
                .collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; BLOCK_WIDTH];

            let isa = score_grouped_pq_block32(
                &lut,
                group_count,
                code_refs
                    .as_slice()
                    .try_into()
                    .expect("test fixture is exactly one block"),
                &mut scores,
            );

            for (code, score) in code_refs.iter().zip(scores.iter()) {
                assert_eq!(
                    score.to_bits(),
                    grouped_pq_score_f32(&lut, group_count, code).to_bits(),
                    "group_count={group_count}"
                );
            }
            assert_eq!(isa, crate::quant::isa::Isa::Scalar);
        }
    }

    #[test]
    fn grouped_pq_neon_backend_matches_scalar_reference_bits_when_available() {
        let group_count = 16;
        let lut = lut(group_count);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH)
            .map(|seed| code(group_count, seed as u8))
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let Some(isa) = score_grouped_pq_block32_neon_for_test(
            &lut,
            group_count,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            &mut scores,
        ) else {
            return;
        };

        assert_eq!(isa, crate::quant::isa::Isa::Neon);
        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(
                score.to_bits(),
                grouped_pq_score_f32(&lut, group_count, code).to_bits()
            );
        }
    }

    #[test]
    fn grouped_pq_sve_backend_matches_scalar_reference_bits_when_available() {
        let group_count = 16;
        let lut = lut(group_count);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH)
            .map(|seed| code(group_count, seed as u8))
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let Some(isa) = score_grouped_pq_block32_sve_for_test(
            &lut,
            group_count,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            &mut scores,
        ) else {
            return;
        };

        assert!(matches!(
            isa,
            crate::quant::isa::Isa::Sve | crate::quant::isa::Isa::Sve2
        ));
        assert!(runtime_sve_vector_lanes_for_test().is_some());
        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(
                score.to_bits(),
                grouped_pq_score_f32(&lut, group_count, code).to_bits()
            );
        }
    }

    #[test]
    fn grouped_pq_batch_under_block_width_matches_scalar_reference_bits() {
        let group_count = 16;
        let lut = lut(group_count);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH - 1)
            .map(|seed| code(group_count, seed as u8))
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; code_refs.len()];

        score_grouped_pq_batch(&lut, group_count, &code_refs, &mut scores).unwrap();

        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(
                score.to_bits(),
                grouped_pq_score_f32(&lut, group_count, code).to_bits()
            );
        }
    }

    #[test]
    fn grouped_pq_batch_with_block_and_tail_matches_scalar_reference_bits() {
        let group_count = 32;
        let lut = lut(group_count);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH + 7)
            .map(|seed| code(group_count, seed as u8))
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; code_refs.len()];

        score_grouped_pq_batch(&lut, group_count, &code_refs, &mut scores).unwrap();

        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(
                score.to_bits(),
                grouped_pq_score_f32(&lut, group_count, code).to_bits()
            );
        }
    }

    #[test]
    fn grouped_pq_scalar_tail_matches_scalar_reference_bits() {
        let group_count = 7;
        let lut = lut(group_count);
        let code = code(group_count, 19);

        assert_eq!(
            score_grouped_pq_scalar(&lut, group_count, &code).to_bits(),
            grouped_pq_score_f32(&lut, group_count, &code).to_bits()
        );
    }

    #[test]
    fn grouped_pq_batch_rejects_shape_mismatch() {
        let lut = lut(8);
        let code = code(8, 1);
        let codes = vec![code.as_slice()];
        let mut scores = vec![0.0, 0.0];

        assert!(score_grouped_pq_batch(&lut, 8, &codes, &mut scores).is_err());

        let mut scores = vec![0.0];
        assert!(score_grouped_pq_batch(&lut[..lut.len() - 1], 8, &codes, &mut scores).is_err());

        let short_code = [0_u8; 3];
        let short_codes = vec![short_code.as_slice()];
        assert!(score_grouped_pq_batch(&lut, 8, &short_codes, &mut scores).is_err());
    }
}
