//! RaBitQ quantizer — ADR-045 Stage 1 (supersedes ADR-031 in scope).
//!
//! This module graduates the ADR-031 binary-prefilter work into a
//! first-class quantizer under the `Quantizer` / `QueryScorer` trait
//! seams. It ships in slices:
//!
//! | slice | lands                                                                    |
//! |-------|--------------------------------------------------------------------------|
//! | 1     | this skeleton: types, public surface, stubbed trait impls                |
//! | 2     | move the binary encode + POPCNT scorer from `src/am/` into this module   |
//! | 3     | rotation front-end seam (SRHT today, OPQ later per task 20)              |
//! | 4     | unbiased distance estimator + error-bound API (Stage 3 consumes this)    |
//! | 5     | Phase 2 recall study via `src/bin/rabitq_feasibility.rs`                 |
//!
//! Until slice 2 lands, AM code continues to call the existing
//! `ProdQuantizer::binary_sign_words_from_packed_no_qjl_4bit` /
//! `training::derive_persisted_binary_words` surface directly.
//! Both paths are not meant to coexist — slice 2 deletes the AM-side
//! entry points in the same commit that wires the trait surface.
//!
//! ## What RaBitQ is here
//!
//! A binary quantizer with a per-vector f32 norm. For a D-dimensional
//! input, the code is `D/8` bytes of sign bits over the rotated vector
//! plus one f32 norm — at D=1536 that is 192 B + 4 B = 196 B per
//! vector (PQ4 parity is 768 B; the Stage 1 gate is recall within 1pp
//! of exact at this storage).

#![allow(dead_code)]

use std::sync::Arc;

use crate::quant::prod::ProdQuantizer;

/// Number of bytes reserved per vector for the f32 norm sidecar.
pub const RABITQ_NORM_LEN: usize = 4;

/// One RaBitQ quantizer instance. Owns the rotation state and
/// per-vector encoding parameters. The rotation is held as an `Arc`
/// so AM build and scan paths can share one instance.
pub struct RaBitQQuantizer {
    /// Original (pre-rotation) input dimensionality.
    dimensions: usize,
    /// Rotation seam — slice 3 replaces this with a first-class
    /// `Rotation` trait. For slice 1 we reuse `ProdQuantizer`'s SRHT
    /// signs to keep the type wired without committing to a layout.
    rotation: Arc<ProdQuantizer>,
}

impl RaBitQQuantizer {
    /// Build a new RaBitQ quantizer.
    pub fn new(dimensions: usize, rotation: Arc<ProdQuantizer>) -> Self {
        assert!(dimensions > 0, "RaBitQ dimensions must be positive");
        Self {
            dimensions,
            rotation,
        }
    }

    /// Input dimensionality the quantizer was trained for.
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Byte length of the sign-bit portion of a code (pre-norm).
    pub fn sign_bytes(&self) -> usize {
        self.dimensions.div_ceil(8)
    }
}

impl crate::quant::Quantizer for RaBitQQuantizer {
    fn encode_code(&self, _v: &[f32]) -> Box<[u8]> {
        unimplemented!("RaBitQQuantizer::encode_code lands in slice 2")
    }

    fn prepare_scorer(
        &self,
        _query: &[f32],
    ) -> Box<dyn crate::quant::QueryScorer + Send + Sync + '_> {
        unimplemented!("RaBitQQuantizer::prepare_scorer lands in slice 2")
    }

    fn code_len(&self) -> usize {
        self.sign_bytes() + RABITQ_NORM_LEN
    }

    fn wire_format_version(&self) -> u32 {
        // Slice 2 wires this to a dedicated INDEX_FORMAT_* constant in
        // `src/am/page.rs`. Stubbed here so the skeleton compiles.
        0
    }
}

/// Prepared scorer for one query vector.
///
/// Holds the rotated query and, in slice 4, the per-query state needed
/// by the unbiased distance estimator (query norm + any precomputed
/// coefficients). Slice 2 fills this in with the XOR+POPCNT path.
pub struct RaBitQScorer {
    _rotated_query: Vec<f32>,
}

impl crate::quant::QueryScorer for RaBitQScorer {
    fn score(&self, _code: &[u8]) -> f32 {
        unimplemented!("RaBitQScorer::score lands in slice 2")
    }
}

/// Unbiased distance estimate with a symmetric error bound
/// (`estimate ± bound`). Stage 3 (task 27) sizes its candidate pool
/// from the bound distribution measured in Phase 2.
///
/// Slice 4 replaces this stub with the RaBitQ estimator; it is declared
/// here so the public API shape appears in the module from the first
/// commit and downstream tasks can see the seam.
#[derive(Debug, Clone, Copy)]
pub struct DistanceEstimate {
    pub estimate: f32,
    pub bound: f32,
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
}
