use std::{cmp::Ordering, ptr};

use pgrx::pg_sys;

use super::{build, graph, options, page, search, shared, source};
use crate::storage::wal;

const P_NEW: pg_sys::BlockNumber = u32::MAX;
// One initial write pass plus up to two read-only replan retries for drifted full slices.
const MAX_BACKLINK_REPLAN_PASSES: usize = 3;
const PQ_FASTSCAN_CODEBOOK_METADATA_UNAVAILABLE: &str =
    "tqhnsw PqFastScan metadata is missing persisted grouped codebooks";

#[derive(Debug)]
enum InsertSearchMetric {
    Code,
    Source(InsertHeapSourceScorer),
}

#[derive(Debug)]
struct InsertHeapSourceScorer {
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attribute: source::SourceAttribute,
}

impl InsertHeapSourceScorer {
    unsafe fn new(heap_relation: pg_sys::Relation, source_column: &str) -> Self {
        let source_attribute = unsafe {
            source::resolve_source_attribute(
                heap_relation,
                source_column,
                "build_source_column",
                source::SourceTypePolicy::BuildSource,
            )
        };
        unsafe { Self::new_with_attribute(heap_relation, source_attribute) }
    }

    unsafe fn new_with_attribute(
        heap_relation: pg_sys::Relation,
        source_attribute: source::SourceAttribute,
    ) -> Self {
        let slot = unsafe {
            source::allocate_heap_slot(
                heap_relation,
                "tqhnsw aminsert failed to allocate a heap source slot",
            )
        };

        Self {
            heap_relation,
            snapshot: std::ptr::addr_of_mut!(pg_sys::SnapshotSelfData),
            slot,
            source_attribute,
        }
    }

    unsafe fn load_source_vector(&mut self, heap_tid: page::ItemPointer, label: &str) -> Vec<f32> {
        let source = unsafe {
            source::load_source_from_heap_row(
                self.heap_relation,
                heap_tid,
                self.snapshot,
                self.slot,
                self.source_attribute,
                label,
            )
        };
        let vector = source.as_slice().to_vec();
        drop(source);
        unsafe { pg_sys::ExecClearTuple(self.slot) };
        vector
    }

    unsafe fn averaged_source_vector(
        &mut self,
        heap_tids: &[page::ItemPointer],
        label: &str,
    ) -> Option<Vec<f32>> {
        let mut representative: Option<Vec<f32>> = None;
        let mut count = 0usize;

        for heap_tid in heap_tids.iter().copied() {
            let source = unsafe {
                source::load_source_from_heap_row(
                    self.heap_relation,
                    heap_tid,
                    self.snapshot,
                    self.slot,
                    self.source_attribute,
                    label,
                )
            };
            match representative.as_mut() {
                Some(existing) => {
                    source::average_source_representatives(existing, count, source.as_slice(), 1);
                    count += 1;
                }
                None => {
                    representative = Some(source.as_slice().to_vec());
                    count = 1;
                }
            }
            drop(source);
            unsafe { pg_sys::ExecClearTuple(self.slot) };
        }

        representative
    }

    unsafe fn score_element_against_query(
        &mut self,
        query_source: &[f32],
        element: &graph::GraphElement,
    ) -> Option<f32> {
        if element.deleted || element.heaptids.is_empty() {
            return None;
        }

        let element_source = unsafe {
            self.averaged_source_vector(&element.heaptids, "live insert source graph element")
        }?;
        Some(source::negative_inner_product(
            query_source,
            &element_source,
        ))
    }

    unsafe fn score_existing_backlink_candidate(
        &mut self,
        target_element: &graph::GraphElement,
        candidate_element: &graph::GraphElement,
    ) -> f32 {
        let target_source = unsafe {
            self.averaged_source_vector(
                &target_element.heaptids,
                "live insert backlink target source vector",
            )
        }
        .unwrap_or_else(|| {
            pgrx::error!("tqhnsw live insert backlink target is missing source data")
        });
        let candidate_source = unsafe {
            self.averaged_source_vector(
                &candidate_element.heaptids,
                "live insert backlink candidate source vector",
            )
        }
        .unwrap_or_else(|| {
            pgrx::error!("tqhnsw live insert backlink candidate is missing source data")
        });
        source::negative_inner_product(&target_source, &candidate_source)
    }

    fn score_new_backlink_candidate(
        &mut self,
        target_element: &graph::GraphElement,
        new_tuple: &build::BuildTuple,
    ) -> f32 {
        let target_source = unsafe {
            self.averaged_source_vector(
                &target_element.heaptids,
                "live insert backlink target source vector",
            )
        }
        .unwrap_or_else(|| {
            pgrx::error!("tqhnsw live insert backlink target is missing source data")
        });
        let new_source = new_tuple.source_vector.as_deref().unwrap_or_else(|| {
            pgrx::error!("tqhnsw live insert source scoring requires source data")
        });
        source::negative_inner_product(&target_source, new_source)
    }
}

impl Drop for InsertHeapSourceScorer {
    fn drop(&mut self) {
        if !self.slot.is_null() {
            unsafe { pg_sys::ExecDropSingleTupleTableSlot(self.slot) };
        }
    }
}

impl InsertSearchMetric {
    unsafe fn score_new_tuple_against_element(
        &mut self,
        metadata: &page::MetadataPage,
        tuple: &build::BuildTuple,
        element: &graph::GraphElement,
    ) -> Option<f32> {
        match self {
            Self::Code => score_insert_graph_element(metadata, &tuple.code, element),
            Self::Source(scorer) => unsafe {
                scorer.score_element_against_query(
                    tuple.source_vector.as_deref().unwrap_or_else(|| {
                        pgrx::error!("tqhnsw live insert source scoring requires source data")
                    }),
                    element,
                )
            },
        }
    }

    unsafe fn score_existing_backlink_candidate(
        &mut self,
        metadata: &page::MetadataPage,
        target_element: &graph::GraphElement,
        candidate_element: &graph::GraphElement,
    ) -> f32 {
        match self {
            Self::Code => {
                score_backlink_candidate(metadata, &target_element.code, &candidate_element.code)
            }
            Self::Source(scorer) => unsafe {
                scorer.score_existing_backlink_candidate(target_element, candidate_element)
            },
        }
    }

