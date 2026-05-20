use std::{collections::HashSet, ffi::c_void};

use pgrx::{itemptr::item_pointer_set_all, pg_sys, PgBox};

use super::{graph, options, page, search, shared, source};
use crate::am::common::heap_slot::HeapSlotReader;
#[cfg(any(test, feature = "pg_test"))]
use crate::storage::relation_guard::{HeapRelationGuard, IndexRelationGuard};
use crate::storage::{buffer_guard::LockedBufferGuard, slot_guard::TupleTableSlotGuard, wal};
type BulkDeleteCallback =
    unsafe extern "C-unwind" fn(itemptr: pg_sys::ItemPointer, state: *mut c_void) -> bool;

#[derive(Debug, Clone, Copy)]
struct VacuumIndexRelation {
    relation: pg_sys::Relation,
}

impl VacuumIndexRelation {
    unsafe fn new(relation: pg_sys::Relation) -> Self {
        if relation.is_null() {
            pgrx::error!("ec_hnsw vacuum received a null index relation");
        }
        Self { relation }
    }

    fn as_ptr(self) -> pg_sys::Relation {
        self.relation
    }

    fn metadata(self) -> page::MetadataPage {
        // SAFETY: This wrapper is constructed only for the live vacuum callback
        // index relation and metadata reads do not outlive that callback.
        unsafe { shared::read_metadata_page(self.relation) }
    }

    fn main_fork_block_count(self) -> pg_sys::BlockNumber {
        // SAFETY: This wrapper owns the invariant that the relation pointer is
        // live for the current vacuum callback.
        unsafe {
            pg_sys::RelationGetNumberOfBlocksInFork(self.relation, pg_sys::ForkNumber::MAIN_FORKNUM)
        }
    }

    fn read_main_locked(
        self,
        block_number: pg_sys::BlockNumber,
        lockmode: i32,
        context: &str,
    ) -> LockedBufferGuard {
        // SAFETY: The relation is live for this vacuum callback; the guard owns
        // the returned pin and lock until drop.
        unsafe {
            LockedBufferGuard::read_main(
                self.relation,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                lockmode,
            )
        }
        .unwrap_or_else(|| pgrx::error!("ec_hnsw failed to open {context} block {block_number}"))
    }

    fn begin_page_rewrite(self, buffer: &LockedBufferGuard) -> VacuumPageRewrite {
        // SAFETY: The buffer belongs to this live relation and remains locked
        // while the returned rewrite guard is active.
        unsafe { VacuumPageRewrite::start(self.relation, buffer) }
    }
}

struct VacuumPageRewrite {
    wal_txn: Option<wal::GenericXLogTxn>,
    page_ptr: *mut u8,
}

impl VacuumPageRewrite {
    unsafe fn start(relation: pg_sys::Relation, buffer: &LockedBufferGuard) -> Self {
        // SAFETY: The caller guarantees `relation` is live and `buffer` belongs
        // to that relation for the GenericXLog transaction.
        let (wal_txn, page_ptr) = unsafe {
            let mut wal_txn = wal::GenericXLogTxn::start(relation);
            let page_ptr = wal_txn.register_locked_buffer_full_image(&buffer);
            (wal_txn, page_ptr)
        };
        Self {
            wal_txn: Some(wal_txn),
            page_ptr: page_ptr.cast::<u8>(),
        }
    }

    fn page_ptr(&self) -> *mut u8 {
        self.page_ptr
    }

    fn finish(mut self) {
        let wal_txn = self
            .wal_txn
            .take()
            .expect("vacuum page rewrite WAL transaction should be present");
        wal_txn.finish();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VacuumFormatAdapter {
    TurboQuant { code_len: usize },
    TurboQuantHotCold(graph::TurboQuantHotColdLayout),
    PqFastScan(graph::PqFastScanLayout),
}

#[derive(Debug)]
enum VacuumSearchMetric {
    Code,
    Source(VacuumHeapSourceScorer),
}

#[derive(Debug)]
struct VacuumHeapSourceScorer {
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: TupleTableSlotGuard,
    source_attribute: source::SourceAttribute,
}

impl VacuumHeapSourceScorer {
    unsafe fn new(heap_relation: pg_sys::Relation, source_column: &str) -> Self {
        // SAFETY: `heap_relation` is the heap relation passed to vacuum; source
        // attribute resolution only reads metadata, and the resolved attribute
        // is immediately bound to the same relation in the scorer.
        unsafe {
            let source_attribute = source::resolve_source_attribute(
                heap_relation,
                source_column,
                "build_source_column",
                source::SourceTypePolicy::BuildSource,
            );
            Self::new_with_attribute(heap_relation, source_attribute)
        }
    }

    unsafe fn new_with_attribute(
        heap_relation: pg_sys::Relation,
        source_attribute: source::SourceAttribute,
    ) -> Self {
        let slot = TupleTableSlotGuard::single_for_heap(heap_relation).unwrap_or_else(|| {
            pgrx::error!("ec_hnsw vacuum failed to allocate a heap source slot")
        });

        Self {
            heap_relation,
            snapshot: std::ptr::addr_of_mut!(pg_sys::SnapshotAnyData),
            slot,
            source_attribute,
        }
    }

    fn heap_reader(&mut self) -> HeapSlotReader<'_> {
        // SAFETY: `self.heap_relation`, snapshot, and tuple slot belong to this
        // scorer for the vacuum planning scope.
        unsafe {
            HeapSlotReader::from_raw(
                self.heap_relation,
                self.snapshot,
                self.slot.as_ptr(),
                "ec_hnsw",
            )
        }
        .unwrap_or_else(|error| pgrx::error!("{error}"))
    }

    fn averaged_source_vector(
        &mut self,
        heap_tids: &[page::ItemPointer],
        label: &str,
    ) -> Option<Vec<f32>> {
        let mut representative: Option<Vec<f32>> = None;
        let mut count = 0usize;

        for heap_tid in heap_tids.iter().copied() {
            let source_attribute = self.source_attribute;
            let mut reader = self.heap_reader();
            source::with_source_from_heap_row_reader(
                &mut reader,
                heap_tid,
                source_attribute,
                label,
                |source| match representative.as_mut() {
                    Some(existing) => {
                        source::average_source_representatives(
                            existing,
                            count,
                            source.as_slice(),
                            1,
                        );
                        count += 1;
                    }
                    None => {
                        representative = Some(source.as_slice().to_vec());
                        count = 1;
                    }
                },
            );
            reader.clear();
        }

        representative
    }

    fn score_graph_element_pair(
        &mut self,
        source_element: &graph::GraphElement,
        candidate_element: &graph::GraphElement,
    ) -> Option<f32> {
        if source_element.deleted
            || source_element.heaptids.is_empty()
            || candidate_element.deleted
            || candidate_element.heaptids.is_empty()
        {
            return None;
        }

        let source_vector = self.averaged_source_vector(
            &source_element.heaptids,
            "vacuum repair source-backed element",
        )?;
        let candidate_vector = self.averaged_source_vector(
            &candidate_element.heaptids,
            "vacuum repair source-backed candidate",
        )?;
        Some(source::negative_inner_product(
            &source_vector,
            &candidate_vector,
        ))
    }
}

impl VacuumSearchMetric {
    unsafe fn for_relation(
        index_relation: pg_sys::Relation,
        heap_relation: pg_sys::Relation,
    ) -> Self {
        if heap_relation.is_null() {
            pgrx::error!("ec_hnsw vacuum requires a heap relation for source-backed indexes");
        }

        // SAFETY: Both relations are live for the vacuum callback; this block
        // reads relation options and resolves any heap source attribute needed
        // for repair scoring.
        let index_options = unsafe { options::relation_options(index_relation) };
        match index_options.build_source_column.as_deref() {
            Some(source_column) => {
                // SAFETY: The heap relation is live and the configured source
                // column is resolved through catalog metadata.
                Self::Source(unsafe {
                    let source_attribute = source::resolve_source_attribute(
                        heap_relation,
                        source_column,
                        "build_source_column",
                        source::SourceTypePolicy::BuildSource,
                    );
                    VacuumHeapSourceScorer::new_with_attribute(heap_relation, source_attribute)
                })
            }
            None => {
                // SAFETY: The heap and index relations are live; this resolves
                // the indexed vector attribute from PostgreSQL metadata.
                let indexed_attribute = unsafe {
                    source::resolve_indexed_vector_attribute(
                        heap_relation,
                        index_relation,
                        "indexed column",
                    )
                };
                match indexed_attribute.kind {
                    // SAFETY: The indexed ecvector attribute can be read through
                    // the heap relation during source-backed repair scoring.
                    source::IndexedVectorKind::Ecvector => {
                        Self::Source(VacuumHeapSourceScorer::new_with_attribute(
                            heap_relation,
                            source::SourceAttribute {
                                attnum: indexed_attribute.attnum,
                                kind: source::SourceDatumKind::Ecvector,
                            },
                        ))
                    }
                    source::IndexedVectorKind::Tqvector => Self::Code,
                }
            }
        }
    }

