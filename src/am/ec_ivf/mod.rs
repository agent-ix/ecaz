//! ec_ivf-specific access-method skeleton.

mod build;
mod insert;
mod options;
mod page;
mod routine;
mod scan;
mod training;
mod vacuum;

pub(super) const EC_IVF_DEFAULT_NLISTS: i32 = 0;
pub(super) const EC_IVF_MIN_NLISTS: i32 = 0;
pub(super) const EC_IVF_MAX_NLISTS: i32 = 1_000_000;
pub(super) const EC_IVF_DEFAULT_NPROBE: i32 = 0;
pub(super) const EC_IVF_MIN_NPROBE: i32 = 0;
pub(super) const EC_IVF_MAX_NPROBE: i32 = 1_000_000;
pub(super) const EC_IVF_DEFAULT_TRAINING_SAMPLE_ROWS: i32 = 0;
pub(super) const EC_IVF_MIN_TRAINING_SAMPLE_ROWS: i32 = 0;
pub(super) const EC_IVF_MAX_TRAINING_SAMPLE_ROWS: i32 = 10_000_000;
pub(super) const EC_IVF_DEFAULT_SEED: i32 = 42;
pub(super) const EC_IVF_MIN_SEED: i32 = 0;
pub(super) const EC_IVF_MAX_SEED: i32 = i32::MAX;
pub(super) const P_NEW: pgrx::pg_sys::BlockNumber = u32::MAX;

pub(crate) fn register_gucs() {
    options::register_gucs();
}

fn not_implemented(callback: &str) -> ! {
    pgrx::error!("ec_ivf {callback} is not implemented yet")
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::scan::{debug_ec_ivf_gettuple_after_rescan_result, debug_ec_ivf_metadata};
