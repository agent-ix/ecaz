//! ec_ivf page layout: metadata, centroid, directory, and posting-list codecs.

use std::mem::size_of;
use std::ptr;

use pgrx::pg_sys;

use super::options::{EcIvfOptions, RerankMode, StorageFormat};
use super::P_NEW;
#[cfg(feature = "pg18")]
use crate::am::stream::{BlockSequencePrefetchState, LinearPrefetchState};
use crate::storage::{
    page::{
        align_up, aligned_tuple_bytes, raw_tuple_storage_bytes, usable_page_bytes, DataPage,
        DataPageChain, ItemPointer, ALIGNMENT_BYTES, HEAPTID_INLINE_CAPACITY, ITEM_POINTER_BYTES,
        PAGE_HEADER_BYTES,
    },
    wal,
};

pub(super) const METADATA_BLOCK_NUMBER: pg_sys::BlockNumber = 0;
pub(super) const FIRST_DATA_BLOCK_NUMBER: pg_sys::BlockNumber = 1;
pub(super) const INDEX_FORMAT_VERSION: u16 = 1;

const METADATA_MAGIC: u32 = 0x5649_4345; // "ECIV" as little-endian bytes.
const METADATA_BYTES: usize = 80;
const BLOCK_REF_BYTES: usize = 4;
const IVF_CENTROID_TAG: u8 = 0x21;
const IVF_LIST_DIRECTORY_TAG: u8 = 0x22;
const IVF_POSTING_TAG: u8 = 0x23;
const POSTING_FLAG_DELETED: u8 = 1 << 0;
const POSTING_FIXED_BYTES: usize =
    1 + 4 + 1 + 1 + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES + 4 + ITEM_POINTER_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MetadataPage {
    pub(super) format_version: u16,
    pub(super) dimensions: u16,
    pub(super) nlists: u32,
    pub(super) nprobe: u32,
    pub(super) training_sample_rows: u32,
    pub(super) training_version: u16,
    pub(super) seed: u64,
    pub(super) storage_format: StorageFormat,
    pub(super) rerank: RerankMode,
    pub(super) centroid_head: ItemPointer,
    pub(super) directory_head: ItemPointer,
    pub(super) total_live_tuples: u64,
    pub(super) total_dead_tuples: u64,
    pub(super) inserted_since_build: u64,
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
        out
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, String> {
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
        })
    }
}