    unsafe fn score_graph_element(
        &mut self,
        metadata: &page::MetadataPage,
        source_element: &graph::GraphElement,
        candidate_element: &graph::GraphElement,
    ) -> Option<f32> {
        match self {
            Self::Code => {
                score_vacuum_code_element(metadata, &source_element.code, candidate_element)
            }
            Self::Source(scorer) => {
                scorer.score_graph_element_pair(source_element, candidate_element)
            }
        }
    }
}

impl VacuumFormatAdapter {
    fn graph_storage(self) -> graph::GraphStorageDescriptor {
        match self {
            Self::TurboQuant { code_len } => graph::GraphStorageDescriptor::TurboQuant { code_len },
            Self::TurboQuantHotCold(layout) => {
                graph::GraphStorageDescriptor::TurboQuantHotCold(layout)
            }
            Self::PqFastScan(layout) => graph::GraphStorageDescriptor::PqFastScan(layout),
        }
    }

    unsafe fn vacuum_cleanup(
        self,
        index: VacuumIndexRelation,
        stats: *mut pg_sys::IndexBulkDeleteResult,
    ) -> *mut pg_sys::IndexBulkDeleteResult {
        let _ = self;
        // SAFETY: PostgreSQL supplied the live index relation and optional
        // stats pointer for this vacuum cleanup callback.
        unsafe { shared::ec_hnsw_noop_vacuum_stats(index.as_ptr(), stats) }
    }

    unsafe fn repair_graph_connections(
        self,
        index: VacuumIndexRelation,
        heap_relation: pg_sys::Relation,
        deleted_tids: &[page::ItemPointer],
    ) {
        // SAFETY: The adapter storage descriptor matches the index metadata
        // resolved for this vacuum pass.
        unsafe {
            repair_graph_connections_with_storage(
                index,
                heap_relation,
                self.graph_storage(),
                deleted_tids,
            )
        }
    }

    unsafe fn finalize_fully_dead_elements(
        self,
        index: VacuumIndexRelation,
        deleted_tids: &[page::ItemPointer],
    ) {
        // SAFETY: The adapter storage descriptor matches the index metadata
        // resolved for this vacuum pass.
        unsafe {
            finalize_fully_dead_elements_with_storage(index, self.graph_storage(), deleted_tids)
        }
    }
}

#[derive(Debug, Clone)]
enum ElementVacuumUpdate {
    TurboQuant {
        tid: page::ItemPointer,
        tuple: page::TqElementTuple,
    },
    TurboQuantHot {
        tid: page::ItemPointer,
        tuple: page::TqTurboHotTuple,
    },
    PqFastScanHot {
        tid: page::ItemPointer,
        tuple: page::TqGroupedHotTuple,
    },
}

#[derive(Debug, Clone)]
struct NeighborVacuumUpdate {
    tid: page::ItemPointer,
    tuple: page::TqNeighborTuple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LayerRepairRequest {
    source_tid: page::ItemPointer,
    neighbor_tid: page::ItemPointer,
    layer: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LayerRepairPlan {
    neighbor_tid: page::ItemPointer,
    source_level: u8,
    layer: u8,
    replacement_tids: Vec<page::ItemPointer>,
}

#[derive(Debug, Clone, Copy)]
struct LinearRepairPlanner<'a> {
    metadata: &'a page::MetadataPage,
    storage: graph::GraphStorageDescriptor,
    source: &'a graph::GraphElement,
    deleted_tids: &'a HashSet<page::ItemPointer>,
    existing_set: &'a HashSet<page::ItemPointer>,
    layer: u8,
}

#[derive(Debug, Clone, Copy)]
struct RepairSearchPlanner<'a> {
    metadata: &'a page::MetadataPage,
    storage: graph::GraphStorageDescriptor,
    source: &'a graph::GraphElement,
    layer: u8,
    deleted_tids: &'a HashSet<page::ItemPointer>,
    existing_layer: &'a [page::ItemPointer],
    existing_set: &'a HashSet<page::ItemPointer>,
    target_len: usize,
}

#[derive(Debug, Default)]
struct PagePass1Plan {
    live_elements: usize,
    removed_heap_tids: usize,
    finalize_tids: Vec<page::ItemPointer>,
    updates: Vec<ElementVacuumUpdate>,
}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: pg_sys::IndexBulkDeleteCallback,
    callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    // SAFETY: PostgreSQL invokes ambulkdelete with callback-duration relation,
    // stats, callback, and callback state pointers.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let index = VacuumIndexRelation::new((*info).index);
            let metadata = index.metadata();
            let format = resolve_vacuum_format_adapter(index, &metadata)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            let Some(callback) = callback else {
                return shared::ec_hnsw_noop_vacuum_stats(index.as_ptr(), stats);
            };
            run_bulkdelete_with_adapter(
                format,
                index,
                (*info).heaprel,
                stats,
                callback,
                callback_state,
            )
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    // SAFETY: PostgreSQL invokes amvacuumcleanup with callback-duration relation
    // and stats pointers.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let index = VacuumIndexRelation::new((*info).index);
            let metadata = index.metadata();
            let format = resolve_vacuum_format_adapter(index, &metadata)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            format.vacuum_cleanup(index, stats)
        })
    }
}

fn resolve_vacuum_format_adapter(
    index: VacuumIndexRelation,
    metadata: &page::MetadataPage,
) -> Result<VacuumFormatAdapter, String> {
    // SAFETY: `index_relation` is live for vacuum and `metadata` is the page
    // snapshot used to interpret graph storage.
    match unsafe { graph::GraphStorageDescriptor::from_index_relation(index.as_ptr(), metadata) }? {
        graph::GraphStorageDescriptor::TurboQuant { code_len } => {
            Ok(VacuumFormatAdapter::TurboQuant { code_len })
        }
        graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
            Ok(VacuumFormatAdapter::TurboQuantHotCold(layout))
        }
        graph::GraphStorageDescriptor::PqFastScan(layout) => {
            Ok(VacuumFormatAdapter::PqFastScan(layout))
        }
    }
}

unsafe fn run_bulkdelete_with_adapter(
    format: VacuumFormatAdapter,
    index: VacuumIndexRelation,
    heap_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let storage = format.graph_storage();
    let stats = if stats.is_null() {
        crate::fault::maybe_fail_palloc("ec_hnsw vacuum stats");
        // SAFETY: PostgreSQL memory context allocation creates a zeroed
        // IndexBulkDeleteResult owned by the current vacuum callback.
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };
    let block_count = index.main_fork_block_count();

    let mut live_elements = 0_usize;
    let mut removed_heap_tids = 0_usize;
    let mut finalize_tids = Vec::new();

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let share_plan = {
            let share_buffer =
                index.read_main_locked(block_number, pg_sys::BUFFER_LOCK_SHARE as i32, "vacuum");

            let share_page_ptr = share_buffer.page().cast::<u8>();
            let share_page_size = share_buffer.page_size();
            // SAFETY: The shared buffer remains pinned/locked while pass-1
            // planning reads tuple bytes on this page.
            unsafe {
                plan_page_pass1(
                    share_page_ptr,
                    share_page_size,
                    block_number,
                    storage,
                    callback,
                    callback_state,
                )
            }
        };

        if share_plan.updates.is_empty() {
            live_elements += share_plan.live_elements;
            removed_heap_tids += share_plan.removed_heap_tids;
            finalize_tids.extend(share_plan.finalize_tids);
            continue;
        }

        let exclusive_buffer =
            index.read_main_locked(block_number, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32, "vacuum");

        // SAFETY: `exclusive_buffer` is locked for this block and the callback
        // state is the same state PostgreSQL supplied to ambulkdelete.
        let final_plan = unsafe {
            rewrite_page_pass1(
                index,
                exclusive_buffer,
                block_number,
                storage,
                callback,
                callback_state,
            )
        };
        live_elements += final_plan.live_elements;
        removed_heap_tids += final_plan.removed_heap_tids;
        finalize_tids.extend(final_plan.finalize_tids);
    }

    // SAFETY: `finalize_tids` was produced by pass-1 graph element scans using
    // the same storage descriptor and names fully dead elements from this index.
    unsafe {
        format.repair_graph_connections(index, heap_relation, &finalize_tids);
        format.finalize_fully_dead_elements(index, &finalize_tids);
        repair_metadata_entry_point_after_vacuum(index, storage, &finalize_tids);
    }

    // SAFETY: `stats` is either PostgreSQL-supplied or allocated above for this
    // callback and remains valid until returned.
    unsafe {
        (*stats).num_pages = block_count;
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = live_elements as f64;
        (*stats).tuples_removed += removed_heap_tids as f64;
    }
    stats
}

