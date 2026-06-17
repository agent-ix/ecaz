//! ec_ivf page layout: metadata, centroid, directory, and posting-list codecs.

#[cfg(any(feature = "pg17", feature = "pg18"))]
use std::collections::BTreeSet;
use std::collections::HashMap;
#[cfg(any(feature = "pg17", feature = "pg18"))]
use std::marker::PhantomData;
use std::mem::size_of;
#[cfg(any(feature = "pg17", feature = "pg18"))]
use std::ptr::{self, NonNull};
use std::sync::{Mutex, OnceLock};

#[cfg(any(feature = "pg17", feature = "pg18"))]
use pgrx::pg_sys;
#[cfg(not(any(feature = "pg17", feature = "pg18")))]
mod pg_sys {
    pub(super) type BlockNumber = u32;
    pub(super) type Oid = u32;

    #[repr(C)]
    pub(super) struct PageHeaderData {
        pub(super) pd_lower: u16,
    }

    #[repr(C)]
    pub(super) struct ItemIdData {
        pub(super) raw: u32,
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
use super::options::{EcIvfOptions, RerankMode, StorageFormat};
#[cfg(any(feature = "pg17", feature = "pg18"))]
use super::P_NEW;
#[cfg(not(any(feature = "pg17", feature = "pg18")))]
const P_NEW: pg_sys::BlockNumber = u32::MAX;
#[cfg(any(feature = "pg17", feature = "pg18"))]
use crate::storage::page::{align_up, raw_tuple_storage_bytes, ALIGNMENT_BYTES, PAGE_HEADER_BYTES};
use crate::storage::page::{
    aligned_tuple_bytes, usable_page_bytes, DataPage, DataPageChain, ItemPointer,
    HEAPTID_INLINE_CAPACITY, ITEM_POINTER_BYTES,
};
#[cfg(any(feature = "pg17", feature = "pg18"))]
use crate::storage::relation::{main_fork_block_count_handle, relation_oid_handle, RelationHandle};
#[cfg(any(feature = "pg17", feature = "pg18"))]
use crate::storage::{buffer_guard::LockedBufferGuard, wal};

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) const METADATA_BLOCK_NUMBER: pg_sys::BlockNumber = 0;
#[cfg(not(any(feature = "pg17", feature = "pg18")))]
pub(super) const METADATA_BLOCK_NUMBER: u32 = 0;
#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) const FIRST_DATA_BLOCK_NUMBER: pg_sys::BlockNumber = 1;
#[cfg(not(any(feature = "pg17", feature = "pg18")))]
pub(super) const FIRST_DATA_BLOCK_NUMBER: u32 = 1;
// v1 = original format
// v2 = stores `quant_bits` (RaBitQ per-dim code width) at byte 34.
//      Indexes written by v1 read back here as `quant_bits = 0` which
//      the decoder coerces to the default of 4.
pub const EC_IVF_INDEX_FORMAT_VERSION: u16 = 2;
pub(super) const EC_IVF_INDEX_FORMAT_VERSION_MIN: u16 = 1;
pub(super) const INDEX_FORMAT_VERSION: u16 = EC_IVF_INDEX_FORMAT_VERSION;

pub const EC_IVF_METADATA_MAGIC: u32 = 0x5649_4345; // "ECIV" as little-endian bytes.
pub const EC_IVF_METADATA_BYTES: usize = 80;
pub const EC_IVF_METADATA_MAGIC_OFFSET: usize = 0;
pub const EC_IVF_METADATA_FORMAT_VERSION_OFFSET: usize = 4;
pub const EC_IVF_METADATA_DIMENSIONS_OFFSET: usize = 6;
pub const EC_IVF_METADATA_NLISTS_OFFSET: usize = 8;
pub const EC_IVF_METADATA_NPROBE_OFFSET: usize = 12;
pub const EC_IVF_METADATA_TRAINING_SAMPLE_ROWS_OFFSET: usize = 16;
pub const EC_IVF_METADATA_TRAINING_VERSION_OFFSET: usize = 20;
pub const EC_IVF_METADATA_SEED_OFFSET: usize = 24;
pub const EC_IVF_METADATA_STORAGE_FORMAT_OFFSET: usize = 32;
pub const EC_IVF_METADATA_RERANK_OFFSET: usize = 33;
pub const EC_IVF_METADATA_CENTROID_HEAD_OFFSET: usize = 36;
pub const EC_IVF_METADATA_DIRECTORY_HEAD_OFFSET: usize = 42;
pub const EC_IVF_METADATA_TOTAL_LIVE_TUPLES_OFFSET: usize = 48;
pub const EC_IVF_METADATA_TOTAL_DEAD_TUPLES_OFFSET: usize = 56;
pub const EC_IVF_METADATA_INSERTED_SINCE_BUILD_OFFSET: usize = 64;
pub const EC_IVF_METADATA_PQ_CODEBOOK_HEAD_OFFSET: usize = 72;
pub const EC_IVF_METADATA_PQ_GROUP_SIZE_OFFSET: usize = 78;

pub const EC_IVF_BLOCK_REF_BYTES: usize = 4;
pub const EC_IVF_BLOCK_REF_BLOCK_NUMBER_OFFSET: usize = 0;
pub const EC_IVF_CENTROID_TAG_OFFSET: usize = 0;
pub const EC_IVF_CENTROID_LIST_ID_OFFSET: usize = 1;
pub const EC_IVF_CENTROID_DIMENSIONS_OFFSET: usize = 5;
pub const EC_IVF_CENTROID_VALUES_OFFSET: usize = 7;
pub const EC_IVF_LIST_DIRECTORY_TAG_OFFSET: usize = 0;
pub const EC_IVF_LIST_DIRECTORY_LIST_ID_OFFSET: usize = 1;
pub const EC_IVF_LIST_DIRECTORY_HEAD_BLOCK_OFFSET: usize = 5;
pub const EC_IVF_LIST_DIRECTORY_TAIL_BLOCK_OFFSET: usize = 9;
pub const EC_IVF_LIST_DIRECTORY_LIVE_COUNT_OFFSET: usize = 13;
pub const EC_IVF_LIST_DIRECTORY_DEAD_COUNT_OFFSET: usize = 21;
pub const EC_IVF_LIST_DIRECTORY_INSERTED_SINCE_BUILD_OFFSET: usize = 29;
pub const EC_IVF_LIST_DIRECTORY_BYTES: usize = 37;
pub const EC_IVF_POSTING_TAG_OFFSET: usize = 0;
pub const EC_IVF_POSTING_LIST_ID_OFFSET: usize = 1;
pub const EC_IVF_POSTING_FLAGS_OFFSET: usize = 5;
pub const EC_IVF_POSTING_HEAPTID_COUNT_OFFSET: usize = 6;
pub const EC_IVF_POSTING_HEAPTIDS_OFFSET: usize = 7;
pub const EC_IVF_POSTING_GAMMA_OFFSET: usize =
    EC_IVF_POSTING_HEAPTIDS_OFFSET + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES;
pub const EC_IVF_POSTING_RERANK_TID_OFFSET: usize = EC_IVF_POSTING_GAMMA_OFFSET + size_of::<f32>();
pub const EC_IVF_POSTING_PAYLOAD_OFFSET: usize =
    EC_IVF_POSTING_RERANK_TID_OFFSET + ITEM_POINTER_BYTES;
pub const EC_IVF_PQ_CODEBOOK_TAG_OFFSET: usize = 0;
pub const EC_IVF_PQ_CODEBOOK_GROUP_INDEX_OFFSET: usize = 1;
pub const EC_IVF_PQ_CODEBOOK_NEXT_TID_OFFSET: usize = 3;
pub const EC_IVF_PQ_CODEBOOK_CENTROIDS_OFFSET: usize =
    EC_IVF_PQ_CODEBOOK_NEXT_TID_OFFSET + ITEM_POINTER_BYTES;

const METADATA_MAGIC: u32 = EC_IVF_METADATA_MAGIC;
const METADATA_BYTES: usize = EC_IVF_METADATA_BYTES;
const BLOCK_REF_BYTES: usize = EC_IVF_BLOCK_REF_BYTES;
const IVF_CENTROID_TAG: u8 = 0x21;
const IVF_LIST_DIRECTORY_TAG: u8 = 0x22;
const IVF_POSTING_TAG: u8 = 0x23;
const IVF_PQ_CODEBOOK_TAG: u8 = 0x24;
const IVF_DENSE_POSTING_BLOCK_TAG: u8 = 0x25;
const IVF_DENSE_POSTING_PACKED_SEGMENT_TAG: u8 = 0x26;
const IVF_DENSE_POSTING_PACKED_CONTINUATION_TAG: u8 = 0x27;
const IVF_DENSE_POSTING_ALIGNED_BLOCK_TAG: u8 = 0x28;
const IVF_COLUMNAR_FROZEN_LIST_HEADER_TAG: u8 = 0x29;
const COLUMNAR_FROZEN_LIST_HEADER_VERSION: u8 = 1;
const POSTING_FLAG_DELETED: u8 = 0b0000_0001;
const POSTING_FIXED_BYTES: usize = EC_IVF_POSTING_PAYLOAD_OFFSET;
const DENSE_POSTING_BLOCK_HEADER_BYTES: usize = 16;
const DENSE_POSTING_PACKED_SEGMENT_HEADER_BYTES: usize = 28;
const DENSE_POSTING_PACKED_CONTINUATION_HEADER_BYTES: usize = 24;
const COLUMNAR_FROZEN_LIST_HEADER_BYTES: usize = 58;

#[cfg(not(any(feature = "pg17", feature = "pg18")))]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageFormat {
    Auto = 0,
    TurboQuant = 1,
    PqFastScan = 2,
    RaBitQ = 3,
}

#[cfg(not(any(feature = "pg17", feature = "pg18")))]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RerankMode {
    Auto = 0,
    Off = 1,
    HeapF32 = 2,
    SourceColumn = 3,
}

#[cfg(not(any(feature = "pg17", feature = "pg18")))]
impl RerankMode {
    pub(super) fn v1_effective(self) -> Self {
        match self {
            Self::Auto => Self::Off,
            other => other,
        }
    }
}

#[cfg(not(any(feature = "pg17", feature = "pg18")))]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(super) struct EcIvfOptions {
    pub(super) nlists: i32,
    pub(super) nprobe: i32,
    pub(super) rerank_width: i32,
    pub(super) training_sample_rows: i32,
    pub(super) seed: i32,
    pub(super) pq_group_size: i32,
    pub(super) posting_slack_percent: i32,
    pub(super) storage_format: StorageFormat,
    pub(super) rerank: RerankMode,
    pub(super) quant_bits: u8,
    pub(super) dense_posting_blocks: bool,
    pub(super) dense_posting_pack_pages: i32,
    pub(super) dense_posting_typed_layout: bool,
    pub(super) columnar_frozen_lists: bool,
}

