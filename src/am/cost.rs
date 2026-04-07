use pgrx::pg_sys;

use super::TQHNSW_PLANNER_SCAN_ENABLED;

pub(super) unsafe extern "C-unwind" fn tqhnsw_amcostestimate(
    _root: *mut pg_sys::PlannerInfo,
    _path: *mut pg_sys::IndexPath,
    _loop_count: f64,
    index_startup_cost: *mut pg_sys::Cost,
    index_total_cost: *mut pg_sys::Cost,
    index_selectivity: *mut pg_sys::Selectivity,
    index_correlation: *mut f64,
    index_pages: *mut f64,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            // Prefer explicit non-selection over accidental planner use until the
            // ordered execution contract is credible.
            if TQHNSW_PLANNER_SCAN_ENABLED {
                pgrx::error!("tqhnsw planner costing is not implemented for enabled planner scans");
            }
            *index_startup_cost = f64::MAX;
            *index_total_cost = f64::MAX;
            *index_selectivity = 0.0;
            *index_correlation = 0.0;
            *index_pages = 0.0;
        })
    }
}
