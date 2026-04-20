use pgrx::pg_sys;

use crate::{
    am::common::cost::{
        current_planner_cost_constants, estimate_planner_cost, gated_planner_cost_estimate,
        PlannerCostEstimate, PlannerCostInputs,
    },
    storage::page::FIRST_DATA_BLOCK_NUMBER,
};

use super::{insert, options};

const DISKANN_SINGLE_LAYER_TREE_HEIGHT: f64 = 1.0;

pub(super) unsafe extern "C-unwind" fn ec_diskann_amcostestimate(
    _root: *mut pg_sys::PlannerInfo,
    path: *mut pg_sys::IndexPath,
    _loop_count: f64,
    index_startup_cost: *mut pg_sys::Cost,
    index_total_cost: *mut pg_sys::Cost,
    index_selectivity: *mut pg_sys::Selectivity,
    index_correlation: *mut f64,
    index_pages: *mut f64,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let index_info = (*path).indexinfo;
            let index_oid = (*index_info).indexoid;
            let index_relation = pg_sys::index_open(index_oid, pg_sys::NoLock as pg_sys::LOCKMODE);
            let estimate = compute_amcostestimate(index_relation);
            pg_sys::index_close(index_relation, pg_sys::NoLock as pg_sys::LOCKMODE);

            *index_startup_cost = estimate.startup_cost;
            *index_total_cost = estimate.total_cost;
            *index_selectivity = estimate.selectivity;
            *index_correlation = estimate.correlation;
            *index_pages = estimate.index_pages;
        })
    }
}

unsafe fn compute_amcostestimate(index_relation: pg_sys::Relation) -> PlannerCostEstimate {
    let relation_options = unsafe { options::relation_options(index_relation) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let index_pages = f64::from(block_count);
    if block_count <= FIRST_DATA_BLOCK_NUMBER {
        return gated_planner_cost_estimate(index_pages);
    }

    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    let metadata = unsafe { insert::read_metadata_page(index_relation) }
        .unwrap_or_else(|e| pgrx::error!("ec_diskann planner could not read metadata: {e}"));
    let constants = unsafe { current_planner_cost_constants() };

    estimate_planner_cost(
        PlannerCostInputs {
            index_pages,
            reltuples,
            m: relation_options.graph_degree,
            ef_search: relation_options.list_size,
            dimensions: metadata.dimensions,
            tree_height: DISKANN_SINGLE_LAYER_TREE_HEIGHT,
        },
        constants,
    )
}
