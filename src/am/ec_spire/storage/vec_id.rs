pub(super) const SPIRE_VEC_ID_MAX_BYTES: usize = 32;
pub(super) const SPIRE_LOCAL_VEC_ID_DISCRIMINATOR: u8 = 0x01;
pub(super) const SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR: u8 = 0x02;

pub(super) const SPIRE_ASSIGNMENT_FLAG_PRIMARY: u16 = 0x0001;
pub(super) const SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA: u16 = 0x0002;
pub(super) const SPIRE_ASSIGNMENT_FLAG_TOMBSTONE: u16 = 0x0004;
pub(super) const SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT: u16 = 0x0008;
pub(super) const SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE: u16 = 0x0010;
pub(super) const SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR: u16 = 0x0020;

pub(super) const SPIRE_PAYLOAD_FORMAT_NONE: u8 = 0;
pub(super) const SPIRE_PAYLOAD_FORMAT_TURBOQUANT: u8 = 1;
pub(super) const SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN: u8 = 2;
pub(super) const SPIRE_PAYLOAD_FORMAT_RABITQ: u8 = 3;

const SPIRE_ASSIGNMENT_KNOWN_FLAGS: u16 = SPIRE_ASSIGNMENT_FLAG_PRIMARY
    | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA
    | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
    | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
    | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
    | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;

const PARTITION_OBJECT_MAGIC: u32 = 0x4f50_5345; // "ESPO" as little-endian bytes.
const PARTITION_OBJECT_FORMAT_VERSION_V1: u16 = 1;
const PARTITION_OBJECT_FORMAT_VERSION_V2: u16 = 2;
const PARTITION_OBJECT_HEADER_BYTES: usize = 54;
const ASSIGNMENT_ROW_FIXED_PREFIX_BYTES: usize = 3;
const ASSIGNMENT_ROW_FIXED_TAIL_BYTES: usize = ITEM_POINTER_BYTES + 1 + 4 + 4;
const ROUTING_OBJECT_BODY_PREFIX_BYTES: usize = 4;
const ROUTING_CHILD_ENTRY_FIXED_BYTES: usize = 4 + 8;
const LEAF_V2_META_FLAG: u32 = 0x0000_0001;
const LEAF_V2_SEGMENT_FLAG: u32 = 0x0000_0002;
const LEAF_V2_LOCAL_VEC_ID_STRIDE: usize = 16;
const LEAF_V2_META_BODY_BYTES: usize = 1 + 1 + 2 + 4 + 2 + 2 + 4 + ITEM_POINTER_BYTES + 8;
const LEAF_V2_SEGMENT_PREFIX_BYTES: usize = 4 + 4 + 4 + ITEM_POINTER_BYTES;
const SPIRE_STORE_RELATION_NAME_PREFIX: &str = "ec_spire_store";

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireVecIdKind {
    LocalU64 = 1,
    GlobalBytes = 2,
}

