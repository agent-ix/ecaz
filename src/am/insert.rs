use std::ptr;

use pgrx::pg_sys;

use super::{build, options, page, shared, wal};

const P_NEW: pg_sys::BlockNumber = u32::MAX;

pub(super) unsafe extern "C-unwind" fn tqhnsw_aminsert(
    index_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: pg_sys::ItemPointer,
    heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let heap_tid = shared::decode_heap_tid(heap_tid);
            let tuple = build::build_heap_tuple(values, isnull, heap_tid);
            let options = options::relation_options(index_relation);
            let code_len = tuple.code.len();

            if let Some(source_column) = options.build_source_column {
                pgrx::error!(
                    "tqhnsw aminsert does not support build_source_column indexes yet: {source_column}"
                );
            }

            shared::with_locked_metadata_page(index_relation, |metadata| {
                if metadata.dimensions == 0 && metadata.bits == 0 {
                    metadata.dimensions = tuple.dimensions;
                    metadata.bits = tuple.bits;
                    metadata.seed = tuple.seed;
                } else if tuple.dimensions != metadata.dimensions
                    || tuple.bits != metadata.bits
                    || tuple.seed != metadata.seed
                {
                    pgrx::error!(
                        "tqhnsw aminsert requires matching tqvector shape ({},{},{}) but got ({},{},{})",
                        metadata.dimensions,
                        metadata.bits,
                        metadata.seed,
                        tuple.dimensions,
                        tuple.bits,
                        tuple.seed
                    );
                }

                if let Some(element_tid) = find_duplicate_element_tid(
                    index_relation,
                    heap_relation,
                    metadata.dimensions,
                    metadata.bits,
                    tuple.gamma,
                    code_len,
                    &tuple.code,
                ) {
                    coalesce_duplicate_heap_tid(index_relation, element_tid, code_len, heap_tid);
                    return;
                }

                let element_tid = append_heap_tuple(index_relation, &tuple);
                if metadata.entry_point == page::ItemPointer::INVALID {
                    metadata.entry_point = element_tid;
                }
            });
            false
        })
    }
}

unsafe fn append_heap_tuple(
    index_relation: pg_sys::Relation,
    tuple: &build::BuildTuple,
) -> page::ItemPointer {
    let neighbor_payload = page::TqNeighborTuple {
        count: 0,
        tids: Vec::new(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode neighbor tuple: {e}"));
    let required_bytes = page::raw_tuple_storage_bytes(neighbor_payload.len())
        + page::raw_tuple_storage_bytes(page::TqElementTuple::encoded_len(tuple.code.len()));
    let mut staged_page =
        page::DataPage::new(page::FIRST_DATA_BLOCK_NUMBER, pg_sys::BLCKSZ as usize);
    staged_page
        .insert_raw_tuple(neighbor_payload.clone())
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to stage aminsert neighbor tuple: {e}"));
    if !staged_page.can_fit_raw_tuple(page::TqElementTuple::encoded_len(tuple.code.len())) {
        pgrx::error!(
            "tqhnsw aminsert does not yet support tuples that require more than one fresh data page"
        );
    }

    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks > page::FIRST_DATA_BLOCK_NUMBER {
        existing_blocks - 1
    } else {
        P_NEW
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
        pgrx::error!("tqhnsw failed to allocate data buffer for aminsert");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    if target_block == P_NEW {
        unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };
    } else {
        let free_space = unsafe { pg_sys::PageGetFreeSpace(page_ptr) as usize };
        if free_space < required_bytes {
            std::mem::drop(wal_txn);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return unsafe {
                append_heap_tuple_to_new_page(index_relation, tuple, &neighbor_payload)
            };
        }
    }

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write neighbor tuple during aminsert");
    }

    let element_payload = page::TqElementTuple {
        level: 0,
        deleted: false,
        heaptids: tuple.heap_tids.clone(),
        gamma: tuple.gamma,
        neighbortid: page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        code: tuple.code.clone(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode element tuple: {e}"));
    let element_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            element_payload.as_ptr().cast_mut().cast(),
            element_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if element_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write element tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: element_offset,
    }
}

unsafe fn append_heap_tuple_to_new_page(
    index_relation: pg_sys::Relation,
    tuple: &build::BuildTuple,
    neighbor_payload: &[u8],
) -> page::ItemPointer {
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
        pgrx::error!("tqhnsw failed to allocate fallback data buffer for aminsert");
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback neighbor tuple during aminsert");
    }

    let element_payload = page::TqElementTuple {
        level: 0,
        deleted: false,
        heaptids: tuple.heap_tids.clone(),
        gamma: tuple.gamma,
        neighbortid: page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        code: tuple.code.clone(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode fallback element tuple: {e}"));
    let element_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            element_payload.as_ptr().cast_mut().cast(),
            element_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if element_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback element tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: element_offset,
    }
}

unsafe fn find_duplicate_element_tid(
    index_relation: pg_sys::Relation,
    _heap_relation: pg_sys::Relation,
    dimensions: u16,
    bits: u8,
    gamma: f32,
    code_len: usize,
    code: &[u8],
) -> Option<page::ItemPointer> {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    if block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        return None;
    }

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
        let line_pointer_count = shared::page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let item_id = unsafe { &*shared::page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid tuple bounds while scanning block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                continue;
            }

            let element = page::TqElementTuple::decode(tuple_bytes, code_len).unwrap_or_else(|e| {
                pgrx::error!("tqhnsw failed to decode candidate duplicate tuple: {e}")
            });
            if element.code == code && element.gamma.to_bits() == gamma.to_bits() {
                unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
                return Some(page::ItemPointer {
                    block_number,
                    offset_number: offset,
                });
            }
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    let _ = dimensions;
    let _ = bits;
    None
}

unsafe fn coalesce_duplicate_heap_tid(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
    heap_tid: page::ItemPointer,
) {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            element_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!(
            "tqhnsw failed to open duplicate element block {}",
            element_tid.block_number
        );
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let item_id = unsafe { &*shared::page_item_id(page_ptr, element_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        pgrx::error!("tqhnsw duplicate element tuple slot is unused");
    }
    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        pgrx::error!(
            "tqhnsw found invalid duplicate tuple bounds on block {}",
            element_tid.block_number
        );
    }

    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    let mut element = page::TqElementTuple::decode(tuple_bytes, code_len)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode duplicate element tuple: {e}"));
    if element.heaptids.contains(&heap_tid) {
        unsafe { wal_txn.finish() };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return;
    }
    if element.heaptids.len() >= page::HEAPTID_INLINE_CAPACITY {
        pgrx::error!(
            "tqhnsw aminsert supports at most {} duplicate heap tids per encoded vector",
            page::HEAPTID_INLINE_CAPACITY
        );
    }
    element.heaptids.push(heap_tid);
    let encoded = element
        .encode()
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode coalesced element tuple: {e}"));
    if encoded.len() != tuple_len {
        pgrx::error!(
            "tqhnsw duplicate element tuple size changed from {} to {}",
            tuple_len,
            encoded.len()
        );
    }
    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), encoded.len());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}
