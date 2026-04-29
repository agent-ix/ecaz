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

#[cfg(test)]
use std::cell::Cell;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use crate::quant::prod::ProdQuantizer;
use crate::quant::rotation;

/// Bytes per vector holding the rotated L2 norm `||o||`.
pub const RABITQ_NORM_LEN: usize = 4;
/// Bytes per vector holding `o_dot = ⟨o_unit, x_dec / ||x_dec||⟩` —
/// the cosine between the unit-normalized rotated vector and the
/// unit-normalized dequantized form. Reduces to
/// `⟨o_unit, sign(o)/√D⟩` at `bits_per_dim = 1`.
pub const RABITQ_UNIT_DOT_LEN: usize = 4;
/// Bytes per vector holding `||x_dec||` — the L2 norm of the
/// dequantized level vector. At `bits_per_dim = 1` this is always
/// `√D`; storing it uniformly keeps the layout bits-agnostic and
/// lets the estimator reuse one formula across `q`.
pub const RABITQ_XNORM_LEN: usize = 4;
/// Total scalar tail on each code: `||o||` + `o_dot` + `||x_dec||`.
pub const RABITQ_SCALAR_LEN: usize = RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN + RABITQ_XNORM_LEN;

/// Valid settings for `bits_per_dim`. Restricted to byte-aligned
/// values so bit packing stays a small-integer lookup; q=3/5/6/7
/// are a follow-up slice (arbitrary bit-level packing, same
/// scalar tail).
pub const RABITQ_SUPPORTED_BITS: [u8; 4] = [1, 2, 4, 8];

/// Clip radius used when scalar-quantizing at `bits_per_dim > 1`.
/// Rotated unit-vector coordinates have std ≈ `1/√D`; scaling by
/// `√D` puts them roughly in `N(0, 1)`. Clipping at 2σ covers
/// ~95% of the mass and keeps quantization levels well-utilized.
const RABITQ_QUANT_CLIP: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SeededSrhtCacheKey {
    dimensions: usize,
    seed: u64,
    bits_per_dim: u8,
}

static SEEDED_SRHT_CACHE: OnceLock<Mutex<HashMap<SeededSrhtCacheKey, Arc<RaBitQQuantizer>>>> =
    OnceLock::new();

