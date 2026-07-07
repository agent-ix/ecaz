use pgrx::pg_sys;

use crate::{
    am::common::callback::pg_am_callback,
    am::common::cost::gated_planner_cost_estimate,
    storage::relation_guard::IndexRelationGuard,
};

pub(super) unsafe extern "C-unwind" fn ec_distann_amcostestimate(
    _root: *mut pg_sys::PlannerInfo,
    path: *mut pg_sys::IndexPath,
    _loop_count: f64,
    index_startup_cost: *mut pg_sys::Cost,
    index_total_cost: *mut pg_sys::Cost,
    index_selectivity: *mut pg_sys::Selectivity,
    index_correlation: *mut f64,
    index_pages: *mut f64,
) {
    // SAFETY: PostgreSQL calls this access-method callback with planner-owned
    // output pointers. The guarded body rejects null inputs before reading the
    // IndexPath or writing the cost outputs.
    pg_am_callback!({
        if path.is_null()
            || index_startup_cost.is_null()
            || index_total_cost.is_null()
            || index_selectivity.is_null()
            || index_correlation.is_null()
            || index_pages.is_null()
        {
            pgrx::error!("ec_distann planner callback received null arguments");
        }
        let index_info = (*path).indexinfo;
        if index_info.is_null() {
            pgrx::error!("ec_distann planner callback received null index info");
        }
        let index_oid = (*index_info).indexoid;
        let index_relation = IndexRelationGuard::open(
            index_oid,
            pg_sys::NoLock as pg_sys::LOCKMODE,
            "ec_distann planner",
        );
        let handle = std::ptr::NonNull::new(index_relation.as_ptr())
            .unwrap_or_else(|| pgrx::error!("ec_distann planner opened a null index relation"));
        let block_count = crate::storage::relation::main_fork_block_count_handle(handle);

        // The FR-081 scan path has not landed yet, so every plan is gated
        // (prohibitively costed) regardless of index contents. The live cost
        // model arrives with the local hop-round loop slice.
        let estimate = gated_planner_cost_estimate(f64::from(block_count));

        *index_startup_cost = estimate.startup_cost;
        *index_total_cost = estimate.total_cost;
        *index_selectivity = estimate.selectivity;
        *index_correlation = estimate.correlation;
        *index_pages = estimate.index_pages;
    })
}
