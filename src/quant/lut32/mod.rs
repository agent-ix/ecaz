//! Blocked LUT scoring for TurboQuant no-QJL 4-bit codes.
//!
//! This is the ADR-076 block-kernel surface for the flagship production
//! lane (TurboQuant no-QJL 4-bit). Scalar scoring is the bit-exact
//! reference; ISA-specific modules return the ISA they actually implement.
//! Block scorers return the ISA actually implemented by the selected backend.
//! Counter call sites must use that returned value for kernel rows; scalar
//! tails are still reported separately under `Isa::Scalar`.

mod avx2;
mod neon;
mod scalar;
mod sve;

pub(crate) const BLOCK_WIDTH: usize = 64;

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

pub(crate) fn score_lut_no_qjl_4bit_partial(
    lut: &[f32],
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    debug_assert_eq!(codes.len(), out_scores.len());
    debug_assert!(codes.len() < BLOCK_WIDTH);
    if codes.is_empty() {
        return crate::quant::isa::Isa::Scalar;
    }

    // Scalar hosts score live lanes directly; a single live lane is also
    // cheaper through the scalar tail than through a padded octet.
    let host_isa = crate::quant::isa::current_isa();
    if host_isa == crate::quant::isa::Isa::Scalar || codes.len() == 1 {
        for (code, out_score) in codes.iter().zip(out_scores.iter_mut()) {
            *out_score = scalar::score_scalar_tail(lut, original_dim, code);
        }
        return crate::quant::isa::Isa::Scalar;
    }

    // SVE hosts score the live width directly: the gather helper's whilelt
    // loop predicates the tail natively, so no padding at all.
    if matches!(
        host_isa,
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve
    ) {
        if let Some(isa) = sve::score_partial_sve(lut, original_dim, codes, out_scores) {
            return isa;
        }
    }

    let mut padded_codes = [codes[0]; BLOCK_WIDTH];
    for (lane, code) in codes.iter().enumerate() {
        padded_codes[lane] = *code;
    }
    let mut block_scores = [0.0_f32; BLOCK_WIDTH];

    // AVX2/NEON tails pay only for the octets they occupy; the HNSW
    // exact-mode flush-width histogram is dominated by sub-8 flushes, where
    // a full padded block wastes three quarters of the kernel work.
    let padded_len = codes.len().next_multiple_of(8);
    if let Some(isa) = avx2::score_octets_avx2(
        lut,
        original_dim,
        &padded_codes[..padded_len],
        &mut block_scores[..padded_len],
    ) {
        out_scores.copy_from_slice(&block_scores[..codes.len()]);
        return isa;
    }
    if let Some(isa) = neon::score_octets_neon(
        lut,
        original_dim,
        &padded_codes[..padded_len],
        &mut block_scores[..padded_len],
    ) {
        out_scores.copy_from_slice(&block_scores[..codes.len()]);
        return isa;
    }

    let isa = score_lut_no_qjl_4bit_block32(lut, original_dim, padded_codes, &mut block_scores);
    out_scores.copy_from_slice(&block_scores[..codes.len()]);
    isa
}

pub(crate) fn score_lut_no_qjl_4bit_batch_with_min_bound(
    lut: &[f32],
    suffix_max: &[f32],
    original_dim: usize,
    mse_codes: &[&[u8]],
    min_ip_to_keep: f32,
    out_scores: &mut [f32],
    out_kept: &mut [bool],
) -> crate::quant::isa::Isa {
    debug_assert_eq!(mse_codes.len(), out_scores.len());
    debug_assert_eq!(mse_codes.len(), out_kept.len());
    debug_assert_eq!(suffix_max.len(), original_dim + 1);
    out_kept.fill(false);
    if mse_codes.is_empty() {
        return crate::quant::isa::Isa::Scalar;
    }

    let mut timing_isa = crate::quant::isa::Isa::Scalar;
    let mut block_start = 0usize;
    while block_start + BLOCK_WIDTH <= mse_codes.len() {
        let block = mse_codes[block_start..block_start + BLOCK_WIDTH]
            .try_into()
            .expect("slice length is exactly one block");
        timing_isa = score_lut_no_qjl_4bit_block32_with_min_bound(
            lut,
            suffix_max,
            original_dim,
            block,
            min_ip_to_keep,
            &mut out_scores[block_start..block_start + BLOCK_WIDTH],
            &mut out_kept[block_start..block_start + BLOCK_WIDTH],
        );
        block_start += BLOCK_WIDTH;
    }

    if block_start < mse_codes.len() {
        let isa = score_lut_no_qjl_4bit_partial_with_min_bound(
            lut,
            suffix_max,
            original_dim,
            &mse_codes[block_start..],
            min_ip_to_keep,
            &mut out_scores[block_start..],
            &mut out_kept[block_start..],
        );
        if block_start == 0 {
            timing_isa = isa;
        }
    }

    timing_isa
}

