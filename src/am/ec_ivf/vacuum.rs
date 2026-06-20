use std::ffi::c_void;
use std::ptr::NonNull;

use pgrx::{itemptr::item_pointer_set_all, pg_sys};

use super::page;
use crate::am::common::{
    callback::pg_am_callback,
    vacuum::{
        add_index_bulk_delete_tuples_removed, alloc_index_bulk_delete_result,
        set_index_bulk_delete_summary,
    },
};
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
    pg_am_callback!({
        if info.is_null() {
            pgrx::error!("ec_ivf ambulkdelete requires vacuum info")
        }
        let Some(callback) = callback else {
            return noop_vacuum_stats((*info).index, stats);
        };

        run_bulkdelete((*info).index, stats, callback, callback_state)
    })
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    pg_am_callback!({
        if info.is_null() {
            pgrx::error!("ec_ivf amvacuumcleanup requires vacuum info")
        }

        noop_vacuum_stats((*info).index, stats)
    })
}

/// # Safety
/// Caller passes the live IVF `index_relation` being vacuumed; `stats`
/// is either the running bulk-delete result or null on first entry.
unsafe fn noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    // SAFETY: caller passes the live IVF index relation being vacuumed.
    let metadata = page::read_metadata_page(index_relation);
    finish_vacuum_stats(index_relation, stats, &metadata)
}

/// # Safety
/// `index_relation` is the live IVF index being vacuumed; `callback` /
/// `callback_state` (when non-None) come from PostgreSQL's bulkdelete
/// entry point and remain valid for the duration of the call.
unsafe fn run_bulkdelete(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        crate::fault::maybe_fail_palloc("ec_ivf bulkdelete stats");
        alloc_index_bulk_delete_result().into()
    } else {
        stats
    };
    // SAFETY: `index_relation` is the live IVF index relation being vacuumed.
    let mut metadata = page::read_metadata_page(index_relation);

    if metadata.directory_head == ItemPointer::INVALID {
        if metadata.total_live_tuples != 0 {
            pgrx::error!("ec_ivf metadata has live tuples but no directory head");
        }
        return finish_vacuum_stats(index_relation, stats, &metadata);
    }

    let payload_len = page_payload_len(&metadata).unwrap_or_else(|e| pgrx::error!("{e}"));
    let mut next_tid = metadata.directory_head;
    let mut removed_heap_tids = 0_u64;
    let mut live_heap_tids = 0_u64;
    for expected_list_id in 0..metadata.nlists {
        let directory_tid = next_tid;
        let (mut directory, following_tid) =
            page::read_ivf_list_directory_and_next(index_relation, directory_tid)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
        if directory.list_id != expected_list_id {
            pgrx::error!(
                "ec_ivf directory order mismatch: got list {}, expected {}",
                directory.list_id,
                expected_list_id
            );
        }

        let list_result = bulkdelete_list_postings(
            index_relation,
            &directory,
            directory_tid.block_number,
            payload_len,
            callback,
            callback_state,
        )
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
            page::rewrite_ivf_list_directory(index_relation, directory_tid, directory)
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
        // SAFETY: `index_relation` is the live IVF index relation and the
        // updated metadata preserves the same on-disk format fields.
        page::initialize_metadata_page(index_relation, metadata);
        let stats_handle = NonNull::new(stats)
            .unwrap_or_else(|| pgrx::error!("ec_ivf vacuum stats unexpectedly null"));
        add_index_bulk_delete_tuples_removed(stats_handle, removed_heap_tids);
    }

    // Task 111g: maintain the compact rerank sidecar. Tombstone dead heap TIDs
    // (set heap_tid = INVALID, same byte length) so the index-side rerank lookup
    // never matches a reused heap line pointer to a stale payload. Space is
    // reclaimed on REINDEX. No-op when there is no sidecar (source placement /
    // v2-era index).
    if metadata.rerank_sidecar_head != ItemPointer::INVALID {
        bulkdelete_rerank_sidecar(
            index_relation,
            metadata.rerank_sidecar_head,
            callback,
            callback_state,
        )
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    }

    finish_vacuum_stats(index_relation, stats, &metadata)
}

