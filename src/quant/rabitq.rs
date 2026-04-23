//! RaBitQ quantizer — ADR-045 Stage 1 (supersedes ADR-031 in scope).
//!
//! Graduates the ADR-031 binary-prefilter work into a first-class
//! quantizer under the `Quantizer` / `QueryScorer` trait seams.
//!
//! ## Module layout — what lives where now
//!
//! - The sign-bit pack primitives (`sign_words_from_rotated`,
//!   `sign_words_from_packed_4bit`, `hamming_similarity`, and the
//!   4-bit codebook sign lookup) moved here from `src/quant/prod.rs`.
//!   [`ProdQuantizer`]'s `binary_sign_*` methods now delegate into
//!   this module; they stay on `ProdQuantizer` because they need its
//!   codebook/signs state and are called from the hot AM paths
//!   through a concrete quantizer reference (not through `&dyn
//!   Quantizer`).
//! - The persisted-sidecar helpers moved here from
//!   `src/am/common/training.rs` (`derive_persisted_sidecar_words`,
//!   `persisted_sidecar_word_count`). AM callers now reach into
//!   `crate::quant::rabitq` directly; the `training.rs` wrappers are
//!   deleted per the "deprecate = delete" rule.
//!
//! ## Slice plan
//!
//! | slice | lands                                                                    |
//! |-------|--------------------------------------------------------------------------|
//! | 1     | skeleton: types, public surface, stubbed trait impls                     |
//! | 2     | **this slice** — move primitives + real `Quantizer` / `QueryScorer`      |
//! | 3     | rotation front-end seam (SRHT today, OPQ later per task 20)              |
//! | 4     | unbiased distance estimator + error-bound API (Stage 3 consumes this)    |
//! | 5     | Phase 2 recall study via `src/bin/rabitq_feasibility.rs`                 |
//!
//! ## Code layout at D=1536
//!
//! `code_len() = dim.div_ceil(8) + 4` → 192 sign bytes + 4-byte `f32`
//! rotated-vector norm = 196 B (PQ4 parity is 768 B; the Stage 1 gate
//! asks recall@10 within 1pp of exact at this storage).

#![allow(dead_code)]

use std::sync::Arc;

use crate::quant::prod::ProdQuantizer;
use crate::quant::rotation;

/// Bytes per vector holding the rotated L2 norm `||c||`.
pub const RABITQ_NORM_LEN: usize = 4;
/// Bytes per vector holding `α_c = mean(|c_i|)` over rotated coordinates.
/// Consumed by the slice-4 unbiased estimator.
pub const RABITQ_ALPHA_LEN: usize = 4;
/// Total scalar tail on each code: `||c||` plus `α_c`.
pub const RABITQ_SCALAR_LEN: usize = RABITQ_NORM_LEN + RABITQ_ALPHA_LEN;

/// Binary-sign prepared query. Produced by
/// [`ProdQuantizer::prepare_ip_query_binary_sign_no_qjl_4bit`] and
/// consumed by the Hamming scorer. Moved here from `prod.rs` in
/// slice 2 so the RaBitQ scoring surface is self-contained.
#[derive(Debug, Clone, PartialEq)]
pub struct BinarySignNoQjl4BitQuery {
    pub words: Vec<u64>,
}

/// Rotation front-end seam.
///
/// A `Rotation` turns a raw D-dimensional vector into a rotated
/// D-dimensional vector whose sign bits RaBitQ will pack. The trait
/// exists so ADR-036 OPQ (task 20) or a learned rotation can replace
/// SRHT without touching the encoder, the scorer, or the estimator.
/// `Send + Sync + 'static` so the rotation can be shared via `Arc`
/// between build and scan paths.
pub trait Rotation: Send + Sync {
    /// Rotated output dimensionality. Must equal the input
    /// dimensionality — RaBitQ's code length is `dim/8 + 4` and does
    /// not carry padding information.
    fn dimensions(&self) -> usize;

    /// Apply the rotation. Implementations must return exactly
    /// `dimensions()` coordinates, even if they pad internally.
    fn apply(&self, v: &[f32]) -> Vec<f32>;
}