unsafe fn repair_metadata_entry_point_after_vacuum(
    index: VacuumIndexRelation,
    storage: graph::GraphStorageDescriptor,
    finalize_tids: &[page::ItemPointer],
) {
    if finalize_tids.is_empty() {
        return;
    }

    let finalized: HashSet<_> = finalize_tids.iter().copied().collect();
    // SAFETY: The storage descriptor matches this index and the helper only
    // reads graph elements to find a replacement entry point.
    let replacement =
        unsafe { shared::highest_level_live_entry_candidate(index.as_ptr(), storage) };

    // SAFETY: Metadata is locked exclusively while entry point fields are
    // updated after vacuum finalization.
    unsafe {
        shared::with_locked_metadata_page(index.as_ptr(), |metadata| {
            if metadata.entry_point != page::ItemPointer::INVALID
                && !finalized.contains(&metadata.entry_point)
            {
                return;
            }

            if let Some(replacement) = replacement {
                metadata.entry_point = replacement.tid;
                metadata.max_level = replacement.level;
            } else {
                metadata.entry_point = page::ItemPointer::INVALID;
                metadata.max_level = 0;
            }
        })
    };
}

unsafe fn rewrite_page_pass1(
    index: VacuumIndexRelation,
    buffer: LockedBufferGuard,
    block_number: u32,
    storage: graph::GraphStorageDescriptor,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> PagePass1Plan {
    let page_ptr = buffer.page().cast::<u8>();
    let page_size = buffer.page_size();
    // SAFETY: The buffer is locked for this block and remains pinned while
    // pass-1 planning reads tuple bytes.
    let plan = unsafe {
        plan_page_pass1(
            page_ptr,
            page_size,
            block_number,
            storage,
            callback,
            callback_state,
        )
    };
    if plan.updates.is_empty() {
        return plan;
    }

    let rewrite = index.begin_page_rewrite(&buffer);
    let wal_page_ptr = rewrite.page_ptr();
    // SAFETY: Updates were planned from this same page/block and preserve tuple
    // lengths when rewritten.
    unsafe { apply_page_pass1_updates(wal_page_ptr, page_size, block_number, &plan.updates) };
    rewrite.finish();
    plan
}

unsafe fn plan_page_pass1(
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
    storage: graph::GraphStorageDescriptor,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> PagePass1Plan {
    let line_pointer_count = shared::page_line_pointer_count(page_ptr);
    let mut plan = PagePass1Plan::default();

    for offset in 1..=line_pointer_count {
        let tid = page::ItemPointer {
            block_number,
            offset_number: offset,
        };
        // SAFETY: The caller holds the page pinned/locked for reading and the
        // helper validates each line pointer before exposing tuple bytes.
        unsafe {
            shared::with_page_line_tuple_bytes(
                page_ptr,
                page_size,
                block_number,
                offset,
                "planning HNSW vacuum pass1",
                |tuple_bytes| match storage {
                    graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                        if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                            return;
                        }
                        let mut element = page::TqElementTuple::decode(tuple_bytes, code_len)
                            .unwrap_or_else(|e| {
                                pgrx::error!("ec_hnsw failed to decode vacuum element tuple: {e}")
                            });
                        let starting_len = element.heaptids.len();
                        element.heaptids.retain(|heap_tid| {
                            !heap_tid_is_dead(*heap_tid, callback, callback_state)
                        });
                        let removed = starting_len.saturating_sub(element.heaptids.len());

                        if !element.deleted && !element.heaptids.is_empty() {
                            plan.live_elements += 1;
                        }
                        if !element.deleted && element.heaptids.is_empty() {
                            plan.finalize_tids.push(tid);
                        }
                        if removed == 0 {
                            return;
                        }

                        plan.removed_heap_tids += removed;
                        plan.updates.push(ElementVacuumUpdate::TurboQuant {
                            tid,
                            tuple: element,
                        });
                    }
                    graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                        if tuple_bytes.first().copied() != Some(page::TQ_TURBO_HOT_TAG) {
                            return;
                        }
                        let mut element =
                            page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                                .unwrap_or_else(|e| {
                                    pgrx::error!(
                                        "ec_hnsw failed to decode vacuum TurboQuant V3 tuple: {e}"
                                    )
                                });
                        let starting_len = element.heaptids.len();
                        element.heaptids.retain(|heap_tid| {
                            !heap_tid_is_dead(*heap_tid, callback, callback_state)
                        });
                        let removed = starting_len.saturating_sub(element.heaptids.len());

                        if !element.deleted && !element.heaptids.is_empty() {
                            plan.live_elements += 1;
                        }
                        if !element.deleted && element.heaptids.is_empty() {
                            plan.finalize_tids.push(tid);
                        }
                        if removed == 0 {
                            return;
                        }

                        plan.removed_heap_tids += removed;
                        plan.updates.push(ElementVacuumUpdate::TurboQuantHot {
                            tid,
                            tuple: element,
                        });
                    }
                    graph::GraphStorageDescriptor::PqFastScan(layout) => {
                        if tuple_bytes.first().copied() != Some(page::TQ_GROUPED_HOT_TAG) {
                            return;
                        }
                        let mut element = page::TqGroupedHotTuple::decode(
                            tuple_bytes,
                            layout.binary_word_count,
                            layout.search_code_len,
                        )
                        .unwrap_or_else(|e| {
                            pgrx::error!("ec_hnsw failed to decode vacuum grouped hot tuple: {e}")
                        });
                        let starting_len = element.heaptids.len();
                        element.heaptids.retain(|heap_tid| {
                            !heap_tid_is_dead(*heap_tid, callback, callback_state)
                        });
                        let removed = starting_len.saturating_sub(element.heaptids.len());

                        if !element.deleted && !element.heaptids.is_empty() {
                            plan.live_elements += 1;
                        }
                        if !element.deleted && element.heaptids.is_empty() {
                            plan.finalize_tids.push(tid);
                        }
                        if removed == 0 {
                            return;
                        }

                        plan.removed_heap_tids += removed;
                        plan.updates.push(ElementVacuumUpdate::PqFastScanHot {
                            tid,
                            tuple: element,
                        });
                    }
                },
            )
            .unwrap_or_else(|e| pgrx::error!("{e}"))
        };
    }

    plan
}

unsafe fn apply_page_pass1_updates(
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
    updates: &[ElementVacuumUpdate],
) {
    for update in updates {
        let tid = match update {
            ElementVacuumUpdate::TurboQuant { tid, .. }
            | ElementVacuumUpdate::TurboQuantHot { tid, .. }
            | ElementVacuumUpdate::PqFastScanHot { tid, .. } => *tid,
        };
        // SAFETY: Each update was planned from this page and its re-encoded
        // tuple must match the existing tuple byte length.
        unsafe {
            shared::with_writable_page_tuple_bytes(
                page_ptr,
                page_size,
                tid,
                "vacuum element",
                |tuple_bytes| {
                    let encoded = match update {
                        ElementVacuumUpdate::TurboQuant { tuple, .. } => {
                            tuple.encode().unwrap_or_else(|e| {
                                pgrx::error!("ec_hnsw failed to encode vacuum element tuple: {e}")
                            })
                        }
                        ElementVacuumUpdate::TurboQuantHot { tuple, .. } => {
                            tuple.encode().unwrap_or_else(|e| {
                                pgrx::error!(
                                    "ec_hnsw failed to encode vacuum TurboQuant V3 tuple: {e}"
                                )
                            })
                        }
                        ElementVacuumUpdate::PqFastScanHot { tuple, .. } => {
                            tuple.encode().unwrap_or_else(|e| {
                                pgrx::error!(
                                    "ec_hnsw failed to encode vacuum grouped hot tuple: {e}"
                                )
                            })
                        }
                    };
                    if encoded.len() != tuple_bytes.len() {
                        pgrx::error!(
                            "ec_hnsw vacuum element tuple size changed from {} to {} on block {}",
                            tuple_bytes.len(),
                            encoded.len(),
                            block_number
                        );
                    }

                    tuple_bytes.copy_from_slice(&encoded);
                },
            )
        }
    }
}