fn write_item_pointer(out: &mut [u8], tid: ItemPointer) {
    debug_assert_eq!(out.len(), ITEM_POINTER_BYTES);
    out[0..4].copy_from_slice(&tid.block_number.to_le_bytes());
    out[4..6].copy_from_slice(&tid.offset_number.to_le_bytes());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BlockRef {
    pub(super) block_number: u32,
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
pub(super) struct IvfCentroidTuple {
    pub(super) list_id: u32,
    pub(super) centroid: Vec<f32>,
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

    pub(super) fn decode(input: &[u8], dimensions: usize) -> Result<Self, String> {
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
pub(super) struct IvfListDirectoryTuple {
    pub(super) list_id: u32,
    pub(super) head_block: BlockRef,
    pub(super) tail_block: BlockRef,
    pub(super) live_count: u64,
    pub(super) dead_count: u64,
    pub(super) inserted_since_build: u64,
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

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
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
pub(super) struct IvfPostingTuple {
    pub(super) list_id: u32,
    pub(super) deleted: bool,
    pub(super) heaptids: Vec<ItemPointer>,
    pub(super) gamma: f32,
    pub(super) rerank_tid: ItemPointer,
    pub(super) payload: Vec<u8>,
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

    pub(super) fn decode(input: &[u8], payload_len: usize) -> Result<Self, String> {
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

pub(super) fn centroid_tuple_fits(dimensions: usize, page_size: usize) -> bool {
    aligned_tuple_bytes(IvfCentroidTuple::encoded_len(dimensions)) <= usable_page_bytes(page_size)
}

pub(super) fn list_directory_tuple_fits(page_size: usize) -> bool {
    aligned_tuple_bytes(IvfListDirectoryTuple::encoded_len()) <= usable_page_bytes(page_size)
}

pub(super) fn posting_tuple_fits(payload_len: usize, page_size: usize) -> bool {
    aligned_tuple_bytes(IvfPostingTuple::encoded_len(payload_len)) <= usable_page_bytes(page_size)
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

pub(super) unsafe fn read_ivf_centroid_and_next(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
    dimensions: usize,
) -> Result<(IvfCentroidTuple, ItemPointer), String> {
    let (centroid, line_pointer_count) = unsafe {
        read_page_tuple(index_relation, tid, "centroid", |tuple_bytes| {
            IvfCentroidTuple::decode(tuple_bytes, dimensions)
        })?
    };
    Ok((centroid, next_physical_tuple_tid(tid, line_pointer_count)?))
}

pub(super) unsafe fn read_ivf_list_directory_and_next(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
) -> Result<(IvfListDirectoryTuple, ItemPointer), String> {
    let (directory, line_pointer_count) = unsafe {
        read_page_tuple(index_relation, tid, "list directory", |tuple_bytes| {
            IvfListDirectoryTuple::decode(tuple_bytes)
        })?
    };
    Ok((directory, next_physical_tuple_tid(tid, line_pointer_count)?))
}

pub(super) unsafe fn read_ivf_postings_for_list_blocks(
    index_relation: pg_sys::Relation,
    list_id: u32,
    head_block: BlockRef,
    tail_block: BlockRef,
    payload_len: usize,
) -> Result<Vec<IvfPostingTuple>, String> {
    let mut postings = Vec::new();
    unsafe {
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
        )?
    };
    Ok(postings)
}

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
        unsafe {
            visit_ivf_posting_blocks_with_read_stream(
                index_relation,
                list_id,
                head_block.block_number,
                tail_block.block_number,
                payload_len,
                &mut visitor,
            )?
        };
    }

    #[cfg(not(feature = "pg18"))]
    {
        for block_number in head_block.block_number..=tail_block.block_number {
            unsafe {
                visit_ivf_postings_for_list_block(
                    index_relation,
                    list_id,
                    block_number,
                    payload_len,
                    &mut visitor,
                )?
            };
        }
    }
    Ok(())
}

pub(super) unsafe fn read_ivf_postings_for_list_blocks_with_tids(
    index_relation: pg_sys::Relation,
    list_id: u32,
    head_block: BlockRef,
    tail_block: BlockRef,
    payload_len: usize,
) -> Result<Vec<(ItemPointer, IvfPostingTuple)>, String> {
    let mut postings = Vec::new();
    unsafe {
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
        )?
    };
    Ok(postings)
}

pub(super) unsafe fn visit_ivf_postings_for_block_sequence<F>(
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
        unsafe {
            visit_ivf_posting_block_sequence_with_read_stream(
                index_relation,
                block_numbers,
                payload_len,
                &mut visitor,
            )?
        };
    }

    #[cfg(not(feature = "pg18"))]
    {
        for block_number in block_numbers {
            unsafe {
                visit_all_ivf_postings_for_block(
                    index_relation,
                    *block_number,
                    payload_len,
                    &mut visitor,
                )?
            };
        }
    }

    Ok(())
}

#[cfg(feature = "pg18")]
unsafe fn visit_ivf_posting_blocks_with_read_stream<F>(
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
    let mut state = LinearPrefetchState::new(head_block, tail_block);
    let stream = unsafe {
        pg_sys::read_stream_begin_relation(
            pg_sys::READ_STREAM_SEQUENTIAL as i32,
            ptr::null_mut(),
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            Some(crate::am::stream::linear_prefetch_cb),
            (&mut state as *mut LinearPrefetchState).cast(),
            size_of::<pg_sys::BlockNumber>(),
        )
    };

    loop {
        let mut per_buffer_data = ptr::null_mut();
        let buffer = unsafe { pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data) };
        if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
            break;
        }
        let block_number = if per_buffer_data.is_null() {
            unsafe { pg_sys::BufferGetBlockNumber(buffer) }
        } else {
            unsafe { *per_buffer_data.cast::<pg_sys::BlockNumber>() }
        };
        let result = unsafe {
            visit_ivf_postings_from_buffer(buffer, list_id, block_number, payload_len, visitor)
        };
        unsafe { pg_sys::ReleaseBuffer(buffer) };
        if let Err(err) = result {
            unsafe { pg_sys::read_stream_end(stream) };
            return Err(err);
        }
    }

    unsafe { pg_sys::read_stream_end(stream) };
    Ok(())
}

#[cfg(feature = "pg18")]
unsafe fn visit_ivf_posting_block_sequence_with_read_stream<F>(
    index_relation: pg_sys::Relation,
    block_numbers: &[pg_sys::BlockNumber],
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    let mut state = BlockSequencePrefetchState::new(block_numbers.to_vec());
    let stream = unsafe {
        pg_sys::read_stream_begin_relation(
            pg_sys::READ_STREAM_SEQUENTIAL as i32,
            ptr::null_mut(),
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            Some(crate::am::stream::block_sequence_prefetch_cb),
            (&mut state as *mut BlockSequencePrefetchState).cast(),
            size_of::<pg_sys::BlockNumber>(),
        )
    };

    loop {
        let mut per_buffer_data = ptr::null_mut();
        let buffer = unsafe { pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data) };
        if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
            break;
        }
        let block_number = if per_buffer_data.is_null() {
            unsafe { pg_sys::BufferGetBlockNumber(buffer) }
        } else {
            unsafe { *per_buffer_data.cast::<pg_sys::BlockNumber>() }
        };
        let result = unsafe {
            visit_all_ivf_postings_from_buffer(buffer, block_number, payload_len, visitor)
        };
        unsafe { pg_sys::ReleaseBuffer(buffer) };
        if let Err(err) = result {
            unsafe { pg_sys::read_stream_end(stream) };
            return Err(err);
        }
    }

    unsafe { pg_sys::read_stream_end(stream) };
    Ok(())
}