fn score_lut_no_qjl_4bit_block32_with_min_bound(
    lut: &[f32],
    suffix_max: &[f32],
    original_dim: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    min_ip_to_keep: f32,
    out_scores: &mut [f32],
    out_kept: &mut [bool],
) -> crate::quant::isa::Isa {
    #[cfg(target_arch = "aarch64")]
    {
        if let Some(isa) = neon::score_block32_neon_with_min_bound(
            lut,
            suffix_max,
            original_dim,
            &codes,
            min_ip_to_keep,
            out_scores,
            out_kept,
        ) {
            return isa;
        }
    }

    let _ = codes;
    score_lut_no_qjl_4bit_scalar_batch_with_min_bound(
        lut,
        suffix_max,
        original_dim,
        &codes,
        min_ip_to_keep,
        out_scores,
        out_kept,
    )
}

fn score_lut_no_qjl_4bit_partial_with_min_bound(
    lut: &[f32],
    suffix_max: &[f32],
    original_dim: usize,
    codes: &[&[u8]],
    min_ip_to_keep: f32,
    out_scores: &mut [f32],
    out_kept: &mut [bool],
) -> crate::quant::isa::Isa {
    #[cfg(target_arch = "aarch64")]
    {
        let padded_len = codes.len().next_multiple_of(8);
        if padded_len <= BLOCK_WIDTH && codes.len() > 1 {
            let mut padded_codes = [codes[0]; BLOCK_WIDTH];
            for (lane, code) in codes.iter().enumerate() {
                padded_codes[lane] = *code;
            }
            let mut block_scores = [0.0_f32; BLOCK_WIDTH];
            let mut block_kept = [false; BLOCK_WIDTH];
            if let Some(isa) = neon::score_octets_neon_with_min_bound(
                lut,
                suffix_max,
                original_dim,
                &padded_codes[..padded_len],
                min_ip_to_keep,
                &mut block_scores[..padded_len],
                &mut block_kept[..padded_len],
            ) {
                out_scores.copy_from_slice(&block_scores[..codes.len()]);
                out_kept.copy_from_slice(&block_kept[..codes.len()]);
                return isa;
            }
        }
    }

    score_lut_no_qjl_4bit_scalar_batch_with_min_bound(
        lut,
        suffix_max,
        original_dim,
        codes,
        min_ip_to_keep,
        out_scores,
        out_kept,
    )
}

fn score_lut_no_qjl_4bit_scalar_batch_with_min_bound(
    lut: &[f32],
    suffix_max: &[f32],
    original_dim: usize,
    codes: &[&[u8]],
    min_ip_to_keep: f32,
    out_scores: &mut [f32],
    out_kept: &mut [bool],
) -> crate::quant::isa::Isa {
    for ((code, out_score), kept) in codes
        .iter()
        .zip(out_scores.iter_mut())
        .zip(out_kept.iter_mut())
    {
        let mut sum = 0.0_f32;
        for dim_index in 0..original_dim {
            let packed = code[dim_index / 2];
            let code = if dim_index & 1 == 0 {
                packed & 0x0F
            } else {
                packed >> 4
            } as usize;
            sum += lut[dim_index * 16 + code];
            if sum + suffix_max[dim_index + 1] < min_ip_to_keep {
                *out_score = sum;
                *kept = false;
                break;
            }
        }
        if sum + suffix_max[original_dim] >= min_ip_to_keep {
            *out_score = sum;
            *kept = true;
        }
    }
    crate::quant::isa::Isa::Scalar
}

