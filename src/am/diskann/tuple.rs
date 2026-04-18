//! Vamana node-tuple layout draft for `tqdiskann`.
//!
//! Phase 1D landing: pins the on-disk shape of a Vamana graph node.
//! Single-level, fixed-capacity `R` neighbor slot list, no per-layer
//! segmentation. Phase 2 (build) actually writes these tuples to
//! index pages; this module only describes the layout and provides
//! roundtrip-safe encode/decode for test coverage.
//!
//! Layout (little-endian, 8-byte aligned payload):
//!
//! ```text
//! [0]  tag: u8              = TQ_VAMANA_NODE_TAG (0x06)
//! [1]  deleted: u8
//! [2]  heaptid_count: u8
//! [3]  reserved: u8         (padding; must be 0)
//! [4]  heaptid_slots: HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES
//!      neighbor_count: u16
//!      graph_degree_r: u16
//!      binary_word_count: u16
//!      search_code_len: u16
//!      rerank_tid: ItemPointer (6 bytes)   -- cold payload chain head
//!      binary_words: [u64; binary_word_count]
//!      search_code: [u8; search_code_len]
//!      neighbor_slots: [ItemPointer; graph_degree_r]
//! ```
//!
//! Empty neighbor slots carry `ItemPointer::INVALID`. Neighbor slot
//! count is always exactly `graph_degree_r` at rest; `neighbor_count`
//! is the filled prefix (ADR-042 fill-only invariant: no
//! live-neighbor eviction, so the prefix only grows until VACUUM
//! unlinks dead TIDs).

use crate::am::page::{ItemPointer, HEAPTID_INLINE_CAPACITY, ITEM_POINTER_BYTES};

/// Tuple-type tag for a Vamana graph node. Distinct from the
/// `tqhnsw` element/neighbor/grouped-hot/rerank tags (0x01-0x05) so
/// shared page walkers can dispatch on tag byte without ambiguity.
pub const TQ_VAMANA_NODE_TAG: u8 = 0x06;

const HEADER_FIXED_BYTES: usize = 1 // tag
    + 1 // deleted
    + 1 // heaptid_count
    + 1 // reserved
    + HEAPTID_INLINE_CAPACITY * ITEM_POINTER_BYTES
    + 2 // neighbor_count
    + 2 // graph_degree_r
    + 2 // binary_word_count
    + 2 // search_code_len
    + ITEM_POINTER_BYTES; // rerank_tid

#[derive(Debug, Clone, PartialEq)]
pub struct VamanaNodeTuple {
    pub deleted: bool,
    pub heaptids: Vec<ItemPointer>,
    pub graph_degree_r: u16,
    pub rerank_tid: ItemPointer,
    pub binary_words: Vec<u64>,
    pub search_code: Vec<u8>,
    /// Exactly `graph_degree_r` slots. Filled prefix is
    /// `neighbor_count` (below); tail slots carry
    /// `ItemPointer::INVALID`.
    pub neighbors: Vec<ItemPointer>,
    pub neighbor_count: u16,
}

impl VamanaNodeTuple {
    pub fn encoded_len(
        graph_degree_r: u16,
        binary_word_count: usize,
        search_code_len: usize,
    ) -> usize {
        HEADER_FIXED_BYTES
            + binary_word_count * 8
            + search_code_len
            + (graph_degree_r as usize) * ITEM_POINTER_BYTES
    }

