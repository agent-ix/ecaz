use std::{ptr::NonNull, slice};

use pgrx::pg_sys;

use crate::am::common::heap_slot;
use crate::storage::{
    buffer_guard::{LockedBufferGuard, LockedPageTupleVisit},
    page::{DataPageChain, ItemPointer, FIRST_DATA_BLOCK_NUMBER},
    relation::RelationHandle,
    relation_guard::HeapRelationGuard,
    snapshot_guard::RegisteredSnapshotGuard,
};

use super::{
    options::TqDiskannOptions,
    page::{
        VamanaMetadataPage, INDEX_FORMAT_V3_DISKANN, PAYLOAD_FLAG_BINARY_SIDECAR,
        VAMANA_METADATA_BYTES,
    },
    reader::{GraphReader, VisitedState},
    scan::ScanResult,
    tuple::{VamanaNodeTuple, TQ_VAMANA_NODE_TAG},
};

#[derive(Debug)]
pub(super) struct DiskannScanOpaque {
    pub(super) metadata: VamanaMetadataPage,
    pub(super) chain: Option<DataPageChain>,
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
        options: TqDiskannOptions,
    ) -> Result<Self, String> {
        let scan_tuning = super::options::resolve_scan_tuning(&options);
        Ok(Self {
            metadata,
            chain: None,
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
    super::quantizer::metadata_search_code_len(metadata)
        .expect("ec_diskann metadata should carry a supported search codec")
}

pub(super) unsafe fn materialize_chain_from_index(
    index_relation: pg_sys::Relation,
) -> Result<(VamanaMetadataPage, DataPageChain), String> {
    let handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_diskann scan materialization needs a valid index relation".to_owned())?;
    materialize_chain_from_index_handle(handle)
}

pub(super) unsafe fn read_metadata_from_index(
    index_relation: pg_sys::Relation,
) -> Result<(VamanaMetadataPage, usize), String> {
    let handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_diskann metadata read needs a valid index relation".to_owned())?;
    read_metadata_from_index_handle(handle)
}

pub(super) fn read_metadata_from_index_handle(
    handle: RelationHandle,
) -> Result<(VamanaMetadataPage, usize), String> {
    let metadata_buffer = LockedBufferGuard::read_main_handle(
        handle,
        0,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    )
    .ok_or_else(|| "ec_diskann beginscan could not open metadata page".to_owned())?;
    let page = metadata_buffer.page();
    let page_size = metadata_buffer.page_size();
    // SAFETY: `page` comes from the locked metadata buffer and remains
    // pinned while the special area is inspected; the special area is at
    // least `VAMANA_METADATA_BYTES`, so the special pointer is viewed as
    // the serialized metadata payload.
    let metadata_bytes = unsafe {
        let special_size = pg_sys::PageGetSpecialSize(page) as usize;
        if special_size < VAMANA_METADATA_BYTES {
            return Err(format!(
                "ec_diskann metadata page special area too small: got {special_size}, expected at least {VAMANA_METADATA_BYTES}"
            ));
        }
        let metadata_ptr = pg_sys::PageGetSpecialPointer(page).cast::<u8>();
        slice::from_raw_parts(metadata_ptr.cast_const(), VAMANA_METADATA_BYTES)
    };
    let format_version = u16::from_le_bytes(metadata_bytes[0..2].try_into().expect("format bytes"));
    if format_version != INDEX_FORMAT_V3_DISKANN {
        return Err(format!(
            "ec_diskann metadata format mismatch: got {format_version}, expected {INDEX_FORMAT_V3_DISKANN}"
        ));
    }
    let metadata = VamanaMetadataPage::decode(metadata_bytes)?;
    Ok((metadata, page_size))
}

pub(super) fn materialize_chain_from_index_handle(
    handle: RelationHandle,
) -> Result<(VamanaMetadataPage, DataPageChain), String> {
    let (metadata, page_size) = read_metadata_from_index_handle(handle)?;

    let block_count = crate::storage::relation::main_fork_block_count_handle(handle);
    let mut chain = DataPageChain::new(page_size);
    for block_number in FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = LockedBufferGuard::read_main_handle(
            handle,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| format!("ec_diskann beginscan could not open data block {block_number}"))?;
        let max_offset = buffer.max_offset_number();
        for offset in 1..=max_offset {
            let tid = ItemPointer {
                block_number,
                offset_number: offset,
            };
            let visit = buffer.visit_tuple_bytes(tid, "ec_diskann data block", |tuple_bytes| {
                Ok(tuple_bytes.to_vec())
            })?;
            if let LockedPageTupleVisit::Present(tuple_bytes) = visit {
                chain.insert_raw_tuple(tuple_bytes)?;
            }
        }
    }

    Ok((metadata, chain))
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RelationGraphReader {
    handle: RelationHandle,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
}

impl RelationGraphReader {
    pub(super) fn new(
        handle: RelationHandle,
        graph_degree_r: u16,
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Self {
        Self {
            handle,
            graph_degree_r,
            binary_word_count,
            search_code_len,
        }
    }

    pub(super) fn handle(&self) -> RelationHandle {
        self.handle
    }

    pub(super) fn read_tuple_bytes<R>(
        &self,
        tid: ItemPointer,
        tuple_kind: &str,
        visit: impl FnOnce(&[u8]) -> Result<R, String>,
    ) -> Result<R, String> {
        let buffer = LockedBufferGuard::read_main_handle(
            self.handle,
            tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| {
            format!(
                "ec_diskann scan could not open data block {}",
                tid.block_number
            )
        })?;
        match buffer.visit_tuple_bytes(tid, tuple_kind, visit)? {
            LockedPageTupleVisit::Present(value) => Ok(value),
            LockedPageTupleVisit::Unused => Err(format!(
                "{tuple_kind} tuple ({},{}) is unused",
                tid.block_number, tid.offset_number
            )),
        }
    }
}

impl GraphReader for RelationGraphReader {
    fn read_node(&self, tid: ItemPointer) -> Result<VamanaNodeTuple, String> {
        self.read_tuple_bytes(tid, "ec_diskann scan node", |raw| {
            VamanaNodeTuple::decode(
                raw,
                self.graph_degree_r,
                self.binary_word_count,
                self.search_code_len,
            )
        })
    }

    /// Task 168 Phase 3: prefetch the batch's distinct blocks through the
    /// read-stream helper, then decode block-grouped so each shared page
    /// is pinned and share-locked once per round instead of once per node.
    /// Output order matches `tids` order.
    fn read_nodes(
        &self,
        tids: &[ItemPointer],
        out: &mut Vec<VamanaNodeTuple>,
    ) -> Result<(), String> {
        if tids.len() <= 1 {
            for &tid in tids {
                out.push(self.read_node(tid)?);
            }
            return Ok(());
        }

        let mut order: Vec<usize> = (0..tids.len()).collect();
        order.sort_by_key(|&i| (tids[i].block_number, tids[i].offset_number));

        let mut blocks: Vec<pg_sys::BlockNumber> =
            order.iter().map(|&i| tids[i].block_number).collect();
        blocks.dedup();
        crate::am::stream::prefetch_relation_blocks(
            self.handle.as_ptr(),
            blocks,
            "ec_diskann graph batch",
        );

        let mut decoded: Vec<Option<VamanaNodeTuple>> = Vec::new();
        decoded.resize_with(tids.len(), || None);
        let mut position = 0;
        while position < order.len() {
            let block_number = tids[order[position]].block_number;
            let group_end = order[position..]
                .iter()
                .position(|&i| tids[i].block_number != block_number)
                .map_or(order.len(), |offset| position + offset);
            let buffer = LockedBufferGuard::read_main_handle(
                self.handle,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
            .ok_or_else(|| {
                format!("ec_diskann scan could not open data block {block_number}")
            })?;
            for &index in &order[position..group_end] {
                let tid = tids[index];
                let visit = buffer.visit_tuple_bytes(tid, "ec_diskann scan node", |raw| {
                    VamanaNodeTuple::decode(
                        raw,
                        self.graph_degree_r,
                        self.binary_word_count,
                        self.search_code_len,
                    )
                })?;
                match visit {
                    LockedPageTupleVisit::Present(tuple) => decoded[index] = Some(tuple),
                    LockedPageTupleVisit::Unused => {
                        return Err(format!(
                            "ec_diskann scan node tuple ({},{}) is unused",
                            tid.block_number, tid.offset_number
                        ))
                    }
                }
            }
            position = group_end;
        }

        out.reserve(decoded.len());
        for tuple in decoded {
            out.push(tuple.expect("every batched tid decodes exactly once"));
        }
        Ok(())
    }

    fn first_live_tid(&self) -> Result<Option<ItemPointer>, String> {
        let block_count = crate::storage::relation::main_fork_block_count_handle(self.handle);
        for block_number in FIRST_DATA_BLOCK_NUMBER..block_count {
            let buffer = LockedBufferGuard::read_main_handle(
                self.handle,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
            .ok_or_else(|| format!("ec_diskann scan could not open data block {block_number}"))?;
            let max_offset = buffer.max_offset_number();
            for offset_number in 1..=max_offset {
                let tid = ItemPointer {
                    block_number,
                    offset_number,
                };
                let visit = buffer.visit_tuple_bytes(tid, "ec_diskann scan node", |raw| {
                    if raw.first().copied() != Some(TQ_VAMANA_NODE_TAG) {
                        return Ok(None);
                    }
                    VamanaNodeTuple::decode(
                        raw,
                        self.graph_degree_r,
                        self.binary_word_count,
                        self.search_code_len,
                    )
                    .map(Some)
                })?;
                if let LockedPageTupleVisit::Present(Some(tuple)) = visit {
                    if tuple.is_live() {
                        return Ok(Some(tid));
                    }
                }
            }
        }
        Ok(None)
    }
}

pub(super) struct DiskannScanDescView<'a> {
    scan: &'a pg_sys::IndexScanDescData,
}

impl<'a> DiskannScanDescView<'a> {
    pub(super) unsafe fn from_raw(scan: pg_sys::IndexScanDesc, context: &str) -> Self {
        let scan = unsafe { scan.as_ref() }
            .unwrap_or_else(|| pgrx::error!("{context} received a null scan descriptor"));
        Self { scan }
    }

    pub(super) fn index_relation(&self) -> pg_sys::Relation {
        self.scan.indexRelation
    }

    pub(super) fn resolve_heap_relation(&self) -> Result<ResolvedScanHeapRelation, String> {
        if !self.scan.heapRelation.is_null() {
            return Ok(ResolvedScanHeapRelation::borrowed(self.scan.heapRelation));
        }

        let index_relation = std::ptr::NonNull::new(self.index_relation())
            .ok_or_else(|| "ec_diskann scan received null index relation".to_owned())?;
        let heap_oid = crate::storage::relation::index_heap_relation_oid_handle(index_relation);
        if heap_oid == pg_sys::InvalidOid {
            return Err("ec_diskann scan could not resolve heap relation".into());
        }
        HeapRelationGuard::try_access_share(heap_oid)
            .map(ResolvedScanHeapRelation::owned)
            .ok_or_else(|| "ec_diskann scan could not open heap relation".into())
    }

    pub(super) fn resolve_snapshot(&self) -> Result<ResolvedScanSnapshot, String> {
        if !self.scan.xs_snapshot.is_null() {
            return Ok(ResolvedScanSnapshot::borrowed(self.scan.xs_snapshot));
        }

        if let Some(active_snapshot) = crate::storage::snapshot_guard::active_snapshot() {
            return Ok(ResolvedScanSnapshot::borrowed(active_snapshot));
        }

        RegisteredSnapshotGuard::latest()
            .map(ResolvedScanSnapshot::owned)
            .ok_or_else(|| "ec_diskann scan could not resolve an active snapshot".into())
    }
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

pub(super) fn required_slot_datum_with_reader(
    reader: &mut heap_slot::HeapSlotReader<'_>,
    attnum: i32,
    label: &str,
) -> Result<pg_sys::Datum, String> {
    reader.required_datum(attnum, label)
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
