//! 32-candidate blocked RaBitQ bits=1 scoring.
//!
//! Phase 2 lands the scalar reference and safe fallback ISA stubs. The scalar
//! path intentionally matches the forced-scalar byte-LUT operation order used
//! as Task 93's deterministic parity anchor.

mod avx2;
mod neon;
mod scalar;
mod sve;

pub(crate) const BLOCK_WIDTH: usize = 32;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PreparedBits1<'a> {
    pub(crate) dimensions: usize,
    pub(crate) code_len: usize,
    pub(crate) query_rotated: &'a [f32],
    pub(crate) dequant_lut: &'a [f32; 256],
    pub(crate) bits1_byte_lut: &'a [[f32; 8]; 256],
}

impl PreparedBits1<'_> {
    pub(crate) fn validate(self) -> Result<(), String> {
        if self.query_rotated.len() != self.dimensions {
            return Err(format!(
                "rabitq32 query length {} does not match dimensions {}",
                self.query_rotated.len(),
                self.dimensions
            ));
        }
        let expected_code_len =
            self.dimensions.div_ceil(8) + crate::quant::rabitq::RABITQ_SCALAR_LEN;
        if self.code_len != expected_code_len {
            return Err(format!(
                "rabitq32 code length mismatch: got {}, expected {}",
                self.code_len, expected_code_len
            ));
        }
        Ok(())
    }
}

pub(crate) fn validate_code_shape(
    index: usize,
    prepared: PreparedBits1<'_>,
    code: &[u8],
) -> Result<(), String> {
    if code.len() < prepared.code_len {
        return Err(format!(
            "rabitq32 code {index} too short: got {}, expected at least {}",
            code.len(),
            prepared.code_len
        ));
    }
    Ok(())
}

pub(crate) fn score_rabitq_bits1_block32(
    prepared: PreparedBits1<'_>,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Avx2 => avx2::score_block32_avx2(prepared, &codes, out_scores),
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve => {
            // The SVE backend is a Phase C (Graviton) deliverable; until it
            // lands, SVE hosts use the validated NEON backend (every SVE
            // host implements NEON) instead of falling back to scalar.
            sve::score_block32_sve(prepared, &codes, out_scores)
        }
        crate::quant::isa::Isa::Neon => neon::score_block32_neon(prepared, &codes, out_scores),
        crate::quant::isa::Isa::Scalar => {
            scalar::score_block32_scalar(prepared, &codes, out_scores)
        }
    }
}

/// Scores a partial batch (1..=31 candidates: sub-width batches and block
/// tails) through the best available backend. Graph AMs produce neighbor
/// batches bounded by graph degree / survivor budgets that rarely reach the
/// 32-wide block, so the partial path must not fall back to forced-scalar
/// scoring on SIMD hosts. Returns the ISA actually used.
pub(crate) fn score_rabitq_bits1_partial(
    prepared: PreparedBits1<'_>,
    codes: &[&[u8]],
    out_scores: &mut [f32],
) -> crate::quant::isa::Isa {
    debug_assert!(!codes.is_empty() && codes.len() < BLOCK_WIDTH);
    debug_assert_eq!(codes.len(), out_scores.len());
    let isa = crate::quant::isa::current_isa();
    match isa {
        crate::quant::isa::Isa::Neon => neon::score_partial_neon(prepared, codes, out_scores),
        crate::quant::isa::Isa::Avx2 => avx2::score_partial_avx2(prepared, codes, out_scores),
        crate::quant::isa::Isa::Sve2 | crate::quant::isa::Isa::Sve => {
            // SVE hosts also have NEON; use the validated NEON partial path
            // until a real SVE backend lands (Phase C, Graviton lane).
            neon::score_partial_neon(prepared, codes, out_scores)
        }
        crate::quant::isa::Isa::Scalar => scalar::score_partial_scalar(prepared, codes, out_scores),
    }
}