/// SRHT rotation backed by a `ProdQuantizer`'s sign vector. This is
/// the default `Rotation` during ADR-045 Stage 1; it reuses the
/// quantizer's existing SRHT state so the canonical RaBitQ encode
/// and the ADR-031 PQ-derived sidecar land on the same rotated basis.
pub struct SrhtRotation {
    dimensions: usize,
    prod: Arc<ProdQuantizer>,
}

impl SrhtRotation {
    pub fn new(dimensions: usize, prod: Arc<ProdQuantizer>) -> Self {
        assert_eq!(
            prod.original_dim, dimensions,
            "SRHT rotation dimensions mismatch: quantizer has {}, asked for {}",
            prod.original_dim, dimensions,
        );
        Self { dimensions, prod }
    }

    /// Access the underlying `ProdQuantizer`. Used by the ADR-031
    /// PQ-derived sidecar helpers, which need the codebook state.
    pub fn prod(&self) -> &Arc<ProdQuantizer> {
        &self.prod
    }
}

impl Rotation for SrhtRotation {
    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn apply(&self, v: &[f32]) -> Vec<f32> {
        assert_eq!(
            v.len(),
            self.dimensions,
            "SRHT input length mismatch: got {}, expected {}",
            v.len(),
            self.dimensions,
        );
        let padded = rotation::srht_padded(v, &self.prod.signs);
        padded[..self.dimensions].to_vec()
    }
}

/// One RaBitQ quantizer instance. Owns the rotation via the
/// [`Rotation`] trait object seam and the per-vector encoding
/// parameters. Build and scan paths share one `Arc<RaBitQQuantizer>`.
pub struct RaBitQQuantizer {
    dimensions: usize,
    rotation: Arc<dyn Rotation>,
}

impl RaBitQQuantizer {
    pub fn new(rotation: Arc<dyn Rotation>) -> Self {
        let dimensions = rotation.dimensions();
        assert!(dimensions > 0, "RaBitQ dimensions must be positive");
        Self {
            dimensions,
            rotation,
        }
    }

    /// Convenience: build a RaBitQ quantizer with the default SRHT
    /// rotation sourced from a `ProdQuantizer`. This is the
    /// ADR-045 Stage 1 entry point.
    pub fn with_srht(dimensions: usize, prod: Arc<ProdQuantizer>) -> Self {
        let rotation: Arc<dyn Rotation> = Arc::new(SrhtRotation::new(dimensions, prod));
        Self::new(rotation)
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Byte length of the sign-bit portion of a code (pre-norm).
    pub fn sign_bytes(&self) -> usize {
        self.dimensions.div_ceil(8)
    }

    fn rotated(&self, v: &[f32]) -> Vec<f32> {
        let out = self.rotation.apply(v);
        debug_assert_eq!(out.len(), self.dimensions);
        out
    }
}

impl crate::quant::Quantizer for RaBitQQuantizer {
    /// Canonical RaBitQ encode: rotate, take the sign bit of each
    /// rotated coordinate into a `dim/8`-byte payload, then append
    /// two f32 scalars — the L2 norm `||c||` and
    /// `α_c = mean(|c_i|)`. Slice 4 consumes both in the unbiased
    /// estimator and its Cauchy-Schwarz error bound.
    fn encode_code(&self, v: &[f32]) -> Box<[u8]> {
        let rotated = self.rotated(v);
        let mut out = vec![0_u8; self.sign_bytes() + RABITQ_SCALAR_LEN];
        for (index, &value) in rotated.iter().enumerate() {
            if value >= 0.0 {
                out[index / 8] |= 1_u8 << (index % 8);
            }
        }
        let norm = l2_norm(&rotated);
        let alpha = mean_abs(&rotated);
        let norm_start = self.sign_bytes();
        out[norm_start..norm_start + RABITQ_NORM_LEN].copy_from_slice(&norm.to_le_bytes());
        out[norm_start + RABITQ_NORM_LEN..norm_start + RABITQ_SCALAR_LEN]
            .copy_from_slice(&alpha.to_le_bytes());
        out.into_boxed_slice()
    }

    fn prepare_scorer(
        &self,
        query: &[f32],
    ) -> Box<dyn crate::quant::QueryScorer + Send + Sync + '_> {
        let rotated = self.rotated(query);
        let norm = l2_norm(&rotated);
        Box::new(RaBitQScorer {
            query_rotated: rotated,
            query_norm: norm,
            dimensions: self.dimensions,
        })
    }

