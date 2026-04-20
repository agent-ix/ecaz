//! Vamana node-tuple slim layout for `ec_diskann` (ADR-045).
//!
//! Phase 5B landing: replaces the original draft with the
//! ADR-045 reference layout. Compared with the draft, this layout:
//!
//! 1. Drops the 60-byte inline `HEAPTID_INLINE_CAPACITY` array — Vamana
//!    is one-node-per-heap-row; the rare multi-version case routes
//!    through an overflow chain (`has_overflow_heaptids` flag, follow-on
//!    chain landed with phase 7 insert).
//! 2. Moves `graph_degree_r`, `binary_word_count`, and `search_code_len`
//!    out of the per-tuple header — they're index-constants and live on
//!    the [`crate::am::ec_diskann::page::VamanaMetadataPage`]. Decoder
//!    threads them in (mirrors `tqhnsw`'s `read_element(tid, code_len)`).
//! 3. Reserves `rerank_tid` unconditionally even though ADR-044's
//!    current default ("rerank from heap via ecvector EXTERNAL") leaves
//!    it `INVALID`. Reserving the slot in V1 means ADR-044's eventual
//!    C1 option (index-side cold-page rerank payload) lands later
//!    without a wire break.
//!
//! Layout (little-endian, 8-byte aligned payload):
//!
//! ```text
//! [0]  tag: u8                          = TQ_VAMANA_NODE_TAG (0x06)
//! [1]  flags: u8                        (bit 0 = deleted, bit 1 = has_overflow_heaptids)
//! [2]  neighbor_count: u16              (filled prefix of neighbor_slots)
//! [4]  primary_heaptid: ItemPointer     (6)
//! [10] rerank_tid: ItemPointer          (6)
//! [16] binary_words:   [u64; W]              -- W from metadata (sidecar width)
//!      search_code:    [u8;  C]              -- C from metadata (grouped-PQ4 length)
//!      neighbor_slots: [ItemPointer; R]      -- R from metadata; tail = INVALID
//! ```
//!
//! Encoded length is **fixed per (R, W, C)** — every node tuple in a
//! given index encodes to the same byte length. This is what makes the
//! ADR-045 placeholder-then-patch persistence pattern possible:
//! `DataPageChain::update_raw_tuple` requires the patched payload to
//! match the placeholder's byte length, and that holds trivially when
//! length is a function of metadata-only inputs.

use crate::storage::page::{ItemPointer, ITEM_POINTER_BYTES};

/// Tuple-type tag for a Vamana graph node. Distinct from the
/// `tqhnsw` element/neighbor/grouped-hot/rerank/turbo-hot tags
/// (0x01-0x05) so shared page walkers can dispatch on tag byte
/// without ambiguity.
pub const TQ_VAMANA_NODE_TAG: u8 = 0x06;

/// Bit positions inside the `flags` byte at offset 1.
pub const FLAG_DELETED: u8 = 1 << 0;
pub const FLAG_HAS_OVERFLOW_HEAPTIDS: u8 = 1 << 1;

/// Fixed header bytes for every Vamana node tuple, regardless of
/// (R, W, C). Sum of: tag(1) + flags(1) + neighbor_count(2) +
/// primary_heaptid(6) + rerank_tid(6) = 16.
pub const HEADER_FIXED_BYTES: usize = 1 + 1 + 2 + ITEM_POINTER_BYTES + ITEM_POINTER_BYTES;

#[derive(Debug, Clone, PartialEq)]
pub struct VamanaNodeTuple {
    pub deleted: bool,
    pub has_overflow_heaptids: bool,
    pub primary_heaptid: ItemPointer,
    pub rerank_tid: ItemPointer,
    /// Length is fixed at `binary_word_count` (W) from metadata. Empty when
    /// `PAYLOAD_FLAG_BINARY_SIDECAR` is off (W = 0).
    pub binary_words: Vec<u64>,
    /// Length is fixed at `search_code_len` (C) from metadata.
    pub search_code: Vec<u8>,
    /// Length is fixed at `graph_degree_r` (R) from metadata.
    /// Filled prefix is `neighbor_count`; tail slots carry
    /// `ItemPointer::INVALID` (ADR-047 fill-only invariant).
    pub neighbors: Vec<ItemPointer>,
    pub neighbor_count: u16,
}

