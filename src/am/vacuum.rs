use std::{collections::HashSet, ffi::c_void, ptr};

use pgrx::{itemptr::item_pointer_set_all, pg_sys, PgBox};

use super::{graph, page, search, shared, wal};
use crate::quant::prod::payload_len;

type BulkDeleteCallback =
    unsafe extern "C-unwind" fn(itemptr: pg_sys::ItemPointer, state: *mut c_void) -> bool;

#[derive(Debug, Clone)]
struct ElementVacuumUpdate {
    tid: page::ItemPointer,
    tuple: page::TqElementTuple,
}

#[derive(Debug, Clone)]
struct NeighborVacuumUpdate {
    tid: page::ItemPointer,
    tuple: page::TqNeighborTuple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Layer0RepairRequest {
    source_tid: page::ItemPointer,
    neighbor_tid: page::ItemPointer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Layer0RepairPlan {
    neighbor_tid: page::ItemPointer,
    replacement_tids: Vec<page::ItemPointer>,
}

#[derive(Debug, Clone, Copy)]
struct LinearRepairPlanner<'a> {
    metadata: &'a page::MetadataPage,
    code_len: usize,
    source_tid: page::ItemPointer,
    source_code: &'a [u8],
    deleted_tids: &'a HashSet<page::ItemPointer>,
    existing_set: &'a HashSet<page::ItemPointer>,
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
            let Some(callback) = callback else {
                return shared::tqhnsw_noop_vacuum_stats((*info).index, stats);
            };
            run_pass1_vacuum((*info).index, stats, callback, callback_state)
        })
    }
}

pub(super) unsafe extern "C-unwind" fn tqhnsw_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe { pgrx::pgrx_extern_c_guard(|| shared::tqhnsw_noop_vacuum_stats((*info).index, stats)) }
}

unsafe fn run_pass1_vacuum(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let metadata = unsafe { shared::read_metadata_page(index_relation) };
    let code_len = payload_len(usize::from(metadata.dimensions), metadata.bits)
        .checked_sub(4)
        .expect("payload length should include gamma");

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
                code_len,
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
                code_len,
                callback,
                callback_state,
            )
        };
        live_elements += final_plan.live_elements;
        removed_heap_tids += final_plan.removed_heap_tids;
        finalize_tids.extend(final_plan.finalize_tids);
    }

    unsafe { repair_graph_connections(index_relation, &finalize_tids) };
    unsafe { finalize_fully_dead_elements(index_relation, code_len, &finalize_tids) };

    unsafe {
        (*stats).num_pages = block_count;
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = live_elements as f64;
        (*stats).tuples_removed += removed_heap_tids as f64;
    }
    stats
}

unsafe fn rewrite_page_pass1(
    index_relation: pg_sys::Relation,
    buffer: pg_sys::Buffer,
    block_number: u32,
    code_len: usize,
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
            code_len,
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
    code_len: usize,
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
        if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
            continue;
        }

        let tid = page::ItemPointer {
            block_number,
            offset_number: offset,
        };
        let mut element = page::TqElementTuple::decode(tuple_bytes, code_len)
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode vacuum element tuple: {e}"));
        let starting_len = element.heaptids.len();
        element
            .heaptids
            .retain(|heap_tid| unsafe { !heap_tid_is_dead(*heap_tid, callback, callback_state) });
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
        plan.updates.push(ElementVacuumUpdate {
            tid,
            tuple: element,
        });
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
        let item_id = unsafe { &*shared::page_item_id(page_ptr, update.tid.offset_number) };
        if item_id.lp_flags() == 0 {
            pgrx::error!(
                "tqhnsw vacuum element tuple slot {}/{} is unused",
                update.tid.block_number,
                update.tid.offset_number
            );
        }

        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw found invalid vacuum rewrite bounds on block {block_number}");
        }

        let encoded = update
            .tuple
            .encode()
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode vacuum element tuple: {e}"));
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

unsafe fn repair_graph_connections(
    index_relation: pg_sys::Relation,
    deleted_tids: &[page::ItemPointer],
) {
    if deleted_tids.is_empty() {
        return;
    }

    let metadata = unsafe { shared::read_metadata_page(index_relation) };
    let code_len = payload_len(usize::from(metadata.dimensions), metadata.bits)
        .checked_sub(4)
        .expect("payload length should include gamma");
    let deleted_tids = deleted_tids.iter().copied().collect::<HashSet<_>>();
    let repair_requests = unsafe {
        collect_layer0_repair_requests(index_relation, code_len, metadata.m, &deleted_tids)
    };
    unsafe { unlink_deleted_graph_connections(index_relation, &deleted_tids) };
    let repair_plans = unsafe {
        plan_layer0_repair_replacements(
            index_relation,
            &metadata,
            code_len,
            &deleted_tids,
            &repair_requests,
        )
    };
    unsafe { apply_layer0_repair_plans(index_relation, metadata.m, &deleted_tids, &repair_plans) };
}