    fn code_len(&self) -> usize {
        self.sign_bytes() + RABITQ_SCALAR_LEN
    }

    fn wire_format_version(&self) -> u32 {
        0
    }
}

impl RaBitQQuantizer {
    /// Prepare the estimator state for `query`. The returned
    /// `PreparedEstimator` holds the rotated query coordinates in
    /// full f32 precision (the asymmetric half of the estimator) and
    /// the query's L2 norm.
    pub fn prepare_estimator(&self, query: &[f32]) -> PreparedEstimator {
        let rotated = self.rotated(query);
        let norm = l2_norm(&rotated);
        PreparedEstimator {
            query_rotated: rotated,
            query_norm: norm,
            dimensions: self.dimensions,
        }
    }

    /// Unbiased inner-product estimate between `prepared` and a
    /// code, plus a Cauchy-Schwarz error bound. See [`DistanceEstimate`].
    pub fn estimate_ip(&self, prepared: &PreparedEstimator, code: &[u8]) -> DistanceEstimate {
        estimate_ip_impl(
            &prepared.query_rotated,
            prepared.query_norm,
            self.dimensions,
            code,
        )
    }
}

/// Prepared estimator state for one query. Separate from
/// `RaBitQScorer` so callers that want just the scalar inner-product
/// estimate can use the plain `QueryScorer::score` trait seam, while
/// callers that need the error bound (Stage 3, task 27) go through
/// [`RaBitQQuantizer::estimate_ip`].
pub struct PreparedEstimator {
    query_rotated: Vec<f32>,
    query_norm: f32,
    dimensions: usize,
}

impl PreparedEstimator {
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }
    pub fn query_norm(&self) -> f32 {
        self.query_norm
    }
}

/// Prepared scorer for the `QueryScorer` trait. Holds the rotated
/// query so `score` can compute the asymmetric RaBitQ estimator. The
/// bound-carrying form lives at [`RaBitQQuantizer::estimate_ip`].
pub struct RaBitQScorer {
    query_rotated: Vec<f32>,
    query_norm: f32,
    dimensions: usize,
}

impl crate::quant::QueryScorer for RaBitQScorer {
    fn score(&self, code: &[u8]) -> f32 {
        estimate_ip_impl(&self.query_rotated, self.query_norm, self.dimensions, code).estimate
    }
}

/// Unbiased distance estimate with a symmetric error bound
/// (`estimate ± bound`). `estimate` is the inner product ⟨q, c⟩
/// recovered from the asymmetric RaBitQ estimator; `bound` is the
/// Cauchy-Schwarz envelope `||q|| * ||c - α_c·sign(c)||`. Stage 3
/// (task 27) sizes its candidate pool from the empirical
/// distribution of `bound` measured in Phase 2.
#[derive(Debug, Clone, Copy)]
pub struct DistanceEstimate {
    pub estimate: f32,
    pub bound: f32,
}

// ---------------------------------------------------------------------------
// Primitives — moved from `src/quant/prod.rs` in slice 2.
// ---------------------------------------------------------------------------

/// Pack sign bits of a rotated f32 vector into 64-bit words.
/// Word layout: bit `index % 64` of word `index / 64` carries the
/// sign of coordinate `index`.
pub(crate) fn sign_words_from_rotated(rotated: &[f32]) -> Vec<u64> {
    let mut words = vec![0_u64; rotated.len().div_ceil(64)];
    for (index, value) in rotated.iter().copied().enumerate() {
        if value >= 0.0 {
            words[index / 64] |= 1_u64 << (index % 64);
        }
    }
    words
}

/// Derive sign words from a 4-bit packed MSE code by looking up the
/// sign of each codebook entry. `sign_lookup[i] != 0` means codebook
/// entry `i` is non-negative (bit set). Word layout matches
/// [`sign_words_from_rotated`].
pub(crate) fn sign_words_from_packed_4bit(
    code_bytes: &[u8],
    dim: usize,
    sign_lookup: &[u8; 16],
) -> Vec<u64> {
    let mut words = vec![0_u64; dim.div_ceil(64)];
    let mut dim_index = 0usize;

    for &packed in code_bytes {
        if dim_index >= dim {
            break;
        }

        let low_nibble = (packed & 0x0F) as usize;
        if sign_lookup[low_nibble] != 0 {
            words[dim_index / 64] |= 1_u64 << (dim_index % 64);
        }
        dim_index += 1;

        if dim_index >= dim {
            break;
        }

        let high_nibble = (packed >> 4) as usize;
        if sign_lookup[high_nibble] != 0 {
            words[dim_index / 64] |= 1_u64 << (dim_index % 64);
        }
        dim_index += 1;
    }

    words
}

