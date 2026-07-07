//! FR-076 graph-node record for `ec_distann`.
//!
//! One record per indexed vector, keyed by global `vec_id`, carrying
//! everything one read needs to score and expand the node: the node's own
//! coarse search code, the adjacency list (neighbor vec_ids), and one
//! compressed code per neighbor (scoreable via `QuantCodec::score_ip_batch`
//! with no further read). The record deliberately has NO full-precision
//! vector field (ADR-085 D11): the exact distance of an expanded node comes
//! from its co-placed heap row via `heap_tid`.
//!
//! At fixed `graph_degree` (R) and code stride the encoded size is
//! independent of vector dimension (FR-076-AC-6): only codes and adjacency
//! are stored.

use crate::storage::page::ItemPointer;

/// Record tag for distann graph-node tuples. 0x0A/0x0B are reserved for the
/// FR-083 delta-buffer and FR-080 head-sample tuples of later slices.
pub const DISTANN_NODE_TAG: u8 = 0x09;

/// `flags` bit 0: tombstoned (deleted; retained for traversal continuity
/// within the published epoch per ADR-085 D10, excluded from results).
pub const DISTANN_FLAG_TOMBSTONE: u16 = 1 << 0;

/// tag(1) + reserved(1) + flags(2) + vec_id(8) + heap_tid(6) +
/// neighbor_count(2).
pub const DISTANN_NODE_HEADER_BYTES: usize = 20;

pub const DISTANN_NODE_TAG_OFFSET: usize = 0;
pub const DISTANN_NODE_FLAGS_OFFSET: usize = 2;
pub const DISTANN_NODE_VEC_ID_OFFSET: usize = 4;
pub const DISTANN_NODE_HEAP_TID_OFFSET: usize = 12;
pub const DISTANN_NODE_NEIGHBOR_COUNT_OFFSET: usize = 18;
pub const DISTANN_NODE_SEARCH_CODE_OFFSET: usize = DISTANN_NODE_HEADER_BYTES;

pub const fn distann_node_neighbor_vec_ids_offset(code_len: usize) -> usize {
    DISTANN_NODE_SEARCH_CODE_OFFSET + code_len
}

pub const fn distann_node_neighbor_codes_offset(graph_degree_r: u16, code_len: usize) -> usize {
    distann_node_neighbor_vec_ids_offset(code_len) + graph_degree_r as usize * 8
}

/// FR-076 graph-node record. `search_code` is always exactly the codec
/// stride; `neighbor_vec_ids` and `neighbor_codes` are always sized for the
/// full `graph_degree` R (slots past `neighbor_count` are zero padding), so
/// every record of an index is the same encoded length.
#[derive(Debug, Clone, PartialEq)]
pub struct DistannNodeTuple {
    pub tombstoned: bool,
    pub vec_id: u64,
    pub heap_tid: ItemPointer,
    pub neighbor_count: u16,
    pub search_code: Vec<u8>,
    pub neighbor_vec_ids: Vec<u64>,
    pub neighbor_codes: Vec<u8>,
}

impl DistannNodeTuple {
    pub const fn encoded_len(graph_degree_r: u16, code_len: usize) -> usize {
        let r = graph_degree_r as usize;
        DISTANN_NODE_HEADER_BYTES + code_len + r * 8 + r * code_len
    }

    /// Zero-filled record shaped for pooled `decode_into` reuse.
    pub fn placeholder(graph_degree_r: u16, code_len: usize) -> Self {
        let r = graph_degree_r as usize;
        Self {
            tombstoned: false,
            vec_id: 0,
            heap_tid: ItemPointer::INVALID,
            neighbor_count: 0,
            search_code: vec![0; code_len],
            neighbor_vec_ids: vec![0; r],
            neighbor_codes: vec![0; r * code_len],
        }
    }

    /// Empty record for pooling; `decode_into` sizes the buffers.
    pub fn empty() -> Self {
        Self {
            tombstoned: false,
            vec_id: 0,
            heap_tid: ItemPointer::INVALID,
            neighbor_count: 0,
            search_code: Vec::new(),
            neighbor_vec_ids: Vec::new(),
            neighbor_codes: Vec::new(),
        }
    }

    pub fn is_live(&self) -> bool {
        !self.tombstoned && self.heap_tid != ItemPointer::INVALID
    }

