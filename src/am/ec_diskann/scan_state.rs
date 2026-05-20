use std::slice;

use pgrx::{itemptr::item_pointer_get_both, pg_sys};

use crate::am::common::heap_slot;
use crate::storage::{
    buffer_guard::LockedBufferGuard,
    page::{DataPageChain, ItemPointer, FIRST_DATA_BLOCK_NUMBER},
    relation_guard::HeapRelationGuard,
    snapshot_guard::RegisteredSnapshotGuard,
};

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
    pub(super) query_binary_words: Vec<u64>,
    pub(super) visited: VisitedState,
    pub(super) result_buf: Vec<ScanResult>,
    pub(super) result_cursor: usize,
    pub(super) rescan_called: bool,
    pub(super) top_k: usize,
    pub(super) list_size: usize,
    pub(super) rerank_budget: usize,
}

pub(super) struct ResolvedScanHeapRelation {
    relation: pg_sys::Relation,
    _owned: Option<HeapRelationGuard>,
}

impl ResolvedScanHeapRelation {
    fn borrowed(relation: pg_sys::Relation) -> Self {
        Self {
            relation,
            _owned: None,
        }
    }

    fn owned(guard: HeapRelationGuard) -> Self {
        let relation = guard.as_ptr();
        Self {
            relation,
            _owned: Some(guard),
        }
    }

    pub(super) fn as_ptr(&self) -> pg_sys::Relation {
        self.relation
    }
}

pub(super) struct ResolvedScanSnapshot {
    snapshot: pg_sys::Snapshot,
    _owned: Option<RegisteredSnapshotGuard>,
}

impl ResolvedScanSnapshot {
    fn borrowed(snapshot: pg_sys::Snapshot) -> Self {
        Self {
            snapshot,
            _owned: None,
        }
    }

    fn owned(guard: RegisteredSnapshotGuard) -> Self {
        let snapshot = guard.as_ptr();
        Self {
            snapshot,
            _owned: Some(guard),
        }
    }

    pub(super) fn as_ptr(&self) -> pg_sys::Snapshot {
        self.snapshot
    }
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
            query_binary_words: Vec::new(),
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
    let metadata_result: Result<(VamanaMetadataPage, usize), String> = {
        // SAFETY: `index_relation` is a live DISKANN index relation during
        // beginscan, and block 0 is the metadata page read under a share lock.
        let metadata_buffer = LockedBufferGuard::read_main(
            index_relation,
            0,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| "ec_diskann beginscan could not open metadata page".to_owned())?;
        let page = metadata_buffer.page();
        let page_size = metadata_buffer.page_size();
        // SAFETY: `page` comes from the locked metadata buffer and remains
        // pinned while the special area is inspected.
        let special_size = unsafe { pg_sys::PageGetSpecialSize(page) as usize };
        if special_size < VAMANA_METADATA_BYTES {
            return Err(format!(
                "ec_diskann metadata page special area too small: got {special_size}, expected at least {VAMANA_METADATA_BYTES}"
            ));
        }
        // SAFETY: The special area is at least `VAMANA_METADATA_BYTES`, so the
        // special pointer can be viewed as the serialized metadata payload.
        let metadata_ptr = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
        // SAFETY: `metadata_ptr` points into the locked metadata page special
        // area, whose size was checked above.
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
    };
    let (metadata, page_size) = metadata_result?;

    // SAFETY: `index_relation` is live while beginscan materializes the index
    // and PostgreSQL can report the current main-fork block count.
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut chain = DataPageChain::new(page_size);
    for block_number in FIRST_DATA_BLOCK_NUMBER..block_count {
        let page_result: Result<(), String> = {
            // SAFETY: `block_number` is within the reported main-fork block
            // count and the buffer guard pins/share-locks the data page.
            let buffer = LockedBufferGuard::read_main(
                index_relation,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
            .ok_or_else(|| {
                format!("ec_diskann beginscan could not open data block {block_number}")
            })?;
            let page = buffer.page();
            let data_page_size = buffer.page_size();
            // SAFETY: `page` comes from the locked data-page buffer and remains
            // pinned while line pointers are enumerated.
            let max_offset = unsafe { pg_sys::PageGetMaxOffsetNumber(page) };
            for offset in 1..=max_offset {
                // SAFETY: `offset` is within the page's max offset, and the
                // helper validates item id and tuple bounds before copying.
                if let Some(tuple_bytes) = unsafe {
                    copy_data_page_tuple_bytes(page, data_page_size, block_number, offset)?
                } {
                    chain.insert_raw_tuple(tuple_bytes)?;
                }
            }
            Ok(())
        };
        page_result?;
    }

    Ok((metadata, chain))
}

unsafe fn copy_data_page_tuple_bytes(
    page: pg_sys::Page,
    page_size: usize,
    block_number: pg_sys::BlockNumber,
    offset: pg_sys::OffsetNumber,
) -> Result<Option<Vec<u8>>, String> {
    // SAFETY: Callers pass an offset within the locked page's max offset.
    let item_id = unsafe { pg_sys::PageGetItemId(page, offset) };
    if item_id.is_null() {
        return Err(format!(
            "ec_diskann data block {block_number} returned a null item id at offset {offset}"
        ));
    }
    // SAFETY: `item_id` was checked non-null and points into the locked page's
    // line pointer array.
    let item_id_ref = unsafe { &*item_id };
    if item_id_ref.lp_flags() == 0 {
        return Ok(None);
    }

    let tuple_offset = item_id_ref.lp_off() as usize;
    let tuple_len = item_id_ref.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        return Err(format!(
            "ec_diskann data block {block_number} tuple bounds exceed page at offset {offset}"
        ));
    }

