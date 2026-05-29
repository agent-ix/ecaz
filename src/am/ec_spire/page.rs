//! Relation-backed root/control page helpers for `ec_spire`.

use std::ptr::{self, NonNull};

use pgrx::pg_sys;

use super::meta::SpireRootControlState;
use crate::storage::{
    buffer_guard::{LockedBufferGuard, LockedPageTupleVisit},
    page::{
        align_up, raw_tuple_storage_bytes, usable_page_bytes, ItemPointer, ALIGNMENT_BYTES,
        FIRST_DATA_BLOCK_NUMBER, ITEM_POINTER_BYTES, METADATA_BLOCK_NUMBER,
    },
    relation::{main_fork_block_count_handle, RelationHandle},
    wal,
};

const P_NEW: pg_sys::BlockNumber = u32::MAX;
const MANIFEST_BLOB_CHAIN_MAGIC: u32 = 0x4342_5053; // "SPBC"
const MANIFEST_BLOB_CHAIN_VERSION: u16 = 1;
const MANIFEST_BLOB_CHAIN_META_FLAG: u16 = 1;
const MANIFEST_BLOB_CHAIN_SEGMENT_FLAG: u16 = 2;
const MANIFEST_BLOB_CHAIN_META_BYTES: usize = 4 + 2 + 2 + 8 + 4 + ITEM_POINTER_BYTES;
const MANIFEST_BLOB_CHAIN_SEGMENT_PREFIX_BYTES: usize = 4 + 2 + 2 + 4 + 8 + ITEM_POINTER_BYTES;

#[derive(Clone, Copy)]
struct SpirePageRelation {
    relation: RelationHandle,
}

impl SpirePageRelation {
    fn new(relation: pg_sys::Relation) -> Self {
        let relation = NonNull::new(relation)
            .unwrap_or_else(|| pgrx::error!("ec_spire page access needs a valid relation"));
        Self { relation }
    }

    fn from_handle(relation: RelationHandle) -> Self {
        Self { relation }
    }

    fn raw(self) -> pg_sys::Relation {
        self.relation.as_ptr()
    }

    fn number_of_blocks(self) -> pg_sys::BlockNumber {
        main_fork_block_count_handle(self.relation)
    }

    fn page_with_free_space(self, required_space: usize) -> pg_sys::BlockNumber {
        // SAFETY: this view is constructed only for an open SPIRE relation;
        // required_space is derived from the tuple that will be appended.
        unsafe { pg_sys::GetPageWithFreeSpace(self.raw(), required_space) }
    }

    fn read_main(
        self,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
        lockmode: i32,
    ) -> Option<LockedBufferGuard> {
        LockedBufferGuard::read_main_handle(self.relation, block_number, mode, lockmode)
    }

    fn read_main_locked(
        self,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
    ) -> Option<LockedBufferGuard> {
        LockedBufferGuard::read_main_locked_handle(self.relation, block_number, mode)
    }

    fn start_wal(self) -> wal::WalTxnScope<'static> {
        wal::WalTxnScope::start_handle(self.relation)
    }
}

struct SpireRegisteredPage {
    relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    page: pg_sys::Page,
}

impl SpireRegisteredPage {
    fn new(
        relation: pg_sys::Relation,
        block_number: pg_sys::BlockNumber,
        page: pg_sys::Page,
    ) -> Self {
        Self {
            relation,
            block_number,
            page,
        }
    }