    pub fn validate(&self, graph_degree_r: u16, code_len: usize) -> Result<(), String> {
        let r = graph_degree_r as usize;
        if self.neighbor_count as usize > r {
            return Err(format!(
                "distann node neighbor_count {} exceeds graph_degree {r}",
                self.neighbor_count
            ));
        }
        if self.search_code.len() != code_len {
            return Err(format!(
                "distann node search_code length mismatch: got {}, expected {code_len}",
                self.search_code.len()
            ));
        }
        if self.neighbor_vec_ids.len() != r {
            return Err(format!(
                "distann node neighbor_vec_ids length mismatch: got {}, expected {r}",
                self.neighbor_vec_ids.len()
            ));
        }
        if self.neighbor_codes.len() != r * code_len {
            return Err(format!(
                "distann node neighbor_codes length mismatch: got {}, expected {}",
                self.neighbor_codes.len(),
                r * code_len
            ));
        }
        Ok(())
    }

    pub fn encode(&self, graph_degree_r: u16, code_len: usize) -> Result<Vec<u8>, String> {
        self.validate(graph_degree_r, code_len)?;
        let mut out = Vec::with_capacity(Self::encoded_len(graph_degree_r, code_len));
        out.push(DISTANN_NODE_TAG);
        out.push(0); // reserved
        let mut flags = 0_u16;
        if self.tombstoned {
            flags |= DISTANN_FLAG_TOMBSTONE;
        }
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(&self.vec_id.to_le_bytes());
        self.heap_tid.encode_into(&mut out);
        out.extend_from_slice(&self.neighbor_count.to_le_bytes());
        out.extend_from_slice(&self.search_code);
        for neighbor_vec_id in &self.neighbor_vec_ids {
            out.extend_from_slice(&neighbor_vec_id.to_le_bytes());
        }
        out.extend_from_slice(&self.neighbor_codes);
        debug_assert_eq!(out.len(), Self::encoded_len(graph_degree_r, code_len));
        Ok(out)
    }

    pub fn decode(input: &[u8], graph_degree_r: u16, code_len: usize) -> Result<Self, String> {
        let mut out = Self::empty();
        Self::decode_into(input, graph_degree_r, code_len, &mut out)?;
        Ok(out)
    }

