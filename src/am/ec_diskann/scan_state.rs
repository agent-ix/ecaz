use std::{ptr, slice};

use pgrx::{itemptr::item_pointer_get_both, pg_sys};

use crate::storage::page::{DataPageChain, ItemPointer, FIRST_DATA_BLOCK_NUMBER};

use super::{
    options::TqDiskannOptions,
    page::{
        VamanaMetadataPage, INDEX_FORMAT_V3_DISKANN, PAYLOAD_FLAG_BINARY_SIDECAR,
        VAMANA_METADATA_BYTES,
    },
    reader::VisitedState,
    scan::ScanResult,
};

#[derive(Debug)]
pub(super) struct DiskannScanOpaque {
    pub(super) metadata: VamanaMetadataPage,
    pub(super) chain: DataPageChain,
    pub(super) flat_codebooks: Vec<f32>,
    pub(super) query_rotated: Vec<f32>,
    pub(super) query_lut: Vec<f32>,
    pub(super) visited: VisitedState,
    pub(super) result_buf: Vec<ScanResult>,
    pub(super) result_cursor: usize,
    pub(super) rescan_called: bool,
    pub(super) top_k: usize,
    pub(super) list_size: usize,
    pub(super) rerank_budget: usize,
}

impl DiskannScanOpaque {
    pub(super) fn new(
        metadata: VamanaMetadataPage,
        chain: DataPageChain,
        options: TqDiskannOptions,
    ) -> Result<Self, String> {
        let scan_tuning = super::options::resolve_scan_tuning(&options);
        Ok(Self {
            metadata,
            chain,
            flat_codebooks: Vec::new(),
            query_rotated: Vec::new(),
            query_lut: Vec::new(),
            visited: VisitedState::new(),
            result_buf: Vec::new(),
            result_cursor: 0,
            rescan_called: false,
            top_k: reloption_usize(options.top_k, "top_k")?,
            list_size: reloption_usize(scan_tuning.effective_list_size, "list_size")?,
            rerank_budget: reloption_usize(options.rerank_budget, "rerank_budget")?,
        })
    }

    pub(super) fn binary_word_count(&self) -> usize {
        metadata_binary_word_count(&self.metadata)
    }

    pub(super) fn search_code_len(&self) -> usize {
        metadata_search_code_len(&self.metadata)
    }
}

fn reloption_usize(value: i32, name: &str) -> Result<usize, String> {
    usize::try_from(value)
        .map_err(|_| format!("ec_diskann {name} reloption must be >= 0, got {value}"))
}

pub(super) fn metadata_binary_word_count(metadata: &VamanaMetadataPage) -> usize {
    if metadata.payload_flags & PAYLOAD_FLAG_BINARY_SIDECAR != 0 {
        usize::from(metadata.dimensions).div_ceil(64)
    } else {
        0
    }
}

pub(super) fn metadata_search_code_len(metadata: &VamanaMetadataPage) -> usize {
    usize::from(metadata.search_subvector_count).div_ceil(2)
}

pub(super) unsafe fn materialize_chain_from_index(
    index_relation: pg_sys::Relation,
) -> Result<(VamanaMetadataPage, DataPageChain), String> {
    let metadata_buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            0,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(metadata_buffer) } {
        return Err("ec_diskann beginscan could not open metadata page".into());
    }
    unsafe { pg_sys::LockBuffer(metadata_buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let metadata_result = (|| -> Result<(VamanaMetadataPage, usize), String> {
        let page = unsafe { pg_sys::BufferGetPage(metadata_buffer) };
        let page_size = unsafe { pg_sys::BufferGetPageSize(metadata_buffer) as usize };
        let special_size = unsafe { pg_sys::PageGetSpecialSize(page) as usize };
        if special_size < VAMANA_METADATA_BYTES {
            return Err(format!(
                "ec_diskann metadata page special area too small: got {special_size}, expected at least {VAMANA_METADATA_BYTES}"
            ));
        }
        let metadata_ptr = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
        let metadata_bytes =
            unsafe { slice::from_raw_parts(metadata_ptr.cast_const(), VAMANA_METADATA_BYTES) };
        let format_version =
            u16::from_le_bytes(metadata_bytes[0..2].try_into().expect("format bytes"));
        if format_version != INDEX_FORMAT_V3_DISKANN {
            return Err(format!(
                "ec_diskann metadata format mismatch: got {format_version}, expected {INDEX_FORMAT_V3_DISKANN}"
            ));
        }
        let metadata = VamanaMetadataPage::decode(metadata_bytes)?;
        Ok((metadata, page_size))
    })();
    unsafe { pg_sys::UnlockReleaseBuffer(metadata_buffer) };
    let (metadata, page_size) = metadata_result?;

    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut chain = DataPageChain::new(page_size);
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
                "ec_diskann beginscan could not open data block {block_number}"
            ));
        }
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_result = (|| -> Result<(), String> {
            let page = unsafe { pg_sys::BufferGetPage(buffer) };
            let max_offset = unsafe { pg_sys::PageGetMaxOffsetNumber(page) };
            for offset in 1..=max_offset {
                let item_id = unsafe { pg_sys::PageGetItemId(page, offset) };
                if item_id.is_null() {
                    return Err(format!(
                        "ec_diskann data block {block_number} returned a null item id at offset {offset}"
                    ));
                }
                let item_id_ref = unsafe { &*item_id };
                if item_id_ref.lp_flags() == 0 {
                    continue;
                }
                let tuple_len = item_id_ref.lp_len() as usize;
                let tuple_ptr = unsafe { pg_sys::PageGetItem(page, item_id) }.cast::<u8>();
                if tuple_ptr.is_null() {
                    return Err(format!(
                        "ec_diskann data block {block_number} returned a null tuple pointer at offset {offset}"
                    ));
                }
                let tuple_bytes =
                    unsafe { slice::from_raw_parts(tuple_ptr.cast_const(), tuple_len) }.to_vec();
                chain.insert_raw_tuple(tuple_bytes)?;
            }
            Ok(())
        })();
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        page_result?;
    }

    Ok((metadata, chain))
}

