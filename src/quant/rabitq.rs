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

/// Default clip radius used when scalar-quantizing at `bits_per_dim > 1`.
/// Rotated unit-vector coordinates have std ≈ `1/√D`; scaling by
/// `√D` puts them roughly in `N(0, 1)`. Clipping at 2σ covers
/// ~95% of the mass and keeps quantization levels well-utilized.
const RABITQ_DEFAULT_QUANT_CLIP: f32 = 2.0;

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
    quant_clip: f32,
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
    /// ±2σ by default (σ = 1/√D on the unit sphere).
    pub fn with_bits(rotation: Arc<dyn Rotation>, bits: u8) -> Result<Self, String> {
        Self::with_bits_clip(rotation, bits, RABITQ_DEFAULT_QUANT_CLIP)
    }

    /// Like [`Self::with_bits`] but with an explicit scalar-quantization clip
    /// radius for `bits_per_dim > 1`. This is intended for measurement and
    /// tuning; the default constructor preserves the existing `2.0σ` profile.
    pub fn with_bits_clip(
        rotation: Arc<dyn Rotation>,
        bits: u8,
        quant_clip: f32,
    ) -> Result<Self, String> {
        if !RABITQ_SUPPORTED_BITS.contains(&bits) {
            return Err(format!(
                "RaBitQ bits_per_dim must be one of {:?}, got {}",
                RABITQ_SUPPORTED_BITS, bits,
            ));
        }
        if quant_clip <= 0.0 || !quant_clip.is_finite() {
            return Err(format!(
                "RaBitQ quant_clip must be positive and finite, got {quant_clip}",
            ));
        }
        let dimensions = rotation.dimensions();
        assert!(dimensions > 0, "RaBitQ dimensions must be positive");
        Ok(Self {
            dimensions,
            rotation,
            bits_per_dim: bits,
            quant_clip,
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

    /// Like [`Self::with_srht_bits`] but with an explicit scalar clip radius.
    pub fn with_srht_bits_clip(
        dimensions: usize,
        prod: Arc<ProdQuantizer>,
        bits: u8,
        quant_clip: f32,
    ) -> Result<Self, String> {
        let rotation: Arc<dyn Rotation> = Arc::new(SrhtRotation::new(dimensions, prod));
        Self::with_bits_clip(rotation, bits, quant_clip)
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

    pub fn prepare_ip_query(&self, query: &[f32]) -> RaBitQScorer {
        let rotated = self.rotated(query);
        let norm = l2_norm(&rotated);
        let dequant_lut =
            build_dequant_lut(self.dimensions, self.bits_per_dim as usize, self.quant_clip);
        let bits1_byte_lut = build_bits1_byte_lut_boxed(&dequant_lut, self.bits_per_dim);
        let (query_bf16, dequant_lut_bf16) = prepare_bf16_state(&rotated, &dequant_lut);
        RaBitQScorer {
            query_rotated: rotated,
            query_norm: norm,
            dimensions: self.dimensions,
            bits_per_dim: self.bits_per_dim,
            dequant_lut,
            bits1_byte_lut,
            query_bf16,
            dequant_lut_bf16,
        }
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

        for (i, &o_i) in rotated.iter().enumerate() {
            let (level_idx, dequant_i) =
                quantize_level(o_i * inv_norm, bits, sqrt_d, self.quant_clip);
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

        out.into_boxed_slice()
    }

    fn prepare_scorer(
        &self,
        query: &[f32],
    ) -> Box<dyn crate::quant::QueryScorer + Send + Sync + '_> {
        Box::new(self.prepare_ip_query(query))
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
        let dequant_lut =
            build_dequant_lut(self.dimensions, self.bits_per_dim as usize, self.quant_clip);
        let bits1_byte_lut = build_bits1_byte_lut_boxed(&dequant_lut, self.bits_per_dim);
        let (query_bf16, dequant_lut_bf16) = prepare_bf16_state(&rotated, &dequant_lut);
        PreparedEstimator {
            query_rotated: rotated,
            query_norm: norm,
            dimensions: self.dimensions,
            bits_per_dim: self.bits_per_dim,
            dequant_lut,
            bits1_byte_lut,
            query_bf16,
            dequant_lut_bf16,
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
            if level == 1 {
                sum_center_sign += center.rotated[i];
            } else {
                sum_center_sign -= center.rotated[i];
            }
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
            if bit == 1 {
                sum_q_sign += self.query_rotated[i];
            } else {
                sum_q_sign -= self.query_rotated[i];
            }
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
    /// Per-quantizer dequant LUT. At `bits_per_dim ∈ {1, 2, 4, 8}`
    /// the table has `1 << bits_per_dim` entries; the remainder is
    /// unused. Precomputed once per prepared query so the NEON inner
    /// loop reuses it for every candidate rather than recomputing
    /// per candidate.
    dequant_lut: [f32; 256],
    /// 256-row × 8-lane f32 byte-LUT for the bits=1 NEON kernel.
    /// Each row carries the 8 dequant lanes for a code byte; the
    /// kernel reads `byte_lut[code_byte]` with two `vld1q_f32`s and
    /// FMAs against query lanes — no per-byte scalar unpack.
    bits1_byte_lut: Option<Box<[[f32; 8]; 256]>>,
    /// Same `query_rotated` data truncated to bfloat16 (stored as raw
    /// u16 bits — the upper half of the f32). Used by the aarch64 +
    /// `bf16` feature path which executes `bfdot` to FMA 8 bf16 lanes
    /// at a time vs 4 f32 lanes. Empty when bf16 is not detected at
    /// runtime so the f32 path doesn't pay the build cost.
    query_bf16: Vec<u16>,
    /// bf16 dequant LUT matching `dequant_lut`. Same truncation.
    dequant_lut_bf16: [u16; 256],
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
            Some(&self.query_bf16),
            Some(&self.dequant_lut_bf16),
            self.query_norm,
            self.dimensions,
            self.bits_per_dim,
            &self.dequant_lut,
            self.bits1_byte_lut.as_deref(),
            code,
        )
    }

    /// IVF-fast estimate that skips the ε-concentration bound (sqrt
    /// + a few multiplies per candidate) since the IVF scan path
    /// only uses the scalar estimate. Use [`Self::estimate_ip`] if
    /// you need the bound.
    #[inline]
    pub fn estimate_ip_scalar_only(&self, code: &[u8]) -> f32 {
        estimate_ip_scalar_only_impl(
            &self.query_rotated,
            Some(&self.query_bf16),
            Some(&self.dequant_lut_bf16),
            self.dimensions,
            self.bits_per_dim,
            &self.dequant_lut,
            self.bits1_byte_lut.as_deref(),
            code,
        )
    }

    /// Least-squares scaled inner-product estimate:
    /// `||o|| * o_dot * <q, x_dec> / ||x_dec||`.
    ///
    /// This is not the paper's unbiased RaBitQ estimator used by the AM scan
    /// path. It is exposed for measurement harnesses that want to evaluate
    /// whether the lower-variance least-squares projection ranks a narrow
    /// rerank candidate frontier better than the unbiased `1 / o_dot` scale.
    #[inline]
    pub fn estimate_ip_least_squares_scalar_only(&self, code: &[u8]) -> f32 {
        estimate_ip_least_squares_scalar_only_impl(
            &self.query_rotated,
            Some(&self.query_bf16),
            Some(&self.dequant_lut_bf16),
            self.dimensions,
            self.bits_per_dim,
            &self.dequant_lut,
            self.bits1_byte_lut.as_deref(),
            code,
        )
    }

    /// Batch IVF-fast estimates for contiguous bits=1 RaBitQ codes.
    ///
    /// This is the production entrypoint for scratch-SoA callers that already
    /// hold a dense payload slab. Each code still uses the target-specific
    /// bits=1 kernel selected by `sum_query_dequant_with_bf16`; the batch API
    /// removes the per-posting quantizer dispatch and lets callers reuse one
    /// output buffer across scan chunks.
    pub fn estimate_ip_bits1_batch(
        &self,
        codes: &[u8],
        code_len: usize,
        out_scores: &mut Vec<f32>,
    ) -> Result<(), String> {
        if self.bits_per_dim != 1 {
            return Err(format!(
                "RaBitQ bits=1 batch scorer requires bits=1, got {}",
                self.bits_per_dim
            ));
        }
        let expected_code_len = code_len_for(self.dimensions, 1)
            .expect("bits=1 RaBitQ code length should be valid for prepared dimensions");
        if code_len != expected_code_len {
            return Err(format!(
                "RaBitQ bits=1 batch scorer code length mismatch: got {code_len}, expected {expected_code_len}"
            ));
        }
        if code_len == 0 || codes.len() % code_len != 0 {
            return Err(format!(
                "RaBitQ bits=1 batch scorer payload slab length {} is not divisible by code length {code_len}",
                codes.len()
            ));
        }

        out_scores.clear();
        out_scores.reserve(codes.len() / code_len);
        for code in codes.chunks_exact(code_len) {
            out_scores.push(estimate_ip_scalar_only_impl(
                &self.query_rotated,
                Some(&self.query_bf16),
                Some(&self.dequant_lut_bf16),
                self.dimensions,
                self.bits_per_dim,
                &self.dequant_lut,
                self.bits1_byte_lut.as_deref(),
                code,
            ));
        }
        Ok(())
    }

    /// Cheap pre-prune via Cauchy-Schwarz. Reads only the per-code
    /// scalar metadata (`||o||`, `o_dot`) and skips the full
    /// `dimensions`-wide SIMD inner product when even the most
    /// optimistic estimate falls below `min_ip_to_keep`.
    ///
    /// Correctness: the asymmetric RaBitQ estimator is
    /// `α · ⟨q, x_dec⟩` with `α = ||o|| · o_dot / ||x_dec||`. By
    /// Cauchy-Schwarz, `|⟨q, x_dec⟩| ≤ ||q|| · ||x_dec||`, so the
    /// estimate is bounded by `||o|| · ||q|| / o_dot` (positive side).
    /// Skipping when that upper bound is below the running top-K cutoff
    /// is recall-safe: this candidate can never reach top-K.
    ///
    /// IVF-fast variant: returns just the scalar estimate (no ε-bound).
    /// Folds the pre-prune scalar read with the post-prune kernel call
    /// so we read `candidate_norm`, `candidate_o_dot`, `candidate_x_norm`
    /// once instead of twice on the keep path.
    pub fn try_estimate_ip_scalar(&self, code: &[u8], min_ip_to_keep: f32) -> Option<f32> {
        let bits = self.bits_per_dim as usize;
        let packed_bytes = (self.dimensions * bits).div_ceil(8);
        if code.len() < packed_bytes + RABITQ_SCALAR_LEN {
            return Some(self.estimate_ip_scalar_only(code));
        }
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
        const O_DOT_FLOOR: f32 = 1e-6;
        if candidate_norm > 0.0
            && candidate_norm.is_finite()
            && candidate_o_dot.abs() >= O_DOT_FLOOR
            && candidate_o_dot.is_finite()
        {
            let max_estimate = candidate_norm * self.query_norm / candidate_o_dot.abs();
            if max_estimate < min_ip_to_keep {
                return None;
            }
        }
        // Inline the keep path so we don't re-read scalars.
        if !candidate_o_dot.is_finite()
            || candidate_o_dot.abs() < O_DOT_FLOOR
            || candidate_x_norm <= 0.0
            || !candidate_x_norm.is_finite()
        {
            return Some(0.0);
        }
        let bits = self.bits_per_dim as usize;
        let sum_q_dequant = sum_query_dequant_with_bf16(
            &self.query_rotated,
            Some(&self.query_bf16),
            Some(&self.dequant_lut_bf16),
            self.dimensions,
            bits,
            &self.dequant_lut,
            self.bits1_byte_lut.as_deref(),
            code,
        );
        Some(candidate_norm * sum_q_dequant / (candidate_o_dot * candidate_x_norm))
    }

    /// Bound-carrying variant retained for AM tooling that wants the
    /// ε-envelope. IVF callers should prefer `try_estimate_ip_scalar`.
    pub fn try_estimate_ip(&self, code: &[u8], min_ip_to_keep: f32) -> Option<DistanceEstimate> {
        let bits = self.bits_per_dim as usize;
        let packed_bytes = (self.dimensions * bits).div_ceil(8);
        if code.len() < packed_bytes + RABITQ_SCALAR_LEN {
            return Some(self.estimate_ip(code));
        }
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
        const O_DOT_FLOOR: f32 = 1e-6;
        if candidate_norm > 0.0
            && candidate_norm.is_finite()
            && candidate_o_dot.abs() >= O_DOT_FLOOR
            && candidate_o_dot.is_finite()
        {
            let max_estimate = candidate_norm * self.query_norm / candidate_o_dot.abs();
            if max_estimate < min_ip_to_keep {
                return None;
            }
        }
        Some(self.estimate_ip(code))
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
    dequant_lut: [f32; 256],
    bits1_byte_lut: Option<Box<[[f32; 8]; 256]>>,
    query_bf16: Vec<u16>,
    dequant_lut_bf16: [u16; 256],
}

impl crate::quant::QueryScorer for RaBitQScorer {
    fn score(&self, code: &[u8]) -> f32 {
        estimate_ip_scalar_only_impl(
            &self.query_rotated,
            Some(&self.query_bf16),
            Some(&self.dequant_lut_bf16),
            self.dimensions,
            self.bits_per_dim,
            &self.dequant_lut,
            self.bits1_byte_lut.as_deref(),
            code,
        )
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
/// `1/√D`), clip to `[-C, +C]` with caller-provided `quant_clip`, then
/// bin uniformly into `2^bits` cells. The level stored is the
/// unsigned bin index `0..2^bits - 1`; the dequantized value is
/// the bin center mapped back to the unit-vector scale.
fn quantize_level(o_hat_i: f32, bits: usize, sqrt_d: f32, quant_clip: f32) -> (u32, f32) {
    if bits == 1 {
        if o_hat_i >= 0.0 {
            (1, 1.0)
        } else {
            (0, -1.0)
        }
    } else {
        let levels = 1_u32 << bits;
        let c = quant_clip;
        let range = 2.0 * c;
        let scaled = o_hat_i * sqrt_d;
        let t = ((scaled + c) / range).clamp(0.0, 1.0);
        let level = ((t * levels as f32) as u32).min(levels - 1);
        // Bin center, mapped back to unit-vector scale.
        let center_scaled = (level as f32 + 0.5) / levels as f32 * range - c;
        let dequant = center_scaled / sqrt_d;
        (level, dequant)
    }
}

/// Inverse of [`quantize_level`]: given a stored level index and
/// `bits`, return the dequantized coordinate value.
fn dequant_level(level: u32, bits: usize, sqrt_d: f32, quant_clip: f32) -> f32 {
    if bits == 1 {
        if level == 1 {
            1.0
        } else {
            -1.0
        }
    } else {
        let levels = 1_u32 << bits;
        let c = quant_clip;
        let center_scaled = (level as f32 + 0.5) / levels as f32 * (2.0 * c) - c;
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

/// Truncate an f32 to bfloat16 by taking the upper 16 bits of its IEEE
/// 754 representation. Round-to-nearest-even would be ~1 ULP better
/// but adds branches; truncation is what the BF16 cast instruction
/// effectively does and is good enough for inner-product accumulation
/// with a wide accumulator.
#[inline]
fn f32_to_bf16_bits(value: f32) -> u16 {
    (value.to_bits() >> 16) as u16
}

/// Build the bf16 mirror of `query_rotated` + `dequant_lut`. Only
/// allocated when the `rabitq-bf16` feature is on (otherwise the
/// returned Vec is empty and the bf16 dispatch is dead code). Avoids
/// the per-query truncation pass + Vec allocation when the kernel
/// will never use it.
fn prepare_bf16_state(query_rotated: &[f32], dequant_lut: &[f32; 256]) -> (Vec<u16>, [u16; 256]) {
    #[cfg(feature = "rabitq-bf16")]
    {
        let query_bf16: Vec<u16> = query_rotated
            .iter()
            .copied()
            .map(f32_to_bf16_bits)
            .collect();
        let mut lut_bf16 = [0_u16; 256];
        for (i, &v) in dequant_lut.iter().enumerate() {
            lut_bf16[i] = f32_to_bf16_bits(v);
        }
        (query_bf16, lut_bf16)
    }
    #[cfg(not(feature = "rabitq-bf16"))]
    {
        let _ = (query_rotated, dequant_lut);
        (Vec::new(), [0_u16; 256])
    }
}

/// Build the bits=1 byte-LUT once per prepared query/scorer. Other
/// code widths never reach the bits=1 kernel, so they keep this
/// state absent and avoid the per-query 8 KB allocation/memset.
fn build_bits1_byte_lut_boxed(lut: &[f32; 256], bits_per_dim: u8) -> Option<Box<[[f32; 8]; 256]>> {
    if bits_per_dim != 1 {
        return None;
    }
    let mut out: Box<[[f32; 8]; 256]> = Box::new([[0.0_f32; 8]; 256]);
    let neg = lut[0];
    let pos = lut[1];
    for (b, row) in out.iter_mut().enumerate() {
        for (i, lane) in row.iter_mut().enumerate() {
            *lane = if ((b >> i) & 1) == 1 { pos } else { neg };
        }
    }
    Some(out)
}

/// Build a 256-entry dequant table indexed by `level`. The first
/// `1 << bits` entries are valid; the rest are zero-filled. The LUT
/// depends only on `(bits, sqrt_d)`, so it can be computed once per
/// prepared query and reused across every candidate code.
fn build_dequant_lut(dimensions: usize, bits: usize, quant_clip: f32) -> [f32; 256] {
    let mut lut = [0.0_f32; 256];
    let sqrt_d = (dimensions as f32).sqrt();
    let entries = 1_usize << bits;
    let mut level = 0_u32;
    while (level as usize) < entries {
        lut[level as usize] = dequant_level(level, bits, sqrt_d, quant_clip);
        level += 1;
    }
    lut
}

/// Σ_i q_i · dequant(level_i) — the inner sum that dominates
/// `estimate_ip_impl` runtime. `lut` is the precomputed dequant table
/// (`build_dequant_lut` filled the first `1 << bits` entries; the rest
/// are zero). Scalar fallback always available; aarch64 NEON path
/// lifts the `bits == 4` hot case (production `storage_format =
/// 'rabitq'` uses 4-bit codes per `DEFAULT_QUANT_BITS`).
#[inline]
fn sum_query_dequant(
    query_rotated: &[f32],
    dimensions: usize,
    bits: usize,
    lut: &[f32; 256],
    code: &[u8],
) -> f32 {
    sum_query_dequant_with_bf16(query_rotated, None, None, dimensions, bits, lut, None, code)
}

/// Variant that also accepts the bf16 mirror state. When the caller
/// already has the bf16 query + LUT computed (per-query, one-time)
/// and the runtime supports the `bf16` aarch64 extension, this path
/// dispatches to a bfdot-based kernel that processes 8 lanes per
/// vbfdotq_f32 call vs 4 lanes per vfmaq_f32.
#[inline]
fn sum_query_dequant_with_bf16(
    query_rotated: &[f32],
    query_bf16: Option<&[u16]>,
    dequant_lut_bf16: Option<&[u16; 256]>,
    dimensions: usize,
    bits: usize,
    lut: &[f32; 256],
    bits1_byte_lut: Option<&[[f32; 8]; 256]>,
    code: &[u8],
) -> f32 {
    let _ = (query_bf16, dequant_lut_bf16, bits1_byte_lut);
    #[cfg(target_arch = "aarch64")]
    {
        if bits == 1 && std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: NEON feature confirmed at runtime; impl owns lane safety.
            if let Some(bits1_byte_lut) = bits1_byte_lut {
                return unsafe {
                    sum_query_dequant_neon_bits1(
                        query_rotated,
                        dimensions,
                        bits1_byte_lut,
                        lut,
                        code,
                    )
                };
            }
        }
        if bits == 4 {
            // bf16 bfdot kernel is gated behind the `rabitq-bf16` Cargo
            // feature. On Neoverse-V2 it measured neutral-to-slightly-
            // slower vs f32 NEON (same 128-bit VL, no drift test gate
            // yet). Kept compilable so future hosts with wider SVE2
            // bf16 issue can flip the feature and benchmark.
            #[cfg(feature = "rabitq-bf16")]
            {
                if let (Some(q_bf16), Some(lut_bf16)) = (query_bf16, dequant_lut_bf16) {
                    if q_bf16.len() == dimensions
                        && std::arch::is_aarch64_feature_detected!("bf16")
                        && std::arch::is_aarch64_feature_detected!("neon")
                    {
                        // SAFETY: features confirmed at runtime.
                        return unsafe {
                            sum_query_dequant_neon_bf16_bits4(q_bf16, dimensions, lut_bf16, code)
                        };
                    }
                }
            }
            if std::arch::is_aarch64_feature_detected!("neon") {
                // SAFETY: NEON feature confirmed at runtime.
                return unsafe {
                    sum_query_dequant_neon_bits4(query_rotated, dimensions, lut, code)
                };
            }
        }
    }
    if bits == 1 {
        if let Some(bits1_byte_lut) = bits1_byte_lut {
            return sum_query_dequant_bits1_byte_lut_scalar(
                query_rotated,
                dimensions,
                bits1_byte_lut,
                code,
            );
        }
    }
    sum_query_dequant_scalar(query_rotated, dimensions, bits, lut, code)
}

#[inline]
fn sum_query_dequant_scalar(
    query_rotated: &[f32],
    dimensions: usize,
    bits: usize,
    lut: &[f32; 256],
    code: &[u8],
) -> f32 {
    let mut sum = 0.0_f32;
    for (i, &query_i) in query_rotated.iter().enumerate().take(dimensions) {
        let level = read_level(code, i, bits) as usize;
        sum += query_i * lut[level];
    }
    sum
}

#[inline]
fn sum_query_dequant_bits1_byte_lut_scalar(
    query_rotated: &[f32],
    dimensions: usize,
    byte_lut: &[[f32; 8]; 256],
    code: &[u8],
) -> f32 {
    let mut sum = 0.0_f32;
    let mut dim_index = 0_usize;

    while dim_index + 8 <= dimensions {
        let row = &byte_lut[code[dim_index / 8] as usize];
        sum += query_rotated[dim_index] * row[0]
            + query_rotated[dim_index + 1] * row[1]
            + query_rotated[dim_index + 2] * row[2]
            + query_rotated[dim_index + 3] * row[3]
            + query_rotated[dim_index + 4] * row[4]
            + query_rotated[dim_index + 5] * row[5]
            + query_rotated[dim_index + 6] * row[6]
            + query_rotated[dim_index + 7] * row[7];
        dim_index += 8;
    }

    while dim_index < dimensions {
        let bit = (code[dim_index / 8] >> (dim_index % 8)) & 1;
        sum += query_rotated[dim_index] * byte_lut[usize::from(bit)][0];
        dim_index += 1;
    }

    sum
}

/// bfdot accumulator using inline asm. Equivalent to the nightly
/// `vbfdotq_f32(acc, a, b)` intrinsic. Operands:
///   - `acc`: f32x4 accumulator
///   - `a`, `b`: each carries 8 bf16 lanes packed as u16x8
/// Result: `acc[i] += a[2i]*b[2i] + a[2i+1]*b[2i+1]` for i ∈ 0..4
///         (all multiplies happen in bf16, accumulation in f32).
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon,bf16")]
unsafe fn bfdot_asm(
    acc: std::arch::aarch64::float32x4_t,
    a: std::arch::aarch64::uint16x8_t,
    b: std::arch::aarch64::uint16x8_t,
) -> std::arch::aarch64::float32x4_t {
    let mut out = acc;
    core::arch::asm!(
        "bfdot {acc:v}.4s, {a:v}.8h, {b:v}.8h",
        acc = inout(vreg) out,
        a = in(vreg) a,
        b = in(vreg) b,
        options(pure, nomem, nostack),
    );
    out
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon,bf16")]
unsafe fn sum_query_dequant_neon_bf16_bits4(
    query_bf16: &[u16],
    dimensions: usize,
    lut_bf16: &[u16; 256],
    code: &[u8],
) -> f32 {
    use std::arch::aarch64::{vaddq_f32, vdupq_n_f32, vld1q_u16, vst1q_f32};

    // 4 accumulator chains to fill V2's 4 SIMD pipes.
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);

    let mut dim_index = 0_usize;
    // Each iteration consumes 32 dimensions = 16 bytes of code,
    // dispatching 4 independent bfdot calls.
    while dim_index + 32 <= dimensions {
        let byte_base = dim_index / 2;
        let b: [usize; 16] = [
            *code.get_unchecked(byte_base) as usize,
            *code.get_unchecked(byte_base + 1) as usize,
            *code.get_unchecked(byte_base + 2) as usize,
            *code.get_unchecked(byte_base + 3) as usize,
            *code.get_unchecked(byte_base + 4) as usize,
            *code.get_unchecked(byte_base + 5) as usize,
            *code.get_unchecked(byte_base + 6) as usize,
            *code.get_unchecked(byte_base + 7) as usize,
            *code.get_unchecked(byte_base + 8) as usize,
            *code.get_unchecked(byte_base + 9) as usize,
            *code.get_unchecked(byte_base + 10) as usize,
            *code.get_unchecked(byte_base + 11) as usize,
            *code.get_unchecked(byte_base + 12) as usize,
            *code.get_unchecked(byte_base + 13) as usize,
            *code.get_unchecked(byte_base + 14) as usize,
            *code.get_unchecked(byte_base + 15) as usize,
        ];

        let mut dq = [0_u16; 32];
        for (idx, bi) in b.iter().enumerate() {
            dq[idx * 2] = lut_bf16[bi & 0x0F];
            dq[idx * 2 + 1] = lut_bf16[bi >> 4];
        }

        let q0 = vld1q_u16(query_bf16.as_ptr().add(dim_index));
        let q1 = vld1q_u16(query_bf16.as_ptr().add(dim_index + 8));
        let q2 = vld1q_u16(query_bf16.as_ptr().add(dim_index + 16));
        let q3 = vld1q_u16(query_bf16.as_ptr().add(dim_index + 24));
        let d0 = vld1q_u16(dq.as_ptr());
        let d1 = vld1q_u16(dq.as_ptr().add(8));
        let d2 = vld1q_u16(dq.as_ptr().add(16));
        let d3 = vld1q_u16(dq.as_ptr().add(24));

        acc0 = bfdot_asm(acc0, q0, d0);
        acc1 = bfdot_asm(acc1, q1, d1);
        acc2 = bfdot_asm(acc2, q2, d2);
        acc3 = bfdot_asm(acc3, q3, d3);

        dim_index += 32;
    }

    // 16-wide tail (one bfdot pair) for the partial trailing chunk.
    while dim_index + 16 <= dimensions {
        let byte_base = dim_index / 2;
        let b0 = *code.get_unchecked(byte_base) as usize;
        let b1 = *code.get_unchecked(byte_base + 1) as usize;
        let b2 = *code.get_unchecked(byte_base + 2) as usize;
        let b3 = *code.get_unchecked(byte_base + 3) as usize;
        let b4 = *code.get_unchecked(byte_base + 4) as usize;
        let b5 = *code.get_unchecked(byte_base + 5) as usize;
        let b6 = *code.get_unchecked(byte_base + 6) as usize;
        let b7 = *code.get_unchecked(byte_base + 7) as usize;
        let dq: [u16; 16] = [
            lut_bf16[b0 & 0x0F],
            lut_bf16[b0 >> 4],
            lut_bf16[b1 & 0x0F],
            lut_bf16[b1 >> 4],
            lut_bf16[b2 & 0x0F],
            lut_bf16[b2 >> 4],
            lut_bf16[b3 & 0x0F],
            lut_bf16[b3 >> 4],
            lut_bf16[b4 & 0x0F],
            lut_bf16[b4 >> 4],
            lut_bf16[b5 & 0x0F],
            lut_bf16[b5 >> 4],
            lut_bf16[b6 & 0x0F],
            lut_bf16[b6 >> 4],
            lut_bf16[b7 & 0x0F],
            lut_bf16[b7 >> 4],
        ];
        let q_lo = vld1q_u16(query_bf16.as_ptr().add(dim_index));
        let q_hi = vld1q_u16(query_bf16.as_ptr().add(dim_index + 8));
        let d_lo = vld1q_u16(dq.as_ptr());
        let d_hi = vld1q_u16(dq.as_ptr().add(8));
        acc0 = bfdot_asm(acc0, q_lo, d_lo);
        acc1 = bfdot_asm(acc1, q_hi, d_hi);
        dim_index += 16;
    }

    let mut lanes = [0.0_f32; 4];
    vst1q_f32(
        lanes.as_mut_ptr(),
        vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3)),
    );
    let mut sum = lanes[0] + lanes[1] + lanes[2] + lanes[3];

    // Scalar tail: rebuild from the bf16 representations for consistency.
    while dim_index < dimensions {
        let level = ((*code.get_unchecked(dim_index / 2) >> ((dim_index % 2) * 4)) & 0x0F) as usize;
        let q_f32 = f32::from_bits((query_bf16[dim_index] as u32) << 16);
        let dq_f32 = f32::from_bits((lut_bf16[level] as u32) << 16);
        sum += q_f32 * dq_f32;
        dim_index += 1;
    }

    sum
}

/// bits=1 NEON sign-popcount kernel. 1-bit codes are 4× smaller than
/// 4-bit codes per vector (192 vs 768 bytes at dim=1536), so this
/// kernel is bandwidth-bound on memory in addition to being compute-
/// efficient. The dequant LUT has only 2 entries: `lut[0] = -1/√D`,
/// `lut[1] = +1/√D`. Each code byte unpacks to 8 lanes of dequant
/// values.
///
/// Loop layout: 32 dims per iteration = 4 code bytes, dispatched to
/// 4 independent `vfmaq_f32` chains. Same 4-way unroll pattern as
/// the bits=4 kernel to fill Neoverse-V2's 4 SIMD pipes.
// build_bits1_byte_lut_boxed (defined above) is the per-query
// builder; the prior per-candidate version was a perf bug and is
// removed.

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn sum_query_dequant_neon_bits1(
    query_rotated: &[f32],
    dimensions: usize,
    byte_lut: &[[f32; 8]; 256],
    lut: &[f32; 256],
    code: &[u8],
) -> f32 {
    use std::arch::aarch64::{vaddq_f32, vdupq_n_f32, vfmaq_f32, vld1q_f32, vst1q_f32};

    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);

    let mut dim_index = 0_usize;
    // Each iter consumes 32 dims = 4 code bytes. The byte LUT gives
    // 8 dequant lanes per byte as one contiguous 8-f32 row, so we do
    // exactly 8 vector loads (4 query + 4 dequant) per 32 dims with
    // zero per-byte scalar work.
    while dim_index + 32 <= dimensions {
        let byte_base = dim_index / 8;
        // SAFETY: dim_index + 32 <= dimensions ⇒ byte_base + 4 <= packed_bytes.
        let b0 = *code.get_unchecked(byte_base) as usize;
        let b1 = *code.get_unchecked(byte_base + 1) as usize;
        let b2 = *code.get_unchecked(byte_base + 2) as usize;
        let b3 = *code.get_unchecked(byte_base + 3) as usize;

        let r0 = byte_lut.as_ptr().add(b0).cast::<f32>();
        let r1 = byte_lut.as_ptr().add(b1).cast::<f32>();
        let r2 = byte_lut.as_ptr().add(b2).cast::<f32>();
        let r3 = byte_lut.as_ptr().add(b3).cast::<f32>();

        // SAFETY: each row is a [f32; 8] inside the byte_lut owned by this
        // stack frame; 8-element row → two 4-lane reads, in-bounds.
        let q0 = vld1q_f32(query_rotated.as_ptr().add(dim_index));
        let q1 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 4));
        let q2 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 8));
        let q3 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 12));
        let q4 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 16));
        let q5 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 20));
        let q6 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 24));
        let q7 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 28));
        let d0 = vld1q_f32(r0);
        let d1 = vld1q_f32(r0.add(4));
        let d2 = vld1q_f32(r1);
        let d3 = vld1q_f32(r1.add(4));
        let d4 = vld1q_f32(r2);
        let d5 = vld1q_f32(r2.add(4));
        let d6 = vld1q_f32(r3);
        let d7 = vld1q_f32(r3.add(4));
        acc0 = vfmaq_f32(acc0, q0, d0);
        acc1 = vfmaq_f32(acc1, q1, d1);
        acc2 = vfmaq_f32(acc2, q2, d2);
        acc3 = vfmaq_f32(acc3, q3, d3);
        acc0 = vfmaq_f32(acc0, q4, d4);
        acc1 = vfmaq_f32(acc1, q5, d5);
        acc2 = vfmaq_f32(acc2, q6, d6);
        acc3 = vfmaq_f32(acc3, q7, d7);

        dim_index += 32;
    }

    let neg = lut[0];
    let pos = lut[1];

    let mut lanes = [0.0_f32; 4];
    vst1q_f32(
        lanes.as_mut_ptr(),
        vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3)),
    );
    let mut sum = lanes[0] + lanes[1] + lanes[2] + lanes[3];

    // Scalar tail for the trailing < 32 dimensions.
    while dim_index < dimensions {
        let bit = (code[dim_index / 8] >> (dim_index % 8)) & 1;
        sum += query_rotated[dim_index] * if bit == 1 { pos } else { neg };
        dim_index += 1;
    }

    sum
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn sum_query_dequant_neon_bits4(
    query_rotated: &[f32],
    dimensions: usize,
    lut: &[f32; 256],
    code: &[u8],
) -> f32 {
    use std::arch::aarch64::{vaddq_f32, vdupq_n_f32, vfmaq_f32, vld1q_f32, vst1q_f32};

    // 4 accumulator chains so Neoverse-V2's 4 SIMD pipes stay busy
    // (each pipe can issue one 128-bit FMA per cycle). With only 2
    // chains the loop bottlenecks on the dependency chain through
    // each accumulator's FMA latency.
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);

    let mut dim_index = 0_usize;
    // Each iteration consumes 16 dimensions = 8 bytes of code,
    // dispatching to 4 vfmaq_f32 calls (4 independent chains).
    while dim_index + 16 <= dimensions {
        let byte_base = dim_index / 2;
        // SAFETY: dim_index + 16 <= dimensions and the code-length
        // invariant checked by the caller guarantees byte_base + 8 <=
        // packed_bytes.
        let b0 = *code.get_unchecked(byte_base) as usize;
        let b1 = *code.get_unchecked(byte_base + 1) as usize;
        let b2 = *code.get_unchecked(byte_base + 2) as usize;
        let b3 = *code.get_unchecked(byte_base + 3) as usize;
        let b4 = *code.get_unchecked(byte_base + 4) as usize;
        let b5 = *code.get_unchecked(byte_base + 5) as usize;
        let b6 = *code.get_unchecked(byte_base + 6) as usize;
        let b7 = *code.get_unchecked(byte_base + 7) as usize;

        // 16 nibbles unpacked in coordinate order (low nibble at 2k,
        // high nibble at 2k+1).
        let dq: [f32; 16] = [
            lut[b0 & 0x0F],
            lut[b0 >> 4],
            lut[b1 & 0x0F],
            lut[b1 >> 4],
            lut[b2 & 0x0F],
            lut[b2 >> 4],
            lut[b3 & 0x0F],
            lut[b3 >> 4],
            lut[b4 & 0x0F],
            lut[b4 >> 4],
            lut[b5 & 0x0F],
            lut[b5 >> 4],
            lut[b6 & 0x0F],
            lut[b6 >> 4],
            lut[b7 & 0x0F],
            lut[b7 >> 4],
        ];

        // SAFETY: dim_index + 16 <= dimensions ⇒ four 4-lane query loads OK.
        let q0 = vld1q_f32(query_rotated.as_ptr().add(dim_index));
        let q1 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 4));
        let q2 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 8));
        let q3 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 12));
        let d0 = vld1q_f32(dq.as_ptr());
        let d1 = vld1q_f32(dq.as_ptr().add(4));
        let d2 = vld1q_f32(dq.as_ptr().add(8));
        let d3 = vld1q_f32(dq.as_ptr().add(12));
        acc0 = vfmaq_f32(acc0, q0, d0);
        acc1 = vfmaq_f32(acc1, q1, d1);
        acc2 = vfmaq_f32(acc2, q2, d2);
        acc3 = vfmaq_f32(acc3, q3, d3);

        dim_index += 16;
    }

    // 8-wide tail for the [dim - 16, dim - 8) range (only fires when
    // `dimensions` is not a multiple of 16).
    while dim_index + 8 <= dimensions {
        let byte_base = dim_index / 2;
        let b0 = *code.get_unchecked(byte_base) as usize;
        let b1 = *code.get_unchecked(byte_base + 1) as usize;
        let b2 = *code.get_unchecked(byte_base + 2) as usize;
        let b3 = *code.get_unchecked(byte_base + 3) as usize;
        let dq = [
            lut[b0 & 0x0F],
            lut[b0 >> 4],
            lut[b1 & 0x0F],
            lut[b1 >> 4],
            lut[b2 & 0x0F],
            lut[b2 >> 4],
            lut[b3 & 0x0F],
            lut[b3 >> 4],
        ];
        let q0 = vld1q_f32(query_rotated.as_ptr().add(dim_index));
        let q1 = vld1q_f32(query_rotated.as_ptr().add(dim_index + 4));
        let d0 = vld1q_f32(dq.as_ptr());
        let d1 = vld1q_f32(dq.as_ptr().add(4));
        acc0 = vfmaq_f32(acc0, q0, d0);
        acc1 = vfmaq_f32(acc1, q1, d1);

        dim_index += 8;
    }

    // Fold the 4 accumulator chains into one f32.
    let mut lanes = [0.0_f32; 4];
    vst1q_f32(
        lanes.as_mut_ptr(),
        vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3)),
    );
    let mut sum = lanes[0] + lanes[1] + lanes[2] + lanes[3];

    // Scalar tail for the trailing < 8 dimensions.
    while dim_index < dimensions {
        let l = read_level(code, dim_index, 4) as usize;
        sum += query_rotated[dim_index] * lut[l];
        dim_index += 1;
    }

    sum
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
    query_bf16: Option<&[u16]>,
    dequant_lut_bf16: Option<&[u16; 256]>,
    query_norm: f32,
    dimensions: usize,
    bits_per_dim: u8,
    dequant_lut: &[f32; 256],
    bits1_byte_lut: Option<&[[f32; 8]; 256]>,
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

    let sum_q_dequant = sum_query_dequant_with_bf16(
        query_rotated,
        query_bf16,
        dequant_lut_bf16,
        dimensions,
        bits,
        dequant_lut,
        bits1_byte_lut,
        code,
    );

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