#[cfg(test)]
pub(crate) fn score_lut_no_qjl_4bit_scalar(lut: &[f32], original_dim: usize, code: &[u8]) -> f32 {
    scalar::score_scalar_tail(lut, original_dim, code)
}

#[cfg(test)]
pub(crate) fn score_lut_no_qjl_4bit_block32_avx2_for_test(
    lut: &[f32],
    original_dim: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    avx2::score_block32_avx2_for_test(lut, original_dim, &codes, out_scores)
}

#[cfg(test)]
pub(crate) fn score_lut_no_qjl_4bit_block32_neon_for_test(
    lut: &[f32],
    original_dim: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    neon::score_block32_neon_for_test(lut, original_dim, &codes, out_scores)
}

#[cfg(test)]
pub(crate) fn score_lut_no_qjl_4bit_block32_sve_for_test(
    lut: &[f32],
    original_dim: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    sve::score_block32_sve_for_test(lut, original_dim, &codes, out_scores)
}

#[cfg(test)]
pub(crate) fn runtime_sve_vector_lanes_for_test() -> Option<usize> {
    sve::runtime_vector_lanes_for_test()
}

#[cfg(test)]
mod tests {
    use super::{
        runtime_sve_vector_lanes_for_test, score_lut_no_qjl_4bit_batch,
        score_lut_no_qjl_4bit_block32, score_lut_no_qjl_4bit_block32_avx2_for_test,
        score_lut_no_qjl_4bit_block32_neon_for_test, score_lut_no_qjl_4bit_block32_sve_for_test,
        score_lut_no_qjl_4bit_partial, score_lut_no_qjl_4bit_scalar, BLOCK_WIDTH,
    };
    use std::hint::black_box;
    use std::time::Instant;

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

    fn profile_iterations() -> usize {
        std::env::var("ECAZ_TQ_PROFILE_ITERS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(200_000)
            .max(1)
    }

    fn unit_vector(dim: usize, seed: u64) -> Vec<f32> {
        let mut state = seed;
        let mut values = Vec::with_capacity(dim);
        for _ in 0..dim {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let unit = ((state >> 40) as u32) as f32 / ((1u32 << 24) as f32);
            values.push(unit * 2.0 - 1.0);
        }
        let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
        for value in &mut values {
            *value /= norm.max(f32::EPSILON);
        }
        values
    }

    fn write_profile_log(contents: &str) {
        let Ok(path) = std::env::var("ECAZ_TQ_PROFILE_LOG") else {
            return;
        };
        let path = std::path::PathBuf::from(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create profile log parent");
        }
        std::fs::write(&path, contents).expect("write profile log");
    }

    #[test]
    #[ignore = "Task 124 scorer microprofile; run explicitly with ECAZ_TQ_PROFILE_LOG"]
    fn task124_profile_lut32_block32_and_query_prep() {
        let dim = 1536;
        let iterations = profile_iterations();
        let lut = lut(dim);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let code_block: [&[u8]; BLOCK_WIDTH] = code_refs
            .as_slice()
            .try_into()
            .expect("profile fixture has exactly one block");
        let mut scores = [0.0_f32; BLOCK_WIDTH];
        let mut isa = crate::quant::isa::Isa::Scalar;

        for _ in 0..iterations.min(256) {
            isa = score_lut_no_qjl_4bit_block32(black_box(&lut), dim, code_block, &mut scores);
            black_box(scores[0]);
        }
        let block_started = Instant::now();
        for _ in 0..iterations {
            isa = score_lut_no_qjl_4bit_block32(black_box(&lut), dim, code_block, &mut scores);
            black_box(scores[0]);
        }
        let block_elapsed = block_started.elapsed();
        let block_candidates = iterations * BLOCK_WIDTH;
        let block_ns_per_candidate = block_elapsed.as_secs_f64() * 1e9 / block_candidates as f64;

        let quantizer = crate::quant::prod::ProdQuantizer::new(dim, 4, 42);
        let query = unit_vector(dim, 17);
        let prep_iterations = (iterations / 4).max(1);
        for _ in 0..prep_iterations.min(256) {
            let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(black_box(&query));
            black_box(prepared.lut[0]);
        }
        let prep_started = Instant::now();
        for _ in 0..prep_iterations {
            let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(black_box(&query));
            black_box(prepared.lut[0]);
        }
        let prep_elapsed = prep_started.elapsed();
        let prep_ns = prep_elapsed.as_secs_f64() * 1e9 / prep_iterations as f64;

        let output = format!(
            "task124_lut32_profile backend={} dim={dim} iterations={iterations}\n\
             score_ip_lut_no_qjl_4bit_block32 isa={} total={block_elapsed:?} candidates={block_candidates} ns_per_candidate={block_ns_per_candidate:.1}\n\
             prepare_ip_query_lut_no_qjl_4bit iterations={prep_iterations} total={prep_elapsed:?} ns_per_iter={prep_ns:.1}\n",
            crate::quant::simd_backend_name(),
            isa.label(),
        );
        print!("{output}");
        write_profile_log(&output);
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
    fn lut32_block32_matches_scalar_tail_bits_across_dims() {
        for dim in [7usize, 8, 16, 1536] {
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
                    score_lut_no_qjl_4bit_scalar(&lut, dim, code).to_bits(),
                    "dim={dim}"
                );
            }
            assert!(matches!(
                isa,
                crate::quant::isa::Isa::Scalar
                    | crate::quant::isa::Isa::Neon
                    | crate::quant::isa::Isa::Sve
                    | crate::quant::isa::Isa::Sve2
                    | crate::quant::isa::Isa::Avx2
            ));
        }
    }