unsafe fn repair_graph_connections_with_storage(
    index: VacuumIndexRelation,
    heap_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
    deleted_tids: &[page::ItemPointer],
) {
    if deleted_tids.is_empty() {
        return;
    }

    let metadata = index.metadata();
    let deleted_tids = deleted_tids.iter().copied().collect::<HashSet<_>>();
    // SAFETY: The heap/index relations are live for this vacuum pass; deleted
    // TIDs came from pass-1 scans and all repair phases use this storage descriptor.
    let repair_plans = unsafe {
        let mut metric = VacuumSearchMetric::for_relation(index.as_ptr(), heap_relation);
        let repair_requests = collect_repair_requests(index, storage, metadata.m, &deleted_tids);
        unlink_deleted_graph_connections(index, &deleted_tids);
        plan_repair_replacements(
            index,
            &metadata,
            &mut metric,
            storage,
            &deleted_tids,
            &repair_requests,
        )
    };
    // SAFETY: Repair plans target neighbor tuples from this index and are
    // applied under page-exclusive locks.
    unsafe { apply_repair_plans(index, metadata.m, &deleted_tids, &repair_plans) };
}

unsafe fn collect_repair_requests(
    index: VacuumIndexRelation,
    storage: graph::GraphStorageDescriptor,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
) -> Vec<LayerRepairRequest> {
    let block_count = index.main_fork_block_count();
    let mut requests = Vec::new();

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = index.read_main_locked(
            block_number,
            pg_sys::BUFFER_LOCK_SHARE as i32,
            "repair-request",
        );

        let page_ptr = buffer.page().cast::<u8>();
        let page_size = buffer.page_size();
        // SAFETY: The shared buffer remains pinned/locked while tuple bytes and
        // same-index neighbor payloads are inspected.
        unsafe {
            collect_repair_requests_on_page(
                index.as_ptr(),
                page_ptr,
                page_size,
                block_number,
                storage,
                m,
                deleted_tids,
                &mut requests,
            )
        };
    }

    requests.sort_unstable_by(|left, right| {
        compare_item_pointers(&left.neighbor_tid, &right.neighbor_tid)
            .then_with(|| left.layer.cmp(&right.layer))
            .then_with(|| compare_item_pointers(&left.source_tid, &right.source_tid))
    });
    requests.dedup();
    requests
}

unsafe fn collect_repair_requests_on_page(
    index_relation: pg_sys::Relation,
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
    storage: graph::GraphStorageDescriptor,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
    requests: &mut Vec<LayerRepairRequest>,
) {
    let line_pointer_count = shared::page_line_pointer_count(page_ptr);

    for offset in 1..=line_pointer_count {
        // SAFETY: The caller holds the page pinned/locked and the helper
        // validates each line pointer before exposing tuple bytes.
        let element_fields = unsafe {
            shared::with_page_line_tuple_bytes(
                page_ptr,
                page_size,
                block_number,
                offset,
                "collecting HNSW repair requests",
                |tuple_bytes| match storage {
                    graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                        if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                            return None;
                        }
                        let element = page::TqElementTuple::decode(tuple_bytes, code_len)
                            .unwrap_or_else(|e| {
                                pgrx::error!(
                                    "ec_hnsw failed to decode repair-request element tuple: {e}"
                                )
                            });
                        Some((
                            element.level,
                            element.deleted,
                            element.heaptids.is_empty(),
                            element.neighbortid,
                        ))
                    }
                    graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                        if tuple_bytes.first().copied() != Some(page::TQ_TURBO_HOT_TAG) {
                            return None;
                        }
                        let element =
                            page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                                .unwrap_or_else(|e| {
                                    pgrx::error!(
                                "ec_hnsw failed to decode repair-request TurboQuant V3 tuple: {e}"
                            )
                                });
                        Some((
                            element.level,
                            element.deleted,
                            element.heaptids.is_empty(),
                            element.neighbortid,
                        ))
                    }
                    graph::GraphStorageDescriptor::PqFastScan(layout) => {
                        if tuple_bytes.first().copied() != Some(page::TQ_GROUPED_HOT_TAG) {
                            return None;
                        }
                        let element = page::TqGroupedHotTuple::decode(
                            tuple_bytes,
                            layout.binary_word_count,
                            layout.search_code_len,
                        )
                        .unwrap_or_else(|e| {
                            pgrx::error!(
                                "ec_hnsw failed to decode repair-request grouped hot tuple: {e}"
                            )
                        });
                        Some((
                            element.level,
                            element.deleted,
                            element.heaptids.is_empty(),
                            element.neighbortid,
                        ))
                    }
                },
            )
        }
        .unwrap_or_else(|e| pgrx::error!("{e}"))
        .flatten();
        let Some((level, deleted, heaptids_empty, neighbortid)) = element_fields else {
            continue;
        };
        if deleted || heaptids_empty || neighbortid == page::ItemPointer::INVALID {
            continue;
        }

        // SAFETY: `neighbortid` came from a live graph element decoded from the
        // same index storage format.
        let neighbors = unsafe { graph::load_graph_neighbors(index_relation, neighbortid) };
        let source_tid = page::ItemPointer {
            block_number,
            offset_number: offset,
        };
        for layer in 0..=level {
            if layer_slice_contains_deleted_ref(&neighbors.tids, level, m, layer, deleted_tids) {
                requests.push(LayerRepairRequest {
                    source_tid,
                    neighbor_tid: neighbortid,
                    layer,
                });
            }
        }
    }
}

fn layer_slice_contains_deleted_ref(
    neighbor_tids: &[page::ItemPointer],
    element_level: u8,
    m: u16,
    layer: u8,
    deleted_tids: &HashSet<page::ItemPointer>,
) -> bool {
    let Some((start, end)) = graph::layer_slot_bounds(element_level, usize::from(m), layer) else {
        return false;
    };

    neighbor_tids
        .iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .any(|tid| deleted_tids.contains(tid))
}

unsafe fn unlink_deleted_graph_connections(
    index: VacuumIndexRelation,
    deleted_tids: &HashSet<page::ItemPointer>,
) {
    let block_count = index.main_fork_block_count();

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let share_updates = {
            let share_buffer =
                index.read_main_locked(block_number, pg_sys::BUFFER_LOCK_SHARE as i32, "repair");

            let share_page_ptr = share_buffer.page().cast::<u8>();
            let share_page_size = share_buffer.page_size();
            // SAFETY: The shared buffer remains pinned/locked while tuple bytes
            // are scanned for deleted neighbor references.
            unsafe { plan_page_pass2(share_page_ptr, share_page_size, block_number, deleted_tids) }
        };

        if share_updates.is_empty() {
            continue;
        }

        let exclusive_buffer =
            index.read_main_locked(block_number, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32, "repair");

        // SAFETY: The exclusive buffer belongs to this block and the deleted set
        // came from the same vacuum pass.
        unsafe { rewrite_page_pass2(index, exclusive_buffer, block_number, deleted_tids) };
    }
}

unsafe fn plan_repair_replacements(
    index: VacuumIndexRelation,
    metadata: &page::MetadataPage,
    metric: &mut VacuumSearchMetric,
    storage: graph::GraphStorageDescriptor,
    deleted_tids: &HashSet<page::ItemPointer>,
    requests: &[LayerRepairRequest],
) -> Vec<LayerRepairPlan> {
    let mut plans = requests
        .iter()
        .filter_map(|request| {
            // SAFETY: Requests were collected from this index/storage scan, and
            // planning only reads graph elements and neighbor tuples.
            unsafe {
                plan_repair_replacement(index, metadata, metric, storage, deleted_tids, request)
            }
        })
        .collect::<Vec<_>>();
    plans.sort_unstable_by(|left, right| {
        compare_item_pointers(&left.neighbor_tid, &right.neighbor_tid)
            .then_with(|| left.layer.cmp(&right.layer))
    });
    plans
}

