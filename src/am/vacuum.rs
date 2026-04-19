use std::{collections::HashSet, ffi::c_void, ptr};

use pgrx::{itemptr::item_pointer_set_all, pg_sys, PgBox};

use super::{graph, options, page, search, shared, source, wal};
type BulkDeleteCallback =
    unsafe extern "C-unwind" fn(itemptr: pg_sys::ItemPointer, state: *mut c_void) -> bool;

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
    slot: *mut pg_sys::TupleTableSlot,
    source_attribute: source::SourceAttribute,
}

impl VacuumHeapSourceScorer {
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
                "tqhnsw vacuum failed to allocate a heap source slot",
            )
        };

        Self {
            heap_relation,
            snapshot: std::ptr::addr_of_mut!(pg_sys::SnapshotAnyData),
            slot,
            source_attribute,
        }
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

    unsafe fn score_graph_element_pair(
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

        let source_vector = unsafe {
            self.averaged_source_vector(
                &source_element.heaptids,
                "vacuum repair source-backed element",
            )
        }?;
        let candidate_vector = unsafe {
            self.averaged_source_vector(
                &candidate_element.heaptids,
                "vacuum repair source-backed candidate",
            )
        }?;
        Some(source::negative_inner_product(
            &source_vector,
            &candidate_vector,
        ))
    }
}

impl Drop for VacuumHeapSourceScorer {
    fn drop(&mut self) {
        if !self.slot.is_null() {
            unsafe { pg_sys::ExecDropSingleTupleTableSlot(self.slot) };
        }
    }
}

impl VacuumSearchMetric {
    unsafe fn for_relation(
        index_relation: pg_sys::Relation,
        heap_relation: pg_sys::Relation,
    ) -> Self {
        if heap_relation.is_null() {
            pgrx::error!("tqhnsw vacuum requires a heap relation for source-backed indexes");
        }

        let index_options = unsafe { options::relation_options(index_relation) };
        match index_options.build_source_column.as_deref() {
            Some(source_column) => {
                let source_attribute = unsafe {
                    source::resolve_source_attribute(
                        heap_relation,
                        source_column,
                        "build_source_column",
                        source::SourceTypePolicy::BuildSource,
                    )
                };
                Self::Source(unsafe {
                    VacuumHeapSourceScorer::new_with_attribute(heap_relation, source_attribute)
                })
            }
            None => {
                let indexed_attribute = unsafe {
                    source::resolve_indexed_vector_attribute(
                        heap_relation,
                        index_relation,
                        "indexed column",
                    )
                };
                match indexed_attribute.kind {
                    source::IndexedVectorKind::Ecvector => Self::Source(unsafe {
                        VacuumHeapSourceScorer::new_with_attribute(
                            heap_relation,
                            source::SourceAttribute {
                                attnum: indexed_attribute.attnum,
                                kind: source::SourceDatumKind::Ecvector,
                            },
                        )
                    }),
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
            Self::Source(scorer) => unsafe {
                scorer.score_graph_element_pair(source_element, candidate_element)
            },
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
        index_relation: pg_sys::Relation,
        stats: *mut pg_sys::IndexBulkDeleteResult,
    ) -> *mut pg_sys::IndexBulkDeleteResult {
        let _ = self;
        unsafe { shared::tqhnsw_noop_vacuum_stats(index_relation, stats) }
    }

    unsafe fn repair_graph_connections(
        self,
        index_relation: pg_sys::Relation,
        heap_relation: pg_sys::Relation,
        deleted_tids: &[page::ItemPointer],
    ) {
        unsafe {
            repair_graph_connections_with_storage(
                index_relation,
                heap_relation,
                self.graph_storage(),
                deleted_tids,
            )
        }
    }

    unsafe fn finalize_fully_dead_elements(
        self,
        index_relation: pg_sys::Relation,
        deleted_tids: &[page::ItemPointer],
    ) {
        unsafe {
            finalize_fully_dead_elements_with_storage(
                index_relation,
                self.graph_storage(),
                deleted_tids,
            )
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

pub(super) unsafe extern "C-unwind" fn tqhnsw_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: pg_sys::IndexBulkDeleteCallback,
    callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let metadata = shared::read_metadata_page((*info).index);
            let format = resolve_vacuum_format_adapter((*info).index, &metadata)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            let Some(callback) = callback else {
                return shared::tqhnsw_noop_vacuum_stats((*info).index, stats);
            };
            run_bulkdelete_with_adapter(
                format,
                (*info).index,
                (*info).heaprel,
                stats,
                callback,
                callback_state,
            )
        })
    }
}

pub(super) unsafe extern "C-unwind" fn tqhnsw_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let metadata = shared::read_metadata_page((*info).index);
            let format = resolve_vacuum_format_adapter((*info).index, &metadata)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            format.vacuum_cleanup((*info).index, stats)
        })
    }
}

