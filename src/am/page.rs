//! Page-layout primitives for `tqhnsw`.

use std::mem::size_of;

const DEFAULT_PAGE_SIZE: usize = 8192;
pub const PAGE_HEADER_BYTES: usize = 24;
const LINE_POINTER_BYTES: usize = 4;
const TUPLE_HEADER_BYTES: usize = 4;
const ALIGNMENT_BYTES: usize = 8;

pub const TQ_ELEMENT_TAG: u8 = 0x01;
pub const TQ_NEIGHBOR_TAG: u8 = 0x02;
pub const TQ_GROUPED_HOT_TAG: u8 = 0x03;
pub const TQ_RERANK_TAG: u8 = 0x04;
pub const TQ_GROUPED_CODEBOOK_TAG: u8 = 0x05;
pub const HEAPTID_INLINE_CAPACITY: usize = 10;
pub const ITEM_POINTER_BYTES: usize = 6;
pub const METADATA_BLOCK_NUMBER: u32 = 0;
pub const FIRST_DATA_BLOCK_NUMBER: u32 = 1;
const LEGACY_METADATA_BYTES: usize = 2 + 2 + ITEM_POINTER_BYTES + 2 + 1 + 1 + 8 + 8;
const METADATA_BYTES: usize =
    LEGACY_METADATA_BYTES + 2 + 1 + 1 + 1 + 1 + 1 + 2 + 2 + ITEM_POINTER_BYTES;