unsafe fn plan_repair_replacement(
    index: VacuumIndexRelation,
    metadata: &page::MetadataPage,
    metric: &mut VacuumSearchMetric,
    storage: graph::GraphStorageDescriptor,
    deleted_tids: &HashSet<page::ItemPointer>,
    request: &LayerRepairRequest,
) -> Option<LayerRepairPlan> {
    // SAFETY: `request.source_tid` and `source.neighbortid` were collected from
    // this graph storage descriptor during repair-request scanning.
    let (source, neighbors) = unsafe {
        let source = graph::load_exact_graph_element(index.as_ptr(), request.source_tid, storage);
        let neighbors = graph::load_graph_neighbors(index.as_ptr(), source.neighbortid);
        (source, neighbors)
    };
    if source.deleted
        || source.heaptids.is_empty()
        || source.neighbortid != request.neighbor_tid
        || request.layer > source.level
    {
        return None;
    }

    let (start, end) =
        graph::layer_slot_bounds(source.level, usize::from(metadata.m), request.layer)?;

    let layer_slice = neighbors
        .tids
        .get(start..end)
        .expect("repair slot bounds should fit within persisted neighbor tuples");
    let free_slots = layer_slice
        .iter()
        .filter(|tid| **tid == page::ItemPointer::INVALID)
        .count();
    if free_slots == 0 {
        return None;
    }

    let existing_layer = layer_slice
        .iter()
        .copied()
        .filter(|tid| *tid != page::ItemPointer::INVALID && !deleted_tids.contains(tid))
        .collect::<Vec<_>>();
    let existing_set = existing_layer.iter().copied().collect::<HashSet<_>>();
    let planner = RepairSearchPlanner {
        metadata,
        storage,
        source: &source,
        layer: request.layer,
        deleted_tids,
        existing_layer: &existing_layer,
        existing_set: &existing_set,
        target_len: free_slots,
    };
    // SAFETY: Planner inputs all come from the same graph storage descriptor and
    // metric owns any source-scoring heap state.
    let replacements = unsafe { search_repair_candidates_for_layer(index, metric, &planner) };
    let mut replacements = replacements;
    let linear_planner = LinearRepairPlanner {
        metadata,
        storage,
        source: &source,
        deleted_tids,
        existing_set: &existing_set,
        layer: request.layer,
    };
    if replacements.len() < free_slots {
        // SAFETY: Linear top-up scans the same index/storage descriptor and
        // appends only local candidate TIDs to `replacements`.
        unsafe {
            top_up_repair_replacements_from_linear_scan(
                index,
                metric,
                &linear_planner,
                &mut replacements,
                free_slots,
            )
        };
    }
    if replacements.is_empty() {
        return None;
    }

    Some(LayerRepairPlan {
        neighbor_tid: source.neighbortid,
        source_level: source.level,
        layer: request.layer,
        replacement_tids: replacements,
    })
}

unsafe fn search_repair_candidates_for_layer(
    index: VacuumIndexRelation,
    metric: &mut VacuumSearchMetric,
    planner: &RepairSearchPlanner<'_>,
) -> Vec<page::ItemPointer> {
    let mut seeds = Vec::new();

    // SAFETY: The planner metadata/storage/source were produced from this
    // vacuum repair pass and metric owns any source-scoring heap state.
    if let Some(entry_candidate) = unsafe {
        load_vacuum_entry_candidate(
            index,
            planner.metadata,
            planner.storage,
            metric,
            planner.source,
        )
    } {
        if planner.layer == 0 {
            // SAFETY: The entry candidate came from metadata and greedy descent
            // reads graph elements through the same storage descriptor.
            seeds.push(unsafe {
                graph::greedy_descend_from_entry_with_storage(
                    index.as_ptr(),
                    planner.storage,
                    usize::from(planner.metadata.m),
                    entry_candidate,
                    |neighbor| {
                        metric.score_graph_element(planner.metadata, planner.source, neighbor)
                    },
                )
            });
        } else {
            let mut upper_seeds = vec![entry_candidate];
            for current_layer in (planner.layer..=planner.metadata.max_level).rev() {
                // SAFETY: Seeds came from prior repair search on the same
                // storage descriptor and the scoring closure filters invalid elements.
                upper_seeds = unsafe {
                    graph::search_layer_result_candidates_with_storage(
                        index.as_ptr(),
                        planner.storage,
                        usize::from(planner.metadata.m),
                        current_layer,
                        repair_ef_construction(planner.metadata),
                        upper_seeds,
                        |_| true,
                        |neighbor| {
                            metric.score_graph_element(planner.metadata, planner.source, neighbor)
                        },
                    )
                };
                if upper_seeds.is_empty() {
                    break;
                }
            }
            seeds.extend(upper_seeds);
        }
    }

    seeds.extend(planner.existing_layer.iter().filter_map(|tid| {
        // SAFETY: Existing-layer TIDs came from the source neighbor slice and
        // are loaded through the same graph storage descriptor.
        unsafe {
            let element = graph::load_exact_graph_element(index.as_ptr(), *tid, planner.storage);
            metric
                .score_graph_element(planner.metadata, planner.source, &element)
                .map(|score| search::BeamCandidate::new(*tid, score))
        }
    }));
    dedup_beam_candidates_by_tid(&mut seeds);
    if seeds.is_empty() {
        return Vec::new();
    }

    let candidates = if planner.layer == 0 {
        // SAFETY: Layer-0 repair search uses seeds gathered from this graph and
        // excludes deleted/source TIDs before accepting candidates.
        unsafe {
            graph::search_layer0_result_candidates_with_storage(
                index.as_ptr(),
                planner.storage,
                usize::from(planner.metadata.m),
                repair_ef_construction(planner.metadata),
                seeds,
                |neighbor_tid| {
                    neighbor_tid != planner.source.tid
                        && !planner.deleted_tids.contains(&neighbor_tid)
                },
                |neighbor| metric.score_graph_element(planner.metadata, planner.source, neighbor),
            )
        }
    } else {
        // SAFETY: Upper-layer repair search uses seeds gathered from this graph
        // and excludes deleted/source TIDs before accepting candidates.
        unsafe {
            graph::search_layer_result_candidates_with_storage(
                index.as_ptr(),
                planner.storage,
                usize::from(planner.metadata.m),
                planner.layer,
                repair_ef_construction(planner.metadata),
                seeds,
                |neighbor_tid| {
                    neighbor_tid != planner.source.tid
                        && !planner.deleted_tids.contains(&neighbor_tid)
                },
                |neighbor| metric.score_graph_element(planner.metadata, planner.source, neighbor),
            )
        }
    };

    candidates
        .into_iter()
        .map(|candidate| candidate.node)
        .filter(|tid| {
            *tid != planner.source.tid
                && *tid != page::ItemPointer::INVALID
                && !planner.existing_set.contains(tid)
                && !planner.deleted_tids.contains(tid)
        })
        .take(planner.target_len)
        .collect::<Vec<_>>()
}

unsafe fn load_vacuum_entry_candidate(
    index: VacuumIndexRelation,
    metadata: &page::MetadataPage,
    storage: graph::GraphStorageDescriptor,
    metric: &mut VacuumSearchMetric,
    source_element: &graph::GraphElement,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    if metadata.entry_point == page::ItemPointer::INVALID {
        return None;
    }

    // SAFETY: Metadata names a non-invalid entry point, storage matches this
    // vacuum repair snapshot, and metric owns any source-scoring heap state.
    let (entry, entry_score) = unsafe {
        let entry = graph::load_exact_graph_element(index.as_ptr(), metadata.entry_point, storage);
        let entry_score = metric.score_graph_element(metadata, source_element, &entry)?;
        (entry, entry_score)
    };
    Some(search::BeamCandidate::new(entry.tid, entry_score))
}

fn score_vacuum_code_element(
    metadata: &page::MetadataPage,
    source_code: &[u8],
    element: &graph::GraphElement,
) -> Option<f32> {
    (!element.deleted && !element.heaptids.is_empty()).then(|| {
        -crate::score_code_inner_product(
            metadata.dimensions as usize,
            metadata.bits,
            metadata.seed,
            source_code,
            &element.code,
        )
    })
}

fn repair_ef_construction(metadata: &page::MetadataPage) -> usize {
    let ef = usize::from(metadata.ef_construction);
    debug_assert!(
        ef > 0,
        "validated ec_hnsw indexes should always persist ef_construction >= 1"
    );
    ef.max(1)
}

