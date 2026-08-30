//! Task 231 fixed-stride graph/vector node format.
//!
//! The pure byte contract lives here separately from PostgreSQL relation I/O so
//! page arithmetic, corruption rejection, and canonical encoding can be tested
//! without a backend. Relation creation and buffer/WAL integration consume
//! these checked shapes in the physical-generation path.

use crate::storage::page::{
    ItemPointer, ALIGNMENT_BYTES, DEFAULT_PAGE_SIZE, ITEM_POINTER_BYTES, PAGE_HEADER_BYTES,
};

use super::canonical_wire::{domain_digest, CanonicalDecoder, CanonicalEncoder};

pub(crate) const FIXED_STRIDE_LAYOUT_VERSION: u16 = 1;
pub(crate) const FIXED_STRIDE_PAGE_FORMAT_VERSION: u16 = 1;
pub(crate) const FIXED_STRIDE_NODE_FORMAT_VERSION: u16 = 1;
pub(crate) const FIXED_STRIDE_PAGE_HEADER_BYTES: usize = 80;
pub(crate) const FIXED_STRIDE_NODE_HEADER_BYTES: usize = 80;
pub(crate) const FIXED_STRIDE_METADATA_BYTES: usize = 160;
pub(crate) const FIXED_STRIDE_LAYOUT_BYTES: usize = 42;

const PAGE_MAGIC: [u8; 4] = *b"EFS1";
const NODE_MAGIC: [u8; 4] = *b"EFN1";
const METADATA_MAGIC: [u8; 4] = *b"EFM1";
const PAGE_DIGEST_OFFSET: usize = 48;
const PAGE_DIGEST_BYTES: usize = 32;
const NODE_DIGEST_OFFSET: usize = 48;
const NODE_DIGEST_BYTES: usize = 32;
const NODE_FLAG_TOMBSTONE: u16 = 1;
const PAGE_DOMAIN: &[u8] = b"ec_distann_fixed_stride_page_v1\0";
const NODE_DOMAIN: &[u8] = b"ec_distann_fixed_stride_node_v1\0";
const GENERATION_TAG_DOMAIN: &[u8] = b"ec_distann_fixed_stride_generation_tag_v1\0";
const LAYOUT_DOMAIN: &[u8] = b"ec_distann_fixed_stride_layout_v1\0";
const METADATA_DOMAIN: &[u8] = b"ec_distann_fixed_stride_metadata_v1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistannFixedStrideLayoutDescriptorV1 {
    pub version: u16,
    pub block_size: u32,
    pub pg_page_header_bytes: u16,
    pub page_header_bytes: u16,
    pub node_header_bytes: u16,
    pub dimensions: u16,
    pub graph_degree: u16,
    pub code_len: u32,
    pub node_body_bytes: u32,
    pub node_record_bytes: u32,
    pub node_stride_bytes: u32,
    pub page_payload_bytes: u32,
    pub nodes_per_page: u16,
    pub extent_blocks: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FixedStrideAddress {
    pub(crate) first_block: u32,
    pub(crate) byte_offset: u16,
    pub(crate) slot_index: u16,
    pub(crate) extent_blocks: u32,
}

impl DistannFixedStrideLayoutDescriptorV1 {
    pub fn new(dimensions: u16, graph_degree: u16, code_len: usize) -> Result<Self, String> {
        Self::with_page_shape(
            dimensions,
            graph_degree,
            code_len,
            DEFAULT_PAGE_SIZE,
            PAGE_HEADER_BYTES,
        )
    }

    fn with_page_shape(
        dimensions: u16,
        graph_degree: u16,
        code_len: usize,
        block_size: usize,
        pg_page_header_bytes: usize,
    ) -> Result<Self, String> {
        if dimensions == 0 || graph_degree == 0 || code_len == 0 {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: dimensions, graph degree, and code length must be non-zero"
                    .to_owned(),
            );
        }
        if block_size <= pg_page_header_bytes + FIXED_STRIDE_PAGE_HEADER_BYTES {
            return Err("EC_FIXED_STRIDE_FORMAT: page has no node payload capacity".to_owned());
        }

        let dimensions = u64::from(dimensions);
        let graph_degree = u64::from(graph_degree);
        let code_len_u64 = u64::try_from(code_len)
            .map_err(|_| "EC_FIXED_STRIDE_FORMAT: code length exceeds u64".to_owned())?;
        let exact_vector_bytes = dimensions
            .checked_mul(4)
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: exact-vector byte count overflow".to_owned())?;
        let neighbor_id_bytes = graph_degree
            .checked_mul(8)
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: neighbor-id byte count overflow".to_owned())?;
        let neighbor_code_bytes = graph_degree.checked_mul(code_len_u64).ok_or_else(|| {
            "EC_FIXED_STRIDE_FORMAT: neighbor-code byte count overflow".to_owned()
        })?;
        let node_body_bytes = exact_vector_bytes
            .checked_add(code_len_u64)
            .and_then(|value| value.checked_add(neighbor_id_bytes))
            .and_then(|value| value.checked_add(neighbor_code_bytes))
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: node body byte count overflow".to_owned())?;
        let node_record_bytes = u64::try_from(FIXED_STRIDE_NODE_HEADER_BYTES)
            .expect("fixed header fits u64")
            .checked_add(node_body_bytes)
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: node record byte count overflow".to_owned())?;
        let alignment = u64::try_from(ALIGNMENT_BYTES).expect("alignment fits u64");
        let node_stride_bytes = node_record_bytes
            .checked_add(alignment - 1)
            .map(|value| value / alignment * alignment)
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: node stride byte count overflow".to_owned())?;
        let page_payload_bytes = block_size - pg_page_header_bytes - FIXED_STRIDE_PAGE_HEADER_BYTES;
        let page_payload_u64 = u64::try_from(page_payload_bytes)
            .map_err(|_| "EC_FIXED_STRIDE_FORMAT: page payload exceeds u64".to_owned())?;

