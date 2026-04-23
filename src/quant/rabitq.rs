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

/// Number of bytes reserved per vector for the f32 norm sidecar.
pub const RABITQ_NORM_LEN: usize = 4;

/// Binary-sign prepared query. Produced by
/// [`ProdQuantizer::prepare_ip_query_binary_sign_no_qjl_4bit`] and
/// consumed by the Hamming scorer. Moved here from `prod.rs` in
/// slice 2 so the RaBitQ scoring surface is self-contained.
#[derive(Debug, Clone, PartialEq)]
pub struct BinarySignNoQjl4BitQuery {
    pub words: Vec<u64>,
}

/// One RaBitQ quantizer instance. Owns the rotation state and
/// per-vector encoding parameters. The rotation is held as an `Arc`
/// so AM build and scan paths can share one instance.
pub struct RaBitQQuantizer {
    dimensions: usize,
    /// Rotation seam — slice 3 replaces this with a first-class
    /// `Rotation` trait. For slice 2 we reuse `ProdQuantizer`'s SRHT
    /// signs so the canonical RaBitQ encode (direct rotate +
    /// sign-extract) shares the same rotation as the ADR-031
    /// PQ-derived sidecar path.
    rotation: Arc<ProdQuantizer>,
}

impl RaBitQQuantizer {
    pub fn new(dimensions: usize, rotation: Arc<ProdQuantizer>) -> Self {
        assert!(dimensions > 0, "RaBitQ dimensions must be positive");
        Self {
            dimensions,
            rotation,
        }
    }

    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Byte length of the sign-bit portion of a code (pre-norm).
    pub fn sign_bytes(&self) -> usize {
        self.dimensions.div_ceil(8)
    }

    /// Rotate `v` via the shared SRHT signs and collect into a fresh
    /// buffer trimmed to `dimensions` coordinates.
    fn rotated(&self, v: &[f32]) -> Vec<f32> {
        assert_eq!(
            v.len(),
            self.dimensions,
            "RaBitQ input length mismatch: got {}, expected {}",
            v.len(),
            self.dimensions,
        );
        let padded = rotation::srht_padded(v, &self.rotation.signs);
        padded[..self.dimensions].to_vec()
    }
}

impl crate::quant::Quantizer for RaBitQQuantizer {
    /// Canonical RaBitQ encode: rotate, take the sign bit of each
    /// rotated coordinate into a `dim/8`-byte payload, append the
    /// rotated vector's L2 norm as a trailing 4-byte `f32`.
    ///
    /// This is the "proper" RaBitQ path. The ADR-031 optimization
    /// that derives sign words from an already-PQ-packed code lives
    /// at [`derive_persisted_sidecar_words`]; it is not reachable
    /// through the `Quantizer` trait because it needs a
    /// [`ProdQuantizer`] reference rather than the raw vector.
    fn encode_code(&self, v: &[f32]) -> Box<[u8]> {
        let rotated = self.rotated(v);
        let mut out = vec![0_u8; self.sign_bytes() + RABITQ_NORM_LEN];
        for (index, &value) in rotated.iter().enumerate() {
            if value >= 0.0 {
                out[index / 8] |= 1_u8 << (index % 8);
            }
        }
        let norm = l2_norm(&rotated);
        let norm_start = self.sign_bytes();
        out[norm_start..norm_start + 4].copy_from_slice(&norm.to_le_bytes());
        out.into_boxed_slice()
    }

    fn prepare_scorer(
        &self,
        query: &[f32],
    ) -> Box<dyn crate::quant::QueryScorer + Send + Sync + '_> {
        let rotated = self.rotated(query);
        let words = sign_words_from_rotated(&rotated);
        let norm = l2_norm(&rotated);
        Box::new(RaBitQScorer {
            query_words: words,
            query_norm: norm,
            dimensions: self.dimensions,
        })
    }

    fn code_len(&self) -> usize {
        self.sign_bytes() + RABITQ_NORM_LEN
    }

    fn wire_format_version(&self) -> u32 {
        // Dedicated INDEX_FORMAT_* constant arrives with AM-side
        // trait dispatch (later slice). No reader consumes this
        // value yet because no AM path holds a `&dyn Quantizer`
        // pointing at RaBitQQuantizer.
        0
    }
}

/// Prepared scorer for one query vector. Holds the rotated query's
/// sign words + L2 norm.
///
/// Slice 2 computes a cosine-like similarity via Hamming distance
/// between query and candidate sign words; the unbiased estimator
/// with an error bound lands in slice 4 and will consume the
/// candidate's persisted norm (bytes `sign_bytes()..code_len()` of
/// the code payload).
pub struct RaBitQScorer {
    query_words: Vec<u64>,
    query_norm: f32,
    dimensions: usize,
}

impl crate::quant::QueryScorer for RaBitQScorer {
    fn score(&self, code: &[u8]) -> f32 {
        let sign_bytes = self.dimensions.div_ceil(8);
        assert!(
            code.len() >= sign_bytes + RABITQ_NORM_LEN,
            "RaBitQ code too short: got {}, expected at least {}",
            code.len(),
            sign_bytes + RABITQ_NORM_LEN,
        );
        let candidate_words = sign_words_from_byte_slice(&code[..sign_bytes], self.dimensions);
        let candidate_norm = f32::from_le_bytes(
            code[sign_bytes..sign_bytes + 4]
                .try_into()
                .expect("norm slice is always 4 bytes"),
        );
        let hamming = hamming_similarity(&self.query_words, &candidate_words, self.dimensions);
        // Slice 2 score: Hamming similarity scaled by the product of
        // query and candidate norms. This is a coarse cosine-like
        // surrogate; slice 4 replaces it with the unbiased estimator.
        hamming * self.query_norm * candidate_norm / (self.dimensions as f32)
    }
}

/// Unbiased distance estimate with a symmetric error bound
/// (`estimate ± bound`). Stage 3 (task 27) sizes its candidate pool
/// from the bound distribution measured in Phase 2. Slice 4
/// replaces the stub with the RaBitQ estimator.
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
        let rotation = ProdQuantizer::cached(1536, 4, 0);
        let q = RaBitQQuantizer::new(1536, rotation);
        assert_eq!(q.sign_bytes(), 192);
        assert_eq!(
            <RaBitQQuantizer as crate::quant::Quantizer>::code_len(&q),
            192 + RABITQ_NORM_LEN
        );
    }

    #[test]
    fn encode_then_score_same_vector_is_nonnegative() {
        // Identity-rotated nonzero vector scores positive against
        // itself. This is a smoke test for slice 2's encode/score
        // round-trip, not a recall claim.
        let dim = 64;
        let rotation = ProdQuantizer::cached(dim, 4, 0);
        let q = RaBitQQuantizer::new(dim, rotation);
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
    fn hamming_similarity_identity_equals_dim() {
        let words = vec![0xAAAA_AAAA_AAAA_AAAA_u64; 2];
        assert_eq!(hamming_similarity(&words, &words, 128), 128.0);
    }
}