#[cfg(test)]
thread_local! {
    static SEEDED_SRHT_CONSTRUCTION_COUNT: Cell<usize> = const { Cell::new(0) };
    static SEEDED_SRHT_CONSTRUCTION_COUNT_DIMENSIONS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_seeded_srht_construction_count_for_test(dimensions: usize) {
    SEEDED_SRHT_CONSTRUCTION_COUNT_DIMENSIONS.with(|cell| cell.set(dimensions));
    SEEDED_SRHT_CONSTRUCTION_COUNT.with(|cell| cell.set(0));
}

#[cfg(test)]
pub(crate) fn seeded_srht_construction_count_for_test() -> usize {
    SEEDED_SRHT_CONSTRUCTION_COUNT.with(Cell::get)
}

#[cfg(test)]
pub(crate) fn clear_seeded_srht_cache_for_test() {
    if let Some(cache) = SEEDED_SRHT_CACHE.get() {
        cache
            .lock()
            .expect("RaBitQ seeded SRHT cache mutex should not be poisoned")
            .clear();
    }
}

#[cfg(test)]
fn note_seeded_srht_construction_for_test(dimensions: usize) {
    SEEDED_SRHT_CONSTRUCTION_COUNT_DIMENSIONS.with(|dimension_cell| {
        if dimension_cell.get() == dimensions {
            SEEDED_SRHT_CONSTRUCTION_COUNT.with(|count_cell| {
                count_cell.set(count_cell.get() + 1);
            });
        }
    });
}

pub fn code_len_for(dimensions: usize, bits_per_dim: u8) -> Result<usize, String> {
    if !RABITQ_SUPPORTED_BITS.contains(&bits_per_dim) {
        return Err(format!(
            "RaBitQ bits_per_dim must be one of {:?}, got {}",
            RABITQ_SUPPORTED_BITS, bits_per_dim,
        ));
    }
    Ok((dimensions * bits_per_dim as usize).div_ceil(8) + RABITQ_SCALAR_LEN)
}

/// Confidence coefficient on the ε-concentration bound returned in
/// `DistanceEstimate.bound`. `2.5` ≈ 99% one-sided confidence under
/// the paper's Gaussian-tail concentration argument. Stage 3 can
/// tune this via a follow-up constructor; picking one value here
/// keeps the trait surface scalar.
pub const RABITQ_BOUND_CONFIDENCE: f32 = 2.5;

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

/// SRHT rotation. Holds its own sign vector so the rotation and
/// its seed are first-class — independent of any quantizer. A
/// `ProdQuantizer`-backed constructor remains for the ADR-031
/// PQ-derived sidecar path that needs to reach into PQ codebook
/// state through the rotation handle.
///
/// Construction choices:
/// - `SrhtRotation::with_seed(dim, seed)` — the recommended path
///   for prod code and for the Stage-1 feasibility study. The seed
///   should live per-index (recorded in the index metadata) so
///   different indexes on the same box get statistically
///   independent rotations.
/// - `SrhtRotation::new(dim, prod)` — derives the sign vector from
///   an existing `ProdQuantizer`'s `signs`. Kept because the
///   ADR-031 sidecar codepath reuses PQ codebook state co-located
///   with the rotation; call `prod()` on the resulting rotation
///   to recover the quantizer.
///
/// Tests across the crate pin the canonical seed at
/// `DEFAULT_QUANT_SEED = 42` for reproducibility; prod deployments
/// should pass a fresh seed per index build.
pub struct SrhtRotation {
    dimensions: usize,
    signs: Arc<Vec<f32>>,
    prod: Option<Arc<ProdQuantizer>>,
    seed: Option<u64>,
}

impl SrhtRotation {
    /// Construct an SRHT rotation with an explicit seed. The seed
    /// deterministically generates the sign vector via
    /// [`rotation::sign_vector`]; quality of the rotation does not
    /// depend on which specific seed is used, only on its
    /// independence from the input data.
    pub fn with_seed(dimensions: usize, seed: u64) -> Self {
        assert!(dimensions > 0, "SRHT dimensions must be positive");
        let transform_dim = rotation::effective_transform_dim(dimensions);
        let signs = Arc::new(rotation::sign_vector(transform_dim, seed));
        Self {
            dimensions,
            signs,
            prod: None,
            seed: Some(seed),
        }
    }

    /// Construct an SRHT rotation backed by a `ProdQuantizer`'s
    /// sign vector. Used when the ADR-031 PQ-derived sidecar path
    /// needs the rotation and the PQ codebook to agree. The signs
    /// are cloned out so the rotation is self-sufficient if the
    /// quantizer is later freed — but the [`prod`](Self::prod)
    /// accessor retains the quantizer reference for callers that
    /// need the codebook.
    pub fn new(dimensions: usize, prod: Arc<ProdQuantizer>) -> Self {
        assert_eq!(
            prod.original_dim, dimensions,
            "SRHT rotation dimensions mismatch: quantizer has {}, asked for {}",
            prod.original_dim, dimensions,
        );
        let signs = Arc::new(prod.signs.clone());
        Self {
            dimensions,
            signs,
            prod: Some(prod),
            seed: None,
        }
    }

    /// Access the underlying `ProdQuantizer`, when this rotation
    /// was built via [`Self::new`]. Returns `None` for
    /// seed-constructed rotations — those do not carry PQ codebook
    /// state and so cannot feed the ADR-031 sidecar encoder.
    pub fn prod(&self) -> Option<&Arc<ProdQuantizer>> {
        self.prod.as_ref()
    }

    /// Seed used to construct this rotation, when it was built via
    /// [`Self::with_seed`]. Returns `None` for quantizer-backed
    /// rotations — those inherit their signs from the
    /// `ProdQuantizer`'s own seed-derived state.
    pub fn seed(&self) -> Option<u64> {
        self.seed
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
        let padded = rotation::srht_padded(v, &self.signs);
        padded[..self.dimensions].to_vec()
    }
}

/// One RaBitQ quantizer instance. Owns the rotation via the
/// [`Rotation`] trait object seam and the per-vector encoding
/// parameters (dimensions, bits-per-dim). Build and scan paths
/// share one `Arc<RaBitQQuantizer>`.
pub struct RaBitQQuantizer {
    dimensions: usize,
    rotation: Arc<dyn Rotation>,
    bits_per_dim: u8,
}

impl RaBitQQuantizer {
    pub fn cached_seeded_srht_bits(
        dimensions: usize,
        seed: u64,
        bits: u8,
    ) -> Result<Arc<Self>, String> {
        let key = SeededSrhtCacheKey {
            dimensions,
            seed,
            bits_per_dim: bits,
        };
        let cache = SEEDED_SRHT_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let mut cache_guard = cache
            .lock()
            .map_err(|_| "RaBitQ seeded SRHT cache mutex poisoned".to_owned())?;
        if let Some(quantizer) = cache_guard.get(&key) {
            return Ok(Arc::clone(quantizer));
        }
        let quantizer = Arc::new(Self::with_seeded_srht_bits(dimensions, seed, bits)?);
        cache_guard.insert(key, Arc::clone(&quantizer));
        Ok(quantizer)
    }

    /// Construct a default 1 bit/dim quantizer (ADR-045 Stage 1
    /// canonical configuration).
    pub fn new(rotation: Arc<dyn Rotation>) -> Self {
        Self::with_bits(rotation, 1).expect("1 bit/dim is always supported")
    }

    /// Construct at a specific `bits_per_dim`. Valid values live
    /// in [`RABITQ_SUPPORTED_BITS`]; returns `Err` otherwise. At
    /// `bits = 1` the encoding is bit-identical to the paper's
    /// binary RaBitQ; at `bits ≥ 2` each coordinate gets a signed
    /// q-bit scalar-quantized level with clipping at
    /// ±[`RABITQ_QUANT_CLIP`]·σ (σ = 1/√D on the unit sphere).
    pub fn with_bits(rotation: Arc<dyn Rotation>, bits: u8) -> Result<Self, String> {
        if !RABITQ_SUPPORTED_BITS.contains(&bits) {
            return Err(format!(
                "RaBitQ bits_per_dim must be one of {:?}, got {}",
                RABITQ_SUPPORTED_BITS, bits,
            ));
        }
        let dimensions = rotation.dimensions();
        assert!(dimensions > 0, "RaBitQ dimensions must be positive");
        Ok(Self {
            dimensions,
            rotation,
            bits_per_dim: bits,
        })
    }

    /// Convenience: build a RaBitQ quantizer with the default SRHT
    /// rotation sourced from a `ProdQuantizer` and `bits = 1`.
    pub fn with_srht(dimensions: usize, prod: Arc<ProdQuantizer>) -> Self {
        let rotation: Arc<dyn Rotation> = Arc::new(SrhtRotation::new(dimensions, prod));
        Self::new(rotation)
    }

    /// Like [`Self::with_srht`] but at a specific `bits_per_dim`.
    pub fn with_srht_bits(
        dimensions: usize,
        prod: Arc<ProdQuantizer>,
        bits: u8,
    ) -> Result<Self, String> {
        let rotation: Arc<dyn Rotation> = Arc::new(SrhtRotation::new(dimensions, prod));
        Self::with_bits(rotation, bits)
    }

    /// Convenience: build a RaBitQ quantizer with a freshly-seeded
    /// SRHT rotation (no `ProdQuantizer` dependency). Recommended
    /// for prod call sites where the seed is recorded in the
    /// index metadata so different indexes get independent rotations.
    pub fn with_seeded_srht_bits(dimensions: usize, seed: u64, bits: u8) -> Result<Self, String> {
        #[cfg(test)]
        note_seeded_srht_construction_for_test(dimensions);
        let rotation: Arc<dyn Rotation> = Arc::new(SrhtRotation::with_seed(dimensions, seed));
        Self::with_bits(rotation, bits)
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    pub fn bits_per_dim(&self) -> u8 {
        self.bits_per_dim
    }

    /// Number of bytes in the packed-levels portion of a code.
    /// At `bits = 1` this is `⌈D/8⌉`; generalizes as `⌈D·bits/8⌉`.
    pub fn packed_bytes(&self) -> usize {
        code_len_for(self.dimensions, self.bits_per_dim)
            .expect("validated RaBitQ bits should have a code length")
            - RABITQ_SCALAR_LEN
    }

    /// Retained for slice-1 / slice-2 call-site compatibility —
    /// returns [`Self::packed_bytes`] since the canonical
    /// `bits = 1` shape has one sign bit per dim.
    pub fn sign_bytes(&self) -> usize {
        self.packed_bytes()
    }

    fn rotated(&self, v: &[f32]) -> Vec<f32> {
        let out = self.rotation.apply(v);
        debug_assert_eq!(out.len(), self.dimensions);
        out
    }
}

impl crate::quant::Quantizer for RaBitQQuantizer {
    /// RaBitQ encode. Rotate the input, quantize each rotated
    /// coordinate to a signed `bits_per_dim`-bit level, pack the
    /// levels LSB-first, then append three f32 scalars: `||o||`,
    /// `o_dot = ⟨o_unit, x_dec/||x_dec||⟩`, and `||x_dec||`.
    ///
    /// At `bits = 1` this collapses to the paper's binary form:
    /// levels are {-1, +1}, `||x_dec|| = √D`, `o_dot = Σ|o_i| /
    /// (||o||·√D)`. At higher bits, each coordinate holds richer
    /// information; the estimator formula is unified across bits
    /// via the stored `||x_dec||`.
    fn encode_code(&self, v: &[f32]) -> Box<[u8]> {
        let rotated = self.rotated(v);
        let packed_bytes = self.packed_bytes();
        let mut out = vec![0_u8; packed_bytes + RABITQ_SCALAR_LEN];

        let norm = l2_norm(&rotated);
        let inv_norm = if norm > 0.0 { 1.0 / norm } else { 0.0 };
        let sqrt_d = (self.dimensions as f32).sqrt();

        // Compute dequantized levels in-line with the packing loop
        // so we can also accumulate ⟨o, x_dec⟩ and ||x_dec||² as we
        // go — one pass, no intermediate Vec.
        let mut inner_o_xdec = 0.0_f32;
        let mut x_dec_norm_sq = 0.0_f32;

        let bits = self.bits_per_dim as usize;
        let levels = 1_u32 << bits;

        for (i, &o_i) in rotated.iter().enumerate() {
            let (level_idx, dequant_i) = quantize_level(o_i * inv_norm, bits, sqrt_d);
            write_level(&mut out, i, bits, level_idx);
            inner_o_xdec += o_i * dequant_i;
            x_dec_norm_sq += dequant_i * dequant_i;
        }

        let x_dec_norm = x_dec_norm_sq.sqrt();
        let denom = norm * x_dec_norm;
        let o_dot = if denom > 0.0 {
            inner_o_xdec / denom
        } else {
            0.0
        };

        let s = packed_bytes;
        out[s..s + RABITQ_NORM_LEN].copy_from_slice(&norm.to_le_bytes());
        out[s + RABITQ_NORM_LEN..s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN]
            .copy_from_slice(&o_dot.to_le_bytes());
        out[s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN..s + RABITQ_SCALAR_LEN]
            .copy_from_slice(&x_dec_norm.to_le_bytes());

        // Suppress unused warning for the `levels` binding at q=1
        // where the binary fast path inside quantize_level ignores it.
        let _ = levels;

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
            bits_per_dim: self.bits_per_dim,
        })
    }

    fn code_len(&self) -> usize {
        self.packed_bytes() + RABITQ_SCALAR_LEN
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
            bits_per_dim: self.bits_per_dim,
        }
    }

    /// Paper-faithful RaBitQ inner-product estimate between
    /// `prepared` and a code, plus an ε-concentration bound at
    /// ~99% confidence (see [`RABITQ_BOUND_CONFIDENCE`]). The
    /// formula is unified across `bits_per_dim`; the bound
    /// formula remains the q=1 Gaussian-tail form and is
    /// conservative (loose) at q>1.
    pub fn estimate_ip(&self, prepared: &PreparedEstimator, code: &[u8]) -> DistanceEstimate {
        debug_assert_eq!(prepared.dimensions, self.dimensions);
        debug_assert_eq!(prepared.bits_per_dim, self.bits_per_dim);
        prepared.estimate_ip(code)
    }

    // ---------------------------------------------------------------
    // Centered API — Symphony Stage 2 (task 27) prerequisite.
    //
    // Symphony paper §3.1.1 stores the RaBitQ code of each
    // graph-neighbor's residual *relative to the visiting vertex*,
    // not the neighbor's absolute embedding. This closes the recall
    // gap at 1 bit/dim because residuals have a much smaller dynamic
    // range than unit-sphere embeddings. The centered methods below
    // expose exactly that seam; the existing c=0 trait path
    // (`encode_code` / `prepare_scorer`) remains untouched for non-
    // Symphony consumers.
    //
    // Restricted to `bits_per_dim = 1`. Symphony exclusively uses
    // q=1; generalizing to q>1 centered is a follow-up if/when a
    // non-Symphony consumer wants it.
    // ---------------------------------------------------------------

    /// Precompute the rotation of `center` once per vertex at index-
    /// build time. The returned [`CenterContext`] is what both
    /// [`Self::encode_code_centered`] (for each of that vertex's
    /// neighbors) and [`CenteredScorer::score_at`] (at each
    /// beam-search visit) consume.
    pub fn prepare_center(&self, center: &[f32]) -> CenterContext {
        assert_eq!(
            center.len(),
            self.dimensions,
            "center length mismatch: got {}, expected {}",
            center.len(),
            self.dimensions,
        );
        let rotated = self.rotated(center);
        CenterContext {
            rotated,
            raw: center.to_vec(),
        }
    }

    /// Encode `v` as the unit-normalized rotated residual against
    /// `center`. Code layout at `bits = 1`:
    ///
    /// ```text
    /// [sign bits: ⌈D/8⌉ B][||v − c|| : 4 B][o_dot : 4 B][center_dot : 4 B]
    /// ```
    ///
    /// where `o_dot = ⟨unit_residual_rotated, x̄⟩` as in the absolute
    /// path, and `center_dot = ⟨x̄, c_rotated⟩ / √D` — the scalar
    /// that Symphony's equation (6) decomposition needs to amortize
    /// per-query / per-center work.
    pub fn encode_code_centered(&self, v: &[f32], center: &CenterContext) -> Box<[u8]> {
        assert_eq!(
            self.bits_per_dim, 1,
            "encode_code_centered is only supported at bits_per_dim = 1 (Symphony's configuration)",
        );
        assert_eq!(
            v.len(),
            self.dimensions,
            "input length mismatch: got {}, expected {}",
            v.len(),
            self.dimensions,
        );
        assert_eq!(
            center.rotated.len(),
            self.dimensions,
            "center context dimensions mismatch",
        );

        // Compute the rotated residual in one rotation (rotation is
        // linear, so r_tilde = rotate(v) − c_rotated).
        let v_rotated = self.rotated(v);
        let residual_rotated: Vec<_> = v_rotated
            .iter()
            .zip(center.rotated.iter())
            .map(|(&v_i, &center_i)| v_i - center_i)
            .collect();

        let residual_mag = l2_norm(&residual_rotated);
        let sqrt_d = (self.dimensions as f32).sqrt();

        let packed_bytes = self.packed_bytes();
        let mut out = vec![0_u8; packed_bytes + RABITQ_SCALAR_LEN];

        let mut sum_abs = 0.0_f32;
        let mut sum_center_sign = 0.0_f32;
        for (i, &r_i) in residual_rotated.iter().enumerate() {
            let level = if r_i >= 0.0 { 1_u32 } else { 0_u32 };
            write_level(&mut out, i, 1, level);
            sum_abs += r_i.abs();
            // sign as ±1 for the center-dot accumulator.
            let sign = if level == 1 { 1.0_f32 } else { -1.0_f32 };
            sum_center_sign += center.rotated[i] * sign;
        }

        let inv_denom = if residual_mag > 0.0 {
            1.0 / (residual_mag * sqrt_d)
        } else {
            0.0
        };
        let o_dot = sum_abs * inv_denom;
        // ⟨x̄, c_tilde⟩ = (1/√D) · Σ c_tilde_i · sign(r_i), since
        // x̄ has elements ±1/√D and we accumulated with ±1 weights.
        let center_dot = sum_center_sign / sqrt_d;

        let s = packed_bytes;
        out[s..s + RABITQ_NORM_LEN].copy_from_slice(&residual_mag.to_le_bytes());
        out[s + RABITQ_NORM_LEN..s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN]
            .copy_from_slice(&o_dot.to_le_bytes());
        out[s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN..s + RABITQ_SCALAR_LEN]
            .copy_from_slice(&center_dot.to_le_bytes());
        out.into_boxed_slice()
    }

    /// Prepare a query-side scorer for the centered path. The
    /// returned `CenteredScorer` can score codes encoded against
    /// any center — the center-dependent arithmetic lives in
    /// [`CenteredScorer::score_at`] and reuses one rotated-query
    /// LUT across every vertex visited.
    pub fn prepare_scorer_centered(&self, query: &[f32]) -> CenteredScorer {
        assert_eq!(
            self.bits_per_dim, 1,
            "prepare_scorer_centered requires bits_per_dim = 1",
        );
        assert_eq!(
            query.len(),
            self.dimensions,
            "query length mismatch: got {}, expected {}",
            query.len(),
            self.dimensions,
        );
        let rotated = self.rotated(query);
        CenteredScorer {
            query_rotated: rotated,
            query_raw: query.to_vec(),
            dimensions: self.dimensions,
        }
    }

    /// Read `||v − center||` from a centered code produced by
    /// [`Self::encode_code_centered`]. Exposed so the AM can
    /// combine the stored residual magnitude with the per-visit
    /// query-residual magnitude via paper eq. (2) without re-
    /// parsing the code.
    pub fn centered_residual_magnitude(&self, code: &[u8]) -> f32 {
        let s = self.packed_bytes();
        f32::from_le_bytes(
            code[s..s + RABITQ_NORM_LEN]
                .try_into()
                .expect("residual_mag slice is always 4 bytes"),
        )
    }
}