    /// Pooled decode: reuses the output record's buffer capacity.
    pub fn decode_into(
        input: &[u8],
        graph_degree_r: u16,
        code_len: usize,
        out: &mut Self,
    ) -> Result<(), String> {
        let expected = Self::encoded_len(graph_degree_r, code_len);
        if input.len() != expected {
            return Err(format!(
                "distann node record length mismatch: got {}, expected {expected}",
                input.len()
            ));
        }
        if input[DISTANN_NODE_TAG_OFFSET] != DISTANN_NODE_TAG {
            return Err(format!(
                "invalid distann node record tag: got {:#04x}, expected {DISTANN_NODE_TAG:#04x}",
                input[DISTANN_NODE_TAG_OFFSET]
            ));
        }
        let flags = u16::from_le_bytes(
            input[DISTANN_NODE_FLAGS_OFFSET..DISTANN_NODE_FLAGS_OFFSET + 2]
                .try_into()
                .expect("flags bytes"),
        );
        if flags & !DISTANN_FLAG_TOMBSTONE != 0 {
            return Err(format!(
                "invalid distann node record flags: got {flags:#06x}"
            ));
        }
        let r = graph_degree_r as usize;
        let neighbor_count = u16::from_le_bytes(
            input[DISTANN_NODE_NEIGHBOR_COUNT_OFFSET..DISTANN_NODE_NEIGHBOR_COUNT_OFFSET + 2]
                .try_into()
                .expect("neighbor count bytes"),
        );
        if neighbor_count as usize > r {
            return Err(format!(
                "distann node neighbor_count {neighbor_count} exceeds graph_degree {r}"
            ));
        }

        out.tombstoned = flags & DISTANN_FLAG_TOMBSTONE != 0;
        out.vec_id = u64::from_le_bytes(
            input[DISTANN_NODE_VEC_ID_OFFSET..DISTANN_NODE_VEC_ID_OFFSET + 8]
                .try_into()
                .expect("vec_id bytes"),
        );
        out.heap_tid = ItemPointer::decode(
            &input[DISTANN_NODE_HEAP_TID_OFFSET..DISTANN_NODE_HEAP_TID_OFFSET + 6],
        )?;
        out.neighbor_count = neighbor_count;

        out.search_code.clear();
        out.search_code
            .extend_from_slice(&input[DISTANN_NODE_SEARCH_CODE_OFFSET..][..code_len]);

        let vec_ids_offset = distann_node_neighbor_vec_ids_offset(code_len);
        out.neighbor_vec_ids.clear();
        out.neighbor_vec_ids.reserve(r);
        for slot in 0..r {
            out.neighbor_vec_ids.push(u64::from_le_bytes(
                input[vec_ids_offset + slot * 8..vec_ids_offset + slot * 8 + 8]
                    .try_into()
                    .expect("neighbor vec_id bytes"),
            ));
        }

        let codes_offset = distann_node_neighbor_codes_offset(graph_degree_r, code_len);
        out.neighbor_codes.clear();
        out.neighbor_codes
            .extend_from_slice(&input[codes_offset..][..r * code_len]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DistannNodeTuple, DISTANN_FLAG_TOMBSTONE, DISTANN_NODE_HEADER_BYTES};
    use crate::storage::page::ItemPointer;

    const R: u16 = 4;
    const CODE_LEN: usize = 8;

    fn sample() -> DistannNodeTuple {
        DistannNodeTuple {
            tombstoned: false,
            vec_id: 0xDEAD_BEEF_1234_5678,
            heap_tid: ItemPointer {
                block_number: 11,
                offset_number: 3,
            },
            neighbor_count: 3,
            search_code: (0..CODE_LEN as u8).collect(),
            neighbor_vec_ids: vec![101, 202, 303, 0],
            neighbor_codes: (0..(R as usize * CODE_LEN) as u8).collect(),
        }
    }

    #[test]
    fn distann_node_round_trip_is_byte_exact() {
        // FR-076-AC-1: encode -> decode preserves search code, adjacency,
        // and neighbor codes byte-exactly.
        let tuple = sample();
        let encoded = tuple.encode(R, CODE_LEN).expect("encode should succeed");
        assert_eq!(encoded.len(), DistannNodeTuple::encoded_len(R, CODE_LEN));
        let decoded = DistannNodeTuple::decode(&encoded, R, CODE_LEN).expect("decode");
        assert_eq!(decoded, tuple);
        let re_encoded = decoded.encode(R, CODE_LEN).expect("re-encode");
        assert_eq!(re_encoded, encoded);
    }

    #[test]
    fn distann_node_tombstone_flag_round_trips() {
        let mut tuple = sample();
        tuple.tombstoned = true;
        let encoded = tuple.encode(R, CODE_LEN).expect("encode should succeed");
        let flags = u16::from_le_bytes(encoded[2..4].try_into().unwrap());
        assert_eq!(flags & DISTANN_FLAG_TOMBSTONE, DISTANN_FLAG_TOMBSTONE);
        let decoded = DistannNodeTuple::decode(&encoded, R, CODE_LEN).expect("decode");
        assert!(decoded.tombstoned);
        assert!(!decoded.is_live());
    }

    #[test]
    fn distann_node_encoded_len_is_dimension_independent() {
        // FR-076-AC-5/AC-6: no full-precision vector field exists, so the
        // record size is a function of R and code stride only.
        let len = DistannNodeTuple::encoded_len(R, CODE_LEN);
        assert_eq!(
            len,
            DISTANN_NODE_HEADER_BYTES + CODE_LEN + R as usize * 8 + R as usize * CODE_LEN
        );
    }

    #[test]
    fn distann_node_rejects_neighbor_count_above_graph_degree() {
        // FR-076-CON-2.
        let mut tuple = sample();
        tuple.neighbor_count = R + 1;
        assert!(tuple.encode(R, CODE_LEN).is_err());

        let mut encoded = sample().encode(R, CODE_LEN).expect("encode");
        encoded[super::DISTANN_NODE_NEIGHBOR_COUNT_OFFSET] = (R + 1) as u8;
        assert!(DistannNodeTuple::decode(&encoded, R, CODE_LEN).is_err());
    }

    #[test]
    fn distann_node_rejects_wrong_tag_and_unknown_flags() {
        let encoded = sample().encode(R, CODE_LEN).expect("encode");

        let mut bad_tag = encoded.clone();
        bad_tag[0] = 0x06;
        assert!(DistannNodeTuple::decode(&bad_tag, R, CODE_LEN).is_err());

        let mut bad_flags = encoded;
        bad_flags[2] = 0xFF;
        assert!(DistannNodeTuple::decode(&bad_flags, R, CODE_LEN).is_err());
    }

    #[test]
    fn distann_node_decode_into_reuses_pooled_buffers() {
        let encoded = sample().encode(R, CODE_LEN).expect("encode");
        let mut pooled = DistannNodeTuple::placeholder(R, CODE_LEN);
        let search_code_ptr = pooled.search_code.as_ptr();
        DistannNodeTuple::decode_into(&encoded, R, CODE_LEN, &mut pooled)
            .expect("pooled decode should succeed");
        assert_eq!(pooled, sample());
        assert_eq!(
            pooled.search_code.as_ptr(),
            search_code_ptr,
            "pooled decode should not reallocate the search-code buffer"
        );
    }

    #[test]
    fn distann_node_rejects_truncated_input() {
        let encoded = sample().encode(R, CODE_LEN).expect("encode");
        assert!(DistannNodeTuple::decode(&encoded[..encoded.len() - 1], R, CODE_LEN).is_err());
    }
}
