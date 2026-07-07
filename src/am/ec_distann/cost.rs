use pgrx::pg_sys;

use crate::{
    am::common::callback::pg_am_callback,
    am::common::cost::{
        current_planner_cost_constants, estimate_planner_cost, gated_planner_cost_estimate,
        PlannerCostInputs,
    },
    storage::{page::FIRST_DATA_BLOCK_NUMBER, relation_guard::IndexRelationGuard},
};

use super::{ambuild, options};

// One global Vamana graph: a single-layer entry phase, like ec_diskann.
const DISTANN_SINGLE_LAYER_TREE_HEIGHT: f64 = 1.0;

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
        let index_pages_value = f64::from(block_count);

        let estimate = if block_count <= FIRST_DATA_BLOCK_NUMBER {
            gated_planner_cost_estimate(index_pages_value)
        } else {
            let metadata = ambuild::read_metadata_from_index_handle(handle).unwrap_or_else(|e| {
                pgrx::error!("ec_distann planner could not read metadata: {e}")
            });
            if metadata.node_count == 0 {
                gated_planner_cost_estimate(index_pages_value)
            } else {
                let reltuples = crate::storage::relation::relation_reltuples_handle(handle);
                // SAFETY: AM cost estimation runs inside the planner callback
                // where backend-local planner cost globals are valid to read.
                let constants = unsafe { current_planner_cost_constants() };
                // Per-query work is BW x H expansions (NFR-019); model the
                // search breadth as that product.
                let ef_search = i32::try_from(
                    options::current_beam_width() * options::current_hop_rounds(),
                )
                .unwrap_or(i32::MAX);
                estimate_planner_cost(
                    PlannerCostInputs {
                        index_pages: index_pages_value,
                        reltuples,
                        m: i32::from(metadata.graph_degree_r),
                        ef_search,
                        dimensions: metadata.dimensions,
                        tree_height: DISTANN_SINGLE_LAYER_TREE_HEIGHT,
                    },
                    constants,
                )
            }
        };

        *index_startup_cost = estimate.startup_cost;
        *index_total_cost = estimate.total_cost;
        *index_selectivity = estimate.selectivity;
        *index_correlation = estimate.correlation;
        *index_pages = estimate.index_pages;
    })
}