fn resolve_vacuum_format_adapter(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> Result<VacuumFormatAdapter, String> {
    match unsafe { graph::GraphStorageDescriptor::from_index_relation(index_relation, metadata) }? {
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
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let storage = format.graph_storage();
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };

    let mut live_elements = 0_usize;
    let mut removed_heap_tids = 0_usize;
    let mut finalize_tids = Vec::new();

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let share_buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(share_buffer) } {
            pgrx::error!("tqhnsw failed to open vacuum block {block_number}");
        }

        unsafe { pg_sys::LockBuffer(share_buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let share_page_ptr = unsafe { pg_sys::BufferGetPage(share_buffer) }.cast::<u8>();
        let share_page_size = unsafe { pg_sys::BufferGetPageSize(share_buffer) as usize };
        let share_plan = unsafe {
            plan_page_pass1(
                share_page_ptr,
                share_page_size,
                block_number,
                storage,
                callback,
                callback_state,
            )
        };
        unsafe { pg_sys::UnlockReleaseBuffer(share_buffer) };

        if share_plan.updates.is_empty() {
            live_elements += share_plan.live_elements;
            removed_heap_tids += share_plan.removed_heap_tids;
            finalize_tids.extend(share_plan.finalize_tids);
            continue;
        }

        let exclusive_buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(exclusive_buffer) } {
            pgrx::error!("tqhnsw failed to reopen vacuum block {block_number}");
        }

        let final_plan = unsafe {
            rewrite_page_pass1(
                index_relation,
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

    unsafe { format.repair_graph_connections(index_relation, heap_relation, &finalize_tids) };
    unsafe { format.finalize_fully_dead_elements(index_relation, &finalize_tids) };
    unsafe { repair_metadata_entry_point_after_vacuum(index_relation, storage, &finalize_tids) };

    unsafe {
        (*stats).num_pages = block_count;
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = live_elements as f64;
        (*stats).tuples_removed += removed_heap_tids as f64;
    }
    stats
}

unsafe fn repair_metadata_entry_point_after_vacuum(
    index_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
    finalize_tids: &[page::ItemPointer],
) {
    if finalize_tids.is_empty() {
        return;
    }

    let finalized: HashSet<_> = finalize_tids.iter().copied().collect();
    let replacement =
        unsafe { shared::highest_level_live_entry_candidate(index_relation, storage) };

    unsafe {
        shared::with_locked_metadata_page(index_relation, |metadata| {
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
    index_relation: pg_sys::Relation,
    buffer: pg_sys::Buffer,
    block_number: u32,
    storage: graph::GraphStorageDescriptor,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> PagePass1Plan {
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
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
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return plan;
    }

    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let wal_page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    unsafe { apply_page_pass1_updates(wal_page_ptr, page_size, block_number, &plan.updates) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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
        let item_id = unsafe { &*shared::page_item_id(page_ptr, offset) };
        if item_id.lp_flags() == 0 {
            continue;
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw found invalid vacuum tuple bounds on block {block_number}");
        }

        let tuple_bytes =
            unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };

        let tid = page::ItemPointer {
            block_number,
            offset_number: offset,
        };
        match storage {
            graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                    continue;
                }
                let mut element = page::TqElementTuple::decode(tuple_bytes, code_len)
                    .unwrap_or_else(|e| {
                        pgrx::error!("tqhnsw failed to decode vacuum element tuple: {e}")
                    });
                let starting_len = element.heaptids.len();
                element.heaptids.retain(|heap_tid| unsafe {
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
                    continue;
                }

                plan.removed_heap_tids += removed;
                plan.updates.push(ElementVacuumUpdate::TurboQuant {
                    tid,
                    tuple: element,
                });
            }
            graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                if tuple_bytes.first().copied() != Some(page::TQ_TURBO_HOT_TAG) {
                    continue;
                }
                let mut element =
                    page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                        .unwrap_or_else(|e| {
                            pgrx::error!("tqhnsw failed to decode vacuum TurboQuant V3 tuple: {e}")
                        });
                let starting_len = element.heaptids.len();
                element.heaptids.retain(|heap_tid| unsafe {
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
                    continue;
                }

                plan.removed_heap_tids += removed;
                plan.updates.push(ElementVacuumUpdate::TurboQuantHot {
                    tid,
                    tuple: element,
                });
            }
            graph::GraphStorageDescriptor::PqFastScan(layout) => {
                if tuple_bytes.first().copied() != Some(page::TQ_GROUPED_HOT_TAG) {
                    continue;
                }
                let mut element = page::TqGroupedHotTuple::decode(
                    tuple_bytes,
                    layout.binary_word_count,
                    layout.search_code_len,
                )
                .unwrap_or_else(|e| {
                    pgrx::error!("tqhnsw failed to decode vacuum grouped hot tuple: {e}")
                });
                let starting_len = element.heaptids.len();
                element.heaptids.retain(|heap_tid| unsafe {
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
                    continue;
                }

                plan.removed_heap_tids += removed;
                plan.updates.push(ElementVacuumUpdate::PqFastScanHot {
                    tid,
                    tuple: element,
                });
            }
        }
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
        let item_id = unsafe { &*shared::page_item_id(page_ptr, tid.offset_number) };
        if item_id.lp_flags() == 0 {
            pgrx::error!(
                "tqhnsw vacuum element tuple slot {}/{} is unused",
                tid.block_number,
                tid.offset_number
            );
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw found invalid vacuum rewrite bounds on block {block_number}");
        }

        let encoded = match update {
            ElementVacuumUpdate::TurboQuant { tuple, .. } => tuple.encode().unwrap_or_else(|e| {
                pgrx::error!("tqhnsw failed to encode vacuum element tuple: {e}")
            }),
            ElementVacuumUpdate::TurboQuantHot { tuple, .. } => {
                tuple.encode().unwrap_or_else(|e| {
                    pgrx::error!("tqhnsw failed to encode vacuum TurboQuant V3 tuple: {e}")
                })
            }
            ElementVacuumUpdate::PqFastScanHot { tuple, .. } => {
                tuple.encode().unwrap_or_else(|e| {
                    pgrx::error!("tqhnsw failed to encode vacuum grouped hot tuple: {e}")
                })
            }
        };
        if encoded.len() != tuple_len {
            pgrx::error!(
                "tqhnsw vacuum element tuple size changed from {} to {} on block {}",
                tuple_len,
                encoded.len(),
                block_number
            );
        }

        unsafe {
            ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), tuple_len);
        }
    }
}

unsafe fn repair_graph_connections_with_storage(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
    deleted_tids: &[page::ItemPointer],
) {
    if deleted_tids.is_empty() {
        return;
    }

    let metadata = unsafe { shared::read_metadata_page(index_relation) };
    let mut metric = unsafe { VacuumSearchMetric::for_relation(index_relation, heap_relation) };
    let deleted_tids = deleted_tids.iter().copied().collect::<HashSet<_>>();
    let repair_requests =
        unsafe { collect_repair_requests(index_relation, storage, metadata.m, &deleted_tids) };
    unsafe { unlink_deleted_graph_connections(index_relation, &deleted_tids) };
    let repair_plans = unsafe {
        plan_repair_replacements(
            index_relation,
            &metadata,
            &mut metric,
            storage,
            &deleted_tids,
            &repair_requests,
        )
    };
    unsafe { apply_repair_plans(index_relation, metadata.m, &deleted_tids, &repair_plans) };
}

unsafe fn collect_repair_requests(
    index_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
) -> Vec<LayerRepairRequest> {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut requests = Vec::new();

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
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            pgrx::error!("tqhnsw failed to open repair-request block {block_number}");
        }

        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        unsafe {
            collect_repair_requests_on_page(
                index_relation,
                page_ptr,
                page_size,
                block_number,
                storage,
                m,
                deleted_tids,
                &mut requests,
            )
        };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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
        let item_id = unsafe { &*shared::page_item_id(page_ptr, offset) };
        if item_id.lp_flags() == 0 {
            continue;
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!(
                "tqhnsw found invalid repair-request tuple bounds on block {block_number}"
            );
        }

        let tuple_bytes =
            unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
        let (level, deleted, heaptids_empty, neighbortid) = match storage {
            graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                    continue;
                }
                let element =
                    page::TqElementTuple::decode(tuple_bytes, code_len).unwrap_or_else(|e| {
                        pgrx::error!("tqhnsw failed to decode repair-request element tuple: {e}")
                    });
                (
                    element.level,
                    element.deleted,
                    element.heaptids.is_empty(),
                    element.neighbortid,
                )
            }
            graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                if tuple_bytes.first().copied() != Some(page::TQ_TURBO_HOT_TAG) {
                    continue;
                }
                let element = page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                    .unwrap_or_else(|e| {
                        pgrx::error!(
                            "tqhnsw failed to decode repair-request TurboQuant V3 tuple: {e}"
                        )
                    });
                (
                    element.level,
                    element.deleted,
                    element.heaptids.is_empty(),
                    element.neighbortid,
                )
            }
            graph::GraphStorageDescriptor::PqFastScan(layout) => {
                if tuple_bytes.first().copied() != Some(page::TQ_GROUPED_HOT_TAG) {
                    continue;
                }
                let element = page::TqGroupedHotTuple::decode(
                    tuple_bytes,
                    layout.binary_word_count,
                    layout.search_code_len,
                )
                .unwrap_or_else(|e| {
                    pgrx::error!("tqhnsw failed to decode repair-request grouped hot tuple: {e}")
                });
                (
                    element.level,
                    element.deleted,
                    element.heaptids.is_empty(),
                    element.neighbortid,
                )
            }
        };
        if deleted || heaptids_empty || neighbortid == page::ItemPointer::INVALID {
            continue;
        }

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
    index_relation: pg_sys::Relation,
    deleted_tids: &HashSet<page::ItemPointer>,
) {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let share_buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(share_buffer) } {
            pgrx::error!("tqhnsw failed to open repair block {block_number}");
        }

        unsafe { pg_sys::LockBuffer(share_buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let share_page_ptr = unsafe { pg_sys::BufferGetPage(share_buffer) }.cast::<u8>();
        let share_page_size = unsafe { pg_sys::BufferGetPageSize(share_buffer) as usize };
        let share_updates =
            unsafe { plan_page_pass2(share_page_ptr, share_page_size, block_number, deleted_tids) };
        unsafe { pg_sys::UnlockReleaseBuffer(share_buffer) };

        if share_updates.is_empty() {
            continue;
        }

        let exclusive_buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(exclusive_buffer) } {
            pgrx::error!("tqhnsw failed to reopen repair block {block_number}");
        }

        unsafe { rewrite_page_pass2(index_relation, exclusive_buffer, block_number, deleted_tids) };
    }
}

