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
mod identity;
mod options;
pub mod page;
mod routine;
pub mod tuple;

pub(crate) fn register_gucs() {
    options::register_gucs();
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

/// FR-077 closure-overlap band. Provisional default until the M1 sharded
/// build measures it; the monolithic M0 build does not consume it.
pub(super) const ECDISTANN_DEFAULT_CLOSURE_EPSILON: f32 = 0.1;
pub(super) const ECDISTANN_MIN_CLOSURE_EPSILON: f32 = 0.0;
pub(super) const ECDISTANN_MAX_CLOSURE_EPSILON: f32 = 1.0;

/// FR-081 BW default; matches the ec_diskann batched-beam width measured in
/// Task 168 (packet 002 A/B).
pub(super) const ECDISTANN_DEFAULT_BEAM_WIDTH: i32 = 4;
pub(super) const ECDISTANN_MAX_BEAM_WIDTH: i32 = 64;

/// FR-081 H default; provisional until the M0 recall-vs-H kill-check
/// measurement (ADR-085 D2) pins it.
pub(super) const ECDISTANN_DEFAULT_HOP_ROUNDS: i32 = 8;
pub(super) const ECDISTANN_MAX_HOP_ROUNDS: i32 = 64;
