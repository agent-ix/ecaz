use pgrx::pg_sys;

use crate::am::ec_hnsw::{options, page, shared};
use crate::storage::relation_guard::IndexRelationGuard;

// Ordered ec_hnsw scoring walks LUT-backed quantized codes, not full raw-f32
// arithmetic at every candidate. Charging the planner the full raw dimension
// count overstates index-side CPU enough to flip the real 10k / 1536-d / ef=200
// probe to Seq Scan + Sort even though the forced index path is still far
// faster. Keep the calibration conservative so small tables still prefer
// seqscan while high-dimension LIMIT queries stay on the index.
const LUT_CPU_DIMENSION_SCALE: f64 = 0.75;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlannerCostInputs {
    pub index_pages: f64,
    pub reltuples: f64,
    // Reserved for future calibration work; the current FR-020 model does not
    // read `m`, but we keep it in the input surface so later adjustments do
    // not need to reshape every call site.
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

#[cfg_attr(feature = "pg18", allow(dead_code))]
pub(crate) fn metadata_fallback_tree_height(max_level: u8) -> PlannerTreeHeightInput {
    PlannerTreeHeightInput {
        tree_height: f64::from(max_level),
        source: "metadata_fallback",
        pg18_callback_ready: false,
    }
}

pub(crate) fn resolved_tree_height_input(max_level: u8) -> PlannerTreeHeightInput {
    #[cfg(feature = "pg18")]
    {
        PlannerTreeHeightInput {
            tree_height: f64::from(amgettreeheight_callback_value(max_level)),
            source: "amgettreeheight_callback",
            pg18_callback_ready: true,
        }
    }

    #[cfg(not(feature = "pg18"))]
    {
        metadata_fallback_tree_height(max_level)
    }
}

pub(crate) fn amgettreeheight_callback_value(max_level: u8) -> i32 {
    i32::from(max_level)
}

pub(crate) fn amtranslatestrategy_callback(strategy: i32) -> PlannerCompareType {
    match strategy {
        1 => PlannerCompareType::Lt,
        _ => PlannerCompareType::Invalid,
    }
}

pub(crate) fn amtranslatecmptype_callback(compare_type: PlannerCompareType) -> i32 {
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
        ordering_compare_type: amtranslatestrategy_callback(1),
        pg18_callback_ready: cfg!(feature = "pg18"),
    }
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn ec_hnsw_amgettreeheight(rel: pg_sys::Relation) -> i32 {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let metadata = shared::read_metadata_page(rel);
            amgettreeheight_callback_value(metadata.max_level)
        })
    }
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn ec_hnsw_amtranslatestrategy(
    strategy: pg_sys::StrategyNumber,
    _opfamily: pg_sys::Oid,
) -> pg_sys::CompareType::Type {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| match amtranslatestrategy_callback(i32::from(strategy)) {
            PlannerCompareType::Invalid => pg_sys::CompareType::COMPARE_INVALID,
            PlannerCompareType::Lt => pg_sys::CompareType::COMPARE_LT,
            PlannerCompareType::Le => pg_sys::CompareType::COMPARE_LE,
            PlannerCompareType::Eq => pg_sys::CompareType::COMPARE_EQ,
            PlannerCompareType::Ge => pg_sys::CompareType::COMPARE_GE,
            PlannerCompareType::Gt => pg_sys::CompareType::COMPARE_GT,
            PlannerCompareType::Ne => pg_sys::CompareType::COMPARE_NE,
            PlannerCompareType::Overlap => pg_sys::CompareType::COMPARE_OVERLAP,
            PlannerCompareType::ContainedBy => pg_sys::CompareType::COMPARE_CONTAINED_BY,
        })
    }
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn ec_hnsw_amtranslatecmptype(
    compare_type: pg_sys::CompareType::Type,
    _opfamily: pg_sys::Oid,
) -> pg_sys::StrategyNumber {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            amtranslatecmptype_callback(match compare_type {
                pg_sys::CompareType::COMPARE_LT => PlannerCompareType::Lt,
                pg_sys::CompareType::COMPARE_LE => PlannerCompareType::Le,
                pg_sys::CompareType::COMPARE_EQ => PlannerCompareType::Eq,
                pg_sys::CompareType::COMPARE_GE => PlannerCompareType::Ge,
                pg_sys::CompareType::COMPARE_GT => PlannerCompareType::Gt,
                pg_sys::CompareType::COMPARE_NE => PlannerCompareType::Ne,
                pg_sys::CompareType::COMPARE_OVERLAP => PlannerCompareType::Overlap,
                pg_sys::CompareType::COMPARE_CONTAINED_BY => PlannerCompareType::ContainedBy,
                _ => PlannerCompareType::Invalid,
            }) as pg_sys::StrategyNumber
        })
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
    let scoring_dimensions = f64::from(inputs.dimensions) * LUT_CPU_DIMENSION_SCALE;
    let ef_search = f64::from(inputs.ef_search);
    let tree_height = inputs.tree_height;

    let (startup_cost, total_cost) = if tree_height <= 0.0 {
        let linear_cost = inputs.index_pages * constants.seq_page_cost;
        let linear_cpu = tuple_estimate * constants.cpu_operator_cost * scoring_dimensions;
        (0.0, linear_cost + linear_cpu)
    } else {
        // Graph phase visits ~1 page per upper level (greedy descent to the
        // entry neighborhood) plus ef_search candidate pages at the bottom
        // layer. The earlier `tree_height * m + ef_search * 2 * m` formula
        // wildly overcharged by multiplying ef_search by m, which kept
        // ec_hnsw costlier than a seqscan on every realistic table size.
        let graph_pages = tree_height + ef_search;
        let graph_cost = graph_pages * constants.random_page_cost;
        let graph_cpu = graph_pages * constants.cpu_operator_cost * scoring_dimensions;
        let linear_pages = (inputs.index_pages - graph_pages).max(0.0);
        let linear_cost = linear_pages * constants.seq_page_cost;
        // Scale per-tuple CPU work by the fraction of pages the graph phase
        // did NOT cover. Charging the full tuple_estimate sweep here would
        // double-count the graph traversal and make ec_hnsw always look
        // costlier than a seqscan, even when the graph covers the whole
        // index (linear_pages == 0).
        let linear_fraction = if inputs.index_pages > 0.0 {
            linear_pages / inputs.index_pages
        } else {
            0.0
        };
        let linear_cpu =
            tuple_estimate * constants.cpu_operator_cost * scoring_dimensions * linear_fraction;
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

pub(crate) unsafe extern "C-unwind" fn ec_hnsw_amcostestimate(
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
            let index_relation = IndexRelationGuard::open(
                index_oid,
                pg_sys::NoLock as pg_sys::LOCKMODE,
                "ec_hnsw planner",
            );
            let estimate = compute_amcostestimate(index_relation.as_ptr(), index_info);

            *index_startup_cost = estimate.startup_cost;
            *index_total_cost = estimate.total_cost;
            *index_selectivity = estimate.selectivity;
            *index_correlation = estimate.correlation;
            *index_pages = estimate.index_pages;
        })
    }
}