/// Hamming-based similarity used by the ADR-031 prefilter:
/// `dim - 2 * hamming(q, c)` so higher = closer.
pub(crate) fn hamming_similarity(
    query_words: &[u64],
    candidate_words: &[u64],
    dim: usize,
) -> f32 {
    let hamming_distance = query_words
        .iter()
        .zip(candidate_words.iter())
        .map(|(query, candidate)| (query ^ candidate).count_ones())
        .sum::<u32>();
    let dim_i32 = i32::try_from(dim).expect("dimensions should fit in i32");
    let distance_i32 =
        i32::try_from(hamming_distance).expect("hamming distance should fit in i32");
    (dim_i32 - (2 * distance_i32)) as f32
}

/// Sign lookup for a 4-bit, 16-entry MSE codebook. Used by
/// [`sign_words_from_packed_4bit`] to translate a packed code index
/// into a sign bit.
pub(crate) fn binary_sign_lookup_4bit(codebook: &[f32]) -> [u8; 16] {
    assert_eq!(
        codebook.len(),
        16,
        "binary sign lookup requires a 16-entry 4-bit codebook"
    );
    let mut signs = [0_u8; 16];
    for (index, value) in codebook.iter().copied().enumerate() {
        signs[index] = u8::from(value >= 0.0);
    }
    signs
}

// ---------------------------------------------------------------------------
// Persisted-sidecar helpers — moved from `src/am/common/training.rs`
// in slice 2. AM callers now reach into this module directly.
// ---------------------------------------------------------------------------

/// Number of `u64` words a per-vector sidecar occupies for this
/// (dim, bits, seed) triple. Returns 0 when RaBitQ is not supported
/// for the quantizer's lane.
pub fn persisted_sidecar_word_count(dimensions: u16, bits: u8, seed: u64) -> usize {
    let quantizer = ProdQuantizer::cached(dimensions as usize, bits, seed);
    if quantizer.binary_sign_no_qjl_4bit_supported() {
        usize::from(dimensions).div_ceil(64)
    } else {
        0
    }
}

