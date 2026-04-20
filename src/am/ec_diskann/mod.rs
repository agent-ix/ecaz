//! Access-method scaffolding for the `ec_diskann` (Vamana) implementation.
//!
//! Phase 1A landing: skeleton + handler registration only. All scan,
//! insert, build, and vacuum callbacks intentionally error at runtime
//! with `not yet implemented` until subsequent phases of task 17 fill
//! them in. Reloption parsing and metadata-page layout are deferred to
//! phase 1B and phase 1C respectively.

mod ambuild;
pub mod build;
mod options;
pub mod page;
pub mod persist;
pub mod reader;
mod routine;
pub mod scan;
pub mod tuple;
pub mod vacuum;
pub mod vamana;

pub(super) const ECDISKANN_DEFAULT_GRAPH_DEGREE: i32 = 32;
pub(super) const ECDISKANN_MIN_GRAPH_DEGREE: i32 = 4;
pub(super) const ECDISKANN_MAX_GRAPH_DEGREE: i32 = 256;

pub(super) const ECDISKANN_DEFAULT_BUILD_LIST_SIZE: i32 = 100;
pub(super) const ECDISKANN_MIN_BUILD_LIST_SIZE: i32 = 10;
pub(super) const ECDISKANN_MAX_BUILD_LIST_SIZE: i32 = 1000;

pub(super) const ECDISKANN_DEFAULT_ALPHA: f32 = 1.2;
pub(super) const ECDISKANN_MIN_ALPHA: f32 = 1.0;
pub(super) const ECDISKANN_MAX_ALPHA: f32 = 2.0;

pub(super) const ECDISKANN_PLANNER_SCAN_ENABLED: bool = false;
