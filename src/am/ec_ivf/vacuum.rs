use std::ffi::c_void;

use pgrx::{itemptr::item_pointer_set_all, pg_sys, PgBox};

use super::page;
use crate::storage::page::ItemPointer;

type BulkDeleteCallback =
    unsafe extern "C-unwind" fn(itemptr: pg_sys::ItemPointer, state: *mut c_void) -> bool;

#[derive(Debug, Default)]
struct ListBulkDeleteResult {
    removed_heap_tids: u64,
    live_heap_tids: u64,
}

impl ListBulkDeleteResult {
    fn record_live_posting(&mut self, heap_tid_count: usize) -> Result<(), String> {
        self.live_heap_tids = self
            .live_heap_tids
            .checked_add(
                u64::try_from(heap_tid_count)
                    .map_err(|_| "ec_ivf live posting heap tid count exceeds u64".to_owned())?,
            )
            .ok_or_else(|| "ec_ivf live heap tid count overflow".to_owned())?;
        Ok(())
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: pg_sys::IndexBulkDeleteCallback,
    callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if info.is_null() {
                pgrx::error!("ec_ivf ambulkdelete requires vacuum info")
            }
            let Some(callback) = callback else {
                return noop_vacuum_stats((*info).index, stats);
            };

            run_bulkdelete((*info).index, stats, callback, callback_state)
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if info.is_null() {
                pgrx::error!("ec_ivf amvacuumcleanup requires vacuum info")
            }

            noop_vacuum_stats((*info).index, stats)
        })
    }
}

unsafe fn noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    unsafe { finish_vacuum_stats(index_relation, stats, &metadata) }
}

unsafe fn run_bulkdelete(
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
    let mut metadata = unsafe { page::read_metadata_page(index_relation) };

    if metadata.directory_head == ItemPointer::INVALID {
        if metadata.total_live_tuples != 0 {
            pgrx::error!("ec_ivf metadata has live tuples but no directory head");
        }
        return unsafe { finish_vacuum_stats(index_relation, stats, &metadata) };
    }

    let payload_len = page_payload_len(&metadata).unwrap_or_else(|e| pgrx::error!("{e}"));
    let mut next_tid = metadata.directory_head;
    let mut removed_heap_tids = 0_u64;
    let mut live_heap_tids = 0_u64;
    for expected_list_id in 0..metadata.nlists {
        let directory_tid = next_tid;
        let (mut directory, following_tid) =
            unsafe { page::read_ivf_list_directory_and_next(index_relation, directory_tid) }
                .unwrap_or_else(|e| pgrx::error!("{e}"));
        if directory.list_id != expected_list_id {
            pgrx::error!(
                "ec_ivf directory order mismatch: got list {}, expected {}",
                directory.list_id,
                expected_list_id
            );
        }

        let list_result = unsafe {
            bulkdelete_list_postings(
                index_relation,
                &directory,
                directory_tid.block_number,
                payload_len,
                callback,
                callback_state,
            )
        }
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        live_heap_tids = live_heap_tids
            .checked_add(list_result.live_heap_tids)
            .unwrap_or_else(|| pgrx::error!("ec_ivf live heap tid count overflow during vacuum"));

        let (repaired_head, repaired_tail) = (directory.head_block, directory.tail_block);
        if list_result.removed_heap_tids > 0
            || directory.live_count != list_result.live_heap_tids
            || directory.head_block != repaired_head
            || directory.tail_block != repaired_tail
        {
            directory.live_count = list_result.live_heap_tids;
            directory.dead_count = directory
                .dead_count
                .checked_add(list_result.removed_heap_tids)
                .unwrap_or_else(|| {
                    pgrx::error!(
                        "ec_ivf list {} dead count overflow during vacuum",
                        directory.list_id
                    )
                });
            directory.head_block = repaired_head;
            directory.tail_block = repaired_tail;
            unsafe { page::rewrite_ivf_list_directory(index_relation, directory_tid, directory) }
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            removed_heap_tids = removed_heap_tids
                .checked_add(list_result.removed_heap_tids)
                .unwrap_or_else(|| pgrx::error!("ec_ivf removed heap tid count overflow"));
        }

        next_tid = following_tid;
    }

    if removed_heap_tids > 0 || metadata.total_live_tuples != live_heap_tids {
        metadata.total_live_tuples = live_heap_tids;
        metadata.total_dead_tuples = metadata
            .total_dead_tuples
            .checked_add(removed_heap_tids)
            .unwrap_or_else(|| pgrx::error!("ec_ivf metadata dead count overflow during vacuum"));
        unsafe { page::initialize_metadata_page(index_relation, metadata) };
        unsafe {
            (*stats).tuples_removed += removed_heap_tids as f64;
        }
    }

    unsafe { finish_vacuum_stats(index_relation, stats, &metadata) }
}

