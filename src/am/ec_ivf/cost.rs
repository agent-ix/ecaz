use std::mem::size_of;

use pgrx::pg_sys;

use super::{options, page};
use crate::am::common::cost::{
    current_planner_cost_constants, strategy_translation_snapshot, PlannerCostConstants,
    PlannerCostEstimate, PlannerTreeHeightInput, StrategyTranslationSnapshot,
};

// Task 28 packet 30076 measured the scan kernels on local AVX2+FMA:
// 1536D centroid f32 IP: 1306.1 ns, 1536D no-QJL 4-bit LUT posting score:
// 1331.0 ns. Model centroid and posting scoring with the same dimension
// scale until a wider microbenchmark shows a real divergence.
const IVF_CENTROID_SCORING_DIMENSION_SCALE: f64 = 0.01;
const IVF_POSTING_SCORING_DIMENSION_SCALE: f64 = 0.01;
// IVF probes scan contiguous posting-list block ranges, so seq_page_cost is
// the least-surprising page basis. Do not apply a sub-sequential warm-cache
// discount in the planner model without a cold/warm buffer-backed measurement.
const IVF_INDEX_PAGE_COST_SCALE: f64 = 1.0;
const IVF_TREE_HEIGHT: i32 = 0;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexCostSnapshot {
    pub planner_scan_enabled: bool,
    pub planner_gate_reason: &'static str,
    pub dimensions: u16,
    pub nlists: u32,
    pub relation_nprobe: u32,
    pub session_nprobe: Option<u32>,
    pub effective_nprobe: u32,
    pub effective_nprobe_source: &'static str,
    pub resolved_tree_height: f64,
    pub tree_height_source: &'static str,
    pub pg18_tree_height_callback_ready: bool,
    pub ordering_compare_type: &'static str,
    pub pg18_strategy_translation_ready: bool,
    pub average_list_live_count: f64,
    pub estimated_centroid_scores: u32,
    pub estimated_selected_lists: u32,
    pub estimated_candidate_rows: f64,
    pub estimated_posting_pages: f64,
    pub storage_format: &'static str,
    pub scoring_mode: &'static str,
    pub scoring_multiplier: f64,
    pub rerank: &'static str,
    pub rerank_multiplier: f64,
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

pub(crate) unsafe extern "C-unwind" fn ec_ivf_amcostestimate(
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

pub(crate) fn ivf_tree_height_callback_value() -> i32 {
    IVF_TREE_HEIGHT
}

pub(crate) fn resolved_ivf_tree_height_input() -> PlannerTreeHeightInput {
    PlannerTreeHeightInput {
        tree_height: f64::from(ivf_tree_height_callback_value()),
        source: if cfg!(feature = "pg18") {
            "amgettreeheight_callback"
        } else {
            "partitioned_ivf"
        },
        pg18_callback_ready: cfg!(feature = "pg18"),
    }
}

pub(crate) fn ivf_strategy_translation_snapshot() -> StrategyTranslationSnapshot {
    strategy_translation_snapshot()
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn ec_ivf_amgettreeheight(_rel: pg_sys::Relation) -> i32 {
    unsafe { pgrx::pgrx_extern_c_guard(ivf_tree_height_callback_value) }
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn ec_ivf_amtranslatestrategy(
    strategy: pg_sys::StrategyNumber,
    _opfamily: pg_sys::Oid,
) -> pg_sys::CompareType::Type {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            match crate::am::common::cost::amtranslatestrategy_callback(i32::from(strategy)) {
                crate::am::common::cost::PlannerCompareType::Invalid => {
                    pg_sys::CompareType::COMPARE_INVALID
                }
                crate::am::common::cost::PlannerCompareType::Lt => pg_sys::CompareType::COMPARE_LT,
                crate::am::common::cost::PlannerCompareType::Le => pg_sys::CompareType::COMPARE_LE,
                crate::am::common::cost::PlannerCompareType::Eq => pg_sys::CompareType::COMPARE_EQ,
                crate::am::common::cost::PlannerCompareType::Ge => pg_sys::CompareType::COMPARE_GE,
                crate::am::common::cost::PlannerCompareType::Gt => pg_sys::CompareType::COMPARE_GT,
                crate::am::common::cost::PlannerCompareType::Ne => pg_sys::CompareType::COMPARE_NE,
                crate::am::common::cost::PlannerCompareType::Overlap => {
                    pg_sys::CompareType::COMPARE_OVERLAP
                }
                crate::am::common::cost::PlannerCompareType::ContainedBy => {
                    pg_sys::CompareType::COMPARE_CONTAINED_BY
                }
            }
        })
    }
}

#[cfg(feature = "pg18")]
pub(crate) unsafe extern "C-unwind" fn ec_ivf_amtranslatecmptype(
    compare_type: pg_sys::CompareType::Type,
    _opfamily: pg_sys::Oid,
) -> pg_sys::StrategyNumber {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            crate::am::common::cost::amtranslatecmptype_callback(match compare_type {
                pg_sys::CompareType::COMPARE_LT => crate::am::common::cost::PlannerCompareType::Lt,
                pg_sys::CompareType::COMPARE_LE => crate::am::common::cost::PlannerCompareType::Le,
                pg_sys::CompareType::COMPARE_EQ => crate::am::common::cost::PlannerCompareType::Eq,
                pg_sys::CompareType::COMPARE_GE => crate::am::common::cost::PlannerCompareType::Ge,
                pg_sys::CompareType::COMPARE_GT => crate::am::common::cost::PlannerCompareType::Gt,
                pg_sys::CompareType::COMPARE_NE => crate::am::common::cost::PlannerCompareType::Ne,
                pg_sys::CompareType::COMPARE_OVERLAP => {
                    crate::am::common::cost::PlannerCompareType::Overlap
                }
                pg_sys::CompareType::COMPARE_CONTAINED_BY => {
                    crate::am::common::cost::PlannerCompareType::ContainedBy
                }
                _ => crate::am::common::cost::PlannerCompareType::Invalid,
            }) as pg_sys::StrategyNumber
        })
    }
}

