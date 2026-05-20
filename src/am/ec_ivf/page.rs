//! ec_ivf page layout: metadata, centroid, directory, and posting-list codecs.

#[cfg(any(feature = "pg17", feature = "pg18"))]
use std::collections::BTreeSet;
use std::collections::HashMap;
#[cfg(any(feature = "pg17", feature = "pg18"))]
use std::marker::PhantomData;
use std::mem::size_of;
#[cfg(any(feature = "pg17", feature = "pg18"))]
use std::ptr;
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
use crate::storage::{buffer_guard::LockedBufferGuard, wal};

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) const METADATA_BLOCK_NUMBER: pg_sys::BlockNumber = 0;
#[cfg(not(any(feature = "pg17", feature = "pg18")))]
pub(super) const METADATA_BLOCK_NUMBER: u32 = 0;
#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) const FIRST_DATA_BLOCK_NUMBER: pg_sys::BlockNumber = 1;
#[cfg(not(any(feature = "pg17", feature = "pg18")))]
pub(super) const FIRST_DATA_BLOCK_NUMBER: u32 = 1;
pub const EC_IVF_INDEX_FORMAT_VERSION: u16 = 1;
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
const POSTING_FLAG_DELETED: u8 = 0b0000_0001;
const POSTING_FIXED_BYTES: usize = EC_IVF_POSTING_PAYLOAD_OFFSET;

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
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
enum PageTupleVisit<R> {
    Unused,
    Present(R),
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
#[derive(Clone, Copy)]
struct IvfPageRelation<'a> {
    relation: pg_sys::Relation,
    _relation: PhantomData<&'a pg_sys::RelationData>,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
impl<'a> IvfPageRelation<'a> {
    fn new(relation: pg_sys::Relation) -> Self {
        Self {
            relation,
            _relation: PhantomData,
        }
    }

    fn raw(self) -> pg_sys::Relation {
        self.relation
    }

    fn relid(self) -> pg_sys::Oid {
        // SAFETY: this view is constructed only for a live IVF index relation.
        unsafe { (*self.relation).rd_id }
    }

    fn number_of_blocks(self) -> pg_sys::BlockNumber {
        // SAFETY: this view is constructed only for a live IVF index relation.
        unsafe {
            pg_sys::RelationGetNumberOfBlocksInFork(self.relation, pg_sys::ForkNumber::MAIN_FORKNUM)
        }
    }

    fn page_with_free_space(self, required_space: usize) -> pg_sys::BlockNumber {
        // SAFETY: this view is constructed only for a live IVF index relation;
        // required_space is derived from the tuple size that will be inserted.
        unsafe { pg_sys::GetPageWithFreeSpace(self.relation, required_space) }
    }

    fn read_main(
        self,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
        lockmode: i32,
    ) -> Option<LockedBufferGuard> {
        // SAFETY: this view owns the live-relation contract; callers choose a
        // block/mode/lock combination appropriate for the local page operation.
        unsafe { LockedBufferGuard::read_main(self.relation, block_number, mode, lockmode) }
    }

    fn read_main_locked(
        self,
        block_number: pg_sys::BlockNumber,
        mode: pg_sys::ReadBufferMode::Type,
    ) -> Option<LockedBufferGuard> {
        // SAFETY: this view owns the live-relation contract; callers pass a
        // read mode that returns an already-locked buffer.
        unsafe { LockedBufferGuard::read_main_locked(self.relation, block_number, mode) }
    }

