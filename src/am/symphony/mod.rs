//! SymphonyQG access-method scaffold.
//!
//! Phase 1 intentionally lands only the module shape and AM callback
//! surface. Build, insert, scan, page, graph, and vacuum behavior
//! follow in later slices.

mod build;
pub(crate) mod graph;
mod insert;
pub(crate) mod page;
mod routine;
mod scan;
mod vacuum;

pub(crate) fn register_gucs() {}

pub(super) fn not_implemented(callback: &str) -> ! {
    pgrx::error!("symphony {callback} is not implemented yet")
}