fn dedup_beam_candidates_by_tid(candidates: &mut Vec<search::BeamCandidate<page::ItemPointer>>) {
    let mut seen = HashSet::new();
    candidates.retain(|candidate| seen.insert(candidate.node));
}

unsafe fn top_up_repair_replacements_from_linear_scan(
    index: VacuumIndexRelation,
    metric: &mut VacuumSearchMetric,
    planner: &LinearRepairPlanner<'_>,
    replacements: &mut Vec<page::ItemPointer>,
    target_len: usize,
) {
    if replacements.len() >= target_len {
        return;
    }

    let block_count = index.main_fork_block_count();
    let mut scored = Vec::new();

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = index.read_main_locked(
            block_number,
            pg_sys::BUFFER_LOCK_SHARE as i32,
            "linear-repair",
        );

        let page_ptr = buffer.page().cast::<u8>();
        let page_size = buffer.page_size();
        // SAFETY: The shared buffer remains pinned/locked while candidate tuple
        // bytes are decoded and scored.
        unsafe {
            collect_linear_repair_candidates_on_page(
                index,
                page_ptr,
                page_size,
                block_number,
                metric,
                planner,
                replacements,
                &mut scored,
            )
        };
    }

    scored.sort_unstable_by(|left, right| {
        left.1
            .total_cmp(&right.1)
            .then_with(|| compare_item_pointers(&left.0, &right.0))
    });
    for (tid, _) in scored {
        if replacements.len() >= target_len {
            break;
        }
        if !replacements.contains(&tid) {
            replacements.push(tid);
        }
    }
}

unsafe fn collect_linear_repair_candidates_on_page(
    index: VacuumIndexRelation,
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
    metric: &mut VacuumSearchMetric,
    planner: &LinearRepairPlanner<'_>,
    replacements: &[page::ItemPointer],
    scored: &mut Vec<(page::ItemPointer, f32)>,
) {
    let line_pointer_count = shared::page_line_pointer_count(page_ptr);

    for offset in 1..=line_pointer_count {
        let tid = page::ItemPointer {
            block_number,
            offset_number: offset,
        };
        if tid == planner.source.tid
            || planner.deleted_tids.contains(&tid)
            || planner.existing_set.contains(&tid)
            || replacements.contains(&tid)
        {
            continue;
        }

        // SAFETY: The page is pinned/locked by the caller and the helper
        // validates the line pointer before exposing tuple bytes.
        let candidate = unsafe {
            shared::with_page_line_tuple_bytes(
                page_ptr,
                page_size,
                block_number,
                offset,
                "collecting HNSW linear-repair candidates",
                |tuple_bytes| match planner.storage {
                    graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                        if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                            return None;
                        }
                        let element = page::TqElementTuple::decode(tuple_bytes, code_len)
                            .unwrap_or_else(|e| {
                                pgrx::error!(
                                    "ec_hnsw failed to decode linear-repair element tuple: {e}"
                                )
                            });
                        Some(graph::GraphElement {
                            tid,
                            level: element.level,
                            deleted: element.deleted,
                            heaptids: element.heaptids,
                            gamma: element.gamma,
                            neighbortid: element.neighbortid,
                            code: element.code,
                        })
                    }
                    graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                        if tuple_bytes.first().copied() != Some(page::TQ_TURBO_HOT_TAG) {
                            return None;
                        }
                        let element =
                            page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                                .unwrap_or_else(|e| {
                                    pgrx::error!(
                                "ec_hnsw failed to decode linear-repair TurboQuant V3 tuple: {e}"
                            )
                                });
                        let rerank = graph::load_rerank_payload(
                            index.as_ptr(),
                            element.reranktid,
                            layout.rerank_code_len,
                        );
                        Some(graph::GraphElement {
                            tid,
                            level: element.level,
                            deleted: element.deleted,
                            heaptids: element.heaptids,
                            gamma: rerank.gamma,
                            neighbortid: element.neighbortid,
                            code: rerank.code,
                        })
                    }
                    graph::GraphStorageDescriptor::PqFastScan(layout) => {
                        if tuple_bytes.first().copied() != Some(page::TQ_GROUPED_HOT_TAG) {
                            return None;
                        }
                        let element = page::TqGroupedHotTuple::decode(
                            tuple_bytes,
                            layout.binary_word_count,
                            layout.search_code_len,
                        )
                        .unwrap_or_else(|e| {
                            pgrx::error!(
                                "ec_hnsw failed to decode linear-repair grouped hot tuple: {e}"
                            )
                        });
                        let rerank = load_grouped_rerank_payload_for_linear_repair_candidate(
                            index,
                            page_ptr,
                            page_size,
                            block_number,
                            element.reranktid,
                            layout,
                        );
                        Some(graph::GraphElement {
                            tid,
                            level: element.level,
                            deleted: element.deleted,
                            heaptids: element.heaptids,
                            gamma: rerank.gamma,
                            neighbortid: element.neighbortid,
                            code: rerank.code,
                        })
                    }
                },
            )
        };
        let Some(candidate) = candidate.unwrap_or_else(|e| pgrx::error!("{e}")).flatten() else {
            continue;
        };
        if candidate.deleted || candidate.heaptids.is_empty() || candidate.level < planner.layer {
            continue;
        }

        // SAFETY: Candidate and source graph elements were loaded from the same
        // storage descriptor and metric owns any source-scoring heap state.
        if let Some(score) =
            unsafe { metric.score_graph_element(planner.metadata, planner.source, &candidate) }
        {
            scored.push((tid, score));
        }
    }
}

unsafe fn load_grouped_rerank_payload_for_linear_repair_candidate(
    index: VacuumIndexRelation,
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
    rerank_tid: page::ItemPointer,
    layout: graph::PqFastScanLayout,
) -> graph::GroupedRerankPayload {
    if rerank_tid == page::ItemPointer::INVALID {
        pgrx::error!("ec_hnsw linear-repair grouped candidate is missing a rerank payload tid");
    }

    if rerank_tid.block_number != block_number {
        // SAFETY: Cross-page grouped rerank payloads are loaded through the
        // graph helper using the persisted layout.
        return unsafe { graph::load_grouped_rerank_payload(index.as_ptr(), rerank_tid, layout) };
    }

    // SAFETY: Same-page rerank TID refers to the pinned page supplied by the
    // caller; the helper validates the line pointer before exposing bytes.
    unsafe {
        shared::with_page_line_tuple_bytes(
            page_ptr,
            page_size,
            block_number,
            rerank_tid.offset_number,
            "loading same-page linear-repair rerank payload",
            |tuple_bytes| {
                let rerank = page::TqRerankTuple::decode(tuple_bytes, layout.rerank_code_len)
                    .unwrap_or_else(|e| {
                        pgrx::error!("ec_hnsw failed to decode linear-repair rerank tuple: {e}")
                    });
                graph::GroupedRerankPayload {
                    tid: rerank_tid,
                    gamma: rerank.gamma,
                    code: rerank.code,
                }
            },
        )
    }
    .unwrap_or_else(|e| pgrx::error!("{e}"))
    .unwrap_or_else(|| {
        pgrx::error!(
            "ec_hnsw linear-repair rerank tuple slot {}/{} is unused",
            rerank_tid.block_number,
            rerank_tid.offset_number
        )
    })
}

unsafe fn apply_repair_plans(
    index: VacuumIndexRelation,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
    plans: &[LayerRepairPlan],
) {
    if plans.is_empty() {
        return;
    }

    let mut start = 0;
    while start < plans.len() {
        let block_number = plans[start].neighbor_tid.block_number;
        let mut end = start + 1;
        while end < plans.len() && plans[end].neighbor_tid.block_number == block_number {
            end += 1;
        }

        // SAFETY: This slice contains only repair plans for one neighbor block,
        // which will be locked exclusively before rewrite.
        unsafe {
            apply_repair_plans_on_page(index, block_number, m, deleted_tids, &plans[start..end])
        };
        start = end;
    }
}