#[cfg(not(feature = "pg18"))]
unsafe fn visit_ivf_postings_for_list_block<F>(
    index_relation: pg_sys::Relation,
    list_id: u32,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
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
            "ec_ivf failed to open posting-list block {block_number}"
        ));
    }

    let result = unsafe {
        visit_ivf_postings_from_buffer(buffer, list_id, block_number, payload_len, visitor)
    };
    unsafe { pg_sys::ReleaseBuffer(buffer) };
    result
}

#[cfg(not(feature = "pg18"))]
unsafe fn visit_all_ivf_postings_for_block<F>(
    index_relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
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
            "ec_ivf failed to open posting-list block {block_number}"
        ));
    }

    let result =
        unsafe { visit_all_ivf_postings_from_buffer(buffer, block_number, payload_len, visitor) };
    unsafe { pg_sys::ReleaseBuffer(buffer) };
    result
}

unsafe fn visit_ivf_postings_from_buffer<F>(
    buffer: pg_sys::Buffer,
    list_id: u32,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    unsafe {
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
}

unsafe fn visit_all_ivf_postings_from_buffer<F>(
    buffer: pg_sys::Buffer,
    block_number: pg_sys::BlockNumber,
    payload_len: usize,
    visitor: &mut F,
) -> Result<(), String>
where
    F: FnMut(ItemPointer, IvfPostingTuple) -> Result<(), String>,
{
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let result = (|| -> Result<(), String> {
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
                return Err(format!(
                    "ec_ivf posting tuple bounds exceed block {block_number}"
                ));
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            if tuple_bytes.first().copied() != Some(IVF_POSTING_TAG) {
                continue;
            }

            let posting = IvfPostingTuple::decode(tuple_bytes, payload_len)?;
            visitor(
                ItemPointer {
                    block_number,
                    offset_number: offset,
                },
                posting,
            )?;
        }
        Ok(())
    })();

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_UNLOCK as i32) };
    result
}

pub(super) unsafe fn append_ivf_posting(
    index_relation: pg_sys::Relation,
    tail_block: Option<pg_sys::BlockNumber>,
    tuple: &IvfPostingTuple,
) -> Result<ItemPointer, String> {
    if !posting_tuple_fits(tuple.payload.len(), pg_sys::BLCKSZ as usize) {
        return Err(format!(
            "ec_ivf posting payload {} does not fit on a page",
            tuple.payload.len()
        ));
    }
    let payload = tuple.encode()?;

    if let Some(block_number) = tail_block {
        if let Some(tid) =
            unsafe { try_append_ivf_posting_to_block(index_relation, block_number, &payload)? }
        {
            return Ok(tid);
        }
    }

    unsafe { append_ivf_posting_to_new_block(index_relation, &payload) }
}

unsafe fn try_append_ivf_posting_to_block(
    index_relation: pg_sys::Relation,
    block_number: pg_sys::BlockNumber,
    payload: &[u8],
) -> Result<Option<ItemPointer>, String> {
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
            "ec_ivf failed to open posting-list tail block {block_number}"
        ));
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let free_space = unsafe { pg_sys::PageGetFreeSpace(page) as usize };
    if free_space < raw_tuple_storage_bytes(payload.len()) {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Ok(None);
    }

    let offset = unsafe {
        pg_sys::PageAddItemExtended(
            page,
            payload.as_ptr().cast_mut().cast(),
            payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if offset == pg_sys::InvalidOffsetNumber {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf failed to append posting tuple to block {block_number}"
        ));
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(Some(ItemPointer {
        block_number,
        offset_number: offset,
    }))
}

