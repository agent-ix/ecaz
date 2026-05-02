//! PID-addressed partition-object storage codecs.

use std::mem::size_of;

use crate::storage::page::{ItemPointer, ITEM_POINTER_BYTES};

pub(super) const SPIRE_VEC_ID_MAX_BYTES: usize = 32;
pub(super) const SPIRE_LOCAL_VEC_ID_DISCRIMINATOR: u8 = 0x01;
pub(super) const SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR: u8 = 0x02;

pub(super) const SPIRE_ASSIGNMENT_FLAG_PRIMARY: u16 = 0x0001;
pub(super) const SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA: u16 = 0x0002;
pub(super) const SPIRE_ASSIGNMENT_FLAG_TOMBSTONE: u16 = 0x0004;
pub(super) const SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT: u16 = 0x0008;
pub(super) const SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE: u16 = 0x0010;
pub(super) const SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR: u16 = 0x0020;

const SPIRE_ASSIGNMENT_KNOWN_FLAGS: u16 = SPIRE_ASSIGNMENT_FLAG_PRIMARY
    | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA
    | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
    | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
    | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
    | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;

const PARTITION_OBJECT_MAGIC: u32 = 0x4f50_5345; // "ESPO" as little-endian bytes.
const PARTITION_OBJECT_HEADER_BYTES: usize = 46;
const ASSIGNMENT_ROW_FIXED_PREFIX_BYTES: usize = 3;
const ASSIGNMENT_ROW_FIXED_TAIL_BYTES: usize = ITEM_POINTER_BYTES + 1 + 4 + 4;

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
        if bytes.is_empty() {
            return Err("ec_spire vec_id must not be empty".to_owned());
        }
        if bytes.len() > SPIRE_VEC_ID_MAX_BYTES {
            return Err(format!(
                "ec_spire vec_id length {} exceeds max {SPIRE_VEC_ID_MAX_BYTES}",
                bytes.len()
            ));
        }
        match bytes[0] {
            SPIRE_LOCAL_VEC_ID_DISCRIMINATOR => {
                if bytes.len() != 1 + size_of::<u64>() {
                    return Err(format!(
                        "ec_spire local vec_id length mismatch: got {}, expected {}",
                        bytes.len(),
                        1 + size_of::<u64>()
                    ));
                }
            }
            SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR => {
                if bytes.len() == 1 {
                    return Err("ec_spire global vec_id payload must not be empty".to_owned());
                }
            }
            other => {
                return Err(format!("ec_spire unknown vec_id discriminator: {other:#x}"));
            }
        }
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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpirePartitionObjectKind {
    Root = 1,
    Internal = 2,
    Leaf = 3,
    Delta = 4,
}

impl SpirePartitionObjectKind {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Root),
            2 => Ok(Self::Internal),
            3 => Ok(Self::Leaf),
            4 => Ok(Self::Delta),
            other => Err(format!("ec_spire invalid partition object kind: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePartitionObjectHeader {
    pub(super) kind: SpirePartitionObjectKind,
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) level: u16,
    pub(super) parent_pid: u64,
    pub(super) child_count: u32,
    pub(super) assignment_count: u32,
    pub(super) flags: u32,
}

impl SpirePartitionObjectHeader {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        if self.pid == 0 {
            return Err("ec_spire partition object pid 0 is invalid".to_owned());
        }
        if self.object_version == 0 {
            return Err("ec_spire partition object version 0 is invalid".to_owned());
        }

