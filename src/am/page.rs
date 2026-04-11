//! Page-layout primitives for `tqhnsw`.

const DEFAULT_PAGE_SIZE: usize = 8192;
pub const PAGE_HEADER_BYTES: usize = 24;
const LINE_POINTER_BYTES: usize = 4;
const TUPLE_HEADER_BYTES: usize = 4;
const ALIGNMENT_BYTES: usize = 8;

pub const TQ_ELEMENT_TAG: u8 = 0x01;
pub const TQ_NEIGHBOR_TAG: u8 = 0x02;
pub const HEAPTID_INLINE_CAPACITY: usize = 10;
pub const ITEM_POINTER_BYTES: usize = 6;
pub const METADATA_BLOCK_NUMBER: u32 = 0;
pub const FIRST_DATA_BLOCK_NUMBER: u32 = 1;
const METADATA_BYTES: usize = 2 + 2 + ITEM_POINTER_BYTES + 2 + 1 + 1 + 8 + 8;

const fn align_up(value: usize, alignment: usize) -> usize {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

const METADATA_SPECIAL_BYTES: usize = align_up(METADATA_BYTES, ALIGNMENT_BYTES);

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
}

impl MetadataPage {
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
        out
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != METADATA_BYTES {
            return Err(format!(
                "metadata page length mismatch: got {}, expected {METADATA_BYTES}",
                input.len()
            ));
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

#[derive(Debug, Clone, PartialEq)]
pub struct TqElementTuple {
    pub level: u8,
    pub deleted: bool,
    pub heaptids: Vec<ItemPointer>,
    pub gamma: f32,
    pub neighbortid: ItemPointer,
    pub code: Vec<u8>,
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
                + self.code.len(),
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
        Ok(out)
    }

    pub fn decode(input: &[u8], code_len: usize) -> Result<Self, String> {
        let expected_len = 1
            + 1
            + 1
            + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES
            + 1
            + 4
            + ITEM_POINTER_BYTES
            + code_len;
        if input.len() != expected_len {
            return Err(format!(
                "element tuple length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if input[0] != TQ_ELEMENT_TAG {
            return Err(format!("invalid element tuple tag: {}", input[0]));
        }

        let mut cursor = 3;
        let mut heaptids = Vec::with_capacity(HEAPTID_INLINE_CAPACITY);
        for _ in 0..HEAPTID_INLINE_CAPACITY {
            let tid = ItemPointer::decode(&input[cursor..cursor + ITEM_POINTER_BYTES])?;
            heaptids.push(tid);
            cursor += ITEM_POINTER_BYTES;
        }

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

        Ok(Self {
            level: input[1],
            deleted: input[2] != 0,
            heaptids: heaptids.into_iter().take(heaptid_count).collect(),
            gamma,
            neighbortid,
            code: input[cursor..].to_vec(),
        })
    }

    pub fn encoded_len(code_len: usize) -> usize {
        1 + 1
            + 1
            + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES
            + 1
            + 4
            + ITEM_POINTER_BYTES
            + code_len
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
            self.pages.iter().enumerate().all(|(i, page)| {
                page.block_number == FIRST_DATA_BLOCK_NUMBER + i as u32
            }),
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
        let metadata = MetadataPage {
            m: 8,
            ef_construction: 64,
            entry_point: tid(12, 3),
            dimensions: 1536,
            bits: 4,
            max_level: 5,
            seed: 42,
            inserted_since_rebuild: 7,
        };
        let encoded = metadata.encode();
        let decoded = MetadataPage::decode(&encoded).unwrap();
        assert_eq!(decoded, metadata);
    }

    #[test]
    fn metadata_page_roundtrip() {
        let metadata = MetadataPage {
            m: 8,
            ef_construction: 64,
            entry_point: tid(12, 3),
            dimensions: 1536,
            bits: 4,
            max_level: 5,
            seed: 42,
            inserted_since_rebuild: 7,
        };

        let page = metadata.encode_page(DEFAULT_PAGE_SIZE).unwrap();
        let decoded = MetadataPage::decode_page(&page).unwrap();
        assert_eq!(decoded, metadata);
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
        };

        let encoded = tuple.encode().unwrap();
        let decoded = TqElementTuple::decode(&encoded, 32).unwrap();
        assert_eq!(decoded, tuple);
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
        };

        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);
        let tuple_tid = page.insert_element(&tuple).unwrap();
        let decoded = page.read_element(tuple_tid, 32).unwrap();
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
        };
        let encoded = tuple.encode().unwrap();
        let decoded = TqElementTuple::decode(&encoded, 16).unwrap();
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
        let meta = MetadataPage {
            m: 8,
            ef_construction: 64,
            entry_point: tid(1, 1),
            dimensions: 32,
            bits: 4,
            max_level: 3,
            seed: 42,
            inserted_since_rebuild: 5,
        };
        let encoded = meta.encode();
        let decoded = MetadataPage::decode(&encoded).unwrap();
        assert_eq!(decoded, meta);
    }
}