unsafe fn append_ivf_posting_to_new_block(
    index_relation: pg_sys::Relation,
    payload: &[u8],
) -> Result<ItemPointer, String> {
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
        return Err("ec_ivf failed to allocate posting-list block".to_owned());
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(page, page_size, 0) };

    let offset = unsafe {
        pg_sys::PageAddItemExtended(
            page,
            payload.as_ptr().cast_mut().cast(),
            payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if offset == pg_sys::InvalidOffsetNumber {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err("ec_ivf failed to append posting tuple to new block".to_owned());
    }
    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(ItemPointer {
        block_number,
        offset_number: offset,
    })
}

pub(super) unsafe fn rewrite_ivf_list_directory(
    index_relation: pg_sys::Relation,
    directory_tid: ItemPointer,
    directory: IvfListDirectoryTuple,
) -> Result<(), String> {
    let encoded = directory.encode();
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            directory_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err(format!(
            "ec_ivf failed to open directory block {}",
            directory_tid.block_number
        ));
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let page_ptr = page.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let line_pointer_count = page_line_pointer_count(page_ptr);
    if directory_tid.offset_number == 0 || directory_tid.offset_number > line_pointer_count {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf directory tuple offset {} out of range on block {}",
            directory_tid.offset_number, directory_tid.block_number
        ));
    }

    let item_id = unsafe { &*page_item_id(page_ptr, directory_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err("ec_ivf directory tuple slot is unused".to_owned());
    }
    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf directory tuple bounds exceed block {}",
            directory_tid.block_number
        ));
    }
    if tuple_len != encoded.len() {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf directory tuple size changed from {} to {}",
            tuple_len,
            encoded.len()
        ));
    }

    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), encoded.len())
    };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(())
}

pub(super) unsafe fn update_ivf_list_directory<F>(
    index_relation: pg_sys::Relation,
    directory_tid: ItemPointer,
    update: F,
) -> Result<IvfListDirectoryTuple, String>
where
    F: FnOnce(&mut IvfListDirectoryTuple) -> Result<(), String>,
{
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            directory_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err(format!(
            "ec_ivf failed to open directory block {}",
            directory_tid.block_number
        ));
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let page_ptr = page.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let line_pointer_count = page_line_pointer_count(page_ptr);
    if directory_tid.offset_number == 0 || directory_tid.offset_number > line_pointer_count {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf directory tuple offset {} out of range on block {}",
            directory_tid.offset_number, directory_tid.block_number
        ));
    }

    let item_id = unsafe { &*page_item_id(page_ptr, directory_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err("ec_ivf directory tuple slot is unused".to_owned());
    }
    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf directory tuple bounds exceed block {}",
            directory_tid.block_number
        ));
    }
    if tuple_len != IvfListDirectoryTuple::encoded_len() {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf directory tuple size changed from {} to {}",
            tuple_len,
            IvfListDirectoryTuple::encoded_len()
        ));
    }

    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    let mut directory = match IvfListDirectoryTuple::decode(tuple_bytes) {
        Ok(directory) => directory,
        Err(err) => {
            std::mem::drop(wal_txn);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return Err(err);
        }
    };
    if let Err(err) = update(&mut directory) {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(err);
    }

    let encoded = directory.encode();
    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), encoded.len())
    };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(directory)
}

pub(super) unsafe fn rewrite_ivf_posting(
    index_relation: pg_sys::Relation,
    posting_tid: ItemPointer,
    posting: &IvfPostingTuple,
) -> Result<(), String> {
    let encoded = posting.encode()?;
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            posting_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err(format!(
            "ec_ivf failed to open posting block {}",
            posting_tid.block_number
        ));
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let page_ptr = page.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let line_pointer_count = page_line_pointer_count(page_ptr);
    if posting_tid.offset_number == 0 || posting_tid.offset_number > line_pointer_count {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf posting tuple offset {} out of range on block {}",
            posting_tid.offset_number, posting_tid.block_number
        ));
    }

    let item_id = unsafe { &*page_item_id(page_ptr, posting_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err("ec_ivf posting tuple slot is unused".to_owned());
    }
    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf posting tuple bounds exceed block {}",
            posting_tid.block_number
        ));
    }
    if tuple_len != encoded.len() {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf posting tuple size changed from {} to {}",
            tuple_len,
            encoded.len()
        ));
    }

    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), encoded.len())
    };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(())
}