/// Derive the sidecar sign words for a single vector from its
/// already-PQ-packed code. Returns an empty vector when RaBitQ is
/// not supported for the quantizer's lane. This is the ADR-031
/// optimization path that reuses the PQ code rather than re-rotating
/// the raw vector.
pub fn derive_persisted_sidecar_words(quantizer: &ProdQuantizer, code: &[u8]) -> Vec<u64> {
    if quantizer.binary_sign_no_qjl_4bit_supported() {
        quantizer.binary_sign_words_from_packed_no_qjl_4bit(code)
    } else {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers used only by the Quantizer trait impl above.
// ---------------------------------------------------------------------------

fn l2_norm(rotated: &[f32]) -> f32 {
    rotated.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Mean absolute value over rotated coordinates. This is `α_c` —
/// the scalar that matches `c` against its sign-vector approximation
/// `α_c · sign(c)` in a least-squares sense:
///
/// ```text
/// α_c = argmin_α ||c - α·sign(c)||² = ⟨c, sign(c)⟩ / D = mean(|c_i|).
/// ```
fn mean_abs(rotated: &[f32]) -> f32 {
    let sum: f32 = rotated.iter().map(|x| x.abs()).sum();
    sum / (rotated.len() as f32)
}

/// Asymmetric RaBitQ inner-product estimator with a Cauchy-Schwarz
/// bound. Given query `q` (rotated, full precision) and candidate
/// code `[sign_bytes | ||c|| | α_c]`, compute:
///
/// ```text
/// ⟨q, c⟩ ≈ α_c · Σ_i q_i · sign(c_i)
/// residual_c = c - α_c · sign(c),    ||residual_c||² = ||c||² − α_c² · D
/// |⟨q, c⟩ − estimate| ≤ ||q|| · ||residual_c||.
/// ```
fn estimate_ip_impl(
    query_rotated: &[f32],
    query_norm: f32,
    dimensions: usize,
    code: &[u8],
) -> DistanceEstimate {
    debug_assert_eq!(query_rotated.len(), dimensions);
    let sign_bytes = dimensions.div_ceil(8);
    assert!(
        code.len() >= sign_bytes + RABITQ_SCALAR_LEN,
        "RaBitQ code too short: got {}, expected at least {}",
        code.len(),
        sign_bytes + RABITQ_SCALAR_LEN,
    );
    let candidate_norm = f32::from_le_bytes(
        code[sign_bytes..sign_bytes + RABITQ_NORM_LEN]
            .try_into()
            .expect("norm slice is always 4 bytes"),
    );
    let candidate_alpha = f32::from_le_bytes(
        code[sign_bytes + RABITQ_NORM_LEN..sign_bytes + RABITQ_SCALAR_LEN]
            .try_into()
            .expect("alpha slice is always 4 bytes"),
    );

    let mut asymmetric_ip = 0.0_f32;
    for index in 0..dimensions {
        let byte = code[index / 8];
        let bit = (byte >> (index % 8)) & 1;
        let sign = if bit == 1 { 1.0_f32 } else { -1.0_f32 };
        asymmetric_ip += query_rotated[index] * sign;
    }
    let estimate = candidate_alpha * asymmetric_ip;

    // Residual norm of the binary approximation to `c`:
    //   ||c||² − α_c² · D. Clamp at zero for numerical safety.
    let residual_sq =
        (candidate_norm * candidate_norm - candidate_alpha * candidate_alpha * dimensions as f32)
            .max(0.0);
    let residual_norm = residual_sq.sqrt();
    let bound = query_norm * residual_norm;

    DistanceEstimate { estimate, bound }
}

fn sign_words_from_byte_slice(bytes: &[u8], dim: usize) -> Vec<u64> {
    let mut words = vec![0_u64; dim.div_ceil(64)];
    for index in 0..dim {
        let byte = bytes[index / 8];
        if (byte >> (index % 8)) & 1 == 1 {
            words[index / 64] |= 1_u64 << (index % 64);
        }
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_len_matches_dimension() {
        let prod = ProdQuantizer::cached(1536, 4, 0);
        let q = RaBitQQuantizer::with_srht(1536, prod);
        assert_eq!(q.sign_bytes(), 192);
        assert_eq!(
            <RaBitQQuantizer as crate::quant::Quantizer>::code_len(&q),
            192 + RABITQ_SCALAR_LEN
        );
    }

    #[test]
    fn estimator_recovers_self_ip_on_sign_aligned_vector() {
        // For a vector whose rotated coordinates are all ±α (sign
        // vector itself, perfectly aligned with α·sign(c)), the
        // residual is zero and the estimator is exact.
        struct Identity {
            dim: usize,
        }
        impl Rotation for Identity {
            fn dimensions(&self) -> usize {
                self.dim
            }
            fn apply(&self, v: &[f32]) -> Vec<f32> {
                v.to_vec()
            }
        }
        let dim = 128;
        let rotation: Arc<dyn Rotation> = Arc::new(Identity { dim });
        let q = RaBitQQuantizer::new(rotation);
        let alpha = 0.5_f32;
        let v: Vec<f32> = (0..dim)
            .map(|i| if i % 3 == 0 { -alpha } else { alpha })
            .collect();
        let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &v);
        let prepared = q.prepare_estimator(&v);
        let est = q.estimate_ip(&prepared, &code);
        let truth: f32 = v.iter().map(|x| x * x).sum();
        let err = (est.estimate - truth).abs();
        assert!(
            err < 1e-3,
            "sign-aligned self-IP exact, got estimate={} truth={} err={}",
            est.estimate,
            truth,
            err
        );
        assert!(
            est.bound < 1e-3,
            "bound should collapse to zero for sign-aligned vector, got {}",
            est.bound
        );
    }

    #[test]
    fn estimator_bound_dominates_error_on_random_vectors() {
        // Unbiased-estimator sanity check: the Cauchy-Schwarz bound
        // must be an upper envelope on the realized error, in
        // expectation and at the tail. Five deterministic seeds
        // provide a reproducible fixture without pulling in a full
        // rand dep on top of what the crate already imports.
        struct Identity {
            dim: usize,
        }
        impl Rotation for Identity {
            fn dimensions(&self) -> usize {
                self.dim
            }
            fn apply(&self, v: &[f32]) -> Vec<f32> {
                v.to_vec()
            }
        }
        let dim = 256;
        let rotation: Arc<dyn Rotation> = Arc::new(Identity { dim });
        let q_rabitq = RaBitQQuantizer::new(rotation);

        let seeds = [1u64, 7, 42, 128, 9001];
        for seed in seeds {
            let c = deterministic_gaussian(dim, seed);
            let query = deterministic_gaussian(dim, seed.wrapping_add(1));
            let truth: f32 = query.iter().zip(c.iter()).map(|(a, b)| a * b).sum();
            let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q_rabitq, &c);
            let prepared = q_rabitq.prepare_estimator(&query);
            let est = q_rabitq.estimate_ip(&prepared, &code);
            let err = (est.estimate - truth).abs();
            assert!(
                err <= est.bound + 1e-4,
                "bound violated for seed={}: err={} bound={} (estimate={}, truth={})",
                seed,
                err,
                est.bound,
                est.estimate,
                truth,
            );
        }
    }

    fn deterministic_gaussian(dim: usize, seed: u64) -> Vec<f32> {
        // Cheap Box-Muller over a splitmix64-seeded LCG. We only
        // need reproducibility and a finite-variance distribution;
        // not production-grade RNG quality.
        let mut state = seed.wrapping_mul(0x9E3779B97F4A7C15);
        let mut uniform = || -> f32 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u = ((state >> 11) as f64) / ((1u64 << 53) as f64);
            // Clamp away from exact 0 to keep ln() finite.
            u.max(f64::EPSILON) as f32
        };
        let mut out = Vec::with_capacity(dim);
        while out.len() < dim {
            let u1 = uniform();
            let u2 = uniform();
            let r = (-2.0_f32 * u1.ln()).sqrt();
            let theta = 2.0 * std::f32::consts::PI * u2;
            out.push(r * theta.cos());
            if out.len() < dim {
                out.push(r * theta.sin());
            }
        }
        out
    }

    #[test]
    fn encode_then_score_same_vector_is_nonnegative() {
        // Identity-rotated nonzero vector scores positive against
        // itself. This is a smoke test for slice 2's encode/score
        // round-trip, not a recall claim.
        let dim = 64;
        let prod = ProdQuantizer::cached(dim, 4, 0);
        let q = RaBitQQuantizer::with_srht(dim, prod);
        let mut v = vec![0.0_f32; dim];
        for (i, slot) in v.iter_mut().enumerate() {
            *slot = if i % 2 == 0 { 1.0 } else { -1.0 };
        }
        let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &v);
        let scorer = <RaBitQQuantizer as crate::quant::Quantizer>::prepare_scorer(&q, &v);
        let score = scorer.score(&code);
        assert!(
            score >= 0.0,
            "self-score should be nonnegative, got {}",
            score
        );
    }

    #[test]
    fn sign_words_from_rotated_matches_manual_pack() {
        let rotated = [-1.0_f32, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0];
        let words = sign_words_from_rotated(&rotated);
        // bits 1, 3, 4, 6 set → 0b01011010 = 0x5a
        assert_eq!(words, vec![0x5a]);
    }

    #[test]
    fn custom_rotation_plugs_into_seam() {
        // Identity rotation: demonstrates that Rotation impls
        // outside the crate can drop into RaBitQQuantizer.
        struct Identity {
            dim: usize,
        }
        impl Rotation for Identity {
            fn dimensions(&self) -> usize {
                self.dim
            }
            fn apply(&self, v: &[f32]) -> Vec<f32> {
                v.to_vec()
            }
        }

        let dim = 16;
        let rotation: Arc<dyn Rotation> = Arc::new(Identity { dim });
        let q = RaBitQQuantizer::new(rotation);
        let v: Vec<f32> = (0..dim).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &v);
        // First sign byte: bits 0,2,4,6 set → 0b01010101 = 0x55
        assert_eq!(code[0], 0x55);
    }

    #[test]
    fn hamming_similarity_identity_equals_dim() {
        let words = vec![0xAAAA_AAAA_AAAA_AAAA_u64; 2];
        assert_eq!(hamming_similarity(&words, &words, 128), 128.0);
    }
}
