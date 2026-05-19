use pgrx::pg_sys;

use crate::{
    am::common::cost::{
        current_planner_cost_constants, estimate_planner_cost, gated_planner_cost_estimate,
        PlannerCostEstimate, PlannerCostInputs,
    },
    storage::{page::FIRST_DATA_BLOCK_NUMBER, relation_guard::IndexRelationGuard},
};

use super::{insert, options};

// DiskANN/Vamana is a single-layer graph, so the planner should model
// one graph-entry phase rather than the HNSW-style metadata-derived
// multilevel descent.
const DISKANN_SINGLE_LAYER_TREE_HEIGHT: f64 = 1.0;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexCostSnapshot {
    pub planner_scan_enabled: bool,
    pub planner_gate_reason: &'static str,
    pub dimensions: u16,
    pub graph_degree: i32,
    pub build_list_size: i32,
    pub relation_list_size: i32,
    pub session_list_size: Option<i32>,
    pub effective_list_size: i32,
    pub effective_list_size_source: &'static str,
    pub rerank_budget: i32,
    pub top_k: i32,
    pub alpha: f64,
    pub storage_format: &'static str,
    pub resolved_tree_height: f64,
    pub tree_height_source: &'static str,
    pub index_pages: f64,
    pub reltuples: f64,
    pub random_page_cost: f64,
    pub seq_page_cost: f64,
    pub cpu_operator_cost: f64,
    pub modeled_startup_cost: f64,
    pub modeled_total_cost: f64,
    pub modeled_selectivity: f64,
    pub modeled_correlation: f64,
}

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
    // SAFETY: PostgreSQL calls this access-method callback with planner-owned
    // output pointers. The guarded body rejects null inputs before reading the
    // IndexPath or writing the cost outputs.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if path.is_null()
                || index_startup_cost.is_null()
                || index_total_cost.is_null()
                || index_selectivity.is_null()
                || index_correlation.is_null()
                || index_pages.is_null()
            {
                pgrx::error!("ec_diskann planner callback received null arguments");
            }
            let index_info = (*path).indexinfo;
            if index_info.is_null() {
                pgrx::error!("ec_diskann planner callback received null index info");
            }
            let index_oid = (*index_info).indexoid;
            let index_relation = IndexRelationGuard::open(
                index_oid,
                pg_sys::NoLock as pg_sys::LOCKMODE,
                "ec_diskann planner",
            );
            let estimate = compute_amcostestimate(index_relation.as_ptr());

            *index_startup_cost = estimate.startup_cost;
            *index_total_cost = estimate.total_cost;
            *index_selectivity = estimate.selectivity;
            *index_correlation = estimate.correlation;
            *index_pages = estimate.index_pages;
        })
    }
}

unsafe fn compute_amcostestimate(index_relation: pg_sys::Relation) -> PlannerCostEstimate {
    // SAFETY: `index_relation` is a live relation pointer owned by the planner
    // callback guard for the duration of this computation.
    let relation_options = unsafe { options::relation_options(index_relation) };
    let scan_tuning = options::resolve_scan_tuning(&relation_options);
    // SAFETY: `index_relation` is live while the planner callback computes cost.
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let index_pages = f64::from(block_count);
    if block_count <= FIRST_DATA_BLOCK_NUMBER {
        return gated_planner_cost_estimate(index_pages);
    }

    // SAFETY: PostgreSQL relation metadata is valid for an opened index relation.
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    // A metadata decode failure means the index itself is structurally
    // broken. Failing loudly during planning is preferable to masking
    // corruption behind a gated cost and continuing with an invalid AM
    // state.
    // SAFETY: Metadata is read from the live index relation opened by the
    // planner callback guard.
    let metadata = unsafe { insert::read_metadata_page(index_relation) }
        .unwrap_or_else(|e| pgrx::error!("ec_diskann planner could not read metadata: {e}"));
    // SAFETY: Reads PostgreSQL planner cost GUCs through backend-local state.
    let constants = unsafe { current_planner_cost_constants() };

    estimate_planner_cost(
        PlannerCostInputs {
            index_pages,
            reltuples,
            m: relation_options.graph_degree,
            ef_search: scan_tuning.effective_list_size,
            dimensions: metadata.dimensions,
            tree_height: DISKANN_SINGLE_LAYER_TREE_HEIGHT,
        },
        constants,
    )
}

pub(crate) unsafe fn index_cost_snapshot(index_relation: pg_sys::Relation) -> IndexCostSnapshot {
    // SAFETY: `index_relation` is a live relation pointer supplied by a SQL
    // diagnostic wrapper.
    let relation_options = unsafe { options::relation_options(index_relation) };
    let scan_tuning = options::resolve_scan_tuning(&relation_options);
    // SAFETY: `index_relation` is live for the duration of the snapshot read.
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let index_pages = f64::from(block_count);
    // SAFETY: PostgreSQL relation metadata is valid for an opened index relation.
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    // SAFETY: Reads PostgreSQL planner cost GUCs through backend-local state.
    let constants = unsafe { current_planner_cost_constants() };

    let (planner_scan_enabled, planner_gate_reason, dimensions, estimate) =
        if block_count <= FIRST_DATA_BLOCK_NUMBER {
            (
                false,
                "planner scan selection is gated: ec_diskann index has no data pages",
                0,
                gated_planner_cost_estimate(index_pages),
            )
        } else {
            // SAFETY: The diagnostic caller supplies a live index relation, and
            // this branch only reads metadata after confirming data pages exist.
            let metadata = unsafe { insert::read_metadata_page(index_relation) }
                .unwrap_or_else(|e| pgrx::error!("ec_diskann cost snapshot failed: {e}"));
            let estimate = estimate_planner_cost(
                PlannerCostInputs {
                    index_pages,
                    reltuples,
                    m: relation_options.graph_degree,
                    ef_search: scan_tuning.effective_list_size,
                    dimensions: metadata.dimensions,
                    tree_height: DISKANN_SINGLE_LAYER_TREE_HEIGHT,
                },
                constants,
            );
            (
                true,
                "planner scan selection is live: ec_diskann cost model active",
                metadata.dimensions,
                estimate,
            )
        };

    IndexCostSnapshot {
        planner_scan_enabled,
        planner_gate_reason,
        dimensions,
        graph_degree: relation_options.graph_degree,
        build_list_size: relation_options.build_list_size,
        relation_list_size: scan_tuning.relation_list_size,
        session_list_size: scan_tuning.session_list_size,
        effective_list_size: scan_tuning.effective_list_size,
        effective_list_size_source: match scan_tuning.source {
            options::ListSizeSource::Relation => "relation",
            options::ListSizeSource::Session => "session",
        },
        rerank_budget: relation_options.rerank_budget,
        top_k: relation_options.top_k,
        alpha: f64::from(relation_options.alpha),
        storage_format: options::storage_format_name(relation_options.storage_format),
        resolved_tree_height: DISKANN_SINGLE_LAYER_TREE_HEIGHT,
        tree_height_source: "single_layer_diskann",
        index_pages,
        reltuples,
        random_page_cost: constants.random_page_cost,
        seq_page_cost: constants.seq_page_cost,
        cpu_operator_cost: constants.cpu_operator_cost,
        modeled_startup_cost: estimate.startup_cost,
        modeled_total_cost: estimate.total_cost,
        modeled_selectivity: estimate.selectivity,
        modeled_correlation: estimate.correlation,
    }
}