/// Per-vertex precomputed state for Symphony's centered RaBitQ
/// path. Built once per vertex at index-build time via
/// [`RaBitQQuantizer::prepare_center`]; consumed by
/// [`RaBitQQuantizer::encode_code_centered`] (to encode each of
/// that vertex's neighbors) and by
/// [`CenteredScorer::score_at`] (at every beam-search visit to
/// this vertex).
///
/// Stores both the rotated and raw center so [`CenteredScorer::score_at`]
/// can compute `||q_r − c||` at visit time without the AM
/// re-supplying the raw center vector.
pub struct CenterContext {
    rotated: Vec<f32>,
    raw: Vec<f32>,
}

impl CenterContext {
    /// The raw center vector.
    pub fn raw(&self) -> &[f32] {
        &self.raw
    }
}

/// Prepared query state for the centered path. Holds the rotated
/// and raw query; `score_at` combines these with a per-vertex
/// [`CenterContext`] via paper equation (6) to estimate the
/// unit-residual inner product.
pub struct CenteredScorer {
    query_rotated: Vec<f32>,
    query_raw: Vec<f32>,
    dimensions: usize,
}

impl CenteredScorer {
    /// Estimate `⟨(q − c)/||q − c||, (v − c)/||v − c||⟩` from a
    /// code produced by
    /// [`RaBitQQuantizer::encode_code_centered(v, c)`] and the
    /// same `center` context. Returns `DistanceEstimate` on the
    /// unit-residual inner product; the AM combines with
    /// `||q − c||` and the stored `||v − c||` (via
    /// [`RaBitQQuantizer::centered_residual_magnitude`]) per paper
    /// eq. (2) to recover L2 distance.
    pub fn score_at(&self, code: &[u8], center: &CenterContext) -> DistanceEstimate {
        assert_eq!(
            center.rotated.len(),
            self.dimensions,
            "center context dimensions mismatch",
        );
        let packed_bytes = self.dimensions.div_ceil(8);
        assert!(
            code.len() >= packed_bytes + RABITQ_SCALAR_LEN,
            "centered code too short: got {}, expected at least {}",
            code.len(),
            packed_bytes + RABITQ_SCALAR_LEN,
        );
        let s = packed_bytes;
        let residual_mag = f32::from_le_bytes(
            code[s..s + RABITQ_NORM_LEN]
                .try_into()
                .expect("residual_mag slice is always 4 bytes"),
        );
        let o_dot = f32::from_le_bytes(
            code[s + RABITQ_NORM_LEN..s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN]
                .try_into()
                .expect("o_dot slice is always 4 bytes"),
        );
        let center_dot = f32::from_le_bytes(
            code[s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN..s + RABITQ_SCALAR_LEN]
                .try_into()
                .expect("center_dot slice is always 4 bytes"),
        );

        // ⟨x̄, q_tilde⟩ = (1/√D) · Σ q_tilde_i · sign(r_i). The
        // per-neighbor hot loop in Stage 3 will replace this with
        // FastScan / signed POPCNT kernels; scalar form here is
        // the correctness reference.
        let mut sum_q_sign = 0.0_f32;
        for i in 0..self.dimensions {
            let byte = code[i / 8];
            let bit = (byte >> (i % 8)) & 1;
            let sign = if bit == 1 { 1.0_f32 } else { -1.0_f32 };
            sum_q_sign += self.query_rotated[i] * sign;
        }
        let sqrt_d = (self.dimensions as f32).sqrt();
        let query_dot_code = sum_q_sign / sqrt_d;

        // ||q_r − c||. Rotation preserves L2 so we can use either
        // raw or rotated vectors; rotated keeps everything in one
        // frame, raw saves a rotation. Use raw.
        let mut query_residual_sq = 0.0_f32;
        for i in 0..self.dimensions {
            let d = self.query_raw[i] - center.raw[i];
            query_residual_sq += d * d;
        }
        let query_residual_mag = query_residual_sq.sqrt();

        // Paper eq (6):
        //   ⟨x̄, q_tilde_unit⟩ = (query_dot_code − center_dot) / ||q_r − c||
        //
        // RaBitQ estimator (residual unit vectors):
        //   ⟨q_unit_res, o_unit_res⟩ ≈ ⟨x̄, q_tilde_unit⟩ / o_dot
        const O_DOT_FLOOR: f32 = 1e-6;
        const QR_FLOOR: f32 = 1e-6;
        if o_dot.abs() < O_DOT_FLOOR
            || !o_dot.is_finite()
            || query_residual_mag < QR_FLOOR
            || residual_mag <= 0.0
        {
            return DistanceEstimate {
                estimate: 0.0,
                bound: f32::INFINITY,
            };
        }
        let x_dot_q_unit = (query_dot_code - center_dot) / query_residual_mag;
        let estimate = x_dot_q_unit / o_dot;

        // ε-concentration bound in the unit-residual frame; both
        // residuals are unit vectors so the ||·|| factors from the
        // absolute path collapse to 1.
        let o_dot_sq = o_dot * o_dot;
        let epsilon_sq = ((1.0 - o_dot_sq).max(0.0)) / (self.dimensions as f32 * o_dot_sq);
        let bound = RABITQ_BOUND_CONFIDENCE * epsilon_sq.sqrt();

        // Silence unused warnings on state kept for AM tooling.
        let _ = residual_mag;

        DistanceEstimate { estimate, bound }
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
    bits_per_dim: u8,
}

impl PreparedEstimator {
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }
    pub fn query_norm(&self) -> f32 {
        self.query_norm
    }
    pub fn bits_per_dim(&self) -> u8 {
        self.bits_per_dim
    }
    pub fn estimate_ip(&self, code: &[u8]) -> DistanceEstimate {
        estimate_ip_impl(
            &self.query_rotated,
            self.query_norm,
            self.dimensions,
            self.bits_per_dim,
            code,
        )
    }
}

