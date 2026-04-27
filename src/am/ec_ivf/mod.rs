//! ec_ivf-specific access-method skeleton.

mod admin;
mod build;
mod cost;
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
pub(super) const EC_IVF_DEFAULT_RERANK_WIDTH: i32 = 0;
pub(super) const EC_IVF_MIN_RERANK_WIDTH: i32 = 0;
pub(super) const EC_IVF_MAX_RERANK_WIDTH: i32 = 10_000_000;
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

pub(crate) use self::admin::{
    index_admin_snapshot, index_drift_snapshot, IndexAdminSnapshot, IndexDriftSnapshot,
};
pub(crate) use self::cost::{index_cost_snapshot, IndexCostSnapshot};
pub(crate) use self::scan::explain_counters_from_index_scan_state;

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::insert::debug_ec_ivf_validate_no_duplicate_heap_tid;

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::scan::{
    debug_ec_ivf_build_metadata, debug_ec_ivf_directory_entry, debug_ec_ivf_directory_summary,
    debug_ec_ivf_gettuple_after_rescan_result, debug_ec_ivf_gettuple_outputs,
    debug_ec_ivf_metadata, debug_ec_ivf_rerank_mode, debug_ec_ivf_rescan_query_prep,
};

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::vacuum::{debug_ec_ivf_vacuum_remove_heap_tids, debug_ec_ivf_vacuum_stats};