    unsafe fn score_new_backlink_candidate(
        &mut self,
        metadata: &page::MetadataPage,
        target_element: &graph::GraphElement,
        new_tuple: &build::BuildTuple,
    ) -> f32 {
        match self {
            Self::Code => score_backlink_candidate(metadata, &target_element.code, &new_tuple.code),
            Self::Source(scorer) => scorer.score_new_backlink_candidate(target_element, new_tuple),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertFormatAdapter {
    TurboQuant { code_len: usize },
    TurboQuantHotCold(graph::TurboQuantHotColdLayout),
    PqFastScan(graph::PqFastScanLayout),
}

impl InsertFormatAdapter {
    fn graph_storage(self) -> graph::GraphStorageDescriptor {
        match self {
            Self::TurboQuant { code_len } => graph::GraphStorageDescriptor::TurboQuant { code_len },
            Self::TurboQuantHotCold(layout) => {
                graph::GraphStorageDescriptor::TurboQuantHotCold(layout)
            }
            Self::PqFastScan(layout) => graph::GraphStorageDescriptor::PqFastScan(layout),
        }
    }

    fn initial_code_len(self, tuple: &build::BuildTuple) -> usize {
        match self {
            Self::TurboQuant { code_len } => code_len.max(tuple.code.len()),
            Self::TurboQuantHotCold(layout) => layout.rerank_code_len.max(tuple.code.len()),
            Self::PqFastScan(_) => tuple.code.len(),
        }
    }

    unsafe fn find_duplicate(
        self,
        index_relation: pg_sys::Relation,
        heap_relation: pg_sys::Relation,
        metadata: &page::MetadataPage,
        tuple: &build::BuildTuple,
        code_len: usize,
    ) -> Option<page::ItemPointer> {
        match self {
            Self::TurboQuant { .. } => unsafe {
                find_duplicate_element_tid(
                    index_relation,
                    heap_relation,
                    metadata.dimensions,
                    metadata.bits,
                    tuple.gamma,
                    code_len,
                    &tuple.code,
                )
            },
            Self::TurboQuantHotCold(layout) => unsafe {
                find_duplicate_turbo_hot_element_tid(
                    index_relation,
                    tuple.gamma,
                    &tuple.code,
                    layout,
                )
            },
            Self::PqFastScan(layout) => unsafe {
                find_duplicate_grouped_element_tid(index_relation, tuple.gamma, &tuple.code, layout)
            },
        }
    }

    unsafe fn discover_forward_neighbors(
        self,
        index_relation: pg_sys::Relation,
        metadata: &page::MetadataPage,
        tuple: &build::BuildTuple,
        metric: &mut InsertSearchMetric,
        insert_level: u8,
        m: u16,
    ) -> (Vec<page::ItemPointer>, Vec<LayerForwardSelection>) {
        unsafe {
            discover_insert_forward_neighbor_slots(
                index_relation,
                metadata,
                self.graph_storage(),
                tuple,
                metric,
                insert_level,
                m,
            )
        }
    }

    unsafe fn append_node(
        self,
        index_relation: pg_sys::Relation,
        metadata: &page::MetadataPage,
        tuple: &build::BuildTuple,
        level: u8,
        neighbor_tids: &[page::ItemPointer],
    ) -> page::ItemPointer {
        match self {
            Self::TurboQuant { .. } => unsafe {
                append_heap_tuple(index_relation, tuple, level, neighbor_tids)
            },
            Self::TurboQuantHotCold(layout) => unsafe {
                append_turbo_hot_cold_tuple(index_relation, tuple, level, neighbor_tids, layout)
            },
            Self::PqFastScan(layout) => unsafe {
                append_pq_fastscan_tuple(
                    index_relation,
                    metadata,
                    tuple,
                    level,
                    neighbor_tids,
                    layout,
                )
            },
        }
    }

    unsafe fn coalesce_duplicate(
        self,
        index_relation: pg_sys::Relation,
        element_tid: page::ItemPointer,
        heap_tid: page::ItemPointer,
    ) {
        match self {
            Self::TurboQuant { code_len } => unsafe {
                coalesce_duplicate_heap_tid(index_relation, element_tid, code_len, heap_tid)
            },
            Self::TurboQuantHotCold(layout) => unsafe {
                coalesce_duplicate_turbo_hot_heap_tid(index_relation, element_tid, layout, heap_tid)
            },
            Self::PqFastScan(layout) => unsafe {
                coalesce_duplicate_grouped_heap_tid(index_relation, element_tid, layout, heap_tid)
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn add_backlinks(
        self,
        index_relation: pg_sys::Relation,
        metadata: &page::MetadataPage,
        tuple: &build::BuildTuple,
        metric: &mut InsertSearchMetric,
        selections: &[LayerForwardSelection],
        new_element_tid: page::ItemPointer,
        m: u16,
    ) {
        unsafe {
            add_backlinks_to_forward_neighbors(
                index_relation,
                metadata,
                self.graph_storage(),
                tuple,
                metric,
                selections,
                new_element_tid,
                m,
            )
        }
    }
}

pub(super) unsafe extern "C-unwind" fn tqhnsw_aminsert(
    index_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: pg_sys::ItemPointer,
    heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    index_info: *mut pg_sys::IndexInfo,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let heap_tid = shared::decode_heap_tid(heap_tid);
            let options = options::relation_options(index_relation);
            let metadata_snapshot = shared::read_metadata_page(index_relation);
            let format = resolve_insert_format_adapter(index_relation, &metadata_snapshot)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            let tuple;
            let mut metric;
            let indexed_attribute = source::resolve_indexed_vector_attribute_from_index_info(
                heap_relation,
                index_info,
                "indexed column",
            );
            if let Some(source_column) = options.build_source_column.as_deref() {
                if values.is_null() || isnull.is_null() {
                    pgrx::error!("tqhnsw aminsert received null tuple value arrays");
                }
                if *isnull {
                    pgrx::error!("tqhnsw does not support NULL indexed values");
                }
                let source_attribute = source::resolve_source_attribute(
                    heap_relation,
                    source_column,
                    "build_source_column",
                    source::SourceTypePolicy::BuildSource,
                );
                let mut source_scorer =
                    InsertHeapSourceScorer::new_with_attribute(heap_relation, source_attribute);
                let source_vector = source_scorer
                    .load_source_vector(heap_tid, "tqhnsw live insert build_source_column");
                tuple = build::build_heap_tuple_with_source(
                    *values,
                    heap_tid,
                    source_vector,
                    indexed_attribute.kind,
                );
                metric = InsertSearchMetric::Source(source_scorer);
            } else {
                tuple = build::build_heap_tuple(values, isnull, heap_tid, indexed_attribute.kind);
                metric = match indexed_attribute.kind {
                    source::IndexedVectorKind::Ecvector => {
                        InsertSearchMetric::Source(InsertHeapSourceScorer::new_with_attribute(
                            heap_relation,
                            source::SourceAttribute {
                                attnum: indexed_attribute.attnum,
                                kind: source::SourceDatumKind::Ecvector,
                            },
                        ))
                    }
                    source::IndexedVectorKind::Tqvector => InsertSearchMetric::Code,
                };
            }
            run_insert_with_adapter(
                format,
                index_relation,
                heap_relation,
                heap_tid,
                &tuple,
                &mut metric,
                &metadata_snapshot,
                u16::try_from(options.m).expect("validated m should fit in u16"),
            )
        })
    }
}

fn resolve_insert_format_adapter(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> Result<InsertFormatAdapter, String> {
    match unsafe { graph::GraphStorageDescriptor::from_index_relation(index_relation, metadata) }? {
        graph::GraphStorageDescriptor::TurboQuant { code_len } => {
            Ok(InsertFormatAdapter::TurboQuant { code_len })
        }
        graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
            Ok(InsertFormatAdapter::TurboQuantHotCold(layout))
        }
        graph::GraphStorageDescriptor::PqFastScan(layout) => {
            Ok(InsertFormatAdapter::PqFastScan(layout))
        }
    }
}

fn initialized_empty_insert_metadata(
    metadata: &page::MetadataPage,
    format: InsertFormatAdapter,
    tuple: &build::BuildTuple,
) -> page::MetadataPage {
    let current = page::CurrentFormatMetadata {
        m: metadata.m,
        ef_construction: metadata.ef_construction,
        entry_point: metadata.entry_point,
        dimensions: tuple.dimensions,
        bits: tuple.bits,
        max_level: metadata.max_level,
        seed: tuple.seed,
        inserted_since_rebuild: metadata.inserted_since_rebuild,
        persisted_binary_sidecar: crate::quant::prod::ProdQuantizer::cached(
            tuple.dimensions as usize,
            tuple.bits,
            tuple.seed,
        )
        .binary_sign_no_qjl_4bit_supported(),
    };
    match format {
        InsertFormatAdapter::TurboQuant { .. } => page::MetadataPage::current_v1_scalar(current),
        InsertFormatAdapter::TurboQuantHotCold(_) => {
            page::MetadataPage::current_v3_turbo_hot_cold(current)
        }
        InsertFormatAdapter::PqFastScan(_) => {
            panic!("empty grouped metadata initialization should use the dedicated bootstrap path")
        }
    }
}

unsafe fn run_insert_with_adapter(
    format: InsertFormatAdapter,
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    heap_tid: page::ItemPointer,
    tuple: &build::BuildTuple,
    metric: &mut InsertSearchMetric,
    metadata_snapshot: &page::MetadataPage,
    m: u16,
) -> bool {
    let code_len = format.initial_code_len(tuple);

    // First-insert path: shape has never been initialized. Keep this on the
    // old exclusive path because shape init atomicity still matters, and the
    // duplicate scan is degenerate on an effectively empty index.
    if metadata_snapshot.dimensions == 0 && metadata_snapshot.bits == 0 {
        if matches!(format, InsertFormatAdapter::PqFastScan(_)) {
            let bootstrapped = unsafe {
                shared::with_locked_metadata_page(index_relation, |metadata| {
                    if metadata.dimensions != 0 || metadata.bits != 0 {
                        return false;
                    }

                    let output = bootstrap_empty_pq_fastscan_flush_output(index_relation, tuple);
                    build::write_data_pages(index_relation, &output.data_pages);
                    *metadata = output.metadata;
                    true
                })
            };
            if bootstrapped {
                return false;
            }

            let refreshed_metadata = unsafe { shared::read_metadata_page(index_relation) };
            return unsafe {
                run_insert_with_adapter(
                    format,
                    index_relation,
                    heap_relation,
                    heap_tid,
                    tuple,
                    metric,
                    &refreshed_metadata,
                    m,
                )
            };
        }
        shared::with_locked_metadata_page(index_relation, |metadata| {
            if metadata.dimensions == 0 && metadata.bits == 0 {
                *metadata = initialized_empty_insert_metadata(metadata, format, tuple);
            } else if tuple.dimensions != metadata.dimensions
                || tuple.bits != metadata.bits
                || tuple.seed != metadata.seed
            {
                pgrx::error!(
                    "tqhnsw aminsert requires matching quantized index shape ({},{},{}) but got ({},{},{})",
                    metadata.dimensions,
                    metadata.bits,
                    metadata.seed,
                    tuple.dimensions,
                    tuple.bits,
                    tuple.seed
                );
            }

            let active_format = resolve_insert_format_adapter(index_relation, metadata)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            if let Some(element_tid) = active_format.find_duplicate(
                index_relation,
                heap_relation,
                metadata,
                tuple,
                code_len,
            ) {
                active_format.coalesce_duplicate(index_relation, element_tid, heap_tid);
                return;
            }

            let insert_level = choose_insert_level(m, metadata.seed, heap_tid, tuple.code.len());
            let forward_neighbor_slots = empty_insert_neighbor_slots(insert_level, m);
            let element_tid = active_format.append_node(
                index_relation,
                metadata,
                tuple,
                insert_level,
                &forward_neighbor_slots,
            );
            metadata.inserted_since_rebuild = metadata.inserted_since_rebuild.saturating_add(1);
            if metadata.entry_point == page::ItemPointer::INVALID {
                metadata.entry_point = element_tid;
                metadata.max_level = insert_level;
            }
        });
        return false;
    }

    // Fast path: shape is known. Those fields are write-once after
    // initialization, so the SHARE-read snapshot is authoritative here.
    if tuple.dimensions != metadata_snapshot.dimensions
        || tuple.bits != metadata_snapshot.bits
        || tuple.seed != metadata_snapshot.seed
    {
        pgrx::error!(
            "tqhnsw aminsert requires matching quantized index shape ({},{},{}) but got ({},{},{})",
            metadata_snapshot.dimensions,
            metadata_snapshot.bits,
            metadata_snapshot.seed,
            tuple.dimensions,
            tuple.bits,
            tuple.seed
        );
    }

    // Duplicate scan runs with only SHARE locks on individual data pages.
    // A concurrent insert that commits the same code between this scan and
    // our append may double-insert; that rare race is acceptable here in
    // exchange for removing the metadata-page serialization point.
    if let Some(element_tid) = format.find_duplicate(
        index_relation,
        heap_relation,
        metadata_snapshot,
        tuple,
        code_len,
    ) {
        format.coalesce_duplicate(index_relation, element_tid, heap_tid);
        return false;
    }

    let insert_level = choose_insert_level(m, metadata_snapshot.seed, heap_tid, tuple.code.len());
    let (forward_neighbor_slots, forward_selections) = format.discover_forward_neighbors(
        index_relation,
        metadata_snapshot,
        tuple,
        metric,
        insert_level,
        m,
    );
    let element_tid = format.append_node(
        index_relation,
        metadata_snapshot,
        tuple,
        insert_level,
        &forward_neighbor_slots,
    );
    format.add_backlinks(
        index_relation,
        metadata_snapshot,
        tuple,
        metric,
        &forward_selections,
        element_tid,
        m,
    );

    // Successful live inserts now always advance the metadata-resident
    // drift counter, so every new-node append takes one final metadata
    // write phase after all data-page writes are complete. Entry-point
    // repair/promotion piggybacks on that same lock scope.
    let storage = format.graph_storage();
    shared::with_locked_metadata_page(index_relation, |metadata| {
        metadata.inserted_since_rebuild = metadata.inserted_since_rebuild.saturating_add(1);
        let entry_point_needs_repair = metadata.entry_point == page::ItemPointer::INVALID || {
            let entry = unsafe {
                graph::load_exact_graph_element(index_relation, metadata.entry_point, storage)
            };
            entry.deleted || entry.heaptids.is_empty()
        };
        if entry_point_needs_repair || insert_level > metadata.max_level {
            // Metadata must always advertise a live element at
            // metadata.max_level. The new tuple is already appended, so
            // repair/promotion happens only after append commits.
            metadata.entry_point = element_tid;
            metadata.max_level = insert_level;
        }
    });
    false
}

pub(super) fn choose_insert_level(
    m: u16,
    seed: u64,
    heap_tid: page::ItemPointer,
    code_len: usize,
) -> u8 {
    choose_insert_level_for_page_size(m, seed, heap_tid, code_len, pg_sys::BLCKSZ as usize)
}

pub(super) fn choose_insert_level_for_page_size(
    m: u16,
    seed: u64,
    heap_tid: page::ItemPointer,
    code_len: usize,
    page_size: usize,
) -> u8 {
    let max_level = max_insert_level_that_fits(m, code_len, page_size);
    if max_level == 0 {
        return 0;
    }

    let random_bits = splitmix64(seed ^ encode_heap_tid(heap_tid));
    level_from_random_bits(random_bits, m, max_level)
}

fn max_insert_level_that_fits(m: u16, code_len: usize, page_size: usize) -> u8 {
    let mut level = page::max_level_that_fits(m, page_size);
    loop {
        let required_bytes =
            page::raw_tuple_storage_bytes(page::neighbor_tuple_encoded_len(level, m))
                + page::raw_tuple_storage_bytes(page::TqElementTuple::encoded_len(code_len));
        if required_bytes <= page_size.saturating_sub(page::PAGE_HEADER_BYTES) {
            return level;
        }
        if level == 0 {
            return 0;
        }
        level = level.saturating_sub(1);
    }
}

fn level_from_random_bits(random_bits: u64, m: u16, max_level: u8) -> u8 {
    // Keep the +1 numerator so bits=0 maps to (0, 1] instead of 0 and cannot
    // hit ln(0); at f64 precision the denominator rounds to exactly 2^64.
    let unit = ((random_bits as f64) + 1.0_f64) / ((u64::MAX as f64) + 1.0_f64);
    let sampled_level = (-unit.ln() / (m as f64).ln()).floor();
    sampled_level.clamp(0.0_f64, max_level as f64) as u8
}

fn encode_heap_tid(heap_tid: page::ItemPointer) -> u64 {
    (u64::from(heap_tid.block_number) << 16) | u64::from(heap_tid.offset_number)
}

fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = (state ^ (state >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^ (state >> 31)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn debug_insert_level_for_heap_tid(
    m: u16,
    seed: u64,
    heap_tid: page::ItemPointer,
    code_len: usize,
) -> u8 {
    choose_insert_level(m, seed, heap_tid, code_len)
}

pub(super) fn empty_insert_neighbor_slots(level: u8, m: u16) -> Vec<page::ItemPointer> {
    vec![page::ItemPointer::INVALID; page::neighbor_slots(level, m)]
}

fn insert_ef_construction(metadata: &page::MetadataPage) -> usize {
    let ef = usize::from(metadata.ef_construction);
    debug_assert!(
        ef > 0,
        "validated tqhnsw indexes should always persist ef_construction >= 1"
    );
    ef.max(1)
}

pub(super) fn select_best_backlink_candidates<NodeId, TieBreakFn>(
    mut candidates: Vec<ScoredBacklinkNode<NodeId>>,
    keep_len: usize,
    mut tie_break: TieBreakFn,
) -> Vec<NodeId>
where
    NodeId: Copy,
    TieBreakFn: FnMut(&NodeId, &NodeId) -> Ordering,
{
    candidates.sort_unstable_by(|left, right| {
        left.score
            .total_cmp(&right.score)
            .then_with(|| left.is_new.cmp(&right.is_new))
            .then_with(|| tie_break(&left.node, &right.node))
    });
    candidates
        .into_iter()
        .take(keep_len)
        .map(|candidate| candidate.node)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LayerForwardSelection {
    layer: u8,
    element_tid: page::ItemPointer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BacklinkMutation {
    target_element_tid: page::ItemPointer,
    neighbor_tid: page::ItemPointer,
    layer: u8,
    kind: BacklinkMutationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BacklinkMutationKind {
    InsertIfFree,
    RewriteFullSlice {
        expected_slice: Vec<page::ItemPointer>,
        replacement_slice: Vec<page::ItemPointer>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ScoredBacklinkNode<NodeId> {
    pub(super) node: NodeId,
    pub(super) score: f32,
    pub(super) is_new: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BacklinkMutationOutcome {
    NoChange,
    Changed,
    RetryReplan,
}

#[derive(Debug, Clone, Copy)]
struct BacklinkPlanner<'a> {
    metadata: &'a page::MetadataPage,
    storage: graph::GraphStorageDescriptor,
    new_tuple: &'a build::BuildTuple,
    new_element_tid: page::ItemPointer,
    m: u16,
}

unsafe fn discover_insert_forward_neighbor_slots(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    storage: graph::GraphStorageDescriptor,
    tuple: &build::BuildTuple,
    metric: &mut InsertSearchMetric,
    insert_level: u8,
    m: u16,
) -> (Vec<page::ItemPointer>, Vec<LayerForwardSelection>) {
    let mut slots = empty_insert_neighbor_slots(insert_level, m);
    let mut selections = Vec::new();
    let Some(entry_candidate) =
        (unsafe { load_insert_entry_candidate(index_relation, metadata, storage, tuple, metric) })
    else {
        return (slots, selections);
    };

    let m_usize = usize::from(m);
    unsafe {
        populate_upper_layer_forward_slots(
            index_relation,
            metadata,
            storage,
            tuple,
            metric,
            insert_level,
            m_usize,
            entry_candidate,
            &mut slots,
            &mut selections,
        );
    }
    let descended_seed = unsafe {
        graph::greedy_descend_from_entry_with_storage(
            index_relation,
            storage,
            m_usize,
            entry_candidate,
            |neighbor| metric.score_new_tuple_against_element(metadata, tuple, neighbor),
        )
    };
    let layer0_candidates = unsafe {
        graph::search_layer0_result_candidates_with_storage(
            index_relation,
            storage,
            m_usize,
            insert_ef_construction(metadata),
            [descended_seed],
            |_| true,
            |neighbor| metric.score_new_tuple_against_element(metadata, tuple, neighbor),
        )
    };

    write_layer_forward_candidates(&mut slots, &mut selections, 0, m_usize, layer0_candidates);

    (slots, selections)
}

unsafe fn load_insert_entry_candidate(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    storage: graph::GraphStorageDescriptor,
    tuple: &build::BuildTuple,
    metric: &mut InsertSearchMetric,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    if metadata.entry_point == page::ItemPointer::INVALID {
        return None;
    }

    let entry =
        unsafe { graph::load_exact_graph_element(index_relation, metadata.entry_point, storage) };
    let entry_score = unsafe { metric.score_new_tuple_against_element(metadata, tuple, &entry) }?;
    Some(search::BeamCandidate::new(entry.tid, entry_score))
}

fn score_insert_graph_element(
    metadata: &page::MetadataPage,
    query_code: &[u8],
    element: &graph::GraphElement,
) -> Option<f32> {
    if element.deleted || element.heaptids.is_empty() {
        return None;
    }

    // BeamCandidate ordering is "lower is better", so negate the inner-product
    // scorer here to keep insert-side graph search aligned with scan/build.
    Some(-crate::score_code_inner_product(
        metadata.dimensions as usize,
        metadata.bits,
        metadata.seed,
        query_code,
        &element.code,
    ))
}

#[allow(clippy::too_many_arguments)]
unsafe fn populate_upper_layer_forward_slots(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    storage: graph::GraphStorageDescriptor,
    tuple: &build::BuildTuple,
    metric: &mut InsertSearchMetric,
    insert_level: u8,
    m: usize,
    entry_candidate: search::BeamCandidate<page::ItemPointer>,
    slots: &mut [page::ItemPointer],
    selections: &mut Vec<LayerForwardSelection>,
) {
    if insert_level == 0 || metadata.max_level == 0 {
        return;
    }

    let mut seeds = vec![entry_candidate];
    for current_layer in (1..=metadata.max_level).rev() {
        seeds = unsafe {
            graph::search_layer_result_candidates_with_storage(
                index_relation,
                storage,
                m,
                current_layer,
                insert_ef_construction(metadata),
                seeds,
                |_| true,
                |neighbor| metric.score_new_tuple_against_element(metadata, tuple, neighbor),
            )
        };
        if current_layer <= insert_level {
            write_layer_forward_candidates(slots, selections, current_layer, m, seeds.clone());
        }
        if seeds.is_empty() {
            break;
        }
    }
}

fn write_layer_forward_candidates(
    slots: &mut [page::ItemPointer],
    selections: &mut Vec<LayerForwardSelection>,
    layer: u8,
    m: usize,
    candidates: impl IntoIterator<Item = search::BeamCandidate<page::ItemPointer>>,
) {
    let Some((start, end)) = selected_forward_slot_bounds(m, slots.len(), layer) else {
        return;
    };

    for (slot, candidate) in slots[start..end]
        .iter_mut()
        .zip(candidates.into_iter().take(end.saturating_sub(start)))
    {
        *slot = candidate.node;
        selections.push(LayerForwardSelection {
            layer,
            element_tid: candidate.node,
        });
    }
}

unsafe fn add_backlinks_to_forward_neighbors(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    storage: graph::GraphStorageDescriptor,
    new_tuple: &build::BuildTuple,
    metric: &mut InsertSearchMetric,
    selections: &[LayerForwardSelection],
    new_element_tid: page::ItemPointer,
    m: u16,
) {
    let planner = BacklinkPlanner {
        metadata,
        storage,
        new_tuple,
        new_element_tid,
        m,
    };
    let mut pending = selections
        .iter()
        .copied()
        .filter(|selection| selection.element_tid != page::ItemPointer::INVALID)
        .collect::<Vec<_>>();
    sort_and_dedup_forward_selections(&mut pending);

    for _ in 0..MAX_BACKLINK_REPLAN_PASSES {
        if pending.is_empty() {
            break;
        }

        let mutations =
            unsafe { plan_backlink_mutations(index_relation, &planner, metric, &pending) };
        if mutations.is_empty() {
            break;
        }

        pending =
            unsafe { apply_backlink_mutations(index_relation, &mutations, new_element_tid, m) };
    }
}

unsafe fn plan_backlink_mutations(
    index_relation: pg_sys::Relation,
    planner: &BacklinkPlanner<'_>,
    metric: &mut InsertSearchMetric,
    selections: &[LayerForwardSelection],
) -> Vec<BacklinkMutation> {
    let mut mutations = selections
        .iter()
        .copied()
        .filter_map(|selection| unsafe {
            let element = graph::load_exact_graph_element(
                index_relation,
                selection.element_tid,
                planner.storage,
            );
            let neighbors = graph::load_graph_neighbors(index_relation, element.neighbortid);
            plan_backlink_mutation(
                index_relation,
                planner,
                metric,
                &element,
                &neighbors,
                selection.layer,
            )
        })
        .filter(|mutation| mutation.neighbor_tid != page::ItemPointer::INVALID)
        .collect::<Vec<_>>();
    mutations.sort_unstable_by(|left, right| {
        compare_item_pointers(&left.neighbor_tid, &right.neighbor_tid)
            .then_with(|| left.layer.cmp(&right.layer))
            .then_with(|| {
                compare_item_pointers(&left.target_element_tid, &right.target_element_tid)
            })
    });
    mutations.dedup();
    mutations
}

unsafe fn apply_backlink_mutations(
    index_relation: pg_sys::Relation,
    mutations: &[BacklinkMutation],
    new_element_tid: page::ItemPointer,
    m: u16,
) -> Vec<LayerForwardSelection> {
    let mut retries = Vec::new();
    let mut start = 0;
    while start < mutations.len() {
        let block_number = mutations[start].neighbor_tid.block_number;
        let mut end = start + 1;
        while end < mutations.len() && mutations[end].neighbor_tid.block_number == block_number {
            end += 1;
        }

        retries.extend(unsafe {
            add_backlinks_on_page(index_relation, &mutations[start..end], new_element_tid, m)
        });
        start = end;
    }

    sort_and_dedup_forward_selections(&mut retries);
    retries
}

unsafe fn add_backlinks_on_page(
    index_relation: pg_sys::Relation,
    mutations: &[BacklinkMutation],
    new_element_tid: page::ItemPointer,
    m: u16,
) -> Vec<LayerForwardSelection> {
    if mutations.is_empty() {
        return Vec::new();
    }

    let block_number = mutations[0].neighbor_tid.block_number;
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to open backlink neighbor block {block_number}");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut changed = false;
    let mut retries = Vec::new();

    let mut start = 0;
    while start < mutations.len() {
        let neighbor_tid = mutations[start].neighbor_tid;
        let mut end = start + 1;
        while end < mutations.len() && mutations[end].neighbor_tid == neighbor_tid {
            end += 1;
        }

        let item_id = unsafe { &*shared::page_item_id(page_ptr, neighbor_tid.offset_number) };
        if item_id.lp_flags() == 0 {
            pgrx::error!(
                "tqhnsw backlink neighbor tuple slot {}/{} is unused",
                neighbor_tid.block_number,
                neighbor_tid.offset_number
            );
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!(
                "tqhnsw found invalid backlink neighbor tuple bounds on block {}",
                neighbor_tid.block_number
            );
        }

        let tuple_bytes =
            unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
        let mut neighbor = page::TqNeighborTuple::decode(tuple_bytes).unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to decode backlink neighbor tuple: {e}")
        });
        if neighbor.count as usize > neighbor.tids.len() {
            pgrx::error!(
                "tqhnsw backlink neighbor tuple count {} exceeds payload tid count {}",
                neighbor.count,
                neighbor.tids.len()
            );
        }

        let mut tuple_changed = false;
        for mutation in &mutations[start..end] {
            match apply_backlink_mutation(&mut neighbor.tids, new_element_tid, m, mutation) {
                BacklinkMutationOutcome::NoChange => {}
                BacklinkMutationOutcome::Changed => tuple_changed = true,
                BacklinkMutationOutcome::RetryReplan => retries.push(LayerForwardSelection {
                    layer: mutation.layer,
                    element_tid: mutation.target_element_tid,
                }),
            }
        }
        if !tuple_changed {
            start = end;
            continue;
        }

        let encoded = neighbor.encode().unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to encode backlink neighbor tuple: {e}")
        });
        if encoded.len() != tuple_len {
            pgrx::error!(
                "tqhnsw backlink neighbor tuple size changed from {} to {}",
                tuple_len,
                encoded.len()
            );
        }
        unsafe {
            ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), encoded.len());
        }
        changed = true;
        start = end;
    }

    if changed {
        unsafe { wal_txn.finish() };
    } else {
        std::mem::drop(wal_txn);
    }
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    retries
}

unsafe fn plan_backlink_mutation(
    index_relation: pg_sys::Relation,
    planner: &BacklinkPlanner<'_>,
    metric: &mut InsertSearchMetric,
    target_element: &graph::GraphElement,
    target_neighbors: &graph::GraphNeighbors,
    layer: u8,
) -> Option<BacklinkMutation> {
    let (start, end) =
        backlink_slot_bounds(usize::from(planner.m), target_neighbors.tids.len(), layer)?;
    let layer_slice = &target_neighbors.tids[start..end];
    if layer_slice.contains(&planner.new_element_tid) {
        return None;
    }

    if layer_slice.contains(&page::ItemPointer::INVALID) {
        return Some(BacklinkMutation {
            target_element_tid: target_element.tid,
            neighbor_tid: target_neighbors.tid,
            layer,
            kind: BacklinkMutationKind::InsertIfFree,
        });
    }

    let replacement_slice = unsafe {
        select_backlink_rewrite_slice(index_relation, planner, metric, target_element, layer_slice)
    };
    replacement_slice
        .contains(&planner.new_element_tid)
        .then_some(BacklinkMutation {
            target_element_tid: target_element.tid,
            neighbor_tid: target_neighbors.tid,
            layer,
            kind: BacklinkMutationKind::RewriteFullSlice {
                expected_slice: layer_slice.to_vec(),
                replacement_slice,
            },
        })
}

unsafe fn select_backlink_rewrite_slice(
    index_relation: pg_sys::Relation,
    planner: &BacklinkPlanner<'_>,
    metric: &mut InsertSearchMetric,
    target_element: &graph::GraphElement,
    existing_slice: &[page::ItemPointer],
) -> Vec<page::ItemPointer> {
    let new_candidate = ScoredBacklinkNode {
        node: planner.new_element_tid,
        score: unsafe {
            metric.score_new_backlink_candidate(planner.metadata, target_element, planner.new_tuple)
        },
        is_new: true,
    };
    let candidates = existing_slice
        .iter()
        .copied()
        .filter(|tid| *tid != page::ItemPointer::INVALID)
        .map(|tid| unsafe {
            let element = graph::load_exact_graph_element(index_relation, tid, planner.storage);
            ScoredBacklinkNode {
                node: tid,
                score: metric.score_existing_backlink_candidate(
                    planner.metadata,
                    target_element,
                    &element,
                ),
                is_new: false,
            }
        })
        .chain(std::iter::once(new_candidate))
        .collect::<Vec<_>>();
    select_best_backlink_candidates(candidates, existing_slice.len(), compare_item_pointers)
}

fn score_backlink_candidate(
    metadata: &page::MetadataPage,
    target_code: &[u8],
    candidate_code: &[u8],
) -> f32 {
    -crate::score_code_inner_product(
        metadata.dimensions as usize,
        metadata.bits,
        metadata.seed,
        target_code,
        candidate_code,
    )
}

fn apply_backlink_mutation(
    neighbor_tids: &mut [page::ItemPointer],
    new_element_tid: page::ItemPointer,
    m: u16,
    mutation: &BacklinkMutation,
) -> BacklinkMutationOutcome {
    let Some((start, end)) =
        backlink_slot_bounds(usize::from(m), neighbor_tids.len(), mutation.layer)
    else {
        return BacklinkMutationOutcome::NoChange;
    };
    let layer_slice = &mut neighbor_tids[start..end];

    match &mutation.kind {
        BacklinkMutationKind::InsertIfFree => {
            if insert_backlink_if_free(layer_slice, new_element_tid) {
                BacklinkMutationOutcome::Changed
            } else {
                BacklinkMutationOutcome::NoChange
            }
        }
        BacklinkMutationKind::RewriteFullSlice {
            expected_slice,
            replacement_slice,
        } => {
            if layer_slice.contains(&new_element_tid) {
                return BacklinkMutationOutcome::NoChange;
            }
            if insert_backlink_if_free(layer_slice, new_element_tid) {
                return BacklinkMutationOutcome::Changed;
            }
            if layer_slice != expected_slice.as_slice() {
                return BacklinkMutationOutcome::RetryReplan;
            }
            if layer_slice == replacement_slice.as_slice() {
                return BacklinkMutationOutcome::NoChange;
            }
            layer_slice.copy_from_slice(replacement_slice);
            BacklinkMutationOutcome::Changed
        }
    }
}

fn insert_backlink_if_free(
    layer_slice: &mut [page::ItemPointer],
    new_element_tid: page::ItemPointer,
) -> bool {
    if layer_slice.contains(&new_element_tid) {
        return false;
    }

    let Some(slot) = layer_slice
        .iter_mut()
        .find(|tid| **tid == page::ItemPointer::INVALID)
    else {
        return false;
    };
    *slot = new_element_tid;
    true
}

pub(super) fn selected_forward_slot_bounds(
    m: usize,
    total_slots: usize,
    layer: u8,
) -> Option<(usize, usize)> {
    let (start, end) = backlink_slot_bounds(m, total_slots, layer)?;
    if layer == 0 {
        return Some((start, start.saturating_add(m).min(end)));
    }
    Some((start, end))
}

pub(super) fn backlink_slot_bounds(
    m: usize,
    total_slots: usize,
    layer: u8,
) -> Option<(usize, usize)> {
    if total_slots == 0 {
        return None;
    }

    if layer == 0 {
        let end = m.saturating_mul(2).min(total_slots);
        return (end > 0).then_some((0, end));
    }

    let start = m
        .saturating_mul(2)
        .saturating_add((usize::from(layer).saturating_sub(1)).saturating_mul(m));
    if start >= total_slots {
        return None;
    }

    Some((start, start.saturating_add(m).min(total_slots)))
}

fn compare_item_pointers(left: &page::ItemPointer, right: &page::ItemPointer) -> Ordering {
    left.block_number
        .cmp(&right.block_number)
        .then_with(|| left.offset_number.cmp(&right.offset_number))
}

fn sort_and_dedup_forward_selections(selections: &mut Vec<LayerForwardSelection>) {
    selections.sort_unstable_by(|left, right| {
        compare_item_pointers(&left.element_tid, &right.element_tid)
            .then_with(|| left.layer.cmp(&right.layer))
    });
    selections.dedup();
}

unsafe fn append_heap_tuple(
    index_relation: pg_sys::Relation,
    tuple: &build::BuildTuple,
    level: u8,
    neighbor_tids: &[page::ItemPointer],
) -> page::ItemPointer {
    let neighbor_payload = page::TqNeighborTuple {
        count: u16::try_from(neighbor_tids.len()).expect("neighbor slot count should fit in u16"),
        tids: neighbor_tids.to_vec(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode neighbor tuple: {e}"));
    let required_bytes = page::raw_tuple_storage_bytes(neighbor_payload.len())
        + page::raw_tuple_storage_bytes(page::TqElementTuple::encoded_len(tuple.code.len()));
    let mut staged_page =
        page::DataPage::new(page::FIRST_DATA_BLOCK_NUMBER, pg_sys::BLCKSZ as usize);
    staged_page
        .insert_raw_tuple(neighbor_payload.clone())
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to stage aminsert neighbor tuple: {e}"));
    if !staged_page.can_fit_raw_tuple(page::TqElementTuple::encoded_len(tuple.code.len())) {
        pgrx::error!(
            "tqhnsw aminsert does not yet support tuples that require more than one fresh data page"
        );
    }

    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks > page::FIRST_DATA_BLOCK_NUMBER {
        existing_blocks - 1
    } else {
        P_NEW
    };
    let read_mode = if target_block == P_NEW {
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK
    } else {
        pg_sys::ReadBufferMode::RBM_NORMAL
    };
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            target_block,
            read_mode,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to allocate data buffer for aminsert");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    if target_block == P_NEW {
        unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };
    } else {
        let free_space = unsafe { pg_sys::PageGetFreeSpace(page_ptr) as usize };
        if free_space < required_bytes {
            std::mem::drop(wal_txn);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return unsafe {
                append_heap_tuple_to_new_page(index_relation, tuple, level, &neighbor_payload)
            };
        }
    }

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write neighbor tuple during aminsert");
    }

    let element_payload = page::TqElementTuple {
        level,
        deleted: false,
        heaptids: tuple.heap_tids.clone(),
        gamma: tuple.gamma,
        neighbortid: page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        code: tuple.code.clone(),
        binary_words: Vec::new(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode element tuple: {e}"));
    let element_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            element_payload.as_ptr().cast_mut().cast(),
            element_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if element_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write element tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: element_offset,
    }
}

unsafe fn append_heap_tuple_to_new_page(
    index_relation: pg_sys::Relation,
    tuple: &build::BuildTuple,
    level: u8,
    neighbor_payload: &[u8],
) -> page::ItemPointer {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            P_NEW,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to allocate fallback data buffer for aminsert");
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback neighbor tuple during aminsert");
    }

    let element_payload = page::TqElementTuple {
        level,
        deleted: false,
        heaptids: tuple.heap_tids.clone(),
        gamma: tuple.gamma,
        neighbortid: page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        code: tuple.code.clone(),
        binary_words: Vec::new(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode fallback element tuple: {e}"));
    let element_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            element_payload.as_ptr().cast_mut().cast(),
            element_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if element_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback element tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: element_offset,
    }
}

unsafe fn append_turbo_hot_cold_tuple(
    index_relation: pg_sys::Relation,
    tuple: &build::BuildTuple,
    level: u8,
    neighbor_tids: &[page::ItemPointer],
    layout: graph::TurboQuantHotColdLayout,
) -> page::ItemPointer {
    let persisted_binary_quantizer = crate::quant::prod::ProdQuantizer::cached(
        tuple.dimensions as usize,
        tuple.bits,
        tuple.seed,
    );
    let placeholder_payload = build::stage_v3_turbo_hot_build_payload(
        tuple,
        level,
        page::ItemPointer::INVALID,
        page::ItemPointer::INVALID,
        &persisted_binary_quantizer,
    );
    if placeholder_payload.hot.binary_words.len() != layout.binary_word_count {
        pgrx::error!(
            "tqhnsw derived TurboQuant V3 binary sidecar len {} does not match metadata layout {}",
            placeholder_payload.hot.binary_words.len(),
            layout.binary_word_count
        );
    }
    if placeholder_payload.rerank.code.len() != layout.rerank_code_len {
        pgrx::error!(
            "tqhnsw derived TurboQuant V3 rerank code len {} does not match metadata layout {}",
            placeholder_payload.rerank.code.len(),
            layout.rerank_code_len
        );
    }

    let neighbor_payload = page::TqNeighborTuple {
        count: u16::try_from(neighbor_tids.len()).expect("neighbor slot count should fit in u16"),
        tids: neighbor_tids.to_vec(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode TurboQuant V3 neighbor tuple: {e}"));
    let rerank_payload = placeholder_payload.rerank.encode();
    let hot_tuple_len = page::TqTurboHotTuple::encoded_len(layout.binary_word_count);
    let required_bytes = page::raw_tuple_storage_bytes(neighbor_payload.len())
        + page::raw_tuple_storage_bytes(rerank_payload.len())
        + page::raw_tuple_storage_bytes(hot_tuple_len);

    let mut staged_page =
        page::DataPage::new(page::FIRST_DATA_BLOCK_NUMBER, pg_sys::BLCKSZ as usize);
    staged_page
        .insert_raw_tuple(neighbor_payload.clone())
        .unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to stage TurboQuant V3 aminsert neighbor tuple: {e}")
        });
    staged_page
        .insert_raw_tuple(rerank_payload.clone())
        .unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to stage TurboQuant V3 aminsert rerank tuple: {e}")
        });
    if !staged_page.can_fit_raw_tuple(hot_tuple_len) {
        pgrx::error!(
            "tqhnsw aminsert does not yet support TurboQuant V3 tuples that require more than one fresh data page"
        );
    }

    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks > page::FIRST_DATA_BLOCK_NUMBER {
        existing_blocks - 1
    } else {
        P_NEW
    };
    let read_mode = if target_block == P_NEW {
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK
    } else {
        pg_sys::ReadBufferMode::RBM_NORMAL
    };
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            target_block,
            read_mode,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to allocate TurboQuant V3 data buffer for aminsert");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    if target_block == P_NEW {
        unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };
    } else {
        let free_space = unsafe { pg_sys::PageGetFreeSpace(page_ptr) as usize };
        if free_space < required_bytes {
            std::mem::drop(wal_txn);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return unsafe {
                append_turbo_hot_cold_tuple_to_new_page(
                    index_relation,
                    tuple,
                    level,
                    &neighbor_payload,
                    &rerank_payload,
                    layout,
                )
            };
        }
    }

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write TurboQuant V3 neighbor tuple during aminsert");
    }
    let rerank_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            rerank_payload.as_ptr().cast_mut().cast(),
            rerank_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if rerank_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write TurboQuant V3 rerank tuple during aminsert");
    }

    let hot_payload = build::stage_v3_turbo_hot_build_payload(
        tuple,
        level,
        page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        page::ItemPointer {
            block_number,
            offset_number: rerank_offset,
        },
        &persisted_binary_quantizer,
    )
    .hot
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode TurboQuant V3 hot tuple: {e}"));
    let hot_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            hot_payload.as_ptr().cast_mut().cast(),
            hot_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if hot_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write TurboQuant V3 hot tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: hot_offset,
    }
}