/// Forced-scalar single-candidate reference; retained as the strict parity
/// anchor entry point for tests and diagnostics.
#[allow(dead_code)]
pub(crate) fn score_rabitq_bits1_scalar(prepared: PreparedBits1<'_>, code: &[u8]) -> f32 {
    scalar::score_scalar_tail(prepared, code)
}

#[allow(dead_code)]
pub(crate) fn raw_popcount_bits1(code: &[u8], dimensions: usize) -> u32 {
    scalar::raw_popcount_bits1(code, dimensions)
}

#[cfg(test)]
mod tests {
    use super::{
        score_rabitq_bits1_block32, score_rabitq_bits1_scalar, PreparedBits1, BLOCK_WIDTH,
    };
    use crate::quant::rabitq::RaBitQQuantizer;
    use crate::quant::Quantizer;

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

    fn prepared(
        dimensions: usize,
    ) -> (
        std::sync::Arc<RaBitQQuantizer>,
        crate::quant::rabitq::PreparedEstimator,
    ) {
        let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(dimensions, 42, 1)
            .expect("test quantizer should be valid");
        let query = vector(dimensions, 7);
        let prepared = quantizer.prepare_estimator(&query);
        (quantizer, prepared)
    }

    /// ADR-076 tolerance: pass when within 4 ULP or 1e-6 relative,
    /// whichever is larger.
    fn assert_close(actual: f32, expected: f32) {
        let ulp = ulp_distance(actual, expected);
        let abs = (actual - expected).abs();
        let rel = 1e-6_f32 * actual.abs().max(expected.abs()).max(1.0);
        assert!(
            ulp <= 4 || abs <= rel,
            "actual={actual} expected={expected} ulp={ulp} abs={abs} rel={rel}"
        );
    }

    fn ulp_distance(a: f32, b: f32) -> u32 {
        if a == b {
            return 0;
        }
        if a.is_sign_positive() != b.is_sign_positive() {
            return u32::MAX;
        }
        a.to_bits().abs_diff(b.to_bits())
    }

    /// SIMD-vs-scalar-anchor envelope. FMA accumulator trees reorder the
    /// summation, so the strict ADR-076 bound (4 ULP / 1e-6 relative) is not
    /// achievable at high dimensions: measured 22 ULP (1.55e-6 relative) at
    /// dim=1536 on NEON. This matches the existing production differential
    /// precedent in `rabitq.rs` tests (`1e-5`); the binding correctness gate
    /// for SIMD backends is recall byte-equality at bench level plus
    /// bit-equality with the same-order production SIMD batch path.
    fn assert_close_simd(actual: f32, expected: f32) {
        let ulp = ulp_distance(actual, expected);
        let abs = (actual - expected).abs();
        let bound = 1e-5_f32 * actual.abs().max(expected.abs()).max(1.0);
        assert!(
            ulp <= 4 || abs <= bound,
            "actual={actual} expected={expected} ulp={ulp} abs={abs} bound={bound}"
        );
    }

    #[test]
    fn scalar_tail_is_forced_scalar_anchor() {
        let (quantizer, prepared) = prepared(37);
        let code = quantizer.encode_code(&vector(37, 11));
        let block_prepared = prepared
            .bits1_block_prepared(quantizer.code_len())
            .expect("bits=1 block prepared should exist");
        let score = score_rabitq_bits1_scalar(block_prepared, &code);
        let anchor = forced_scalar_anchor(block_prepared, &code);

        assert_eq!(score.to_bits(), anchor.to_bits());
    }

