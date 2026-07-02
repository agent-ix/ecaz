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

pub(crate) fn validate_lut_shape(lut: &[i16], original_dim: usize) -> Result<(), String> {
    if lut.len() != original_dim * 16 {
        return Err(format!(
            "lut32 LUT length mismatch: got {}, expected {}",
            lut.len(),
            original_dim * 16
        ));
    }
    Ok(())
}

#[inline]
pub(super) fn lut_value(lut: &[i16], lut_scale: f32, index: usize) -> f32 {
    lut[index] as f32 * lut_scale
}

#[inline]
#[allow(dead_code)]
pub(super) fn load_dim_table_f32(lut: &[i16], lut_scale: f32, dim_index: usize) -> [f32; 16] {
    let base = dim_index * 16;
    [
        lut_value(lut, lut_scale, base),
        lut_value(lut, lut_scale, base + 1),
        lut_value(lut, lut_scale, base + 2),
        lut_value(lut, lut_scale, base + 3),
        lut_value(lut, lut_scale, base + 4),
        lut_value(lut, lut_scale, base + 5),
        lut_value(lut, lut_scale, base + 6),
        lut_value(lut, lut_scale, base + 7),
        lut_value(lut, lut_scale, base + 8),
        lut_value(lut, lut_scale, base + 9),
        lut_value(lut, lut_scale, base + 10),
        lut_value(lut, lut_scale, base + 11),
        lut_value(lut, lut_scale, base + 12),
        lut_value(lut, lut_scale, base + 13),
        lut_value(lut, lut_scale, base + 14),
        lut_value(lut, lut_scale, base + 15),
    ]
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
    lut: &[i16],
    lut_scale: f32,
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
            lut_scale,
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
        *out_score = score_lut_no_qjl_4bit_scalar(lut, lut_scale, original_dim, code);
    }

    Ok(())
}

pub(crate) fn score_lut_no_qjl_4bit_block32(
    lut: &[i16],
    lut_scale: f32,
    original_dim: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Avx2 => {
            avx2::score_block32_avx2(lut, lut_scale, original_dim, &codes, out_scores)
        }
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve => {
            sve::score_block32_sve(lut, lut_scale, original_dim, &codes, out_scores)
        }
        crate::quant::isa::Isa::Neon => {
            neon::score_block32_neon(lut, lut_scale, original_dim, &codes, out_scores)
        }
        crate::quant::isa::Isa::Scalar => {
            scalar::score_block32_scalar(lut, lut_scale, original_dim, &codes, out_scores)
        }
    }
}

pub(crate) fn score_lut_no_qjl_4bit_batch_tiled(
    lut: &[i16],
    lut_scale: f32,
    original_dim: usize,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    debug_assert_eq!(codes.len(), out_scores.len());
    if codes.is_empty() {
        return crate::quant::isa::Isa::Scalar;
    }

    // Allocation-free NEON driver: full BLOCK_WIDTH slices through the
    // stack-scratch octet kernel, then the entire sub-block remainder in ONE
    // octet-granular partial call (splitting the remainder into separate
    // kernel calls would re-stream the LUT once per call). Capping each
    // invocation at BLOCK_WIDTH keeps every scratch buffer on the stack
    // regardless of batch size (the previous whole-batch tiled kernel
    // heap-allocated its accumulators per call and its transpose columns per
    // 16-byte chunk).
    let host_isa = crate::quant::isa::current_isa();
    if host_isa == crate::quant::isa::Isa::Neon {
        let full_blocks_len = codes.len() - codes.len() % BLOCK_WIDTH;
        let mut start = 0usize;
        let mut neon_available = true;
        while start < full_blocks_len {
            let end = start + BLOCK_WIDTH;
            if neon::score_octets_neon(
                lut,
                lut_scale,
                original_dim,
                &codes[start..end],
                &mut out_scores[start..end],
            )
            .is_none()
            {
                neon_available = false;
                break;
            }
            start = end;
        }
        if neon_available {
            if full_blocks_len < codes.len() {
                score_lut_no_qjl_4bit_partial(
                    lut,
                    lut_scale,
                    original_dim,
                    &codes[full_blocks_len..],
                    &mut out_scores[full_blocks_len..],
                );
            }
            return crate::quant::isa::Isa::Neon;
        }
    }

    let mut block_start = 0usize;
    while block_start + BLOCK_WIDTH <= codes.len() {
        let block = codes[block_start..block_start + BLOCK_WIDTH]
            .try_into()
            .expect("slice length is exactly one block");
        score_lut_no_qjl_4bit_block32(
            lut,
            lut_scale,
            original_dim,
            block,
            &mut out_scores[block_start..block_start + BLOCK_WIDTH],
        );
        block_start += BLOCK_WIDTH;
    }
    if block_start < codes.len() {
        score_lut_no_qjl_4bit_partial(
            lut,
            lut_scale,
            original_dim,
            &codes[block_start..],
            &mut out_scores[block_start..],
        );
    }
    host_isa
}

