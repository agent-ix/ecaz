//! SymphonyQG page layout: metadata page, tuple codecs, and the
//! Symphony-typed convenience methods that extend the cross-AM
//! [`crate::storage::page::DataPage`] / [`DataPageChain`].
//!
//! Unlike `ec_hnsw`, Symphony stores centered RaBitQ codes with the
//! adjacency, not with the element payload, because the same neighbor
//! has a different code under each visited center.

use crate::quant::rabitq::RABITQ_SCALAR_LEN;
use crate::storage::page::{
    align_up, aligned_tuple_bytes, usable_page_bytes, ALIGNMENT_BYTES, DEFAULT_PAGE_SIZE,
};

pub use crate::storage::page::{
    DataPage, DataPageChain, ItemPointer, HEAPTID_INLINE_CAPACITY, ITEM_POINTER_BYTES,
    PAGE_HEADER_BYTES,
};

pub const SYMPHONY_ELEMENT_TAG: u8 = 0x01;
pub const SYMPHONY_NEIGHBOR_TAG: u8 = 0x02;
pub const INDEX_FORMAT_V5_SYMPHONY: u16 = 5;

const METADATA_BYTES: usize = 2 + 2 + ITEM_POINTER_BYTES + 2 + 1 + 1 + 8 + 8 + 2 + 2;
const METADATA_SPECIAL_BYTES: usize = align_up(METADATA_BYTES, ALIGNMENT_BYTES);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentFormatMetadata {
    pub m: u16,
    pub ef_construction: u16,
    pub entry_point: ItemPointer,
    pub dimensions: u16,
    pub rabitq_bits: u8,
    pub max_level: u8,
    pub seed: u64,
    pub inserted_since_rebuild: u64,
    pub padding_factor: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataPage {
    pub m: u16,
    pub ef_construction: u16,
    pub entry_point: ItemPointer,
    pub dimensions: u16,
    pub rabitq_bits: u8,
    pub max_level: u8,
    pub seed: u64,
    pub inserted_since_rebuild: u64,
    pub format_version: u16,
    pub padding_factor: u16,
}

impl MetadataPage {
    pub fn current_v5_symphony(current: CurrentFormatMetadata) -> Self {
        Self {
            m: current.m,
            ef_construction: current.ef_construction,
            entry_point: current.entry_point,
            dimensions: current.dimensions,
            rabitq_bits: current.rabitq_bits,
            max_level: current.max_level,
            seed: current.seed,
            inserted_since_rebuild: current.inserted_since_rebuild,
            format_version: INDEX_FORMAT_V5_SYMPHONY,
            padding_factor: current.padding_factor,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(METADATA_BYTES);
        out.extend_from_slice(&self.m.to_le_bytes());
        out.extend_from_slice(&self.ef_construction.to_le_bytes());
        self.entry_point.encode_into(&mut out);
        out.extend_from_slice(&self.dimensions.to_le_bytes());
        out.push(self.rabitq_bits);
        out.push(self.max_level);
        out.extend_from_slice(&self.seed.to_le_bytes());
        out.extend_from_slice(&self.inserted_since_rebuild.to_le_bytes());
        out.extend_from_slice(&self.format_version.to_le_bytes());
        out.extend_from_slice(&self.padding_factor.to_le_bytes());
        out
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != METADATA_BYTES {
            return Err(format!(
                "metadata page length mismatch: got {}, expected {METADATA_BYTES}",
                input.len()
            ));
        }

        let format_version =
            u16::from_le_bytes(input[30..32].try_into().expect("format version bytes"));
        if format_version != INDEX_FORMAT_V5_SYMPHONY {
            return Err(format!(
                "invalid symphony metadata format version: {format_version}"
            ));
        }

        let rabitq_bits = input[12];
        if rabitq_bits != 1 {
            return Err(format!(
                "invalid symphony RaBitQ bits setting: expected 1, got {rabitq_bits}"
            ));
        }

        let padding_factor =
            u16::from_le_bytes(input[32..34].try_into().expect("padding factor bytes"));
        if padding_factor == 0 {
            return Err("symphony padding factor must be at least 1".into());
        }

        Ok(Self {
            m: u16::from_le_bytes(input[0..2].try_into().expect("m bytes")),
            ef_construction: u16::from_le_bytes(
                input[2..4].try_into().expect("ef construction bytes"),
            ),
            entry_point: ItemPointer::decode(&input[4..10])?,
            dimensions: u16::from_le_bytes(input[10..12].try_into().expect("dimension bytes")),
            rabitq_bits,
            max_level: input[13],
            seed: u64::from_le_bytes(input[14..22].try_into().expect("seed bytes")),
            inserted_since_rebuild: u64::from_le_bytes(
                input[22..30]
                    .try_into()
                    .expect("inserted-since-rebuild bytes"),
            ),
            format_version,
            padding_factor,
        })
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
        if page.len() < PAGE_HEADER_BYTES + METADATA_BYTES {
            return Err(format!(
                "page too short: got {}, need at least {}",
                page.len(),
                PAGE_HEADER_BYTES + METADATA_BYTES
            ));
        }

        let special_offset = page.len() - METADATA_SPECIAL_BYTES;
        Self::decode(&page[special_offset..special_offset + METADATA_BYTES])
    }

    pub fn decode_contents(contents: &[u8]) -> Result<Self, String> {
        if contents.len() < METADATA_BYTES {
            return Err(format!(
                "page contents too short: got {}, need at least {METADATA_BYTES}",
                contents.len()
            ));
        }

        Self::decode(&contents[..METADATA_BYTES])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymphonyElementTuple {
    pub level: u8,
    pub deleted: bool,
    pub heaptids: Vec<ItemPointer>,
    pub neighbortid: ItemPointer,
}

#[derive(Debug, Clone, Copy)]
pub struct SymphonyElementTupleRef<'a> {
    pub level: u8,
    pub deleted: bool,
    heaptid_bytes: &'a [u8],
    heaptid_count: usize,
    pub neighbortid: ItemPointer,
}

impl<'a> SymphonyElementTupleRef<'a> {
    pub fn decode(input: &'a [u8]) -> Result<Self, String> {
        let expected_len = SymphonyElementTuple::encoded_len();
        if input.len() != expected_len {
            return Err(format!(
                "symphony element tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if input[0] != SYMPHONY_ELEMENT_TAG {
            return Err(format!("invalid symphony element tuple tag: {}", input[0]));
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

        Ok(Self {
            level: input[1],
            deleted: input[2] != 0,
            heaptid_bytes,
            heaptid_count,
            neighbortid,
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
                    .expect("validated symphony element tuple should only expose valid tids")
            })
    }

    pub fn collect_heaptids(&self) -> Vec<ItemPointer> {
        self.heaptids().collect()
    }
}

impl SymphonyElementTuple {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        if self.heaptids.len() > HEAPTID_INLINE_CAPACITY {
            return Err(format!(
                "too many heap tids: got {}, max {}",
                self.heaptids.len(),
                HEAPTID_INLINE_CAPACITY
            ));
        }

        let mut out = Vec::with_capacity(Self::encoded_len());
        out.push(SYMPHONY_ELEMENT_TAG);
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
        Ok(out)
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        let element = SymphonyElementTupleRef::decode(input)?;
        Ok(Self {
            level: element.level,
            deleted: element.deleted,
            heaptids: element.collect_heaptids(),
            neighbortid: element.neighbortid,
        })
    }

    pub const fn encoded_len() -> usize {
        1 + 1 + 1 + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES + 1 + ITEM_POINTER_BYTES
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymphonyNeighborTuple {
    pub count: u16,
    pub tids: Vec<ItemPointer>,
    pub centered_codes: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub struct SymphonyNeighborTupleRef<'a> {
    pub count: u16,
    tid_bytes: &'a [u8],
    code_bytes: &'a [u8],
    centered_code_len: usize,
}

impl<'a> SymphonyNeighborTupleRef<'a> {
    pub fn decode(input: &'a [u8], centered_code_len: usize) -> Result<Self, String> {
        if centered_code_len == 0 {
            return Err("centered code length must be positive".into());
        }
        if input.len() < 3 {
            return Err("symphony neighbor tuple too short".into());
        }
        if input[0] != SYMPHONY_NEIGHBOR_TAG {
            return Err(format!("invalid symphony neighbor tuple tag: {}", input[0]));
        }

        let count = u16::from_le_bytes(input[1..3].try_into().expect("neighbor count bytes"));
        let count_usize = usize::from(count);
        let tid_bytes_len = count_usize * ITEM_POINTER_BYTES;
        let code_bytes_len = count_usize * centered_code_len;
        let expected_len = 1 + 2 + tid_bytes_len + code_bytes_len;
        if input.len() != expected_len {
            return Err(format!(
                "symphony neighbor tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }

        let tid_bytes_end = 3 + tid_bytes_len;
        Ok(Self {
            count,
            tid_bytes: &input[3..tid_bytes_end],
            code_bytes: &input[tid_bytes_end..],
            centered_code_len,
        })
    }

    pub fn tids(&self) -> impl Iterator<Item = ItemPointer> + '_ {
        self.tid_bytes
            .chunks_exact(ITEM_POINTER_BYTES)
            .map(|chunk| {
                ItemPointer::decode(chunk)
                    .expect("validated symphony neighbor tuple should only expose valid tids")
            })
    }

    pub fn centered_codes(&self) -> impl Iterator<Item = &'a [u8]> + '_ {
        self.code_bytes.chunks_exact(self.centered_code_len)
    }

    pub fn centered_code(&self, index: usize) -> Option<&'a [u8]> {
        self.centered_codes().nth(index)
    }

    pub fn collect_tids(&self) -> Vec<ItemPointer> {
        self.tids().collect()
    }

    pub fn collect_centered_codes(&self) -> Vec<Vec<u8>> {
        self.centered_codes().map(ToOwned::to_owned).collect()
    }
}

impl SymphonyNeighborTuple {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        if usize::from(self.count) != self.tids.len() {
            return Err(format!(
                "neighbor count {} must equal tids length {}",
                self.count,
                self.tids.len()
            ));
        }
        if self.tids.len() != self.centered_codes.len() {
            return Err(format!(
                "neighbor tids length {} must equal centered code length {}",
                self.tids.len(),
                self.centered_codes.len()
            ));
        }

        let first_code_len = self.centered_codes.first().map_or(0, Vec::len);
        if first_code_len == 0 && !self.centered_codes.is_empty() {
            return Err("centered codes must not be empty".into());
        }
        for code in &self.centered_codes {
            if code.len() != first_code_len {
                return Err(format!(
                    "centered code length mismatch: expected {first_code_len}, got {}",
                    code.len()
                ));
            }
        }

        let mut out = Vec::with_capacity(neighbor_tuple_encoded_len(self.count, first_code_len));
        out.push(SYMPHONY_NEIGHBOR_TAG);
        out.extend_from_slice(&self.count.to_le_bytes());
        for tid in &self.tids {
            tid.encode_into(&mut out);
        }
        for code in &self.centered_codes {
            out.extend_from_slice(code);
        }
        Ok(out)
    }

    pub fn decode(input: &[u8], centered_code_len: usize) -> Result<Self, String> {
        let neighbor = SymphonyNeighborTupleRef::decode(input, centered_code_len)?;
        Ok(Self {
            count: neighbor.count,
            tids: neighbor.collect_tids(),
            centered_codes: neighbor.collect_centered_codes(),
        })
    }
}

pub fn centered_code_len(dimensions: u16) -> usize {
    usize::from(dimensions).div_ceil(8) + RABITQ_SCALAR_LEN
}

pub fn neighbor_tuple_encoded_len(count: u16, centered_code_len: usize) -> usize {
    1 + 2 + usize::from(count) * ITEM_POINTER_BYTES + usize::from(count) * centered_code_len
}

pub fn max_padded_degree_that_fits(centered_code_len: usize, page_size: usize) -> u16 {
    let usable = usable_page_bytes(page_size);
    let mut count = 0_u16;
    loop {
        let tuple_bytes = aligned_tuple_bytes(neighbor_tuple_encoded_len(count, centered_code_len));
        if tuple_bytes > usable {
            return count.saturating_sub(1);
        }
        if count == u16::MAX {
            return count;
        }
        count = count.saturating_add(1);
    }
}

pub fn symphony_element_tuple_fits_on_page(page_size: usize) -> bool {
    aligned_tuple_bytes(SymphonyElementTuple::encoded_len()) <= usable_page_bytes(page_size)
}

pub fn symphony_neighbor_tuple_fits_on_page(
    count: u16,
    centered_code_len: usize,
    page_size: usize,
) -> bool {
    aligned_tuple_bytes(neighbor_tuple_encoded_len(count, centered_code_len))
        <= usable_page_bytes(page_size)
}

pub fn default_max_padded_degree(dimensions: u16) -> u16 {
    max_padded_degree_that_fits(centered_code_len(dimensions), DEFAULT_PAGE_SIZE)
}

impl DataPage {
    pub fn insert_symphony_element(
        &mut self,
        tuple: &SymphonyElementTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn read_symphony_element(&self, tid: ItemPointer) -> Result<SymphonyElementTuple, String> {
        SymphonyElementTuple::decode(self.raw_tuple(tid)?)
    }

    pub fn read_symphony_element_ref(
        &self,
        tid: ItemPointer,
    ) -> Result<SymphonyElementTupleRef<'_>, String> {
        SymphonyElementTupleRef::decode(self.raw_tuple(tid)?)
    }

    pub fn update_symphony_element(
        &mut self,
        tid: ItemPointer,
        tuple: &SymphonyElementTuple,
    ) -> Result<(), String> {
        self.update_raw_tuple(tid, tuple.encode()?)
    }

    pub fn insert_symphony_neighbor(
        &mut self,
        tuple: &SymphonyNeighborTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn read_symphony_neighbor(
        &self,
        tid: ItemPointer,
        centered_code_len: usize,
    ) -> Result<SymphonyNeighborTuple, String> {
        SymphonyNeighborTuple::decode(self.raw_tuple(tid)?, centered_code_len)
    }

    pub fn read_symphony_neighbor_ref(
        &self,
        tid: ItemPointer,
        centered_code_len: usize,
    ) -> Result<SymphonyNeighborTupleRef<'_>, String> {
        SymphonyNeighborTupleRef::decode(self.raw_tuple(tid)?, centered_code_len)
    }

    pub fn update_symphony_neighbor(
        &mut self,
        tid: ItemPointer,
        tuple: &SymphonyNeighborTuple,
    ) -> Result<(), String> {
        self.update_raw_tuple(tid, tuple.encode()?)
    }
}

impl DataPageChain {
    pub fn insert_symphony_element(
        &mut self,
        tuple: &SymphonyElementTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn read_symphony_element(&self, tid: ItemPointer) -> Result<SymphonyElementTuple, String> {
        self.get_page(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .read_symphony_element(tid)
    }

    pub fn read_symphony_element_ref(
        &self,
        tid: ItemPointer,
    ) -> Result<SymphonyElementTupleRef<'_>, String> {
        self.get_page(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .read_symphony_element_ref(tid)
    }

    pub fn insert_symphony_neighbor(
        &mut self,
        tuple: &SymphonyNeighborTuple,
    ) -> Result<ItemPointer, String> {
        self.insert_raw_tuple(tuple.encode()?)
    }

    pub fn read_symphony_neighbor(
        &self,
        tid: ItemPointer,
        centered_code_len: usize,
    ) -> Result<SymphonyNeighborTuple, String> {
        self.get_page(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .read_symphony_neighbor(tid, centered_code_len)
    }

    pub fn read_symphony_neighbor_ref(
        &self,
        tid: ItemPointer,
        centered_code_len: usize,
    ) -> Result<SymphonyNeighborTupleRef<'_>, String> {
        self.get_page(tid.block_number)
            .ok_or_else(|| format!("block {} not found", tid.block_number))?
            .read_symphony_neighbor_ref(tid, centered_code_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::page::FIRST_DATA_BLOCK_NUMBER;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    #[test]
    fn centered_code_len_matches_task25_layout() {
        assert_eq!(centered_code_len(1536), 204);
        assert_eq!(centered_code_len(16), 14);
    }

    #[test]
    fn metadata_roundtrip_preserves_padding_factor() {
        let metadata = MetadataPage::current_v5_symphony(CurrentFormatMetadata {
            m: 16,
            ef_construction: 64,
            entry_point: tid(12, 3),
            dimensions: 1536,
            rabitq_bits: 1,
            max_level: 5,
            seed: 42,
            inserted_since_rebuild: 19,
            padding_factor: 8,
        });

        let encoded = metadata.encode();
        let decoded = MetadataPage::decode(&encoded).unwrap();
        assert_eq!(decoded, metadata);
    }

    #[test]
    fn metadata_page_roundtrip_uses_page_special_space() {
        let metadata = MetadataPage::current_v5_symphony(CurrentFormatMetadata {
            m: 12,
            ef_construction: 48,
            entry_point: tid(7, 1),
            dimensions: 1536,
            rabitq_bits: 1,
            max_level: 4,
            seed: 9,
            inserted_since_rebuild: 3,
            padding_factor: 1,
        });

        let page = metadata.encode_page(DEFAULT_PAGE_SIZE).unwrap();
        let decoded = MetadataPage::decode_page(&page).unwrap();
        assert_eq!(decoded, metadata);
    }

    #[test]
    fn metadata_rejects_invalid_bits_and_padding() {
        let mut encoded = MetadataPage::current_v5_symphony(CurrentFormatMetadata {
            m: 12,
            ef_construction: 48,
            entry_point: tid(7, 1),
            dimensions: 1536,
            rabitq_bits: 1,
            max_level: 4,
            seed: 9,
            inserted_since_rebuild: 3,
            padding_factor: 8,
        })
        .encode();

        encoded[12] = 2;
        assert!(MetadataPage::decode(&encoded).is_err());

        let mut encoded = MetadataPage::current_v5_symphony(CurrentFormatMetadata {
            m: 12,
            ef_construction: 48,
            entry_point: tid(7, 1),
            dimensions: 1536,
            rabitq_bits: 1,
            max_level: 4,
            seed: 9,
            inserted_since_rebuild: 3,
            padding_factor: 8,
        })
        .encode();
        encoded[32] = 0;
        encoded[33] = 0;
        assert!(MetadataPage::decode(&encoded).is_err());
    }

    #[test]
    fn symphony_element_tuple_roundtrip() {
        let tuple = SymphonyElementTuple {
            level: 3,
            deleted: false,
            heaptids: vec![tid(11, 1), tid(11, 2)],
            neighbortid: tid(22, 4),
        };

        let encoded = tuple.encode().unwrap();
        let decoded = SymphonyElementTuple::decode(&encoded).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn symphony_neighbor_tuple_roundtrip() {
        let tuple = SymphonyNeighborTuple {
            count: 2,
            tids: vec![tid(10, 1), tid(11, 2)],
            centered_codes: vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = SymphonyNeighborTuple::decode(&encoded, 4).unwrap();
        assert_eq!(decoded, tuple);
    }

    #[test]
    fn symphony_neighbor_tuple_ref_exposes_parallel_slabs() {
        let tuple = SymphonyNeighborTuple {
            count: 3,
            tids: vec![tid(10, 1), tid(11, 2), tid(12, 3)],
            centered_codes: vec![vec![1, 2], vec![3, 4], vec![5, 6]],
        };

        let encoded = tuple.encode().unwrap();
        let decoded = SymphonyNeighborTupleRef::decode(&encoded, 2).unwrap();

        assert_eq!(decoded.collect_tids(), tuple.tids);
        assert_eq!(decoded.collect_centered_codes(), tuple.centered_codes);
        assert_eq!(decoded.centered_code(1), Some(&[3, 4][..]));
    }

    #[test]
    fn symphony_neighbor_tuple_rejects_length_mismatch() {
        let tuple = SymphonyNeighborTuple {
            count: 2,
            tids: vec![tid(10, 1)],
            centered_codes: vec![vec![1, 2], vec![3, 4]],
        };

        let err = tuple.encode().unwrap_err();
        assert!(err.contains("neighbor count"));
    }

    #[test]
    fn data_page_roundtrip_supports_symphony_tuples() {
        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);

        let element = SymphonyElementTuple {
            level: 1,
            deleted: false,
            heaptids: vec![tid(20, 1)],
            neighbortid: tid(30, 2),
        };
        let neighbor = SymphonyNeighborTuple {
            count: 2,
            tids: vec![tid(30, 3), tid(30, 4)],
            centered_codes: vec![vec![9, 8, 7], vec![6, 5, 4]],
        };

        let element_tid = page.insert_symphony_element(&element).unwrap();
        let neighbor_tid = page.insert_symphony_neighbor(&neighbor).unwrap();

        assert_eq!(page.read_symphony_element(element_tid).unwrap(), element);
        assert_eq!(
            page.read_symphony_neighbor(neighbor_tid, 3).unwrap(),
            neighbor
        );
        assert_eq!(
            page.read_symphony_neighbor_ref(neighbor_tid, 3)
                .unwrap()
                .centered_code(0),
            Some(&[9, 8, 7][..])
        );
    }

    #[test]
    fn default_max_padded_degree_is_positive_for_real_centered_codes() {
        assert!(default_max_padded_degree(1536) > 0);
        assert!(symphony_element_tuple_fits_on_page(DEFAULT_PAGE_SIZE));
    }
}