    pub fn new_empty(graph_degree_r: u16) -> Self {
        Self {
            deleted: false,
            heaptids: Vec::new(),
            graph_degree_r,
            rerank_tid: ItemPointer::INVALID,
            binary_words: Vec::new(),
            search_code: Vec::new(),
            neighbors: vec![ItemPointer::INVALID; graph_degree_r as usize],
            neighbor_count: 0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.heaptids.len() > HEAPTID_INLINE_CAPACITY {
            return Err(format!(
                "heaptid list too long: {}, max {HEAPTID_INLINE_CAPACITY}",
                self.heaptids.len()
            ));
        }
        if self.neighbors.len() != self.graph_degree_r as usize {
            return Err(format!(
                "neighbor slot count mismatch: got {}, expected {}",
                self.neighbors.len(),
                self.graph_degree_r
            ));
        }
        if (self.neighbor_count as usize) > self.neighbors.len() {
            return Err(format!(
                "neighbor_count {} exceeds capacity {}",
                self.neighbor_count,
                self.neighbors.len()
            ));
        }
        if self.binary_words.len() > u16::MAX as usize {
            return Err(format!(
                "binary_words too long: {}, max {}",
                self.binary_words.len(),
                u16::MAX
            ));
        }
        if self.search_code.len() > u16::MAX as usize {
            return Err(format!(
                "search_code too long: {}, max {}",
                self.search_code.len(),
                u16::MAX
            ));
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut out = Vec::with_capacity(Self::encoded_len(
            self.graph_degree_r,
            self.binary_words.len(),
            self.search_code.len(),
        ));

        out.push(TQ_VAMANA_NODE_TAG);
        out.push(self.deleted as u8);
        out.push(self.heaptids.len() as u8);
        out.push(0); // reserved

        for slot in 0..HEAPTID_INLINE_CAPACITY {
            if let Some(tid) = self.heaptids.get(slot) {
                tid.encode_into(&mut out);
            } else {
                ItemPointer::INVALID.encode_into(&mut out);
            }
        }

        out.extend_from_slice(&self.neighbor_count.to_le_bytes());
        out.extend_from_slice(&self.graph_degree_r.to_le_bytes());
        out.extend_from_slice(&(self.binary_words.len() as u16).to_le_bytes());
        out.extend_from_slice(&(self.search_code.len() as u16).to_le_bytes());
        self.rerank_tid.encode_into(&mut out);

        for word in &self.binary_words {
            out.extend_from_slice(&word.to_le_bytes());
        }
        out.extend_from_slice(&self.search_code);

        for slot in &self.neighbors {
            slot.encode_into(&mut out);
        }

        Ok(out)
    }

    pub fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() < HEADER_FIXED_BYTES {
            return Err(format!(
                "vamana node tuple too short: got {}, need at least {HEADER_FIXED_BYTES}",
                input.len()
            ));
        }
        if input[0] != TQ_VAMANA_NODE_TAG {
            return Err(format!(
                "invalid vamana node tuple tag: got 0x{:02x}, expected 0x{:02x}",
                input[0], TQ_VAMANA_NODE_TAG
            ));
        }

        let deleted = input[1] != 0;
        let heaptid_count = input[2] as usize;
        if heaptid_count > HEAPTID_INLINE_CAPACITY {
            return Err(format!(
                "invalid heap tid count: got {heaptid_count}, max {HEAPTID_INLINE_CAPACITY}"
            ));
        }
        if input[3] != 0 {
            return Err(format!(
                "vamana node reserved byte must be 0, got {}",
                input[3]
            ));
        }

        let mut cursor = 4_usize;
        let mut heaptids = Vec::with_capacity(heaptid_count);
        for slot in 0..HEAPTID_INLINE_CAPACITY {
            let tid = ItemPointer::decode(&input[cursor..cursor + ITEM_POINTER_BYTES])?;
            if slot < heaptid_count {
                heaptids.push(tid);
            }
            cursor += ITEM_POINTER_BYTES;
        }

        let neighbor_count =
            u16::from_le_bytes(input[cursor..cursor + 2].try_into().expect("n_count"));
        cursor += 2;
        let graph_degree_r =
            u16::from_le_bytes(input[cursor..cursor + 2].try_into().expect("R bytes"));
        cursor += 2;
        let binary_word_count =
            u16::from_le_bytes(input[cursor..cursor + 2].try_into().expect("bw count")) as usize;
        cursor += 2;
        let search_code_len =
            u16::from_le_bytes(input[cursor..cursor + 2].try_into().expect("code len")) as usize;
        cursor += 2;
        let rerank_tid = ItemPointer::decode(&input[cursor..cursor + ITEM_POINTER_BYTES])?;
        cursor += ITEM_POINTER_BYTES;

        let expected_len =
            HEADER_FIXED_BYTES + binary_word_count * 8 + search_code_len + (graph_degree_r as usize) * ITEM_POINTER_BYTES;
        if input.len() < expected_len {
            return Err(format!(
                "vamana node tuple truncated: got {}, expected {expected_len}",
                input.len()
            ));
        }
        if (neighbor_count as usize) > (graph_degree_r as usize) {
            return Err(format!(
                "neighbor_count {neighbor_count} exceeds graph_degree_r {graph_degree_r}"
            ));
        }

        let mut binary_words = Vec::with_capacity(binary_word_count);
        for _ in 0..binary_word_count {
            binary_words.push(u64::from_le_bytes(
                input[cursor..cursor + 8].try_into().expect("word bytes"),
            ));
            cursor += 8;
        }

        let search_code = input[cursor..cursor + search_code_len].to_vec();
        cursor += search_code_len;

        let mut neighbors = Vec::with_capacity(graph_degree_r as usize);
        for _ in 0..(graph_degree_r as usize) {
            neighbors.push(ItemPointer::decode(
                &input[cursor..cursor + ITEM_POINTER_BYTES],
            )?);
            cursor += ITEM_POINTER_BYTES;
        }

        Ok(Self {
            deleted,
            heaptids,
            graph_degree_r,
            rerank_tid,
            binary_words,
            search_code,
            neighbors,
            neighbor_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tid(b: u32, o: u16) -> ItemPointer {
        ItemPointer {
            block_number: b,
            offset_number: o,
        }
    }

    // LA-010: empty node tuple with R=32 round-trips.
    #[test]
    fn la_010_empty_node_roundtrip() {
        let tuple = VamanaNodeTuple::new_empty(32);
        let encoded = tuple.encode().expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded).expect("decode");
        assert_eq!(tuple, decoded);
    }

    // LA-011: filled node with heaptids, neighbors, binary sidecar, code.
    #[test]
    fn la_011_filled_node_roundtrip() {
        let mut tuple = VamanaNodeTuple::new_empty(8);
        tuple.heaptids = vec![make_tid(10, 1), make_tid(10, 2)];
        tuple.rerank_tid = make_tid(42, 3);
        tuple.binary_words = vec![0xdeadbeef_cafebabe, 0x0123456789abcdef];
        tuple.search_code = vec![1, 2, 3, 4, 5, 6, 7, 8];
        tuple.neighbor_count = 3;
        tuple.neighbors[0] = make_tid(100, 1);
        tuple.neighbors[1] = make_tid(100, 2);
        tuple.neighbors[2] = make_tid(100, 3);
        let encoded = tuple.encode().expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded).expect("decode");
        assert_eq!(tuple, decoded);
    }

    // LA-012: encoded length matches the size computation.
    #[test]
    fn la_012_encoded_len_matches_computation() {
        let mut tuple = VamanaNodeTuple::new_empty(32);
        tuple.binary_words = vec![0; 4];
        tuple.search_code = vec![0; 24];
        let encoded = tuple.encode().expect("encode");
        assert_eq!(
            encoded.len(),
            VamanaNodeTuple::encoded_len(32, 4, 24)
        );
    }

    // LA-013: foreign tag byte is rejected.
    #[test]
    fn la_013_foreign_tag_rejected() {
        let tuple = VamanaNodeTuple::new_empty(4);
        let mut encoded = tuple.encode().expect("encode");
        encoded[0] = 0x01; // TQ_ELEMENT_TAG from tqhnsw
        let err = VamanaNodeTuple::decode(&encoded).expect_err("decode should fail");
        assert!(err.contains("invalid vamana node tuple tag"), "got: {err}");
    }

    // LA-014: validate rejects a neighbors Vec whose length != R.
    #[test]
    fn la_014_validate_neighbor_capacity() {
        let mut tuple = VamanaNodeTuple::new_empty(8);
        tuple.neighbors.pop();
        let err = tuple.validate().expect_err("validate should fail");
        assert!(err.contains("neighbor slot count mismatch"), "got: {err}");
    }

    // LA-015: validate rejects neighbor_count > R.
    #[test]
    fn la_015_validate_neighbor_count_bounded_by_r() {
        let mut tuple = VamanaNodeTuple::new_empty(4);
        tuple.neighbor_count = 5;
        let err = tuple.validate().expect_err("validate should fail");
        assert!(err.contains("exceeds capacity"), "got: {err}");
    }

    // LA-016: empty neighbor slots decode as ItemPointer::INVALID.
    #[test]
    fn la_016_empty_slots_are_invalid_pointer() {
        let mut tuple = VamanaNodeTuple::new_empty(4);
        tuple.neighbor_count = 1;
        tuple.neighbors[0] = make_tid(7, 1);
        let encoded = tuple.encode().expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded).expect("decode");
        assert_eq!(decoded.neighbors[0], make_tid(7, 1));
        for slot in &decoded.neighbors[1..] {
            assert_eq!(*slot, ItemPointer::INVALID);
        }
    }

    // LA-017: deleted flag survives roundtrip.
    #[test]
    fn la_017_deleted_flag_roundtrips() {
        let mut tuple = VamanaNodeTuple::new_empty(4);
        tuple.deleted = true;
        let encoded = tuple.encode().expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded).expect("decode");
        assert!(decoded.deleted);
    }
}