impl VamanaNodeTuple {
    /// Encoded length for the slim layout at the given
    /// `(graph_degree_r, binary_word_count, search_code_len)` triple.
    /// Constant for every tuple in a given index — see ADR-045
    /// Decision 3.
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

    /// Build an empty placeholder tuple with the right body sizes.
    /// All neighbor slots are `INVALID`, both heaptid slots are
    /// `INVALID`, body byte arrays are zero-filled. Used by
    /// persistence pass 1 (placeholder-then-patch, ADR-045 Decision 5).
    pub fn placeholder(
        graph_degree_r: u16,
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Self {
        Self {
            deleted: false,
            has_overflow_heaptids: false,
            primary_heaptid: ItemPointer::INVALID,
            rerank_tid: ItemPointer::INVALID,
            binary_words: vec![0; binary_word_count],
            search_code: vec![0; search_code_len],
            neighbors: vec![ItemPointer::INVALID; graph_degree_r as usize],
            neighbor_count: 0,
        }
    }

    /// Verify the tuple matches the index-constant `(R, W, C)` triple.
    /// All three must be supplied by the caller from the metadata page;
    /// they are not stored per-tuple.
    pub fn validate(
        &self,
        graph_degree_r: u16,
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Result<(), String> {
        if self.binary_words.len() != binary_word_count {
            return Err(format!(
                "binary_words length mismatch: got {}, expected {binary_word_count}",
                self.binary_words.len()
            ));
        }
        if self.search_code.len() != search_code_len {
            return Err(format!(
                "search_code length mismatch: got {}, expected {search_code_len}",
                self.search_code.len()
            ));
        }
        if self.neighbors.len() != graph_degree_r as usize {
            return Err(format!(
                "neighbor slot count mismatch: got {}, expected {graph_degree_r}",
                self.neighbors.len()
            ));
        }
        if (self.neighbor_count as usize) > self.neighbors.len() {
            return Err(format!(
                "neighbor_count {} exceeds capacity {}",
                self.neighbor_count,
                self.neighbors.len()
            ));
        }
        Ok(())
    }

    pub fn encode(
        &self,
        graph_degree_r: u16,
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Result<Vec<u8>, String> {
        self.validate(graph_degree_r, binary_word_count, search_code_len)?;

        let mut out = Vec::with_capacity(Self::encoded_len(
            graph_degree_r,
            binary_word_count,
            search_code_len,
        ));

        out.push(TQ_VAMANA_NODE_TAG);
        let mut flags: u8 = 0;
        if self.deleted {
            flags |= FLAG_DELETED;
        }
        if self.has_overflow_heaptids {
            flags |= FLAG_HAS_OVERFLOW_HEAPTIDS;
        }
        out.push(flags);
        out.extend_from_slice(&self.neighbor_count.to_le_bytes());
        self.primary_heaptid.encode_into(&mut out);
        self.rerank_tid.encode_into(&mut out);

        for word in &self.binary_words {
            out.extend_from_slice(&word.to_le_bytes());
        }
        out.extend_from_slice(&self.search_code);
        for slot in &self.neighbors {
            slot.encode_into(&mut out);
        }

        debug_assert_eq!(
            out.len(),
            Self::encoded_len(graph_degree_r, binary_word_count, search_code_len)
        );
        Ok(out)
    }

    pub fn decode(
        input: &[u8],
        graph_degree_r: u16,
        binary_word_count: usize,
        search_code_len: usize,
    ) -> Result<Self, String> {
        let expected_len =
            Self::encoded_len(graph_degree_r, binary_word_count, search_code_len);
        if input.len() != expected_len {
            return Err(format!(
                "vamana node tuple length mismatch: got {}, expected {expected_len}"
                ,
                input.len()
            ));
        }
        if input[0] != TQ_VAMANA_NODE_TAG {
            return Err(format!(
                "invalid vamana node tuple tag: got 0x{:02x}, expected 0x{:02x}",
                input[0], TQ_VAMANA_NODE_TAG
            ));
        }

        let flags = input[1];
        let deleted = (flags & FLAG_DELETED) != 0;
        let has_overflow_heaptids = (flags & FLAG_HAS_OVERFLOW_HEAPTIDS) != 0;
        let neighbor_count = u16::from_le_bytes(input[2..4].try_into().expect("nc bytes"));
        let primary_heaptid = ItemPointer::decode(&input[4..10])?;
        let rerank_tid = ItemPointer::decode(&input[10..16])?;

        if (neighbor_count as usize) > (graph_degree_r as usize) {
            return Err(format!(
                "neighbor_count {neighbor_count} exceeds graph_degree_r {graph_degree_r}"
            ));
        }

        let mut cursor = HEADER_FIXED_BYTES;

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
            has_overflow_heaptids,
            primary_heaptid,
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

    // LA-010: empty placeholder tuple round-trips.
    #[test]
    fn la_010_empty_node_roundtrip() {
        let tuple = VamanaNodeTuple::placeholder(32, 24, 192);
        let encoded = tuple.encode(32, 24, 192).expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded, 32, 24, 192).expect("decode");
        assert_eq!(tuple, decoded);
    }

    // LA-011: filled node with primary_heaptid, rerank_tid, neighbors,
    // binary sidecar, code round-trips.
    #[test]
    fn la_011_filled_node_roundtrip() {
        let mut tuple = VamanaNodeTuple::placeholder(8, 2, 8);
        tuple.primary_heaptid = make_tid(10, 1);
        tuple.rerank_tid = make_tid(42, 3);
        tuple.binary_words = vec![0xdeadbeef_cafebabe, 0x0123456789abcdef];
        tuple.search_code = vec![1, 2, 3, 4, 5, 6, 7, 8];
        tuple.neighbor_count = 3;
        tuple.neighbors[0] = make_tid(100, 1);
        tuple.neighbors[1] = make_tid(100, 2);
        tuple.neighbors[2] = make_tid(100, 3);
        let encoded = tuple.encode(8, 2, 8).expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded, 8, 2, 8).expect("decode");
        assert_eq!(tuple, decoded);
    }

    // LA-012: encoded length matches the size computation.
    #[test]
    fn la_012_encoded_len_matches_computation() {
        let tuple = VamanaNodeTuple::placeholder(32, 4, 24);
        let encoded = tuple.encode(32, 4, 24).expect("encode");
        assert_eq!(encoded.len(), VamanaNodeTuple::encoded_len(32, 4, 24));
    }

    // LA-013: foreign tag byte is rejected.
    #[test]
    fn la_013_foreign_tag_rejected() {
        let tuple = VamanaNodeTuple::placeholder(4, 0, 0);
        let mut encoded = tuple.encode(4, 0, 0).expect("encode");
        encoded[0] = 0x01; // TQ_ELEMENT_TAG from tqhnsw
        let err = VamanaNodeTuple::decode(&encoded, 4, 0, 0).expect_err("decode should fail");
        assert!(err.contains("invalid vamana node tuple tag"), "got: {err}");
    }

    // LA-014: validate rejects a neighbors Vec whose length != R.
    #[test]
    fn la_014_validate_neighbor_capacity() {
        let mut tuple = VamanaNodeTuple::placeholder(8, 0, 0);
        tuple.neighbors.pop();
        let err = tuple.validate(8, 0, 0).expect_err("validate should fail");
        assert!(err.contains("neighbor slot count mismatch"), "got: {err}");
    }

    // LA-015: validate rejects neighbor_count > R.
    #[test]
    fn la_015_validate_neighbor_count_bounded_by_r() {
        let mut tuple = VamanaNodeTuple::placeholder(4, 0, 0);
        tuple.neighbor_count = 5;
        let err = tuple.validate(4, 0, 0).expect_err("validate should fail");
        assert!(err.contains("exceeds capacity"), "got: {err}");
    }

    // LA-016: empty neighbor slots decode as ItemPointer::INVALID.
    #[test]
    fn la_016_empty_slots_are_invalid_pointer() {
        let mut tuple = VamanaNodeTuple::placeholder(4, 0, 0);
        tuple.neighbor_count = 1;
        tuple.neighbors[0] = make_tid(7, 1);
        let encoded = tuple.encode(4, 0, 0).expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded, 4, 0, 0).expect("decode");
        assert_eq!(decoded.neighbors[0], make_tid(7, 1));
        for slot in &decoded.neighbors[1..] {
            assert_eq!(*slot, ItemPointer::INVALID);
        }
    }

    // LA-017: deleted flag survives roundtrip.
    #[test]
    fn la_017_deleted_flag_roundtrips() {
        let mut tuple = VamanaNodeTuple::placeholder(4, 0, 0);
        tuple.deleted = true;
        let encoded = tuple.encode(4, 0, 0).expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded, 4, 0, 0).expect("decode");
        assert!(decoded.deleted);
        assert!(!decoded.has_overflow_heaptids);
    }