    #[test]
    fn lut32_avx2_backend_matches_scalar_reference_bits_when_available() {
        for dim in [7usize, 1536] {
            let lut = lut(dim);
            let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; BLOCK_WIDTH];

            let Some(isa) = score_lut_no_qjl_4bit_block32_avx2_for_test(
                &lut,
                dim,
                code_refs
                    .as_slice()
                    .try_into()
                    .expect("test fixture is exactly one block"),
                &mut scores,
            ) else {
                return;
            };

            assert_eq!(isa, crate::quant::isa::Isa::Avx2);
            for (code, score) in code_refs.iter().zip(scores.iter()) {
                assert_eq!(
                    score.to_bits(),
                    score_lut_no_qjl_4bit_scalar(&lut, dim, code).to_bits(),
                    "dim={dim}"
                );
            }
        }
    }

    #[test]
    fn lut32_neon_backend_matches_scalar_reference_bits_when_available() {
        for dim in [7usize, 1536] {
            let lut = lut(dim);
            let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; BLOCK_WIDTH];

            let Some(isa) = score_lut_no_qjl_4bit_block32_neon_for_test(
                &lut,
                dim,
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
                    score_lut_no_qjl_4bit_scalar(&lut, dim, code).to_bits(),
                    "dim={dim}"
                );
            }
        }
    }

    #[test]
    fn lut32_sve_backend_matches_scalar_reference_bits_when_available() {
        for dim in [7usize, 1536] {
            let lut = lut(dim);
            let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; BLOCK_WIDTH];

            let Some(isa) = score_lut_no_qjl_4bit_block32_sve_for_test(
                &lut,
                dim,
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
                    score_lut_no_qjl_4bit_scalar(&lut, dim, code).to_bits(),
                    "dim={dim}"
                );
            }
        }
    }

    #[test]
    fn lut32_partial_matches_scalar_tail_bits_across_widths() {
        let dim = 1536;
        let lut = lut(dim);
        // Widths cover the single-lane scalar fast path and every octet
        // count of the AVX2 partial entry, including exact octet multiples.
        for width in [1usize, 2, 7, 8, 9, 16, 17, 25, 31] {
            let codes: Vec<Vec<u8>> = (0..width).map(|seed| code(dim, seed as u8)).collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; code_refs.len()];

            let isa = score_lut_no_qjl_4bit_partial(&lut, dim, &code_refs, &mut scores);

            for (code, score) in code_refs.iter().zip(scores.iter()) {
                assert_eq!(
                    score.to_bits(),
                    score_lut_no_qjl_4bit_scalar(&lut, dim, code).to_bits(),
                    "width={width}"
                );
            }
            assert!(matches!(
                isa,
                crate::quant::isa::Isa::Scalar
                    | crate::quant::isa::Isa::Neon
                    | crate::quant::isa::Isa::Sve
                    | crate::quant::isa::Isa::Sve2
                    | crate::quant::isa::Isa::Avx2
            ));
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
