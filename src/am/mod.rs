//! Access-method scaffolding for the future `tqhnsw` implementation.

use std::ffi::c_void;
use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, varlena, PgBox};

use self::build::{BuildState, BuildTuple};

mod cost;
mod build;
mod options;
pub mod page;
mod routine;
mod scan;
mod vacuum;
pub mod wal;

#[cfg(any(test, feature = "pg_test"))]
pub(crate) use self::scan::{
    debug_begin_end_scan, debug_end_scan_twice, debug_gettuple_after_rescan_result,
    debug_entry_point_neighbor_tids,
    debug_gettuple_backward_after_rescan,
    debug_gettuple_current_result_lifecycle, debug_gettuple_current_result_neighbors,
    debug_gettuple_current_result_state,
    debug_gettuple_exhaustion_state, debug_gettuple_rescan_after_exhaustion,
    debug_gettuple_rescan_after_partial, debug_gettuple_scan_heap_tids,
    debug_gettuple_without_rescan, debug_rescan_null_query, debug_rescan_overwrites_query_dimensions,
    debug_rescan_query_dimensions, debug_rescan_with_index_qual, debug_rescan_with_multiple_orderbys,
};

const TQHNSW_DEFAULT_M: i32 = 8;
const TQHNSW_MIN_M: i32 = 2;
const TQHNSW_MAX_M: i32 = 100;
const TQHNSW_DEFAULT_EF_CONSTRUCTION: i32 = 64;
const TQHNSW_MIN_EF_CONSTRUCTION: i32 = 10;
const TQHNSW_MAX_EF_CONSTRUCTION: i32 = 1000;
const P_NEW: pg_sys::BlockNumber = u32::MAX;

unsafe extern "C-unwind" fn tqhnsw_aminsert(
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
            let heap_tid = decode_heap_tid(heap_tid);
            let tuple = build::build_heap_tuple(values, isnull, heap_tid);
            let options = options::relation_options(index_relation);
            let code_len = tuple.code.len();

            if let Some(source_column) = options.build_source_column {
                pgrx::error!(
                    "tqhnsw aminsert does not support build_source_column indexes yet: {source_column}"
                );
            }

            with_locked_metadata_page(index_relation, |metadata| {
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

pub(super) unsafe extern "C-unwind" fn tqhnsw_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = &mut *state.cast::<BuildState>();
            let heap_tid = decode_heap_tid(tid);
            let tuple = build::build_heap_tuple(values, isnull, heap_tid);
            state.push(tuple);
        })
    }
}