const fn align_up(value: usize, alignment: usize) -> usize {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

const LEGACY_METADATA_SPECIAL_BYTES: usize = align_up(LEGACY_METADATA_BYTES, ALIGNMENT_BYTES);
const METADATA_SPECIAL_BYTES: usize = align_up(METADATA_BYTES, ALIGNMENT_BYTES);

pub const INDEX_FORMAT_V1_SCALAR: u16 = 1;
pub const INDEX_FORMAT_V2_GROUPED: u16 = 2;
pub const PAYLOAD_FLAG_BINARY_SIDECAR: u8 = 1 << 0;
pub const PAYLOAD_FLAG_GROUPED_SEARCH_CODE: u8 = 1 << 1;
pub const PAYLOAD_FLAG_COLD_RERANK_PAYLOAD: u8 = 1 << 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransformKind {
    Unknown = 0,
    Srht = 1,
    Opq = 2,
}

impl TransformKind {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Srht),
            2 => Ok(Self::Opq),
            other => Err(format!("invalid transform kind: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SearchCodecKind {
    Unknown = 0,
    ScalarQuantized = 1,
    GroupedPq = 2,
}

impl SearchCodecKind {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::ScalarQuantized),
            2 => Ok(Self::GroupedPq),
            other => Err(format!("invalid search codec kind: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RerankCodecKind {
    None = 0,
    ScalarQuantized = 1,
    GroupedPq = 2,
}

impl RerankCodecKind {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::ScalarQuantized),
            2 => Ok(Self::GroupedPq),
            other => Err(format!("invalid rerank codec kind: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphStorageFormat {
    ScalarV1,
    GroupedV2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemPointer {
    pub block_number: u32,
    pub offset_number: u16,
}

impl ItemPointer {
    pub const INVALID: Self = Self {
        block_number: u32::MAX,
        offset_number: u16::MAX,
    };

    pub fn encode_into(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.block_number.to_le_bytes());
        out.extend_from_slice(&self.offset_number.to_le_bytes());
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != ITEM_POINTER_BYTES {
            return Err(format!(
                "item pointer length mismatch: got {}, expected {ITEM_POINTER_BYTES}",
                input.len()
            ));
        }

        Ok(Self {
            block_number: u32::from_le_bytes(input[..4].try_into().expect("block number bytes")),
            offset_number: u16::from_le_bytes(input[4..6].try_into().expect("offset bytes")),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataPage {
    pub m: u16,
    pub ef_construction: u16,
    pub entry_point: ItemPointer,
    pub dimensions: u16,
    pub bits: u8,
    pub max_level: u8,
    pub seed: u64,
    pub inserted_since_rebuild: u64,
    pub format_version: u16,
    pub transform_kind: TransformKind,
    pub search_codec_kind: SearchCodecKind,
    pub payload_flags: u8,
    pub search_bits: u8,
    pub rerank_codec_kind: RerankCodecKind,
    pub search_subvector_count: u16,
    pub search_subvector_dim: u16,
    pub grouped_codebook_head: ItemPointer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentFormatMetadata {
    pub m: u16,
    pub ef_construction: u16,
    pub entry_point: ItemPointer,
    pub dimensions: u16,
    pub bits: u8,
    pub max_level: u8,
    pub seed: u64,
    pub inserted_since_rebuild: u64,
    pub persisted_binary_sidecar: bool,
}

impl MetadataPage {
    pub fn graph_storage_format(&self) -> Result<GraphStorageFormat, String> {
        match self.format_version {
            INDEX_FORMAT_V1_SCALAR => Ok(GraphStorageFormat::ScalarV1),
            INDEX_FORMAT_V2_GROUPED => Ok(GraphStorageFormat::GroupedV2),
            other => Err(format!("unsupported metadata format version: {other}")),
        }
    }

    pub fn current_v1_scalar(current: CurrentFormatMetadata) -> Self {
        let mut payload_flags = 0_u8;
        if current.persisted_binary_sidecar {
            payload_flags |= PAYLOAD_FLAG_BINARY_SIDECAR;
        }

        Self {
            m: current.m,
            ef_construction: current.ef_construction,
            entry_point: current.entry_point,
            dimensions: current.dimensions,
            bits: current.bits,
            max_level: current.max_level,
            seed: current.seed,
            inserted_since_rebuild: current.inserted_since_rebuild,
            format_version: INDEX_FORMAT_V1_SCALAR,
            transform_kind: if current.dimensions == 0 {
                TransformKind::Unknown
            } else {
                TransformKind::Srht
            },
            search_codec_kind: if current.dimensions == 0 || current.bits == 0 {
                SearchCodecKind::Unknown
            } else {
                SearchCodecKind::ScalarQuantized
            },
            payload_flags,
            search_bits: current.bits,
            rerank_codec_kind: RerankCodecKind::None,
            search_subvector_count: 0,
            search_subvector_dim: 0,
            grouped_codebook_head: ItemPointer::INVALID,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(METADATA_BYTES);
        out.extend_from_slice(&self.m.to_le_bytes());
        out.extend_from_slice(&self.ef_construction.to_le_bytes());
        self.entry_point.encode_into(&mut out);
        out.extend_from_slice(&self.dimensions.to_le_bytes());
        out.push(self.bits);
        out.push(self.max_level);
        out.extend_from_slice(&self.seed.to_le_bytes());
        out.extend_from_slice(&self.inserted_since_rebuild.to_le_bytes());
        out.extend_from_slice(&self.format_version.to_le_bytes());
        out.push(self.transform_kind as u8);
        out.push(self.search_codec_kind as u8);
        out.push(self.payload_flags);
        out.push(self.search_bits);
        out.push(self.rerank_codec_kind as u8);
        out.extend_from_slice(&self.search_subvector_count.to_le_bytes());
        out.extend_from_slice(&self.search_subvector_dim.to_le_bytes());
        self.grouped_codebook_head.encode_into(&mut out);
        out
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() == LEGACY_METADATA_BYTES {
            return Self::decode_legacy(input);
        }
        if input.len() != METADATA_BYTES {
            return Err(format!(
                "metadata page length mismatch: got {}, expected {METADATA_BYTES}",
                input.len()
            ));
        }

        let format_version =
            u16::from_le_bytes(input[30..32].try_into().expect("format version bytes"));
        if !matches!(
            format_version,
            INDEX_FORMAT_V1_SCALAR | INDEX_FORMAT_V2_GROUPED
        ) {
            return Err(format!("invalid metadata format version: {format_version}"));
        }

        Ok(Self {
            m: u16::from_le_bytes(input[0..2].try_into().expect("m bytes")),
            ef_construction: u16::from_le_bytes(
                input[2..4].try_into().expect("ef construction bytes"),
            ),
            entry_point: ItemPointer::decode(&input[4..10])?,
            dimensions: u16::from_le_bytes(input[10..12].try_into().expect("dimension bytes")),
            bits: input[12],
            max_level: input[13],
            seed: u64::from_le_bytes(input[14..22].try_into().expect("seed bytes")),
            inserted_since_rebuild: u64::from_le_bytes(
                input[22..30]
                    .try_into()
                    .expect("inserted-since-rebuild bytes"),
            ),
            format_version,
            transform_kind: TransformKind::decode(input[32])?,
            search_codec_kind: SearchCodecKind::decode(input[33])?,
            payload_flags: input[34],
            search_bits: input[35],
            rerank_codec_kind: RerankCodecKind::decode(input[36])?,
            search_subvector_count: u16::from_le_bytes(
                input[37..39]
                    .try_into()
                    .expect("search subvector count bytes"),
            ),
            search_subvector_dim: u16::from_le_bytes(
                input[39..41]
                    .try_into()
                    .expect("search subvector dim bytes"),
            ),
            grouped_codebook_head: ItemPointer::decode(&input[41..47])?,
        })
    }

    fn decode_legacy(input: &[u8]) -> Result<Self, String> {
        Ok(Self::current_v1_scalar(CurrentFormatMetadata {
            m: u16::from_le_bytes(input[0..2].try_into().expect("m bytes")),
            ef_construction: u16::from_le_bytes(
                input[2..4].try_into().expect("ef construction bytes"),
            ),
            entry_point: ItemPointer::decode(&input[4..10])?,
            dimensions: u16::from_le_bytes(input[10..12].try_into().expect("dimension bytes")),
            bits: input[12],
            max_level: input[13],
            seed: u64::from_le_bytes(input[14..22].try_into().expect("seed bytes")),
            inserted_since_rebuild: u64::from_le_bytes(
                input[22..30]
                    .try_into()
                    .expect("inserted-since-rebuild bytes"),
            ),
            persisted_binary_sidecar: false,
        }))
    }

    pub fn encode_page(&self, page_size: usize) -> Result<Vec<u8>, String> {
        if page_size < PAGE_HEADER_BYTES + METADATA_BYTES {
            return Err(format!(
                "page size {page_size} too small for metadata page payload {}",
                PAGE_HEADER_BYTES + METADATA_BYTES
            ));
        }

        let mut page = vec![0_u8; page_size];
        let metadata = self.encode();
        let special_offset = page_size - METADATA_SPECIAL_BYTES;
        page[special_offset..special_offset + metadata.len()].copy_from_slice(&metadata);
        Ok(page)
    }

    pub fn decode_page(page: &[u8]) -> Result<Self, String> {
        if page.len() < PAGE_HEADER_BYTES + LEGACY_METADATA_BYTES {
            return Err(format!(
                "page too short: got {}, need at least {}",
                page.len(),
                PAGE_HEADER_BYTES + LEGACY_METADATA_BYTES
            ));
        }

        if page.len() >= PAGE_HEADER_BYTES + METADATA_BYTES {
            let special_offset = page.len() - METADATA_SPECIAL_BYTES;
            let candidate = &page[special_offset..special_offset + METADATA_BYTES];
            if let Ok(metadata) = Self::decode(candidate) {
                return Ok(metadata);
            }
        }

        let legacy_offset = page.len() - LEGACY_METADATA_SPECIAL_BYTES;
        Self::decode(&page[legacy_offset..legacy_offset + LEGACY_METADATA_BYTES])
    }

    pub fn decode_contents(contents: &[u8]) -> Result<Self, String> {
        if contents.len() < LEGACY_METADATA_BYTES {
            return Err(format!(
                "page contents too short: got {}, need at least {LEGACY_METADATA_BYTES}",
                contents.len()
            ));
        }

        if contents.len() >= METADATA_BYTES {
            if let Ok(metadata) = Self::decode(&contents[..METADATA_BYTES]) {
                return Ok(metadata);
            }
        }

        Self::decode(&contents[..LEGACY_METADATA_BYTES])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TqElementTuple {
    pub level: u8,
    pub deleted: bool,
    pub heaptids: Vec<ItemPointer>,
    pub gamma: f32,
    pub neighbortid: ItemPointer,
    pub code: Vec<u8>,
    pub binary_words: Vec<u64>,
}

#[derive(Debug, Clone, Copy)]
pub struct TqElementTupleRef<'a> {
    pub level: u8,
    pub deleted: bool,
    heaptid_bytes: &'a [u8],
    heaptid_count: usize,
    pub gamma: f32,
    pub neighbortid: ItemPointer,
    pub code: &'a [u8],
    binary_word_bytes: &'a [u8],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TqGroupedHotTuple {
    pub level: u8,
    pub deleted: bool,
    pub heaptids: Vec<ItemPointer>,
    pub neighbortid: ItemPointer,
    pub reranktid: ItemPointer,
    pub binary_words: Vec<u64>,
    pub search_code: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct TqGroupedHotTupleRef<'a> {
    pub level: u8,
    pub deleted: bool,
    heaptid_bytes: &'a [u8],
    heaptid_count: usize,
    pub neighbortid: ItemPointer,
    pub reranktid: ItemPointer,
    binary_word_bytes: &'a [u8],
    pub search_code: &'a [u8],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TqRerankTuple {
    pub gamma: f32,
    pub code: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct TqRerankTupleRef<'a> {
    pub gamma: f32,
    pub code: &'a [u8],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TqGroupedCodebookTuple {
    pub group_index: u16,
    pub nexttid: ItemPointer,
    pub centroids: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct TqGroupedCodebookTupleRef<'a> {
    pub group_index: u16,
    pub nexttid: ItemPointer,
    centroid_bytes: &'a [u8],
}

impl<'a> TqElementTupleRef<'a> {
    pub fn decode(input: &'a [u8], code_len: usize) -> Result<Self, String> {
        let min_expected_len = TqElementTuple::encoded_len(code_len);
        if input.len() < min_expected_len {
            return Err(format!(
                "element tuple length mismatch: got {}, expected at least {min_expected_len}",
                input.len(),
            ));
        }
        if input[0] != TQ_ELEMENT_TAG {
            return Err(format!("invalid element tuple tag: {}", input[0]));
        }

        let heaptid_bytes_start = 3;
        let heaptid_bytes_len = HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES;
        let heaptid_bytes = &input[heaptid_bytes_start..heaptid_bytes_start + heaptid_bytes_len];
        let mut cursor = heaptid_bytes_start + heaptid_bytes_len;

        let heaptid_count = input[cursor] as usize;
        cursor += 1;
        if heaptid_count > HEAPTID_INLINE_CAPACITY {
            return Err(format!(
                "invalid heap tid count: got {heaptid_count}, max {}",
                HEAPTID_INLINE_CAPACITY
            ));
        }

        let gamma = f32::from_le_bytes(input[cursor..cursor + 4].try_into().expect("gamma bytes"));
        cursor += 4;
        let neighbortid = ItemPointer::decode(&input[cursor..cursor + ITEM_POINTER_BYTES])?;
        cursor += ITEM_POINTER_BYTES;
        let code_end = cursor + code_len;
        let code = &input[cursor..code_end];
        let binary_word_bytes = &input[code_end..];
        if binary_word_bytes.len() % size_of::<u64>() != 0 {
            return Err(format!(
                "element tuple binary sidecar length {} is not aligned to {}",
                binary_word_bytes.len(),
                size_of::<u64>(),
            ));
        }

        Ok(Self {
            level: input[1],
            deleted: input[2] != 0,
            heaptid_bytes,
            heaptid_count,
            gamma,
            neighbortid,
            code,
            binary_word_bytes,
        })
    }

    pub fn heaptid_count(&self) -> usize {
        self.heaptid_count
    }

    pub fn heaptids(&self) -> impl Iterator<Item = ItemPointer> + '_ {
        self.heaptid_bytes
            .chunks_exact(ITEM_POINTER_BYTES)
            .take(self.heaptid_count)
            .map(|chunk| {
                ItemPointer::decode(chunk)
                    .expect("borrowed element tuple view should only expose validated tid bytes")
            })
    }

    pub fn collect_heaptids(&self) -> Vec<ItemPointer> {
        self.heaptids().collect()
    }

    pub fn binary_word_count(&self) -> usize {
        self.binary_word_bytes.len() / size_of::<u64>()
    }

    pub fn binary_words(&self) -> impl Iterator<Item = u64> + '_ {
        self.binary_word_bytes
            .chunks_exact(size_of::<u64>())
            .map(|chunk| u64::from_le_bytes(chunk.try_into().expect("validated u64 sidecar chunk")))
    }

    pub fn collect_binary_words(&self) -> Vec<u64> {
        self.binary_words().collect()
    }
}

impl TqElementTuple {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        if self.heaptids.len() > HEAPTID_INLINE_CAPACITY {
            return Err(format!(
                "too many heap tids: got {}, max {}",
                self.heaptids.len(),
                HEAPTID_INLINE_CAPACITY
            ));
        }

        let mut out = Vec::with_capacity(
            1 + 1
                + 1
                + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES
                + 1
                + 4
                + ITEM_POINTER_BYTES
                + self.code.len()
                + self.binary_words.len() * size_of::<u64>(),
        );
        out.push(TQ_ELEMENT_TAG);
        out.push(self.level);
        out.push(u8::from(self.deleted));

        for tid in &self.heaptids {
            tid.encode_into(&mut out);
        }
        for _ in self.heaptids.len()..HEAPTID_INLINE_CAPACITY {
            ItemPointer::INVALID.encode_into(&mut out);
        }

        out.push(self.heaptids.len() as u8);
        out.extend_from_slice(&self.gamma.to_le_bytes());
        self.neighbortid.encode_into(&mut out);
        out.extend_from_slice(&self.code);
        for word in &self.binary_words {
            out.extend_from_slice(&word.to_le_bytes());
        }
        Ok(out)
    }

    pub fn decode(input: &[u8], code_len: usize) -> Result<Self, String> {
        let element = TqElementTupleRef::decode(input, code_len)?;
        Ok(Self {
            level: element.level,
            deleted: element.deleted,
            heaptids: element.collect_heaptids(),
            gamma: element.gamma,
            neighbortid: element.neighbortid,
            code: element.code.to_vec(),
            binary_words: element.collect_binary_words(),
        })
    }

    pub fn encoded_len(code_len: usize) -> usize {
        Self::encoded_len_with_binary(code_len, 0)
    }

    pub fn encoded_len_with_binary(code_len: usize, binary_word_count: usize) -> usize {
        1 + 1
            + 1
            + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES
            + 1
            + 4
            + ITEM_POINTER_BYTES
            + code_len
            + binary_word_count * size_of::<u64>()
    }
}

impl<'a> TqGroupedHotTupleRef<'a> {
    pub fn decode(
        input: &'a [u8],
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Result<Self, String> {
        let expected_len = TqGroupedHotTuple::encoded_len(binary_word_count, search_code_len);
        if input.len() != expected_len {
            return Err(format!(
                "grouped hot tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if input[0] != TQ_GROUPED_HOT_TAG {
            return Err(format!("invalid grouped hot tuple tag: {}", input[0]));
        }

        let heaptid_bytes_start = 3;
        let heaptid_bytes_len = HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES;
        let heaptid_bytes = &input[heaptid_bytes_start..heaptid_bytes_start + heaptid_bytes_len];
        let mut cursor = heaptid_bytes_start + heaptid_bytes_len;

        let heaptid_count = input[cursor] as usize;
        cursor += 1;
        if heaptid_count > HEAPTID_INLINE_CAPACITY {
            return Err(format!(
                "invalid heap tid count: got {heaptid_count}, max {}",
                HEAPTID_INLINE_CAPACITY
            ));
        }

        let neighbortid = ItemPointer::decode(&input[cursor..cursor + ITEM_POINTER_BYTES])?;
        cursor += ITEM_POINTER_BYTES;
        let reranktid = ItemPointer::decode(&input[cursor..cursor + ITEM_POINTER_BYTES])?;
        cursor += ITEM_POINTER_BYTES;
        let binary_word_bytes_len = binary_word_count * size_of::<u64>();
        let binary_word_bytes = &input[cursor..cursor + binary_word_bytes_len];
        cursor += binary_word_bytes_len;
        let search_code = &input[cursor..cursor + search_code_len];

        Ok(Self {
            level: input[1],
            deleted: input[2] != 0,
            heaptid_bytes,
            heaptid_count,
            neighbortid,
            reranktid,
            binary_word_bytes,
            search_code,
        })
    }

    pub fn heaptid_count(&self) -> usize {
        self.heaptid_count
    }

    pub fn heaptids(&self) -> impl Iterator<Item = ItemPointer> + '_ {
        self.heaptid_bytes
            .chunks_exact(ITEM_POINTER_BYTES)
            .take(self.heaptid_count)
            .map(|chunk| {
                ItemPointer::decode(chunk)
                    .expect("borrowed grouped hot tuple should only expose validated tid bytes")
            })
    }

    pub fn collect_heaptids(&self) -> Vec<ItemPointer> {
        self.heaptids().collect()
    }

    pub fn binary_words(&self) -> impl Iterator<Item = u64> + '_ {
        self.binary_word_bytes
            .chunks_exact(size_of::<u64>())
            .map(|chunk| u64::from_le_bytes(chunk.try_into().expect("validated u64 sidecar chunk")))
    }

    pub fn collect_binary_words(&self) -> Vec<u64> {
        self.binary_words().collect()
    }
}

impl TqGroupedHotTuple {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        if self.heaptids.len() > HEAPTID_INLINE_CAPACITY {
            return Err(format!(
                "too many heap tids: got {}, max {}",
                self.heaptids.len(),
                HEAPTID_INLINE_CAPACITY
            ));
        }

        let mut out = Vec::with_capacity(Self::encoded_len(
            self.binary_words.len(),
            self.search_code.len(),
        ));
        out.push(TQ_GROUPED_HOT_TAG);
        out.push(self.level);
        out.push(u8::from(self.deleted));

        for tid in &self.heaptids {
            tid.encode_into(&mut out);
        }
        for _ in self.heaptids.len()..HEAPTID_INLINE_CAPACITY {
            ItemPointer::INVALID.encode_into(&mut out);
        }

        out.push(self.heaptids.len() as u8);
        self.neighbortid.encode_into(&mut out);
        self.reranktid.encode_into(&mut out);
        for word in &self.binary_words {
            out.extend_from_slice(&word.to_le_bytes());
        }
        out.extend_from_slice(&self.search_code);
        Ok(out)
    }

    pub fn decode(
        input: &[u8],
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Result<Self, String> {
        let hot = TqGroupedHotTupleRef::decode(input, binary_word_count, search_code_len)?;
        Ok(Self {
            level: hot.level,
            deleted: hot.deleted,
            heaptids: hot.collect_heaptids(),
            neighbortid: hot.neighbortid,
            reranktid: hot.reranktid,
            binary_words: hot.collect_binary_words(),
            search_code: hot.search_code.to_vec(),
        })
    }

    pub fn encoded_len(binary_word_count: usize, search_code_len: usize) -> usize {
        1 + 1
            + 1
            + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES
            + 1
            + ITEM_POINTER_BYTES
            + ITEM_POINTER_BYTES
            + binary_word_count * size_of::<u64>()
            + search_code_len
    }
}

impl<'a> TqRerankTupleRef<'a> {
    pub fn decode(input: &'a [u8], code_len: usize) -> Result<Self, String> {
        let expected_len = TqRerankTuple::encoded_len(code_len);
        if input.len() != expected_len {
            return Err(format!(
                "rerank tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if input[0] != TQ_RERANK_TAG {
            return Err(format!("invalid rerank tuple tag: {}", input[0]));
        }

        Ok(Self {
            gamma: f32::from_le_bytes(input[1..5].try_into().expect("gamma bytes")),
            code: &input[5..5 + code_len],
        })
    }
}

impl TqRerankTuple {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::encoded_len(self.code.len()));
        out.push(TQ_RERANK_TAG);
        out.extend_from_slice(&self.gamma.to_le_bytes());
        out.extend_from_slice(&self.code);
        out
    }

    pub fn decode(input: &[u8], code_len: usize) -> Result<Self, String> {
        let rerank = TqRerankTupleRef::decode(input, code_len)?;
        Ok(Self {
            gamma: rerank.gamma,
            code: rerank.code.to_vec(),
        })
    }

    pub fn encoded_len(code_len: usize) -> usize {
        1 + 4 + code_len
    }
}

impl<'a> TqGroupedCodebookTupleRef<'a> {
    pub fn decode(input: &'a [u8], centroid_count: usize) -> Result<Self, String> {
        let expected_len = TqGroupedCodebookTuple::encoded_len(centroid_count);
        if input.len() != expected_len {
            return Err(format!(
                "grouped codebook tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if input[0] != TQ_GROUPED_CODEBOOK_TAG {
            return Err(format!("invalid grouped codebook tuple tag: {}", input[0]));
        }

        Ok(Self {
            group_index: u16::from_le_bytes(input[1..3].try_into().expect("group index bytes")),
            nexttid: ItemPointer::decode(&input[3..3 + ITEM_POINTER_BYTES])?,
            centroid_bytes: &input[3 + ITEM_POINTER_BYTES..],
        })
    }

    pub fn centroids(&self) -> impl Iterator<Item = f32> + '_ {
        self.centroid_bytes
            .chunks_exact(size_of::<f32>())
            .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("validated f32 chunk")))
    }

    pub fn collect_centroids(&self) -> Vec<f32> {
        self.centroids().collect()
    }
}

impl TqGroupedCodebookTuple {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::encoded_len(self.centroids.len()));
        out.push(TQ_GROUPED_CODEBOOK_TAG);
        out.extend_from_slice(&self.group_index.to_le_bytes());
        self.nexttid.encode_into(&mut out);
        for centroid in &self.centroids {
            out.extend_from_slice(&centroid.to_le_bytes());
        }
        out
    }

    pub fn decode(input: &[u8], centroid_count: usize) -> Result<Self, String> {
        let codebook = TqGroupedCodebookTupleRef::decode(input, centroid_count)?;
        Ok(Self {
            group_index: codebook.group_index,
            nexttid: codebook.nexttid,
            centroids: codebook.collect_centroids(),
        })
    }

    pub fn encoded_len(centroid_count: usize) -> usize {
        1 + 2 + ITEM_POINTER_BYTES + centroid_count * size_of::<f32>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TqNeighborTuple {
    pub count: u16,
    pub tids: Vec<ItemPointer>,
}

impl TqNeighborTuple {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        if self.tids.len() > self.count as usize {
            return Err(format!(
                "neighbor count {} smaller than tids length {}",
                self.count,
                self.tids.len()
            ));
        }

        let mut out = Vec::with_capacity(1 + 2 + self.tids.len() * ITEM_POINTER_BYTES);
        out.push(TQ_NEIGHBOR_TAG);
        out.extend_from_slice(&self.count.to_le_bytes());
        for tid in &self.tids {
            tid.encode_into(&mut out);
        }
        Ok(out)
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() < 3 {
            return Err("neighbor tuple too short".into());
        }
        if input[0] != TQ_NEIGHBOR_TAG {
            return Err(format!("invalid neighbor tuple tag: {}", input[0]));
        }
        let count = u16::from_le_bytes(input[1..3].try_into().expect("neighbor count bytes"));
        let tid_bytes = &input[3..];
        if tid_bytes.len() % ITEM_POINTER_BYTES != 0 {
            return Err("neighbor tuple tid payload is misaligned".into());
        }

        let tids = tid_bytes
            .chunks_exact(ITEM_POINTER_BYTES)
            .map(ItemPointer::decode)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { count, tids })
    }
}

pub fn neighbor_slots(level: u8, m: u16) -> usize {
    (2 * m as usize) + (level as usize * m as usize)
}

pub fn neighbor_tuple_encoded_len(level: u8, m: u16) -> usize {
    1 + 2 + neighbor_slots(level, m) * ITEM_POINTER_BYTES
}

pub fn max_level_that_fits(m: u16, page_size: usize) -> u8 {
    let usable = usable_page_bytes(page_size);
    let mut level = 0_u8;
    loop {
        let tuple_bytes = aligned_tuple_bytes(neighbor_tuple_encoded_len(level, m));
        if tuple_bytes > usable {
            return level.saturating_sub(1);
        }
        if level == u8::MAX {
            return level;
        }
        level = level.saturating_add(1);
    }
}

pub fn element_tuple_fits_on_page(code_len: usize, page_size: usize) -> bool {
    aligned_tuple_bytes(TqElementTuple::encoded_len(code_len)) <= usable_page_bytes(page_size)
}

pub fn neighbor_tuple_fits_on_page(level: u8, m: u16, page_size: usize) -> bool {
    aligned_tuple_bytes(neighbor_tuple_encoded_len(level, m)) <= usable_page_bytes(page_size)
}

pub fn default_max_level_cap(m: u16) -> u8 {
    max_level_that_fits(m, DEFAULT_PAGE_SIZE)
}

#[derive(Debug, Clone)]
pub struct DataPage {
    block_number: u32,
    page_size: usize,
    used_bytes: usize,
    tuples: Vec<Vec<u8>>,
}

impl DataPage {
    pub fn new(block_number: u32, page_size: usize) -> Self {
        Self {
            block_number,
            page_size,
            used_bytes: PAGE_HEADER_BYTES,
            tuples: Vec::new(),
        }
    }

    pub fn block_number(&self) -> u32 {
        self.block_number
    }

    pub fn tuple_count(&self) -> usize {
        self.tuples.len()
    }

    pub fn tuples(&self) -> &[Vec<u8>] {
        &self.tuples
    }

    pub fn free_bytes(&self) -> usize {
        self.page_size.saturating_sub(self.used_bytes)
    }

    pub fn can_fit_raw_tuple(&self, payload_len: usize) -> bool {
        aligned_tuple_bytes(payload_len) <= self.free_bytes()
    }

    pub fn insert_raw_tuple(&mut self, payload: Vec<u8>) -> Result<ItemPointer, String> {
        if !self.can_fit_raw_tuple(payload.len()) {
            return Err(format!(
                "tuple payload {} does not fit on block {} with {} bytes free",
                payload.len(),
                self.block_number,
                self.free_bytes()
            ));
        }

        self.used_bytes += aligned_tuple_bytes(payload.len());
        self.tuples.push(payload);
        Ok(ItemPointer {
            block_number: self.block_number,
            offset_number: u16::try_from(self.tuples.len()).expect("tuple count should fit in u16"),
        })
    }

    pub fn raw_tuple(&self, tid: ItemPointer) -> Result<&[u8], String> {
        if tid.block_number != self.block_number {
            return Err(format!(
                "tuple block mismatch: got {}, page is {}",
                tid.block_number, self.block_number
            ));
        }
        if tid.offset_number == 0 {
            return Err("offset number must be 1-based".into());
        }

        let index = (tid.offset_number - 1) as usize;
        self.tuples
            .get(index)
            .map(Vec::as_slice)
            .ok_or_else(|| format!("tuple offset {} out of range", tid.offset_number))
    }

    pub fn update_raw_tuple(&mut self, tid: ItemPointer, payload: Vec<u8>) -> Result<(), String> {
        if tid.block_number != self.block_number {
            return Err(format!(
                "tuple block mismatch: got {}, page is {}",
                tid.block_number, self.block_number
            ));
        }
        if tid.offset_number == 0 {
            return Err("offset number must be 1-based".into());
        }

        let index = (tid.offset_number - 1) as usize;
        let existing = self
            .tuples
            .get_mut(index)
            .ok_or_else(|| format!("tuple offset {} out of range", tid.offset_number))?;
        if payload.len() != existing.len() {
            return Err(format!(
                "tuple length mismatch: got {}, expected {}",
                payload.len(),
                existing.len()
            ));
        }
        *existing = payload;
        Ok(())
    }

    pub fn insert_element(&mut self, tuple: &TqElementTuple) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn read_element(
        &self,
        tid: ItemPointer,
        code_len: usize,
    ) -> Result<TqElementTuple, String> {
        TqElementTuple::decode(self.raw_tuple(tid)?, code_len)
    }

    pub fn insert_neighbor(&mut self, tuple: &TqNeighborTuple) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn read_neighbor(&self, tid: ItemPointer) -> Result<TqNeighborTuple, String> {
        TqNeighborTuple::decode(self.raw_tuple(tid)?)
    }

    pub fn insert_grouped_hot(&mut self, tuple: &TqGroupedHotTuple) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn read_grouped_hot(
        &self,
        tid: ItemPointer,
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Result<TqGroupedHotTuple, String> {
        TqGroupedHotTuple::decode(self.raw_tuple(tid)?, binary_word_count, search_code_len)
    }

    pub fn insert_rerank(&mut self, tuple: &TqRerankTuple) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode())
    }

    pub fn read_rerank(&self, tid: ItemPointer, code_len: usize) -> Result<TqRerankTuple, String> {
        TqRerankTuple::decode(self.raw_tuple(tid)?, code_len)
    }

    pub fn insert_grouped_codebook(
        &mut self,
        tuple: &TqGroupedCodebookTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode())
    }

    pub fn read_grouped_codebook(
        &self,
        tid: ItemPointer,
        centroid_count: usize,
    ) -> Result<TqGroupedCodebookTuple, String> {
        TqGroupedCodebookTuple::decode(self.raw_tuple(tid)?, centroid_count)
    }

    pub fn update_element(
        &mut self,
        tid: ItemPointer,
        tuple: &TqElementTuple,
    ) -> Result<(), String> {
        self.update_raw_tuple(tid, tuple.encode()?)
    }

    pub fn update_neighbor(
        &mut self,
        tid: ItemPointer,
        tuple: &TqNeighborTuple,
    ) -> Result<(), String> {
        self.update_raw_tuple(tid, tuple.encode()?)
    }

    pub fn update_grouped_hot(
        &mut self,
        tid: ItemPointer,
        tuple: &TqGroupedHotTuple,
    ) -> Result<(), String> {
        self.update_raw_tuple(tid, tuple.encode()?)
    }

    pub fn update_rerank(&mut self, tid: ItemPointer, tuple: &TqRerankTuple) -> Result<(), String> {
        self.update_raw_tuple(tid, tuple.encode())
    }

    pub fn update_grouped_codebook(
        &mut self,
        tid: ItemPointer,
        tuple: &TqGroupedCodebookTuple,
    ) -> Result<(), String> {
        self.update_raw_tuple(tid, tuple.encode())
    }
}

#[derive(Debug, Clone)]
pub struct DataPageChain {
    page_size: usize,
    pages: Vec<DataPage>,
}

impl DataPageChain {
    pub fn new(page_size: usize) -> Self {
        Self {
            page_size,
            pages: vec![DataPage::new(FIRST_DATA_BLOCK_NUMBER, page_size)],
        }
    }

    pub fn pages(&self) -> &[DataPage] {
        &self.pages
    }

    pub fn get_page(&self, block_number: u32) -> Option<&DataPage> {
        let index = block_number.checked_sub(FIRST_DATA_BLOCK_NUMBER)? as usize;
        self.pages.get(index)
    }

    pub fn get_page_mut(&mut self, block_number: u32) -> Option<&mut DataPage> {
        let index = block_number.checked_sub(FIRST_DATA_BLOCK_NUMBER)? as usize;
        self.pages.get_mut(index)
    }

    pub fn insert_raw_tuple(&mut self, payload: Vec<u8>) -> Result<ItemPointer, String> {
        if !element_or_neighbor_tuple_fits(payload.len(), self.page_size) {
            return Err(format!(
                "tuple payload {} exceeds maximum page capacity {}",
                payload.len(),
                usable_page_bytes(self.page_size)
            ));
        }

        if self
            .pages
            .last()
            .is_some_and(|page| page.can_fit_raw_tuple(payload.len()))
        {
            let last = self.pages.last_mut().expect("page chain is non-empty");
            return last.insert_raw_tuple(payload);
        }

        let next_block = self
            .pages
            .last()
            .expect("page chain is non-empty")
            .block_number
            + 1;
        self.pages.push(DataPage::new(next_block, self.page_size));
        debug_assert!(
            self.pages
                .iter()
                .enumerate()
                .all(|(i, page)| { page.block_number == FIRST_DATA_BLOCK_NUMBER + i as u32 }),
            "DataPageChain pages are not contiguous"
        );
        self.pages
            .last_mut()
            .expect("new page was pushed")
            .insert_raw_tuple(payload)
    }

    pub fn insert_element(&mut self, tuple: &TqElementTuple) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn insert_neighbor(&mut self, tuple: &TqNeighborTuple) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn insert_grouped_hot(&mut self, tuple: &TqGroupedHotTuple) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn insert_rerank(&mut self, tuple: &TqRerankTuple) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode())
    }

    pub fn insert_grouped_codebook(
        &mut self,
        tuple: &TqGroupedCodebookTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode())
    }

    pub fn read_element(
        &self,
        tid: ItemPointer,
        code_len: usize,
    ) -> Result<TqElementTuple, String> {
        self.get_page(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .read_element(tid, code_len)
    }

    pub fn read_neighbor(&self, tid: ItemPointer) -> Result<TqNeighborTuple, String> {
        self.get_page(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .read_neighbor(tid)
    }

    pub fn read_grouped_hot(
        &self,
        tid: ItemPointer,
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Result<TqGroupedHotTuple, String> {
        self.get_page(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .read_grouped_hot(tid, binary_word_count, search_code_len)
    }

    pub fn read_rerank(&self, tid: ItemPointer, code_len: usize) -> Result<TqRerankTuple, String> {
        self.get_page(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .read_rerank(tid, code_len)
    }

    pub fn read_grouped_codebook(
        &self,
        tid: ItemPointer,
        centroid_count: usize,
    ) -> Result<TqGroupedCodebookTuple, String> {
        self.get_page(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .read_grouped_codebook(tid, centroid_count)
    }

    pub fn update_element(
        &mut self,
        tid: ItemPointer,
        tuple: &TqElementTuple,
    ) -> Result<(), String> {
        self.get_page_mut(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .update_element(tid, tuple)
    }

    pub fn update_neighbor(
        &mut self,
        tid: ItemPointer,
        tuple: &TqNeighborTuple,
    ) -> Result<(), String> {
        self.get_page_mut(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .update_neighbor(tid, tuple)
    }

    pub fn update_grouped_hot(
        &mut self,
        tid: ItemPointer,
        tuple: &TqGroupedHotTuple,
    ) -> Result<(), String> {
        self.get_page_mut(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .update_grouped_hot(tid, tuple)
    }

    pub fn update_rerank(&mut self, tid: ItemPointer, tuple: &TqRerankTuple) -> Result<(), String> {
        self.get_page_mut(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .update_rerank(tid, tuple)
    }

    pub fn update_grouped_codebook(
        &mut self,
        tid: ItemPointer,
        tuple: &TqGroupedCodebookTuple,
    ) -> Result<(), String> {
        self.get_page_mut(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .update_grouped_codebook(tid, tuple)
    }
}

fn usable_page_bytes(page_size: usize) -> usize {
    page_size.saturating_sub(PAGE_HEADER_BYTES)
}

fn element_or_neighbor_tuple_fits(payload_len: usize, page_size: usize) -> bool {
    aligned_tuple_bytes(payload_len) <= usable_page_bytes(page_size)
}

fn aligned_tuple_bytes(payload_len: usize) -> usize {
    let tuple_len = TUPLE_HEADER_BYTES + payload_len + LINE_POINTER_BYTES;
    let remainder = tuple_len % ALIGNMENT_BYTES;
    if remainder == 0 {
        tuple_len
    } else {
        tuple_len + (ALIGNMENT_BYTES - remainder)
    }
}

pub fn raw_tuple_storage_bytes(payload_len: usize) -> usize {
    aligned_tuple_bytes(payload_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    #[test]
    fn metadata_roundtrip() {
        let metadata = MetadataPage::current_v1_scalar(CurrentFormatMetadata {
            m: 8,
            ef_construction: 64,
            entry_point: tid(12, 3),
            dimensions: 1536,
            bits: 4,
            max_level: 5,
            seed: 42,
            inserted_since_rebuild: 7,
            persisted_binary_sidecar: true,
        });
        let encoded = metadata.encode();
        let decoded = MetadataPage::decode(&encoded).unwrap();
        assert_eq!(decoded, metadata);
    }

    #[test]
    fn metadata_page_roundtrip() {
        let metadata = MetadataPage::current_v1_scalar(CurrentFormatMetadata {
            m: 8,
            ef_construction: 64,
            entry_point: tid(12, 3),
            dimensions: 1536,
            bits: 4,
            max_level: 5,
            seed: 42,
            inserted_since_rebuild: 7,
            persisted_binary_sidecar: true,
        });

        let page = metadata.encode_page(DEFAULT_PAGE_SIZE).unwrap();
        let decoded = MetadataPage::decode_page(&page).unwrap();
        assert_eq!(decoded, metadata);
    }

    #[test]
    fn metadata_decode_page_accepts_legacy_layout() {
        let legacy = {
            let mut out = Vec::with_capacity(LEGACY_METADATA_BYTES);
            out.extend_from_slice(&8_u16.to_le_bytes());
            out.extend_from_slice(&64_u16.to_le_bytes());
            tid(12, 3).encode_into(&mut out);
            out.extend_from_slice(&1536_u16.to_le_bytes());
            out.push(4);
            out.push(5);
            out.extend_from_slice(&42_u64.to_le_bytes());
            out.extend_from_slice(&7_u64.to_le_bytes());
            out
        };
        let mut page = vec![0_u8; DEFAULT_PAGE_SIZE];
        let legacy_offset = DEFAULT_PAGE_SIZE - LEGACY_METADATA_SPECIAL_BYTES;
        page[legacy_offset..legacy_offset + legacy.len()].copy_from_slice(&legacy);

        let decoded = MetadataPage::decode_page(&page).unwrap();
        assert_eq!(decoded.format_version, INDEX_FORMAT_V1_SCALAR);
        assert_eq!(decoded.transform_kind, TransformKind::Srht);
        assert_eq!(decoded.search_codec_kind, SearchCodecKind::ScalarQuantized);
        assert_eq!(decoded.payload_flags, 0);
        assert_eq!(decoded.grouped_codebook_head, ItemPointer::INVALID);
    }

    #[test]
    fn metadata_graph_storage_format_distinguishes_v1_and_v2() {
        let v1 = MetadataPage::current_v1_scalar(CurrentFormatMetadata {
            m: 8,
            ef_construction: 64,
            entry_point: tid(1, 1),
            dimensions: 16,
            bits: 4,
            max_level: 2,
            seed: 42,
            inserted_since_rebuild: 0,
            persisted_binary_sidecar: false,
        });
        assert_eq!(
            v1.graph_storage_format().unwrap(),
            GraphStorageFormat::ScalarV1
        );

        let v2 = MetadataPage {
            m: 8,
            ef_construction: 64,
            entry_point: tid(1, 1),
            dimensions: 16,
            bits: 4,
            max_level: 2,
            seed: 42,
            inserted_since_rebuild: 0,
            format_version: INDEX_FORMAT_V2_GROUPED,
            transform_kind: TransformKind::Srht,
            search_codec_kind: SearchCodecKind::GroupedPq,
            payload_flags: PAYLOAD_FLAG_GROUPED_SEARCH_CODE | PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            search_bits: 4,
            rerank_codec_kind: RerankCodecKind::ScalarQuantized,
            search_subvector_count: 1,
            search_subvector_dim: 16,
            grouped_codebook_head: tid(1, 2),
        };
        assert_eq!(
            v2.graph_storage_format().unwrap(),
            GraphStorageFormat::GroupedV2
        );
    }

    #[test]
    fn element_tuple_roundtrip() {
        let tuple = TqElementTuple {
            level: 3,
            deleted: false,
            heaptids: vec![tid(10, 1), tid(11, 2)],
            gamma: 1.25,
            neighbortid: tid(20, 4),
            code: vec![0xAA; 32],
            binary_words: Vec::new(),
        };

        let encoded = tuple.encode().unwrap();
        let decoded = TqElementTuple::decode(&encoded, 32).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn element_tuple_ref_exposes_borrowed_code_and_live_heaptids() {
        let tuple = TqElementTuple {
            level: 2,
            deleted: false,
            heaptids: vec![tid(10, 1), tid(11, 2)],
            gamma: 0.75,
            neighbortid: tid(20, 4),
            code: vec![0xAB; 24],
            binary_words: vec![0x0123_4567_89AB_CDEF, 0x0F0E_0D0C_0B0A_0908],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = TqElementTupleRef::decode(&encoded, 24).unwrap();

        assert_eq!(decoded.level, tuple.level);
        assert_eq!(decoded.deleted, tuple.deleted);
        assert_eq!(decoded.heaptid_count(), tuple.heaptids.len());
        assert_eq!(decoded.collect_heaptids(), tuple.heaptids);
        assert_eq!(decoded.gamma.to_bits(), tuple.gamma.to_bits());
        assert_eq!(decoded.neighbortid, tuple.neighbortid);
        assert_eq!(decoded.code, tuple.code.as_slice());
        assert_eq!(decoded.binary_word_count(), tuple.binary_words.len());
        assert_eq!(decoded.collect_binary_words(), tuple.binary_words);
    }

    #[test]
    fn element_tuple_page_roundtrip() {
        let tuple = TqElementTuple {
            level: 3,
            deleted: false,
            heaptids: vec![tid(10, 1), tid(11, 2)],
            gamma: -0.5,
            neighbortid: tid(20, 4),
            code: vec![0xAA; 32],
            binary_words: vec![0x1111_2222_3333_4444],
        };

        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);
        let tuple_tid = page.insert_element(&tuple).unwrap();
        let decoded = page.read_element(tuple_tid, 32).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn grouped_hot_tuple_roundtrip() {
        let tuple = TqGroupedHotTuple {
            level: 2,
            deleted: false,
            heaptids: vec![tid(10, 1), tid(11, 2)],
            neighbortid: tid(20, 4),
            reranktid: tid(21, 5),
            binary_words: vec![0x0123_4567_89AB_CDEF, 0x0F0E_0D0C_0B0A_0908],
            search_code: vec![0x12, 0x34, 0x56],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = TqGroupedHotTuple::decode(&encoded, 2, 3).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn grouped_hot_tuple_ref_exposes_borrowed_payloads() {
        let tuple = TqGroupedHotTuple {
            level: 1,
            deleted: true,
            heaptids: vec![tid(10, 1), tid(11, 2), tid(12, 3)],
            neighbortid: tid(20, 4),
            reranktid: tid(21, 5),
            binary_words: vec![0xAAAA_BBBB_CCCC_DDDD],
            search_code: vec![0x9A, 0xBC],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = TqGroupedHotTupleRef::decode(&encoded, 1, 2).unwrap();

        assert_eq!(decoded.level, tuple.level);
        assert_eq!(decoded.deleted, tuple.deleted);
        assert_eq!(decoded.heaptid_count(), tuple.heaptids.len());
        assert_eq!(decoded.collect_heaptids(), tuple.heaptids);
        assert_eq!(decoded.neighbortid, tuple.neighbortid);
        assert_eq!(decoded.reranktid, tuple.reranktid);
        assert_eq!(decoded.collect_binary_words(), tuple.binary_words);
        assert_eq!(decoded.search_code, tuple.search_code.as_slice());
    }

    #[test]
    fn rerank_tuple_roundtrip() {
        let tuple = TqRerankTuple {
            gamma: -0.75,
            code: vec![0xAA; 32],
        };

        let encoded = tuple.encode();
        let decoded = TqRerankTuple::decode(&encoded, 32).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn grouped_codebook_tuple_roundtrip() {
        let tuple = TqGroupedCodebookTuple {
            group_index: 7,
            nexttid: tid(20, 4),
            centroids: vec![0.25, -0.5, 1.25, 2.0],
        };

        let encoded = tuple.encode();
        let decoded = TqGroupedCodebookTuple::decode(&encoded, 4).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn grouped_hot_tuple_page_roundtrip() {
        let tuple = TqGroupedHotTuple {
            level: 2,
            deleted: false,
            heaptids: vec![tid(10, 1), tid(11, 2)],
            neighbortid: tid(20, 4),
            reranktid: tid(21, 5),
            binary_words: vec![0x0123_4567_89AB_CDEF, 0x0F0E_0D0C_0B0A_0908],
            search_code: vec![0x12, 0x34, 0x56],
        };

        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);
        let tuple_tid = page.insert_grouped_hot(&tuple).unwrap();
        let decoded = page.read_grouped_hot(tuple_tid, 2, 3).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn rerank_tuple_page_roundtrip() {
        let tuple = TqRerankTuple {
            gamma: -0.75,
            code: vec![0xAA; 32],
        };

        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);
        let tuple_tid = page.insert_rerank(&tuple).unwrap();
        let decoded = page.read_rerank(tuple_tid, 32).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn grouped_codebook_tuple_page_roundtrip() {
        let tuple = TqGroupedCodebookTuple {
            group_index: 3,
            nexttid: tid(20, 5),
            centroids: vec![0.1; 64],
        };

        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);
        let tuple_tid = page.insert_grouped_codebook(&tuple).unwrap();
        let decoded = page.read_grouped_codebook(tuple_tid, 64).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn neighbor_tuple_roundtrip() {
        let tuple = TqNeighborTuple {
            count: 4,
            tids: vec![tid(1, 1), tid(2, 2), tid(3, 3), tid(4, 4)],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = TqNeighborTuple::decode(&encoded).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn neighbor_tuple_page_roundtrip() {
        let tuple = TqNeighborTuple {
            count: 4,
            tids: vec![tid(1, 1), tid(2, 2), tid(3, 3), tid(4, 4)],
        };

        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);
        let tuple_tid = page.insert_neighbor(&tuple).unwrap();
        let decoded = page.read_neighbor(tuple_tid).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn level_cap_produces_fitting_neighbor_tuple() {
        let m = 16;
        let cap = default_max_level_cap(m);
        assert!(neighbor_tuple_fits_on_page(cap, m, DEFAULT_PAGE_SIZE));
        if cap < u8::MAX {
            assert!(!neighbor_tuple_fits_on_page(
                cap.saturating_add(1),
                m,
                DEFAULT_PAGE_SIZE
            ));
        }
    }

    #[test]
    fn compressed_element_tuple_fits_on_default_page() {
        assert!(element_tuple_fits_on_page(772, DEFAULT_PAGE_SIZE));
    }

    #[test]
    fn page_chain_extends_for_multiple_element_tuples() {
        let tuple = TqElementTuple {
            level: 0,
            deleted: false,
            heaptids: vec![tid(10, 1)],
            gamma: 0.75,
            neighbortid: tid(20, 4),
            code: vec![0xAA; 772],
            binary_words: Vec::new(),
        };

        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let mut last_tid = ItemPointer::INVALID;
        for _ in 0..20 {
            last_tid = chain.insert_element(&tuple).unwrap();
        }

        assert!(chain.pages().len() > 1);
        assert!(last_tid.block_number > FIRST_DATA_BLOCK_NUMBER);
        let decoded = chain.read_element(last_tid, 772).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn page_chain_extends_for_multiple_grouped_hot_tuples() {
        let tuple = TqGroupedHotTuple {
            level: 0,
            deleted: false,
            heaptids: vec![tid(10, 1)],
            neighbortid: tid(20, 4),
            reranktid: tid(21, 5),
            binary_words: vec![0x0123_4567_89AB_CDEF; 8],
            search_code: vec![0xAA; 700],
        };

        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let mut last_tid = ItemPointer::INVALID;
        for _ in 0..20 {
            last_tid = chain.insert_grouped_hot(&tuple).unwrap();
        }

        assert!(chain.pages().len() > 1);
        assert!(last_tid.block_number > FIRST_DATA_BLOCK_NUMBER);
        let decoded = chain.read_grouped_hot(last_tid, 8, 700).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn page_chain_extends_for_multiple_rerank_tuples() {
        let tuple = TqRerankTuple {
            gamma: 0.25,
            code: vec![0xBB; 900],
        };

        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let mut last_tid = ItemPointer::INVALID;
        for _ in 0..20 {
            last_tid = chain.insert_rerank(&tuple).unwrap();
        }

        assert!(chain.pages().len() > 1);
        assert!(last_tid.block_number > FIRST_DATA_BLOCK_NUMBER);
        let decoded = chain.read_rerank(last_tid, 900).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn page_chain_extends_for_multiple_grouped_codebook_tuples() {
        let tuple = TqGroupedCodebookTuple {
            group_index: 0,
            nexttid: ItemPointer::INVALID,
            centroids: vec![0.125; 512],
        };

        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let mut last_tid = ItemPointer::INVALID;
        for group_index in 0..20 {
            last_tid = chain
                .insert_grouped_codebook(&TqGroupedCodebookTuple {
                    group_index,
                    nexttid: ItemPointer::INVALID,
                    centroids: tuple.centroids.clone(),
                })
                .unwrap();
        }

        assert!(chain.pages().len() > 1);
        assert!(last_tid.block_number > FIRST_DATA_BLOCK_NUMBER);
        let decoded = chain.read_grouped_codebook(last_tid, 512).unwrap();
        assert_eq!(decoded.group_index, 19);
        assert_eq!(decoded.centroids, tuple.centroids);
    }

    // --- Miri tests ---

    #[test]
    fn miri_item_pointer_roundtrip() {
        let ptr = ItemPointer {
            block_number: 42,
            offset_number: 7,
        };
        let mut buf = Vec::new();
        ptr.encode_into(&mut buf);
        let decoded = ItemPointer::decode(&buf).unwrap();
        assert_eq!(decoded, ptr);
    }

    #[test]
    fn miri_element_tuple_roundtrip() {
        let tuple = TqElementTuple {
            level: 1,
            deleted: false,
            heaptids: vec![tid(1, 1)],
            gamma: 0.5,
            neighbortid: tid(2, 1),
            code: vec![0xAB; 16],
            binary_words: vec![0xDEAD_BEEF_CAFE_BABE],
        };
        let encoded = tuple.encode().unwrap();
        let decoded = TqElementTuple::decode(&encoded, 16).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn miri_grouped_hot_tuple_roundtrip() {
        let tuple = TqGroupedHotTuple {
            level: 1,
            deleted: false,
            heaptids: vec![tid(1, 1)],
            neighbortid: tid(2, 1),
            reranktid: tid(3, 1),
            binary_words: vec![0xDEAD_BEEF_CAFE_BABE],
            search_code: vec![0xAB, 0xCD],
        };
        let encoded = tuple.encode().unwrap();
        let decoded = TqGroupedHotTuple::decode(&encoded, 1, 2).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn miri_rerank_tuple_roundtrip() {
        let tuple = TqRerankTuple {
            gamma: 0.5,
            code: vec![0xAB; 16],
        };
        let encoded = tuple.encode();
        let decoded = TqRerankTuple::decode(&encoded, 16).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn miri_neighbor_tuple_roundtrip() {
        let tuple = TqNeighborTuple {
            count: 2,
            tids: vec![tid(1, 1), tid(2, 2)],
        };
        let encoded = tuple.encode().unwrap();
        let decoded = TqNeighborTuple::decode(&encoded).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn miri_metadata_roundtrip() {
        let meta = MetadataPage::current_v1_scalar(CurrentFormatMetadata {
            m: 8,
            ef_construction: 64,
            entry_point: tid(1, 1),
            dimensions: 32,
            bits: 4,
            max_level: 3,
            seed: 42,
            inserted_since_rebuild: 5,
            persisted_binary_sidecar: false,
        });
        let encoded = meta.encode();
        let decoded = MetadataPage::decode(&encoded).unwrap();
        assert_eq!(decoded, meta);
    }
}
