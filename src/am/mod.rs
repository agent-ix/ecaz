//! Access-method scaffolding for the future `tqhnsw` implementation.

mod build;
mod cost;
mod explain;
pub(crate) mod graph;
mod insert;
mod options;
pub mod page;
mod routine;
mod scan;
mod scan_debug;
mod search;
mod shared;
mod source;
mod stats;
mod stream;
mod vacuum;

pub(super) const TQHNSW_DEFAULT_M: i32 = 8;
pub(super) const TQHNSW_MIN_M: i32 = 2;
pub(super) const TQHNSW_MAX_M: i32 = 100;
pub(super) const TQHNSW_DEFAULT_EF_CONSTRUCTION: i32 = 64;
pub(super) const TQHNSW_MIN_EF_CONSTRUCTION: i32 = 10;
pub(super) const TQHNSW_MAX_EF_CONSTRUCTION: i32 = 1000;
pub(super) const TQHNSW_DEFAULT_EF_SEARCH: i32 = 40;
pub(super) const TQHNSW_MIN_EF_SEARCH: i32 = 1;
pub(super) const TQHNSW_MAX_EF_SEARCH: i32 = 1000;
pub(super) const TQHNSW_PLANNER_SCAN_ENABLED: bool = true;
pub(super) const P_NEW: pgrx::pg_sys::BlockNumber = u32::MAX;

pub(crate) fn register_gucs() {
    options::register_gucs();
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::scan::{
    resolve_pq_fastscan_rerank_mode_decision, resolve_pq_fastscan_traversal_score_mode_decision,
    PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW, PQ_FASTSCAN_DEFAULT_RERANK_MODE_NAME,
    PQ_FASTSCAN_DEFAULT_TRAVERSAL_SCORE_MODE_NAME,
};

pub(crate) unsafe fn index_cost_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> shared::IndexCostSnapshot {
    unsafe { shared::index_cost_snapshot(index_relation) }
}

pub(crate) unsafe fn index_admin_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> shared::IndexAdminSnapshot {
    unsafe { shared::index_admin_snapshot(index_relation) }
}

pub(crate) unsafe fn planner_integration_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> shared::PlannerIntegrationSnapshot {
    unsafe { shared::planner_integration_snapshot(index_relation) }
}

#[cfg(any(test, feature = "pg_test"))]
#[allow(unused_imports)]
pub(crate) use self::shared::{
    debug_index_metadata, debug_index_pages, debug_planner_tuning_snapshot,
    debug_update_index_metadata, debug_vacuum_stats, DebugIndexDataPage,
    DebugPlannerTuningSnapshot,
};

#[cfg(any(test, feature = "pg_test"))]
#[allow(unused_imports)]
pub(crate) use self::insert::debug_insert_level_for_heap_tid;

#[cfg(any(test, feature = "pg_test"))]
#[allow(unused_imports)]
pub(crate) use self::vacuum::debug_vacuum_remove_heap_tids;

#[cfg(any(test, feature = "pg_test"))]
#[allow(unused_imports)]
pub(crate) use self::scan_debug::{
    debug_all_top_level_heap_tids, debug_begin_end_scan, debug_bootstrap_phase_transition,
    debug_candidate_frontier_head_lifecycle, debug_consume_candidate_frontier_head,
    debug_consume_candidate_frontier_head_slots, debug_end_scan_twice,
    debug_entry_candidate_lifecycle, debug_entry_point_neighbor_tids,
    debug_exact_seed_scan_heap_tids, debug_gettuple_after_rescan_result,
    debug_gettuple_backward_after_rescan, debug_gettuple_consumes_bootstrap_candidate,
    debug_gettuple_current_result_heap_progress, debug_gettuple_current_result_lifecycle,
    debug_gettuple_current_result_neighbors, debug_gettuple_current_result_state,
    debug_gettuple_exhaustion_state, debug_gettuple_orderby_score,
    debug_gettuple_orderby_score_lifecycle, debug_gettuple_rescan_after_exhaustion,
    debug_gettuple_rescan_after_partial, debug_gettuple_scan_heap_tids,
    debug_gettuple_scan_heap_tids_with_score_comparisons,
    debug_gettuple_scan_heap_tids_with_scores, debug_gettuple_without_rescan,
    debug_grouped_rerank_profile, debug_grouped_scan_comparison_rows,
    debug_grouped_scan_comparison_summary, debug_grouped_scan_order_drift_summary,
    debug_grouped_scan_windowed_rows, debug_grouped_scan_windowed_summary,
    debug_layer0_reachable_live_element_tids, debug_layer_oracle_k_carrydown_scan_heap_tids,
    debug_layer_oracle_k_seed_layer0_neighbor_heap_tids,
    debug_materialize_bootstrap_candidate_result, debug_profile_ordered_scan,
    debug_profile_ordered_scan_with_heap_fetch, debug_profile_ordered_scan_with_limit,
    debug_rescan_candidate_frontier, debug_rescan_entry_candidate_state, debug_rescan_null_query,
    debug_rescan_overwrites_query_dimensions, debug_rescan_query_dimensions,
    debug_rescan_successor_candidate_state, debug_rescan_with_index_qual,
    debug_rescan_with_multiple_orderbys, debug_rescan_with_unused_key_buffer,
    debug_top_level_oracle_k_seed_heap_tids, debug_top_level_oracle_k_seed_scan_heap_tids,
    debug_top_level_oracle_scan_heap_tids, debug_top_level_reachable_heap_tids,
    debug_turboquant_scan_stage_profile, debug_visited_seed_lifecycle,
};