unsafe fn append_turbo_hot_cold_tuple_to_new_page(
    index_relation: pg_sys::Relation,
    tuple: &build::BuildTuple,
    level: u8,
    neighbor_payload: &[u8],
    rerank_payload: &[u8],
    layout: graph::TurboQuantHotColdLayout,
) -> page::ItemPointer {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            P_NEW,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to allocate fallback TurboQuant V3 data buffer for aminsert");
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!(
            "tqhnsw failed to write fallback TurboQuant V3 neighbor tuple during aminsert"
        );
    }
    let rerank_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            rerank_payload.as_ptr().cast_mut().cast(),
            rerank_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if rerank_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback TurboQuant V3 rerank tuple during aminsert");
    }

    let persisted_binary_quantizer = crate::quant::prod::ProdQuantizer::cached(
        tuple.dimensions as usize,
        tuple.bits,
        tuple.seed,
    );
    let hot_payload = build::stage_v3_turbo_hot_build_payload(
        tuple,
        level,
        page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        page::ItemPointer {
            block_number,
            offset_number: rerank_offset,
        },
        &persisted_binary_quantizer,
    )
    .hot
    .encode()
    .unwrap_or_else(|e| {
        pgrx::error!("tqhnsw failed to encode fallback TurboQuant V3 hot tuple: {e}")
    });
    if hot_payload.len() != page::TqTurboHotTuple::encoded_len(layout.binary_word_count) {
        pgrx::error!(
            "tqhnsw fallback TurboQuant V3 hot tuple len {} does not match metadata layout {}",
            hot_payload.len(),
            page::TqTurboHotTuple::encoded_len(layout.binary_word_count)
        );
    }
    let hot_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            hot_payload.as_ptr().cast_mut().cast(),
            hot_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if hot_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback TurboQuant V3 hot tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: hot_offset,
    }
}

