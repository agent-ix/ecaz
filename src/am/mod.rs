//! Access-method scaffolding for the future `tqhnsw` implementation.

mod build;
mod cost;
mod explain;
mod graph;
mod insert;
mod options;
pub mod page;
mod routine;
mod scan;
mod scan_debug;
mod search;
mod shared;
mod stats;
mod vacuum;
pub mod wal;

pub(super) const TQHNSW_DEFAULT_M: i32 = 8;
pub(super) const TQHNSW_MIN_M: i32 = 2;
pub(super) const TQHNSW_MAX_M: i32 = 100;
pub(super) const TQHNSW_DEFAULT_EF_CONSTRUCTION: i32 = 64;
pub(super) const TQHNSW_MIN_EF_CONSTRUCTION: i32 = 10;
pub(super) const TQHNSW_MAX_EF_CONSTRUCTION: i32 = 1000;
pub(super) const TQHNSW_DEFAULT_EF_SEARCH: i32 = 40;
pub(super) const TQHNSW_MIN_EF_SEARCH: i32 = 1;
pub(super) const TQHNSW_MAX_EF_SEARCH: i32 = 1000;
pub(super) const TQHNSW_PLANNER_SCAN_ENABLED: bool = false;
pub(super) const P_NEW: pgrx::pg_sys::BlockNumber = u32::MAX;

pub(crate) fn register_gucs() {
    options::register_gucs();
}

pub(crate) unsafe fn index_admin_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> shared::IndexAdminSnapshot {
    unsafe { shared::index_admin_snapshot(index_relation) }
}

pub(crate) unsafe fn index_explain_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> shared::IndexExplainSnapshot {
    unsafe { shared::index_explain_snapshot(index_relation) }
}

pub(crate) unsafe fn index_cost_snapshot(
    index_relation: pgrx::pg_sys::Relation,
) -> shared::IndexCostSnapshot {
    unsafe { shared::index_cost_snapshot(index_relation) }
}

pub(crate) fn stats_snapshot() -> stats::StatsSnapshot {
    stats::stats_snapshot()
}

pub(crate) fn pg18_upgrade_snapshot() -> shared::Pg18UpgradeSnapshot {
    shared::pg18_upgrade_snapshot()
}

pub(crate) fn pg18_diagnostics_snapshot() -> shared::Pg18DiagnosticsSnapshot {
    shared::pg18_diagnostics_snapshot()
}

#[cfg(any(test, feature = "pg_test"))]
#[allow(unused_imports)]
pub(crate) use self::shared::{
    debug_index_metadata, debug_index_pages, debug_planner_tuning_snapshot, debug_vacuum_stats,
    DebugIndexDataPage, DebugPlannerTuningSnapshot,
};

#[cfg(any(test, feature = "pg_test"))]
#[allow(unused_imports)]
pub(crate) use self::scan_debug::{
    debug_begin_end_scan, debug_candidate_frontier_head_lifecycle,
    debug_consume_candidate_frontier_head, debug_consume_candidate_frontier_head_slots,
    debug_end_scan_twice, debug_entry_candidate_lifecycle, debug_entry_point_neighbor_tids,
    debug_gettuple_after_rescan_result, debug_gettuple_backward_after_rescan,
    debug_gettuple_consumes_bootstrap_candidate, debug_gettuple_current_result_heap_progress,
    debug_gettuple_current_result_lifecycle, debug_gettuple_current_result_neighbors,
    debug_gettuple_current_result_state, debug_gettuple_exhaustion_state,
    debug_gettuple_orderby_score, debug_gettuple_orderby_score_lifecycle,
    debug_gettuple_rescan_after_exhaustion, debug_gettuple_rescan_after_partial,
    debug_gettuple_scan_heap_tids, debug_gettuple_without_rescan,
    debug_bootstrap_phase_transition,
    debug_materialize_bootstrap_candidate_result, debug_rescan_candidate_frontier,
    debug_rescan_entry_candidate_state, debug_rescan_null_query,
    debug_rescan_overwrites_query_dimensions, debug_rescan_query_dimensions,
    debug_rescan_successor_candidate_state, debug_rescan_with_index_qual,
    debug_rescan_with_multiple_orderbys, debug_visited_seed_lifecycle,
};