pub(crate) unsafe fn index_cost_snapshot(index_relation: pg_sys::Relation) -> IndexCostSnapshot {
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let index_pages = f64::from(block_count);
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    let constants = unsafe { current_planner_cost_constants() };
    let nprobe = options::resolve_scan_nprobe(metadata.nlists, metadata.nprobe);
    let tree_height = resolved_ivf_tree_height_input();
    let translation = ivf_strategy_translation_snapshot();
    let estimate = estimate_ivf_cost(&metadata, index_pages, reltuples, constants);
    let details = estimate_details(&metadata, index_pages, reltuples);

    IndexCostSnapshot {
        planner_scan_enabled: true,
        planner_gate_reason: "planner scan selection is live: Task 28 IVF cost model active",
        dimensions: metadata.dimensions,
        nlists: metadata.nlists,
        relation_nprobe: nprobe.relation_nprobe,
        session_nprobe: nprobe.session_nprobe,
        effective_nprobe: nprobe.effective_nprobe,
        effective_nprobe_source: nprobe.source,
        resolved_tree_height: tree_height.tree_height,
        tree_height_source: tree_height.source,
        pg18_tree_height_callback_ready: tree_height.pg18_callback_ready,
        ordering_compare_type: translation.ordering_compare_type.as_str(),
        pg18_strategy_translation_ready: translation.pg18_callback_ready,
        average_list_live_count: details.average_list_live_count,
        estimated_centroid_scores: details.estimated_centroid_scores,
        estimated_selected_lists: details.estimated_selected_lists,
        estimated_candidate_rows: details.estimated_candidate_rows,
        estimated_posting_pages: details.estimated_posting_pages,
        storage_format: metadata.storage_format.reloption_name(),
        scoring_mode: scoring_mode_name(metadata.storage_format),
        scoring_multiplier: storage_scoring_multiplier(metadata.storage_format),
        rerank: metadata.rerank.reloption_name(),
        rerank_multiplier: rerank_multiplier(metadata.rerank),
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

unsafe fn compute_amcostestimate(index_relation: pg_sys::Relation) -> PlannerCostEstimate {
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let index_pages = f64::from(block_count);
    let reltuples = unsafe { (*(*index_relation).rd_rel).reltuples } as f64;
    let constants = unsafe { current_planner_cost_constants() };

    estimate_ivf_cost(&metadata, index_pages, reltuples, constants)
}

fn estimate_ivf_cost(
    metadata: &page::MetadataPage,
    index_pages: f64,
    reltuples: f64,
    constants: PlannerCostConstants,
) -> PlannerCostEstimate {
    let nprobe = options::resolve_scan_nprobe(metadata.nlists, metadata.nprobe);
    estimate_ivf_cost_with_nprobe(
        metadata,
        index_pages,
        reltuples,
        constants,
        nprobe.effective_nprobe,
    )
}

fn estimate_ivf_cost_with_nprobe(
    metadata: &page::MetadataPage,
    index_pages: f64,
    reltuples: f64,
    constants: PlannerCostConstants,
    effective_nprobe: u32,
) -> PlannerCostEstimate {
    let details = estimate_details_for_nprobe(metadata, index_pages, reltuples, effective_nprobe);
    let centroid_scoring_dimensions =
        f64::from(metadata.dimensions) * IVF_CENTROID_SCORING_DIMENSION_SCALE;
    let posting_scoring_dimensions =
        f64::from(metadata.dimensions) * IVF_POSTING_SCORING_DIMENSION_SCALE;
    let centroid_cpu = f64::from(details.estimated_centroid_scores)
        * constants.cpu_operator_cost
        * centroid_scoring_dimensions;
    let index_page_cost = constants.seq_page_cost * IVF_INDEX_PAGE_COST_SCALE;
    let centroid_page_cost = centroid_page_estimate(metadata) * index_page_cost;
    let posting_page_cost = details.estimated_posting_pages * index_page_cost;
    let candidate_cpu = details.estimated_candidate_rows
        * constants.cpu_operator_cost
        * posting_scoring_dimensions
        * storage_scoring_multiplier(metadata.storage_format)
        * rerank_multiplier(metadata.rerank);
    let metadata_page_cost = if index_pages > 0.0 {
        constants.random_page_cost
    } else {
        0.0
    };
    let startup_cost = metadata_page_cost + centroid_page_cost + centroid_cpu;

    PlannerCostEstimate {
        startup_cost,
        total_cost: startup_cost + posting_page_cost + candidate_cpu,
        selectivity: 1.0,
        correlation: 0.0,
        index_pages,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct IvfCostDetails {
    average_list_live_count: f64,
    estimated_centroid_scores: u32,
    estimated_selected_lists: u32,
    estimated_candidate_rows: f64,
    estimated_posting_pages: f64,
}

fn estimate_details(
    metadata: &page::MetadataPage,
    index_pages: f64,
    reltuples: f64,
) -> IvfCostDetails {
    let nprobe = options::resolve_scan_nprobe(metadata.nlists, metadata.nprobe);
    estimate_details_for_nprobe(metadata, index_pages, reltuples, nprobe.effective_nprobe)
}

fn estimate_details_for_nprobe(
    metadata: &page::MetadataPage,
    index_pages: f64,
    reltuples: f64,
    effective_nprobe: u32,
) -> IvfCostDetails {
    let total_live = if metadata.total_live_tuples > 0 {
        metadata.total_live_tuples as f64
    } else {
        reltuples.max(0.0)
    };
    let average_list_live_count = if metadata.nlists == 0 {
        0.0
    } else {
        total_live / f64::from(metadata.nlists)
    };
    let estimated_selected_lists = effective_nprobe.clamp(0, metadata.nlists);
    let estimated_candidate_rows =
        (average_list_live_count * f64::from(estimated_selected_lists)).min(total_live);
    let list_fraction = if metadata.nlists == 0 {
        0.0
    } else {
        f64::from(estimated_selected_lists) / f64::from(metadata.nlists)
    };
    let data_pages = (index_pages - 1.0 - centroid_page_estimate(metadata)).max(0.0);
    let estimated_posting_pages = (data_pages * list_fraction).min(data_pages);

    IvfCostDetails {
        average_list_live_count,
        estimated_centroid_scores: metadata.nlists,
        estimated_selected_lists,
        estimated_candidate_rows,
        estimated_posting_pages,
    }
}

fn centroid_page_estimate(metadata: &page::MetadataPage) -> f64 {
    if metadata.nlists == 0 || metadata.dimensions == 0 {
        return 0.0;
    }
    let centroid_bytes = f64::from(metadata.nlists)
        * (7.0 + f64::from(metadata.dimensions) * size_of::<f32>() as f64);
    (centroid_bytes / pg_sys::BLCKSZ as f64).ceil().max(1.0)
}

fn scoring_mode_name(storage_format: options::StorageFormat) -> &'static str {
    match storage_format {
        options::StorageFormat::Auto | options::StorageFormat::TurboQuant => "turboquant_lut",
        options::StorageFormat::PqFastScan => "pq_fastscan_lut",
        options::StorageFormat::RaBitQ => "rabitq_binary",
    }
}

fn storage_scoring_multiplier(storage_format: options::StorageFormat) -> f64 {
    match storage_format {
        options::StorageFormat::Auto | options::StorageFormat::TurboQuant => 1.0,
        options::StorageFormat::PqFastScan => 0.65,
        options::StorageFormat::RaBitQ => 0.45,
    }
}

fn rerank_multiplier(rerank: options::RerankMode) -> f64 {
    match rerank.v1_effective() {
        options::RerankMode::Auto | options::RerankMode::Off => 1.0,
        options::RerankMode::HeapF32 | options::RerankMode::SourceColumn => 1.35,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(
        dimensions: u16,
        nlists: u32,
        nprobe: u32,
        total_live_tuples: u64,
    ) -> page::MetadataPage {
        let mut metadata = page::MetadataPage::empty(options::EcIvfOptions {
            nlists: nlists as i32,
            nprobe: nprobe as i32,
            rerank_width: 25,
            training_sample_rows: 1000,
            seed: 7,
            storage_format: options::StorageFormat::TurboQuant,
            rerank: options::RerankMode::HeapF32,
        });
        metadata.dimensions = dimensions;
        metadata.training_version = 1;
        metadata.total_live_tuples = total_live_tuples;
        metadata
    }

    fn default_constants() -> PlannerCostConstants {
        PlannerCostConstants {
            random_page_cost: 4.0,
            seq_page_cost: 1.0,
            cpu_operator_cost: 0.0025,
        }
    }

    #[test]
    fn high_dimensional_posting_cost_uses_quantized_scale() {
        let metadata = metadata(1536, 128, 8, 10_000);
        let estimate =
            estimate_ivf_cost_with_nprobe(&metadata, 1300.0, 10_000.0, default_constants(), 8);

        assert!(estimate.startup_cost.is_finite());
        assert!(estimate.total_cost.is_finite());
        assert!(estimate.total_cost > estimate.startup_cost);
        assert!(
            estimate.total_cost < 1_500.0,
            "quantized nprobe=8 IVF cost should stay below a full-dimensional candidate model: {:?}",
            estimate
        );
    }

    #[test]
    fn cost_increases_with_selected_probe_count() {
        let low_probe = metadata(1536, 128, 8, 10_000);
        let high_probe = metadata(1536, 128, 64, 10_000);
        let low =
            estimate_ivf_cost_with_nprobe(&low_probe, 1300.0, 10_000.0, default_constants(), 8);
        let high =
            estimate_ivf_cost_with_nprobe(&high_probe, 1300.0, 10_000.0, default_constants(), 64);

        assert!(high.total_cost > low.total_cost);
    }
}
