use std::ptr;

use pgrx::pg_sys;

use crate::storage::{page::METADATA_BLOCK_NUMBER, wal};

const P_NEW: pg_sys::BlockNumber = u32::MAX;

pub(crate) unsafe fn initialize_metadata_page(
    index_relation: pg_sys::Relation,
    metadata_bytes: &[u8],
    access_method_name: &str,
) {
    if metadata_bytes.is_empty() {
        pgrx::error!("{access_method_name} metadata page payload must not be empty");
    }

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
        pgrx::error!("{access_method_name} failed to allocate metadata buffer");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let special_size = align_special_size(metadata_bytes.len());
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    unsafe { write_metadata_bytes(page, metadata_bytes) };

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

unsafe fn write_metadata_bytes(page: pg_sys::Page, metadata_bytes: &[u8]) {
    let page_contents = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    unsafe {
        ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), page_contents, metadata_bytes.len());
    }
}

fn align_special_size(metadata_len: usize) -> usize {
    (metadata_len + 7) & !7
}

#[cfg(test)]
mod tests {
    use super::align_special_size;

    #[test]
    fn align_special_size_rounds_up_to_eight_bytes() {
        assert_eq!(align_special_size(1), 8);
        assert_eq!(align_special_size(8), 8);
        assert_eq!(align_special_size(9), 16);
    }
}