/// Bound-free variant of `estimate_ip_impl`: skips the ε-concentration
/// computation (sqrt + scalar multiplies) for callers that don't need
/// the error envelope. ec_ivf scoring is the canonical user.
#[inline]
fn estimate_ip_scalar_only_impl(
    query_rotated: &[f32],
    query_bf16: Option<&[u16]>,
    dequant_lut_bf16: Option<&[u16; 256]>,
    dimensions: usize,
    bits_per_dim: u8,
    dequant_lut: &[f32; 256],
    bits1_byte_lut: Option<&[[f32; 8]; 256]>,
    code: &[u8],
) -> f32 {
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

    let sum_q_dequant = sum_query_dequant_with_bf16(
        query_rotated,
        query_bf16,
        dequant_lut_bf16,
        dimensions,
        bits,
        dequant_lut,
        bits1_byte_lut,
        code,
    );

    const O_DOT_FLOOR: f32 = 1e-6;
    if candidate_o_dot.abs() < O_DOT_FLOOR
        || !candidate_o_dot.is_finite()
        || candidate_x_norm <= 0.0
        || !candidate_x_norm.is_finite()
    {
        return 0.0;
    }
    candidate_norm * sum_q_dequant / (candidate_o_dot * candidate_x_norm)
}

#[inline]
fn estimate_ip_least_squares_scalar_only_impl(
    query_rotated: &[f32],
    query_bf16: Option<&[u16]>,
    dequant_lut_bf16: Option<&[u16; 256]>,
    dimensions: usize,
    bits_per_dim: u8,
    dequant_lut: &[f32; 256],
    bits1_byte_lut: Option<&[[f32; 8]; 256]>,
    code: &[u8],
) -> f32 {
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

    let sum_q_dequant = sum_query_dequant_with_bf16(
        query_rotated,
        query_bf16,
        dequant_lut_bf16,
        dimensions,
        bits,
        dequant_lut,
        bits1_byte_lut,
        code,
    );

    const O_DOT_FLOOR: f32 = 1e-6;
    if candidate_o_dot.abs() < O_DOT_FLOOR
        || !candidate_o_dot.is_finite()
        || candidate_x_norm <= 0.0
        || !candidate_x_norm.is_finite()
    {
        return 0.0;
    }
    candidate_norm * candidate_o_dot * sum_q_dequant / candidate_x_norm
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

    fn identity_quantizer(dim: usize, bits: u8) -> RaBitQQuantizer {
        let rotation: Arc<dyn Rotation> = Arc::new(Identity { dim });
        RaBitQQuantizer::with_bits(rotation, bits).unwrap()
    }

    fn read_tail_f32(code: &[u8], packed_bytes: usize, offset: usize) -> f32 {
        f32::from_le_bytes(
            code[packed_bytes + offset..packed_bytes + offset + 4]
                .try_into()
                .unwrap(),
        )
    }

    fn write_tail_f32(code: &mut [u8], packed_bytes: usize, offset: usize, value: f32) {
        code[packed_bytes + offset..packed_bytes + offset + 4]
            .copy_from_slice(&value.to_le_bytes());
    }

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
    fn quantizer_and_prepared_estimator_accessors_are_stable() {
        let q = identity_quantizer(17, 4);
        assert_eq!(q.dimensions(), 17);
        assert_eq!(q.bits_per_dim(), 4);
        assert_eq!(q.packed_bytes(), 9);
        assert_eq!(q.sign_bytes(), 9);
        assert_eq!(
            <RaBitQQuantizer as crate::quant::Quantizer>::code_len(&q),
            9 + RABITQ_SCALAR_LEN
        );
        assert_eq!(
            <RaBitQQuantizer as crate::quant::Quantizer>::wire_format_version(&q),
            0
        );

        let query: Vec<f32> = (0..17).map(|i| i as f32 - 8.0).collect();
        let prepared = q.prepare_estimator(&query);
        assert_eq!(prepared.dimensions(), 17);
        assert_eq!(prepared.bits_per_dim(), 4);
        let expected_norm = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((prepared.query_norm() - expected_norm).abs() < 1e-6);
    }

    #[test]
    fn query_scorer_matches_estimator_for_nontrivial_pair() {
        let q = identity_quantizer(12, 2);
        let query: Vec<f32> = (0..12).map(|i| i as f32 * 0.25 - 1.5).collect();
        let candidate: Vec<f32> = (0..12).map(|i| 1.0 - i as f32 * 0.125).collect();
        let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &candidate);
        let prepared = q.prepare_estimator(&query);
        let expected = q.estimate_ip(&prepared, &code).estimate;
        let scorer = <RaBitQQuantizer as crate::quant::Quantizer>::prepare_scorer(&q, &query);
        let score = scorer.score(&code);

        assert!((score - expected).abs() < 1e-6);
        assert!(score != 0.0);
        assert!(score != 1.0);
    }

    #[test]
    fn bits1_batch_estimator_matches_scalar_order() {
        let q = identity_quantizer(17, 1);
        let query: Vec<f32> = (0..17).map(|i| i as f32 * 0.125 - 0.75).collect();
        let candidates = (0..5)
            .map(|row| {
                (0..17)
                    .map(|col| (row as f32 - col as f32) * 0.0625)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let codes = candidates
            .iter()
            .flat_map(|candidate| {
                <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, candidate).into_vec()
            })
            .collect::<Vec<_>>();
        let prepared = q.prepare_estimator(&query);
        let code_len = <RaBitQQuantizer as crate::quant::Quantizer>::code_len(&q);
        let mut batch_scores = vec![123.0];

        prepared
            .estimate_ip_bits1_batch(&codes, code_len, &mut batch_scores)
            .unwrap();

        assert_eq!(batch_scores.len(), candidates.len());
        for (index, candidate) in candidates.iter().enumerate() {
            let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, candidate);
            let scalar = prepared.estimate_ip_scalar_only(&code);
            assert!(
                (batch_scores[index] - scalar).abs() < 1e-6,
                "index={index} batch={} scalar={scalar}",
                batch_scores[index]
            );
        }
    }

    #[test]
    fn bits1_batch_estimator_rejects_wrong_width_and_stride() {
        let q = identity_quantizer(17, 2);
        let query: Vec<f32> = (0..17).map(|i| i as f32 * 0.125 - 0.75).collect();
        let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &query);
        let prepared = q.prepare_estimator(&query);
        let mut batch_scores = Vec::new();

        let err = prepared
            .estimate_ip_bits1_batch(&code, code.len(), &mut batch_scores)
            .unwrap_err();
        assert!(err.contains("requires bits=1"));

        let q = identity_quantizer(17, 1);
        let prepared = q.prepare_estimator(&query);
        let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &query);
        let err = prepared
            .estimate_ip_bits1_batch(&code[..code.len() - 1], code.len(), &mut batch_scores)
            .unwrap_err();
        assert!(err.contains("not divisible"));
    }

    #[test]
    fn encode_code_pins_qbit_payload_and_scalars() {
        let q = identity_quantizer(5, 2);
        let v = [-0.25_f32, 0.0, 0.25, 0.5, -0.5];
        let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &v);

        assert_eq!(&code[..2], &[0xe9, 0x00]);
        let packed_bytes = q.packed_bytes();
        let norm = read_tail_f32(&code, packed_bytes, 0);
        let o_dot = read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN);
        let x_dec_norm = read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN);

        let expected_norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        let sqrt_d = (v.len() as f32).sqrt();
        let dequant = [
            -0.5 / sqrt_d,
            0.5 / sqrt_d,
            0.5 / sqrt_d,
            1.5 / sqrt_d,
            -1.5 / sqrt_d,
        ];
        let expected_x_dec_norm = dequant.iter().map(|x| x * x).sum::<f32>().sqrt();
        let inner: f32 = v.iter().zip(dequant.iter()).map(|(a, b)| a * b).sum();
        let expected_o_dot = inner / (expected_norm * expected_x_dec_norm);

        assert!((norm - expected_norm).abs() < 1e-6);
        assert!((x_dec_norm - expected_x_dec_norm).abs() < 1e-6);
        assert!((o_dot - expected_o_dot).abs() < 1e-6);
    }

    #[test]
    fn encode_code_pins_zero_vector_tail_and_levels() {
        let q = identity_quantizer(5, 2);
        let code = <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &[0.0_f32; 5]);
        let packed_bytes = q.packed_bytes();

        assert_eq!(&code[..packed_bytes], &[0xaa, 0x02]);
        assert_eq!(read_tail_f32(&code, packed_bytes, 0), 0.0);
        assert_eq!(read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN), 0.0);
        assert!(
            read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN,).is_finite()
        );
    }

    #[test]
    fn estimate_ip_guards_and_bound_are_pinned() {
        let q = identity_quantizer(5, 2);
        let query = [0.75_f32, -0.25, 1.25, -1.5, 0.5];
        let candidate = [-0.25_f32, 0.0, 0.25, 0.5, -0.5];
        let mut code =
            <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &candidate).into_vec();
        let prepared = q.prepare_estimator(&query);
        let packed_bytes = q.packed_bytes();

        let estimate = q.estimate_ip(&prepared, &code);
        let candidate_norm = read_tail_f32(&code, packed_bytes, 0);
        let o_dot = read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN);
        let x_norm = read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN);
        let sqrt_d = (q.dimensions() as f32).sqrt();
        let mut sum_q_dequant = 0.0_f32;
        for (i, &query_i) in query.iter().enumerate() {
            sum_q_dequant += query_i
                * dequant_level(
                    read_level(&code, i, 2),
                    2,
                    sqrt_d,
                    RABITQ_DEFAULT_QUANT_CLIP,
                );
        }
        let expected_estimate = candidate_norm * sum_q_dequant / (o_dot * x_norm);
        let o_dot_sq = o_dot * o_dot;
        let expected_bound = RABITQ_BOUND_CONFIDENCE
            * prepared.query_norm()
            * candidate_norm
            * (((1.0 - o_dot_sq).max(0.0)) / (q.dimensions() as f32 * o_dot_sq)).sqrt();
        assert!((estimate.estimate - expected_estimate).abs() < 1e-6);
        assert!((estimate.bound - expected_bound).abs() < 1e-6);

        write_tail_f32(&mut code, packed_bytes, RABITQ_NORM_LEN, 1e-6);
        assert!(q.estimate_ip(&prepared, &code).estimate.is_finite());

        write_tail_f32(&mut code, packed_bytes, RABITQ_NORM_LEN, f32::NAN);
        assert!(q.estimate_ip(&prepared, &code).bound.is_infinite());

        write_tail_f32(&mut code, packed_bytes, RABITQ_NORM_LEN, o_dot);
        write_tail_f32(
            &mut code,
            packed_bytes,
            RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN,
            0.0,
        );
        assert!(q.estimate_ip(&prepared, &code).bound.is_infinite());

        write_tail_f32(
            &mut code,
            packed_bytes,
            RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN,
            f32::NAN,
        );
        assert!(q.estimate_ip(&prepared, &code).bound.is_infinite());
    }

    #[test]
    fn least_squares_estimator_uses_o_dot_as_shrinkage() {
        let q = identity_quantizer(5, 2);
        let query = [0.75_f32, -0.25, 1.25, -1.5, 0.5];
        let candidate = [-0.25_f32, 0.0, 0.25, 0.5, -0.5];
        let code =
            <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &candidate).into_vec();
        let prepared = q.prepare_estimator(&query);
        let packed_bytes = q.packed_bytes();

        let candidate_norm = read_tail_f32(&code, packed_bytes, 0);
        let o_dot = read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN);
        let x_norm = read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN);
        let sqrt_d = (q.dimensions() as f32).sqrt();
        let mut sum_q_dequant = 0.0_f32;
        for (i, &query_i) in query.iter().enumerate() {
            sum_q_dequant += query_i
                * dequant_level(
                    read_level(&code, i, 2),
                    2,
                    sqrt_d,
                    RABITQ_DEFAULT_QUANT_CLIP,
                );
        }
        let expected = candidate_norm * o_dot * sum_q_dequant / x_norm;

        let estimate = prepared.estimate_ip_least_squares_scalar_only(&code);
        assert!((estimate - expected).abs() < 1e-6);
        assert_ne!(estimate, prepared.estimate_ip_scalar_only(&code));
    }

    #[test]
    fn estimate_ip_o_dot_floor_covers_below_equal_and_above() {
        let q = identity_quantizer(5, 2);
        let query = [0.75_f32, -0.25, 1.25, -1.5, 0.5];
        let candidate = [-0.25_f32, 0.0, 0.25, 0.5, -0.5];
        let mut code =
            <RaBitQQuantizer as crate::quant::Quantizer>::encode_code(&q, &candidate).into_vec();
        let prepared = q.prepare_estimator(&query);
        let packed_bytes = q.packed_bytes();

        write_tail_f32(&mut code, packed_bytes, RABITQ_NORM_LEN, 0.5e-6);
        let below = q.estimate_ip(&prepared, &code);
        assert_eq!(below.estimate, 0.0);
        assert!(below.bound.is_infinite());

        // Floor uses strict `<`: at o_dot == floor the guard must NOT fire, so
        // the bound is finite. Mutating `<` to `<=` would return INFINITY here.
        write_tail_f32(&mut code, packed_bytes, RABITQ_NORM_LEN, 1.0e-6);
        let at = q.estimate_ip(&prepared, &code);
        assert!(at.estimate.is_finite());
        assert!(at.bound.is_finite());

        write_tail_f32(&mut code, packed_bytes, RABITQ_NORM_LEN, 2.0e-6);
        let above = q.estimate_ip(&prepared, &code);
        assert!(above.estimate.is_finite());
        assert!(above.bound.is_finite());
    }

    #[test]
    fn bit_level_packers_round_trip_all_supported_widths() {
        for bits in [1_usize, 2, 4, 8] {
            let dim = 17;
            let levels = 1_u32 << bits;
            let mut packed = vec![0_u8; (dim * bits).div_ceil(8)];
            let expected: Vec<u32> = (0..dim).map(|i| ((i * 3 + bits) as u32) % levels).collect();

            for (i, &level) in expected.iter().enumerate() {
                write_level(&mut packed, i, bits, level);
            }
            for (i, &level) in expected.iter().enumerate() {
                assert_eq!(read_level(&packed, i, bits), level, "bits={bits} i={i}");
            }
        }

        let packed_q2 = [0b1110_0100_u8, 0b1001_0011];
        assert_eq!(
            (0..8)
                .map(|i| read_level(&packed_q2, i, 2))
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 3, 0, 1, 2]
        );
    }

    #[test]
    fn qbit_quantize_and_dequantize_pin_level_midpoints() {
        let sqrt_d = 4.0_f32;
        for bits in [2_usize, 4, 8] {
            let levels = 1_u32 << bits;
            for level in [0_u32, 1, levels / 2, levels - 2, levels - 1] {
                let dequant = dequant_level(level, bits, sqrt_d, RABITQ_DEFAULT_QUANT_CLIP);
                let (roundtrip_level, roundtrip_dequant) =
                    quantize_level(dequant, bits, sqrt_d, RABITQ_DEFAULT_QUANT_CLIP);
                assert_eq!(roundtrip_level, level, "bits={bits} level={level}");
                assert!(
                    (roundtrip_dequant - dequant).abs() < 1e-6,
                    "bits={bits} level={level} dequant={dequant} roundtrip={roundtrip_dequant}"
                );
            }
        }

        assert_eq!(
            quantize_level(0.0, 1, sqrt_d, RABITQ_DEFAULT_QUANT_CLIP),
            (1, 1.0)
        );
        assert_eq!(
            quantize_level(-0.01, 1, sqrt_d, RABITQ_DEFAULT_QUANT_CLIP),
            (0, -1.0)
        );
        assert_eq!(dequant_level(1, 1, sqrt_d, RABITQ_DEFAULT_QUANT_CLIP), 1.0);
        assert_eq!(dequant_level(0, 1, sqrt_d, RABITQ_DEFAULT_QUANT_CLIP), -1.0);

        assert_eq!(
            quantize_level(-0.25, 2, sqrt_d, RABITQ_DEFAULT_QUANT_CLIP).0,
            1
        );
        assert_eq!(
            quantize_level(0.25, 2, sqrt_d, RABITQ_DEFAULT_QUANT_CLIP).0,
            3
        );
    }

    #[test]
    fn qbit_quantize_pins_non_roundtrip_scale_term() {
        let sqrt_d = 4.0_f32;
        let (level, dequant) = quantize_level(-0.125, 2, sqrt_d, RABITQ_DEFAULT_QUANT_CLIP);

        assert_eq!(level, 1);
        assert!((dequant - (-0.5 / sqrt_d)).abs() < 1e-6);
    }

    #[test]
    fn sign_sidecar_helpers_pack_exact_words() {
        let codebook: Vec<f32> = (0..16).map(|i| i as f32 - 8.0).collect();
        let lookup = binary_sign_lookup_4bit(&codebook);
        assert_eq!(lookup, [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1]);

        let dim = 70_usize;
        let mut packed = vec![0_u8; dim.div_ceil(2)];
        let mut expected_words = vec![0_u64; dim.div_ceil(64)];
        for i in 0..dim {
            let nibble = ((i * 5 + 3) & 0x0f) as u8;
            if i % 2 == 0 {
                packed[i / 2] |= nibble;
            } else {
                packed[i / 2] |= nibble << 4;
            }
            if nibble >= 8 {
                expected_words[i / 64] |= 1_u64 << (i % 64);
            }
        }

        assert_eq!(
            sign_words_from_packed_4bit(&packed, dim, &lookup),
            expected_words
        );
        assert_eq!(
            sign_words_from_byte_slice(&[0b1010_0101, 0b0000_0011], 10),
            vec![0b11_1010_0101]
        );
    }

    #[test]
    fn persisted_sidecar_helpers_preserve_supported_and_unsupported_lanes() {
        let supported = ProdQuantizer::cached(1536, 4, 11);
        assert!(supported.binary_sign_no_qjl_4bit_supported());
        assert_eq!(persisted_sidecar_word_count(1536, 4, 11), 24);

        let vector = deterministic_gaussian(1536, 29);
        let encoded = supported.encode(&vector);
        let words = derive_persisted_sidecar_words(&supported, &encoded.mse_packed);
        assert_eq!(
            words,
            supported.binary_sign_words_from_packed_no_qjl_4bit(&encoded.mse_packed)
        );
        assert_eq!(words.len(), 24);

        let unsupported = ProdQuantizer::cached(1536, 2, 11);
        assert!(!unsupported.binary_sign_no_qjl_4bit_supported());
        assert_eq!(persisted_sidecar_word_count(1536, 2, 11), 0);
        assert!(derive_persisted_sidecar_words(&unsupported, &[]).is_empty());
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
    fn centered_context_and_scorer_cover_raw_and_degenerate_paths() {
        let dim = 8;
        let q = identity_quantizer(dim, 1);
        let center: Vec<f32> = (0..dim).map(|i| i as f32 * 0.25 - 0.5).collect();
        let mut v = center.clone();
        for (i, value) in v.iter_mut().enumerate() {
            *value += if i % 2 == 0 { 1.0 } else { -1.0 };
        }

        let ctx = q.prepare_center(&center);
        assert_eq!(ctx.raw(), center.as_slice());
        let mut code = q.encode_code_centered(&v, &ctx).into_vec();
        let scorer = q.prepare_scorer_centered(&v);

        let s = q.packed_bytes();
        let mut degenerate = code.clone();
        degenerate[s + RABITQ_NORM_LEN..s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN]
            .copy_from_slice(&0.0_f32.to_le_bytes());
        let est = scorer.score_at(&degenerate, &ctx);
        assert_eq!(est.estimate, 0.0);
        assert!(est.bound.is_infinite());

        degenerate[s + RABITQ_NORM_LEN..s + RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN]
            .copy_from_slice(&f32::NAN.to_le_bytes());
        let est = scorer.score_at(&degenerate, &ctx);
        assert_eq!(est.estimate, 0.0);
        assert!(est.bound.is_infinite());

        code[s..s + RABITQ_NORM_LEN].copy_from_slice(&0.0_f32.to_le_bytes());
        let est = scorer.score_at(&code, &ctx);
        assert_eq!(est.estimate, 0.0);
        assert!(est.bound.is_infinite());

        let mut floor_code = q.encode_code_centered(&v, &ctx).into_vec();
        write_tail_f32(&mut floor_code, s, RABITQ_NORM_LEN, 1e-6);
        assert!(scorer.score_at(&floor_code, &ctx).estimate.is_finite());

        let mut floor_query = center.clone();
        floor_query[0] += 1e-6;
        let floor_scorer = q.prepare_scorer_centered(&floor_query);
        let est = floor_scorer.score_at(&floor_code, &ctx);
        assert!(est.estimate.is_finite());
    }

    #[test]
    fn centered_encode_pins_asymmetric_center_tail_and_zero_residual() {
        let q = identity_quantizer(4, 1);
        let center = [2.0_f32, -0.5, 1.25, -3.0];
        let residual = [1.0_f32, -2.0, 0.5, -0.25];
        let v: Vec<f32> = center
            .iter()
            .zip(residual)
            .map(|(center_i, residual_i)| center_i + residual_i)
            .collect();
        let ctx = q.prepare_center(&center);
        let code = q.encode_code_centered(&v, &ctx);
        let packed_bytes = q.packed_bytes();

        assert_eq!(code[0] & 0x0f, 0b0101);
        assert!((read_tail_f32(&code, packed_bytes, 0) - 2.304886).abs() < 1e-5);
        assert!((read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN) - 0.8134892).abs() < 1e-6);
        assert!(
            (read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN,) - 3.375)
                .abs()
                < 1e-6
        );

        let zero_code = q.encode_code_centered(&center, &ctx);
        assert_eq!(read_tail_f32(&zero_code, packed_bytes, 0), 0.0);
        assert_eq!(
            read_tail_f32(&zero_code, packed_bytes, RABITQ_NORM_LEN),
            0.0
        );
        assert!(read_tail_f32(
            &zero_code,
            packed_bytes,
            RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN,
        )
        .is_finite());
    }

    #[test]
    fn centered_scorer_floor_boundaries_cover_below_equal_and_above() {
        let q = identity_quantizer(4, 1);
        let center = [0.0_f32; 4];
        let v = [1.0_f32, -1.0, 1.0, -1.0];
        let ctx = q.prepare_center(&center);
        let mut code = q.encode_code_centered(&v, &ctx).into_vec();
        let packed_bytes = q.packed_bytes();

        write_tail_f32(&mut code, packed_bytes, RABITQ_NORM_LEN, 0.5e-6);
        let scorer = q.prepare_scorer_centered(&[1.0_f32, 0.0, 0.0, 0.0]);
        let below_o_dot = scorer.score_at(&code, &ctx);
        assert_eq!(below_o_dot.estimate, 0.0);
        assert!(below_o_dot.bound.is_infinite());

        // o_dot == floor: strict `<` keeps the guard off, so bound is finite.
        // Mutating `<` to `<=` returns INFINITY for the bound.
        write_tail_f32(&mut code, packed_bytes, RABITQ_NORM_LEN, 1.0e-6);
        let at_o_dot = scorer.score_at(&code, &ctx);
        assert!(at_o_dot.estimate.is_finite());
        assert!(at_o_dot.bound.is_finite());

        write_tail_f32(&mut code, packed_bytes, RABITQ_NORM_LEN, 2.0e-6);
        let above_o_dot = scorer.score_at(&code, &ctx);
        assert!(above_o_dot.estimate.is_finite());
        assert!(above_o_dot.bound.is_finite());

        let below_query = q.prepare_scorer_centered(&[0.5e-6_f32, 0.0, 0.0, 0.0]);
        let below_qr = below_query.score_at(&code, &ctx);
        assert_eq!(below_qr.estimate, 0.0);
        assert!(below_qr.bound.is_infinite());

        // query_residual_mag == floor: same boundary semantics as above.
        let equal_query = q.prepare_scorer_centered(&[1.0e-6_f32, 0.0, 0.0, 0.0]);
        let at_qr = equal_query.score_at(&code, &ctx);
        assert!(at_qr.estimate.is_finite());
        assert!(at_qr.bound.is_finite());

        let above_query = q.prepare_scorer_centered(&[2.0e-6_f32, 0.0, 0.0, 0.0]);
        let above_qr = above_query.score_at(&code, &ctx);
        assert!(above_qr.estimate.is_finite());
        assert!(above_qr.bound.is_finite());
    }

    #[test]
    fn centered_scorer_matches_manual_unit_residual_ip() {
        let dim = 8;
        let q = identity_quantizer(dim, 1);
        let center = vec![0.25_f32; dim];
        let v_residual = [1.0_f32, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        let q_residual = [2.0_f32, 1.0, -1.0, -2.0, 2.0, 1.0, -1.0, -2.0];
        let v: Vec<f32> = center
            .iter()
            .zip(v_residual)
            .map(|(center_i, residual_i)| center_i + residual_i)
            .collect();
        let query: Vec<f32> = center
            .iter()
            .zip(q_residual)
            .map(|(center_i, residual_i)| center_i + residual_i)
            .collect();

        let ctx = q.prepare_center(&center);
        let code = q.encode_code_centered(&v, &ctx);
        let scorer = q.prepare_scorer_centered(&query);
        let est = scorer.score_at(&code, &ctx);

        let raw_ip: f32 = v_residual
            .iter()
            .zip(q_residual.iter())
            .map(|(a, b)| a * b)
            .sum();
        let v_mag = v_residual.iter().map(|x| x * x).sum::<f32>().sqrt();
        let q_mag = q_residual.iter().map(|x| x * x).sum::<f32>().sqrt();
        let truth = raw_ip / (v_mag * q_mag);

        assert!((est.estimate - truth).abs() < 1e-5);
        assert!(est.estimate != 0.0);
        assert!(est.estimate != 1.0);
        assert!(est.bound < 1e-4);
    }

    #[test]
    fn centered_scorer_pins_nontrivial_bound_formula() {
        let dim = 8;
        let q = identity_quantizer(dim, 1);
        let center = vec![0.25_f32; dim];
        let v_residual = [2.0_f32, -1.0, 0.5, -0.25, 1.5, -2.0, 0.75, -1.25];
        let q_residual = [1.5_f32, 0.75, -1.25, -0.5, 2.0, -1.0, -0.75, 0.25];
        let v: Vec<f32> = center
            .iter()
            .zip(v_residual)
            .map(|(center_i, residual_i)| center_i + residual_i)
            .collect();
        let query: Vec<f32> = center
            .iter()
            .zip(q_residual)
            .map(|(center_i, residual_i)| center_i + residual_i)
            .collect();

        let ctx = q.prepare_center(&center);
        let code = q.encode_code_centered(&v, &ctx);
        let scorer = q.prepare_scorer_centered(&query);
        let est = scorer.score_at(&code, &ctx);
        let packed_bytes = q.packed_bytes();
        let o_dot = read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN);
        let center_dot = read_tail_f32(&code, packed_bytes, RABITQ_NORM_LEN + RABITQ_UNIT_DOT_LEN);
        let sqrt_d = (dim as f32).sqrt();
        let mut sum_q_sign = 0.0_f32;
        for (i, query_i) in query.iter().enumerate() {
            let sign = if (code[i / 8] >> (i % 8)) & 1 == 1 {
                1.0
            } else {
                -1.0
            };
            sum_q_sign += query_i * sign;
        }
        let query_dot_code = sum_q_sign / sqrt_d;
        let query_residual_mag = q_residual.iter().map(|x| x * x).sum::<f32>().sqrt();
        let expected_estimate = ((query_dot_code - center_dot) / query_residual_mag) / o_dot;
        let o_dot_sq = o_dot * o_dot;
        let expected_bound = RABITQ_BOUND_CONFIDENCE
            * (((1.0 - o_dot_sq).max(0.0)) / (dim as f32 * o_dot_sq)).sqrt();

        assert!((est.estimate - expected_estimate).abs() < 1e-6);
        assert!((est.bound - expected_bound).abs() < 1e-6);
        assert!(est.bound > 0.01);
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
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            q.encode_code_centered(&v, &ctx);
        }));
        std::panic::set_hook(previous_hook);
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

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn neon_sum_query_dequant_matches_scalar_bits1() {
        // Cover the 32-wide loop, 32-wide tail re-entry, and short
        // sub-32 tails. 1536 is the production dim.
        for &dim in &[8_usize, 16, 24, 31, 32, 33, 48, 63, 64, 95, 96, 128, 1536] {
            let bits = 1_usize;
            let packed_bytes = (dim * bits).div_ceil(8);
            let mut code = vec![0_u8; packed_bytes + RABITQ_SCALAR_LEN];
            for i in 0..dim {
                let level = ((i * 13) as u32) & 1;
                write_level(&mut code, i, bits, level);
            }
            let query: Vec<f32> = (0..dim)
                .map(|i| ((i as f32) - dim as f32 / 2.0) * 0.07)
                .collect();
            let lut = build_dequant_lut(dim, bits, RABITQ_DEFAULT_QUANT_CLIP);
            let byte_lut = build_bits1_byte_lut_boxed(&lut, 1).expect("bits=1 builds byte LUT");
            let scalar = sum_query_dequant_scalar(&query, dim, bits, &lut, &code);
            // SAFETY: aarch64 cfg + valid byte_lut for bits=1.
            let neon = unsafe { sum_query_dequant_neon_bits1(&query, dim, &byte_lut, &lut, &code) };
            let tol = 1e-4_f32 * scalar.abs().max(1.0);
            assert!(
                (scalar - neon).abs() <= tol,
                "bits=1 dim={dim}: scalar={scalar} neon={neon}"
            );
        }
    }

    #[test]
    fn bits1_byte_lut_matches_per_bit_decode() {
        // The byte LUT must agree with the per-bit scalar decode for
        // every input byte and every bit position.
        let lut = build_dequant_lut(1536, 1, RABITQ_DEFAULT_QUANT_CLIP);
        let byte_lut = build_bits1_byte_lut_boxed(&lut, 1).expect("bits=1 builds byte LUT");
        for byte in 0_usize..256 {
            for i in 0_usize..8 {
                let bit = (byte >> i) & 1;
                let expected = lut[bit];
                let actual = byte_lut[byte][i];
                assert_eq!(actual, expected, "byte={byte} i={i}");
            }
        }
        assert!(build_bits1_byte_lut_boxed(&lut, 4).is_none());
    }

    #[test]
    fn bits1_byte_lut_scalar_sum_matches_per_bit_decode() {
        for &dim in &[1_usize, 7, 8, 9, 15, 16, 31, 32, 65, 1536] {
            let bits = 1_usize;
            let packed_bytes = (dim * bits).div_ceil(8);
            let mut code = vec![0_u8; packed_bytes + RABITQ_SCALAR_LEN];
            for i in 0..dim {
                let level = ((i * 17 + 3) as u32) & 1;
                write_level(&mut code, i, bits, level);
            }
            let query: Vec<f32> = (0..dim).map(|i| ((i as f32) * 0.03125).sin()).collect();
            let lut = build_dequant_lut(dim, bits, RABITQ_DEFAULT_QUANT_CLIP);
            let byte_lut = build_bits1_byte_lut_boxed(&lut, 1).expect("bits=1 builds byte LUT");

            let per_bit = sum_query_dequant_scalar(&query, dim, bits, &lut, &code);
            let byte_lut_sum =
                sum_query_dequant_bits1_byte_lut_scalar(&query, dim, &byte_lut, &code);
            let tol = 1e-5_f32 * per_bit.abs().max(1.0);
            assert!(
                (per_bit - byte_lut_sum).abs() <= tol,
                "bits=1 dim={dim}: per_bit={per_bit} byte_lut={byte_lut_sum}"
            );
        }
    }

    #[test]
    fn prepared_queries_only_keep_bits1_byte_lut_for_bits1() {
        let query = vec![0.25_f32; 16];
        let bits1 = identity_quantizer(16, 1).prepare_estimator(&query);
        let bits4 = identity_quantizer(16, 4).prepare_estimator(&query);

        assert!(bits1.bits1_byte_lut.is_some());
        assert!(bits4.bits1_byte_lut.is_none());
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn neon_sum_query_dequant_matches_scalar_bits4() {
        // bits=4 is the production path (DEFAULT_QUANT_BITS=4); cover both
        // the 8-wide NEON body and a non-multiple-of-8 tail by sweeping a
        // few dimensions.
        for &dim in &[8_usize, 16, 24, 31, 64, 1536] {
            let bits = 4_usize;
            let packed_bytes = (dim * bits).div_ceil(8);
            // Deterministic non-trivial inputs.
            let mut code = vec![0_u8; packed_bytes + RABITQ_SCALAR_LEN];
            for i in 0..dim {
                let level = ((i * 7) as u32) & 0x0F;
                write_level(&mut code, i, bits, level);
            }
            let query: Vec<f32> = (0..dim)
                .map(|i| ((i as f32) - dim as f32 / 2.0) * 0.13)
                .collect();
            let sqrt_d = (dim as f32).sqrt();

            let lut = build_dequant_lut(dim, bits, RABITQ_DEFAULT_QUANT_CLIP);
            let scalar = sum_query_dequant_scalar(&query, dim, bits, &lut, &code);
            // SAFETY: aarch64 cfg + always-present NEON on the supported
            // bench hosts (Graviton).
            let neon = unsafe { sum_query_dequant_neon_bits4(&query, dim, &lut, &code) };
            let tol = 1e-4_f32 * scalar.abs().max(1.0);
            assert!(
                (scalar - neon).abs() <= tol,
                "scalar={scalar} neon={neon} dim={dim} delta={}",
                (scalar - neon).abs(),
            );
        }
    }
}