unsafe fn derive_pq_fastscan_search_code_for_insert(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    tuple: &build::BuildTuple,
    layout: graph::PqFastScanLayout,
) -> Vec<u8> {
    if metadata.grouped_codebook_head == page::ItemPointer::INVALID
        || metadata.search_subvector_count == 0
        || metadata.search_subvector_dim == 0
    {
        pgrx::error!("{PQ_FASTSCAN_CODEBOOK_METADATA_UNAVAILABLE}");
    }
    let source_vector = tuple
        .source_vector
        .as_deref()
        .unwrap_or_else(|| pgrx::error!("tqhnsw PqFastScan live insert requires raw source data"));
    let model = unsafe { graph::load_grouped_codebook_model(index_relation, metadata) };
    let search_code =
        graph::derive_grouped_search_code_from_source(metadata, &model, source_vector)
            .unwrap_or_else(|e| {
                pgrx::error!("tqhnsw failed to derive PqFastScan search code: {e}")
            });
    if search_code.len() != layout.search_code_len {
        pgrx::error!(
            "tqhnsw derived PqFastScan search code len {} does not match metadata layout {}",
            search_code.len(),
            layout.search_code_len
        );
    }
    search_code
}

unsafe fn bootstrap_empty_pq_fastscan_flush_output(
    index_relation: pg_sys::Relation,
    tuple: &build::BuildTuple,
) -> build::BuildFlushOutput {
    let options = unsafe { options::relation_options(index_relation) };
    let state = build::BuildState {
        options,
        indexed_vector_kind: source::IndexedVectorKind::Ecvector,
        page_size: pg_sys::BLCKSZ as usize,
        scanned_tuples: tuple.heap_tids.len(),
        heap_tuples: vec![tuple.clone()],
        dimensions: Some(tuple.dimensions),
        bits: Some(tuple.bits),
        seed: Some(tuple.seed),
    };

    build::default_pq_fastscan_flush_output(&state)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to bootstrap empty PqFastScan index: {e}"))
}

