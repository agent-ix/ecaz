//! Access-method scaffolding for the `tqdiskann` (Vamana) implementation.
//!
//! Phase 1A landing: skeleton + handler registration only. All scan,
//! insert, build, and vacuum callbacks intentionally error at runtime
//! with `not yet implemented` until subsequent phases of task 17 fill
//! them in. Reloption parsing and metadata-page layout are deferred to
//! phase 1B and phase 1C respectively.

mod options;
pub mod page;
mod routine;
pub mod tuple;

pub(super) const TQDISKANN_DEFAULT_GRAPH_DEGREE: i32 = 32;
pub(super) const TQDISKANN_MIN_GRAPH_DEGREE: i32 = 4;
pub(super) const TQDISKANN_MAX_GRAPH_DEGREE: i32 = 256;

pub(super) const TQDISKANN_DEFAULT_BUILD_LIST_SIZE: i32 = 100;
pub(super) const TQDISKANN_MIN_BUILD_LIST_SIZE: i32 = 10;
pub(super) const TQDISKANN_MAX_BUILD_LIST_SIZE: i32 = 1000;

pub(super) const TQDISKANN_DEFAULT_ALPHA: f32 = 1.2;
pub(super) const TQDISKANN_MIN_ALPHA: f32 = 1.0;
pub(super) const TQDISKANN_MAX_ALPHA: f32 = 2.0;

pub(super) const TQDISKANN_PLANNER_SCAN_ENABLED: bool = false;