unsafe fn initialize_metadata_page(index_relation: pg_sys::Relation, metadata: page::MetadataPage) {
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

unsafe fn update_metadata_page(index_relation: pg_sys::Relation, metadata: page::MetadataPage) {
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

unsafe fn with_locked_metadata_page<T>(
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

unsafe fn append_heap_tuple(
    index_relation: pg_sys::Relation,
    tuple: &BuildTuple,
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
    tuple: &BuildTuple,
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
    heap_relation: pg_sys::Relation,
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
            if element.code == code
                && !element.heaptids.is_empty()
                && candidate_element_gamma(index_relation, heap_relation, element.heaptids[0])
                    .to_bits()
                    == gamma.to_bits()
            {
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

unsafe fn candidate_element_gamma(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    heap_tid: page::ItemPointer,
) -> f32 {
    heap_tqvector_gamma(
        index_relation,
        heap_relation,
        heap_tid,
        "duplicate gamma check",
    )
}

unsafe fn heap_tqvector_gamma(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    heap_tid: page::ItemPointer,
    context: &str,
) -> f32 {
    let (heap_relation, opened_heap_relation) = if heap_relation.is_null() {
        let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
        if heap_oid == pg_sys::InvalidOid {
            pgrx::error!("tqhnsw {context} could not resolve heap relation for index");
        }
        (
            unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) },
            true,
        )
    } else {
        (heap_relation, false)
    };

    let index_form = unsafe { &*(*index_relation).rd_index };
    if index_form.indnkeyatts != 1 {
        pgrx::error!(
            "tqhnsw duplicate lookup currently requires exactly one key attribute, found {}",
            index_form.indnkeyatts
        );
    }

    let indkey = unsafe { index_form.indkey.values.as_slice(index_form.indnkeyatts as usize) };
    let attnum = *indkey
        .first()
        .expect("single-column tqhnsw index should expose one heap attribute");
    if attnum <= 0 {
        pgrx::error!("tqhnsw duplicate lookup does not support expression index attributes");
    }

    let mut fetched_tid = pg_sys::ItemPointerData::default();
    pgrx::itemptr::item_pointer_set_all(
        &mut fetched_tid,
        heap_tid.block_number,
        heap_tid.offset_number,
    );

    let mut heap_tuple = pg_sys::HeapTupleData {
        t_self: fetched_tid,
        ..Default::default()
    };
    let mut buffer = pg_sys::InvalidBuffer as pg_sys::Buffer;
    let found = unsafe {
        pg_sys::heap_fetch(
            heap_relation,
            std::ptr::addr_of_mut!(pg_sys::SnapshotAnyData),
            &mut heap_tuple,
            &mut buffer,
            false,
        )
    };
    if !found {
        pgrx::error!(
            "tqhnsw failed to fetch representative heap tuple ({},{}) for {context}",
            heap_tid.block_number, heap_tid.offset_number
        );
    }

    let tupdesc = unsafe { pgrx::PgTupleDesc::from_pg_unchecked((*heap_relation).rd_att) };
    let datum = unsafe {
        pgrx::heap_getattr_raw(
            &mut heap_tuple,
            std::num::NonZeroUsize::new(attnum as usize)
                .expect("positive indexed heap attribute number"),
            tupdesc.as_ptr(),
        )
    }
    .unwrap_or_else(|| {
        pgrx::error!(
            "tqhnsw found NULL representative heap value for {context} on attribute {}",
            attnum
        )
    });

    let original = datum.cast_mut_ptr::<std::ffi::c_void>().cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    let is_copy = !std::ptr::eq(varlena, original);
    let bytes = unsafe { varlena::varlena_to_byte_slice(varlena) };
    let (_, _, _, gamma, _) =
        crate::unpack(bytes).unwrap_or_else(|e| pgrx::error!("tqhnsw found invalid heap tqvector during {context}: {e}"));
    if is_copy {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }
    if buffer != pg_sys::InvalidBuffer as pg_sys::Buffer {
        unsafe { pg_sys::ReleaseBuffer(buffer) };
    }
    if opened_heap_relation {
        unsafe { pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }
    gamma
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
    let item_id = unsafe { &*page_item_id(page_ptr, element_tid.offset_number) };
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

unsafe fn page_item_id(page_ptr: *mut u8, offset: u16) -> *const pg_sys::ItemIdData {
    unsafe {
        page_ptr
            .add(
                page::PAGE_HEADER_BYTES + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()),
            )
            .cast::<pg_sys::ItemIdData>()
    }
}

fn page_line_pointer_count(page_ptr: *mut u8) -> u16 {
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

pub(super) fn average_source_representatives(
    existing: &mut [f32],
    existing_count: usize,
    incoming: &[f32],
    incoming_count: usize,
) {
    assert_eq!(existing.len(), incoming.len());
    assert!(existing_count > 0);
    assert!(incoming_count > 0);

    let total_count = existing_count + incoming_count;
    for (existing_value, incoming_value) in existing.iter_mut().zip(incoming.iter()) {
        *existing_value = ((*existing_value * existing_count as f32)
            + (*incoming_value * incoming_count as f32))
            / total_count as f32;
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

unsafe fn read_metadata_page(index_relation: pg_sys::Relation) -> page::MetadataPage {
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
unsafe fn read_data_page(
    index_relation: pg_sys::Relation,
    block_number: u32,
) -> DebugIndexDataPage {
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
    DebugIndexDataPage {
        block_number,
        tuples,
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_index_metadata(
    index_oid: pg_sys::Oid,
) -> (u32, i32, i32, page::MetadataPage) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let options = unsafe { options::relation_options(index_relation) };
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

    let stats = unsafe { vacuum::tqhnsw_ambulkdelete(info_ptr, ptr::null_mut(), None, ptr::null_mut()) };
    let stats = unsafe { vacuum::tqhnsw_amvacuumcleanup(info_ptr, stats) };
    let result = unsafe { *stats };

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::build::{
        build_hnsw_graph, build_scored_neighbor_graph, choose_entry_point, BuildState, BuildTuple,
        HnswBuildNode,
    };
    use super::options::TqHnswOptions;

    fn encoded_code(vector: &[f32], bits: u8, seed: u64) -> Vec<u8> {
        let quantizer = crate::quant::prod::ProdQuantizer::cached(vector.len(), bits, seed);
        let encoded = quantizer.encode(vector);
        let mut code = encoded.mse_packed;
        code.extend_from_slice(&encoded.qjl_packed);
        code
    }

    #[test]
    fn scored_neighbor_graph_prefers_similarity_over_insert_order() {
        let seed = 42_u64;
        let bits = 8_u8;
        let tuples = vec![
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 1,
                }],
                dimensions: 8,
                bits,
                seed,
                gamma: 0.0,
                code: encoded_code(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 2,
                }],
                dimensions: 8,
                bits,
                seed,
                gamma: 0.0,
                code: encoded_code(&[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 3,
                }],
                dimensions: 8,
                bits,
                seed,
                gamma: 0.0,
                code: encoded_code(&[0.98, 0.02, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
        ];
        let state = BuildState {
            options: TqHnswOptions {
                m: 1,
                ef_construction: 32,
                build_source_column: None,
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            dimensions: Some(8),
            bits: Some(bits),
            seed: Some(seed),
        };

        let graph = build_scored_neighbor_graph(&state);

        assert_eq!(graph.len(), 3);
        assert_eq!(graph[0], vec![2]);
        assert_eq!(graph[2], vec![0]);
    }

    #[test]
    fn hnsw_graph_builds_for_small_dataset() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = vec![
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 1,
                }],
                dimensions: 4,
                bits,
                seed,
                gamma: 0.0,
                code: encoded_code(&[1.0, 0.0, 0.5, -1.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 2,
                }],
                dimensions: 4,
                bits,
                seed,
                gamma: 0.0,
                code: encoded_code(&[0.0, 1.0, 0.25, -0.5], bits, seed),
                source_vector: None,
                source_count: 0,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 3,
                }],
                dimensions: 4,
                bits,
                seed,
                gamma: 0.0,
                code: encoded_code(&[-1.0, 0.5, 0.0, 1.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
        ];
        let state = BuildState {
            options: TqHnswOptions {
                m: 10,
                ef_construction: 90,
                build_source_column: None,
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            dimensions: Some(4),
            bits: Some(bits),
            seed: Some(seed),
        };

        let nodes = build_hnsw_graph(&state);

        assert_eq!(nodes.len(), 3);
        assert!(nodes.iter().any(|node| !node.neighbors.is_empty()));
    }

    #[test]
    fn source_scored_entry_point_prefers_raw_vectors() {
        let seed = 42_u64;
        let bits = 4_u8;
        let state = BuildState {
            options: TqHnswOptions {
                m: 2,
                ef_construction: 64,
                build_source_column: Some("source".to_owned()),
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: vec![
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: 1,
                    }],
                    dimensions: 2,
                    bits,
                    seed,
                    gamma: 0.0,
                    code: vec![0x00, 0x00],
                    source_vector: Some(vec![1.0, 0.0]),
                    source_count: 1,
                },
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: 2,
                    }],
                    dimensions: 2,
                    bits,
                    seed,
                    gamma: 0.0,
                    code: vec![0xff, 0xff],
                    source_vector: Some(vec![0.9, 0.1]),
                    source_count: 1,
                },
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: 3,
                    }],
                    dimensions: 2,
                    bits,
                    seed,
                    gamma: 0.0,
                    code: vec![0x00, 0x01],
                    source_vector: Some(vec![-1.0, 0.0]),
                    source_count: 1,
                },
            ],
            dimensions: Some(2),
            bits: Some(bits),
            seed: Some(seed),
        };

        let graph_nodes = vec![
            HnswBuildNode {
                level: 0,
                neighbors: vec![1],
            },
            HnswBuildNode {
                level: 0,
                neighbors: vec![2],
            },
            HnswBuildNode {
                level: 0,
                neighbors: vec![1],
            },
        ];
        let element_tids = vec![
            page::ItemPointer {
                block_number: 2,
                offset_number: 1,
            },
            page::ItemPointer {
                block_number: 2,
                offset_number: 2,
            },
            page::ItemPointer {
                block_number: 2,
                offset_number: 3,
            },
        ];

        let entry_point = choose_entry_point(&element_tids, &graph_nodes, &state)
            .expect("entry point should exist");
        assert_eq!(entry_point, element_tids[0]);
    }

    #[test]
    fn average_source_representative_weights_by_duplicate_count() {
        let mut representative = vec![1.0, 0.0];
        average_source_representatives(&mut representative, 1, &[0.0, 1.0], 1);
        assert_eq!(representative, vec![0.5, 0.5]);

        average_source_representatives(&mut representative, 2, &[1.0, 1.0], 2);
        assert_eq!(representative, vec![0.75, 0.75]);
    }
}
