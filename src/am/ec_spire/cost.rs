use pgrx::pg_sys;

use super::{
    active_snapshot_diagnostics, index_hierarchy_snapshot, options, SpireActiveSnapshotDiagnostics,
    SpireIndexHierarchySnapshot,
};
use crate::am::common::cost::{
    self, current_planner_cost_constants, PlannerCostConstants, PlannerCostEstimate,
};

const SPIRE_ROUTING_SCORING_DIMENSION_SCALE: f64 = 0.01;
const SPIRE_LEAF_SCORING_DIMENSION_SCALE: f64 = 0.01;
const SPIRE_INDEX_PAGE_COST_SCALE: f64 = 1.0;
const SPIRE_LOCAL_STORE_PAGE_FANOUT_SCALE: f64 = 0.05;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireIndexCostSnapshot {
    pub(crate) planner_scan_enabled: bool,
    pub(crate) planner_gate_reason: &'static str,
    pub(crate) dimensions: u16,
    pub(crate) nlists: u32,
    pub(crate) active_leaf_count: u32,
    pub(crate) relation_nprobe: u32,
    pub(crate) session_nprobe: Option<u32>,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) local_store_count: u32,
    pub(crate) recursive_fanout: Option<u32>,
    pub(crate) resolved_tree_height: f64,
    pub(crate) tree_height_source: &'static str,
    pub(crate) pg18_tree_height_callback_ready: bool,
    pub(crate) average_leaf_live_count: f64,
    pub(crate) estimated_routing_scores: u64,
    pub(crate) estimated_selected_leaves: u32,
    pub(crate) estimated_candidate_rows: f64,
    pub(crate) estimated_routing_pages: f64,
    pub(crate) estimated_leaf_pages: f64,
    pub(crate) storage_format: &'static str,
    pub(crate) relation_rerank_width: i32,
    pub(crate) session_rerank_width: Option<i32>,
    pub(crate) effective_rerank_width: i32,
    pub(crate) effective_rerank_width_source: &'static str,
    pub(crate) index_pages: f64,
    pub(crate) reltuples: f64,
    pub(crate) modeled_startup_cost: f64,
    pub(crate) modeled_total_cost: f64,
    pub(crate) modeled_selectivity: f64,
    pub(crate) modeled_correlation: f64,
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amcostestimate(
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

pub(crate) unsafe fn index_cost_snapshot(
    index_relation: pg_sys::Relation,
) -> SpireIndexCostSnapshot {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let index_pages = f64::from(block_count);
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    let constants = unsafe { current_planner_cost_constants() };
    let relation_options = unsafe { options::relation_options(index_relation) };
    let diagnostics = unsafe { active_snapshot_diagnostics(index_relation) };
    let hierarchy = unsafe { index_hierarchy_snapshot(index_relation) };
    let inputs = SpireCostInputs::from_snapshots(
        &relation_options,
        &diagnostics,
        &hierarchy,
        index_pages,
        reltuples,
    );
    let details = estimate_spire_details(&inputs);
    let estimate = estimate_spire_cost(&inputs, constants);

    SpireIndexCostSnapshot {
        planner_scan_enabled: true,
        planner_gate_reason: "planner scan selection is live: Phase 8 SPIRE cost model active",
        dimensions: inputs.dimensions,
        nlists: inputs.nlists,
        active_leaf_count: active_leaf_count(&inputs),
        relation_nprobe: inputs.relation_nprobe,
        session_nprobe: inputs.session_nprobe,
        effective_nprobe: inputs.effective_nprobe,
        effective_nprobe_source: inputs.effective_nprobe_source,
        local_store_count: inputs.local_store_count,
        recursive_fanout: inputs.recursive_fanout,
        resolved_tree_height: f64::from(spire_tree_height_callback_value(index_relation)),
        tree_height_source: if cfg!(feature = "pg18") {
            "amgettreeheight_callback"
        } else {
            "hierarchy_snapshot"
        },
        pg18_tree_height_callback_ready: cfg!(feature = "pg18"),
        average_leaf_live_count: details.average_leaf_live_count,
        estimated_routing_scores: details.estimated_routing_scores,
        estimated_selected_leaves: details.estimated_selected_leaves,
        estimated_candidate_rows: details.estimated_candidate_rows,
        estimated_routing_pages: details.estimated_routing_pages,
        estimated_leaf_pages: details.estimated_leaf_pages,
        storage_format: inputs.storage_format.reloption_name(),
        relation_rerank_width: inputs.relation_rerank_width,
        session_rerank_width: inputs.session_rerank_width,
        effective_rerank_width: inputs.effective_rerank_width,
        effective_rerank_width_source: inputs.effective_rerank_width_source,
        index_pages: inputs.index_pages,
        reltuples: inputs.reltuples,
        modeled_startup_cost: estimate.startup_cost,
        modeled_total_cost: estimate.total_cost,
        modeled_selectivity: estimate.selectivity,
        modeled_correlation: estimate.correlation,
    }
}

unsafe fn compute_amcostestimate(index_relation: pg_sys::Relation) -> PlannerCostEstimate {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let index_pages = f64::from(block_count);
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    let constants = unsafe { current_planner_cost_constants() };
    let relation_options = unsafe { options::relation_options(index_relation) };
    let diagnostics = unsafe { active_snapshot_diagnostics(index_relation) };
    let hierarchy = unsafe { index_hierarchy_snapshot(index_relation) };
    let inputs = SpireCostInputs::from_snapshots(
        &relation_options,
        &diagnostics,
        &hierarchy,
        index_pages,
        reltuples,
    );

    estimate_spire_cost(&inputs, constants)
}

fn spire_tree_height_callback_value(index_relation: pg_sys::Relation) -> i32 {
    let hierarchy = unsafe { index_hierarchy_snapshot(index_relation) };
    i32::from(hierarchy.hierarchy_depth)
}

#[cfg(feature = "pg18")]
pub(super) unsafe extern "C-unwind" fn ec_spire_amgettreeheight(rel: pg_sys::Relation) -> i32 {
    unsafe { pgrx::pgrx_extern_c_guard(|| spire_tree_height_callback_value(rel)) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpireCostInputs {
    dimensions: u16,
    nlists: u32,
    relation_nprobe: u32,
    session_nprobe: Option<u32>,
    effective_nprobe: u32,
    effective_nprobe_source: &'static str,
    local_store_count: u32,
    recursive_fanout: Option<u32>,
    relation_rerank_width: i32,
    session_rerank_width: Option<i32>,
    effective_rerank_width: i32,
    effective_rerank_width_source: &'static str,
    storage_format: options::SpireStorageFormat,
    hierarchy_depth: u16,
    routing_object_count: u64,
    routing_child_count: u64,
    root_child_count: u64,
    leaf_object_count: u64,
    leaf_assignment_count: u64,
    routing_object_bytes: u64,
    leaf_object_bytes: u64,
    index_pages: f64,
    reltuples: f64,
}

impl SpireCostInputs {
    fn from_snapshots(
        relation_options: &options::EcSpireOptions,
        diagnostics: &SpireActiveSnapshotDiagnostics,
        hierarchy: &SpireIndexHierarchySnapshot,
        index_pages: f64,
        reltuples: f64,
    ) -> Self {
        let relation_nlists = u32::try_from(relation_options.nlists).unwrap_or(0);
        let active_leaf_count = u32::try_from(hierarchy.leaf_object_count).unwrap_or(u32::MAX);
        let probe_leaf_count = if active_leaf_count == 0 {
            relation_nlists
        } else {
            active_leaf_count
        };
        let relation_nprobe = u32::try_from(relation_options.nprobe).unwrap_or(0);
        let nprobe = options::resolve_scan_nprobe(probe_leaf_count, relation_nprobe);
        let rerank_width = options::resolve_scan_rerank_width(relation_options.rerank_width);

        Self {
            dimensions: hierarchy.centroid_dimensions,
            nlists: relation_nlists,
            relation_nprobe: nprobe.relation_nprobe,
            session_nprobe: nprobe.session_nprobe,
            effective_nprobe: nprobe.effective_nprobe,
            effective_nprobe_source: nprobe.source,
            local_store_count: u32::try_from(relation_options.local_store_count).unwrap_or(1),
            recursive_fanout: relation_options.recursive_fanout(),
            relation_rerank_width: rerank_width.relation_rerank_width,
            session_rerank_width: rerank_width.session_rerank_width,
            effective_rerank_width: rerank_width.effective_rerank_width,
            effective_rerank_width_source: rerank_width.source,
            storage_format: relation_options.storage_format,
            hierarchy_depth: hierarchy.hierarchy_depth,
            routing_object_count: hierarchy.routing_object_count,
            routing_child_count: diagnostics.routing_child_count,
            root_child_count: hierarchy.root_child_count,
            leaf_object_count: hierarchy.leaf_object_count,
            leaf_assignment_count: diagnostics.leaf_assignment_count,
            routing_object_bytes: diagnostics.routing_object_bytes,
            leaf_object_bytes: diagnostics.leaf_object_bytes,
            index_pages,
            reltuples,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpireCostDetails {
    average_leaf_live_count: f64,
    estimated_routing_scores: u64,
    estimated_selected_leaves: u32,
    estimated_candidate_rows: f64,
    estimated_routing_pages: f64,
    estimated_leaf_pages: f64,
}

fn estimate_spire_cost(
    inputs: &SpireCostInputs,
    constants: PlannerCostConstants,
) -> PlannerCostEstimate {
    let details = estimate_spire_details(inputs);
    let routing_scoring_dimensions =
        f64::from(inputs.dimensions) * SPIRE_ROUTING_SCORING_DIMENSION_SCALE;
    let leaf_scoring_dimensions = f64::from(inputs.dimensions) * SPIRE_LEAF_SCORING_DIMENSION_SCALE;
    let routing_cpu = details.estimated_routing_scores as f64
        * constants.cpu_operator_cost
        * routing_scoring_dimensions;
    let candidate_cpu = details.estimated_candidate_rows
        * constants.cpu_operator_cost
        * leaf_scoring_dimensions
        * storage_scoring_multiplier(inputs.storage_format)
        * rerank_multiplier(inputs.effective_rerank_width);
    let index_page_cost = constants.seq_page_cost
        * SPIRE_INDEX_PAGE_COST_SCALE
        * local_store_page_multiplier(inputs.local_store_count);
    let routing_page_cost = details.estimated_routing_pages * index_page_cost;
    let leaf_page_cost = details.estimated_leaf_pages * index_page_cost;
    let metadata_page_cost = if inputs.index_pages > 0.0 {
        constants.random_page_cost
    } else {
        0.0
    };
    let store_coordination_cpu =
        f64::from(inputs.local_store_count.max(1)) * constants.cpu_operator_cost;
    let startup_cost =
        metadata_page_cost + routing_page_cost + routing_cpu + store_coordination_cpu;

    PlannerCostEstimate {
        startup_cost,
        total_cost: startup_cost + leaf_page_cost + candidate_cpu,
        selectivity: 1.0,
        correlation: 0.0,
        index_pages: inputs.index_pages,
    }
}

fn estimate_spire_details(inputs: &SpireCostInputs) -> SpireCostDetails {
    let leaf_count = active_leaf_count(inputs);
    let total_live = if inputs.leaf_assignment_count > 0 {
        inputs.leaf_assignment_count as f64
    } else {
        inputs.reltuples.max(0.0)
    };
    let average_leaf_live_count = if leaf_count == 0 {
        0.0
    } else {
        total_live / f64::from(leaf_count)
    };
    let estimated_selected_leaves = inputs.effective_nprobe.clamp(0, leaf_count);
    let estimated_candidate_rows =
        (average_leaf_live_count * f64::from(estimated_selected_leaves)).min(total_live);
    let leaf_fraction = if leaf_count == 0 {
        0.0
    } else {
        f64::from(estimated_selected_leaves) / f64::from(leaf_count)
    };
    let total_data_pages = (inputs.index_pages - 1.0).max(0.0);
    let estimated_routing_pages = bytes_to_pages(inputs.routing_object_bytes).min(total_data_pages);
    let available_leaf_pages = if inputs.leaf_object_bytes > 0 {
        bytes_to_pages(inputs.leaf_object_bytes)
    } else {
        (total_data_pages - estimated_routing_pages).max(0.0)
    };
    let estimated_leaf_pages = (available_leaf_pages * leaf_fraction).min(available_leaf_pages);

    SpireCostDetails {
        average_leaf_live_count,
        estimated_routing_scores: estimated_routing_scores(inputs, leaf_count),
        estimated_selected_leaves,
        estimated_candidate_rows,
        estimated_routing_pages,
        estimated_leaf_pages,
    }
}

fn active_leaf_count(inputs: &SpireCostInputs) -> u32 {
    let active_leaf_count = u32::try_from(inputs.leaf_object_count).unwrap_or(u32::MAX);
    if active_leaf_count == 0 {
        inputs.nlists
    } else {
        active_leaf_count
    }
}

fn estimated_routing_scores(inputs: &SpireCostInputs, leaf_count: u32) -> u64 {
    if leaf_count == 0 {
        return 0;
    }

    let root_child_count = inputs.root_child_count.max(u64::from(leaf_count));
    if inputs.hierarchy_depth <= 1 || inputs.routing_object_count == 0 {
        return root_child_count;
    }

    let fanout = u64::from(inputs.recursive_fanout.unwrap_or(leaf_count).max(1));
    let lower_level_count = u64::from(inputs.hierarchy_depth.saturating_sub(1));
    let selected_branch_count = u64::from(inputs.effective_nprobe.max(1));
    let modeled_scores = root_child_count.saturating_add(
        selected_branch_count
            .saturating_mul(fanout)
            .saturating_mul(lower_level_count),
    );
    let routing_child_bound = inputs.routing_child_count.max(root_child_count);

    modeled_scores.min(routing_child_bound)
}

fn bytes_to_pages(bytes: u64) -> f64 {
    if bytes == 0 {
        0.0
    } else {
        (bytes as f64 / pg_sys::BLCKSZ as f64).ceil()
    }
}

fn local_store_page_multiplier(local_store_count: u32) -> f64 {
    1.0 + f64::from(local_store_count.saturating_sub(1)) * SPIRE_LOCAL_STORE_PAGE_FANOUT_SCALE
}

fn storage_scoring_multiplier(storage_format: options::SpireStorageFormat) -> f64 {
    match storage_format {
        options::SpireStorageFormat::Auto | options::SpireStorageFormat::TurboQuant => 1.0,
        options::SpireStorageFormat::PqFastScan => 0.65,
        options::SpireStorageFormat::RaBitQ => 0.45,
    }
}

fn rerank_multiplier(rerank_width: i32) -> f64 {
    if rerank_width == 0 {
        1.35
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(
        effective_nprobe: u32,
        hierarchy_depth: u16,
        local_store_count: u32,
    ) -> SpireCostInputs {
        SpireCostInputs {
            dimensions: 1536,
            nlists: 128,
            relation_nprobe: effective_nprobe,
            session_nprobe: None,
            effective_nprobe,
            effective_nprobe_source: "test",
            local_store_count,
            recursive_fanout: Some(16),
            relation_rerank_width: 100,
            session_rerank_width: None,
            effective_rerank_width: 100,
            effective_rerank_width_source: "test",
            storage_format: options::SpireStorageFormat::TurboQuant,
            hierarchy_depth,
            routing_object_count: if hierarchy_depth > 1 { 8 } else { 1 },
            routing_child_count: 4_096,
            root_child_count: 128,
            leaf_object_count: 128,
            leaf_assignment_count: 12_800,
            routing_object_bytes: 8 * pg_sys::BLCKSZ as u64,
            leaf_object_bytes: 512 * pg_sys::BLCKSZ as u64,
            index_pages: 600.0,
            reltuples: 12_800.0,
        }
    }

    fn default_constants() -> PlannerCostConstants {
        PlannerCostConstants {
            random_page_cost: 4.0,
            seq_page_cost: 1.0,
            cpu_operator_cost: 0.0025,
        }
    }

    #[test]
    fn cost_increases_with_effective_nprobe() {
        let low = estimate_spire_cost(&inputs(4, 2, 1), default_constants());
        let high = estimate_spire_cost(&inputs(32, 2, 1), default_constants());

        assert!(low.total_cost.is_finite());
        assert!(high.total_cost > low.total_cost);
    }

    #[test]
    fn startup_cost_increases_with_recursive_depth() {
        let shallow = estimate_spire_cost(&inputs(8, 1, 1), default_constants());
        let deep = estimate_spire_cost(&inputs(8, 4, 1), default_constants());

        assert!(deep.startup_cost > shallow.startup_cost);
    }

    #[test]
    fn local_store_count_affects_page_and_coordination_cost() {
        let single_store = estimate_spire_cost(&inputs(8, 2, 1), default_constants());
        let multi_store = estimate_spire_cost(&inputs(8, 2, 4), default_constants());

        assert!(multi_store.total_cost > single_store.total_cost);
    }
}

#[cfg(feature = "pg18")]
pub(super) unsafe extern "C-unwind" fn ec_spire_amtranslatestrategy(
    strategy: pg_sys::StrategyNumber,
    _opfamily: pg_sys::Oid,
) -> pg_sys::CompareType::Type {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            match cost::amtranslatestrategy_callback(i32::from(strategy)) {
                cost::PlannerCompareType::Invalid => pg_sys::CompareType::COMPARE_INVALID,
                cost::PlannerCompareType::Lt => pg_sys::CompareType::COMPARE_LT,
                cost::PlannerCompareType::Le => pg_sys::CompareType::COMPARE_LE,
                cost::PlannerCompareType::Eq => pg_sys::CompareType::COMPARE_EQ,
                cost::PlannerCompareType::Ge => pg_sys::CompareType::COMPARE_GE,
                cost::PlannerCompareType::Gt => pg_sys::CompareType::COMPARE_GT,
                cost::PlannerCompareType::Ne => pg_sys::CompareType::COMPARE_NE,
                cost::PlannerCompareType::Overlap => pg_sys::CompareType::COMPARE_OVERLAP,
                cost::PlannerCompareType::ContainedBy => pg_sys::CompareType::COMPARE_CONTAINED_BY,
            }
        })
    }
}

#[cfg(feature = "pg18")]
pub(super) unsafe extern "C-unwind" fn ec_spire_amtranslatecmptype(
    compare_type: pg_sys::CompareType::Type,
    _opfamily: pg_sys::Oid,
) -> pg_sys::StrategyNumber {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            cost::amtranslatecmptype_callback(match compare_type {
                pg_sys::CompareType::COMPARE_LT => cost::PlannerCompareType::Lt,
                pg_sys::CompareType::COMPARE_LE => cost::PlannerCompareType::Le,
                pg_sys::CompareType::COMPARE_EQ => cost::PlannerCompareType::Eq,
                pg_sys::CompareType::COMPARE_GE => cost::PlannerCompareType::Ge,
                pg_sys::CompareType::COMPARE_GT => cost::PlannerCompareType::Gt,
                pg_sys::CompareType::COMPARE_NE => cost::PlannerCompareType::Ne,
                pg_sys::CompareType::COMPARE_OVERLAP => cost::PlannerCompareType::Overlap,
                pg_sys::CompareType::COMPARE_CONTAINED_BY => cost::PlannerCompareType::ContainedBy,
                _ => cost::PlannerCompareType::Invalid,
            }) as pg_sys::StrategyNumber
        })
    }
}