        let (nodes_per_page, extent_blocks) = if node_stride_bytes <= page_payload_u64 {
            let nodes = page_payload_u64 / node_stride_bytes;
            let nodes = u16::try_from(nodes)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: packed node count exceeds u16".to_owned())?;
            if nodes == 0 {
                return Err("EC_FIXED_STRIDE_FORMAT: packed node count is zero".to_owned());
            }
            (nodes, 1)
        } else {
            let blocks = node_stride_bytes
                .checked_add(page_payload_u64 - 1)
                .map(|value| value / page_payload_u64)
                .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: extent block count overflow".to_owned())?;
            let blocks = u32::try_from(blocks)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: extent block count exceeds u32".to_owned())?;
            (0, blocks)
        };

        Ok(Self {
            version: FIXED_STRIDE_LAYOUT_VERSION,
            block_size: u32::try_from(block_size)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: block size exceeds u32".to_owned())?,
            pg_page_header_bytes: u16::try_from(pg_page_header_bytes).map_err(|_| {
                "EC_FIXED_STRIDE_FORMAT: PostgreSQL page header exceeds u16".to_owned()
            })?,
            page_header_bytes: FIXED_STRIDE_PAGE_HEADER_BYTES as u16,
            node_header_bytes: FIXED_STRIDE_NODE_HEADER_BYTES as u16,
            dimensions: u16::try_from(dimensions).expect("input dimensions are u16"),
            graph_degree: u16::try_from(graph_degree).expect("input graph degree is u16"),
            code_len: u32::try_from(code_len_u64)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: code length exceeds u32".to_owned())?,
            node_body_bytes: u32::try_from(node_body_bytes)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: node body exceeds u32".to_owned())?,
            node_record_bytes: u32::try_from(node_record_bytes)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: node record exceeds u32".to_owned())?,
            node_stride_bytes: u32::try_from(node_stride_bytes)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: node stride exceeds u32".to_owned())?,
            page_payload_bytes: u32::try_from(page_payload_bytes)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: page payload exceeds u32".to_owned())?,
            nodes_per_page,
            extent_blocks,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        let derived = Self::with_page_shape(
            self.dimensions,
            self.graph_degree,
            usize::try_from(self.code_len)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: code length exceeds usize".to_owned())?,
            usize::try_from(self.block_size)
                .map_err(|_| "EC_FIXED_STRIDE_FORMAT: block size exceeds usize".to_owned())?,
            usize::from(self.pg_page_header_bytes),
        )?;
        if &derived != self {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: persisted layout does not match derived arithmetic"
                    .to_owned(),
            );
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut encoder = CanonicalEncoder::with_capacity(48);
        encoder.put_u16(self.version);
        encoder.put_u32(self.block_size);
        encoder.put_u16(self.pg_page_header_bytes);
        encoder.put_u16(self.page_header_bytes);
        encoder.put_u16(self.node_header_bytes);
        encoder.put_u16(self.dimensions);
        encoder.put_u16(self.graph_degree);
        encoder.put_u32(self.code_len);
        encoder.put_u32(self.node_body_bytes);
        encoder.put_u32(self.node_record_bytes);
        encoder.put_u32(self.node_stride_bytes);
        encoder.put_u32(self.page_payload_bytes);
        encoder.put_u16(self.nodes_per_page);
        encoder.put_u32(self.extent_blocks);
        let encoded = encoder.finish()?;
        if encoded.len() != FIXED_STRIDE_LAYOUT_BYTES {
            return Err("EC_FIXED_STRIDE_FORMAT: layout encoded length drift".to_owned());
        }
        Ok(encoded)
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let mut decoder = CanonicalDecoder::new(input, "fixed-stride layout")?;
        let version = decoder.get_u16("fixed-stride layout version")?;
        if version != FIXED_STRIDE_LAYOUT_VERSION {
            return Err(format!(
                "EC_FIXED_STRIDE_FORMAT: unsupported layout version {version}"
            ));
        }
        let layout = Self {
            version,
            block_size: decoder.get_u32("block size")?,
            pg_page_header_bytes: decoder.get_u16("PostgreSQL page header bytes")?,
            page_header_bytes: decoder.get_u16("node page header bytes")?,
            node_header_bytes: decoder.get_u16("node header bytes")?,
            dimensions: decoder.get_u16("dimensions")?,
            graph_degree: decoder.get_u16("graph degree")?,
            code_len: decoder.get_u32("code length")?,
            node_body_bytes: decoder.get_u32("node body bytes")?,
            node_record_bytes: decoder.get_u32("node record bytes")?,
            node_stride_bytes: decoder.get_u32("node stride bytes")?,
            page_payload_bytes: decoder.get_u32("page payload bytes")?,
            nodes_per_page: decoder.get_u16("nodes per page")?,
            extent_blocks: decoder.get_u32("extent blocks")?,
        };
        decoder.finish("fixed-stride layout")?;
        layout.validate()?;
        Ok(layout)
    }

    pub fn digest(&self) -> Result<[u8; 32], String> {
        Ok(domain_digest(LAYOUT_DOMAIN, &self.encode()?))
    }

    pub(crate) fn is_packed(&self) -> bool {
        self.nodes_per_page != 0
    }

    pub(crate) fn data_offset(&self) -> Result<u16, String> {
        self.pg_page_header_bytes
            .checked_add(self.page_header_bytes)
            .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: page data offset overflow".to_owned())
    }

    pub(crate) fn address(&self, node_ordinal: u64) -> Result<FixedStrideAddress, String> {
        self.validate()?;
        let data_offset = self.data_offset()?;
        if self.is_packed() {
            let nodes_per_page = u64::from(self.nodes_per_page);
            let page_index = node_ordinal / nodes_per_page;
            let first_block = page_index
                .checked_add(1)
                .and_then(|value| u32::try_from(value).ok())
                .ok_or_else(|| {
                    "EC_FIXED_STRIDE_FORMAT: packed node block exceeds BlockNumber".to_owned()
                })?;
            let slot_index = u16::try_from(node_ordinal % nodes_per_page)
                .expect("remainder is below persisted u16 nodes-per-page");
            let slot_bytes = u32::from(slot_index)
                .checked_mul(self.node_stride_bytes)
                .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: packed slot offset overflow".to_owned())?;
            let byte_offset = u32::from(data_offset)
                .checked_add(slot_bytes)
                .and_then(|value| u16::try_from(value).ok())
                .ok_or_else(|| {
                    "EC_FIXED_STRIDE_FORMAT: packed byte offset exceeds u16".to_owned()
                })?;
            Ok(FixedStrideAddress {
                first_block,
                byte_offset,
                slot_index,
                extent_blocks: 1,
            })
        } else {
            let block_index = node_ordinal
                .checked_mul(u64::from(self.extent_blocks))
                .and_then(|value| value.checked_add(1))
                .ok_or_else(|| {
                    "EC_FIXED_STRIDE_FORMAT: multi-block node address overflow".to_owned()
                })?;
            let last_block = block_index
                .checked_add(u64::from(self.extent_blocks) - 1)
                .ok_or_else(|| {
                    "EC_FIXED_STRIDE_FORMAT: multi-block node end overflow".to_owned()
                })?;
            if last_block > u64::from(u32::MAX) {
                return Err(
                    "EC_FIXED_STRIDE_FORMAT: multi-block node exceeds BlockNumber".to_owned(),
                );
            }
            Ok(FixedStrideAddress {
                first_block: block_index as u32,
                byte_offset: data_offset,
                slot_index: 0,
                extent_blocks: self.extent_blocks,
            })
        }
    }

    fn exact_vector_offset(&self) -> usize {
        FIXED_STRIDE_NODE_HEADER_BYTES
    }

    fn search_code_offset(&self) -> usize {
        self.exact_vector_offset() + usize::from(self.dimensions) * 4
    }

    fn neighbor_ids_offset(&self) -> usize {
        self.search_code_offset() + self.code_len as usize
    }

    fn neighbor_codes_offset(&self) -> usize {
        self.neighbor_ids_offset() + usize::from(self.graph_degree) * 8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FixedStrideMetadataV1 {
    pub(crate) generation_tag: [u8; 16],
    pub(crate) layout: DistannFixedStrideLayoutDescriptorV1,
}

impl FixedStrideMetadataV1 {
    pub(crate) fn encode(&self) -> Result<[u8; FIXED_STRIDE_METADATA_BYTES], String> {
        let layout = self.layout.encode()?;
        let layout_digest = self.layout.digest()?;
        let mut out = [0_u8; FIXED_STRIDE_METADATA_BYTES];
        out[..4].copy_from_slice(&METADATA_MAGIC);
        out[4..6].copy_from_slice(&FIXED_STRIDE_LAYOUT_VERSION.to_le_bytes());
        out[6..8].copy_from_slice(&(FIXED_STRIDE_METADATA_BYTES as u16).to_le_bytes());
        out[8..24].copy_from_slice(&self.generation_tag);
        out[24..56].copy_from_slice(&layout_digest);
        out[56..56 + FIXED_STRIDE_LAYOUT_BYTES].copy_from_slice(&layout);
        let digest = domain_digest(METADATA_DOMAIN, &out);
        out[128..160].copy_from_slice(&digest);
        Ok(out)
    }

    pub(crate) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() < 6 {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: metadata is too short for magic/version".to_owned(),
            );
        }
        if input[..4] != METADATA_MAGIC {
            return Err("EC_FIXED_STRIDE_FORMAT: metadata magic mismatch".to_owned());
        }
        let version = u16::from_le_bytes(input[4..6].try_into().expect("version bytes"));
        if version != FIXED_STRIDE_LAYOUT_VERSION {
            return Err(format!(
                "EC_FIXED_STRIDE_FORMAT: unsupported metadata version {version}"
            ));
        }
        if input.len() != FIXED_STRIDE_METADATA_BYTES
            || u16::from_le_bytes(input[6..8].try_into().expect("metadata length bytes"))
                != FIXED_STRIDE_METADATA_BYTES as u16
            || input[98..128].iter().any(|byte| *byte != 0)
        {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: metadata length or reserved bytes mismatch".to_owned(),
            );
        }
        let supplied_digest: [u8; 32] = input[128..160].try_into().expect("metadata digest bytes");
        let mut canonical = input.to_vec();
        canonical[128..160].fill(0);
        if domain_digest(METADATA_DOMAIN, &canonical) != supplied_digest {
            return Err("EC_FIXED_STRIDE_FORMAT: metadata digest mismatch".to_owned());
        }
        let layout = DistannFixedStrideLayoutDescriptorV1::decode(
            &input[56..56 + FIXED_STRIDE_LAYOUT_BYTES],
        )?;
        let expected_layout_digest: [u8; 32] =
            input[24..56].try_into().expect("layout digest bytes");
        if layout.digest()? != expected_layout_digest {
            return Err("EC_FIXED_STRIDE_FORMAT: metadata layout digest mismatch".to_owned());
        }
        Ok(Self {
            generation_tag: input[8..24].try_into().expect("generation tag bytes"),
            layout,
        })
    }
}