unsafe fn apply_repair_plans_on_page(
    index: VacuumIndexRelation,
    block_number: u32,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
    plans: &[LayerRepairPlan],
) {
    let buffer = index.read_main_locked(
        block_number,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        "layer0-repair",
    );
    let rewrite = index.begin_page_rewrite(&buffer);
    let page_ptr = rewrite.page_ptr();
    let page_size = buffer.page_size();
    let mut changed = false;

    let mut start = 0;
    while start < plans.len() {
        let neighbor_tid = plans[start].neighbor_tid;
        let mut end = start + 1;
        while end < plans.len() && plans[end].neighbor_tid == neighbor_tid {
            end += 1;
        }

        // SAFETY: `neighbor_tid` points at a tuple on the registered page and
        // the closure preserves the encoded tuple length.
        let tuple_changed = unsafe {
            shared::with_writable_page_tuple_bytes(
                page_ptr,
                page_size,
                neighbor_tid,
                "repair neighbor",
                |tuple_bytes| {
                    let mut neighbor =
                        page::TqNeighborTuple::decode(tuple_bytes).unwrap_or_else(|e| {
                            pgrx::error!("ec_hnsw failed to decode repair neighbor tuple: {e}")
                        });
                    if neighbor.count as usize > neighbor.tids.len() {
                        pgrx::error!(
                            "ec_hnsw repair neighbor tuple count {} exceeds payload tid count {}",
                            neighbor.count,
                            neighbor.tids.len()
                        );
                    }
                    let mut tuple_changed =
                        unlink_deleted_neighbor_refs(&mut neighbor.tids, deleted_tids);
                    for plan in &plans[start..end] {
                        tuple_changed |= apply_repair_plan(
                            &mut neighbor.tids,
                            plan.source_level,
                            m,
                            plan.layer,
                            deleted_tids,
                            &plan.replacement_tids,
                        );
                    }
                    if !tuple_changed {
                        return false;
                    }

                    let encoded = neighbor.encode().unwrap_or_else(|e| {
                        pgrx::error!("ec_hnsw failed to encode repair neighbor tuple: {e}")
                    });
                    if encoded.len() != tuple_bytes.len() {
                        pgrx::error!(
                            "ec_hnsw repair neighbor tuple size changed from {} to {} on block {}",
                            tuple_bytes.len(),
                            encoded.len(),
                            block_number
                        );
                    }

                    tuple_bytes.copy_from_slice(&encoded);
                    true
                },
            )
        };
        if !tuple_changed {
            start = end;
            continue;
        }
        changed = true;
        start = end;
    }

    if changed {
        rewrite.finish();
    } else {
        std::mem::drop(rewrite);
    }
}

fn apply_repair_plan(
    neighbor_tids: &mut [page::ItemPointer],
    source_level: u8,
    m: u16,
    layer: u8,
    deleted_tids: &HashSet<page::ItemPointer>,
    replacement_tids: &[page::ItemPointer],
) -> bool {
    let Some((start, end)) = graph::layer_slot_bounds(source_level, usize::from(m), layer) else {
        return false;
    };
    let layer_slice = neighbor_tids
        .get_mut(start..end)
        .expect("repair slot bounds should fit within persisted neighbor tuples");
    let mut changed = false;
    for candidate_tid in replacement_tids {
        if *candidate_tid == page::ItemPointer::INVALID
            || deleted_tids.contains(candidate_tid)
            || layer_slice.contains(candidate_tid)
        {
            continue;
        }

        let Some(slot) = layer_slice
            .iter_mut()
            .find(|tid| **tid == page::ItemPointer::INVALID)
        else {
            break;
        };
        *slot = *candidate_tid;
        changed = true;
    }

    changed
}

unsafe fn rewrite_page_pass2(
    index: VacuumIndexRelation,
    buffer: LockedBufferGuard,
    block_number: u32,
    deleted_tids: &HashSet<page::ItemPointer>,
) {
    let page_ptr = buffer.page().cast::<u8>();
    let page_size = buffer.page_size();
    // SAFETY: The caller holds the page locked and pinned while pass-2 planning
    // reads neighbor tuple bytes.
    let updates = unsafe { plan_page_pass2(page_ptr, page_size, block_number, deleted_tids) };
    if updates.is_empty() {
        return;
    }

    let rewrite = index.begin_page_rewrite(&buffer);
    let wal_page_ptr = rewrite.page_ptr();
    // SAFETY: Updates were planned from this same page/block and preserve tuple
    // lengths when rewritten.
    unsafe { apply_page_pass2_updates(wal_page_ptr, page_size, block_number, &updates) };
    rewrite.finish();
}

unsafe fn plan_page_pass2(
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
    deleted_tids: &HashSet<page::ItemPointer>,
) -> Vec<NeighborVacuumUpdate> {
    let line_pointer_count = shared::page_line_pointer_count(page_ptr);
    let mut updates = Vec::new();

    for offset in 1..=line_pointer_count {
        // SAFETY: The caller holds the page pinned/locked for reading and the
        // helper validates the line pointer before exposing tuple bytes.
        let update = unsafe {
            shared::with_page_line_tuple_bytes(
                page_ptr,
                page_size,
                block_number,
                offset,
                "planning HNSW vacuum pass2 repair",
                |tuple_bytes| {
                    if tuple_bytes.first().copied() != Some(page::TQ_NEIGHBOR_TAG) {
                        return None;
                    }

                    let mut neighbor =
                        page::TqNeighborTuple::decode(tuple_bytes).unwrap_or_else(|e| {
                            pgrx::error!("ec_hnsw failed to decode repair neighbor tuple: {e}")
                        });
                    if neighbor.count as usize > neighbor.tids.len() {
                        pgrx::error!(
                            "ec_hnsw repair neighbor tuple count {} exceeds payload tid count {}",
                            neighbor.count,
                            neighbor.tids.len()
                        );
                    }
                    if !unlink_deleted_neighbor_refs(&mut neighbor.tids, deleted_tids) {
                        return None;
                    }

                    Some(NeighborVacuumUpdate {
                        tid: page::ItemPointer {
                            block_number,
                            offset_number: offset,
                        },
                        tuple: neighbor,
                    })
                },
            )
        }
        .unwrap_or_else(|e| pgrx::error!("{e}"))
        .flatten();
        if let Some(update) = update {
            updates.push(update);
        }
    }

    updates
}

fn unlink_deleted_neighbor_refs(
    neighbor_tids: &mut [page::ItemPointer],
    deleted_tids: &HashSet<page::ItemPointer>,
) -> bool {
    let mut changed = false;
    for tid in neighbor_tids.iter_mut() {
        if deleted_tids.contains(&*tid) {
            *tid = page::ItemPointer::INVALID;
            changed = true;
        }
    }
    changed
}

unsafe fn apply_page_pass2_updates(
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
    updates: &[NeighborVacuumUpdate],
) {
    for update in updates {
        // SAFETY: Each update was planned from this page and its re-encoded
        // tuple must match the existing tuple byte length.
        unsafe {
            shared::with_writable_page_tuple_bytes(
                page_ptr,
                page_size,
                update.tid,
                "repair neighbor",
                |tuple_bytes| {
                    let encoded = update.tuple.encode().unwrap_or_else(|e| {
                        pgrx::error!("ec_hnsw failed to encode repair neighbor tuple: {e}")
                    });
                    if encoded.len() != tuple_bytes.len() {
                        pgrx::error!(
                            "ec_hnsw repair neighbor tuple size changed from {} to {} on block {}",
                            tuple_bytes.len(),
                            encoded.len(),
                            block_number
                        );
                    }

                    tuple_bytes.copy_from_slice(&encoded);
                },
            )
        }
    }
}

unsafe fn finalize_fully_dead_elements_with_storage(
    index: VacuumIndexRelation,
    storage: graph::GraphStorageDescriptor,
    tids: &[page::ItemPointer],
) {
    if tids.is_empty() {
        return;
    }

    let mut tids = tids.to_vec();
    tids.sort_unstable_by(compare_item_pointers);
    tids.dedup();

    let mut start = 0;
    while start < tids.len() {
        let block_number = tids[start].block_number;
        let mut end = start + 1;
        while end < tids.len() && tids[end].block_number == block_number {
            end += 1;
        }

        // SAFETY: This slice contains only fully-dead element TIDs for one
        // block, which will be locked exclusively before finalization.
        unsafe {
            finalize_fully_dead_elements_on_page_with_storage(
                index,
                block_number,
                storage,
                &tids[start..end],
            )
        };
        start = end;
    }
}