unsafe fn planner_tree_height_from_index_info(
    index_info: *mut pg_sys::IndexOptInfo,
    max_level: u8,
) -> PlannerTreeHeightInput {
    #[cfg(feature = "pg18")]
    {
        let planner_tree_height = unsafe { (*index_info).tree_height };
        if planner_tree_height > 0 {
            return PlannerTreeHeightInput {
                tree_height: f64::from(planner_tree_height),
                source: "amgettreeheight_callback",
                pg18_callback_ready: true,
            };
        }

        resolved_tree_height_input(max_level)
    }

    #[cfg(not(feature = "pg18"))]
    {
        let _ = index_info;
        metadata_fallback_tree_height(max_level)
    }
}

unsafe fn compute_amcostestimate(
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexOptInfo,
) -> PlannerCostEstimate {
    let relation_options = unsafe { options::relation_options(index_relation) };
    let tuning = options::resolve_scan_tuning(&relation_options);
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let index_pages = f64::from(block_count);
    // Block 0 is always the metadata page; an index with no data pages still
    // reports block_count == 1. FR-020's "Empty index (0 data pages)" gate
    // must trip on `block_count <= FIRST_DATA_BLOCK_NUMBER`, not on the raw
    // page count.
    if block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        return gated_planner_cost_estimate(index_pages);
    }
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    let metadata = unsafe { shared::read_metadata_page(index_relation) };
    let tree_height =
        unsafe { planner_tree_height_from_index_info(index_info, metadata.max_level) };
    let constants = unsafe { current_planner_cost_constants() };

    estimate_planner_cost(
        PlannerCostInputs {
            index_pages,
            reltuples,
            m: relation_options.m,
            ef_search: tuning.effective_ef_search,
            dimensions: metadata.dimensions,
            tree_height: tree_height.tree_height,
        },
        constants,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        amgettreeheight_callback_value, amtranslatecmptype_callback, amtranslatestrategy_callback,
        estimate_planner_cost, metadata_fallback_tree_height, resolved_tree_height_input,
        strategy_translation_snapshot, PlannerCompareType, PlannerCostConstants,
        PlannerCostEstimate, PlannerCostInputs, PlannerTreeHeightInput,
        StrategyTranslationSnapshot, LUT_CPU_DIMENSION_SCALE,
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
            "large-table planner scaffolding should model ec_hnsw as cheaper than seqscan once ADR-011 is retired"
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
            + expected_tuple_estimate
                * constants.cpu_operator_cost
                * (128.0 * LUT_CPU_DIMENSION_SCALE);

        assert_eq!(estimate.startup_cost, 0.0);
        assert_eq!(estimate.total_cost, expected_total_cost);
    }

    #[test]
    fn planner_cost_model_keeps_real_10k_ef200_probe_below_seqscan_sort_crossover() {
        let constants = default_constants();
        let estimate = estimate_planner_cost(
            PlannerCostInputs {
                index_pages: 1251.0,
                reltuples: 10_000.0,
                m: 8,
                ef_search: 200,
                dimensions: 1536,
                tree_height: 4.0,
            },
            constants,
        );

        // Observed live real-10k planner crossover on 2026-04-11:
        // the seqscan+sort alternative for LIMIT 10 costs ~1526.10. The
        // ec_hnsw startup cost needs to stay below that boundary or the
        // planner abandons the index even though the forced index path is
        // materially faster than the seqscan fallback.
        assert!(
            estimate.startup_cost < 1526.10,
            "real 10k / 1536-d / ef=200 startup cost must stay below the observed seqscan+sort crossover: {estimate:?}"
        );
    }

    #[test]
    fn planner_cost_tree_height_snapshot_matches_build_target() {
        assert_eq!(
            resolved_tree_height_input(4),
            PlannerTreeHeightInput {
                tree_height: 4.0,
                source: if cfg!(feature = "pg18") {
                    "amgettreeheight_callback"
                } else {
                    "metadata_fallback"
                },
                pg18_callback_ready: cfg!(feature = "pg18"),
            }
        );
        if !cfg!(feature = "pg18") {
            assert_eq!(
                metadata_fallback_tree_height(4),
                PlannerTreeHeightInput {
                    tree_height: 4.0,
                    source: "metadata_fallback",
                    pg18_callback_ready: false,
                }
            );
        }
    }

    #[test]
    fn strategy_translation_snapshot_matches_build_target() {
        assert_eq!(
            strategy_translation_snapshot(),
            StrategyTranslationSnapshot {
                ordering_strategy: 1,
                ordering_compare_type: PlannerCompareType::Lt,
                pg18_callback_ready: cfg!(feature = "pg18"),
            }
        );
    }

    #[test]
    fn tree_height_callback_value_returns_metadata_max_level() {
        assert_eq!(amgettreeheight_callback_value(0), 0);
        assert_eq!(amgettreeheight_callback_value(4), 4);
        assert_eq!(amgettreeheight_callback_value(u8::MAX), i32::from(u8::MAX));
    }

    #[test]
    fn strategy_translation_maps_ordering_strategy_to_compare_lt() {
        assert_eq!(amtranslatestrategy_callback(1), PlannerCompareType::Lt);
        assert_eq!(amtranslatecmptype_callback(PlannerCompareType::Lt), 1);
    }

    #[test]
    fn strategy_translation_rejects_invalid_inputs() {
        assert_eq!(amtranslatestrategy_callback(0), PlannerCompareType::Invalid);
        assert_eq!(
            amtranslatestrategy_callback(99),
            PlannerCompareType::Invalid
        );
        assert_eq!(amtranslatecmptype_callback(PlannerCompareType::Invalid), 0);
        assert_eq!(amtranslatecmptype_callback(PlannerCompareType::Le), 0);
        assert_eq!(amtranslatecmptype_callback(PlannerCompareType::Eq), 0);
        assert_eq!(amtranslatecmptype_callback(PlannerCompareType::Ge), 0);
        assert_eq!(amtranslatecmptype_callback(PlannerCompareType::Gt), 0);
        assert_eq!(amtranslatecmptype_callback(PlannerCompareType::Ne), 0);
        assert_eq!(amtranslatecmptype_callback(PlannerCompareType::Overlap), 0);
        assert_eq!(
            amtranslatecmptype_callback(PlannerCompareType::ContainedBy),
            0
        );
    }
}