/// Prepared scorer for the `QueryScorer` trait. Holds the rotated
/// query so `score` can compute the asymmetric RaBitQ estimator. The
/// bound-carrying form lives at [`RaBitQQuantizer::estimate_ip`].
pub struct RaBitQScorer {
    query_rotated: Vec<f32>,
    query_norm: f32,
    dimensions: usize,
    bits_per_dim: u8,
}

impl crate::quant::QueryScorer for RaBitQScorer {
    fn score(&self, code: &[u8]) -> f32 {
        estimate_ip_impl(
            &self.query_rotated,
            self.query_norm,
            self.dimensions,
            self.bits_per_dim,
            code,
        )
        .estimate
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
pub(crate) fn hamming_similarity(query_words: &[u64], candidate_words: &[u64], dim: usize) -> f32 {
    let hamming_distance = query_words
        .iter()
        .zip(candidate_words.iter())
        .map(|(query, candidate)| (query ^ candidate).count_ones())
        .sum::<u32>();
    let dim_i32 = i32::try_from(dim).expect("dimensions should fit in i32");
    let distance_i32 = i32::try_from(hamming_distance).expect("hamming distance should fit in i32");
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

/// Quantize one unit-vector coordinate `o_hat_i = o_i / ||o||` to
/// a `bits`-bit signed level. Returns `(level_index, dequant_value)`:
/// the level index is what gets packed into the code; the dequant
/// value is the reconstruction used by the encoder to compute
/// `⟨o, x_dec⟩` and `||x_dec||` in-line.
///
/// At `bits = 1` the behavior is exactly the slice-9 binary form:
/// non-negative → level 1 (dequant +1), negative → level 0 (-1),
/// independent of magnitude and of `sqrt_d`.
///
/// At `bits ≥ 2`: multiply by `sqrt_d` to put the coordinate
/// distribution on `N(0, 1)` (coord std of a unit vector is
/// `1/√D`), clip to `[-C, +C]` with `C = RABITQ_QUANT_CLIP`, then
/// bin uniformly into `2^bits` cells. The level stored is the
/// unsigned bin index `0..2^bits - 1`; the dequantized value is
/// the bin center mapped back to the unit-vector scale.
fn quantize_level(o_hat_i: f32, bits: usize, sqrt_d: f32) -> (u32, f32) {
    if bits == 1 {
        if o_hat_i >= 0.0 {
            (1, 1.0)
        } else {
            (0, -1.0)
        }
    } else {
        let levels = 1_u32 << bits;
        let c = RABITQ_QUANT_CLIP;
        let scaled = o_hat_i * sqrt_d;
        let t = ((scaled + c) / (2.0 * c)).clamp(0.0, 1.0);
        let level = ((t * levels as f32) as u32).min(levels - 1);
        // Bin center, mapped back to unit-vector scale.
        let center_scaled = (level as f32 + 0.5) / levels as f32 * 2.0 * c - c;
        let dequant = center_scaled / sqrt_d;
        (level, dequant)
    }
}

/// Inverse of [`quantize_level`]: given a stored level index and
/// `bits`, return the dequantized coordinate value.
fn dequant_level(level: u32, bits: usize, sqrt_d: f32) -> f32 {
    if bits == 1 {
        if level == 1 {
            1.0
        } else {
            -1.0
        }
    } else {
        let levels = 1_u32 << bits;
        let c = RABITQ_QUANT_CLIP;
        let center_scaled = (level as f32 + 0.5) / levels as f32 * 2.0 * c - c;
        center_scaled / sqrt_d
    }
}

/// Write a `bits`-wide level index at coordinate position `i` into
/// the LSB-first packed buffer `out`. Restricted to `bits ∈ {1, 2,
/// 4, 8}` so the bit offset is always byte-aligned within each
/// coord.
fn write_level(out: &mut [u8], i: usize, bits: usize, level: u32) {
    match bits {
        1 => {
            if level == 1 {
                out[i / 8] |= 1_u8 << (i % 8);
            }
        }
        2 => {
            let byte = i / 4;
            let shift = (i % 4) * 2;
            out[byte] |= ((level as u8) & 0x03) << shift;
        }
        4 => {
            let byte = i / 2;
            let shift = (i % 2) * 4;
            out[byte] |= ((level as u8) & 0x0F) << shift;
        }
        8 => {
            out[i] = level as u8;
        }
        _ => unreachable!("unsupported bits_per_dim: {}", bits),
    }
}

/// Inverse of [`write_level`].
fn read_level(code: &[u8], i: usize, bits: usize) -> u32 {
    match bits {
        1 => ((code[i / 8] >> (i % 8)) & 1) as u32,
        2 => ((code[i / 4] >> ((i % 4) * 2)) & 0x03) as u32,
        4 => ((code[i / 2] >> ((i % 2) * 4)) & 0x0F) as u32,
        8 => code[i] as u32,
        _ => unreachable!("unsupported bits_per_dim: {}", bits),
    }
}

/// Paper-faithful RaBitQ inner-product estimator with an
/// ε-concentration error bound.
///
/// The candidate code holds a `bits_per_dim`-bit level per
/// coordinate plus three scalars: `||o||`, `o_dot = ⟨o_unit,
/// x_dec/||x_dec||⟩`, and `||x_dec||`. Given query `q` (rotated,
/// full precision, norm `||q||`):
///
/// ```text
/// α      = ||o|| · o_dot / ||x_dec||          (least-squares α of o ≈ α·x_dec)
/// ⟨q, o⟩ ≈ α · Σ_i q_i · dequant(level_i)
/// ```
///
/// At `bits = 1`, `||x_dec|| = √D` and `dequant = sign(o)`, so the
/// formula collapses to the slice-9 binary form:
/// `||o|| · Σ q_i · sign(o_i) / (o_dot · √D)`.
///
/// ε-concentration bound (paper's binary form; conservative at
/// `bits > 1`, tracked as an open question in the slice-12 packet):
///
/// ```text
/// ε²(o) ≈ (1 − o_dot²) / (D · o_dot²)
/// |⟨q, o⟩ − estimate| ≤ C · ||q|| · ||o|| · ε(o)
/// ```
fn estimate_ip_impl(
    query_rotated: &[f32],
    query_norm: f32,
    dimensions: usize,
    bits_per_dim: u8,
    code: &[u8],
) -> DistanceEstimate {
    debug_assert_eq!(query_rotated.len(), dimensions);
    let bits = bits_per_dim as usize;
    let packed_bytes = (dimensions * bits).div_ceil(8);
    assert!(
        code.len() >= packed_bytes + RABITQ_SCALAR_LEN,
        "RaBitQ code too short: got {}, expected at least {}",
        code.len(),
        packed_bytes + RABITQ_SCALAR_LEN,
    );
    let s = packed_bytes;
    let candidate_norm = f32::from_le_bytes(
        code[s..s + RABITQ_NORM_LEN]
            .try_into()
            .expect("norm slice is always 4 bytes"),
    );
    let candidate_o_dot = f32::from_le_bytes(
        code[s + RABITQ_NORM_LEN..s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN]
            .try_into()
            .expect("o_dot slice is always 4 bytes"),
    );
    let candidate_x_norm = f32::from_le_bytes(
        code[s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN..s + RABITQ_SCALAR_LEN]
            .try_into()
            .expect("x_norm slice is always 4 bytes"),
    );

    let sqrt_d = (dimensions as f32).sqrt();

    // Σ_i q_i · dequant(level_i) in the rotated basis.
    let mut sum_q_dequant = 0.0_f32;
    for (i, &query_i) in query_rotated.iter().enumerate().take(dimensions) {
        let level = read_level(code, i, bits);
        let dequant = dequant_level(level, bits, sqrt_d);
        sum_q_dequant += query_i * dequant;
    }

    // Guard degenerate cases.
    const O_DOT_FLOOR: f32 = 1e-6;
    if candidate_o_dot.abs() < O_DOT_FLOOR
        || !candidate_o_dot.is_finite()
        || candidate_x_norm <= 0.0
        || !candidate_x_norm.is_finite()
    {
        return DistanceEstimate {
            estimate: 0.0,
            bound: f32::INFINITY,
        };
    }

    // Paper's asymmetric estimator. Derivation:
    //   ⟨q_unit, o_unit⟩ ≈ ⟨q_unit, x̄⟩ / o_dot
    //     where x̄ = x_dec / ||x_dec||
    //   ⟨q, o⟩ = ||q|| · ||o|| · ⟨q_unit, o_unit⟩
    //         ≈ ||o|| · ⟨q, x̄⟩ / o_dot
    //         = ||o|| · ⟨q, x_dec⟩ / (o_dot · ||x_dec||)
    // The α form (estimate = α · ⟨q, x_dec⟩ with α = ||o||·o_dot/||x_dec||)
    // is the least-squares fit of o ≈ α·x_dec; it preserves ranking but
    // under-scales by 1/o_dot² and breaks absolute error.
    let estimate = candidate_norm * sum_q_dequant / (candidate_o_dot * candidate_x_norm);

    let o_dot_sq = candidate_o_dot * candidate_o_dot;
    let epsilon_sq = ((1.0 - o_dot_sq).max(0.0)) / (dimensions as f32 * o_dot_sq);
    let bound = RABITQ_BOUND_CONFIDENCE * query_norm * candidate_norm * epsilon_sq.sqrt();

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
    fn qbit_encoder_reduces_error_vs_binary() {
        // At higher bits-per-dim, the estimator error on random
        // Gaussian candidates should be strictly smaller (on average)
        // than at bits = 1. Five seeds, compare mean |err|.
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
        let seeds = [1u64, 7, 42, 128, 9001];

        let rotation_bin: Arc<dyn Rotation> = Arc::new(Identity { dim });
        let q_bin = RaBitQQuantizer::with_bits(rotation_bin, 1).unwrap();
        let rotation_q4: Arc<dyn Rotation> = Arc::new(Identity { dim });
        let q_q4 = RaBitQQuantizer::with_bits(rotation_q4, 4).unwrap();

        let mut err_bin = 0.0_f32;
        let mut err_q4 = 0.0_f32;
        for seed in seeds {
            let c = deterministic_gaussian(dim, seed);
            let query = deterministic_gaussian(dim, seed.wrapping_add(1));
            let truth: f32 = query.iter().zip(c.iter()).map(|(a, b)| a * b).sum();

            let code_bin = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q_bin, &c);
            let prep_bin = q_bin.prepare_estimator(&query);
            err_bin += (q_bin.estimate_ip(&prep_bin, &code_bin).estimate - truth).abs();

            let code_q4 = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q_q4, &c);
            let prep_q4 = q_q4.prepare_estimator(&query);
            err_q4 += (q_q4.estimate_ip(&prep_q4, &code_q4).estimate - truth).abs();
        }
        assert!(
            err_q4 < err_bin,
            "q=4 should reduce error vs q=1, got err_bin={} err_q4={}",
            err_bin,
            err_q4,
        );
    }

    #[test]
    fn qbit_code_len_scales_with_bits() {
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
        for bits in [1u8, 2, 4, 8] {
            let rotation: Arc<dyn Rotation> = Arc::new(Identity { dim: 1536 });
            let q = RaBitQQuantizer::with_bits(rotation, bits).unwrap();
            let expected_packed = (1536 * bits as usize).div_ceil(8);
            assert_eq!(q.packed_bytes(), expected_packed, "bits={}", bits);
            assert_eq!(
                <RaBitQQuantizer as crate::quant::Quantizer>::code_len(&q),
                expected_packed + RABITQ_SCALAR_LEN,
                "bits={}",
                bits,
            );
        }
    }

    #[test]
    fn srht_seeded_rotation_is_deterministic_and_independent_of_prod() {
        // Same seed → same rotation output, regardless of whether
        // we have a ProdQuantizer lying around. Different seeds →
        // different signs (so different rotations).
        let dim = 64;
        let v: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.01 - 0.3).collect();

        let r1 = SrhtRotation::with_seed(dim, 7);
        let r2 = SrhtRotation::with_seed(dim, 7);
        assert_eq!(r1.apply(&v), r2.apply(&v));
        assert_eq!(r1.seed(), Some(7));
        assert!(r1.prod().is_none());

        let r3 = SrhtRotation::with_seed(dim, 8);
        assert_ne!(r1.apply(&v), r3.apply(&v));

        // Prod-backed rotation still constructs with the same API
        // shape as before.
        let prod = ProdQuantizer::cached(dim, 4, 42);
        let r4 = SrhtRotation::new(dim, prod);
        assert!(r4.prod().is_some());
        assert_eq!(r4.seed(), None);
    }