unsafe fn plan_repair_replacements(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    metric: &mut VacuumSearchMetric,
    storage: graph::GraphStorageDescriptor,
    deleted_tids: &HashSet<page::ItemPointer>,
    requests: &[LayerRepairRequest],
) -> Vec<LayerRepairPlan> {
    let mut plans = requests
        .iter()
        .filter_map(|request| unsafe {
            plan_repair_replacement(
                index_relation,
                metadata,
                metric,
                storage,
                deleted_tids,
                request,
            )
        })
        .collect::<Vec<_>>();
    plans.sort_unstable_by(|left, right| {
        compare_item_pointers(&left.neighbor_tid, &right.neighbor_tid)
            .then_with(|| left.layer.cmp(&right.layer))
    });
    plans
}

unsafe fn plan_repair_replacement(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    metric: &mut VacuumSearchMetric,
    storage: graph::GraphStorageDescriptor,
    deleted_tids: &HashSet<page::ItemPointer>,
    request: &LayerRepairRequest,
) -> Option<LayerRepairPlan> {
    let source =
        unsafe { graph::load_exact_graph_element(index_relation, request.source_tid, storage) };
    if source.deleted
        || source.heaptids.is_empty()
        || source.neighbortid != request.neighbor_tid
        || request.layer > source.level
    {
        return None;
    }

    let neighbors = unsafe { graph::load_graph_neighbors(index_relation, source.neighbortid) };
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
    let replacements =
        unsafe { search_repair_candidates_for_layer(index_relation, metric, &planner) };
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
        unsafe {
            top_up_repair_replacements_from_linear_scan(
                index_relation,
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
    index_relation: pg_sys::Relation,
    metric: &mut VacuumSearchMetric,
    planner: &RepairSearchPlanner<'_>,
) -> Vec<page::ItemPointer> {
    let mut seeds = Vec::new();

    if let Some(entry_candidate) = unsafe {
        load_vacuum_entry_candidate(
            index_relation,
            planner.metadata,
            planner.storage,
            metric,
            planner.source,
        )
    } {
        if planner.layer == 0 {
            seeds.push(unsafe {
                graph::greedy_descend_from_entry_with_storage(
                    index_relation,
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
                upper_seeds = unsafe {
                    graph::search_layer_result_candidates_with_storage(
                        index_relation,
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

    seeds.extend(planner.existing_layer.iter().filter_map(|tid| unsafe {
        let element = graph::load_exact_graph_element(index_relation, *tid, planner.storage);
        metric
            .score_graph_element(planner.metadata, planner.source, &element)
            .map(|score| search::BeamCandidate::new(*tid, score))
    }));
    dedup_beam_candidates_by_tid(&mut seeds);
    if seeds.is_empty() {
        return Vec::new();
    }

    let candidates = if planner.layer == 0 {
        unsafe {
            graph::search_layer0_result_candidates_with_storage(
                index_relation,
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
        unsafe {
            graph::search_layer_result_candidates_with_storage(
                index_relation,
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
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    storage: graph::GraphStorageDescriptor,
    metric: &mut VacuumSearchMetric,
    source_element: &graph::GraphElement,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    if metadata.entry_point == page::ItemPointer::INVALID {
        return None;
    }

    let entry =
        unsafe { graph::load_exact_graph_element(index_relation, metadata.entry_point, storage) };
    let entry_score = unsafe { metric.score_graph_element(metadata, source_element, &entry) }?;
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
        "validated tqhnsw indexes should always persist ef_construction >= 1"
    );
    ef.max(1)
}

fn dedup_beam_candidates_by_tid(candidates: &mut Vec<search::BeamCandidate<page::ItemPointer>>) {
    let mut seen = HashSet::new();
    candidates.retain(|candidate| seen.insert(candidate.node));
}

unsafe fn top_up_repair_replacements_from_linear_scan(
    index_relation: pg_sys::Relation,
    metric: &mut VacuumSearchMetric,
    planner: &LinearRepairPlanner<'_>,
    replacements: &mut Vec<page::ItemPointer>,
    target_len: usize,
) {
    if replacements.len() >= target_len {
        return;
    }

    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut scored = Vec::new();

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
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            pgrx::error!("tqhnsw failed to open linear-repair block {block_number}");
        }

        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        unsafe {
            collect_linear_repair_candidates_on_page(
                index_relation,
                page_ptr,
                page_size,
                block_number,
                metric,
                planner,
                replacements,
                &mut scored,
            )
        };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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
    index_relation: pg_sys::Relation,
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
        let item_id = unsafe { &*shared::page_item_id(page_ptr, offset) };
        if item_id.lp_flags() == 0 {
            continue;
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw found invalid linear-repair tuple bounds on block {block_number}");
        }

        let tuple_bytes =
            unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
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

        let candidate = match planner.storage {
            graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                    continue;
                }
                let element =
                    page::TqElementTuple::decode(tuple_bytes, code_len).unwrap_or_else(|e| {
                        pgrx::error!("tqhnsw failed to decode linear-repair element tuple: {e}")
                    });
                graph::GraphElement {
                    tid,
                    level: element.level,
                    deleted: element.deleted,
                    heaptids: element.heaptids,
                    gamma: element.gamma,
                    neighbortid: element.neighbortid,
                    code: element.code,
                }
            }
            graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                if tuple_bytes.first().copied() != Some(page::TQ_TURBO_HOT_TAG) {
                    continue;
                }
                let element = page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                    .unwrap_or_else(|e| {
                        pgrx::error!(
                            "tqhnsw failed to decode linear-repair TurboQuant V3 tuple: {e}"
                        )
                    });
                let rerank = unsafe {
                    graph::load_rerank_payload(
                        index_relation,
                        element.reranktid,
                        layout.rerank_code_len,
                    )
                };
                graph::GraphElement {
                    tid,
                    level: element.level,
                    deleted: element.deleted,
                    heaptids: element.heaptids,
                    gamma: rerank.gamma,
                    neighbortid: element.neighbortid,
                    code: rerank.code,
                }
            }
            graph::GraphStorageDescriptor::PqFastScan(layout) => {
                if tuple_bytes.first().copied() != Some(page::TQ_GROUPED_HOT_TAG) {
                    continue;
                }
                let element = page::TqGroupedHotTuple::decode(
                    tuple_bytes,
                    layout.binary_word_count,
                    layout.search_code_len,
                )
                .unwrap_or_else(|e| {
                    pgrx::error!("tqhnsw failed to decode linear-repair grouped hot tuple: {e}")
                });
                let rerank = unsafe {
                    load_grouped_rerank_payload_for_linear_repair_candidate(
                        index_relation,
                        page_ptr,
                        page_size,
                        block_number,
                        element.reranktid,
                        layout,
                    )
                };
                graph::GraphElement {
                    tid,
                    level: element.level,
                    deleted: element.deleted,
                    heaptids: element.heaptids,
                    gamma: rerank.gamma,
                    neighbortid: element.neighbortid,
                    code: rerank.code,
                }
            }
        };
        if candidate.deleted || candidate.heaptids.is_empty() || candidate.level < planner.layer {
            continue;
        }

        if let Some(score) =
            unsafe { metric.score_graph_element(planner.metadata, planner.source, &candidate) }
        {
            scored.push((tid, score));
        }
    }
}

unsafe fn load_grouped_rerank_payload_for_linear_repair_candidate(
    index_relation: pg_sys::Relation,
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
    rerank_tid: page::ItemPointer,
    layout: graph::PqFastScanLayout,
) -> graph::GroupedRerankPayload {
    if rerank_tid == page::ItemPointer::INVALID {
        pgrx::error!("tqhnsw linear-repair grouped candidate is missing a rerank payload tid");
    }

    if rerank_tid.block_number != block_number {
        return unsafe { graph::load_grouped_rerank_payload(index_relation, rerank_tid, layout) };
    }

    let item_id = unsafe { &*shared::page_item_id(page_ptr, rerank_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        pgrx::error!(
            "tqhnsw linear-repair rerank tuple slot {}/{} is unused",
            rerank_tid.block_number,
            rerank_tid.offset_number
        );
    }

    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        pgrx::error!(
            "tqhnsw found invalid linear-repair rerank tuple bounds on block {block_number}"
        );
    }

    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    let rerank =
        page::TqRerankTuple::decode(tuple_bytes, layout.rerank_code_len).unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to decode linear-repair rerank tuple: {e}")
        });
    graph::GroupedRerankPayload {
        tid: rerank_tid,
        gamma: rerank.gamma,
        code: rerank.code,
    }
}

unsafe fn apply_repair_plans(
    index_relation: pg_sys::Relation,
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

        unsafe {
            apply_repair_plans_on_page(
                index_relation,
                block_number,
                m,
                deleted_tids,
                &plans[start..end],
            )
        };
        start = end;
    }
}

unsafe fn apply_repair_plans_on_page(
    index_relation: pg_sys::Relation,
    block_number: u32,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
    plans: &[LayerRepairPlan],
) {
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
        pgrx::error!("tqhnsw failed to open layer0-repair block {block_number}");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut changed = false;

    let mut start = 0;
    while start < plans.len() {
        let neighbor_tid = plans[start].neighbor_tid;
        let mut end = start + 1;
        while end < plans.len() && plans[end].neighbor_tid == neighbor_tid {
            end += 1;
        }

        let item_id = unsafe { &*shared::page_item_id(page_ptr, neighbor_tid.offset_number) };
        if item_id.lp_flags() == 0 {
            pgrx::error!(
                "tqhnsw repair neighbor tuple slot {}/{} is unused",
                neighbor_tid.block_number,
                neighbor_tid.offset_number
            );
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw found invalid repair rewrite bounds on block {block_number}");
        }

        let tuple_bytes =
            unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
        let mut neighbor = page::TqNeighborTuple::decode(tuple_bytes)
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode repair neighbor tuple: {e}"));
        if neighbor.count as usize > neighbor.tids.len() {
            pgrx::error!(
                "tqhnsw repair neighbor tuple count {} exceeds payload tid count {}",
                neighbor.count,
                neighbor.tids.len()
            );
        }
        let mut tuple_changed = unlink_deleted_neighbor_refs(&mut neighbor.tids, deleted_tids);
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
            start = end;
            continue;
        }

        let encoded = neighbor
            .encode()
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode repair neighbor tuple: {e}"));
        if encoded.len() != tuple_len {
            pgrx::error!(
                "tqhnsw repair neighbor tuple size changed from {} to {} on block {}",
                tuple_len,
                encoded.len(),
                block_number
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
    index_relation: pg_sys::Relation,
    buffer: pg_sys::Buffer,
    block_number: u32,
    deleted_tids: &HashSet<page::ItemPointer>,
) {
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let updates = unsafe { plan_page_pass2(page_ptr, page_size, block_number, deleted_tids) };
    if updates.is_empty() {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return;
    }

    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let wal_page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    unsafe { apply_page_pass2_updates(wal_page_ptr, page_size, block_number, &updates) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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
        let item_id = unsafe { &*shared::page_item_id(page_ptr, offset) };
        if item_id.lp_flags() == 0 {
            continue;
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw found invalid repair tuple bounds on block {block_number}");
        }

        let tuple_bytes =
            unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
        if tuple_bytes.first().copied() != Some(page::TQ_NEIGHBOR_TAG) {
            continue;
        }

        let mut neighbor = page::TqNeighborTuple::decode(tuple_bytes)
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode repair neighbor tuple: {e}"));
        if neighbor.count as usize > neighbor.tids.len() {
            pgrx::error!(
                "tqhnsw repair neighbor tuple count {} exceeds payload tid count {}",
                neighbor.count,
                neighbor.tids.len()
            );
        }
        if !unlink_deleted_neighbor_refs(&mut neighbor.tids, deleted_tids) {
            continue;
        }

        updates.push(NeighborVacuumUpdate {
            tid: page::ItemPointer {
                block_number,
                offset_number: offset,
            },
            tuple: neighbor,
        });
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
        let item_id = unsafe { &*shared::page_item_id(page_ptr, update.tid.offset_number) };
        if item_id.lp_flags() == 0 {
            pgrx::error!(
                "tqhnsw repair neighbor tuple slot {}/{} is unused",
                update.tid.block_number,
                update.tid.offset_number
            );
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw found invalid repair rewrite bounds on block {block_number}");
        }

        let encoded = update
            .tuple
            .encode()
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode repair neighbor tuple: {e}"));
        if encoded.len() != tuple_len {
            pgrx::error!(
                "tqhnsw repair neighbor tuple size changed from {} to {} on block {}",
                tuple_len,
                encoded.len(),
                block_number
            );
        }

        unsafe {
            ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), tuple_len);
        }
    }
}

unsafe fn finalize_fully_dead_elements_with_storage(
    index_relation: pg_sys::Relation,
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

        unsafe {
            finalize_fully_dead_elements_on_page_with_storage(
                index_relation,
                block_number,
                storage,
                &tids[start..end],
            )
        };
        start = end;
    }
}

unsafe fn finalize_fully_dead_elements_on_page_with_storage(
    index_relation: pg_sys::Relation,
    block_number: u32,
    storage: graph::GraphStorageDescriptor,
    tids: &[page::ItemPointer],
) {
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
        pgrx::error!("tqhnsw failed to open finalize block {block_number}");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut updates = Vec::new();

    for tid in tids {
        let item_id = unsafe { &*shared::page_item_id(page_ptr, tid.offset_number) };
        if item_id.lp_flags() == 0 {
            pgrx::error!(
                "tqhnsw finalize element tuple slot {}/{} is unused",
                tid.block_number,
                tid.offset_number
            );
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw found invalid finalize tuple bounds on block {block_number}");
        }

        let tuple_bytes =
            unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
        match storage {
            graph::GraphStorageDescriptor::TurboQuant { code_len } => {
                let mut element = page::TqElementTuple::decode(tuple_bytes, code_len)
                    .unwrap_or_else(|e| {
                        pgrx::error!("tqhnsw failed to decode finalize element tuple: {e}")
                    });
                if element.deleted || !element.heaptids.is_empty() {
                    continue;
                }

                element.deleted = true;
                updates.push(ElementVacuumUpdate::TurboQuant {
                    tid: *tid,
                    tuple: element,
                });
            }
            graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => {
                let mut element =
                    page::TqTurboHotTuple::decode(tuple_bytes, layout.binary_word_count)
                        .unwrap_or_else(|e| {
                            pgrx::error!(
                                "tqhnsw failed to decode finalize TurboQuant V3 tuple: {e}"
                            )
                        });
                if element.deleted || !element.heaptids.is_empty() {
                    continue;
                }

                element.deleted = true;
                updates.push(ElementVacuumUpdate::TurboQuantHot {
                    tid: *tid,
                    tuple: element,
                });
            }
            graph::GraphStorageDescriptor::PqFastScan(layout) => {
                let mut element = page::TqGroupedHotTuple::decode(
                    tuple_bytes,
                    layout.binary_word_count,
                    layout.search_code_len,
                )
                .unwrap_or_else(|e| {
                    pgrx::error!("tqhnsw failed to decode finalize grouped hot tuple: {e}")
                });
                if element.deleted || !element.heaptids.is_empty() {
                    continue;
                }

                element.deleted = true;
                updates.push(ElementVacuumUpdate::PqFastScanHot {
                    tid: *tid,
                    tuple: element,
                });
            }
        }
    }

    if updates.is_empty() {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return;
    }

    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let wal_page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    unsafe { apply_page_pass1_updates(wal_page_ptr, page_size, block_number, &updates) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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
    let state = unsafe { &*(state.cast::<DebugVacuumCallbackState>()) };
    state
        .dead_tids
        .contains(&unsafe { shared::decode_heap_tid(itemptr) })
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_vacuum_remove_heap_tids(
    index_oid: pg_sys::Oid,
    dead_tids: &[page::ItemPointer],
) -> pg_sys::IndexBulkDeleteResult {
    let index_relation = unsafe {
        pg_sys::index_open(
            index_oid,
            pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
        )
    };
    let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
    let heap_relation = if heap_oid == pg_sys::InvalidOid {
        ptr::null_mut()
    } else {
        unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) }
    };
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
    info.heaprel = heap_relation;
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;
    let mut callback_state = DebugVacuumCallbackState {
        dead_tids: dead_tids.iter().copied().collect(),
    };

    let stats = unsafe {
        tqhnsw_ambulkdelete(
            info_ptr,
            ptr::null_mut(),
            Some(debug_vacuum_dead_tid_callback),
            (&mut callback_state as *mut DebugVacuumCallbackState).cast(),
        )
    };
    let stats = unsafe { tqhnsw_amvacuumcleanup(info_ptr, stats) };
    let result = unsafe { *stats };

    unsafe {
        if !heap_relation.is_null() {
            pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }
        pg_sys::index_close(
            index_relation,
            pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
        )
    };
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