unsafe fn append_pq_fastscan_tuple(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    tuple: &build::BuildTuple,
    level: u8,
    neighbor_tids: &[page::ItemPointer],
    layout: graph::PqFastScanLayout,
) -> page::ItemPointer {
    let search_code = unsafe {
        derive_pq_fastscan_search_code_for_insert(index_relation, metadata, tuple, layout)
    };
    let persisted_binary_quantizer = crate::quant::prod::ProdQuantizer::cached(
        tuple.dimensions as usize,
        tuple.bits,
        tuple.seed,
    );
    let placeholder_payload = build::stage_v2_grouped_build_payload(
        tuple,
        level,
        page::ItemPointer::INVALID,
        page::ItemPointer::INVALID,
        search_code.clone(),
        &persisted_binary_quantizer,
    );
    if placeholder_payload.hot.binary_words.len() != layout.binary_word_count {
        pgrx::error!(
            "tqhnsw derived PqFastScan binary sidecar len {} does not match metadata layout {}",
            placeholder_payload.hot.binary_words.len(),
            layout.binary_word_count
        );
    }

    let neighbor_payload = page::TqNeighborTuple {
        count: u16::try_from(neighbor_tids.len()).expect("neighbor slot count should fit in u16"),
        tids: neighbor_tids.to_vec(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode grouped neighbor tuple: {e}"));
    let rerank_payload = placeholder_payload.rerank.encode();
    let hot_tuple_len =
        page::TqGroupedHotTuple::encoded_len(layout.binary_word_count, search_code.len());
    let required_bytes = page::raw_tuple_storage_bytes(neighbor_payload.len())
        + page::raw_tuple_storage_bytes(rerank_payload.len())
        + page::raw_tuple_storage_bytes(hot_tuple_len);

    let mut staged_page =
        page::DataPage::new(page::FIRST_DATA_BLOCK_NUMBER, pg_sys::BLCKSZ as usize);
    staged_page
        .insert_raw_tuple(neighbor_payload.clone())
        .unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to stage PqFastScan aminsert neighbor tuple: {e}")
        });
    staged_page
        .insert_raw_tuple(rerank_payload.clone())
        .unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to stage PqFastScan aminsert rerank tuple: {e}")
        });
    if !staged_page.can_fit_raw_tuple(hot_tuple_len) {
        pgrx::error!(
            "tqhnsw aminsert does not yet support PqFastScan tuples that require more than one fresh data page"
        );
    }

    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks > page::FIRST_DATA_BLOCK_NUMBER {
        existing_blocks - 1
    } else {
        P_NEW
    };
    let read_mode = if target_block == P_NEW {
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK
    } else {
        pg_sys::ReadBufferMode::RBM_NORMAL
    };
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            target_block,
            read_mode,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to allocate PqFastScan data buffer for aminsert");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    if target_block == P_NEW {
        unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };
    } else {
        let free_space = unsafe { pg_sys::PageGetFreeSpace(page_ptr) as usize };
        if free_space < required_bytes {
            std::mem::drop(wal_txn);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return unsafe {
                append_pq_fastscan_tuple_to_new_page(
                    index_relation,
                    tuple,
                    level,
                    &neighbor_payload,
                    &rerank_payload,
                    search_code,
                )
            };
        }
    }

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write PqFastScan neighbor tuple during aminsert");
    }
    let rerank_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            rerank_payload.as_ptr().cast_mut().cast(),
            rerank_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if rerank_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write PqFastScan rerank tuple during aminsert");
    }

    let payload = build::stage_v2_grouped_build_payload(
        tuple,
        level,
        page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        page::ItemPointer {
            block_number,
            offset_number: rerank_offset,
        },
        search_code,
        &persisted_binary_quantizer,
    );
    let hot_payload = payload
        .hot
        .encode()
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode PqFastScan hot tuple: {e}"));
    let hot_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            hot_payload.as_ptr().cast_mut().cast(),
            hot_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if hot_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write PqFastScan hot tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: hot_offset,
    }
}

