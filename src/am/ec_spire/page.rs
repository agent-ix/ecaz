//! Relation-backed root/control page helpers for `ec_spire`.

use std::ptr;

use pgrx::pg_sys;

use super::meta::SpireRootControlState;
use crate::storage::{
    page::{
        align_up, raw_tuple_storage_bytes, ALIGNMENT_BYTES, FIRST_DATA_BLOCK_NUMBER,
        METADATA_BLOCK_NUMBER,
    },
    wal,
};

const P_NEW: pg_sys::BlockNumber = u32::MAX;

pub(super) unsafe fn initialize_root_control_page(
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
    let read_mode = if target_block == P_NEW {
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK
    } else {
        pg_sys::ReadBufferMode::RBM_NORMAL
    };
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            target_block,
            read_mode,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("ec_spire failed to allocate root/control buffer");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
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
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

pub(super) unsafe fn read_root_control_page(
    index_relation: pg_sys::Relation,
) -> SpireRootControlState {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("ec_spire failed to open root/control buffer");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let page = unsafe { pg_sys::BufferGetPage(buffer) };
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
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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

    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err(format!(
            "ec_spire failed to open object block {}",
            tid.block_number
        ));
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let page = unsafe { pg_sys::BufferGetPage(buffer) };
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let result = unsafe { with_object_tuple_from_locked_page(page, page_size, tid, f) };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    result
}

pub(super) unsafe fn scan_object_tuples<F>(
    index_relation: pg_sys::Relation,
    mut visit: F,
) -> Result<(), String>
where
    F: FnMut(crate::storage::page::ItemPointer, &[u8]) -> Result<(), String>,
{
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    for block_number in FIRST_DATA_BLOCK_NUMBER..block_count {
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
            return Err(format!(
                "ec_spire failed to open object block {block_number}"
            ));
        }

        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page = unsafe { pg_sys::BufferGetPage(buffer) };
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
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
                    &mut visit,
                )
            };
            if result.is_err() {
                break;
            }
        }
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        result?;
    }
    Ok(())
}

unsafe fn try_append_object_tuple_to_block(
    index_relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    payload: &[u8],
) -> Result<Option<crate::storage::page::ItemPointer>, String> {
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
        return Err(format!(
            "ec_spire failed to open object block {block_number}"
        ));
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    if raw_tuple_storage_bytes(payload.len()) > page_size {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_spire object tuple payload {} exceeds page size {page_size}",
            payload.len()
        ));
    }

    let free_space = unsafe { pg_sys::PageGetFreeSpace(page) as usize };
    if free_space < raw_tuple_storage_bytes(payload.len()) {
        unsafe { pg_sys::RecordPageWithFreeSpace(index_relation, block_number, free_space) };
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_spire failed to append object tuple to block {block_number}"
        ));
    }

    unsafe { wal_txn.finish() };
    let free_space = unsafe { pg_sys::PageGetFreeSpace(page) as usize };
    unsafe { pg_sys::RecordPageWithFreeSpace(index_relation, block_number, free_space) };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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

    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            P_NEW,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err("ec_spire failed to allocate object block".to_owned());
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(page, page_size, 0) };
    if unsafe { pg_sys::PageGetFreeSpace(page) as usize } < raw_tuple_storage_bytes(payload.len()) {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err("ec_spire failed to append object tuple to new block".to_owned());
    }
    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };

    unsafe { wal_txn.finish() };
    let free_space = unsafe { pg_sys::PageGetFreeSpace(page) as usize };
    unsafe { pg_sys::RecordPageWithFreeSpace(index_relation, block_number, free_space) };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
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

    let item_id = unsafe { pg_sys::PageGetItemId(page, tid.offset_number) };
    if item_id.is_null() {
        return Err(format!(
            "ec_spire object tuple ({},{}) returned a null item id",
            tid.block_number, tid.offset_number
        ));
    }
    let item_id_ref = unsafe { &*item_id };
    if item_id_ref.lp_flags() == 0 {
        return Err(format!(
            "ec_spire object tuple ({},{}) points at an unused slot",
            tid.block_number, tid.offset_number
        ));
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
    f(tuple)
}

unsafe fn visit_object_tuple_from_locked_page<F>(
    page: pg_sys::Page,
    page_size: usize,
    tid: crate::storage::page::ItemPointer,
    visit: &mut F,
) -> Result<(), String>
where
    F: FnMut(crate::storage::page::ItemPointer, &[u8]) -> Result<(), String>,
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
        return Ok(());
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
    visit(tid, tuple)
}