unsafe fn collect_layer0_repair_requests(
    index_relation: pg_sys::Relation,
    code_len: usize,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
) -> Vec<Layer0RepairRequest> {
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
            collect_layer0_repair_requests_on_page(
                index_relation,
                page_ptr,
                page_size,
                block_number,
                code_len,
                m,
                deleted_tids,
                &mut requests,
            )
        };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    requests.sort_unstable_by(|left, right| {
        compare_item_pointers(&left.neighbor_tid, &right.neighbor_tid)
            .then_with(|| compare_item_pointers(&left.source_tid, &right.source_tid))
    });
    requests.dedup();
    requests
}

unsafe fn collect_layer0_repair_requests_on_page(
    index_relation: pg_sys::Relation,
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
    code_len: usize,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
    requests: &mut Vec<Layer0RepairRequest>,
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
        if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
            continue;
        }

        let element = page::TqElementTuple::decode(tuple_bytes, code_len).unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to decode repair-request element tuple: {e}")
        });
        if element.deleted
            || element.heaptids.is_empty()
            || element.neighbortid == page::ItemPointer::INVALID
        {
            continue;
        }

        let neighbors = unsafe { graph::load_graph_neighbors(index_relation, element.neighbortid) };
        if layer0_slice_contains_deleted_ref(&neighbors.tids, m, deleted_tids) {
            requests.push(Layer0RepairRequest {
                source_tid: page::ItemPointer {
                    block_number,
                    offset_number: offset,
                },
                neighbor_tid: element.neighbortid,
            });
        }
    }
}

fn layer0_slice_contains_deleted_ref(
    neighbor_tids: &[page::ItemPointer],
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
) -> bool {
    neighbor_tids
        .iter()
        .take(layer0_slot_end(m, neighbor_tids.len()))
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

unsafe fn plan_layer0_repair_replacements(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    code_len: usize,
    deleted_tids: &HashSet<page::ItemPointer>,
    requests: &[Layer0RepairRequest],
) -> Vec<Layer0RepairPlan> {
    let mut plans = requests
        .iter()
        .filter_map(|request| unsafe {
            plan_layer0_repair_replacement(
                index_relation,
                metadata,
                code_len,
                deleted_tids,
                request,
            )
        })
        .collect::<Vec<_>>();
    plans.sort_unstable_by(|left, right| {
        compare_item_pointers(&left.neighbor_tid, &right.neighbor_tid)
    });
    plans
}

unsafe fn plan_layer0_repair_replacement(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    code_len: usize,
    deleted_tids: &HashSet<page::ItemPointer>,
    request: &Layer0RepairRequest,
) -> Option<Layer0RepairPlan> {
    let source = unsafe { graph::load_graph_element(index_relation, request.source_tid, code_len) };
    if source.deleted || source.heaptids.is_empty() || source.neighbortid != request.neighbor_tid {
        return None;
    }

    let neighbors = unsafe { graph::load_graph_neighbors(index_relation, source.neighbortid) };
    let layer0_end = layer0_slot_end(metadata.m, neighbors.tids.len());
    if layer0_end == 0 {
        return None;
    }

    let layer0_slice = &neighbors.tids[..layer0_end];
    let free_slots = layer0_slice
        .iter()
        .filter(|tid| **tid == page::ItemPointer::INVALID)
        .count();
    if free_slots == 0 {
        return None;
    }

    let existing_layer0 = layer0_slice
        .iter()
        .copied()
        .filter(|tid| *tid != page::ItemPointer::INVALID && !deleted_tids.contains(tid))
        .collect::<Vec<_>>();
    let existing_set = existing_layer0.iter().copied().collect::<HashSet<_>>();
    let mut seeds = Vec::new();

    if let Some(entry_candidate) =
        unsafe { load_vacuum_entry_candidate(index_relation, metadata, &source.code) }
    {
        seeds.push(unsafe {
            graph::greedy_descend_from_entry(
                index_relation,
                code_len,
                usize::from(metadata.m),
                entry_candidate,
                |neighbor| score_vacuum_graph_element(metadata, &source.code, neighbor),
            )
        });
    }

    seeds.extend(existing_layer0.iter().filter_map(|tid| unsafe {
        let element = graph::load_graph_element(index_relation, *tid, code_len);
        score_vacuum_graph_element(metadata, &source.code, &element)
            .map(|score| search::BeamCandidate::new(*tid, score))
    }));
    dedup_beam_candidates_by_tid(&mut seeds);
    if seeds.is_empty() {
        return None;
    }

    let replacements = unsafe {
        graph::search_layer0_result_candidates(
            index_relation,
            code_len,
            usize::from(metadata.m),
            repair_ef_construction(metadata),
            seeds,
            |neighbor_tid| neighbor_tid != source.tid && !deleted_tids.contains(&neighbor_tid),
            |neighbor| score_vacuum_graph_element(metadata, &source.code, neighbor),
        )
    }
    .into_iter()
    .map(|candidate| candidate.node)
    .filter(|tid| {
        *tid != source.tid
            && *tid != page::ItemPointer::INVALID
            && !existing_set.contains(tid)
            && !deleted_tids.contains(tid)
    })
    .take(free_slots)
    .collect::<Vec<_>>();
    let mut replacements = replacements;
    if replacements.len() < free_slots {
        let linear_planner = LinearRepairPlanner {
            metadata,
            code_len,
            source_tid: source.tid,
            source_code: &source.code,
            deleted_tids,
            existing_set: &existing_set,
        };
        unsafe {
            top_up_layer0_repair_replacements_from_linear_scan(
                index_relation,
                &linear_planner,
                &mut replacements,
                free_slots,
            )
        };
    }
    if replacements.is_empty() {
        return None;
    }

    Some(Layer0RepairPlan {
        neighbor_tid: source.neighbortid,
        replacement_tids: replacements,
    })
}

unsafe fn load_vacuum_entry_candidate(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
    source_code: &[u8],
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    if metadata.entry_point == page::ItemPointer::INVALID {
        return None;
    }

    let entry = unsafe {
        graph::load_graph_element(index_relation, metadata.entry_point, source_code.len())
    };
    let entry_score = score_vacuum_graph_element(metadata, source_code, &entry)?;
    Some(search::BeamCandidate::new(entry.tid, entry_score))
}

fn score_vacuum_graph_element(
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

unsafe fn top_up_layer0_repair_replacements_from_linear_scan(
    index_relation: pg_sys::Relation,
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
                page_ptr,
                page_size,
                block_number,
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
    page_ptr: *mut u8,
    page_size: usize,
    block_number: u32,
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
        if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
            continue;
        }

        let tid = page::ItemPointer {
            block_number,
            offset_number: offset,
        };
        if tid == planner.source_tid
            || planner.deleted_tids.contains(&tid)
            || planner.existing_set.contains(&tid)
            || replacements.contains(&tid)
        {
            continue;
        }

        let element = page::TqElementTuple::decode(tuple_bytes, planner.code_len).unwrap_or_else(
            |e| pgrx::error!("tqhnsw failed to decode linear-repair element tuple: {e}"),
        );
        if element.deleted || element.heaptids.is_empty() {
            continue;
        }

        scored.push((
            tid,
            -crate::score_code_inner_product(
                planner.metadata.dimensions as usize,
                planner.metadata.bits,
                planner.metadata.seed,
                planner.source_code,
                &element.code,
            ),
        ));
    }
}

