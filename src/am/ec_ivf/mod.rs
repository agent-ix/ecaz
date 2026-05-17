//! ec_ivf-specific access-method skeleton.

mod admin;
mod build;
mod cost;
mod insert;
mod options;
mod page;
mod quantizer;
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
pub(super) const EC_IVF_DEFAULT_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_IVF_MIN_PQ_GROUP_SIZE: i32 = 0;
pub(super) const EC_IVF_MAX_PQ_GROUP_SIZE: i32 = 32;
pub(super) const EC_IVF_DEFAULT_POSTING_SLACK_PERCENT: i32 = 0;
pub(super) const EC_IVF_MIN_POSTING_SLACK_PERCENT: i32 = 0;
pub(super) const EC_IVF_MAX_POSTING_SLACK_PERCENT: i32 = 1000;
pub(super) const P_NEW: pgrx::pg_sys::BlockNumber = u32::MAX;

pub(crate) fn register_gucs() {
    options::register_gucs();
}

fn not_implemented(callback: &str) -> ! {
    pgrx::error!("ec_ivf {callback} is not implemented yet")
}

pub(crate) use self::admin::{
    index_admin_snapshot, index_drift_snapshot, index_page_ownership, IndexAdminSnapshot,
    IndexDriftSnapshot, IndexPageOwnershipSnapshot,
};
pub(crate) use self::cost::{index_cost_snapshot, IndexCostSnapshot};
pub use self::page::{
    EC_IVF_BLOCK_REF_BLOCK_NUMBER_OFFSET, EC_IVF_BLOCK_REF_BYTES,
    EC_IVF_CENTROID_DIMENSIONS_OFFSET, EC_IVF_CENTROID_LIST_ID_OFFSET, EC_IVF_CENTROID_TAG_OFFSET,
    EC_IVF_CENTROID_VALUES_OFFSET, EC_IVF_INDEX_FORMAT_VERSION, EC_IVF_LIST_DIRECTORY_BYTES,
    EC_IVF_LIST_DIRECTORY_DEAD_COUNT_OFFSET, EC_IVF_LIST_DIRECTORY_HEAD_BLOCK_OFFSET,
    EC_IVF_LIST_DIRECTORY_INSERTED_SINCE_BUILD_OFFSET, EC_IVF_LIST_DIRECTORY_LIST_ID_OFFSET,
    EC_IVF_LIST_DIRECTORY_LIVE_COUNT_OFFSET, EC_IVF_LIST_DIRECTORY_TAG_OFFSET,
    EC_IVF_LIST_DIRECTORY_TAIL_BLOCK_OFFSET, EC_IVF_METADATA_BYTES,
    EC_IVF_METADATA_CENTROID_HEAD_OFFSET, EC_IVF_METADATA_DIMENSIONS_OFFSET,
    EC_IVF_METADATA_DIRECTORY_HEAD_OFFSET, EC_IVF_METADATA_FORMAT_VERSION_OFFSET,
    EC_IVF_METADATA_INSERTED_SINCE_BUILD_OFFSET, EC_IVF_METADATA_MAGIC,
    EC_IVF_METADATA_MAGIC_OFFSET, EC_IVF_METADATA_NLISTS_OFFSET, EC_IVF_METADATA_NPROBE_OFFSET,
    EC_IVF_METADATA_PQ_CODEBOOK_HEAD_OFFSET, EC_IVF_METADATA_PQ_GROUP_SIZE_OFFSET,
    EC_IVF_METADATA_RERANK_OFFSET, EC_IVF_METADATA_SEED_OFFSET,
    EC_IVF_METADATA_STORAGE_FORMAT_OFFSET, EC_IVF_METADATA_TOTAL_DEAD_TUPLES_OFFSET,
    EC_IVF_METADATA_TOTAL_LIVE_TUPLES_OFFSET, EC_IVF_METADATA_TRAINING_SAMPLE_ROWS_OFFSET,
    EC_IVF_METADATA_TRAINING_VERSION_OFFSET, EC_IVF_POSTING_FLAGS_OFFSET,
    EC_IVF_POSTING_GAMMA_OFFSET, EC_IVF_POSTING_HEAPTIDS_OFFSET,
    EC_IVF_POSTING_HEAPTID_COUNT_OFFSET, EC_IVF_POSTING_LIST_ID_OFFSET,
    EC_IVF_POSTING_PAYLOAD_OFFSET, EC_IVF_POSTING_RERANK_TID_OFFSET, EC_IVF_POSTING_TAG_OFFSET,
    EC_IVF_PQ_CODEBOOK_CENTROIDS_OFFSET, EC_IVF_PQ_CODEBOOK_GROUP_INDEX_OFFSET,
    EC_IVF_PQ_CODEBOOK_NEXT_TID_OFFSET, EC_IVF_PQ_CODEBOOK_TAG_OFFSET,
};
#[cfg(feature = "pg18")]
pub(crate) use self::scan::explain_counters_from_index_scan_state;

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::insert::debug_ec_ivf_validate_no_duplicate_heap_tid;

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::scan::{
    debug_ec_ivf_build_metadata, debug_ec_ivf_directory_entry, debug_ec_ivf_directory_summary,
    debug_ec_ivf_gettuple_after_rescan_result, debug_ec_ivf_gettuple_outputs,
    debug_ec_ivf_metadata, debug_ec_ivf_pq_fastscan_model_cache_reused,
    debug_ec_ivf_quantizer_cache_ptr, debug_ec_ivf_rerank_mode, debug_ec_ivf_rescan_query_prep,
};

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::vacuum::{debug_ec_ivf_vacuum_remove_heap_tids, debug_ec_ivf_vacuum_stats};