#[cfg(not(any(feature = "pg17", feature = "pg18")))]
impl EcIvfOptions {
    fn effective_quant_bits(&self) -> u8 {
        self.quant_bits
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
enum PageTupleVisit<R> {
    Unused,
    Present(R),
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
#[derive(Clone, Copy)]
struct IvfPageRelation<'a> {
    relation: RelationHandle,
    _relation: PhantomData<&'a pg_sys::RelationData>,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
impl<'a> IvfPageRelation<'a> {
    fn new(relation: RelationHandle) -> Self {
        Self {
            relation,
            _relation: PhantomData,
        }
    }

    fn raw(self) -> pg_sys::Relation {
        self.relation.as_ptr()
    }

    fn relid(self) -> pg_sys::Oid {
        relation_oid_handle(self.relation)
    }

    fn number_of_blocks(self) -> pg_sys::BlockNumber {
        main_fork_block_count_handle(self.relation)
    }

    fn page_with_free_space(self, required_space: usize) -> pg_sys::BlockNumber {
        // SAFETY: this view is constructed only for a live IVF index relation;
        // required_space is derived from the tuple size that will be inserted.
        unsafe { pg_sys::GetPageWithFreeSpace(self.relation.as_ptr(), required_space) }
    }

    fn read_main(
        self,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
        lockmode: i32,
    ) -> Option<LockedBufferGuard> {
        LockedBufferGuard::read_main_handle(self.relation, block_number, mode, lockmode)
    }

    fn read_main_locked(
        self,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
    ) -> Option<LockedBufferGuard> {
        LockedBufferGuard::read_main_locked_handle(self.relation, block_number, mode)
    }

    fn start_wal(self) -> wal::GenericXLogTxn {
        // SAFETY: this view is constructed only for a live IVF index relation.
        unsafe { wal::GenericXLogTxn::start(self.relation.as_ptr()) }
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn ivf_relation_nonnull(index_relation: pg_sys::Relation, context: &str) -> RelationHandle {
    NonNull::new(index_relation).unwrap_or_else(|| pgrx::error!("{context} received null relation"))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn read_posting_block(
    index: IvfPageRelation<'_>,
    block_number: pg_sys::BlockNumber,
    context: &str,
) -> Result<LockedBufferGuard, String> {
    index
        .read_main(
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| format!("ec_ivf failed to open {context} block {block_number}"))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
struct PageTuplePage {
    page_ptr: *mut u8,
    page_size: usize,
    block_number: pg_sys::BlockNumber,
    line_pointer_count: u16,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
impl PageTuplePage {
    fn new(page_ptr: *mut u8, page_size: usize, block_number: pg_sys::BlockNumber) -> Self {
        Self {
            page_ptr,
            page_size,
            block_number,
            line_pointer_count: page_line_pointer_count(page_ptr),
        }
    }

    fn line_pointer_count(&self) -> u16 {
        self.line_pointer_count
    }

    fn visit_line<R, F>(
        &self,
        offset: u16,
        tuple_kind: &str,
        visit: F,
    ) -> Result<PageTupleVisit<R>, String>
    where
        F: for<'tuple> FnOnce(&'tuple [u8]) -> Result<R, String>,
    {
        let Some(slot) = self.optional_slot(offset, tuple_kind)? else {
            return Ok(PageTupleVisit::Unused);
        };

        // SAFETY: tuple offset and length were checked against `page_size`, and
        // the page remains locked for the duration of the visitor call.
        let tuple_bytes =
            unsafe { std::slice::from_raw_parts(self.page_ptr.add(slot.offset), slot.len) };
        visit(tuple_bytes).map(PageTupleVisit::Present)
    }

    fn visit_required<R, F>(&self, offset: u16, tuple_kind: &str, visit: F) -> Result<R, String>
    where
        F: for<'tuple> FnOnce(&'tuple [u8]) -> Result<R, String>,
    {
        match self.visit_line(offset, tuple_kind, visit)? {
            PageTupleVisit::Unused => Err(format!("ec_ivf {tuple_kind} tuple slot is unused")),
            PageTupleVisit::Present(tuple) => Ok(tuple),
        }
    }

    fn copy_required_exact(
        &self,
        tid: ItemPointer,
        tuple_kind: &str,
        encoded: &[u8],
    ) -> Result<(), String> {
        let slot = self.required_slot(tid.offset_number, tuple_kind)?;
        if slot.len != encoded.len() {
            return Err(format!(
                "ec_ivf {tuple_kind} tuple size changed from {} to {}",
                slot.len,
                encoded.len()
            ));
        }

        // SAFETY: the slot is live, in bounds, and exactly the same length as
        // `encoded`; the page remains WAL-registered and locked by the caller.
        unsafe {
            ptr::copy_nonoverlapping(
                encoded.as_ptr(),
                self.page_ptr.add(slot.offset),
                encoded.len(),
            )
        };
        Ok(())
    }

    fn required_slot(&self, offset: u16, tuple_kind: &str) -> Result<PageTupleSlot, String> {
        self.optional_slot(offset, tuple_kind)?
            .ok_or_else(|| format!("ec_ivf {tuple_kind} tuple slot is unused"))
    }

    fn optional_slot(
        &self,
        offset: u16,
        tuple_kind: &str,
    ) -> Result<Option<PageTupleSlot>, String> {
        if offset == 0 || offset > self.line_pointer_count {
            return Err(format!(
                "ec_ivf {tuple_kind} tuple offset {offset} out of range on block {}",
                self.block_number
            ));
        }

        // SAFETY: offset is nonzero and range-checked against the page's line
        // pointer count before computing the ItemId address.
        let item_id = unsafe {
            &*self
                .page_ptr
                .add(PAGE_HEADER_BYTES + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()))
                .cast::<pg_sys::ItemIdData>()
        };
        if item_id.lp_flags() == 0 {
            return Ok(None);
        }
        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > self.page_size {
            return Err(format!(
                "ec_ivf {tuple_kind} tuple bounds exceed block {}",
                self.block_number
            ));
        }
        Ok(Some(PageTupleSlot {
            offset: tuple_offset,
            len: tuple_len,
        }))
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
struct PageTupleReader<'a> {
    page: PageTuplePage,
    _buffer: PhantomData<&'a LockedBufferGuard>,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
impl<'a> PageTupleReader<'a> {
    fn new(buffer: &'a LockedBufferGuard, block_number: pg_sys::BlockNumber) -> Self {
        Self {
            page: PageTuplePage::new(buffer.page().cast::<u8>(), buffer.page_size(), block_number),
            _buffer: PhantomData,
        }
    }

    fn line_pointer_count(&self) -> u16 {
        self.page.line_pointer_count()
    }

    fn visit_line<R, F>(
        &self,
        offset: u16,
        tuple_kind: &str,
        visit: F,
    ) -> Result<PageTupleVisit<R>, String>
    where
        F: for<'tuple> FnOnce(&'tuple [u8]) -> Result<R, String>,
    {
        self.page.visit_line(offset, tuple_kind, visit)
    }

    fn visit_required<R, F>(&self, offset: u16, tuple_kind: &str, visit: F) -> Result<R, String>
    where
        F: for<'tuple> FnOnce(&'tuple [u8]) -> Result<R, String>,
    {
        self.page.visit_required(offset, tuple_kind, visit)
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
struct PageTupleWriter {
    page: PageTuplePage,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
impl PageTupleWriter {
    fn new(page: pg_sys::Page, page_size: usize, block_number: pg_sys::BlockNumber) -> Self {
        Self {
            page: PageTuplePage::new(page.cast::<u8>(), page_size, block_number),
        }
    }

    fn line_pointer_count(&self) -> u16 {
        self.page.line_pointer_count()
    }

    fn visit_line<R, F>(
        &self,
        offset: u16,
        tuple_kind: &str,
        visit: F,
    ) -> Result<PageTupleVisit<R>, String>
    where
        F: for<'tuple> FnOnce(&'tuple [u8]) -> Result<R, String>,
    {
        self.page.visit_line(offset, tuple_kind, visit)
    }

    fn visit_required<R, F>(
        &self,
        tid: ItemPointer,
        tuple_kind: &str,
        visit: F,
    ) -> Result<R, String>
    where
        F: for<'tuple> FnOnce(&'tuple [u8]) -> Result<R, String>,
    {
        self.page
            .visit_required(tid.offset_number, tuple_kind, visit)
    }

    fn copy_required_exact(
        &self,
        tid: ItemPointer,
        tuple_kind: &str,
        encoded: &[u8],
    ) -> Result<(), String> {
        self.page.copy_required_exact(tid, tuple_kind, encoded)
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
struct PageTupleSlot {
    offset: usize,
    len: usize,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
struct WalRegisteredPage {
    relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    page: pg_sys::Page,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
impl WalRegisteredPage {
    fn new(
        relation: pg_sys::Relation,
        block_number: pg_sys::BlockNumber,
        page: pg_sys::Page,
    ) -> Self {
        Self {
            relation,
            block_number,
            page,
        }
    }

    fn page(&self) -> pg_sys::Page {
        self.page
    }

    fn init(&self, page_size: usize, special_size: usize) {
        // SAFETY: callers construct this wrapper only around a WAL-registered
        // page image whose buffer remains locked for initialization.
        unsafe { pg_sys::PageInit(self.page, page_size, special_size) };
    }

    fn free_space(&self) -> usize {
        // SAFETY: `page` is the still-registered image for the held buffer.
        unsafe { pg_sys::PageGetFreeSpace(self.page) as usize }
    }

    fn record_free_space(&self, free_space: usize) {
        // SAFETY: relation and block number identify the live registered page.
        unsafe { pg_sys::RecordPageWithFreeSpace(self.relation, self.block_number, free_space) };
    }

    fn add_item(&self, payload: &[u8]) -> pg_sys::OffsetNumber {
        // SAFETY: `page` is WAL-registered and locked; callers pass an encoded
        // tuple payload already checked for the target page capacity.
        unsafe {
            pg_sys::PageAddItemExtended(
                self.page,
                payload.as_ptr().cast_mut().cast(),
                payload.len(),
                pg_sys::InvalidOffsetNumber,
                0,
            )
        }
    }

    fn special_bytes(&self, len: usize) -> &[u8] {
        // SAFETY: callers request the fixed special area size for this page
        // type while the registered page remains locked.
        unsafe {
            std::slice::from_raw_parts(pg_sys::PageGetSpecialPointer(self.page).cast::<u8>(), len)
        }
    }

    fn copy_to_special(&self, bytes: &[u8]) {
        // SAFETY: callers provide a fixed-size special-area encoding for this
        // registered page type.
        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                pg_sys::PageGetSpecialPointer(self.page).cast::<u8>(),
                bytes.len(),
            )
        };
    }

    fn multi_delete(&self, offsets: &mut [u16]) -> Result<(), String> {
        // SAFETY: offsets were collected from valid line pointers on this
        // registered page and the count is checked before calling PostgreSQL.
        unsafe {
            pg_sys::PageIndexMultiDelete(
                self.page,
                offsets.as_mut_ptr(),
                offsets
                    .len()
                    .try_into()
                    .map_err(|_| "ec_ivf posting delete count exceeds c_int".to_owned())?,
            )
        };
        Ok(())
    }

    fn delete_no_compact(&self, offset: u16) {
        // SAFETY: offset was collected from a valid line pointer on this
        // registered page.
        unsafe { pg_sys::PageIndexTupleDeleteNoCompact(self.page, offset) };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetadataPage {
    pub format_version: u16,
    pub dimensions: u16,
    pub nlists: u32,
    pub nprobe: u32,
    pub training_sample_rows: u32,
    pub training_version: u16,
    pub seed: u64,
    pub storage_format: StorageFormat,
    pub rerank: RerankMode,
    /// RaBitQ per-dim code width. Valid values are {1, 2, 4, 8}. v1
    /// indexes write 0 here and the decoder coerces to 4 (the legacy
    /// hardcoded value).
    pub quant_bits: u8,
    pub centroid_head: ItemPointer,
    pub directory_head: ItemPointer,
    pub total_live_tuples: u64,
    pub total_dead_tuples: u64,
    pub inserted_since_build: u64,
    pub pq_codebook_head: ItemPointer,
    pub pq_group_size: u16,
}

impl MetadataPage {
    pub(super) fn empty(options: EcIvfOptions) -> Self {
        Self {
            format_version: INDEX_FORMAT_VERSION,
            dimensions: 0,
            nlists: u32::try_from(options.nlists).expect("validated nlists should fit in u32"),
            nprobe: u32::try_from(options.nprobe).expect("validated nprobe should fit in u32"),
            training_sample_rows: u32::try_from(options.training_sample_rows)
                .expect("validated training_sample_rows should fit in u32"),
            training_version: 0,
            seed: u64::try_from(options.seed).expect("validated seed should fit in u64"),
            storage_format: options.storage_format,
            rerank: options.rerank.v1_effective(),
            quant_bits: options.effective_quant_bits(),
            centroid_head: ItemPointer::INVALID,
            directory_head: ItemPointer::INVALID,
            total_live_tuples: 0,
            total_dead_tuples: 0,
            inserted_since_build: 0,
            pq_codebook_head: ItemPointer::INVALID,
            pq_group_size: 0,
        }
    }

    pub(super) fn encode(&self) -> [u8; METADATA_BYTES] {
        let mut out = [0_u8; METADATA_BYTES];
        out[0..4].copy_from_slice(&METADATA_MAGIC.to_le_bytes());
        out[4..6].copy_from_slice(&self.format_version.to_le_bytes());
        out[6..8].copy_from_slice(&self.dimensions.to_le_bytes());
        out[8..12].copy_from_slice(&self.nlists.to_le_bytes());
        out[12..16].copy_from_slice(&self.nprobe.to_le_bytes());
        out[16..20].copy_from_slice(&self.training_sample_rows.to_le_bytes());
        out[20..22].copy_from_slice(&self.training_version.to_le_bytes());
        out[24..32].copy_from_slice(&self.seed.to_le_bytes());
        out[32] = self.storage_format as u8;
        out[33] = self.rerank as u8;
        out[34] = self.quant_bits;
        write_item_pointer(&mut out[36..42], self.centroid_head);
        write_item_pointer(&mut out[42..48], self.directory_head);
        out[48..56].copy_from_slice(&self.total_live_tuples.to_le_bytes());
        out[56..64].copy_from_slice(&self.total_dead_tuples.to_le_bytes());
        out[64..72].copy_from_slice(&self.inserted_since_build.to_le_bytes());
        write_item_pointer(&mut out[72..78], self.pq_codebook_head);
        out[78..80].copy_from_slice(&self.pq_group_size.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < METADATA_BYTES {
            return Err(format!(
                "ec_ivf metadata length mismatch: got {}, expected at least {METADATA_BYTES}",
                bytes.len()
            ));
        }
        let magic = u32::from_le_bytes(
            bytes[0..4]
                .try_into()
                .expect("metadata magic slice should be 4 bytes"),
        );
        if magic != METADATA_MAGIC {
            return Err(format!("invalid ec_ivf metadata magic: {magic:#x}"));
        }
        let format_version = u16::from_le_bytes(
            bytes[4..6]
                .try_into()
                .expect("metadata format slice should be 2 bytes"),
        );
        if !(EC_IVF_INDEX_FORMAT_VERSION_MIN..=INDEX_FORMAT_VERSION).contains(&format_version) {
            return Err(format!(
                "unsupported ec_ivf metadata format version: {format_version}"
            ));
        }
        Ok(Self {
            format_version,
            dimensions: u16::from_le_bytes(
                bytes[6..8]
                    .try_into()
                    .expect("metadata dimensions slice should be 2 bytes"),
            ),
            nlists: u32::from_le_bytes(
                bytes[8..12]
                    .try_into()
                    .expect("metadata nlists slice should be 4 bytes"),
            ),
            nprobe: u32::from_le_bytes(
                bytes[12..16]
                    .try_into()
                    .expect("metadata nprobe slice should be 4 bytes"),
            ),
            training_sample_rows: u32::from_le_bytes(
                bytes[16..20]
                    .try_into()
                    .expect("metadata training sample slice should be 4 bytes"),
            ),
            training_version: u16::from_le_bytes(
                bytes[20..22]
                    .try_into()
                    .expect("metadata training version slice should be 2 bytes"),
            ),
            seed: u64::from_le_bytes(
                bytes[24..32]
                    .try_into()
                    .expect("metadata seed slice should be 8 bytes"),
            ),
            storage_format: decode_storage_format(bytes[32])?,
            rerank: decode_rerank(bytes[33])?,
            // v1 metadata didn't write here, leaving 0 — coerce to the
            // legacy default of 4 so old indexes keep working.
            quant_bits: match bytes[34] {
                0 => 4,
                b @ (1 | 2 | 4 | 8) => b,
                other => {
                    return Err(format!(
                        "invalid ec_ivf quant_bits stored in metadata: {other}"
                    ))
                }
            },
            centroid_head: ItemPointer::decode(&bytes[36..42])?,
            directory_head: ItemPointer::decode(&bytes[42..48])?,
            total_live_tuples: u64::from_le_bytes(
                bytes[48..56]
                    .try_into()
                    .expect("metadata live tuple slice should be 8 bytes"),
            ),
            total_dead_tuples: u64::from_le_bytes(
                bytes[56..64]
                    .try_into()
                    .expect("metadata dead tuple slice should be 8 bytes"),
            ),
            inserted_since_build: u64::from_le_bytes(
                bytes[64..72]
                    .try_into()
                    .expect("metadata inserted-since-build slice should be 8 bytes"),
            ),
            pq_codebook_head: ItemPointer::decode(&bytes[72..78])?,
            pq_group_size: u16::from_le_bytes(
                bytes[78..80]
                    .try_into()
                    .expect("metadata pq group size slice should be 2 bytes"),
            ),
        })
    }
}

fn write_item_pointer(out: &mut [u8], tid: ItemPointer) {
    debug_assert_eq!(out.len(), ITEM_POINTER_BYTES);
    out[0..4].copy_from_slice(&tid.block_number.to_le_bytes());
    out[4..6].copy_from_slice(&tid.offset_number.to_le_bytes());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRef {
    pub block_number: u32,
}

impl BlockRef {
    pub(super) const INVALID: Self = Self {
        block_number: u32::MAX,
    };

    pub(super) fn encode_into(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.block_number.to_le_bytes());
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != BLOCK_REF_BYTES {
            return Err(format!(
                "ec_ivf block ref length mismatch: got {}, expected {BLOCK_REF_BYTES}",
                input.len()
            ));
        }

        Ok(Self {
            block_number: u32::from_le_bytes(
                input
                    .try_into()
                    .expect("validated block ref slice should be 4 bytes"),
            ),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IvfCentroidTuple {
    pub list_id: u32,
    pub centroid: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IvfCentroidTupleRef<'a> {
    pub(super) list_id: u32,
    centroid_bytes: &'a [u8],
}

impl<'a> IvfCentroidTupleRef<'a> {
    pub(super) fn decode(input: &'a [u8], dimensions: usize) -> Result<Self, String> {
        let expected_len = IvfCentroidTuple::encoded_len(dimensions);
        if input.len() != expected_len {
            return Err(format!(
                "ec_ivf centroid tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if input[0] != IVF_CENTROID_TAG {
            return Err(format!("invalid ec_ivf centroid tuple tag: {}", input[0]));
        }

        let tuple_dimensions = u16::from_le_bytes(
            input[5..7]
                .try_into()
                .expect("centroid dimensions slice should be 2 bytes"),
        ) as usize;
        if tuple_dimensions != dimensions {
            return Err(format!(
                "ec_ivf centroid dimensions mismatch: got {tuple_dimensions}, expected {dimensions}"
            ));
        }

        Ok(Self {
            list_id: u32::from_le_bytes(
                input[1..5]
                    .try_into()
                    .expect("centroid list id slice should be 4 bytes"),
            ),
            centroid_bytes: &input[7..],
        })
    }

    pub(super) fn centroid_values(&self) -> impl Iterator<Item = f32> + '_ {
        self.centroid_bytes
            .chunks_exact(std::mem::size_of::<f32>())
            .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("validated f32 chunk")))
    }

    pub(super) fn collect_centroid(&self) -> Vec<f32> {
        self.centroid_values().collect()
    }
}

impl IvfCentroidTuple {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        let dimensions = u16::try_from(self.centroid.len()).map_err(|_| {
            format!(
                "ec_ivf centroid dimensions {} exceed persisted u16 limit",
                self.centroid.len()
            )
        })?;
        if self.centroid.iter().any(|value| !value.is_finite()) {
            return Err("ec_ivf centroid contains a non-finite value".into());
        }

        let mut out = Vec::with_capacity(Self::encoded_len(self.centroid.len()));
        out.push(IVF_CENTROID_TAG);
        out.extend_from_slice(&self.list_id.to_le_bytes());
        out.extend_from_slice(&dimensions.to_le_bytes());
        for value in &self.centroid {
            out.extend_from_slice(&value.to_le_bytes());
        }
        Ok(out)
    }

    pub fn decode(input: &[u8], dimensions: usize) -> Result<Self, String> {
        let centroid = IvfCentroidTupleRef::decode(input, dimensions)?;
        Ok(Self {
            list_id: centroid.list_id,
            centroid: centroid.collect_centroid(),
        })
    }

    pub(super) fn encoded_len(dimensions: usize) -> usize {
        1 + 4 + 2 + dimensions * std::mem::size_of::<f32>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IvfListDirectoryTuple {
    pub list_id: u32,
    pub head_block: BlockRef,
    pub tail_block: BlockRef,
    pub live_count: u64,
    pub dead_count: u64,
    pub inserted_since_build: u64,
}

impl IvfListDirectoryTuple {
    pub(super) fn empty(list_id: u32) -> Self {
        Self {
            list_id,
            head_block: BlockRef::INVALID,
            tail_block: BlockRef::INVALID,
            live_count: 0,
            dead_count: 0,
            inserted_since_build: 0,
        }
    }

    pub(super) fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::encoded_len());
        out.push(IVF_LIST_DIRECTORY_TAG);
        out.extend_from_slice(&self.list_id.to_le_bytes());
        self.head_block.encode_into(&mut out);
        self.tail_block.encode_into(&mut out);
        out.extend_from_slice(&self.live_count.to_le_bytes());
        out.extend_from_slice(&self.dead_count.to_le_bytes());
        out.extend_from_slice(&self.inserted_since_build.to_le_bytes());
        out
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != Self::encoded_len() {
            return Err(format!(
                "ec_ivf list directory tuple length mismatch: got {}, expected {}",
                input.len(),
                Self::encoded_len()
            ));
        }
        if input[0] != IVF_LIST_DIRECTORY_TAG {
            return Err(format!(
                "invalid ec_ivf list directory tuple tag: {}",
                input[0]
            ));
        }

        Ok(Self {
            list_id: u32::from_le_bytes(
                input[1..5]
                    .try_into()
                    .expect("directory list id slice should be 4 bytes"),
            ),
            head_block: BlockRef::decode(&input[5..9])?,
            tail_block: BlockRef::decode(&input[9..13])?,
            live_count: u64::from_le_bytes(
                input[13..21]
                    .try_into()
                    .expect("directory live count slice should be 8 bytes"),
            ),
            dead_count: u64::from_le_bytes(
                input[21..29]
                    .try_into()
                    .expect("directory dead count slice should be 8 bytes"),
            ),
            inserted_since_build: u64::from_le_bytes(
                input[29..37]
                    .try_into()
                    .expect("directory inserted count slice should be 8 bytes"),
            ),
        })
    }

    pub(super) const fn encoded_len() -> usize {
        1 + 4 + BLOCK_REF_BYTES + BLOCK_REF_BYTES + 8 + 8 + 8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct IvfColumnarFrozenListHeaderTuple {
    pub(super) list_id: u32,
    pub(super) posting_count: u32,
    pub(super) payload_len: u16,
    pub(super) total_heap_tids: u32,
    pub(super) gamma_offset: u32,
    pub(super) payload_offset: u32,
    pub(super) heap_tid_count_offset: u32,
    pub(super) heap_tid_offset_offset: u32,
    pub(super) heap_tid_offset: u32,
    pub(super) rerank_tid_offset: u32,
    pub(super) deleted_bitmap_offset: u32,
    pub(super) total_column_bytes: u32,
    pub(super) first_column_block: BlockRef,
    pub(super) last_column_block: BlockRef,
}

impl IvfColumnarFrozenListHeaderTuple {
    pub(super) fn from_shape(
        list_id: u32,
        posting_count: usize,
        payload_len: usize,
        total_heap_tids: usize,
        first_column_block: BlockRef,
        last_column_block: BlockRef,
    ) -> Result<Self, String> {
        if posting_count == 0 {
            return Err("ec_ivf columnar frozen list header requires postings".to_owned());
        }
        if payload_len == 0 {
            return Err("ec_ivf columnar frozen list header requires payload bytes".to_owned());
        }
        if total_heap_tids < posting_count {
            return Err(
                "ec_ivf columnar frozen list header heap tid count is smaller than posting count"
                    .to_owned(),
            );
        }
        validate_columnar_frozen_list_block_range(first_column_block, last_column_block)?;
        let offsets =
            checked_columnar_frozen_list_offsets(posting_count, payload_len, total_heap_tids)?;
        Ok(Self {
            list_id,
            posting_count: u32::try_from(posting_count).map_err(|_| {
                "ec_ivf columnar frozen list header posting count exceeds u32".to_owned()
            })?,
            payload_len: u16::try_from(payload_len).map_err(|_| {
                "ec_ivf columnar frozen list header payload length exceeds u16".to_owned()
            })?,
            total_heap_tids: u32::try_from(total_heap_tids).map_err(|_| {
                "ec_ivf columnar frozen list header heap tid count exceeds u32".to_owned()
            })?,
            gamma_offset: offsets.gamma_offset,
            payload_offset: offsets.payload_offset,
            heap_tid_count_offset: offsets.heap_tid_count_offset,
            heap_tid_offset_offset: offsets.heap_tid_offset_offset,
            heap_tid_offset: offsets.heap_tid_offset,
            rerank_tid_offset: offsets.rerank_tid_offset,
            deleted_bitmap_offset: offsets.deleted_bitmap_offset,
            total_column_bytes: offsets.total_column_bytes,
            first_column_block,
            last_column_block,
        })
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut out = Vec::with_capacity(Self::encoded_len());
        out.push(IVF_COLUMNAR_FROZEN_LIST_HEADER_TAG);
        out.push(COLUMNAR_FROZEN_LIST_HEADER_VERSION);
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.list_id.to_le_bytes());
        out.extend_from_slice(&self.posting_count.to_le_bytes());
        out.extend_from_slice(&self.payload_len.to_le_bytes());
        out.extend_from_slice(&self.total_heap_tids.to_le_bytes());
        out.extend_from_slice(&self.gamma_offset.to_le_bytes());
        out.extend_from_slice(&self.payload_offset.to_le_bytes());
        out.extend_from_slice(&self.heap_tid_count_offset.to_le_bytes());
        out.extend_from_slice(&self.heap_tid_offset_offset.to_le_bytes());
        out.extend_from_slice(&self.heap_tid_offset.to_le_bytes());
        out.extend_from_slice(&self.rerank_tid_offset.to_le_bytes());
        out.extend_from_slice(&self.deleted_bitmap_offset.to_le_bytes());
        out.extend_from_slice(&self.total_column_bytes.to_le_bytes());
        self.first_column_block.encode_into(&mut out);
        self.last_column_block.encode_into(&mut out);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        let header = IvfColumnarFrozenListHeaderRef::decode(input)?;
        Ok(Self {
            list_id: header.list_id,
            posting_count: header.posting_count,
            payload_len: header.payload_len,
            total_heap_tids: header.total_heap_tids,
            gamma_offset: header.gamma_offset,
            payload_offset: header.payload_offset,
            heap_tid_count_offset: header.heap_tid_count_offset,
            heap_tid_offset_offset: header.heap_tid_offset_offset,
            heap_tid_offset: header.heap_tid_offset,
            rerank_tid_offset: header.rerank_tid_offset,
            deleted_bitmap_offset: header.deleted_bitmap_offset,
            total_column_bytes: header.total_column_bytes,
            first_column_block: header.first_column_block,
            last_column_block: header.last_column_block,
        })
    }

    pub(super) const fn encoded_len() -> usize {
        COLUMNAR_FROZEN_LIST_HEADER_BYTES
    }

    fn validate(&self) -> Result<(), String> {
        validate_columnar_frozen_list_shape(
            self.posting_count,
            self.payload_len,
            self.total_heap_tids,
            self.first_column_block,
            self.last_column_block,
        )?;
        let expected = checked_columnar_frozen_list_offsets(
            self.posting_count as usize,
            self.payload_len as usize,
            self.total_heap_tids as usize,
        )?;
        if self.gamma_offset != expected.gamma_offset
            || self.payload_offset != expected.payload_offset
            || self.heap_tid_count_offset != expected.heap_tid_count_offset
            || self.heap_tid_offset_offset != expected.heap_tid_offset_offset
            || self.heap_tid_offset != expected.heap_tid_offset
            || self.rerank_tid_offset != expected.rerank_tid_offset
            || self.deleted_bitmap_offset != expected.deleted_bitmap_offset
            || self.total_column_bytes != expected.total_column_bytes
        {
            return Err("ec_ivf columnar frozen list header column offsets mismatch".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct IvfColumnarFrozenListColumns {
    pub(super) posting_count: usize,
    pub(super) payload_len: usize,
    pub(super) total_heap_tids: usize,
    pub(super) gamma_bytes: Vec<u8>,
    pub(super) payload_bytes: Vec<u8>,
    pub(super) heap_tid_count_bytes: Vec<u8>,
    pub(super) heap_tid_offset_bytes: Vec<u8>,
    pub(super) heap_tid_bytes: Vec<u8>,
    pub(super) rerank_tid_bytes: Vec<u8>,
    pub(super) deleted_bitmap: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct IvfColumnarFrozenListPageChunk<'a> {
    pub(super) start_item: usize,
    pub(super) item_count: usize,
    pub(super) bytes: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IvfColumnarRawPageRange {
    page_index: usize,
    page_offset: usize,
}

impl IvfColumnarFrozenListColumns {
    pub(super) fn from_single_heaptid_postings(
        postings: &[(ItemPointer, f32, ItemPointer, Vec<u8>)],
        payload_len: usize,
    ) -> Result<Self, String> {
        let posting_count = postings.len();
        if posting_count == 0 {
            return Err("ec_ivf columnar frozen list requires postings".to_owned());
        }
        if payload_len == 0 {
            return Err("ec_ivf columnar frozen list requires payload bytes".to_owned());
        }

        let mut gamma_bytes = Vec::with_capacity(posting_count * size_of::<f32>());
        let mut payload_bytes = Vec::with_capacity(posting_count * payload_len);
        let mut heap_tid_count_bytes = Vec::with_capacity(posting_count * size_of::<u16>());
        let mut heap_tid_offset_bytes = Vec::with_capacity(posting_count * size_of::<u32>());
        let mut heap_tid_bytes = Vec::with_capacity(posting_count * ITEM_POINTER_BYTES);
        let mut rerank_tid_bytes = Vec::with_capacity(posting_count * ITEM_POINTER_BYTES);
        for (heap_tid, gamma, rerank_tid, payload) in postings {
            if !gamma.is_finite() {
                return Err("ec_ivf columnar frozen list gamma must be finite".to_owned());
            }
            if payload.len() != payload_len {
                return Err(format!(
                    "ec_ivf columnar frozen list payload length mismatch: got {}, expected {payload_len}",
                    payload.len()
                ));
            }
            let heap_tid_offset = heap_tid_bytes.len() / ITEM_POINTER_BYTES;
            gamma_bytes.extend_from_slice(&gamma.to_le_bytes());
            payload_bytes.extend_from_slice(payload);
            heap_tid_count_bytes.extend_from_slice(&1_u16.to_le_bytes());
            heap_tid_offset_bytes.extend_from_slice(
                &u32::try_from(heap_tid_offset)
                    .map_err(|_| {
                        "ec_ivf columnar frozen list heap tid offset exceeds u32".to_owned()
                    })?
                    .to_le_bytes(),
            );
            heap_tid.encode_into(&mut heap_tid_bytes);
            rerank_tid.encode_into(&mut rerank_tid_bytes);
        }

        checked_columnar_frozen_list_offsets(posting_count, payload_len, posting_count)?;
        Ok(Self {
            posting_count,
            payload_len,
            total_heap_tids: posting_count,
            gamma_bytes,
            payload_bytes,
            heap_tid_count_bytes,
            heap_tid_offset_bytes,
            heap_tid_bytes,
            rerank_tid_bytes,
            deleted_bitmap: vec![0; dense_deleted_bitmap_len(posting_count)],
        })
    }

    pub(super) fn header(
        &self,
        list_id: u32,
        first_column_block: BlockRef,
        last_column_block: BlockRef,
    ) -> Result<IvfColumnarFrozenListHeaderTuple, String> {
        IvfColumnarFrozenListHeaderTuple::from_shape(
            list_id,
            self.posting_count,
            self.payload_len,
            self.total_heap_tids,
            first_column_block,
            last_column_block,
        )
    }

    pub(super) fn total_column_bytes(&self) -> Result<usize, String> {
        checked_columnar_frozen_list_offsets(
            self.posting_count,
            self.payload_len,
            self.total_heap_tids,
        )
        .map(|offsets| offsets.total_column_bytes as usize)
    }

    pub(super) fn logical_bytes(&self) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(self.total_column_bytes()?);
        out.extend_from_slice(&self.gamma_bytes);
        out.extend_from_slice(&self.payload_bytes);
        out.extend_from_slice(&self.heap_tid_count_bytes);
        out.extend_from_slice(&self.heap_tid_offset_bytes);
        out.extend_from_slice(&self.heap_tid_bytes);
        out.extend_from_slice(&self.rerank_tid_bytes);
        out.extend_from_slice(&self.deleted_bitmap);
        Ok(out)
    }

    pub(super) fn raw_page_bytes(&self, page_size: usize) -> Result<Vec<Vec<u8>>, String> {
        let mut pages = Vec::new();
        self.push_column_raw_pages(&mut pages, &self.gamma_bytes, size_of::<f32>(), page_size)?;
        self.push_column_raw_pages(&mut pages, &self.payload_bytes, self.payload_len, page_size)?;
        self.push_column_raw_pages(
            &mut pages,
            &self.heap_tid_count_bytes,
            size_of::<u16>(),
            page_size,
        )?;
        self.push_column_raw_pages(
            &mut pages,
            &self.heap_tid_offset_bytes,
            size_of::<u32>(),
            page_size,
        )?;
        self.push_column_raw_pages(
            &mut pages,
            &self.heap_tid_bytes,
            ITEM_POINTER_BYTES,
            page_size,
        )?;
        self.push_column_raw_pages(
            &mut pages,
            &self.rerank_tid_bytes,
            ITEM_POINTER_BYTES,
            page_size,
        )?;
        self.push_column_raw_pages(&mut pages, &self.deleted_bitmap, 1, page_size)?;
        Ok(pages)
    }

    fn push_column_raw_pages(
        &self,
        pages: &mut Vec<Vec<u8>>,
        bytes: &[u8],
        item_width: usize,
        page_size: usize,
    ) -> Result<(), String> {
        for chunk in columnar_page_chunks(bytes, item_width, page_size)? {
            pages.push(chunk.bytes.to_vec());
        }
        Ok(())
    }

    pub(super) fn gamma(&self, index: usize) -> f32 {
        let start = index * size_of::<f32>();
        f32::from_le_bytes(
            self.gamma_bytes[start..start + size_of::<f32>()]
                .try_into()
                .expect("validated columnar gamma bytes"),
        )
    }

    pub(super) fn payload(&self, index: usize) -> &[u8] {
        let start = index * self.payload_len;
        &self.payload_bytes[start..start + self.payload_len]
    }

    pub(super) fn heap_tid_count(&self, index: usize) -> usize {
        let start = index * size_of::<u16>();
        u16::from_le_bytes(
            self.heap_tid_count_bytes[start..start + size_of::<u16>()]
                .try_into()
                .expect("validated columnar heap tid count bytes"),
        ) as usize
    }

    pub(super) fn heap_tid_offset(&self, index: usize) -> usize {
        let start = index * size_of::<u32>();
        u32::from_le_bytes(
            self.heap_tid_offset_bytes[start..start + size_of::<u32>()]
                .try_into()
                .expect("validated columnar heap tid offset bytes"),
        ) as usize
    }

    pub(super) fn heap_tids(&self, index: usize) -> impl Iterator<Item = ItemPointer> + '_ {
        let start = self.heap_tid_offset(index);
        let count = self.heap_tid_count(index);
        self.heap_tid_bytes[start * ITEM_POINTER_BYTES..(start + count) * ITEM_POINTER_BYTES]
            .chunks_exact(ITEM_POINTER_BYTES)
            .map(|chunk| ItemPointer::decode(chunk).expect("validated columnar heap tid bytes"))
    }

    pub(super) fn rerank_tid(&self, index: usize) -> ItemPointer {
        let start = index * ITEM_POINTER_BYTES;
        ItemPointer::decode(&self.rerank_tid_bytes[start..start + ITEM_POINTER_BYTES])
            .expect("validated columnar rerank tid bytes")
    }

    pub(super) fn is_deleted(&self, index: usize) -> bool {
        dense_deleted_bitmap_get(&self.deleted_bitmap, index)
    }

    pub(super) fn gamma_values_native_le(&self) -> Option<&[f32]> {
        native_le_f32_slice(&self.gamma_bytes)
    }

    pub(super) fn heap_tid_counts_native_le(&self) -> Option<&[u16]> {
        native_le_u16_slice(&self.heap_tid_count_bytes)
    }

    pub(super) fn heap_tid_offsets_native_le(&self) -> Option<&[u32]> {
        native_le_u32_slice(&self.heap_tid_offset_bytes)
    }

    pub(super) fn payload_page_chunks(
        &self,
        page_size: usize,
    ) -> Result<Vec<IvfColumnarFrozenListPageChunk<'_>>, String> {
        columnar_page_chunks(&self.payload_bytes, self.payload_len, page_size)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IvfPostingTuple {
    pub list_id: u32,
    pub deleted: bool,
    pub heaptids: Vec<ItemPointer>,
    pub gamma: f32,
    pub rerank_tid: ItemPointer,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct IvfDensePostingBlockTuple {
    pub(super) list_id: u32,
    pub(super) gammas: Vec<f32>,
    pub(super) heap_tid_counts: Vec<u16>,
    pub(super) heap_tid_offsets: Vec<u32>,
    pub(super) rerank_tids: Vec<ItemPointer>,
    pub(super) heap_tids: Vec<ItemPointer>,
    pub(super) deleted_bitmap: Vec<u8>,
    pub(super) payload_len: usize,
    pub(super) payloads: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct IvfDensePostingPackedSegmentTuple {
    pub(super) list_id: u32,
    pub(super) logical_block_id: u32,
    pub(super) segment_index: u16,
    pub(super) segment_count: u16,
    pub(super) total_posting_count: u16,
    pub(super) gammas: Vec<f32>,
    pub(super) heap_tid_counts: Vec<u16>,
    pub(super) heap_tid_offsets: Vec<u32>,
    pub(super) rerank_tids: Vec<ItemPointer>,
    pub(super) heap_tids: Vec<ItemPointer>,
    pub(super) deleted_bitmap: Vec<u8>,
    pub(super) payload_len: usize,
    pub(super) payloads: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct IvfDensePostingPackedContinuationTuple {
    pub(super) list_id: u32,
    pub(super) logical_block_id: u32,
    pub(super) segment_index: u16,
    pub(super) segment_count: u16,
    pub(super) payload_offset: u32,
    pub(super) payloads: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IvfDensePostingBlockRef<'a> {
    pub(super) list_id: u32,
    count: usize,
    total_heap_tids: usize,
    payload_len: usize,
    deleted_bitmap: &'a [u8],
    gamma_bytes: &'a [u8],
    heap_tid_count_bytes: &'a [u8],
    heap_tid_offset_bytes: &'a [u8],
    rerank_tid_bytes: &'a [u8],
    heap_tid_bytes: &'a [u8],
    pub(super) payloads: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IvfDensePostingPackedSegmentRef<'a> {
    pub(super) list_id: u32,
    pub(super) logical_block_id: u32,
    pub(super) segment_index: u16,
    pub(super) segment_count: u16,
    pub(super) total_posting_count: u16,
    count: usize,
    total_heap_tids: usize,
    payload_len: usize,
    deleted_bitmap: &'a [u8],
    gamma_bytes: &'a [u8],
    heap_tid_count_bytes: &'a [u8],
    heap_tid_offset_bytes: &'a [u8],
    rerank_tid_bytes: &'a [u8],
    heap_tid_bytes: &'a [u8],
    pub(super) payloads: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IvfDensePostingPackedContinuationRef<'a> {
    pub(super) list_id: u32,
    pub(super) logical_block_id: u32,
    pub(super) segment_index: u16,
    pub(super) segment_count: u16,
    pub(super) payload_offset: u32,
    pub(super) payloads: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IvfColumnarFrozenListHeaderRef {
    pub(super) list_id: u32,
    pub(super) posting_count: u32,
    pub(super) payload_len: u16,
    pub(super) total_heap_tids: u32,
    pub(super) gamma_offset: u32,
    pub(super) payload_offset: u32,
    pub(super) heap_tid_count_offset: u32,
    pub(super) heap_tid_offset_offset: u32,
    pub(super) heap_tid_offset: u32,
    pub(super) rerank_tid_offset: u32,
    pub(super) deleted_bitmap_offset: u32,
    pub(super) total_column_bytes: u32,
    pub(super) first_column_block: BlockRef,
    pub(super) last_column_block: BlockRef,
}

impl IvfColumnarFrozenListHeaderRef {
    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != COLUMNAR_FROZEN_LIST_HEADER_BYTES {
            return Err(format!(
                "ec_ivf columnar frozen list header length mismatch: got {}, expected {COLUMNAR_FROZEN_LIST_HEADER_BYTES}",
                input.len()
            ));
        }
        if input[0] != IVF_COLUMNAR_FROZEN_LIST_HEADER_TAG {
            return Err(format!(
                "invalid ec_ivf columnar frozen list header tag: {}",
                input[0]
            ));
        }
        if input[1] != COLUMNAR_FROZEN_LIST_HEADER_VERSION {
            return Err(format!(
                "invalid ec_ivf columnar frozen list header version: {}",
                input[1]
            ));
        }
        let reserved = u16::from_le_bytes(
            input[2..4]
                .try_into()
                .expect("columnar header reserved slice should be 2 bytes"),
        );
        if reserved != 0 {
            return Err("ec_ivf columnar frozen list header reserved flags are set".to_owned());
        }

        let header = Self {
            list_id: u32::from_le_bytes(
                input[4..8]
                    .try_into()
                    .expect("columnar header list id slice should be 4 bytes"),
            ),
            posting_count: u32::from_le_bytes(
                input[8..12]
                    .try_into()
                    .expect("columnar header posting count slice should be 4 bytes"),
            ),
            payload_len: u16::from_le_bytes(
                input[12..14]
                    .try_into()
                    .expect("columnar header payload length slice should be 2 bytes"),
            ),
            total_heap_tids: u32::from_le_bytes(
                input[14..18]
                    .try_into()
                    .expect("columnar header heap tid count slice should be 4 bytes"),
            ),
            gamma_offset: u32::from_le_bytes(
                input[18..22]
                    .try_into()
                    .expect("columnar header gamma offset slice should be 4 bytes"),
            ),
            payload_offset: u32::from_le_bytes(
                input[22..26]
                    .try_into()
                    .expect("columnar header payload offset slice should be 4 bytes"),
            ),
            heap_tid_count_offset: u32::from_le_bytes(
                input[26..30]
                    .try_into()
                    .expect("columnar header heap tid count offset slice should be 4 bytes"),
            ),
            heap_tid_offset_offset: u32::from_le_bytes(
                input[30..34]
                    .try_into()
                    .expect("columnar header heap tid offset offset slice should be 4 bytes"),
            ),
            heap_tid_offset: u32::from_le_bytes(
                input[34..38]
                    .try_into()
                    .expect("columnar header heap tid offset slice should be 4 bytes"),
            ),
            rerank_tid_offset: u32::from_le_bytes(
                input[38..42]
                    .try_into()
                    .expect("columnar header rerank tid offset slice should be 4 bytes"),
            ),
            deleted_bitmap_offset: u32::from_le_bytes(
                input[42..46]
                    .try_into()
                    .expect("columnar header deleted bitmap offset slice should be 4 bytes"),
            ),
            total_column_bytes: u32::from_le_bytes(
                input[46..50]
                    .try_into()
                    .expect("columnar header total column bytes slice should be 4 bytes"),
            ),
            first_column_block: BlockRef::decode(&input[50..54])?,
            last_column_block: BlockRef::decode(&input[54..58])?,
        };
        header.validate()?;
        Ok(header)
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        validate_columnar_frozen_list_shape(
            self.posting_count,
            self.payload_len,
            self.total_heap_tids,
            self.first_column_block,
            self.last_column_block,
        )?;
        let expected = checked_columnar_frozen_list_offsets(
            self.posting_count as usize,
            self.payload_len as usize,
            self.total_heap_tids as usize,
        )?;
        if self.gamma_offset != expected.gamma_offset
            || self.payload_offset != expected.payload_offset
            || self.heap_tid_count_offset != expected.heap_tid_count_offset
            || self.heap_tid_offset_offset != expected.heap_tid_offset_offset
            || self.heap_tid_offset != expected.heap_tid_offset
            || self.rerank_tid_offset != expected.rerank_tid_offset
            || self.deleted_bitmap_offset != expected.deleted_bitmap_offset
            || self.total_column_bytes != expected.total_column_bytes
        {
            return Err("ec_ivf columnar frozen list header column offsets mismatch".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IvfColumnarFrozenListRef<'a> {
    pub(super) list_id: u32,
    count: usize,
    total_heap_tids: usize,
    payload_len: usize,
    gamma_bytes: &'a [u8],
    payloads: &'a [u8],
    heap_tid_count_bytes: &'a [u8],
    heap_tid_offset_bytes: &'a [u8],
    heap_tid_bytes: &'a [u8],
    rerank_tid_bytes: &'a [u8],
    deleted_bitmap: &'a [u8],
}

impl<'a> IvfColumnarFrozenListRef<'a> {
    pub(super) fn decode(
        header: IvfColumnarFrozenListHeaderRef,
        logical_bytes: &'a [u8],
    ) -> Result<Self, String> {
        header.validate()?;
        let total_column_bytes = header.total_column_bytes as usize;
        if logical_bytes.len() != total_column_bytes {
            return Err(format!(
                "ec_ivf columnar frozen list byte length mismatch: got {}, expected {total_column_bytes}",
                logical_bytes.len()
            ));
        }

        let gamma_start = header.gamma_offset as usize;
        let payload_start = header.payload_offset as usize;
        let heap_tid_count_start = header.heap_tid_count_offset as usize;
        let heap_tid_offset_start = header.heap_tid_offset_offset as usize;
        let heap_tid_start = header.heap_tid_offset as usize;
        let rerank_tid_start = header.rerank_tid_offset as usize;
        let deleted_bitmap_start = header.deleted_bitmap_offset as usize;

        Ok(Self {
            list_id: header.list_id,
            count: header.posting_count as usize,
            total_heap_tids: header.total_heap_tids as usize,
            payload_len: header.payload_len as usize,
            gamma_bytes: &logical_bytes[gamma_start..payload_start],
            payloads: &logical_bytes[payload_start..heap_tid_count_start],
            heap_tid_count_bytes: &logical_bytes[heap_tid_count_start..heap_tid_offset_start],
            heap_tid_offset_bytes: &logical_bytes[heap_tid_offset_start..heap_tid_start],
            heap_tid_bytes: &logical_bytes[heap_tid_start..rerank_tid_start],
            rerank_tid_bytes: &logical_bytes[rerank_tid_start..deleted_bitmap_start],
            deleted_bitmap: &logical_bytes[deleted_bitmap_start..total_column_bytes],
        })
    }

    pub(super) fn len(&self) -> usize {
        self.count
    }

    pub(super) fn payload_len(&self) -> usize {
        self.payload_len
    }

    pub(super) fn is_deleted(&self, index: usize) -> bool {
        dense_deleted_bitmap_get(self.deleted_bitmap, index)
    }

    pub(super) fn gammas_native_le(&self) -> Option<&'a [f32]> {
        native_le_f32_slice(self.gamma_bytes)
    }

    pub(super) fn gamma(&self, index: usize) -> f32 {
        let start = index * size_of::<f32>();
        f32::from_le_bytes(
            self.gamma_bytes[start..start + size_of::<f32>()]
                .try_into()
                .expect("validated columnar gamma chunk"),
        )
    }

    pub(super) fn heap_tid_count(&self, index: usize) -> usize {
        let start = index * size_of::<u16>();
        u16::from_le_bytes(
            self.heap_tid_count_bytes[start..start + size_of::<u16>()]
                .try_into()
                .expect("validated columnar heap tid count chunk"),
        ) as usize
    }

    pub(super) fn heap_tid_offset(&self, index: usize) -> usize {
        let start = index * size_of::<u32>();
        u32::from_le_bytes(
            self.heap_tid_offset_bytes[start..start + size_of::<u32>()]
                .try_into()
                .expect("validated columnar heap tid offset chunk"),
        ) as usize
    }

    pub(super) fn heap_tids(&self, index: usize) -> impl Iterator<Item = ItemPointer> + '_ {
        let start = self.heap_tid_offset(index);
        let count = self.heap_tid_count(index);
        self.heap_tid_bytes[start * ITEM_POINTER_BYTES..(start + count) * ITEM_POINTER_BYTES]
            .chunks_exact(ITEM_POINTER_BYTES)
            .map(|chunk| ItemPointer::decode(chunk).expect("validated columnar heap tid bytes"))
    }

    pub(super) fn payload(&self, index: usize) -> &[u8] {
        let start = index * self.payload_len;
        &self.payloads[start..start + self.payload_len]
    }

    pub(super) fn validate_offsets(&self) -> Result<(), String> {
        if self.gamma_bytes.len() != self.count * size_of::<f32>() {
            return Err("ec_ivf columnar frozen list gamma bytes length mismatch".to_owned());
        }
        if self.payloads.len() != self.count * self.payload_len {
            return Err("ec_ivf columnar frozen list payload bytes length mismatch".to_owned());
        }
        if self.heap_tid_count_bytes.len() != self.count * size_of::<u16>() {
            return Err("ec_ivf columnar frozen list heap tid count length mismatch".to_owned());
        }
        if self.heap_tid_offset_bytes.len() != self.count * size_of::<u32>() {
            return Err("ec_ivf columnar frozen list heap tid offset length mismatch".to_owned());
        }
        if self.heap_tid_bytes.len() != self.total_heap_tids * ITEM_POINTER_BYTES {
            return Err("ec_ivf columnar frozen list heap tid bytes length mismatch".to_owned());
        }
        if self.rerank_tid_bytes.len() != self.count * ITEM_POINTER_BYTES {
            return Err("ec_ivf columnar frozen list rerank tid bytes length mismatch".to_owned());
        }
        if self.deleted_bitmap.len() != dense_deleted_bitmap_len(self.count) {
            return Err("ec_ivf columnar frozen list deleted bitmap length mismatch".to_owned());
        }
        for index in 0..self.count {
            let start = self.heap_tid_offset(index);
            let count = self.heap_tid_count(index);
            if start
                .checked_add(count)
                .is_none_or(|end| end > self.total_heap_tids)
            {
                return Err(
                    "ec_ivf columnar frozen list heap tid range is out of bounds".to_owned(),
                );
            }
            let rerank_start = index * ITEM_POINTER_BYTES;
            ItemPointer::decode(
                &self.rerank_tid_bytes[rerank_start..rerank_start + ITEM_POINTER_BYTES],
            )?;
        }
        Ok(())
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) struct IvfColumnarFrozenListPinnedPages {
    header: IvfColumnarFrozenListHeaderRef,
    page_lengths: Vec<usize>,
    buffers: Vec<LockedBufferGuard>,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
impl IvfColumnarFrozenListPinnedPages {
    pub(super) fn read(
        index_relation: RelationHandle,
        header: IvfColumnarFrozenListHeaderRef,
    ) -> Result<Self, String> {
        header.validate()?;
        let first_block = header.first_column_block.block_number;
        let last_block = header.last_column_block.block_number;
        let expected_block_count = last_block
            .checked_sub(first_block)
            .and_then(|delta| delta.checked_add(1))
            .ok_or_else(|| "ec_ivf columnar frozen list block range underflow".to_owned())?
            as usize;

        let first_buffer = read_columnar_raw_page(
            index_relation,
            first_block,
            "columnar frozen list first page",
        )?;
        let page_size = first_buffer.page_size();
        let page_lengths = columnar_frozen_list_raw_page_lengths(header, page_size)?;
        if page_lengths.len() != expected_block_count {
            return Err(format!(
                "ec_ivf columnar frozen list block count mismatch: header has {expected_block_count}, derived {}",
                page_lengths.len()
            ));
        }

        let mut buffers = Vec::with_capacity(page_lengths.len());
        buffers.push(first_buffer);
        for page_index in 1..page_lengths.len() {
            let block_number = first_block
                .checked_add(page_index as u32)
                .ok_or_else(|| "ec_ivf columnar frozen list block number overflow".to_owned())?;
            buffers.push(read_columnar_raw_page(
                index_relation,
                block_number,
                "columnar frozen list page",
            )?);
        }

        let total: usize = page_lengths.iter().sum();
        if total != header.total_column_bytes as usize {
            return Err(format!(
                "ec_ivf columnar frozen list pinned byte count mismatch: got {total}, expected {}",
                header.total_column_bytes
            ));
        }

        Ok(Self {
            header,
            page_lengths,
            buffers,
        })
    }

    pub(super) fn list_id(&self) -> u32 {
        self.header.list_id
    }

    pub(super) fn len(&self) -> usize {
        self.header.posting_count as usize
    }

    pub(super) fn payload_len(&self) -> usize {
        self.header.payload_len as usize
    }

    pub(super) fn gamma(&self, index: usize) -> Result<f32, String> {
        let start = (self.header.gamma_offset as usize)
            .checked_add(
                index
                    .checked_mul(size_of::<f32>())
                    .ok_or_else(|| "ec_ivf columnar gamma offset overflow".to_owned())?,
            )
            .ok_or_else(|| "ec_ivf columnar gamma offset overflow".to_owned())?;
        let bytes = self.single_page_slice(start, size_of::<f32>())?;
        Ok(f32::from_le_bytes(bytes.try_into().expect(
            "validated columnar gamma slice should be 4 bytes",
        )))
    }

    pub(super) fn is_deleted(&self, index: usize) -> Result<bool, String> {
        let byte = self.single_page_slice(
            (self.header.deleted_bitmap_offset as usize)
                .checked_add(index / 8)
                .ok_or_else(|| "ec_ivf columnar deleted bitmap offset overflow".to_owned())?,
            1,
        )?[0];
        Ok((byte & (1 << (index % 8))) != 0)
    }

    pub(super) fn heap_tid_count(&self, index: usize) -> Result<usize, String> {
        let start = (self.header.heap_tid_count_offset as usize)
            .checked_add(
                index
                    .checked_mul(size_of::<u16>())
                    .ok_or_else(|| "ec_ivf columnar heap tid count offset overflow".to_owned())?,
            )
            .ok_or_else(|| "ec_ivf columnar heap tid count offset overflow".to_owned())?;
        let bytes = self.single_page_slice(start, size_of::<u16>())?;
        Ok(u16::from_le_bytes(
            bytes
                .try_into()
                .expect("validated columnar heap tid count slice should be 2 bytes"),
        ) as usize)
    }

    fn heap_tid_offset(&self, index: usize) -> Result<usize, String> {
        let start = (self.header.heap_tid_offset_offset as usize)
            .checked_add(
                index
                    .checked_mul(size_of::<u32>())
                    .ok_or_else(|| "ec_ivf columnar heap tid offset overflow".to_owned())?,
            )
            .ok_or_else(|| "ec_ivf columnar heap tid offset overflow".to_owned())?;
        let bytes = self.single_page_slice(start, size_of::<u32>())?;
        Ok(u32::from_le_bytes(
            bytes
                .try_into()
                .expect("validated columnar heap tid offset slice should be 4 bytes"),
        ) as usize)
    }

    pub(super) fn heap_tids(&self, index: usize) -> Result<Vec<ItemPointer>, String> {
        let start = self.heap_tid_offset(index)?;
        let count = self.heap_tid_count(index)?;
        let byte_start = (self.header.heap_tid_offset as usize)
            .checked_add(
                start
                    .checked_mul(ITEM_POINTER_BYTES)
                    .ok_or_else(|| "ec_ivf columnar heap tid byte offset overflow".to_owned())?,
            )
            .ok_or_else(|| "ec_ivf columnar heap tid byte offset overflow".to_owned())?;
        let byte_len = count
            .checked_mul(ITEM_POINTER_BYTES)
            .ok_or_else(|| "ec_ivf columnar heap tid byte length overflow".to_owned())?;
        let bytes = self.single_page_slice(byte_start, byte_len)?;
        bytes
            .chunks_exact(ITEM_POINTER_BYTES)
            .map(ItemPointer::decode)
            .collect()
    }

    pub(super) fn extend_heap_tids_into(
        &self,
        index: usize,
        out: &mut Vec<ItemPointer>,
    ) -> Result<(), String> {
        let start = self.heap_tid_offset(index)?;
        let count = self.heap_tid_count(index)?;
        let byte_start = (self.header.heap_tid_offset as usize)
            .checked_add(
                start
                    .checked_mul(ITEM_POINTER_BYTES)
                    .ok_or_else(|| "ec_ivf columnar heap tid byte offset overflow".to_owned())?,
            )
            .ok_or_else(|| "ec_ivf columnar heap tid byte offset overflow".to_owned())?;
        let byte_len = count
            .checked_mul(ITEM_POINTER_BYTES)
            .ok_or_else(|| "ec_ivf columnar heap tid byte length overflow".to_owned())?;
        let bytes = self.single_page_slice(byte_start, byte_len)?;
        out.reserve(count);
        for chunk in bytes.chunks_exact(ITEM_POINTER_BYTES) {
            out.push(ItemPointer::decode(chunk)?);
        }
        Ok(())
    }

    pub(super) fn payload(&self, index: usize) -> Result<&[u8], String> {
        let start = (self.header.payload_offset as usize)
            .checked_add(
                index
                    .checked_mul(self.payload_len())
                    .ok_or_else(|| "ec_ivf columnar payload offset overflow".to_owned())?,
            )
            .ok_or_else(|| "ec_ivf columnar payload offset overflow".to_owned())?;
        self.single_page_slice(start, self.payload_len())
    }

    fn single_page_slice(&self, logical_start: usize, len: usize) -> Result<&[u8], String> {
        let range = columnar_single_page_range(&self.page_lengths, logical_start, len)?;
        let buffer = self
            .buffers
            .get(range.page_index)
            .ok_or_else(|| "ec_ivf columnar raw page range has no buffer".to_owned())?;
        let block_number = buffer.block_number();
        let page = buffer.page();
        let special_size = unsafe { pg_sys::PageGetSpecialSize(page) } as usize;
        let end = range
            .page_offset
            .checked_add(len)
            .ok_or_else(|| "ec_ivf columnar raw page slice overflow".to_owned())?;
        if special_size < end {
            return Err(format!(
                "ec_ivf columnar frozen list page {block_number} special area too small: got {special_size}, expected at least {end}"
            ));
        }
        let special = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
        if special.is_null() {
            return Err(format!(
                "ec_ivf columnar frozen list page {block_number} returned a null special pointer"
            ));
        }
        Ok(unsafe { std::slice::from_raw_parts(special.add(range.page_offset).cast_const(), len) })
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum IvfDensePostingRef<'a> {
    Block(IvfDensePostingBlockRef<'a>),
    PackedSegment(IvfDensePostingPackedSegmentRef<'a>),
}

pub(super) struct IvfDensePostingHeapTids<'a> {
    chunks: std::slice::ChunksExact<'a, u8>,
}

impl Iterator for IvfDensePostingHeapTids<'_> {
    type Item = ItemPointer;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks
            .next()
            .map(|chunk| ItemPointer::decode(chunk).expect("validated dense posting tid bytes"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColumnarFrozenListOffsets {
    gamma_offset: u32,
    payload_offset: u32,
    heap_tid_count_offset: u32,
    heap_tid_offset_offset: u32,
    heap_tid_offset: u32,
    rerank_tid_offset: u32,
    deleted_bitmap_offset: u32,
    total_column_bytes: u32,
}

fn checked_columnar_frozen_list_offsets(
    posting_count: usize,
    payload_len: usize,
    total_heap_tids: usize,
) -> Result<ColumnarFrozenListOffsets, String> {
    let mut offset = 0_usize;
    let gamma_offset = offset;
    offset = offset
        .checked_add(
            posting_count
                .checked_mul(size_of::<f32>())
                .ok_or_else(|| "ec_ivf columnar frozen list gamma bytes overflow".to_owned())?,
        )
        .ok_or_else(|| "ec_ivf columnar frozen list gamma offset overflow".to_owned())?;
    let payload_offset = offset;
    offset = offset
        .checked_add(
            posting_count
                .checked_mul(payload_len)
                .ok_or_else(|| "ec_ivf columnar frozen list payload bytes overflow".to_owned())?,
        )
        .ok_or_else(|| "ec_ivf columnar frozen list payload offset overflow".to_owned())?;
    let heap_tid_count_offset = offset;
    offset = offset
        .checked_add(posting_count.checked_mul(size_of::<u16>()).ok_or_else(|| {
            "ec_ivf columnar frozen list heap tid count bytes overflow".to_owned()
        })?)
        .ok_or_else(|| "ec_ivf columnar frozen list heap tid count offset overflow".to_owned())?;
    let heap_tid_offset_offset = offset;
    offset = offset
        .checked_add(posting_count.checked_mul(size_of::<u32>()).ok_or_else(|| {
            "ec_ivf columnar frozen list heap tid offset bytes overflow".to_owned()
        })?)
        .ok_or_else(|| "ec_ivf columnar frozen list heap tid offset offset overflow".to_owned())?;
    let heap_tid_offset = offset;
    offset = offset
        .checked_add(
            total_heap_tids
                .checked_mul(ITEM_POINTER_BYTES)
                .ok_or_else(|| "ec_ivf columnar frozen list heap tid bytes overflow".to_owned())?,
        )
        .ok_or_else(|| "ec_ivf columnar frozen list heap tid offset overflow".to_owned())?;
    let rerank_tid_offset = offset;
    offset = offset
        .checked_add(
            posting_count
                .checked_mul(ITEM_POINTER_BYTES)
                .ok_or_else(|| {
                    "ec_ivf columnar frozen list rerank tid bytes overflow".to_owned()
                })?,
        )
        .ok_or_else(|| "ec_ivf columnar frozen list rerank tid offset overflow".to_owned())?;
    let deleted_bitmap_offset = offset;
    offset = offset
        .checked_add(dense_deleted_bitmap_len(posting_count))
        .ok_or_else(|| "ec_ivf columnar frozen list deleted bitmap offset overflow".to_owned())?;

    let to_u32 = |value: usize, name: &str| {
        u32::try_from(value).map_err(|_| format!("ec_ivf columnar frozen list {name} exceeds u32"))
    };
    Ok(ColumnarFrozenListOffsets {
        gamma_offset: to_u32(gamma_offset, "gamma offset")?,
        payload_offset: to_u32(payload_offset, "payload offset")?,
        heap_tid_count_offset: to_u32(heap_tid_count_offset, "heap tid count offset")?,
        heap_tid_offset_offset: to_u32(heap_tid_offset_offset, "heap tid offset offset")?,
        heap_tid_offset: to_u32(heap_tid_offset, "heap tid offset")?,
        rerank_tid_offset: to_u32(rerank_tid_offset, "rerank tid offset")?,
        deleted_bitmap_offset: to_u32(deleted_bitmap_offset, "deleted bitmap offset")?,
        total_column_bytes: to_u32(offset, "total column bytes")?,
    })
}

fn validate_columnar_frozen_list_shape(
    posting_count: u32,
    payload_len: u16,
    total_heap_tids: u32,
    first_column_block: BlockRef,
    last_column_block: BlockRef,
) -> Result<(), String> {
    if posting_count == 0 {
        return Err("ec_ivf columnar frozen list header posting count is zero".to_owned());
    }
    if payload_len == 0 {
        return Err("ec_ivf columnar frozen list header payload length is zero".to_owned());
    }
    if total_heap_tids < posting_count {
        return Err(
            "ec_ivf columnar frozen list header heap tid count is smaller than posting count"
                .to_owned(),
        );
    }
    validate_columnar_frozen_list_block_range(first_column_block, last_column_block)
}

fn validate_columnar_frozen_list_block_range(
    first_column_block: BlockRef,
    last_column_block: BlockRef,
) -> Result<(), String> {
    if first_column_block == BlockRef::INVALID || last_column_block == BlockRef::INVALID {
        return Err("ec_ivf columnar frozen list column block range is invalid".to_owned());
    }
    if first_column_block.block_number > last_column_block.block_number {
        return Err("ec_ivf columnar frozen list column block range is inverted".to_owned());
    }
    Ok(())
}

fn columnar_page_chunks(
    bytes: &[u8],
    item_width: usize,
    page_size: usize,
) -> Result<Vec<IvfColumnarFrozenListPageChunk<'_>>, String> {
    let chunk_lengths = columnar_page_chunk_lengths(bytes.len(), item_width, page_size)?;
    let mut chunks = Vec::with_capacity(chunk_lengths.len());
    let mut start_byte = 0_usize;
    let mut start_item = 0_usize;
    for chunk_len in chunk_lengths {
        let end_byte = start_byte + chunk_len;
        let item_count = chunk_len / item_width;
        chunks.push(IvfColumnarFrozenListPageChunk {
            start_item,
            item_count,
            bytes: &bytes[start_byte..end_byte],
        });
        start_byte = end_byte;
        start_item += item_count;
    }
    Ok(chunks)
}

fn columnar_page_chunk_lengths(
    byte_len: usize,
    item_width: usize,
    page_size: usize,
) -> Result<Vec<usize>, String> {
    if item_width == 0 {
        return Err("ec_ivf columnar frozen list item width is zero".to_owned());
    }
    if byte_len % item_width != 0 {
        return Err("ec_ivf columnar frozen list bytes are not item-aligned".to_owned());
    }
    let item_capacity = columnar_frozen_list_raw_page_capacity(page_size) / item_width;
    if item_capacity == 0 {
        return Err(format!(
            "ec_ivf columnar frozen list item width {item_width} does not fit on a page"
        ));
    }

    let chunk_bytes = item_capacity * item_width;
    let mut remaining = byte_len;
    let mut lengths = Vec::new();
    while remaining > 0 {
        let chunk_len = remaining.min(chunk_bytes);
        lengths.push(chunk_len);
        remaining -= chunk_len;
    }
    Ok(lengths)
}

fn columnar_single_page_range(
    page_lengths: &[usize],
    logical_start: usize,
    len: usize,
) -> Result<IvfColumnarRawPageRange, String> {
    let logical_end = logical_start
        .checked_add(len)
        .ok_or_else(|| "ec_ivf columnar logical byte range overflow".to_owned())?;
    let mut page_start = 0_usize;
    for (page_index, page_len) in page_lengths.iter().copied().enumerate() {
        let page_end = page_start
            .checked_add(page_len)
            .ok_or_else(|| "ec_ivf columnar raw page length overflow".to_owned())?;
        if logical_start >= page_start && logical_start < page_end {
            if logical_end <= page_end {
                return Ok(IvfColumnarRawPageRange {
                    page_index,
                    page_offset: logical_start - page_start,
                });
            }
            return Err(format!(
                "ec_ivf columnar logical byte range [{logical_start}, {logical_end}) crosses raw page boundary at {page_end}"
            ));
        }
        page_start = page_end;
    }
    if len == 0 && logical_start == page_start {
        return Ok(IvfColumnarRawPageRange {
            page_index: page_lengths.len(),
            page_offset: 0,
        });
    }
    Err(format!(
        "ec_ivf columnar logical byte range [{logical_start}, {logical_end}) is outside {} raw bytes",
        page_start
    ))
}

pub(super) fn columnar_frozen_list_raw_page_lengths(
    header: IvfColumnarFrozenListHeaderRef,
    page_size: usize,
) -> Result<Vec<usize>, String> {
    header.validate()?;
    columnar_frozen_list_raw_page_lengths_for_shape(
        header.posting_count as usize,
        header.payload_len as usize,
        header.total_heap_tids as usize,
        page_size,
    )
}

fn columnar_frozen_list_raw_page_lengths_for_shape(
    posting_count: usize,
    payload_len: usize,
    total_heap_tids: usize,
    page_size: usize,
) -> Result<Vec<usize>, String> {
    let mut lengths = Vec::new();
    lengths.extend(columnar_page_chunk_lengths(
        posting_count * size_of::<f32>(),
        size_of::<f32>(),
        page_size,
    )?);
    lengths.extend(columnar_page_chunk_lengths(
        posting_count * payload_len,
        payload_len,
        page_size,
    )?);
    lengths.extend(columnar_page_chunk_lengths(
        posting_count * size_of::<u16>(),
        size_of::<u16>(),
        page_size,
    )?);
    lengths.extend(columnar_page_chunk_lengths(
        posting_count * size_of::<u32>(),
        size_of::<u32>(),
        page_size,
    )?);
    lengths.extend(columnar_page_chunk_lengths(
        total_heap_tids * ITEM_POINTER_BYTES,
        ITEM_POINTER_BYTES,
        page_size,
    )?);
    lengths.extend(columnar_page_chunk_lengths(
        posting_count * ITEM_POINTER_BYTES,
        ITEM_POINTER_BYTES,
        page_size,
    )?);
    lengths.extend(columnar_page_chunk_lengths(
        dense_deleted_bitmap_len(posting_count),
        1,
        page_size,
    )?);
    Ok(lengths)
}

fn native_le_f32_slice(bytes: &[u8]) -> Option<&[f32]> {
    if !cfg!(target_endian = "little") || bytes.len() % size_of::<f32>() != 0 {
        return None;
    }
    let ptr = bytes.as_ptr().cast::<f32>();
    if !ptr.is_aligned() {
        return None;
    }
    // SAFETY: little-endian host byte order matches durable LE order; the
    // caller validated length as a whole number of f32 values; f32 has no
    // invalid bit patterns; and alignment was checked above.
    Some(unsafe { std::slice::from_raw_parts(ptr, bytes.len() / size_of::<f32>()) })
}

fn native_le_u16_slice(bytes: &[u8]) -> Option<&[u16]> {
    if !cfg!(target_endian = "little") || bytes.len() % size_of::<u16>() != 0 {
        return None;
    }
    let ptr = bytes.as_ptr().cast::<u16>();
    if !ptr.is_aligned() {
        return None;
    }
    // SAFETY: little-endian host byte order matches durable LE order; length
    // and alignment were checked; u16 has no invalid bit patterns.
    Some(unsafe { std::slice::from_raw_parts(ptr, bytes.len() / size_of::<u16>()) })
}

fn native_le_u32_slice(bytes: &[u8]) -> Option<&[u32]> {
    if !cfg!(target_endian = "little") || bytes.len() % size_of::<u32>() != 0 {
        return None;
    }
    let ptr = bytes.as_ptr().cast::<u32>();
    if !ptr.is_aligned() {
        return None;
    }
    // SAFETY: little-endian host byte order matches durable LE order; length
    // and alignment were checked; u32 has no invalid bit patterns.
    Some(unsafe { std::slice::from_raw_parts(ptr, bytes.len() / size_of::<u32>()) })
}

#[derive(Debug, Clone, Copy)]
pub(super) enum IvfPostingEntryRef<'a> {
    Row(IvfPostingTupleRef<'a>),
    DenseBlock(IvfDensePostingBlockRef<'a>),
    DensePackedSegment(IvfDensePostingPackedSegmentRef<'a>),
    DensePackedContinuation(IvfDensePostingPackedContinuationRef<'a>),
    ColumnarHeader(IvfColumnarFrozenListHeaderRef),
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IvfPostingTupleRef<'a> {
    pub(super) list_id: u32,
    pub(super) deleted: bool,
    heaptid_bytes: &'a [u8],
    heaptid_count: usize,
    pub(super) gamma: f32,
    pub(super) rerank_tid: ItemPointer,
    pub(super) payload: &'a [u8],
}

impl<'a> IvfDensePostingBlockRef<'a> {
    pub(super) fn decode(input: &'a [u8], payload_len: usize) -> Result<Self, String> {
        if input.len() < DENSE_POSTING_BLOCK_HEADER_BYTES {
            return Err(format!(
                "ec_ivf dense posting block length mismatch: got {}, expected at least {DENSE_POSTING_BLOCK_HEADER_BYTES}",
                input.len()
            ));
        }
        let is_aligned_layout = input[0] == IVF_DENSE_POSTING_ALIGNED_BLOCK_TAG;
        if input[0] != IVF_DENSE_POSTING_BLOCK_TAG && !is_aligned_layout {
            return Err(format!(
                "invalid ec_ivf dense posting block tag: {}",
                input[0]
            ));
        }
        let count = u16::from_le_bytes(
            input[5..7]
                .try_into()
                .expect("dense block count slice should be 2 bytes"),
        ) as usize;
        let stored_payload_len = u16::from_le_bytes(
            input[7..9]
                .try_into()
                .expect("dense block payload length slice should be 2 bytes"),
        ) as usize;
        if stored_payload_len != payload_len {
            return Err(format!(
                "ec_ivf dense posting block payload length mismatch: got {stored_payload_len}, expected {payload_len}"
            ));
        }
        let total_heap_tids = u32::from_le_bytes(
            input[9..13]
                .try_into()
                .expect("dense block heap tid count slice should be 4 bytes"),
        ) as usize;
        let deleted_bitmap_len = dense_deleted_bitmap_len(count);
        let gamma_start;
        let gamma_end;
        let heap_tid_count_start;
        let heap_tid_count_end;
        let heap_tid_offset_start;
        let heap_tid_offset_end;
        let deleted_bitmap_start;
        let deleted_bitmap_end;
        if is_aligned_layout {
            gamma_start = DENSE_POSTING_BLOCK_HEADER_BYTES;
            gamma_end = gamma_start + count * size_of::<f32>();
            heap_tid_offset_start = gamma_end;
            heap_tid_offset_end = heap_tid_offset_start + count * size_of::<u32>();
            heap_tid_count_start = heap_tid_offset_end;
            heap_tid_count_end = heap_tid_count_start + count * size_of::<u16>();
            deleted_bitmap_start = heap_tid_count_end;
            deleted_bitmap_end = deleted_bitmap_start + deleted_bitmap_len;
        } else {
            deleted_bitmap_start = DENSE_POSTING_BLOCK_HEADER_BYTES;
            deleted_bitmap_end = deleted_bitmap_start + deleted_bitmap_len;
            gamma_start = deleted_bitmap_end;
            gamma_end = gamma_start + count * size_of::<f32>();
            heap_tid_count_start = gamma_end;
            heap_tid_count_end = heap_tid_count_start + count * size_of::<u16>();
            heap_tid_offset_start = heap_tid_count_end;
            heap_tid_offset_end = heap_tid_offset_start + count * size_of::<u32>();
        }
        let rerank_tid_start = if is_aligned_layout {
            deleted_bitmap_end
        } else {
            heap_tid_offset_end
        };
        let rerank_tid_end = rerank_tid_start + count * ITEM_POINTER_BYTES;
        let heap_tid_end = rerank_tid_end + total_heap_tids * ITEM_POINTER_BYTES;
        let payload_end = heap_tid_end + count * payload_len;
        if input.len() != payload_end {
            return Err(format!(
                "ec_ivf dense posting block length mismatch: got {}, expected {payload_end}",
                input.len()
            ));
        }
        Ok(Self {
            list_id: u32::from_le_bytes(
                input[1..5]
                    .try_into()
                    .expect("dense block list id slice should be 4 bytes"),
            ),
            count,
            total_heap_tids,
            payload_len,
            deleted_bitmap: &input[deleted_bitmap_start..deleted_bitmap_end],
            gamma_bytes: &input[gamma_start..gamma_end],
            heap_tid_count_bytes: &input[heap_tid_count_start..heap_tid_count_end],
            heap_tid_offset_bytes: &input[heap_tid_offset_start..heap_tid_offset_end],
            rerank_tid_bytes: &input[rerank_tid_start..rerank_tid_end],
            heap_tid_bytes: &input[rerank_tid_end..heap_tid_end],
            payloads: &input[heap_tid_end..payload_end],
        })
    }

    pub(super) fn len(&self) -> usize {
        self.count
    }

    pub(super) fn total_heap_tids(&self) -> usize {
        self.total_heap_tids
    }

    pub(super) fn payload_len(&self) -> usize {
        self.payload_len
    }

    pub(super) fn deleted_bitmap(&self) -> &'a [u8] {
        self.deleted_bitmap
    }

    pub(super) fn is_deleted(&self, index: usize) -> bool {
        dense_deleted_bitmap_get(self.deleted_bitmap, index)
    }

    pub(super) fn gammas(&self) -> Vec<f32> {
        self.gamma_bytes
            .chunks_exact(size_of::<f32>())
            .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("validated gamma chunk")))
            .collect()
    }

    pub(super) fn gammas_native_le(&self) -> Option<&'a [f32]> {
        native_le_f32_slice(self.gamma_bytes)
    }

    pub(super) fn heap_tid_counts_native_le(&self) -> Option<&'a [u16]> {
        native_le_u16_slice(self.heap_tid_count_bytes)
    }

    pub(super) fn heap_tid_offsets_native_le(&self) -> Option<&'a [u32]> {
        native_le_u32_slice(self.heap_tid_offset_bytes)
    }

    pub(super) fn copy_gammas_to(&self, out: &mut Vec<f32>) {
        out.clear();
        out.reserve(self.count);
        out.extend(
            self.gamma_bytes
                .chunks_exact(size_of::<f32>())
                .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("validated gamma chunk"))),
        );
    }

    pub(super) fn gamma(&self, index: usize) -> f32 {
        let start = index * size_of::<f32>();
        f32::from_le_bytes(
            self.gamma_bytes[start..start + size_of::<f32>()]
                .try_into()
                .expect("validated gamma chunk"),
        )
    }

    pub(super) fn heap_tid_count(&self, index: usize) -> usize {
        let start = index * size_of::<u16>();
        u16::from_le_bytes(
            self.heap_tid_count_bytes[start..start + size_of::<u16>()]
                .try_into()
                .expect("validated heap tid count chunk"),
        ) as usize
    }

    pub(super) fn heap_tid_offset(&self, index: usize) -> usize {
        let start = index * size_of::<u32>();
        u32::from_le_bytes(
            self.heap_tid_offset_bytes[start..start + size_of::<u32>()]
                .try_into()
                .expect("validated heap tid offset chunk"),
        ) as usize
    }

    pub(super) fn heap_tids(&self, index: usize) -> impl Iterator<Item = ItemPointer> + '_ {
        let start = self.heap_tid_offset(index);
        let count = self.heap_tid_count(index);
        self.heap_tid_bytes[start * ITEM_POINTER_BYTES..(start + count) * ITEM_POINTER_BYTES]
            .chunks_exact(ITEM_POINTER_BYTES)
            .map(|chunk| ItemPointer::decode(chunk).expect("validated dense block tid bytes"))
    }

    pub(super) fn payload(&self, index: usize) -> &[u8] {
        let start = index * self.payload_len;
        &self.payloads[start..start + self.payload_len]
    }

    pub(super) fn validate_offsets(&self) -> Result<(), String> {
        for index in 0..self.count {
            let start = self.heap_tid_offset(index);
            let count = self.heap_tid_count(index);
            if start
                .checked_add(count)
                .is_none_or(|end| end > self.total_heap_tids)
            {
                return Err("ec_ivf dense posting block heap tid range is out of bounds".to_owned());
            }
            let rerank_start = index * ITEM_POINTER_BYTES;
            ItemPointer::decode(
                &self.rerank_tid_bytes[rerank_start..rerank_start + ITEM_POINTER_BYTES],
            )?;
        }
        Ok(())
    }
}

impl<'a> IvfDensePostingPackedSegmentRef<'a> {
    pub(super) fn decode(input: &'a [u8], payload_len: usize) -> Result<Self, String> {
        if input.len() < DENSE_POSTING_PACKED_SEGMENT_HEADER_BYTES {
            return Err(format!(
                "ec_ivf dense posting packed segment length mismatch: got {}, expected at least {DENSE_POSTING_PACKED_SEGMENT_HEADER_BYTES}",
                input.len()
            ));
        }
        if input[0] != IVF_DENSE_POSTING_PACKED_SEGMENT_TAG {
            return Err(format!(
                "invalid ec_ivf dense posting packed segment tag: {}",
                input[0]
            ));
        }
        let count = u16::from_le_bytes(
            input[15..17]
                .try_into()
                .expect("packed dense segment count slice should be 2 bytes"),
        ) as usize;
        let stored_payload_len = u16::from_le_bytes(
            input[17..19]
                .try_into()
                .expect("packed dense segment payload length slice should be 2 bytes"),
        ) as usize;
        if stored_payload_len != payload_len {
            return Err(format!(
                "ec_ivf dense posting packed segment payload length mismatch: got {stored_payload_len}, expected {payload_len}"
            ));
        }
        let total_heap_tids = u32::from_le_bytes(
            input[19..23]
                .try_into()
                .expect("packed dense segment heap tid count slice should be 4 bytes"),
        ) as usize;
        let header_payload_len = u32::from_le_bytes(
            input[23..27]
                .try_into()
                .expect("packed dense segment payload byte count slice should be 4 bytes"),
        ) as usize;
        let deleted_bitmap_len = dense_deleted_bitmap_len(count);
        let gamma_start = DENSE_POSTING_PACKED_SEGMENT_HEADER_BYTES;
        let gamma_end = gamma_start + count * size_of::<f32>();
        let heap_tid_offset_start = gamma_end;
        let heap_tid_offset_end = heap_tid_offset_start + count * size_of::<u32>();
        let heap_tid_count_start = heap_tid_offset_end;
        let heap_tid_count_end = heap_tid_count_start + count * size_of::<u16>();
        let deleted_bitmap_start = heap_tid_count_end;
        let deleted_bitmap_end = deleted_bitmap_start + deleted_bitmap_len;
        let rerank_tid_end = deleted_bitmap_end + count * ITEM_POINTER_BYTES;
        let heap_tid_end = rerank_tid_end + total_heap_tids * ITEM_POINTER_BYTES;
        let total_payload_len = count.checked_mul(payload_len).ok_or_else(|| {
            "ec_ivf dense posting packed segment payload length overflow".to_owned()
        })?;
        if header_payload_len > total_payload_len {
            return Err(
                "ec_ivf dense posting packed segment payload bytes exceed logical payload length"
                    .to_owned(),
            );
        }
        let payload_end = heap_tid_end + header_payload_len;
        if input.len() != payload_end {
            return Err(format!(
                "ec_ivf dense posting packed segment length mismatch: got {}, expected {payload_end}",
                input.len()
            ));
        }
        Ok(Self {
            list_id: u32::from_le_bytes(
                input[1..5]
                    .try_into()
                    .expect("packed dense segment list id slice should be 4 bytes"),
            ),
            logical_block_id: u32::from_le_bytes(
                input[5..9]
                    .try_into()
                    .expect("packed dense segment logical block id slice should be 4 bytes"),
            ),
            segment_index: u16::from_le_bytes(
                input[9..11]
                    .try_into()
                    .expect("packed dense segment index slice should be 2 bytes"),
            ),
            segment_count: u16::from_le_bytes(
                input[11..13]
                    .try_into()
                    .expect("packed dense segment count slice should be 2 bytes"),
            ),
            total_posting_count: u16::from_le_bytes(
                input[13..15]
                    .try_into()
                    .expect("packed dense segment total posting count slice should be 2 bytes"),
            ),
            count,
            total_heap_tids,
            payload_len,
            deleted_bitmap: &input[deleted_bitmap_start..deleted_bitmap_end],
            gamma_bytes: &input[gamma_start..gamma_end],
            heap_tid_count_bytes: &input[heap_tid_count_start..heap_tid_count_end],
            heap_tid_offset_bytes: &input[heap_tid_offset_start..heap_tid_offset_end],
            rerank_tid_bytes: &input[deleted_bitmap_end..rerank_tid_end],
            heap_tid_bytes: &input[rerank_tid_end..heap_tid_end],
            payloads: &input[heap_tid_end..payload_end],
        })
    }

    pub(super) fn len(&self) -> usize {
        self.count
    }

    pub(super) fn total_heap_tids(&self) -> usize {
        self.total_heap_tids
    }

    pub(super) fn payload_len(&self) -> usize {
        self.payload_len
    }

    pub(super) fn is_deleted(&self, index: usize) -> bool {
        dense_deleted_bitmap_get(self.deleted_bitmap, index)
    }

    pub(super) fn gammas(&self) -> Vec<f32> {
        self.gamma_bytes
            .chunks_exact(size_of::<f32>())
            .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("validated gamma chunk")))
            .collect()
    }

    pub(super) fn gammas_native_le(&self) -> Option<&'a [f32]> {
        native_le_f32_slice(self.gamma_bytes)
    }

    pub(super) fn heap_tid_counts_native_le(&self) -> Option<&'a [u16]> {
        native_le_u16_slice(self.heap_tid_count_bytes)
    }

    pub(super) fn heap_tid_offsets_native_le(&self) -> Option<&'a [u32]> {
        native_le_u32_slice(self.heap_tid_offset_bytes)
    }

    pub(super) fn copy_gammas_to(&self, out: &mut Vec<f32>) {
        out.clear();
        out.reserve(self.count);
        out.extend(
            self.gamma_bytes
                .chunks_exact(size_of::<f32>())
                .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("validated gamma chunk"))),
        );
    }

    pub(super) fn gamma(&self, index: usize) -> f32 {
        let start = index * size_of::<f32>();
        f32::from_le_bytes(
            self.gamma_bytes[start..start + size_of::<f32>()]
                .try_into()
                .expect("validated gamma chunk"),
        )
    }

    pub(super) fn heap_tid_count(&self, index: usize) -> usize {
        let start = index * size_of::<u16>();
        u16::from_le_bytes(
            self.heap_tid_count_bytes[start..start + size_of::<u16>()]
                .try_into()
                .expect("validated heap tid count chunk"),
        ) as usize
    }

    pub(super) fn heap_tid_offset(&self, index: usize) -> usize {
        let start = index * size_of::<u32>();
        u32::from_le_bytes(
            self.heap_tid_offset_bytes[start..start + size_of::<u32>()]
                .try_into()
                .expect("validated heap tid offset chunk"),
        ) as usize
    }

    pub(super) fn heap_tids(&self, index: usize) -> impl Iterator<Item = ItemPointer> + '_ {
        let start = self.heap_tid_offset(index);
        let count = self.heap_tid_count(index);
        self.heap_tid_bytes[start * ITEM_POINTER_BYTES..(start + count) * ITEM_POINTER_BYTES]
            .chunks_exact(ITEM_POINTER_BYTES)
            .map(|chunk| ItemPointer::decode(chunk).expect("validated dense segment tid bytes"))
    }

    pub(super) fn payload(&self, index: usize) -> &[u8] {
        let start = index * self.payload_len;
        &self.payloads[start..start + self.payload_len]
    }

    pub(super) fn validate_offsets(&self) -> Result<(), String> {
        if self.segment_count == 0 {
            return Err("ec_ivf dense posting packed segment count is zero".to_owned());
        }
        if self.segment_index >= self.segment_count {
            return Err("ec_ivf dense posting packed segment index is out of bounds".to_owned());
        }
        if usize::from(self.total_posting_count) < self.count {
            return Err(
                "ec_ivf dense posting packed segment total count is smaller than segment count"
                    .to_owned(),
            );
        }
        for index in 0..self.count {
            let start = self.heap_tid_offset(index);
            let count = self.heap_tid_count(index);
            if start
                .checked_add(count)
                .is_none_or(|end| end > self.total_heap_tids)
            {
                return Err(
                    "ec_ivf dense posting packed segment heap tid range is out of bounds"
                        .to_owned(),
                );
            }
            let rerank_start = index * ITEM_POINTER_BYTES;
            ItemPointer::decode(
                &self.rerank_tid_bytes[rerank_start..rerank_start + ITEM_POINTER_BYTES],
            )?;
        }
        Ok(())
    }
}

impl<'a> IvfDensePostingPackedContinuationRef<'a> {
    pub(super) fn decode(input: &'a [u8]) -> Result<Self, String> {
        if input.len() < DENSE_POSTING_PACKED_CONTINUATION_HEADER_BYTES {
            return Err(format!(
                "ec_ivf dense posting packed continuation length mismatch: got {}, expected at least {DENSE_POSTING_PACKED_CONTINUATION_HEADER_BYTES}",
                input.len()
            ));
        }
        if input[0] != IVF_DENSE_POSTING_PACKED_CONTINUATION_TAG {
            return Err(format!(
                "invalid ec_ivf dense posting packed continuation tag: {}",
                input[0]
            ));
        }
        let payload_len = u32::from_le_bytes(
            input[17..21]
                .try_into()
                .expect("packed dense continuation payload length slice should be 4 bytes"),
        ) as usize;
        let payload_start = DENSE_POSTING_PACKED_CONTINUATION_HEADER_BYTES;
        let payload_end = payload_start + payload_len;
        if input.len() != payload_end {
            return Err(format!(
                "ec_ivf dense posting packed continuation length mismatch: got {}, expected {payload_end}",
                input.len()
            ));
        }
        Ok(Self {
            list_id: u32::from_le_bytes(
                input[1..5]
                    .try_into()
                    .expect("packed dense continuation list id slice should be 4 bytes"),
            ),
            logical_block_id: u32::from_le_bytes(
                input[5..9]
                    .try_into()
                    .expect("packed dense continuation logical block id slice should be 4 bytes"),
            ),
            segment_index: u16::from_le_bytes(
                input[9..11]
                    .try_into()
                    .expect("packed dense continuation index slice should be 2 bytes"),
            ),
            segment_count: u16::from_le_bytes(
                input[11..13]
                    .try_into()
                    .expect("packed dense continuation count slice should be 2 bytes"),
            ),
            payload_offset: u32::from_le_bytes(
                input[13..17]
                    .try_into()
                    .expect("packed dense continuation payload offset slice should be 4 bytes"),
            ),
            payloads: &input[payload_start..payload_end],
        })
    }
}

impl<'a> IvfDensePostingRef<'a> {
    pub(super) fn list_id(self) -> u32 {
        match self {
            Self::Block(block) => block.list_id,
            Self::PackedSegment(segment) => segment.list_id,
        }
    }

    pub(super) fn len(self) -> usize {
        match self {
            Self::Block(block) => block.len(),
            Self::PackedSegment(segment) => segment.len(),
        }
    }

    pub(super) fn payload_len(self) -> usize {
        match self {
            Self::Block(block) => block.payload_len(),
            Self::PackedSegment(segment) => segment.payload_len(),
        }
    }

    pub(super) fn payloads(self) -> &'a [u8] {
        match self {
            Self::Block(block) => block.payloads,
            Self::PackedSegment(segment) => segment.payloads,
        }
    }

    pub(super) fn is_deleted(self, index: usize) -> bool {
        match self {
            Self::Block(block) => block.is_deleted(index),
            Self::PackedSegment(segment) => segment.is_deleted(index),
        }
    }

    pub(super) fn copy_gammas_to(self, out: &mut Vec<f32>) {
        match self {
            Self::Block(block) => block.copy_gammas_to(out),
            Self::PackedSegment(segment) => segment.copy_gammas_to(out),
        }
    }

    pub(super) fn gammas_native_le(self) -> Option<&'a [f32]> {
        match self {
            Self::Block(block) => block.gammas_native_le(),
            Self::PackedSegment(segment) => segment.gammas_native_le(),
        }
    }

    pub(super) fn gamma(self, index: usize) -> f32 {
        match self {
            Self::Block(block) => block.gamma(index),
            Self::PackedSegment(segment) => segment.gamma(index),
        }
    }

    pub(super) fn heap_tid_count(self, index: usize) -> usize {
        match self {
            Self::Block(block) => block.heap_tid_count(index),
            Self::PackedSegment(segment) => segment.heap_tid_count(index),
        }
    }

    pub(super) fn heap_tids(self, index: usize) -> IvfDensePostingHeapTids<'a> {
        let bytes = match self {
            Self::Block(block) => {
                let start = block.heap_tid_offset(index);
                let count = block.heap_tid_count(index);
                &block.heap_tid_bytes
                    [start * ITEM_POINTER_BYTES..(start + count) * ITEM_POINTER_BYTES]
            }
            Self::PackedSegment(segment) => {
                let start = segment.heap_tid_offset(index);
                let count = segment.heap_tid_count(index);
                &segment.heap_tid_bytes
                    [start * ITEM_POINTER_BYTES..(start + count) * ITEM_POINTER_BYTES]
            }
        };
        IvfDensePostingHeapTids {
            chunks: bytes.chunks_exact(ITEM_POINTER_BYTES),
        }
    }

    pub(super) fn payload(self, index: usize) -> &'a [u8] {
        match self {
            Self::Block(block) => {
                let start = index * block.payload_len;
                &block.payloads[start..start + block.payload_len]
            }
            Self::PackedSegment(segment) => {
                let start = index * segment.payload_len;
                &segment.payloads[start..start + segment.payload_len]
            }
        }
    }
}

impl IvfDensePostingBlockTuple {
    pub(super) fn len(&self) -> usize {
        self.gammas.len()
    }

    pub(super) fn is_deleted(&self, index: usize) -> bool {
        dense_deleted_bitmap_get(&self.deleted_bitmap, index)
    }

    pub(super) fn mark_deleted(&mut self, index: usize) {
        dense_deleted_bitmap_set(&mut self.deleted_bitmap, index);
    }

    pub(super) fn from_single_heaptid_postings(
        list_id: u32,
        postings: &[(ItemPointer, f32, ItemPointer, Vec<u8>)],
        payload_len: usize,
    ) -> Result<Self, String> {
        let count = postings.len();
        if count == 0 {
            return Err("ec_ivf dense posting block requires at least one posting".to_owned());
        }
        if count > u16::MAX as usize {
            return Err("ec_ivf dense posting block count exceeds u16".to_owned());
        }
        if payload_len > u16::MAX as usize {
            return Err("ec_ivf dense posting block payload length exceeds u16".to_owned());
        }
        let mut gammas = Vec::with_capacity(count);
        let mut heap_tid_counts = Vec::with_capacity(count);
        let mut heap_tid_offsets = Vec::with_capacity(count);
        let mut rerank_tids = Vec::with_capacity(count);
        let mut heap_tids = Vec::with_capacity(count);
        let mut payloads = Vec::with_capacity(count * payload_len);
        for (heap_tid, gamma, rerank_tid, payload) in postings {
            if !gamma.is_finite() {
                return Err("ec_ivf dense posting block gamma must be finite".to_owned());
            }
            if payload.len() != payload_len {
                return Err(format!(
                    "ec_ivf dense posting block payload length mismatch: got {}, expected {payload_len}",
                    payload.len()
                ));
            }
            heap_tid_offsets.push(u32::try_from(heap_tids.len()).map_err(|_| {
                "ec_ivf dense posting block heap tid offset exceeds u32".to_owned()
            })?);
            heap_tid_counts.push(1);
            heap_tids.push(*heap_tid);
            gammas.push(*gamma);
            rerank_tids.push(*rerank_tid);
            payloads.extend_from_slice(payload);
        }
        Ok(Self {
            list_id,
            gammas,
            heap_tid_counts,
            heap_tid_offsets,
            rerank_tids,
            heap_tids,
            deleted_bitmap: vec![0; dense_deleted_bitmap_len(count)],
            payload_len,
            payloads,
        })
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.encode_with_layout(false)
    }

    pub(super) fn encode_aligned(&self) -> Result<Vec<u8>, String> {
        self.encode_with_layout(true)
    }

    fn encode_with_layout(&self, aligned_layout: bool) -> Result<Vec<u8>, String> {
        if self.gammas.len() != self.heap_tid_counts.len()
            || self.gammas.len() != self.heap_tid_offsets.len()
            || self.gammas.len() != self.rerank_tids.len()
            || self.payloads.len() != self.gammas.len() * self.payload_len
            || self.deleted_bitmap.len() != dense_deleted_bitmap_len(self.gammas.len())
        {
            return Err("ec_ivf dense posting block array length mismatch".to_owned());
        }
        if self.gammas.len() > u16::MAX as usize {
            return Err("ec_ivf dense posting block count exceeds u16".to_owned());
        }
        if self.heap_tids.len() > u32::MAX as usize {
            return Err("ec_ivf dense posting block heap tid count exceeds u32".to_owned());
        }
        if self.payload_len > u16::MAX as usize {
            return Err("ec_ivf dense posting block payload length exceeds u16".to_owned());
        }
        if self.gammas.iter().any(|gamma| !gamma.is_finite()) {
            return Err("ec_ivf dense posting block gamma must be finite".to_owned());
        }

        let mut out = Vec::with_capacity(Self::encoded_len(
            self.gammas.len(),
            self.heap_tids.len(),
            self.payload_len,
        ));
        out.push(if aligned_layout {
            IVF_DENSE_POSTING_ALIGNED_BLOCK_TAG
        } else {
            IVF_DENSE_POSTING_BLOCK_TAG
        });
        out.extend_from_slice(&self.list_id.to_le_bytes());
        out.extend_from_slice(&(self.gammas.len() as u16).to_le_bytes());
        out.extend_from_slice(&(self.payload_len as u16).to_le_bytes());
        out.extend_from_slice(&(self.heap_tids.len() as u32).to_le_bytes());
        out.extend_from_slice(&[0, 0, 0]);
        if aligned_layout {
            for gamma in &self.gammas {
                out.extend_from_slice(&gamma.to_le_bytes());
            }
            for offset in &self.heap_tid_offsets {
                out.extend_from_slice(&offset.to_le_bytes());
            }
            for count in &self.heap_tid_counts {
                out.extend_from_slice(&count.to_le_bytes());
            }
            out.extend_from_slice(&self.deleted_bitmap);
        } else {
            out.extend_from_slice(&self.deleted_bitmap);
            for gamma in &self.gammas {
                out.extend_from_slice(&gamma.to_le_bytes());
            }
            for count in &self.heap_tid_counts {
                out.extend_from_slice(&count.to_le_bytes());
            }
            for offset in &self.heap_tid_offsets {
                out.extend_from_slice(&offset.to_le_bytes());
            }
        }
        for tid in &self.rerank_tids {
            tid.encode_into(&mut out);
        }
        for tid in &self.heap_tids {
            tid.encode_into(&mut out);
        }
        out.extend_from_slice(&self.payloads);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8], payload_len: usize) -> Result<Self, String> {
        let block = IvfDensePostingBlockRef::decode(input, payload_len)?;
        block.validate_offsets()?;
        let mut heap_tid_counts = Vec::with_capacity(block.len());
        let mut heap_tid_offsets = Vec::with_capacity(block.len());
        let mut rerank_tids = Vec::with_capacity(block.len());
        for index in 0..block.len() {
            heap_tid_counts.push(block.heap_tid_count(index) as u16);
            heap_tid_offsets.push(block.heap_tid_offset(index) as u32);
            let start = index * ITEM_POINTER_BYTES;
            rerank_tids.push(ItemPointer::decode(
                &block.rerank_tid_bytes[start..start + ITEM_POINTER_BYTES],
            )?);
        }
        Ok(Self {
            list_id: block.list_id,
            gammas: block.gammas(),
            heap_tid_counts,
            heap_tid_offsets,
            rerank_tids,
            heap_tids: block
                .heap_tid_bytes
                .chunks_exact(ITEM_POINTER_BYTES)
                .map(ItemPointer::decode)
                .collect::<Result<Vec<_>, _>>()?,
            deleted_bitmap: block.deleted_bitmap().to_vec(),
            payload_len: block.payload_len,
            payloads: block.payloads.to_vec(),
        })
    }

    pub(super) const fn encoded_len(
        count: usize,
        total_heap_tids: usize,
        payload_len: usize,
    ) -> usize {
        DENSE_POSTING_BLOCK_HEADER_BYTES
            + dense_deleted_bitmap_len(count)
            + count * size_of::<f32>()
            + count * size_of::<u16>()
            + count * size_of::<u32>()
            + count * ITEM_POINTER_BYTES
            + total_heap_tids * ITEM_POINTER_BYTES
            + count * payload_len
    }
}

const fn dense_deleted_bitmap_len(count: usize) -> usize {
    (count + 7) / 8
}

fn dense_deleted_bitmap_get(bitmap: &[u8], index: usize) -> bool {
    bitmap
        .get(index / 8)
        .is_some_and(|byte| byte & (1 << (index % 8)) != 0)
}

fn dense_deleted_bitmap_set(bitmap: &mut [u8], index: usize) {
    if let Some(byte) = bitmap.get_mut(index / 8) {
        *byte |= 1 << (index % 8);
    }
}

impl<'a> IvfPostingTupleRef<'a> {
    pub(super) fn decode(input: &'a [u8], payload_len: usize) -> Result<Self, String> {
        let expected_len = IvfPostingTuple::encoded_len(payload_len);
        if input.len() != expected_len {
            return Err(format!(
                "ec_ivf posting tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if input[0] != IVF_POSTING_TAG {
            return Err(format!("invalid ec_ivf posting tuple tag: {}", input[0]));
        }

        let flags = input[5];
        if flags & !POSTING_FLAG_DELETED != 0 {
            return Err(format!("invalid ec_ivf posting tuple flags: {flags:#x}"));
        }
        let heaptid_count = input[6] as usize;
        if heaptid_count > HEAPTID_INLINE_CAPACITY {
            return Err(format!(
                "invalid ec_ivf posting heap tid count: got {heaptid_count}, max {}",
                HEAPTID_INLINE_CAPACITY
            ));
        }

        let heaptid_start = 7;
        let heaptid_end = heaptid_start + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES;
        let gamma = f32::from_le_bytes(
            input[heaptid_end..heaptid_end + 4]
                .try_into()
                .expect("posting gamma slice should be 4 bytes"),
        );
        let rerank_start = heaptid_end + 4;
        let payload_start = rerank_start + ITEM_POINTER_BYTES;

        Ok(Self {
            list_id: u32::from_le_bytes(
                input[1..5]
                    .try_into()
                    .expect("posting list id slice should be 4 bytes"),
            ),
            deleted: flags & POSTING_FLAG_DELETED != 0,
            heaptid_bytes: &input[heaptid_start..heaptid_end],
            heaptid_count,
            gamma,
            rerank_tid: ItemPointer::decode(&input[rerank_start..payload_start])?,
            payload: &input[payload_start..],
        })
    }

    pub(super) fn heaptid_count(&self) -> usize {
        self.heaptid_count
    }

    pub(super) fn heaptids(&self) -> impl Iterator<Item = ItemPointer> + '_ {
        self.heaptid_bytes
            .chunks_exact(ITEM_POINTER_BYTES)
            .take(self.heaptid_count)
            .map(|chunk| {
                ItemPointer::decode(chunk)
                    .expect("borrowed ec_ivf posting tuple should expose validated tid bytes")
            })
    }

    pub(super) fn collect_heaptids(&self) -> Vec<ItemPointer> {
        self.heaptids().collect()
    }
}

impl IvfDensePostingPackedSegmentTuple {
    pub(super) fn len(&self) -> usize {
        self.gammas.len()
    }

    pub(super) fn is_deleted(&self, index: usize) -> bool {
        dense_deleted_bitmap_get(&self.deleted_bitmap, index)
    }

    pub(super) fn mark_deleted(&mut self, index: usize) {
        dense_deleted_bitmap_set(&mut self.deleted_bitmap, index);
    }

    pub(super) fn from_single_heaptid_postings(
        list_id: u32,
        logical_block_id: u32,
        segment_index: u16,
        segment_count: u16,
        total_posting_count: u16,
        postings: &[(ItemPointer, f32, ItemPointer, Vec<u8>)],
        payload_len: usize,
    ) -> Result<Self, String> {
        let count = postings.len();
        if count == 0 {
            return Err(
                "ec_ivf dense posting packed segment requires at least one posting".to_owned(),
            );
        }
        if segment_count == 0 || segment_index >= segment_count {
            return Err("ec_ivf dense posting packed segment ordinal is invalid".to_owned());
        }
        if count > u16::MAX as usize {
            return Err("ec_ivf dense posting packed segment count exceeds u16".to_owned());
        }
        if payload_len > u16::MAX as usize {
            return Err(
                "ec_ivf dense posting packed segment payload length exceeds u16".to_owned(),
            );
        }
        let mut gammas = Vec::with_capacity(count);
        let mut heap_tid_counts = Vec::with_capacity(count);
        let mut heap_tid_offsets = Vec::with_capacity(count);
        let mut rerank_tids = Vec::with_capacity(count);
        let mut heap_tids = Vec::with_capacity(count);
        let mut payloads = Vec::with_capacity(count * payload_len);
        for (heap_tid, gamma, rerank_tid, payload) in postings {
            if !gamma.is_finite() {
                return Err("ec_ivf dense posting packed segment gamma must be finite".to_owned());
            }
            if payload.len() != payload_len {
                return Err(format!(
                    "ec_ivf dense posting packed segment payload length mismatch: got {}, expected {payload_len}",
                    payload.len()
                ));
            }
            heap_tid_offsets.push(u32::try_from(heap_tids.len()).map_err(|_| {
                "ec_ivf dense posting packed segment heap tid offset exceeds u32".to_owned()
            })?);
            heap_tid_counts.push(1);
            heap_tids.push(*heap_tid);
            gammas.push(*gamma);
            rerank_tids.push(*rerank_tid);
            payloads.extend_from_slice(payload);
        }
        Ok(Self {
            list_id,
            logical_block_id,
            segment_index,
            segment_count,
            total_posting_count,
            gammas,
            heap_tid_counts,
            heap_tid_offsets,
            rerank_tids,
            heap_tids,
            deleted_bitmap: vec![0; dense_deleted_bitmap_len(count)],
            payload_len,
            payloads,
        })
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        if self.gammas.len() != self.heap_tid_counts.len()
            || self.gammas.len() != self.heap_tid_offsets.len()
            || self.gammas.len() != self.rerank_tids.len()
            || self.payloads.len() > self.gammas.len() * self.payload_len
            || self.deleted_bitmap.len() != dense_deleted_bitmap_len(self.gammas.len())
        {
            return Err("ec_ivf dense posting packed segment array length mismatch".to_owned());
        }
        if self.segment_count == 0 || self.segment_index >= self.segment_count {
            return Err("ec_ivf dense posting packed segment ordinal is invalid".to_owned());
        }
        if self.gammas.len() > u16::MAX as usize {
            return Err("ec_ivf dense posting packed segment count exceeds u16".to_owned());
        }
        if self.heap_tids.len() > u32::MAX as usize {
            return Err(
                "ec_ivf dense posting packed segment heap tid count exceeds u32".to_owned(),
            );
        }
        if self.payload_len > u16::MAX as usize {
            return Err(
                "ec_ivf dense posting packed segment payload length exceeds u16".to_owned(),
            );
        }
        if self.gammas.iter().any(|gamma| !gamma.is_finite()) {
            return Err("ec_ivf dense posting packed segment gamma must be finite".to_owned());
        }

        let mut out = Vec::with_capacity(Self::encoded_len(
            self.gammas.len(),
            self.heap_tids.len(),
            self.payload_len,
        ));
        out.push(IVF_DENSE_POSTING_PACKED_SEGMENT_TAG);
        out.extend_from_slice(&self.list_id.to_le_bytes());
        out.extend_from_slice(&self.logical_block_id.to_le_bytes());
        out.extend_from_slice(&self.segment_index.to_le_bytes());
        out.extend_from_slice(&self.segment_count.to_le_bytes());
        out.extend_from_slice(&self.total_posting_count.to_le_bytes());
        out.extend_from_slice(&(self.gammas.len() as u16).to_le_bytes());
        out.extend_from_slice(&(self.payload_len as u16).to_le_bytes());
        out.extend_from_slice(&(self.heap_tids.len() as u32).to_le_bytes());
        out.extend_from_slice(&(self.payloads.len() as u32).to_le_bytes());
        out.push(0);
        for gamma in &self.gammas {
            out.extend_from_slice(&gamma.to_le_bytes());
        }
        for offset in &self.heap_tid_offsets {
            out.extend_from_slice(&offset.to_le_bytes());
        }
        for count in &self.heap_tid_counts {
            out.extend_from_slice(&count.to_le_bytes());
        }
        out.extend_from_slice(&self.deleted_bitmap);
        for tid in &self.rerank_tids {
            tid.encode_into(&mut out);
        }
        for tid in &self.heap_tids {
            tid.encode_into(&mut out);
        }
        out.extend_from_slice(&self.payloads);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8], payload_len: usize) -> Result<Self, String> {
        let segment = IvfDensePostingPackedSegmentRef::decode(input, payload_len)?;
        segment.validate_offsets()?;
        let mut heap_tid_counts = Vec::with_capacity(segment.len());
        let mut heap_tid_offsets = Vec::with_capacity(segment.len());
        let mut rerank_tids = Vec::with_capacity(segment.len());
        for index in 0..segment.len() {
            heap_tid_counts.push(segment.heap_tid_count(index) as u16);
            heap_tid_offsets.push(segment.heap_tid_offset(index) as u32);
            let start = index * ITEM_POINTER_BYTES;
            rerank_tids.push(ItemPointer::decode(
                &segment.rerank_tid_bytes[start..start + ITEM_POINTER_BYTES],
            )?);
        }
        Ok(Self {
            list_id: segment.list_id,
            logical_block_id: segment.logical_block_id,
            segment_index: segment.segment_index,
            segment_count: segment.segment_count,
            total_posting_count: segment.total_posting_count,
            gammas: segment.gammas(),
            heap_tid_counts,
            heap_tid_offsets,
            rerank_tids,
            heap_tids: segment
                .heap_tid_bytes
                .chunks_exact(ITEM_POINTER_BYTES)
                .map(ItemPointer::decode)
                .collect::<Result<Vec<_>, _>>()?,
            deleted_bitmap: segment.deleted_bitmap.to_vec(),
            payload_len: segment.payload_len,
            payloads: segment.payloads.to_vec(),
        })
    }

    pub(super) const fn encoded_len(
        count: usize,
        total_heap_tids: usize,
        payload_len: usize,
    ) -> usize {
        Self::encoded_len_with_payload_bytes(count, total_heap_tids, count * payload_len)
    }

    pub(super) const fn encoded_len_with_payload_bytes(
        count: usize,
        total_heap_tids: usize,
        payload_bytes: usize,
    ) -> usize {
        DENSE_POSTING_PACKED_SEGMENT_HEADER_BYTES
            + count * size_of::<f32>()
            + count * size_of::<u32>()
            + count * size_of::<u16>()
            + dense_deleted_bitmap_len(count)
            + count * ITEM_POINTER_BYTES
            + total_heap_tids * ITEM_POINTER_BYTES
            + payload_bytes
    }
}

impl IvfDensePostingPackedContinuationTuple {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        if self.segment_count == 0
            || self.segment_index == 0
            || self.segment_index >= self.segment_count
        {
            return Err("ec_ivf dense posting packed continuation ordinal is invalid".to_owned());
        }
        if self.payloads.len() > u32::MAX as usize {
            return Err(
                "ec_ivf dense posting packed continuation payload length exceeds u32".to_owned(),
            );
        }
        let mut out = Vec::with_capacity(Self::encoded_len(self.payloads.len()));
        out.push(IVF_DENSE_POSTING_PACKED_CONTINUATION_TAG);
        out.extend_from_slice(&self.list_id.to_le_bytes());
        out.extend_from_slice(&self.logical_block_id.to_le_bytes());
        out.extend_from_slice(&self.segment_index.to_le_bytes());
        out.extend_from_slice(&self.segment_count.to_le_bytes());
        out.extend_from_slice(&self.payload_offset.to_le_bytes());
        out.extend_from_slice(&(self.payloads.len() as u32).to_le_bytes());
        out.extend_from_slice(&[0, 0, 0]);
        out.extend_from_slice(&self.payloads);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        let continuation = IvfDensePostingPackedContinuationRef::decode(input)?;
        Ok(Self {
            list_id: continuation.list_id,
            logical_block_id: continuation.logical_block_id,
            segment_index: continuation.segment_index,
            segment_count: continuation.segment_count,
            payload_offset: continuation.payload_offset,
            payloads: continuation.payloads.to_vec(),
        })
    }

    pub(super) const fn encoded_len(payload_bytes: usize) -> usize {
        DENSE_POSTING_PACKED_CONTINUATION_HEADER_BYTES + payload_bytes
    }
}

impl IvfPostingTuple {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        if self.heaptids.len() > HEAPTID_INLINE_CAPACITY {
            return Err(format!(
                "too many ec_ivf posting heap tids: got {}, max {}",
                self.heaptids.len(),
                HEAPTID_INLINE_CAPACITY
            ));
        }
        if !self.gamma.is_finite() {
            return Err("ec_ivf posting tuple gamma must be finite".into());
        }

        let mut out = Vec::with_capacity(Self::encoded_len(self.payload.len()));
        out.push(IVF_POSTING_TAG);
        out.extend_from_slice(&self.list_id.to_le_bytes());
        out.push(if self.deleted {
            POSTING_FLAG_DELETED
        } else {
            0
        });
        out.push(self.heaptids.len() as u8);
        for tid in &self.heaptids {
            tid.encode_into(&mut out);
        }
        for _ in self.heaptids.len()..HEAPTID_INLINE_CAPACITY {
            ItemPointer::INVALID.encode_into(&mut out);
        }
        out.extend_from_slice(&self.gamma.to_le_bytes());
        self.rerank_tid.encode_into(&mut out);
        out.extend_from_slice(&self.payload);
        Ok(out)
    }

    pub fn decode(input: &[u8], payload_len: usize) -> Result<Self, String> {
        let posting = IvfPostingTupleRef::decode(input, payload_len)?;
        Ok(Self {
            list_id: posting.list_id,
            deleted: posting.deleted,
            heaptids: posting.collect_heaptids(),
            gamma: posting.gamma,
            rerank_tid: posting.rerank_tid,
            payload: posting.payload.to_vec(),
        })
    }

    pub(super) const fn encoded_len(payload_len: usize) -> usize {
        POSTING_FIXED_BYTES + payload_len
    }

    pub(super) fn encode_single_heaptid(
        list_id: u32,
        deleted: bool,
        heaptid: ItemPointer,
        gamma: f32,
        rerank_tid: ItemPointer,
        payload: &[u8],
    ) -> Result<Vec<u8>, String> {
        if !gamma.is_finite() {
            return Err("ec_ivf posting tuple gamma must be finite".into());
        }

        let mut out = Vec::with_capacity(Self::encoded_len(payload.len()));
        out.push(IVF_POSTING_TAG);
        out.extend_from_slice(&list_id.to_le_bytes());
        out.push(if deleted { POSTING_FLAG_DELETED } else { 0 });
        out.push(1);
        heaptid.encode_into(&mut out);
        for _ in 1..HEAPTID_INLINE_CAPACITY {
            ItemPointer::INVALID.encode_into(&mut out);
        }
        out.extend_from_slice(&gamma.to_le_bytes());
        rerank_tid.encode_into(&mut out);
        out.extend_from_slice(payload);
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IvfPqCodebookTuple {
    pub group_index: u16,
    pub next_tid: ItemPointer,
    pub centroids: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IvfPqCodebookTupleRef<'a> {
    pub(super) group_index: u16,
    pub(super) next_tid: ItemPointer,
    centroid_bytes: &'a [u8],
}

impl<'a> IvfPqCodebookTupleRef<'a> {
    pub(super) fn decode(input: &'a [u8], centroid_count: usize) -> Result<Self, String> {
        let expected_len = IvfPqCodebookTuple::encoded_len(centroid_count);
        if input.len() != expected_len {
            return Err(format!(
                "ec_ivf pq codebook tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if input[0] != IVF_PQ_CODEBOOK_TAG {
            return Err(format!(
                "invalid ec_ivf pq codebook tuple tag: {}",
                input[0]
            ));
        }

        Ok(Self {
            group_index: u16::from_le_bytes(
                input[1..3]
                    .try_into()
                    .expect("pq codebook group index slice should be 2 bytes"),
            ),
            next_tid: ItemPointer::decode(&input[3..9])?,
            centroid_bytes: &input[9..],
        })
    }

    pub(super) fn centroid_values(&self) -> impl Iterator<Item = f32> + '_ {
        self.centroid_bytes
            .chunks_exact(std::mem::size_of::<f32>())
            .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("validated f32 chunk")))
    }

    pub(super) fn collect_centroids(&self) -> Vec<f32> {
        self.centroid_values().collect()
    }
}

impl IvfPqCodebookTuple {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        if self.centroids.iter().any(|value| !value.is_finite()) {
            return Err("ec_ivf pq codebook contains a non-finite value".into());
        }

        let mut out = Vec::with_capacity(Self::encoded_len(self.centroids.len()));
        out.push(IVF_PQ_CODEBOOK_TAG);
        out.extend_from_slice(&self.group_index.to_le_bytes());
        self.next_tid.encode_into(&mut out);
        for value in &self.centroids {
            out.extend_from_slice(&value.to_le_bytes());
        }
        Ok(out)
    }

    pub fn decode(input: &[u8], centroid_count: usize) -> Result<Self, String> {
        let tuple = IvfPqCodebookTupleRef::decode(input, centroid_count)?;
        Ok(Self {
            group_index: tuple.group_index,
            next_tid: tuple.next_tid,
            centroids: tuple.collect_centroids(),
        })
    }

    pub(super) const fn encoded_len(centroid_count: usize) -> usize {
        1 + 2 + ITEM_POINTER_BYTES + centroid_count * std::mem::size_of::<f32>()
    }
}

pub(super) fn centroid_tuple_fits(dimensions: usize, page_size: usize) -> bool {
    aligned_tuple_bytes(IvfCentroidTuple::encoded_len(dimensions)) <= usable_page_bytes(page_size)
}

pub(super) fn list_directory_tuple_fits(page_size: usize) -> bool {
    aligned_tuple_bytes(IvfListDirectoryTuple::encoded_len()) <= usable_page_bytes(page_size)
}

pub(super) fn posting_tuple_fits(payload_len: usize, page_size: usize) -> bool {
    aligned_tuple_bytes(IvfPostingTuple::encoded_len(payload_len)) <= usable_page_bytes(page_size)
}

pub(super) fn columnar_frozen_list_header_tuple_fits(page_size: usize) -> bool {
    aligned_tuple_bytes(IvfColumnarFrozenListHeaderTuple::encoded_len())
        <= usable_page_bytes(page_size)
}

pub(super) fn columnar_frozen_list_raw_page_capacity(page_size: usize) -> usize {
    const RAW_PAGE_GUARD_BYTES: usize = 8;
    let usable = usable_page_bytes(page_size);
    if usable > RAW_PAGE_GUARD_BYTES {
        usable - RAW_PAGE_GUARD_BYTES
    } else {
        usable
    }
}

pub(super) fn dense_posting_block_tuple_fits(
    count: usize,
    total_heap_tids: usize,
    payload_len: usize,
    page_size: usize,
) -> bool {
    aligned_tuple_bytes(IvfDensePostingBlockTuple::encoded_len(
        count,
        total_heap_tids,
        payload_len,
    )) <= usable_page_bytes(page_size)
}

pub(super) fn dense_posting_packed_segment_tuple_fits(
    count: usize,
    total_heap_tids: usize,
    payload_len: usize,
    page_size: usize,
) -> bool {
    aligned_tuple_bytes(IvfDensePostingPackedSegmentTuple::encoded_len(
        count,
        total_heap_tids,
        payload_len,
    )) <= usable_page_bytes(page_size)
}

pub(super) fn dense_posting_packed_segment_header_tuple_fits(
    count: usize,
    total_heap_tids: usize,
    payload_bytes: usize,
    page_size: usize,
) -> bool {
    aligned_tuple_bytes(
        IvfDensePostingPackedSegmentTuple::encoded_len_with_payload_bytes(
            count,
            total_heap_tids,
            payload_bytes,
        ),
    ) <= usable_page_bytes(page_size)
}

pub(super) fn dense_posting_packed_continuation_tuple_fits(
    payload_bytes: usize,
    page_size: usize,
) -> bool {
    aligned_tuple_bytes(IvfDensePostingPackedContinuationTuple::encoded_len(
        payload_bytes,
    )) <= usable_page_bytes(page_size)
}

pub(super) fn pq_codebook_tuple_fits(centroid_count: usize, page_size: usize) -> bool {
    aligned_tuple_bytes(IvfPqCodebookTuple::encoded_len(centroid_count))
        <= usable_page_bytes(page_size)
}

impl DataPage {
    pub(super) fn insert_ivf_centroid(
        &mut self,
        tuple: &IvfCentroidTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn read_ivf_centroid(
        &self,
        tid: ItemPointer,
        dimensions: usize,
    ) -> Result<IvfCentroidTuple, String> {
        IvfCentroidTuple::decode(self.raw_tuple(tid)?, dimensions)
    }

    pub(super) fn insert_ivf_list_directory(
        &mut self,
        tuple: IvfListDirectoryTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode())
    }

    pub(super) fn read_ivf_list_directory(
        &self,
        tid: ItemPointer,
    ) -> Result<IvfListDirectoryTuple, String> {
        IvfListDirectoryTuple::decode(self.raw_tuple(tid)?)
    }

    pub(super) fn insert_ivf_posting(
        &mut self,
        tuple: &IvfPostingTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_dense_posting_block(
        &mut self,
        tuple: &IvfDensePostingBlockTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_dense_posting_aligned_block(
        &mut self,
        tuple: &IvfDensePostingBlockTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode_aligned()?)
    }

    pub(super) fn insert_ivf_dense_posting_packed_segment(
        &mut self,
        tuple: &IvfDensePostingPackedSegmentTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_dense_posting_packed_continuation(
        &mut self,
        tuple: &IvfDensePostingPackedContinuationTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_columnar_frozen_list_header(
        &mut self,
        tuple: &IvfColumnarFrozenListHeaderTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_single_heaptid_posting(
        &mut self,
        list_id: u32,
        heaptid: ItemPointer,
        gamma: f32,
        rerank_tid: ItemPointer,
        payload: &[u8],
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(IvfPostingTuple::encode_single_heaptid(
            list_id, false, heaptid, gamma, rerank_tid, payload,
        )?)
    }

    pub(super) fn insert_ivf_pq_codebook(
        &mut self,
        tuple: &IvfPqCodebookTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn update_ivf_pq_codebook(
        &mut self,
        tid: ItemPointer,
        tuple: &IvfPqCodebookTuple,
    ) -> Result<(), String> {
        self.update_raw_tuple(tid, tuple.encode()?)
    }

    pub(super) fn read_ivf_posting(
        &self,
        tid: ItemPointer,
        payload_len: usize,
    ) -> Result<IvfPostingTuple, String> {
        IvfPostingTuple::decode(self.raw_tuple(tid)?, payload_len)
    }

    pub(super) fn read_ivf_dense_posting_block(
        &self,
        tid: ItemPointer,
        payload_len: usize,
    ) -> Result<IvfDensePostingBlockTuple, String> {
        IvfDensePostingBlockTuple::decode(self.raw_tuple(tid)?, payload_len)
    }

    pub(super) fn read_ivf_dense_posting_packed_segment(
        &self,
        tid: ItemPointer,
        payload_len: usize,
    ) -> Result<IvfDensePostingPackedSegmentTuple, String> {
        IvfDensePostingPackedSegmentTuple::decode(self.raw_tuple(tid)?, payload_len)
    }

    pub(super) fn read_ivf_dense_posting_packed_continuation(
        &self,
        tid: ItemPointer,
    ) -> Result<IvfDensePostingPackedContinuationTuple, String> {
        IvfDensePostingPackedContinuationTuple::decode(self.raw_tuple(tid)?)
    }

    pub(super) fn read_ivf_columnar_frozen_list_header(
        &self,
        tid: ItemPointer,
    ) -> Result<IvfColumnarFrozenListHeaderTuple, String> {
        IvfColumnarFrozenListHeaderTuple::decode(self.raw_tuple(tid)?)
    }
}

impl DataPageChain {
    pub(super) fn insert_ivf_centroid(
        &mut self,
        tuple: &IvfCentroidTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn read_ivf_centroid(
        &self,
        tid: ItemPointer,
        dimensions: usize,
    ) -> Result<IvfCentroidTuple, String> {
        let page = self
            .get_page(tid.block_number)
            .ok_or_else(|| format!("ec_ivf centroid block {} not found", tid.block_number))?;
        page.read_ivf_centroid(tid, dimensions)
    }

    pub(super) fn insert_ivf_list_directory(
        &mut self,
        tuple: IvfListDirectoryTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode())
    }

    pub(super) fn read_ivf_list_directory(
        &self,
        tid: ItemPointer,
    ) -> Result<IvfListDirectoryTuple, String> {
        let page = self
            .get_page(tid.block_number)
            .ok_or_else(|| format!("ec_ivf directory block {} not found", tid.block_number))?;
        page.read_ivf_list_directory(tid)
    }

    pub(super) fn insert_ivf_posting(
        &mut self,
        tuple: &IvfPostingTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_dense_posting_block(
        &mut self,
        tuple: &IvfDensePostingBlockTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_dense_posting_aligned_block(
        &mut self,
        tuple: &IvfDensePostingBlockTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode_aligned()?)
    }

    pub(super) fn insert_ivf_dense_posting_packed_segment(
        &mut self,
        tuple: &IvfDensePostingPackedSegmentTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_dense_posting_packed_continuation(
        &mut self,
        tuple: &IvfDensePostingPackedContinuationTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_columnar_frozen_list_header(
        &mut self,
        tuple: &IvfColumnarFrozenListHeaderTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn insert_ivf_single_heaptid_posting(
        &mut self,
        list_id: u32,
        heaptid: ItemPointer,
        gamma: f32,
        rerank_tid: ItemPointer,
        payload: &[u8],
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(IvfPostingTuple::encode_single_heaptid(
            list_id, false, heaptid, gamma, rerank_tid, payload,
        )?)
    }

    pub(super) fn insert_ivf_pq_codebook(
        &mut self,
        tuple: &IvfPqCodebookTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub(super) fn update_ivf_pq_codebook(
        &mut self,
        tid: ItemPointer,
        tuple: &IvfPqCodebookTuple,
    ) -> Result<(), String> {
        self.get_page_mut(tid.block_number)
            .ok_or_else(|| format!("ec_ivf pq codebook block {} not found", tid.block_number))?
            .update_ivf_pq_codebook(tid, tuple)
    }

    pub(super) fn read_ivf_posting(
        &self,
        tid: ItemPointer,
        payload_len: usize,
    ) -> Result<IvfPostingTuple, String> {
        let page = self
            .get_page(tid.block_number)
            .ok_or_else(|| format!("ec_ivf posting block {} not found", tid.block_number))?;
        page.read_ivf_posting(tid, payload_len)
    }

    pub(super) fn read_ivf_dense_posting_block(
        &self,
        tid: ItemPointer,
        payload_len: usize,
    ) -> Result<IvfDensePostingBlockTuple, String> {
        let page = self
            .get_page(tid.block_number)
            .ok_or_else(|| format!("ec_ivf dense posting block {} not found", tid.block_number))?;
        page.read_ivf_dense_posting_block(tid, payload_len)
    }

    pub(super) fn read_ivf_dense_posting_packed_segment(
        &self,
        tid: ItemPointer,
        payload_len: usize,
    ) -> Result<IvfDensePostingPackedSegmentTuple, String> {
        let page = self.get_page(tid.block_number).ok_or_else(|| {
            format!(
                "ec_ivf dense posting packed segment block {} not found",
                tid.block_number
            )
        })?;
        page.read_ivf_dense_posting_packed_segment(tid, payload_len)
    }

    pub(super) fn read_ivf_dense_posting_packed_continuation(
        &self,
        tid: ItemPointer,
    ) -> Result<IvfDensePostingPackedContinuationTuple, String> {
        let page = self.get_page(tid.block_number).ok_or_else(|| {
            format!(
                "ec_ivf dense posting packed continuation block {} not found",
                tid.block_number
            )
        })?;
        page.read_ivf_dense_posting_packed_continuation(tid)
    }

    pub(super) fn read_ivf_columnar_frozen_list_header(
        &self,
        tid: ItemPointer,
    ) -> Result<IvfColumnarFrozenListHeaderTuple, String> {
        let page = self.get_page(tid.block_number).ok_or_else(|| {
            format!(
                "ec_ivf columnar frozen list header block {} not found",
                tid.block_number
            )
        })?;
        page.read_ivf_columnar_frozen_list_header(tid)
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn read_ivf_centroid_and_next(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
    dimensions: usize,
) -> Result<(IvfCentroidTuple, ItemPointer), String> {
    let index = IvfPageRelation::new(ivf_relation_nonnull(index_relation, "ec_ivf centroid read"));
    let (centroid, line_pointer_count) = read_page_tuple(index, tid, "centroid", |tuple_bytes| {
        IvfCentroidTuple::decode(tuple_bytes, dimensions)
    })?;
    Ok((centroid, next_physical_tuple_tid(tid, line_pointer_count)?))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn read_ivf_list_directory_and_next(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
) -> Result<(IvfListDirectoryTuple, ItemPointer), String> {
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf list directory read",
    ));
    let (directory, line_pointer_count) =
        read_page_tuple(index, tid, "list directory", |tuple_bytes| {
            IvfListDirectoryTuple::decode(tuple_bytes)
        })?;
    let physical_next = next_physical_tuple_tid(tid, line_pointer_count)?;
    let next_directory = find_next_tuple_with_tag(
        index,
        physical_next,
        IVF_LIST_DIRECTORY_TAG,
        "list directory",
    )?;
    Ok((directory, next_directory))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn read_ivf_pq_codebook(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
    centroid_count: usize,
) -> Result<IvfPqCodebookTuple, String> {
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf pq codebook read",
    ));
    let (codebook, _) = read_page_tuple(index, tid, "pq codebook", |tuple_bytes| {
        IvfPqCodebookTuple::decode(tuple_bytes, centroid_count)
    })?;
    Ok(codebook)
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn read_ivf_postings_for_list_blocks(
    index_relation: pg_sys::Relation,
    list_id: u32,
    head_block: BlockRef,
    tail_block: BlockRef,
    payload_len: usize,
) -> Result<Vec<IvfPostingTuple>, String> {
    let mut postings = Vec::new();
    visit_ivf_postings_for_list_blocks(
        index_relation,
        list_id,
        head_block,
        tail_block,
        payload_len,
        |_, posting| {
            postings.push(posting);
            Ok(())
        },
    )?;
    Ok(postings)
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn visit_ivf_postings_for_list_blocks<F>(
    index_relation: pg_sys::Relation,
    list_id: u32,
    head_block: BlockRef,
    tail_block: BlockRef,
    payload_len: usize,
    mut visitor: F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    if head_block == BlockRef::INVALID && tail_block == BlockRef::INVALID {
        return Ok(());
    }
    if head_block == BlockRef::INVALID || tail_block == BlockRef::INVALID {
        return Err(format!(
            "ec_ivf list {list_id} has partial posting block refs"
        ));
    }
    if head_block.block_number > tail_block.block_number {
        return Err(format!(
            "ec_ivf list {list_id} posting block range is inverted"
        ));
    }

    #[cfg(feature = "pg18")]
    {
        visit_ivf_posting_blocks_with_read_stream(
            index_relation,
            list_id,
            head_block.block_number,
            tail_block.block_number,
            payload_len,
            &mut visitor,
        )?;
    }

    #[cfg(not(feature = "pg18"))]
    {
        for block_number in head_block.block_number..=tail_block.block_number {
            visit_ivf_postings_for_list_block(
                index_relation,
                list_id,
                block_number,
                payload_len,
                &mut visitor,
            )?;
        }
    }
    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn rewrite_ivf_postings_for_list_blocks<F>(
    index_relation: pg_sys::Relation,
    list_id: u32,
    head_block: BlockRef,
    tail_block: BlockRef,
    payload_len: usize,
    no_compact_blocks: &[pg_sys::BlockNumber],
    rewrite: F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<IvfPostingRewrite, String>,
{
    rewrite_ivf_posting_entries_for_list_blocks(
        index_relation,
        list_id,
        head_block,
        tail_block,
        payload_len,
        no_compact_blocks,
        rewrite,
        |_, _| Ok(IvfDensePostingBlockRewrite::Keep),
        |_, _| Ok(IvfDensePostingPackedSegmentRewrite::Keep),
    )
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn rewrite_ivf_posting_entries_for_list_blocks<RowFn, DenseFn, PackedFn>(
    index_relation: pg_sys::Relation,
    list_id: u32,
    head_block: BlockRef,
    tail_block: BlockRef,
    payload_len: usize,
    no_compact_blocks: &[pg_sys::BlockNumber],
    mut rewrite_row: RowFn,
    mut rewrite_dense: DenseFn,
    mut rewrite_packed: PackedFn,
) -> Result<(), String>
where
    RowFn: FnMut(ItemPointer, IvfPostingTuple) -> Result<IvfPostingRewrite, String>,
    DenseFn: FnMut(
        ItemPointer,
        IvfDensePostingBlockTuple,
    ) -> Result<IvfDensePostingBlockRewrite, String>,
    PackedFn: FnMut(
        ItemPointer,
        IvfDensePostingPackedSegmentTuple,
    ) -> Result<IvfDensePostingPackedSegmentRewrite, String>,
{
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf posting rewrite",
    ));
    if head_block == BlockRef::INVALID && tail_block == BlockRef::INVALID {
        return Ok(());
    }
    if head_block == BlockRef::INVALID || tail_block == BlockRef::INVALID {
        return Err(format!(
            "ec_ivf list {list_id} has partial posting block refs"
        ));
    }
    if head_block.block_number > tail_block.block_number {
        return Err(format!(
            "ec_ivf list {list_id} posting block range is inverted"
        ));
    }

    for block_number in head_block.block_number..=tail_block.block_number {
        rewrite_ivf_postings_for_list_block(
            index,
            list_id,
            block_number,
            payload_len,
            !no_compact_blocks.contains(&block_number),
            &mut rewrite_row,
            &mut rewrite_dense,
            &mut rewrite_packed,
        )?;
    }

    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn visit_ivf_posting_refs_for_block_sequence<F>(
    index_relation: RelationHandle,
    block_numbers: &[pg_sys::BlockNumber],
    payload_len: usize,
    mut visitor: F,
) -> Result<(), String>
where
    F: for<'a> FnMut(ItemPointer, IvfPostingTupleRef<'a>) -> Result<(), String>,
{
    if block_numbers.is_empty() {
        return Ok(());
    }

    #[cfg(feature = "pg18")]
    {
        visit_ivf_posting_ref_block_sequence_with_read_stream(
            index_relation.as_ptr(),
            block_numbers,
            payload_len,
            &mut visitor,
        )?;
    }

    #[cfg(not(feature = "pg18"))]
    {
        for block_number in block_numbers {
            visit_all_ivf_posting_refs_for_block(
                index_relation.as_ptr(),
                *block_number,
                payload_len,
                &mut visitor,
            )?;
        }
    }

    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn visit_ivf_posting_entries_for_block_sequence<F>(
    index_relation: RelationHandle,
    block_numbers: &[pg_sys::BlockNumber],
    payload_len: usize,
    mut visitor: F,
) -> Result<(), String>
where
    F: for<'a> FnMut(ItemPointer, IvfPostingEntryRef<'a>) -> Result<(), String>,
{
    if block_numbers.is_empty() {
        return Ok(());
    }

    #[cfg(feature = "pg18")]
    {
        visit_ivf_posting_entry_block_sequence_with_read_stream(
            index_relation.as_ptr(),
            block_numbers,
            payload_len,
            &mut visitor,
        )?;
    }

    #[cfg(not(feature = "pg18"))]
    {
        for block_number in block_numbers {
            visit_all_ivf_posting_entries_for_block(
                index_relation.as_ptr(),
                *block_number,
                payload_len,
                &mut visitor,
            )?;
        }
    }

    Ok(())
}

#[cfg(feature = "pg18")]
fn visit_ivf_posting_blocks_with_read_stream<F>(
    index_relation: pg_sys::Relation,
    list_id: u32,
    head_block: pg_sys::BlockNumber,
    tail_block: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    crate::am::stream::visit_relation_linear_read_stream(
        index_relation,
        head_block,
        tail_block,
        "ec_ivf posting list",
        |buffer, block_number| {
            visit_ivf_postings_from_buffer(buffer, list_id, block_number, payload_len, visitor)
        },
    )
}

#[cfg(feature = "pg18")]
fn visit_ivf_posting_ref_block_sequence_with_read_stream<F>(
    index_relation: pg_sys::Relation,
    block_numbers: &[pg_sys::BlockNumber],
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: for<'a> FnMut(ItemPointer, IvfPostingTupleRef<'a>) -> Result<(), String>,
{
    crate::am::stream::visit_relation_block_sequence_read_stream(
        index_relation,
        block_numbers,
        "ec_ivf posting ref block sequence",
        |buffer, block_number| {
            visit_all_ivf_posting_refs_from_buffer(buffer, block_number, payload_len, visitor)
        },
    )
}

#[cfg(feature = "pg18")]
fn visit_ivf_posting_entry_block_sequence_with_read_stream<F>(
    index_relation: pg_sys::Relation,
    block_numbers: &[pg_sys::BlockNumber],
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: for<'a> FnMut(ItemPointer, IvfPostingEntryRef<'a>) -> Result<(), String>,
{
    crate::am::stream::visit_relation_block_sequence_read_stream(
        index_relation,
        block_numbers,
        "ec_ivf posting entry block sequence",
        |buffer, block_number| {
            visit_all_ivf_posting_entries_from_buffer(buffer, block_number, payload_len, visitor)
        },
    )
}

#[cfg(all(any(feature = "pg17", feature = "pg18"), not(feature = "pg18")))]
fn visit_ivf_postings_for_list_block<F>(
    index_relation: pg_sys::Relation,
    list_id: u32,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf posting-list read",
    ));
    let buffer = read_posting_block(index, block_number, "posting-list")?;

    let result =
        visit_ivf_postings_from_buffer(&buffer, list_id, block_number, payload_len, visitor);
    result
}

#[cfg(all(any(feature = "pg17", feature = "pg18"), not(feature = "pg18")))]
fn visit_all_ivf_posting_refs_for_block<F>(
    index_relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: for<'a> FnMut(ItemPointer, IvfPostingTupleRef<'a>) -> Result<(), String>,
{
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf posting-list ref read",
    ));
    let buffer = read_posting_block(index, block_number, "posting-list")?;

    let result =
        visit_all_ivf_posting_refs_from_buffer(&buffer, block_number, payload_len, visitor);
    result
}

#[cfg(all(any(feature = "pg17", feature = "pg18"), not(feature = "pg18")))]
fn visit_all_ivf_posting_entries_for_block<F>(
    index_relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: for<'a> FnMut(ItemPointer, IvfPostingEntryRef<'a>) -> Result<(), String>,
{
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf posting-list entry read",
    ));
    let buffer = read_posting_block(index, block_number, "posting-list")?;

    visit_all_ivf_posting_entries_from_buffer(&buffer, block_number, payload_len, visitor)
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn visit_ivf_postings_from_buffer<F>(
    buffer: &LockedBufferGuard,
    list_id: u32,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    visit_all_ivf_postings_from_buffer(
        buffer,
        block_number,
        payload_len,
        &mut |posting_tid, posting| {
            if posting.list_id == list_id {
                visitor(posting_tid, posting)?;
            }
            Ok(())
        },
    )
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn visit_all_ivf_postings_from_buffer<F>(
    buffer: &LockedBufferGuard,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    let page = PageTupleReader::new(buffer, block_number);
    for offset in 1..=page.line_pointer_count() {
        page.visit_line(offset, "posting", |tuple_bytes| {
            if tuple_bytes.first().copied() != Some(IVF_POSTING_TAG) {
                return Ok(());
            }

            let posting = IvfPostingTuple::decode(tuple_bytes, payload_len)?;
            visitor(
                ItemPointer {
                    block_number,
                    offset_number: offset,
                },
                posting,
            )
        })?;
    }
    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn visit_all_ivf_posting_refs_from_buffer<F>(
    buffer: &LockedBufferGuard,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: for<'a> FnMut(ItemPointer, IvfPostingTupleRef<'a>) -> Result<(), String>,
{
    let page = PageTupleReader::new(buffer, block_number);
    for offset in 1..=page.line_pointer_count() {
        page.visit_line(offset, "posting", |tuple_bytes| {
            if tuple_bytes.first().copied() != Some(IVF_POSTING_TAG) {
                return Ok(());
            }

            let posting = IvfPostingTupleRef::decode(tuple_bytes, payload_len)?;
            visitor(
                ItemPointer {
                    block_number,
                    offset_number: offset,
                },
                posting,
            )
        })?;
    }
    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn visit_all_ivf_posting_entries_from_buffer<F>(
    buffer: &LockedBufferGuard,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: for<'a> FnMut(ItemPointer, IvfPostingEntryRef<'a>) -> Result<(), String>,
{
    let page = PageTupleReader::new(buffer, block_number);
    for offset in 1..=page.line_pointer_count() {
        page.visit_line(offset, "posting entry", |tuple_bytes| {
            let Some(tag) = tuple_bytes.first().copied() else {
                return Ok(());
            };
            let tid = ItemPointer {
                block_number,
                offset_number: offset,
            };
            match tag {
                IVF_POSTING_TAG => {
                    let posting = IvfPostingTupleRef::decode(tuple_bytes, payload_len)?;
                    visitor(tid, IvfPostingEntryRef::Row(posting))
                }
                IVF_DENSE_POSTING_BLOCK_TAG | IVF_DENSE_POSTING_ALIGNED_BLOCK_TAG => {
                    let block = IvfDensePostingBlockRef::decode(tuple_bytes, payload_len)?;
                    block.validate_offsets()?;
                    visitor(tid, IvfPostingEntryRef::DenseBlock(block))
                }
                IVF_DENSE_POSTING_PACKED_SEGMENT_TAG => {
                    let segment =
                        IvfDensePostingPackedSegmentRef::decode(tuple_bytes, payload_len)?;
                    segment.validate_offsets()?;
                    visitor(tid, IvfPostingEntryRef::DensePackedSegment(segment))
                }
                IVF_DENSE_POSTING_PACKED_CONTINUATION_TAG => {
                    let continuation = IvfDensePostingPackedContinuationRef::decode(tuple_bytes)?;
                    visitor(
                        tid,
                        IvfPostingEntryRef::DensePackedContinuation(continuation),
                    )
                }
                IVF_COLUMNAR_FROZEN_LIST_HEADER_TAG => {
                    let header = IvfColumnarFrozenListHeaderRef::decode(tuple_bytes)?;
                    visitor(tid, IvfPostingEntryRef::ColumnarHeader(header))
                }
                _ => Ok(()),
            }
        })?;
    }
    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn read_columnar_frozen_list_logical_bytes(
    index_relation: RelationHandle,
    header: IvfColumnarFrozenListHeaderRef,
) -> Result<Vec<u8>, String> {
    header.validate()?;
    let first_block = header.first_column_block.block_number;
    let last_block = header.last_column_block.block_number;
    let expected_block_count = last_block
        .checked_sub(first_block)
        .and_then(|delta| delta.checked_add(1))
        .ok_or_else(|| "ec_ivf columnar frozen list block range underflow".to_owned())?
        as usize;

    let first_buffer = read_columnar_raw_page(
        index_relation,
        first_block,
        "columnar frozen list first page",
    )?;
    let page_size = first_buffer.page_size();
    let page_lengths = columnar_frozen_list_raw_page_lengths(header, page_size)?;
    if page_lengths.len() != expected_block_count {
        return Err(format!(
            "ec_ivf columnar frozen list block count mismatch: header has {expected_block_count}, derived {}",
            page_lengths.len()
        ));
    }

    let mut logical_bytes = Vec::with_capacity(header.total_column_bytes as usize);
    append_columnar_raw_page_bytes(
        &first_buffer,
        first_block,
        page_lengths[0],
        &mut logical_bytes,
    )?;
    drop(first_buffer);

    for (page_index, expected_len) in page_lengths.iter().copied().enumerate().skip(1) {
        let block_number = first_block
            .checked_add(page_index as u32)
            .ok_or_else(|| "ec_ivf columnar frozen list block number overflow".to_owned())?;
        let buffer =
            read_columnar_raw_page(index_relation, block_number, "columnar frozen list page")?;
        append_columnar_raw_page_bytes(&buffer, block_number, expected_len, &mut logical_bytes)?;
    }

    if logical_bytes.len() != header.total_column_bytes as usize {
        return Err(format!(
            "ec_ivf columnar frozen list copied byte count mismatch: got {}, expected {}",
            logical_bytes.len(),
            header.total_column_bytes
        ));
    }
    Ok(logical_bytes)
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn rewrite_columnar_frozen_list_logical_bytes(
    index_relation: RelationHandle,
    header: IvfColumnarFrozenListHeaderRef,
    logical_bytes: &[u8],
) -> Result<(), String> {
    let block = IvfColumnarFrozenListRef::decode(header, logical_bytes)?;
    block.validate_offsets()?;
    let first_block = header.first_column_block.block_number;
    let last_block = header.last_column_block.block_number;
    let expected_block_count = last_block
        .checked_sub(first_block)
        .and_then(|delta| delta.checked_add(1))
        .ok_or_else(|| "ec_ivf columnar frozen list block range underflow".to_owned())?
        as usize;

    let first_buffer = read_columnar_raw_page_for_update(
        index_relation,
        first_block,
        "columnar frozen list first page",
    )?;
    let page_size = first_buffer.page_size();
    let page_lengths = columnar_frozen_list_raw_page_lengths(header, page_size)?;
    if page_lengths.len() != expected_block_count {
        return Err(format!(
            "ec_ivf columnar frozen list block count mismatch: header has {expected_block_count}, derived {}",
            page_lengths.len()
        ));
    }

    let index = IvfPageRelation::new(index_relation);
    let mut start = 0_usize;
    let first_len = page_lengths[0];
    rewrite_columnar_raw_page_bytes(
        index,
        &first_buffer,
        first_block,
        &logical_bytes[..first_len],
    )?;
    start += first_len;
    drop(first_buffer);

    for (page_index, expected_len) in page_lengths.iter().copied().enumerate().skip(1) {
        let block_number = first_block
            .checked_add(page_index as u32)
            .ok_or_else(|| "ec_ivf columnar frozen list block number overflow".to_owned())?;
        let end = start
            .checked_add(expected_len)
            .ok_or_else(|| "ec_ivf columnar frozen list byte range overflow".to_owned())?;
        let buffer = read_columnar_raw_page_for_update(
            index_relation,
            block_number,
            "columnar frozen list page",
        )?;
        rewrite_columnar_raw_page_bytes(index, &buffer, block_number, &logical_bytes[start..end])?;
        start = end;
    }

    if start != logical_bytes.len() {
        return Err(format!(
            "ec_ivf columnar frozen list rewrite byte count mismatch: wrote {start}, expected {}",
            logical_bytes.len()
        ));
    }
    Ok(())
}

pub(super) fn columnar_frozen_list_deleted_bitmap_mut<'a>(
    header: IvfColumnarFrozenListHeaderRef,
    logical_bytes: &'a mut [u8],
) -> Result<&'a mut [u8], String> {
    let block = IvfColumnarFrozenListRef::decode(header, logical_bytes)?;
    block.validate_offsets()?;
    let bitmap_start = header.deleted_bitmap_offset as usize;
    Ok(&mut logical_bytes[bitmap_start..])
}

pub(super) fn columnar_frozen_list_mark_deleted(bitmap: &mut [u8], index: usize) {
    dense_deleted_bitmap_set(bitmap, index);
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn read_columnar_raw_page(
    index_relation: RelationHandle,
    block_number: pg_sys::BlockNumber,
    context: &str,
) -> Result<LockedBufferGuard, String> {
    LockedBufferGuard::read_main_handle(
        index_relation,
        block_number,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    )
    .ok_or_else(|| format!("ec_ivf failed to open {context} block {block_number}"))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn read_columnar_raw_page_for_update(
    index_relation: RelationHandle,
    block_number: pg_sys::BlockNumber,
    context: &str,
) -> Result<LockedBufferGuard, String> {
    LockedBufferGuard::read_main_handle(
        index_relation,
        block_number,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    )
    .ok_or_else(|| format!("ec_ivf failed to open {context} block {block_number}"))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn append_columnar_raw_page_bytes(
    buffer: &LockedBufferGuard,
    block_number: pg_sys::BlockNumber,
    expected_len: usize,
    out: &mut Vec<u8>,
) -> Result<(), String> {
    let page = buffer.page();
    let special_size = unsafe { pg_sys::PageGetSpecialSize(page) } as usize;
    if special_size < expected_len {
        return Err(format!(
            "ec_ivf columnar frozen list page {block_number} special area too small: got {special_size}, expected at least {expected_len}"
        ));
    }
    let special = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    if special.is_null() {
        return Err(format!(
            "ec_ivf columnar frozen list page {block_number} returned a null special pointer"
        ));
    }
    let bytes = unsafe { std::slice::from_raw_parts(special.cast_const(), expected_len) };
    out.extend_from_slice(bytes);
    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn rewrite_columnar_raw_page_bytes(
    index: IvfPageRelation<'_>,
    buffer: &LockedBufferGuard,
    block_number: pg_sys::BlockNumber,
    bytes: &[u8],
) -> Result<(), String> {
    let mut wal_txn = index.start_wal();
    let page = wal_txn.register_locked_buffer_full_image(buffer);
    let registered = WalRegisteredPage::new(index.raw(), block_number, page);
    let special_size = unsafe { pg_sys::PageGetSpecialSize(registered.page()) } as usize;
    if special_size < bytes.len() {
        std::mem::drop(wal_txn);
        return Err(format!(
            "ec_ivf columnar frozen list page {block_number} special area too small for rewrite: got {special_size}, expected at least {}",
            bytes.len()
        ));
    }
    registered.copy_to_special(bytes);
    wal_txn.finish();
    registered.record_free_space(registered.free_space());
    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn append_ivf_posting_to_list_range(
    index_relation: pg_sys::Relation,
    block_range: Option<(pg_sys::BlockNumber, pg_sys::BlockNumber)>,
    tuple: &IvfPostingTuple,
) -> Result<ItemPointer, String> {
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf posting append",
    ));
    if !posting_tuple_fits(tuple.payload.len(), pg_sys::BLCKSZ as usize) {
        return Err(format!(
            "ec_ivf posting payload {} does not fit on a page",
            tuple.payload.len()
        ));
    }
    let payload = tuple.encode()?;

    if let Some((head_block, tail_block)) = block_range {
        if head_block > tail_block {
            return Err(format!(
                "ec_ivf list {} has invalid posting block range {}..{}",
                tuple.list_id, head_block, tail_block
            ));
        }

        let relid = index.relid();
        let mut range_walk_start = tail_block.saturating_sub(1);
        let mut tried_tail_hint = false;
        if let Some(hint_block) = posting_free_hint(relid, tuple.list_id) {
            if block_in_range(hint_block, head_block, tail_block) {
                tried_tail_hint = hint_block == tail_block;
                if let Some(tid) = try_append_ivf_posting_to_block(index, hint_block, &payload)? {
                    remember_posting_free_hint(relid, tuple.list_id, tid.block_number);
                    return Ok(tid);
                }
                if hint_block > head_block {
                    range_walk_start = hint_block - 1;
                    remember_posting_free_hint(relid, tuple.list_id, range_walk_start);
                } else {
                    forget_posting_free_hint(relid, tuple.list_id);
                }
            } else {
                forget_posting_free_hint(relid, tuple.list_id);
            }
        }

        if !tried_tail_hint {
            if let Some(tid) = try_append_ivf_posting_to_block(index, tail_block, &payload)? {
                remember_posting_free_hint(relid, tuple.list_id, tid.block_number);
                return Ok(tid);
            }
        }

        let required_space = raw_tuple_storage_bytes(payload.len());
        let fsm_block = index.page_with_free_space(required_space);
        if block_in_range(fsm_block, head_block, tail_block) && fsm_block != tail_block {
            if let Some(tid) = try_append_ivf_posting_to_block(index, fsm_block, &payload)? {
                remember_posting_free_hint(relid, tuple.list_id, tid.block_number);
                return Ok(tid);
            }
        }

        // Vacuum can free space before the current tail. This v1 reuse path is
        // intentionally conservative: use the global index FSM as a hint, then
        // fall back to a bounded range walk because free space is not list-keyed.
        for block_number in (head_block..=range_walk_start).rev() {
            if let Some(tid) = try_append_ivf_posting_to_block(index, block_number, &payload)? {
                remember_posting_free_hint(relid, tuple.list_id, tid.block_number);
                return Ok(tid);
            }
        }

        // Vacuum can leave reusable capacity on the immediate boundary pages
        // of neighboring lists. Keep this deliberately bounded to one block
        // on either side so reuse does not turn one list into a wide scan range.
        if let Some(left_neighbor) = head_block.checked_sub(1) {
            if left_neighbor >= FIRST_DATA_BLOCK_NUMBER {
                if let Some(tid) = try_append_ivf_posting_to_block(index, left_neighbor, &payload)?
                {
                    remember_posting_free_hint(relid, tuple.list_id, tid.block_number);
                    return Ok(tid);
                }
            }
        }

        let relation_blocks = index.number_of_blocks();
        if let Some(right_neighbor) = tail_block.checked_add(1) {
            if right_neighbor < relation_blocks {
                if let Some(tid) = try_append_ivf_posting_to_block(index, right_neighbor, &payload)?
                {
                    remember_posting_free_hint(relid, tuple.list_id, tid.block_number);
                    return Ok(tid);
                }
            }
        }
    }

    append_ivf_posting_to_new_block(index, &payload)
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn try_append_ivf_posting_to_block(
    index: IvfPageRelation<'_>,
    block_number: pg_sys::BlockNumber,
    payload: &[u8],
) -> Result<Option<ItemPointer>, String> {
    let buffer = index
        .read_main(
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
        .ok_or_else(|| format!("ec_ivf failed to open posting-list block {block_number}"))?;

    let mut wal_txn = index.start_wal();
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let registered = WalRegisteredPage::new(index.raw(), block_number, page);
    let free_space = registered.free_space();
    if free_space < raw_tuple_storage_bytes(payload.len()) {
        registered.record_free_space(free_space);
        std::mem::drop(wal_txn);
        return Ok(None);
    }

    let offset = registered.add_item(payload);
    if offset == pg_sys::InvalidOffsetNumber {
        std::mem::drop(wal_txn);
        return Err(format!(
            "ec_ivf failed to append posting tuple to block {block_number}"
        ));
    }

    wal_txn.finish();
    registered.record_free_space(registered.free_space());
    Ok(Some(ItemPointer {
        block_number,
        offset_number: offset,
    }))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn append_ivf_posting_to_new_block(
    index: IvfPageRelation<'_>,
    payload: &[u8],
) -> Result<ItemPointer, String> {
    let buffer = index
        .read_main_locked(P_NEW, pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK)
        .ok_or_else(|| "ec_ivf failed to allocate posting-list block".to_owned())?;

    let page_size = buffer.page_size();
    let mut wal_txn = index.start_wal();
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let registered = WalRegisteredPage::new(index.raw(), buffer.block_number(), page);
    registered.init(page_size, 0);

    let offset = registered.add_item(payload);
    if offset == pg_sys::InvalidOffsetNumber {
        std::mem::drop(wal_txn);
        return Err("ec_ivf failed to append posting tuple to new block".to_owned());
    }
    let block_number = buffer.block_number();

    wal_txn.finish();
    registered.record_free_space(registered.free_space());
    Ok(ItemPointer {
        block_number,
        offset_number: offset,
    })
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn rewrite_ivf_list_directory(
    index_relation: pg_sys::Relation,
    directory_tid: ItemPointer,
    directory: IvfListDirectoryTuple,
) -> Result<(), String> {
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf directory rewrite",
    ));
    let encoded = directory.encode();
    let buffer = index
        .read_main(
            directory_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
        .ok_or_else(|| {
            format!(
                "ec_ivf failed to open directory block {}",
                directory_tid.block_number
            )
        })?;

    let mut wal_txn = index.start_wal();
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let writer = PageTupleWriter::new(page, buffer.page_size(), directory_tid.block_number);
    if let Err(err) = writer.copy_required_exact(directory_tid, "directory", &encoded) {
        std::mem::drop(wal_txn);
        return Err(err);
    }
    wal_txn.finish();
    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn update_ivf_list_directory<F>(
    index_relation: pg_sys::Relation,
    directory_tid: ItemPointer,
    update: F,
) -> Result<IvfListDirectoryTuple, String>
where
    F: FnOnce(&mut IvfListDirectoryTuple) -> Result<(), String>,
{
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf directory update",
    ));
    let buffer = index
        .read_main(
            directory_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
        .ok_or_else(|| {
            format!(
                "ec_ivf failed to open directory block {}",
                directory_tid.block_number
            )
        })?;

    let mut wal_txn = index.start_wal();
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let writer = PageTupleWriter::new(page, buffer.page_size(), directory_tid.block_number);
    let mut directory = match writer.visit_required(directory_tid, "directory", |tuple_bytes| {
        if tuple_bytes.len() != IvfListDirectoryTuple::encoded_len() {
            return Err(format!(
                "ec_ivf directory tuple size changed from {} to {}",
                tuple_bytes.len(),
                IvfListDirectoryTuple::encoded_len()
            ));
        }

        IvfListDirectoryTuple::decode(tuple_bytes)
    }) {
        Ok(directory) => directory,
        Err(err) => {
            std::mem::drop(wal_txn);
            return Err(err);
        }
    };
    if let Err(err) = update(&mut directory) {
        std::mem::drop(wal_txn);
        return Err(err);
    }

    let encoded = directory.encode();
    if let Err(err) = writer.copy_required_exact(directory_tid, "directory", &encoded) {
        std::mem::drop(wal_txn);
        return Err(err);
    }
    wal_txn.finish();
    Ok(directory)
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
#[derive(Debug, Clone, PartialEq)]
pub(super) enum IvfPostingRewrite {
    Keep,
    Rewrite(IvfPostingTuple),
    Delete,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
#[derive(Debug, Clone, PartialEq)]
pub(super) enum IvfDensePostingBlockRewrite {
    Keep,
    Rewrite(IvfDensePostingBlockTuple),
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
#[derive(Debug, Clone, PartialEq)]
pub(super) enum IvfDensePostingPackedSegmentRewrite {
    Keep,
    Rewrite(IvfDensePostingPackedSegmentTuple),
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct IvfPostingBlockSummary {
    pub(super) block_number: pg_sys::BlockNumber,
    pub(super) line_pointer_count: u16,
    pub(super) unused_line_pointers: u16,
    pub(super) non_posting_tuples: u16,
    pub(super) posting_tuples: u16,
    pub(super) live_posting_tuples: u16,
    pub(super) deleted_posting_tuples: u16,
    pub(super) heap_tid_refs: u32,
    pub(super) list_ids: Vec<u32>,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn debug_ivf_posting_block_summaries(
    index_relation: pg_sys::Relation,
    payload_len: usize,
) -> Result<Vec<IvfPostingBlockSummary>, String> {
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf posting diagnostics",
    ));
    let block_count = index.number_of_blocks();
    let mut summaries = Vec::new();
    for block_number in FIRST_DATA_BLOCK_NUMBER..block_count {
        let summary = debug_ivf_posting_block_summary(index, block_number, payload_len)?;
        if summary.line_pointer_count > 0
            || summary.posting_tuples > 0
            || summary.non_posting_tuples > 0
            || summary.unused_line_pointers > 0
        {
            summaries.push(summary);
        }
    }
    Ok(summaries)
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn rewrite_ivf_postings_for_list_block<RowFn, DenseFn, PackedFn>(
    index: IvfPageRelation<'_>,
    list_id: u32,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    compact_deletes: bool,
    rewrite_row: &mut RowFn,
    rewrite_dense: &mut DenseFn,
    rewrite_packed: &mut PackedFn,
) -> Result<(), String>
where
    RowFn: FnMut(ItemPointer, IvfPostingTuple) -> Result<IvfPostingRewrite, String>,
    DenseFn: FnMut(
        ItemPointer,
        IvfDensePostingBlockTuple,
    ) -> Result<IvfDensePostingBlockRewrite, String>,
    PackedFn: FnMut(
        ItemPointer,
        IvfDensePostingPackedSegmentTuple,
    ) -> Result<IvfDensePostingPackedSegmentRewrite, String>,
{
    let buffer = index
        .read_main(
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
        .ok_or_else(|| format!("ec_ivf failed to open posting-list block {block_number}"))?;

    rewrite_ivf_postings_from_exclusive_buffer(
        index,
        &buffer,
        list_id,
        block_number,
        payload_len,
        compact_deletes,
        rewrite_row,
        rewrite_dense,
        rewrite_packed,
    )
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn debug_ivf_posting_block_summary(
    index: IvfPageRelation<'_>,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
) -> Result<IvfPostingBlockSummary, String> {
    let buffer = index
        .read_main(
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| format!("ec_ivf failed to open block {block_number}"))?;

    let result = (|| -> Result<IvfPostingBlockSummary, String> {
        let page = PageTupleReader::new(&buffer, block_number);
        let line_pointer_count = page.line_pointer_count();
        let mut unused_line_pointers = 0_u16;
        let mut non_posting_tuples = 0_u16;
        let mut posting_tuples = 0_u16;
        let mut live_posting_tuples = 0_u16;
        let mut deleted_posting_tuples = 0_u16;
        let mut heap_tid_refs = 0_u32;
        let mut list_ids = BTreeSet::new();

        for offset in 1..=line_pointer_count {
            match page.visit_line(offset, "posting", |tuple_bytes| {
                if tuple_bytes.first().copied() != Some(IVF_POSTING_TAG) {
                    return Ok(false);
                }

                let posting = IvfPostingTupleRef::decode(tuple_bytes, payload_len)?;
                posting_tuples = posting_tuples.saturating_add(1);
                if posting.deleted {
                    deleted_posting_tuples = deleted_posting_tuples.saturating_add(1);
                } else {
                    live_posting_tuples = live_posting_tuples.saturating_add(1);
                }
                heap_tid_refs = heap_tid_refs.saturating_add(
                    u32::try_from(posting.heaptid_count())
                        .map_err(|_| "ec_ivf posting heap tid count exceeds u32".to_owned())?,
                );
                list_ids.insert(posting.list_id);
                Ok(true)
            })? {
                PageTupleVisit::Unused => {
                    unused_line_pointers = unused_line_pointers.saturating_add(1);
                }
                PageTupleVisit::Present(false) => {
                    non_posting_tuples = non_posting_tuples.saturating_add(1);
                }
                PageTupleVisit::Present(true) => {}
            }
        }

        Ok(IvfPostingBlockSummary {
            block_number,
            line_pointer_count,
            unused_line_pointers,
            non_posting_tuples,
            posting_tuples,
            live_posting_tuples,
            deleted_posting_tuples,
            heap_tid_refs,
            list_ids: list_ids.into_iter().collect(),
        })
    })();
    result
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn rewrite_ivf_postings_from_exclusive_buffer<RowFn, DenseFn, PackedFn>(
    index: IvfPageRelation<'_>,
    buffer: &LockedBufferGuard,
    list_id: u32,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    compact_deletes: bool,
    rewrite_row: &mut RowFn,
    rewrite_dense: &mut DenseFn,
    rewrite_packed: &mut PackedFn,
) -> Result<(), String>
where
    RowFn: FnMut(ItemPointer, IvfPostingTuple) -> Result<IvfPostingRewrite, String>,
    DenseFn: FnMut(
        ItemPointer,
        IvfDensePostingBlockTuple,
    ) -> Result<IvfDensePostingBlockRewrite, String>,
    PackedFn: FnMut(
        ItemPointer,
        IvfDensePostingPackedSegmentTuple,
    ) -> Result<IvfDensePostingPackedSegmentRewrite, String>,
{
    enum PostingVisit {
        NonPosting,
        OtherList,
        Keep,
        Rewrite(Vec<u8>),
        Delete,
    }

    let mut wal_txn = index.start_wal();
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let registered = WalRegisteredPage::new(index.raw(), block_number, page);
    let writer = PageTupleWriter::new(registered.page(), buffer.page_size(), block_number);
    let mut delete_offsets = Vec::new();
    let mut changed = false;
    let mut saw_non_posting_tuple = false;

    for offset in 1..=writer.line_pointer_count() {
        let tuple_visit = writer.visit_line(offset, "posting", |tuple_bytes| {
            let posting_tid = ItemPointer {
                block_number,
                offset_number: offset,
            };
            match tuple_bytes.first().copied() {
                Some(IVF_POSTING_TAG) => {
                    let posting = IvfPostingTuple::decode(tuple_bytes, payload_len)?;
                    if posting.list_id != list_id {
                        return Ok(PostingVisit::OtherList);
                    }

                    match rewrite_row(posting_tid, posting)? {
                        IvfPostingRewrite::Keep => Ok(PostingVisit::Keep),
                        IvfPostingRewrite::Rewrite(updated) => {
                            let encoded = updated.encode()?;
                            if encoded.len() != tuple_bytes.len() {
                                return Err(format!(
                                    "ec_ivf posting tuple size changed from {} to {}",
                                    tuple_bytes.len(),
                                    encoded.len()
                                ));
                            }
                            Ok(PostingVisit::Rewrite(encoded))
                        }
                        IvfPostingRewrite::Delete => Ok(PostingVisit::Delete),
                    }
                }
                Some(tag @ (IVF_DENSE_POSTING_BLOCK_TAG | IVF_DENSE_POSTING_ALIGNED_BLOCK_TAG)) => {
                    let block = IvfDensePostingBlockTuple::decode(tuple_bytes, payload_len)?;
                    if block.list_id != list_id {
                        return Ok(PostingVisit::OtherList);
                    }
                    match rewrite_dense(posting_tid, block)? {
                        IvfDensePostingBlockRewrite::Keep => Ok(PostingVisit::Keep),
                        IvfDensePostingBlockRewrite::Rewrite(updated) => {
                            let encoded = if tag == IVF_DENSE_POSTING_ALIGNED_BLOCK_TAG {
                                updated.encode_aligned()?
                            } else {
                                updated.encode()?
                            };
                            if encoded.len() != tuple_bytes.len() {
                                return Err(format!(
                                    "ec_ivf dense posting block tuple size changed from {} to {}",
                                    tuple_bytes.len(),
                                    encoded.len()
                                ));
                            }
                            Ok(PostingVisit::Rewrite(encoded))
                        }
                    }
                }
                Some(IVF_DENSE_POSTING_PACKED_SEGMENT_TAG) => {
                    let segment =
                        IvfDensePostingPackedSegmentTuple::decode(tuple_bytes, payload_len)?;
                    if segment.list_id != list_id {
                        return Ok(PostingVisit::OtherList);
                    }
                    match rewrite_packed(posting_tid, segment)? {
                        IvfDensePostingPackedSegmentRewrite::Keep => Ok(PostingVisit::Keep),
                        IvfDensePostingPackedSegmentRewrite::Rewrite(updated) => {
                            let encoded = updated.encode()?;
                            if encoded.len() != tuple_bytes.len() {
                                return Err(format!(
                                    "ec_ivf dense posting packed segment tuple size changed from {} to {}",
                                    tuple_bytes.len(),
                                    encoded.len()
                                ));
                            }
                            Ok(PostingVisit::Rewrite(encoded))
                        }
                    }
                }
                _ => Ok(PostingVisit::NonPosting),
            }
        });
        match tuple_visit {
            Ok(PageTupleVisit::Unused) => {}
            Ok(PageTupleVisit::Present(PostingVisit::NonPosting)) => saw_non_posting_tuple = true,
            Ok(PageTupleVisit::Present(PostingVisit::OtherList | PostingVisit::Keep)) => {}
            Ok(PageTupleVisit::Present(PostingVisit::Rewrite(encoded))) => {
                if let Err(err) = writer.copy_required_exact(
                    ItemPointer {
                        block_number,
                        offset_number: offset,
                    },
                    "posting",
                    &encoded,
                ) {
                    std::mem::drop(wal_txn);
                    return Err(err);
                }
                changed = true;
            }
            Ok(PageTupleVisit::Present(PostingVisit::Delete)) => {
                delete_offsets.push(offset);
                changed = true;
            }
            Err(err) => {
                std::mem::drop(wal_txn);
                return Err(err);
            }
        }
    }

    if should_compact_posting_deletes(compact_deletes, saw_non_posting_tuple)
        && !delete_offsets.is_empty()
    {
        registered.multi_delete(&mut delete_offsets)?;
    } else {
        for offset in delete_offsets.iter().rev() {
            registered.delete_no_compact(*offset);
        }
    }

    if changed {
        wal_txn.finish();
    }
    registered.record_free_space(registered.free_space());
    Ok(())
}

fn block_in_range(
    block_number: pg_sys::BlockNumber,
    head_block: pg_sys::BlockNumber,
    tail_block: pg_sys::BlockNumber,
) -> bool {
    block_number != P_NEW && head_block <= block_number && block_number <= tail_block
}

type PostingFreeHintKey = (pg_sys::Oid, u32);

static POSTING_FREE_HINTS: OnceLock<Mutex<HashMap<PostingFreeHintKey, pg_sys::BlockNumber>>> =
    OnceLock::new();

fn posting_free_hint(relid: pg_sys::Oid, list_id: u32) -> Option<pg_sys::BlockNumber> {
    POSTING_FREE_HINTS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("ec_ivf posting free hint mutex poisoned")
        .get(&(relid, list_id))
        .copied()
}

fn remember_posting_free_hint(relid: pg_sys::Oid, list_id: u32, block_number: pg_sys::BlockNumber) {
    POSTING_FREE_HINTS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("ec_ivf posting free hint mutex poisoned")
        .insert((relid, list_id), block_number);
}

fn forget_posting_free_hint(relid: pg_sys::Oid, list_id: u32) {
    POSTING_FREE_HINTS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("ec_ivf posting free hint mutex poisoned")
        .remove(&(relid, list_id));
}

fn should_compact_posting_deletes(compact_deletes: bool, saw_non_posting_tuple: bool) -> bool {
    // Directory and centroid tuple TIDs are persistent metadata links. Compacting
    // a mixed page can renumber those line pointers, so mixed pages must use
    // no-compact deletion even when their deleted postings are reclaimable.
    compact_deletes && !saw_non_posting_tuple
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn read_page_tuple<T, DecodeFn>(
    index: IvfPageRelation<'_>,
    tuple_tid: ItemPointer,
    tuple_kind: &str,
    decode: DecodeFn,
) -> Result<(T, u16), String>
where
    DecodeFn: for<'a> FnOnce(&'a [u8]) -> Result<T, String>,
{
    let buffer = index
        .read_main(
            tuple_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| {
            format!(
                "ec_ivf failed to open block {} for {tuple_kind} tuple",
                tuple_tid.block_number
            )
        })?;

    let page = PageTupleReader::new(&buffer, tuple_tid.block_number);
    let line_pointer_count = page.line_pointer_count();
    if tuple_tid.offset_number == 0 || tuple_tid.offset_number > line_pointer_count {
        return Err(format!(
            "ec_ivf {tuple_kind} tuple offset {} out of range on block {}",
            tuple_tid.offset_number, tuple_tid.block_number
        ));
    }

    let decoded = page.visit_required(tuple_tid.offset_number, tuple_kind, decode);
    decoded.map(|tuple| (tuple, line_pointer_count))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn find_next_tuple_with_tag(
    relation: IvfPageRelation<'_>,
    start_tid: ItemPointer,
    tag: u8,
    tuple_kind: &str,
) -> Result<ItemPointer, String> {
    let block_count = relation.number_of_blocks();
    let mut block_number = start_tid.block_number;
    let mut offset_number = start_tid.offset_number;
    while block_number < block_count {
        let buffer = relation
            .read_main(
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
            .ok_or_else(|| {
                format!(
                    "ec_ivf failed to open block {block_number} while locating next {tuple_kind}"
                )
            })?;

        let page = PageTupleReader::new(&buffer, block_number);
        let line_pointer_count = page.line_pointer_count();
        let result = (|| -> Result<Option<ItemPointer>, String> {
            for offset in offset_number..=line_pointer_count {
                let visit = page.visit_line(offset, tuple_kind, |tuple_bytes| {
                    Ok(tuple_bytes.first().copied() == Some(tag))
                })?;
                if matches!(visit, PageTupleVisit::Present(true)) {
                    return Ok(Some(ItemPointer {
                        block_number,
                        offset_number: offset,
                    }));
                }
            }
            Ok(None)
        })();
        if let Some(next_tid) = result? {
            return Ok(next_tid);
        }

        block_number = block_number
            .checked_add(1)
            .ok_or_else(|| "ec_ivf tuple block number overflow".to_owned())?;
        offset_number = 1;
    }

    Ok(ItemPointer {
        block_number,
        offset_number: 1,
    })
}

fn next_physical_tuple_tid(
    tid: ItemPointer,
    line_pointer_count: u16,
) -> Result<ItemPointer, String> {
    if tid.offset_number < line_pointer_count {
        return Ok(ItemPointer {
            block_number: tid.block_number,
            offset_number: tid.offset_number + 1,
        });
    }

    Ok(ItemPointer {
        block_number: tid
            .block_number
            .checked_add(1)
            .ok_or_else(|| "ec_ivf tuple block number overflow".to_owned())?,
        offset_number: 1,
    })
}

fn page_line_pointer_count(page_ptr: *mut u8) -> u16 {
    let page_header = page_ptr.cast::<pg_sys::PageHeaderData>();
    // SAFETY: callers pass a valid PostgreSQL page pointer; `pd_lower`
    // identifies the end of the line-pointer array.
    ((unsafe { (*page_header).pd_lower } as usize - size_of::<pg_sys::PageHeaderData>())
        / size_of::<pg_sys::ItemIdData>()) as u16
}

fn decode_storage_format(value: u8) -> Result<StorageFormat, String> {
    match value {
        value if value == StorageFormat::Auto as u8 => Ok(StorageFormat::Auto),
        value if value == StorageFormat::TurboQuant as u8 => Ok(StorageFormat::TurboQuant),
        value if value == StorageFormat::PqFastScan as u8 => Ok(StorageFormat::PqFastScan),
        value if value == StorageFormat::RaBitQ as u8 => Ok(StorageFormat::RaBitQ),
        other => Err(format!("invalid ec_ivf storage format code: {other}")),
    }
}

fn decode_rerank(value: u8) -> Result<RerankMode, String> {
    match value {
        value if value == RerankMode::Auto as u8 => Ok(RerankMode::Auto),
        value if value == RerankMode::Off as u8 => Ok(RerankMode::Off),
        value if value == RerankMode::HeapF32 as u8 => Ok(RerankMode::HeapF32),
        value if value == RerankMode::SourceColumn as u8 => Ok(RerankMode::SourceColumn),
        other => Err(format!("invalid ec_ivf rerank code: {other}")),
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn initialize_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: MetadataPage,
) {
    let index = IvfPageRelation::new(ivf_relation_nonnull(index_relation, "ec_ivf metadata init"));
    let existing_blocks = index.number_of_blocks();
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
    let buffer = if target_block == P_NEW {
        index.read_main_locked(target_block, read_mode)
    } else {
        index.read_main(
            target_block,
            read_mode,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
    }
    .unwrap_or_else(|| pgrx::error!("ec_ivf failed to allocate metadata buffer"));

    let page_size = buffer.page_size();
    let mut wal_txn = index.start_wal();
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let registered = WalRegisteredPage::new(index.raw(), buffer.block_number(), page);
    let metadata_bytes = metadata.encode();
    let special_size = align_up(metadata_bytes.len(), ALIGNMENT_BYTES);
    registered.init(page_size, special_size);
    registered.copy_to_special(&metadata_bytes);

    wal_txn.finish();
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn read_metadata_page(index_relation: pg_sys::Relation) -> MetadataPage {
    let index = IvfPageRelation::new(ivf_relation_nonnull(index_relation, "ec_ivf metadata read"));
    let buffer = index.read_main(
        METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    );
    let buffer = buffer.unwrap_or_else(|| pgrx::error!("ec_ivf failed to open metadata buffer"));

    let page = WalRegisteredPage::new(index.raw(), METADATA_BLOCK_NUMBER, buffer.page());
    let metadata_bytes = page.special_bytes(METADATA_BYTES);
    MetadataPage::decode(metadata_bytes).unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) unsafe fn update_metadata_page<F>(
    index_relation: pg_sys::Relation,
    update: F,
) -> Result<MetadataPage, String>
where
    F: FnOnce(&mut MetadataPage) -> Result<(), String>,
{
    let index = IvfPageRelation::new(ivf_relation_nonnull(
        index_relation,
        "ec_ivf metadata update",
    ));
    let buffer = index.read_main(
        METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    );
    let buffer = buffer.ok_or_else(|| "ec_ivf failed to open metadata buffer".to_owned())?;

    let mut wal_txn = index.start_wal();
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let registered = WalRegisteredPage::new(index.raw(), METADATA_BLOCK_NUMBER, page);
    let metadata_bytes = registered.special_bytes(METADATA_BYTES);
    let mut metadata = match MetadataPage::decode(metadata_bytes) {
        Ok(metadata) => metadata,
        Err(err) => {
            std::mem::drop(wal_txn);
            return Err(err);
        }
    };
    if let Err(err) = update(&mut metadata) {
        std::mem::drop(wal_txn);
        return Err(err);
    }

    let encoded = metadata.encode();
    registered.copy_to_special(&encoded);
    wal_txn.finish();
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::page::DEFAULT_PAGE_SIZE;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn block(block_number: u32) -> BlockRef {
        BlockRef { block_number }
    }

    #[test]
    fn metadata_roundtrip() {
        let mut metadata = MetadataPage::empty(EcIvfOptions {
            nlists: 128,
            nprobe: 8,
            rerank_width: 0,
            training_sample_rows: 10_000,
            seed: 7,
            pq_group_size: 0,
            posting_slack_percent: 0,
            quant_bits: 4,
            dense_posting_blocks: false,
            dense_posting_pack_pages: 1,
            dense_posting_typed_layout: false,
            columnar_frozen_lists: false,
            storage_format: StorageFormat::RaBitQ,
            rerank: RerankMode::HeapF32,
        });
        metadata.dimensions = 1536;
        metadata.training_version = 3;
        metadata.centroid_head = tid(12, 2);
        metadata.directory_head = tid(13, 4);
        metadata.total_live_tuples = 42;
        metadata.total_dead_tuples = 5;
        metadata.inserted_since_build = 7;

        let decoded = MetadataPage::decode(&metadata.encode()).unwrap();

        assert_eq!(decoded, metadata);
        assert_eq!(decoded.format_version, INDEX_FORMAT_VERSION);
    }

    #[test]
    fn metadata_decode_rejects_truncated_input() {
        let metadata = MetadataPage::empty(EcIvfOptions {
            nlists: 0,
            nprobe: 0,
            rerank_width: 0,
            training_sample_rows: 0,
            seed: 42,
            pq_group_size: 0,
            posting_slack_percent: 0,
            quant_bits: 4,
            dense_posting_blocks: false,
            dense_posting_pack_pages: 1,
            dense_posting_typed_layout: false,
            columnar_frozen_lists: false,
            storage_format: StorageFormat::Auto,
            rerank: RerankMode::Auto,
        });
        let encoded = metadata.encode();
        let err = MetadataPage::decode(&encoded[..METADATA_BYTES - 1]).unwrap_err();
        assert!(err.contains("metadata length mismatch"));
    }

    #[test]
    fn block_ref_roundtrip() {
        let original = block(99);
        let mut encoded = Vec::new();
        original.encode_into(&mut encoded);
        assert_eq!(BlockRef::decode(&encoded).unwrap(), original);
        assert_eq!(
            BlockRef::decode(&[1, 2, 3]).unwrap_err(),
            "ec_ivf block ref length mismatch: got 3, expected 4"
        );
    }

    #[test]
    fn centroid_tuple_roundtrip() {
        let tuple = IvfCentroidTuple {
            list_id: 3,
            centroid: vec![0.25, -0.5, 1.0],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = IvfCentroidTuple::decode(&encoded, 3).unwrap();
        let borrowed = IvfCentroidTupleRef::decode(&encoded, 3).unwrap();

        assert_eq!(decoded, tuple);
        assert_eq!(borrowed.list_id, 3);
        assert_eq!(borrowed.collect_centroid(), tuple.centroid);
    }

    #[test]
    fn centroid_tuple_rejects_dimension_mismatch() {
        let tuple = IvfCentroidTuple {
            list_id: 0,
            centroid: vec![1.0, 0.0],
        };
        let encoded = tuple.encode().unwrap();

        let err = IvfCentroidTuple::decode(&encoded, 3).unwrap_err();

        assert!(err.contains("centroid tuple length mismatch"));
    }

    #[test]
    fn list_directory_tuple_roundtrip() {
        let tuple = IvfListDirectoryTuple {
            list_id: 9,
            head_block: block(20),
            tail_block: block(25),
            live_count: 101,
            dead_count: 7,
            inserted_since_build: 11,
        };

        let encoded = tuple.encode();
        let decoded = IvfListDirectoryTuple::decode(&encoded).unwrap();

        assert_eq!(decoded, tuple);
        assert_eq!(
            IvfListDirectoryTuple::empty(10).head_block,
            BlockRef::INVALID
        );
    }

    #[test]
    fn columnar_frozen_list_header_roundtrip_records_v1_column_contract() {
        let tuple =
            IvfColumnarFrozenListHeaderTuple::from_shape(7, 3, 5, 4, block(20), block(22)).unwrap();

        let encoded = tuple.encode().unwrap();
        let decoded = IvfColumnarFrozenListHeaderTuple::decode(&encoded).unwrap();
        let borrowed = IvfColumnarFrozenListHeaderRef::decode(&encoded).unwrap();

        assert_eq!(encoded.len(), COLUMNAR_FROZEN_LIST_HEADER_BYTES);
        assert_eq!(encoded[0], IVF_COLUMNAR_FROZEN_LIST_HEADER_TAG);
        assert_eq!(encoded[1], COLUMNAR_FROZEN_LIST_HEADER_VERSION);
        assert_eq!(decoded, tuple);
        assert_eq!(borrowed.list_id, 7);
        assert_eq!(borrowed.posting_count, 3);
        assert_eq!(borrowed.payload_len, 5);
        assert_eq!(borrowed.total_heap_tids, 4);
        assert_eq!(borrowed.gamma_offset, 0);
        assert_eq!(borrowed.payload_offset, 12);
        assert_eq!(borrowed.heap_tid_count_offset, 27);
        assert_eq!(borrowed.heap_tid_offset_offset, 33);
        assert_eq!(borrowed.heap_tid_offset, 45);
        assert_eq!(borrowed.rerank_tid_offset, 69);
        assert_eq!(borrowed.deleted_bitmap_offset, 87);
        assert_eq!(borrowed.total_column_bytes, 88);
        assert_eq!(borrowed.first_column_block, block(20));
        assert_eq!(borrowed.last_column_block, block(22));
        assert!(columnar_frozen_list_header_tuple_fits(DEFAULT_PAGE_SIZE));
    }

    #[test]
    fn columnar_frozen_list_header_rejects_invalid_shape_and_offsets() {
        assert!(
            IvfColumnarFrozenListHeaderTuple::from_shape(1, 0, 4, 0, block(20), block(20))
                .unwrap_err()
                .contains("requires postings")
        );
        assert!(
            IvfColumnarFrozenListHeaderTuple::from_shape(1, 1, 0, 1, block(20), block(20))
                .unwrap_err()
                .contains("requires payload")
        );
        assert!(
            IvfColumnarFrozenListHeaderTuple::from_shape(1, 2, 4, 1, block(20), block(20))
                .unwrap_err()
                .contains("smaller than posting count")
        );
        assert!(IvfColumnarFrozenListHeaderTuple::from_shape(
            1,
            1,
            4,
            1,
            BlockRef::INVALID,
            block(20)
        )
        .unwrap_err()
        .contains("block range is invalid"));
        assert!(
            IvfColumnarFrozenListHeaderTuple::from_shape(1, 1, 4, 1, block(21), block(20))
                .unwrap_err()
                .contains("block range is inverted")
        );

        let tuple =
            IvfColumnarFrozenListHeaderTuple::from_shape(7, 3, 5, 4, block(20), block(22)).unwrap();
        let mut encoded = tuple.encode().unwrap();
        encoded[2] = 1;
        assert!(IvfColumnarFrozenListHeaderTuple::decode(&encoded)
            .unwrap_err()
            .contains("reserved flags"));

        let mut encoded = tuple.encode().unwrap();
        encoded[22..26].copy_from_slice(&13_u32.to_le_bytes());
        assert!(IvfColumnarFrozenListHeaderTuple::decode(&encoded)
            .unwrap_err()
            .contains("column offsets mismatch"));
    }

    #[test]
    fn columnar_frozen_list_columns_preserve_logical_postings() {
        let postings = vec![
            (tid(11, 1), 0.125, tid(101, 1), vec![1, 2, 3]),
            (tid(12, 2), 0.25, tid(102, 2), vec![4, 5, 6]),
            (tid(13, 3), 0.5, tid(103, 3), vec![7, 8, 9]),
            (tid(14, 4), 0.75, tid(104, 4), vec![10, 11, 12]),
        ];
        let columns =
            IvfColumnarFrozenListColumns::from_single_heaptid_postings(&postings, 3).unwrap();
        let header = columns.header(9, block(40), block(43)).unwrap();

        assert_eq!(columns.posting_count, 4);
        assert_eq!(columns.payload_len, 3);
        assert_eq!(columns.total_heap_tids, 4);
        assert_eq!(columns.gamma(0), 0.125);
        assert_eq!(columns.gamma(3), 0.75);
        assert_eq!(columns.payload(1), &[4, 5, 6]);
        assert_eq!(columns.heap_tid_count(2), 1);
        assert_eq!(
            columns.heap_tids(2).collect::<Vec<_>>(),
            vec![postings[2].0]
        );
        assert_eq!(columns.rerank_tid(3), postings[3].2);
        assert!(!columns.is_deleted(0));
        assert_eq!(columns.total_column_bytes().unwrap(), 101);
        assert_eq!(header.list_id, 9);
        assert_eq!(header.posting_count, 4);
        assert_eq!(header.payload_len, 3);
        assert_eq!(header.total_heap_tids, 4);
        assert_eq!(header.total_column_bytes, 101);
    }

    #[test]
    fn columnar_frozen_list_payload_chunks_keep_whole_postings_per_page() {
        let postings = vec![
            (tid(11, 1), 0.125, ItemPointer::INVALID, vec![1, 2, 3]),
            (tid(12, 2), 0.25, ItemPointer::INVALID, vec![4, 5, 6]),
            (tid(13, 3), 0.5, ItemPointer::INVALID, vec![7, 8, 9]),
            (tid(14, 4), 0.75, ItemPointer::INVALID, vec![10, 11, 12]),
            (tid(15, 5), 1.0, ItemPointer::INVALID, vec![13, 14, 15]),
        ];
        let columns =
            IvfColumnarFrozenListColumns::from_single_heaptid_postings(&postings, 3).unwrap();

        let chunks = columns.payload_page_chunks(PAGE_HEADER_BYTES + 7).unwrap();

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].start_item, 0);
        assert_eq!(chunks[0].item_count, 2);
        assert_eq!(chunks[0].bytes, &[1, 2, 3, 4, 5, 6]);
        assert_eq!(chunks[1].start_item, 2);
        assert_eq!(chunks[1].item_count, 2);
        assert_eq!(chunks[1].bytes, &[7, 8, 9, 10, 11, 12]);
        assert_eq!(chunks[2].start_item, 4);
        assert_eq!(chunks[2].item_count, 1);
        assert_eq!(chunks[2].bytes, &[13, 14, 15]);
    }

    #[test]
    fn columnar_frozen_list_raw_pages_keep_all_column_items_whole() {
        let postings = vec![
            (tid(11, 1), 0.125, tid(101, 1), vec![1, 2, 3]),
            (tid(12, 2), 0.25, tid(102, 2), vec![4, 5, 6]),
            (tid(13, 3), 0.5, tid(103, 3), vec![7, 8, 9]),
        ];
        let columns =
            IvfColumnarFrozenListColumns::from_single_heaptid_postings(&postings, 3).unwrap();

        let raw_pages = columns.raw_page_bytes(PAGE_HEADER_BYTES + 7).unwrap();
        let reassembled = raw_pages.concat();

        assert_eq!(reassembled, columns.logical_bytes().unwrap());
        let raw_capacity = columnar_frozen_list_raw_page_capacity(PAGE_HEADER_BYTES + 7);
        assert!(raw_pages.iter().all(|page| page.len() <= raw_capacity));
        assert_eq!(
            raw_pages.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![4, 4, 4, 6, 3, 6, 4, 4, 4, 6, 6, 6, 6, 6, 6, 1]
        );

        let header = columns
            .header(
                11,
                block(40),
                block(40 + u32::try_from(raw_pages.len()).unwrap() - 1),
            )
            .unwrap();
        let encoded_header = header.encode().unwrap();
        let header_ref = IvfColumnarFrozenListHeaderRef::decode(&encoded_header).unwrap();
        assert_eq!(
            columnar_frozen_list_raw_page_lengths(header_ref, PAGE_HEADER_BYTES + 7).unwrap(),
            raw_pages.iter().map(Vec::len).collect::<Vec<_>>()
        );
        let decoded = IvfColumnarFrozenListRef::decode(header_ref, &reassembled).unwrap();
        decoded.validate_offsets().unwrap();
        assert_eq!(decoded.list_id, 11);
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded.gamma(1), 0.25);
        assert_eq!(decoded.payload(2), &[7, 8, 9]);
        assert_eq!(
            decoded.heap_tids(0).collect::<Vec<_>>(),
            vec![postings[0].0]
        );
        assert!(!decoded.is_deleted(2));
    }

    #[test]
    fn columnar_frozen_list_raw_page_chunks_obey_guard_capacity() {
        let raw_capacity = columnar_frozen_list_raw_page_capacity(DEFAULT_PAGE_SIZE);
        let lengths = columnar_page_chunk_lengths(raw_capacity + 6, 1, DEFAULT_PAGE_SIZE).unwrap();

        assert_eq!(lengths, vec![raw_capacity, 6]);
        assert!(lengths.iter().all(|len| *len <= raw_capacity));
    }

    #[test]
    fn columnar_single_page_range_maps_logical_offsets_to_raw_pages() {
        let lengths = vec![4, 6, 3];

        assert_eq!(
            columnar_single_page_range(&lengths, 0, 4).unwrap(),
            IvfColumnarRawPageRange {
                page_index: 0,
                page_offset: 0
            }
        );
        assert_eq!(
            columnar_single_page_range(&lengths, 4, 3).unwrap(),
            IvfColumnarRawPageRange {
                page_index: 1,
                page_offset: 0
            }
        );
        assert_eq!(
            columnar_single_page_range(&lengths, 8, 2).unwrap(),
            IvfColumnarRawPageRange {
                page_index: 1,
                page_offset: 4
            }
        );
    }

    #[test]
    fn columnar_single_page_range_rejects_cross_page_ranges() {
        let err = columnar_single_page_range(&[4, 6, 3], 3, 2).unwrap_err();

        assert!(err.contains("crosses raw page boundary"));
        assert!(columnar_single_page_range(&[4, 6, 3], 13, 1)
            .unwrap_err()
            .contains("outside 13 raw bytes"));
    }

    #[test]
    fn columnar_frozen_list_columns_reject_invalid_input() {
        assert!(
            IvfColumnarFrozenListColumns::from_single_heaptid_postings(&[], 3)
                .unwrap_err()
                .contains("requires postings")
        );
        assert!(IvfColumnarFrozenListColumns::from_single_heaptid_postings(
            &[(tid(1, 1), 0.5, ItemPointer::INVALID, vec![1, 2, 3])],
            0,
        )
        .unwrap_err()
        .contains("requires payload"));
        assert!(IvfColumnarFrozenListColumns::from_single_heaptid_postings(
            &[(tid(1, 1), f32::NAN, ItemPointer::INVALID, vec![1, 2, 3])],
            3,
        )
        .unwrap_err()
        .contains("gamma must be finite"));
        assert!(IvfColumnarFrozenListColumns::from_single_heaptid_postings(
            &[(tid(1, 1), 0.5, ItemPointer::INVALID, vec![1, 2])],
            3,
        )
        .unwrap_err()
        .contains("payload length mismatch"));

        let columns = IvfColumnarFrozenListColumns::from_single_heaptid_postings(
            &[(tid(1, 1), 0.5, ItemPointer::INVALID, vec![1, 2, 3])],
            3,
        )
        .unwrap();
        assert!(columns
            .payload_page_chunks(PAGE_HEADER_BYTES + 2)
            .unwrap_err()
            .contains("does not fit on a page"));
    }

    #[test]
    fn posting_tuple_roundtrip_preserves_duplicate_heap_tids() {
        let tuple = IvfPostingTuple {
            list_id: 2,
            deleted: false,
            heaptids: vec![tid(1, 1), tid(1, 4), tid(2, 1)],
            gamma: 0.75,
            rerank_tid: tid(7, 2),
            payload: vec![1, 2, 3, 4, 5],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = IvfPostingTuple::decode(&encoded, tuple.payload.len()).unwrap();
        let borrowed = IvfPostingTupleRef::decode(&encoded, tuple.payload.len()).unwrap();

        assert_eq!(decoded, tuple);
        assert_eq!(borrowed.heaptid_count(), tuple.heaptids.len());
        assert_eq!(borrowed.collect_heaptids(), tuple.heaptids);
        assert_eq!(borrowed.payload, tuple.payload.as_slice());
    }

    #[test]
    fn dense_posting_block_roundtrip_preserves_scan_arrays() {
        let postings = vec![
            (tid(11, 1), 0.25, ItemPointer::INVALID, vec![1, 2, 3]),
            (tid(12, 2), 0.5, ItemPointer::INVALID, vec![4, 5, 6]),
            (tid(13, 3), 0.75, ItemPointer::INVALID, vec![7, 8, 9]),
        ];
        let tuple =
            IvfDensePostingBlockTuple::from_single_heaptid_postings(4, &postings, 3).unwrap();

        let encoded = tuple.encode().unwrap();
        let decoded = IvfDensePostingBlockTuple::decode(&encoded, 3).unwrap();
        let borrowed = IvfDensePostingBlockRef::decode(&encoded, 3).unwrap();

        assert_eq!(decoded, tuple);
        assert!(dense_posting_block_tuple_fits(
            tuple.gammas.len(),
            tuple.heap_tids.len(),
            tuple.payload_len,
            DEFAULT_PAGE_SIZE
        ));
        assert_eq!(borrowed.list_id, 4);
        assert_eq!(borrowed.len(), 3);
        assert_eq!(borrowed.total_heap_tids(), 3);
        assert_eq!(borrowed.gammas(), vec![0.25, 0.5, 0.75]);
        assert_eq!(borrowed.payload(1), &[4, 5, 6]);
        assert_eq!(
            borrowed.heap_tids(2).collect::<Vec<_>>(),
            vec![postings[2].0]
        );
    }

    #[test]
    fn dense_posting_aligned_block_roundtrip_exposes_native_views() {
        let postings = vec![
            (tid(11, 1), 0.25, ItemPointer::INVALID, vec![1, 2]),
            (tid(12, 2), 0.5, ItemPointer::INVALID, vec![3, 4]),
            (tid(13, 3), 0.75, ItemPointer::INVALID, vec![5, 6]),
        ];
        let tuple =
            IvfDensePostingBlockTuple::from_single_heaptid_postings(4, &postings, 2).unwrap();

        let aligned = tuple.encode_aligned().unwrap();
        let legacy = tuple.encode().unwrap();
        let aligned_ref = IvfDensePostingBlockRef::decode(&aligned, 2).unwrap();
        let legacy_ref = IvfDensePostingBlockRef::decode(&legacy, 2).unwrap();

        assert_eq!(aligned[0], IVF_DENSE_POSTING_ALIGNED_BLOCK_TAG);
        assert_eq!(
            IvfDensePostingBlockTuple::decode(&aligned, 2).unwrap(),
            tuple
        );
        assert_eq!(aligned_ref.gammas_native_le().unwrap(), &[0.25, 0.5, 0.75]);
        assert_eq!(aligned_ref.heap_tid_counts_native_le().unwrap(), &[1, 1, 1]);
        assert_eq!(
            aligned_ref.heap_tid_offsets_native_le().unwrap(),
            &[0, 1, 2]
        );
        assert_eq!(legacy_ref.gammas(), aligned_ref.gammas());
        assert!(legacy_ref.gammas_native_le().is_none());
    }

    #[test]
    fn dense_posting_packed_segment_roundtrip_preserves_logical_block_metadata() {
        let postings = vec![
            (tid(21, 1), 0.125, ItemPointer::INVALID, vec![1, 3]),
            (tid(22, 2), 0.25, ItemPointer::INVALID, vec![5, 7]),
        ];
        let tuple = IvfDensePostingPackedSegmentTuple::from_single_heaptid_postings(
            9, 17, 1, 3, 7, &postings, 2,
        )
        .unwrap();

        let encoded = tuple.encode().unwrap();
        let decoded = IvfDensePostingPackedSegmentTuple::decode(&encoded, 2).unwrap();
        let borrowed = IvfDensePostingPackedSegmentRef::decode(&encoded, 2).unwrap();

        assert_eq!(decoded, tuple);
        assert!(dense_posting_packed_segment_tuple_fits(
            tuple.gammas.len(),
            tuple.heap_tids.len(),
            tuple.payload_len,
            DEFAULT_PAGE_SIZE
        ));
        assert_eq!(borrowed.list_id, 9);
        assert_eq!(borrowed.logical_block_id, 17);
        assert_eq!(borrowed.segment_index, 1);
        assert_eq!(borrowed.segment_count, 3);
        assert_eq!(borrowed.total_posting_count, 7);
        assert_eq!(borrowed.len(), 2);
        assert_eq!(borrowed.total_heap_tids(), 2);
        assert_eq!(borrowed.gammas(), vec![0.125, 0.25]);
        assert_eq!(borrowed.gammas_native_le().unwrap(), &[0.125, 0.25]);
        assert_eq!(borrowed.heap_tid_counts_native_le().unwrap(), &[1, 1]);
        assert_eq!(borrowed.heap_tid_offsets_native_le().unwrap(), &[0, 1]);
        assert_eq!(borrowed.payload(1), &[5, 7]);
        assert_eq!(
            borrowed.heap_tids(0).collect::<Vec<_>>(),
            vec![postings[0].0]
        );
    }

    #[test]
    fn dense_posting_packed_continuation_roundtrip_preserves_payload_slice() {
        let tuple = IvfDensePostingPackedContinuationTuple {
            list_id: 9,
            logical_block_id: 17,
            segment_index: 2,
            segment_count: 4,
            payload_offset: 8192,
            payloads: vec![11, 13, 17, 19],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = IvfDensePostingPackedContinuationTuple::decode(&encoded).unwrap();
        let borrowed = IvfDensePostingPackedContinuationRef::decode(&encoded).unwrap();

        assert_eq!(decoded, tuple);
        assert!(dense_posting_packed_continuation_tuple_fits(
            tuple.payloads.len(),
            DEFAULT_PAGE_SIZE
        ));
        assert_eq!(borrowed.list_id, 9);
        assert_eq!(borrowed.logical_block_id, 17);
        assert_eq!(borrowed.segment_index, 2);
        assert_eq!(borrowed.segment_count, 4);
        assert_eq!(borrowed.payload_offset, 8192);
        assert_eq!(borrowed.payloads, &[11, 13, 17, 19]);
    }

    #[test]
    fn single_heaptid_posting_encoder_matches_generic_encoder() {
        let tuple = IvfPostingTuple {
            list_id: 2,
            deleted: false,
            heaptids: vec![tid(1, 1)],
            gamma: 0.75,
            rerank_tid: ItemPointer::INVALID,
            payload: vec![1, 2, 3, 4, 5],
        };

        let generic = tuple.encode().unwrap();
        let single = IvfPostingTuple::encode_single_heaptid(
            tuple.list_id,
            tuple.deleted,
            tuple.heaptids[0],
            tuple.gamma,
            tuple.rerank_tid,
            &tuple.payload,
        )
        .unwrap();

        assert_eq!(single, generic);
    }

    #[test]
    fn posting_tuple_rejects_heaptid_overflow() {
        let tuple = IvfPostingTuple {
            list_id: 0,
            deleted: false,
            heaptids: (0..=HEAPTID_INLINE_CAPACITY)
                .map(|i| tid(i as u32, 1))
                .collect(),
            gamma: 1.0,
            rerank_tid: ItemPointer::INVALID,
            payload: vec![0],
        };

        let err = tuple.encode().unwrap_err();

        assert!(err.contains("too many ec_ivf posting heap tids"));
    }

    #[test]
    fn pq_codebook_tuple_roundtrip() {
        let tuple = IvfPqCodebookTuple {
            group_index: 2,
            next_tid: tid(9, 3),
            centroids: vec![0.0, 0.25, -0.5, 1.0],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = IvfPqCodebookTuple::decode(&encoded, tuple.centroids.len()).unwrap();
        let borrowed = IvfPqCodebookTupleRef::decode(&encoded, tuple.centroids.len()).unwrap();

        assert_eq!(decoded, tuple);
        assert_eq!(borrowed.group_index, 2);
        assert_eq!(borrowed.next_tid, tuple.next_tid);
        assert_eq!(borrowed.collect_centroids(), tuple.centroids);
    }

    #[test]
    fn data_page_ivf_tuple_roundtrips() {
        let centroid = IvfCentroidTuple {
            list_id: 1,
            centroid: vec![0.0, 1.0],
        };
        let directory = IvfListDirectoryTuple {
            list_id: 1,
            head_block: block(FIRST_DATA_BLOCK_NUMBER),
            tail_block: block(FIRST_DATA_BLOCK_NUMBER),
            live_count: 1,
            dead_count: 0,
            inserted_since_build: 0,
        };
        let posting = IvfPostingTuple {
            list_id: 1,
            deleted: false,
            heaptids: vec![tid(3, 2)],
            gamma: 1.25,
            rerank_tid: ItemPointer::INVALID,
            payload: vec![0xaa, 0xbb],
        };
        let codebook = IvfPqCodebookTuple {
            group_index: 0,
            next_tid: ItemPointer::INVALID,
            centroids: vec![0.0, 0.5],
        };
        let columnar_header =
            IvfColumnarFrozenListHeaderTuple::from_shape(1, 3, 2, 3, block(100), block(101))
                .unwrap();
        let updated_codebook = IvfPqCodebookTuple {
            group_index: 0,
            next_tid: tid(9, 1),
            centroids: vec![1.0, -0.5],
        };

        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);
        let centroid_tid = page.insert_ivf_centroid(&centroid).unwrap();
        let directory_tid = page.insert_ivf_list_directory(directory).unwrap();
        let posting_tid = page.insert_ivf_posting(&posting).unwrap();
        let codebook_tid = page.insert_ivf_pq_codebook(&codebook).unwrap();
        let columnar_tid = page
            .insert_ivf_columnar_frozen_list_header(&columnar_header)
            .unwrap();
        page.update_ivf_pq_codebook(codebook_tid, &updated_codebook)
            .unwrap();

        assert_eq!(page.read_ivf_centroid(centroid_tid, 2).unwrap(), centroid);
        assert_eq!(
            page.read_ivf_list_directory(directory_tid).unwrap(),
            directory
        );
        assert_eq!(
            page.read_ivf_posting(posting_tid, posting.payload.len())
                .unwrap(),
            posting
        );
        assert_eq!(
            page.read_ivf_columnar_frozen_list_header(columnar_tid)
                .unwrap(),
            columnar_header
        );
        assert_eq!(
            IvfPqCodebookTuple::decode(
                page.raw_tuple(codebook_tid).unwrap(),
                updated_codebook.centroids.len()
            )
            .unwrap(),
            updated_codebook
        );
    }

    #[test]
    fn data_page_chain_extends_for_large_posting_tuples() {
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let tuple = IvfPostingTuple {
            list_id: 1,
            deleted: false,
            heaptids: vec![tid(1, 1)],
            gamma: 0.0,
            rerank_tid: ItemPointer::INVALID,
            payload: vec![0x55; 3900],
        };

        let first = chain.insert_ivf_posting(&tuple).unwrap();
        let second = chain.insert_ivf_posting(&tuple).unwrap();
        let third = chain.insert_ivf_posting(&tuple).unwrap();

        assert_eq!(first.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(second.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(third.block_number, FIRST_DATA_BLOCK_NUMBER + 1);
        assert_eq!(
            chain.read_ivf_posting(third, tuple.payload.len()).unwrap(),
            tuple
        );
    }

    #[test]
    fn data_page_chain_ivf_tuple_roundtrips() {
        let centroid = IvfCentroidTuple {
            list_id: 2,
            centroid: vec![0.25, 0.75],
        };
        let directory = IvfListDirectoryTuple {
            list_id: 2,
            head_block: block(FIRST_DATA_BLOCK_NUMBER),
            tail_block: block(FIRST_DATA_BLOCK_NUMBER),
            live_count: 3,
            dead_count: 1,
            inserted_since_build: 4,
        };
        let codebook = IvfPqCodebookTuple {
            group_index: 1,
            next_tid: ItemPointer::INVALID,
            centroids: vec![0.0, 1.0, 2.0, 3.0],
        };
        let columnar_header =
            IvfColumnarFrozenListHeaderTuple::from_shape(2, 2, 4, 2, block(110), block(111))
                .unwrap();
        let updated_codebook = IvfPqCodebookTuple {
            group_index: 1,
            next_tid: tid(4, 2),
            centroids: vec![3.0, 2.0, 1.0, 0.0],
        };

        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let centroid_tid = chain.insert_ivf_centroid(&centroid).unwrap();
        let directory_tid = chain.insert_ivf_list_directory(directory).unwrap();
        let codebook_tid = chain.insert_ivf_pq_codebook(&codebook).unwrap();
        let columnar_tid = chain
            .insert_ivf_columnar_frozen_list_header(&columnar_header)
            .unwrap();
        chain
            .update_ivf_pq_codebook(codebook_tid, &updated_codebook)
            .unwrap();

        assert_eq!(chain.read_ivf_centroid(centroid_tid, 2).unwrap(), centroid);
        assert_eq!(
            chain.read_ivf_list_directory(directory_tid).unwrap(),
            directory
        );
        assert_eq!(
            chain
                .read_ivf_columnar_frozen_list_header(columnar_tid)
                .unwrap(),
            columnar_header
        );
        assert_eq!(
            IvfPqCodebookTuple::decode(
                chain
                    .get_page(codebook_tid.block_number)
                    .unwrap()
                    .raw_tuple(codebook_tid)
                    .unwrap(),
                updated_codebook.centroids.len()
            )
            .unwrap(),
            updated_codebook
        );
    }

    #[test]
    fn layout_fit_helpers_track_page_capacity() {
        assert_eq!(METADATA_BLOCK_NUMBER, 0);
        assert_eq!(FIRST_DATA_BLOCK_NUMBER, 1);
        assert!(centroid_tuple_fits(1536, DEFAULT_PAGE_SIZE));
        assert!(list_directory_tuple_fits(DEFAULT_PAGE_SIZE));
        assert!(posting_tuple_fits(4096, DEFAULT_PAGE_SIZE));
        assert!(columnar_frozen_list_header_tuple_fits(DEFAULT_PAGE_SIZE));
        assert!(pq_codebook_tuple_fits(256, DEFAULT_PAGE_SIZE));
        assert!(!centroid_tuple_fits(1536, 64));
        assert!(!list_directory_tuple_fits(32));
        assert!(!columnar_frozen_list_header_tuple_fits(64));
        assert!(!posting_tuple_fits(DEFAULT_PAGE_SIZE, DEFAULT_PAGE_SIZE));
        assert!(!pq_codebook_tuple_fits(
            DEFAULT_PAGE_SIZE,
            DEFAULT_PAGE_SIZE
        ));
    }

    #[test]
    fn layout_constants_pin_tuple_offsets_and_flags() {
        assert_eq!(
            EC_IVF_POSTING_GAMMA_OFFSET,
            EC_IVF_POSTING_HEAPTIDS_OFFSET + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES
        );
        assert_eq!(
            EC_IVF_POSTING_RERANK_TID_OFFSET,
            EC_IVF_POSTING_GAMMA_OFFSET + size_of::<f32>()
        );
        assert_eq!(
            EC_IVF_POSTING_PAYLOAD_OFFSET,
            EC_IVF_POSTING_RERANK_TID_OFFSET + ITEM_POINTER_BYTES
        );
        assert_eq!(
            EC_IVF_PQ_CODEBOOK_CENTROIDS_OFFSET,
            EC_IVF_PQ_CODEBOOK_NEXT_TID_OFFSET + ITEM_POINTER_BYTES
        );
        assert_eq!(POSTING_FLAG_DELETED, 0b0000_0001);
    }

    #[test]
    fn posting_tuple_rejects_invalid_flags_and_heap_tid_counts() {
        let tuple = IvfPostingTuple {
            list_id: 0,
            deleted: false,
            heaptids: (0..HEAPTID_INLINE_CAPACITY)
                .map(|i| tid(i as u32, 1))
                .collect(),
            gamma: 1.0,
            rerank_tid: ItemPointer::INVALID,
            payload: vec![0],
        };
        let encoded = tuple.encode().unwrap();
        assert_eq!(
            IvfPostingTupleRef::decode(&encoded, tuple.payload.len())
                .unwrap()
                .heaptid_count(),
            HEAPTID_INLINE_CAPACITY
        );

        let mut invalid_flags = encoded.clone();
        invalid_flags[EC_IVF_POSTING_FLAGS_OFFSET] = 0b0000_0010;
        assert!(
            IvfPostingTupleRef::decode(&invalid_flags, tuple.payload.len())
                .unwrap_err()
                .contains("invalid ec_ivf posting tuple flags")
        );

        let mut invalid_count = encoded;
        invalid_count[EC_IVF_POSTING_HEAPTID_COUNT_OFFSET] =
            u8::try_from(HEAPTID_INLINE_CAPACITY + 1).unwrap();
        assert!(
            IvfPostingTupleRef::decode(&invalid_count, tuple.payload.len())
                .unwrap_err()
                .contains("invalid ec_ivf posting heap tid count")
        );
    }

    #[test]
    fn metadata_decode_accepts_known_format_codes_and_rejects_unknown_codes() {
        let mut metadata = MetadataPage::empty(EcIvfOptions {
            nlists: 16,
            nprobe: 4,
            rerank_width: 0,
            training_sample_rows: 512,
            seed: 1,
            pq_group_size: 0,
            posting_slack_percent: 0,
            quant_bits: 4,
            dense_posting_blocks: false,
            dense_posting_pack_pages: 1,
            dense_posting_typed_layout: false,
            columnar_frozen_lists: false,
            storage_format: StorageFormat::Auto,
            rerank: RerankMode::Auto,
        });

        for storage_format in [
            StorageFormat::Auto,
            StorageFormat::TurboQuant,
            StorageFormat::PqFastScan,
            StorageFormat::RaBitQ,
        ] {
            metadata.storage_format = storage_format;
            assert_eq!(
                MetadataPage::decode(&metadata.encode())
                    .unwrap()
                    .storage_format,
                storage_format
            );
        }

        for rerank in [
            RerankMode::Auto,
            RerankMode::Off,
            RerankMode::HeapF32,
            RerankMode::SourceColumn,
        ] {
            metadata.rerank = rerank;
            assert_eq!(
                MetadataPage::decode(&metadata.encode()).unwrap().rerank,
                rerank
            );
        }

        let mut encoded = metadata.encode();
        encoded[EC_IVF_METADATA_STORAGE_FORMAT_OFFSET] = 255;
        assert!(MetadataPage::decode(&encoded)
            .unwrap_err()
            .contains("invalid ec_ivf storage format code"));

        encoded = metadata.encode();
        encoded[EC_IVF_METADATA_RERANK_OFFSET] = 255;
        assert!(MetadataPage::decode(&encoded)
            .unwrap_err()
            .contains("invalid ec_ivf rerank code"));
    }

    #[test]
    fn posting_delete_compaction_is_disabled_on_mixed_pages() {
        assert!(should_compact_posting_deletes(true, false));
        assert!(!should_compact_posting_deletes(true, true));
        assert!(!should_compact_posting_deletes(false, false));
    }

    #[test]
    fn block_in_range_rejects_invalid_and_out_of_range_blocks() {
        assert!(block_in_range(7, 5, 9));
        assert!(!block_in_range(P_NEW, 5, 9));
        assert!(!block_in_range(4, 5, 9));
        assert!(!block_in_range(10, 5, 9));
    }

    #[test]
    fn posting_free_hint_roundtrip_is_keyed_by_relation_and_list() {
        let relid = pg_sys::Oid::from(4242_u32);
        forget_posting_free_hint(relid, 7);
        forget_posting_free_hint(relid, 8);

        assert_eq!(posting_free_hint(relid, 7), None);
        remember_posting_free_hint(relid, 7, 12);

        assert_eq!(posting_free_hint(relid, 7), Some(12));
        assert_eq!(posting_free_hint(relid, 8), None);

        forget_posting_free_hint(relid, 7);
        assert_eq!(posting_free_hint(relid, 7), None);
    }

    #[test]
    fn next_physical_tuple_tid_advances_within_page_and_across_blocks() {
        assert_eq!(next_physical_tuple_tid(tid(5, 2), 4).unwrap(), tid(5, 3));
        assert_eq!(next_physical_tuple_tid(tid(5, 4), 4).unwrap(), tid(6, 1));
        assert!(next_physical_tuple_tid(tid(u32::MAX, 1), 1)
            .unwrap_err()
            .contains("tuple block number overflow"));
    }

    #[test]
    #[cfg(not(any(feature = "pg17", feature = "pg18")))]
    fn page_line_pointer_count_uses_header_lower_bound() {
        let mut header = pg_sys::PageHeaderData {
            pd_lower: (size_of::<pg_sys::PageHeaderData>() + 3 * size_of::<pg_sys::ItemIdData>())
                as u16,
        };

        assert_eq!(
            page_line_pointer_count(std::ptr::addr_of_mut!(header).cast()),
            3
        );
    }
}
