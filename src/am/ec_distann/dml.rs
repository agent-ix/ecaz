//! FR-083 DML — tombstone delete (Task 165 M3 early slice).
//!
//! Within a Published epoch the ONLY reclaim is a monotonic tombstone-flag set
//! (ADR-085 D10 / FR-082 immutability): `ambulkdelete` flips the FR-076
//! tombstone bit on every record whose co-placed heap row the vacuum callback
//! reports dead. Records, adjacency, and the co-placed heap rows are all
//! retained — a tombstoned record stays traversable (so a still-reachable
//! tombstone's `exact_dist` read never faults) but is excluded from results by
//! the FR-081 scan (`is_tombstone`). Physical reclaim + edge repair happen only
//! at the next epoch build (FR-083 physical-reclaim clause).
//!
//! This slice is the single-node / degenerate-FR-078 path: the record is local,
//! so the tombstone is an in-place flag flip. Multi-node routing of the write
//! to the hash-owning node (`ec_distann_apply_record_writes`) is a later M3
//! slice. It works for both identity modes — records store `heap_tid` and the
//! vacuum callback is keyed on the heap TID, so no vec_id recomputation is
//! needed at delete time.

use std::ffi::c_void;
use std::ptr::NonNull;

use pgrx::{pg_extern, pg_sys};

use crate::storage::buffer_guard::LockedBufferGuard;
use crate::storage::page::ItemPointer;
use crate::storage::relation::RelationHandle;
use crate::storage::relation_guard::IndexRelationGuard;
use crate::storage::wal;

use super::ambuild::read_metadata_from_index_handle;
use super::reader::{
    directory_lookup, read_directory_from_relation, read_raw_tuple_bytes_from_relation,
};
use super::tuple::{
    DISTANN_FLAG_TOMBSTONE, DISTANN_NODE_FLAGS_OFFSET, DISTANN_NODE_HEAP_TID_OFFSET,
    DISTANN_NODE_TAG, DISTANN_NODE_TAG_OFFSET,
};

/// Tombstone every record whose heap row is dead per the vacuum `callback`.
/// Returns the count newly tombstoned. A failed flag write errors: a lost
/// tombstone would silently resurrect a deleted row (FR-083 / NFR-020).
///
/// # Safety
/// `index_relation`, `callback`, and `callback_state` are the live pgrx
/// `ambulkdelete` arguments.
pub(super) unsafe fn tombstone_dead_records(
    index_relation: pg_sys::Relation,
    callback: pg_sys::IndexBulkDeleteCallback,
    callback_state: *mut c_void,
) -> Result<u64, String> {
    // No callback ⇒ nothing to delete (e.g. cleanup-only vacuum).
    let Some(callback) = callback else {
        return Ok(0);
    };
    let handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann tombstone delete needs a valid index relation".to_owned())?;
    let metadata = read_metadata_from_index_handle(handle)?;
    if metadata.node_count == 0 || metadata.directory_head == ItemPointer::INVALID {
        return Ok(0);
    }
    let directory = read_directory_from_relation(
        handle,
        metadata.directory_head,
        metadata.node_count as usize,
    )?;

    // Pass 1 (shared reads): collect records whose heap row is dead and that
    // are not already tombstoned (monotone: never re-flip).
    let mut to_tombstone: Vec<ItemPointer> = Vec::new();
    for (_vec_id, record_tid) in &directory {
        let raw = read_raw_tuple_bytes_from_relation(
            handle,
            *record_tid,
            "ec_distann tombstone scan",
        )?;
        if raw.first().copied() != Some(DISTANN_NODE_TAG)
            || raw.len() < DISTANN_NODE_HEAP_TID_OFFSET + 6
        {
            return Err(format!(
                "ec_distann tombstone: tuple ({},{}) is not a graph-node record",
                record_tid.block_number, record_tid.offset_number
            ));
        }
        let flags = u16::from_le_bytes(
            raw[DISTANN_NODE_FLAGS_OFFSET..DISTANN_NODE_FLAGS_OFFSET + 2]
                .try_into()
                .expect("flags bytes"),
        );
        if flags & DISTANN_FLAG_TOMBSTONE != 0 {
            continue;
        }
        let heap_tid = ItemPointer::decode(
            &raw[DISTANN_NODE_HEAP_TID_OFFSET..DISTANN_NODE_HEAP_TID_OFFSET + 6],
        )?;
        if callback_marks_dead(callback, callback_state, heap_tid) {
            to_tombstone.push(*record_tid);
        }
    }

    // Pass 2 (exclusive WAL writes): flip the flag in place.
    let mut removed = 0_u64;
    for record_tid in &to_tombstone {
        set_tombstone_flag(handle, *record_tid)?;
        removed += 1;
    }
    Ok(removed)
}

