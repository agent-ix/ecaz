use std::{ffi::c_void, ptr};

use pgrx::{itemptr::item_pointer_set_all, pg_sys, PgBox};

use super::{page, shared, wal};
use crate::quant::prod::payload_len;

type BulkDeleteCallback =
    unsafe extern "C-unwind" fn(itemptr: pg_sys::ItemPointer, state: *mut c_void) -> bool;

#[derive(Debug, Clone)]
struct ElementVacuumUpdate {
    tid: page::ItemPointer,
    tuple: page::TqElementTuple,
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