        let mut out = Vec::with_capacity(PARTITION_OBJECT_HEADER_BYTES);
        out.extend_from_slice(&PARTITION_OBJECT_MAGIC.to_le_bytes());
        out.extend_from_slice(&1_u16.to_le_bytes());
        out.push(self.kind as u8);
        out.push(0);
        out.extend_from_slice(&self.pid.to_le_bytes());
        out.extend_from_slice(&self.object_version.to_le_bytes());
        out.extend_from_slice(&self.level.to_le_bytes());
        out.extend_from_slice(&self.parent_pid.to_le_bytes());
        out.extend_from_slice(&self.child_count.to_le_bytes());
        out.extend_from_slice(&self.assignment_count.to_le_bytes());
        out.extend_from_slice(&self.flags.to_le_bytes());
        debug_assert_eq!(out.len(), PARTITION_OBJECT_HEADER_BYTES);
        Ok(out)
    }

    pub(super) fn decode_prefix(input: &[u8]) -> Result<(Self, &[u8]), String> {
        if input.len() < PARTITION_OBJECT_HEADER_BYTES {
            return Err(format!(
                "ec_spire partition object header too short: got {}, expected at least {PARTITION_OBJECT_HEADER_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("magic bytes"));
        if magic != PARTITION_OBJECT_MAGIC {
            return Err(format!(
                "ec_spire invalid partition object magic: {magic:#x}"
            ));
        }
        let format_version =
            u16::from_le_bytes(input[4..6].try_into().expect("format version bytes"));
        if format_version != 1 {
            return Err(format!(
                "ec_spire unsupported partition object format version: {format_version}"
            ));
        }
        if input[7] != 0 {
            return Err(format!(
                "ec_spire partition object reserved byte must be zero, got {}",
                input[7]
            ));
        }

        let header = Self {
            kind: SpirePartitionObjectKind::decode(input[6])?,
            pid: u64::from_le_bytes(input[8..16].try_into().expect("pid bytes")),
            object_version: u64::from_le_bytes(
                input[16..24].try_into().expect("object version bytes"),
            ),
            level: u16::from_le_bytes(input[24..26].try_into().expect("level bytes")),
            parent_pid: u64::from_le_bytes(input[26..34].try_into().expect("parent pid bytes")),
            child_count: u32::from_le_bytes(input[34..38].try_into().expect("child count bytes")),
            assignment_count: u32::from_le_bytes(
                input[38..42].try_into().expect("assignment count bytes"),
            ),
            flags: u32::from_le_bytes(input[42..46].try_into().expect("flags bytes")),
        };
        if header.pid == 0 {
            return Err("ec_spire partition object pid 0 is invalid".to_owned());
        }
        if header.object_version == 0 {
            return Err("ec_spire partition object version 0 is invalid".to_owned());
        }
        Ok((header, &input[PARTITION_OBJECT_HEADER_BYTES..]))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafAssignmentRow {
    pub(super) flags: u16,
    pub(super) vec_id: SpireVecId,
    pub(super) heap_tid: ItemPointer,
    pub(super) payload_format: u8,
    pub(super) gamma: f32,
    pub(super) encoded_payload: Vec<u8>,
}

impl SpireLeafAssignmentRow {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        validate_assignment_flags(self.flags)?;
        if self.heap_tid == ItemPointer::INVALID {
            return Err("ec_spire assignment row heap_tid must be valid".to_owned());
        }
        if !self.gamma.is_finite() {
            return Err("ec_spire assignment row gamma must be finite".to_owned());
        }
        let vec_id_len = u8::try_from(self.vec_id.as_bytes().len())
            .map_err(|_| "ec_spire vec_id length exceeds u8".to_owned())?;
        let payload_len = u32::try_from(self.encoded_payload.len())
            .map_err(|_| "ec_spire assignment payload length exceeds u32".to_owned())?;

        let mut out = Vec::with_capacity(
            ASSIGNMENT_ROW_FIXED_PREFIX_BYTES
                + usize::from(vec_id_len)
                + ASSIGNMENT_ROW_FIXED_TAIL_BYTES
                + self.encoded_payload.len(),
        );
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.push(vec_id_len);
        out.extend_from_slice(self.vec_id.as_bytes());
        self.heap_tid.encode_into(&mut out);
        out.push(self.payload_format);
        out.extend_from_slice(&self.gamma.to_le_bytes());
        out.extend_from_slice(&payload_len.to_le_bytes());
        out.extend_from_slice(&self.encoded_payload);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        let (row, tail) = Self::decode_prefix(input)?;
        if !tail.is_empty() {
            return Err(format!(
                "ec_spire assignment row length mismatch: got trailing {} bytes",
                tail.len()
            ));
        }
        Ok(row)
    }

    fn decode_prefix(input: &[u8]) -> Result<(Self, &[u8]), String> {
        if input.len() < ASSIGNMENT_ROW_FIXED_PREFIX_BYTES + ASSIGNMENT_ROW_FIXED_TAIL_BYTES {
            return Err(format!(
                "ec_spire assignment row too short: got {}, expected at least {}",
                input.len(),
                ASSIGNMENT_ROW_FIXED_PREFIX_BYTES + ASSIGNMENT_ROW_FIXED_TAIL_BYTES
            ));
        }
        let flags = u16::from_le_bytes(input[0..2].try_into().expect("assignment flags bytes"));
        validate_assignment_flags(flags)?;
        let vec_id_len = input[2] as usize;
        if vec_id_len == 0 || vec_id_len > SPIRE_VEC_ID_MAX_BYTES {
            return Err(format!(
                "ec_spire assignment row vec_id length {vec_id_len} is invalid"
            ));
        }
        let min_len =
            ASSIGNMENT_ROW_FIXED_PREFIX_BYTES + vec_id_len + ASSIGNMENT_ROW_FIXED_TAIL_BYTES;
        if input.len() < min_len {
            return Err(format!(
                "ec_spire assignment row length {} is too short for vec_id length {vec_id_len}",
                input.len()
            ));
        }

        let vec_id_start = ASSIGNMENT_ROW_FIXED_PREFIX_BYTES;
        let vec_id_end = vec_id_start + vec_id_len;
        let heap_tid_start = vec_id_end;
        let heap_tid_end = heap_tid_start + ITEM_POINTER_BYTES;
        let payload_format_offset = heap_tid_end;
        let gamma_start = payload_format_offset + 1;
        let gamma_end = gamma_start + size_of::<f32>();
        let payload_len_start = gamma_end;
        let payload_len_end = payload_len_start + size_of::<u32>();

        let heap_tid = ItemPointer::decode(&input[heap_tid_start..heap_tid_end])?;
        if heap_tid == ItemPointer::INVALID {
            return Err("ec_spire assignment row heap_tid must be valid".to_owned());
        }
        let gamma = f32::from_le_bytes(input[gamma_start..gamma_end].try_into().expect("gamma"));
        if !gamma.is_finite() {
            return Err("ec_spire assignment row gamma must be finite".to_owned());
        }
        let payload_len = u32::from_le_bytes(
            input[payload_len_start..payload_len_end]
                .try_into()
                .expect("payload len"),
        ) as usize;
        let expected_len = payload_len_end + payload_len;
        if input.len() < expected_len {
            return Err(format!(
                "ec_spire assignment row length {} is too short for payload length {payload_len}",
                input.len()
            ));
        }

        Ok((
            Self {
                flags,
                vec_id: SpireVecId::from_bytes(&input[vec_id_start..vec_id_end])?,
                heap_tid,
                payload_format: input[payload_format_offset],
                gamma,
                encoded_payload: input[payload_len_end..expected_len].to_vec(),
            },
            &input[expected_len..],
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafPartitionObject {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) assignments: Vec<SpireLeafAssignmentRow>,
}

impl SpireLeafPartitionObject {
    pub(super) fn new(
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignments: Vec<SpireLeafAssignmentRow>,
    ) -> Result<Self, String> {
        let assignment_count = u32::try_from(assignments.len())
            .map_err(|_| "ec_spire leaf assignment count exceeds u32".to_owned())?;
        Ok(Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::Leaf,
                pid,
                object_version,
                level: 0,
                parent_pid,
                child_count: 0,
                assignment_count,
                flags: 0,
            },
            assignments,
        })
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate_header()?;

        let mut out = self.header.encode()?;
        for assignment in &self.assignments {
            out.extend_from_slice(&assignment.encode()?);
        }
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        let (header, mut tail) = SpirePartitionObjectHeader::decode_prefix(input)?;
        let mut object = Self {
            header,
            assignments: Vec::with_capacity(header.assignment_count as usize),
        };
        object.validate_header_without_assignment_len()?;

        for _ in 0..header.assignment_count {
            let (assignment, next_tail) = SpireLeafAssignmentRow::decode_prefix(tail)?;
            object.assignments.push(assignment);
            tail = next_tail;
        }
        if !tail.is_empty() {
            return Err(format!(
                "ec_spire leaf partition object has {} trailing bytes",
                tail.len()
            ));
        }
        object.validate_header()?;
        Ok(object)
    }

    fn validate_header(&self) -> Result<(), String> {
        let assignment_count = u32::try_from(self.assignments.len())
            .map_err(|_| "ec_spire leaf assignment count exceeds u32".to_owned())?;
        if self.header.assignment_count != assignment_count {
            return Err(format!(
                "ec_spire leaf assignment count mismatch: header {}, rows {assignment_count}",
                self.header.assignment_count
            ));
        }
        self.validate_header_without_assignment_len()
    }

    fn validate_header_without_assignment_len(&self) -> Result<(), String> {
        if self.header.kind != SpirePartitionObjectKind::Leaf {
            return Err(format!(
                "ec_spire leaf partition object header kind must be Leaf, got {:?}",
                self.header.kind
            ));
        }
        if self.header.child_count != 0 {
            return Err(format!(
                "ec_spire leaf partition object child_count must be 0, got {}",
                self.header.child_count
            ));
        }
        Ok(())
    }
}

fn validate_assignment_flags(flags: u16) -> Result<(), String> {
    let unknown = flags & !SPIRE_ASSIGNMENT_KNOWN_FLAGS;
    if unknown != 0 {
        return Err(format!(
            "ec_spire unknown assignment row flags: {unknown:#x}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        SpireLeafAssignmentRow, SpireLeafPartitionObject, SpirePartitionObjectHeader,
        SpirePartitionObjectKind, SpireVecId, SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
        SPIRE_ASSIGNMENT_FLAG_PRIMARY, SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR,
        SPIRE_LOCAL_VEC_ID_DISCRIMINATOR, SPIRE_VEC_ID_MAX_BYTES,
    };
    use crate::storage::page::ItemPointer;

    #[test]
    fn local_vec_id_round_trips_sequence() {
        let vec_id = SpireVecId::local(42);

        assert_eq!(vec_id.discriminator(), SPIRE_LOCAL_VEC_ID_DISCRIMINATOR);
        assert_eq!(vec_id.local_sequence(), Some(42));
        assert_eq!(
            SpireVecId::from_bytes(vec_id.as_bytes())
                .unwrap()
                .local_sequence(),
            Some(42)
        );
    }

    #[test]
    fn vec_id_rejects_invalid_shapes() {
        assert!(SpireVecId::from_bytes(&[]).is_err());
        assert!(SpireVecId::from_bytes(&[0xff, 1, 2]).is_err());
        assert!(SpireVecId::from_bytes(&[SPIRE_LOCAL_VEC_ID_DISCRIMINATOR, 1]).is_err());
        assert!(SpireVecId::from_bytes(&[SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR]).is_err());
        assert!(SpireVecId::global(&vec![7; SPIRE_VEC_ID_MAX_BYTES]).is_err());
    }

    #[test]
    fn global_vec_id_preserves_payload() {
        let vec_id = SpireVecId::global(&[9, 8, 7]).unwrap();

        assert_eq!(vec_id.discriminator(), SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR);
        assert_eq!(
            vec_id.as_bytes(),
            &[SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR, 9, 8, 7]
        );
        assert_eq!(SpireVecId::from_bytes(vec_id.as_bytes()).unwrap(), vec_id);
    }

    #[test]
    fn partition_object_header_decodes_prefix_and_payload_tail() {
        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 17,
            object_version: 3,
            level: 1,
            parent_pid: 5,
            child_count: 0,
            assignment_count: 99,
            flags: 0x10,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&[1, 2, 3]);

        let (decoded, tail) = SpirePartitionObjectHeader::decode_prefix(&encoded).unwrap();

        assert_eq!(decoded, header);
        assert_eq!(tail, &[1, 2, 3]);
    }

    #[test]
    fn partition_object_header_rejects_invalid_identity() {
        let mut header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Internal,
            pid: 0,
            object_version: 1,
            level: 0,
            parent_pid: 0,
            child_count: 1,
            assignment_count: 0,
            flags: 0,
        };
        assert!(header.encode().is_err());
        header.pid = 1;
        header.object_version = 0;
        assert!(header.encode().is_err());
    }

    #[test]
    fn assignment_row_round_trips() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
            vec_id: SpireVecId::local(123),
            heap_tid: ItemPointer {
                block_number: 44,
                offset_number: 7,
            },
            payload_format: 2,
            gamma: 1.25,
            encoded_payload: vec![4, 5, 6],
        };

        let decoded = SpireLeafAssignmentRow::decode(&row.encode().unwrap()).unwrap();

        assert_eq!(decoded, row);
    }

    #[test]
    fn assignment_row_prefix_decoder_returns_tail() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(123),
            heap_tid: ItemPointer {
                block_number: 44,
                offset_number: 7,
            },
            payload_format: 2,
            gamma: 1.25,
            encoded_payload: vec![4, 5, 6],
        };
        let mut encoded = row.encode().unwrap();
        encoded.extend_from_slice(&[9, 8]);

        let (decoded, tail) = SpireLeafAssignmentRow::decode_prefix(&encoded).unwrap();

        assert_eq!(decoded, row);
        assert_eq!(tail, &[9, 8]);
        assert!(SpireLeafAssignmentRow::decode(&encoded).is_err());
    }

    #[test]
    fn assignment_row_rejects_unknown_flags_and_invalid_locator() {
        let mut row = SpireLeafAssignmentRow {
            flags: 0x8000,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            payload_format: 0,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        };
        assert!(row.encode().is_err());

        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY;
        row.heap_tid = ItemPointer::INVALID;
        assert!(row.encode().is_err());
    }

    #[test]
    fn assignment_row_rejects_length_mismatch() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            payload_format: 0,
            gamma: 0.0,
            encoded_payload: vec![1, 2, 3],
        };
        let mut encoded = row.encode().unwrap();
        encoded.pop();

        assert!(SpireLeafAssignmentRow::decode(&encoded).is_err());
    }

    #[test]
    fn leaf_partition_object_round_trips_assignments() {
        let assignments = vec![
            SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(1),
                heap_tid: ItemPointer {
                    block_number: 10,
                    offset_number: 1,
                },
                payload_format: 1,
                gamma: 0.5,
                encoded_payload: vec![1, 2],
            },
            SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
                vec_id: SpireVecId::local(2),
                heap_tid: ItemPointer {
                    block_number: 10,
                    offset_number: 2,
                },
                payload_format: 1,
                gamma: 0.75,
                encoded_payload: vec![3, 4],
            },
        ];
        let object = SpireLeafPartitionObject::new(17, 3, 5, assignments).unwrap();

        let decoded = SpireLeafPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.pid, 17);
        assert_eq!(decoded.header.assignment_count, 2);
    }

    #[test]
    fn leaf_partition_object_rejects_non_leaf_header_and_children() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };
        let mut object = SpireLeafPartitionObject::new(17, 3, 0, vec![row]).unwrap();

        object.header.kind = SpirePartitionObjectKind::Internal;
        assert!(object.encode().is_err());

        object.header.kind = SpirePartitionObjectKind::Leaf;
        object.header.child_count = 1;
        assert!(object.encode().is_err());
    }

    #[test]
    fn leaf_partition_object_rejects_count_mismatch_and_trailing_bytes() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };
        let mut object = SpireLeafPartitionObject::new(17, 3, 0, vec![row]).unwrap();

        object.header.assignment_count = 2;
        assert!(object.encode().is_err());

        object.header.assignment_count = 1;
        let mut encoded = object.encode().unwrap();
        encoded.push(99);
        assert!(SpireLeafPartitionObject::decode(&encoded).is_err());
    }
}