    #[test]
    fn centered_estimator_is_exact_on_sign_aligned_residual() {
        // Sign-aligned residual (all ±α) has o_dot = 1, so the
        // estimator collapses to exact on the unit-residual IP.
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
        let dim = 64;
        let rotation: Arc<dyn Rotation> = Arc::new(Identity { dim });
        let q = RaBitQQuantizer::new(rotation);

        let center = vec![0.5_f32; dim];
        // Residual `v - center` has coords ±1 (sign-aligned) for
        // `v[i] = center[i] ± 1`.
        let mut v = center.clone();
        for (i, x) in v.iter_mut().enumerate() {
            *x += if i % 3 == 0 { -1.0 } else { 1.0 };
        }
        let ctx = q.prepare_center(&center);
        let code = q.encode_code_centered(&v, &ctx);

        // Query: same as v → unit IP of q_residual with v_residual = 1.
        let scorer = q.prepare_scorer_centered(&v);
        let est = scorer.score_at(&code, &ctx);
        assert!(
            (est.estimate - 1.0).abs() < 1e-4,
            "self-centered unit IP should be 1.0, got {}",
            est.estimate
        );
        assert!(
            est.bound < 1e-4,
            "bound collapses on sign-aligned residual, got {}",
            est.bound
        );

        // Residual magnitude stored = √dim (each coord is ±1).
        let expected_mag = (dim as f32).sqrt();
        let stored_mag = q.centered_residual_magnitude(&code);
        assert!((stored_mag - expected_mag).abs() < 1e-3);
    }