unsafe fn append_pq_fastscan_tuple_to_new_page(
    index_relation: pg_sys::Relation,
    tuple: &build::BuildTuple,
    level: u8,
    neighbor_payload: &[u8],
    rerank_payload: &[u8],
    search_code: Vec<u8>,
) -> page::ItemPointer {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            P_NEW,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to allocate fallback PqFastScan data buffer for aminsert");
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback PqFastScan neighbor tuple during aminsert");
    }
    let rerank_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            rerank_payload.as_ptr().cast_mut().cast(),
            rerank_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if rerank_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback PqFastScan rerank tuple during aminsert");
    }

    let persisted_binary_quantizer = crate::quant::prod::ProdQuantizer::cached(
        tuple.dimensions as usize,
        tuple.bits,
        tuple.seed,
    );
    let hot_payload = build::stage_v2_grouped_build_payload(
        tuple,
        level,
        page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        page::ItemPointer {
            block_number,
            offset_number: rerank_offset,
        },
        search_code,
        &persisted_binary_quantizer,
    )
    .hot
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode fallback PqFastScan hot tuple: {e}"));
    let hot_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            hot_payload.as_ptr().cast_mut().cast(),
            hot_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if hot_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback PqFastScan hot tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: hot_offset,
    }
}

unsafe fn find_duplicate_element_tid(
    index_relation: pg_sys::Relation,
    _heap_relation: pg_sys::Relation,
    dimensions: u16,
    bits: u8,
    gamma: f32,
    code_len: usize,
    code: &[u8],
) -> Option<page::ItemPointer> {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    if block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        return None;
    }

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let line_pointer_count = shared::page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let item_id = unsafe { &*shared::page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid tuple bounds while scanning block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                continue;
            }

            let element = page::TqElementTuple::decode(tuple_bytes, code_len).unwrap_or_else(|e| {
                pgrx::error!("tqhnsw failed to decode candidate duplicate tuple: {e}")
            });
            if element.deleted || element.heaptids.is_empty() {
                continue;
            }
            if element.code == code && element.gamma.to_bits() == gamma.to_bits() {
                unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
                return Some(page::ItemPointer {
                    block_number,
                    offset_number: offset,
                });
            }
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    let _ = dimensions;
    let _ = bits;
    None
}