/// Debug/test surface for the D10 tombstone path: tombstone the given vec_ids
/// on `index_regclass` (the write-endpoint operation, in-transaction so pg_test
/// can exercise it — VACUUM/ambulkdelete cannot run in a txn). Returns the count
/// newly tombstoned.
#[pg_extern]
fn ec_distann_debug_tombstone(index_regclass: pg_sys::Oid, vec_ids: Vec<i64>) -> i64 {
    let ids: Vec<u64> = vec_ids.iter().map(|&v| v as u64).collect();
    let guard = IndexRelationGuard::open(
        index_regclass,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        "ec_distann_debug_tombstone",
    );
    // SAFETY: the guard holds the index open for write for the call.
    let removed = unsafe { tombstone_by_vec_ids(guard.as_ptr(), &ids) }
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    i64::try_from(removed).unwrap_or(i64::MAX)
}

fn callback_marks_dead(
    callback: unsafe extern "C-unwind" fn(*mut pg_sys::ItemPointerData, *mut c_void) -> bool,
    callback_state: *mut c_void,
    heap_tid: ItemPointer,
) -> bool {
    let mut raw_tid = pg_sys::ItemPointerData::default();
    // SAFETY: `callback`/`callback_state` are the live ambulkdelete args and
    // `raw_tid` outlives the call.
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut raw_tid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
        callback(&mut raw_tid, callback_state)
    }
}

/// Tombstone records by vec_id (the FR-083 write-endpoint "tombstone set"
/// primitive: the coordinator routes a delete to the hash-owning node, which
/// sets the flag on the record it owns). Idempotent per vec_id; a vec_id not in
/// the directory is a structural fault (a delete must not silently miss). This
/// is also the in-transaction-testable path (VACUUM/ambulkdelete cannot run in
/// a txn). Returns the count newly tombstoned.
///
/// # Safety
/// `index_relation` is a live index relation opened for write.
pub(super) unsafe fn tombstone_by_vec_ids(
    index_relation: pg_sys::Relation,
    vec_ids: &[u64],
) -> Result<u64, String> {
    if vec_ids.is_empty() {
        return Ok(0);
    }
    let handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann tombstone needs a valid index relation".to_owned())?;
    let metadata = read_metadata_from_index_handle(handle)?;
    if metadata.directory_head == ItemPointer::INVALID {
        return Err("ec_distann tombstone: index has no directory".to_owned());
    }
    let directory = read_directory_from_relation(
        handle,
        metadata.directory_head,
        metadata.node_count as usize,
    )?;

    let mut removed = 0_u64;
    for &vec_id in vec_ids {
        let record_tid = directory_lookup(&directory, vec_id).ok_or_else(|| {
            format!("ec_distann tombstone: vec_id {vec_id:#018x} is not in the directory")
        })?;
        // Skip if already tombstoned (monotone).
        let raw = read_raw_tuple_bytes_from_relation(
            handle,
            record_tid,
            "ec_distann tombstone by vec_id",
        )?;
        let flags = u16::from_le_bytes(
            raw[DISTANN_NODE_FLAGS_OFFSET..DISTANN_NODE_FLAGS_OFFSET + 2]
                .try_into()
                .expect("flags bytes"),
        );
        if flags & DISTANN_FLAG_TOMBSTONE != 0 {
            continue;
        }
        set_tombstone_flag(handle, record_tid)?;
        removed += 1;
    }
    Ok(removed)
}

/// Set the FR-076 tombstone flag bit on one record, WAL-logged. Same-length,
/// in-place: only the 2-byte flags field changes.
fn set_tombstone_flag(handle: RelationHandle, record_tid: ItemPointer) -> Result<(), String> {
    let buffer = LockedBufferGuard::read_main_handle(
        handle,
        record_tid.block_number,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    )
    .ok_or_else(|| {
        format!(
            "ec_distann tombstone: could not open block {}",
            record_tid.block_number
        )
    })?;

    let mut wal_txn = wal::WalTxnScope::start_handle(handle);
    {
        let mut page = wal_txn.register_page(&buffer);
        page.visit_tuple_bytes_mut(record_tid, "ec_distann tombstone write", |raw| {
            if raw.first().copied() != Some(DISTANN_NODE_TAG)
                || raw.len() < DISTANN_NODE_FLAGS_OFFSET + 2
            {
                return Err(format!(
                    "ec_distann tombstone write: tuple ({},{}) is not a graph-node record \
                     (tag at offset {DISTANN_NODE_TAG_OFFSET})",
                    record_tid.block_number, record_tid.offset_number
                ));
            }
            let mut flags = u16::from_le_bytes(
                raw[DISTANN_NODE_FLAGS_OFFSET..DISTANN_NODE_FLAGS_OFFSET + 2]
                    .try_into()
                    .expect("flags bytes"),
            );
            flags |= DISTANN_FLAG_TOMBSTONE;
            raw[DISTANN_NODE_FLAGS_OFFSET..DISTANN_NODE_FLAGS_OFFSET + 2]
                .copy_from_slice(&flags.to_le_bytes());
            Ok(())
        })?;
    }
    wal_txn.finish();
    Ok(())
}