pub(crate) fn fixed_stride_generation_tag(
    descriptor_digest: &[u8; 32],
    logical_index_uuid: &[u8; 16],
    build_id: &[u8; 16],
) -> [u8; 16] {
    let mut canonical = Vec::with_capacity(64);
    canonical.extend_from_slice(descriptor_digest);
    canonical.extend_from_slice(logical_index_uuid);
    canonical.extend_from_slice(build_id);
    let digest = domain_digest(GENERATION_TAG_DOMAIN, &canonical);
    digest[..16].try_into().expect("digest prefix is 16 bytes")
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FixedStrideNodeV1 {
    pub(crate) tombstoned: bool,
    pub(crate) node_ordinal: u64,
    pub(crate) vec_id: u64,
    pub(crate) row_tid: ItemPointer,
    pub(crate) neighbor_count: u16,
    pub(crate) exact_vector: Vec<f32>,
    pub(crate) search_code: Vec<u8>,
    pub(crate) neighbor_vec_ids: Vec<u64>,
    pub(crate) neighbor_codes: Vec<u8>,
}

impl FixedStrideNodeV1 {
    pub(crate) fn empty() -> Self {
        Self {
            tombstoned: false,
            node_ordinal: 0,
            vec_id: 0,
            row_tid: ItemPointer::INVALID,
            neighbor_count: 0,
            exact_vector: Vec::new(),
            search_code: Vec::new(),
            neighbor_vec_ids: Vec::new(),
            neighbor_codes: Vec::new(),
        }
    }

    fn validate(&self, layout: &DistannFixedStrideLayoutDescriptorV1) -> Result<(), String> {
        layout.validate()?;
        let degree = usize::from(layout.graph_degree);
        let code_len = layout.code_len as usize;
        if self.row_tid == ItemPointer::INVALID {
            return Err("EC_FIXED_STRIDE_FORMAT: node row TID is invalid".to_owned());
        }
        if usize::from(self.neighbor_count) > degree {
            return Err("EC_FIXED_STRIDE_FORMAT: neighbor count exceeds graph degree".to_owned());
        }
        if self.exact_vector.len() != usize::from(layout.dimensions)
            || self.exact_vector.iter().any(|value| !value.is_finite())
        {
            return Err(
                "EC_FIXED_STRIDE_FORMAT: exact vector shape or finiteness mismatch".to_owned(),
            );
        }
        if self.search_code.len() != code_len
            || self.neighbor_vec_ids.len() != degree
            || self.neighbor_codes.len() != degree * code_len
        {
            return Err("EC_FIXED_STRIDE_FORMAT: node array shape mismatch".to_owned());
        }
        let live = usize::from(self.neighbor_count);
        if self.neighbor_vec_ids[live..]
            .iter()
            .any(|value| *value != 0)
            || self.neighbor_codes[live * code_len..]
                .iter()
                .any(|value| *value != 0)
        {
            return Err("EC_FIXED_STRIDE_FORMAT: adjacency padding is non-zero".to_owned());
        }
        Ok(())
    }

    pub(crate) fn encode(
        &self,
        layout: &DistannFixedStrideLayoutDescriptorV1,
    ) -> Result<Vec<u8>, String> {
        self.validate(layout)?;
        let stride = layout.node_stride_bytes as usize;
        let mut out = vec![0_u8; stride];
        out[..4].copy_from_slice(&NODE_MAGIC);
        out[4..6].copy_from_slice(&FIXED_STRIDE_NODE_FORMAT_VERSION.to_le_bytes());
        let flags = if self.tombstoned {
            NODE_FLAG_TOMBSTONE
        } else {
            0
        };
        out[6..8].copy_from_slice(&flags.to_le_bytes());
        out[8..10].copy_from_slice(&(FIXED_STRIDE_NODE_HEADER_BYTES as u16).to_le_bytes());
        out[10..12].copy_from_slice(&self.neighbor_count.to_le_bytes());
        out[16..24].copy_from_slice(&self.node_ordinal.to_le_bytes());
        out[24..32].copy_from_slice(&self.vec_id.to_le_bytes());
        out[32..36].copy_from_slice(&self.row_tid.block_number.to_le_bytes());
        out[36..38].copy_from_slice(&self.row_tid.offset_number.to_le_bytes());

        let mut cursor = layout.exact_vector_offset();
        for value in &self.exact_vector {
            out[cursor..cursor + 4].copy_from_slice(&value.to_le_bytes());
            cursor += 4;
        }
        debug_assert_eq!(cursor, layout.search_code_offset());
        out[cursor..cursor + self.search_code.len()].copy_from_slice(&self.search_code);
        cursor += self.search_code.len();
        debug_assert_eq!(cursor, layout.neighbor_ids_offset());
        for vec_id in &self.neighbor_vec_ids {
            out[cursor..cursor + 8].copy_from_slice(&vec_id.to_le_bytes());
            cursor += 8;
        }
        debug_assert_eq!(cursor, layout.neighbor_codes_offset());
        out[cursor..cursor + self.neighbor_codes.len()].copy_from_slice(&self.neighbor_codes);
        cursor += self.neighbor_codes.len();
        debug_assert_eq!(cursor, layout.node_record_bytes as usize);

        let digest = domain_digest(NODE_DOMAIN, &out);
        out[NODE_DIGEST_OFFSET..NODE_DIGEST_OFFSET + NODE_DIGEST_BYTES].copy_from_slice(&digest);
        Ok(out)
    }

    pub(crate) fn decode_into(
        input: &[u8],
        layout: &DistannFixedStrideLayoutDescriptorV1,
        expected_ordinal: u64,
        expected_vec_id: u64,
        out: &mut Self,
    ) -> Result<(), String> {
        layout.validate()?;
        if input.len() < 6 {
            return Err("EC_FIXED_STRIDE_FORMAT: node is too short for magic/version".to_owned());
        }
        if input[..4] != NODE_MAGIC {
            return Err("EC_FIXED_STRIDE_FORMAT: node magic mismatch".to_owned());
        }
        let version = u16::from_le_bytes(input[4..6].try_into().expect("version bytes"));
        if version != FIXED_STRIDE_NODE_FORMAT_VERSION {
            return Err(format!(
                "EC_FIXED_STRIDE_FORMAT: unsupported node version {version}"
            ));
        }
        if input.len() != layout.node_stride_bytes as usize {
            return Err(format!(
                "EC_FIXED_STRIDE_FORMAT: node length is {}, expected {}",
                input.len(),
                layout.node_stride_bytes
            ));
        }
        if u16::from_le_bytes(input[8..10].try_into().expect("header bytes"))
            != FIXED_STRIDE_NODE_HEADER_BYTES as u16
        {
            return Err("EC_FIXED_STRIDE_FORMAT: node header length mismatch".to_owned());
        }
        let flags = u16::from_le_bytes(input[6..8].try_into().expect("flag bytes"));
        if flags & !NODE_FLAG_TOMBSTONE != 0 {
            return Err("EC_FIXED_STRIDE_FORMAT: node flags contain unknown bits".to_owned());
        }
        if input[12..16].iter().any(|byte| *byte != 0)
            || input[38..48].iter().any(|byte| *byte != 0)
        {
            return Err("EC_FIXED_STRIDE_FORMAT: node reserved bytes are non-zero".to_owned());
        }

        let supplied_digest: [u8; 32] = input
            [NODE_DIGEST_OFFSET..NODE_DIGEST_OFFSET + NODE_DIGEST_BYTES]
            .try_into()
            .expect("node digest bytes");
        let mut canonical = input.to_vec();
        canonical[NODE_DIGEST_OFFSET..NODE_DIGEST_OFFSET + NODE_DIGEST_BYTES].fill(0);
        if domain_digest(NODE_DOMAIN, &canonical) != supplied_digest {
            return Err("EC_FIXED_STRIDE_FORMAT: node digest mismatch".to_owned());
        }

        let node_ordinal = u64::from_le_bytes(input[16..24].try_into().expect("ordinal bytes"));
        let vec_id = u64::from_le_bytes(input[24..32].try_into().expect("vec-id bytes"));
        if node_ordinal != expected_ordinal || vec_id != expected_vec_id {
            return Err("EC_FIXED_STRIDE_FORMAT: node directory identity mismatch".to_owned());
        }
        let row_tid = ItemPointer::decode(&input[32..32 + ITEM_POINTER_BYTES])?;
        if row_tid == ItemPointer::INVALID {
            return Err("EC_FIXED_STRIDE_FORMAT: node row TID is invalid".to_owned());
        }
        let neighbor_count =
            u16::from_le_bytes(input[10..12].try_into().expect("neighbor-count bytes"));
        if neighbor_count > layout.graph_degree {
            return Err("EC_FIXED_STRIDE_FORMAT: neighbor count exceeds graph degree".to_owned());
        }

        out.exact_vector.clear();
        out.exact_vector.reserve(usize::from(layout.dimensions));
        let mut cursor = layout.exact_vector_offset();
        for _ in 0..layout.dimensions {
            let value =
                f32::from_le_bytes(input[cursor..cursor + 4].try_into().expect("vector bytes"));
            if !value.is_finite() {
                return Err("EC_FIXED_STRIDE_FORMAT: exact vector is non-finite".to_owned());
            }
            out.exact_vector.push(value);
            cursor += 4;
        }
        let code_len = layout.code_len as usize;
        out.search_code.clear();
        out.search_code
            .extend_from_slice(&input[cursor..cursor + code_len]);
        cursor += code_len;
        out.neighbor_vec_ids.clear();
        out.neighbor_vec_ids
            .reserve(usize::from(layout.graph_degree));
        for _ in 0..layout.graph_degree {
            out.neighbor_vec_ids.push(u64::from_le_bytes(
                input[cursor..cursor + 8]
                    .try_into()
                    .expect("neighbor-id bytes"),
            ));
            cursor += 8;
        }
        let neighbor_code_bytes = usize::from(layout.graph_degree) * code_len;
        out.neighbor_codes.clear();
        out.neighbor_codes
            .extend_from_slice(&input[cursor..cursor + neighbor_code_bytes]);
        cursor += neighbor_code_bytes;
        if input[cursor..].iter().any(|byte| *byte != 0) {
            return Err("EC_FIXED_STRIDE_FORMAT: node alignment padding is non-zero".to_owned());
        }
        let live = usize::from(neighbor_count);
        if out.neighbor_vec_ids[live..].iter().any(|value| *value != 0)
            || out.neighbor_codes[live * code_len..]
                .iter()
                .any(|value| *value != 0)
        {
            return Err("EC_FIXED_STRIDE_FORMAT: adjacency padding is non-zero".to_owned());
        }

        out.tombstoned = flags & NODE_FLAG_TOMBSTONE != 0;
        out.node_ordinal = node_ordinal;
        out.vec_id = vec_id;
        out.row_tid = row_tid;
        out.neighbor_count = neighbor_count;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum FixedStridePageKind {
    Packed = 1,
    MultiBlock = 2,
}

impl FixedStridePageKind {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Packed),
            2 => Ok(Self::MultiBlock),
            other => Err(format!(
                "EC_FIXED_STRIDE_FORMAT: unsupported page kind {other}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FixedStridePageEnvelopeV1 {
    pub(crate) kind: FixedStridePageKind,
    pub(crate) base_ordinal: u64,
    pub(crate) slot_count: u16,
    pub(crate) segment_index: u16,
    pub(crate) segment_count: u16,
    pub(crate) content_bytes: u16,
    pub(crate) generation_tag: [u8; 16],
}

impl FixedStridePageEnvelopeV1 {
    fn validate_shape(
        &self,
        layout: &DistannFixedStrideLayoutDescriptorV1,
        payload_len: usize,
        block_number: u32,
    ) -> Result<(), String> {
        layout.validate()?;
        if usize::from(self.content_bytes) != payload_len
            || payload_len > layout.page_payload_bytes as usize
        {
            return Err("EC_FIXED_STRIDE_FORMAT: page content length mismatch".to_owned());
        }
        match self.kind {
            FixedStridePageKind::Packed => {
                if !layout.is_packed()
                    || self.slot_count == 0
                    || self.slot_count > layout.nodes_per_page
                    || self.segment_index != 0
                    || self.segment_count != 1
                    || self.base_ordinal % u64::from(layout.nodes_per_page) != 0
                    || payload_len
                        != usize::from(self.slot_count) * layout.node_stride_bytes as usize
                {
                    return Err("EC_FIXED_STRIDE_FORMAT: invalid packed page shape".to_owned());
                }
                let expected = layout.address(self.base_ordinal)?.first_block;
                if block_number != expected {
                    return Err("EC_FIXED_STRIDE_FORMAT: packed page block mismatch".to_owned());
                }
            }
            FixedStridePageKind::MultiBlock => {
                let expected_segments = u16::try_from(layout.extent_blocks).map_err(|_| {
                    "EC_FIXED_STRIDE_FORMAT: extent block count exceeds page header".to_owned()
                })?;
                if layout.is_packed()
                    || self.slot_count != 1
                    || self.segment_count != expected_segments
                    || self.segment_index >= self.segment_count
                {
                    return Err("EC_FIXED_STRIDE_FORMAT: invalid multi-block page shape".to_owned());
                }
                let consumed = usize::from(self.segment_index)
                    .checked_mul(layout.page_payload_bytes as usize)
                    .ok_or_else(|| {
                        "EC_FIXED_STRIDE_FORMAT: segment byte offset overflow".to_owned()
                    })?;
                let remaining = (layout.node_stride_bytes as usize)
                    .checked_sub(consumed)
                    .ok_or_else(|| {
                        "EC_FIXED_STRIDE_FORMAT: segment exceeds node stride".to_owned()
                    })?;
                let expected_payload = remaining.min(layout.page_payload_bytes as usize);
                let expected_block = layout
                    .address(self.base_ordinal)?
                    .first_block
                    .checked_add(u32::from(self.segment_index))
                    .ok_or_else(|| "EC_FIXED_STRIDE_FORMAT: segment block overflow".to_owned())?;
                if payload_len != expected_payload || block_number != expected_block {
                    return Err(
                        "EC_FIXED_STRIDE_FORMAT: multi-block segment identity mismatch".to_owned(),
                    );
                }
            }
        }
        Ok(())
    }

    pub(crate) fn encode(
        &self,
        layout: &DistannFixedStrideLayoutDescriptorV1,
        payload: &[u8],
        block_number: u32,
    ) -> Result<[u8; FIXED_STRIDE_PAGE_HEADER_BYTES], String> {
        self.validate_shape(layout, payload.len(), block_number)?;
        let mut out = [0_u8; FIXED_STRIDE_PAGE_HEADER_BYTES];
        out[..4].copy_from_slice(&PAGE_MAGIC);
        out[4..6].copy_from_slice(&FIXED_STRIDE_PAGE_FORMAT_VERSION.to_le_bytes());
        out[6] = self.kind as u8;
        out[8..10].copy_from_slice(&(FIXED_STRIDE_PAGE_HEADER_BYTES as u16).to_le_bytes());
        out[12..16].copy_from_slice(&layout.node_stride_bytes.to_le_bytes());
        out[16..24].copy_from_slice(&self.base_ordinal.to_le_bytes());
        out[24..26].copy_from_slice(&self.slot_count.to_le_bytes());
        out[26..28].copy_from_slice(&self.segment_index.to_le_bytes());
        out[28..30].copy_from_slice(&self.segment_count.to_le_bytes());
        out[30..32].copy_from_slice(&self.content_bytes.to_le_bytes());
        out[32..48].copy_from_slice(&self.generation_tag);
        let mut canonical = Vec::with_capacity(out.len() + payload.len());
        canonical.extend_from_slice(&out);
        canonical.extend_from_slice(payload);
        let digest = domain_digest(PAGE_DOMAIN, &canonical);
        out[PAGE_DIGEST_OFFSET..PAGE_DIGEST_OFFSET + PAGE_DIGEST_BYTES].copy_from_slice(&digest);
        Ok(out)
    }

    pub(crate) fn decode(
        input: &[u8],
        payload: &[u8],
        layout: &DistannFixedStrideLayoutDescriptorV1,
        expected_generation_tag: &[u8; 16],
        block_number: u32,
    ) -> Result<Self, String> {
        if input.len() < 6 {
            return Err("EC_FIXED_STRIDE_FORMAT: page is too short for magic/version".to_owned());
        }
        if input[..4] != PAGE_MAGIC {
            return Err("EC_FIXED_STRIDE_FORMAT: page magic mismatch".to_owned());
        }
        let version = u16::from_le_bytes(input[4..6].try_into().expect("version bytes"));
        if version != FIXED_STRIDE_PAGE_FORMAT_VERSION {
            return Err(format!(
                "EC_FIXED_STRIDE_FORMAT: unsupported page version {version}"
            ));
        }
        if input.len() != FIXED_STRIDE_PAGE_HEADER_BYTES {
            return Err("EC_FIXED_STRIDE_FORMAT: page header length mismatch".to_owned());
        }
        if input[7] != 0
            || u16::from_le_bytes(input[8..10].try_into().expect("header bytes"))
                != FIXED_STRIDE_PAGE_HEADER_BYTES as u16
            || input[10..12].iter().any(|byte| *byte != 0)
        {
            return Err("EC_FIXED_STRIDE_FORMAT: page flags/header/reserved mismatch".to_owned());
        }
        let record_bytes = u32::from_le_bytes(input[12..16].try_into().expect("record bytes"));
        if record_bytes != layout.node_stride_bytes {
            return Err("EC_FIXED_STRIDE_FORMAT: page record stride mismatch".to_owned());
        }
        let generation_tag: [u8; 16] = input[32..48].try_into().expect("generation-tag bytes");
        if &generation_tag != expected_generation_tag {
            return Err("EC_FIXED_STRIDE_FORMAT: page generation binding mismatch".to_owned());
        }
        let supplied_digest: [u8; 32] = input
            [PAGE_DIGEST_OFFSET..PAGE_DIGEST_OFFSET + PAGE_DIGEST_BYTES]
            .try_into()
            .expect("page digest bytes");
        let mut canonical = Vec::with_capacity(input.len() + payload.len());
        canonical.extend_from_slice(input);
        canonical[PAGE_DIGEST_OFFSET..PAGE_DIGEST_OFFSET + PAGE_DIGEST_BYTES].fill(0);
        canonical.extend_from_slice(payload);
        if domain_digest(PAGE_DOMAIN, &canonical) != supplied_digest {
            return Err("EC_FIXED_STRIDE_FORMAT: page digest mismatch".to_owned());
        }
        let envelope = Self {
            kind: FixedStridePageKind::decode(input[6])?,
            base_ordinal: u64::from_le_bytes(input[16..24].try_into().expect("base-ordinal bytes")),
            slot_count: u16::from_le_bytes(input[24..26].try_into().expect("slot-count bytes")),
            segment_index: u16::from_le_bytes(
                input[26..28].try_into().expect("segment-index bytes"),
            ),
            segment_count: u16::from_le_bytes(
                input[28..30].try_into().expect("segment-count bytes"),
            ),
            content_bytes: u16::from_le_bytes(
                input[30..32].try_into().expect("content-byte bytes"),
            ),
            generation_tag,
        };
        envelope.validate_shape(layout, payload.len(), block_number)?;
        Ok(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node(
        layout: &DistannFixedStrideLayoutDescriptorV1,
        ordinal: u64,
    ) -> FixedStrideNodeV1 {
        let degree = usize::from(layout.graph_degree);
        let code_len = layout.code_len as usize;
        let live = degree.saturating_sub(1);
        let mut neighbor_vec_ids = vec![0; degree];
        for (index, vec_id) in neighbor_vec_ids.iter_mut().take(live).enumerate() {
            *vec_id = 100 + index as u64;
        }
        let mut neighbor_codes = vec![0; degree * code_len];
        for (index, byte) in neighbor_codes.iter_mut().take(live * code_len).enumerate() {
            *byte = index as u8;
        }
        FixedStrideNodeV1 {
            tombstoned: false,
            node_ordinal: ordinal,
            vec_id: 0x0102_0304_0506_0708 ^ ordinal,
            row_tid: ItemPointer {
                block_number: 17,
                offset_number: 3,
            },
            neighbor_count: live as u16,
            exact_vector: (0..layout.dimensions)
                .map(|value| f32::from(value) / 7.0)
                .collect(),
            search_code: (0..code_len).map(|value| value as u8).collect(),
            neighbor_vec_ids,
            neighbor_codes,
        }
    }

    #[test]
    fn layout_covers_packed_one_page_and_multi_block_cases() {
        let packed = DistannFixedStrideLayoutDescriptorV1::new(16, 4, 8).expect("packed layout");
        assert!(packed.nodes_per_page > 1);
        assert_eq!(packed.extent_blocks, 1);
        let last_first_page = packed
            .address(u64::from(packed.nodes_per_page - 1))
            .unwrap();
        let next_page = packed.address(u64::from(packed.nodes_per_page)).unwrap();
        assert_eq!(last_first_page.first_block, 1);
        assert_eq!(next_page.first_block, 2);
        assert_eq!(next_page.slot_index, 0);

        let one_page =
            DistannFixedStrideLayoutDescriptorV1::new(1024, 16, 128).expect("one-page layout");
        assert_eq!(one_page.nodes_per_page, 1);
        assert_eq!(one_page.extent_blocks, 1);

        let multi =
            DistannFixedStrideLayoutDescriptorV1::new(1536, 32, 192).expect("multi-block layout");
        assert_eq!(multi.nodes_per_page, 0);
        assert_eq!(multi.extent_blocks, 2);
        assert_eq!(multi.address(3).unwrap().first_block, 7);
        assert!(multi.address(u64::from(u32::MAX)).is_err());
    }

    #[test]
    fn persisted_layout_must_match_derived_arithmetic() {
        let layout = DistannFixedStrideLayoutDescriptorV1::new(32, 8, 16).unwrap();
        let encoded = layout.encode().unwrap();
        assert_eq!(
            DistannFixedStrideLayoutDescriptorV1::decode(&encoded).unwrap(),
            layout
        );

        let mut unknown = encoded;
        unknown[..2].copy_from_slice(&2_u16.to_le_bytes());
        assert!(DistannFixedStrideLayoutDescriptorV1::decode(&unknown).is_err());

        let mut layout = layout;
        layout.node_stride_bytes += 8;
        assert!(layout.validate().is_err());
    }

    #[test]
    fn node_round_trip_reuses_buffers_and_rejects_corruption() {
        let layout = DistannFixedStrideLayoutDescriptorV1::new(16, 4, 8).unwrap();
        let node = sample_node(&layout, 7);
        let encoded = node.encode(&layout).unwrap();
        assert_eq!(encoded.len(), layout.node_stride_bytes as usize);
        let mut decoded = FixedStrideNodeV1::empty();
        decoded.exact_vector.reserve(100);
        FixedStrideNodeV1::decode_into(
            &encoded,
            &layout,
            node.node_ordinal,
            node.vec_id,
            &mut decoded,
        )
        .unwrap();
        assert_eq!(decoded, node);

        let mut wrong_version = encoded.clone();
        wrong_version[4..6].copy_from_slice(&2_u16.to_le_bytes());
        assert!(FixedStrideNodeV1::decode_into(
            &wrong_version,
            &layout,
            node.node_ordinal,
            node.vec_id,
            &mut decoded
        )
        .is_err());
        assert!(FixedStrideNodeV1::decode_into(
            &encoded,
            &layout,
            node.node_ordinal + 1,
            node.vec_id,
            &mut decoded
        )
        .is_err());

        let mut digest_corrupt = encoded.clone();
        digest_corrupt[layout.search_code_offset()] ^= 0x80;
        assert!(FixedStrideNodeV1::decode_into(
            &digest_corrupt,
            &layout,
            node.node_ordinal,
            node.vec_id,
            &mut decoded
        )
        .is_err());

        let mut bad_padding_node = node.clone();
        *bad_padding_node.neighbor_vec_ids.last_mut().unwrap() = 999;
        assert!(bad_padding_node.encode(&layout).is_err());

        let mut bad_alignment = encoded;
        *bad_alignment.last_mut().unwrap() = 1;
        assert!(FixedStrideNodeV1::decode_into(
            &bad_alignment,
            &layout,
            node.node_ordinal,
            node.vec_id,
            &mut decoded
        )
        .is_err());
    }

    #[test]
    fn page_envelopes_bind_packed_and_every_multi_block_segment() {
        let tag = [7_u8; 16];
        let packed = DistannFixedStrideLayoutDescriptorV1::new(16, 4, 8).unwrap();
        let payload = vec![9_u8; packed.node_stride_bytes as usize * 2];
        let envelope = FixedStridePageEnvelopeV1 {
            kind: FixedStridePageKind::Packed,
            base_ordinal: 0,
            slot_count: 2,
            segment_index: 0,
            segment_count: 1,
            content_bytes: payload.len() as u16,
            generation_tag: tag,
        };
        let encoded = envelope.encode(&packed, &payload, 1).unwrap();
        assert_eq!(
            FixedStridePageEnvelopeV1::decode(&encoded, &payload, &packed, &tag, 1).unwrap(),
            envelope
        );
        assert!(FixedStridePageEnvelopeV1::decode(&encoded, &payload, &packed, &tag, 2).is_err());
        let mut corrupt = payload.clone();
        corrupt[0] ^= 1;
        assert!(FixedStridePageEnvelopeV1::decode(&encoded, &corrupt, &packed, &tag, 1).is_err());

        let multi = DistannFixedStrideLayoutDescriptorV1::new(1536, 32, 192).unwrap();
        let address = multi.address(2).unwrap();
        let full = vec![3_u8; multi.node_stride_bytes as usize];
        for segment in 0..multi.extent_blocks {
            let start = segment as usize * multi.page_payload_bytes as usize;
            let end = (start + multi.page_payload_bytes as usize).min(full.len());
            let segment_payload = &full[start..end];
            let envelope = FixedStridePageEnvelopeV1 {
                kind: FixedStridePageKind::MultiBlock,
                base_ordinal: 2,
                slot_count: 1,
                segment_index: segment as u16,
                segment_count: multi.extent_blocks as u16,
                content_bytes: segment_payload.len() as u16,
                generation_tag: tag,
            };
            let block = address.first_block + segment;
            let encoded = envelope.encode(&multi, segment_payload, block).unwrap();
            assert_eq!(
                FixedStridePageEnvelopeV1::decode(&encoded, segment_payload, &multi, &tag, block)
                    .unwrap(),
                envelope
            );
        }
    }

    #[test]
    fn generation_tag_binds_descriptor_index_and_build() {
        let descriptor = [1_u8; 32];
        let logical = [2_u8; 16];
        let build = [3_u8; 16];
        let tag = fixed_stride_generation_tag(&descriptor, &logical, &build);
        let mut other_build = build;
        other_build[0] ^= 1;
        assert_ne!(
            tag,
            fixed_stride_generation_tag(&descriptor, &logical, &other_build)
        );

        let metadata = FixedStrideMetadataV1 {
            generation_tag: tag,
            layout: DistannFixedStrideLayoutDescriptorV1::new(1536, 32, 192).unwrap(),
        };
        let encoded = metadata.encode().unwrap();
        assert_eq!(FixedStrideMetadataV1::decode(&encoded).unwrap(), metadata);
        let mut corrupt = encoded;
        corrupt[60] ^= 1;
        assert!(FixedStrideMetadataV1::decode(&corrupt).is_err());
    }
}
