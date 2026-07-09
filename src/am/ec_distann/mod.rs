//! `ec_distann` is the DistributedANN-style fifth access method (ADR-085):
//! one global Vamana graph over all indexed vectors, lean node records
//! (coarse search code + adjacency + embedded neighbor codes, FR-076) with
//! the full-precision vector held once in a co-placed heap row for
//! node-local exact rerank (D11), searched by a coordinator loop of
//! head-index descent (FR-080) plus batched hop rounds (FR-081).
//!
//! This module currently carries the M0 single-node slice (Task 162): the
//! AM callback surface, reloptions/GUCs, and the metadata page. The graph
//! record format, monolithic build, head index, and local hop-round loop
//! land in the following Task 162 slices; sharding and the remote path are
//! M1+ (Tasks 163+).

mod ambuild;
#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::ambuild::read_metadata_from_index;
mod cost;
mod custom_scan;
mod dml;
mod epoch;
mod epoch_manifest;
mod expand;
mod expand_error;
mod head_cache;
mod identity;
mod insert;
mod options;
pub mod page;
pub(crate) mod placement;
pub(crate) mod quantizer;
pub(crate) mod reader;
mod remote_endpoint;
mod remote_transport;
mod roster;
mod routine;
pub(crate) mod scan;
mod shard_build;
pub mod tuple;

pub(crate) fn register_gucs() {
    options::register_gucs();
    roster::register_gucs();
}

/// Installs the multi-node CustomScan provider + planner hook (from `_PG_init`).
pub(crate) fn register_custom_scan() {
    custom_scan::register_custom_scan();
}

pub(super) const ECDISTANN_DEFAULT_GRAPH_DEGREE: i32 = 32;
pub(super) const ECDISTANN_MIN_GRAPH_DEGREE: i32 = 4;
pub(super) const ECDISTANN_MAX_GRAPH_DEGREE: i32 = 256;

pub(super) const ECDISTANN_DEFAULT_BUILD_LIST_SIZE: i32 = 100;
pub(super) const ECDISTANN_MIN_BUILD_LIST_SIZE: i32 = 10;
pub(super) const ECDISTANN_MAX_BUILD_LIST_SIZE: i32 = 1000;

pub(super) const ECDISTANN_DEFAULT_ALPHA: f32 = 1.2;
pub(super) const ECDISTANN_MIN_ALPHA: f32 = 1.0;
pub(super) const ECDISTANN_MAX_ALPHA: f32 = 2.0;

/// ADR-085 D3: fixed head-index cap C, default 4096; the M0 C-sensitivity
/// bench cell informs whether this default is frozen.
pub(super) const ECDISTANN_DEFAULT_HEAD_INDEX_CAP: i32 = 4096;
pub(super) const ECDISTANN_MIN_HEAD_INDEX_CAP: i32 = 16;
pub(super) const ECDISTANN_MAX_HEAD_INDEX_CAP: i32 = 1_048_576;

/// FR-077 closure-overlap band. Measured at M1 (task-163 packet 002): the M0
/// provisional 0.1 starved boundary nodes of cross-shard edges and the
/// stitched build trailed monolithic recall by up to 0.06 at 100k (gap growing
/// with corpus size). Widening to 0.3 recovers parity — stitched recall@10
/// matches or exceeds monolithic across the operational search band (ef>=64)
/// at 50k/100k — at near-monolithic build cost, and beats wider bands (0.6/1.0)
/// on the recall/cost tradeoff. Only consumed when `build_shards >= 2`.
pub(super) const ECDISTANN_DEFAULT_CLOSURE_EPSILON: f32 = 0.3;
pub(super) const ECDISTANN_MIN_CLOSURE_EPSILON: f32 = 0.0;
pub(super) const ECDISTANN_MAX_CLOSURE_EPSILON: f32 = 1.0;

/// FR-077 build-shard count. `0` = auto (monolithic below ~20k rows, then
/// ~one shard per 25k up to a cap); `1` forces the monolithic fallback path;
/// `>=2` selects the sharded closure-overlap build + stitch. Default is `1`
/// (monolithic) so the single-node default matches the M0-measured behavior
/// until the M1 A/B promotes the sharded path.
pub(super) const ECDISTANN_DEFAULT_BUILD_SHARDS: i32 = 1;
pub(super) const ECDISTANN_MIN_BUILD_SHARDS: i32 = 0;
pub(super) const ECDISTANN_MAX_BUILD_SHARDS: i32 = 4096;

/// FR-081 BW default; matches the ec_diskann batched-beam width measured in
/// Task 168 (packet 002 A/B).
pub(super) const ECDISTANN_DEFAULT_BEAM_WIDTH: i32 = 4;
pub(super) const ECDISTANN_MAX_BEAM_WIDTH: i32 = 64;

/// FR-081 H default. H is the NFR-019 hard round cap, not the quality
/// knob: the D9 early-exit (bounded by ec_distann.top_k) terminates real
/// scans, so the default is generous and the M0 kill-check measures the
/// recall-vs-H curve explicitly.
pub(super) const ECDISTANN_DEFAULT_HOP_ROUNDS: i32 = 100;
/// High ceiling so the bench sweep can match ec_diskann expansion budgets
/// (BW x H up to ~800 at the default BW=4); D9 early-exit terminates long
/// sweeps in practice.
pub(super) const ECDISTANN_MAX_HOP_ROUNDS: i32 = 256;

/// Result-heap size k for the FR-081 convergence early-exit; session GUC
/// because k is a query property, not an index property.
pub(super) const ECDISTANN_DEFAULT_TOP_K: i32 = 10;
pub(super) const ECDISTANN_MAX_TOP_K: i32 = 10_000;
