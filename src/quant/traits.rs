//! Quantizer seam traits for multi-family dispatch (ADR-041 stage 0).
//!
//! `scan.rs` and peer AM modules hold a `&dyn Quantizer` plus a
//! prepared-query object. Neither side knows which quantizer family it
//! holds; dispatch collapses to a single trait-object call instead of a
//! `match GraphStorageDescriptor` arm per scoring site.
//!
//! ## Naming deviation from ADR-041
//!
//! ADR-041 names the scorer trait `PreparedQuery`. That collides with
//! an existing concrete struct at `crate::quant::prod::PreparedQuery`
//! (TurboQuant's prepared query state, referenced in scan.rs, lib.rs,
//! explain.rs). Renaming that struct would ripple into user-visible
//! EXPLAIN text and 20+ source-site changes. The trait is named
//! [`QueryScorer`] here to sidestep the collision; the ADR's semantics
//! are preserved.
//!
//! Revisit during ADR-041 stage 2 (`am/tqhnsw/` rename): if the struct
//! naturally renames along with its module move, the trait can reclaim
//! `PreparedQuery`.

/// One quantizer family. Owns training, encode, and query preparation.
/// Object-safe: scan.rs holds `&dyn Quantizer`.
#[allow(dead_code)]
pub trait Quantizer: Send + Sync {
    /// Encode a raw f32 vector into this family's flat on-disk payload
    /// (ADR-041 stage 0 — the trait method name is `encode_code` rather
    /// than `encode` to stay clear of each family's inherent `encode`
    /// method, which typically returns a family-specific struct rather
    /// than raw bytes).
    fn encode_code(&self, v: &[f32]) -> Box<[u8]>;

    /// Prepare a query vector for scoring. Returns a boxed scorer the
    /// caller can score many candidate codes against. Named
    /// `prepare_scorer` rather than `prepare` for the same reason as
    /// `encode_code` above.
    fn prepare_scorer(&self, query: &[f32]) -> Box<dyn QueryScorer + Send + Sync + '_>;

    /// Byte length of a single encoded code. Layout-checked at
    /// index-build time; scan uses it to step through page payloads.
    fn code_len(&self) -> usize;

    /// Wire-format version written to the metadata page for an index
    /// built against this quantizer. Matches the `INDEX_FORMAT_*`
    /// constants in `src/am/page.rs`.
    fn wire_format_version(&self) -> u32;
}

/// Prepared scorer for one query vector. The trait hides per-family
/// state (TurboQuant LUT + rotated vector + QJL projection vs.
/// grouped-PQ LUT) behind a scalar `score(code) -> f32` call.
///
/// Batched scoring paths (FastScan's 32-wide SIMD) stay on
/// family-specific APIs; this trait is the point-scoring seam used by
/// the warm-steady-state / exact-rerank call sites. ADR-041 stage 0
/// explicitly keeps the batched `match GraphStorageDescriptor` outer
/// shape.
pub trait QueryScorer {
    /// Score one candidate code against the prepared query. Higher is
    /// closer per the index's inner-product convention.
    fn score(&self, code: &[u8]) -> f32;
}