pub(crate) fn score_lut_no_qjl_4bit_partial(
    lut: &[i16],
    lut_scale: f32,
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
            *out_score = scalar::score_scalar_tail(lut, lut_scale, original_dim, code);
        }
        return crate::quant::isa::Isa::Scalar;
    }

    // SVE hosts score the live width directly: the gather helper's whilelt
    // loop predicates the tail natively, so no padding at all.
    if matches!(
        host_isa,
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve
    ) {
        if let Some(isa) = sve::score_partial_sve(lut, lut_scale, original_dim, codes, out_scores) {
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
        lut_scale,
        original_dim,
        &padded_codes[..padded_len],
        &mut block_scores[..padded_len],
    ) {
        out_scores.copy_from_slice(&block_scores[..codes.len()]);
        return isa;
    }
    if let Some(isa) = neon::score_octets_neon(
        lut,
        lut_scale,
        original_dim,
        &padded_codes[..padded_len],
        &mut block_scores[..padded_len],
    ) {
        out_scores.copy_from_slice(&block_scores[..codes.len()]);
        return isa;
    }

    let isa = score_lut_no_qjl_4bit_block32(
        lut,
        lut_scale,
        original_dim,
        padded_codes,
        &mut block_scores,
    );
    out_scores.copy_from_slice(&block_scores[..codes.len()]);
    isa
}

#[cfg(test)]
pub(crate) fn score_lut_no_qjl_4bit_scalar(
    lut: &[i16],
    lut_scale: f32,
    original_dim: usize,
    code: &[u8],
) -> f32 {
    scalar::score_scalar_tail(lut, lut_scale, original_dim, code)
}

#[cfg(test)]
pub(crate) fn score_lut_no_qjl_4bit_block32_avx2_for_test(
    lut: &[i16],
    lut_scale: f32,
    original_dim: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    avx2::score_block32_avx2_for_test(lut, lut_scale, original_dim, &codes, out_scores)
}

#[cfg(test)]
pub(crate) fn score_lut_no_qjl_4bit_block32_neon_for_test(
    lut: &[i16],
    lut_scale: f32,
    original_dim: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    neon::score_block32_neon_for_test(lut, lut_scale, original_dim, &codes, out_scores)
}