    #[test]
    fn scalar_block32_matches_forced_scalar_anchor_bits() {
        let (quantizer, prepared) = prepared(65);
        let codes: Vec<_> = (0..BLOCK_WIDTH)
            .map(|seed| {
                quantizer
                    .encode_code(&vector(65, seed as u64 + 100))
                    .into_vec()
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let block_prepared = prepared
            .bits1_block_prepared(quantizer.code_len())
            .expect("bits=1 block prepared should exist");
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let isa = super::scalar::score_block32_scalar(
            block_prepared,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            &mut scores,
        );

        for (score, code) in scores.iter().zip(code_refs.iter()) {
            let anchor = forced_scalar_anchor(block_prepared, code);
            assert_eq!(score.to_bits(), anchor.to_bits());
        }
        assert_eq!(isa, crate::quant::isa::Isa::Scalar);
    }

    #[test]
    fn dispatched_block32_matches_anchor_within_tolerance() {
        let (quantizer, prepared) = prepared(1536);
        let codes: Vec<_> = (0..BLOCK_WIDTH)
            .map(|seed| {
                quantizer
                    .encode_code(&vector(1536, seed as u64 + 300))
                    .into_vec()
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let block_prepared = prepared
            .bits1_block_prepared(quantizer.code_len())
            .expect("bits=1 block prepared should exist");
        let mut scores = vec![0.0; BLOCK_WIDTH];

        let isa = score_rabitq_bits1_block32(
            block_prepared,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            &mut scores,
        );

        for (score, code) in scores.iter().zip(code_refs.iter()) {
            assert_close_simd(*score, forced_scalar_anchor(block_prepared, code));
        }
        assert_eq!(isa, host_expected_simd_isa());
    }

    fn host_expected_simd_isa() -> crate::quant::isa::Isa {
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                return crate::quant::isa::Isa::Neon;
            }
        }
        #[cfg(target_arch = "x86_64")]
        {
            if std::arch::is_x86_feature_detected!("avx2")
                && std::arch::is_x86_feature_detected!("fma")
            {
                return crate::quant::isa::Isa::Avx2;
            }
        }
        crate::quant::isa::Isa::Scalar
    }

    #[test]
    fn partial_dispatch_matches_anchor_and_production_batch() {
        let (quantizer, prepared) = prepared(1536);
        let codes: Vec<_> = (0..7)
            .map(|seed| {
                quantizer
                    .encode_code(&vector(1536, seed as u64 + 900))
                    .into_vec()
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let block_prepared = prepared
            .bits1_block_prepared(quantizer.code_len())
            .expect("bits=1 block prepared should exist");
        let mut scores = vec![0.0; codes.len()];

        let isa = super::score_rabitq_bits1_partial(block_prepared, &code_refs, &mut scores);

        for (score, code) in scores.iter().zip(code_refs.iter()) {
            assert_close_simd(*score, forced_scalar_anchor(block_prepared, code));
        }
        // The partial NEON path pairs candidates exactly like the production
        // batch slab scorer, so scores are bit-equal on NEON hosts; the
        // scalar partial path is bit-equal with the forced-scalar anchor.
        let slab = codes.iter().flatten().copied().collect::<Vec<_>>();
        let mut batch_scores = Vec::new();
        prepared
            .estimate_ip_bits1_batch(&slab, quantizer.code_len(), &mut batch_scores)
            .unwrap();
        if isa != crate::quant::isa::Isa::Scalar {
            for (score, production) in scores.iter().zip(batch_scores.iter()) {
                assert_eq!(score.to_bits(), production.to_bits());
            }
        } else {
            for (score, code) in scores.iter().zip(code_refs.iter()) {
                assert_eq!(
                    score.to_bits(),
                    forced_scalar_anchor(block_prepared, code).to_bits()
                );
            }
        }
    }

    #[test]
    fn simd_block32_is_bit_equal_with_production_batch() {
        let expected_isa = host_expected_simd_isa();
        if expected_isa == crate::quant::isa::Isa::Scalar {
            return;
        }
        let (quantizer, prepared) = prepared(1536);
        let codes: Vec<_> = (0..BLOCK_WIDTH)
            .map(|seed| {
                quantizer
                    .encode_code(&vector(1536, seed as u64 + 500))
                    .into_vec()
            })
            .collect();
        let code_refs: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
        let block_prepared = prepared
            .bits1_block_prepared(quantizer.code_len())
            .expect("bits=1 block prepared should exist");
        let mut kernel_scores = vec![0.0; BLOCK_WIDTH];

        let isa = score_rabitq_bits1_block32(
            block_prepared,
            code_refs
                .as_slice()
                .try_into()
                .expect("test fixture is exactly one block"),
            &mut kernel_scores,
        );
        assert_eq!(isa, expected_isa);

        let slab = codes.iter().flatten().copied().collect::<Vec<_>>();
        let mut batch_scores = Vec::new();
        prepared
            .estimate_ip_bits1_batch(&slab, quantizer.code_len(), &mut batch_scores)
            .unwrap();

        // The kernel and the production batch slab scorer call the same
        // per-ISA pair primitive with the same candidate pairing, so scores
        // are equal by construction on both NEON and AVX2 hosts. A mismatch
        // means the kernel and production paths diverged.
        for (kernel, production) in kernel_scores.iter().zip(batch_scores.iter()) {
            assert_eq!(kernel.to_bits(), production.to_bits());
        }
    }

    #[test]
    fn production_dispatch_is_within_phase2_tolerance() {
        let (quantizer, prepared) = prepared(65);
        let codes: Vec<_> = (0..(BLOCK_WIDTH + 3))
            .map(|seed| {
                quantizer
                    .encode_code(&vector(65, seed as u64 + 200))
                    .into_vec()
            })
            .collect();
        let slab = codes.iter().flatten().copied().collect::<Vec<_>>();
        let mut batch_scores = Vec::new();
        prepared
            .estimate_ip_bits1_batch(&slab, quantizer.code_len(), &mut batch_scores)
            .unwrap();
        let block_prepared = prepared
            .bits1_block_prepared(quantizer.code_len())
            .expect("bits=1 block prepared should exist");

        for (index, code) in codes.iter().enumerate() {
            let scalar_anchor = score_rabitq_bits1_scalar(block_prepared, code);
            assert_close(scalar_anchor, prepared.estimate_ip_scalar_only(code));
            assert_close(batch_scores[index], prepared.estimate_ip_scalar_only(code));
        }
    }

    fn forced_scalar_anchor(prepared: PreparedBits1<'_>, code: &[u8]) -> f32 {
        let mut sum = 0.0_f32;
        let mut dim_index = 0usize;
        while dim_index + 8 <= prepared.dimensions {
            let row = &prepared.bits1_byte_lut[code[dim_index / 8] as usize];
            sum += prepared.query_rotated[dim_index] * row[0]
                + prepared.query_rotated[dim_index + 1] * row[1]
                + prepared.query_rotated[dim_index + 2] * row[2]
                + prepared.query_rotated[dim_index + 3] * row[3]
                + prepared.query_rotated[dim_index + 4] * row[4]
                + prepared.query_rotated[dim_index + 5] * row[5]
                + prepared.query_rotated[dim_index + 6] * row[6]
                + prepared.query_rotated[dim_index + 7] * row[7];
            dim_index += 8;
        }
        while dim_index < prepared.dimensions {
            let bit = (code[dim_index / 8] >> (dim_index % 8)) & 1;
            sum += prepared.query_rotated[dim_index] * prepared.bits1_byte_lut[usize::from(bit)][0];
            dim_index += 1;
        }

        let packed_bytes = prepared.dimensions.div_ceil(8);
        let candidate_norm =
            f32::from_le_bytes(code[packed_bytes..packed_bytes + 4].try_into().unwrap());
        let candidate_o_dot =
            f32::from_le_bytes(code[packed_bytes + 4..packed_bytes + 8].try_into().unwrap());
        let candidate_x_norm = f32::from_le_bytes(
            code[packed_bytes + 8..packed_bytes + 12]
                .try_into()
                .unwrap(),
        );
        if candidate_o_dot.abs() < 1e-6
            || !candidate_o_dot.is_finite()
            || candidate_x_norm <= 0.0
            || !candidate_x_norm.is_finite()
        {
            return 0.0;
        }
        candidate_norm * sum / (candidate_o_dot * candidate_x_norm)
    }
}