impl SpireVecIdKind {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::LocalU64),
            2 => Ok(Self::GlobalBytes),
            other => Err(format!("ec_spire invalid leaf V2 vec_id kind: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireLeafObjectColumnRowRef<'a> {
    pub(super) row_index: u32,
    pub(super) flags: u16,
    pub(super) vec_id_bytes: &'a [u8],
    pub(super) heap_tid: ItemPointer,
    pub(super) gamma: f32,
    pub(super) encoded_payload: &'a [u8],
}

impl SpireLeafObjectColumnRowRef<'_> {
    pub(super) fn local_vec_seq(&self) -> Result<u64, String> {
        decode_leaf_v2_local_vec_id(self.vec_id_bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireLeafObjectColumns<'a> {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) payload_format: u8,
    pub(super) payload_stride: usize,
    pub(super) vec_id_kind: SpireVecIdKind,
    pub(super) vec_id_stride: usize,
    pub(super) row_base: u32,
    pub(super) flags: &'a [u16],
    pub(super) vec_ids: &'a [u8],
    pub(super) heap_tids: &'a [ItemPointer],
    pub(super) gammas: &'a [f32],
    pub(super) payloads: &'a [u8],
}

impl<'a> SpireLeafObjectColumns<'a> {
    pub(super) fn row_count(&self) -> usize {
        self.flags.len()
    }

    pub(super) fn row(&self, row_offset: usize) -> Result<SpireLeafObjectColumnRowRef<'a>, String> {
        if row_offset >= self.row_count() {
            return Err(format!(
                "ec_spire leaf V2 column row offset {row_offset} exceeds row count {}",
                self.row_count()
            ));
        }
        let row_offset_u32 = u32::try_from(row_offset)
            .map_err(|_| "ec_spire leaf V2 column row offset exceeds u32".to_owned())?;
        let row_index = self
            .row_base
            .checked_add(row_offset_u32)
            .ok_or_else(|| "ec_spire leaf V2 column row index overflow".to_owned())?;
        let vec_id_start = row_offset
            .checked_mul(self.vec_id_stride)
            .ok_or_else(|| "ec_spire leaf V2 column vec_id offset overflow".to_owned())?;
        let vec_id_end = vec_id_start
            .checked_add(self.vec_id_stride)
            .ok_or_else(|| "ec_spire leaf V2 column vec_id end overflow".to_owned())?;
        let payload_start = row_offset
            .checked_mul(self.payload_stride)
            .ok_or_else(|| "ec_spire leaf V2 column payload offset overflow".to_owned())?;
        let payload_end = payload_start
            .checked_add(self.payload_stride)
            .ok_or_else(|| "ec_spire leaf V2 column payload end overflow".to_owned())?;

        Ok(SpireLeafObjectColumnRowRef {
            row_index,
            flags: self.flags[row_offset],
            vec_id_bytes: self
                .vec_ids
                .get(vec_id_start..vec_id_end)
                .ok_or_else(|| "ec_spire leaf V2 column vec_id slice out of bounds".to_owned())?,
            heap_tid: self.heap_tids[row_offset],
            gamma: self.gammas[row_offset],
            encoded_payload: self
                .payloads
                .get(payload_start..payload_end)
                .ok_or_else(|| "ec_spire leaf V2 column payload slice out of bounds".to_owned())?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct SpireVecId {
    bytes: Vec<u8>,
}

impl SpireVecId {
    pub(super) fn local(local_vec_seq: u64) -> Self {
        let mut bytes = Vec::with_capacity(1 + size_of::<u64>());
        bytes.push(SPIRE_LOCAL_VEC_ID_DISCRIMINATOR);
        bytes.extend_from_slice(&local_vec_seq.to_le_bytes());
        Self { bytes }
    }

    pub(super) fn global(global_id: &[u8]) -> Result<Self, String> {
        if global_id.is_empty() {
            return Err("ec_spire global vec_id payload must not be empty".to_owned());
        }
        if global_id.len() + 1 > SPIRE_VEC_ID_MAX_BYTES {
            return Err(format!(
                "ec_spire global vec_id length {} exceeds max payload {}",
                global_id.len(),
                SPIRE_VEC_ID_MAX_BYTES - 1
            ));
        }
        let mut bytes = Vec::with_capacity(global_id.len() + 1);
        bytes.push(SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR);
        bytes.extend_from_slice(global_id);
        Ok(Self { bytes })
    }

    pub(super) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        validate_vec_id_bytes(bytes)?;
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    pub(super) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(super) fn discriminator(&self) -> u8 {
        self.bytes[0]
    }

    pub(super) fn local_sequence(&self) -> Option<u64> {
        if self.discriminator() != SPIRE_LOCAL_VEC_ID_DISCRIMINATOR {
            return None;
        }
        Some(u64::from_le_bytes(
            self.bytes[1..]
                .try_into()
                .expect("local vec_id length is validated"),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SpireVecIdRef<'a> {
    bytes: &'a [u8],
}

impl<'a> SpireVecIdRef<'a> {
    fn from_bytes(bytes: &'a [u8]) -> Result<Self, String> {
        validate_vec_id_bytes(bytes)?;
        Ok(Self { bytes })
    }

    pub(super) fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    pub(super) fn discriminator(&self) -> u8 {
        self.bytes[0]
    }

    pub(super) fn local_sequence(&self) -> Option<u64> {
        if self.discriminator() != SPIRE_LOCAL_VEC_ID_DISCRIMINATOR {
            return None;
        }
        Some(u64::from_le_bytes(
            self.bytes[1..]
                .try_into()
                .expect("local vec_id length is validated"),
        ))
    }

    pub(super) fn to_owned(self) -> SpireVecId {
        SpireVecId {
            bytes: self.bytes.to_vec(),
        }
    }
}
