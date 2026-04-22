use std::ptr;
use std::sync::Arc;
#[cfg(any(test, feature = "pg_test"))]
use std::time::Instant;

use hashbrown::{HashMap, HashSet};
use pgrx::{pg_sys, FromDatum, IntoDatum, PgBox};

use crate::quant::grouped_pq::{build_grouped_pq_lut_f32, grouped_pq_score_f32};
use crate::quant::prod::{
    BinarySignNoQjl4BitQuery, Int8ApproxNoQjl4BitQuery, PreparedLutNoQjl4BitQuery, PreparedQuery,
    PreparedTiledLutNoQjl4BitQuery, ProdQuantizer,
};

use super::explain::TqExplainCounters;
use super::graph;
use super::page;
use super::search;
use super::source;
use super::stats::TqStatsCounters;
use super::stream::{GraphPrefetchState, LinearPrefetchState};

const MAX_BOOTSTRAP_FRONTIER_CANDIDATES: usize = 3;
const ADR031_BINARY_PREFILTER_MIN_CANDIDATES: usize = 16;
const ADR031_BINARY_PREFILTER_REJECTIONS: usize = 4;
const ADR031_INLINE_BINARY_WORD_CAPACITY: usize = 24;
const PQ_FASTSCAN_SCAN_WINDOW_ENV: &str = "TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW";
const LEGACY_ADR030_EXPERIMENTAL_SCAN_WINDOW_ENV: &str =
    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW";
const PQ_FASTSCAN_TRAVERSAL_SCORE_MODE_ENV: &str = "TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE";
const LEGACY_ADR030_EXPERIMENTAL_GROUPED_SCORE_MODE_ENV: &str =
    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_GROUPED_SCORE_MODE";
const PQ_FASTSCAN_RERANK_MODE_ENV: &str = "TQVECTOR_PQ_FASTSCAN_RERANK_MODE";
const LEGACY_ADR030_EXPERIMENTAL_RERANK_MODE_ENV: &str =
    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_MODE";
const PQ_FASTSCAN_RERANK_SOURCE_COLUMN_ENV: &str = "TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN";
const LEGACY_ADR030_EXPERIMENTAL_RERANK_SOURCE_COLUMN_ENV: &str =
    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_SOURCE_COLUMN";
const PQ_FASTSCAN_EXACT_TRAVERSAL_ENV: &str = "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL";
const LEGACY_ADR030_EXPERIMENTAL_EXACT_TRAVERSAL_ENV: &str =
    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL";
const PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE_ENV: &str = "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE";
const LEGACY_ADR030_EXPERIMENTAL_EXACT_TRAVERSAL_SCOPE_ENV: &str =
    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE";
const PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT_ENV: &str = "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT";
const LEGACY_ADR030_EXPERIMENTAL_EXACT_TRAVERSAL_LIMIT_ENV: &str =
    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT";
const PQ_FASTSCAN_EXACT_TRAVERSAL_STRATEGY_ENV: &str =
    "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_STRATEGY";
const LEGACY_ADR030_EXPERIMENTAL_EXACT_TRAVERSAL_STRATEGY_ENV: &str =
    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_STRATEGY";
const TURBOQUANT_EXACT_SCORE_MODE_ENV: &str = "TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE";
const TURBOQUANT_TILED_LUT_TILE_SIZE: usize = 512;
pub(crate) const PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW: usize = 64;
const PQ_FASTSCAN_MAX_LIVE_RERANK_WINDOW: usize = 64;
pub(crate) const PQ_FASTSCAN_DEFAULT_TRAVERSAL_SCORE_MODE_NAME: &str = "binary";
pub(crate) const PQ_FASTSCAN_DEFAULT_RERANK_MODE_NAME: &str = "heap_f32";
const PQ_FASTSCAN_EXACT_SCORE_UNAVAILABLE: &str =
    "ec_hnsw PqFastScan exact scoring requires the cold rerank payload path";

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootstrapExpandPolicy {
    ScoreOrder,
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct ScanDebugProfile {
    pub(super) amrescan_total_elapsed_us: u64,
    pub(super) query_decode_elapsed_us: u64,
    pub(super) scan_setup_elapsed_us: u64,
    pub(super) store_query_elapsed_us: u64,
    pub(super) prepare_query_elapsed_us: u64,
    pub(super) reset_state_elapsed_us: u64,
    pub(super) initialize_entry_elapsed_us: u64,
    pub(super) upper_layer_seed_elapsed_us: u64,
    pub(super) layer0_seed_elapsed_us: u64,
    pub(super) stage_ordered_results_elapsed_us: u64,
    pub(super) initial_prefetch_elapsed_us: u64,
    pub(super) frontier_consume_elapsed_us: u64,
    pub(super) graph_result_materialize_elapsed_us: u64,
    pub(super) graph_element_cache_hits: u64,
    pub(super) graph_element_cache_misses: u64,
    pub(super) graph_element_load_elapsed_us: u64,
    pub(super) graph_neighbor_cache_hits: u64,
    pub(super) graph_neighbor_cache_misses: u64,
    pub(super) graph_neighbor_load_elapsed_us: u64,
    pub(super) score_cache_hits: u64,
    pub(super) score_cache_misses: u64,
    pub(super) binary_prefilter_score_calls: u64,
    pub(super) binary_prefilter_score_elapsed_us: u64,
    pub(super) binary_prefilter_survivor_candidates: u64,
    pub(super) candidate_score_calls: u64,
    pub(super) candidate_score_elapsed_us: u64,
    pub(super) grouped_traversal_approx_score_calls: u64,
    pub(super) grouped_traversal_approx_score_elapsed_us: u64,
    pub(super) grouped_traversal_exact_score_calls: u64,
    pub(super) grouped_traversal_exact_score_elapsed_us: u64,
    pub(super) grouped_traversal_budgeted_expansions: u64,
    pub(super) grouped_traversal_budgeted_candidates: u64,
    pub(super) grouped_traversal_budgeted_exact_candidates: u64,
    pub(super) grouped_rerank_quantized_score_calls: u64,
    pub(super) grouped_rerank_quantized_score_elapsed_us: u64,
    pub(super) grouped_rerank_heap_score_calls: u64,
    pub(super) grouped_rerank_heap_score_elapsed_us: u64,
    pub(super) grouped_rerank_heap_rows_fetched: u64,
    pub(super) grouped_rerank_heap_fetch_elapsed_us: u64,
    pub(super) grouped_rerank_heap_decode_elapsed_us: u64,
    pub(super) grouped_rerank_heap_dot_elapsed_us: u64,
}

#[cfg(any(test, feature = "pg_test"))]
fn reset_scan_debug_profile(opaque: &mut TqScanOpaque) {
    opaque.debug_profile = ScanDebugProfile::default();
}

#[cfg(not(any(test, feature = "pg_test")))]
fn reset_scan_debug_profile(_opaque: &mut TqScanOpaque) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_amrescan_total_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.amrescan_total_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_amrescan_total_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_query_decode_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.query_decode_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_query_decode_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_scan_setup_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.scan_setup_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_scan_setup_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_store_query_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.store_query_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_store_query_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_prepare_query_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.prepare_query_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_prepare_query_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_reset_state_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.reset_state_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_reset_state_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_initialize_entry_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.initialize_entry_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_initialize_entry_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_upper_layer_seed_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.upper_layer_seed_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_upper_layer_seed_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_layer0_seed_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.layer0_seed_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_layer0_seed_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_stage_ordered_results_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.stage_ordered_results_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_stage_ordered_results_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CachedHeapTids {
    len: u8,
    tids: [page::ItemPointer; page::HEAPTID_INLINE_CAPACITY],
}

impl CachedHeapTids {
    fn from_iter<I>(heaptids: I) -> Self
    where
        I: IntoIterator<Item = page::ItemPointer>,
    {
        let mut tids = [page::ItemPointer::INVALID; page::HEAPTID_INLINE_CAPACITY];
        let mut len = 0usize;
        for tid in heaptids {
            assert!(
                len < page::HEAPTID_INLINE_CAPACITY,
                "cached heap tids should respect inline tuple capacity"
            );
            tids[len] = tid;
            len += 1;
        }
        Self {
            len: u8::try_from(len).expect("heap tid count should fit in u8"),
            tids,
        }
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn as_slice(&self) -> &[page::ItemPointer] {
        &self.tids[..self.len as usize]
    }
}

impl Default for CachedHeapTids {
    fn default() -> Self {
        Self {
            len: 0,
            tids: [page::ItemPointer::INVALID; page::HEAPTID_INLINE_CAPACITY],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CachedBinaryWords {
    Inline {
        len: u8,
        words: [u64; ADR031_INLINE_BINARY_WORD_CAPACITY],
    },
    Heap(Vec<u64>),
}

impl CachedBinaryWords {
    fn empty() -> Self {
        Self::Inline {
            len: 0,
            words: [0_u64; ADR031_INLINE_BINARY_WORD_CAPACITY],
        }
    }

    fn from_iter<I>(len: usize, words: I) -> Self
    where
        I: IntoIterator<Item = u64>,
    {
        if len <= ADR031_INLINE_BINARY_WORD_CAPACITY {
            let mut inline = [0_u64; ADR031_INLINE_BINARY_WORD_CAPACITY];
            let mut actual_len = 0usize;
            for word in words {
                debug_assert!(
                    actual_len < ADR031_INLINE_BINARY_WORD_CAPACITY,
                    "inline binary-word iterator should stay within capacity"
                );
                inline[actual_len] = word;
                actual_len += 1;
            }
            debug_assert_eq!(
                actual_len, len,
                "binary word iterator should match advertised word count"
            );
            Self::Inline {
                len: u8::try_from(actual_len).expect("inline binary word count should fit in u8"),
                words: inline,
            }
        } else {
            Self::Heap(words.into_iter().collect())
        }
    }

    fn from_vec(words: Vec<u64>) -> Self {
        let len = words.len();
        Self::from_iter(len, words)
    }

    fn as_slice(&self) -> &[u64] {
        match self {
            Self::Inline { len, words } => &words[..*len as usize],
            Self::Heap(words) => words.as_slice(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CachedGraphElement {
    tid: page::ItemPointer,
    level: u8,
    deleted: bool,
    heaptids: CachedHeapTids,
    neighbortid: page::ItemPointer,
    reranktid: Option<page::ItemPointer>,
    binary_words: CachedBinaryWords,
    grouped_search_code: CachedGroupedSearchCode,
}

impl CachedGraphElement {
    fn from_graph_tuple_ref(
        tid: page::ItemPointer,
        element: graph::GraphTupleRef<'_>,
        binary_words: CachedBinaryWords,
    ) -> Self {
        Self {
            tid,
            level: element.level(),
            deleted: element.deleted(),
            heaptids: CachedHeapTids::from_iter(element.collect_heaptids()),
            neighbortid: element.neighbortid(),
            reranktid: element.reranktid(),
            binary_words,
            grouped_search_code: CachedGroupedSearchCode::from_tuple_ref(element),
        }
    }

    fn grouped_score_input(&self) -> Option<GroupedScoreInput<'_>> {
        match (
            self.reranktid,
            self.grouped_search_code.as_slice(),
            self.binary_words.as_slice(),
        ) {
            (Some(reranktid), Some(search_code), binary_words) => Some(GroupedScoreInput {
                reranktid,
                search_code,
                binary_words,
            }),
            _ => None,
        }
    }
}

struct LoadedElementScoreInput {
    gamma: f32,
    code_bytes: Vec<u8>,
}

enum LoadedElementState {
    None,
    ExactScore(f32),
    ExactPayload(LoadedElementScoreInput),
    ExactUnavailable,
}

#[derive(Debug, Clone, PartialEq)]
enum CachedGroupedSearchCode {
    None,
    Bytes(Vec<u8>),
}

impl CachedGroupedSearchCode {
    fn from_tuple_ref(element: graph::GraphTupleRef<'_>) -> Self {
        match element.grouped_search_code() {
            Some(search_code) => Self::Bytes(search_code.to_vec()),
            None => Self::None,
        }
    }

    fn as_slice(&self) -> Option<&[u8]> {
        match self {
            Self::None => None,
            Self::Bytes(search_code) => Some(search_code.as_slice()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GroupedScoreInput<'a> {
    reranktid: page::ItemPointer,
    search_code: &'a [u8],
    binary_words: &'a [u64],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GroupedScoreShape {
    binary_word_count: usize,
    search_code_len: usize,
    rerank_code_len: usize,
}

impl GroupedScoreShape {
    fn from_scan_graph_storage(scan_graph_storage: graph::GraphStorageDescriptor) -> Option<Self> {
        match scan_graph_storage {
            graph::GraphStorageDescriptor::TurboQuant { .. }
            | graph::GraphStorageDescriptor::TurboQuantHotCold(_) => None,
            graph::GraphStorageDescriptor::PqFastScan(layout) => Some(Self {
                binary_word_count: layout.binary_word_count,
                search_code_len: layout.search_code_len,
                rerank_code_len: layout.rerank_code_len,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GroupedScoreCall<'a> {
    shape: GroupedScoreShape,
    input: GroupedScoreInput<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GroupedScoreContext<'a> {
    element_tid: page::ItemPointer,
    call: GroupedScoreCall<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PreparedGroupedScanQuery {
    group_count: usize,
    search_code_len: usize,
    lut_f32: Vec<f32>,
}

impl crate::quant::QueryScorer for PreparedGroupedScanQuery {
    fn score(&self, search_code: &[u8]) -> f32 {
        debug_assert_eq!(
            search_code.len(),
            self.search_code_len,
            "grouped search-code length {} should match prepared grouped query width {}",
            search_code.len(),
            self.search_code_len,
        );
        grouped_pq_score_f32(&self.lut_f32, self.group_count, search_code)
    }
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq)]
struct GroupedScorePayloadView<'a> {
    element_tid: page::ItemPointer,
    reranktid: page::ItemPointer,
    binary_words: &'a [u64],
    search_code: &'a [u8],
    rerank_code_len: usize,
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
#[derive(Debug, Clone, PartialEq)]
struct GroupedScoreRerankPayload<'a> {
    element_tid: page::ItemPointer,
    reranktid: page::ItemPointer,
    binary_words: &'a [u64],
    search_code: &'a [u8],
    rerank_gamma: f32,
    rerank_code: Vec<u8>,
}

enum CandidateScoreDispatch<'a> {
    Exact(LoadedElementState),
    Grouped(GroupedScoreContext<'a>),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupedExactTraversalMode {
    Disabled = 0,
    AllLayers = 1,
    Layer0Only = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupedExactTraversalStrategy {
    Expansion = 0,
    FrontierHead = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupedTraversalScoreMode {
    GroupedPq = 0,
    Binary = 1,
}

impl GroupedTraversalScoreMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::GroupedPq => "pq",
            Self::Binary => "binary",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PqFastScanTraversalScoreModeResolution {
    EnvOverride,
    DefaultBinaryWithBinarySidecar,
    FallbackGroupedPqMissingBinarySidecar,
    NonPqFastScanStorage,
}

impl PqFastScanTraversalScoreModeResolution {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::EnvOverride => "env_override",
            Self::DefaultBinaryWithBinarySidecar => "default_binary_with_binary_sidecar",
            Self::FallbackGroupedPqMissingBinarySidecar => {
                "fallback_grouped_pq_missing_binary_sidecar"
            }
            Self::NonPqFastScanStorage => "non_pq_fastscan_storage",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PqFastScanTraversalScoreModeDecision {
    mode: GroupedTraversalScoreMode,
    pub(crate) resolution: PqFastScanTraversalScoreModeResolution,
}

impl PqFastScanTraversalScoreModeDecision {
    pub(crate) const fn mode_name(self) -> &'static str {
        self.mode.as_str()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TurboQuantExactScoreMode {
    Exact = 0,
    FullLut = 1,
    TiledLut = 2,
    Int8Approx = 3,
}

impl TurboQuantExactScoreMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::FullLut => "full_lut_no_qjl_4bit",
            Self::TiledLut => "tiled_lut_no_qjl_4bit",
            Self::Int8Approx => "int8_approx_no_qjl_4bit",
        }
    }

    const fn uses_lut(self) -> bool {
        matches!(self, Self::FullLut | Self::TiledLut)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupedRerankMode {
    Quantized = 0,
    HeapF32 = 1,
}

impl GroupedRerankMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Quantized => "quantized",
            Self::HeapF32 => "heap_f32",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PqFastScanRerankModeResolution {
    EnvOverride,
    DefaultHeapF32WithIndexedColumn,
    DefaultHeapF32WithRerankSourceColumn,
    DefaultHeapF32WithBuildSourceColumn,
    DefaultQuantizedWithIndexedTqvector,
    DefaultQuantizedTurboQuantStorage,
    NonPqFastScanStorage,
}

impl PqFastScanRerankModeResolution {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::EnvOverride => "env_override",
            Self::DefaultHeapF32WithIndexedColumn => "default_heap_f32_with_indexed_column",
            Self::DefaultHeapF32WithRerankSourceColumn => {
                "default_heap_f32_with_rerank_source_column"
            }
            Self::DefaultHeapF32WithBuildSourceColumn => {
                "default_heap_f32_with_build_source_column"
            }
            Self::DefaultQuantizedWithIndexedTqvector => "default_quantized_with_indexed_tqvector",
            Self::DefaultQuantizedTurboQuantStorage => "default_quantized_turboquant_storage",
            Self::NonPqFastScanStorage => "non_pq_fastscan_storage",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PqFastScanRerankModeDecision {
    mode: GroupedRerankMode,
    pub(crate) resolution: PqFastScanRerankModeResolution,
    pub(crate) source_column: Option<String>,
}

impl PqFastScanRerankModeDecision {
    pub(crate) const fn mode_name(&self) -> &'static str {
        self.mode.as_str()
    }
}

struct BinaryPrefilterCandidate {
    ordinal: usize,
    element: Arc<CachedGraphElement>,
    approx_score: f32,
    loaded_state: LoadedElementState,
}

struct GroupedTraversalCandidate {
    ordinal: usize,
    element: Arc<CachedGraphElement>,
    approx_score: f32,
}

#[cfg(any(test, feature = "pg_test"))]
fn record_initial_prefetch_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.initial_prefetch_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_initial_prefetch_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_frontier_consume_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.frontier_consume_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_frontier_consume_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_graph_result_materialize_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.graph_result_materialize_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_graph_result_materialize_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_graph_element_cache_hit(opaque: &mut TqScanOpaque) {
    opaque.debug_profile.graph_element_cache_hits += 1;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_graph_element_cache_hit(_opaque: &mut TqScanOpaque) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_graph_element_cache_miss_load(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.graph_element_cache_misses += 1;
    opaque.debug_profile.graph_element_load_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_graph_element_cache_miss_load(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_graph_neighbor_cache_hit(opaque: &mut TqScanOpaque) {
    opaque.debug_profile.graph_neighbor_cache_hits += 1;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_graph_neighbor_cache_hit(_opaque: &mut TqScanOpaque) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_graph_neighbor_cache_miss_load(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.graph_neighbor_cache_misses += 1;
    opaque.debug_profile.graph_neighbor_load_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_graph_neighbor_cache_miss_load(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_binary_prefilter_score_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.binary_prefilter_score_calls += 1;
    opaque.debug_profile.binary_prefilter_score_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_binary_prefilter_score_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_binary_prefilter_survivors(opaque: &mut TqScanOpaque, survivor_count: usize) {
    opaque.debug_profile.binary_prefilter_survivor_candidates +=
        u64::try_from(survivor_count).expect("binary prefilter survivor count should fit in u64");
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_binary_prefilter_survivors(_opaque: &mut TqScanOpaque, _survivor_count: usize) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_candidate_score_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.candidate_score_calls += 1;
    opaque.debug_profile.candidate_score_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_candidate_score_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_grouped_traversal_approx_score_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    record_candidate_score_elapsed(opaque, elapsed_us);
    opaque.debug_profile.grouped_traversal_approx_score_calls += 1;
    opaque
        .debug_profile
        .grouped_traversal_approx_score_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_grouped_traversal_approx_score_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_grouped_traversal_exact_score_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.grouped_traversal_exact_score_calls += 1;
    opaque
        .debug_profile
        .grouped_traversal_exact_score_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_grouped_traversal_exact_score_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_grouped_traversal_budget(
    opaque: &mut TqScanOpaque,
    candidate_count: usize,
    exact_candidate_count: usize,
) {
    if candidate_count == 0 {
        return;
    }

    opaque.debug_profile.grouped_traversal_budgeted_expansions += 1;
    opaque.debug_profile.grouped_traversal_budgeted_candidates +=
        u64::try_from(candidate_count).expect("budgeted candidate count should fit in u64");
    opaque
        .debug_profile
        .grouped_traversal_budgeted_exact_candidates += u64::try_from(exact_candidate_count)
        .expect("budgeted exact candidate count should fit in u64");
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_grouped_traversal_budget(
    _opaque: &mut TqScanOpaque,
    _candidate_count: usize,
    _exact_candidate_count: usize,
) {
}

#[cfg(any(test, feature = "pg_test"))]
fn record_grouped_rerank_quantized_score_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.grouped_rerank_quantized_score_calls += 1;
    opaque
        .debug_profile
        .grouped_rerank_quantized_score_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_grouped_rerank_quantized_score_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_grouped_rerank_heap_score_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.grouped_rerank_heap_score_calls += 1;
    opaque.debug_profile.grouped_rerank_heap_score_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_grouped_rerank_heap_score_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_grouped_rerank_heap_fetch(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.grouped_rerank_heap_rows_fetched += 1;
    opaque.debug_profile.grouped_rerank_heap_fetch_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_grouped_rerank_heap_fetch(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_grouped_rerank_heap_decode_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.grouped_rerank_heap_decode_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_grouped_rerank_heap_decode_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_grouped_rerank_heap_dot_elapsed(opaque: &mut TqScanOpaque, elapsed_us: u64) {
    opaque.debug_profile.grouped_rerank_heap_dot_elapsed_us += elapsed_us;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_grouped_rerank_heap_dot_elapsed(_opaque: &mut TqScanOpaque, _elapsed_us: u64) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_score_cache_hit(opaque: &mut TqScanOpaque) {
    opaque.debug_profile.score_cache_hits += 1;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_score_cache_hit(_opaque: &mut TqScanOpaque) {}

#[cfg(any(test, feature = "pg_test"))]
fn record_score_cache_miss(opaque: &mut TqScanOpaque) {
    opaque.debug_profile.score_cache_misses += 1;
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_score_cache_miss(_opaque: &mut TqScanOpaque) {}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_ambeginscan(
    index_relation: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let scan = pg_sys::RelationGetIndexScan(index_relation, nkeys, norderbys);
            if scan.is_null() {
                pgrx::error!("ec_hnsw failed to allocate scan descriptor");
            }

            (*scan).parallel_scan = ptr::null_mut();
            (*scan).opaque = PgBox::<TqScanOpaque>::alloc0().into_pg().cast();
            scan
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_amrescan(
    scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    orderbys: pg_sys::ScanKey,
    norderbys: std::ffi::c_int,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_hnsw amrescan received a null scan descriptor");
            }
            // PostgreSQL may still pass an allocated key buffer for pure
            // ORDER BY scans even when the actual qual count is zero.
            if nkeys != 0 {
                pgrx::error!("ec_hnsw scan does not support index quals yet");
            }
            if norderbys != 1 {
                pgrx::error!("ec_hnsw scan currently requires exactly one ORDER BY query");
            }
            if orderbys.is_null() {
                pgrx::error!("ec_hnsw amrescan received null order-by scan keys");
            }

            #[cfg(any(test, feature = "pg_test"))]
            let amrescan_started = Instant::now();
            let orderby = &*orderbys;
            if (orderby.sk_flags as u32) & pg_sys::SK_ISNULL != 0 {
                pgrx::error!("ec_hnsw scan query must not be NULL");
            }

            #[cfg(any(test, feature = "pg_test"))]
            let query_decode_started = Instant::now();
            let query = Vec::<f32>::from_polymorphic_datum(
                orderby.sk_argument,
                false,
                pg_sys::FLOAT4ARRAYOID,
            )
            .unwrap_or_else(|| pgrx::error!("ec_hnsw scan requires a real[] ORDER BY query"));
            #[cfg(any(test, feature = "pg_test"))]
            let query_decode_elapsed_us = u64::try_from(query_decode_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let query_decode_elapsed_us = 0;
            if query.is_empty() {
                pgrx::error!("ec_hnsw scan query must not be empty");
            }
            if query.len() > u16::MAX as usize {
                pgrx::error!(
                    "ec_hnsw scan query dimension {} exceeds maximum {}",
                    query.len(),
                    u16::MAX
                );
            }

            #[cfg(any(test, feature = "pg_test"))]
            let scan_setup_started = Instant::now();
            let metadata = super::shared::read_metadata_page((*scan).indexRelation);
            let graph_storage = validate_runtime_scan_format((*scan).indexRelation, &metadata)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            if metadata.dimensions != 0 && query.len() != metadata.dimensions as usize {
                pgrx::error!(
                    "ec_hnsw scan query dimension mismatch: index dim {}, query dim {}",
                    metadata.dimensions,
                    query.len()
                );
            }

            (*scan).xs_recheck = false;
            (*scan).xs_recheckorderby = false;
            (*scan).xs_orderbyvals = ptr::null_mut();
            (*scan).xs_orderbynulls = ptr::null_mut();

            let index_options = super::options::relation_options((*scan).indexRelation);
            let opaque = &mut *(*scan).opaque.cast::<TqScanOpaque>();
            bind_parallel_scan_state(scan, opaque);
            if opaque.rescan_called {
                finalize_scan_stats(opaque);
                flush_scan_stats(opaque);
            }
            opaque.rescan_called = true;
            opaque.scan_dimensions = metadata.dimensions;
            opaque.scan_m = metadata.m;
            opaque.scan_bits = metadata.bits;
            opaque.scan_seed = metadata.seed;
            opaque.scan_code_len = if metadata.dimensions == 0 {
                0
            } else {
                crate::code_len(metadata.dimensions as usize, metadata.bits)
            };
            opaque.scan_graph_storage = graph_storage;
            opaque.grouped_live_rerank_window = if matches!(
                opaque.scan_graph_storage,
                graph::GraphStorageDescriptor::PqFastScan(_)
            ) {
                u8::try_from(resolve_grouped_live_rerank_window())
                    .expect("grouped live rerank window should fit in u8")
            } else {
                u8::try_from(PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW)
                    .expect("default grouped live rerank window should fit in u8")
            };
            opaque.grouped_traversal_score_mode = if matches!(
                opaque.scan_graph_storage,
                graph::GraphStorageDescriptor::PqFastScan(_)
            ) {
                resolve_grouped_traversal_score_mode(opaque.scan_graph_storage)
            } else {
                GroupedTraversalScoreMode::GroupedPq
            };
            opaque.grouped_exact_traversal_mode = if matches!(
                opaque.scan_graph_storage,
                graph::GraphStorageDescriptor::PqFastScan(_)
            ) {
                resolve_grouped_exact_traversal_mode()
            } else {
                GroupedExactTraversalMode::Disabled
            };
            opaque.grouped_exact_traversal_strategy =
                if opaque.grouped_exact_traversal_mode == GroupedExactTraversalMode::Disabled {
                    GroupedExactTraversalStrategy::Expansion
                } else {
                    resolve_grouped_exact_traversal_strategy(opaque.grouped_exact_traversal_mode)
                };
            opaque.grouped_exact_traversal_limit =
                if opaque.grouped_exact_traversal_mode == GroupedExactTraversalMode::Disabled {
                    0
                } else {
                    resolve_grouped_exact_traversal_limit()
                };
            configure_grouped_heap_rerank_state(scan, opaque, &index_options);
            opaque.scan_block_count = pg_sys::RelationGetNumberOfBlocksInFork(
                (*scan).indexRelation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
            );
            let scan_tuning = super::options::resolve_scan_tuning(&index_options);
            opaque.bootstrap_frontier_limit = resolve_bootstrap_frontier_limit(
                scan_tuning,
                opaque.parallel_scan_worker_slot_count,
            );
            #[cfg(any(test, feature = "pg_test"))]
            let scan_setup_elapsed_us = u64::try_from(scan_setup_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let scan_setup_elapsed_us = 0;
            record_query_decode_elapsed(opaque, query_decode_elapsed_us);
            record_scan_setup_elapsed(opaque, scan_setup_elapsed_us);
            #[cfg(any(test, feature = "pg_test"))]
            let store_query_started = Instant::now();
            store_scan_query(opaque, &query);
            #[cfg(any(test, feature = "pg_test"))]
            let store_query_elapsed_us = u64::try_from(store_query_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let store_query_elapsed_us = 0;
            record_store_query_elapsed(opaque, store_query_elapsed_us);
            opaque.explain_counters.reset();
            opaque.stats_delta.reset();
            super::stats::record_scan_started();
            opaque.stats_delta.record_scan_started();
            #[cfg(any(test, feature = "pg_test"))]
            let prepare_started = Instant::now();
            store_scan_prepared_query(opaque, &query, &metadata);
            store_grouped_scan_query((*scan).indexRelation, opaque, &metadata);
            #[cfg(any(test, feature = "pg_test"))]
            let prepare_elapsed_us = u64::try_from(prepare_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let prepare_elapsed_us = 0;
            record_prepare_query_elapsed(opaque, prepare_elapsed_us);
            #[cfg(any(test, feature = "pg_test"))]
            let reset_started = Instant::now();
            reset_scan_position(opaque);
            reset_linear_prefetch_state(opaque);
            reset_graph_prefetch_state(opaque);
            #[cfg(feature = "pg18")]
            {
                let graph_stream = ensure_graph_read_stream((*scan).indexRelation, opaque);
                let linear_stream = ensure_linear_read_stream((*scan).indexRelation, opaque);
                pg_sys::read_stream_reset(graph_stream);
                pg_sys::read_stream_reset(linear_stream);
            }
            #[cfg(any(test, feature = "pg_test"))]
            let reset_elapsed_us = u64::try_from(reset_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let reset_elapsed_us = 0;
            record_reset_state_elapsed(opaque, reset_elapsed_us);
            #[cfg(any(test, feature = "pg_test"))]
            let initialize_started = Instant::now();
            initialize_scan_entry_candidate(
                (*scan).indexRelation,
                (*scan).heapRelation,
                opaque,
                &metadata,
            );
            #[cfg(any(test, feature = "pg_test"))]
            let initialize_elapsed_us = u64::try_from(initialize_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let initialize_elapsed_us = 0;
            record_initialize_entry_elapsed(opaque, initialize_elapsed_us);
            let opaque_ptr = opaque as *mut TqScanOpaque;
            #[cfg(any(test, feature = "pg_test"))]
            let prefetch_started = Instant::now();
            if !graph_traversal_cursor(opaque)
                .ensure_prefetched_output((*scan).indexRelation, opaque_ptr)
            {
                enter_linear_fallback_phase(opaque);
                reset_linear_prefetch_state(opaque);
            }
            #[cfg(any(test, feature = "pg_test"))]
            let initial_prefetch_elapsed_us = u64::try_from(prefetch_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let initial_prefetch_elapsed_us = 0;
            record_initial_prefetch_elapsed(opaque, initial_prefetch_elapsed_us);
            #[cfg(any(test, feature = "pg_test"))]
            let amrescan_total_elapsed_us = u64::try_from(amrescan_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let amrescan_total_elapsed_us = 0;
            record_amrescan_total_elapsed(opaque, amrescan_total_elapsed_us);
            sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
        })
    }
}

fn validate_runtime_scan_format(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> Result<graph::GraphStorageDescriptor, String> {
    unsafe { graph::GraphStorageDescriptor::from_index_relation(index_relation, metadata) }
}

const INVALID_PARALLEL_SCAN_WORKER_SLOT: u32 = u32::MAX;

fn saturating_u32_from_usize(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn parallel_item_pointer(tid: page::ItemPointer) -> super::parallel::EcParallelItemPointer {
    super::parallel::EcParallelItemPointer {
        block_number: tid.block_number,
        offset_number: tid.offset_number,
    }
}

fn parallel_scan_worker_phase(phase: ScanExecutionPhase) -> u32 {
    match phase {
        ScanExecutionPhase::GraphTraversal => {
            super::parallel::EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL
        }
        ScanExecutionPhase::LinearFallback => {
            super::parallel::EC_PARALLEL_WORKER_PHASE_LINEAR_FALLBACK
        }
        ScanExecutionPhase::Exhausted => super::parallel::EC_PARALLEL_WORKER_PHASE_EXHAUSTED,
    }
}

fn published_parallel_worker_result_state(
    opaque: &TqScanOpaque,
) -> Option<(ScanExecutionPhase, &ScanResultState)> {
    let active_result_state = active_result_state_ref(opaque);
    if active_result_state.current().has_element() {
        return Some((opaque.execution_phase, active_result_state));
    }

    best_deferred_parallel_blocked_output(opaque)
        .map(|deferred| (deferred.source_phase, &deferred.state))
}

fn publish_parallel_scan_worker_slot_snapshot(opaque: &TqScanOpaque) {
    if opaque.parallel_scan_state.is_null()
        || opaque.parallel_scan_worker_slot_index == INVALID_PARALLEL_SCAN_WORKER_SLOT
    {
        return;
    }

    let active_result_state = active_result_state_ref(opaque);
    let visible_result_state = published_parallel_worker_result_state(opaque);
    let published_phase = visible_result_state
        .map(|(phase, _)| phase)
        .unwrap_or(opaque.execution_phase);
    let published_pending_count = visible_result_state
        .map(|(_, state)| u32::from(state.pending_count()))
        .unwrap_or(0);
    let published_has_current = visible_result_state
        .map(|(_, state)| state.current().has_element())
        .unwrap_or(false);
    let scheduler_frontier_len = if opaque.bootstrap_expansion.is_null() {
        0
    } else {
        saturating_u32_from_usize(unsafe { &*opaque.bootstrap_expansion }.frontier_len())
    };
    let visited_count = if opaque.visited_tids.is_null() {
        0
    } else {
        saturating_u32_from_usize(unsafe { &*opaque.visited_tids }.len())
    };
    let emitted_count = if opaque.emitted_result_tids.is_null() {
        0
    } else {
        saturating_u32_from_usize(unsafe { &*opaque.emitted_result_tids }.len())
    };
    let published_blocker = opaque.parallel_owned_output_blocker.or_else(|| {
        if opaque.parallel_local_only_output_active {
            return opaque
                .retained_parallel_owned_output_blocker
                .map(|retained| retained.blocker);
        }

        best_deferred_parallel_blocked_output(opaque)
            .and_then(|deferred| deferred.retained_blocker)
            .map(|retained| retained.blocker)
    });
    let (
        owned_output_blocker_kind,
        owned_output_blocker_slot_index,
        owned_output_blocker_generation,
    ) = published_blocker
        .map(|blocker| {
            (
                super::parallel::owned_output_blocker_kind_code(blocker.kind),
                blocker.slot_index,
                blocker.generation,
            )
        })
        .unwrap_or((
            super::parallel::EC_PARALLEL_OWNED_OUTPUT_BLOCKER_NONE,
            None,
            0,
        ));

    let snapshot = super::parallel::EcParallelWorkerSlotRuntimeSnapshot {
        execution_phase: parallel_scan_worker_phase(published_phase),
        scan_dimensions: u32::from(opaque.scan_dimensions),
        bootstrap_frontier_limit: saturating_u32_from_usize(opaque.bootstrap_frontier_limit),
        visible_frontier_len: saturating_u32_from_usize(visible_frontier_ref(opaque).len()),
        scheduler_frontier_len,
        visited_count,
        emitted_count,
        active_result_pending_count: published_pending_count,
        active_result_has_current: published_has_current,
        owned_output_blocker_kind,
        owned_output_blocker_slot_index,
        owned_output_blocker_generation,
    };

    match unsafe {
        super::parallel::publish_parallel_scan_worker_slot_runtime_snapshot(
            opaque.parallel_scan_state,
            opaque.parallel_scan_worker_slot_index,
            opaque.parallel_scan_rescan_epoch,
            snapshot,
        )
    } {
        Ok(_) => {}
        Err(err) => pgrx::error!("ec_hnsw parallel scan snapshot publish failed: {err}"),
    }

    let current_result = active_result_state.current();
    if current_result.has_element() && !opaque.parallel_local_only_output_active {
        let pending_heap_tids = std::array::from_fn(|index| {
            active_result_state
                .pending_heap_tids()
                .get(index)
                .copied()
                .map(parallel_item_pointer)
                .unwrap_or(super::parallel::EcParallelItemPointer::INVALID)
        });
        let next_pending_heap_tid = pending_heap_tids
            .get(usize::from(active_result_state.pending_index()))
            .copied()
            .filter(|tid| tid.is_valid())
            .unwrap_or_else(|| {
                if current_result.heap_tid() == page::ItemPointer::INVALID {
                    super::parallel::EcParallelItemPointer::INVALID
                } else {
                    parallel_item_pointer(current_result.heap_tid())
                }
            });
        let result_snapshot = super::parallel::EcParallelCoordinatorResultSlotRuntimeSnapshot {
            element_tid: parallel_item_pointer(current_result.element_tid()),
            heap_tid: next_pending_heap_tid,
            score: current_result.score(),
            approx_score: current_result
                .approx_score_valid()
                .then_some(current_result.approx_score()),
            comparison_score: current_result
                .comparison_score_valid()
                .then_some(current_result.comparison_score()),
            approx_rank_base: current_result
                .approx_rank_valid()
                .then_some(current_result.approx_rank_base()),
            pending_count: u32::from(active_result_state.pending_count()),
            pending_index: u32::from(active_result_state.pending_index()),
            pending_heap_tids,
        };

        match unsafe {
            super::parallel::publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
                opaque.parallel_scan_rescan_epoch,
                result_snapshot,
            )
        } {
            Ok(_) => {}
            Err(err) => {
                pgrx::error!("ec_hnsw parallel scan coordinator-result publish failed: {err}")
            }
        }
    } else {
        match unsafe {
            super::parallel::clear_parallel_scan_coordinator_result_slot_runtime_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
                opaque.parallel_scan_rescan_epoch,
            )
        } {
            Ok(_) => {}
            Err(err) => {
                pgrx::error!("ec_hnsw parallel scan coordinator-result clear failed: {err}")
            }
        }
    }
}

fn sync_and_publish_parallel_scan_worker_slot_snapshot(opaque: &mut TqScanOpaque) {
    reconcile_parallel_owner_progress_from_shared_slot(opaque);
    publish_parallel_scan_worker_slot_snapshot(opaque);
}

fn resolve_bootstrap_frontier_limit(
    tuning: super::options::ScanTuning,
    parallel_worker_slot_count: u32,
) -> usize {
    usize::try_from(super::options::resolve_parallel_scan_ef_search(
        tuning,
        parallel_worker_slot_count,
    ))
    .expect("ef_search should fit in usize")
    .max(1)
}

fn release_parallel_scan_state(opaque: &mut TqScanOpaque) {
    if opaque.parallel_scan_state.is_null()
        || opaque.parallel_scan_worker_slot_index == INVALID_PARALLEL_SCAN_WORKER_SLOT
    {
        return;
    }

    match unsafe {
        super::parallel::release_parallel_scan_worker_slot(
            opaque.parallel_scan_state,
            opaque.parallel_scan_worker_slot_index,
            opaque.parallel_scan_rescan_epoch,
        )
    } {
        Ok(_) => {}
        Err(err) => pgrx::error!("ec_hnsw parallel scan release failed: {err}"),
    }
}

fn clear_parallel_scan_state(opaque: &mut TqScanOpaque) {
    release_parallel_scan_state(opaque);
    opaque.parallel_scan_state = ptr::null_mut();
    opaque.parallel_scan_rescan_epoch = 0;
    opaque.parallel_scan_worker_slot_count = 0;
    opaque.parallel_scan_worker_slot_index = INVALID_PARALLEL_SCAN_WORKER_SLOT;
    opaque.parallel_owned_output_blocker = None;
    opaque.retained_parallel_owned_output_blocker = None;
    opaque.parallel_local_only_output_active = false;
}

fn bind_parallel_scan_state(scan: pg_sys::IndexScanDesc, opaque: &mut TqScanOpaque) {
    clear_parallel_scan_state(opaque);
    if scan.is_null() {
        return;
    }

    match unsafe { super::parallel::parallel_scan_attachment((*scan).parallel_scan) } {
        Ok(Some(attachment)) => {
            let worker_slot_index =
                unsafe { super::parallel::claim_parallel_scan_worker_slot(&attachment) }
                    .unwrap_or_else(|err| {
                        pgrx::error!("ec_hnsw parallel scan claim failed: {err}")
                    });
            opaque.parallel_scan_state = attachment.state;
            opaque.parallel_scan_rescan_epoch = attachment.rescan_epoch;
            opaque.parallel_scan_worker_slot_count = attachment.worker_slot_count;
            opaque.parallel_scan_worker_slot_index = worker_slot_index;
            opaque.parallel_owned_output_blocker = None;
            opaque.retained_parallel_owned_output_blocker = None;
            opaque.parallel_local_only_output_active = false;
            sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
        }
        Ok(None) => clear_parallel_scan_state(opaque),
        Err(err) => pgrx::error!("ec_hnsw parallel scan attach failed: {err}"),
    }
}

fn pq_fastscan_env_var(canonical: &str, legacy: &str) -> Option<std::ffi::OsString> {
    std::env::var_os(canonical).or_else(|| std::env::var_os(legacy))
}

fn pq_fastscan_exact_traversal_enabled() -> bool {
    pq_fastscan_env_var(
        PQ_FASTSCAN_EXACT_TRAVERSAL_ENV,
        LEGACY_ADR030_EXPERIMENTAL_EXACT_TRAVERSAL_ENV,
    )
    .is_some()
}

pub(crate) fn resolve_pq_fastscan_traversal_score_mode_decision(
    graph_storage: graph::GraphStorageDescriptor,
) -> PqFastScanTraversalScoreModeDecision {
    let Some(raw_mode) = pq_fastscan_env_var(
        PQ_FASTSCAN_TRAVERSAL_SCORE_MODE_ENV,
        LEGACY_ADR030_EXPERIMENTAL_GROUPED_SCORE_MODE_ENV,
    ) else {
        return match graph_storage {
            graph::GraphStorageDescriptor::PqFastScan(layout) if layout.binary_word_count > 0 => {
                PqFastScanTraversalScoreModeDecision {
                    mode: GroupedTraversalScoreMode::Binary,
                    resolution:
                        PqFastScanTraversalScoreModeResolution::DefaultBinaryWithBinarySidecar,
                }
            }
            graph::GraphStorageDescriptor::PqFastScan(_) => PqFastScanTraversalScoreModeDecision {
                mode: GroupedTraversalScoreMode::GroupedPq,
                resolution:
                    PqFastScanTraversalScoreModeResolution::FallbackGroupedPqMissingBinarySidecar,
            },
            graph::GraphStorageDescriptor::TurboQuant { .. }
            | graph::GraphStorageDescriptor::TurboQuantHotCold(_) => {
                PqFastScanTraversalScoreModeDecision {
                    mode: GroupedTraversalScoreMode::GroupedPq,
                    resolution: PqFastScanTraversalScoreModeResolution::NonPqFastScanStorage,
                }
            }
        };
    };

    let mode = match raw_mode.to_string_lossy().as_ref() {
        "pq" => GroupedTraversalScoreMode::GroupedPq,
        "binary" => GroupedTraversalScoreMode::Binary,
        other => pgrx::error!(
            "ec_hnsw PqFastScan traversal score mode must be one of [pq, binary], got {:?}",
            other
        ),
    };

    PqFastScanTraversalScoreModeDecision {
        mode,
        resolution: PqFastScanTraversalScoreModeResolution::EnvOverride,
    }
}

fn resolve_grouped_traversal_score_mode(
    graph_storage: graph::GraphStorageDescriptor,
) -> GroupedTraversalScoreMode {
    resolve_pq_fastscan_traversal_score_mode_decision(graph_storage).mode
}

fn grouped_binary_traversal_score_enabled(opaque: &TqScanOpaque) -> bool {
    matches!(
        opaque.scan_graph_storage,
        graph::GraphStorageDescriptor::PqFastScan(_)
    ) && opaque.grouped_traversal_score_mode == GroupedTraversalScoreMode::Binary
}

fn turboquant_scan_storage(graph_storage: graph::GraphStorageDescriptor) -> bool {
    matches!(
        graph_storage,
        graph::GraphStorageDescriptor::TurboQuant { .. }
            | graph::GraphStorageDescriptor::TurboQuantHotCold(_)
    )
}

fn resolve_turboquant_exact_score_mode() -> TurboQuantExactScoreMode {
    let Some(raw_mode) = std::env::var_os(TURBOQUANT_EXACT_SCORE_MODE_ENV) else {
        return TurboQuantExactScoreMode::Exact;
    };

    match raw_mode.to_string_lossy().as_ref() {
        "exact" => TurboQuantExactScoreMode::Exact,
        "full_lut" => TurboQuantExactScoreMode::FullLut,
        "tiled_lut" => TurboQuantExactScoreMode::TiledLut,
        "int8_approx" => TurboQuantExactScoreMode::Int8Approx,
        other => pgrx::error!(
            "ec_hnsw TurboQuant exact score mode must be one of [exact, full_lut, tiled_lut, int8_approx], got {:?}",
            other
        ),
    }
}

fn turboquant_non_default_exact_score_enabled(opaque: &TqScanOpaque) -> bool {
    turboquant_scan_storage(opaque.scan_graph_storage)
        && opaque.turboquant_exact_score_mode != TurboQuantExactScoreMode::Exact
}

pub(super) fn turboquant_exact_score_mode_name(opaque: &TqScanOpaque) -> &'static str {
    if turboquant_non_default_exact_score_enabled(opaque) {
        opaque.turboquant_exact_score_mode.as_str()
    } else if opaque.cached_quantizer.is_null() {
        TurboQuantExactScoreMode::Exact.as_str()
    } else {
        unsafe { &*opaque.cached_quantizer }.exact_score_mode_name()
    }
}

pub(super) fn turboquant_exact_score_uses_lut(opaque: &TqScanOpaque) -> bool {
    if turboquant_non_default_exact_score_enabled(opaque) {
        opaque.turboquant_exact_score_mode.uses_lut()
    } else {
        !opaque.cached_quantizer.is_null()
            && unsafe { &*opaque.cached_quantizer }.exact_score_uses_lut()
    }
}

pub(super) fn turboquant_exact_score_uses_qjl(opaque: &TqScanOpaque) -> bool {
    !turboquant_non_default_exact_score_enabled(opaque)
        && !opaque.cached_quantizer.is_null()
        && unsafe { &*opaque.cached_quantizer }.exact_score_uses_qjl()
}

unsafe fn index_has_default_heap_f32_source(index_relation: pg_sys::Relation) -> bool {
    let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        return false;
    }
    let heap_relation =
        unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let indexed_attribute = unsafe {
        source::resolve_indexed_vector_attribute(heap_relation, index_relation, "indexed column")
    };
    unsafe { pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    matches!(indexed_attribute.kind, source::IndexedVectorKind::Ecvector)
}

fn default_grouped_rerank_mode(
    index_options: &super::options::TqHnswOptions,
    has_default_heap_f32_source: bool,
) -> GroupedRerankMode {
    if matches!(
        index_options.storage_format,
        super::options::StorageFormat::PqFastScan
    ) && (has_default_heap_f32_source
        || index_options.rerank_source_column.is_some()
        || index_options.build_source_column.is_some())
    {
        GroupedRerankMode::HeapF32
    } else {
        GroupedRerankMode::Quantized
    }
}

fn default_grouped_rerank_mode_resolution(
    index_options: &super::options::TqHnswOptions,
    has_default_heap_f32_source: bool,
) -> PqFastScanRerankModeResolution {
    match index_options.storage_format {
        super::options::StorageFormat::PqFastScan => {
            if index_options.rerank_source_column.is_some() {
                PqFastScanRerankModeResolution::DefaultHeapF32WithRerankSourceColumn
            } else if index_options.build_source_column.is_some() {
                PqFastScanRerankModeResolution::DefaultHeapF32WithBuildSourceColumn
            } else if has_default_heap_f32_source {
                PqFastScanRerankModeResolution::DefaultHeapF32WithIndexedColumn
            } else {
                PqFastScanRerankModeResolution::DefaultQuantizedWithIndexedTqvector
            }
        }
        super::options::StorageFormat::TurboQuant => {
            PqFastScanRerankModeResolution::DefaultQuantizedTurboQuantStorage
        }
    }
}

fn effective_grouped_rerank_source_column(
    index_options: &super::options::TqHnswOptions,
    mode: GroupedRerankMode,
) -> Option<String> {
    if mode != GroupedRerankMode::HeapF32 {
        return None;
    }

    pq_fastscan_env_var(
        PQ_FASTSCAN_RERANK_SOURCE_COLUMN_ENV,
        LEGACY_ADR030_EXPERIMENTAL_RERANK_SOURCE_COLUMN_ENV,
    )
    .map(|value| value.to_string_lossy().into_owned())
    .or_else(|| index_options.rerank_source_column.clone())
    .or_else(|| index_options.build_source_column.clone())
}

fn resolve_grouped_rerank_mode_decision(
    index_relation: pg_sys::Relation,
    index_options: &super::options::TqHnswOptions,
) -> PqFastScanRerankModeDecision {
    let has_default_heap_f32_source = unsafe { index_has_default_heap_f32_source(index_relation) };
    let Some(raw_mode) = pq_fastscan_env_var(
        PQ_FASTSCAN_RERANK_MODE_ENV,
        LEGACY_ADR030_EXPERIMENTAL_RERANK_MODE_ENV,
    ) else {
        let mode = default_grouped_rerank_mode(index_options, has_default_heap_f32_source);
        return PqFastScanRerankModeDecision {
            mode,
            resolution: default_grouped_rerank_mode_resolution(
                index_options,
                has_default_heap_f32_source,
            ),
            source_column: effective_grouped_rerank_source_column(index_options, mode),
        };
    };

    let mode = match raw_mode.to_string_lossy().as_ref() {
        "quantized" => GroupedRerankMode::Quantized,
        "heap_f32" => GroupedRerankMode::HeapF32,
        other => pgrx::error!(
            "ec_hnsw grouped rerank mode must be one of [quantized, heap_f32], got {:?}",
            other
        ),
    };

    PqFastScanRerankModeDecision {
        mode,
        resolution: PqFastScanRerankModeResolution::EnvOverride,
        source_column: effective_grouped_rerank_source_column(index_options, mode),
    }
}

pub(crate) unsafe fn resolve_pq_fastscan_rerank_mode_decision(
    index_relation: pg_sys::Relation,
    graph_storage: graph::GraphStorageDescriptor,
) -> PqFastScanRerankModeDecision {
    if !matches!(graph_storage, graph::GraphStorageDescriptor::PqFastScan(_)) {
        return PqFastScanRerankModeDecision {
            mode: GroupedRerankMode::Quantized,
            resolution: PqFastScanRerankModeResolution::NonPqFastScanStorage,
            source_column: None,
        };
    }

    let index_options = unsafe { super::options::relation_options(index_relation) };
    resolve_grouped_rerank_mode_decision(index_relation, &index_options)
}

fn resolve_grouped_rerank_mode(
    index_relation: pg_sys::Relation,
    index_options: &super::options::TqHnswOptions,
) -> GroupedRerankMode {
    resolve_grouped_rerank_mode_decision(index_relation, index_options).mode
}

fn grouped_heap_rerank_enabled(opaque: &TqScanOpaque) -> bool {
    opaque.grouped_rerank_mode == GroupedRerankMode::HeapF32
}

fn turboquant_binary_live_rerank_enabled(opaque: &TqScanOpaque) -> bool {
    matches!(
        opaque.scan_graph_storage,
        graph::GraphStorageDescriptor::TurboQuant { .. }
            | graph::GraphStorageDescriptor::TurboQuantHotCold(_)
    ) && binary_sign_query(opaque).is_some()
}

fn resolve_grouped_exact_traversal_mode() -> GroupedExactTraversalMode {
    if !pq_fastscan_exact_traversal_enabled() {
        return GroupedExactTraversalMode::Disabled;
    }

    let Some(raw_scope) = pq_fastscan_env_var(
        PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE_ENV,
        LEGACY_ADR030_EXPERIMENTAL_EXACT_TRAVERSAL_SCOPE_ENV,
    ) else {
        return GroupedExactTraversalMode::AllLayers;
    };

    match raw_scope.to_string_lossy().as_ref() {
        "all" => GroupedExactTraversalMode::AllLayers,
        "layer0" => GroupedExactTraversalMode::Layer0Only,
        other => pgrx::error!(
            "ec_hnsw PqFastScan exact traversal scope must be one of [all, layer0], got {:?}",
            other
        ),
    }
}

fn grouped_exact_traversal_enabled_for_layer(mode: GroupedExactTraversalMode, layer: u8) -> bool {
    match mode {
        GroupedExactTraversalMode::Disabled => false,
        GroupedExactTraversalMode::AllLayers => true,
        GroupedExactTraversalMode::Layer0Only => layer == 0,
    }
}

fn resolve_grouped_exact_traversal_strategy(
    mode: GroupedExactTraversalMode,
) -> GroupedExactTraversalStrategy {
    let Some(raw_strategy) = pq_fastscan_env_var(
        PQ_FASTSCAN_EXACT_TRAVERSAL_STRATEGY_ENV,
        LEGACY_ADR030_EXPERIMENTAL_EXACT_TRAVERSAL_STRATEGY_ENV,
    ) else {
        return GroupedExactTraversalStrategy::Expansion;
    };

    match raw_strategy.to_string_lossy().as_ref() {
        "expansion" => GroupedExactTraversalStrategy::Expansion,
        "frontier_head" => {
            if mode != GroupedExactTraversalMode::Layer0Only {
                pgrx::error!(
                    "ec_hnsw PqFastScan exact traversal strategy frontier_head requires scope layer0"
                );
            }
            GroupedExactTraversalStrategy::FrontierHead
        }
        other => pgrx::error!(
            "ec_hnsw PqFastScan exact traversal strategy must be one of [expansion, frontier_head], got {:?}",
            other
        ),
    }
}

fn resolve_grouped_exact_traversal_limit() -> u8 {
    let Some(raw_limit) = pq_fastscan_env_var(
        PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT_ENV,
        LEGACY_ADR030_EXPERIMENTAL_EXACT_TRAVERSAL_LIMIT_ENV,
    ) else {
        return 0;
    };

    let raw_limit = raw_limit.to_string_lossy();
    let parsed_limit = raw_limit.parse::<u8>().unwrap_or_else(|_| {
        pgrx::error!(
            "ec_hnsw PqFastScan exact traversal limit must be a positive integer, got {}",
            raw_limit
        )
    });
    if parsed_limit == 0 {
        pgrx::error!(
            "ec_hnsw PqFastScan exact traversal limit must be a positive integer, got {}",
            raw_limit
        );
    }
    parsed_limit
}

fn grouped_exact_traversal_candidate_budget_for_layer(
    opaque: &TqScanOpaque,
    layer: u8,
) -> Option<usize> {
    if opaque.grouped_exact_traversal_strategy != GroupedExactTraversalStrategy::Expansion {
        return None;
    }
    if !grouped_exact_traversal_enabled_for_layer(opaque.grouped_exact_traversal_mode, layer) {
        return None;
    }
    match opaque.grouped_exact_traversal_limit {
        0 => None,
        limit => Some(usize::from(limit)),
    }
}

fn grouped_exact_traversal_full_candidate_scoring_for_layer(
    opaque: &TqScanOpaque,
    layer: u8,
) -> bool {
    opaque.grouped_exact_traversal_strategy == GroupedExactTraversalStrategy::Expansion
        && opaque.grouped_exact_traversal_limit == 0
        && grouped_exact_traversal_enabled_for_layer(opaque.grouped_exact_traversal_mode, layer)
}

fn grouped_exact_traversal_frontier_head_enabled(opaque: &TqScanOpaque) -> bool {
    opaque.grouped_exact_traversal_strategy == GroupedExactTraversalStrategy::FrontierHead
        && opaque.grouped_exact_traversal_mode == GroupedExactTraversalMode::Layer0Only
}

fn resolve_grouped_live_rerank_window() -> usize {
    let Some(raw_window) = pq_fastscan_env_var(
        PQ_FASTSCAN_SCAN_WINDOW_ENV,
        LEGACY_ADR030_EXPERIMENTAL_SCAN_WINDOW_ENV,
    ) else {
        return PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW;
    };

    let raw_window = raw_window.to_string_lossy();
    let parsed_window = raw_window.parse::<usize>().unwrap_or_else(|_| {
        pgrx::error!(
            "ec_hnsw PqFastScan live rerank window must be an integer between 1 and {}, got {:?}",
            PQ_FASTSCAN_MAX_LIVE_RERANK_WINDOW,
            raw_window
        )
    });
    if !(1..=PQ_FASTSCAN_MAX_LIVE_RERANK_WINDOW).contains(&parsed_window) {
        pgrx::error!(
            "ec_hnsw PqFastScan live rerank window must be between 1 and {}, got {}",
            PQ_FASTSCAN_MAX_LIVE_RERANK_WINDOW,
            parsed_window
        );
    }
    parsed_window
}

unsafe fn resolve_scan_heap_relation(scan: pg_sys::IndexScanDesc) -> (pg_sys::Relation, bool) {
    if !unsafe { (*scan).heapRelation }.is_null() {
        return (unsafe { (*scan).heapRelation }, false);
    }

    let heap_oid = unsafe { pg_sys::IndexGetRelation((*(*scan).indexRelation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        pgrx::error!("ec_hnsw grouped heap-f32 rerank could not resolve heap relation");
    }
    (
        unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) },
        true,
    )
}

unsafe fn resolve_scan_snapshot(scan: pg_sys::IndexScanDesc) -> (pg_sys::Snapshot, bool) {
    if !unsafe { (*scan).xs_snapshot }.is_null() {
        return (unsafe { (*scan).xs_snapshot }, false);
    }

    let active_snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if !active_snapshot.is_null() {
        return (active_snapshot, false);
    }

    let registered_snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot()) };
    if registered_snapshot.is_null() {
        pgrx::error!("ec_hnsw grouped heap-f32 rerank could not resolve an active snapshot");
    }
    (registered_snapshot, true)
}

unsafe fn free_grouped_heap_rerank_state(opaque: &mut TqScanOpaque) {
    if !opaque.grouped_heap_rerank_slot.is_null() {
        unsafe { pg_sys::ExecDropSingleTupleTableSlot(opaque.grouped_heap_rerank_slot) };
        opaque.grouped_heap_rerank_slot = ptr::null_mut();
    }
    if opaque.grouped_heap_rerank_snapshot_owned && !opaque.grouped_heap_rerank_snapshot.is_null() {
        unsafe { pg_sys::UnregisterSnapshot(opaque.grouped_heap_rerank_snapshot) };
    }
    opaque.grouped_heap_rerank_snapshot = ptr::null_mut();
    opaque.grouped_heap_rerank_snapshot_owned = false;
    if opaque.grouped_heap_rerank_relation_owned && !opaque.grouped_heap_rerank_relation.is_null() {
        unsafe {
            pg_sys::table_close(
                opaque.grouped_heap_rerank_relation,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )
        };
    }
    opaque.grouped_heap_rerank_relation = ptr::null_mut();
    opaque.grouped_heap_rerank_relation_owned = false;
    opaque.grouped_heap_rerank_source_attnum = 0;
    opaque.grouped_heap_rerank_source_kind = source::SourceDatumKind::Unknown;
}

unsafe fn configure_grouped_heap_rerank_state(
    scan: pg_sys::IndexScanDesc,
    opaque: &mut TqScanOpaque,
    index_options: &super::options::TqHnswOptions,
) {
    unsafe { free_grouped_heap_rerank_state(opaque) };
    let rerank = resolve_grouped_rerank_mode_decision((*scan).indexRelation, index_options);
    opaque.grouped_rerank_mode = rerank.mode;

    if !grouped_heap_rerank_enabled(opaque) {
        return;
    }

    let source_label = if pq_fastscan_env_var(
        PQ_FASTSCAN_RERANK_SOURCE_COLUMN_ENV,
        LEGACY_ADR030_EXPERIMENTAL_RERANK_SOURCE_COLUMN_ENV,
    )
    .is_some()
    {
        PQ_FASTSCAN_RERANK_SOURCE_COLUMN_ENV
    } else if index_options.rerank_source_column.is_some() {
        "rerank_source_column"
    } else {
        "indexed column"
    };
    let (heap_relation, heap_relation_owned) = unsafe { resolve_scan_heap_relation(scan) };
    let (snapshot, snapshot_owned) = unsafe { resolve_scan_snapshot(scan) };
    let source_attribute = if let Some(source_column) = rerank.source_column {
        unsafe {
            source::resolve_source_attribute(
                heap_relation,
                &source_column,
                source_label,
                source::SourceTypePolicy::RerankSource,
            )
        }
    } else {
        let indexed_attribute = unsafe {
            source::resolve_indexed_vector_attribute(
                heap_relation,
                (*scan).indexRelation,
                source_label,
            )
        };
        match indexed_attribute.kind {
            source::IndexedVectorKind::Ecvector => source::SourceAttribute {
                attnum: indexed_attribute.attnum,
                kind: source::SourceDatumKind::Ecvector,
            },
            source::IndexedVectorKind::Tqvector => pgrx::error!(
                "ec_hnsw grouped heap-f32 rerank requires build_source_column, rerank_source_column, or {} to name a raw real[], bytea, or ecvector heap column",
                PQ_FASTSCAN_RERANK_SOURCE_COLUMN_ENV
            ),
        }
    };
    let slot = unsafe {
        source::allocate_heap_slot(
            heap_relation,
            "ec_hnsw grouped heap-f32 rerank failed to allocate a heap tuple slot",
        )
    };

    opaque.grouped_heap_rerank_relation = heap_relation;
    opaque.grouped_heap_rerank_relation_owned = heap_relation_owned;
    opaque.grouped_heap_rerank_snapshot = snapshot;
    opaque.grouped_heap_rerank_snapshot_owned = snapshot_owned;
    opaque.grouped_heap_rerank_slot = slot;
    opaque.grouped_heap_rerank_source_attnum = i16::try_from(source_attribute.attnum)
        .expect("heap rerank source attnum should fit in i16");
    opaque.grouped_heap_rerank_source_kind = source_attribute.kind;
}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_amgettuple(
    scan: pg_sys::IndexScanDesc,
    direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_hnsw amgettuple received a null scan descriptor");
            }

            let opaque_ptr = (*scan).opaque.cast::<TqScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_hnsw amgettuple missing scan opaque state");
            }

            let opaque = &*opaque_ptr;
            if !opaque.rescan_called {
                pgrx::error!("ec_hnsw amgettuple requires amrescan before scan execution");
            }
            if direction != pg_sys::ScanDirection::ForwardScanDirection {
                pgrx::error!("ec_hnsw amgettuple only supports forward scan direction");
            }

            if opaque.scan_dimensions == 0 {
                clear_scan_orderby_output(scan);
                return false;
            }

            let opaque = &mut *opaque_ptr;
            if produce_next_scan_heap_tid(scan, (*scan).indexRelation, opaque, opaque.scan_code_len)
            {
                return true;
            }

            clear_scan_orderby_output(scan);
            false
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_amendscan(scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                return;
            }

            let opaque_ptr = (*scan).opaque;
            if !opaque_ptr.is_null() {
                let opaque = &mut *opaque_ptr.cast::<TqScanOpaque>();
                finalize_scan_stats(opaque);
                flush_scan_stats(opaque);
                clear_parallel_scan_state(opaque);
                #[cfg(feature = "pg18")]
                {
                    end_read_stream(&mut opaque.graph_read_stream);
                    end_read_stream(&mut opaque.linear_read_stream);
                }
                free_graph_prefetch_state(opaque);
                free_scan_graph_cache(opaque);
                free_scan_score_cache(opaque);
                free_scan_candidate_frontier(opaque);
                free_bootstrap_expansion(opaque);
                free_scan_expanded_set(opaque);
                free_scan_visited_set(opaque);
                free_scan_emitted_set(opaque);
                free_scan_prepared_query(opaque);
                free_scan_query(opaque);
                free_grouped_heap_rerank_state(opaque);
                pg_sys::pfree(opaque_ptr);
                (*scan).opaque = ptr::null_mut();
            }
        })
    }
}

pub(crate) unsafe fn explain_counters_from_index_scan_state(
    index_state: *mut pg_sys::IndexScanState,
) -> TqExplainCounters {
    if index_state.is_null() {
        return TqExplainCounters::default();
    }

    let scan_desc = unsafe { (*index_state).iss_ScanDesc };
    if scan_desc.is_null() {
        return TqExplainCounters::default();
    }

    let opaque = unsafe { (*scan_desc).opaque };
    if opaque.is_null() {
        return TqExplainCounters::default();
    }

    unsafe { (*opaque.cast::<TqScanOpaque>()).explain_counters }
}

unsafe fn store_scan_query(opaque: &mut TqScanOpaque, query: &[f32]) {
    free_scan_query(opaque);

    let query_bytes = std::mem::size_of_val(query);
    let query_values = unsafe { pg_sys::palloc(query_bytes) }.cast::<f32>();
    if query_values.is_null() {
        pgrx::error!("ec_hnsw failed to allocate scan query state");
    }

    unsafe {
        ptr::copy_nonoverlapping(query.as_ptr(), query_values, query.len());
    }
    opaque.query_dimensions = u16::try_from(query.len()).expect("query length should fit in u16");
    opaque.query_values = query_values;
}

fn scan_query_values(opaque: &TqScanOpaque) -> &[f32] {
    if opaque.query_values.is_null() || opaque.query_dimensions == 0 {
        pgrx::error!("ec_hnsw scan state is missing raw query values");
    }

    unsafe { std::slice::from_raw_parts(opaque.query_values, opaque.query_dimensions as usize) }
}

unsafe fn free_scan_query(opaque: &mut TqScanOpaque) {
    if !opaque.query_values.is_null() {
        unsafe { pg_sys::pfree(opaque.query_values.cast()) };
        opaque.query_values = ptr::null_mut();
    }
    opaque.query_dimensions = 0;
}

fn store_scan_prepared_query(
    opaque: &mut TqScanOpaque,
    query: &[f32],
    metadata: &page::MetadataPage,
) {
    free_scan_prepared_query(opaque);
    if metadata.dimensions == 0 {
        return;
    }

    let (quantizer, cache_hit) = ProdQuantizer::cached_with_presence(
        metadata.dimensions as usize,
        metadata.bits,
        metadata.seed,
    );
    let prepared = quantizer.prepare_ip_query(query);
    let binary_query_requested = quantizer.binary_sign_no_qjl_4bit_supported()
        && (grouped_binary_traversal_score_enabled(opaque)
            || !super::options::disable_binary_prefilter());
    let binary_prepared =
        binary_query_requested.then(|| quantizer.prepare_ip_query_binary_sign_no_qjl_4bit(query));
    let turboquant_exact_score_mode = if turboquant_scan_storage(opaque.scan_graph_storage) {
        resolve_turboquant_exact_score_mode()
    } else {
        TurboQuantExactScoreMode::Exact
    };
    let (turboquant_lut_prepared, turboquant_tiled_lut_prepared, turboquant_int8_prepared) =
        match turboquant_exact_score_mode {
            TurboQuantExactScoreMode::Exact => (None, None, None),
            TurboQuantExactScoreMode::FullLut => {
                if !quantizer.int8_approx_no_qjl_4bit_supported() {
                    pgrx::error!(
                        "ec_hnsw TurboQuant exact score mode full_lut requires the no-QJL 4-bit lane"
                    );
                }
                (
                    Some(quantizer.prepare_ip_query_lut_no_qjl_4bit(query)),
                    None,
                    None,
                )
            }
            TurboQuantExactScoreMode::TiledLut => {
                if !quantizer.int8_approx_no_qjl_4bit_supported() {
                    pgrx::error!(
                        "ec_hnsw TurboQuant exact score mode tiled_lut requires the no-QJL 4-bit lane"
                    );
                }
                (
                    None,
                    Some(quantizer.prepare_ip_query_tiled_lut_no_qjl_4bit(
                        query,
                        TURBOQUANT_TILED_LUT_TILE_SIZE,
                    )),
                    None,
                )
            }
            TurboQuantExactScoreMode::Int8Approx => {
                if !quantizer.int8_approx_no_qjl_4bit_supported() {
                    pgrx::error!(
                        "ec_hnsw TurboQuant exact score mode int8_approx requires the no-QJL 4-bit lane"
                    );
                }
                (
                    None,
                    None,
                    Some(quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(query)),
                )
            }
        };
    opaque.turboquant_lut_query = turboquant_lut_prepared
        .map(|prepared| Box::into_raw(Box::new(prepared)))
        .unwrap_or(ptr::null_mut());
    opaque.turboquant_tiled_lut_query = turboquant_tiled_lut_prepared
        .map(|prepared| Box::into_raw(Box::new(prepared)))
        .unwrap_or(ptr::null_mut());
    opaque.turboquant_int8_query = turboquant_int8_prepared
        .map(|prepared| Box::into_raw(Box::new(prepared)))
        .unwrap_or(ptr::null_mut());
    opaque.prepared_query = Box::into_raw(Box::new(prepared));
    opaque.binary_sign_query = binary_prepared
        .map(|prepared| Box::into_raw(Box::new(prepared)))
        .unwrap_or(ptr::null_mut());
    if grouped_binary_traversal_score_enabled(opaque) && opaque.binary_sign_query.is_null() {
        pgrx::error!(
            "ec_hnsw PqFastScan binary traversal scoring requires the no-QJL 4-bit binary-sign lane"
        );
    }
    opaque.turboquant_exact_score_mode = turboquant_exact_score_mode;
    opaque.cached_quantizer = Arc::into_raw(quantizer);
    if cache_hit {
        opaque.explain_counters.record_quantizer_cache_hit();
        super::stats::record_quantizer_cache_hit();
        opaque.stats_delta.record_quantizer_cache_hit();
    } else {
        super::stats::record_quantizer_cache_miss();
        opaque.stats_delta.record_quantizer_cache_miss();
    }
}

fn free_scan_prepared_query(opaque: &mut TqScanOpaque) {
    if !opaque.grouped_query.is_null() {
        drop(unsafe { Box::from_raw(opaque.grouped_query) });
        opaque.grouped_query = ptr::null_mut();
    }
    if !opaque.prepared_query.is_null() {
        drop(unsafe { Box::from_raw(opaque.prepared_query) });
        opaque.prepared_query = ptr::null_mut();
    }
    if !opaque.binary_sign_query.is_null() {
        drop(unsafe { Box::from_raw(opaque.binary_sign_query) });
        opaque.binary_sign_query = ptr::null_mut();
    }
    if !opaque.turboquant_lut_query.is_null() {
        drop(unsafe { Box::from_raw(opaque.turboquant_lut_query) });
        opaque.turboquant_lut_query = ptr::null_mut();
    }
    if !opaque.turboquant_tiled_lut_query.is_null() {
        drop(unsafe { Box::from_raw(opaque.turboquant_tiled_lut_query) });
        opaque.turboquant_tiled_lut_query = ptr::null_mut();
    }
    if !opaque.turboquant_int8_query.is_null() {
        drop(unsafe { Box::from_raw(opaque.turboquant_int8_query) });
        opaque.turboquant_int8_query = ptr::null_mut();
    }
    opaque.turboquant_exact_score_mode = TurboQuantExactScoreMode::Exact;
    if !opaque.cached_quantizer.is_null() {
        drop(unsafe { Arc::from_raw(opaque.cached_quantizer) });
        opaque.cached_quantizer = ptr::null();
    }
}

fn build_prepared_grouped_scan_query(
    prepared_query: &PreparedQuery,
    model: &graph::GroupedCodebookModel,
) -> PreparedGroupedScanQuery {
    let expected_rotated_len = model.group_count * model.group_size;
    assert_eq!(
        prepared_query.rotated.len(),
        expected_rotated_len,
        "grouped scan prepared query length mismatch: got {}, expected {}",
        prepared_query.rotated.len(),
        expected_rotated_len
    );

    PreparedGroupedScanQuery {
        group_count: model.group_count,
        search_code_len: model.group_count.div_ceil(2),
        lut_f32: build_grouped_pq_lut_f32(
            &prepared_query.rotated,
            &model.flat_codebooks,
            model.group_size,
        ),
    }
}

unsafe fn load_grouped_scan_query(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    prepared_query: &PreparedQuery,
) -> PreparedGroupedScanQuery {
    let model = unsafe { graph::load_grouped_codebook_model(index_relation, metadata) };
    build_prepared_grouped_scan_query(prepared_query, &model)
}

unsafe fn store_grouped_scan_query(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    metadata: &page::MetadataPage,
) {
    if !matches!(
        opaque.scan_graph_storage,
        graph::GraphStorageDescriptor::PqFastScan(_)
    ) {
        return;
    }
    if opaque.prepared_query.is_null() {
        pgrx::error!(
            "ec_hnsw PqFastScan scan cannot prepare PqFastScan query state without a query"
        );
    }
    let prepared_query = unsafe { &*opaque.prepared_query };
    let grouped_prepared =
        unsafe { load_grouped_scan_query(index_relation, metadata, prepared_query) };
    opaque.grouped_query = Box::into_raw(Box::new(grouped_prepared));
}

fn grouped_scan_query(opaque: &TqScanOpaque) -> Option<&PreparedGroupedScanQuery> {
    if opaque.grouped_query.is_null() {
        None
    } else {
        Some(unsafe { &*opaque.grouped_query })
    }
}

fn reset_scan_position(opaque: &mut TqScanOpaque) {
    opaque.next_block_number = page::FIRST_DATA_BLOCK_NUMBER;
    opaque.next_offset_number = 1;
    opaque.execution_phase = ScanExecutionPhase::GraphTraversal;
    opaque.stats_used_linear_fallback = false;
    opaque.stats_scan_finalized = false;
    clear_last_emitted_scan_scores(opaque);
    clear_grouped_live_rerank_buffer(opaque);
    clear_scan_candidate_state(opaque);
    reset_scan_graph_cache(opaque);
    reset_scan_score_cache(opaque);
    reset_scan_debug_profile(opaque);
    opaque.result_state.clear();
    opaque.fallback_result_state.clear();
    clear_deferred_parallel_blocked_outputs(opaque);
    opaque.parallel_owned_output_blocker = None;
    opaque.retained_parallel_owned_output_blocker = None;
    opaque.parallel_local_only_output_active = false;
    reset_bootstrap_expansion_state(opaque, bootstrap_frontier_limit(opaque));
    reset_scan_expanded_state(opaque);
    reset_scan_visited_state(opaque);
    reset_scan_emitted_state(opaque);
    sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
}

fn reset_scan_graph_cache(opaque: &mut TqScanOpaque) {
    if opaque.graph_element_cache.is_null() {
        opaque.graph_element_cache = Box::into_raw(Box::new(HashMap::new()));
    } else {
        unsafe { &mut *opaque.graph_element_cache }.clear();
    }

    if opaque.graph_neighbor_cache.is_null() {
        opaque.graph_neighbor_cache = Box::into_raw(Box::new(HashMap::new()));
    } else {
        unsafe { &mut *opaque.graph_neighbor_cache }.clear();
    }
}

fn reset_scan_score_cache(opaque: &mut TqScanOpaque) {
    if opaque.score_cache.is_null() {
        opaque.score_cache = Box::into_raw(Box::new(HashMap::new()));
    } else {
        unsafe { &mut *opaque.score_cache }.clear();
    }
}

fn free_scan_graph_cache(opaque: &mut TqScanOpaque) {
    if !opaque.graph_element_cache.is_null() {
        drop(unsafe { Box::from_raw(opaque.graph_element_cache) });
        opaque.graph_element_cache = ptr::null_mut();
    }

    if !opaque.graph_neighbor_cache.is_null() {
        drop(unsafe { Box::from_raw(opaque.graph_neighbor_cache) });
        opaque.graph_neighbor_cache = ptr::null_mut();
    }
}

fn free_scan_score_cache(opaque: &mut TqScanOpaque) {
    if !opaque.score_cache.is_null() {
        drop(unsafe { Box::from_raw(opaque.score_cache) });
        opaque.score_cache = ptr::null_mut();
    }
}

fn graph_element_cache_mut(
    opaque: &mut TqScanOpaque,
) -> &mut HashMap<page::ItemPointer, Arc<CachedGraphElement>> {
    if opaque.graph_element_cache.is_null() {
        opaque.graph_element_cache = Box::into_raw(Box::new(HashMap::new()));
    }

    unsafe { &mut *opaque.graph_element_cache }
}

fn graph_neighbor_cache_mut(
    opaque: &mut TqScanOpaque,
) -> &mut HashMap<page::ItemPointer, Arc<graph::GraphNeighbors>> {
    if opaque.graph_neighbor_cache.is_null() {
        opaque.graph_neighbor_cache = Box::into_raw(Box::new(HashMap::new()));
    }

    unsafe { &mut *opaque.graph_neighbor_cache }
}

fn score_cache_mut(opaque: &mut TqScanOpaque) -> &mut HashMap<page::ItemPointer, f32> {
    if opaque.score_cache.is_null() {
        opaque.score_cache = Box::into_raw(Box::new(HashMap::new()));
    }

    unsafe { &mut *opaque.score_cache }
}

fn cached_scan_element_score(opaque: &TqScanOpaque, element_tid: page::ItemPointer) -> Option<f32> {
    if opaque.score_cache.is_null() {
        return None;
    }

    unsafe { &*opaque.score_cache }.get(&element_tid).copied()
}

unsafe fn live_loaded_state_from_exact_payload(
    opaque: &mut TqScanOpaque,
    element_tid: page::ItemPointer,
    binary_query_active: bool,
    exact_payload: Option<(f32, &[u8])>,
) -> LoadedElementState {
    match exact_payload {
        Some((gamma, code_bytes)) if binary_query_active => {
            LoadedElementState::ExactPayload(LoadedElementScoreInput {
                gamma,
                code_bytes: code_bytes.to_vec(),
            })
        }
        Some((gamma, code_bytes)) => LoadedElementState::ExactScore(score_and_cache_scan_element(
            opaque,
            element_tid,
            gamma,
            code_bytes,
        )),
        None => LoadedElementState::ExactUnavailable,
    }
}

fn binary_sign_query(opaque: &TqScanOpaque) -> Option<&BinarySignNoQjl4BitQuery> {
    if opaque.binary_sign_query.is_null() {
        None
    } else {
        Some(unsafe { &*opaque.binary_sign_query })
    }
}

fn binary_prefilter_survivor_budget(candidate_count: usize) -> usize {
    if candidate_count < ADR031_BINARY_PREFILTER_MIN_CANDIDATES {
        return candidate_count;
    }

    candidate_count.saturating_sub(ADR031_BINARY_PREFILTER_REJECTIONS)
}

unsafe fn score_and_cache_scan_element(
    opaque: &mut TqScanOpaque,
    element_tid: page::ItemPointer,
    gamma: f32,
    code_bytes: &[u8],
) -> f32 {
    record_score_cache_miss(opaque);
    #[cfg(any(test, feature = "pg_test"))]
    let started = Instant::now();
    let score = unsafe { score_scan_element_result(opaque, gamma, code_bytes) };
    #[cfg(any(test, feature = "pg_test"))]
    let elapsed_us =
        u64::try_from(started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let elapsed_us = 0;
    record_candidate_score_elapsed(opaque, elapsed_us);
    score_cache_mut(opaque).insert(element_tid, score);
    score
}

fn build_cached_graph_element(
    opaque_ref: &mut TqScanOpaque,
    element_tid: page::ItemPointer,
    element: graph::GraphTupleRef<'_>,
) -> (CachedGraphElement, LoadedElementState) {
    let binary_query_active = binary_sign_query(opaque_ref).is_some();
    let live_element = !element.deleted() && element.heaptid_count() > 0;
    let binary_words = if binary_query_active {
        if !super::options::force_binary_derivation() && element.binary_word_count() > 0 {
            CachedBinaryWords::from_vec(element.collect_binary_words())
        } else {
            match element.exact_payload() {
                Some((_gamma, code_bytes)) => {
                    let quantizer = unsafe { &*opaque_ref.cached_quantizer };
                    CachedBinaryWords::from_vec(
                        quantizer.binary_sign_words_from_packed_no_qjl_4bit(code_bytes),
                    )
                }
                None => CachedBinaryWords::empty(),
            }
        }
    } else {
        CachedBinaryWords::empty()
    };

    let mut loaded_state = LoadedElementState::None;
    if live_element {
        loaded_state = match (opaque_ref.scan_graph_storage, element.exact_payload()) {
            (graph::GraphStorageDescriptor::TurboQuantHotCold(_), None) => LoadedElementState::None,
            (_, exact_payload) => unsafe {
                live_loaded_state_from_exact_payload(
                    opaque_ref,
                    element_tid,
                    binary_query_active,
                    exact_payload,
                )
            },
        };
    }

    (
        CachedGraphElement::from_graph_tuple_ref(element_tid, element, binary_words),
        loaded_state,
    )
}

unsafe fn cached_graph_element(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    element_tid: page::ItemPointer,
) -> (Arc<CachedGraphElement>, LoadedElementState) {
    let opaque_ref = unsafe { &mut *opaque };
    if !opaque_ref.graph_element_cache.is_null() {
        if let Some(element) = unsafe { &*opaque_ref.graph_element_cache }.get(&element_tid) {
            record_graph_element_cache_hit(opaque_ref);
            return (Arc::clone(element), LoadedElementState::None);
        }
    }

    #[cfg(any(test, feature = "pg_test"))]
    let started = Instant::now();
    let (element, loaded_state) = unsafe {
        graph::with_graph_storage_tuple(
            index_relation,
            element_tid,
            opaque_ref.scan_graph_storage,
            |element| build_cached_graph_element(opaque_ref, element_tid, element),
        )
    };
    let element = Arc::new(element);
    #[cfg(any(test, feature = "pg_test"))]
    let elapsed_us =
        u64::try_from(started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let elapsed_us = 0;
    record_graph_element_cache_miss_load(opaque_ref, elapsed_us);
    graph_element_cache_mut(opaque_ref).insert(element_tid, Arc::clone(&element));
    debug_assert!(
        element.deleted
            || element.heaptids.is_empty()
            || matches!(
                opaque_ref.scan_graph_storage,
                graph::GraphStorageDescriptor::TurboQuantHotCold(_)
            )
            || !matches!(loaded_state, LoadedElementState::None),
        "live graph elements should populate exact-score or binary-prefilter state on load"
    );
    (element, loaded_state)
}

#[cfg(feature = "pg18")]
unsafe fn cached_graph_element_from_buffer(
    opaque: *mut TqScanOpaque,
    buffer: pg_sys::Buffer,
    element_tid: page::ItemPointer,
) -> (Arc<CachedGraphElement>, LoadedElementState) {
    let opaque_ref = unsafe { &mut *opaque };
    if !opaque_ref.graph_element_cache.is_null() {
        if let Some(element) = unsafe { &*opaque_ref.graph_element_cache }.get(&element_tid) {
            record_graph_element_cache_hit(opaque_ref);
            return (Arc::clone(element), LoadedElementState::None);
        }
    }

    #[cfg(any(test, feature = "pg_test"))]
    let started = Instant::now();
    let (element, loaded_state) = unsafe {
        graph::with_graph_storage_tuple_from_buffer(
            buffer,
            element_tid,
            opaque_ref.scan_graph_storage,
            |element| build_cached_graph_element(opaque_ref, element_tid, element),
        )
    };
    let element = Arc::new(element);
    #[cfg(any(test, feature = "pg_test"))]
    let elapsed_us =
        u64::try_from(started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let elapsed_us = 0;
    record_graph_element_cache_miss_load(opaque_ref, elapsed_us);
    graph_element_cache_mut(opaque_ref).insert(element_tid, Arc::clone(&element));
    (element, loaded_state)
}

unsafe fn score_cached_graph_element_from_storage(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    element_tid: page::ItemPointer,
) -> f32 {
    let opaque_ref = unsafe { &mut *opaque };
    let element = unsafe {
        graph::load_exact_graph_element(index_relation, element_tid, opaque_ref.scan_graph_storage)
    };
    if element.deleted || element.heaptids.is_empty() {
        pgrx::error!(
            "ec_hnsw cannot exact-score dead or heapless graph element {}:{}",
            element_tid.block_number,
            element_tid.offset_number
        );
    }
    score_and_cache_scan_element(opaque_ref, element_tid, element.gamma, &element.code)
}

unsafe fn exact_score_cached_graph_element(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    element_tid: page::ItemPointer,
    loaded_state: LoadedElementState,
) -> f32 {
    match loaded_state {
        LoadedElementState::ExactScore(score) => score,
        LoadedElementState::ExactPayload(loaded) => {
            let opaque_ref = unsafe { &mut *opaque };
            if let Some(score) = cached_scan_element_score(opaque_ref, element_tid) {
                record_score_cache_hit(opaque_ref);
                score
            } else {
                score_and_cache_scan_element(
                    opaque_ref,
                    element_tid,
                    loaded.gamma,
                    &loaded.code_bytes,
                )
            }
        }
        LoadedElementState::ExactUnavailable => {
            pgrx::error!("{PQ_FASTSCAN_EXACT_SCORE_UNAVAILABLE}")
        }
        LoadedElementState::None => {
            let opaque_ref = unsafe { &mut *opaque };
            if let Some(score) = cached_scan_element_score(opaque_ref, element_tid) {
                record_score_cache_hit(opaque_ref);
                score
            } else {
                unsafe {
                    score_cached_graph_element_from_storage(index_relation, opaque, element_tid)
                }
            }
        }
    }
}

fn grouped_score_context_from_scan_state<'a>(
    scan_graph_storage: graph::GraphStorageDescriptor,
    element: &'a CachedGraphElement,
) -> Option<GroupedScoreContext<'a>> {
    Some(GroupedScoreContext {
        element_tid: element.tid,
        call: GroupedScoreCall {
            shape: GroupedScoreShape::from_scan_graph_storage(scan_graph_storage)?,
            input: element.grouped_score_input()?,
        },
    })
}

fn grouped_score_search_code(grouped: GroupedScoreContext<'_>) -> Option<&[u8]> {
    if grouped.call.input.search_code.len() != grouped.call.shape.search_code_len {
        return None;
    }
    Some(grouped.call.input.search_code)
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
fn grouped_score_payload_view<'a>(
    grouped: GroupedScoreContext<'a>,
) -> Option<GroupedScorePayloadView<'a>> {
    if grouped.call.input.binary_words.len() != grouped.call.shape.binary_word_count {
        return None;
    }
    if grouped.call.input.search_code.len() != grouped.call.shape.search_code_len {
        return None;
    }
    Some(GroupedScorePayloadView {
        element_tid: grouped.element_tid,
        reranktid: grouped.call.input.reranktid,
        binary_words: grouped.call.input.binary_words,
        search_code: grouped.call.input.search_code,
        rerank_code_len: grouped.call.shape.rerank_code_len,
    })
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
fn grouped_score_rerank_payload<'a>(
    payload: GroupedScorePayloadView<'a>,
    rerank: graph::GroupedRerankPayload,
) -> Option<GroupedScoreRerankPayload<'a>> {
    if rerank.tid != payload.reranktid {
        return None;
    }
    if rerank.code.len() != payload.rerank_code_len {
        return None;
    }
    Some(GroupedScoreRerankPayload {
        element_tid: payload.element_tid,
        reranktid: payload.reranktid,
        binary_words: payload.binary_words,
        search_code: payload.search_code,
        rerank_gamma: rerank.gamma,
        rerank_code: rerank.code,
    })
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
unsafe fn load_grouped_score_rerank_payload<'a>(
    index_relation: pg_sys::Relation,
    grouped: GroupedScoreContext<'a>,
) -> Option<GroupedScoreRerankPayload<'a>> {
    let payload = grouped_score_payload_view(grouped)?;
    let rerank = unsafe {
        graph::load_grouped_rerank_payload(
            index_relation,
            payload.reranktid,
            graph::PqFastScanLayout {
                binary_word_count: payload.binary_words.len(),
                search_code_len: payload.search_code.len(),
                rerank_code_len: payload.rerank_code_len,
            },
        )
    };
    grouped_score_rerank_payload(payload, rerank)
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
fn score_grouped_rerank_payload_result(
    quantizer: &ProdQuantizer,
    prepared_query: &PreparedQuery,
    payload: &GroupedScoreRerankPayload<'_>,
) -> f32 {
    // Negate inner product to produce distance, matching the scalar exact-score path.
    -quantizer.score_ip_from_parts(prepared_query, payload.rerank_gamma, &payload.rerank_code)
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
unsafe fn score_grouped_rerank_payload_from_scan_state(
    opaque: *mut TqScanOpaque,
    payload: &GroupedScoreRerankPayload<'_>,
) -> f32 {
    let opaque = unsafe { &*opaque };
    if opaque.prepared_query.is_null() {
        pgrx::error!("ec_hnsw scan state is missing prepared query");
    }
    if opaque.cached_quantizer.is_null() {
        pgrx::error!("ec_hnsw scan state is missing cached quantizer");
    }
    let prepared_query = unsafe { &*opaque.prepared_query };
    let quantizer = unsafe { &*opaque.cached_quantizer };
    score_grouped_rerank_payload_result(quantizer, prepared_query, payload)
}

unsafe fn score_grouped_heap_source_from_scan_state(
    opaque: &mut TqScanOpaque,
    heap_tid: page::ItemPointer,
) -> f32 {
    if opaque.grouped_heap_rerank_relation.is_null()
        || opaque.grouped_heap_rerank_snapshot.is_null()
        || opaque.grouped_heap_rerank_slot.is_null()
        || opaque.grouped_heap_rerank_source_attnum <= 0
    {
        pgrx::error!("ec_hnsw grouped heap-f32 rerank is missing heap fetch state");
    }

    let source_attribute = source::SourceAttribute {
        attnum: i32::from(opaque.grouped_heap_rerank_source_attnum),
        kind: opaque.grouped_heap_rerank_source_kind,
    };
    #[cfg(any(test, feature = "pg_test"))]
    let fetch_started = Instant::now();
    unsafe {
        source::fetch_heap_row_version(
            opaque.grouped_heap_rerank_relation,
            heap_tid,
            opaque.grouped_heap_rerank_snapshot,
            opaque.grouped_heap_rerank_slot,
            "PqFastScan heap rerank source vector",
        )
    };
    #[cfg(any(test, feature = "pg_test"))]
    let fetch_elapsed_us =
        u64::try_from(fetch_started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let fetch_elapsed_us = 0;
    record_grouped_rerank_heap_fetch(opaque, fetch_elapsed_us);
    #[cfg(any(test, feature = "pg_test"))]
    let decode_started = Instant::now();
    let source = unsafe {
        source::FlatFloat4SourceRef::from_datum(
            source::required_slot_datum(
                opaque.grouped_heap_rerank_slot,
                source_attribute.attnum,
                "PqFastScan heap rerank source vector",
            ),
            source_attribute.kind,
            "PqFastScan heap rerank source vector",
        )
    };
    #[cfg(any(test, feature = "pg_test"))]
    let decode_elapsed_us =
        u64::try_from(decode_started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let decode_elapsed_us = 0;
    record_grouped_rerank_heap_decode_elapsed(opaque, decode_elapsed_us);
    #[cfg(any(test, feature = "pg_test"))]
    let dot_started = Instant::now();
    let score = source::negative_inner_product(scan_query_values(opaque), source.as_slice());
    #[cfg(any(test, feature = "pg_test"))]
    let dot_elapsed_us =
        u64::try_from(dot_started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let dot_elapsed_us = 0;
    record_grouped_rerank_heap_dot_elapsed(opaque, dot_elapsed_us);
    drop(source);
    unsafe { pg_sys::ExecClearTuple(opaque.grouped_heap_rerank_slot) };
    score
}

unsafe fn score_grouped_candidate_heap_rerank(
    opaque: *mut TqScanOpaque,
    element: &CachedGraphElement,
) -> Option<f32> {
    let opaque = unsafe { &mut *opaque };
    #[cfg(any(test, feature = "pg_test"))]
    let started = Instant::now();
    let mut best_score: Option<f32> = None;
    for heap_tid in element.heaptids.as_slice().iter().copied() {
        let score = unsafe { score_grouped_heap_source_from_scan_state(opaque, heap_tid) };
        best_score = Some(match best_score {
            Some(current) => current.min(score),
            None => score,
        });
    }
    #[cfg(any(test, feature = "pg_test"))]
    let elapsed_us =
        u64::try_from(started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let elapsed_us = 0;
    if best_score.is_some() {
        record_grouped_rerank_heap_score_elapsed(opaque, elapsed_us);
    }
    best_score
}

unsafe fn exact_score_grouped_candidate_context(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    grouped: GroupedScoreContext<'_>,
) -> f32 {
    let opaque_ref = unsafe { &mut *opaque };
    if let Some(score) = cached_scan_element_score(opaque_ref, grouped.element_tid) {
        record_score_cache_hit(opaque_ref);
        return score;
    }

    let payload = unsafe { load_grouped_score_rerank_payload(index_relation, grouped) }
        .unwrap_or_else(|| {
            pgrx::error!("ec_hnsw PqFastScan exact scoring requires metadata-aligned cold payload")
        });
    unsafe {
        score_and_cache_scan_element(
            opaque_ref,
            grouped.element_tid,
            payload.rerank_gamma,
            &payload.rerank_code,
        )
    }
}

unsafe fn score_grouped_candidate_context_exact(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    grouped: GroupedScoreContext<'_>,
) -> f32 {
    #[cfg(any(test, feature = "pg_test"))]
    let started = Instant::now();
    let score = unsafe { exact_score_grouped_candidate_context(index_relation, opaque, grouped) };
    #[cfg(any(test, feature = "pg_test"))]
    let elapsed_us =
        u64::try_from(started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let elapsed_us = 0;
    record_grouped_traversal_exact_score_elapsed(unsafe { &mut *opaque }, elapsed_us);
    score
}

fn score_grouped_search_code_result(
    prepared_query: &PreparedGroupedScanQuery,
    search_code: &[u8],
) -> f32 {
    // ADR-041 stage 0: grouped-PQ LUT scoring routes through the
    // `QueryScorer` trait. The inherent debug_assert on search-code
    // length lives in the trait impl on `PreparedGroupedScanQuery`.
    use crate::quant::QueryScorer;
    -prepared_query.score(search_code)
}

unsafe fn score_grouped_search_code_from_scan_state(
    opaque: *mut TqScanOpaque,
    search_code: &[u8],
) -> f32 {
    let opaque = unsafe { &*opaque };
    let prepared_query = grouped_scan_query(opaque).unwrap_or_else(|| {
        pgrx::error!("ec_hnsw PqFastScan scan is missing PqFastScan query state")
    });
    score_grouped_search_code_result(prepared_query, search_code)
}

unsafe fn grouped_candidate_rerank_comparison_score(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    element: &CachedGraphElement,
) -> Option<f32> {
    if grouped_heap_rerank_enabled(unsafe { &*opaque }) {
        return unsafe { score_grouped_candidate_heap_rerank(opaque, element) };
    }

    let scan_graph_storage = unsafe { (&*opaque).scan_graph_storage };
    if matches!(
        scan_graph_storage,
        graph::GraphStorageDescriptor::TurboQuant { .. }
            | graph::GraphStorageDescriptor::TurboQuantHotCold(_)
    ) {
        #[cfg(any(test, feature = "pg_test"))]
        let started = Instant::now();
        let score = unsafe {
            exact_score_cached_graph_element(
                index_relation,
                opaque,
                element.tid,
                LoadedElementState::None,
            )
        };
        #[cfg(any(test, feature = "pg_test"))]
        let elapsed_us =
            u64::try_from(started.elapsed().as_micros()).expect("timing should fit in u64");
        #[cfg(not(any(test, feature = "pg_test")))]
        let elapsed_us = 0;
        record_grouped_rerank_quantized_score_elapsed(unsafe { &mut *opaque }, elapsed_us);
        return Some(score);
    }

    let grouped = grouped_score_context_from_scan_state(scan_graph_storage, element)?;
    #[cfg(any(test, feature = "pg_test"))]
    let started = Instant::now();
    let score = unsafe { exact_score_grouped_candidate_context(index_relation, opaque, grouped) };
    #[cfg(any(test, feature = "pg_test"))]
    let elapsed_us =
        u64::try_from(started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let elapsed_us = 0;
    record_grouped_rerank_quantized_score_elapsed(unsafe { &mut *opaque }, elapsed_us);
    Some(score)
}

fn candidate_score_dispatch<'a>(
    scan_graph_storage: graph::GraphStorageDescriptor,
    element: &'a CachedGraphElement,
    loaded_state: LoadedElementState,
) -> CandidateScoreDispatch<'a> {
    let grouped = grouped_score_context_from_scan_state(scan_graph_storage, element);
    match loaded_state {
        LoadedElementState::ExactUnavailable | LoadedElementState::None if grouped.is_some() => {
            CandidateScoreDispatch::Grouped(grouped.unwrap_or_else(|| {
                panic!("grouped score dispatch requires grouped score context for grouped payloads")
            }))
        }
        other => CandidateScoreDispatch::Exact(other),
    }
}

fn grouped_exact_traversal_candidate_indices(
    candidates: &[GroupedTraversalCandidate],
    budget: usize,
) -> Vec<usize> {
    let mut indices = (0..candidates.len()).collect::<Vec<_>>();
    indices.sort_by(|&left, &right| {
        candidates[left]
            .approx_score
            .total_cmp(&candidates[right].approx_score)
            .then_with(|| candidates[left].ordinal.cmp(&candidates[right].ordinal))
    });
    indices.truncate(budget.min(indices.len()));
    indices
}

unsafe fn score_grouped_candidate_context_approx(
    opaque: *mut TqScanOpaque,
    grouped: GroupedScoreContext<'_>,
) -> f32 {
    let search_code = grouped_score_search_code(grouped).unwrap_or_else(|| {
        panic!("grouped approximate scoring requires metadata-aligned grouped search codes")
    });
    #[cfg(any(test, feature = "pg_test"))]
    let started = Instant::now();
    let score = unsafe { score_grouped_search_code_from_scan_state(opaque, search_code) };
    #[cfg(any(test, feature = "pg_test"))]
    let elapsed_us =
        u64::try_from(started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let elapsed_us = 0;
    record_grouped_traversal_approx_score_elapsed(unsafe { &mut *opaque }, elapsed_us);
    score
}

unsafe fn score_grouped_candidate_context_binary(
    opaque: *mut TqScanOpaque,
    grouped: GroupedScoreContext<'_>,
) -> f32 {
    assert_eq!(
        grouped.call.input.binary_words.len(),
        grouped.call.shape.binary_word_count,
        "grouped binary traversal scoring requires metadata-aligned binary sidecars",
    );
    let opaque = unsafe { &*opaque };
    let binary_query = binary_sign_query(opaque).unwrap_or_else(|| {
        pgrx::error!("ec_hnsw PqFastScan binary traversal scoring requires a prepared binary query")
    });
    let quantizer = unsafe { &*opaque.cached_quantizer };
    -quantizer.score_binary_sign_words_no_qjl_4bit(binary_query, grouped.call.input.binary_words)
}

unsafe fn score_budgeted_grouped_traversal_candidates(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    source_tid: page::ItemPointer,
    budget: usize,
    candidates: Vec<GroupedTraversalCandidate>,
) -> Vec<search::BeamCandidate<page::ItemPointer>> {
    let scan_graph_storage = unsafe { (&*opaque).scan_graph_storage };
    let mut final_scores = candidates
        .iter()
        .map(|candidate| candidate.approx_score)
        .collect::<Vec<_>>();

    let exact_indices = grouped_exact_traversal_candidate_indices(&candidates, budget);
    record_grouped_traversal_budget(
        unsafe { &mut *opaque },
        candidates.len(),
        exact_indices.len(),
    );

    for exact_idx in exact_indices {
        let grouped = grouped_score_context_from_scan_state(
            scan_graph_storage,
            &candidates[exact_idx].element,
        )
        .unwrap_or_else(|| {
            panic!("budgeted grouped exact traversal requires metadata-aligned grouped payloads")
        });
        final_scores[exact_idx] =
            unsafe { score_grouped_candidate_context_exact(index_relation, opaque, grouped) };
    }

    candidates
        .into_iter()
        .enumerate()
        .map(|(idx, candidate)| {
            search::BeamCandidate::with_source(candidate.element.tid, final_scores[idx], source_tid)
        })
        .collect()
}

unsafe fn score_grouped_candidate_context(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    grouped: GroupedScoreContext<'_>,
    traversal_layer: u8,
) -> f32 {
    let opaque_ref = unsafe { &*opaque };
    if grouped_exact_traversal_full_candidate_scoring_for_layer(opaque_ref, traversal_layer) {
        return unsafe { score_grouped_candidate_context_exact(index_relation, opaque, grouped) };
    }

    let _ = index_relation;
    if grouped_binary_traversal_score_enabled(opaque_ref) {
        return unsafe { score_grouped_candidate_context_binary(opaque, grouped) };
    }

    unsafe { score_grouped_candidate_context_approx(opaque, grouped) }
}

unsafe fn score_cached_graph_element_dispatch(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    element: &CachedGraphElement,
    loaded_state: LoadedElementState,
    traversal_layer: u8,
) -> f32 {
    let scan_graph_storage = unsafe { (&*opaque).scan_graph_storage };
    match candidate_score_dispatch(scan_graph_storage, element, loaded_state) {
        CandidateScoreDispatch::Exact(loaded_state) => unsafe {
            exact_score_cached_graph_element(index_relation, opaque, element.tid, loaded_state)
        },
        CandidateScoreDispatch::Grouped(grouped) => unsafe {
            score_grouped_candidate_context(index_relation, opaque, grouped, traversal_layer)
        },
    }
}

unsafe fn cached_graph_element_and_score(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    element_tid: page::ItemPointer,
    traversal_layer: u8,
) -> (Arc<CachedGraphElement>, Option<f32>) {
    let (element, loaded_state) =
        unsafe { cached_graph_element(index_relation, opaque, element_tid) };
    if element.deleted || element.heaptids.is_empty() {
        return (element, None);
    }
    let score = unsafe {
        score_cached_graph_element_dispatch(
            index_relation,
            opaque,
            &element,
            loaded_state,
            traversal_layer,
        )
    };
    (element, Some(score))
}

unsafe fn cached_graph_neighbors(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    neighbor_tid: page::ItemPointer,
) -> Arc<graph::GraphNeighbors> {
    let opaque_ref = unsafe { &mut *opaque };
    if !opaque_ref.graph_neighbor_cache.is_null() {
        if let Some(neighbors) = unsafe { &*opaque_ref.graph_neighbor_cache }.get(&neighbor_tid) {
            record_graph_neighbor_cache_hit(opaque_ref);
            return Arc::clone(neighbors);
        }
    }

    #[cfg(any(test, feature = "pg_test"))]
    let started = Instant::now();
    let neighbors = Arc::new(unsafe { graph::load_graph_neighbors(index_relation, neighbor_tid) });
    #[cfg(any(test, feature = "pg_test"))]
    let elapsed_us =
        u64::try_from(started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let elapsed_us = 0;
    record_graph_neighbor_cache_miss_load(opaque_ref, elapsed_us);
    graph_neighbor_cache_mut(opaque_ref).insert(neighbor_tid, Arc::clone(&neighbors));
    neighbors
}

unsafe fn cached_graph_adjacency(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    element_tid: page::ItemPointer,
) -> (Arc<CachedGraphElement>, Arc<graph::GraphNeighbors>) {
    let (element, _) = unsafe { cached_graph_element(index_relation, opaque, element_tid) };
    let neighbors = unsafe { cached_graph_neighbors(index_relation, opaque, element.neighbortid) };
    (element, neighbors)
}

#[cfg(feature = "pg18")]
unsafe fn prefetch_graph_buffers(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    neighbor_tids: &[page::ItemPointer],
) -> HashMap<u32, pg_sys::Buffer> {
    let mut blocks = Vec::new();
    let mut seen_blocks = HashSet::new();
    for neighbor_tid in neighbor_tids.iter().copied() {
        if neighbor_tid == page::ItemPointer::INVALID {
            continue;
        }
        if seen_blocks.insert(neighbor_tid.block_number) {
            blocks.push(neighbor_tid.block_number);
        }
    }

    if blocks.is_empty() {
        return HashMap::new();
    }

    reset_graph_prefetch_blocks(opaque, blocks);
    let stream = ensure_graph_read_stream(index_relation, opaque);
    unsafe { pg_sys::read_stream_reset(stream) };

    let mut prefetched_buffers = HashMap::new();
    loop {
        let mut per_buffer_data = ptr::null_mut();
        let buffer = unsafe { pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data) };
        if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
            break;
        }
        let block_number = if per_buffer_data.is_null() {
            unsafe { pg_sys::ReleaseBuffer(buffer) };
            continue;
        } else {
            unsafe { *per_buffer_data.cast::<pg_sys::BlockNumber>() }
        };
        prefetched_buffers.insert(block_number, buffer);
    }

    prefetched_buffers
}

#[cfg(feature = "pg18")]
fn release_prefetched_graph_buffers(prefetched_buffers: HashMap<u32, pg_sys::Buffer>) {
    for buffer in prefetched_buffers.into_values() {
        unsafe { pg_sys::ReleaseBuffer(buffer) };
    }
}

fn release_prefetched_graph_buffers_if_any(
    prefetched_buffers: Option<HashMap<u32, pg_sys::Buffer>>,
) {
    #[cfg(feature = "pg18")]
    if let Some(prefetched_buffers) = prefetched_buffers {
        release_prefetched_graph_buffers(prefetched_buffers);
    }

    #[cfg(not(feature = "pg18"))]
    let _ = prefetched_buffers;
}

unsafe fn cached_graph_element_with_prefetch(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    #[cfg_attr(not(feature = "pg18"), allow(unused_variables))] prefetched_buffers: Option<
        &HashMap<u32, pg_sys::Buffer>,
    >,
    element_tid: page::ItemPointer,
) -> (Arc<CachedGraphElement>, LoadedElementState) {
    #[cfg(feature = "pg18")]
    if let Some(prefetched_buffers) = prefetched_buffers {
        if let Some(buffer) = prefetched_buffers.get(&element_tid.block_number).copied() {
            unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
            let loaded = unsafe { cached_graph_element_from_buffer(opaque, buffer, element_tid) };
            unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_UNLOCK as i32) };
            return loaded;
        }
    }

    unsafe { cached_graph_element(index_relation, opaque, element_tid) }
}

unsafe fn cached_scan_successor_candidates_for_layer<KeepFn>(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    source_tid: page::ItemPointer,
    layer: u8,
    mut keep_neighbor_tid: KeepFn,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    KeepFn: FnMut(page::ItemPointer) -> bool,
{
    let (element, neighbors) =
        unsafe { cached_graph_adjacency(index_relation, opaque, source_tid) };
    let scan_graph_storage = unsafe { (&*opaque).scan_graph_storage };
    let exact_budget =
        grouped_exact_traversal_candidate_budget_for_layer(unsafe { &*opaque }, layer);
    let scan_m = usize::from(unsafe { &*opaque }.scan_m);
    let capacity = graph::layer_slot_bounds(element.level, scan_m, layer)
        .map(|(start, end)| {
            end.min(neighbors.tids.len())
                .saturating_sub(start.min(neighbors.tids.len()))
        })
        .unwrap_or(0);
    let mut candidates = Vec::with_capacity(capacity);
    let neighbor_tids =
        graph::valid_neighbor_tids_for_layer(&neighbors.tids, element.level, scan_m, layer);
    #[cfg(feature = "pg18")]
    let prefetched_buffers =
        Some(unsafe { prefetch_graph_buffers(index_relation, &mut *opaque, &neighbor_tids) });
    #[cfg(not(feature = "pg18"))]
    let prefetched_buffers: Option<HashMap<u32, pg_sys::Buffer>> = None;

    let binary_query = unsafe { (*opaque).binary_sign_query.as_ref() };
    if binary_query.is_none() {
        let mut grouped_candidates = exact_budget.map(|_| Vec::with_capacity(capacity));
        for neighbor_tid in neighbor_tids.iter().copied() {
            if keep_neighbor_tid(neighbor_tid) {
                let (neighbor, loaded_state) = unsafe {
                    cached_graph_element_with_prefetch(
                        index_relation,
                        opaque,
                        prefetched_buffers.as_ref(),
                        neighbor_tid,
                    )
                };
                if neighbor.deleted || neighbor.heaptids.is_empty() {
                    continue;
                }
                match candidate_score_dispatch(scan_graph_storage, &neighbor, loaded_state) {
                    CandidateScoreDispatch::Exact(loaded_state) => {
                        let score = unsafe {
                            exact_score_cached_graph_element(
                                index_relation,
                                opaque,
                                neighbor.tid,
                                loaded_state,
                            )
                        };
                        candidates.push(search::BeamCandidate::with_source(
                            neighbor.tid,
                            score,
                            source_tid,
                        ));
                    }
                    CandidateScoreDispatch::Grouped(grouped) => {
                        if let Some(grouped_candidates) = grouped_candidates.as_mut() {
                            let approx_score =
                                unsafe { score_grouped_candidate_context_approx(opaque, grouped) };
                            let ordinal = grouped_candidates.len();
                            grouped_candidates.push(GroupedTraversalCandidate {
                                ordinal,
                                element: neighbor,
                                approx_score,
                            });
                        } else {
                            let score = unsafe {
                                score_grouped_candidate_context(
                                    index_relation,
                                    opaque,
                                    grouped,
                                    layer,
                                )
                            };
                            candidates.push(search::BeamCandidate::with_source(
                                neighbor.tid,
                                score,
                                source_tid,
                            ));
                        }
                    }
                }
            }
        }

        if let Some(grouped_candidates) = grouped_candidates {
            candidates.extend(unsafe {
                score_budgeted_grouped_traversal_candidates(
                    index_relation,
                    opaque,
                    source_tid,
                    exact_budget.expect("grouped exact traversal budget should exist"),
                    grouped_candidates,
                )
            });
        }

        release_prefetched_graph_buffers_if_any(prefetched_buffers);
        return candidates;
    }

    let binary_query = binary_query.expect("binary query should remain available during scan");
    let quantizer = unsafe { &*(*opaque).cached_quantizer };
    let mut approx_candidates = Vec::with_capacity(capacity);

    for neighbor_tid in neighbor_tids.iter().copied() {
        if keep_neighbor_tid(neighbor_tid) {
            let (neighbor, loaded_state) = unsafe {
                cached_graph_element_with_prefetch(
                    index_relation,
                    opaque,
                    prefetched_buffers.as_ref(),
                    neighbor_tid,
                )
            };
            if neighbor.deleted || neighbor.heaptids.is_empty() {
                continue;
            }

            if let Some(score) = cached_scan_element_score(unsafe { &*opaque }, neighbor.tid) {
                record_score_cache_hit(unsafe { &mut *opaque });
                candidates.push(search::BeamCandidate::with_source(
                    neighbor.tid,
                    score,
                    source_tid,
                ));
                continue;
            }

            #[cfg(any(test, feature = "pg_test"))]
            let binary_started = Instant::now();
            let approx_score = -quantizer.score_binary_sign_words_no_qjl_4bit(
                binary_query,
                neighbor.binary_words.as_slice(),
            );
            #[cfg(any(test, feature = "pg_test"))]
            let binary_elapsed_us = u64::try_from(binary_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let binary_elapsed_us = 0;
            record_binary_prefilter_score_elapsed(unsafe { &mut *opaque }, binary_elapsed_us);
            approx_candidates.push(BinaryPrefilterCandidate {
                ordinal: approx_candidates.len(),
                element: neighbor,
                approx_score,
                loaded_state,
            });
        }
    }

    let survivor_budget = binary_prefilter_survivor_budget(approx_candidates.len());
    if survivor_budget < approx_candidates.len() {
        approx_candidates.sort_by(|left, right| left.approx_score.total_cmp(&right.approx_score));
        approx_candidates.truncate(survivor_budget);
        approx_candidates.sort_by_key(|candidate| candidate.ordinal);
    }
    record_binary_prefilter_survivors(unsafe { &mut *opaque }, approx_candidates.len());

    let mut grouped_candidates = exact_budget.map(|_| Vec::with_capacity(approx_candidates.len()));
    for candidate in approx_candidates {
        match candidate_score_dispatch(
            scan_graph_storage,
            &candidate.element,
            candidate.loaded_state,
        ) {
            CandidateScoreDispatch::Exact(loaded_state) => {
                let score = if turboquant_binary_live_rerank_enabled(unsafe { &*opaque }) {
                    candidate.approx_score
                } else {
                    unsafe {
                        exact_score_cached_graph_element(
                            index_relation,
                            opaque,
                            candidate.element.tid,
                            loaded_state,
                        )
                    }
                };
                candidates.push(search::BeamCandidate::with_source(
                    candidate.element.tid,
                    score,
                    source_tid,
                ));
            }
            CandidateScoreDispatch::Grouped(grouped) => {
                if let Some(grouped_candidates) = grouped_candidates.as_mut() {
                    let approx_score =
                        if grouped_binary_traversal_score_enabled(unsafe { &*opaque }) {
                            candidate.approx_score
                        } else {
                            unsafe { score_grouped_candidate_context_approx(opaque, grouped) }
                        };
                    grouped_candidates.push(GroupedTraversalCandidate {
                        ordinal: candidate.ordinal,
                        element: candidate.element,
                        approx_score,
                    });
                } else {
                    let score = if grouped_binary_traversal_score_enabled(unsafe { &*opaque })
                        && !grouped_exact_traversal_full_candidate_scoring_for_layer(
                            unsafe { &*opaque },
                            layer,
                        ) {
                        candidate.approx_score
                    } else {
                        unsafe {
                            score_grouped_candidate_context(index_relation, opaque, grouped, layer)
                        }
                    };
                    candidates.push(search::BeamCandidate::with_source(
                        candidate.element.tid,
                        score,
                        source_tid,
                    ));
                }
            }
        }
    }

    if let Some(grouped_candidates) = grouped_candidates {
        candidates.extend(unsafe {
            score_budgeted_grouped_traversal_candidates(
                index_relation,
                opaque,
                source_tid,
                exact_budget.expect("grouped exact traversal budget should exist"),
                grouped_candidates,
            )
        });
    }

    release_prefetched_graph_buffers_if_any(prefetched_buffers);
    candidates
}

unsafe fn cached_upper_layer_seed_candidate(
    index_relation: pg_sys::Relation,
    opaque: *mut TqScanOpaque,
    entry_candidate: search::BeamCandidate<page::ItemPointer>,
    entry_level: u8,
) -> search::BeamCandidate<page::ItemPointer> {
    if entry_level == 0 {
        return entry_candidate;
    }

    graph::greedy_descend_with_successors(
        entry_candidate,
        entry_level,
        |source_tid, layer| unsafe {
            cached_scan_successor_candidates_for_layer(
                index_relation,
                opaque,
                source_tid,
                layer,
                |_| true,
            )
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PendingScanOutput {
    heap_tid: page::ItemPointer,
    score: f32,
    approx_score: Option<f32>,
    approx_rank: Option<i32>,
    comparison_score: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParallelScanOutputState {
    Empty,
    Blocked(super::parallel::EcParallelOwnedOutputBlocker),
    Emitted(PendingScanOutput),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RetainedParallelOwnedOutputBlocker {
    blocker: super::parallel::EcParallelOwnedOutputBlocker,
    element_tid: page::ItemPointer,
}

#[derive(Debug, Clone, Copy)]
struct DeferredParallelBlockedOutput {
    source_phase: ScanExecutionPhase,
    state: ScanResultState,
    retained_blocker: Option<RetainedParallelOwnedOutputBlocker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockedParallelScanDisposition {
    KeepLocalEmit,
    RetryShared,
    DropAndContinue,
}

fn reconcile_parallel_owner_progress_from_shared_slot(opaque: &mut TqScanOpaque) -> bool {
    if opaque.parallel_scan_state.is_null()
        || opaque.parallel_scan_worker_slot_index == INVALID_PARALLEL_SCAN_WORKER_SLOT
        || opaque.parallel_local_only_output_active
    {
        return false;
    }

    let current = active_result_state_ref(opaque).current();
    if !current.has_element() {
        return false;
    }

    let shared_slot = unsafe {
        super::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
            opaque.parallel_scan_state,
            opaque.parallel_scan_worker_slot_index,
        )
    }
    .unwrap_or_else(|err| pgrx::error!("ec_hnsw parallel owner-slot read failed: {err}"));

    let current_element =
        parallel_item_pointer(active_result_state_ref(opaque).current().element_tid());

    if shared_slot.runtime.element_tid == current_element {
        let shared_pending_count = usize::try_from(shared_slot.runtime.pending_count)
            .expect("shared pending count should fit in usize");
        let local_pending_count = usize::from(active_result_state_ref(opaque).pending_count());
        let shared_pending_index = usize::try_from(shared_slot.runtime.pending_index)
            .expect("shared pending index should fit in usize");
        let local_pending_index = usize::from(active_result_state_ref(opaque).pending_index());

        let shared_pending_heap_tids = &shared_slot.runtime.pending_heap_tids
            [..shared_pending_count.min(page::HEAPTID_INLINE_CAPACITY)];
        let local_pending_heap_tids = active_result_state_ref(opaque).pending_heap_tids();

        if shared_pending_count == local_pending_count
            && shared_pending_heap_tids
                .iter()
                .copied()
                .map(item_pointer_from_parallel_item_pointer)
                .eq(local_pending_heap_tids.iter().copied())
            && shared_pending_index > local_pending_index
        {
            let result_state = active_result_state_mut(opaque);
            while usize::from(result_state.pending_index()) < shared_pending_index {
                if result_state.take_pending_output().is_none() {
                    break;
                }
            }
            if result_state.pending_count() == 0 {
                result_state.clear_current();
            }
            opaque.parallel_owned_output_blocker = None;
            sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
            return true;
        }

        return false;
    }

    if !shared_slot.runtime.element_tid.is_valid() {
        let shared_worker = unsafe {
            super::parallel::read_parallel_scan_worker_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .unwrap_or_else(|err| pgrx::error!("ec_hnsw parallel worker-slot read failed: {err}"));
        if shared_worker.runtime.active_result_has_current
            && active_result_state_ref(opaque).current().heap_tid() == page::ItemPointer::INVALID
        {
            let result_state = active_result_state_mut(opaque);
            if result_state.current().has_element() {
                result_state.clear();
                opaque.parallel_owned_output_blocker = None;
                publish_parallel_scan_worker_slot_snapshot(opaque);
                return true;
            }
        }
    }

    false
}

fn blocked_parallel_scan_disposition(
    opaque: &mut TqScanOpaque,
    blocker: super::parallel::EcParallelOwnedOutputBlocker,
) -> BlockedParallelScanDisposition {
    let element_tid = active_result_state_ref(opaque).current().element_tid();
    let blocker_element_tid = item_pointer_from_parallel_item_pointer(blocker.element_tid);
    match blocker.kind {
        super::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow => {
            opaque.retained_parallel_owned_output_blocker = None;
            BlockedParallelScanDisposition::DropAndContinue
        }
        super::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending
        | super::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead => {
            if blocker_element_tid != page::ItemPointer::INVALID
                && blocker_element_tid == element_tid
            {
                opaque.retained_parallel_owned_output_blocker = None;
                return BlockedParallelScanDisposition::DropAndContinue;
            }
            if reconcile_parallel_owner_progress_from_shared_slot(opaque) {
                opaque.retained_parallel_owned_output_blocker = None;
                return BlockedParallelScanDisposition::RetryShared;
            }
            let local_heap_tid = active_result_state_ref(opaque)
                .pending_heap_tids()
                .get(usize::from(active_result_state_ref(opaque).pending_index()))
                .copied()
                .unwrap_or(page::ItemPointer::INVALID);
            if local_heap_tid != page::ItemPointer::INVALID
                && live_foreign_blocker_heap_tid(opaque, blocker) == Some(local_heap_tid)
            {
                let pending_count = {
                    let result_state = active_result_state_mut(opaque);
                    let _ = result_state.take_pending_output();
                    let pending_count = result_state.pending_count();
                    if pending_count == 0 {
                        result_state.clear_current();
                    }
                    pending_count
                };
                opaque.parallel_owned_output_blocker = None;
                opaque.retained_parallel_owned_output_blocker = None;
                sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
                return if pending_count == 0 {
                    BlockedParallelScanDisposition::DropAndContinue
                } else {
                    BlockedParallelScanDisposition::RetryShared
                };
            }
            let previous = opaque
                .retained_parallel_owned_output_blocker
                .filter(|retained| retained.element_tid == element_tid)
                .map(|retained| retained.blocker);
            opaque.retained_parallel_owned_output_blocker = (element_tid
                != page::ItemPointer::INVALID)
                .then_some(RetainedParallelOwnedOutputBlocker {
                    blocker,
                    element_tid,
                });
            if previous == Some(blocker) {
                BlockedParallelScanDisposition::KeepLocalEmit
            } else {
                BlockedParallelScanDisposition::RetryShared
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BufferedGroupedScanResult {
    element_tid: page::ItemPointer,
    approx_score: f32,
    approx_rank_base: i32,
    comparison_score: Option<f32>,
    heap_tids: CachedHeapTids,
}

impl Default for BufferedGroupedScanResult {
    fn default() -> Self {
        Self {
            element_tid: page::ItemPointer::INVALID,
            approx_score: 0.0,
            approx_rank_base: 0,
            comparison_score: None,
            heap_tids: CachedHeapTids::default(),
        }
    }
}

struct GraphTraversalCursor<'a> {
    result_state: &'a mut ScanResultState,
}

impl<'a> GraphTraversalCursor<'a> {
    fn new(result_state: &'a mut ScanResultState) -> Self {
        Self { result_state }
    }

    fn has_prefetched_output(&self) -> bool {
        self.result_state.pending_count() != 0
    }

    fn prefetch_ready(&mut self) -> bool {
        if self.has_prefetched_output() {
            return true;
        }

        if self.result_state.current().has_element() {
            self.result_state.clear_current();
        }

        false
    }

    fn needs_prefetch_refresh(&self) -> bool {
        self.result_state.pending_count() == 0
    }

    fn take_pending_output(&mut self) -> Option<PendingScanOutput> {
        self.result_state.take_pending_output()
    }

    fn emit_prefetched_output(&mut self) -> Option<PendingScanOutput> {
        self.take_pending_output()
    }

    unsafe fn prefetch_next(
        &mut self,
        index_relation: pg_sys::Relation,
        opaque: *mut TqScanOpaque,
    ) -> bool {
        let result_state = self.result_state as *mut ScanResultState;
        unsafe {
            prefetch_next_graph_result_from_frontier(index_relation, &mut *opaque, result_state)
        }
    }

    unsafe fn ensure_prefetched_output(
        &mut self,
        index_relation: pg_sys::Relation,
        opaque: *mut TqScanOpaque,
    ) -> bool {
        let opaque = unsafe { &mut *opaque };
        if !opaque.execution_phase.is_graph_traversal() {
            return false;
        }

        if self.prefetch_ready() {
            return true;
        }

        if !unsafe { self.prefetch_next(index_relation, opaque as *mut TqScanOpaque) } {
            mark_scan_exhausted(opaque);
            return false;
        }

        true
    }
}

fn graph_traversal_cursor(opaque: &mut TqScanOpaque) -> GraphTraversalCursor<'_> {
    GraphTraversalCursor::new(&mut opaque.result_state)
}

struct LinearFallbackCursor<'a> {
    result_state: &'a mut ScanResultState,
}

impl<'a> LinearFallbackCursor<'a> {
    fn new(result_state: &'a mut ScanResultState) -> Self {
        Self { result_state }
    }

    fn materialize(&mut self, selected: SelectedScanResult) {
        self.result_state.materialize(selected);
    }

    fn take_pending_output(&mut self) -> Option<PendingScanOutput> {
        self.result_state.take_pending_output()
    }

    fn emit_pending_output(&mut self) -> Option<PendingScanOutput> {
        self.take_pending_output()
    }

    fn advance_after_emit(&mut self) {
        if self.result_state.pending_count() == 0 {
            self.result_state.clear_current();
        }
    }

    fn emit_materialized_output(
        &mut self,
        selected: SelectedScanResult,
    ) -> Option<PendingScanOutput> {
        self.materialize(selected);
        let emitted = self.emit_pending_output();
        debug_assert!(
            emitted.is_some(),
            "linear fallback result materialization should seed pending heap tids before returning true"
        );
        if emitted.is_some() {
            self.advance_after_emit();
        }
        emitted
    }
}

fn linear_fallback_cursor(opaque: &mut TqScanOpaque) -> LinearFallbackCursor<'_> {
    LinearFallbackCursor::new(&mut opaque.fallback_result_state)
}

fn item_pointer_from_parallel_item_pointer(
    tid: super::parallel::EcParallelItemPointer,
) -> page::ItemPointer {
    if !tid.is_valid() {
        return page::ItemPointer::INVALID;
    }

    page::ItemPointer {
        block_number: tid.block_number,
        offset_number: tid.offset_number,
    }
}

fn item_pointer_order_key(tid: page::ItemPointer) -> (u32, u16) {
    (tid.block_number, tid.offset_number)
}

fn deferred_parallel_blocked_output_order_key(
    deferred: &DeferredParallelBlockedOutput,
) -> (f32, (u32, u16)) {
    (
        deferred.state.current().score(),
        item_pointer_order_key(deferred.state.current().element_tid()),
    )
}

fn should_drop_deferred_parallel_blocked_output(deferred: &DeferredParallelBlockedOutput) -> bool {
    let Some(retained) = deferred.retained_blocker else {
        return false;
    };

    match retained.blocker.kind {
        super::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow => true,
        super::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending
        | super::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead => {
            let blocker_element_tid =
                item_pointer_from_parallel_item_pointer(retained.blocker.element_tid);
            blocker_element_tid != page::ItemPointer::INVALID
                && blocker_element_tid == deferred.state.current().element_tid()
        }
    }
}

fn live_foreign_blocker_heap_tid(
    opaque: &TqScanOpaque,
    blocker: super::parallel::EcParallelOwnedOutputBlocker,
) -> Option<page::ItemPointer> {
    if opaque.parallel_scan_state.is_null() {
        return None;
    }

    match blocker.kind {
        super::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending => {
            let expected_slot = blocker.slot_index?;
            let selected = unsafe {
                super::parallel::read_parallel_scan_selected_pending_output_snapshot(
                    opaque.parallel_scan_state,
                )
            }
            .unwrap_or_else(|err| {
                pgrx::error!(
                    "ec_hnsw parallel scan selected-pending live-blocker read failed: {err}"
                )
            })?;
            (selected.coordinator.selected_result_slot_index == Some(expected_slot)
                && selected.coordinator.result_publish_generation == blocker.generation)
                .then(|| item_pointer_from_parallel_item_pointer(selected.pending_output.heap_tid))
        }
        super::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead => {
            let admitted = unsafe {
                super::parallel::read_parallel_scan_admitted_head_snapshot(
                    opaque.parallel_scan_state,
                )
            }
            .unwrap_or_else(|err| {
                pgrx::error!("ec_hnsw parallel scan admitted-head live-blocker read failed: {err}")
            })?;
            (admitted.coordinator.admitted_result_generation == blocker.generation)
                .then(|| item_pointer_from_parallel_item_pointer(admitted.admitted_head.heap_tid))
        }
        super::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow => None,
    }
}

fn deferred_parallel_blocked_output_duplicates_live_foreign_heap_tid(
    opaque: &TqScanOpaque,
    deferred: &DeferredParallelBlockedOutput,
) -> bool {
    let Some(retained) = deferred.retained_blocker else {
        return false;
    };
    if opaque.parallel_scan_state.is_null() || deferred.state.pending_count() == 0 {
        return false;
    }

    let local_heap_tid = deferred
        .state
        .pending_heap_tids()
        .get(usize::from(deferred.state.pending_index()))
        .copied()
        .unwrap_or(page::ItemPointer::INVALID);
    if local_heap_tid == page::ItemPointer::INVALID {
        return false;
    }

    live_foreign_blocker_heap_tid(opaque, retained.blocker) == Some(local_heap_tid)
}

fn sort_deferred_parallel_blocked_outputs(opaque: &mut TqScanOpaque) {
    opaque
        .deferred_parallel_blocked_results
        .sort_by(|left, right| {
            deferred_parallel_blocked_output_order_key(left)
                .0
                .total_cmp(&deferred_parallel_blocked_output_order_key(right).0)
                .then_with(|| {
                    deferred_parallel_blocked_output_order_key(left)
                        .1
                        .cmp(&deferred_parallel_blocked_output_order_key(right).1)
                })
        });
}

fn best_deferred_parallel_blocked_output(
    opaque: &TqScanOpaque,
) -> Option<&DeferredParallelBlockedOutput> {
    opaque
        .deferred_parallel_blocked_results
        .iter()
        .min_by(|left, right| {
            deferred_parallel_blocked_output_order_key(left)
                .0
                .total_cmp(&deferred_parallel_blocked_output_order_key(right).0)
                .then_with(|| {
                    deferred_parallel_blocked_output_order_key(left)
                        .1
                        .cmp(&deferred_parallel_blocked_output_order_key(right).1)
                })
        })
}

fn deferred_parallel_blocked_output_preference_score(
    opaque: &TqScanOpaque,
    deferred: &DeferredParallelBlockedOutput,
) -> Option<f32> {
    if !deferred.state.current().has_element() || deferred.state.pending_count() == 0 {
        return None;
    }

    let Some(retained) = deferred.retained_blocker else {
        return Some(deferred.state.current().score());
    };

    if should_drop_deferred_parallel_blocked_output(deferred)
        || opaque.parallel_scan_state.is_null()
    {
        return None;
    }

    match retained.blocker.kind {
        super::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending => {
            let expected_slot = retained.blocker.slot_index?;
            let selected = unsafe {
                super::parallel::read_parallel_scan_selected_pending_output_snapshot(
                    opaque.parallel_scan_state,
                )
            }
            .unwrap_or_else(|err| {
                pgrx::error!("ec_hnsw parallel scan selected-pending preference read failed: {err}")
            })?;
            (selected.coordinator.selected_result_slot_index == Some(expected_slot)
                && selected.coordinator.result_publish_generation == retained.blocker.generation)
                .then_some(selected.pending_output.score)
        }
        super::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead => {
            let admitted = unsafe {
                super::parallel::read_parallel_scan_admitted_head_snapshot(
                    opaque.parallel_scan_state,
                )
            }
            .unwrap_or_else(|err| {
                pgrx::error!("ec_hnsw parallel scan admitted-head preference read failed: {err}")
            })?;
            (admitted.coordinator.admitted_result_generation == retained.blocker.generation)
                .then_some(admitted.admitted_head.score)
        }
        super::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow => None,
    }
}

fn best_preferable_deferred_parallel_blocked_output(
    opaque: &TqScanOpaque,
) -> Option<(DeferredParallelBlockedOutput, f32)> {
    opaque
        .deferred_parallel_blocked_results
        .iter()
        .filter_map(|deferred| {
            let preference_score =
                deferred_parallel_blocked_output_preference_score(opaque, deferred)?;
            Some((*deferred, preference_score))
        })
        .min_by(|(left, left_score), (right, right_score)| {
            left_score.total_cmp(right_score).then_with(|| {
                item_pointer_order_key(left.state.current().element_tid())
                    .cmp(&item_pointer_order_key(right.state.current().element_tid()))
            })
        })
}

fn should_prefer_deferred_parallel_blocked_output(
    opaque: &TqScanOpaque,
) -> Option<DeferredParallelBlockedOutput> {
    let (deferred, deferred_score) = best_preferable_deferred_parallel_blocked_output(opaque)?;
    let active = active_result_state_ref(opaque).current();
    if !active.has_element() {
        return Some(deferred);
    }

    if active.score() <= deferred_score {
        return None;
    }

    Some(deferred)
}

fn try_emit_preferred_deferred_parallel_blocked_output(
    scan: pg_sys::IndexScanDesc,
    opaque: &mut TqScanOpaque,
) -> bool {
    let Some(output) = take_preferred_deferred_parallel_blocked_output(opaque) else {
        return false;
    };
    emit_scan_output(scan, opaque, output);
    opaque.explain_counters.record_heap_tid_returned();
    true
}

fn pending_scan_output_from_parallel_pending_output(
    output: super::parallel::EcParallelPendingOutputSnapshot,
) -> PendingScanOutput {
    PendingScanOutput {
        heap_tid: item_pointer_from_parallel_item_pointer(output.heap_tid),
        score: output.score,
        approx_score: output.approx_score,
        approx_rank: output.approx_rank,
        comparison_score: output.comparison_score,
    }
}

pub(super) fn active_result_state_ref(opaque: &TqScanOpaque) -> &ScanResultState {
    if opaque.execution_phase == ScanExecutionPhase::LinearFallback {
        &opaque.fallback_result_state
    } else {
        &opaque.result_state
    }
}

fn active_result_state_mut(opaque: &mut TqScanOpaque) -> &mut ScanResultState {
    if opaque.execution_phase == ScanExecutionPhase::LinearFallback {
        &mut opaque.fallback_result_state
    } else {
        &mut opaque.result_state
    }
}

fn active_result_state_ref_for_phase(
    opaque: &TqScanOpaque,
    phase: ScanExecutionPhase,
) -> &ScanResultState {
    if phase == ScanExecutionPhase::LinearFallback {
        &opaque.fallback_result_state
    } else {
        &opaque.result_state
    }
}

fn active_result_state_mut_for_phase(
    opaque: &mut TqScanOpaque,
    phase: ScanExecutionPhase,
) -> &mut ScanResultState {
    if phase == ScanExecutionPhase::LinearFallback {
        &mut opaque.fallback_result_state
    } else {
        &mut opaque.result_state
    }
}

fn consume_parallel_scan_admitted_result(
    opaque: &mut TqScanOpaque,
    admitted_result: super::parallel::EcParallelCoordinatorAdmittedResultSnapshot,
) -> PendingScanOutput {
    mark_emitted_element(
        opaque,
        item_pointer_from_parallel_item_pointer(admitted_result.element_tid),
    );

    if admitted_result.source_slot_index == Some(opaque.parallel_scan_worker_slot_index) {
        let element_tid = item_pointer_from_parallel_item_pointer(admitted_result.element_tid);
        let expected_heap_tid =
            item_pointer_from_parallel_item_pointer(admitted_result.pending_output.heap_tid);
        let result_state = active_result_state_mut(opaque);
        if result_state.current().element_tid() == element_tid {
            if let Some(local_output) = result_state.take_pending_output() {
                debug_assert_eq!(
                    local_output.heap_tid, expected_heap_tid,
                    "local scan result state should stay aligned with the admitted heap-tid drain for this worker slot"
                );
            }
            if result_state.pending_count() == 0 {
                result_state.clear_current();
            }
        }
    }

    pending_scan_output_from_parallel_pending_output(admitted_result.pending_output)
}

unsafe fn try_take_parallel_scan_handoff_output(
    opaque: &mut TqScanOpaque,
    blocker: super::parallel::EcParallelOwnedOutputBlocker,
) -> Option<PendingScanOutput> {
    if opaque.parallel_scan_state.is_null() {
        return None;
    }

    match blocker.kind {
        super::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending
        | super::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead => {}
        super::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow => {
            return None;
        }
    }

    if opaque.parallel_scan_worker_slot_index == INVALID_PARALLEL_SCAN_WORKER_SLOT {
        return None;
    }

    let admitted_result = match blocker.kind {
        super::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending => {
            let slot_index = blocker.slot_index?;
            unsafe {
                super::parallel::take_parallel_scan_foreign_selected_pending_output_snapshot(
                    opaque.parallel_scan_state,
                    slot_index,
                    blocker.generation,
                )
            }
            .unwrap_or_else(|err| {
                pgrx::error!("ec_hnsw parallel scan foreign-selected handoff failed: {err}")
            })?
        }
        super::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead => unsafe {
            super::parallel::take_parallel_scan_admitted_result_snapshot(opaque.parallel_scan_state)
        }
        .unwrap_or_else(|err| {
            pgrx::error!("ec_hnsw parallel scan admitted-head handoff failed: {err}")
        })?,
        super::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow => return None,
    };

    opaque.parallel_local_only_output_active = false;
    opaque.parallel_owned_output_blocker = None;
    opaque.retained_parallel_owned_output_blocker = None;
    let output = consume_parallel_scan_admitted_result(opaque, admitted_result.admitted_result);
    sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
    Some(output)
}

fn parallel_scan_admission_result_limit() -> u32 {
    // The planner-visible LIMIT budget is not wired into the scan descriptor yet.
    // Until that seam lands, let the shared admission path use the full staged
    // descriptor capacity instead of baking in a fake query limit here.
    u32::MAX
}

unsafe fn try_take_parallel_scan_next_output(opaque: &mut TqScanOpaque) -> ParallelScanOutputState {
    if opaque.parallel_scan_state.is_null()
        || opaque.parallel_scan_worker_slot_index == INVALID_PARALLEL_SCAN_WORKER_SLOT
    {
        return ParallelScanOutputState::Empty;
    }

    opaque.parallel_local_only_output_active = false;
    opaque.parallel_owned_output_blocker = None;
    sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
    match unsafe {
        super::parallel::read_parallel_scan_owned_output_state(
            opaque.parallel_scan_state,
            opaque.parallel_scan_worker_slot_index,
            parallel_scan_admission_result_limit(),
        )
    }
    .unwrap_or_else(|err| pgrx::error!("ec_hnsw parallel scan readiness read failed: {err}"))
    {
        super::parallel::EcParallelOwnedOutputState::Empty => {
            return ParallelScanOutputState::Empty;
        }
        super::parallel::EcParallelOwnedOutputState::Blocked(blocker) => {
            if let Some(output) = unsafe { try_take_parallel_scan_handoff_output(opaque, blocker) }
            {
                return ParallelScanOutputState::Emitted(output);
            }
            match blocker.kind {
                super::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending => {
                    opaque
                        .explain_counters
                        .record_parallel_blocked_foreign_selected_pending();
                }
                super::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead => {
                    opaque
                        .explain_counters
                        .record_parallel_blocked_foreign_admitted_head();
                }
                super::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow => {
                    opaque
                        .explain_counters
                        .record_parallel_blocked_admission_window();
                }
            }
            opaque.parallel_owned_output_blocker = Some(blocker);
            sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
            return ParallelScanOutputState::Blocked(blocker);
        }
        super::parallel::EcParallelOwnedOutputState::Ready => {}
    }

    let admitted_result = unsafe {
        super::parallel::take_parallel_scan_owned_next_output_snapshot(
            opaque.parallel_scan_state,
            opaque.parallel_scan_worker_slot_index,
            parallel_scan_admission_result_limit(),
        )
    }
    .unwrap_or_else(|err| pgrx::error!("ec_hnsw parallel scan next-output take failed: {err}"))
    .unwrap_or_else(|| {
        pgrx::error!("ec_hnsw parallel scan ready state produced no owned output to take")
    });
    opaque.parallel_local_only_output_active = false;
    opaque.parallel_owned_output_blocker = None;
    opaque.retained_parallel_owned_output_blocker = None;
    let output = consume_parallel_scan_admitted_result(opaque, admitted_result.admitted_result);
    sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
    ParallelScanOutputState::Emitted(output)
}

unsafe fn emit_materialized_parallel_scan_output(
    opaque: &mut TqScanOpaque,
    selected: SelectedScanResult,
) -> ParallelScanOutputState {
    if opaque.parallel_scan_state.is_null()
        || opaque.parallel_scan_worker_slot_index == INVALID_PARALLEL_SCAN_WORKER_SLOT
    {
        return ParallelScanOutputState::Empty;
    }

    opaque.parallel_local_only_output_active = false;
    opaque.retained_parallel_owned_output_blocker = None;
    active_result_state_mut(opaque).materialize(selected);
    unsafe { try_take_parallel_scan_next_output(opaque) }
}

unsafe fn try_take_parallel_scan_deferred_handoff_output(
    opaque: &mut TqScanOpaque,
    deferred: &mut DeferredParallelBlockedOutput,
) -> Option<PendingScanOutput> {
    let saved_phase = opaque.execution_phase;
    opaque.execution_phase = deferred.source_phase;
    *active_result_state_mut_for_phase(opaque, deferred.source_phase) = deferred.state;
    opaque.parallel_local_only_output_active = false;
    opaque.parallel_owned_output_blocker = None;
    opaque.retained_parallel_owned_output_blocker = deferred.retained_blocker;

    let output_state = unsafe { try_take_parallel_scan_next_output(opaque) };
    let remaining_state = *active_result_state_ref_for_phase(opaque, deferred.source_phase);
    let remaining_retained_blocker = opaque
        .retained_parallel_owned_output_blocker
        .filter(|retained| retained.element_tid == remaining_state.current().element_tid());
    active_result_state_mut_for_phase(opaque, deferred.source_phase).clear();
    opaque.execution_phase = saved_phase;
    opaque.parallel_local_only_output_active = false;
    opaque.parallel_owned_output_blocker = None;
    opaque.retained_parallel_owned_output_blocker = None;

    match output_state {
        ParallelScanOutputState::Emitted(output) => {
            *deferred = DeferredParallelBlockedOutput {
                source_phase: deferred.source_phase,
                state: remaining_state,
                retained_blocker: remaining_retained_blocker,
            };
            if !deferred.state.current().has_element() || deferred.state.pending_count() == 0 {
                deferred.state.clear();
                deferred.retained_blocker = None;
            }
            Some(output)
        }
        ParallelScanOutputState::Blocked(blocker) => {
            *deferred = DeferredParallelBlockedOutput {
                source_phase: deferred.source_phase,
                state: remaining_state,
                retained_blocker: remaining_state.current().has_element().then_some(
                    RetainedParallelOwnedOutputBlocker {
                        blocker,
                        element_tid: remaining_state.current().element_tid(),
                    },
                ),
            };
            None
        }
        ParallelScanOutputState::Empty => {
            *deferred = DeferredParallelBlockedOutput {
                source_phase: deferred.source_phase,
                state: remaining_state,
                retained_blocker: remaining_retained_blocker,
            };
            None
        }
    }
}

unsafe fn emit_prefetched_parallel_scan_output(
    opaque: &mut TqScanOpaque,
) -> ParallelScanOutputState {
    if opaque.parallel_scan_state.is_null()
        || opaque.parallel_scan_worker_slot_index == INVALID_PARALLEL_SCAN_WORKER_SLOT
        || !graph_traversal_cursor(opaque).has_prefetched_output()
    {
        return ParallelScanOutputState::Empty;
    }

    unsafe { try_take_parallel_scan_next_output(opaque) }
}

fn discard_active_parallel_scan_output(opaque: &mut TqScanOpaque) {
    active_result_state_mut(opaque).clear();
    opaque.parallel_local_only_output_active = false;
    opaque.parallel_owned_output_blocker = None;
    opaque.retained_parallel_owned_output_blocker = None;
    if !opaque.parallel_scan_state.is_null()
        && opaque.parallel_scan_worker_slot_index != INVALID_PARALLEL_SCAN_WORKER_SLOT
    {
        sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
    }
}

fn clear_deferred_parallel_blocked_outputs(opaque: &mut TqScanOpaque) {
    opaque.deferred_parallel_blocked_results.clear();
}

fn stash_active_parallel_blocked_output(opaque: &mut TqScanOpaque) -> bool {
    let active = *active_result_state_ref(opaque);
    if !active.current().has_element() || active.pending_count() == 0 {
        return false;
    }

    opaque
        .deferred_parallel_blocked_results
        .push(DeferredParallelBlockedOutput {
            source_phase: opaque.execution_phase,
            state: active,
            retained_blocker: opaque
                .retained_parallel_owned_output_blocker
                .filter(|retained| retained.element_tid == active.current().element_tid()),
        });
    sort_deferred_parallel_blocked_outputs(opaque);
    active_result_state_mut(opaque).clear();
    opaque.parallel_local_only_output_active = false;
    opaque.parallel_owned_output_blocker = None;
    opaque.retained_parallel_owned_output_blocker = None;
    if !opaque.parallel_scan_state.is_null()
        && opaque.parallel_scan_worker_slot_index != INVALID_PARALLEL_SCAN_WORKER_SLOT
    {
        sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
    }
    true
}

fn restore_deferred_parallel_blocked_outputs(
    opaque: &mut TqScanOpaque,
    deferred_rows: impl IntoIterator<Item = DeferredParallelBlockedOutput>,
) {
    opaque
        .deferred_parallel_blocked_results
        .extend(deferred_rows);
    if opaque.deferred_parallel_blocked_results.is_empty() {
        opaque.retained_parallel_owned_output_blocker = None;
    } else {
        sort_deferred_parallel_blocked_outputs(opaque);
    }
    if !opaque.parallel_scan_state.is_null()
        && opaque.parallel_scan_worker_slot_index != INVALID_PARALLEL_SCAN_WORKER_SLOT
    {
        sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
    }
}

fn take_next_deferred_parallel_blocked_output(
    opaque: &mut TqScanOpaque,
    allow_local_emit: bool,
) -> Option<PendingScanOutput> {
    let mut blocked_fallback = Vec::new();

    while let Some((selected_idx, _)) = opaque
        .deferred_parallel_blocked_results
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            deferred_parallel_blocked_output_order_key(left)
                .0
                .total_cmp(&deferred_parallel_blocked_output_order_key(right).0)
                .then_with(|| {
                    deferred_parallel_blocked_output_order_key(left)
                        .1
                        .cmp(&deferred_parallel_blocked_output_order_key(right).1)
                })
        })
    {
        let mut deferred = opaque
            .deferred_parallel_blocked_results
            .remove(selected_idx);
        if deferred.retained_blocker.is_some() {
            if let Some(output) =
                unsafe { try_take_parallel_scan_deferred_handoff_output(opaque, &mut deferred) }
            {
                if deferred.state.current().has_element() && deferred.state.pending_count() != 0 {
                    blocked_fallback.push(deferred);
                }
                restore_deferred_parallel_blocked_outputs(opaque, blocked_fallback);
                return Some(output);
            }
            if should_drop_deferred_parallel_blocked_output(&deferred) {
                continue;
            }
            if !allow_local_emit {
                blocked_fallback.push(deferred);
                continue;
            }
            blocked_fallback.push(deferred);
            continue;
        }
        let Some(output) = deferred.state.take_pending_output() else {
            continue;
        };
        mark_emitted_element(opaque, deferred.state.current().element_tid());
        if deferred.state.pending_count() != 0 && deferred.state.current().has_element() {
            blocked_fallback.push(deferred);
        }
        opaque.explain_counters.record_heap_tid_returned();
        restore_deferred_parallel_blocked_outputs(opaque, blocked_fallback);
        return Some(output);
    }

    if !allow_local_emit {
        restore_deferred_parallel_blocked_outputs(opaque, blocked_fallback);
        return None;
    }

    loop {
        let Some((selected_idx, _)) =
            blocked_fallback
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    deferred_parallel_blocked_output_order_key(left)
                        .0
                        .total_cmp(&deferred_parallel_blocked_output_order_key(right).0)
                        .then_with(|| {
                            deferred_parallel_blocked_output_order_key(left)
                                .1
                                .cmp(&deferred_parallel_blocked_output_order_key(right).1)
                        })
                })
        else {
            restore_deferred_parallel_blocked_outputs(opaque, blocked_fallback);
            return None;
        };

        let mut deferred = blocked_fallback.swap_remove(selected_idx);
        if deferred_parallel_blocked_output_duplicates_live_foreign_heap_tid(opaque, &deferred) {
            let _ = deferred.state.take_pending_output();
            if deferred.state.pending_count() != 0 && deferred.state.current().has_element() {
                if let Some(retained_blocker) = deferred.retained_blocker {
                    if let Some(output) = unsafe {
                        try_take_parallel_scan_handoff_output(opaque, retained_blocker.blocker)
                    } {
                        if deferred.state.current().has_element()
                            && deferred.state.pending_count() != 0
                        {
                            blocked_fallback.push(deferred);
                        }
                        restore_deferred_parallel_blocked_outputs(opaque, blocked_fallback);
                        return Some(output);
                    }
                    if should_drop_deferred_parallel_blocked_output(&deferred) {
                        continue;
                    }
                }
            }
            if deferred.state.pending_count() != 0 && deferred.state.current().has_element() {
                blocked_fallback.push(deferred);
            }
            continue;
        }

        let output = deferred.state.take_pending_output()?;
        mark_emitted_element(opaque, deferred.state.current().element_tid());
        if deferred.state.pending_count() != 0 && deferred.state.current().has_element() {
            blocked_fallback.push(deferred);
        }
        opaque
            .explain_counters
            .record_parallel_deferred_local_emit();
        opaque.explain_counters.record_heap_tid_returned();
        restore_deferred_parallel_blocked_outputs(opaque, blocked_fallback);
        return Some(output);
    }
}

fn take_preferred_deferred_parallel_blocked_output(
    opaque: &mut TqScanOpaque,
) -> Option<PendingScanOutput> {
    should_prefer_deferred_parallel_blocked_output(opaque)?;
    take_next_deferred_parallel_blocked_output(opaque, false)
}

fn emit_next_deferred_parallel_blocked_output(
    scan: pg_sys::IndexScanDesc,
    opaque: &mut TqScanOpaque,
) -> bool {
    let Some(output) = take_next_deferred_parallel_blocked_output(opaque, true) else {
        return false;
    };
    emit_scan_output(scan, opaque, output);
    opaque.explain_counters.record_heap_tid_returned();
    true
}

unsafe fn produce_next_scan_heap_tid(
    scan: pg_sys::IndexScanDesc,
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    code_len: usize,
) -> bool {
    if let ParallelScanOutputState::Emitted(output) =
        unsafe { try_take_parallel_scan_next_output(opaque) }
    {
        emit_scan_output(scan, opaque, output);
        opaque.explain_counters.record_heap_tid_returned();
        if opaque.execution_phase.is_graph_traversal()
            && !graph_traversal_cursor(opaque).has_prefetched_output()
        {
            let opaque_ptr = opaque as *mut TqScanOpaque;
            unsafe {
                graph_traversal_cursor(opaque).ensure_prefetched_output(index_relation, opaque_ptr);
            }
        }
        return true;
    }

    if try_emit_preferred_deferred_parallel_blocked_output(scan, opaque) {
        return true;
    }

    let produced = match opaque.execution_phase {
        ScanExecutionPhase::GraphTraversal => unsafe {
            produce_next_graph_traversal_heap_tid(scan, index_relation, opaque)
        },
        ScanExecutionPhase::LinearFallback => unsafe {
            produce_next_linear_fallback_heap_tid(scan, index_relation, opaque, code_len)
        },
        ScanExecutionPhase::Exhausted => false,
    };

    if produced {
        return true;
    }

    if opaque.execution_phase.is_exhausted() {
        return emit_next_deferred_parallel_blocked_output(scan, opaque);
    }

    false
}

fn clear_scan_candidate_state(opaque: &mut TqScanOpaque) {
    visible_frontier_mut(opaque).clear();
}

fn clear_graph_traversal_state(opaque: &mut TqScanOpaque) {
    clear_grouped_live_rerank_buffer(opaque);
    clear_scan_candidate_state(opaque);
    reset_bootstrap_expansion_state(opaque, bootstrap_frontier_limit(opaque));
    reset_scan_expanded_state(opaque);
}

fn grouped_live_rerank_enabled(opaque: &TqScanOpaque) -> bool {
    matches!(
        opaque.scan_graph_storage,
        graph::GraphStorageDescriptor::PqFastScan(_)
    ) || turboquant_binary_live_rerank_enabled(opaque)
}

fn clear_grouped_live_rerank_buffer(opaque: &mut TqScanOpaque) {
    opaque
        .grouped_live_rerank_buffer
        .fill(BufferedGroupedScanResult::default());
    opaque.grouped_live_rerank_buffer_len = 0;
    opaque.grouped_live_rerank_next_approx_rank = 1;
}

fn grouped_live_rerank_window(opaque: &TqScanOpaque) -> usize {
    usize::from(opaque.grouped_live_rerank_window)
}

fn buffered_grouped_scan_results(opaque: &TqScanOpaque) -> &[BufferedGroupedScanResult] {
    &opaque.grouped_live_rerank_buffer[..usize::from(opaque.grouped_live_rerank_buffer_len)]
}

fn push_buffered_grouped_scan_result(
    opaque: &mut TqScanOpaque,
    buffered: BufferedGroupedScanResult,
) {
    let len = usize::from(opaque.grouped_live_rerank_buffer_len);
    assert!(
        len < grouped_live_rerank_window(opaque),
        "grouped live rerank buffer should respect configured window capacity"
    );
    opaque.grouped_live_rerank_buffer[len] = buffered;
    opaque.grouped_live_rerank_buffer_len += 1;
}

fn pop_best_buffered_grouped_scan_result(
    opaque: &mut TqScanOpaque,
) -> Option<BufferedGroupedScanResult> {
    let buffer_len = usize::from(opaque.grouped_live_rerank_buffer_len);
    if buffer_len == 0 {
        return None;
    }

    let selected_idx = buffered_grouped_scan_results(opaque)
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            let left_exact = left.comparison_score.unwrap_or(left.approx_score);
            let right_exact = right.comparison_score.unwrap_or(right.approx_score);
            left_exact
                .total_cmp(&right_exact)
                .then_with(|| left.approx_rank_base.cmp(&right.approx_rank_base))
        })
        .map(|(idx, _)| idx)
        .expect("grouped live rerank buffer should have a best candidate when non-empty");
    let selected = opaque.grouped_live_rerank_buffer[selected_idx];
    for idx in selected_idx..buffer_len.saturating_sub(1) {
        opaque.grouped_live_rerank_buffer[idx] = opaque.grouped_live_rerank_buffer[idx + 1];
    }
    opaque.grouped_live_rerank_buffer[buffer_len - 1] = BufferedGroupedScanResult::default();
    opaque.grouped_live_rerank_buffer_len -= 1;
    Some(selected)
}

fn grouped_live_rerank_output_score(
    opaque: &TqScanOpaque,
    buffered: &BufferedGroupedScanResult,
) -> f32 {
    if grouped_heap_rerank_enabled(opaque)
        || matches!(
            opaque.scan_graph_storage,
            graph::GraphStorageDescriptor::TurboQuant { .. }
                | graph::GraphStorageDescriptor::TurboQuantHotCold(_)
        )
    {
        buffered.comparison_score.unwrap_or(buffered.approx_score)
    } else {
        buffered.approx_score
    }
}

unsafe fn prefetch_next_graph_result_from_frontier(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    result_state: *mut ScanResultState,
) -> bool {
    if !opaque.execution_phase.is_graph_traversal()
        || opaque.scan_dimensions == 0
        || unsafe { (&*result_state).pending_count() != 0 }
    {
        return false;
    }

    if grouped_live_rerank_enabled(opaque) {
        return unsafe {
            prefetch_next_grouped_windowed_graph_result(index_relation, opaque, result_state)
        };
    }

    loop {
        unsafe { refine_grouped_frontier_head_exact(index_relation, opaque) };
        #[cfg(any(test, feature = "pg_test"))]
        let consume_started = Instant::now();
        let candidate = consume_candidate_frontier_head(opaque);
        #[cfg(any(test, feature = "pg_test"))]
        let consume_elapsed_us =
            u64::try_from(consume_started.elapsed().as_micros()).expect("timing should fit in u64");
        #[cfg(not(any(test, feature = "pg_test")))]
        let consume_elapsed_us = 0;
        record_frontier_consume_elapsed(opaque, consume_elapsed_us);
        let Some(candidate) = candidate else {
            break;
        };

        mark_expanded_source(opaque, candidate.node);
        opaque.explain_counters.record_bootstrap_expansion();
        super::stats::record_graph_hop();
        opaque.stats_delta.record_graph_hop();
        #[cfg(any(test, feature = "pg_test"))]
        let materialize_started = Instant::now();
        if unsafe {
            materialize_graph_result_candidate(index_relation, opaque, result_state, candidate)
        }
        .is_some()
        {
            #[cfg(any(test, feature = "pg_test"))]
            let materialize_elapsed_us = u64::try_from(materialize_started.elapsed().as_micros())
                .expect("timing should fit in u64");
            #[cfg(not(any(test, feature = "pg_test")))]
            let materialize_elapsed_us = 0;
            record_graph_result_materialize_elapsed(opaque, materialize_elapsed_us);
            return true;
        }

        #[cfg(any(test, feature = "pg_test"))]
        let materialize_elapsed_us = u64::try_from(materialize_started.elapsed().as_micros())
            .expect("timing should fit in u64");
        #[cfg(not(any(test, feature = "pg_test")))]
        let materialize_elapsed_us = 0;
        record_graph_result_materialize_elapsed(opaque, materialize_elapsed_us);
    }

    false
}

unsafe fn prefetch_next_grouped_windowed_graph_result(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    result_state: *mut ScanResultState,
) -> bool {
    let active_window = grouped_live_rerank_window(opaque);
    while usize::from(opaque.grouped_live_rerank_buffer_len) < active_window {
        unsafe { refine_grouped_frontier_head_exact(index_relation, opaque) };
        #[cfg(any(test, feature = "pg_test"))]
        let consume_started = Instant::now();
        let candidate = consume_candidate_frontier_head(opaque);
        #[cfg(any(test, feature = "pg_test"))]
        let consume_elapsed_us =
            u64::try_from(consume_started.elapsed().as_micros()).expect("timing should fit in u64");
        #[cfg(not(any(test, feature = "pg_test")))]
        let consume_elapsed_us = 0;
        record_frontier_consume_elapsed(opaque, consume_elapsed_us);
        let Some(candidate) = candidate else {
            break;
        };

        mark_expanded_source(opaque, candidate.node);
        opaque.explain_counters.record_bootstrap_expansion();
        super::stats::record_graph_hop();
        opaque.stats_delta.record_graph_hop();
        #[cfg(any(test, feature = "pg_test"))]
        let materialize_started = Instant::now();
        unsafe { buffer_grouped_graph_result_candidate(index_relation, opaque, candidate) };
        #[cfg(any(test, feature = "pg_test"))]
        let materialize_elapsed_us = u64::try_from(materialize_started.elapsed().as_micros())
            .expect("timing should fit in u64");
        #[cfg(not(any(test, feature = "pg_test")))]
        let materialize_elapsed_us = 0;
        record_graph_result_materialize_elapsed(opaque, materialize_elapsed_us);
    }

    let Some(buffered) = pop_best_buffered_grouped_scan_result(opaque) else {
        return false;
    };

    let output_score = grouped_live_rerank_output_score(opaque, &buffered);
    let result_state = unsafe { &mut *result_state };
    result_state.materialize_with_details(
        buffered.element_tid,
        output_score,
        Some(buffered.approx_score),
        Some(buffered.approx_rank_base),
        buffered.comparison_score,
        buffered.heap_tids.as_slice(),
    );
    true
}

unsafe fn buffer_grouped_graph_result_candidate(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    candidate: search::BeamCandidate<page::ItemPointer>,
) {
    if staged_or_emitted_contains_element(opaque, candidate.node) {
        opaque.explain_counters.record_element_skipped();
        return;
    }

    opaque.explain_counters.record_bootstrap_page_read();
    let (element, _) = unsafe { cached_graph_element(index_relation, opaque, candidate.node) };
    if element.deleted || element.heaptids.is_empty() {
        opaque.explain_counters.record_element_skipped();
        return;
    }

    let comparison_score = unsafe {
        grouped_candidate_rerank_comparison_score(
            index_relation,
            opaque as *mut TqScanOpaque,
            &element,
        )
    };
    let approx_rank_base = opaque.grouped_live_rerank_next_approx_rank;
    let emitted_heap_rows =
        i32::try_from(element.heaptids.as_slice().len()).expect("heap tid count should fit in i32");
    opaque.grouped_live_rerank_next_approx_rank = opaque
        .grouped_live_rerank_next_approx_rank
        .checked_add(emitted_heap_rows)
        .expect("grouped approx rank should remain in i32 range");
    opaque.explain_counters.record_element_scored();
    push_buffered_grouped_scan_result(
        opaque,
        BufferedGroupedScanResult {
            element_tid: candidate.node,
            approx_score: candidate.score,
            approx_rank_base,
            comparison_score,
            heap_tids: element.heaptids,
        },
    );
}

unsafe fn materialize_graph_result_candidate(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    result_state: *mut ScanResultState,
    candidate: search::BeamCandidate<page::ItemPointer>,
) -> Option<()> {
    if staged_or_emitted_contains_element(opaque, candidate.node) {
        opaque.explain_counters.record_element_skipped();
        return None;
    }

    opaque.explain_counters.record_bootstrap_page_read();
    let (element, _) = unsafe { cached_graph_element(index_relation, opaque, candidate.node) };
    if element.deleted || element.heaptids.is_empty() {
        opaque.explain_counters.record_element_skipped();
        return None;
    }

    // Keep traversal/output ordering on the grouped approximate score for now, but
    // capture the cold rerank score alongside emitted results so the next packets
    // can compare approximate-vs-exact behavior on real scan outputs.
    let comparison_score = unsafe {
        grouped_candidate_rerank_comparison_score(
            index_relation,
            opaque as *mut TqScanOpaque,
            &element,
        )
    };
    opaque.explain_counters.record_element_scored();
    let result_state = unsafe { &mut *result_state };
    result_state.materialize_with_details(
        candidate.node,
        candidate.score,
        None,
        None,
        comparison_score,
        element.heaptids.as_slice(),
    );
    Some(())
}

fn enter_linear_fallback_phase(opaque: &mut TqScanOpaque) {
    clear_graph_traversal_state(opaque);
    opaque.fallback_result_state.clear();
    opaque.execution_phase = ScanExecutionPhase::LinearFallback;
    opaque.stats_used_linear_fallback = true;
    sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
}

fn mark_scan_exhausted(opaque: &mut TqScanOpaque) {
    clear_graph_traversal_state(opaque);
    opaque.result_state.clear();
    opaque.fallback_result_state.clear();
    opaque.execution_phase = ScanExecutionPhase::Exhausted;
    finalize_scan_stats(opaque);
    sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
}

fn reset_bootstrap_expansion_state(opaque: &mut TqScanOpaque, ef_search: usize) {
    let ef_search = ef_search.max(1);
    if opaque.bootstrap_expansion.is_null() {
        opaque.bootstrap_expansion = Box::into_raw(Box::new(search::BeamSearch::new(ef_search)));
    } else {
        *unsafe { &mut *opaque.bootstrap_expansion } = search::BeamSearch::new(ef_search);
    }
}

fn bootstrap_frontier_limit(opaque: &TqScanOpaque) -> usize {
    opaque.bootstrap_frontier_limit.max(1)
}

fn free_scan_candidate_frontier(opaque: &mut TqScanOpaque) {
    if !opaque.candidate_frontier.is_null() {
        drop(unsafe { Box::from_raw(opaque.candidate_frontier) });
        opaque.candidate_frontier = ptr::null_mut();
    }
}

fn free_bootstrap_expansion(opaque: &mut TqScanOpaque) {
    if !opaque.bootstrap_expansion.is_null() {
        drop(unsafe { Box::from_raw(opaque.bootstrap_expansion) });
        opaque.bootstrap_expansion = ptr::null_mut();
    }
}

fn free_graph_prefetch_state(opaque: &mut TqScanOpaque) {
    if !opaque.graph_prefetch_state.is_null() {
        drop(unsafe { Box::from_raw(opaque.graph_prefetch_state) });
        opaque.graph_prefetch_state = ptr::null_mut();
    }
}

fn reset_graph_prefetch_state(opaque: &mut TqScanOpaque) {
    if opaque.graph_prefetch_state.is_null() {
        opaque.graph_prefetch_state = Box::into_raw(Box::new(GraphPrefetchState::new(Vec::new())));
    } else {
        unsafe { &mut *opaque.graph_prefetch_state }.reset(Vec::new());
    }
}

#[cfg(feature = "pg18")]
fn reset_graph_prefetch_blocks(opaque: &mut TqScanOpaque, blocks: Vec<u32>) {
    if opaque.graph_prefetch_state.is_null() {
        opaque.graph_prefetch_state = Box::into_raw(Box::new(GraphPrefetchState::new(blocks)));
    } else {
        unsafe { &mut *opaque.graph_prefetch_state }.reset(blocks);
    }
}

fn reset_linear_prefetch_state(opaque: &mut TqScanOpaque) {
    let first = page::FIRST_DATA_BLOCK_NUMBER;
    let max_block = opaque.scan_block_count.saturating_sub(1).max(first);
    opaque.linear_prefetch_state.reset(first, max_block);
}

#[cfg(feature = "pg18")]
fn end_read_stream(stream: &mut *mut pg_sys::ReadStream) {
    if !(*stream).is_null() {
        unsafe { pg_sys::read_stream_end(*stream) };
        *stream = ptr::null_mut();
    }
}

#[cfg(feature = "pg18")]
fn ensure_graph_read_stream(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> *mut pg_sys::ReadStream {
    if opaque.graph_prefetch_state.is_null() {
        reset_graph_prefetch_state(opaque);
    }
    if opaque.graph_read_stream.is_null() {
        opaque.graph_read_stream = unsafe {
            pg_sys::read_stream_begin_relation(
                pg_sys::READ_STREAM_DEFAULT as i32,
                ptr::null_mut(),
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                Some(super::stream::graph_prefetch_cb),
                opaque.graph_prefetch_state.cast(),
                std::mem::size_of::<pg_sys::BlockNumber>(),
            )
        };
    }
    opaque.graph_read_stream
}

#[cfg(feature = "pg18")]
fn ensure_linear_read_stream(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> *mut pg_sys::ReadStream {
    if opaque.linear_read_stream.is_null() {
        opaque.linear_read_stream = unsafe {
            pg_sys::read_stream_begin_relation(
                pg_sys::READ_STREAM_SEQUENTIAL as i32,
                ptr::null_mut(),
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                Some(super::stream::linear_prefetch_cb),
                (&mut opaque.linear_prefetch_state as *mut LinearPrefetchState).cast(),
                std::mem::size_of::<pg_sys::BlockNumber>(),
            )
        };
    }
    opaque.linear_read_stream
}

fn finalize_scan_stats(opaque: &mut TqScanOpaque) {
    if opaque.stats_scan_finalized || !opaque.rescan_called {
        return;
    }
    if !opaque.stats_used_linear_fallback {
        super::stats::record_bootstrap_only_scan();
        opaque.stats_delta.record_bootstrap_only_scan();
    }
    opaque.stats_scan_finalized = true;
}

fn flush_scan_stats(opaque: &mut TqScanOpaque) {
    if !opaque.rescan_called || opaque.stats_delta.is_zero() {
        return;
    }
    // Shared pgstat snapshots are updated during scan teardown/rescan, not mid-scan.
    super::stats::flush_shared_delta(opaque.stats_delta);
    opaque.stats_delta.reset();
}

type VisibleCandidateFrontierState = search::VisibleFrontier<page::ItemPointer>;

static EMPTY_VISIBLE_FRONTIER_STATE: VisibleCandidateFrontierState =
    VisibleCandidateFrontierState::empty();

fn visible_frontier_ref(opaque: &TqScanOpaque) -> &VisibleCandidateFrontierState {
    if opaque.candidate_frontier.is_null() {
        &EMPTY_VISIBLE_FRONTIER_STATE
    } else {
        unsafe { &*opaque.candidate_frontier }
    }
}

fn visible_frontier_mut(opaque: &mut TqScanOpaque) -> &mut VisibleCandidateFrontierState {
    if opaque.candidate_frontier.is_null() {
        opaque.candidate_frontier =
            Box::into_raw(Box::new(VisibleCandidateFrontierState::default()));
    }
    unsafe { &mut *opaque.candidate_frontier }
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) fn visible_frontier_candidates(
    opaque: &TqScanOpaque,
) -> Vec<search::BeamCandidate<page::ItemPointer>> {
    visible_frontier_ref(opaque).iter().collect()
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) fn visible_frontier_slot(
    opaque: &TqScanOpaque,
    index: usize,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    visible_frontier_ref(opaque).slot(index)
}

#[cfg(any(test, feature = "pg_test"))]
fn with_visible_frontier_and_bootstrap_expansion<R>(
    opaque: &mut TqScanOpaque,
    f: impl FnOnce(&VisibleCandidateFrontierState, &mut search::BeamSearch<page::ItemPointer>) -> R,
) -> R {
    let visible_frontier = visible_frontier_ref(opaque) as *const VisibleCandidateFrontierState;
    let expansion = bootstrap_expansion_mut(opaque) as *mut search::BeamSearch<page::ItemPointer>;
    // SAFETY: `candidate_frontier` and `bootstrap_expansion` are separate Box-backed heap
    // allocations owned by `TqScanOpaque`, so borrowing the frontier immutably and the
    // scheduler mutably at the same time cannot alias.
    unsafe { f(&*visible_frontier, &mut *expansion) }
}

fn with_visible_frontier_mut_and_bootstrap_expansion<R>(
    opaque: &mut TqScanOpaque,
    f: impl FnOnce(&mut VisibleCandidateFrontierState, &mut search::BeamSearch<page::ItemPointer>) -> R,
) -> R {
    let visible_frontier = visible_frontier_mut(opaque) as *mut VisibleCandidateFrontierState;
    let expansion = bootstrap_expansion_mut(opaque) as *mut search::BeamSearch<page::ItemPointer>;
    // SAFETY: `candidate_frontier` and `bootstrap_expansion` are separate Box-backed heap
    // allocations owned by `TqScanOpaque`, so borrowing the frontier and the scheduler mutably
    // at the same time cannot alias.
    unsafe { f(&mut *visible_frontier, &mut *expansion) }
}

#[cfg(any(test, feature = "pg_test"))]
fn candidate_frontier_head(
    opaque: &mut TqScanOpaque,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    with_visible_frontier_and_bootstrap_expansion(opaque, |visible_frontier, expansion| {
        visible_frontier.best_candidate(expansion)
    })
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) fn current_candidate_frontier_head(
    opaque: &mut TqScanOpaque,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    candidate_frontier_head(opaque)
}

fn bootstrap_expansion_mut(
    opaque: &mut TqScanOpaque,
) -> &mut search::BeamSearch<page::ItemPointer> {
    if opaque.bootstrap_expansion.is_null() {
        reset_bootstrap_expansion_state(opaque, bootstrap_frontier_limit(opaque));
    }
    unsafe { &mut *opaque.bootstrap_expansion }
}

fn reset_scan_visited_state(opaque: &mut TqScanOpaque) {
    if opaque.visited_tids.is_null() {
        opaque.visited_tids = Box::into_raw(Box::new(HashSet::new()));
    } else {
        unsafe { &mut *opaque.visited_tids }.clear();
    }
}

fn free_scan_visited_set(opaque: &mut TqScanOpaque) {
    if !opaque.visited_tids.is_null() {
        drop(unsafe { Box::from_raw(opaque.visited_tids) });
        opaque.visited_tids = ptr::null_mut();
    }
}

fn mark_visited_element(opaque: &mut TqScanOpaque, element_tid: page::ItemPointer) {
    if opaque.visited_tids.is_null() || element_tid == page::ItemPointer::INVALID {
        return;
    }

    unsafe { &mut *opaque.visited_tids }.insert(element_tid);
}

fn visited_contains_element(opaque: &TqScanOpaque, element_tid: page::ItemPointer) -> bool {
    if opaque.visited_tids.is_null() || element_tid == page::ItemPointer::INVALID {
        return false;
    }

    unsafe { &*opaque.visited_tids }.contains(&element_tid)
}

fn reset_scan_expanded_state(opaque: &mut TqScanOpaque) {
    if opaque.expanded_source_tids.is_null() {
        opaque.expanded_source_tids = Box::into_raw(Box::new(HashSet::new()));
    } else {
        unsafe { &mut *opaque.expanded_source_tids }.clear();
    }
}

fn free_scan_expanded_set(opaque: &mut TqScanOpaque) {
    if !opaque.expanded_source_tids.is_null() {
        drop(unsafe { Box::from_raw(opaque.expanded_source_tids) });
        opaque.expanded_source_tids = ptr::null_mut();
    }
}

fn mark_expanded_source(opaque: &mut TqScanOpaque, source_tid: page::ItemPointer) {
    if opaque.expanded_source_tids.is_null() || source_tid == page::ItemPointer::INVALID {
        return;
    }

    unsafe { &mut *opaque.expanded_source_tids }.insert(source_tid);
}

fn expanded_contains_source(opaque: &TqScanOpaque, source_tid: page::ItemPointer) -> bool {
    if opaque.expanded_source_tids.is_null() || source_tid == page::ItemPointer::INVALID {
        return false;
    }

    unsafe { &*opaque.expanded_source_tids }.contains(&source_tid)
}

fn reset_scan_emitted_state(opaque: &mut TqScanOpaque) {
    if opaque.emitted_result_tids.is_null() {
        opaque.emitted_result_tids = Box::into_raw(Box::new(HashSet::new()));
    } else {
        unsafe { &mut *opaque.emitted_result_tids }.clear();
    }
}

fn free_scan_emitted_set(opaque: &mut TqScanOpaque) {
    if !opaque.emitted_result_tids.is_null() {
        drop(unsafe { Box::from_raw(opaque.emitted_result_tids) });
        opaque.emitted_result_tids = ptr::null_mut();
    }
}

fn mark_emitted_element(opaque: &mut TqScanOpaque, element_tid: page::ItemPointer) {
    if opaque.emitted_result_tids.is_null() || element_tid == page::ItemPointer::INVALID {
        return;
    }

    unsafe { &mut *opaque.emitted_result_tids }.insert(element_tid);
}

fn emitted_contains_element(opaque: &TqScanOpaque, element_tid: page::ItemPointer) -> bool {
    if opaque.emitted_result_tids.is_null() || element_tid == page::ItemPointer::INVALID {
        return false;
    }

    unsafe { &*opaque.emitted_result_tids }.contains(&element_tid)
}

fn staged_or_emitted_contains_element(
    opaque: &TqScanOpaque,
    element_tid: page::ItemPointer,
) -> bool {
    if element_tid == page::ItemPointer::INVALID {
        return false;
    }

    emitted_contains_element(opaque, element_tid)
        || opaque.result_state.current().element_tid() == element_tid
        || opaque.fallback_result_state.current().element_tid() == element_tid
        || opaque
            .deferred_parallel_blocked_results
            .iter()
            .any(|deferred| deferred.state.current().element_tid() == element_tid)
}

unsafe fn initialize_scan_entry_candidate(
    index_relation: pg_sys::Relation,
    _heap_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    metadata: &page::MetadataPage,
) {
    clear_scan_candidate_state(opaque);
    if metadata.dimensions == 0 {
        return;
    }

    let entry_candidate = if metadata.entry_point != page::ItemPointer::INVALID {
        let (entry, entry_score) = unsafe {
            cached_graph_element_and_score(
                index_relation,
                opaque,
                metadata.entry_point,
                metadata.max_level,
            )
        };
        (!entry.deleted && !entry.heaptids.is_empty()).then_some((entry, entry_score))
    } else {
        None
    };
    let (entry, entry_score) = match entry_candidate {
        Some(candidate) => candidate,
        None => {
            let Some(fallback) = (unsafe {
                super::shared::highest_level_live_entry_candidate(
                    index_relation,
                    opaque.scan_graph_storage,
                )
            }) else {
                return;
            };
            let (entry, entry_score) = unsafe {
                cached_graph_element_and_score(index_relation, opaque, fallback.tid, fallback.level)
            };
            if entry.deleted || entry.heaptids.is_empty() {
                return;
            }
            (entry, entry_score)
        }
    };

    let entry_candidate = search::BeamCandidate::new(
        entry.tid,
        entry_score.expect("live entry candidates should have a cached score"),
    );
    let opaque_ptr = opaque as *mut TqScanOpaque;
    #[cfg(any(test, feature = "pg_test"))]
    let upper_layer_started = Instant::now();
    let upper_layer_seed = unsafe {
        cached_upper_layer_seed_candidate(index_relation, opaque_ptr, entry_candidate, entry.level)
    };
    #[cfg(any(test, feature = "pg_test"))]
    let upper_layer_elapsed_us =
        u64::try_from(upper_layer_started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let upper_layer_elapsed_us = 0;
    record_upper_layer_seed_elapsed(opaque, upper_layer_elapsed_us);
    #[cfg(any(test, feature = "pg_test"))]
    let layer0_started = Instant::now();
    let ordered_candidates = graph::search_layer0_result_candidates_with_successors(
        bootstrap_frontier_limit(opaque),
        [upper_layer_seed],
        |source_tid| unsafe {
            cached_scan_successor_candidates_for_layer(
                index_relation,
                opaque_ptr,
                source_tid,
                0,
                |neighbor_tid| !visited_contains_element(&*opaque_ptr, neighbor_tid),
            )
        },
    );
    #[cfg(any(test, feature = "pg_test"))]
    let layer0_elapsed_us =
        u64::try_from(layer0_started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let layer0_elapsed_us = 0;
    record_layer0_seed_elapsed(opaque, layer0_elapsed_us);
    #[cfg(any(test, feature = "pg_test"))]
    let stage_started = Instant::now();
    stage_ordered_graph_results(opaque, ordered_candidates);
    #[cfg(any(test, feature = "pg_test"))]
    let stage_elapsed_us =
        u64::try_from(stage_started.elapsed().as_micros()).expect("timing should fit in u64");
    #[cfg(not(any(test, feature = "pg_test")))]
    let stage_elapsed_us = 0;
    record_stage_ordered_results_elapsed(opaque, stage_elapsed_us);
}

fn stage_ordered_graph_results(
    opaque: &mut TqScanOpaque,
    candidates: impl IntoIterator<Item = search::BeamCandidate<page::ItemPointer>>,
) {
    let candidates = parallel_scan_worker_bootstrap_candidates(
        opaque,
        candidates.into_iter().collect::<Vec<_>>(),
    );
    clear_scan_candidate_state(opaque);
    reset_bootstrap_expansion_state(opaque, bootstrap_frontier_limit(opaque));
    reset_scan_expanded_state(opaque);
    seed_discovered_candidates(opaque, candidates);
}

fn parallel_scan_worker_bootstrap_candidates(
    opaque: &TqScanOpaque,
    candidates: Vec<search::BeamCandidate<page::ItemPointer>>,
) -> Vec<search::BeamCandidate<page::ItemPointer>> {
    if candidates.len() <= 1
        || opaque.parallel_scan_state.is_null()
        || opaque.parallel_scan_worker_slot_index == INVALID_PARALLEL_SCAN_WORKER_SLOT
    {
        return candidates;
    }

    let worker_slot_count = match usize::try_from(opaque.parallel_scan_worker_slot_count) {
        Ok(worker_slot_count) if worker_slot_count > 1 => worker_slot_count,
        _ => return candidates,
    };
    let worker_slot_index = match usize::try_from(opaque.parallel_scan_worker_slot_index) {
        Ok(worker_slot_index) if worker_slot_index < worker_slot_count => worker_slot_index,
        _ => return candidates,
    };

    let mut candidates = candidates;
    let head = candidates[0];
    let mut tail = candidates.split_off(1);
    if tail.is_empty() {
        return vec![head];
    }

    let worker_seed = splitmix64(
        opaque.scan_seed
            ^ u64::from(opaque.parallel_scan_worker_slot_index)
            ^ u64::from(opaque.parallel_scan_worker_slot_count) << 32,
    );
    let rotation = usize::try_from(worker_seed % tail.len() as u64)
        .expect("tail rotation should fit in usize");
    tail.rotate_left(rotation);

    let mut selected = Vec::with_capacity(1 + tail.len().div_ceil(worker_slot_count));
    selected.push(head);
    selected.extend(
        tail.into_iter()
            .skip(worker_slot_index)
            .step_by(worker_slot_count),
    );
    selected
}

fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = (state ^ (state >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^ (state >> 31)
}

#[cfg(any(test, feature = "pg_test"))]
fn seed_bootstrap_trace(
    opaque: &mut TqScanOpaque,
    max_candidates: usize,
    trace: search::BeamTrace<page::ItemPointer>,
) {
    reset_bootstrap_expansion_state(opaque, max_candidates);
    reset_scan_expanded_state(opaque);
    let opaque_ptr = opaque as *mut TqScanOpaque;
    with_visible_frontier_mut_and_bootstrap_expansion(
        unsafe { &mut *opaque_ptr },
        |visible_frontier, expansion| {
            visible_frontier.seed_bootstrap_trace(
                expansion,
                trace,
                max_candidates,
                |node| mark_visited_element(unsafe { &mut *opaque_ptr }, node),
                |node| mark_expanded_source(unsafe { &mut *opaque_ptr }, node),
            );
        },
    );
}

fn seed_discovered_candidates(
    opaque: &mut TqScanOpaque,
    candidates: impl IntoIterator<Item = impl Into<search::BeamCandidate<page::ItemPointer>>>,
) {
    let candidates = candidates.into_iter().map(Into::into).collect::<Vec<_>>();
    if candidates.is_empty() {
        return;
    }

    let opaque_ptr = opaque as *mut TqScanOpaque;
    with_visible_frontier_mut_and_bootstrap_expansion(
        unsafe { &mut *opaque_ptr },
        |visible_frontier, expansion| {
            visible_frontier.seed_discovered(expansion, candidates, |node| {
                mark_visited_element(unsafe { &mut *opaque_ptr }, node)
            });
        },
    );
}

fn seed_existing_frontier_into_expansion(opaque: &mut TqScanOpaque) {
    let candidates = visible_frontier_ref(opaque)
        .iter()
        .filter(|candidate| !expanded_contains_source(opaque, candidate.node))
        .collect::<Vec<_>>();
    bootstrap_expansion_mut(opaque).seed_many(candidates);
}

#[cfg(any(test, feature = "pg_test"))]
fn fill_bootstrap_frontier<F>(
    opaque: &mut TqScanOpaque,
    max_candidates: usize,
    policy: BootstrapExpandPolicy,
    refill: F,
) where
    F: FnMut(page::ItemPointer, &mut TqScanOpaque),
{
    reset_bootstrap_expansion_state(opaque, max_candidates);
    reset_scan_expanded_state(opaque);
    seed_existing_frontier_into_expansion(opaque);
    top_up_bootstrap_frontier(opaque, max_candidates, policy, refill);
}

#[cfg(any(test, feature = "pg_test"))]
fn top_up_bootstrap_frontier<F>(
    opaque: &mut TqScanOpaque,
    max_candidates: usize,
    policy: BootstrapExpandPolicy,
    mut refill: F,
) where
    F: FnMut(page::ItemPointer, &mut TqScanOpaque),
{
    while visible_frontier_ref(opaque).len() < max_candidates {
        let source_tid = match policy {
            BootstrapExpandPolicy::ScoreOrder => bootstrap_expansion_mut(opaque)
                .expand_one(|_| std::iter::empty::<search::BeamCandidate<page::ItemPointer>>())
                .map(|candidate| candidate.node),
        };
        let Some(source_tid) = source_tid else {
            break;
        };

        if expanded_contains_source(opaque, source_tid) {
            continue;
        }
        mark_expanded_source(opaque, source_tid);
        refill(source_tid, opaque);
    }
}

fn consume_candidate_frontier_head(
    opaque: &mut TqScanOpaque,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    with_visible_frontier_mut_and_bootstrap_expansion(opaque, |visible_frontier, expansion| {
        visible_frontier.consume_best(expansion)
    })
}

unsafe fn refine_grouped_frontier_head_exact(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) {
    if !grouped_exact_traversal_frontier_head_enabled(opaque) {
        return;
    }

    loop {
        let candidate = with_visible_frontier_mut_and_bootstrap_expansion(
            opaque,
            |visible_frontier, expansion| visible_frontier.best_candidate(expansion),
        );
        let Some(candidate) = candidate else {
            return;
        };
        if cached_scan_element_score(opaque, candidate.node).is_some() {
            return;
        }

        let opaque_ptr = opaque as *mut TqScanOpaque;
        let (element, loaded_state) =
            unsafe { cached_graph_element(index_relation, opaque_ptr, candidate.node) };
        if element.deleted || element.heaptids.is_empty() {
            return;
        }

        let exact_score =
            match candidate_score_dispatch(opaque.scan_graph_storage, &element, loaded_state) {
                CandidateScoreDispatch::Exact(loaded_state) => unsafe {
                    exact_score_cached_graph_element(
                        index_relation,
                        opaque_ptr,
                        element.tid,
                        loaded_state,
                    )
                },
                CandidateScoreDispatch::Grouped(grouped) => unsafe {
                    score_grouped_candidate_context_exact(index_relation, opaque_ptr, grouped)
                },
            };
        let updated = search::BeamCandidate {
            score: exact_score,
            ..candidate
        };
        let replaced =
            with_visible_frontier_mut_and_bootstrap_expansion(opaque, |visible_frontier, _| {
                visible_frontier.replace_candidate(updated)
            });
        if !replaced {
            return;
        }

        reset_bootstrap_expansion_state(opaque, bootstrap_frontier_limit(opaque));
        seed_existing_frontier_into_expansion(opaque);
    }
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn refill_candidate_frontier_from_source_into(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    visible_frontier: &mut VisibleCandidateFrontierState,
    expansion: &mut search::BeamSearch<page::ItemPointer>,
    source_tid: page::ItemPointer,
) {
    let opaque_ptr = opaque as *mut TqScanOpaque;
    visible_frontier.refill_from_source(
        expansion,
        bootstrap_frontier_limit(unsafe { &*opaque_ptr }),
        source_tid,
        |source_tid, max_successor_candidates| unsafe {
            graph::load_layer0_refill_successors_with_storage(
                index_relation,
                (&*opaque_ptr).scan_graph_storage,
                usize::from((&*opaque_ptr).scan_m),
                source_tid,
                max_successor_candidates,
                |neighbor_tid| !visited_contains_element(&*opaque_ptr, neighbor_tid),
                |neighbor| {
                    Some(score_scan_element_result(
                        &mut *opaque_ptr,
                        neighbor.gamma,
                        &neighbor.code,
                    ))
                },
            )
        },
        |node| mark_visited_element(unsafe { &mut *opaque_ptr }, node),
    );
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn top_up_bootstrap_frontier_from_visible_seeds_into(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    visible_frontier: &mut VisibleCandidateFrontierState,
    expansion: &mut search::BeamSearch<page::ItemPointer>,
) {
    let opaque_ptr = opaque as *mut TqScanOpaque;
    visible_frontier.top_up_from_visible_seeds(
        expansion,
        bootstrap_frontier_limit(unsafe { &*opaque_ptr }),
        |node| expanded_contains_source(unsafe { &*opaque_ptr }, node),
        |seed_candidates, max_successor_candidates| {
            let expansion_trace = unsafe {
                graph::expand_layer0_visible_seeds_with_storage(
                    index_relation,
                    (&*opaque_ptr).scan_graph_storage,
                    usize::from((&*opaque_ptr).scan_m),
                    max_successor_candidates,
                    seed_candidates.iter().copied(),
                    |neighbor_tid| !visited_contains_element(&*opaque_ptr, neighbor_tid),
                    |neighbor| {
                        Some(score_scan_element_result(
                            &mut *opaque_ptr,
                            neighbor.gamma,
                            &neighbor.code,
                        ))
                    },
                )
            };
            (
                expansion_trace.expanded_source_tids,
                expansion_trace.discovered_candidates,
            )
        },
        |node| mark_expanded_source(unsafe { &mut *opaque_ptr }, node),
        |node| mark_visited_element(unsafe { &mut *opaque_ptr }, node),
    );
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn refill_bootstrap_frontier_after_success(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    consumed: search::BeamCandidate<page::ItemPointer>,
) {
    let opaque_ptr = opaque as *mut TqScanOpaque;
    with_visible_frontier_mut_and_bootstrap_expansion(
        unsafe { &mut *opaque_ptr },
        |visible_frontier, expansion| unsafe {
            visible_frontier.advance_after_consume(
                expansion,
                consumed,
                |node| expanded_contains_source(&*opaque_ptr, node),
                |node| mark_expanded_source(&mut *opaque_ptr, node),
                |source_tid, visible_frontier, expansion| {
                    refill_candidate_frontier_from_source_into(
                        index_relation,
                        &mut *opaque_ptr,
                        visible_frontier,
                        expansion,
                        source_tid,
                    );
                },
                |visible_frontier, expansion| {
                    top_up_bootstrap_frontier_from_visible_seeds_into(
                        index_relation,
                        &mut *opaque_ptr,
                        visible_frontier,
                        expansion,
                    );
                },
            );
        },
    );
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) unsafe fn consume_and_refill_bootstrap_frontier(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    let consumed = consume_candidate_frontier_head(opaque)?;
    unsafe { refill_bootstrap_frontier_after_success(index_relation, opaque, consumed) };
    Some(consumed)
}

#[cfg(any(test, feature = "pg_test"))]
fn seed_scan_result_state(opaque: &mut TqScanOpaque, selected: SelectedScanResult) {
    opaque.result_state.materialize(selected);
}

#[cfg(any(test, feature = "pg_test"))]
fn refill_bootstrap_frontier_after_consume<F>(
    opaque: &mut TqScanOpaque,
    consumed: search::BeamCandidate<page::ItemPointer>,
    mut refill: F,
) where
    F: FnMut(page::ItemPointer, &mut TqScanOpaque),
{
    if !expanded_contains_source(opaque, consumed.node) {
        mark_expanded_source(opaque, consumed.node);
        refill(consumed.node, opaque);
    }

    top_up_bootstrap_frontier(
        opaque,
        bootstrap_frontier_limit(opaque),
        BootstrapExpandPolicy::ScoreOrder,
        refill,
    );
}

#[cfg(test)]
fn select_next_bootstrap_candidate<CandidateFn, SelectFn>(
    mut next_candidate: CandidateFn,
    mut select: SelectFn,
) -> Option<SelectedScanResult>
where
    CandidateFn: FnMut() -> Option<search::BeamCandidate<page::ItemPointer>>,
    SelectFn: FnMut(search::BeamCandidate<page::ItemPointer>) -> Option<SelectedScanResult>,
{
    while let Some(candidate) = next_candidate() {
        if let Some(selected) = select(candidate) {
            return Some(selected);
        }
    }

    None
}

#[cfg(test)]
fn select_next_bootstrap_candidate_with_refill<CandidateFn, SelectFn, RefillFn>(
    mut next_candidate: CandidateFn,
    mut select: SelectFn,
    mut refill_after_success: RefillFn,
) -> Option<SelectedScanResult>
where
    CandidateFn: FnMut() -> Option<search::BeamCandidate<page::ItemPointer>>,
    SelectFn: FnMut(search::BeamCandidate<page::ItemPointer>) -> Option<SelectedScanResult>,
    RefillFn: FnMut(search::BeamCandidate<page::ItemPointer>),
{
    while let Some(candidate) = next_candidate() {
        if let Some(selected) = select(candidate) {
            refill_after_success(candidate);
            return Some(selected);
        }
    }

    None
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) unsafe fn prefetch_next_graph_traversal_result(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> bool {
    if !opaque.execution_phase.is_graph_traversal() || opaque.scan_dimensions == 0 {
        return false;
    }

    let opaque_ptr = opaque as *mut TqScanOpaque;
    unsafe { graph_traversal_cursor(opaque).prefetch_next(index_relation, opaque_ptr) }
}

unsafe fn produce_next_graph_traversal_heap_tid(
    scan: pg_sys::IndexScanDesc,
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> bool {
    if !opaque.execution_phase.is_graph_traversal()
        || !graph_traversal_cursor(opaque).has_prefetched_output()
    {
        debug_assert!(
            opaque.execution_phase.is_exhausted(),
            "graph traversal tuple production should only run with prefetched output or an exhausted graph phase"
        );
        return false;
    }

    loop {
        match unsafe { emit_prefetched_parallel_scan_output(opaque) } {
            ParallelScanOutputState::Emitted(output) => {
                mark_emitted_element(opaque, opaque.result_state.current().element_tid());
                emit_scan_output(scan, opaque, output);
                opaque.explain_counters.record_heap_tid_returned();
                if graph_traversal_cursor(opaque).needs_prefetch_refresh() {
                    let opaque_ptr = opaque as *mut TqScanOpaque;
                    unsafe {
                        graph_traversal_cursor(opaque)
                            .ensure_prefetched_output(index_relation, opaque_ptr);
                    }
                }
                return true;
            }
            ParallelScanOutputState::Blocked(blocker) => {
                match blocked_parallel_scan_disposition(opaque, blocker) {
                    BlockedParallelScanDisposition::DropAndContinue => {
                        discard_active_parallel_scan_output(opaque);
                        let opaque_ptr = opaque as *mut TqScanOpaque;
                        if !unsafe {
                            graph_traversal_cursor(opaque)
                                .ensure_prefetched_output(index_relation, opaque_ptr)
                        } {
                            return false;
                        }
                        continue;
                    }
                    BlockedParallelScanDisposition::RetryShared => continue,
                    BlockedParallelScanDisposition::KeepLocalEmit => {
                        if !stash_active_parallel_blocked_output(opaque) {
                            break;
                        }
                        let opaque_ptr = opaque as *mut TqScanOpaque;
                        if !unsafe {
                            graph_traversal_cursor(opaque)
                                .ensure_prefetched_output(index_relation, opaque_ptr)
                        } {
                            return false;
                        }
                        continue;
                    }
                }
            }
            ParallelScanOutputState::Empty => break,
        }
    }

    let emitted = graph_traversal_cursor(opaque)
        .emit_prefetched_output()
        .map(|output| {
            mark_emitted_element(opaque, opaque.result_state.current().element_tid());
            emit_scan_output(scan, opaque, output);
            opaque.parallel_local_only_output_active =
                active_result_state_ref(opaque).current().has_element();
            opaque.parallel_owned_output_blocker = None;
            sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
            true
        })
        .unwrap_or(false);
    debug_assert!(
        emitted,
        "graph traversal should materialize pending output before returning true from graph-phase tuple production"
    );
    if emitted {
        opaque.explain_counters.record_heap_tid_returned();
    }
    if emitted && graph_traversal_cursor(opaque).needs_prefetch_refresh() {
        let opaque_ptr = opaque as *mut TqScanOpaque;
        unsafe {
            graph_traversal_cursor(opaque).ensure_prefetched_output(index_relation, opaque_ptr);
        }
    }
    emitted
}

unsafe fn produce_next_linear_fallback_heap_tid(
    scan: pg_sys::IndexScanDesc,
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    code_len: usize,
) -> bool {
    if linear_fallback_cursor(opaque)
        .emit_pending_output()
        .map(|output| {
            mark_emitted_element(opaque, opaque.fallback_result_state.current().element_tid());
            emit_scan_output(scan, opaque, output);
            true
        })
        .unwrap_or(false)
    {
        linear_fallback_cursor(opaque).advance_after_emit();
        opaque.explain_counters.record_heap_tid_returned();
        return true;
    }

    loop {
        let Some(selected) =
            (unsafe { select_next_linear_scan_result(index_relation, opaque, code_len) })
        else {
            return false;
        };

        match unsafe { emit_materialized_parallel_scan_output(opaque, selected) } {
            ParallelScanOutputState::Emitted(output) => {
                mark_emitted_element(opaque, selected.element_tid);
                emit_scan_output(scan, opaque, output);
                opaque.explain_counters.record_heap_tid_returned();
                return true;
            }
            ParallelScanOutputState::Blocked(blocker) => {
                match blocked_parallel_scan_disposition(opaque, blocker) {
                    BlockedParallelScanDisposition::DropAndContinue => {
                        discard_active_parallel_scan_output(opaque);
                        continue;
                    }
                    BlockedParallelScanDisposition::RetryShared => continue,
                    BlockedParallelScanDisposition::KeepLocalEmit => {
                        if !stash_active_parallel_blocked_output(opaque) {
                            continue;
                        }
                        continue;
                    }
                }
            }
            ParallelScanOutputState::Empty => {
                let emitted = linear_fallback_cursor(opaque)
                    .emit_materialized_output(selected)
                    .map(|output| {
                        mark_emitted_element(opaque, selected.element_tid);
                        emit_scan_output(scan, opaque, output);
                        opaque.parallel_local_only_output_active =
                            active_result_state_ref(opaque).current().has_element();
                        opaque.parallel_owned_output_blocker = None;
                        sync_and_publish_parallel_scan_worker_slot_snapshot(opaque);
                        true
                    })
                    .unwrap_or(false);
                if emitted {
                    opaque.explain_counters.record_heap_tid_returned();
                }
                return emitted;
            }
        }
    }
}

unsafe fn select_next_linear_scan_result(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    code_len: usize,
) -> Option<SelectedScanResult> {
    if opaque.scan_block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        mark_scan_exhausted(opaque);
        return None;
    }

    #[cfg(feature = "pg18")]
    {
        let max_block = opaque.scan_block_count.saturating_sub(1);
        opaque
            .linear_prefetch_state
            .reset(opaque.next_block_number, max_block);
        let stream = ensure_linear_read_stream(index_relation, opaque);
        unsafe { pg_sys::read_stream_reset(stream) };

        loop {
            let mut per_buffer_data = ptr::null_mut();
            let buffer = unsafe { pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data) };
            if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
                break;
            }

            let block_number = if per_buffer_data.is_null() {
                opaque.next_block_number
            } else {
                unsafe { *per_buffer_data.cast::<pg_sys::BlockNumber>() }
            };
            let selected = unsafe {
                select_linear_scan_result_from_buffer(opaque, code_len, buffer, block_number)
            };
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            if selected.is_some() {
                return selected;
            }
        }
    }

    #[cfg(not(feature = "pg18"))]
    {
        let max_block = opaque.scan_block_count.saturating_sub(1);
        opaque
            .linear_prefetch_state
            .reset(opaque.next_block_number, max_block);
        while let Some(block_number) = opaque.linear_prefetch_state.next_block() {
            let buffer = unsafe {
                pg_sys::ReadBufferExtended(
                    index_relation,
                    pg_sys::ForkNumber::MAIN_FORKNUM,
                    block_number,
                    pg_sys::ReadBufferMode::RBM_NORMAL,
                    ptr::null_mut(),
                )
            };
            let selected = unsafe {
                select_linear_scan_result_from_buffer(opaque, code_len, buffer, block_number)
            };
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            if selected.is_some() {
                return selected;
            }
        }
    }

    mark_scan_exhausted(opaque);
    None
}

unsafe fn select_linear_scan_result_from_buffer(
    opaque: &mut TqScanOpaque,
    code_len: usize,
    buffer: pg_sys::Buffer,
    block_number: u32,
) -> Option<SelectedScanResult> {
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    opaque.explain_counters.record_linear_page_read();
    super::stats::record_linear_page();
    opaque.stats_delta.record_linear_page();
    let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let line_pointer_count = super::shared::page_line_pointer_count(page_ptr);
    let offset_start = if block_number == opaque.next_block_number {
        opaque.next_offset_number.max(1)
    } else {
        1
    };

    for offset in offset_start..=line_pointer_count {
        let item_id = unsafe { &*super::shared::page_item_id(page_ptr, offset) };
        if item_id.lp_flags() == 0 {
            opaque.explain_counters.record_element_skipped();
            continue;
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("ec_hnsw found invalid tuple bounds while scanning block {block_number}");
        }

        let tuple_bytes =
            unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
        if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
            opaque.explain_counters.record_element_skipped();
            continue;
        }

        let element = page::TqElementTuple::decode(tuple_bytes, code_len)
            .unwrap_or_else(|e| pgrx::error!("ec_hnsw failed to decode scan element tuple: {e}"));
        if element.deleted || element.heaptids.is_empty() {
            opaque.explain_counters.record_element_skipped();
            continue;
        }

        opaque.next_block_number = block_number;
        debug_assert!(
            offset < u16::MAX,
            "scan offset should fit in page-local u16 range"
        );
        opaque.next_offset_number = offset + 1;
        let element_tid = page::ItemPointer {
            block_number,
            offset_number: offset,
        };
        if staged_or_emitted_contains_element(opaque, element_tid) {
            opaque.explain_counters.record_element_skipped();
            continue;
        }
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_UNLOCK as i32) };
        opaque.explain_counters.record_element_scored();
        let score = score_scan_element_result(opaque, element.gamma, &element.code);
        return Some(SelectedScanResult {
            element_tid,
            score,
            approx_score: None,
            approx_rank_base: None,
            comparison_score: None,
            heap_tids: CachedHeapTids::from_iter(element.heaptids.iter().copied()),
        });
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_UNLOCK as i32) };
    opaque.next_block_number = block_number + 1;
    opaque.next_offset_number = 1;
    None
}

#[cfg(test)]
fn collect_successor_candidates<F>(
    neighbor_tids: &[page::ItemPointer],
    max_candidates: usize,
    mut candidate_for_tid: F,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    F: FnMut(page::ItemPointer) -> Option<search::BeamCandidate<page::ItemPointer>>,
{
    let mut candidates = Vec::new();
    if max_candidates == 0 {
        return candidates;
    }

    for neighbor_tid in neighbor_tids.iter().copied() {
        if neighbor_tid == page::ItemPointer::INVALID {
            continue;
        }

        if let Some(candidate) = candidate_for_tid(neighbor_tid) {
            candidates.push(candidate);
            if candidates.len() >= max_candidates {
                break;
            }
        }
    }

    candidates
}

unsafe fn score_scan_element_result(
    opaque: &mut TqScanOpaque,
    gamma: f32,
    code_bytes: &[u8],
) -> f32 {
    if opaque.cached_quantizer.is_null() {
        pgrx::error!("ec_hnsw scan scoring requires a cached quantizer");
    }

    super::stats::record_distance_calc();
    opaque.stats_delta.record_distance_calc();
    let quantizer = unsafe { &*opaque.cached_quantizer };
    match opaque.turboquant_exact_score_mode {
        TurboQuantExactScoreMode::Exact => {}
        TurboQuantExactScoreMode::FullLut => {
            if opaque.turboquant_lut_query.is_null() {
                pgrx::error!(
                    "ec_hnsw TurboQuant full_lut exact-score mode requires a prepared LUT query"
                );
            }
            let prepared = unsafe { &*opaque.turboquant_lut_query };
            return -quantizer.score_ip_from_parts_lut_no_qjl_4bit(prepared, code_bytes);
        }
        TurboQuantExactScoreMode::TiledLut => {
            if opaque.turboquant_tiled_lut_query.is_null() {
                pgrx::error!(
                    "ec_hnsw TurboQuant tiled_lut exact-score mode requires a prepared tiled LUT query"
                );
            }
            let prepared = unsafe { &*opaque.turboquant_tiled_lut_query };
            return -quantizer.score_ip_from_parts_tiled_lut_no_qjl_4bit(prepared, code_bytes);
        }
        TurboQuantExactScoreMode::Int8Approx => {
            if opaque.turboquant_int8_query.is_null() {
                pgrx::error!(
                    "ec_hnsw TurboQuant int8 exact-score mode requires a prepared int8 query"
                );
            }
            let prepared = unsafe { &*opaque.turboquant_int8_query };
            return -quantizer.score_ip_from_parts_int8_approx_no_qjl_4bit(prepared, code_bytes);
        }
    }
    if opaque.prepared_query.is_null() {
        pgrx::error!("ec_hnsw scan scoring requires a prepared query");
    }
    let prepared_query = unsafe { &*opaque.prepared_query };
    -quantizer.score_ip_from_parts(prepared_query, gamma, code_bytes)
}

fn set_scan_heap_tid(scan: pg_sys::IndexScanDesc, heap_tid: page::ItemPointer) {
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut (*scan).xs_heaptid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
    }
}

fn clear_last_emitted_scan_scores(opaque: &mut TqScanOpaque) {
    opaque.last_emitted_approx_score = 0.0;
    opaque.last_emitted_approx_score_valid = false;
    opaque.last_emitted_approx_rank = 0;
    opaque.last_emitted_approx_rank_valid = false;
    opaque.last_emitted_comparison_score = 0.0;
    opaque.last_emitted_comparison_score_valid = false;
}

fn emit_scan_output(
    scan: pg_sys::IndexScanDesc,
    opaque: &mut TqScanOpaque,
    output: PendingScanOutput,
) {
    set_scan_heap_tid(scan, output.heap_tid);
    set_scan_orderby_score(scan, output.score);
    match output.approx_score {
        Some(score) => {
            opaque.last_emitted_approx_score = score;
            opaque.last_emitted_approx_score_valid = true;
        }
        None => {
            opaque.last_emitted_approx_score = 0.0;
            opaque.last_emitted_approx_score_valid = false;
        }
    }
    match output.approx_rank {
        Some(rank) => {
            opaque.last_emitted_approx_rank = rank;
            opaque.last_emitted_approx_rank_valid = true;
        }
        None => {
            opaque.last_emitted_approx_rank = 0;
            opaque.last_emitted_approx_rank_valid = false;
        }
    }
    match output.comparison_score {
        Some(score) => {
            opaque.last_emitted_comparison_score = score;
            opaque.last_emitted_comparison_score_valid = true;
        }
        None => {
            opaque.last_emitted_comparison_score = 0.0;
            opaque.last_emitted_comparison_score_valid = false;
        }
    }
}

fn set_scan_orderby_score(scan: pg_sys::IndexScanDesc, score: f32) {
    unsafe {
        if (*scan).xs_orderbyvals.is_null() {
            (*scan).xs_orderbyvals =
                pg_sys::palloc0(std::mem::size_of::<pg_sys::Datum>()).cast::<pg_sys::Datum>();
        }
        if (*scan).xs_orderbynulls.is_null() {
            (*scan).xs_orderbynulls = pg_sys::palloc0(std::mem::size_of::<bool>()).cast::<bool>();
        }

        *(*scan).xs_orderbyvals = score.into_datum().expect("score should convert to datum");
        *(*scan).xs_orderbynulls = false;
    }
}

fn clear_scan_orderby_output(scan: pg_sys::IndexScanDesc) {
    unsafe {
        if !(*scan).xs_orderbynulls.is_null() {
            *(*scan).xs_orderbynulls = true;
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(super) struct CurrentScanResult {
    element_tid: page::ItemPointer,
    heap_tid: page::ItemPointer,
    score: f32,
    score_valid: bool,
    approx_score: f32,
    approx_score_valid: bool,
    approx_rank_base: i32,
    approx_rank_valid: bool,
    comparison_score: f32,
    comparison_score_valid: bool,
}

impl CurrentScanResult {
    pub(super) fn has_element(&self) -> bool {
        self.element_tid != page::ItemPointer::INVALID
    }

    pub(super) fn element_tid(&self) -> page::ItemPointer {
        self.element_tid
    }

    pub(super) fn heap_tid(&self) -> page::ItemPointer {
        self.heap_tid
    }

    pub(super) fn score(&self) -> f32 {
        self.score
    }

    pub(super) fn score_valid(&self) -> bool {
        self.score_valid
    }

    pub(super) fn approx_score(&self) -> f32 {
        self.approx_score
    }

    pub(super) fn approx_score_valid(&self) -> bool {
        self.approx_score_valid
    }

    pub(super) fn approx_rank_base(&self) -> i32 {
        self.approx_rank_base
    }

    pub(super) fn approx_rank_valid(&self) -> bool {
        self.approx_rank_valid
    }

    pub(super) fn comparison_score(&self) -> f32 {
        self.comparison_score
    }

    pub(super) fn comparison_score_valid(&self) -> bool {
        self.comparison_score_valid
    }
}

impl Default for CurrentScanResult {
    fn default() -> Self {
        Self {
            element_tid: page::ItemPointer::INVALID,
            heap_tid: page::ItemPointer::INVALID,
            score: 0.0,
            score_valid: false,
            approx_score: 0.0,
            approx_score_valid: false,
            approx_rank_base: 0,
            approx_rank_valid: false,
            comparison_score: 0.0,
            comparison_score_valid: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SelectedScanResult {
    element_tid: page::ItemPointer,
    score: f32,
    approx_score: Option<f32>,
    approx_rank_base: Option<i32>,
    comparison_score: Option<f32>,
    heap_tids: CachedHeapTids,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(super) struct ScanResultState {
    current: CurrentScanResult,
    pending_heaptids: [page::ItemPointer; page::HEAPTID_INLINE_CAPACITY],
    pending_heaptid_count: u8,
    pending_heaptid_index: u8,
}

impl ScanResultState {
    fn clear_pending(&mut self) {
        self.pending_heaptids.fill(page::ItemPointer::INVALID);
        self.pending_heaptid_count = 0;
        self.pending_heaptid_index = 0;
    }

    fn store_pending(&mut self, heaptids: &[page::ItemPointer]) {
        debug_assert!(heaptids.len() <= page::HEAPTID_INLINE_CAPACITY);

        self.clear_pending();
        self.pending_heaptid_count =
            u8::try_from(heaptids.len()).expect("heap tid count should fit in u8");

        for (index, tid) in heaptids.iter().copied().enumerate() {
            self.pending_heaptids[index] = tid;
        }
    }

    fn take_pending(&mut self) -> Option<page::ItemPointer> {
        if self.pending_heaptid_index >= self.pending_heaptid_count {
            return None;
        }

        let tid = self.pending_heaptids[self.pending_heaptid_index as usize];
        self.pending_heaptid_index += 1;
        if self.pending_heaptid_index >= self.pending_heaptid_count {
            self.clear_pending();
        }
        self.update_current_heap_tid(tid);
        Some(tid)
    }

    fn take_pending_output(&mut self) -> Option<PendingScanOutput> {
        let approx_rank = self
            .current
            .approx_rank_valid()
            .then(|| self.current.approx_rank_base() + i32::from(self.pending_heaptid_index));
        let heap_tid = self.take_pending()?;
        Some(PendingScanOutput {
            heap_tid,
            score: self.current.score(),
            approx_score: self
                .current
                .approx_score_valid()
                .then_some(self.current.approx_score()),
            approx_rank,
            comparison_score: self
                .current
                .comparison_score_valid()
                .then_some(self.current.comparison_score()),
        })
    }

    pub(super) fn clear(&mut self) {
        self.clear_pending();
        self.current = CurrentScanResult::default();
    }

    fn clear_current(&mut self) {
        self.current = CurrentScanResult::default();
    }

    fn materialize(&mut self, selected: SelectedScanResult) {
        self.materialize_with_details(
            selected.element_tid,
            selected.score,
            selected.approx_score,
            selected.approx_rank_base,
            selected.comparison_score,
            selected.heap_tids.as_slice(),
        );
    }

    fn materialize_from_parts(
        &mut self,
        element_tid: page::ItemPointer,
        score: f32,
        heaptids: &[page::ItemPointer],
    ) {
        self.materialize_with_details(element_tid, score, None, None, None, heaptids);
    }

    fn materialize_with_details(
        &mut self,
        element_tid: page::ItemPointer,
        score: f32,
        approx_score: Option<f32>,
        approx_rank_base: Option<i32>,
        comparison_score: Option<f32>,
        heaptids: &[page::ItemPointer],
    ) {
        self.set_current_with_details(
            element_tid,
            score,
            approx_score,
            approx_rank_base,
            comparison_score,
        );
        self.store_pending(heaptids);
    }

    fn set_current(&mut self, element_tid: page::ItemPointer, score: f32) {
        self.set_current_with_details(element_tid, score, None, None, None);
    }

    fn set_current_with_details(
        &mut self,
        element_tid: page::ItemPointer,
        score: f32,
        approx_score: Option<f32>,
        approx_rank_base: Option<i32>,
        comparison_score: Option<f32>,
    ) {
        self.current = CurrentScanResult {
            element_tid,
            heap_tid: page::ItemPointer::INVALID,
            score,
            score_valid: true,
            approx_score: approx_score.unwrap_or(0.0),
            approx_score_valid: approx_score.is_some(),
            approx_rank_base: approx_rank_base.unwrap_or(0),
            approx_rank_valid: approx_rank_base.is_some(),
            comparison_score: comparison_score.unwrap_or(0.0),
            comparison_score_valid: comparison_score.is_some(),
        };
    }

    fn set_current_comparison_score(&mut self, score: f32) {
        if self.current.element_tid == page::ItemPointer::INVALID {
            return;
        }
        self.current.comparison_score = score;
        self.current.comparison_score_valid = true;
    }

    fn update_current_heap_tid(&mut self, heap_tid: page::ItemPointer) {
        if self.current.element_tid != page::ItemPointer::INVALID {
            self.current.heap_tid = heap_tid;
        }
    }

    pub(super) fn current(&self) -> CurrentScanResult {
        self.current
    }

    pub(super) fn pending_count(&self) -> u8 {
        self.pending_heaptid_count
    }

    pub(super) fn pending_index(&self) -> u8 {
        self.pending_heaptid_index
    }

    pub(super) fn pending_heap_tids(&self) -> &[page::ItemPointer] {
        &self.pending_heaptids[..self.pending_heaptid_count as usize]
    }
}

impl Default for ScanResultState {
    fn default() -> Self {
        Self {
            current: CurrentScanResult::default(),
            pending_heaptids: [page::ItemPointer::INVALID; page::HEAPTID_INLINE_CAPACITY],
            pending_heaptid_count: 0,
            pending_heaptid_index: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum ScanExecutionPhase {
    #[default]
    GraphTraversal,
    LinearFallback,
    Exhausted,
}

impl ScanExecutionPhase {
    pub(super) fn is_graph_traversal(self) -> bool {
        matches!(self, Self::GraphTraversal)
    }

    pub(super) fn is_exhausted(self) -> bool {
        matches!(self, Self::Exhausted)
    }
}

#[repr(C)]
#[derive(Debug)]
pub(super) struct TqScanOpaque {
    pub(super) rescan_called: bool,
    parallel_scan_state: *mut super::parallel::EcParallelScanState,
    parallel_scan_rescan_epoch: u32,
    parallel_scan_worker_slot_count: u32,
    parallel_scan_worker_slot_index: u32,
    parallel_owned_output_blocker: Option<super::parallel::EcParallelOwnedOutputBlocker>,
    retained_parallel_owned_output_blocker: Option<RetainedParallelOwnedOutputBlocker>,
    parallel_local_only_output_active: bool,
    deferred_parallel_blocked_results: Vec<DeferredParallelBlockedOutput>,
    pub(super) query_dimensions: u16,
    pub(super) query_values: *mut f32,
    pub(super) prepared_query: *mut PreparedQuery,
    pub(super) grouped_query: *mut PreparedGroupedScanQuery,
    pub(super) binary_sign_query: *mut BinarySignNoQjl4BitQuery,
    pub(super) turboquant_lut_query: *mut PreparedLutNoQjl4BitQuery,
    pub(super) turboquant_tiled_lut_query: *mut PreparedTiledLutNoQjl4BitQuery,
    pub(super) turboquant_int8_query: *mut Int8ApproxNoQjl4BitQuery,
    turboquant_exact_score_mode: TurboQuantExactScoreMode,
    pub(super) cached_quantizer: *const ProdQuantizer,
    pub(super) scan_dimensions: u16,
    pub(super) scan_m: u16,
    pub(super) scan_bits: u8,
    pub(super) scan_seed: u64,
    pub(super) scan_code_len: usize,
    pub(super) scan_graph_storage: graph::GraphStorageDescriptor,
    pub(super) scan_block_count: u32,
    pub(super) bootstrap_frontier_limit: usize,
    pub(super) visited_tids: *mut HashSet<page::ItemPointer>,
    pub(super) expanded_source_tids: *mut HashSet<page::ItemPointer>,
    pub(super) emitted_result_tids: *mut HashSet<page::ItemPointer>,
    pub(super) graph_element_cache: *mut HashMap<page::ItemPointer, Arc<CachedGraphElement>>,
    pub(super) graph_neighbor_cache: *mut HashMap<page::ItemPointer, Arc<graph::GraphNeighbors>>,
    pub(super) score_cache: *mut HashMap<page::ItemPointer, f32>,
    pub(super) candidate_frontier: *mut VisibleCandidateFrontierState,
    pub(super) bootstrap_expansion: *mut search::BeamSearch<page::ItemPointer>,
    pub(super) result_state: ScanResultState,
    pub(super) fallback_result_state: ScanResultState,
    // This remains the authoritative cross-call cursor until PG18 ReadStream
    // flips cursor ownership fully into `linear_prefetch_state`.
    pub(super) next_block_number: u32,
    pub(super) next_offset_number: u16,
    pub(super) execution_phase: ScanExecutionPhase,
    grouped_live_rerank_buffer: [BufferedGroupedScanResult; PQ_FASTSCAN_MAX_LIVE_RERANK_WINDOW],
    grouped_live_rerank_buffer_len: u8,
    grouped_live_rerank_window: u8,
    grouped_traversal_score_mode: GroupedTraversalScoreMode,
    grouped_rerank_mode: GroupedRerankMode,
    grouped_heap_rerank_relation: pg_sys::Relation,
    grouped_heap_rerank_relation_owned: bool,
    grouped_heap_rerank_snapshot: pg_sys::Snapshot,
    grouped_heap_rerank_snapshot_owned: bool,
    grouped_heap_rerank_slot: *mut pg_sys::TupleTableSlot,
    grouped_heap_rerank_source_attnum: i16,
    grouped_heap_rerank_source_kind: source::SourceDatumKind,
    grouped_exact_traversal_mode: GroupedExactTraversalMode,
    grouped_exact_traversal_strategy: GroupedExactTraversalStrategy,
    grouped_exact_traversal_limit: u8,
    grouped_live_rerank_next_approx_rank: i32,
    pub(super) last_emitted_approx_score: f32,
    pub(super) last_emitted_approx_score_valid: bool,
    pub(super) last_emitted_approx_rank: i32,
    pub(super) last_emitted_approx_rank_valid: bool,
    pub(super) last_emitted_comparison_score: f32,
    pub(super) last_emitted_comparison_score_valid: bool,
    #[cfg(feature = "pg18")]
    graph_read_stream: *mut pg_sys::ReadStream,
    #[cfg(feature = "pg18")]
    linear_read_stream: *mut pg_sys::ReadStream,
    pub(super) graph_prefetch_state: *mut GraphPrefetchState,
    pub(super) linear_prefetch_state: LinearPrefetchState,
    pub(super) explain_counters: TqExplainCounters,
    pub(super) stats_delta: TqStatsCounters,
    stats_used_linear_fallback: bool,
    stats_scan_finalized: bool,
    #[cfg(any(test, feature = "pg_test"))]
    pub(super) debug_profile: ScanDebugProfile,
}

impl Default for TqScanOpaque {
    fn default() -> Self {
        Self {
            rescan_called: false,
            parallel_scan_state: ptr::null_mut(),
            parallel_scan_rescan_epoch: 0,
            parallel_scan_worker_slot_count: 0,
            parallel_scan_worker_slot_index: INVALID_PARALLEL_SCAN_WORKER_SLOT,
            parallel_owned_output_blocker: None,
            retained_parallel_owned_output_blocker: None,
            parallel_local_only_output_active: false,
            deferred_parallel_blocked_results: Vec::new(),
            query_dimensions: 0,
            query_values: ptr::null_mut(),
            prepared_query: ptr::null_mut(),
            grouped_query: ptr::null_mut(),
            binary_sign_query: ptr::null_mut(),
            turboquant_lut_query: ptr::null_mut(),
            turboquant_tiled_lut_query: ptr::null_mut(),
            turboquant_int8_query: ptr::null_mut(),
            turboquant_exact_score_mode: TurboQuantExactScoreMode::Exact,
            cached_quantizer: ptr::null(),
            scan_dimensions: 0,
            scan_m: 0,
            scan_bits: 0,
            scan_seed: 0,
            scan_code_len: 0,
            scan_graph_storage: graph::GraphStorageDescriptor::TurboQuant { code_len: 0 },
            scan_block_count: 0,
            bootstrap_frontier_limit: MAX_BOOTSTRAP_FRONTIER_CANDIDATES,
            visited_tids: ptr::null_mut(),
            expanded_source_tids: ptr::null_mut(),
            emitted_result_tids: ptr::null_mut(),
            graph_element_cache: ptr::null_mut(),
            graph_neighbor_cache: ptr::null_mut(),
            score_cache: ptr::null_mut(),
            candidate_frontier: ptr::null_mut(),
            bootstrap_expansion: ptr::null_mut(),
            result_state: ScanResultState::default(),
            fallback_result_state: ScanResultState::default(),
            next_block_number: page::FIRST_DATA_BLOCK_NUMBER,
            next_offset_number: 1,
            execution_phase: ScanExecutionPhase::GraphTraversal,
            grouped_live_rerank_buffer: [BufferedGroupedScanResult::default();
                PQ_FASTSCAN_MAX_LIVE_RERANK_WINDOW],
            grouped_live_rerank_buffer_len: 0,
            grouped_live_rerank_window: PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW as u8,
            grouped_traversal_score_mode: GroupedTraversalScoreMode::Binary,
            grouped_rerank_mode: GroupedRerankMode::Quantized,
            grouped_heap_rerank_relation: ptr::null_mut(),
            grouped_heap_rerank_relation_owned: false,
            grouped_heap_rerank_snapshot: ptr::null_mut(),
            grouped_heap_rerank_snapshot_owned: false,
            grouped_heap_rerank_slot: ptr::null_mut(),
            grouped_heap_rerank_source_attnum: 0,
            grouped_heap_rerank_source_kind: source::SourceDatumKind::Unknown,
            grouped_exact_traversal_mode: GroupedExactTraversalMode::Disabled,
            grouped_exact_traversal_strategy: GroupedExactTraversalStrategy::Expansion,
            grouped_exact_traversal_limit: 0,
            grouped_live_rerank_next_approx_rank: 1,
            last_emitted_approx_score: 0.0,
            last_emitted_approx_score_valid: false,
            last_emitted_approx_rank: 0,
            last_emitted_approx_rank_valid: false,
            last_emitted_comparison_score: 0.0,
            last_emitted_comparison_score_valid: false,
            #[cfg(feature = "pg18")]
            graph_read_stream: ptr::null_mut(),
            #[cfg(feature = "pg18")]
            linear_read_stream: ptr::null_mut(),
            graph_prefetch_state: ptr::null_mut(),
            linear_prefetch_state: LinearPrefetchState::new(
                page::FIRST_DATA_BLOCK_NUMBER,
                page::FIRST_DATA_BLOCK_NUMBER,
            ),
            explain_counters: TqExplainCounters::default(),
            stats_delta: TqStatsCounters::default(),
            stats_used_linear_fallback: false,
            stats_scan_finalized: false,
            #[cfg(any(test, feature = "pg_test"))]
            debug_profile: ScanDebugProfile::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tid(block_number: u32, offset_number: u16) -> page::ItemPointer {
        page::ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn beam_candidate(
        block_number: u32,
        offset_number: u16,
        score: f32,
    ) -> search::BeamCandidate<page::ItemPointer> {
        search::BeamCandidate::new(tid(block_number, offset_number), score)
    }

    fn sourced_beam_candidate(
        block_number: u32,
        offset_number: u16,
        source_tid: page::ItemPointer,
        score: f32,
    ) -> search::BeamCandidate<page::ItemPointer> {
        search::BeamCandidate::with_source(tid(block_number, offset_number), score, source_tid)
    }

    #[test]
    fn select_next_bootstrap_candidate_skips_unselectable_candidates() {
        let mut queued = vec![beam_candidate(21, 1, -3.0), beam_candidate(21, 2, -2.0)].into_iter();
        let mut attempted = Vec::new();

        let selected = select_next_bootstrap_candidate(
            || queued.next(),
            |candidate| {
                attempted.push((candidate.node.block_number, candidate.node.offset_number));
                (candidate.node.offset_number == 2).then(|| SelectedScanResult {
                    element_tid: candidate.node,
                    score: candidate.score,
                    approx_score: None,
                    approx_rank_base: None,
                    comparison_score: None,
                    heap_tids: CachedHeapTids::from_iter([tid(41, 1)]),
                })
            },
        );

        assert!(
            selected.is_some(),
            "bootstrap selection should keep trying later candidates after one fails"
        );
        assert_eq!(
            attempted,
            vec![(21, 1), (21, 2)],
            "candidate selection should proceed in consumption order until one succeeds"
        );
    }

    #[test]
    fn select_next_bootstrap_candidate_returns_none_when_frontier_never_selects() {
        let mut queued = vec![beam_candidate(22, 1, -3.0), beam_candidate(22, 2, -2.0)].into_iter();
        let mut attempts = 0;

        let selected = select_next_bootstrap_candidate(
            || queued.next(),
            |_candidate| {
                attempts += 1;
                None
            },
        );

        assert!(
            selected.is_none(),
            "bootstrap selection should return none only after every candidate fails"
        );
        assert_eq!(
            attempts, 2,
            "bootstrap selection should exhaust the queued frontier before giving up"
        );
    }

    #[test]
    fn select_next_bootstrap_candidate_refills_only_after_successful_adjudication() {
        let candidate_a = beam_candidate(23, 1, -3.0);
        let candidate_b = beam_candidate(23, 2, -2.0);
        let mut queued = vec![candidate_a, candidate_b].into_iter();
        let mut attempted = Vec::new();
        let mut refilled_after = Vec::new();

        let selected = select_next_bootstrap_candidate_with_refill(
            || queued.next(),
            |candidate| {
                attempted.push(candidate.node);
                (candidate == candidate_b).then(|| SelectedScanResult {
                    element_tid: candidate.node,
                    score: candidate.score,
                    approx_score: None,
                    approx_rank_base: None,
                    comparison_score: None,
                    heap_tids: CachedHeapTids::from_iter([tid(42, 1)]),
                })
            },
            |candidate| refilled_after.push(candidate.node),
        );

        assert!(
            selected.is_some(),
            "bootstrap selection should still succeed once a later visible candidate selects"
        );
        assert_eq!(
            attempted,
            vec![candidate_a.node, candidate_b.node],
            "bootstrap candidates should be adjudicated in consume order before any refill path runs"
        );
        assert_eq!(
            refilled_after,
            vec![candidate_b.node],
            "bootstrap refill should only run for the candidate that actually materialized"
        );
    }

    #[test]
    fn bind_parallel_scan_state_captures_shared_rescan_epoch() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 1024],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 1024] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");
        assert_eq!(
            unsafe { crate::am::ec_hnsw::parallel::reset_parallel_scan_state(parallel_scan) }
                .expect("parallel scan reset should succeed")
                .expect("parallel scan reset should see initialized state"),
            1,
            "shared rescan epoch should advance before scan-side attachment"
        );

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque::default();

        bind_parallel_scan_state(&mut scan_desc, &mut opaque);

        assert!(
            !opaque.parallel_scan_state.is_null(),
            "scan state should retain the shared AM-private descriptor when parallel scan is present"
        );
        assert_eq!(
            opaque.parallel_scan_rescan_epoch, 1,
            "scan state should capture the current shared rescan epoch"
        );
        assert_eq!(
            opaque.parallel_scan_worker_slot_count, 2,
            "scan state should capture the shared worker slot capacity too"
        );
        assert_eq!(
            opaque.parallel_scan_worker_slot_index, 0,
            "first scan attachment should claim the first shared worker slot"
        );
    }

    #[test]
    fn clear_parallel_scan_state_releases_claimed_worker_slot() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque::default();

        bind_parallel_scan_state(&mut scan_desc, &mut opaque);
        let attachment =
            unsafe { crate::am::ec_hnsw::parallel::parallel_scan_attachment(parallel_scan) }
                .expect("parallel scan attachment should validate")
                .expect("parallel scan attachment should expose AM-private state");
        assert_eq!(
            unsafe { &*attachment.coordinator }
                .claimed_worker_slots
                .load(std::sync::atomic::Ordering::Acquire),
            1,
            "scan attachment should publish its worker-slot claim to the shared coordinator state"
        );

        clear_parallel_scan_state(&mut opaque);

        let attachment =
            unsafe { crate::am::ec_hnsw::parallel::parallel_scan_attachment(parallel_scan) }
                .expect("parallel scan attachment should keep validating")
                .expect("parallel scan attachment should keep exposing AM-private state");
        assert_eq!(
            unsafe { &*attachment.coordinator }
                .claimed_worker_slots
                .load(std::sync::atomic::Ordering::Acquire),
            0,
            "clearing scan state should release the previously claimed worker slot"
        );
        assert_eq!(
            opaque.parallel_scan_worker_slot_index, INVALID_PARALLEL_SCAN_WORKER_SLOT,
            "clearing scan state should drop the local worker-slot binding"
        );
    }

    #[test]
    fn publish_parallel_scan_worker_slot_snapshot_mirrors_scan_runtime_state() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque::default();

        bind_parallel_scan_state(&mut scan_desc, &mut opaque);
        opaque.scan_dimensions = 1536;
        opaque.bootstrap_frontier_limit = 64;
        visible_frontier_mut(&mut opaque).push(beam_candidate(24, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(24, 2, -2.0));
        reset_bootstrap_expansion_state(&mut opaque, 64);
        bootstrap_expansion_mut(&mut opaque).seed(beam_candidate(25, 1, -4.0));
        bootstrap_expansion_mut(&mut opaque).seed(beam_candidate(25, 2, -5.0));
        reset_scan_visited_state(&mut opaque);
        mark_visited_element(&mut opaque, tid(26, 1));
        mark_visited_element(&mut opaque, tid(26, 2));
        reset_scan_emitted_state(&mut opaque);
        mark_emitted_element(&mut opaque, tid(27, 1));
        opaque.result_state.set_current(tid(28, 1), -6.0);
        opaque.result_state.store_pending(&[tid(29, 1), tid(29, 2)]);

        publish_parallel_scan_worker_slot_snapshot(&opaque);

        let snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_worker_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel worker slot snapshot should read back");
        assert_eq!(
            snapshot.flags, 1,
            "claimed scan slots should stay marked as live in the shared worker snapshot"
        );
        assert_eq!(
            snapshot.observed_rescan_epoch, 0,
            "worker snapshot should stay keyed to the active shared epoch"
        );
        assert_eq!(
            snapshot.runtime.execution_phase,
            crate::am::ec_hnsw::parallel::EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL,
            "worker snapshot should report the current graph-traversal phase"
        );
        assert_eq!(
            snapshot.runtime.scan_dimensions, 1536,
            "worker snapshot should mirror the bound scan dimensions"
        );
        assert_eq!(
            snapshot.runtime.bootstrap_frontier_limit, 64,
            "worker snapshot should mirror the staged bootstrap frontier limit"
        );
        assert_eq!(
            snapshot.runtime.visible_frontier_len, 2,
            "worker snapshot should report the current visible frontier size"
        );
        assert_eq!(
            snapshot.runtime.scheduler_frontier_len, 2,
            "worker snapshot should report the scheduler frontier size"
        );
        assert_eq!(
            snapshot.runtime.visited_count, 2,
            "worker snapshot should report the current visited-element count"
        );
        assert_eq!(
            snapshot.runtime.emitted_count, 1,
            "worker snapshot should report the current emitted-result count"
        );
        assert_eq!(
            snapshot.runtime.active_result_pending_count, 2,
            "worker snapshot should report the pending heap-drain count"
        );
        assert!(
            snapshot.runtime.active_result_has_current,
            "worker snapshot should record whether the active result state still has a current row"
        );
        assert_eq!(
            snapshot.runtime.owned_output_blocker_kind,
            crate::am::ec_hnsw::parallel::EC_PARALLEL_OWNED_OUTPUT_BLOCKER_NONE,
            "idle worker snapshots should not report an ownership blocker when no blocked state has been observed"
        );
        assert_eq!(
            snapshot.runtime.owned_output_blocker_slot_index, None,
            "idle worker snapshots should not report a blocker owner slot"
        );
        assert_eq!(
            snapshot.runtime.owned_output_blocker_generation, 0,
            "idle worker snapshots should not report a blocker generation"
        );

        let coordinator_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_snapshot(
                opaque.parallel_scan_state,
            )
        }
        .expect("parallel coordinator snapshot should read back");
        assert_eq!(
            coordinator_snapshot.claimed_worker_slots, 1,
            "coordinator snapshot should report the single claimed worker slot"
        );
        assert_eq!(
            coordinator_snapshot.published_result_slots, 1,
            "coordinator snapshot should report the single staged current result"
        );

        let result_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel coordinator result-slot snapshot should read back");
        assert_eq!(
            result_snapshot.flags, 0x23,
            "plain staged current results should carry the published, score-valid, and staged-heap-tid flags"
        );
        assert_eq!(
            result_snapshot.runtime.element_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 28,
                offset_number: 1,
            },
            "coordinator result snapshot should mirror the active result element tid"
        );
        assert_eq!(
            result_snapshot.runtime.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 29,
                offset_number: 1,
            },
            "coordinator result snapshot should stage the next heap tid ready for coordinator-side drain"
        );
        assert_eq!(
            result_snapshot.runtime.score, -6.0,
            "coordinator result snapshot should mirror the active result score"
        );
        assert_eq!(
            result_snapshot.runtime.approx_score, None,
            "plain active results should not synthesize an approximate score"
        );
        assert_eq!(
            result_snapshot.runtime.comparison_score, None,
            "plain active results should not synthesize a comparison score"
        );
        assert_eq!(
            result_snapshot.runtime.approx_rank_base, None,
            "plain active results should not synthesize an approximate rank"
        );
        assert_eq!(
            result_snapshot.runtime.pending_count, 2,
            "coordinator result snapshot should mirror the pending heap-drain count"
        );
        assert_eq!(
            result_snapshot.runtime.pending_index, 0,
            "coordinator result snapshot should mirror the pending heap-drain cursor"
        );
        assert_eq!(
            result_snapshot.runtime.pending_heap_tids[0],
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 29,
                offset_number: 1,
            },
            "coordinator result snapshot should stage the first pending heap tid inline"
        );
        assert_eq!(
            result_snapshot.runtime.pending_heap_tids[1],
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 29,
                offset_number: 2,
            },
            "coordinator result snapshot should stage the second pending heap tid inline"
        );
    }

    #[test]
    fn publish_parallel_scan_worker_slot_snapshot_hides_local_only_output_from_coordinator() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque::default();

        bind_parallel_scan_state(&mut scan_desc, &mut opaque);
        opaque.result_state.set_current(tid(30, 1), -7.0);
        opaque.result_state.store_pending(&[tid(31, 1), tid(31, 2)]);
        opaque.retained_parallel_owned_output_blocker = Some(RetainedParallelOwnedOutputBlocker {
            blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                slot_index: Some(1),
                generation: 9,
                element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
            },
            element_tid: tid(30, 1),
        });
        opaque.parallel_local_only_output_active = true;

        publish_parallel_scan_worker_slot_snapshot(&opaque);

        let worker_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_worker_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel worker slot snapshot should read back");
        assert!(
            worker_snapshot.runtime.active_result_has_current,
            "local-only fallback should stay visible in the worker snapshot"
        );
        assert_eq!(
            worker_snapshot.runtime.active_result_pending_count, 2,
            "local-only fallback should preserve pending duplicate count in the worker snapshot"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_kind,
            crate::am::ec_hnsw::parallel::EC_PARALLEL_OWNED_OUTPUT_BLOCKER_FOREIGN_SELECTED_PENDING,
            "local-only fallback should keep the retained foreign blocker visible in the worker snapshot"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_slot_index,
            Some(1),
            "local-only fallback should keep the retained blocker owner slot visible in the worker snapshot"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_generation, 9,
            "local-only fallback should keep the retained blocker generation visible in the worker snapshot"
        );

        let coordinator_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_snapshot(
                opaque.parallel_scan_state,
            )
        }
        .expect("parallel coordinator snapshot should read back");
        assert_eq!(
            coordinator_snapshot.published_result_slots, 0,
            "local-only fallback should clear the worker's coordinator slot from the shared published set"
        );

        let result_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel coordinator result-slot snapshot should read back");
        assert_eq!(
            result_snapshot.flags, 0,
            "local-only fallback should leave the coordinator result slot cleared"
        );
        assert_eq!(
            result_snapshot.runtime.element_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
            "cleared local-only coordinator slots should not expose a staged element tid"
        );
    }

    #[test]
    fn publish_parallel_scan_worker_slot_snapshot_uses_best_deferred_blocker() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque::default();

        bind_parallel_scan_state(&mut scan_desc, &mut opaque);

        let mut slower = ScanResultState::default();
        slower.set_current(tid(70, 1), -4.0);
        slower.store_pending(&[tid(71, 1)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: slower,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                        slot_index: Some(2),
                        generation: 20,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
                    },
                    element_tid: tid(70, 1),
                }),
            });

        let mut best = ScanResultState::default();
        best.set_current(tid(72, 1), -8.0);
        best.store_pending(&[tid(73, 1)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: best,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead,
                        slot_index: Some(1),
                        generation: 11,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
                    },
                    element_tid: tid(72, 1),
                }),
            });

        publish_parallel_scan_worker_slot_snapshot(&opaque);

        let worker_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_worker_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel worker slot snapshot should read back");
        assert_eq!(
            worker_snapshot.runtime.execution_phase,
            crate::am::ec_hnsw::parallel::EC_PARALLEL_WORKER_PHASE_GRAPH_TRAVERSAL,
            "worker snapshot should publish the source phase for the best deferred blocked row"
        );
        assert!(
            worker_snapshot.runtime.active_result_has_current,
            "worker snapshot should expose that a best deferred blocked row is still staged locally"
        );
        assert_eq!(
            worker_snapshot.runtime.active_result_pending_count, 1,
            "worker snapshot should publish the pending duplicate count from the best deferred blocked row"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_kind,
            crate::am::ec_hnsw::parallel::EC_PARALLEL_OWNED_OUTPUT_BLOCKER_FOREIGN_ADMITTED_HEAD,
            "worker snapshot should publish the blocker from the best deferred blocked row"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_slot_index,
            Some(1),
            "worker snapshot should publish the blocker owner slot from the best deferred blocked row"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_generation, 11,
            "worker snapshot should publish the blocker generation from the best deferred blocked row"
        );
    }

    #[test]
    fn enter_linear_fallback_phase_clears_frontier_scheduler_and_expanded_state() {
        let mut opaque = TqScanOpaque::default();
        visible_frontier_mut(&mut opaque).push(beam_candidate(24, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(24, 2, -2.0));
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        reset_scan_expanded_state(&mut opaque);
        seed_existing_frontier_into_expansion(&mut opaque);
        mark_expanded_source(&mut opaque, tid(24, 1));

        enter_linear_fallback_phase(&mut opaque);

        assert!(
            opaque.execution_phase == ScanExecutionPhase::LinearFallback,
            "entering linear fallback should transition the scan into its explicit fallback phase"
        );
        assert!(
            visible_frontier_candidates(&opaque).is_empty(),
            "entering linear fallback should clear any leftover visible frontier candidates"
        );
        assert!(
            bootstrap_expansion_mut(&mut opaque).peek_best().is_none(),
            "entering linear fallback should clear the scan-owned scheduler too"
        );
        assert!(
            !expanded_contains_source(&opaque, tid(24, 1)),
            "entering linear fallback should reset expanded-source bookkeeping for the next rescan"
        );
    }

    #[test]
    fn mark_scan_exhausted_clears_result_state() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(25, 1), -3.0);
        opaque.result_state.store_pending(&[tid(30, 1), tid(30, 2)]);

        mark_scan_exhausted(&mut opaque);

        assert!(
            opaque.execution_phase == ScanExecutionPhase::Exhausted,
            "exhausting the scan should transition into the explicit exhausted phase"
        );
        assert!(
            !opaque.result_state.current().has_element(),
            "exhausting the scan should clear the current result slot"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "exhausting the scan should also clear pending duplicate-drain state"
        );
    }

    #[test]
    fn reset_scan_position_restores_bootstrap_execution_phase() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };

        reset_scan_position(&mut opaque);

        assert!(
            opaque.execution_phase == ScanExecutionPhase::GraphTraversal,
            "amrescan/reset should allow graph traversal to run again after prior fallback-phase scans"
        );
        assert!(
            candidate_frontier_head(&mut opaque).is_none(),
            "amrescan/reset should clear prior graph traversal frontier state before rebuilding it"
        );
    }

    #[test]
    fn reset_scan_position_clears_scan_local_caches() {
        let mut opaque = TqScanOpaque::default();
        graph_element_cache_mut(&mut opaque).insert(
            tid(91, 1),
            Arc::new(CachedGraphElement {
                tid: tid(91, 1),
                level: 1,
                deleted: false,
                heaptids: CachedHeapTids::from_iter([tid(191, 1)]),
                neighbortid: tid(91, 2),
                reranktid: None,
                binary_words: CachedBinaryWords::from_iter(1, [0_u64]),
                grouped_search_code: CachedGroupedSearchCode::None,
            }),
        );
        graph_neighbor_cache_mut(&mut opaque).insert(
            tid(91, 2),
            Arc::new(graph::GraphNeighbors {
                tid: tid(91, 2),
                count: 1,
                tids: vec![tid(92, 1)],
            }),
        );
        score_cache_mut(&mut opaque).insert(tid(91, 1), -7.5);

        reset_scan_position(&mut opaque);

        assert!(
            unsafe { &*opaque.graph_element_cache }.is_empty(),
            "amrescan/reset should drop cached graph elements before reseeding the ordered scan"
        );
        assert!(
            unsafe { &*opaque.graph_neighbor_cache }.is_empty(),
            "amrescan/reset should drop cached graph neighbors before reseeding the ordered scan"
        );
        assert!(
            unsafe { &*opaque.score_cache }.is_empty(),
            "amrescan/reset should drop cached element scores before reseeding the ordered scan"
        );

        free_scan_graph_cache(&mut opaque);
        free_scan_score_cache(&mut opaque);
    }

    #[test]
    fn cached_heap_tids_use_inline_storage() {
        let cached = CachedHeapTids::from_iter([tid(41, 1), tid(41, 2)]);

        assert_eq!(
            cached.as_slice(),
            &[tid(41, 1), tid(41, 2)],
            "cached heap tids should preserve heap tids in inline scan-local storage"
        );
        assert!(
            !cached.is_empty(),
            "inline cached heap tids should report non-empty when tids are present"
        );
    }

    #[test]
    fn cached_binary_words_inline_target_adr031_width() {
        let words: Vec<u64> = (0..ADR031_INLINE_BINARY_WORD_CAPACITY as u64).collect();
        let cached = CachedBinaryWords::from_iter(words.len(), words.iter().copied());

        assert!(
            matches!(cached, CachedBinaryWords::Inline { .. }),
            "ADR-031 target binary width should stay in inline scan-local storage"
        );
        assert_eq!(
            cached.as_slice(),
            words.as_slice(),
            "inline cached binary words should preserve the persisted sidecar payload"
        );
    }

    #[test]
    fn cached_binary_words_fallback_for_wider_code_paths() {
        let words: Vec<u64> = (0..=ADR031_INLINE_BINARY_WORD_CAPACITY as u64).collect();
        let cached = CachedBinaryWords::from_iter(words.len(), words.iter().copied());

        assert!(
            matches!(cached, CachedBinaryWords::Heap(_)),
            "wider binary code paths should fall back to heap-backed storage instead of truncating inline words"
        );
        assert_eq!(
            cached.as_slice(),
            words.as_slice(),
            "fallback binary-word storage should preserve every word when inline capacity is exceeded"
        );
    }

    #[test]
    fn binary_prefilter_survivor_budget_only_filters_full_source_widths() {
        assert_eq!(binary_prefilter_survivor_budget(0), 0);
        assert_eq!(binary_prefilter_survivor_budget(8), 8);
        assert_eq!(binary_prefilter_survivor_budget(15), 15);
        assert_eq!(binary_prefilter_survivor_budget(16), 12);
        assert_eq!(binary_prefilter_survivor_budget(32), 28);
    }

    #[test]
    fn unseeded_scans_enter_linear_fallback_explicitly() {
        let mut opaque = TqScanOpaque::default();

        enter_linear_fallback_phase(&mut opaque);

        assert_eq!(
            opaque.execution_phase,
            ScanExecutionPhase::LinearFallback,
            "unseeded scans should enter the explicit linear fallback phase"
        );
    }

    #[test]
    fn scan_result_state_take_pending_advances_current_result_progress() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(25, 1), -3.0);
        opaque.result_state.store_pending(&[tid(30, 1), tid(30, 2)]);

        let first = opaque.result_state.take_pending();
        let second = opaque.result_state.take_pending();
        let exhausted = opaque.result_state.take_pending();

        assert_eq!(
            first,
            Some(tid(30, 1)),
            "pending result drain should return the first queued heap tid first"
        );
        assert_eq!(
            second,
            Some(tid(30, 2)),
            "pending result drain should continue through later heap tids in order"
        );
        assert_eq!(
            exhausted, None,
            "pending result drain should stop once the queued heap tids are exhausted"
        );
        assert_eq!(
            opaque.result_state.current().heap_tid(),
            tid(30, 2),
            "draining pending heap tids should keep the current-result heap tid aligned with the last emitted duplicate"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "draining all queued heap tids should reset the pending count"
        );
        assert_eq!(
            opaque.result_state.pending_index(),
            0,
            "draining all queued heap tids should reset the pending index too"
        );
    }

    #[test]
    fn scan_result_state_take_pending_output_preserves_score_and_heap_progress() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(26, 1), -4.0);
        opaque.result_state.store_pending(&[tid(31, 1), tid(31, 2)]);

        let first = opaque.result_state.take_pending_output();
        let second = opaque.result_state.take_pending_output();
        let exhausted = opaque.result_state.take_pending_output();

        assert_eq!(
            first,
            Some(PendingScanOutput {
                heap_tid: tid(31, 1),
                score: -4.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: None,
            }),
            "pending output should expose the first heap tid together with the current result score"
        );
        assert_eq!(
            second,
            Some(PendingScanOutput {
                heap_tid: tid(31, 2),
                score: -4.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: None,
            }),
            "pending output should preserve score while draining later heap tids from the same result"
        );
        assert_eq!(
            exhausted, None,
            "pending output should report exhaustion once the duplicate drain is complete"
        );
    }

    #[test]
    fn linear_fallback_cursor_advance_after_emit_keeps_current_result_until_last_duplicate() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };
        opaque.fallback_result_state.set_current(tid(26, 1), -4.0);
        opaque
            .fallback_result_state
            .store_pending(&[tid(31, 1), tid(31, 2)]);

        let first = linear_fallback_cursor(&mut opaque).take_pending_output();
        linear_fallback_cursor(&mut opaque).advance_after_emit();

        assert_eq!(
            first,
            Some(PendingScanOutput {
                heap_tid: tid(31, 1),
                score: -4.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: None,
            }),
            "linear fallback duplicate drain should still emit the first queued heap tid"
        );
        assert!(
            opaque.fallback_result_state.current().has_element(),
            "linear fallback should keep the current result populated while duplicate drain still remains"
        );
        assert_eq!(
            opaque.fallback_result_state.current().heap_tid(),
            tid(31, 1),
            "linear fallback should keep heap progress aligned with the last emitted duplicate"
        );
    }

    #[test]
    fn linear_fallback_cursor_advance_after_emit_clears_current_result_after_last_duplicate() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };
        opaque.fallback_result_state.set_current(tid(27, 1), -5.0);
        opaque.fallback_result_state.store_pending(&[tid(32, 1)]);

        let emitted = linear_fallback_cursor(&mut opaque).take_pending_output();
        linear_fallback_cursor(&mut opaque).advance_after_emit();

        assert_eq!(
            emitted,
            Some(PendingScanOutput {
                heap_tid: tid(32, 1),
                score: -5.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: None,
            }),
            "linear fallback should still emit the final queued heap tid before teardown"
        );
        assert!(
            !opaque.fallback_result_state.current().has_element(),
            "linear fallback should clear stale current-result state after the last duplicate drains"
        );
        assert_eq!(
            opaque.fallback_result_state.pending_count(),
            0,
            "linear fallback teardown should only happen once duplicate drain is exhausted"
        );
    }

    #[test]
    fn consume_parallel_scan_admitted_result_syncs_local_graph_result_state() {
        let mut opaque = TqScanOpaque {
            parallel_scan_worker_slot_index: 1,
            ..TqScanOpaque::default()
        };
        opaque.result_state.set_current_with_details(
            tid(26, 1),
            -4.0,
            Some(-3.5),
            Some(7),
            Some(-4.5),
        );
        opaque.result_state.store_pending(&[tid(31, 1), tid(31, 2)]);

        let output = consume_parallel_scan_admitted_result(
            &mut opaque,
            crate::am::ec_hnsw::parallel::EcParallelCoordinatorAdmittedResultSnapshot {
                flags: 0,
                source_slot_index: Some(1),
                element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                    block_number: 26,
                    offset_number: 1,
                },
                pending_output: crate::am::ec_hnsw::parallel::EcParallelPendingOutputSnapshot {
                    heap_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 31,
                        offset_number: 1,
                    },
                    score: -4.0,
                    approx_score: Some(-3.5),
                    approx_rank: Some(7),
                    comparison_score: Some(-4.5),
                },
            },
        );

        assert_eq!(
            output,
            PendingScanOutput {
                heap_tid: tid(31, 1),
                score: -4.0,
                approx_score: Some(-3.5),
                approx_rank: Some(7),
                comparison_score: Some(-4.5),
            },
            "parallel admitted-result consume should project the shared snapshot back into the normal scan output shape"
        );
        assert!(
            opaque.result_state.current().has_element(),
            "consuming one duplicate from this worker slot should keep the local current result live while more heap tids remain"
        );
        assert_eq!(
            opaque.result_state.current().heap_tid(),
            tid(31, 1),
            "local graph result state should advance to the emitted duplicate when the admitted row came from this worker slot"
        );
        assert_eq!(
            opaque.result_state.pending_index(),
            1,
            "local graph result state should advance its pending cursor too"
        );
    }

    #[test]
    fn consume_parallel_scan_admitted_result_syncs_linear_fallback_state_only_for_owned_slot() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            parallel_scan_worker_slot_index: 3,
            ..TqScanOpaque::default()
        };
        opaque.fallback_result_state.set_current(tid(27, 1), -5.0);
        opaque
            .fallback_result_state
            .store_pending(&[tid(32, 1), tid(32, 2)]);

        let foreign = consume_parallel_scan_admitted_result(
            &mut opaque,
            crate::am::ec_hnsw::parallel::EcParallelCoordinatorAdmittedResultSnapshot {
                flags: 0,
                source_slot_index: Some(2),
                element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                    block_number: 27,
                    offset_number: 1,
                },
                pending_output: crate::am::ec_hnsw::parallel::EcParallelPendingOutputSnapshot {
                    heap_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 32,
                        offset_number: 1,
                    },
                    score: -5.0,
                    approx_score: None,
                    approx_rank: None,
                    comparison_score: None,
                },
            },
        );
        assert_eq!(
            foreign.heap_tid,
            tid(32, 1),
            "foreign-slot admitted rows should still project into a normal scan output"
        );
        assert_eq!(
            opaque.fallback_result_state.pending_index(),
            0,
            "foreign-slot admitted rows should not advance the local fallback cursor"
        );

        let owned = consume_parallel_scan_admitted_result(
            &mut opaque,
            crate::am::ec_hnsw::parallel::EcParallelCoordinatorAdmittedResultSnapshot {
                flags: 0,
                source_slot_index: Some(3),
                element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                    block_number: 27,
                    offset_number: 1,
                },
                pending_output: crate::am::ec_hnsw::parallel::EcParallelPendingOutputSnapshot {
                    heap_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 32,
                        offset_number: 1,
                    },
                    score: -5.0,
                    approx_score: None,
                    approx_rank: None,
                    comparison_score: None,
                },
            },
        );
        assert_eq!(
            owned.heap_tid,
            tid(32, 1),
            "owned-slot admitted rows should keep the same output projection"
        );
        assert_eq!(
            opaque.fallback_result_state.pending_index(),
            1,
            "owned-slot admitted rows should advance the local fallback cursor"
        );
    }

    #[test]
    fn try_take_parallel_scan_next_output_advances_owned_slot_and_republishes() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 1,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque::default();
        bind_parallel_scan_state(&mut scan_desc, &mut opaque);
        opaque.result_state.set_current_with_details(
            tid(26, 1),
            -4.0,
            Some(-3.5),
            Some(7),
            Some(-4.5),
        );
        opaque.result_state.store_pending(&[tid(31, 1), tid(31, 2)]);

        let output = unsafe { try_take_parallel_scan_next_output(&mut opaque) };

        assert_eq!(
            output,
            ParallelScanOutputState::Emitted(PendingScanOutput {
                heap_tid: tid(31, 1),
                score: -4.0,
                approx_score: Some(-3.5),
                approx_rank: Some(7),
                comparison_score: Some(-4.5),
            }),
            "taking the next staged parallel output should project the admitted row into a normal pending output"
        );
        assert_eq!(
            opaque.result_state.pending_index(),
            1,
            "owned-slot coordinator consume should advance the local duplicate-drain cursor"
        );

        let result_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel coordinator result-slot snapshot should read back");
        assert_eq!(
            result_snapshot.runtime.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 31,
                offset_number: 2,
            },
            "republish after consume should advance the staged next heap tid to the remaining local duplicate"
        );
        assert_eq!(
            result_snapshot.runtime.pending_index, 1,
            "republish after consume should keep shared pending-index state aligned with the local cursor"
        );
    }

    #[test]
    fn try_take_parallel_scan_handoff_output_drains_foreign_admitted_head() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut local_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign = TqScanOpaque::default();
        let mut local = TqScanOpaque::default();
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);
        bind_parallel_scan_state(&mut local_scan_desc, &mut local);

        foreign.result_state.set_current_with_details(
            tid(50, 1),
            -10.0,
            Some(-9.5),
            Some(3),
            Some(-10.5),
        );
        foreign.result_state.store_pending(&[tid(60, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);

        let first_admitted = unsafe {
            crate::am::ec_hnsw::parallel::admit_parallel_scan_selected_pending_output(
                local.parallel_scan_state,
                2,
            )
        }
        .expect("foreign selected output should admit")
        .expect("foreign selected output should seed the admission window");
        assert!(
            first_admitted.admitted,
            "the foreign slot should seed the shared admitted window before handoff"
        );

        let admitted_head = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_admitted_result_snapshot(
                local.parallel_scan_state,
                0,
            )
        }
        .expect("admitted-head snapshot should read back");
        assert_eq!(
            admitted_head.source_slot_index,
            Some(foreign.parallel_scan_worker_slot_index),
            "the foreign slot should own the admitted head before the local worker probes"
        );

        local.result_state.set_current_with_details(
            tid(51, 1),
            -6.0,
            Some(-5.5),
            Some(7),
            Some(-6.5),
        );
        local.result_state.store_pending(&[tid(61, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&local);

        let output = unsafe {
            try_take_parallel_scan_handoff_output(
                &mut local,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead,
                    slot_index: Some(foreign.parallel_scan_worker_slot_index),
                    generation: 1,
                    element_tid: admitted_head.element_tid,
                },
            )
        };

        assert_eq!(
            output,
            Some(PendingScanOutput {
                heap_tid: tid(60, 1),
                score: -10.0,
                approx_score: Some(-9.5),
                approx_rank: Some(3),
                comparison_score: Some(-10.5),
            }),
            "a local worker blocked by a foreign admitted head should drain that admitted row through the shared handoff path"
        );
        assert_eq!(
            local.result_state.current().element_tid(),
            tid(51, 1),
            "draining a foreign admitted row should not mutate the local worker's staged current result"
        );
        assert_eq!(
            local.result_state.pending_index(),
            0,
            "draining a foreign admitted row should not advance the local worker's duplicate-drain cursor"
        );
    }

    #[test]
    fn try_take_parallel_scan_handoff_output_drains_foreign_selected_pending() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut local_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign = TqScanOpaque::default();
        let mut local = TqScanOpaque::default();
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);
        bind_parallel_scan_state(&mut local_scan_desc, &mut local);

        foreign.result_state.set_current_with_details(
            tid(53, 1),
            -8.0,
            Some(-7.5),
            Some(5),
            Some(-8.5),
        );
        foreign.result_state.store_pending(&[tid(63, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);

        local.result_state.set_current_with_details(
            tid(54, 1),
            -6.0,
            Some(-5.5),
            Some(9),
            Some(-6.5),
        );
        local.result_state.store_pending(&[tid(64, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&local);

        let blocker = match unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_owned_output_state(
                local.parallel_scan_state,
                local.parallel_scan_worker_slot_index,
                2,
            )
        }
        .expect("owned output state should read back")
        {
            crate::am::ec_hnsw::parallel::EcParallelOwnedOutputState::Blocked(blocker) => blocker,
            state => panic!("expected foreign-selected blocker, got {state:?}"),
        };

        let output = unsafe { try_take_parallel_scan_handoff_output(&mut local, blocker) };

        assert_eq!(
            output,
            Some(PendingScanOutput {
                heap_tid: tid(63, 1),
                score: -8.0,
                approx_score: Some(-7.5),
                approx_rank: Some(5),
                comparison_score: Some(-8.5),
            }),
            "a local worker blocked by a foreign selected pending row should be able to drain the shared global next output through the handoff helper"
        );
        assert_eq!(
            local.result_state.current().element_tid(),
            tid(54, 1),
            "draining a foreign selected row should not mutate the local worker's staged current result"
        );
        assert_eq!(
            local.result_state.pending_index(),
            0,
            "draining a foreign selected row should not advance the local worker's duplicate-drain cursor"
        );
    }

    #[test]
    fn try_take_parallel_scan_deferred_handoff_output_restores_linear_fallback_row() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut local_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign = TqScanOpaque::default();
        let mut local = TqScanOpaque::default();
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);
        bind_parallel_scan_state(&mut local_scan_desc, &mut local);

        foreign.result_state.set_current_with_details(
            tid(80, 1),
            -8.0,
            Some(-7.5),
            Some(5),
            Some(-8.5),
        );
        foreign.result_state.store_pending(&[tid(81, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);

        local.execution_phase = ScanExecutionPhase::Exhausted;
        let deferred = DeferredParallelBlockedOutput {
            source_phase: ScanExecutionPhase::LinearFallback,
            state: {
                let mut state = ScanResultState::default();
                state.set_current_with_details(
                    tid(82, 1),
                    -6.0,
                    Some(-5.5),
                    Some(9),
                    Some(-6.5),
                );
                state.store_pending(&[tid(83, 1)]);
                state
            },
            retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                    slot_index: Some(foreign.parallel_scan_worker_slot_index),
                    generation: 1,
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 80,
                        offset_number: 1,
                    },
                },
                element_tid: tid(82, 1),
            }),
        };

        let mut deferred = deferred;
        let output =
            unsafe { try_take_parallel_scan_deferred_handoff_output(&mut local, &mut deferred) }
                .expect("deferred local row should be able to drain the foreign selected handoff");

        assert_eq!(
            output,
            PendingScanOutput {
                heap_tid: tid(81, 1),
                score: -8.0,
                approx_score: Some(-7.5),
                approx_rank: Some(5),
                comparison_score: Some(-8.5),
            },
            "deferred handoff retry should drain the foreign selected row before falling back to local emit"
        );
        assert_eq!(
            deferred.source_phase,
            ScanExecutionPhase::LinearFallback,
            "deferred handoff retry should preserve the original source phase for the local row"
        );
        assert_eq!(
            deferred.state.current().element_tid(),
            tid(82, 1),
            "deferred handoff retry should keep the local deferred row intact"
        );
        assert_eq!(
            deferred.state.pending_index(),
            0,
            "draining a foreign handoff row should not advance the deferred local duplicate cursor"
        );
        assert_eq!(
            local.execution_phase,
            ScanExecutionPhase::Exhausted,
            "deferred handoff retry should restore the caller's exhausted execution phase"
        );
        assert!(
            !local.fallback_result_state.current().has_element(),
            "deferred handoff retry should clear the temporary restored linear-fallback state after the handoff attempt"
        );
    }

    #[test]
    fn foreign_selected_handoff_republish_advances_to_next_pending_output() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut local_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign = TqScanOpaque::default();
        let mut local = TqScanOpaque::default();
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);
        bind_parallel_scan_state(&mut local_scan_desc, &mut local);

        foreign.result_state.set_current_with_details(
            tid(55, 1),
            -8.0,
            Some(-7.5),
            Some(5),
            Some(-8.5),
        );
        foreign
            .result_state
            .store_pending(&[tid(65, 1), tid(65, 2)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);

        local.result_state.set_current_with_details(
            tid(56, 1),
            -6.0,
            Some(-5.5),
            Some(9),
            Some(-6.5),
        );
        local.result_state.store_pending(&[tid(66, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&local);

        let first_blocker = match unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_owned_output_state(
                local.parallel_scan_state,
                local.parallel_scan_worker_slot_index,
                2,
            )
        }
        .expect("owned output state should read back")
        {
            crate::am::ec_hnsw::parallel::EcParallelOwnedOutputState::Blocked(blocker) => blocker,
            state => panic!("expected foreign-selected blocker, got {state:?}"),
        };

        let first = unsafe { try_take_parallel_scan_handoff_output(&mut local, first_blocker) };
        assert_eq!(
            first,
            Some(PendingScanOutput {
                heap_tid: tid(65, 1),
                score: -8.0,
                approx_score: Some(-7.5),
                approx_rank: Some(5),
                comparison_score: Some(-8.5),
            }),
            "the first foreign selected handoff should drain the first pending heap tid"
        );

        sync_and_publish_parallel_scan_worker_slot_snapshot(&mut foreign);

        let foreign_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
                foreign.parallel_scan_state,
                foreign.parallel_scan_worker_slot_index,
            )
        }
        .expect("foreign result-slot snapshot should read back");
        assert_eq!(
            foreign_snapshot.runtime.pending_index, 1,
            "foreign republish after handoff should reconcile to the advanced shared pending index"
        );
        assert_eq!(
            foreign_snapshot.runtime.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 65,
                offset_number: 2,
            },
            "foreign republish after handoff should expose the next pending heap tid instead of restaging the drained one"
        );

        let second_blocker = match unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_owned_output_state(
                local.parallel_scan_state,
                local.parallel_scan_worker_slot_index,
                2,
            )
        }
        .expect("owned output state should read back")
        {
            crate::am::ec_hnsw::parallel::EcParallelOwnedOutputState::Blocked(blocker) => blocker,
            state => panic!("expected foreign-selected blocker, got {state:?}"),
        };

        let second = unsafe { try_take_parallel_scan_handoff_output(&mut local, second_blocker) };
        assert_eq!(
            second,
            Some(PendingScanOutput {
                heap_tid: tid(65, 2),
                score: -8.0,
                approx_score: Some(-7.5),
                approx_rank: Some(6),
                comparison_score: Some(-8.5),
            }),
            "a second foreign handoff after republish should drain the next pending heap tid, not re-emit the one already drained"
        );
    }

    #[test]
    fn stale_foreign_selected_handoff_does_not_drain_new_selected_slot() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 320],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 320] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 3,
            )
        }
        .expect("parallel scan target should initialize");

        let mut first_foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut second_foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut local_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut first_foreign = TqScanOpaque::default();
        let mut second_foreign = TqScanOpaque::default();
        let mut local = TqScanOpaque::default();
        bind_parallel_scan_state(&mut first_foreign_scan_desc, &mut first_foreign);
        bind_parallel_scan_state(&mut second_foreign_scan_desc, &mut second_foreign);
        bind_parallel_scan_state(&mut local_scan_desc, &mut local);

        first_foreign.result_state.set_current_with_details(
            tid(70, 1),
            -8.0,
            Some(-7.5),
            Some(5),
            Some(-8.5),
        );
        first_foreign.result_state.store_pending(&[tid(80, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&first_foreign);

        let stale_blocker = crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
            kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
            slot_index: Some(first_foreign.parallel_scan_worker_slot_index),
            generation: 1,
            element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 70,
                offset_number: 1,
            },
        };

        second_foreign.result_state.set_current_with_details(
            tid(71, 1),
            -10.0,
            Some(-9.5),
            Some(3),
            Some(-10.5),
        );
        second_foreign.result_state.store_pending(&[tid(81, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&second_foreign);

        local.result_state.set_current_with_details(
            tid(72, 1),
            -6.0,
            Some(-5.5),
            Some(9),
            Some(-6.5),
        );
        local.result_state.store_pending(&[tid(82, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&local);

        let output = unsafe { try_take_parallel_scan_handoff_output(&mut local, stale_blocker) };
        assert_eq!(
            output, None,
            "a stale foreign-selected blocker should not drain a newly selected foreign slot"
        );

        let selected = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_selected_pending_output_snapshot(
                local.parallel_scan_state,
            )
        }
        .expect("selected pending snapshot should read back")
        .expect("a newer selected pending slot should remain published");
        assert_eq!(
            selected.selected_result_slot.slot_index,
            second_foreign.parallel_scan_worker_slot_index,
            "a stale handoff attempt should leave the newer selected foreign slot intact"
        );
        assert_eq!(
            selected.pending_output.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 81,
                offset_number: 1,
            },
            "a stale handoff attempt should not consume the newer selected foreign row"
        );
    }

    #[test]
    fn emit_materialized_parallel_scan_output_routes_new_local_row_through_shared_merge() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 1,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };
        reset_scan_emitted_state(&mut opaque);
        bind_parallel_scan_state(&mut scan_desc, &mut opaque);

        let output = unsafe {
            emit_materialized_parallel_scan_output(
                &mut opaque,
                SelectedScanResult {
                    element_tid: tid(27, 1),
                    score: -5.0,
                    approx_score: None,
                    approx_rank_base: None,
                    comparison_score: Some(-5.5),
                    heap_tids: CachedHeapTids::from_iter([tid(32, 1), tid(32, 2)]),
                },
            )
        };

        assert_eq!(
            output,
            ParallelScanOutputState::Emitted(PendingScanOutput {
                heap_tid: tid(32, 1),
                score: -5.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: Some(-5.5),
            }),
            "newly materialized local fallback rows should flow through the shared coordinator merge path"
        );
        assert!(
            emitted_contains_element(&opaque, tid(27, 1)),
            "a shared-merge materialized emit should mark the element as emitted only after returning a heap tid"
        );
        assert_eq!(
            opaque.fallback_result_state.pending_index(),
            1,
            "shared merge consume should advance the local fallback duplicate-drain cursor immediately"
        );

        let result_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel coordinator result-slot snapshot should read back");
        assert_eq!(
            result_snapshot.runtime.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 32,
                offset_number: 2,
            },
            "republish after the merged consume should expose the next local duplicate in the shared result-slot snapshot"
        );
    }

    #[test]
    fn emit_materialized_parallel_scan_output_handoffs_foreign_selected_pending() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 320],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 320] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };
        reset_scan_emitted_state(&mut opaque);
        bind_parallel_scan_state(&mut scan_desc, &mut opaque);

        let attachment =
            unsafe { crate::am::ec_hnsw::parallel::parallel_scan_attachment(parallel_scan) }
                .expect("parallel scan attachment should validate")
                .expect("parallel scan attachment should expose AM-private state");
        let second_slot =
            unsafe { crate::am::ec_hnsw::parallel::claim_parallel_scan_worker_slot(&attachment) }
                .expect("second worker claim should succeed");
        assert_eq!(
            second_slot, 1,
            "the first opaque should already own slot zero, so the extra claim should get slot one"
        );

        unsafe {
            crate::am::ec_hnsw::parallel::publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                opaque.parallel_scan_state,
                second_slot,
                opaque.parallel_scan_rescan_epoch,
                crate::am::ec_hnsw::parallel::EcParallelCoordinatorResultSlotRuntimeSnapshot {
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 70,
                        offset_number: 1,
                    },
                    heap_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
                    score: -9.0,
                    approx_score: None,
                    comparison_score: None,
                    approx_rank_base: None,
                    pending_count: 1,
                    pending_index: 0,
                    pending_heap_tids: [crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 71,
                        offset_number: 2,
                    }; page::HEAPTID_INLINE_CAPACITY],
                },
            )
        }
        .expect("foreign publish should succeed");
        assert!(
            unsafe {
                crate::am::ec_hnsw::parallel::admit_parallel_scan_selected_pending_output(
                    opaque.parallel_scan_state,
                    2,
                )
            }
            .expect("foreign admission should succeed")
            .expect("foreign admission should expose the selected pending output")
            .admitted,
            "foreign worker should seed the admitted head"
        );

        let output = unsafe {
            emit_materialized_parallel_scan_output(
                &mut opaque,
                SelectedScanResult {
                    element_tid: tid(27, 1),
                    score: -5.0,
                    approx_score: None,
                    approx_rank_base: None,
                    comparison_score: Some(-5.5),
                    heap_tids: CachedHeapTids::from_iter([tid(32, 1), tid(32, 2)]),
                },
            )
        };

        assert_eq!(
            output,
            ParallelScanOutputState::Emitted(PendingScanOutput {
                heap_tid: tid(71, 2),
                score: -9.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: None,
            }),
            "materialized local staging should hand off a better foreign selected-pending row through the shared merge seam"
        );
        assert!(
            emitted_contains_element(&opaque, tid(70, 1)),
            "foreign handoff should mark the foreign element as emitted once its heap tid is returned"
        );
        let result_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel coordinator result-slot snapshot should read back");
        assert_eq!(
            result_snapshot.runtime.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 32,
                offset_number: 1,
            },
            "foreign handoff should leave the local materialized row staged in the shared slot for the next retry"
        );
        let worker_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_worker_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel worker slot snapshot should read back");
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_kind,
            crate::am::ec_hnsw::parallel::EC_PARALLEL_OWNED_OUTPUT_BLOCKER_NONE,
            "foreign handoff should clear the local blocker after draining the shared output"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_slot_index, None,
            "foreign handoff should clear the blocker slot from the shared worker runtime snapshot"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_generation,
            0,
            "foreign handoff should clear the blocker generation from the shared worker runtime snapshot"
        );
        assert_eq!(
            opaque
                .explain_counters
                .stats_parallel_blocked_foreign_selected_pending,
            0,
            "successful foreign handoff should not increment the blocked foreign-selected EXPLAIN counter"
        );
        assert_eq!(
            opaque
                .explain_counters
                .stats_parallel_blocked_foreign_admitted_head,
            0,
            "foreign selected handoff should not increment the foreign-head EXPLAIN counter"
        );
    }

    #[test]
    fn emit_prefetched_parallel_scan_output_routes_prefetched_row_through_shared_merge() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 1,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque::default();
        reset_scan_emitted_state(&mut opaque);
        bind_parallel_scan_state(&mut scan_desc, &mut opaque);
        opaque.result_state.set_current_with_details(
            tid(26, 1),
            -4.0,
            Some(-3.5),
            Some(7),
            Some(-4.5),
        );
        opaque.result_state.store_pending(&[tid(31, 1), tid(31, 2)]);

        let output = unsafe { emit_prefetched_parallel_scan_output(&mut opaque) };
        assert_eq!(
            output,
            ParallelScanOutputState::Emitted(PendingScanOutput {
                heap_tid: tid(31, 1),
                score: -4.0,
                approx_score: Some(-3.5),
                approx_rank: Some(7),
                comparison_score: Some(-4.5),
            }),
            "prefetched graph rows should drain through the shared merge seam"
        );
        assert!(
            emitted_contains_element(&opaque, tid(26, 1)),
            "a shared-merge prefetched emit should mark the element as emitted only after returning a heap tid"
        );
        assert_eq!(
            opaque.result_state.pending_index(),
            1,
            "shared merge consume should advance the local graph duplicate-drain cursor"
        );

        let result_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel coordinator result-slot snapshot should read back");
        assert_eq!(
            result_snapshot.runtime.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 31,
                offset_number: 2,
            },
            "republish after graph-side consume should expose the next local duplicate in the shared result-slot snapshot"
        );
    }

    #[test]
    fn emit_prefetched_parallel_scan_output_handoffs_foreign_selected_pending() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 320],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 320] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque::default();
        reset_scan_emitted_state(&mut opaque);
        bind_parallel_scan_state(&mut scan_desc, &mut opaque);
        opaque.result_state.set_current_with_details(
            tid(26, 1),
            -4.0,
            Some(-3.5),
            Some(7),
            Some(-4.5),
        );
        opaque.result_state.store_pending(&[tid(31, 1), tid(31, 2)]);

        let attachment =
            unsafe { crate::am::ec_hnsw::parallel::parallel_scan_attachment(parallel_scan) }
                .expect("parallel scan attachment should validate")
                .expect("parallel scan attachment should expose AM-private state");
        let second_slot =
            unsafe { crate::am::ec_hnsw::parallel::claim_parallel_scan_worker_slot(&attachment) }
                .expect("second worker claim should succeed");
        assert_eq!(second_slot, 1, "foreign claim should bind slot one");
        unsafe {
            crate::am::ec_hnsw::parallel::publish_parallel_scan_coordinator_result_slot_runtime_snapshot(
                opaque.parallel_scan_state,
                second_slot,
                opaque.parallel_scan_rescan_epoch,
                crate::am::ec_hnsw::parallel::EcParallelCoordinatorResultSlotRuntimeSnapshot {
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 80,
                        offset_number: 1,
                    },
                    heap_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
                    score: -9.0,
                    approx_score: None,
                    comparison_score: None,
                    approx_rank_base: None,
                    pending_count: 1,
                    pending_index: 0,
                    pending_heap_tids: [crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 81,
                        offset_number: 2,
                    }; page::HEAPTID_INLINE_CAPACITY],
                },
            )
        }
        .expect("foreign publish should succeed");
        assert!(
            unsafe {
                crate::am::ec_hnsw::parallel::admit_parallel_scan_selected_pending_output(
                    opaque.parallel_scan_state,
                    2,
                )
            }
            .expect("foreign admission should succeed")
            .expect("foreign admission should expose the selected pending output")
            .admitted,
            "foreign worker should seed the admitted head"
        );

        let output = unsafe { emit_prefetched_parallel_scan_output(&mut opaque) };
        assert_eq!(
            output,
            ParallelScanOutputState::Emitted(PendingScanOutput {
                heap_tid: tid(81, 2),
                score: -9.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: None,
            }),
            "prefetched local staging should hand off a better foreign selected-pending row through the shared merge seam"
        );
        assert!(
            emitted_contains_element(&opaque, tid(80, 1)),
            "foreign handoff should mark the foreign prefetched element as emitted once its heap tid is returned"
        );
        let result_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel coordinator result-slot snapshot should read back");
        assert_eq!(
            result_snapshot.runtime.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 31,
                offset_number: 1,
            },
            "foreign handoff should leave the local prefetched row staged at the current heap tid"
        );
        let worker_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_worker_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel worker slot snapshot should read back");
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_kind,
            crate::am::ec_hnsw::parallel::EC_PARALLEL_OWNED_OUTPUT_BLOCKER_NONE,
            "foreign handoff should clear the blocker kind from the shared worker runtime snapshot"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_slot_index, None,
            "foreign handoff should clear the blocker slot from the shared worker runtime snapshot"
        );
        assert_eq!(
            worker_snapshot.runtime.owned_output_blocker_generation,
            0,
            "foreign handoff should clear the blocker generation from the shared worker runtime snapshot"
        );
        assert_eq!(
            opaque
                .explain_counters
                .stats_parallel_blocked_foreign_selected_pending,
            0,
            "successful foreign handoff should not increment the blocked foreign-selected EXPLAIN counter"
        );
        assert_eq!(
            opaque
                .explain_counters
                .stats_parallel_blocked_foreign_admitted_head,
            0,
            "foreign selected handoff should not increment the foreign-head EXPLAIN counter"
        );
    }

    #[test]
    fn graph_traversal_prefetch_ready_clears_stale_current_without_pending_output() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::GraphTraversal,
            ..TqScanOpaque::default()
        };
        opaque.result_state.set_current(tid(28, 1), -6.0);

        let ready = graph_traversal_cursor(&mut opaque).prefetch_ready();

        assert!(
            !ready,
            "graph traversal should request a fresh materialization when only stale current-result state remains"
        );
        assert!(
            !opaque.result_state.current().has_element(),
            "graph traversal should clear stale current-result state before trying to prefill a fresh ordered result"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "graph traversal stale-current cleanup should not invent pending duplicate-drain state"
        );
    }

    #[test]
    fn graph_traversal_cursor_has_prefetched_output_requires_pending_duplicate_drain() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::GraphTraversal,
            ..TqScanOpaque::default()
        };
        opaque.result_state.set_current(tid(29, 1), -7.0);

        assert!(
            !graph_traversal_cursor(&mut opaque).has_prefetched_output(),
            "graph traversal should only report prefetched output when duplicate drain is actually queued"
        );

        opaque.result_state.store_pending(&[tid(33, 1)]);

        assert!(
            graph_traversal_cursor(&mut opaque).has_prefetched_output(),
            "graph traversal should report prefetched output once a current result has pending heap tids ready to emit"
        );
    }

    #[test]
    fn graph_traversal_cursor_take_pending_output_drains_prefetched_heap_tid() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(34, 1), -8.0);
        opaque.result_state.store_pending(&[tid(35, 1)]);

        let emitted = graph_traversal_cursor(&mut opaque).take_pending_output();

        assert!(
            emitted.is_some(),
            "graph cursor should surface pending output when prefetched duplicate drain is queued"
        );
        assert_eq!(
            opaque.result_state.current().heap_tid(),
            tid(35, 1),
            "graph cursor pending-output drain should keep current-result heap progress aligned with the drained heap tid"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "graph cursor pending-output drain should consume the prefetched heap tid from pending state"
        );
    }

    #[test]
    fn linear_fallback_cursor_uses_fallback_storage_in_linear_phase() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };
        opaque.result_state.set_current(tid(36, 1), -9.0);

        linear_fallback_cursor(&mut opaque).materialize(SelectedScanResult {
            element_tid: tid(37, 1),
            score: -10.0,
            approx_score: None,
            approx_rank_base: None,
            comparison_score: None,
            heap_tids: CachedHeapTids::from_iter([tid(38, 1)]),
        });

        assert_eq!(
            opaque.fallback_result_state.current().element_tid(),
            tid(37, 1),
            "linear fallback should read and write through its dedicated fallback result-state storage"
        );
        assert_eq!(
            opaque.result_state.current().element_tid(),
            tid(36, 1),
            "linear fallback cursor should not backfill graph cursor result-state storage"
        );
    }

    #[test]
    fn linear_fallback_cursor_materialize_uses_fallback_storage() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };

        linear_fallback_cursor(&mut opaque).materialize(SelectedScanResult {
            element_tid: tid(38, 1),
            score: -11.0,
            approx_score: None,
            approx_rank_base: None,
            comparison_score: None,
            heap_tids: CachedHeapTids::from_iter([tid(39, 1)]),
        });

        assert_eq!(
            opaque.fallback_result_state.current().element_tid(),
            tid(38, 1),
            "linear fallback materialization should populate fallback-only result-state storage"
        );
        assert_eq!(
            opaque.result_state.current().element_tid(),
            page::ItemPointer::INVALID,
            "linear fallback materialization should not backfill graph cursor result-state storage"
        );
    }

    #[test]
    fn blocked_parallel_scan_disposition_drops_admission_window() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(40, 1), -1.0);

        assert_eq!(
            blocked_parallel_scan_disposition(
                &mut opaque,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow,
                    slot_index: None,
                    generation: 0,
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
                }
            ),
            BlockedParallelScanDisposition::DropAndContinue,
            "admission-window blockers should drop the staged local row and continue searching"
        );
    }

    #[test]
    fn blocked_parallel_scan_disposition_retries_on_new_foreign_blocker() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(41, 1), -2.0);

        assert_eq!(
            blocked_parallel_scan_disposition(
                &mut opaque,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                    slot_index: Some(1),
                    generation: 7,
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 99,
                        offset_number: 1,
                    },
                }
            ),
            BlockedParallelScanDisposition::RetryShared,
            "the first observation of a foreign-owner blocker should retry the shared seam before falling back to local emit"
        );
    }

    #[test]
    fn blocked_parallel_scan_disposition_keeps_local_emit_for_stable_foreign_blocker() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(42, 1), -3.0);
        let blocker = crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
            kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
            slot_index: Some(1),
            generation: 7,
            element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 99,
                offset_number: 1,
            },
        };

        assert_eq!(
            blocked_parallel_scan_disposition(&mut opaque, blocker),
            BlockedParallelScanDisposition::RetryShared,
            "the first foreign-owner blocker observation should still retry once"
        );
        assert_eq!(
            blocked_parallel_scan_disposition(&mut opaque, blocker),
            BlockedParallelScanDisposition::KeepLocalEmit,
            "a repeated foreign-owner blocker for the same staged row should fall back to local emit"
        );
    }

    #[test]
    fn blocked_parallel_scan_disposition_retries_when_foreign_generation_changes() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(43, 1), -4.0);

        assert_eq!(
            blocked_parallel_scan_disposition(
                &mut opaque,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                    slot_index: Some(1),
                    generation: 7,
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 99,
                        offset_number: 1,
                    },
                }
            ),
            BlockedParallelScanDisposition::RetryShared,
            "the first foreign-owner blocker observation should retry once"
        );
        assert_eq!(
            blocked_parallel_scan_disposition(
                &mut opaque,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                    slot_index: Some(1),
                    generation: 8,
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 99,
                        offset_number: 1,
                    },
                }
            ),
            BlockedParallelScanDisposition::RetryShared,
            "a changed foreign-owner generation should reopen one retry against the shared seam"
        );
    }

    #[test]
    fn should_drop_deferred_parallel_blocked_output_for_admission_window() {
        let mut state = ScanResultState::default();
        state.set_current(tid(44, 1), -4.0);
        state.store_pending(&[tid(45, 1)]);

        assert!(should_drop_deferred_parallel_blocked_output(
            &DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::LinearFallback,
                state,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow,
                        slot_index: None,
                        generation: 9,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
                    },
                    element_tid: tid(44, 1),
                }),
            }
        ));
    }

    #[test]
    fn should_drop_deferred_parallel_blocked_output_for_same_element_foreign_blocker() {
        let mut state = ScanResultState::default();
        state.set_current(tid(46, 1), -6.0);
        state.store_pending(&[tid(47, 1)]);

        assert!(should_drop_deferred_parallel_blocked_output(
            &DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                        slot_index: Some(2),
                        generation: 11,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                            block_number: 46,
                            offset_number: 1,
                        },
                    },
                    element_tid: tid(46, 1),
                }),
            }
        ));
    }

    #[test]
    fn should_keep_deferred_parallel_blocked_output_for_distinct_foreign_blocker() {
        let mut state = ScanResultState::default();
        state.set_current(tid(48, 1), -7.0);
        state.store_pending(&[tid(49, 1)]);

        assert!(!should_drop_deferred_parallel_blocked_output(
            &DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead,
                        slot_index: Some(3),
                        generation: 12,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                            block_number: 88,
                            offset_number: 1,
                        },
                    },
                    element_tid: tid(48, 1),
                }),
            }
        ));
    }

    #[test]
    fn emit_next_deferred_parallel_blocked_output_skips_obsolete_row() {
        let mut opaque = TqScanOpaque::default();

        let mut obsolete = ScanResultState::default();
        obsolete.set_current(tid(50, 1), -9.0);
        obsolete.store_pending(&[tid(51, 1)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: obsolete,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::AdmissionWindow,
                        slot_index: None,
                        generation: 17,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
                    },
                    element_tid: tid(50, 1),
                }),
            });

        let mut next = ScanResultState::default();
        next.set_current(tid(52, 1), -5.0);
        next.store_pending(&[tid(53, 1)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::LinearFallback,
                state: next,
                retained_blocker: None,
            });

        assert_eq!(
            take_next_deferred_parallel_blocked_output(&mut opaque, true),
            Some(PendingScanOutput {
                heap_tid: tid(53, 1),
                score: -5.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: None,
            }),
            "deferred drain should skip the obsolete row and return the next eligible deferred output"
        );
        assert!(
            !staged_or_emitted_contains_element(&opaque, tid(50, 1)),
            "dropping an obsolete deferred row should remove that element from staged-or-emitted ownership tracking"
        );
        assert!(
            opaque.deferred_parallel_blocked_results.is_empty(),
            "the next eligible deferred row should also drain completely in this single-pending fixture"
        );
    }

    #[test]
    fn emit_next_deferred_parallel_blocked_output_returns_false_for_only_obsolete_row() {
        let mut scan_desc = pg_sys::IndexScanDescData::default();
        let scan = &mut scan_desc as pg_sys::IndexScanDesc;
        let mut opaque = TqScanOpaque::default();

        let mut obsolete = ScanResultState::default();
        obsolete.set_current(tid(54, 1), -8.0);
        obsolete.store_pending(&[tid(55, 1)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: obsolete,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                        slot_index: Some(4),
                        generation: 18,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                            block_number: 54,
                            offset_number: 1,
                        },
                    },
                    element_tid: tid(54, 1),
                }),
            });

        assert!(
            !emit_next_deferred_parallel_blocked_output(scan, &mut opaque),
            "if the only deferred row is obsolete, the deferred drain should finish without emitting a local row"
        );
        assert!(
            opaque.deferred_parallel_blocked_results.is_empty(),
            "obsolete deferred rows should drop out of the stash entirely"
        );
        let emitted_heap_tid = unsafe { (*scan).xs_heaptid };
        assert_eq!(
            (
                emitted_heap_tid.ip_blkid.bi_hi,
                emitted_heap_tid.ip_blkid.bi_lo,
                emitted_heap_tid.ip_posid
            ),
            (0_u16, 0_u16, 0_u16),
            "dropping the only obsolete deferred row should leave the scan heap tid untouched"
        );
    }

    #[test]
    fn emit_next_deferred_parallel_blocked_output_skips_live_blocked_row_for_ready_next_row() {
        let mut opaque = TqScanOpaque::default();

        let mut blocked = ScanResultState::default();
        blocked.set_current(tid(56, 1), -9.0);
        blocked.store_pending(&[tid(57, 1)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: blocked,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                        slot_index: Some(4),
                        generation: 19,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                            block_number: 99,
                            offset_number: 1,
                        },
                    },
                    element_tid: tid(56, 1),
                }),
            });

        let mut ready = ScanResultState::default();
        ready.set_current(tid(58, 1), -5.0);
        ready.store_pending(&[tid(59, 1)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::LinearFallback,
                state: ready,
                retained_blocker: None,
            });

        assert_eq!(
            take_next_deferred_parallel_blocked_output(&mut opaque, true),
            Some(PendingScanOutput {
                heap_tid: tid(59, 1),
                score: -5.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: None,
            }),
            "deferred drain should skip a still-blocked best row and return the next ready deferred output first"
        );
        assert_eq!(
            opaque.deferred_parallel_blocked_results.len(),
            1,
            "the still-blocked deferred row should remain stashed after the ready row drains"
        );
        assert_eq!(
            opaque.deferred_parallel_blocked_results[0]
                .state
                .current()
                .element_tid(),
            tid(56, 1),
            "the blocked deferred row should stay in the stash for a later retry"
        );
    }

    #[test]
    fn emit_next_deferred_parallel_blocked_output_locally_emits_only_live_blocked_row() {
        let mut scan_desc = pg_sys::IndexScanDescData::default();
        let scan = &mut scan_desc as pg_sys::IndexScanDesc;
        let mut opaque = TqScanOpaque::default();

        let mut blocked = ScanResultState::default();
        blocked.set_current(tid(66, 1), -9.0);
        blocked.store_pending(&[tid(67, 1)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: blocked,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead,
                        slot_index: Some(4),
                        generation: 22,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                            block_number: 120,
                            offset_number: 1,
                        },
                    },
                    element_tid: tid(66, 1),
                }),
            });

        assert!(
            emit_next_deferred_parallel_blocked_output(scan, &mut opaque),
            "if no deferred row can hand off or drain safely, the staged path should still make progress by locally emitting the remaining blocked row"
        );
        let emitted_heap_tid = unsafe { (*scan).xs_heaptid };
        assert_eq!(
            (
                emitted_heap_tid.ip_blkid.bi_hi,
                emitted_heap_tid.ip_blkid.bi_lo,
                emitted_heap_tid.ip_posid
            ),
            (0_u16, 67_u16, 1_u16),
            "the final fallback emit should still come from the blocked row's own pending output"
        );
        assert!(
            opaque.deferred_parallel_blocked_results.is_empty(),
            "the single blocked row should drain completely once it becomes the only remaining deferred work"
        );
        assert_eq!(
            opaque.explain_counters.stats_parallel_deferred_local_emits,
            1,
            "forcing the last still-blocked deferred row through a local emit should be visible in EXPLAIN counters"
        );
    }

    #[test]
    fn take_next_deferred_parallel_blocked_output_skips_live_shared_heap_tid_duplicate() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 320],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 320] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut owner_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut owner = TqScanOpaque::default();
        let mut foreign = TqScanOpaque::default();
        bind_parallel_scan_state(&mut owner_scan_desc, &mut owner);
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);

        foreign.result_state.set_current(tid(73, 1), -10.0);
        foreign.result_state.store_pending(&[tid(74, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);
        let selected = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_selected_pending_output_snapshot(
                owner.parallel_scan_state,
            )
        }
        .expect("parallel selected snapshot should read back")
        .expect("foreign worker should seed the shared selected pending output");

        let mut blocked = ScanResultState::default();
        blocked.set_current(tid(75, 1), -9.0);
        blocked.store_pending(&[tid(74, 1), tid(75, 2)]);
        owner
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: blocked,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                        slot_index: selected.coordinator.selected_result_slot_index,
                        generation: selected.coordinator.result_publish_generation,
                        element_tid: selected.selected_result_slot.runtime.element_tid,
                    },
                    element_tid: tid(75, 1),
                }),
            });

        assert!(
            deferred_parallel_blocked_output_duplicates_live_foreign_heap_tid(
                &owner,
                &owner.deferred_parallel_blocked_results[0]
            ),
            "the duplicate-suppression helper should detect when a still-live foreign selected row already owns the same heap tid as the deferred local row"
        );
    }

    #[test]
    fn take_next_deferred_parallel_blocked_output_retries_handoff_after_duplicate_skip() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 320],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 320] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut owner_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut owner = TqScanOpaque::default();
        let mut foreign = TqScanOpaque::default();
        bind_parallel_scan_state(&mut owner_scan_desc, &mut owner);
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);

        foreign.result_state.set_current_with_details(
            tid(90, 1),
            -9.0,
            Some(-8.5),
            Some(4),
            Some(-9.5),
        );
        foreign.result_state.store_pending(&[tid(91, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);
        let selected = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_selected_pending_output_snapshot(
                owner.parallel_scan_state,
            )
        }
        .expect("parallel selected snapshot should read back")
        .expect("foreign worker should seed the shared selected pending output");

        let mut blocked = ScanResultState::default();
        blocked.set_current_with_details(tid(92, 1), -8.0, Some(-7.5), Some(9), Some(-8.5));
        blocked.store_pending(&[tid(91, 1), tid(92, 2)]);
        owner
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: blocked,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                        slot_index: selected.coordinator.selected_result_slot_index,
                        generation: selected.coordinator.result_publish_generation,
                        element_tid: selected.selected_result_slot.runtime.element_tid,
                    },
                    element_tid: tid(92, 1),
                }),
            });

        assert_eq!(
            take_next_deferred_parallel_blocked_output(&mut owner, true),
            Some(PendingScanOutput {
                heap_tid: tid(91, 1),
                score: -9.0,
                approx_score: Some(-8.5),
                approx_rank: Some(4),
                comparison_score: Some(-9.5),
            }),
            "after skipping a foreign-owned duplicate heap tid, deferred drain should re-enter the shared handoff path before locally emitting the remaining row"
        );
        assert_eq!(
            owner.deferred_parallel_blocked_results.len(),
            1,
            "the local deferred row should stay stashed after the foreign handoff output drains"
        );
        assert!(
            owner.deferred_parallel_blocked_results[0].state.pending_count() > 0,
            "retrying the shared handoff after duplicate suppression should leave the remaining local row staged for a later drain"
        );
        assert_eq!(
            owner.deferred_parallel_blocked_results[0]
                .state
                .current()
                .element_tid(),
            tid(92, 1),
            "the local deferred row should remain stashed for later ownership resolution after the foreign handoff drains"
        );
    }

    #[test]
    fn should_prefer_deferred_parallel_blocked_output_when_it_beats_active_result() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..Default::default()
        };
        opaque
            .fallback_result_state
            .set_current_with_details(tid(60, 1), -4.0, None, None, None);
        opaque.fallback_result_state.store_pending(&[tid(60, 2)]);

        let mut deferred = ScanResultState::default();
        deferred.set_current_with_details(tid(61, 1), -8.0, None, None, None);
        deferred.store_pending(&[tid(61, 2)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: deferred,
                retained_blocker: None,
            });

        let preferred = should_prefer_deferred_parallel_blocked_output(&opaque)
            .expect("better deferred row should be preferred over the active local row");
        assert_eq!(
            preferred.state.current().element_tid(),
            tid(61, 1),
            "the preferred deferred row should be the lower-score blocked row"
        );
    }

    #[test]
    fn should_prefer_deferred_parallel_blocked_output_without_active_result() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..Default::default()
        };

        let mut deferred = ScanResultState::default();
        deferred.set_current_with_details(tid(83, 1), -8.0, None, None, None);
        deferred.store_pending(&[tid(83, 2)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: deferred,
                retained_blocker: None,
            });

        let preferred = should_prefer_deferred_parallel_blocked_output(&opaque)
            .expect("with no active local row, the best deferred row should be preferred");
        assert_eq!(
            preferred.state.current().element_tid(),
            tid(83, 1),
            "the best deferred row should be preferred when no active local row is staged"
        );
    }

    #[test]
    fn take_preferred_deferred_parallel_blocked_output_prefers_better_deferred_row() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..Default::default()
        };
        opaque.fallback_result_state.set_current_with_details(
            tid(62, 1),
            -4.0,
            Some(-4.5),
            Some(2),
            Some(-4.25),
        );
        opaque.fallback_result_state.store_pending(&[tid(62, 2)]);

        let mut deferred = ScanResultState::default();
        deferred.set_current_with_details(tid(63, 1), -8.0, Some(-8.5), Some(1), Some(-8.25));
        deferred.store_pending(&[tid(63, 2)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: deferred,
                retained_blocker: None,
            });

        assert_eq!(
            take_preferred_deferred_parallel_blocked_output(&mut opaque)
                .expect("a better deferred row should emit before the worse active local row"),
            PendingScanOutput {
                heap_tid: tid(63, 2),
                score: -8.0,
                approx_score: Some(-8.5),
                approx_rank: Some(1),
                comparison_score: Some(-8.25),
            },
            "the preferred deferred take should return the deferred row's pending output first"
        );
        assert_eq!(
            opaque.fallback_result_state.current().element_tid(),
            tid(62, 1),
            "preferring a deferred row should leave the active local row intact for the next turn"
        );
        assert!(
            opaque.deferred_parallel_blocked_results.is_empty(),
            "the single-pending deferred row should drain completely after emitting first"
        );
    }

    #[test]
    fn take_preferred_deferred_parallel_blocked_output_prefers_ready_deferred_without_active_row() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..Default::default()
        };

        let mut deferred = ScanResultState::default();
        deferred.set_current_with_details(tid(84, 1), -8.0, Some(-8.5), Some(1), Some(-8.25));
        deferred.store_pending(&[tid(84, 2)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: deferred,
                retained_blocker: None,
            });

        assert_eq!(
            take_preferred_deferred_parallel_blocked_output(&mut opaque)
                .expect("a ready deferred row should drain before the scan looks for fresh local work"),
            PendingScanOutput {
                heap_tid: tid(84, 2),
                score: -8.0,
                approx_score: Some(-8.5),
                approx_rank: Some(1),
                comparison_score: Some(-8.25),
            },
            "the preferred deferred take should return the staged deferred row even when no active local row exists"
        );
        assert!(
            opaque.deferred_parallel_blocked_results.is_empty(),
            "the single-pending deferred row should drain completely after emitting first"
        );
    }

    #[test]
    fn take_preferred_deferred_parallel_blocked_output_keeps_live_blocked_row_deferred() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..Default::default()
        };
        opaque
            .fallback_result_state
            .set_current_with_details(tid(64, 1), -4.0, None, None, None);
        opaque.fallback_result_state.store_pending(&[tid(64, 2)]);

        let mut deferred = ScanResultState::default();
        deferred.set_current_with_details(tid(65, 1), -8.0, None, None, None);
        deferred.store_pending(&[tid(65, 2)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: deferred,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                        slot_index: Some(2),
                        generation: 21,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                            block_number: 99,
                            offset_number: 1,
                        },
                    },
                    element_tid: tid(65, 1),
                }),
            });

        assert_eq!(
            take_preferred_deferred_parallel_blocked_output(&mut opaque),
            None,
            "a better deferred row that is still blocked by a live foreign owner should stay deferred instead of locally emitting early"
        );
        assert_eq!(
            opaque.deferred_parallel_blocked_results.len(),
            1,
            "an unresolved preferred deferred row should remain in the stash"
        );
        assert_eq!(
            opaque.deferred_parallel_blocked_results[0]
                .state
                .current()
                .element_tid(),
            tid(65, 1),
            "the blocked deferred row should stay intact for a later retry"
        );
        assert_eq!(
            opaque.fallback_result_state.current().element_tid(),
            tid(64, 1),
            "keeping the blocked deferred row deferred should leave the active local row untouched"
        );
    }

    #[test]
    fn take_preferred_deferred_parallel_blocked_output_skips_blocked_best_for_ready_next() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..Default::default()
        };
        opaque
            .fallback_result_state
            .set_current_with_details(tid(80, 1), -4.0, None, None, None);
        opaque.fallback_result_state.store_pending(&[tid(80, 2)]);

        let mut blocked = ScanResultState::default();
        blocked.set_current_with_details(tid(81, 1), -9.0, None, None, None);
        blocked.store_pending(&[tid(81, 2)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: blocked,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                        slot_index: Some(2),
                        generation: 31,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                            block_number: 140,
                            offset_number: 1,
                        },
                    },
                    element_tid: tid(81, 1),
                }),
            });

        let mut ready = ScanResultState::default();
        ready.set_current_with_details(tid(82, 1), -8.0, None, None, None);
        ready.store_pending(&[tid(82, 2)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: ready,
                retained_blocker: None,
            });

        assert_eq!(
            take_preferred_deferred_parallel_blocked_output(&mut opaque),
            Some(PendingScanOutput {
                heap_tid: tid(82, 2),
                score: -8.0,
                approx_score: None,
                approx_rank: None,
                comparison_score: None,
            }),
            "a blocked best deferred row should not prevent a ready next deferred row from outranking the active local row"
        );
        assert_eq!(
            opaque.fallback_result_state.current().element_tid(),
            tid(80, 1),
            "preferring a ready deferred row should leave the active local row intact"
        );
        assert_eq!(
            opaque.deferred_parallel_blocked_results.len(),
            1,
            "the blocked best deferred row should remain deferred after the ready next row drains"
        );
        assert_eq!(
            opaque.deferred_parallel_blocked_results[0]
                .state
                .current()
                .element_tid(),
            tid(81, 1),
            "the still-blocked best deferred row should stay in the stash for a later retry"
        );
    }

    #[test]
    fn take_preferred_deferred_parallel_blocked_output_does_not_emit_worse_ready_row() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..Default::default()
        };
        opaque
            .fallback_result_state
            .set_current_with_details(tid(85, 1), -7.0, None, None, None);
        opaque.fallback_result_state.store_pending(&[tid(85, 2)]);

        let mut blocked = ScanResultState::default();
        blocked.set_current_with_details(tid(86, 1), -9.0, None, None, None);
        blocked.store_pending(&[tid(86, 2)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: blocked,
                retained_blocker: Some(RetainedParallelOwnedOutputBlocker {
                    blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                        kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                        slot_index: Some(3),
                        generation: 33,
                        element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                            block_number: 150,
                            offset_number: 1,
                        },
                    },
                    element_tid: tid(86, 1),
                }),
            });

        let mut ready = ScanResultState::default();
        ready.set_current_with_details(tid(87, 1), -6.0, None, None, None);
        ready.store_pending(&[tid(87, 2)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: ready,
                retained_blocker: None,
            });

        assert_eq!(
            take_preferred_deferred_parallel_blocked_output(&mut opaque),
            None,
            "a blocked best deferred row should not cause a worse ready deferred row to outrank the active local row"
        );
        assert_eq!(
            opaque.fallback_result_state.current().element_tid(),
            tid(85, 1),
            "rejecting the worse ready deferred row should leave the active local row intact"
        );
        assert_eq!(
            opaque.deferred_parallel_blocked_results.len(),
            2,
            "both deferred rows should remain stashed when no preferable deferred output exists"
        );
    }

    #[test]
    fn blocked_parallel_scan_disposition_retries_when_owner_slot_progresses() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 320],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 320] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut owner_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut owner = TqScanOpaque::default();
        let mut foreign = TqScanOpaque::default();
        bind_parallel_scan_state(&mut owner_scan_desc, &mut owner);
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);

        owner.result_state.set_current(tid(45, 1), -7.0);
        owner.result_state.store_pending(&[tid(46, 1), tid(46, 2)]);
        publish_parallel_scan_worker_slot_snapshot(&owner);

        let advanced = unsafe {
            crate::am::ec_hnsw::parallel::take_parallel_scan_selected_pending_output_snapshot(
                owner.parallel_scan_state,
            )
        }
        .expect("parallel selected take should succeed")
        .expect("owner slot should seed the selected pending output");
        assert_eq!(
            advanced.pending_output.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 46,
                offset_number: 1,
            },
            "the first shared drain should advance the owner slot to its second heap tid"
        );

        foreign.result_state.set_current(tid(47, 1), -9.0);
        foreign.result_state.store_pending(&[tid(48, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);

        assert_eq!(
            blocked_parallel_scan_disposition(
                &mut owner,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                    slot_index: Some(foreign.parallel_scan_worker_slot_index),
                    generation: 2,
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 47,
                        offset_number: 1,
                    },
                }
            ),
            BlockedParallelScanDisposition::RetryShared,
            "a stale local owner cursor should retry the shared seam after reconciling to the advanced shared slot"
        );
        assert_eq!(
            owner.result_state.pending_index(),
            1,
            "owner reconciliation should advance local duplicate-drain progress to the shared pending index"
        );
        assert_eq!(
            owner.result_state.pending_count(),
            2,
            "owner reconciliation should preserve the remaining staged duplicate set"
        );
    }

    #[test]
    fn blocked_parallel_scan_disposition_retries_when_owner_slot_was_fully_drained() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 320],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 320] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut owner_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut owner = TqScanOpaque::default();
        let mut foreign = TqScanOpaque::default();
        bind_parallel_scan_state(&mut owner_scan_desc, &mut owner);
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);

        owner.result_state.set_current(tid(49, 1), -7.0);
        owner.result_state.store_pending(&[tid(50, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&owner);

        let drained = unsafe {
            crate::am::ec_hnsw::parallel::take_parallel_scan_selected_pending_output_snapshot(
                owner.parallel_scan_state,
            )
        }
        .expect("parallel selected take should succeed")
        .expect("owner slot should publish one selected pending output");
        assert_eq!(
            drained.pending_output.heap_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                block_number: 50,
                offset_number: 1,
            },
            "the shared take should fully drain the staged owner slot"
        );

        foreign.result_state.set_current(tid(51, 1), -9.0);
        foreign.result_state.store_pending(&[tid(52, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);

        assert_eq!(
            blocked_parallel_scan_disposition(
                &mut owner,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                    slot_index: Some(foreign.parallel_scan_worker_slot_index),
                    generation: 2,
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 51,
                        offset_number: 1,
                    },
                }
            ),
            BlockedParallelScanDisposition::RetryShared,
            "a fully drained shared owner slot should retry the shared seam after clearing stale local state"
        );
        assert!(
            !owner.result_state.current().has_element(),
            "owner reconciliation should clear a stale local current result once the shared slot was fully drained elsewhere"
        );
        assert_eq!(
            owner.result_state.pending_count(),
            0,
            "owner reconciliation should clear the local duplicate-drain buffer after full shared drain"
        );
    }

    #[test]
    fn blocked_parallel_scan_disposition_retries_after_consuming_live_foreign_duplicate_heap_tid() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 320],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 320] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut owner_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut owner = TqScanOpaque::default();
        let mut foreign = TqScanOpaque::default();
        bind_parallel_scan_state(&mut owner_scan_desc, &mut owner);
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);

        foreign.result_state.set_current(tid(53, 1), -9.0);
        foreign.result_state.store_pending(&[tid(54, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);
        let selected = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_selected_pending_output_snapshot(
                owner.parallel_scan_state,
            )
        }
        .expect("parallel selected snapshot should read back")
        .expect("foreign worker should seed the shared selected pending output");

        owner.result_state.set_current(tid(55, 1), -8.0);
        owner.result_state.store_pending(&[tid(54, 1), tid(54, 2)]);
        owner.parallel_owned_output_blocker = Some(
            crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                slot_index: selected.coordinator.selected_result_slot_index,
                generation: selected.coordinator.result_publish_generation,
                element_tid: selected.selected_result_slot.runtime.element_tid,
            },
        );

        assert_eq!(
            blocked_parallel_scan_disposition(
                &mut owner,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                    slot_index: selected.coordinator.selected_result_slot_index,
                    generation: selected.coordinator.result_publish_generation,
                    element_tid: selected.selected_result_slot.runtime.element_tid,
                }
            ),
            BlockedParallelScanDisposition::RetryShared,
            "consuming a live foreign-owned duplicate heap tid should reopen the shared retry path before the local row is deferred"
        );
        assert_eq!(
            owner.result_state.pending_index(),
            1,
            "active duplicate suppression should consume the foreign-owned duplicate heap tid from the local pending queue"
        );
        assert_eq!(
            owner.result_state.current().heap_tid(),
            tid(54, 1),
            "active duplicate suppression should keep current-result heap progress aligned with the consumed duplicate heap tid"
        );
        assert_eq!(
            owner.parallel_owned_output_blocker,
            None,
            "active duplicate suppression should clear the transient blocker before retrying the shared seam"
        );
        assert_eq!(
            owner.retained_parallel_owned_output_blocker,
            None,
            "active duplicate suppression should not retain a blocker once the duplicate heap tid has been consumed"
        );
    }

    #[test]
    fn blocked_parallel_scan_disposition_drops_after_consuming_last_live_foreign_duplicate_heap_tid(
    ) {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 320],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 320] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 2,
            )
        }
        .expect("parallel scan target should initialize");

        let mut owner_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut foreign_scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut owner = TqScanOpaque::default();
        let mut foreign = TqScanOpaque::default();
        bind_parallel_scan_state(&mut owner_scan_desc, &mut owner);
        bind_parallel_scan_state(&mut foreign_scan_desc, &mut foreign);

        foreign.result_state.set_current(tid(56, 1), -9.0);
        foreign.result_state.store_pending(&[tid(57, 1)]);
        publish_parallel_scan_worker_slot_snapshot(&foreign);
        let selected = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_selected_pending_output_snapshot(
                owner.parallel_scan_state,
            )
        }
        .expect("parallel selected snapshot should read back")
        .expect("foreign worker should seed the shared selected pending output");

        owner.result_state.set_current(tid(58, 1), -8.0);
        owner.result_state.store_pending(&[tid(57, 1)]);
        owner.parallel_owned_output_blocker = Some(
            crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                slot_index: selected.coordinator.selected_result_slot_index,
                generation: selected.coordinator.result_publish_generation,
                element_tid: selected.selected_result_slot.runtime.element_tid,
            },
        );

        assert_eq!(
            blocked_parallel_scan_disposition(
                &mut owner,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                    slot_index: selected.coordinator.selected_result_slot_index,
                    generation: selected.coordinator.result_publish_generation,
                    element_tid: selected.selected_result_slot.runtime.element_tid,
                }
            ),
            BlockedParallelScanDisposition::DropAndContinue,
            "consuming the last live foreign-owned duplicate heap tid should drop the exhausted local row instead of retaining a blocker"
        );
        assert!(
            !owner.result_state.current().has_element(),
            "active duplicate suppression should clear the exhausted local row once no pending heap tids remain"
        );
        assert_eq!(
            owner.result_state.pending_count(),
            0,
            "active duplicate suppression should fully drain the pending heap tid buffer when the duplicate was the last local output"
        );
        assert_eq!(
            owner.parallel_owned_output_blocker, None,
            "drop-after-duplicate suppression should clear the transient blocker"
        );
        assert_eq!(
            owner.retained_parallel_owned_output_blocker,
            None,
            "drop-after-duplicate suppression should not retain a blocker once the local row is exhausted"
        );
    }

    #[test]
    fn blocked_parallel_scan_disposition_drops_foreign_duplicate_element() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(44, 1), -5.0);

        assert_eq!(
            blocked_parallel_scan_disposition(
                &mut opaque,
                crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                    kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignAdmittedHead,
                    slot_index: Some(2),
                    generation: 11,
                    element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                        block_number: 44,
                        offset_number: 1,
                    },
                }
            ),
            BlockedParallelScanDisposition::DropAndContinue,
            "a foreign owner that already holds the same element should suppress local duplicate emit instead of falling back to local-only output"
        );
    }

    #[test]
    fn stash_active_parallel_blocked_output_hides_active_row_and_preserves_stage() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(53, 1), -6.0);
        opaque.result_state.store_pending(&[tid(54, 1), tid(54, 2)]);
        opaque.retained_parallel_owned_output_blocker = Some(RetainedParallelOwnedOutputBlocker {
            blocker: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlocker {
                kind: crate::am::ec_hnsw::parallel::EcParallelOwnedOutputBlockerKind::ForeignSelectedPending,
                slot_index: Some(2),
                generation: 11,
                element_tid: crate::am::ec_hnsw::parallel::EcParallelItemPointer {
                    block_number: 99,
                    offset_number: 1,
                },
            },
            element_tid: tid(53, 1),
        });

        assert!(
            stash_active_parallel_blocked_output(&mut opaque),
            "a staged local row with pending heap tids should move into the deferred blocked-output stash"
        );
        assert!(
            !opaque.result_state.current().has_element(),
            "stashing a blocked local row should clear the active scan result state"
        );
        assert_eq!(
            opaque.deferred_parallel_blocked_results.len(),
            1,
            "the blocked local row should be retained in the hidden deferred-output stash"
        );
        assert!(
            staged_or_emitted_contains_element(&opaque, tid(53, 1)),
            "a deferred blocked local row should still count as staged so the scan does not rematerialize the same element"
        );
        assert_eq!(
            opaque.deferred_parallel_blocked_results[0]
                .retained_blocker
                .map(|retained| retained.element_tid),
            Some(tid(53, 1)),
            "stashing a blocked local row should preserve per-row blocker metadata inside the deferred stash"
        );
        assert_eq!(
            opaque.retained_parallel_owned_output_blocker,
            None,
            "stashing a blocked local row should clear the active retained blocker because deferred rows now own that metadata"
        );
    }

    #[test]
    fn staged_or_emitted_contains_element_includes_deferred_blocked_rows() {
        let mut opaque = TqScanOpaque::default();
        let mut deferred = ScanResultState::default();
        deferred.set_current(tid(60, 1), -7.0);
        deferred.store_pending(&[tid(61, 1)]);
        opaque
            .deferred_parallel_blocked_results
            .push(DeferredParallelBlockedOutput {
                source_phase: ScanExecutionPhase::GraphTraversal,
                state: deferred,
                retained_blocker: None,
            });

        assert!(
            staged_or_emitted_contains_element(&opaque, tid(60, 1)),
            "elements hidden in the deferred blocked-output stash should still count as staged"
        );
    }

    #[test]
    fn discard_active_parallel_scan_output_clears_fallback_state_and_snapshot() {
        #[repr(C, align(8))]
        struct TestParallelScanStorage {
            bytes: [u8; 256],
        }

        let mut storage = TestParallelScanStorage { bytes: [0; 256] };
        let parallel_scan = storage
            .bytes
            .as_mut_ptr()
            .cast::<pg_sys::ParallelIndexScanDescData>();
        #[cfg(feature = "pg17")]
        unsafe {
            (*parallel_scan).ps_offset = 64;
        }
        #[cfg(feature = "pg18")]
        unsafe {
            (*parallel_scan).ps_offset_am = 64;
        }

        let target = unsafe { storage.bytes.as_mut_ptr().add(64) }.cast::<std::ffi::c_void>();
        unsafe {
            crate::am::ec_hnsw::parallel::initialize_parallel_scan_target_with_worker_slots(
                target, 1,
            )
        }
        .expect("parallel scan target should initialize");

        let mut scan_desc = pg_sys::IndexScanDescData {
            parallel_scan,
            ..Default::default()
        };
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };
        bind_parallel_scan_state(&mut scan_desc, &mut opaque);
        opaque.fallback_result_state.materialize_with_details(
            tid(41, 1),
            -12.0,
            None,
            None,
            Some(-12.5),
            &[tid(42, 1)],
        );
        publish_parallel_scan_worker_slot_snapshot(&opaque);

        discard_active_parallel_scan_output(&mut opaque);

        assert_eq!(
            opaque.fallback_result_state.current().element_tid(),
            page::ItemPointer::INVALID,
            "discarding the active parallel output should clear the local fallback current result"
        );
        assert_eq!(
            opaque.fallback_result_state.pending_count(),
            0,
            "discarding the active parallel output should clear local duplicate drain state"
        );

        let result_snapshot = unsafe {
            crate::am::ec_hnsw::parallel::read_parallel_scan_coordinator_result_slot_snapshot(
                opaque.parallel_scan_state,
                opaque.parallel_scan_worker_slot_index,
            )
        }
        .expect("parallel coordinator result-slot snapshot should read back");
        assert_eq!(
            result_snapshot.runtime.element_tid,
            crate::am::ec_hnsw::parallel::EcParallelItemPointer::INVALID,
            "discarding the active parallel output should clear the shared staged element snapshot too"
        );
        assert_eq!(
            result_snapshot.runtime.pending_count, 0,
            "discarding the active parallel output should clear shared pending duplicate state too"
        );
    }

    #[test]
    fn scan_result_state_clear_clears_pending_heap_tid_drain() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(26, 1), -4.0);
        opaque.result_state.store_pending(&[tid(31, 1), tid(31, 2)]);

        opaque.result_state.clear();

        assert!(
            !opaque.result_state.current().has_element(),
            "clearing scan result state should also clear the current result slot"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "clearing scan result state should clear any pending duplicate drain state"
        );
        assert_eq!(
            opaque.result_state.pending_index(),
            0,
            "clearing scan result state should reset duplicate drain progress"
        );
        assert_eq!(
            opaque
                .result_state
                .pending_heap_tids()
                .first()
                .copied()
                .unwrap_or(page::ItemPointer::INVALID),
            page::ItemPointer::INVALID,
            "clearing scan result state should wipe the pending heap-tid buffer too"
        );
        assert!(
            opaque.result_state.pending_heap_tids().is_empty(),
            "clearing scan result state should expose no pending heap tids after reset"
        );
    }

    #[test]
    fn seed_scan_result_state_seeds_current_result_and_pending_drain() {
        let mut opaque = TqScanOpaque::default();

        seed_scan_result_state(
            &mut opaque,
            SelectedScanResult {
                element_tid: tid(26, 1),
                score: -4.5,
                approx_score: None,
                approx_rank_base: None,
                comparison_score: None,
                heap_tids: CachedHeapTids::from_iter([tid(31, 1), tid(31, 2)]),
            },
        );

        assert_eq!(
            opaque.result_state.current().element_tid(),
            tid(26, 1),
            "shared result materialization should record the element tid on current-result state"
        );
        assert_eq!(
            opaque.result_state.current().score(),
            -4.5,
            "shared result materialization should preserve the supplied score"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            2,
            "shared result materialization should seed pending duplicate drain"
        );
        assert_eq!(
            opaque.result_state.pending_heap_tids()[0],
            tid(31, 1),
            "shared result materialization should preserve heap-tid order for later drain"
        );
        assert_eq!(
            opaque.result_state.pending_heap_tids()[1],
            tid(31, 2),
            "shared result materialization should retain all supplied heap tids"
        );
        assert!(
            !opaque.result_state.current().comparison_score_valid(),
            "plain result materialization should leave comparison-score state empty by default"
        );
    }

    #[test]
    fn scan_result_state_comparison_score_tracks_current_result_lifecycle() {
        let mut state = ScanResultState::default();
        state.materialize_from_parts(tid(42, 1), -7.0, &[tid(43, 1), tid(43, 2)]);

        assert!(
            !state.current().comparison_score_valid(),
            "materializing a result should not implicitly mark comparison score valid"
        );

        state.set_current_comparison_score(-6.5);

        assert!(state.current().comparison_score_valid());
        assert_eq!(state.current().comparison_score(), -6.5);

        state.clear_current();

        assert!(
            !state.current().comparison_score_valid(),
            "clearing current-result state should also clear any grouped rerank comparison score"
        );
    }

    #[test]
    fn prepared_query_cache_lifetime_tracks_scan_state() {
        let metadata = page::MetadataPage::current_v1_scalar(page::CurrentFormatMetadata {
            m: 8,
            ef_construction: 32,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 4,
            bits: 4,
            max_level: 0,
            seed: 42,
            inserted_since_rebuild: 0,
            persisted_binary_sidecar: false,
        });
        let query = [1.0_f32, 2.0, 3.0, 4.0];
        let mut opaque = TqScanOpaque::default();

        store_scan_prepared_query(&mut opaque, &query, &metadata);

        assert!(
            !opaque.prepared_query.is_null(),
            "storing a prepared query should retain the prepared-query payload"
        );
        assert!(
            opaque.grouped_query.is_null(),
            "scalar scan state should not allocate grouped query preparation"
        );
        assert!(
            !opaque.cached_quantizer.is_null(),
            "storing a prepared query should retain the quantizer used to score future elements"
        );

        free_scan_prepared_query(&mut opaque);

        assert!(
            opaque.prepared_query.is_null(),
            "freeing scan prepared-query state should release the prepared query payload"
        );
        assert!(
            opaque.grouped_query.is_null(),
            "freeing scan prepared-query state should release grouped query preparation too"
        );
        assert!(
            opaque.cached_quantizer.is_null(),
            "freeing scan prepared-query state should release the cached quantizer too"
        );
    }

    #[test]
    fn cached_graph_element_from_grouped_tuple_ref_keeps_grouped_hot_payloads() {
        let tuple = page::TqGroupedHotTuple {
            level: 2,
            deleted: false,
            heaptids: vec![tid(9, 1), tid(9, 2)],
            neighbortid: tid(5, 4),
            reranktid: tid(5, 5),
            binary_words: vec![0x55AA55AA55AA55AA],
            search_code: vec![0x21, 0x43],
        };
        let encoded = tuple.encode().unwrap();
        let tuple_ref = graph::GraphTupleRef::GroupedHot(
            page::TqGroupedHotTupleRef::decode(&encoded, 1, 2).unwrap(),
        );

        let cached = CachedGraphElement::from_graph_tuple_ref(
            tid(7, 3),
            tuple_ref,
            CachedBinaryWords::from_vec(tuple.binary_words.clone()),
        );

        assert_eq!(cached.tid, tid(7, 3));
        assert_eq!(cached.level, tuple.level);
        assert!(!cached.deleted);
        assert_eq!(cached.heaptids.as_slice(), tuple.heaptids.as_slice());
        assert_eq!(cached.neighbortid, tuple.neighbortid);
        assert_eq!(cached.reranktid, Some(tuple.reranktid));
        assert_eq!(
            cached.binary_words.as_slice(),
            tuple.binary_words.as_slice()
        );
        assert_eq!(
            cached.grouped_search_code.as_slice(),
            Some(tuple.search_code.as_slice())
        );
    }

    #[test]
    fn cached_graph_element_from_scalar_tuple_ref_has_no_grouped_hot_payloads() {
        let tuple = page::TqElementTuple {
            level: 1,
            deleted: false,
            heaptids: vec![tid(4, 1)],
            gamma: 1.25,
            neighbortid: tid(4, 2),
            code: vec![0x11, 0x22, 0x33, 0x44],
            binary_words: vec![0xA5A5A5A5A5A5A5A5],
        };
        let encoded = tuple.encode().unwrap();
        let tuple_ref = graph::GraphTupleRef::Scalar(
            page::TqElementTupleRef::decode(&encoded, tuple.code.len()).unwrap(),
        );

        let cached = CachedGraphElement::from_graph_tuple_ref(
            tid(8, 3),
            tuple_ref,
            CachedBinaryWords::from_vec(tuple.binary_words.clone()),
        );

        assert_eq!(cached.reranktid, None);
        assert_eq!(cached.grouped_search_code.as_slice(), None);
        assert_eq!(cached.grouped_score_input(), None);
    }

    #[test]
    fn grouped_score_input_uses_cached_grouped_hot_payloads() {
        let tuple = page::TqGroupedHotTuple {
            level: 3,
            deleted: false,
            heaptids: vec![tid(11, 1)],
            neighbortid: tid(11, 2),
            reranktid: tid(11, 3),
            binary_words: vec![0x0123_4567_89AB_CDEF, 0x0FED_CBA9_7654_3210],
            search_code: vec![0x10, 0x32, 0x54],
        };
        let encoded = tuple.encode().unwrap();
        let tuple_ref = graph::GraphTupleRef::GroupedHot(
            page::TqGroupedHotTupleRef::decode(&encoded, 2, 3).unwrap(),
        );

        let cached = CachedGraphElement::from_graph_tuple_ref(
            tid(11, 4),
            tuple_ref,
            CachedBinaryWords::from_vec(tuple.binary_words.clone()),
        );
        let grouped = cached
            .grouped_score_input()
            .expect("grouped hot tuples should expose grouped score input from cached payloads");

        assert_eq!(grouped.reranktid, tuple.reranktid);
        assert_eq!(grouped.search_code, tuple.search_code.as_slice());
        assert_eq!(grouped.binary_words, tuple.binary_words.as_slice());
    }

    #[test]
    fn grouped_score_shape_uses_grouped_scan_layout() {
        let shape = GroupedScoreShape::from_scan_graph_storage(
            graph::GraphStorageDescriptor::PqFastScan(graph::PqFastScanLayout {
                binary_word_count: 24,
                search_code_len: 48,
                rerank_code_len: 768,
            }),
        )
        .expect("grouped scan storage should produce grouped score shape");

        assert_eq!(
            shape,
            GroupedScoreShape {
                binary_word_count: 24,
                search_code_len: 48,
                rerank_code_len: 768,
            }
        );
    }

    #[test]
    fn grouped_score_context_uses_scan_shape_and_cached_payloads() {
        let tuple = page::TqGroupedHotTuple {
            level: 2,
            deleted: false,
            heaptids: vec![tid(15, 1)],
            neighbortid: tid(15, 2),
            reranktid: tid(15, 3),
            binary_words: vec![0x1234_5678_9ABC_DEF0, 0x0FED_CBA9_8765_4321],
            search_code: vec![0x9A, 0xBC, 0xDE],
        };
        let encoded = tuple.encode().unwrap();
        let tuple_ref = graph::GraphTupleRef::GroupedHot(
            page::TqGroupedHotTupleRef::decode(&encoded, 2, 3).unwrap(),
        );
        let cached = CachedGraphElement::from_graph_tuple_ref(
            tid(15, 4),
            tuple_ref,
            CachedBinaryWords::from_vec(tuple.binary_words.clone()),
        );

        let context = grouped_score_context_from_scan_state(
            graph::GraphStorageDescriptor::PqFastScan(graph::PqFastScanLayout {
                binary_word_count: 2,
                search_code_len: 3,
                rerank_code_len: 96,
            }),
            &cached,
        )
        .expect(
            "grouped scan state and grouped cached element should produce grouped score context",
        );

        assert_eq!(context.element_tid, tid(15, 4));
        assert_eq!(
            context.call.shape,
            GroupedScoreShape {
                binary_word_count: 2,
                search_code_len: 3,
                rerank_code_len: 96,
            }
        );
        assert_eq!(context.call.input.reranktid, tuple.reranktid);
        assert_eq!(context.call.input.search_code, tuple.search_code.as_slice());
        assert_eq!(
            context.call.input.binary_words,
            tuple.binary_words.as_slice()
        );
    }

    #[test]
    fn grouped_score_context_requires_grouped_scan_storage() {
        let tuple = page::TqGroupedHotTuple {
            level: 1,
            deleted: false,
            heaptids: vec![tid(16, 1)],
            neighbortid: tid(16, 2),
            reranktid: tid(16, 3),
            binary_words: vec![0x0123_4567_89AB_CDEF],
            search_code: vec![0x21, 0x43],
        };
        let encoded = tuple.encode().unwrap();
        let tuple_ref = graph::GraphTupleRef::GroupedHot(
            page::TqGroupedHotTupleRef::decode(&encoded, 1, 2).unwrap(),
        );
        let cached = CachedGraphElement::from_graph_tuple_ref(
            tid(16, 4),
            tuple_ref,
            CachedBinaryWords::from_vec(tuple.binary_words.clone()),
        );

        assert_eq!(
            grouped_score_context_from_scan_state(
                graph::GraphStorageDescriptor::TurboQuant { code_len: 4 },
                &cached,
            ),
            None
        );
    }

    #[test]
    fn grouped_exact_traversal_mode_enables_every_layer_in_all_scope() {
        assert!(grouped_exact_traversal_enabled_for_layer(
            GroupedExactTraversalMode::AllLayers,
            0
        ));
        assert!(grouped_exact_traversal_enabled_for_layer(
            GroupedExactTraversalMode::AllLayers,
            3
        ));
    }

    #[test]
    fn grouped_exact_traversal_mode_limits_layer0_scope_to_layer0() {
        assert!(grouped_exact_traversal_enabled_for_layer(
            GroupedExactTraversalMode::Layer0Only,
            0
        ));
        assert!(!grouped_exact_traversal_enabled_for_layer(
            GroupedExactTraversalMode::Layer0Only,
            1
        ));
        assert!(!grouped_exact_traversal_enabled_for_layer(
            GroupedExactTraversalMode::Disabled,
            0
        ));
    }

    #[test]
    fn grouped_exact_traversal_candidate_budget_requires_enabled_layer() {
        let mut opaque = TqScanOpaque {
            grouped_exact_traversal_mode: GroupedExactTraversalMode::AllLayers,
            grouped_exact_traversal_strategy: GroupedExactTraversalStrategy::Expansion,
            grouped_exact_traversal_limit: 4,
            ..TqScanOpaque::default()
        };
        assert_eq!(
            grouped_exact_traversal_candidate_budget_for_layer(&opaque, 0),
            Some(4)
        );
        opaque.grouped_exact_traversal_mode = GroupedExactTraversalMode::Layer0Only;
        assert_eq!(
            grouped_exact_traversal_candidate_budget_for_layer(&opaque, 0),
            Some(4)
        );
        assert_eq!(
            grouped_exact_traversal_candidate_budget_for_layer(&opaque, 1),
            None
        );
        opaque.grouped_exact_traversal_limit = 0;
        assert_eq!(
            grouped_exact_traversal_candidate_budget_for_layer(&opaque, 0),
            None
        );
        opaque.grouped_exact_traversal_strategy = GroupedExactTraversalStrategy::FrontierHead;
        opaque.grouped_exact_traversal_limit = 4;
        assert_eq!(
            grouped_exact_traversal_candidate_budget_for_layer(&opaque, 0),
            None
        );
    }

    #[test]
    fn grouped_exact_traversal_full_candidate_scoring_requires_expansion_strategy_without_limit() {
        let mut opaque = TqScanOpaque {
            grouped_exact_traversal_mode: GroupedExactTraversalMode::AllLayers,
            grouped_exact_traversal_strategy: GroupedExactTraversalStrategy::Expansion,
            grouped_exact_traversal_limit: 0,
            ..TqScanOpaque::default()
        };
        assert!(grouped_exact_traversal_full_candidate_scoring_for_layer(
            &opaque, 0
        ));
        opaque.grouped_exact_traversal_limit = 2;
        assert!(!grouped_exact_traversal_full_candidate_scoring_for_layer(
            &opaque, 0
        ));
        opaque.grouped_exact_traversal_limit = 0;
        opaque.grouped_exact_traversal_strategy = GroupedExactTraversalStrategy::FrontierHead;
        assert!(!grouped_exact_traversal_full_candidate_scoring_for_layer(
            &opaque, 0
        ));
    }

    #[test]
    fn grouped_exact_traversal_candidate_indices_pick_lowest_scores_stably() {
        let candidate = |ordinal| {
            Arc::new(CachedGraphElement {
                tid: tid(90, ordinal),
                level: 0,
                deleted: false,
                heaptids: CachedHeapTids::default(),
                neighbortid: page::ItemPointer::INVALID,
                reranktid: None,
                binary_words: CachedBinaryWords::empty(),
                grouped_search_code: CachedGroupedSearchCode::None,
            })
        };
        let candidates = vec![
            GroupedTraversalCandidate {
                ordinal: 3,
                element: candidate(3),
                approx_score: -2.0,
            },
            GroupedTraversalCandidate {
                ordinal: 1,
                element: candidate(1),
                approx_score: -4.0,
            },
            GroupedTraversalCandidate {
                ordinal: 2,
                element: candidate(2),
                approx_score: -4.0,
            },
        ];

        assert_eq!(
            grouped_exact_traversal_candidate_indices(&candidates, 2),
            vec![1, 2]
        );
        assert_eq!(
            grouped_exact_traversal_candidate_indices(&candidates, 99),
            vec![1, 2, 0]
        );
    }

    #[test]
    fn candidate_score_dispatch_uses_grouped_input_for_exact_unavailable() {
        let tuple = page::TqGroupedHotTuple {
            level: 1,
            deleted: false,
            heaptids: vec![tid(12, 1)],
            neighbortid: tid(12, 2),
            reranktid: tid(12, 3),
            binary_words: vec![0xAABBCCDD00112233],
            search_code: vec![0xAB, 0xCD],
        };
        let encoded = tuple.encode().unwrap();
        let tuple_ref = graph::GraphTupleRef::GroupedHot(
            page::TqGroupedHotTupleRef::decode(&encoded, 1, 2).unwrap(),
        );
        let cached = CachedGraphElement::from_graph_tuple_ref(
            tid(12, 4),
            tuple_ref,
            CachedBinaryWords::from_vec(tuple.binary_words.clone()),
        );

        match candidate_score_dispatch(
            graph::GraphStorageDescriptor::PqFastScan(graph::PqFastScanLayout {
                binary_word_count: 1,
                search_code_len: 2,
                rerank_code_len: 96,
            }),
            &cached,
            LoadedElementState::ExactUnavailable,
        ) {
            CandidateScoreDispatch::Grouped(grouped) => {
                assert_eq!(grouped.element_tid, tid(12, 4));
                assert_eq!(
                    grouped.call.shape,
                    GroupedScoreShape {
                        binary_word_count: 1,
                        search_code_len: 2,
                        rerank_code_len: 96,
                    }
                );
                assert_eq!(grouped.call.input.reranktid, tuple.reranktid);
                assert_eq!(grouped.call.input.search_code, tuple.search_code.as_slice());
                assert_eq!(
                    grouped.call.input.binary_words,
                    tuple.binary_words.as_slice()
                );
            }
            CandidateScoreDispatch::Exact(_) => {
                panic!("exact-unavailable grouped tuples should dispatch through grouped input")
            }
        }
    }

    #[test]
    fn candidate_score_dispatch_uses_grouped_input_for_cached_grouped_hit() {
        let tuple = page::TqGroupedHotTuple {
            level: 1,
            deleted: false,
            heaptids: vec![tid(12, 1)],
            neighbortid: tid(12, 2),
            reranktid: tid(12, 3),
            binary_words: vec![0xAABBCCDD00112233],
            search_code: vec![0xAB, 0xCD],
        };
        let encoded = tuple.encode().unwrap();
        let tuple_ref = graph::GraphTupleRef::GroupedHot(
            page::TqGroupedHotTupleRef::decode(&encoded, 1, 2).unwrap(),
        );
        let cached = CachedGraphElement::from_graph_tuple_ref(
            tid(12, 4),
            tuple_ref,
            CachedBinaryWords::from_vec(tuple.binary_words.clone()),
        );

        match candidate_score_dispatch(
            graph::GraphStorageDescriptor::PqFastScan(graph::PqFastScanLayout {
                binary_word_count: 1,
                search_code_len: 2,
                rerank_code_len: 96,
            }),
            &cached,
            LoadedElementState::None,
        ) {
            CandidateScoreDispatch::Grouped(grouped) => {
                assert_eq!(grouped.element_tid, tid(12, 4));
                assert_eq!(grouped.call.input.reranktid, tuple.reranktid);
                assert_eq!(grouped.call.input.search_code, tuple.search_code.as_slice());
            }
            CandidateScoreDispatch::Exact(_) => {
                panic!("cached grouped tuples should keep grouped score dispatch on cache hits")
            }
        }
    }

    #[test]
    fn grouped_score_payload_view_preserves_context_fields() {
        let grouped = GroupedScoreContext {
            element_tid: tid(20, 4),
            call: GroupedScoreCall {
                shape: GroupedScoreShape {
                    binary_word_count: 2,
                    search_code_len: 3,
                    rerank_code_len: 96,
                },
                input: GroupedScoreInput {
                    reranktid: tid(20, 3),
                    binary_words: &[0x0123_4567_89AB_CDEF, 0x0FED_CBA9_7654_3210],
                    search_code: &[0x10, 0x32, 0x54],
                },
            },
        };

        let payload = grouped_score_payload_view(grouped)
            .expect("metadata-aligned grouped context should produce grouped payload view");

        assert_eq!(payload.element_tid, tid(20, 4));
        assert_eq!(payload.reranktid, tid(20, 3));
        assert_eq!(
            payload.binary_words,
            &[0x0123_4567_89AB_CDEF, 0x0FED_CBA9_7654_3210]
        );
        assert_eq!(payload.search_code, &[0x10, 0x32, 0x54]);
        assert_eq!(payload.rerank_code_len, 96);
    }

    #[test]
    fn grouped_score_search_code_preserves_metadata_aligned_search_codes() {
        let grouped = GroupedScoreContext {
            element_tid: tid(20, 4),
            call: GroupedScoreCall {
                shape: GroupedScoreShape {
                    binary_word_count: 2,
                    search_code_len: 3,
                    rerank_code_len: 96,
                },
                input: GroupedScoreInput {
                    reranktid: tid(20, 3),
                    binary_words: &[0x0123_4567_89AB_CDEF, 0x0FED_CBA9_7654_3210],
                    search_code: &[0x10, 0x32, 0x54],
                },
            },
        };

        assert_eq!(
            grouped_score_search_code(grouped),
            Some(&[0x10, 0x32, 0x54][..])
        );
    }

    #[test]
    fn grouped_score_payload_view_rejects_shape_mismatch() {
        let grouped = GroupedScoreContext {
            element_tid: tid(21, 4),
            call: GroupedScoreCall {
                shape: GroupedScoreShape {
                    binary_word_count: 2,
                    search_code_len: 4,
                    rerank_code_len: 96,
                },
                input: GroupedScoreInput {
                    reranktid: tid(21, 3),
                    binary_words: &[0x0123_4567_89AB_CDEF],
                    search_code: &[0x10, 0x32, 0x54],
                },
            },
        };

        assert_eq!(grouped_score_payload_view(grouped), None);
    }

    #[test]
    fn grouped_score_search_code_rejects_search_code_shape_mismatch() {
        let grouped = GroupedScoreContext {
            element_tid: tid(21, 4),
            call: GroupedScoreCall {
                shape: GroupedScoreShape {
                    binary_word_count: 2,
                    search_code_len: 4,
                    rerank_code_len: 96,
                },
                input: GroupedScoreInput {
                    reranktid: tid(21, 3),
                    binary_words: &[0x0123_4567_89AB_CDEF],
                    search_code: &[0x10, 0x32, 0x54],
                },
            },
        };

        assert_eq!(grouped_score_search_code(grouped), None);
    }

    #[test]
    fn grouped_score_rerank_payload_preserves_hot_and_cold_fields() {
        let payload = GroupedScorePayloadView {
            element_tid: tid(30, 4),
            reranktid: tid(30, 3),
            binary_words: &[0x0123_4567_89AB_CDEF],
            search_code: &[0x10, 0x32, 0x54],
            rerank_code_len: 4,
        };
        let rerank = graph::GroupedRerankPayload {
            tid: tid(30, 3),
            gamma: 0.75,
            code: vec![0xAA, 0xBB, 0xCC, 0xDD],
        };

        let merged = grouped_score_rerank_payload(payload, rerank)
            .expect("matching rerank tuple should compose with grouped hot payload");

        assert_eq!(merged.element_tid, tid(30, 4));
        assert_eq!(merged.reranktid, tid(30, 3));
        assert_eq!(merged.binary_words, &[0x0123_4567_89AB_CDEF]);
        assert_eq!(merged.search_code, &[0x10, 0x32, 0x54]);
        assert_eq!(merged.rerank_gamma, 0.75);
        assert_eq!(merged.rerank_code, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn grouped_score_rerank_payload_rejects_mismatched_cold_payload() {
        let payload = GroupedScorePayloadView {
            element_tid: tid(31, 4),
            reranktid: tid(31, 3),
            binary_words: &[0x0123_4567_89AB_CDEF],
            search_code: &[0x10, 0x32, 0x54],
            rerank_code_len: 4,
        };

        assert_eq!(
            grouped_score_rerank_payload(
                payload,
                graph::GroupedRerankPayload {
                    tid: tid(31, 5),
                    gamma: 0.5,
                    code: vec![0xAA, 0xBB, 0xCC, 0xDD],
                }
            ),
            None
        );
        assert_eq!(
            grouped_score_rerank_payload(
                payload,
                graph::GroupedRerankPayload {
                    tid: tid(31, 3),
                    gamma: 0.5,
                    code: vec![0xAA, 0xBB],
                }
            ),
            None
        );
    }

    #[test]
    fn score_grouped_rerank_payload_result_matches_prod_quantizer_path() {
        let vector = vec![0.1_f32, -0.2, 0.3, -0.4];
        let query = vec![0.25_f32, 0.5, -0.75, 0.125];
        let quantizer = ProdQuantizer::new(vector.len(), 4, 42);
        let encoded = quantizer.encode(&vector);
        let prepared = quantizer.prepare_ip_query(&query);
        let mut code_bytes = encoded.mse_packed.clone();
        code_bytes.extend_from_slice(&encoded.qjl_packed);
        let payload = GroupedScoreRerankPayload {
            element_tid: tid(40, 4),
            reranktid: tid(40, 3),
            binary_words: &[],
            search_code: &[],
            rerank_gamma: encoded.gamma,
            rerank_code: code_bytes,
        };

        let observed = score_grouped_rerank_payload_result(&quantizer, &prepared, &payload);
        let expected =
            -quantizer.score_ip_from_parts(&prepared, encoded.gamma, &payload.rerank_code);

        assert_eq!(observed, expected);
    }

    #[test]
    fn miri_score_scan_element_result_via_raw_opaque_ptr_updates_stats_delta() {
        let vector = vec![0.1_f32];
        let query = vec![0.25_f32];
        let quantizer = Arc::new(ProdQuantizer::new(vector.len(), 4, 42));
        let prepared_query = Box::new(quantizer.prepare_ip_query(&query));
        let encoded = quantizer.encode(&vector);
        let mut code_bytes = encoded.mse_packed.clone();
        code_bytes.extend_from_slice(&encoded.qjl_packed);
        let cached_quantizer = Arc::into_raw(quantizer);
        let prepared_query = Box::into_raw(prepared_query);

        let mut opaque = TqScanOpaque {
            cached_quantizer,
            prepared_query,
            ..TqScanOpaque::default()
        };
        let opaque_ptr = &mut opaque as *mut TqScanOpaque;

        let score =
            unsafe { score_scan_element_result(&mut *opaque_ptr, encoded.gamma, &code_bytes) };

        assert!(score.is_finite());
        assert_eq!(unsafe { (*opaque_ptr).stats_delta.total_distance_calcs }, 1);

        free_scan_prepared_query(&mut opaque);
    }

    #[test]
    fn build_prepared_grouped_scan_query_uses_persisted_codebooks() {
        let prepared = PreparedQuery {
            lut: Vec::new(),
            rotated: vec![1.0_f32, 2.0, 3.0, 4.0],
            sq: Vec::new(),
            qjl_scale: 0.0,
        };
        let model = graph::GroupedCodebookModel {
            head_tid: page::ItemPointer::INVALID,
            group_count: 2,
            group_size: 2,
            flat_codebooks: vec![
                1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0, 5.0, 0.0, 6.0, 0.0, 7.0, 0.0, 8.0, 0.0,
                9.0, 0.0, 10.0, 0.0, 11.0, 0.0, 12.0, 0.0, 13.0, 0.0, 14.0, 0.0, 15.0, 0.0, 16.0,
                0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0, 5.0, 0.0, 6.0, 0.0, 7.0, 0.0,
                8.0, 0.0, 9.0, 0.0, 10.0, 0.0, 11.0, 0.0, 12.0, 0.0, 13.0, 0.0, 14.0, 0.0, 15.0,
                0.0, 16.0,
            ],
        };

        let grouped = build_prepared_grouped_scan_query(&prepared, &model);

        assert_eq!(grouped.group_count, 2);
        assert_eq!(grouped.search_code_len, 1);
        assert_eq!(grouped.lut_f32.len(), 32);
        assert_eq!(grouped.lut_f32[0], 1.0);
        assert_eq!(grouped.lut_f32[1], 2.0);
        assert_eq!(grouped.lut_f32[15], 16.0);
        assert_eq!(grouped.lut_f32[16], 4.0);
        assert_eq!(grouped.lut_f32[17], 8.0);
        assert_eq!(grouped.lut_f32[31], 64.0);
    }

    #[test]
    fn score_grouped_search_code_result_negates_shared_grouped_pq_score() {
        let prepared = PreparedGroupedScanQuery {
            group_count: 3,
            search_code_len: 2,
            lut_f32: {
                let mut lut = vec![0.0_f32; 3 * 16];
                lut[1] = 1.5;
                lut[16 + 3] = -0.25;
                lut[32 + 2] = 2.0;
                lut
            },
        };
        let search_code = &[0x31, 0x02];

        let observed = score_grouped_search_code_result(&prepared, search_code);
        let expected = -crate::quant::grouped_pq::grouped_pq_score_f32(
            &prepared.lut_f32,
            prepared.group_count,
            search_code,
        );

        assert_eq!(observed, expected);
        assert_eq!(observed, -3.25);
    }

    #[test]
    fn candidate_score_dispatch_keeps_scalar_loaded_state_exact() {
        let tuple = page::TqElementTuple {
            level: 1,
            deleted: false,
            heaptids: vec![tid(13, 1)],
            gamma: 0.75,
            neighbortid: tid(13, 2),
            code: vec![0x01, 0x23, 0x45, 0x67],
            binary_words: vec![0x0102030405060708],
        };
        let encoded = tuple.encode().unwrap();
        let tuple_ref = graph::GraphTupleRef::Scalar(
            page::TqElementTupleRef::decode(&encoded, tuple.code.len()).unwrap(),
        );
        let cached = CachedGraphElement::from_graph_tuple_ref(
            tid(13, 3),
            tuple_ref,
            CachedBinaryWords::from_vec(tuple.binary_words.clone()),
        );

        match candidate_score_dispatch(
            graph::GraphStorageDescriptor::TurboQuant {
                code_len: tuple.code.len(),
            },
            &cached,
            LoadedElementState::None,
        ) {
            CandidateScoreDispatch::Exact(LoadedElementState::None) => {}
            CandidateScoreDispatch::Exact(_) => {
                panic!("scalar fallback dispatch should preserve the original exact loaded state")
            }
            CandidateScoreDispatch::Grouped(_) => {
                panic!("scalar tuples should never dispatch through grouped score input")
            }
        }
    }

    #[test]
    fn validate_runtime_scan_format_accepts_pq_fastscan_metadata() {
        let metadata = page::MetadataPage {
            m: 8,
            ef_construction: 64,
            entry_point: tid(1, 1),
            dimensions: 16,
            bits: 4,
            max_level: 2,
            seed: 42,
            inserted_since_rebuild: 0,
            format_version: page::INDEX_FORMAT_V2_GROUPED,
            transform_kind: page::TransformKind::Srht,
            search_codec_kind: page::SearchCodecKind::GroupedPq,
            payload_flags: page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE
                | page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            search_bits: 4,
            rerank_codec_kind: page::RerankCodecKind::ScalarQuantized,
            search_subvector_count: 1,
            search_subvector_dim: 16,
            grouped_codebook_head: tid(1, 2),
        };

        assert_eq!(
            graph::GraphStorageDescriptor::from_metadata(&metadata).unwrap(),
            graph::GraphStorageDescriptor::PqFastScan(graph::PqFastScanLayout {
                binary_word_count: 0,
                search_code_len: 1,
                rerank_code_len: crate::code_len(16, 4),
            })
        );
    }

    #[test]
    fn consume_candidate_frontier_head_reselects_then_clears() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(7, 1, -2.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(7, 2, 3.5));
        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((7, 1)),
            "frontier head should start at the lower-scoring valid candidate"
        );

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("frontier head consumption should return the current best slot");
        assert_eq!(
            (consumed.node.block_number, consumed.node.offset_number),
            (7, 1),
            "consumption should return the previously best frontier slot"
        );
        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((7, 2)),
            "consuming the best slot should reselect the remaining valid candidate"
        );
        assert!(
            visible_frontier_slot(&opaque, 0).is_some(),
            "consuming the head should keep the remaining candidate valid"
        );
        assert_eq!(
            visible_frontier_slot(&opaque, 0)
                .map(|candidate| candidate.score)
                .unwrap_or(0.0),
            3.5,
            "consuming the head should preserve the remaining candidate after compaction"
        );

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("a remaining valid slot should still be consumable");
        assert_eq!(
            (consumed.node.block_number, consumed.node.offset_number),
            (7, 2),
            "the second consumption should return the reseated head slot"
        );
        assert_eq!(
            candidate_frontier_head(&mut opaque).map(|candidate| candidate.node),
            None,
            "consuming the last valid slot should invalidate the frontier head"
        );
        assert!(
            visible_frontier_candidates(&opaque).is_empty(),
            "consuming both valid slots should leave the candidate vector empty"
        );
        assert!(
            consume_candidate_frontier_head(&mut opaque).is_none(),
            "consuming an empty frontier should stay a no-op"
        );
    }

    #[test]
    fn consuming_frontier_head_forgets_it_from_bootstrap_scheduler() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(13, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(13, 2, -1.0));
        seed_existing_frontier_into_expansion(&mut opaque);

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("frontier head consumption should succeed");
        assert_eq!(
            (consumed.node.block_number, consumed.node.offset_number),
            (13, 1),
            "the lower-score candidate should be consumed first"
        );
        assert_eq!(
            bootstrap_expansion_mut(&mut opaque)
                .peek_best()
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((13, 2)),
            "consuming a frontier head should immediately forget it from the scan-owned scheduler"
        );
    }

    #[test]
    fn current_candidate_frontier_head_tid_prefers_scheduler_best_node() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(14, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(14, 2, -1.0));

        bootstrap_expansion_mut(&mut opaque).seed(search::BeamCandidate::new(
            page::ItemPointer {
                block_number: 14,
                offset_number: 2,
            },
            -1.0,
        ));
        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((14, 2)),
            "frontier-head derivation should prefer the scan-owned scheduler's current best queued node"
        );
    }

    #[test]
    fn current_candidate_frontier_head_tid_falls_back_after_scheduler_drains() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(17, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(17, 2, -1.0));
        seed_existing_frontier_into_expansion(&mut opaque);

        bootstrap_expansion_mut(&mut opaque)
            .expand_one(|_| std::iter::empty::<search::BeamCandidate<page::ItemPointer>>());
        bootstrap_expansion_mut(&mut opaque)
            .expand_one(|_| std::iter::empty::<search::BeamCandidate<page::ItemPointer>>());

        assert!(
            bootstrap_expansion_mut(&mut opaque).peek_best().is_none(),
            "expanding both seeded sources should drain the scheduler while leaving the visible frontier intact"
        );
        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((17, 1)),
            "frontier-head derivation must still fall back to the visible frontier once the scheduler has no queued expansion sources"
        );
    }

    #[test]
    fn consume_candidate_frontier_head_prefers_scheduler_best_node() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(15, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(15, 2, -1.0));

        bootstrap_expansion_mut(&mut opaque).seed(search::BeamCandidate::new(
            page::ItemPointer {
                block_number: 15,
                offset_number: 2,
            },
            -1.0,
        ));

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("frontier consumption should prefer the scheduler's best queued node");
        assert_eq!(
            (consumed.node.block_number, consumed.node.offset_number),
            (15, 2),
            "scheduler-owned best-node selection should override Vec score order during consumption"
        );
        assert_eq!(
            visible_frontier_slot(&opaque, 0).map(|candidate| candidate.node),
            Some(page::ItemPointer {
                block_number: 15,
                offset_number: 1,
            }),
            "consumption should remove the scheduler-selected visible candidate from the compacted frontier"
        );
    }

    #[test]
    fn current_candidate_frontier_head_tid_drops_stale_scheduler_nodes() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(16, 1, -2.0));

        bootstrap_expansion_mut(&mut opaque).seed(search::BeamCandidate::new(
            page::ItemPointer {
                block_number: 16,
                offset_number: 9,
            },
            -3.0,
        ));
        bootstrap_expansion_mut(&mut opaque).seed(search::BeamCandidate::new(
            page::ItemPointer {
                block_number: 16,
                offset_number: 1,
            },
            -2.0,
        ));

        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((16, 1)),
            "stale scheduler nodes should be dropped until the best queued visible frontier node can be mapped"
        );
        assert_eq!(
            bootstrap_expansion_mut(&mut opaque)
                .peek_best()
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((16, 1)),
            "recompute should purge unmappable scheduler nodes instead of leaving them at the head forever"
        );
    }

    #[test]
    fn collect_successor_candidates_skips_invalid_and_collects_multiple() {
        let skipped = page::ItemPointer::INVALID;
        let first_valid = page::ItemPointer {
            block_number: 8,
            offset_number: 1,
        };
        let second_valid = page::ItemPointer {
            block_number: 8,
            offset_number: 2,
        };
        let mut visited = Vec::new();

        let candidates = collect_successor_candidates(
            &[skipped, first_valid, second_valid],
            2,
            |neighbor_tid| {
                visited.push((neighbor_tid.block_number, neighbor_tid.offset_number));

                Some(search::BeamCandidate::new(neighbor_tid, 2.5))
            },
        );

        assert_eq!(
            visited,
            vec![
                (first_valid.block_number, first_valid.offset_number),
                (second_valid.block_number, second_valid.offset_number)
            ],
            "collection should skip INVALID neighbors and continue through live candidates in order"
        );
        assert_eq!(
            candidates
                .into_iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![first_valid, second_valid],
            "collection should return live candidates in neighbor order up to the requested limit"
        );
    }

    #[test]
    fn fill_bootstrap_frontier_can_expand_beyond_entry_neighbors() {
        let entry_tid = page::ItemPointer {
            block_number: 9,
            offset_number: 1,
        };
        let child_tid = page::ItemPointer {
            block_number: 9,
            offset_number: 2,
        };
        let grandchild_tid = page::ItemPointer {
            block_number: 9,
            offset_number: 3,
        };
        let mut opaque = TqScanOpaque::default();
        visible_frontier_mut(&mut opaque).push(beam_candidate(9, 1, -3.0));

        fill_bootstrap_frontier(
            &mut opaque,
            3,
            BootstrapExpandPolicy::ScoreOrder,
            |source_tid, opaque| match (source_tid.block_number, source_tid.offset_number) {
                (9, 1) => {
                    seed_discovered_candidates(
                        opaque,
                        [sourced_beam_candidate(9, 2, source_tid, -2.0)],
                    );
                }
                (9, 2) => {
                    seed_discovered_candidates(
                        opaque,
                        [sourced_beam_candidate(9, 3, source_tid, -1.0)],
                    );
                }
                _ => {}
            },
        );

        assert_eq!(
            visible_frontier_candidates(&opaque)
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![entry_tid, child_tid, grandchild_tid],
            "bootstrap frontier filling should keep expanding from newly seeded candidates until capacity is reached"
        );
        assert_eq!(
            visible_frontier_candidates(&opaque)[0].source,
            None,
            "entry-seeded candidates should not claim a discovery source"
        );
        assert_eq!(
            visible_frontier_candidates(&opaque)[1].source,
            Some(entry_tid),
            "first-hop candidates should record the entry candidate as their source"
        );
        assert_eq!(
            visible_frontier_candidates(&opaque)[2].source,
            Some(child_tid),
            "second-hop candidates should record the candidate they were expanded from"
        );
    }

    #[test]
    fn top_up_bootstrap_frontier_preserves_expanded_state() {
        let entry_tid = page::ItemPointer {
            block_number: 11,
            offset_number: 1,
        };
        let sibling_tid = page::ItemPointer {
            block_number: 11,
            offset_number: 2,
        };
        let grandchild_tid = page::ItemPointer {
            block_number: 11,
            offset_number: 3,
        };
        let mut opaque = TqScanOpaque::default();
        reset_scan_expanded_state(&mut opaque);
        visible_frontier_mut(&mut opaque).push(beam_candidate(11, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(sourced_beam_candidate(11, 2, entry_tid, -2.0));
        mark_expanded_source(&mut opaque, entry_tid);
        reset_bootstrap_expansion_state(&mut opaque, 3);
        seed_existing_frontier_into_expansion(&mut opaque);

        top_up_bootstrap_frontier(
            &mut opaque,
            3,
            BootstrapExpandPolicy::ScoreOrder,
            |source_tid, opaque| {
                if source_tid == sibling_tid {
                    seed_discovered_candidates(
                        opaque,
                        [sourced_beam_candidate(11, 3, source_tid, -1.0)],
                    );
                }
            },
        );

        assert_eq!(
            visible_frontier_candidates(&opaque)
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![entry_tid, sibling_tid, grandchild_tid],
            "top-up should keep expanding from remaining unexpanded candidates without resetting prior expanded-source state"
        );
        assert!(
            expanded_contains_source(&opaque, entry_tid),
            "top-up should preserve previously expanded sources"
        );
        assert!(
            expanded_contains_source(&opaque, sibling_tid),
            "top-up should record the newly expanded candidate source"
        );
    }

    #[test]
    fn top_up_bootstrap_frontier_requires_seeded_scheduler() {
        let entry_tid = page::ItemPointer {
            block_number: 12,
            offset_number: 1,
        };
        let mut opaque = TqScanOpaque::default();
        visible_frontier_mut(&mut opaque).push(beam_candidate(12, 1, -3.0));
        reset_bootstrap_expansion_state(&mut opaque, 3);

        top_up_bootstrap_frontier(
            &mut opaque,
            3,
            BootstrapExpandPolicy::ScoreOrder,
            |_, opaque| {
                seed_discovered_candidates(
                    opaque,
                    [sourced_beam_candidate(12, 2, entry_tid, -2.0)],
                );
            },
        );

        assert_eq!(
            visible_frontier_candidates(&opaque)
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![entry_tid],
            "top-up should not silently rebuild beam state from the visible frontier when the scheduler is empty"
        );
        assert!(
            !expanded_contains_source(&opaque, entry_tid),
            "without a seeded scheduler, top-up should not mark any source as expanded"
        );
    }

    #[test]
    fn refill_after_consume_skips_already_expanded_source() {
        let consumed_tid = page::ItemPointer {
            block_number: 12,
            offset_number: 1,
        };
        let sibling_tid = page::ItemPointer {
            block_number: 12,
            offset_number: 2,
        };
        let grandchild_tid = page::ItemPointer {
            block_number: 12,
            offset_number: 3,
        };
        let mut opaque = TqScanOpaque::default();
        reset_scan_expanded_state(&mut opaque);
        visible_frontier_mut(&mut opaque).push(sourced_beam_candidate(12, 2, consumed_tid, -2.0));
        mark_expanded_source(&mut opaque, consumed_tid);
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        seed_existing_frontier_into_expansion(&mut opaque);

        let mut refilled_sources = Vec::new();
        refill_bootstrap_frontier_after_consume(
            &mut opaque,
            search::BeamCandidate::new(consumed_tid, -3.0),
            |source_tid, opaque| {
                refilled_sources.push(source_tid);
                if source_tid == sibling_tid {
                    seed_discovered_candidates(
                        opaque,
                        [sourced_beam_candidate(12, 3, source_tid, -1.0)],
                    );
                }
            },
        );

        assert!(
            !refilled_sources.contains(&consumed_tid),
            "consume/refill should not reread a source that was already expanded during earlier bootstrap work"
        );
        assert_eq!(
            refilled_sources.first().copied(),
            Some(sibling_tid),
            "consume/refill should continue by expanding another remaining frontier candidate first"
        );
        assert_eq!(
            visible_frontier_candidates(&opaque)
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![sibling_tid, grandchild_tid],
            "consume/refill should still top up from another remaining unexpanded frontier candidate"
        );
    }

    #[test]
    fn score_order_policy_prefers_lowest_score_unexpanded_frontier_candidate() {
        let mut opaque = TqScanOpaque::default();
        reset_scan_expanded_state(&mut opaque);
        visible_frontier_mut(&mut opaque).push(beam_candidate(10, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(sourced_beam_candidate(10, 2, tid(10, 1), -4.0));

        assert_eq!(
            visible_frontier_ref(&opaque)
                .iter()
                .filter(|candidate| !expanded_contains_source(&opaque, candidate.node))
                .min_by(|left, right| left.score.total_cmp(&right.score))
                .map(|candidate| candidate.node),
            Some(page::ItemPointer {
                block_number: 10,
                offset_number: 2,
            }),
            "the explicit score-order policy should expand the lowest-score unexpanded seeded candidate first"
        );

        mark_expanded_source(
            &mut opaque,
            page::ItemPointer {
                block_number: 10,
                offset_number: 2,
            },
        );
        assert_eq!(
            visible_frontier_ref(&opaque)
                .iter()
                .filter(|candidate| !expanded_contains_source(&opaque, candidate.node))
                .min_by(|left, right| left.score.total_cmp(&right.score))
                .map(|candidate| candidate.node),
            Some(page::ItemPointer {
                block_number: 10,
                offset_number: 1,
            }),
            "after the best candidate is marked expanded, the score-order policy should fall back to the next best seeded candidate"
        );
    }

    #[test]
    fn parallel_scan_worker_bootstrap_candidates_preserves_serial_order() {
        let candidates = vec![
            beam_candidate(18, 1, -4.0),
            beam_candidate(18, 2, -3.0),
            beam_candidate(18, 3, -2.0),
            beam_candidate(18, 4, -1.0),
        ];
        let opaque = TqScanOpaque::default();

        assert_eq!(
            parallel_scan_worker_bootstrap_candidates(&opaque, candidates.clone()),
            candidates,
            "unbound scans should keep the serial bootstrap candidate order"
        );
    }

    #[test]
    fn parallel_scan_worker_bootstrap_candidates_diversify_parallel_tail() {
        let candidates = vec![
            beam_candidate(19, 1, -5.0),
            beam_candidate(19, 2, -4.0),
            beam_candidate(19, 3, -3.0),
            beam_candidate(19, 4, -2.0),
            beam_candidate(19, 5, -1.0),
        ];
        let first_worker = TqScanOpaque {
            parallel_scan_state: std::ptr::dangling_mut(),
            parallel_scan_worker_slot_count: 3,
            parallel_scan_worker_slot_index: 0,
            scan_seed: 41,
            ..Default::default()
        };

        let second_worker = TqScanOpaque {
            parallel_scan_state: std::ptr::dangling_mut(),
            parallel_scan_worker_slot_count: 3,
            parallel_scan_worker_slot_index: 1,
            scan_seed: 41,
            ..Default::default()
        };

        let first_worker_candidates =
            parallel_scan_worker_bootstrap_candidates(&first_worker, candidates.clone());
        let second_worker_candidates =
            parallel_scan_worker_bootstrap_candidates(&second_worker, candidates);

        assert_eq!(
            first_worker_candidates.first().copied(),
            Some(beam_candidate(19, 1, -5.0)),
            "parallel bootstrap diversification should retain the shared best seed candidate"
        );
        assert_eq!(
            second_worker_candidates.first().copied(),
            Some(beam_candidate(19, 1, -5.0)),
            "parallel bootstrap diversification should retain the shared best seed candidate"
        );
        assert_ne!(
            first_worker_candidates, second_worker_candidates,
            "different worker slots should stage different bootstrap tails from the same scan seed"
        );
    }

    #[test]
    fn resolve_bootstrap_frontier_limit_keeps_serial_budget_without_parallel_slots() {
        let tuning = super::super::options::ScanTuning {
            relation_ef_search: 100,
            session_ef_search: None,
            effective_ef_search: 100,
            source: super::super::options::EfSearchSource::Relation,
        };

        assert_eq!(
            resolve_bootstrap_frontier_limit(tuning, 0),
            100,
            "unbound scans should keep the serial ef_search frontier budget"
        );
        assert_eq!(
            resolve_bootstrap_frontier_limit(tuning, 1),
            100,
            "single-worker staged scans should keep the serial ef_search frontier budget"
        );
    }

    #[test]
    fn resolve_bootstrap_frontier_limit_uses_parallel_overlap_split() {
        let tuning = super::super::options::ScanTuning {
            relation_ef_search: 100,
            session_ef_search: None,
            effective_ef_search: 100,
            source: super::super::options::EfSearchSource::Relation,
        };

        assert_eq!(
            resolve_bootstrap_frontier_limit(tuning, 4),
            28,
            "parallel-bound scans should use the staged per-worker ef_search split with overlap"
        );
    }

    #[test]
    fn seed_bootstrap_trace_marks_only_seed_entry_as_expanded() {
        let entry_tid = tid(15, 1);
        let sibling_tid = tid(15, 2);
        let grandchild_tid = tid(15, 3);
        let mut opaque = TqScanOpaque::default();

        seed_bootstrap_trace(
            &mut opaque,
            3,
            search::BeamTrace {
                discovered: vec![
                    beam_candidate(15, 1, -3.0),
                    sourced_beam_candidate(15, 2, entry_tid, -2.0),
                    sourced_beam_candidate(15, 3, sibling_tid, -1.0),
                ],
                expanded: vec![
                    beam_candidate(15, 1, -3.0),
                    sourced_beam_candidate(15, 2, entry_tid, -2.0),
                ],
                frontier: vec![sourced_beam_candidate(15, 3, sibling_tid, -1.0)],
            },
        );

        assert!(
            expanded_contains_source(&opaque, entry_tid),
            "trace seeding should keep the entry candidate marked expanded"
        );
        assert!(
            !expanded_contains_source(&opaque, sibling_tid),
            "trace seeding should not pre-mark later discovered candidates as expanded"
        );
        assert!(
            !expanded_contains_source(&opaque, grandchild_tid),
            "trace seeding should leave deeper discovered candidates available for later refill"
        );
    }

    #[test]
    fn source_backed_pq_fastscan_default_rerank_resolves_to_heap_f32() {
        let options = super::super::options::TqHnswOptions {
            m: super::super::EC_HNSW_DEFAULT_M,
            ef_construction: super::super::EC_HNSW_DEFAULT_EF_CONSTRUCTION,
            ef_search: super::super::EC_HNSW_DEFAULT_EF_SEARCH,
            build_source_column: Some("source".to_owned()),
            rerank_source_column: None,
            storage_format: super::super::options::StorageFormat::PqFastScan,
        };

        assert_eq!(
            default_grouped_rerank_mode(&options, false),
            GroupedRerankMode::HeapF32,
            "source-backed pq_fastscan indexes should default rerank to heap_f32"
        );
        assert_eq!(
            default_grouped_rerank_mode_resolution(&options, false),
            PqFastScanRerankModeResolution::DefaultHeapF32WithBuildSourceColumn,
            "source-backed pq_fastscan defaults should explain that heap_f32 came from build_source_column"
        );
    }

    #[test]
    fn source_backed_turboquant_default_rerank_resolves_to_quantized() {
        let options = super::super::options::TqHnswOptions {
            m: super::super::EC_HNSW_DEFAULT_M,
            ef_construction: super::super::EC_HNSW_DEFAULT_EF_CONSTRUCTION,
            ef_search: super::super::EC_HNSW_DEFAULT_EF_SEARCH,
            build_source_column: Some("source".to_owned()),
            rerank_source_column: None,
            storage_format: super::super::options::StorageFormat::TurboQuant,
        };

        assert_eq!(
            default_grouped_rerank_mode(&options, false),
            GroupedRerankMode::Quantized,
            "source-backed turboquant indexes should default rerank to quantized"
        );
        assert_eq!(
            default_grouped_rerank_mode_resolution(&options, false),
            PqFastScanRerankModeResolution::DefaultQuantizedTurboQuantStorage,
            "source-backed turboquant defaults should explain that quantized came from turboquant storage"
        );
    }

    #[test]
    fn indexed_tqvector_pq_fastscan_default_rerank_resolves_to_quantized() {
        let options = super::super::options::TqHnswOptions {
            m: super::super::EC_HNSW_DEFAULT_M,
            ef_construction: super::super::EC_HNSW_DEFAULT_EF_CONSTRUCTION,
            ef_search: super::super::EC_HNSW_DEFAULT_EF_SEARCH,
            build_source_column: None,
            rerank_source_column: None,
            storage_format: super::super::options::StorageFormat::PqFastScan,
        };

        assert_eq!(
            default_grouped_rerank_mode(&options, false),
            GroupedRerankMode::Quantized,
            "indexed tqvector pq_fastscan indexes should default rerank to quantized"
        );
        assert_eq!(
            default_grouped_rerank_mode_resolution(&options, false),
            PqFastScanRerankModeResolution::DefaultQuantizedWithIndexedTqvector,
            "indexed tqvector pq_fastscan defaults should explain that quantized came from the indexed tqvector column"
        );
    }

    #[test]
    fn indexed_ecvector_pq_fastscan_default_rerank_resolves_to_heap_f32() {
        let options = super::super::options::TqHnswOptions {
            m: super::super::EC_HNSW_DEFAULT_M,
            ef_construction: super::super::EC_HNSW_DEFAULT_EF_CONSTRUCTION,
            ef_search: super::super::EC_HNSW_DEFAULT_EF_SEARCH,
            build_source_column: None,
            rerank_source_column: None,
            storage_format: super::super::options::StorageFormat::PqFastScan,
        };

        assert_eq!(
            default_grouped_rerank_mode(&options, true),
            GroupedRerankMode::HeapF32,
            "indexed ecvector pq_fastscan indexes should default rerank to heap_f32"
        );
        assert_eq!(
            default_grouped_rerank_mode_resolution(&options, true),
            PqFastScanRerankModeResolution::DefaultHeapF32WithIndexedColumn,
            "indexed ecvector pq_fastscan defaults should explain that heap_f32 came from the indexed ecvector column"
        );
    }

    #[test]
    fn rerank_source_backed_pq_fastscan_default_rerank_resolves_to_heap_f32() {
        let options = super::super::options::TqHnswOptions {
            m: super::super::EC_HNSW_DEFAULT_M,
            ef_construction: super::super::EC_HNSW_DEFAULT_EF_CONSTRUCTION,
            ef_search: super::super::EC_HNSW_DEFAULT_EF_SEARCH,
            build_source_column: Some("source".to_owned()),
            rerank_source_column: Some("source_raw".to_owned()),
            storage_format: super::super::options::StorageFormat::PqFastScan,
        };

        assert_eq!(
            default_grouped_rerank_mode(&options, false),
            GroupedRerankMode::HeapF32,
            "pq_fastscan indexes with a persisted rerank source should default rerank to heap_f32"
        );
        assert_eq!(
            default_grouped_rerank_mode_resolution(&options, false),
            PqFastScanRerankModeResolution::DefaultHeapF32WithRerankSourceColumn,
            "pq_fastscan defaults should explain when heap_f32 came from a persisted rerank_source_column"
        );
        assert_eq!(
            effective_grouped_rerank_source_column(&options, GroupedRerankMode::HeapF32)
                .as_deref(),
            Some("source_raw"),
            "a persisted rerank_source_column should win over build_source_column for default heap rerank"
        );
    }

    #[test]
    fn grouped_binary_traversal_score_gate_requires_pq_fastscan_storage() {
        let mut opaque = TqScanOpaque {
            grouped_traversal_score_mode: GroupedTraversalScoreMode::Binary,
            scan_graph_storage: graph::GraphStorageDescriptor::TurboQuant { code_len: 64 },
            ..TqScanOpaque::default()
        };
        assert!(
            !grouped_binary_traversal_score_enabled(&opaque),
            "binary traversal score mode should stay off for non-pq_fastscan storage even when the mode is binary",
        );

        opaque.scan_graph_storage =
            graph::GraphStorageDescriptor::PqFastScan(graph::PqFastScanLayout {
                binary_word_count: 24,
                search_code_len: 48,
                rerank_code_len: 768,
            });
        assert!(
            grouped_binary_traversal_score_enabled(&opaque),
            "binary traversal score mode should activate for pq_fastscan layouts when the mode is binary",
        );

        opaque.grouped_traversal_score_mode = GroupedTraversalScoreMode::GroupedPq;
        assert!(
            !grouped_binary_traversal_score_enabled(&opaque),
            "grouped-pq traversal mode should disable the binary traversal gate even for pq_fastscan layouts",
        );
    }
}