    fn start_wal(self) -> wal::GenericXLogTxn {
        // SAFETY: this view is constructed only for a live IVF index relation.
        unsafe { wal::GenericXLogTxn::start(self.relation) }
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
struct PageTupleReader<'a> {
    page_ptr: *mut u8,
    page_size: usize,
    block_number: pg_sys::BlockNumber,
    line_pointer_count: u16,
    _buffer: PhantomData<&'a LockedBufferGuard>,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
impl<'a> PageTupleReader<'a> {
    fn new(buffer: &'a LockedBufferGuard, block_number: pg_sys::BlockNumber) -> Self {
        let page_ptr = buffer.page().cast::<u8>();
        Self {
            page_ptr,
            page_size: buffer.page_size(),
            block_number,
            line_pointer_count: page_line_pointer_count(page_ptr),
            _buffer: PhantomData,
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
        if offset > self.line_pointer_count {
            return Err(format!(
                "ec_ivf {tuple_kind} tuple offset {offset} out of range on block {}",
                self.block_number
            ));
        }

        // SAFETY: this reader is constructed only from a live `LockedBufferGuard`;
        // `offset` is checked against the cached line-pointer count before the
        // helper exposes tuple bytes for the duration of `visit`.
        unsafe {
            with_page_line_tuple_bytes(
                self.page_ptr,
                self.page_size,
                self.block_number,
                offset,
                tuple_kind,
                visit,
            )
        }
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
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
struct PageTupleWriter {
    page_ptr: *mut u8,
    page_size: usize,
    block_number: pg_sys::BlockNumber,
    line_pointer_count: u16,
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
impl PageTupleWriter {
    fn new(page: pg_sys::Page, page_size: usize, block_number: pg_sys::BlockNumber) -> Self {
        let page_ptr = page.cast::<u8>();
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
        if offset > self.line_pointer_count {
            return Err(format!(
                "ec_ivf {tuple_kind} tuple offset {offset} out of range on block {}",
                self.block_number
            ));
        }

        // SAFETY: this writer is constructed only from a WAL-registered page
        // whose buffer remains locked by the caller. `offset` is checked
        // against the cached line-pointer count before tuple bytes are exposed.
        unsafe {
            with_page_line_tuple_bytes(
                self.page_ptr,
                self.page_size,
                self.block_number,
                offset,
                tuple_kind,
                visit,
            )
        }
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
        match self.visit_line(tid.offset_number, tuple_kind, visit)? {
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
        if offset == 0 || offset > self.line_pointer_count {
            return Err(format!(
                "ec_ivf {tuple_kind} tuple offset {offset} out of range on block {}",
                self.block_number
            ));
        }

        // SAFETY: offset is nonzero and bounded by this writer's cached
        // line-pointer count.
        let item_id = unsafe { &*page_item_id(self.page_ptr, offset) };
        if item_id.lp_flags() == 0 {
            return Err(format!("ec_ivf {tuple_kind} tuple slot is unused"));
        }
        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > self.page_size {
            return Err(format!(
                "ec_ivf {tuple_kind} tuple bounds exceed block {}",
                self.block_number
            ));
        }
        Ok(PageTupleSlot {
            offset: tuple_offset,
            len: tuple_len,
        })
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
        if format_version != INDEX_FORMAT_VERSION {
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

#[derive(Debug, Clone, PartialEq)]
pub struct IvfPostingTuple {
    pub list_id: u32,
    pub deleted: bool,
    pub heaptids: Vec<ItemPointer>,
    pub gamma: f32,
    pub rerank_tid: ItemPointer,
    pub payload: Vec<u8>,
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
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn read_ivf_centroid_and_next(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
    dimensions: usize,
) -> Result<(IvfCentroidTuple, ItemPointer), String> {
    let (centroid, line_pointer_count) =
        read_page_tuple(index_relation, tid, "centroid", |tuple_bytes| {
            IvfCentroidTuple::decode(tuple_bytes, dimensions)
        })?;
    Ok((centroid, next_physical_tuple_tid(tid, line_pointer_count)?))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn read_ivf_list_directory_and_next(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
) -> Result<(IvfListDirectoryTuple, ItemPointer), String> {
    let (directory, line_pointer_count) =
        read_page_tuple(index_relation, tid, "list directory", |tuple_bytes| {
            IvfListDirectoryTuple::decode(tuple_bytes)
        })?;
    let physical_next = next_physical_tuple_tid(tid, line_pointer_count)?;
    let next_directory = find_next_tuple_with_tag(
        index_relation,
        physical_next,
        IVF_LIST_DIRECTORY_TAG,
        "list directory",
    )?;
    Ok((directory, next_directory))
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn read_ivf_pq_codebook(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
    centroid_count: usize,
) -> Result<IvfPqCodebookTuple, String> {
    let (codebook, _) = read_page_tuple(index_relation, tid, "pq codebook", |tuple_bytes| {
        IvfPqCodebookTuple::decode(tuple_bytes, centroid_count)
    })?;
    Ok(codebook)
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn read_ivf_postings_for_list_blocks(
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
pub(super) fn visit_ivf_postings_for_list_blocks<F>(
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
pub(super) fn read_ivf_postings_for_list_blocks_with_tids(
    index_relation: pg_sys::Relation,
    list_id: u32,
    head_block: BlockRef,
    tail_block: BlockRef,
    payload_len: usize,
) -> Result<Vec<(ItemPointer, IvfPostingTuple)>, String> {
    let mut postings = Vec::new();
    visit_ivf_postings_for_list_blocks(
        index_relation,
        list_id,
        head_block,
        tail_block,
        payload_len,
        |posting_tid, posting| {
            postings.push((posting_tid, posting));
            Ok(())
        },
    )?;
    Ok(postings)
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn rewrite_ivf_postings_for_list_blocks<F>(
    index_relation: pg_sys::Relation,
    list_id: u32,
    head_block: BlockRef,
    tail_block: BlockRef,
    payload_len: usize,
    no_compact_blocks: &[pg_sys::BlockNumber],
    mut rewrite: F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<IvfPostingRewrite, String>,
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

    for block_number in head_block.block_number..=tail_block.block_number {
        rewrite_ivf_postings_for_list_block(
            index_relation,
            list_id,
            block_number,
            payload_len,
            !no_compact_blocks.contains(&block_number),
            &mut rewrite,
        )?;
    }

    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn visit_ivf_postings_for_block_sequence<F>(
    index_relation: pg_sys::Relation,
    block_numbers: &[pg_sys::BlockNumber],
    payload_len: usize,
    mut visitor: F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    if block_numbers.is_empty() {
        return Ok(());
    }

    #[cfg(feature = "pg18")]
    {
        visit_ivf_posting_block_sequence_with_read_stream(
            index_relation,
            block_numbers,
            payload_len,
            &mut visitor,
        )?;
    }

    #[cfg(not(feature = "pg18"))]
    {
        for block_number in block_numbers {
            visit_all_ivf_postings_for_block(
                index_relation,
                *block_number,
                payload_len,
                &mut visitor,
            )?;
        }
    }

    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
pub(super) fn visit_ivf_posting_refs_for_block_sequence<F>(
    index_relation: pg_sys::Relation,
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
            index_relation,
            block_numbers,
            payload_len,
            &mut visitor,
        )?;
    }

    #[cfg(not(feature = "pg18"))]
    {
        for block_number in block_numbers {
            visit_all_ivf_posting_refs_for_block(
                index_relation,
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
fn visit_ivf_posting_block_sequence_with_read_stream<F>(
    index_relation: pg_sys::Relation,
    block_numbers: &[pg_sys::BlockNumber],
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    crate::am::stream::visit_relation_block_sequence_read_stream(
        index_relation,
        block_numbers,
        "ec_ivf posting block sequence",
        |buffer, block_number| {
            visit_all_ivf_postings_from_buffer(buffer, block_number, payload_len, visitor)
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
    // SAFETY: the caller supplies a live index relation and a posting-list
    // block number from IVF list metadata; the guard pins and share-locks it.
    let buffer = unsafe {
        LockedBufferGuard::read_main(
            index_relation,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
    }
    .ok_or_else(|| format!("ec_ivf failed to open posting-list block {block_number}"))?;

    let result =
        visit_ivf_postings_from_buffer(&buffer, list_id, block_number, payload_len, visitor);
    result
}

#[cfg(all(any(feature = "pg17", feature = "pg18"), not(feature = "pg18")))]
fn visit_all_ivf_postings_for_block<F>(
    index_relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    // SAFETY: the caller supplies a live index relation and a posting block
    // from an IVF block sequence; the guard pins and share-locks it.
    let buffer = unsafe {
        LockedBufferGuard::read_main(
            index_relation,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
    }
    .ok_or_else(|| format!("ec_ivf failed to open posting-list block {block_number}"))?;

    let result = visit_all_ivf_postings_from_buffer(&buffer, block_number, payload_len, visitor);
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
    // SAFETY: the caller supplies a live index relation and a posting block
    // from an IVF block sequence; the guard pins and share-locks it.
    let buffer = unsafe {
        LockedBufferGuard::read_main(
            index_relation,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
    }
    .ok_or_else(|| format!("ec_ivf failed to open posting-list block {block_number}"))?;

    let result =
        visit_all_ivf_posting_refs_from_buffer(&buffer, block_number, payload_len, visitor);
    result
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
pub(super) fn append_ivf_posting_to_list_range(
    index_relation: pg_sys::Relation,
    block_range: Option<(pg_sys::BlockNumber, pg_sys::BlockNumber)>,
    tuple: &IvfPostingTuple,
) -> Result<ItemPointer, String> {
    // SAFETY: caller supplies a live IVF index relation for the append path.
    let index = IvfPageRelation::new(index_relation);
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
pub(super) fn rewrite_ivf_list_directory(
    index_relation: pg_sys::Relation,
    directory_tid: ItemPointer,
    directory: IvfListDirectoryTuple,
) -> Result<(), String> {
    // SAFETY: caller supplies a live IVF index relation for the rewrite path.
    let index = IvfPageRelation::new(index_relation);
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
pub(super) fn update_ivf_list_directory<F>(
    index_relation: pg_sys::Relation,
    directory_tid: ItemPointer,
    update: F,
) -> Result<IvfListDirectoryTuple, String>
where
    F: FnOnce(&mut IvfListDirectoryTuple) -> Result<(), String>,
{
    // SAFETY: caller supplies a live IVF index relation for the update path.
    let index = IvfPageRelation::new(index_relation);
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
pub(super) unsafe fn rewrite_ivf_posting(
    index_relation: pg_sys::Relation,
    posting_tid: ItemPointer,
    posting: &IvfPostingTuple,
) -> Result<(), String> {
    // SAFETY: caller supplies a live IVF index relation for the rewrite path.
    let index = IvfPageRelation::new(index_relation);
    let encoded = posting.encode()?;
    let buffer = index
        .read_main(
            posting_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
        .ok_or_else(|| {
            format!(
                "ec_ivf failed to open posting block {}",
                posting_tid.block_number
            )
        })?;

    let mut wal_txn = index.start_wal();
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let writer = PageTupleWriter::new(page, buffer.page_size(), posting_tid.block_number);
    if let Err(err) = writer.copy_required_exact(posting_tid, "posting", &encoded) {
        std::mem::drop(wal_txn);
        return Err(err);
    }
    wal_txn.finish();
    Ok(())
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
#[derive(Debug, Clone, PartialEq)]
pub(super) enum IvfPostingRewrite {
    Keep,
    Rewrite(IvfPostingTuple),
    Delete,
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
    // SAFETY: `index_relation` is live; this only reads the current main-fork
    // block count to bound the debug scan.
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut summaries = Vec::new();
    for block_number in FIRST_DATA_BLOCK_NUMBER..block_count {
        let summary = debug_ivf_posting_block_summary(index_relation, block_number, payload_len)?;
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
fn rewrite_ivf_postings_for_list_block<F>(
    index_relation: pg_sys::Relation,
    list_id: u32,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    compact_deletes: bool,
    rewrite: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<IvfPostingRewrite, String>,
{
    // SAFETY: `block_number` comes from IVF posting-list metadata, and an
    // exclusive lock is required before rewriting or deleting tuples.
    let buffer = unsafe {
        LockedBufferGuard::read_main(
            index_relation,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
    }
    .ok_or_else(|| format!("ec_ivf failed to open posting-list block {block_number}"))?;

    rewrite_ivf_postings_from_exclusive_buffer(
        index_relation,
        &buffer,
        list_id,
        block_number,
        payload_len,
        compact_deletes,
        rewrite,
    )
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
fn debug_ivf_posting_block_summary(
    index_relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
) -> Result<IvfPostingBlockSummary, String> {
    // SAFETY: `block_number` is bounded by the caller's block-count scan, and
    // a share lock is sufficient for read-only debug summarization.
    let buffer = unsafe {
        LockedBufferGuard::read_main(
            index_relation,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
    }
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
fn rewrite_ivf_postings_from_exclusive_buffer<F>(
    index_relation: pg_sys::Relation,
    buffer: &LockedBufferGuard,
    list_id: u32,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    compact_deletes: bool,
    rewrite: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<IvfPostingRewrite, String>,
{
    enum PostingVisit {
        NonPosting,
        OtherList,
        Keep,
        Rewrite(Vec<u8>),
        Delete,
    }

    // SAFETY: starts a generic WAL transaction for the live index relation;
    // the caller holds `buffer` with an exclusive lock for this block.
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = wal_txn.register_locked_buffer_full_image(&buffer);
    let registered = WalRegisteredPage::new(index_relation, block_number, page);
    let writer = PageTupleWriter::new(registered.page(), buffer.page_size(), block_number);
    let mut delete_offsets = Vec::new();
    let mut changed = false;
    let mut saw_non_posting_tuple = false;

    for offset in 1..=writer.line_pointer_count() {
        let tuple_visit = writer.visit_line(offset, "posting", |tuple_bytes| {
            if tuple_bytes.first().copied() != Some(IVF_POSTING_TAG) {
                return Ok(PostingVisit::NonPosting);
            }

            let posting = IvfPostingTuple::decode(tuple_bytes, payload_len)?;
            if posting.list_id != list_id {
                return Ok(PostingVisit::OtherList);
            }

            let posting_tid = ItemPointer {
                block_number,
                offset_number: offset,
            };
            match rewrite(posting_tid, posting)? {
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
    index_relation: pg_sys::Relation,
    tuple_tid: ItemPointer,
    tuple_kind: &str,
    decode: DecodeFn,
) -> Result<(T, u16), String>
where
    DecodeFn: for<'a> FnOnce(&'a [u8]) -> Result<T, String>,
{
    // SAFETY: `tuple_tid` identifies a tuple on the live index relation; a
    // share lock is sufficient because this helper only reads and decodes.
    let buffer = unsafe {
        LockedBufferGuard::read_main(
            index_relation,
            tuple_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
    }
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
    index_relation: pg_sys::Relation,
    start_tid: ItemPointer,
    tag: u8,
    tuple_kind: &str,
) -> Result<ItemPointer, String> {
    // SAFETY: `index_relation` is live; this only reads the main-fork block
    // count to bound the forward scan.
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut block_number = start_tid.block_number;
    let mut offset_number = start_tid.offset_number;
    while block_number < block_count {
        // SAFETY: `block_number` is bounded by the relation block count above,
        // and a share lock is sufficient for scanning tuple tags.
        let buffer = unsafe {
            LockedBufferGuard::read_main(
                index_relation,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
        }
        .ok_or_else(|| {
            format!("ec_ivf failed to open block {block_number} while locating next {tuple_kind}")
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

#[cfg(any(feature = "pg17", feature = "pg18"))]
unsafe fn page_item_id(page_ptr: *mut u8, offset: u16) -> *const pg_sys::ItemIdData {
    // SAFETY: callers pass a page pointer and a nonzero line pointer offset
    // that has been range-checked against `page_line_pointer_count`.
    unsafe {
        page_ptr
            .add(PAGE_HEADER_BYTES + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()))
            .cast::<pg_sys::ItemIdData>()
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
unsafe fn with_page_line_tuple_bytes<R, F>(
    page_ptr: *mut u8,
    page_size: usize,
    block_number: pg_sys::BlockNumber,
    offset: u16,
    tuple_kind: &str,
    visit: F,
) -> Result<PageTupleVisit<R>, String>
where
    F: for<'a> FnOnce(&'a [u8]) -> Result<R, String>,
{
    if offset == 0 {
        return Err(format!(
            "ec_ivf {tuple_kind} tuple offset 0 out of range on block {block_number}"
        ));
    }

    // SAFETY: `offset` is nonzero and callers only use this helper after
    // bounding offsets by the page's line-pointer count.
    let item_id = unsafe { &*page_item_id(page_ptr, offset) };
    if item_id.lp_flags() == 0 {
        return Ok(PageTupleVisit::Unused);
    }

    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        return Err(format!(
            "ec_ivf {tuple_kind} tuple bounds exceed block {block_number}"
        ));
    }
    // SAFETY: tuple offset and length were checked against `page_size`, and
    // the page remains locked for the duration of the visitor call.
    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    visit(tuple_bytes).map(PageTupleVisit::Present)
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
pub(super) fn initialize_metadata_page(index_relation: pg_sys::Relation, metadata: MetadataPage) {
    // SAFETY: caller supplies a live IVF index relation for metadata init.
    let index = IvfPageRelation::new(index_relation);
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
pub(super) fn read_metadata_page(index_relation: pg_sys::Relation) -> MetadataPage {
    // SAFETY: caller supplies a live IVF index relation for metadata read.
    let index = IvfPageRelation::new(index_relation);
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
pub(super) fn update_metadata_page<F>(
    index_relation: pg_sys::Relation,
    update: F,
) -> Result<MetadataPage, String>
where
    F: FnOnce(&mut MetadataPage) -> Result<(), String>,
{
    // SAFETY: caller supplies a live IVF index relation for metadata update.
    let index = IvfPageRelation::new(index_relation);
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
        let updated_codebook = IvfPqCodebookTuple {
            group_index: 1,
            next_tid: tid(4, 2),
            centroids: vec![3.0, 2.0, 1.0, 0.0],
        };

        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let centroid_tid = chain.insert_ivf_centroid(&centroid).unwrap();
        let directory_tid = chain.insert_ivf_list_directory(directory).unwrap();
        let codebook_tid = chain.insert_ivf_pq_codebook(&codebook).unwrap();
        chain
            .update_ivf_pq_codebook(codebook_tid, &updated_codebook)
            .unwrap();

        assert_eq!(chain.read_ivf_centroid(centroid_tid, 2).unwrap(), centroid);
        assert_eq!(
            chain.read_ivf_list_directory(directory_tid).unwrap(),
            directory
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
        assert!(pq_codebook_tuple_fits(256, DEFAULT_PAGE_SIZE));
        assert!(!centroid_tuple_fits(1536, 64));
        assert!(!list_directory_tuple_fits(32));
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
        let mut bytes =
            vec![0_u8; size_of::<pg_sys::PageHeaderData>() + 4 * size_of::<pg_sys::ItemIdData>()];
        let header = bytes.as_mut_ptr().cast::<pg_sys::PageHeaderData>();
        // SAFETY: `bytes` is large enough for `PageHeaderData`, and this test
        // writes only the synthetic `pd_lower` field before reading it back.
        unsafe {
            (*header).pd_lower =
                (size_of::<pg_sys::PageHeaderData>() + 3 * size_of::<pg_sys::ItemIdData>()) as u16;
        }

        assert_eq!(page_line_pointer_count(bytes.as_mut_ptr()), 3);
    }
}