#[cfg(test)]
pub(crate) fn score_lut_no_qjl_4bit_block32_sve_for_test(
    lut: &[i16],
    lut_scale: f32,
    original_dim: usize,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Option<crate::quant::isa::Isa> {
    sve::score_block32_sve_for_test(lut, lut_scale, original_dim, &codes, out_scores)
}

#[cfg(test)]
pub(crate) fn runtime_sve_vector_lanes_for_test() -> Option<usize> {
    sve::runtime_vector_lanes_for_test()
}

#[cfg(test)]
mod tests {
    use super::{
        runtime_sve_vector_lanes_for_test, score_lut_no_qjl_4bit_batch,
        score_lut_no_qjl_4bit_batch_tiled, score_lut_no_qjl_4bit_block32,
        score_lut_no_qjl_4bit_block32_avx2_for_test, score_lut_no_qjl_4bit_block32_neon_for_test,
        score_lut_no_qjl_4bit_block32_sve_for_test, score_lut_no_qjl_4bit_partial,
        score_lut_no_qjl_4bit_scalar, BLOCK_WIDTH,
    };
    use std::hint::black_box;
    use std::time::Instant;

    fn lut(dim: usize) -> (Vec<i16>, f32) {
        let values: Vec<f32> = (0..dim * 16)
            .map(|index| ((index as i32 % 29) - 14) as f32 * 0.125)
            .collect();
        let max_abs = values
            .iter()
            .map(|value| value.abs())
            .fold(0.0_f32, f32::max);
        let scale = max_abs / i16::MAX as f32;
        let compact = values
            .iter()
            .map(|value| {
                (value / scale)
                    .round()
                    .clamp(i16::MIN as f32, i16::MAX as f32) as i16
            })
            .collect();
        (compact, scale)
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
    #[ignore = "Task 132 batch-tiled width microprofile; run explicitly with ECAZ_TQ_PROFILE_LOG"]
    fn task132_profile_lut32_batch_tiled_widths() {
        let dim = 1536;
        let widths: &[usize] = &[8, 16, 24, 32, 39, 48, 64, 128, 256, 512, 1024];
        // Fixed candidate budget per width so each row costs roughly the same
        // wall time; ECAZ_TQ_PROFILE_ITERS scales the budget (default 200k
        // candidates per width).
        let candidate_budget = profile_iterations();
        let (lut, lut_scale) = lut(dim);
        let max_width = *widths.iter().max().expect("widths non-empty");
        let codes: Vec<Vec<u8>> = (0..max_width).map(|seed| code(dim, seed as u8)).collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0_f32; max_width];
        let mut lines = vec![format!(
            "task132 batch_tiled width profile dim={dim} candidate_budget={candidate_budget} isa={:?}",
            crate::quant::isa::current_isa()
        )];
        for &width in widths {
            let iterations = (candidate_budget / width).max(1);
            for _ in 0..iterations.min(64) {
                score_lut_no_qjl_4bit_batch_tiled(
                    black_box(&lut),
                    lut_scale,
                    dim,
                    &code_refs[..width],
                    &mut scores[..width],
                );
                black_box(scores[0]);
            }
            let started = Instant::now();
            for _ in 0..iterations {
                score_lut_no_qjl_4bit_batch_tiled(
                    black_box(&lut),
                    lut_scale,
                    dim,
                    &code_refs[..width],
                    &mut scores[..width],
                );
                black_box(scores[0]);
            }
            let elapsed = started.elapsed();
            let ns_per_candidate = elapsed.as_secs_f64() * 1e9 / (iterations * width) as f64;
            lines.push(format!(
                "width={width} iterations={iterations} ns_per_candidate={ns_per_candidate:.3}"
            ));
        }
        let contents = lines.join("\n");
        println!("{contents}");
        write_profile_log(&contents);
    }

    #[test]
    #[ignore = "Task 124 scorer microprofile; run explicitly with ECAZ_TQ_PROFILE_LOG"]
    fn task124_profile_lut32_block32_and_query_prep() {
        let dim = 1536;
        let iterations = profile_iterations();
        let (lut, lut_scale) = lut(dim);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let code_block: [&[u8]; BLOCK_WIDTH] = code_refs
            .as_slice()
            .try_into()
            .expect("profile fixture has exactly one block");
        let mut scores = [0.0_f32; BLOCK_WIDTH];
        let mut isa = crate::quant::isa::Isa::Scalar;

        for _ in 0..iterations.min(256) {
            isa = score_lut_no_qjl_4bit_block32(
                black_box(&lut),
                lut_scale,
                dim,
                code_block,
                &mut scores,
            );
            black_box(scores[0]);
        }
        let block_started = Instant::now();
        for _ in 0..iterations {
            isa = score_lut_no_qjl_4bit_block32(
                black_box(&lut),
                lut_scale,
                dim,
                code_block,
                &mut scores,
            );
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
        let (lut, lut_scale) = lut(dim);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH - 1)
            .map(|seed| code(dim, seed as u8))
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; code_refs.len()];

        score_lut_no_qjl_4bit_batch(&lut, lut_scale, dim, &code_refs, &mut scores).unwrap();

        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(
                score.to_bits(),
                score_lut_no_qjl_4bit_scalar(&lut, lut_scale, dim, code).to_bits()
            );
        }
    }

    #[test]
    fn lut32_batch_with_blocks_and_tail_matches_scalar_tail_bits() {
        let dim = 1536;
        let (lut, lut_scale) = lut(dim);
        let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH + 7)
            .map(|seed| code(dim, seed as u8))
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let mut scores = vec![0.0; code_refs.len()];

        score_lut_no_qjl_4bit_batch(&lut, lut_scale, dim, &code_refs, &mut scores).unwrap();

        for (code, score) in code_refs.iter().zip(scores.iter()) {
            assert_eq!(
                score.to_bits(),
                score_lut_no_qjl_4bit_scalar(&lut, lut_scale, dim, code).to_bits()
            );
        }
    }

    #[test]
    fn lut32_tiled_batch_matches_scalar_tail_bits_across_widths_and_dims() {
        for dim in [7usize, 8, 1536] {
            for width in [
                7usize,
                8,
                BLOCK_WIDTH - 1,
                BLOCK_WIDTH,
                BLOCK_WIDTH + 7,
                2 * BLOCK_WIDTH,
            ] {
                let (lut, lut_scale) = lut(dim);
                let codes: Vec<Vec<u8>> = (0..width).map(|seed| code(dim, seed as u8)).collect();
                let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
                let mut scores = vec![0.0; code_refs.len()];

                let isa = score_lut_no_qjl_4bit_batch_tiled(
                    &lut,
                    lut_scale,
                    dim,
                    &code_refs,
                    &mut scores,
                );

                for (code, score) in code_refs.iter().zip(scores.iter()) {
                    assert_eq!(
                        score.to_bits(),
                        score_lut_no_qjl_4bit_scalar(&lut, lut_scale, dim, code).to_bits(),
                        "dim={dim} width={width} isa={}",
                        isa.label()
                    );
                }
            }
        }
    }

    #[test]
    fn lut32_block32_matches_scalar_tail_bits_across_dims() {
        for dim in [7usize, 8, 16, 1536] {
            let (lut, lut_scale) = lut(dim);
            let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; BLOCK_WIDTH];

            let isa = score_lut_no_qjl_4bit_block32(
                &lut,
                lut_scale,
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
                    score_lut_no_qjl_4bit_scalar(&lut, lut_scale, dim, code).to_bits(),
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
            let (lut, lut_scale) = lut(dim);
            let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; BLOCK_WIDTH];

            let Some(isa) = score_lut_no_qjl_4bit_block32_avx2_for_test(
                &lut,
                lut_scale,
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
                    score_lut_no_qjl_4bit_scalar(&lut, lut_scale, dim, code).to_bits(),
                    "dim={dim}"
                );
            }
        }
    }

    #[test]
    fn lut32_neon_backend_matches_scalar_reference_bits_when_available() {
        for dim in [7usize, 1536] {
            let (lut, lut_scale) = lut(dim);
            let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; BLOCK_WIDTH];

            let Some(isa) = score_lut_no_qjl_4bit_block32_neon_for_test(
                &lut,
                lut_scale,
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
                    score_lut_no_qjl_4bit_scalar(&lut, lut_scale, dim, code).to_bits(),
                    "dim={dim}"
                );
            }
        }
    }

    #[test]
    fn lut32_sve_backend_matches_scalar_reference_bits_when_available() {
        for dim in [7usize, 1536] {
            let (lut, lut_scale) = lut(dim);
            let codes: Vec<Vec<u8>> = (0..BLOCK_WIDTH).map(|seed| code(dim, seed as u8)).collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; BLOCK_WIDTH];

            let Some(isa) = score_lut_no_qjl_4bit_block32_sve_for_test(
                &lut,
                lut_scale,
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
                    score_lut_no_qjl_4bit_scalar(&lut, lut_scale, dim, code).to_bits(),
                    "dim={dim}"
                );
            }
        }
    }

    #[test]
    fn lut32_partial_matches_scalar_tail_bits_across_widths() {
        let dim = 1536;
        let (lut, lut_scale) = lut(dim);
        // Widths cover the single-lane scalar fast path and every octet
        // count of the AVX2 partial entry, including exact octet multiples.
        for width in [1usize, 2, 7, 8, 9, 16, 17, 25, 31] {
            let codes: Vec<Vec<u8>> = (0..width).map(|seed| code(dim, seed as u8)).collect();
            let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
            let mut scores = vec![0.0; code_refs.len()];

            let isa = score_lut_no_qjl_4bit_partial(&lut, lut_scale, dim, &code_refs, &mut scores);

            for (code, score) in code_refs.iter().zip(scores.iter()) {
                assert_eq!(
                    score.to_bits(),
                    score_lut_no_qjl_4bit_scalar(&lut, lut_scale, dim, code).to_bits(),
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
        let (lut, lut_scale) = lut(8);
        let code = code(8, 1);
        let codes = vec![code.as_slice()];
        let mut scores = vec![0.0, 0.0];

        assert!(score_lut_no_qjl_4bit_batch(&lut, lut_scale, 8, &codes, &mut scores).is_err());
    }
}
