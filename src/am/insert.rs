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
            let m = u16::try_from(options.m).expect("validated m should fit in u16");
            let code_len = tuple.code.len();

            if let Some(source_column) = options.build_source_column {
                pgrx::error!(
                    "tqhnsw aminsert does not support build_source_column indexes yet: {source_column}"
                );
            }

            // Snapshot metadata under a SHARE lock so the duplicate scan does not
            // serialize concurrent inserts behind the metadata-page exclusive lock.
            let metadata_snapshot = shared::read_metadata_page(index_relation);

            // First-insert path: shape has never been initialized. Keep this on the
            // old exclusive path because shape init atomicity still matters, and the
            // duplicate scan is degenerate on an effectively empty index.
            if metadata_snapshot.dimensions == 0 && metadata_snapshot.bits == 0 {
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
                        coalesce_duplicate_heap_tid(
                            index_relation,
                            element_tid,
                            code_len,
                            heap_tid,
                        );
                        return;
                    }

                    let insert_level =
                        choose_insert_level(m, metadata.seed, heap_tid, tuple.code.len());
                    let element_tid = append_heap_tuple(index_relation, &tuple, insert_level, m);
                    if metadata.entry_point == page::ItemPointer::INVALID {
                        metadata.entry_point = element_tid;
                        metadata.max_level = insert_level;
                    }
                });
                return false;
            }

            // Fast path: shape is known. Those fields are write-once after
            // initialization, so the SHARE-read snapshot is authoritative here.
            if tuple.dimensions != metadata_snapshot.dimensions
                || tuple.bits != metadata_snapshot.bits
                || tuple.seed != metadata_snapshot.seed
            {
                pgrx::error!(
                    "tqhnsw aminsert requires matching tqvector shape ({},{},{}) but got ({},{},{})",
                    metadata_snapshot.dimensions,
                    metadata_snapshot.bits,
                    metadata_snapshot.seed,
                    tuple.dimensions,
                    tuple.bits,
                    tuple.seed
                );
            }

            // Duplicate scan runs with only SHARE locks on individual data pages.
            // A concurrent insert that commits the same code between this scan and
            // our append may double-insert; that rare race is acceptable here in
            // exchange for removing the metadata-page serialization point.
            if let Some(element_tid) = find_duplicate_element_tid(
                index_relation,
                heap_relation,
                metadata_snapshot.dimensions,
                metadata_snapshot.bits,
                tuple.gamma,
                code_len,
                &tuple.code,
            ) {
                coalesce_duplicate_heap_tid(index_relation, element_tid, code_len, heap_tid);
                return false;
            }

            let insert_level =
                choose_insert_level(m, metadata_snapshot.seed, heap_tid, tuple.code.len());
            let element_tid = append_heap_tuple(index_relation, &tuple, insert_level, m);

            // Only reacquire the metadata exclusive lock when the snapshot says
            // entry_point still needs repair or the new node outranks the current
            // maximum level. Re-check under the lock so we do not clobber a
            // racing initializer or promotion.
            if metadata_snapshot.entry_point == page::ItemPointer::INVALID
                || insert_level > metadata_snapshot.max_level
            {
                shared::with_locked_metadata_page(index_relation, |metadata| {
                    if metadata.entry_point == page::ItemPointer::INVALID
                        || insert_level > metadata.max_level
                    {
                        metadata.entry_point = element_tid;
                        metadata.max_level = insert_level;
                    }
                });
            }
            false
        })
    }
}

fn choose_insert_level(m: u16, seed: u64, heap_tid: page::ItemPointer, code_len: usize) -> u8 {
    let max_level = max_insert_level_that_fits(m, code_len, pg_sys::BLCKSZ as usize);
    if max_level == 0 {
        return 0;
    }

    let random_bits = splitmix64(seed ^ encode_heap_tid(heap_tid));
    level_from_random_bits(random_bits, m, max_level)
}

fn max_insert_level_that_fits(m: u16, code_len: usize, page_size: usize) -> u8 {
    let mut level = page::max_level_that_fits(m, page_size);
    loop {
        let required_bytes =
            page::raw_tuple_storage_bytes(page::neighbor_tuple_encoded_len(level, m))
                + page::raw_tuple_storage_bytes(page::TqElementTuple::encoded_len(code_len));
        if required_bytes <= page_size.saturating_sub(page::PAGE_HEADER_BYTES) {
            return level;
        }
        if level == 0 {
            return 0;
        }
        level = level.saturating_sub(1);
    }
}

fn level_from_random_bits(random_bits: u64, m: u16, max_level: u8) -> u8 {
    let unit = ((random_bits as f64) + 1.0_f64) / ((u64::MAX as f64) + 1.0_f64);
    let sampled_level = (-unit.ln() / (m as f64).ln()).floor();
    sampled_level.clamp(0.0_f64, max_level as f64) as u8
}

fn encode_heap_tid(heap_tid: page::ItemPointer) -> u64 {
    (u64::from(heap_tid.block_number) << 16) | u64::from(heap_tid.offset_number)
}

fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = (state ^ (state >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^ (state >> 31)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn debug_insert_level_for_heap_tid(
    m: u16,
    seed: u64,
    heap_tid: page::ItemPointer,
    code_len: usize,
) -> u8 {
    choose_insert_level(m, seed, heap_tid, code_len)
}

unsafe fn append_heap_tuple(
    index_relation: pg_sys::Relation,
    tuple: &build::BuildTuple,
    level: u8,
    m: u16,
) -> page::ItemPointer {
    let neighbor_slot_count = page::neighbor_slots(level, m);
    let neighbor_payload = page::TqNeighborTuple {
        count: u16::try_from(neighbor_slot_count).expect("neighbor slot count should fit in u16"),
        tids: vec![page::ItemPointer::INVALID; neighbor_slot_count],
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
                append_heap_tuple_to_new_page(index_relation, tuple, level, &neighbor_payload)
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
        level,
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
    level: u8,
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
        level,
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