unsafe fn read_page_tuple<T, DecodeFn>(
    index_relation: pg_sys::Relation,
    tuple_tid: ItemPointer,
    tuple_kind: &str,
    decode: DecodeFn,
) -> Result<(T, u16), String>
where
    DecodeFn: FnOnce(&[u8]) -> Result<T, String>,
{
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            tuple_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err(format!(
            "ec_ivf failed to open block {} for {tuple_kind} tuple",
            tuple_tid.block_number
        ));
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let line_pointer_count = page_line_pointer_count(page_ptr);
    if tuple_tid.offset_number == 0 || tuple_tid.offset_number > line_pointer_count {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf {tuple_kind} tuple offset {} out of range on block {}",
            tuple_tid.offset_number, tuple_tid.block_number
        ));
    }

    let item_id = unsafe { &*page_item_id(page_ptr, tuple_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!("ec_ivf {tuple_kind} tuple slot is unused"));
    }

    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(format!(
            "ec_ivf {tuple_kind} tuple bounds exceed block {}",
            tuple_tid.block_number
        ));
    }

    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    let decoded = decode(tuple_bytes);
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    decoded.map(|tuple| (tuple, line_pointer_count))
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

unsafe fn page_item_id(page_ptr: *mut u8, offset: u16) -> *const pg_sys::ItemIdData {
    unsafe {
        page_ptr
            .add(PAGE_HEADER_BYTES + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()))
            .cast::<pg_sys::ItemIdData>()
    }
}

fn page_line_pointer_count(page_ptr: *mut u8) -> u16 {
    let page_header = page_ptr.cast::<pg_sys::PageHeaderData>();
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

pub(super) unsafe fn initialize_metadata_page(
    index_relation: pg_sys::Relation,
    metadata: MetadataPage,
) {
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
        pgrx::error!("ec_ivf failed to allocate metadata buffer");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    let special_size = align_up(metadata_bytes.len(), ALIGNMENT_BYTES);
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    let page_contents = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    unsafe {
        ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), page_contents, metadata_bytes.len());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

pub(super) unsafe fn read_metadata_page(index_relation: pg_sys::Relation) -> MetadataPage {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("ec_ivf failed to open metadata buffer");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let page = unsafe { pg_sys::BufferGetPage(buffer) };
    let metadata_ptr = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    let metadata_bytes = unsafe { std::slice::from_raw_parts(metadata_ptr, METADATA_BYTES) };
    let metadata = MetadataPage::decode(metadata_bytes).unwrap_or_else(|e| pgrx::error!("{e}"));
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    metadata
}

pub(super) unsafe fn update_metadata_page<F>(
    index_relation: pg_sys::Relation,
    update: F,
) -> Result<MetadataPage, String>
where
    F: FnOnce(&mut MetadataPage) -> Result<(), String>,
{
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err("ec_ivf failed to open metadata buffer".to_owned());
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_ptr = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    let metadata_bytes = unsafe { std::slice::from_raw_parts(metadata_ptr, METADATA_BYTES) };
    let mut metadata = match MetadataPage::decode(metadata_bytes) {
        Ok(metadata) => metadata,
        Err(err) => {
            std::mem::drop(wal_txn);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return Err(err);
        }
    };
    if let Err(err) = update(&mut metadata) {
        std::mem::drop(wal_txn);
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return Err(err);
    }

    let encoded = metadata.encode();
    unsafe { ptr::copy_nonoverlapping(encoded.as_ptr(), metadata_ptr, encoded.len()) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_ivf::options::EcIvfOptions;
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

        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);
        let centroid_tid = page.insert_ivf_centroid(&centroid).unwrap();
        let directory_tid = page.insert_ivf_list_directory(directory).unwrap();
        let posting_tid = page.insert_ivf_posting(&posting).unwrap();

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
    fn layout_fit_helpers_track_page_capacity() {
        assert!(centroid_tuple_fits(1536, DEFAULT_PAGE_SIZE));
        assert!(list_directory_tuple_fits(DEFAULT_PAGE_SIZE));
        assert!(posting_tuple_fits(4096, DEFAULT_PAGE_SIZE));
        assert!(!posting_tuple_fits(DEFAULT_PAGE_SIZE, DEFAULT_PAGE_SIZE));
    }
}
