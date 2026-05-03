//! Relation-backed root/control page helpers for `ec_spire`.

use std::ptr;

use pgrx::pg_sys;

use super::meta::SpireRootControlState;
use crate::storage::{
    page::{align_up, ALIGNMENT_BYTES, METADATA_BLOCK_NUMBER},
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
    let root_control_ptr = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    let root_control_bytes = unsafe {
        std::slice::from_raw_parts(root_control_ptr, SpireRootControlState::encoded_len())
    };
    let root_control =
        SpireRootControlState::decode(root_control_bytes).unwrap_or_else(|e| pgrx::error!("{e}"));
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    root_control
}
