use pgrx::pg_sys;

use super::TQHNSW_PLANNER_SCAN_ENABLED;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlannerCostInputs {
    pub index_pages: f64,
    pub reltuples: f64,
    pub m: i32,
    pub ef_search: i32,
    pub dimensions: u16,
    pub tree_height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlannerCostConstants {
    pub random_page_cost: f64,
    pub seq_page_cost: f64,
    pub cpu_operator_cost: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlannerCostEstimate {
    pub startup_cost: f64,
    pub total_cost: f64,
    pub selectivity: f64,
    pub correlation: f64,
    pub index_pages: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlannerTreeHeightInput {
    pub tree_height: f64,
    pub source: &'static str,
    pub pg18_callback_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlannerCompareType {
    Invalid,
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
    Ne,
    Overlap,
    ContainedBy,
}

impl PlannerCompareType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "COMPARE_INVALID",
            Self::Lt => "COMPARE_LT",
            Self::Le => "COMPARE_LE",
            Self::Eq => "COMPARE_EQ",
            Self::Ge => "COMPARE_GE",
            Self::Gt => "COMPARE_GT",
            Self::Ne => "COMPARE_NE",
            Self::Overlap => "COMPARE_OVERLAP",
            Self::ContainedBy => "COMPARE_CONTAINED_BY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StrategyTranslationSnapshot {
    pub ordering_strategy: i32,
    pub ordering_compare_type: PlannerCompareType,
    pub pg18_callback_ready: bool,
}

pub(crate) fn gated_planner_cost_estimate(index_pages: f64) -> PlannerCostEstimate {
    PlannerCostEstimate {
        startup_cost: f64::MAX,
        total_cost: f64::MAX,
        selectivity: 0.0,
        correlation: 0.0,
        index_pages,
    }
}

pub(crate) unsafe fn current_planner_cost_constants() -> PlannerCostConstants {
    PlannerCostConstants {
        random_page_cost: unsafe { pg_sys::random_page_cost },
        seq_page_cost: unsafe { pg_sys::seq_page_cost },
        cpu_operator_cost: unsafe { pg_sys::cpu_operator_cost },
    }
}

pub(crate) fn metadata_fallback_tree_height(max_level: u8) -> PlannerTreeHeightInput {
    PlannerTreeHeightInput {
        tree_height: f64::from(max_level),
        source: "metadata_fallback",
        pg18_callback_ready: false,
    }
}

pub(crate) fn metadata_tree_height_callback_value(max_level: u8) -> i32 {
    i32::from(max_level)
}

pub(crate) fn strategy_to_compare_type(strategy: i32) -> PlannerCompareType {
    match strategy {
        1 => PlannerCompareType::Lt,
        _ => PlannerCompareType::Invalid,
    }
}

pub(crate) fn compare_type_to_strategy(compare_type: PlannerCompareType) -> i32 {
    match compare_type {
        PlannerCompareType::Lt => 1,
        PlannerCompareType::Invalid
        | PlannerCompareType::Le
        | PlannerCompareType::Eq
        | PlannerCompareType::Ge
        | PlannerCompareType::Gt
        | PlannerCompareType::Ne
        | PlannerCompareType::Overlap
        | PlannerCompareType::ContainedBy => 0,
    }
}

pub(crate) fn strategy_translation_snapshot() -> StrategyTranslationSnapshot {
    StrategyTranslationSnapshot {
        ordering_strategy: 1,
        ordering_compare_type: strategy_to_compare_type(1),
        pg18_callback_ready: false,
    }
}

pub(crate) fn estimate_planner_cost(
    inputs: PlannerCostInputs,
    constants: PlannerCostConstants,
) -> PlannerCostEstimate {
    if inputs.index_pages <= 0.0 {
        return gated_planner_cost_estimate(inputs.index_pages);
    }

    let tuple_estimate = if inputs.reltuples > 0.0 {
        inputs.reltuples
    } else {
        inputs.index_pages * 10.0
    };
    let dimensions = f64::from(inputs.dimensions);
    let m = f64::from(inputs.m);
    let ef_search = f64::from(inputs.ef_search);
    let tree_height = inputs.tree_height;

    let (startup_cost, total_cost) = if tree_height <= 0.0 {
        let linear_cost = inputs.index_pages * constants.seq_page_cost;
        let linear_cpu = tuple_estimate * constants.cpu_operator_cost * dimensions;
        (0.0, linear_cost + linear_cpu)
    } else {
        let graph_pages = tree_height * m + ef_search * 2.0 * m;
        let graph_cost = graph_pages * constants.random_page_cost;
        let graph_cpu = graph_pages * constants.cpu_operator_cost * dimensions;
        let linear_pages = (inputs.index_pages - graph_pages).max(0.0);
        let linear_cost = linear_pages * constants.seq_page_cost;
        let linear_cpu = tuple_estimate * constants.cpu_operator_cost * dimensions;
        let startup_cost = graph_cost + graph_cpu;
        (startup_cost, startup_cost + linear_cost + linear_cpu)
    };

    PlannerCostEstimate {
        startup_cost,
        total_cost,
        selectivity: 1.0,
        correlation: 0.0,
        index_pages: inputs.index_pages,
    }
}

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
            let gated = gated_planner_cost_estimate(0.0);
            *index_startup_cost = gated.startup_cost;
            *index_total_cost = gated.total_cost;
            *index_selectivity = gated.selectivity;
            *index_correlation = gated.correlation;
            *index_pages = gated.index_pages;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compare_type_to_strategy, estimate_planner_cost, metadata_fallback_tree_height,
        metadata_tree_height_callback_value, strategy_to_compare_type,
        strategy_translation_snapshot, PlannerCompareType, PlannerCostConstants,
        PlannerCostEstimate, PlannerCostInputs, PlannerTreeHeightInput,
        StrategyTranslationSnapshot,
    };

    fn default_constants() -> PlannerCostConstants {
        PlannerCostConstants {
            random_page_cost: 4.0,
            seq_page_cost: 1.0,
            cpu_operator_cost: 0.0025,
        }
    }

    fn estimate_seqscan_cost(
        table_pages: f64,
        reltuples: f64,
        dimensions: u16,
        constants: PlannerCostConstants,
    ) -> f64 {
        let tuple_estimate = if reltuples > 0.0 {
            reltuples
        } else {
            table_pages * 10.0
        };
        table_pages * constants.seq_page_cost
            + tuple_estimate * constants.cpu_operator_cost * f64::from(dimensions)
    }

    #[test]
    fn planner_cost_model_stays_cheaper_than_seqscan_for_large_tables() {
        let constants = default_constants();
        let index_cost = estimate_planner_cost(
            PlannerCostInputs {
                index_pages: 400.0,
                reltuples: 10_000.0,
                m: 8,
                ef_search: 40,
                dimensions: 1536,
                tree_height: 3.0,
            },
            constants,
        );
        let seqscan_cost = estimate_seqscan_cost(8_000.0, 10_000.0, 1536, constants);

        assert!(
            index_cost.total_cost < seqscan_cost,
            "large-table planner scaffolding should model tqhnsw as cheaper than seqscan once ADR-011 is retired"
        );
    }

    #[test]
    fn planner_cost_model_stays_more_expensive_than_seqscan_for_small_tables() {
        let constants = default_constants();
        let index_cost = estimate_planner_cost(
            PlannerCostInputs {
                index_pages: 8.0,
                reltuples: 50.0,
                m: 8,
                ef_search: 40,
                dimensions: 1536,
                tree_height: 3.0,
            },
            constants,
        );
        let seqscan_cost = estimate_seqscan_cost(4.0, 50.0, 1536, constants);

        assert!(
            index_cost.total_cost > seqscan_cost,
            "small-table planner scaffolding should still model seqscan as cheaper"
        );
    }

    #[test]
    fn planner_cost_model_returns_max_for_empty_index() {
        let estimate = estimate_planner_cost(
            PlannerCostInputs {
                index_pages: 0.0,
                reltuples: 0.0,
                m: 8,
                ef_search: 40,
                dimensions: 1536,
                tree_height: 3.0,
            },
            default_constants(),
        );

        assert_eq!(
            estimate,
            PlannerCostEstimate {
                startup_cost: f64::MAX,
                total_cost: f64::MAX,
                selectivity: 0.0,
                correlation: 0.0,
                index_pages: 0.0,
            }
        );
    }

    #[test]
    fn planner_cost_model_uses_reltuples_heuristic_when_stats_are_missing() {
        let constants = default_constants();
        let estimate = estimate_planner_cost(
            PlannerCostInputs {
                index_pages: 12.0,
                reltuples: 0.0,
                m: 8,
                ef_search: 40,
                dimensions: 128,
                tree_height: 0.0,
            },
            constants,
        );
        let expected_tuple_estimate = 120.0;
        let expected_total_cost = 12.0 * constants.seq_page_cost
            + expected_tuple_estimate * constants.cpu_operator_cost * 128.0;

        assert_eq!(estimate.startup_cost, 0.0);
        assert_eq!(estimate.total_cost, expected_total_cost);
    }

    #[test]
    fn planner_cost_tree_height_defaults_to_metadata_until_pg18_callback_exists() {
        assert_eq!(
            metadata_fallback_tree_height(4),
            PlannerTreeHeightInput {
                tree_height: 4.0,
                source: "metadata_fallback",
                pg18_callback_ready: false,
            }
        );
    }

    #[test]
    fn strategy_translation_snapshot_stays_explicitly_unwired_until_pg18_support_exists() {
        assert_eq!(
            strategy_translation_snapshot(),
            StrategyTranslationSnapshot {
                ordering_strategy: 1,
                ordering_compare_type: PlannerCompareType::Lt,
                pg18_callback_ready: false,
            }
        );
    }

    #[test]
    fn tree_height_callback_value_returns_metadata_max_level() {
        assert_eq!(metadata_tree_height_callback_value(0), 0);
        assert_eq!(metadata_tree_height_callback_value(4), 4);
        assert_eq!(metadata_tree_height_callback_value(u8::MAX), i32::from(u8::MAX));
    }

    #[test]
    fn strategy_translation_maps_ordering_strategy_to_compare_lt() {
        assert_eq!(strategy_to_compare_type(1), PlannerCompareType::Lt);
        assert_eq!(compare_type_to_strategy(PlannerCompareType::Lt), 1);
    }

    #[test]
    fn strategy_translation_rejects_invalid_inputs() {
        assert_eq!(strategy_to_compare_type(0), PlannerCompareType::Invalid);
        assert_eq!(strategy_to_compare_type(99), PlannerCompareType::Invalid);
        assert_eq!(compare_type_to_strategy(PlannerCompareType::Invalid), 0);
        assert_eq!(compare_type_to_strategy(PlannerCompareType::Le), 0);
        assert_eq!(compare_type_to_strategy(PlannerCompareType::Eq), 0);
        assert_eq!(compare_type_to_strategy(PlannerCompareType::Ge), 0);
        assert_eq!(compare_type_to_strategy(PlannerCompareType::Gt), 0);
        assert_eq!(compare_type_to_strategy(PlannerCompareType::Ne), 0);
        assert_eq!(compare_type_to_strategy(PlannerCompareType::Overlap), 0);
        assert_eq!(
            compare_type_to_strategy(PlannerCompareType::ContainedBy),
            0
        );
    }
}
