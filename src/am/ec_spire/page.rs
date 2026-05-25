//! Relation-backed root/control page helpers for `ec_spire`.

use std::ptr::{self, NonNull};

use pgrx::pg_sys;

use super::meta::SpireRootControlState;
use crate::storage::{
    buffer_guard::{LockedBufferGuard, LockedPageTupleVisit},
    page::{
        align_up, raw_tuple_storage_bytes, ALIGNMENT_BYTES, FIRST_DATA_BLOCK_NUMBER,
        METADATA_BLOCK_NUMBER,
    },
    relation::{main_fork_block_count_handle, RelationHandle},
    wal,
};

const P_NEW: pg_sys::BlockNumber = u32::MAX;

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

    fn start_wal(self) -> wal::GenericXLogTxn {
        wal::GenericXLogTxn::start_handle(self.relation)
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
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let registered = SpireRegisteredPage::new(relation.raw(), buffer.block_number(), page);
    let root_control_bytes = root_control
        .encode()
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let special_size = align_up(root_control_bytes.len(), ALIGNMENT_BYTES);
    registered.init_with_special(page_size, special_size, &root_control_bytes);

    wal_txn.finish();
}

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
    let mut page = wal_txn.register_locked_buffer_full_image_page(&buffer);
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
        let page = wal_txn.register_locked_buffer_full_image(&buffer);
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
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
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
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
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