pub(super) unsafe fn resolve_scan_heap_relation(
    scan: pg_sys::IndexScanDesc,
) -> Result<(pg_sys::Relation, bool), String> {
    if unsafe { !(*scan).heapRelation.is_null() } {
        return Ok((unsafe { (*scan).heapRelation }, false));
    }

    let heap_oid = unsafe { pg_sys::IndexGetRelation((*(*scan).indexRelation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        return Err("ec_diskann scan could not resolve heap relation".into());
    }
    Ok((
        unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) },
        true,
    ))
}

pub(super) unsafe fn resolve_scan_snapshot(
    scan: pg_sys::IndexScanDesc,
) -> Result<(pg_sys::Snapshot, bool), String> {
    if unsafe { !(*scan).xs_snapshot.is_null() } {
        return Ok((unsafe { (*scan).xs_snapshot }, false));
    }

    let active_snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if !active_snapshot.is_null() {
        return Ok((active_snapshot, false));
    }

    let registered_snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot()) };
    if registered_snapshot.is_null() {
        return Err("ec_diskann scan could not resolve an active snapshot".into());
    }
    Ok((registered_snapshot, true))
}

pub(super) unsafe fn allocate_heap_slot(
    heap_relation: pg_sys::Relation,
) -> Result<*mut pg_sys::TupleTableSlot, String> {
    let slot = unsafe {
        pg_sys::MakeSingleTupleTableSlot(
            (*heap_relation).rd_att,
            pg_sys::table_slot_callbacks(heap_relation),
        )
    };
    if slot.is_null() {
        return Err("ec_diskann scan failed to allocate a heap tuple slot".into());
    }
    Ok(slot)
}

pub(super) unsafe fn fetch_heap_row_version(
    heap_relation: pg_sys::Relation,
    heap_tid: ItemPointer,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
) -> Result<(), String> {
    let mut tid = pg_sys::ItemPointerData::default();
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut tid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
        pg_sys::ExecClearTuple(slot);
    }
    let fetched =
        unsafe { pg_sys::table_tuple_fetch_row_version(heap_relation, &mut tid, snapshot, slot) };
    if !fetched {
        return Err(format!(
            "ec_diskann scan could not fetch heap tuple at ({},{})",
            heap_tid.block_number, heap_tid.offset_number
        ));
    }
    Ok(())
}

pub(super) unsafe fn required_slot_datum(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    label: &str,
) -> Result<pg_sys::Datum, String> {
    if unsafe { (*slot).tts_nvalid } < attnum as i16 {
        unsafe { pg_sys::slot_getsomeattrs_int(slot, attnum) };
    }
    let attr_index = usize::try_from(attnum - 1).expect("attribute number should be positive");
    if unsafe { *(*slot).tts_isnull.add(attr_index) } {
        return Err(format!("ec_diskann does not support NULL {label}"));
    }
    Ok(unsafe { *(*slot).tts_values.add(attr_index) })
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer) -> Result<ItemPointer, String> {
    if tid.is_null() {
        return Err("ec_diskann scan received a null heap tid".into());
    }
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    Ok(ItemPointer {
        block_number,
        offset_number,
    })
}

pub(super) unsafe fn release_owned_scan_heap_state(
    heap_relation: pg_sys::Relation,
    heap_relation_owned: bool,
    snapshot: pg_sys::Snapshot,
    snapshot_owned: bool,
    slot: *mut pg_sys::TupleTableSlot,
) {
    if !slot.is_null() {
        unsafe { pg_sys::ExecDropSingleTupleTableSlot(slot) };
    }
    if snapshot_owned && !snapshot.is_null() {
        unsafe { pg_sys::UnregisterSnapshot(snapshot) };
    }
    if heap_relation_owned && !heap_relation.is_null() {
        unsafe { pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }
}

pub(super) fn set_scan_heap_tid(scan: pg_sys::IndexScanDesc, heap_tid: ItemPointer) {
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut (*scan).xs_heaptid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
    }
}