fn page_payload_len(metadata: &page::MetadataPage) -> Result<usize, String> {
    let pq_group_size = if metadata.storage_format == super::options::StorageFormat::PqFastScan
        && metadata.pq_group_size > 0
    {
        Some(usize::from(metadata.pq_group_size))
    } else {
        None
    };
    super::quantizer::IvfQuantizer::resolve_with_pq_group_size(
        metadata.storage_format,
        metadata.dimensions as usize,
        pq_group_size,
    )
    .map(|quantizer| quantizer.payload_len())
}

unsafe fn bulkdelete_list_postings(
    index_relation: pg_sys::Relation,
    directory: &page::IvfListDirectoryTuple,
    directory_block_number: pg_sys::BlockNumber,
    payload_len: usize,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> Result<ListBulkDeleteResult, String> {
    let mut result = ListBulkDeleteResult::default();
    unsafe {
        page::rewrite_ivf_postings_for_list_blocks(
            index_relation,
            directory.list_id,
            directory.head_block,
            directory.tail_block,
            payload_len,
            &[directory_block_number],
            |posting_tid, mut posting| {
                bulkdelete_posting(
                    &mut result,
                    posting_tid,
                    &mut posting,
                    callback,
                    callback_state,
                )
            },
        )?
    };

    Ok(result)
}

fn bulkdelete_posting(
    result: &mut ListBulkDeleteResult,
    _posting_tid: ItemPointer,
    posting: &mut page::IvfPostingTuple,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> Result<page::IvfPostingRewrite, String> {
    if posting.deleted {
        return Ok(page::IvfPostingRewrite::Delete);
    }
    let starting_len = posting.heaptids.len();
    posting
        .heaptids
        .retain(|heap_tid| unsafe { !heap_tid_is_dead(*heap_tid, callback, callback_state) });
    let removed = starting_len.saturating_sub(posting.heaptids.len());

    let rewrite = if posting.heaptids.is_empty() {
        page::IvfPostingRewrite::Delete
    } else {
        result.record_live_posting(posting.heaptids.len())?;
        if removed > 0 {
            page::IvfPostingRewrite::Rewrite(posting.clone())
        } else {
            page::IvfPostingRewrite::Keep
        }
    };
    if removed > 0 {
        result.removed_heap_tids = result
            .removed_heap_tids
            .checked_add(
                u64::try_from(removed)
                    .map_err(|_| "ec_ivf removed heap tid count exceeds u64".to_owned())?,
            )
            .ok_or_else(|| "ec_ivf removed heap tid count overflow".to_owned())?;
    }

    Ok(rewrite)
}

unsafe fn finish_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    metadata: &page::MetadataPage,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };

    unsafe {
        (*stats).num_pages = block_count;
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = metadata.total_live_tuples as f64;
    }

    stats
}

unsafe fn heap_tid_is_dead(
    heap_tid: ItemPointer,
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
    dead_tids: std::collections::HashSet<ItemPointer>,
}

#[cfg(any(test, feature = "pg_test"))]
unsafe extern "C-unwind" fn debug_vacuum_dead_tid_callback(
    itemptr: pg_sys::ItemPointer,
    state: *mut c_void,
) -> bool {
    let state = unsafe { &*(state.cast::<DebugVacuumCallbackState>()) };
    state
        .dead_tids
        .contains(&unsafe { super::build::decode_heap_tid(itemptr, "debug vacuum") })
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_vacuum_stats(
    index_oid: pg_sys::Oid,
) -> pg_sys::IndexBulkDeleteResult {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;

    let stats =
        unsafe { ec_ivf_ambulkdelete(info_ptr, std::ptr::null_mut(), None, std::ptr::null_mut()) };
    let stats = unsafe { ec_ivf_amvacuumcleanup(info_ptr, stats) };
    let result = unsafe { *stats };

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_vacuum_remove_heap_tids(
    index_oid: pg_sys::Oid,
    dead_tids: &[ItemPointer],
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
        ec_ivf_ambulkdelete(
            info_ptr,
            std::ptr::null_mut(),
            Some(debug_vacuum_dead_tid_callback),
            (&mut callback_state as *mut DebugVacuumCallbackState).cast(),
        )
    };
    let stats = unsafe { ec_ivf_amvacuumcleanup(info_ptr, stats) };
    let result = unsafe { *stats };

    unsafe {
        pg_sys::index_close(
            index_relation,
            pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
        )
    };
    result
}
