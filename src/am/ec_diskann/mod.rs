//! `ec_diskann` is the Vamana-based secondary access method.
//!
//! The module owns the AM callback surface plus the persisted Vamana
//! helpers that support build, ordered scan, live insert, vacuum
//! repair, and planner costing for grouped-PQ-backed `ecvector`
//! indexes.

mod ambuild;
pub mod build;
mod cost;
mod insert;
mod options;
pub mod page;
pub mod persist;
pub mod reader;
mod routine;
pub mod scan;
pub mod scan_query;
mod scan_state;
pub mod tuple;
pub mod vacuum;
pub mod vamana;

pub(crate) fn register_gucs() {
    options::register_gucs();
}

pub(super) const ECDISKANN_DEFAULT_GRAPH_DEGREE: i32 = 32;
pub(super) const ECDISKANN_MIN_GRAPH_DEGREE: i32 = 4;
pub(super) const ECDISKANN_MAX_GRAPH_DEGREE: i32 = 256;

pub(super) const ECDISKANN_DEFAULT_BUILD_LIST_SIZE: i32 = 100;
pub(super) const ECDISKANN_MIN_BUILD_LIST_SIZE: i32 = 10;
pub(super) const ECDISKANN_MAX_BUILD_LIST_SIZE: i32 = 1000;

pub(super) const ECDISKANN_DEFAULT_SCAN_LIST_SIZE: i32 = 100;
pub(super) const ECDISKANN_MIN_SCAN_LIST_SIZE: i32 = 1;
pub(super) const ECDISKANN_MAX_SCAN_LIST_SIZE: i32 = 10_000;

pub(super) const ECDISKANN_DEFAULT_RERANK_BUDGET: i32 = 64;
pub(super) const ECDISKANN_MIN_RERANK_BUDGET: i32 = 1;
pub(super) const ECDISKANN_MAX_RERANK_BUDGET: i32 = 10_000;

pub(super) const ECDISKANN_DEFAULT_TOP_K: i32 = 10;
pub(super) const ECDISKANN_MIN_TOP_K: i32 = 1;
pub(super) const ECDISKANN_MAX_TOP_K: i32 = 10_000;

pub(super) const ECDISKANN_DEFAULT_ALPHA: f32 = 1.2;
pub(super) const ECDISKANN_MIN_ALPHA: f32 = 1.0;
pub(super) const ECDISKANN_MAX_ALPHA: f32 = 2.0;

pub(super) const ECDISKANN_PLANNER_SCAN_ENABLED: bool = true;
pub(super) const ECDISKANN_UNIT_NORM_DISTANCE_BIAS: f32 = 1.0;