unsafe fn finalize_fully_dead_elements_on_page_with_storage(
    index: VacuumIndexRelation,
    block_number: u32,
    storage: graph::GraphStorageDescriptor,
    tids: &[page::ItemPointer],
) {
    let buffer = index.read_main_locked(
        block_number,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        "finalize",
    );

    let page_ptr = buffer.page().cast::<u8>();
    let page_size = buffer.page_size();
    let mut updates = Vec::new();

    for tid in tids {
        // SAFETY: `tid` targets this locked page and the helper validates the
        // line pointer before exposing tuple bytes.
        let update = unsafe {
            shared::with_page_line_tuple_bytes(
                page_ptr,
                page_size,
                block_number,
                tid.offset_number,
                "planning fully-dead element finalization",
                |tuple_bytes| match storage {
                    graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                        let mut element = page::TqElementTuple::decode(tuple_bytes, code_len)
                            .unwrap_or_else(|e| {
                                pgrx::error!("ec_hnsw failed to decode finalize element tuple: {e}")
                            });
                        if element.deleted || !element.heaptids.is_empty() {
                            return None;
                        }

                        element.deleted = true;
                        Some(ElementVacuumUpdate::TurboQuant {
                            tid: *tid,
                            tuple: element,
                        })
                    }
                    graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                        let mut element =
                            page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                                .unwrap_or_else(|e| {
                                    pgrx::error!(
                                        "ec_hnsw failed to decode finalize TurboQuant V3 tuple: {e}"
                                    )
                                });
                        if element.deleted || !element.heaptids.is_empty() {
                            return None;
                        }

                        element.deleted = true;
                        Some(ElementVacuumUpdate::TurboQuantHot {
                            tid: *tid,
                            tuple: element,
                        })
                    }
                    graph::GraphStorageDescriptor::PqFastScan(layout) => {
                        let mut element = page::TqGroupedHotTuple::decode(
                            tuple_bytes,
                            layout.binary_word_count,
                            layout.search_code_len,
                        )
                        .unwrap_or_else(|e| {
                            pgrx::error!("ec_hnsw failed to decode finalize grouped hot tuple: {e}")
                        });
                        if element.deleted || !element.heaptids.is_empty() {
                            return None;
                        }

                        element.deleted = true;
                        Some(ElementVacuumUpdate::PqFastScanHot {
                            tid: *tid,
                            tuple: element,
                        })
                    }
                },
            )
        }
        .unwrap_or_else(|e| pgrx::error!("{e}"))
        .unwrap_or_else(|| {
            pgrx::error!(
                "ec_hnsw finalize element tuple slot {}/{} is unused",
                tid.block_number,
                tid.offset_number
            )
        });
        if let Some(update) = update {
            updates.push(update);
        }
    }

    if updates.is_empty() {
        return;
    }

    let rewrite = index.begin_page_rewrite(&buffer);
    let wal_page_ptr = rewrite.page_ptr();
    // SAFETY: Updates were planned from this same page/block and preserve tuple
    // lengths when rewritten.
    unsafe { apply_page_pass1_updates(wal_page_ptr, page_size, block_number, &updates) };
    rewrite.finish();
}

fn compare_item_pointers(
    left: &page::ItemPointer,
    right: &page::ItemPointer,
) -> std::cmp::Ordering {
    left.block_number
        .cmp(&right.block_number)
        .then_with(|| left.offset_number.cmp(&right.offset_number))
}

unsafe fn heap_tid_is_dead(
    heap_tid: page::ItemPointer,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> bool {
    let mut tid = pg_sys::ItemPointerData::default();
    item_pointer_set_all(&mut tid, heap_tid.block_number, heap_tid.offset_number);
    // SAFETY: The callback and callback state were supplied by PostgreSQL for
    // this ambulkdelete invocation, and `tid` lives for the callback call.
    unsafe { callback((&mut tid) as pg_sys::ItemPointer, callback_state) }
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Default)]
struct DebugVacuumCallbackState {
    dead_tids: std::collections::HashSet<page::ItemPointer>,
}

#[cfg(any(test, feature = "pg_test"))]
unsafe extern "C-unwind" fn debug_vacuum_dead_tid_callback(
    itemptr: pg_sys::ItemPointer,
    state: *mut c_void,
) -> bool {
    // SAFETY: Debug vacuum passes a `DebugVacuumCallbackState` pointer as the
    // callback state for the duration of this guarded callback.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = &*(state.cast::<DebugVacuumCallbackState>());
            state.dead_tids.contains(&shared::decode_heap_tid(itemptr))
        })
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_vacuum_remove_heap_tids(
    index_oid: pg_sys::Oid,
    dead_tids: &[page::ItemPointer],
) -> pg_sys::IndexBulkDeleteResult {
    let index_relation_guard = IndexRelationGuard::try_open(
        index_oid,
        pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
    )
    .unwrap_or_else(|| pgrx::error!("ec_hnsw debug vacuum could not open index relation"));
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The opened index relation is valid and IndexGetRelation only
    // resolves its heap OID.
    let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
    let heap_relation_guard = if heap_oid == pg_sys::InvalidOid {
        None
    } else {
        Some(
            HeapRelationGuard::try_access_share(heap_oid).unwrap_or_else(|| {
                pgrx::error!("ec_hnsw debug vacuum could not open heap relation")
            }),
        )
    };
    let heap_relation = heap_relation_guard
        .as_ref()
        .map_or(std::ptr::null_mut(), HeapRelationGuard::as_ptr);
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
    info.heaprel = heap_relation;
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;
    let mut callback_state = DebugVacuumCallbackState {
        dead_tids: dead_tids.iter().copied().collect(),
    };

    // SAFETY: Debug vacuum constructs callback-duration IndexVacuumInfo and
    // callback state and invokes the AM bulkdelete entry point directly.
    let stats = unsafe {
        ec_hnsw_ambulkdelete(
            info_ptr,
            std::ptr::null_mut(),
            Some(debug_vacuum_dead_tid_callback),
            (&mut callback_state as *mut DebugVacuumCallbackState).cast(),
        )
    };
    // SAFETY: The same debug IndexVacuumInfo and stats pointer are valid for the
    // follow-up cleanup call.
    let stats = unsafe { ec_hnsw_amvacuumcleanup(info_ptr, stats) };
    // SAFETY: The AM returned a valid stats pointer owned by the current
    // PostgreSQL memory context; copy the result before dropping guards.
    let result = unsafe { *stats };
    drop(heap_relation_guard);
    drop(index_relation_guard);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar_v1_metadata() -> page::MetadataPage {
        page::MetadataPage::current_v1_scalar(page::CurrentFormatMetadata {
            m: 8,
            ef_construction: 64,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 16,
            bits: 4,
            max_level: 0,
            seed: 42,
            inserted_since_rebuild: 0,
            persisted_binary_sidecar: false,
        })
    }

    fn pq_fastscan_metadata() -> page::MetadataPage {
        page::MetadataPage {
            format_version: page::INDEX_FORMAT_V2_GROUPED,
            transform_kind: page::TransformKind::Srht,
            search_codec_kind: page::SearchCodecKind::GroupedPq,
            payload_flags: page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE
                | page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            search_bits: 4,
            rerank_codec_kind: page::RerankCodecKind::ScalarQuantized,
            search_subvector_count: 1,
            search_subvector_dim: 16,
            grouped_codebook_head: page::ItemPointer {
                block_number: 1,
                offset_number: 2,
            },
            ..scalar_v1_metadata()
        }
    }

    #[test]
    fn resolve_vacuum_format_adapter_accepts_scalar_v1() {
        let format =
            match graph::GraphStorageDescriptor::from_metadata(&scalar_v1_metadata()).unwrap() {
                graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                    VacuumFormatAdapter::TurboQuant { code_len }
                }
                graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                    VacuumFormatAdapter::TurboQuantHotCold(layout)
                }
                graph::GraphStorageDescriptor::PqFastScan(layout) => {
                    VacuumFormatAdapter::PqFastScan(layout)
                }
            };
        assert_eq!(
            format,
            VacuumFormatAdapter::TurboQuant {
                code_len: crate::code_len(16, 4),
            }
        );
    }

    #[test]
    fn resolve_vacuum_format_adapter_recognizes_pq_fastscan() {
        let format =
            match graph::GraphStorageDescriptor::from_metadata(&pq_fastscan_metadata()).unwrap() {
                graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                    VacuumFormatAdapter::TurboQuant { code_len }
                }
                graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                    VacuumFormatAdapter::TurboQuantHotCold(layout)
                }
                graph::GraphStorageDescriptor::PqFastScan(layout) => {
                    VacuumFormatAdapter::PqFastScan(layout)
                }
            };
        assert_eq!(
            format,
            VacuumFormatAdapter::PqFastScan(graph::PqFastScanLayout {
                binary_word_count: 0,
                search_code_len: 1,
                rerank_code_len: crate::code_len(16, 4),
            })
        );
    }
}