    #[test]
    fn centered_estimator_bound_dominates_error_on_random_vectors() {
        // Same seeds as the absolute-path test; confirm the
        // ε-bound (now on unit-residual IP) envelopes realized
        // error.
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
        let q = RaBitQQuantizer::new(rotation);

        let seeds = [1u64, 7, 42, 128, 9001];
        for seed in seeds {
            let center = deterministic_gaussian(dim, seed.wrapping_mul(17));
            let v = deterministic_gaussian(dim, seed);
            let query = deterministic_gaussian(dim, seed.wrapping_add(1));

            let ctx = q.prepare_center(&center);
            let code = q.encode_code_centered(&v, &ctx);
            let scorer = q.prepare_scorer_centered(&query);
            let est = scorer.score_at(&code, &ctx);

            // Truth: unit-residual IP.
            let v_res: Vec<f32> = v.iter().zip(&center).map(|(a, b)| a - b).collect();
            let q_res: Vec<f32> = query.iter().zip(&center).map(|(a, b)| a - b).collect();
            let v_mag: f32 = v_res.iter().map(|x| x * x).sum::<f32>().sqrt();
            let q_mag: f32 = q_res.iter().map(|x| x * x).sum::<f32>().sqrt();
            let raw_ip: f32 = v_res.iter().zip(&q_res).map(|(a, b)| a * b).sum();
            let truth = raw_ip / (v_mag * q_mag);

            let err = (est.estimate - truth).abs();
            assert!(
                err <= est.bound + 1e-3,
                "centered bound violated for seed={}: err={} bound={}",
                seed,
                err,
                est.bound
            );
        }
    }

    #[test]
    fn centered_api_rejects_qbit_bits() {
        // Symphony's centered path is q=1 only; q=2/4/8 paths are
        // explicitly rejected to prevent silent misuse.
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
        let q = RaBitQQuantizer::with_bits(rotation, 4).unwrap();
        let center = vec![0.0_f32; dim];
        let ctx = q.prepare_center(&center);
        let v = vec![0.1_f32; dim];
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            q.encode_code_centered(&v, &ctx);
        }));
        assert!(panicked.is_err(), "q>1 centered encode should panic");
    }

    #[test]
    fn qbit_rejects_unsupported_bits() {
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
        let rotation: Arc<dyn Rotation> = Arc::new(Identity { dim: 16 });
        assert!(RaBitQQuantizer::with_bits(rotation, 3).is_err());
    }

    #[test]
    fn estimator_bound_dominates_error_on_random_vectors() {
        // The ε-concentration bound is probabilistic at ~99%
        // confidence; over five deterministic Gaussian seeds the
        // realized error must stay within `bound + numerical
        // slack` on every one. Test flakes would indicate either
        // a bug or a too-aggressive RABITQ_BOUND_CONFIDENCE.
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
        let v: Vec<f32> = (0..dim)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
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