    fn init_with_special(&self, page_size: usize, special_size: usize, bytes: &[u8]) {
        if bytes.len() > special_size {
            pgrx::error!(
                "ec_spire root/control special area too small: got {special_size}, need {}",
                bytes.len()
            );
        }
        // SAFETY: this wrapper is only constructed for a WAL-registered page
        // whose locked buffer is still live. The page is initialized with a
        // special area sized for `bytes`, and the source slice does not overlap
        // PostgreSQL page memory.
        unsafe {
            pg_sys::PageInit(self.page, page_size, special_size);
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                pg_sys::PageGetSpecialPointer(self.page).cast::<u8>(),
                bytes.len(),
            )
        };
    }

    fn init_empty(&self, page_size: usize) {
        // SAFETY: this wrapper is only constructed for a WAL-registered page
        // whose locked buffer is still live.
        unsafe { pg_sys::PageInit(self.page, page_size, 0) };
    }

    fn add_item_if_space(&self, payload: &[u8]) -> Result<pg_sys::OffsetNumber, usize> {
        // SAFETY: this wrapper owns the WAL-registered page while the buffer is
        // locked. The payload bytes are copied into the registered page by
        // PostgreSQL. When there is insufficient space, the recorded FSM entry
        // is for this same relation/block pair.
        unsafe {
            let free_space = pg_sys::PageGetFreeSpace(self.page) as usize;
            if free_space < raw_tuple_storage_bytes(payload.len()) {
                pg_sys::RecordPageWithFreeSpace(self.relation, self.block_number, free_space);
                return Err(free_space);
            }

            Ok(pg_sys::PageAddItemExtended(
                self.page,
                payload.as_ptr().cast_mut().cast(),
                payload.len(),
                pg_sys::InvalidOffsetNumber,
                0,
            ))
        }
    }

    fn record_current_free_space(&self) {
        // SAFETY: this wrapper owns the WAL-registered page while the buffer is
        // locked, and relation/block identify that same page in the FSM.
        unsafe {
            let free_space = pg_sys::PageGetFreeSpace(self.page) as usize;
            pg_sys::RecordPageWithFreeSpace(self.relation, self.block_number, free_space);
        };
    }

    fn delete_no_compact_checked(
        &self,
        offset: pg_sys::OffsetNumber,
    ) -> Result<(), pg_sys::OffsetNumber> {
        // SAFETY: this wrapper owns the WAL-registered page while the buffer is
        // locked. The max-offset read and no-compact delete apply to the same
        // page, so the range check cannot drift between helper calls.
        unsafe {
            let max_offset = pg_sys::PageGetMaxOffsetNumber(self.page);
            if offset == pg_sys::InvalidOffsetNumber || offset > max_offset {
                return Err(max_offset);
            }
            pg_sys::PageIndexTupleDeleteNoCompact(self.page, offset);
        };
        Ok(())
    }
}

/// # Safety
/// Caller guarantees `index_relation` is live for metadata rewrite.
pub(super) unsafe fn initialize_root_control_page(
    index_relation: pg_sys::Relation,
    root_control: SpireRootControlState,
) {
    initialize_spire_metadata_block_zero(index_relation, root_control);
}

/// # Safety
/// Caller guarantees `store_relation` is live for metadata rewrite.
pub(super) unsafe fn initialize_aux_store_metadata_page(store_relation: pg_sys::Relation) {
    initialize_spire_metadata_block_zero(store_relation, SpireRootControlState::empty());
}

/// # Safety
/// Caller passes a live SPIRE index or store relation that owns block 0
/// for the metadata rewrite.
unsafe fn initialize_spire_metadata_block_zero(
    index_relation: pg_sys::Relation,
    root_control: SpireRootControlState,
) {
    let relation = SpirePageRelation::new(index_relation);
    let existing_blocks = relation.number_of_blocks();
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        METADATA_BLOCK_NUMBER
    };
    let buffer = if target_block == P_NEW {
        relation.read_main_locked(target_block, pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK)
    } else {
        relation.read_main(
            target_block,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
    }
    .unwrap_or_else(|| pgrx::error!("ec_spire failed to allocate root/control buffer"));
    let page_size = buffer.page_size();
    let mut wal_txn = relation.start_wal();
    let page = wal_txn.register_page(&buffer).page_ptr();
    let registered = SpireRegisteredPage::new(relation.raw(), buffer.block_number(), page);
    let root_control_bytes = root_control
        .encode()
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let special_size = align_up(root_control_bytes.len(), ALIGNMENT_BYTES);
    registered.init_with_special(page_size, special_size, &root_control_bytes);

    wal_txn.finish();
}

/// Safe handle-based variant of [`read_root_control_page`] — the
/// `RelationHandle` carries the "live SPIRE index/store relation"
/// invariant that the unsafe entrypoint requires.
pub(super) fn read_root_control_page_handle(
    index_relation: RelationHandle,
) -> SpireRootControlState {
    // SAFETY: `RelationHandle` is a non-null pointer for a live relation.
    unsafe { read_root_control_page(index_relation.as_ptr()) }
}