unsafe fn apply_layer0_repair_plans(
    index_relation: pg_sys::Relation,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
    plans: &[Layer0RepairPlan],
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
            apply_layer0_repair_plans_on_page(
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

unsafe fn apply_layer0_repair_plans_on_page(
    index_relation: pg_sys::Relation,
    block_number: u32,
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
    plans: &[Layer0RepairPlan],
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

    for plan in plans {
        let item_id = unsafe { &*shared::page_item_id(page_ptr, plan.neighbor_tid.offset_number) };
        if item_id.lp_flags() == 0 {
            pgrx::error!(
                "tqhnsw repair neighbor tuple slot {}/{} is unused",
                plan.neighbor_tid.block_number,
                plan.neighbor_tid.offset_number
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
        if !apply_layer0_repair_plan(&mut neighbor.tids, m, deleted_tids, &plan.replacement_tids) {
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
    }

    if changed {
        unsafe { wal_txn.finish() };
    } else {
        std::mem::drop(wal_txn);
    }
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

fn apply_layer0_repair_plan(
    neighbor_tids: &mut [page::ItemPointer],
    m: u16,
    deleted_tids: &HashSet<page::ItemPointer>,
    replacement_tids: &[page::ItemPointer],
) -> bool {
    let mut changed = unlink_deleted_neighbor_refs(neighbor_tids, deleted_tids);
    let layer0_end = layer0_slot_end(m, neighbor_tids.len());
    if layer0_end == 0 {
        return changed;
    }

    let layer0_slice = &mut neighbor_tids[..layer0_end];
    for candidate_tid in replacement_tids {
        if *candidate_tid == page::ItemPointer::INVALID
            || deleted_tids.contains(candidate_tid)
            || layer0_slice.contains(candidate_tid)
        {
            continue;
        }

        let Some(slot) = layer0_slice
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

fn layer0_slot_end(m: u16, total_slots: usize) -> usize {
    usize::from(m).saturating_mul(2).min(total_slots)
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

unsafe fn finalize_fully_dead_elements(
    index_relation: pg_sys::Relation,
    code_len: usize,
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
            finalize_fully_dead_elements_on_page(
                index_relation,
                block_number,
                code_len,
                &tids[start..end],
            )
        };
        start = end;
    }
}

unsafe fn finalize_fully_dead_elements_on_page(
    index_relation: pg_sys::Relation,
    block_number: u32,
    code_len: usize,
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
        let mut element = page::TqElementTuple::decode(tuple_bytes, code_len).unwrap_or_else(|e| {
            pgrx::error!("tqhnsw failed to decode finalize element tuple: {e}")
        });
        if element.deleted || !element.heaptids.is_empty() {
            continue;
        }

        element.deleted = true;
        updates.push(ElementVacuumUpdate {
            tid: *tid,
            tuple: element,
        });
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
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
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
        pg_sys::index_close(
            index_relation,
            pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
        )
    };
    result
}