/// Walk the rerank sidecar chain and tombstone entries whose heap TID is now
/// dead. Each block is rewritten in place at the same byte length (dead entries
/// keep their payload slot but get heap_tid = INVALID); only blocks that change
/// are rewritten.
///
/// # Safety
/// `index_relation` is the live IVF index being vacuumed; the chain rooted at
/// `head` is read page by page; `callback`/`callback_state` are valid for the
/// call.
unsafe fn bulkdelete_rerank_sidecar(
    index_relation: pg_sys::Relation,
    head: ItemPointer,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> Result<(), String> {
    let mut next_tid = head;
    while next_tid != ItemPointer::INVALID {
        let block_tid = next_tid;
        let (mut block, following) =
            page::read_ivf_rerank_sidecar_block_and_next(index_relation, block_tid)?;
        let mut changed = false;
        for heap_tid in block.heap_tids.iter_mut() {
            if *heap_tid != ItemPointer::INVALID
                && heap_tid_is_dead(*heap_tid, callback, callback_state)
            {
                *heap_tid = ItemPointer::INVALID;
                changed = true;
            }
        }
        if changed {
            page::rewrite_ivf_rerank_sidecar_block(index_relation, block_tid, &block)?;
        }
        next_tid = following;
    }
    Ok(())
}

fn page_payload_len(metadata: &page::MetadataPage) -> Result<usize, String> {
    let pq_group_size = if metadata.storage_format == super::options::StorageFormat::PqFastScan
        && metadata.pq_group_size > 0
    {
        Some(usize::from(metadata.pq_group_size))
    } else {
        None
    };
    // Pass the stored quant_bits: RaBitQ payload width depends on the per-dim
    // code width (coarse_rerank builds at 1 bit). Resolving without bits would
    // default to 4-bit and mis-size the dense posting payload during vacuum.
    super::quantizer::IvfQuantizer::resolve_with_pq_group_size_and_bits(
        metadata.storage_format,
        metadata.dimensions as usize,
        pq_group_size,
        Some(metadata.quant_bits),
    )
    .map(|quantizer| quantizer.payload_len())
}

/// # Safety
/// `index_relation` is the live IVF index relation; `result` is the
/// running aggregator owned by the bulkdelete pass; `callback` and
/// `callback_state` remain valid for the duration of the call.
unsafe fn bulkdelete_list_postings(
    index_relation: pg_sys::Relation,
    directory: &page::IvfListDirectoryTuple,
    directory_block_number: pg_sys::BlockNumber,
    payload_len: usize,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> Result<ListBulkDeleteResult, String> {
    let result = std::cell::RefCell::new(ListBulkDeleteResult::default());
    page::rewrite_ivf_posting_entries_for_list_blocks(
        index_relation,
        directory.list_id,
        directory.head_block,
        directory.tail_block,
        payload_len,
        &[directory_block_number],
        |posting_tid, mut posting| {
            let mut result = result.borrow_mut();
            bulkdelete_posting(
                &mut result,
                posting_tid,
                &mut posting,
                callback,
                callback_state,
            )
        },
        |posting_tid, posting_block| {
            let mut result = result.borrow_mut();
            bulkdelete_dense_posting_block(
                &mut result,
                posting_tid,
                posting_block,
                callback,
                callback_state,
            )
        },
    )?;

    Ok(result.into_inner())
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
        .retain(|heap_tid| !heap_tid_is_dead(*heap_tid, callback, callback_state));
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

fn bulkdelete_dense_posting_block(
    result: &mut ListBulkDeleteResult,
    _posting_tid: ItemPointer,
    mut posting_block: page::IvfDensePostingBlockTuple,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> Result<page::IvfDensePostingBlockRewrite, String> {
    let mut removed = 0_usize;
    let mut changed = false;
    for index in 0..posting_block.len() {
        if posting_block.is_deleted(index) {
            continue;
        }
        let start = usize::try_from(posting_block.heap_tid_offsets[index])
            .map_err(|_| "ec_ivf dense posting heap tid offset exceeds usize".to_owned())?;
        let count = usize::from(posting_block.heap_tid_counts[index]);
        let end = start
            .checked_add(count)
            .ok_or_else(|| "ec_ivf dense posting heap tid range overflow".to_owned())?;
        let heap_tids = posting_block
            .heap_tids
            .get(start..end)
            .ok_or_else(|| "ec_ivf dense posting heap tid range is out of bounds".to_owned())?;
        if heap_tids
            .iter()
            .all(|heap_tid| heap_tid_is_dead(*heap_tid, callback, callback_state))
        {
            posting_block.mark_deleted(index);
            removed = removed.saturating_add(count);
            changed = true;
        } else {
            result.record_live_posting(count)?;
        }
    }

    if removed > 0 {
        result.removed_heap_tids = result
            .removed_heap_tids
            .checked_add(
                u64::try_from(removed)
                    .map_err(|_| "ec_ivf removed heap tid count exceeds u64".to_owned())?,
            )
            .ok_or_else(|| "ec_ivf removed heap tid count overflow".to_owned())?;
    }

    if changed {
        Ok(page::IvfDensePostingBlockRewrite::Rewrite(posting_block))
    } else {
        Ok(page::IvfDensePostingBlockRewrite::Keep)
    }
}

/// # Safety
/// `index_relation` is the live IVF index relation being vacuumed and
/// `metadata` was read from it; `stats` is either non-null running
/// aggregator state or null on the cleanup-only path.
unsafe fn finish_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    metadata: &page::MetadataPage,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        crate::fault::maybe_fail_palloc("ec_ivf vacuum stats");
        alloc_index_bulk_delete_result().into()
    } else {
        stats
    };
    let index_relation = NonNull::new(index_relation)
        .unwrap_or_else(|| pgrx::error!("ec_ivf vacuum stats need a valid index relation"));
    let block_count = crate::storage::relation::main_fork_block_count_handle(index_relation);

    let stats_handle = NonNull::new(stats)
        .unwrap_or_else(|| pgrx::error!("ec_ivf vacuum stats unexpectedly null"));
    set_index_bulk_delete_summary(stats_handle, block_count, metadata.total_live_tuples);

    stats
}

fn heap_tid_is_dead(
    heap_tid: ItemPointer,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> bool {
    let mut tid = pg_sys::ItemPointerData::default();
    item_pointer_set_all(&mut tid, heap_tid.block_number, heap_tid.offset_number);
    // SAFETY: callback and callback_state come from PostgreSQL's bulkdelete
    // invocation and remain valid for this call; `tid` lives until callback
    // returns.
    unsafe { callback((&mut tid) as pg_sys::ItemPointer, callback_state) }
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Default)]
struct DebugVacuumCallbackState {
    dead_tids: std::collections::HashSet<ItemPointer>,
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) struct DebugEcIvfVacuumStats {
    pub(crate) estimated_count: bool,
    pub(crate) num_index_tuples: f64,
    pub(crate) tuples_removed: f64,
    pub(crate) num_pages: pg_sys::BlockNumber,
    pub(crate) pages_newly_deleted: pg_sys::BlockNumber,
    pub(crate) pages_deleted: pg_sys::BlockNumber,
    pub(crate) pages_free: pg_sys::BlockNumber,
}

#[cfg(any(test, feature = "pg_test"))]
unsafe extern "C-unwind" fn debug_vacuum_dead_tid_callback(
    itemptr: pg_sys::ItemPointer,
    state: *mut c_void,
) -> bool {
    pg_am_callback!({
        let state = &*(state.cast::<DebugVacuumCallbackState>());
        state
            .dead_tids
            .contains(&super::build::decode_heap_tid(itemptr, "debug vacuum"))
    })
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_ec_ivf_vacuum_stats_row(
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> DebugEcIvfVacuumStats {
    if stats.is_null() {
        pgrx::error!("ec_ivf debug vacuum stats returned NULL");
    }
    let stats = crate::am::common::vacuum::copy_index_bulk_delete_result(
        std::ptr::NonNull::new(stats).expect("ec_ivf debug vacuum stats should be non-null"),
    );
    DebugEcIvfVacuumStats {
        estimated_count: stats.estimated_count,
        num_index_tuples: stats.num_index_tuples,
        tuples_removed: stats.tuples_removed,
        num_pages: stats.num_pages,
        pages_newly_deleted: stats.pages_newly_deleted,
        pages_deleted: stats.pages_deleted,
        pages_free: stats.pages_free,
    }
}

/// # Safety
/// Debug helper runs the bulkdelete + vacuumcleanup callback pair on the
/// live index relation with a zeroed vacuum-info struct allocated in the
/// current PostgreSQL memory context; the stats pointer is reused into
/// cleanup and consumed by `debug_ec_ivf_vacuum_stats_row`.
#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_vacuum_stats(index_oid: pg_sys::Oid) -> DebugEcIvfVacuumStats {
    let index_relation = crate::storage::relation_guard::IndexRelationGuard::access_share(
        index_oid,
        "ec_ivf debug vacuum stats",
    );
    let mut info = crate::am::common::vacuum::alloc_index_vacuum_info();
    info.index = index_relation.as_ptr();
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;

    let stats = ec_ivf_ambulkdelete(info_ptr, std::ptr::null_mut(), None, std::ptr::null_mut());
    let stats = ec_ivf_amvacuumcleanup(info_ptr, stats);
    debug_ec_ivf_vacuum_stats_row(stats)
}

/// # Safety
/// Debug helper runs the bulkdelete + vacuumcleanup callback pair on a
/// share-update-exclusive-locked index with a zeroed vacuum-info struct in
/// the current PostgreSQL memory context; `callback_state` outlives the
/// callback invocations.
#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_vacuum_remove_heap_tids(
    index_oid: pg_sys::Oid,
    dead_tids: &[ItemPointer],
) -> DebugEcIvfVacuumStats {
    let index_relation = crate::storage::relation_guard::IndexRelationGuard::open(
        index_oid,
        pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
        "ec_ivf debug vacuum remove heap tids",
    );
    let mut info = crate::am::common::vacuum::alloc_index_vacuum_info();
    info.index = index_relation.as_ptr();
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;
    let mut callback_state = DebugVacuumCallbackState {
        dead_tids: dead_tids.iter().copied().collect(),
    };

    let stats = ec_ivf_ambulkdelete(
        info_ptr,
        std::ptr::null_mut(),
        Some(debug_vacuum_dead_tid_callback),
        (&mut callback_state as *mut DebugVacuumCallbackState).cast(),
    );
    let stats = ec_ivf_amvacuumcleanup(info_ptr, stats);
    debug_ec_ivf_vacuum_stats_row(stats)
}