/// # Safety
/// Caller passes a live SPIRE index or store relation that owns the
/// metadata block.
pub(super) unsafe fn read_root_control_page(
    index_relation: pg_sys::Relation,
) -> SpireRootControlState {
    let buffer = SpirePageRelation::new(index_relation)
        .read_main(
            METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .unwrap_or_else(|| pgrx::error!("ec_spire failed to open root/control buffer"));
    let page = buffer.page();
    // SAFETY: buffer is share-locked and pinned; special size is checked before
    // exposing the encoded root/control bytes from PostgreSQL page memory.
    let root_control_bytes = unsafe {
        let special_size = pg_sys::PageGetSpecialSize(page) as usize;
        if special_size < SpireRootControlState::encoded_len() {
            pgrx::error!(
                "ec_spire root/control special area too small: got {special_size}, expected at least {}",
                SpireRootControlState::encoded_len()
            );
        }
        let root_control_ptr = pg_sys::PageGetSpecialPointer(page).cast::<u8>();
        std::slice::from_raw_parts(root_control_ptr, SpireRootControlState::encoded_len())
    };
    let root_control =
        SpireRootControlState::decode(root_control_bytes).unwrap_or_else(|e| pgrx::error!("{e}"));
    root_control
}

/// # Safety
/// Caller passes a live SPIRE index relation whose root/control block
/// has been initialized.
pub(super) unsafe fn append_object_tuple(
    index_relation: pg_sys::Relation,
    payload: &[u8],
) -> Result<crate::storage::page::ItemPointer, String> {
    if payload.is_empty() {
        return Err("ec_spire object tuple payload must not be empty".to_owned());
    }

    let relation = SpirePageRelation::new(index_relation);
    let existing_blocks = relation.number_of_blocks();
    if existing_blocks < FIRST_DATA_BLOCK_NUMBER {
        return Err(
            "ec_spire root/control block must be initialized before object tuples".to_owned(),
        );
    }

    if existing_blocks > FIRST_DATA_BLOCK_NUMBER {
        let last_data_block = existing_blocks - 1;
        if let Some(tid) = try_append_object_tuple_to_block(relation, last_data_block, payload)? {
            return Ok(tid);
        }

        let required_space = raw_tuple_storage_bytes(payload.len());
        let fsm_block = relation.page_with_free_space(required_space);
        if fsm_block >= FIRST_DATA_BLOCK_NUMBER
            && fsm_block < existing_blocks
            && fsm_block != last_data_block
        {
            if let Some(tid) = try_append_object_tuple_to_block(relation, fsm_block, payload)? {
                return Ok(tid);
            }
        }
    }

    append_object_tuple_to_new_block(relation, payload)
}

/// Safe handle-based variant of [`read_object_tuple`].
pub(super) fn read_object_tuple_handle(
    index_relation: RelationHandle,
    tid: crate::storage::page::ItemPointer,
) -> Result<Vec<u8>, String> {
    // SAFETY: `RelationHandle` is a non-null pointer for a live relation.
    unsafe { read_object_tuple(index_relation.as_ptr(), tid) }
}

/// # Safety
/// Caller passes a live SPIRE index relation; the helper validates the
/// TID and keeps the page share-locked while copying tuple bytes into an
/// owned Vec.
pub(super) unsafe fn read_object_tuple(
    index_relation: pg_sys::Relation,
    tid: crate::storage::page::ItemPointer,
) -> Result<Vec<u8>, String> {
    with_pinned_object_tuple(index_relation, tid, |tuple| Ok(tuple.to_vec()))
}

/// # Safety
/// Caller passes a live SPIRE index relation. The payload is appended either
/// as one object tuple or as a manifest blob chain whose head locator is
/// returned.
pub(super) unsafe fn append_manifest_blob_tuple(
    index_relation: pg_sys::Relation,
    payload: &[u8],
) -> Result<ItemPointer, String> {
    if relation_object_tuple_fits(payload.len()) {
        return unsafe { append_object_tuple(index_relation, payload) };
    }

    let chunk_bytes = max_manifest_blob_chain_segment_payload_bytes()?;
    let segment_count = payload
        .len()
        .checked_add(chunk_bytes - 1)
        .and_then(|value| value.checked_div(chunk_bytes))
        .ok_or_else(|| "ec_spire manifest blob segment count overflow".to_owned())?;
    let segment_count_u32 = u32::try_from(segment_count)
        .map_err(|_| "ec_spire manifest blob segment count exceeds u32".to_owned())?;

    let mut next_segment_locator = ItemPointer::INVALID;
    for segment_index in (0..segment_count).rev() {
        let byte_base = segment_index
            .checked_mul(chunk_bytes)
            .ok_or_else(|| "ec_spire manifest blob byte_base overflow".to_owned())?;
        let byte_end = payload.len().min(byte_base + chunk_bytes);
        let segment = encode_manifest_blob_chain_segment(
            u32::try_from(segment_index)
                .map_err(|_| "ec_spire manifest blob segment index exceeds u32".to_owned())?,
            u64::try_from(byte_base)
                .map_err(|_| "ec_spire manifest blob byte_base exceeds u64".to_owned())?,
            next_segment_locator,
            &payload[byte_base..byte_end],
        )?;
        next_segment_locator = unsafe { append_object_tuple(index_relation, &segment) }?;
    }

    let meta = encode_manifest_blob_chain_meta(
        u64::try_from(payload.len())
            .map_err(|_| "ec_spire manifest blob length exceeds u64".to_owned())?,
        segment_count_u32,
        next_segment_locator,
    )?;
    unsafe { append_object_tuple(index_relation, &meta) }
}

pub(super) unsafe fn read_manifest_blob_tuple(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
) -> Result<Vec<u8>, String> {
    let first = unsafe { read_object_tuple(index_relation, tid) }?;
    let Some(meta) = decode_manifest_blob_chain_meta(&first)? else {
        return Ok(first);
    };
    read_manifest_blob_chain(index_relation, &meta)
}

pub(super) unsafe fn manifest_blob_tuple_locators(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
) -> Result<Vec<ItemPointer>, String> {
    if tid == ItemPointer::INVALID {
        return Ok(Vec::new());
    }
    let first = unsafe { read_object_tuple(index_relation, tid) }?;
    let Some(meta) = decode_manifest_blob_chain_meta(&first)? else {
        return Ok(vec![tid]);
    };
    let mut locators = vec![tid];
    let mut next_locator = meta.first_segment_locator;
    for expected_segment_no in 0..meta.segment_count {
        if next_locator == ItemPointer::INVALID {
            return Err("ec_spire manifest blob segment chain ended early".to_owned());
        }
        locators.push(next_locator);
        let segment_raw = unsafe { read_object_tuple(index_relation, next_locator) }?;
        let segment = decode_manifest_blob_chain_segment(&segment_raw, expected_segment_no)?;
        next_locator = segment.next_segment_locator;
    }
    if next_locator != ItemPointer::INVALID {
        return Err("ec_spire manifest blob segment chain has trailing locator".to_owned());
    }
    Ok(locators)
}

/// # Safety
/// Caller passes a live SPIRE index relation; the helper validates the
/// TID, share-locks the page, and exposes tuple bytes only for the
/// callback's duration.
pub(super) unsafe fn with_pinned_object_tuple<F, R>(
    index_relation: pg_sys::Relation,
    tid: crate::storage::page::ItemPointer,
    f: F,
) -> Result<R, String>
where
    F: FnOnce(&[u8]) -> Result<R, String>,
{
    if tid.block_number < FIRST_DATA_BLOCK_NUMBER {
        return Err(format!(
            "ec_spire object tuple cannot use metadata block {}",
            tid.block_number
        ));
    }

    let buffer = SpirePageRelation::new(index_relation)
        .read_main(
            tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| format!("ec_spire failed to open object block {}", tid.block_number))?;
    match buffer.visit_tuple_bytes(tid, "ec_spire object", f)? {
        LockedPageTupleVisit::Unused => Err(format!(
            "ec_spire object tuple ({},{}) points at an unused slot",
            tid.block_number, tid.offset_number
        )),
        LockedPageTupleVisit::Present(result) => Ok(result),
    }
}

/// Safe handle-based variant of [`scan_object_tuples`].
pub(super) fn scan_object_tuples_handle<F>(
    index_relation: RelationHandle,
    visit: F,
) -> Result<(), String>
where
    F: FnMut(crate::storage::page::ItemPointer, &[u8]) -> Result<(), String>,
{
    // SAFETY: `RelationHandle` is a non-null pointer for a live relation.
    unsafe { scan_object_tuples(index_relation.as_ptr(), visit) }
}

/// # Safety
/// Caller passes a live SPIRE object relation; the visitor runs while
/// each page is held under BUFFER_LOCK_SHARE and must not re-enter the
/// relation. Tuple bytes are valid only for the callback's duration.
pub(super) unsafe fn scan_object_tuples<F>(
    index_relation: pg_sys::Relation,
    mut visit: F,
) -> Result<(), String>
where
    F: FnMut(crate::storage::page::ItemPointer, &[u8]) -> Result<(), String>,
{
    // The visitor runs while the current object page is held under
    // BUFFER_LOCK_SHARE. Keep visitors limited to CPU-only tuple inspection
    // and copying bytes into caller-owned state; do not read or pin other pages
    // in this relation from inside the callback.
    let relation = SpirePageRelation::new(index_relation);
    let block_count = relation.number_of_blocks();
    for block_number in FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = relation
            .read_main(
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
            .ok_or_else(|| format!("ec_spire failed to open object block {block_number}"))?;
        let mut result: Result<(), String> = Ok(());
        let max_offset = buffer.max_offset_number();
        for offset_number in 1..=max_offset {
            let tid = crate::storage::page::ItemPointer {
                block_number,
                offset_number,
            };
            result = buffer
                .visit_tuple_bytes(tid, "ec_spire object", |tuple| visit(tid, tuple))
                .map(|_| ());
            if result.is_err() {
                break;
            }
        }
        result?;
    }
    Ok(())
}

/// # Safety
/// Caller passes a live SPIRE index relation; `tid` and `payload` must
/// match an existing object tuple's slot length on the indicated page.
pub(super) unsafe fn rewrite_object_tuple_same_len(
    index_relation: pg_sys::Relation,
    tid: crate::storage::page::ItemPointer,
    payload: &[u8],
) -> Result<(), String> {
    let relation = SpirePageRelation::new(index_relation);
    let buffer = relation
        .read_main(
            tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
        .ok_or_else(|| format!("ec_spire failed to open object block {}", tid.block_number))?;
    let mut wal_txn = relation.start_wal();
    let mut page = wal_txn.register_page(&buffer);
    let result = page.visit_tuple_bytes_mut(tid, "ec_spire object", |tuple| {
        if tuple.len() != payload.len() {
            return Err(format!(
                "ec_spire object tuple rewrite length changed from {} to {}",
                tuple.len(),
                payload.len()
            ));
        }

        tuple.copy_from_slice(payload);
        Ok(())
    });
    match result {
        Ok(()) => {
            wal_txn.finish();
            Ok(())
        }
        Err(error) => {
            std::mem::drop(wal_txn);
            Err(error)
        }
    }
}

/// # Safety
/// Caller passes a live SPIRE index relation; `tids` reference object
/// tuples on existing data blocks.
pub(super) unsafe fn delete_object_tuples_no_compact(
    index_relation: pg_sys::Relation,
    tids: &[crate::storage::page::ItemPointer],
) -> Result<(u64, u64), String> {
    let relation = SpirePageRelation::new(index_relation);
    let mut offsets_by_block = std::collections::BTreeMap::<pg_sys::BlockNumber, Vec<u16>>::new();
    for tid in tids {
        if tid.block_number < FIRST_DATA_BLOCK_NUMBER {
            return Err(format!(
                "ec_spire object tuple delete cannot remove metadata block {}",
                tid.block_number
            ));
        }
        offsets_by_block
            .entry(tid.block_number)
            .or_default()
            .push(tid.offset_number);
    }

    let mut removed_tuple_count = 0_u64;
    let mut removed_tuple_bytes = 0_u64;
    for (block_number, mut offsets) in offsets_by_block {
        offsets.sort_unstable();
        offsets.dedup();
        let buffer = relation
            .read_main(
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
            )
            .ok_or_else(|| format!("ec_spire failed to open object block {block_number}"))?;
        let mut wal_txn = relation.start_wal();
        let page = wal_txn.register_page(&buffer).page_ptr();
        let registered = SpireRegisteredPage::new(relation.raw(), block_number, page);
        let mut changed = false;
        for offset in offsets.into_iter().rev() {
            let tid = crate::storage::page::ItemPointer {
                block_number,
                offset_number: offset,
            };
            let tuple_len =
                match buffer
                    .visit_tuple_bytes(tid, "ec_spire object delete", |tuple| Ok(tuple.len()))?
                {
                    LockedPageTupleVisit::Unused => continue,
                    LockedPageTupleVisit::Present(tuple_len) => tuple_len,
                };
            if registered.delete_no_compact_checked(offset).is_err() {
                std::mem::drop(wal_txn);
                return Err(format!(
                    "ec_spire object tuple delete offset {} out of range on block {}",
                    offset, block_number
                ));
            }
            removed_tuple_count = removed_tuple_count
                .checked_add(1)
                .ok_or_else(|| "ec_spire removed tuple count overflow".to_owned())?;
            removed_tuple_bytes = removed_tuple_bytes
                .checked_add(
                    u64::try_from(tuple_len)
                        .map_err(|_| "ec_spire removed tuple bytes exceed u64".to_owned())?,
                )
                .ok_or_else(|| "ec_spire removed tuple bytes overflow".to_owned())?;
            changed = true;
        }
        if changed {
            wal_txn.finish();
        }
        registered.record_current_free_space();
    }
    Ok((removed_tuple_count, removed_tuple_bytes))
}

fn try_append_object_tuple_to_block(
    relation: SpirePageRelation,
    block_number: pg_sys::BlockNumber,
    payload: &[u8],
) -> Result<Option<crate::storage::page::ItemPointer>, String> {
    let buffer = relation
        .read_main(
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
        .ok_or_else(|| format!("ec_spire failed to open object block {block_number}"))?;
    let mut wal_txn = relation.start_wal();
    let page = wal_txn.register_page(&buffer).page_ptr();
    let registered = SpireRegisteredPage::new(relation.raw(), block_number, page);
    let page_size = buffer.page_size();
    if raw_tuple_storage_bytes(payload.len()) > page_size {
        std::mem::drop(wal_txn);
        return Err(format!(
            "ec_spire object tuple payload {} exceeds page size {page_size}",
            payload.len()
        ));
    }

    let offset = match registered.add_item_if_space(payload) {
        Ok(offset) => offset,
        Err(_) => {
            std::mem::drop(wal_txn);
            return Ok(None);
        }
    };
    if offset == pg_sys::InvalidOffsetNumber {
        std::mem::drop(wal_txn);
        return Err(format!(
            "ec_spire failed to append object tuple to block {block_number}"
        ));
    }

    wal_txn.finish();
    registered.record_current_free_space();
    Ok(Some(crate::storage::page::ItemPointer {
        block_number,
        offset_number: offset,
    }))
}

fn append_object_tuple_to_new_block(
    relation: SpirePageRelation,
    payload: &[u8],
) -> Result<crate::storage::page::ItemPointer, String> {
    let existing_blocks = relation.number_of_blocks();
    if existing_blocks < FIRST_DATA_BLOCK_NUMBER {
        return Err(
            "ec_spire root/control block must be initialized before object tuples".to_owned(),
        );
    }

    let buffer = relation
        .read_main_locked(P_NEW, pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK)
        .ok_or_else(|| "ec_spire failed to allocate object block".to_owned())?;
    let page_size = buffer.page_size();
    let mut wal_txn = relation.start_wal();
    let page = wal_txn.register_page(&buffer).page_ptr();
    let registered = SpireRegisteredPage::new(relation.raw(), buffer.block_number(), page);
    registered.init_empty(page_size);

    let offset = match registered.add_item_if_space(payload) {
        Ok(offset) => offset,
        Err(_) => {
            std::mem::drop(wal_txn);
            return Err(format!(
                "ec_spire object tuple payload {} exceeds page capacity",
                payload.len()
            ));
        }
    };
    if offset == pg_sys::InvalidOffsetNumber {
        std::mem::drop(wal_txn);
        return Err("ec_spire failed to append object tuple to new block".to_owned());
    }
    let block_number = buffer.block_number();

    wal_txn.finish();
    registered.record_current_free_space();
    Ok(crate::storage::page::ItemPointer {
        block_number,
        offset_number: offset,
    })
}

#[derive(Debug, Clone)]
struct ManifestBlobChainMeta {
    object_bytes_total: u64,
    segment_count: u32,
    first_segment_locator: ItemPointer,
}

#[derive(Debug, Clone)]
struct ManifestBlobChainSegment {
    byte_base: u64,
    next_segment_locator: ItemPointer,
    payload: Vec<u8>,
}

fn relation_object_tuple_fits(payload_len: usize) -> bool {
    raw_tuple_storage_bytes(payload_len) <= usable_page_bytes(pg_sys::BLCKSZ as usize)
}

fn max_manifest_blob_chain_segment_payload_bytes() -> Result<usize, String> {
    let max_tuple_payload = max_relation_object_tuple_payload_bytes(pg_sys::BLCKSZ as usize)?;
    max_tuple_payload
        .checked_sub(MANIFEST_BLOB_CHAIN_SEGMENT_PREFIX_BYTES)
        .filter(|value| *value > 0)
        .ok_or_else(|| "ec_spire manifest blob segment payload capacity is zero".to_owned())
}

fn max_relation_object_tuple_payload_bytes(page_size: usize) -> Result<usize, String> {
    let usable = usable_page_bytes(page_size).min(7_000);
    for payload_len in (1..=usable).rev() {
        if raw_tuple_storage_bytes(payload_len) <= usable {
            return Ok(payload_len);
        }
    }
    Err("ec_spire relation object tuple capacity is zero".to_owned())
}

fn encode_manifest_blob_chain_meta(
    object_bytes_total: u64,
    segment_count: u32,
    first_segment_locator: ItemPointer,
) -> Result<Vec<u8>, String> {
    if object_bytes_total == 0 {
        return Err("ec_spire manifest blob length 0 is invalid".to_owned());
    }
    if segment_count == 0 {
        return Err("ec_spire manifest blob chain requires at least one segment".to_owned());
    }
    if first_segment_locator == ItemPointer::INVALID {
        return Err("ec_spire manifest blob chain requires first segment".to_owned());
    }
    let mut out = Vec::with_capacity(MANIFEST_BLOB_CHAIN_META_BYTES);
    out.extend_from_slice(&MANIFEST_BLOB_CHAIN_MAGIC.to_le_bytes());
    out.extend_from_slice(&MANIFEST_BLOB_CHAIN_VERSION.to_le_bytes());
    out.extend_from_slice(&MANIFEST_BLOB_CHAIN_META_FLAG.to_le_bytes());
    out.extend_from_slice(&object_bytes_total.to_le_bytes());
    out.extend_from_slice(&segment_count.to_le_bytes());
    first_segment_locator.encode_into(&mut out);
    debug_assert_eq!(out.len(), MANIFEST_BLOB_CHAIN_META_BYTES);
    Ok(out)
}

fn decode_manifest_blob_chain_meta(input: &[u8]) -> Result<Option<ManifestBlobChainMeta>, String> {
    if input.len() < 8 {
        return Ok(None);
    }
    let magic = u32::from_le_bytes(input[0..4].try_into().expect("blob magic"));
    if magic != MANIFEST_BLOB_CHAIN_MAGIC {
        return Ok(None);
    }
    if input.len() != MANIFEST_BLOB_CHAIN_META_BYTES {
        return Err(format!(
            "ec_spire manifest blob chain meta length mismatch: got {}, expected {MANIFEST_BLOB_CHAIN_META_BYTES}",
            input.len()
        ));
    }
    let version = u16::from_le_bytes(input[4..6].try_into().expect("blob version"));
    if version != MANIFEST_BLOB_CHAIN_VERSION {
        return Err(format!(
            "ec_spire manifest blob chain unsupported version: {version}"
        ));
    }
    let flags = u16::from_le_bytes(input[6..8].try_into().expect("blob flags"));
    if flags != MANIFEST_BLOB_CHAIN_META_FLAG {
        return Err(format!(
            "ec_spire manifest blob chain expected meta flag, got {flags}"
        ));
    }
    let object_bytes_total =
        u64::from_le_bytes(input[8..16].try_into().expect("blob total length"));
    let segment_count = u32::from_le_bytes(input[16..20].try_into().expect("blob segments"));
    let first_segment_locator = ItemPointer::decode(&input[20..26])?;
    if object_bytes_total == 0 {
        return Err("ec_spire manifest blob chain total length 0 is invalid".to_owned());
    }
    if segment_count == 0 {
        return Err("ec_spire manifest blob chain segment_count 0 is invalid".to_owned());
    }
    if first_segment_locator == ItemPointer::INVALID {
        return Err("ec_spire manifest blob chain first segment is invalid".to_owned());
    }
    Ok(Some(ManifestBlobChainMeta {
        object_bytes_total,
        segment_count,
        first_segment_locator,
    }))
}

fn encode_manifest_blob_chain_segment(
    segment_no: u32,
    byte_base: u64,
    next_segment_locator: ItemPointer,
    payload: &[u8],
) -> Result<Vec<u8>, String> {
    if payload.is_empty() {
        return Err("ec_spire manifest blob segment payload must not be empty".to_owned());
    }
    let mut out = Vec::with_capacity(MANIFEST_BLOB_CHAIN_SEGMENT_PREFIX_BYTES + payload.len());
    out.extend_from_slice(&MANIFEST_BLOB_CHAIN_MAGIC.to_le_bytes());
    out.extend_from_slice(&MANIFEST_BLOB_CHAIN_VERSION.to_le_bytes());
    out.extend_from_slice(&MANIFEST_BLOB_CHAIN_SEGMENT_FLAG.to_le_bytes());
    out.extend_from_slice(&segment_no.to_le_bytes());
    out.extend_from_slice(&byte_base.to_le_bytes());
    next_segment_locator.encode_into(&mut out);
    out.extend_from_slice(payload);
    Ok(out)
}

fn decode_manifest_blob_chain_segment(
    input: &[u8],
    expected_segment_no: u32,
) -> Result<ManifestBlobChainSegment, String> {
    if input.len() <= MANIFEST_BLOB_CHAIN_SEGMENT_PREFIX_BYTES {
        return Err(format!(
            "ec_spire manifest blob segment too short: got {}, expected more than {MANIFEST_BLOB_CHAIN_SEGMENT_PREFIX_BYTES}",
            input.len()
        ));
    }
    let magic = u32::from_le_bytes(input[0..4].try_into().expect("blob magic"));
    if magic != MANIFEST_BLOB_CHAIN_MAGIC {
        return Err(format!(
            "ec_spire manifest blob segment invalid magic: {magic:#x}"
        ));
    }
    let version = u16::from_le_bytes(input[4..6].try_into().expect("blob version"));
    if version != MANIFEST_BLOB_CHAIN_VERSION {
        return Err(format!(
            "ec_spire manifest blob segment unsupported version: {version}"
        ));
    }
    let flags = u16::from_le_bytes(input[6..8].try_into().expect("blob flags"));
    if flags != MANIFEST_BLOB_CHAIN_SEGMENT_FLAG {
        return Err(format!(
            "ec_spire manifest blob segment expected segment flag, got {flags}"
        ));
    }
    let segment_no = u32::from_le_bytes(input[8..12].try_into().expect("blob segment no"));
    if segment_no != expected_segment_no {
        return Err(format!(
            "ec_spire manifest blob segment number mismatch: got {segment_no}, expected {expected_segment_no}"
        ));
    }
    let byte_base = u64::from_le_bytes(input[12..20].try_into().expect("blob byte base"));
    let next_segment_locator = ItemPointer::decode(&input[20..26])?;
    Ok(ManifestBlobChainSegment {
        byte_base,
        next_segment_locator,
        payload: input[MANIFEST_BLOB_CHAIN_SEGMENT_PREFIX_BYTES..].to_vec(),
    })
}

fn read_manifest_blob_chain(
    index_relation: pg_sys::Relation,
    meta: &ManifestBlobChainMeta,
) -> Result<Vec<u8>, String> {
    let expected_len = usize::try_from(meta.object_bytes_total)
        .map_err(|_| "ec_spire manifest blob length exceeds usize".to_owned())?;
    let mut out = Vec::with_capacity(expected_len);
    let mut next_locator = meta.first_segment_locator;
    for expected_segment_no in 0..meta.segment_count {
        if next_locator == ItemPointer::INVALID {
            return Err("ec_spire manifest blob segment chain ended early".to_owned());
        }
        let segment_raw = unsafe { read_object_tuple(index_relation, next_locator) }?;
        let segment = decode_manifest_blob_chain_segment(&segment_raw, expected_segment_no)?;
        let expected_base = u64::try_from(out.len())
            .map_err(|_| "ec_spire manifest blob byte_base exceeds u64".to_owned())?;
        if segment.byte_base != expected_base {
            return Err(format!(
                "ec_spire manifest blob byte_base mismatch: got {}, expected {expected_base}",
                segment.byte_base
            ));
        }
        out.extend_from_slice(&segment.payload);
        next_locator = segment.next_segment_locator;
    }
    if next_locator != ItemPointer::INVALID {
        return Err("ec_spire manifest blob segment chain has trailing locator".to_owned());
    }
    if out.len() != expected_len {
        return Err(format!(
            "ec_spire manifest blob length mismatch: got {}, expected {expected_len}",
            out.len()
        ));
    }
    Ok(out)
}
