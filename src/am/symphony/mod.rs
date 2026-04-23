//! SymphonyQG access-method scaffold.
//!
//! Phase 1 intentionally lands only the module shape and AM callback
//! surface. Build, insert, scan, page, graph, and vacuum behavior
//! follow in later slices.

mod build;
pub(crate) mod graph;
mod insert;
mod options;
pub(crate) mod page;
mod routine;
mod scan;
mod vacuum;

pub(super) const SYMPHONY_DEFAULT_M: u16 = 8;
pub(super) const SYMPHONY_MIN_M: i32 = 2;
pub(super) const SYMPHONY_MAX_M: i32 = 100;
pub(super) const SYMPHONY_DEFAULT_EF_CONSTRUCTION: u16 = 64;
pub(super) const SYMPHONY_MIN_EF_CONSTRUCTION: i32 = 10;
pub(super) const SYMPHONY_MAX_EF_CONSTRUCTION: i32 = 1000;
pub(super) const SYMPHONY_BOOTSTRAP_PADDING_FACTOR: u16 = 1;
pub(super) const SYMPHONY_MIN_PADDING_FACTOR: i32 = 1;
pub(super) const SYMPHONY_MAX_PADDING_FACTOR: i32 = 1024;
pub(super) const SYMPHONY_RABITQ_BITS: u8 = 1;

pub(crate) fn register_gucs() {}

pub(super) fn not_implemented(callback: &str) -> ! {
    pgrx::error!("symphony {callback} is not implemented yet")
}
