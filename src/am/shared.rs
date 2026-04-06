use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, PgBox};

use super::{page, wal, P_NEW};

pub(super) unsafe fn initialize_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: page::MetadataPage,
) {
    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        page::METADATA_BLOCK_NUMBER
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
        pgrx::error!("tqhnsw failed to allocate metadata buffer");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    let special_size = (metadata_bytes.len() + 7) & !7;
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    unsafe { write_metadata_bytes(page, &metadata_bytes) };

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

unsafe fn write_metadata_bytes(page: pg_sys::Page, metadata_bytes: &[u8]) {
    let page_contents = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    unsafe {
        ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), page_contents, metadata_bytes.len());
    }
}

pub(super) unsafe fn update_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: page::MetadataPage,
) {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to open metadata buffer");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    unsafe { write_metadata_bytes(page, &metadata_bytes) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

pub(super) unsafe fn with_locked_metadata_page<T>(
    index_relation: pg_sys::Relation,
    f: impl FnOnce(&mut page::MetadataPage) -> T,
) -> T {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to open metadata buffer");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let raw_page = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let page_bytes = unsafe { std::slice::from_raw_parts(raw_page, page_size) };
    let mut metadata =
        page::MetadataPage::decode_page(page_bytes).expect("metadata page should decode");
    let result = f(&mut metadata);

    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    unsafe { write_metadata_bytes(page, &metadata_bytes) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    result
}

pub(super) unsafe fn tqhnsw_noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };

    unsafe {
        (*stats).num_pages = pg_sys::RelationGetNumberOfBlocksInFork(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
        );
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = count_element_tuples(index_relation) as f64;
    }

    stats
}

pub(super) unsafe fn count_element_tuples(index_relation: pg_sys::Relation) -> usize {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut count = 0_usize;

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let line_pointer_count = page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let item_id = unsafe { &*page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid tuple bounds while counting vacuum tuples on block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            if tuple_bytes.first().copied() == Some(page::TQ_ELEMENT_TAG) {
                count += 1;
            }
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    count
}

pub(super) unsafe fn page_item_id(page_ptr: *mut u8, offset: u16) -> *const pg_sys::ItemIdData {
    unsafe {
        page_ptr
            .add(
                page::PAGE_HEADER_BYTES + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()),
            )
            .cast::<pg_sys::ItemIdData>()
    }
}

pub(super) fn page_line_pointer_count(page_ptr: *mut u8) -> u16 {
    let page_header = page_ptr.cast::<pg_sys::PageHeaderData>();
    ((unsafe { (*page_header).pd_lower } as usize - size_of::<pg_sys::PageHeaderData>())
        / size_of::<pg_sys::ItemIdData>()) as u16
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer) -> page::ItemPointer {
    if tid.is_null() {
        pgrx::error!("tqhnsw ambuild received a null heap tid");
    }
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    page::ItemPointer {
        block_number,
        offset_number,
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebugIndexDataPage {
    pub block_number: u32,
    pub tuples: Vec<Vec<u8>>,
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_index_pages(
    index_oid: pg_sys::Oid,
) -> (u32, page::MetadataPage, Vec<DebugIndexDataPage>) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };

    let metadata = unsafe { read_metadata_page(index_relation) };
    let mut data_pages = Vec::new();
    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        data_pages.push(unsafe { read_data_page(index_relation, block_number) });
    }

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (block_count, metadata, data_pages)
}

pub(super) unsafe fn read_metadata_page(index_relation: pg_sys::Relation) -> page::MetadataPage {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let raw_page = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let page_bytes = unsafe { std::slice::from_raw_parts(raw_page, page_size) };
    let metadata =
        page::MetadataPage::decode_page(page_bytes).expect("metadata page should decode");
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    metadata
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn read_data_page(index_relation: pg_sys::Relation, block_number: u32) -> DebugIndexDataPage {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let raw_page = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let page_header = raw_page.cast::<pg_sys::PageHeaderData>();
    let line_pointer_count = ((unsafe { (*page_header).pd_lower } as usize
        - size_of::<pg_sys::PageHeaderData>())
        / size_of::<pg_sys::ItemIdData>()) as u16;

    let mut tuples = Vec::with_capacity(line_pointer_count as usize);
    for offset in 1..=line_pointer_count {
        let item_id_ptr = unsafe {
            raw_page
                .add(
                    page::PAGE_HEADER_BYTES
                        + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()),
                )
                .cast::<pg_sys::ItemIdData>()
        };
        let item_id = unsafe { &*item_id_ptr };
        if item_id.lp_flags() == 0 {
            continue;
        }
        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw debug read found invalid tuple bounds on block {block_number}");
        }
        tuples.push(
            unsafe { std::slice::from_raw_parts(raw_page.add(tuple_offset), tuple_len) }.to_vec(),
        );
    }

    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    DebugIndexDataPage { block_number, tuples }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_index_metadata(
    index_oid: pg_sys::Oid,
) -> (u32, i32, i32, page::MetadataPage) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let options = unsafe { super::options::relation_options(index_relation) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let metadata = unsafe { read_metadata_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    (block_count, options.m, options.ef_construction, metadata)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_vacuum_stats(index_oid: pg_sys::Oid) -> pg_sys::IndexBulkDeleteResult {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;

    let stats =
        unsafe {
            super::vacuum::tqhnsw_ambulkdelete(info_ptr, ptr::null_mut(), None, ptr::null_mut())
        };
    let stats = unsafe { super::vacuum::tqhnsw_amvacuumcleanup(info_ptr, stats) };
    let result = unsafe { *stats };

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}
