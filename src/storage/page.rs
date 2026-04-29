//! Cross-AM page-layout primitives.
//!
//! Holds the physical-page constants, [`ItemPointer`], and the raw
//! [`DataPage`] / [`DataPageChain`] page-and-chain machinery shared by every
//! access method. AM-specific tuple codecs (e.g. `TqElementTuple`,
//! `TqNeighborTuple`) live in their owning AM module and extend [`DataPage`]
//! / [`DataPageChain`] through additional inherent `impl` blocks.

pub const DEFAULT_PAGE_SIZE: usize = 8192;
pub const PAGE_HEADER_BYTES: usize = 24;
pub(crate) const LINE_POINTER_BYTES: usize = 4;
pub(crate) const TUPLE_HEADER_BYTES: usize = 4;
pub(crate) const ALIGNMENT_BYTES: usize = 8;

pub const HEAPTID_INLINE_CAPACITY: usize = 10;
pub const ITEM_POINTER_BYTES: usize = 6;
pub const METADATA_BLOCK_NUMBER: u32 = 0;
pub const FIRST_DATA_BLOCK_NUMBER: u32 = 1;

pub(crate) const fn align_up(value: usize, alignment: usize) -> usize {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
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

#[derive(Debug, Clone)]
pub struct DataPage {
    pub(crate) block_number: u32,
    pub(crate) page_size: usize,
    pub(crate) used_bytes: usize,
    pub(crate) tuples: Vec<Vec<u8>>,
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
}

#[derive(Debug, Clone)]
pub struct DataPageChain {
    pub(crate) page_size: usize,
    pub(crate) pages: Vec<DataPage>,
}

impl DataPageChain {
    pub fn new(page_size: usize) -> Self {
        Self {
            page_size,
            pages: vec![DataPage::new(FIRST_DATA_BLOCK_NUMBER, page_size)],
        }
    }

    pub fn page_size(&self) -> usize {
        self.page_size
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

    pub fn start_new_page_if_current_has_tuples(&mut self) {
        if self.pages.last().is_none_or(|page| page.tuple_count() == 0) {
            return;
        }
        let next_block = self
            .pages
            .last()
            .expect("page chain is non-empty")
            .block_number
            + 1;
        self.pages.push(DataPage::new(next_block, self.page_size));
    }

    pub fn append_empty_pages(&mut self, count: usize) -> Option<(u32, u32)> {
        if count == 0 {
            return None;
        }

        let first_block = self
            .pages
            .last()
            .expect("page chain is non-empty")
            .block_number
            + 1;
        for offset in 0..count {
            self.pages
                .push(DataPage::new(first_block + offset as u32, self.page_size));
        }
        let last_block = self
            .pages
            .last()
            .expect("empty pages were appended")
            .block_number;
        Some((first_block, last_block))
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
}

pub(crate) fn usable_page_bytes(page_size: usize) -> usize {
    page_size.saturating_sub(PAGE_HEADER_BYTES)
}

pub(crate) fn element_or_neighbor_tuple_fits(payload_len: usize, page_size: usize) -> bool {
    aligned_tuple_bytes(payload_len) <= usable_page_bytes(page_size)
}

pub(crate) fn aligned_tuple_bytes(payload_len: usize) -> usize {
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
    fn data_page_raw_round_trip() {
        let mut page = DataPage::new(FIRST_DATA_BLOCK_NUMBER, DEFAULT_PAGE_SIZE);
        let payload = vec![0x42; 64];
        let tid = page.insert_raw_tuple(payload.clone()).unwrap();
        assert_eq!(tid.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(tid.offset_number, 1);
        assert_eq!(page.raw_tuple(tid).unwrap(), payload.as_slice());
    }

    #[test]
    fn data_page_chain_overflow_creates_new_block() {
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let big_payload = vec![0x55; 4000];
        let first = chain.insert_raw_tuple(big_payload.clone()).unwrap();
        let second = chain.insert_raw_tuple(big_payload.clone()).unwrap();
        let third = chain.insert_raw_tuple(big_payload).unwrap();
        assert_eq!(first.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(second.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(third.block_number, FIRST_DATA_BLOCK_NUMBER + 1);
        assert_eq!(chain.pages().len(), 2);
    }

    #[test]
    fn data_page_chain_can_start_next_tuple_on_fresh_page() {
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let first = chain.insert_raw_tuple(vec![0; 8]).unwrap();

        chain.start_new_page_if_current_has_tuples();
        chain.start_new_page_if_current_has_tuples();
        let second = chain.insert_raw_tuple(vec![0; 8]).unwrap();

        assert_eq!(first.block_number, FIRST_DATA_BLOCK_NUMBER);
        assert_eq!(second.block_number, FIRST_DATA_BLOCK_NUMBER + 1);
        assert_eq!(second.offset_number, 1);
        assert_eq!(chain.pages().len(), 2);
    }

    #[test]
    fn data_page_chain_appends_empty_pages_contiguously() {
        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let range = chain.append_empty_pages(2).unwrap();

        assert_eq!(
            range,
            (FIRST_DATA_BLOCK_NUMBER + 1, FIRST_DATA_BLOCK_NUMBER + 2)
        );
        assert_eq!(chain.pages().len(), 3);
        assert_eq!(chain.pages()[1].tuple_count(), 0);
        assert_eq!(chain.pages()[2].tuple_count(), 0);
    }
}
