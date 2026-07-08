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

use crate::am::ec_diskann::{decode_heap_tid, ecvector_datum_to_vec};

use super::ambuild::{overwrite_metadata_page_handle, read_metadata_from_index_handle};
use super::expand_error::DistannExpandError;
use super::identity::vec_id_from_local_heap_tid;
use super::options::{self, DistannSourceIdentityProvider};
use super::reader::{
    directory_lookup, read_directory_from_relation, read_raw_tuple_bytes_from_relation,
};
use super::tuple::{
    DistannDeltaTuple, DISTANN_FLAG_TOMBSTONE, DISTANN_NODE_FLAGS_OFFSET,
    DISTANN_NODE_HEAP_TID_OFFSET, DISTANN_NODE_TAG, DISTANN_NODE_TAG_OFFSET,
};

/// FR-083 / ADR-085 D5: the interim delta buffer is bounded; when full the
/// operator drains it with a REINDEX (the interim posture is explicitly not a
/// terminal state — incremental insert replaces it in M5).
const DISTANN_DELTA_BUFFER_CAP: usize = 4096;

/// Constant mirroring `ambuild::P_NEW` (extend the relation with a fresh page).
const P_NEW: pg_sys::BlockNumber = u32::MAX;

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
) -> Result<u64, DistannExpandError> {
    // Reviewer 003-P3: distinguish a genuinely owned-but-absent record
    // (EC_RECORD_MISSING) from broader structural/storage faults (Internal), so
    // the write endpoint's error class matches the TC-042/NFR-020 contract
    // instead of labeling every failure OwnedRecordMissing.
    if vec_ids.is_empty() {
        return Ok(0);
    }
    let handle = NonNull::new(index_relation).ok_or_else(|| {
        DistannExpandError::Internal("ec_distann tombstone needs a valid index relation".to_owned())
    })?;
    let metadata =
        read_metadata_from_index_handle(handle).map_err(DistannExpandError::Internal)?;
    if metadata.directory_head == ItemPointer::INVALID {
        return Err(DistannExpandError::Internal(
            "ec_distann tombstone: index has no directory".to_owned(),
        ));
    }
    let directory =
        read_directory_from_relation(handle, metadata.directory_head, metadata.node_count as usize)
            .map_err(DistannExpandError::Internal)?;

    let mut removed = 0_u64;
    for &vec_id in vec_ids {
        // Owned-but-absent is the EC_RECORD_MISSING contract case; the reads and
        // flag write below are internal/storage faults if they fail.
        let record_tid = directory_lookup(&directory, vec_id).ok_or_else(|| {
            DistannExpandError::OwnedRecordMissing(format!(
                "ec_distann tombstone: vec_id {vec_id:#018x} is not in the directory"
            ))
        })?;
        let raw =
            read_raw_tuple_bytes_from_relation(handle, record_tid, "ec_distann tombstone by vec_id")
                .map_err(DistannExpandError::Internal)?;
        let flags = u16::from_le_bytes(
            raw[DISTANN_NODE_FLAGS_OFFSET..DISTANN_NODE_FLAGS_OFFSET + 2]
                .try_into()
                .expect("flags bytes"),
        );
        if flags & DISTANN_FLAG_TOMBSTONE != 0 {
            continue;
        }
        set_tombstone_flag(handle, record_tid).map_err(DistannExpandError::Internal)?;
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

/// FR-083 / D5 interim delta-buffer insert: spool the new vector into a bounded
/// exact-scan buffer (prepended to the chain from `metadata.delta_buffer_head`)
/// with same-statement visibility; the buffer is drained by the next epoch
/// build. Local identity mode only for this slice — `source_identity='include'`
/// delta insert routes through the write endpoint in a later M3 slice.
///
/// # Safety
/// `values`/`isnull`/`heap_tid` are the live `aminsert` callback args and
/// `index_relation` is the live index.
pub(super) unsafe fn delta_insert(
    index_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: pg_sys::ItemPointer,
) -> Result<(), String> {
    let handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann delta insert needs a valid index relation".to_owned())?;
    if values.is_null() || isnull.is_null() {
        return Err("ec_distann delta insert received null datum arrays".to_owned());
    }
    if *isnull {
        return Err("ec_distann does not support NULL indexed values".to_owned());
    }
    let datum = *values;
    if datum.is_null() {
        return Err("ec_distann delta insert received a null indexed datum".to_owned());
    }

    let opts = options::relation_options(index_relation);
    if matches!(opts.source_identity, DistannSourceIdentityProvider::Include) {
        return Err(
            "ec_distann delta insert for source_identity='include' lands in a later M3 slice \
             (multi-node write endpoint)"
                .to_owned(),
        );
    }

    let vector = ecvector_datum_to_vec(datum);
    let heap_tid = decode_heap_tid(heap_tid);
    let vec_id = vec_id_from_local_heap_tid(heap_tid);

    let mut metadata = read_metadata_from_index_handle(handle)?;
    if metadata.node_count == 0 || metadata.dimensions == 0 {
        // The FR-081 scan early-returns for an empty graph, so a delta-only
        // buffer would not be visible. Reject rather than silently drop; a
        // delta-only scan path (index built on an empty table, then bulk
        // inserted) is a later M3 slice.
        return Err(
            "ec_distann delta insert into an empty index is not supported yet: build the index \
             on non-empty data first (delta-only scan lands in a later slice)"
                .to_owned(),
        );
    }
    if vector.len() != usize::from(metadata.dimensions) {
        return Err(format!(
            "ec_distann delta insert dimension mismatch: index dim {}, inserted dim {}",
            metadata.dimensions,
            vector.len()
        ));
    }

    // Bounded (D5): reviewer 002-P2 — the cap check reads the metadata count,
    // not the whole buffer, so an insert near the cap is O(1) instead of
    // deserializing tens of MB of buffered vectors. A lost insert must never be
    // silent. Duplicate-vec_id idempotence is not needed here: a delta entry is
    // a new heap row with its own TID-derived vec_id, distinct from every
    // buffered and built vec_id by construction (see the scan-merge invariant in
    // routine.rs), and the scan merge dedupes defensively regardless.
    if metadata.delta_count as usize >= DISTANN_DELTA_BUFFER_CAP {
        return Err(format!(
            "ec_distann delta buffer is full ({} entries, cap {DISTANN_DELTA_BUFFER_CAP}); \
             REINDEX (or fold) to drain the interim buffer (ADR-085 D5)",
            metadata.delta_count
        ));
    }

    let tuple = DistannDeltaTuple {
        next_tid: metadata.delta_buffer_head,
        vec_id,
        heap_tid,
        vector,
    };
    let new_head = append_delta_tuple(handle, tuple.encode())?;
    metadata.delta_buffer_head = new_head;
    metadata.delta_count += 1;
    overwrite_metadata_page_handle(handle, &metadata);
    Ok(())
}

/// Append one delta tuple on a freshly-extended page (one entry per page: a
/// dim-1536 entry is ~6 KB, near a page anyway, and the buffer is bounded).
/// WAL-logged. Returns the new tuple's TID.
fn append_delta_tuple(
    handle: RelationHandle,
    payload: Vec<u8>,
) -> Result<ItemPointer, String> {
    let buffer = LockedBufferGuard::read_main_locked_handle(
        handle,
        P_NEW,
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
    )
    .ok_or_else(|| "ec_distann delta insert could not extend the relation".to_owned())?;
    let block_number = buffer.block_number();

    let mut wal_txn = wal::WalTxnScope::start_handle(handle);
    let offset_number = {
        let mut page = wal_txn.register_page(&buffer);
        page.init(0);
        page.add_item(&payload)
            .map_err(|error| format!("ec_distann delta insert page add failed: {error:?}"))?
    };
    wal_txn.finish();

    Ok(ItemPointer {
        block_number,
        offset_number,
    })
}