    // LA-018: has_overflow_heaptids flag is independent of `deleted`.
    #[test]
    fn la_018_overflow_heaptids_flag_independent() {
        let mut tuple = VamanaNodeTuple::placeholder(4, 0, 0);
        tuple.has_overflow_heaptids = true;
        let encoded = tuple.encode(4, 0, 0).expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded, 4, 0, 0).expect("decode");
        assert!(decoded.has_overflow_heaptids);
        assert!(!decoded.deleted);

        tuple.deleted = true;
        let encoded = tuple.encode(4, 0, 0).expect("encode");
        let decoded = VamanaNodeTuple::decode(&encoded, 4, 0, 0).expect("decode");
        assert!(decoded.has_overflow_heaptids);
        assert!(decoded.deleted);
    }

    // LA-019: ADR-045 Decision 3 — encoded length is a pure function of
    // (R, W, C); two tuples with the same triple but different content
    // (flags, primary_heaptid, neighbor_count, neighbor slot fill) must
    // encode to the same byte length. This is what makes the
    // placeholder-then-patch persistence pattern possible.
    #[test]
    fn la_019_fixed_length_invariant() {
        let r = 32u16;
        let w = 24usize;
        let c = 48usize;

        let placeholder = VamanaNodeTuple::placeholder(r, w, c);
        let placeholder_bytes = placeholder.encode(r, w, c).expect("encode");

        let mut filled = VamanaNodeTuple::placeholder(r, w, c);
        filled.deleted = true;
        filled.has_overflow_heaptids = true;
        filled.primary_heaptid = make_tid(123, 4);
        filled.rerank_tid = make_tid(987, 6);
        filled.binary_words = (0..w as u64).collect();
        filled.search_code = (0..c).map(|i| (i & 0xff) as u8).collect();
        filled.neighbor_count = r;
        for (i, slot) in filled.neighbors.iter_mut().enumerate() {
            *slot = make_tid(i as u32, 1);
        }
        let filled_bytes = filled.encode(r, w, c).expect("encode");

        assert_eq!(
            placeholder_bytes.len(),
            filled_bytes.len(),
            "placeholder and filled tuples must encode to the same length \
             for identical (R, W, C) — required by ADR-045 Decision 5"
        );
        assert_eq!(
            placeholder_bytes.len(),
            VamanaNodeTuple::encoded_len(r, w, c),
            "encoded length must match encoded_len()"
        );
    }

    // LA-020: header is exactly 16 bytes (ADR-045 reference layout).
    #[test]
    fn la_020_header_size_locked() {
        assert_eq!(HEADER_FIXED_BYTES, 16);
    }

    // LA-021: length-mismatch on decode reports the right number.
    #[test]
    fn la_021_decode_rejects_wrong_length() {
        let tuple = VamanaNodeTuple::placeholder(4, 1, 2);
        let mut encoded = tuple.encode(4, 1, 2).expect("encode");
        encoded.push(0); // off by one
        let err = VamanaNodeTuple::decode(&encoded, 4, 1, 2).expect_err("decode should fail");
        assert!(err.contains("length mismatch"), "got: {err}");
    }

    // LA-022: validate rejects body sizes that don't match metadata.
    #[test]
    fn la_022_validate_body_sizes_against_metadata() {
        let mut tuple = VamanaNodeTuple::placeholder(4, 2, 8);
        tuple.binary_words.pop();
        let err = tuple.validate(4, 2, 8).expect_err("validate should fail");
        assert!(err.contains("binary_words length mismatch"), "got: {err}");

        let mut tuple = VamanaNodeTuple::placeholder(4, 2, 8);
        tuple.search_code.pop();
        let err = tuple.validate(4, 2, 8).expect_err("validate should fail");
        assert!(err.contains("search_code length mismatch"), "got: {err}");
    }
}