unsafe fn find_duplicate_turbo_hot_element_tid(
    index_relation: pg_sys::Relation,
    gamma: f32,
    code: &[u8],
    layout: graph::TurboQuantHotColdLayout,
) -> Option<page::ItemPointer> {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    if block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        return None;
    }

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let line_pointer_count = shared::page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let item_id = unsafe { &*shared::page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid TurboQuant V3 tuple bounds while scanning block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            if tuple_bytes.first().copied() != Some(page::TQ_TURBO_HOT_TAG) {
                continue;
            }

            let element = page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                .unwrap_or_else(|e| {
                    pgrx::error!("tqhnsw failed to decode candidate TurboQuant V3 tuple: {e}")
                });
            if element.deleted || element.heaptids.is_empty() {
                continue;
            }

            let rerank = unsafe {
                graph::load_rerank_payload(
                    index_relation,
                    element.reranktid,
                    layout.rerank_code_len,
                )
            };
            if rerank.code == code && rerank.gamma.to_bits() == gamma.to_bits() {
                unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
                return Some(page::ItemPointer {
                    block_number,
                    offset_number: offset,
                });
            }
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    None
}

unsafe fn find_duplicate_grouped_element_tid(
    index_relation: pg_sys::Relation,
    gamma: f32,
    code: &[u8],
    layout: graph::PqFastScanLayout,
) -> Option<page::ItemPointer> {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    if block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        return None;
    }

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let line_pointer_count = shared::page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let item_id = unsafe { &*shared::page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid grouped tuple bounds while scanning block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            if tuple_bytes.first().copied() != Some(page::TQ_GROUPED_HOT_TAG) {
                continue;
            }

            let element = page::TqGroupedHotTuple::decode(
                tuple_bytes,
                layout.binary_word_count,
                layout.search_code_len,
            )
            .unwrap_or_else(|e| {
                pgrx::error!("tqhnsw failed to decode candidate grouped duplicate tuple: {e}")
            });
            if element.deleted || element.heaptids.is_empty() {
                continue;
            }

            let rerank = unsafe {
                graph::load_grouped_rerank_payload(index_relation, element.reranktid, layout)
            };
            if rerank.code == code && rerank.gamma.to_bits() == gamma.to_bits() {
                unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
                return Some(page::ItemPointer {
                    block_number,
                    offset_number: offset,
                });
            }
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    None
}