    // SAFETY: The item id is valid and its byte bounds were checked against the
    // locked page size before retrieving the tuple pointer.
    let tuple_ptr = unsafe { pg_sys::PageGetItem(page, item_id) }.cast::<u8>();
    if tuple_ptr.is_null() {
        return Err(format!(
            "ec_diskann data block {block_number} returned a null tuple pointer at offset {offset}"
        ));
    }
    // SAFETY: `tuple_ptr` is non-null and the validated tuple length is copied
    // into owned memory before the page lock is released.
    let tuple_bytes = unsafe { slice::from_raw_parts(tuple_ptr.cast_const(), tuple_len) }.to_vec();
    Ok(Some(tuple_bytes))
}

pub(super) unsafe fn resolve_scan_heap_relation(
    scan: pg_sys::IndexScanDesc,
) -> Result<ResolvedScanHeapRelation, String> {
    // SAFETY: `scan` is the live IndexScanDesc supplied by PostgreSQL; when
    // heapRelation is present, PostgreSQL owns it for the scan duration.
    if unsafe { !(*scan).heapRelation.is_null() } {
        // SAFETY: The heapRelation null check above proved this scan-owned
        // relation pointer can be borrowed.
        return Ok(ResolvedScanHeapRelation::borrowed(unsafe {
            (*scan).heapRelation
        }));
    }

    // SAFETY: The scan owns a live index relation descriptor, whose OID can be
    // resolved to the heap relation when PostgreSQL did not attach one.
    let heap_oid = unsafe { pg_sys::IndexGetRelation((*(*scan).indexRelation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        return Err("ec_diskann scan could not resolve heap relation".into());
    }
    HeapRelationGuard::try_access_share(heap_oid)
        .map(ResolvedScanHeapRelation::owned)
        .ok_or_else(|| "ec_diskann scan could not open heap relation".into())
}

pub(super) unsafe fn resolve_scan_snapshot(
    scan: pg_sys::IndexScanDesc,
) -> Result<ResolvedScanSnapshot, String> {
    // SAFETY: `scan` is the live IndexScanDesc supplied by PostgreSQL; when
    // xs_snapshot is set, PostgreSQL owns it for the scan duration.
    if unsafe { !(*scan).xs_snapshot.is_null() } {
        // SAFETY: The xs_snapshot null check above proved this scan-owned
        // snapshot pointer can be borrowed.
        return Ok(ResolvedScanSnapshot::borrowed(unsafe {
            (*scan).xs_snapshot
        }));
    }

    // SAFETY: PostgreSQL exposes the current active snapshot for this backend;
    // a null return is handled by registering our own snapshot below.
    let active_snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if !active_snapshot.is_null() {
        return Ok(ResolvedScanSnapshot::borrowed(active_snapshot));
    }

    RegisteredSnapshotGuard::latest()
        .map(ResolvedScanSnapshot::owned)
        .ok_or_else(|| "ec_diskann scan could not resolve an active snapshot".into())
}

pub(super) unsafe fn fetch_heap_row_version(
    heap_relation: pg_sys::Relation,
    heap_tid: ItemPointer,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
) -> Result<(), String> {
    // SAFETY: caller owns the heap relation, snapshot, and tuple slot for this
    // scan callback; the common helper owns slot clearing and TID fetch.
    let fetched = unsafe {
        heap_slot::fetch_heap_row_version(heap_relation, heap_tid, snapshot, slot, "ec_diskann")?
    };
    if !fetched {
        return Err(format!(
            "ec_diskann scan could not fetch heap tuple at ({},{})",
            heap_tid.block_number, heap_tid.offset_number
        ));
    }
    Ok(())
}

pub(super) fn fetch_heap_row_version_with_reader(
    reader: &mut heap_slot::HeapSlotReader<'_>,
    heap_tid: ItemPointer,
) -> Result<(), String> {
    if !reader.fetch_row_version(heap_tid)? {
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
    // SAFETY: caller owns a live TupleTableSlot and attnum was resolved from
    // relation metadata for the heap source column.
    unsafe { heap_slot::required_slot_datum(slot, attnum, "ec_diskann", label) }
}

pub(super) fn required_slot_datum_with_reader(
    reader: &mut heap_slot::HeapSlotReader<'_>,
    attnum: i32,
    label: &str,
) -> Result<pg_sys::Datum, String> {
    reader.required_datum(attnum, label)
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer) -> Result<ItemPointer, String> {
    if tid.is_null() {
        return Err("ec_diskann scan received a null heap tid".into());
    }
    // SAFETY: `tid` was checked non-null and points at PostgreSQL ItemPointer
    // storage valid for this callback/scan step.
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    Ok(ItemPointer {
        block_number,
        offset_number,
    })
}

pub(super) fn set_scan_heap_tid(scan: pg_sys::IndexScanDesc, heap_tid: ItemPointer) {
    // SAFETY: `scan` is the live IndexScanDesc currently returning a tuple, and
    // `xs_heaptid` is PostgreSQL-owned output storage for the heap TID.
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut (*scan).xs_heaptid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
    }
}
