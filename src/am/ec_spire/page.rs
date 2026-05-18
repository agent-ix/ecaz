//! Relation-backed root/control page helpers for `ec_spire`.

use std::ptr;

use pgrx::pg_sys;

use super::meta::SpireRootControlState;
use crate::storage::{
    buffer_guard::LockedBufferGuard,
    page::{
        align_up, raw_tuple_storage_bytes, ALIGNMENT_BYTES, FIRST_DATA_BLOCK_NUMBER,
        METADATA_BLOCK_NUMBER,
    },
    wal,
};

const P_NEW: pg_sys::BlockNumber = u32::MAX;

enum SpireObjectTupleVisit<R> {
    Unused,
    Present(R),
}

pub(super) unsafe fn initialize_root_control_page(
    index_relation: pg_sys::Relation,
    root_control: SpireRootControlState,
) {
    unsafe { initialize_spire_metadata_block_zero(index_relation, root_control) };
}

pub(super) unsafe fn initialize_aux_store_metadata_page(store_relation: pg_sys::Relation) {
    unsafe { initialize_spire_metadata_block_zero(store_relation, SpireRootControlState::empty()) };
}

unsafe fn initialize_spire_metadata_block_zero(
    index_relation: pg_sys::Relation,
    root_control: SpireRootControlState,
) {
    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        METADATA_BLOCK_NUMBER
    };
    let buffer = if target_block == P_NEW {
        LockedBufferGuard::read_main_locked(
            index_relation,
            target_block,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
        )
    } else {
        LockedBufferGuard::read_main(
            index_relation,
            target_block,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
    }
    .unwrap_or_else(|| pgrx::error!("ec_spire failed to allocate root/control buffer"));
    let page_size = buffer.page_size();
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page =
        unsafe { wal_txn.register_buffer(buffer.buffer(), pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let root_control_bytes = root_control
        .encode()
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    let special_size = align_up(root_control_bytes.len(), ALIGNMENT_BYTES);
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    let page_contents = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    unsafe {
        ptr::copy_nonoverlapping(
            root_control_bytes.as_ptr(),
            page_contents,
            root_control_bytes.len(),
        );
    }

    unsafe { wal_txn.finish() };
}

pub(super) unsafe fn read_root_control_page(
    index_relation: pg_sys::Relation,
) -> SpireRootControlState {
    let buffer = LockedBufferGuard::read_main(
        index_relation,
        METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    )
    .unwrap_or_else(|| pgrx::error!("ec_spire failed to open root/control buffer"));
    let page = buffer.page();
    let special_size = unsafe { pg_sys::PageGetSpecialSize(page) as usize };
    if special_size < SpireRootControlState::encoded_len() {
        pgrx::error!(
            "ec_spire root/control special area too small: got {special_size}, expected at least {}",
            SpireRootControlState::encoded_len()
        );
    }
    let root_control_ptr = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    let root_control_bytes = unsafe {
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

    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    if existing_blocks < FIRST_DATA_BLOCK_NUMBER {
        return Err(
            "ec_spire root/control block must be initialized before object tuples".to_owned(),
        );
    }

    if existing_blocks > FIRST_DATA_BLOCK_NUMBER {
        let last_data_block = existing_blocks - 1;
        if let Some(tid) =
            unsafe { try_append_object_tuple_to_block(index_relation, last_data_block, payload)? }
        {
            return Ok(tid);
        }

        let required_space = raw_tuple_storage_bytes(payload.len());
        let fsm_block = unsafe { pg_sys::GetPageWithFreeSpace(index_relation, required_space) };
        if fsm_block >= FIRST_DATA_BLOCK_NUMBER
            && fsm_block < existing_blocks
            && fsm_block != last_data_block
        {
            if let Some(tid) =
                unsafe { try_append_object_tuple_to_block(index_relation, fsm_block, payload)? }
            {
                return Ok(tid);
            }
        }
    }

    unsafe { append_object_tuple_to_new_block(index_relation, payload) }
}

pub(super) unsafe fn read_object_tuple(
    index_relation: pg_sys::Relation,
    tid: crate::storage::page::ItemPointer,
) -> Result<Vec<u8>, String> {
    unsafe { with_pinned_object_tuple(index_relation, tid, |tuple| Ok(tuple.to_vec())) }
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

    let buffer = LockedBufferGuard::read_main(
        index_relation,
        tid.block_number,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    )
    .ok_or_else(|| format!("ec_spire failed to open object block {}", tid.block_number))?;
    let page = buffer.page();
    let page_size = buffer.page_size();
    let result = unsafe { with_object_tuple_from_locked_page(page, page_size, tid, f) };
    result
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
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    for block_number in FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = LockedBufferGuard::read_main(
            index_relation,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| format!("ec_spire failed to open object block {block_number}"))?;
        let page = buffer.page();
        let page_size = buffer.page_size();
        let max_offset = unsafe { pg_sys::PageGetMaxOffsetNumber(page) };
        let mut result = Ok(());
        for offset_number in 1..=max_offset {
            result = unsafe {
                visit_object_tuple_from_locked_page(
                    page,
                    page_size,
                    crate::storage::page::ItemPointer {
                        block_number,
                        offset_number,
                    },
                    |tuple| {
                        visit(
                            crate::storage::page::ItemPointer {
                                block_number,
                                offset_number,
                            },
                            tuple,
                        )
                    },
                )
            }
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
    let buffer = LockedBufferGuard::read_main(
        index_relation,
        tid.block_number,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    )
    .ok_or_else(|| format!("ec_spire failed to open object block {}", tid.block_number))?;
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page =
        unsafe { wal_txn.register_buffer(buffer.buffer(), pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let page_size = buffer.page_size();
    let result = unsafe {
        with_object_tuple_from_locked_page(page, page_size, tid, |tuple| {
            if tuple.len() != payload.len() {
                return Err(format!(
                    "ec_spire object tuple rewrite length changed from {} to {}",
                    tuple.len(),
                    payload.len()
                ));
            }

            ptr::copy_nonoverlapping(payload.as_ptr(), tuple.as_ptr() as *mut u8, payload.len());
            Ok(())
        })
    };
    match result {
        Ok(()) => unsafe {
            wal_txn.finish();
            Ok(())
        },
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
        let buffer = LockedBufferGuard::read_main(
            index_relation,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
        .ok_or_else(|| format!("ec_spire failed to open object block {block_number}"))?;
        let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
        let page = unsafe {
            wal_txn.register_buffer(buffer.buffer(), pg_sys::GENERIC_XLOG_FULL_IMAGE as i32)
        };
        let page_size = buffer.page_size();
        let max_offset = unsafe { pg_sys::PageGetMaxOffsetNumber(page) };
        let mut changed = false;
        for offset in offsets.into_iter().rev() {
            if offset == pg_sys::InvalidOffsetNumber || offset > max_offset {
                std::mem::drop(wal_txn);
                return Err(format!(
                    "ec_spire object tuple delete offset {} out of range on block {}",
                    offset, block_number
                ));
            }
            let item_id = unsafe { pg_sys::PageGetItemId(page, offset) };
            if item_id.is_null() {
                std::mem::drop(wal_txn);
                return Err(format!(
                    "ec_spire object tuple delete ({block_number},{offset}) returned a null item id"
                ));
            }
            let item_id_ref = unsafe { &*item_id };
            if item_id_ref.lp_flags() == 0 {
                continue;
            }
            let tuple_offset = item_id_ref.lp_off() as usize;
            let tuple_len = item_id_ref.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                std::mem::drop(wal_txn);
                return Err(format!(
                    "ec_spire object tuple delete ({block_number},{offset}) has invalid bounds"
                ));
            }
            unsafe { pg_sys::PageIndexTupleDeleteNoCompact(page, offset) };
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
            unsafe { wal_txn.finish() };
        }
        let free_space = unsafe { pg_sys::PageGetFreeSpace(page) as usize };
        unsafe { pg_sys::RecordPageWithFreeSpace(index_relation, block_number, free_space) };
    }
    Ok((removed_tuple_count, removed_tuple_bytes))
}

unsafe fn try_append_object_tuple_to_block(
    index_relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    payload: &[u8],
) -> Result<Option<crate::storage::page::ItemPointer>, String> {
    let buffer = LockedBufferGuard::read_main(
        index_relation,
        block_number,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    )
    .ok_or_else(|| format!("ec_spire failed to open object block {block_number}"))?;
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page =
        unsafe { wal_txn.register_buffer(buffer.buffer(), pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let page_size = buffer.page_size();
    if raw_tuple_storage_bytes(payload.len()) > page_size {
        std::mem::drop(wal_txn);
        return Err(format!(
            "ec_spire object tuple payload {} exceeds page size {page_size}",
            payload.len()
        ));
    }

    let free_space = unsafe { pg_sys::PageGetFreeSpace(page) as usize };
    if free_space < raw_tuple_storage_bytes(payload.len()) {
        unsafe { pg_sys::RecordPageWithFreeSpace(index_relation, block_number, free_space) };
        std::mem::drop(wal_txn);
        return Ok(None);
    }

    let offset = unsafe {
        pg_sys::PageAddItemExtended(
            page,
            payload.as_ptr().cast_mut().cast(),
            payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if offset == pg_sys::InvalidOffsetNumber {
        std::mem::drop(wal_txn);
        return Err(format!(
            "ec_spire failed to append object tuple to block {block_number}"
        ));
    }

    unsafe { wal_txn.finish() };
    let free_space = unsafe { pg_sys::PageGetFreeSpace(page) as usize };
    unsafe { pg_sys::RecordPageWithFreeSpace(index_relation, block_number, free_space) };
    Ok(Some(crate::storage::page::ItemPointer {
        block_number,
        offset_number: offset,
    }))
}

unsafe fn append_object_tuple_to_new_block(
    index_relation: pg_sys::Relation,
    payload: &[u8],
) -> Result<crate::storage::page::ItemPointer, String> {
    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    if existing_blocks < FIRST_DATA_BLOCK_NUMBER {
        return Err(
            "ec_spire root/control block must be initialized before object tuples".to_owned(),
        );
    }

    let buffer = LockedBufferGuard::read_main_locked(
        index_relation,
        P_NEW,
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
    )
    .ok_or_else(|| "ec_spire failed to allocate object block".to_owned())?;
    let page_size = buffer.page_size();
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page =
        unsafe { wal_txn.register_buffer(buffer.buffer(), pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(page, page_size, 0) };
    if unsafe { pg_sys::PageGetFreeSpace(page) as usize } < raw_tuple_storage_bytes(payload.len()) {
        std::mem::drop(wal_txn);
        return Err(format!(
            "ec_spire object tuple payload {} exceeds page capacity",
            payload.len()
        ));
    }

    let offset = unsafe {
        pg_sys::PageAddItemExtended(
            page,
            payload.as_ptr().cast_mut().cast(),
            payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if offset == pg_sys::InvalidOffsetNumber {
        std::mem::drop(wal_txn);
        return Err("ec_spire failed to append object tuple to new block".to_owned());
    }
    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer.buffer()) };

    unsafe { wal_txn.finish() };
    let free_space = unsafe { pg_sys::PageGetFreeSpace(page) as usize };
    unsafe { pg_sys::RecordPageWithFreeSpace(index_relation, block_number, free_space) };
    Ok(crate::storage::page::ItemPointer {
        block_number,
        offset_number: offset,
    })
}

unsafe fn with_object_tuple_from_locked_page<F, R>(
    page: pg_sys::Page,
    page_size: usize,
    tid: crate::storage::page::ItemPointer,
    f: F,
) -> Result<R, String>
where
    F: FnOnce(&[u8]) -> Result<R, String>,
{
    let max_offset = unsafe { pg_sys::PageGetMaxOffsetNumber(page) };
    if tid.offset_number == pg_sys::InvalidOffsetNumber || tid.offset_number > max_offset {
        return Err(format!(
            "ec_spire object tuple offset {} out of range on block {}",
            tid.offset_number, tid.block_number
        ));
    }

    match unsafe { visit_object_tuple_from_locked_page(page, page_size, tid, f)? } {
        SpireObjectTupleVisit::Unused => Err(format!(
            "ec_spire object tuple ({},{}) points at an unused slot",
            tid.block_number, tid.offset_number
        )),
        SpireObjectTupleVisit::Present(result) => Ok(result),
    }
}

unsafe fn visit_object_tuple_from_locked_page<F, R>(
    page: pg_sys::Page,
    page_size: usize,
    tid: crate::storage::page::ItemPointer,
    visit: F,
) -> Result<SpireObjectTupleVisit<R>, String>
where
    F: FnOnce(&[u8]) -> Result<R, String>,
{
    let item_id = unsafe { pg_sys::PageGetItemId(page, tid.offset_number) };
    if item_id.is_null() {
        return Err(format!(
            "ec_spire object tuple ({},{}) returned a null item id",
            tid.block_number, tid.offset_number
        ));
    }
    let item_id_ref = unsafe { &*item_id };
    if item_id_ref.lp_flags() == 0 {
        return Ok(SpireObjectTupleVisit::Unused);
    }

    let tuple_offset = item_id_ref.lp_off() as usize;
    let tuple_len = item_id_ref.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        return Err(format!(
            "ec_spire object tuple ({},{}) has invalid bounds",
            tid.block_number, tid.offset_number
        ));
    }

    let tuple_ptr = unsafe { pg_sys::PageGetItem(page, item_id) }.cast::<u8>();
    if tuple_ptr.is_null() {
        return Err(format!(
            "ec_spire object tuple ({},{}) returned a null tuple pointer",
            tid.block_number, tid.offset_number
        ));
    }
    let tuple = unsafe { std::slice::from_raw_parts(tuple_ptr, tuple_len) };
    visit(tuple).map(SpireObjectTupleVisit::Present)
}