unsafe fn coalesce_duplicate_heap_tid(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
    heap_tid: page::ItemPointer,
) {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            element_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!(
            "tqhnsw failed to open duplicate element block {}",
            element_tid.block_number
        );
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let item_id = unsafe { &*shared::page_item_id(page_ptr, element_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        pgrx::error!("tqhnsw duplicate element tuple slot is unused");
    }
    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        pgrx::error!(
            "tqhnsw found invalid duplicate tuple bounds on block {}",
            element_tid.block_number
        );
    }

    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    let mut element = page::TqElementTuple::decode(tuple_bytes, code_len)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode duplicate element tuple: {e}"));
    if element.heaptids.contains(&heap_tid) {
        unsafe { wal_txn.finish() };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return;
    }
    if element.heaptids.len() >= page::HEAPTID_INLINE_CAPACITY {
        pgrx::error!(
            "tqhnsw aminsert supports at most {} duplicate heap tids per encoded vector",
            page::HEAPTID_INLINE_CAPACITY
        );
    }
    element.heaptids.push(heap_tid);
    let encoded = element
        .encode()
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode coalesced element tuple: {e}"));
    if encoded.len() != tuple_len {
        pgrx::error!(
            "tqhnsw duplicate element tuple size changed from {} to {}",
            tuple_len,
            encoded.len()
        );
    }
    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), encoded.len());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

unsafe fn coalesce_duplicate_turbo_hot_heap_tid(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    layout: graph::TurboQuantHotColdLayout,
    heap_tid: page::ItemPointer,
) {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            element_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!(
            "tqhnsw failed to open duplicate TurboQuant V3 element block {}",
            element_tid.block_number
        );
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let item_id = unsafe { &*shared::page_item_id(page_ptr, element_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        pgrx::error!("tqhnsw duplicate TurboQuant V3 tuple slot is unused");
    }
    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        pgrx::error!(
            "tqhnsw found invalid duplicate TurboQuant V3 tuple bounds on block {}",
            element_tid.block_number
        );
    }

    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    let mut element = page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
        .unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to decode duplicate TurboQuant V3 tuple: {e}")
        });
    if element.heaptids.contains(&heap_tid) {
        unsafe { wal_txn.finish() };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return;
    }
    if element.heaptids.len() >= page::HEAPTID_INLINE_CAPACITY {
        pgrx::error!(
            "tqhnsw aminsert supports at most {} duplicate heap tids per encoded vector",
            page::HEAPTID_INLINE_CAPACITY
        );
    }
    element.heaptids.push(heap_tid);
    let encoded = element.encode().unwrap_or_else(|e| {
        pgrx::error!("tqhnsw failed to encode coalesced TurboQuant V3 tuple: {e}")
    });
    if encoded.len() != tuple_len {
        pgrx::error!(
            "tqhnsw duplicate TurboQuant V3 tuple size changed from {} to {}",
            tuple_len,
            encoded.len()
        );
    }
    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), encoded.len());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

unsafe fn coalesce_duplicate_grouped_heap_tid(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    layout: graph::PqFastScanLayout,
    heap_tid: page::ItemPointer,
) {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            element_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!(
            "tqhnsw failed to open duplicate PqFastScan element block {}",
            element_tid.block_number
        );
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let item_id = unsafe { &*shared::page_item_id(page_ptr, element_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        pgrx::error!("tqhnsw duplicate PqFastScan element tuple slot is unused");
    }
    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        pgrx::error!(
            "tqhnsw found invalid grouped duplicate tuple bounds on block {}",
            element_tid.block_number
        );
    }

    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    let mut element = page::TqGroupedHotTuple::decode(
        tuple_bytes,
        layout.binary_word_count,
        layout.search_code_len,
    )
    .unwrap_or_else(|e| {
        pgrx::error!("tqhnsw failed to decode duplicate PqFastScan element tuple: {e}")
    });
    if element.heaptids.contains(&heap_tid) {
        unsafe { wal_txn.finish() };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return;
    }
    if element.heaptids.len() >= page::HEAPTID_INLINE_CAPACITY {
        pgrx::error!(
            "tqhnsw aminsert supports at most {} duplicate heap tids per encoded vector",
            page::HEAPTID_INLINE_CAPACITY
        );
    }
    element.heaptids.push(heap_tid);
    let encoded = element.encode().unwrap_or_else(|e| {
        pgrx::error!("tqhnsw failed to encode coalesced PqFastScan element tuple: {e}")
    });
    if encoded.len() != tuple_len {
        pgrx::error!(
            "tqhnsw duplicate PqFastScan element tuple size changed from {} to {}",
            tuple_len,
            encoded.len()
        );
    }
    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), encoded.len());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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

    #[test]
    fn resolve_insert_format_adapter_accepts_scalar_v1() {
        let metadata = page::MetadataPage::current_v1_scalar(page::CurrentFormatMetadata {
            m: 8,
            ef_construction: 64,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 16,
            bits: 4,
            max_level: 0,
            seed: 42,
            inserted_since_rebuild: 0,
            persisted_binary_sidecar: false,
        });
        let format = match graph::GraphStorageDescriptor::from_metadata(&metadata).unwrap() {
            graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                InsertFormatAdapter::TurboQuant { code_len }
            }
            graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                InsertFormatAdapter::TurboQuantHotCold(layout)
            }
            graph::GraphStorageDescriptor::PqFastScan(layout) => {
                InsertFormatAdapter::PqFastScan(layout)
            }
        };
        assert_eq!(
            format,
            InsertFormatAdapter::TurboQuant {
                code_len: crate::code_len(16, 4),
            }
        );
    }

    #[test]
    fn resolve_insert_format_adapter_recognizes_pq_fastscan() {
        let metadata = page::MetadataPage {
            m: 8,
            ef_construction: 64,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 16,
            bits: 4,
            max_level: 0,
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
        let format = match graph::GraphStorageDescriptor::from_metadata(&metadata).unwrap() {
            graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                InsertFormatAdapter::TurboQuant { code_len }
            }
            graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                InsertFormatAdapter::TurboQuantHotCold(layout)
            }
            graph::GraphStorageDescriptor::PqFastScan(layout) => {
                InsertFormatAdapter::PqFastScan(layout)
            }
        };

        assert_eq!(
            format,
            InsertFormatAdapter::PqFastScan(graph::PqFastScanLayout {
                binary_word_count: 0,
                search_code_len: 1,
                rerank_code_len: crate::code_len(16, 4),
            })
        );
    }

    #[test]
    fn rewrite_full_slice_requests_retry_when_snapshot_drifted() {
        let new_element_tid = tid(9, 9);
        let mut layer_tids = vec![tid(1, 1), tid(1, 2), tid(1, 3), tid(1, 4)];
        let mutation = BacklinkMutation {
            target_element_tid: tid(7, 1),
            neighbor_tid: tid(8, 1),
            layer: 0,
            kind: BacklinkMutationKind::RewriteFullSlice {
                expected_slice: layer_tids.clone(),
                replacement_slice: vec![new_element_tid, tid(1, 1), tid(1, 2), tid(1, 3)],
            },
        };

        layer_tids[0] = tid(5, 5);

        assert_eq!(
            apply_backlink_mutation(&mut layer_tids, new_element_tid, 2, &mutation),
            BacklinkMutationOutcome::RetryReplan,
        );
        assert_eq!(
            layer_tids,
            vec![tid(5, 5), tid(1, 2), tid(1, 3), tid(1, 4)],
            "a stale full-slice plan should leave the live layer unchanged and request replanning",
        );
    }

    #[test]
    fn rewrite_full_slice_applies_after_replanning_against_current_slice() {
        let new_element_tid = tid(9, 9);
        let mut layer_tids = vec![tid(5, 5), tid(1, 2), tid(1, 3), tid(1, 4)];
        let mutation = BacklinkMutation {
            target_element_tid: tid(7, 1),
            neighbor_tid: tid(8, 1),
            layer: 0,
            kind: BacklinkMutationKind::RewriteFullSlice {
                expected_slice: layer_tids.clone(),
                replacement_slice: vec![new_element_tid, tid(5, 5), tid(1, 2), tid(1, 3)],
            },
        };

        assert_eq!(
            apply_backlink_mutation(&mut layer_tids, new_element_tid, 2, &mutation),
            BacklinkMutationOutcome::Changed,
        );
        assert_eq!(
            layer_tids,
            vec![new_element_tid, tid(5, 5), tid(1, 2), tid(1, 3)],
            "a replanned full-slice mutation should admit the new node against the current live slice",
        );
    }

    #[test]
    fn choose_insert_level_for_page_size_respects_supplied_page_size() {
        let heap_tid = tid(1, 1);
        let full_page_level = choose_insert_level_for_page_size(
            8,
            42,
            heap_tid,
            crate::code_len(1536, 4),
            pg_sys::BLCKSZ as usize,
        );
        let tighter_page_level =
            choose_insert_level_for_page_size(8, 42, heap_tid, crate::code_len(1536, 4), 1024);

        assert!(
            tighter_page_level <= full_page_level,
            "smaller build pages should never admit a higher sampled level than BLCKSZ pages",
        );
    }
}
