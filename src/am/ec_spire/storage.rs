//! PID-addressed partition-object storage codecs.

use std::{collections::HashSet, mem::size_of};

use pgrx::pg_sys;

use super::meta::{
    SpirePlacementEntry, SpirePlacementState, SPIRE_LOCAL_NODE_ID, SPIRE_SINGLE_LOCAL_STORE_ID,
};
use super::page;
use crate::storage::page::{
    element_or_neighbor_tuple_fits, usable_page_bytes, DataPageChain, ItemPointer,
    DEFAULT_PAGE_SIZE, ITEM_POINTER_BYTES,
};

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
    pub(super) published_epoch_backref: u64,
    pub(super) level: u16,
    pub(super) parent_pid: u64,
    pub(super) child_count: u32,
    pub(super) assignment_count: u32,
    pub(super) flags: u32,
}

impl SpirePartitionObjectHeader {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.encode_with_format_version(PARTITION_OBJECT_FORMAT_VERSION_V1)
    }

    fn encode_with_format_version(&self, format_version: u16) -> Result<Vec<u8>, String> {
        self.validate_for_format_version(format_version)?;
        Ok(self.encode_after_validation(format_version))
    }

    fn validate_for_format_version(&self, format_version: u16) -> Result<(), String> {
        if self.pid == 0 {
            return Err("ec_spire partition object pid 0 is invalid".to_owned());
        }
        if self.object_version == 0 {
            return Err("ec_spire partition object version 0 is invalid".to_owned());
        }
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V1
            && format_version != PARTITION_OBJECT_FORMAT_VERSION_V2
        {
            return Err(format!(
                "ec_spire unsupported partition object format version: {format_version}"
            ));
        }
        Ok(())
    }

    fn encode_after_validation(&self, format_version: u16) -> Vec<u8> {
        let mut out = Vec::with_capacity(PARTITION_OBJECT_HEADER_BYTES);
        out.extend_from_slice(&PARTITION_OBJECT_MAGIC.to_le_bytes());
        out.extend_from_slice(&format_version.to_le_bytes());
        out.push(self.kind as u8);
        out.push(0);
        out.extend_from_slice(&self.pid.to_le_bytes());
        out.extend_from_slice(&self.object_version.to_le_bytes());
        out.extend_from_slice(&self.published_epoch_backref.to_le_bytes());
        out.extend_from_slice(&self.level.to_le_bytes());
        out.extend_from_slice(&self.parent_pid.to_le_bytes());
        out.extend_from_slice(&self.child_count.to_le_bytes());
        out.extend_from_slice(&self.assignment_count.to_le_bytes());
        out.extend_from_slice(&self.flags.to_le_bytes());
        debug_assert_eq!(out.len(), PARTITION_OBJECT_HEADER_BYTES);
        out
    }

    pub(super) fn decode_prefix(input: &[u8]) -> Result<(Self, &[u8]), String> {
        let (header, format_version, tail) = Self::decode_prefix_with_format_version(input)?;
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V1 {
            return Err(format!(
                "ec_spire unsupported partition object format version: {format_version}"
            ));
        }
        Ok((header, tail))
    }

    fn decode_prefix_with_format_version(input: &[u8]) -> Result<(Self, u16, &[u8]), String> {
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
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V1
            && format_version != PARTITION_OBJECT_FORMAT_VERSION_V2
        {
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
            published_epoch_backref: u64::from_le_bytes(
                input[24..32].try_into().expect("published epoch bytes"),
            ),
            level: u16::from_le_bytes(input[32..34].try_into().expect("level bytes")),
            parent_pid: u64::from_le_bytes(input[34..42].try_into().expect("parent pid bytes")),
            child_count: u32::from_le_bytes(input[42..46].try_into().expect("child count bytes")),
            assignment_count: u32::from_le_bytes(
                input[46..50].try_into().expect("assignment count bytes"),
            ),
            flags: u32::from_le_bytes(input[50..54].try_into().expect("flags bytes")),
        };
        if header.pid == 0 {
            return Err("ec_spire partition object pid 0 is invalid".to_owned());
        }
        if header.object_version == 0 {
            return Err("ec_spire partition object version 0 is invalid".to_owned());
        }
        Ok((
            header,
            format_version,
            &input[PARTITION_OBJECT_HEADER_BYTES..],
        ))
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireLeafAssignmentRowRef<'a> {
    pub(super) flags: u16,
    pub(super) vec_id: SpireVecIdRef<'a>,
    pub(super) heap_tid: ItemPointer,
    pub(super) payload_format: u8,
    pub(super) gamma: f32,
    pub(super) encoded_payload: &'a [u8],
}

impl<'a> SpireLeafAssignmentRowRef<'a> {
    pub(super) fn to_owned(self) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: self.flags,
            vec_id: self.vec_id.to_owned(),
            heap_tid: self.heap_tid,
            payload_format: self.payload_format,
            gamma: self.gamma,
            encoded_payload: self.encoded_payload.to_vec(),
        }
    }
}

impl SpireLeafAssignmentRow {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate_wire_shape()?;
        Ok(self.encode_after_validation())
    }

    fn validate_wire_shape(&self) -> Result<(), String> {
        validate_assignment_flags(self.flags)?;
        validate_assignment_payload_format(self.payload_format)?;
        validate_vec_id_bytes(self.vec_id.as_bytes())?;
        if self.heap_tid == ItemPointer::INVALID {
            return Err("ec_spire assignment row heap_tid must be valid".to_owned());
        }
        if !self.gamma.is_finite() {
            return Err("ec_spire assignment row gamma must be finite".to_owned());
        }
        u8::try_from(self.vec_id.as_bytes().len())
            .map_err(|_| "ec_spire vec_id length exceeds u8".to_owned())?;
        u32::try_from(self.encoded_payload.len())
            .map_err(|_| "ec_spire assignment payload length exceeds u32".to_owned())?;
        self.encoded_len_after_validation()?;
        Ok(())
    }

    fn encoded_len_after_validation(&self) -> Result<usize, String> {
        ASSIGNMENT_ROW_FIXED_PREFIX_BYTES
            .checked_add(self.vec_id.as_bytes().len())
            .and_then(|len| len.checked_add(ASSIGNMENT_ROW_FIXED_TAIL_BYTES))
            .and_then(|len| len.checked_add(self.encoded_payload.len()))
            .ok_or_else(|| "ec_spire assignment row encoded length overflow".to_owned())
    }

    fn encode_after_validation(&self) -> Vec<u8> {
        let encoded_len = self
            .encoded_len_after_validation()
            .expect("assignment row was validated before encoding");
        let vec_id_len = u8::try_from(self.vec_id.as_bytes().len())
            .expect("assignment row vec_id length was validated");
        let payload_len = u32::try_from(self.encoded_payload.len())
            .expect("assignment row payload length was validated");

        let mut out = Vec::with_capacity(encoded_len);
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.push(vec_id_len);
        out.extend_from_slice(self.vec_id.as_bytes());
        self.heap_tid.encode_into(&mut out);
        out.push(self.payload_format);
        out.extend_from_slice(&self.gamma.to_le_bytes());
        out.extend_from_slice(&payload_len.to_le_bytes());
        out.extend_from_slice(&self.encoded_payload);
        out
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
        let (row_ref, tail) = Self::decode_prefix_ref(input)?;
        Ok((row_ref.to_owned(), tail))
    }

    pub(super) fn decode_prefix_ref(
        input: &[u8],
    ) -> Result<(SpireLeafAssignmentRowRef<'_>, &[u8]), String> {
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
        let payload_format = input[payload_format_offset];
        validate_assignment_payload_format(payload_format)?;
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
            SpireLeafAssignmentRowRef {
                flags,
                vec_id: SpireVecIdRef::from_bytes(&input[vec_id_start..vec_id_end])?,
                heap_tid,
                payload_format,
                gamma,
                encoded_payload: &input[payload_len_end..expected_len],
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
        let object = Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::Leaf,
                pid,
                object_version,
                published_epoch_backref: 0,
                level: 0,
                parent_pid,
                child_count: 0,
                assignment_count,
                flags: 0,
            },
            assignments,
        };
        object.validate_header()?;
        Ok(object)
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate_header()?;

        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V1);
        for assignment in &self.assignments {
            out.extend_from_slice(&assignment.encode_after_validation());
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
        self.validate_header_without_assignment_len()?;
        validate_leaf_assignments(&self.assignments)?;
        Ok(())
    }

    fn validate_header_without_assignment_len(&self) -> Result<(), String> {
        self.header
            .validate_for_format_version(PARTITION_OBJECT_FORMAT_VERSION_V1)?;
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

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafPartitionObjectV2Meta {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) payload_format: u8,
    pub(super) payload_stride: u32,
    pub(super) vec_id_kind: SpireVecIdKind,
    pub(super) vec_id_stride: u16,
    pub(super) segment_count: u32,
    pub(super) first_segment_locator: ItemPointer,
    pub(super) object_bytes_total: u64,
}

impl SpireLeafPartitionObjectV2Meta {
    fn new(
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignment_count: u32,
        payload_format: u8,
        payload_stride: u32,
        segment_count: u32,
        first_segment_locator: ItemPointer,
        object_bytes_total: u64,
        published_epoch_backref: u64,
    ) -> Result<Self, String> {
        let meta = Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::Leaf,
                pid,
                object_version,
                published_epoch_backref,
                level: 0,
                parent_pid,
                child_count: 0,
                assignment_count,
                flags: LEAF_V2_META_FLAG,
            },
            payload_format,
            payload_stride,
            vec_id_kind: SpireVecIdKind::LocalU64,
            vec_id_stride: LEAF_V2_LOCAL_VEC_ID_STRIDE as u16,
            segment_count,
            first_segment_locator,
            object_bytes_total,
        };
        meta.validate()?;
        Ok(meta)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V2);
        out.push(self.payload_format);
        out.push(self.vec_id_kind as u8);
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.payload_stride.to_le_bytes());
        out.extend_from_slice(&self.vec_id_stride.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.segment_count.to_le_bytes());
        self.first_segment_locator.encode_into(&mut out);
        out.extend_from_slice(&self.object_bytes_total.to_le_bytes());
        debug_assert_eq!(
            out.len(),
            PARTITION_OBJECT_HEADER_BYTES + LEAF_V2_META_BODY_BYTES
        );
        Ok(out)
    }

    fn decode(input: &[u8]) -> Result<Self, String> {
        let (header, format_version, tail) =
            SpirePartitionObjectHeader::decode_prefix_with_format_version(input)?;
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V2 {
            return Err(format!(
                "ec_spire leaf V2 meta format version must be 2, got {format_version}"
            ));
        }
        if tail.len() != LEAF_V2_META_BODY_BYTES {
            return Err(format!(
                "ec_spire leaf V2 meta length mismatch: got {}, expected {}",
                tail.len(),
                LEAF_V2_META_BODY_BYTES
            ));
        }
        let reserved = u16::from_le_bytes(tail[2..4].try_into().expect("leaf V2 reserved"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire leaf V2 meta reserved bytes must be zero, got {reserved}"
            ));
        }
        let reserved2 = u16::from_le_bytes(tail[10..12].try_into().expect("leaf V2 reserved2"));
        if reserved2 != 0 {
            return Err(format!(
                "ec_spire leaf V2 meta reserved2 bytes must be zero, got {reserved2}"
            ));
        }
        let first_segment_locator = ItemPointer::decode(&tail[16..22])?;
        let meta = Self {
            header,
            payload_format: tail[0],
            vec_id_kind: SpireVecIdKind::decode(tail[1])?,
            payload_stride: u32::from_le_bytes(tail[4..8].try_into().expect("payload stride")),
            vec_id_stride: u16::from_le_bytes(tail[8..10].try_into().expect("vec id stride")),
            segment_count: u32::from_le_bytes(tail[12..16].try_into().expect("segment count")),
            first_segment_locator,
            object_bytes_total: u64::from_le_bytes(
                tail[22..30].try_into().expect("object bytes total"),
            ),
        };
        meta.validate()?;
        Ok(meta)
    }

    fn validate(&self) -> Result<(), String> {
        validate_leaf_v2_header(&self.header, LEAF_V2_META_FLAG)?;
        validate_assignment_payload_format(self.payload_format)?;
        validate_leaf_v2_locator(self.first_segment_locator, "first segment")?;
        if self.header.published_epoch_backref == 0 {
            return Err("ec_spire leaf V2 published epoch backref 0 is invalid".to_owned());
        }
        if self.object_bytes_total == 0 {
            return Err("ec_spire leaf V2 object_bytes_total 0 is invalid".to_owned());
        }
        if self.vec_id_kind != SpireVecIdKind::LocalU64 {
            return Err("ec_spire leaf V2 Phase 1 only supports local_u64 vec_ids".to_owned());
        }
        if usize::from(self.vec_id_stride) != LEAF_V2_LOCAL_VEC_ID_STRIDE {
            return Err(format!(
                "ec_spire leaf V2 local vec_id stride mismatch: got {}, expected {LEAF_V2_LOCAL_VEC_ID_STRIDE}",
                self.vec_id_stride
            ));
        }
        if self.header.assignment_count == 0 {
            if self.segment_count != 0 {
                return Err("ec_spire empty leaf V2 meta cannot reference segments".to_owned());
            }
            if self.first_segment_locator != ItemPointer::INVALID {
                return Err("ec_spire empty leaf V2 meta first segment must be invalid".to_owned());
            }
            return Ok(());
        }
        if self.segment_count == 0 {
            return Err("ec_spire non-empty leaf V2 meta requires at least one segment".to_owned());
        }
        if self.first_segment_locator == ItemPointer::INVALID {
            return Err("ec_spire non-empty leaf V2 meta requires a first segment".to_owned());
        }
        if self.payload_format == SPIRE_PAYLOAD_FORMAT_NONE {
            return Err(
                "ec_spire non-empty leaf V2 meta payload format must not be NONE".to_owned(),
            );
        }
        if self.payload_stride == 0 {
            return Err("ec_spire non-empty leaf V2 meta payload stride 0 is invalid".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafPartitionObjectV2Segment {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) segment_no: u32,
    pub(super) row_base: u32,
    pub(super) next_segment_locator: ItemPointer,
    pub(super) flags: Vec<u16>,
    pub(super) vec_ids: Vec<u8>,
    pub(super) heap_tids: Vec<ItemPointer>,
    pub(super) gammas: Vec<f32>,
    pub(super) payloads: Vec<u8>,
}

impl SpireLeafPartitionObjectV2Segment {
    fn new(
        meta: &SpireLeafPartitionObjectV2Meta,
        segment_no: u32,
        row_base: u32,
        next_segment_locator: ItemPointer,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<Self, String> {
        let row_count = u32::try_from(rows.len())
            .map_err(|_| "ec_spire leaf V2 segment row count exceeds u32".to_owned())?;
        let mut flags = Vec::with_capacity(rows.len());
        let mut vec_ids = Vec::with_capacity(usize::from(meta.vec_id_stride) * rows.len());
        let mut heap_tids = Vec::with_capacity(rows.len());
        let mut gammas = Vec::with_capacity(rows.len());
        let mut payloads =
            Vec::with_capacity(usize::try_from(meta.payload_stride).unwrap_or(0) * rows.len());
        for row in rows {
            validate_leaf_assignment(row)?;
            if row.payload_format != meta.payload_format {
                return Err(format!(
                    "ec_spire leaf V2 segment payload format mismatch: got {}, expected {}",
                    row.payload_format, meta.payload_format
                ));
            }
            if row.encoded_payload.len() != meta.payload_stride as usize {
                return Err(format!(
                    "ec_spire leaf V2 segment payload stride mismatch: got {}, expected {}",
                    row.encoded_payload.len(),
                    meta.payload_stride
                ));
            }
            encode_leaf_v2_local_vec_id(&row.vec_id, &mut vec_ids)?;
            flags.push(row.flags);
            heap_tids.push(row.heap_tid);
            gammas.push(row.gamma);
            payloads.extend_from_slice(&row.encoded_payload);
        }

        let segment = Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::Leaf,
                pid: meta.header.pid,
                object_version: meta.header.object_version,
                published_epoch_backref: meta.header.published_epoch_backref,
                level: meta.header.level,
                parent_pid: meta.header.parent_pid,
                child_count: 0,
                assignment_count: row_count,
                flags: LEAF_V2_SEGMENT_FLAG,
            },
            segment_no,
            row_base,
            next_segment_locator,
            flags,
            vec_ids,
            heap_tids,
            gammas,
            payloads,
        };
        segment.validate_against_meta(meta)?;
        Ok(segment)
    }

    pub(super) fn columns<'a>(
        &'a self,
        meta: &'a SpireLeafPartitionObjectV2Meta,
    ) -> Result<SpireLeafObjectColumns<'a>, String> {
        self.validate_against_meta(meta)?;
        Ok(SpireLeafObjectColumns {
            header: self.header,
            payload_format: meta.payload_format,
            payload_stride: usize::try_from(meta.payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds usize".to_owned())?,
            vec_id_kind: meta.vec_id_kind,
            vec_id_stride: usize::from(meta.vec_id_stride),
            row_base: self.row_base,
            flags: &self.flags,
            vec_ids: &self.vec_ids,
            heap_tids: &self.heap_tids,
            gammas: &self.gammas,
            payloads: &self.payloads,
        })
    }

    fn encode(&self, meta: &SpireLeafPartitionObjectV2Meta) -> Result<Vec<u8>, String> {
        self.validate_against_meta(meta)?;
        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V2);
        out.extend_from_slice(&self.segment_no.to_le_bytes());
        out.extend_from_slice(&self.row_base.to_le_bytes());
        out.extend_from_slice(&self.header.assignment_count.to_le_bytes());
        self.next_segment_locator.encode_into(&mut out);
        for flag in &self.flags {
            out.extend_from_slice(&flag.to_le_bytes());
        }
        out.extend_from_slice(&self.vec_ids);
        for heap_tid in &self.heap_tids {
            heap_tid.encode_into(&mut out);
        }
        for gamma in &self.gammas {
            out.extend_from_slice(&gamma.to_le_bytes());
        }
        out.extend_from_slice(&self.payloads);
        Ok(out)
    }

    fn decode(input: &[u8], meta: &SpireLeafPartitionObjectV2Meta) -> Result<Self, String> {
        let (header, format_version, tail) =
            SpirePartitionObjectHeader::decode_prefix_with_format_version(input)?;
        if format_version != PARTITION_OBJECT_FORMAT_VERSION_V2 {
            return Err(format!(
                "ec_spire leaf V2 segment format version must be 2, got {format_version}"
            ));
        }
        if tail.len() < LEAF_V2_SEGMENT_PREFIX_BYTES {
            return Err(format!(
                "ec_spire leaf V2 segment too short: got {}, expected at least {}",
                tail.len(),
                LEAF_V2_SEGMENT_PREFIX_BYTES
            ));
        }
        let segment_no = u32::from_le_bytes(tail[0..4].try_into().expect("segment no"));
        let row_base = u32::from_le_bytes(tail[4..8].try_into().expect("row base"));
        let row_count = u32::from_le_bytes(tail[8..12].try_into().expect("row count"));
        let next_segment_locator = ItemPointer::decode(&tail[12..18])?;
        if header.assignment_count != row_count {
            return Err(format!(
                "ec_spire leaf V2 segment row count mismatch: header {}, body {row_count}",
                header.assignment_count
            ));
        }

        let row_count_usize = usize::try_from(row_count)
            .map_err(|_| "ec_spire leaf V2 segment row count exceeds usize".to_owned())?;
        let flags_bytes = row_count_usize
            .checked_mul(size_of::<u16>())
            .ok_or_else(|| "ec_spire leaf V2 flags byte length overflow".to_owned())?;
        let vec_id_bytes = row_count_usize
            .checked_mul(usize::from(meta.vec_id_stride))
            .ok_or_else(|| "ec_spire leaf V2 vec_id byte length overflow".to_owned())?;
        let heap_tid_bytes = row_count_usize
            .checked_mul(ITEM_POINTER_BYTES)
            .ok_or_else(|| "ec_spire leaf V2 heap_tid byte length overflow".to_owned())?;
        let gamma_bytes = row_count_usize
            .checked_mul(size_of::<f32>())
            .ok_or_else(|| "ec_spire leaf V2 gamma byte length overflow".to_owned())?;
        let payload_bytes = row_count_usize
            .checked_mul(
                usize::try_from(meta.payload_stride)
                    .map_err(|_| "ec_spire leaf V2 payload stride exceeds usize".to_owned())?,
            )
            .ok_or_else(|| "ec_spire leaf V2 payload byte length overflow".to_owned())?;
        let expected_tail_len = LEAF_V2_SEGMENT_PREFIX_BYTES
            .checked_add(flags_bytes)
            .and_then(|len| len.checked_add(vec_id_bytes))
            .and_then(|len| len.checked_add(heap_tid_bytes))
            .and_then(|len| len.checked_add(gamma_bytes))
            .and_then(|len| len.checked_add(payload_bytes))
            .ok_or_else(|| "ec_spire leaf V2 segment length overflow".to_owned())?;
        if tail.len() != expected_tail_len {
            return Err(format!(
                "ec_spire leaf V2 segment length mismatch: got {}, expected {expected_tail_len}",
                tail.len()
            ));
        }

        let mut cursor = LEAF_V2_SEGMENT_PREFIX_BYTES;
        let mut flags = Vec::with_capacity(row_count_usize);
        for chunk in tail[cursor..cursor + flags_bytes].chunks_exact(size_of::<u16>()) {
            flags.push(u16::from_le_bytes(chunk.try_into().expect("flag bytes")));
        }
        cursor += flags_bytes;
        let vec_ids = tail[cursor..cursor + vec_id_bytes].to_vec();
        cursor += vec_id_bytes;
        let mut heap_tids = Vec::with_capacity(row_count_usize);
        for chunk in tail[cursor..cursor + heap_tid_bytes].chunks_exact(ITEM_POINTER_BYTES) {
            heap_tids.push(ItemPointer::decode(chunk)?);
        }
        cursor += heap_tid_bytes;
        let mut gammas = Vec::with_capacity(row_count_usize);
        for chunk in tail[cursor..cursor + gamma_bytes].chunks_exact(size_of::<f32>()) {
            let gamma = f32::from_le_bytes(chunk.try_into().expect("gamma bytes"));
            if !gamma.is_finite() {
                return Err("ec_spire leaf V2 segment gamma must be finite".to_owned());
            }
            gammas.push(gamma);
        }
        cursor += gamma_bytes;
        let payloads = tail[cursor..cursor + payload_bytes].to_vec();

        let segment = Self {
            header,
            segment_no,
            row_base,
            next_segment_locator,
            flags,
            vec_ids,
            heap_tids,
            gammas,
            payloads,
        };
        segment.validate_against_meta(meta)?;
        Ok(segment)
    }

    fn validate_against_meta(&self, meta: &SpireLeafPartitionObjectV2Meta) -> Result<(), String> {
        validate_leaf_v2_header(&self.header, LEAF_V2_SEGMENT_FLAG)?;
        if self.header.pid != meta.header.pid
            || self.header.object_version != meta.header.object_version
            || self.header.parent_pid != meta.header.parent_pid
        {
            return Err("ec_spire leaf V2 segment header does not match meta".to_owned());
        }
        validate_leaf_v2_locator(self.next_segment_locator, "next segment")?;
        let row_count = usize::try_from(self.header.assignment_count)
            .map_err(|_| "ec_spire leaf V2 segment row count exceeds usize".to_owned())?;
        if row_count == 0 {
            return Err("ec_spire leaf V2 segment row count 0 is invalid".to_owned());
        }
        if self.flags.len() != row_count {
            return Err(format!(
                "ec_spire leaf V2 segment flags length mismatch: got {}, expected {row_count}",
                self.flags.len()
            ));
        }
        for flags in &self.flags {
            validate_assignment_flags(*flags)?;
            if flags & (SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE)
                != 0
            {
                return Err("ec_spire leaf V2 segment rows cannot set delta flags".to_owned());
            }
        }
        let expected_vec_id_bytes = row_count
            .checked_mul(usize::from(meta.vec_id_stride))
            .ok_or_else(|| "ec_spire leaf V2 vec_id length overflow".to_owned())?;
        if self.vec_ids.len() != expected_vec_id_bytes {
            return Err(format!(
                "ec_spire leaf V2 segment vec_id length mismatch: got {}, expected {expected_vec_id_bytes}",
                self.vec_ids.len()
            ));
        }
        for chunk in self.vec_ids.chunks_exact(usize::from(meta.vec_id_stride)) {
            decode_leaf_v2_local_vec_id(chunk)?;
        }
        if self.heap_tids.len() != row_count {
            return Err(format!(
                "ec_spire leaf V2 segment heap_tid length mismatch: got {}, expected {row_count}",
                self.heap_tids.len()
            ));
        }
        if self.heap_tids.contains(&ItemPointer::INVALID) {
            return Err("ec_spire leaf V2 segment heap_tid must be valid".to_owned());
        }
        if self.gammas.len() != row_count {
            return Err(format!(
                "ec_spire leaf V2 segment gamma length mismatch: got {}, expected {row_count}",
                self.gammas.len()
            ));
        }
        if self.gammas.iter().any(|gamma| !gamma.is_finite()) {
            return Err("ec_spire leaf V2 segment gamma must be finite".to_owned());
        }
        let expected_payload_bytes = row_count
            .checked_mul(
                usize::try_from(meta.payload_stride)
                    .map_err(|_| "ec_spire leaf V2 payload stride exceeds usize".to_owned())?,
            )
            .ok_or_else(|| "ec_spire leaf V2 payload length overflow".to_owned())?;
        if self.payloads.len() != expected_payload_bytes {
            return Err(format!(
                "ec_spire leaf V2 segment payload length mismatch: got {}, expected {expected_payload_bytes}",
                self.payloads.len()
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafPartitionObjectV2 {
    pub(super) meta: SpireLeafPartitionObjectV2Meta,
    pub(super) segments: Vec<SpireLeafPartitionObjectV2Segment>,
}

impl SpireLeafPartitionObjectV2 {
    fn new(
        meta: SpireLeafPartitionObjectV2Meta,
        segments: Vec<SpireLeafPartitionObjectV2Segment>,
    ) -> Result<Self, String> {
        let object = Self { meta, segments };
        object.validate()?;
        Ok(object)
    }

    fn validate(&self) -> Result<(), String> {
        let segment_count = u32::try_from(self.segments.len())
            .map_err(|_| "ec_spire leaf V2 segment count exceeds u32".to_owned())?;
        if self.meta.segment_count != segment_count {
            return Err(format!(
                "ec_spire leaf V2 segment count mismatch: meta {}, segments {segment_count}",
                self.meta.segment_count
            ));
        }
        let mut expected_row_base = 0_u32;
        for (expected_segment_no, segment) in self.segments.iter().enumerate() {
            segment.validate_against_meta(&self.meta)?;
            let expected_segment_no = u32::try_from(expected_segment_no)
                .map_err(|_| "ec_spire leaf V2 segment index exceeds u32".to_owned())?;
            if segment.segment_no != expected_segment_no {
                return Err(format!(
                    "ec_spire leaf V2 segment number mismatch: got {}, expected {expected_segment_no}",
                    segment.segment_no
                ));
            }
            if segment.row_base != expected_row_base {
                return Err(format!(
                    "ec_spire leaf V2 segment row_base mismatch: got {}, expected {expected_row_base}",
                    segment.row_base
                ));
            }
            expected_row_base = expected_row_base
                .checked_add(segment.header.assignment_count)
                .ok_or_else(|| "ec_spire leaf V2 assignment count overflow".to_owned())?;
            if expected_segment_no + 1 == segment_count {
                if segment.next_segment_locator != ItemPointer::INVALID {
                    return Err(
                        "ec_spire leaf V2 final segment next locator must be invalid".to_owned(),
                    );
                }
            } else if segment.next_segment_locator == ItemPointer::INVALID {
                return Err("ec_spire leaf V2 non-final segment requires next locator".to_owned());
            }
        }
        if self.meta.header.assignment_count != expected_row_base {
            return Err(format!(
                "ec_spire leaf V2 assignment count mismatch: meta {}, segments {expected_row_base}",
                self.meta.header.assignment_count
            ));
        }
        Ok(())
    }

    pub(super) fn column_segments(&self) -> Result<Vec<SpireLeafObjectColumns<'_>>, String> {
        self.validate()?;
        self.segments
            .iter()
            .map(|segment| segment.columns(&self.meta))
            .collect()
    }

    pub(super) fn assignment_rows(&self) -> Result<Vec<SpireLeafAssignmentRow>, String> {
        let column_segments = self.column_segments()?;
        let row_count = usize::try_from(self.meta.header.assignment_count)
            .map_err(|_| "ec_spire leaf V2 assignment count exceeds usize".to_owned())?;
        let mut rows = Vec::with_capacity(row_count);
        for columns in column_segments {
            for row_offset in 0..columns.row_count() {
                let row = columns.row(row_offset)?;
                rows.push(SpireLeafAssignmentRow {
                    flags: row.flags,
                    vec_id: SpireVecId::local(row.local_vec_seq()?),
                    heap_tid: row.heap_tid,
                    payload_format: columns.payload_format,
                    gamma: row.gamma,
                    encoded_payload: row.encoded_payload.to_vec(),
                });
            }
        }
        Ok(rows)
    }
}

pub(super) trait SpireObjectReader {
    fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String>;

    fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String>;

    fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String>;

    fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String>;

    fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String>;
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRoutingChildEntry {
    pub(super) centroid_index: u32,
    pub(super) child_pid: u64,
    pub(super) centroid: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireRoutingChildView<'a> {
    pub(super) centroid_index: u32,
    pub(super) child_pid: u64,
    pub(super) centroid: &'a [f32],
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRoutingPartitionObject {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) dimensions: u16,
    pub(super) centroid_ordinals: Vec<u32>,
    pub(super) child_pids: Vec<u64>,
    pub(super) centroids: Vec<f32>,
}

impl SpireRoutingPartitionObject {
    pub(super) fn root(
        pid: u64,
        object_version: u64,
        dimensions: u16,
        children: Vec<SpireRoutingChildEntry>,
    ) -> Result<Self, String> {
        Self::new(
            SpirePartitionObjectKind::Root,
            pid,
            object_version,
            1,
            0,
            dimensions,
            children,
        )
    }

    pub(super) fn internal(
        pid: u64,
        object_version: u64,
        level: u16,
        parent_pid: u64,
        dimensions: u16,
        children: Vec<SpireRoutingChildEntry>,
    ) -> Result<Self, String> {
        Self::new(
            SpirePartitionObjectKind::Internal,
            pid,
            object_version,
            level,
            parent_pid,
            dimensions,
            children,
        )
    }

    fn new(
        kind: SpirePartitionObjectKind,
        pid: u64,
        object_version: u64,
        level: u16,
        parent_pid: u64,
        dimensions: u16,
        children: Vec<SpireRoutingChildEntry>,
    ) -> Result<Self, String> {
        let child_count = u32::try_from(children.len())
            .map_err(|_| "ec_spire routing child count exceeds u32".to_owned())?;
        let dimensions_usize = usize::from(dimensions);
        let centroid_capacity = children
            .len()
            .checked_mul(dimensions_usize)
            .ok_or_else(|| "ec_spire routing centroid component count overflow".to_owned())?;
        let mut centroid_ordinals = Vec::with_capacity(children.len());
        let mut child_pids = Vec::with_capacity(children.len());
        let mut centroids = Vec::with_capacity(centroid_capacity);
        for child in children {
            centroid_ordinals.push(child.centroid_index);
            child_pids.push(child.child_pid);
            centroids.extend_from_slice(&child.centroid);
        }
        let object = Self {
            header: SpirePartitionObjectHeader {
                kind,
                pid,
                object_version,
                published_epoch_backref: 0,
                level,
                parent_pid,
                child_count,
                assignment_count: 0,
                flags: 0,
            },
            dimensions,
            centroid_ordinals,
            child_pids,
            centroids,
        };
        object.validate()?;
        Ok(object)
    }

    pub(super) fn child_count(&self) -> usize {
        self.child_pids.len()
    }

    pub(super) fn child_centroid(&self, child_index: usize) -> Option<&[f32]> {
        let dimensions = usize::from(self.dimensions);
        let start = child_index.checked_mul(dimensions)?;
        let end = start.checked_add(dimensions)?;
        self.centroids.get(start..end)
    }

    pub(super) fn children(&self) -> impl Iterator<Item = SpireRoutingChildView<'_>> + '_ {
        let dimensions = usize::from(self.dimensions);
        self.centroid_ordinals
            .iter()
            .copied()
            .zip(self.child_pids.iter().copied())
            .zip(self.centroids.chunks_exact(dimensions))
            .map(
                |((centroid_index, child_pid), centroid)| SpireRoutingChildView {
                    centroid_index,
                    child_pid,
                    centroid,
                },
            )
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V1);
        out.extend_from_slice(&self.dimensions.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        for child in self.children() {
            out.extend_from_slice(&child.centroid_index.to_le_bytes());
            out.extend_from_slice(&child.child_pid.to_le_bytes());
            for component in child.centroid {
                out.extend_from_slice(&component.to_le_bytes());
            }
        }
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        let (header, tail) = SpirePartitionObjectHeader::decode_prefix(input)?;
        if tail.len() < ROUTING_OBJECT_BODY_PREFIX_BYTES {
            return Err(format!(
                "ec_spire routing partition object body too short: got {}, expected at least {ROUTING_OBJECT_BODY_PREFIX_BYTES}",
                tail.len()
            ));
        }
        let dimensions = u16::from_le_bytes(tail[0..2].try_into().expect("routing dimensions"));
        let reserved = u16::from_le_bytes(tail[2..4].try_into().expect("routing reserved"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire routing partition object reserved bytes must be zero, got {reserved}"
            ));
        }

        let child_count = usize::try_from(header.child_count)
            .map_err(|_| "ec_spire routing child count exceeds usize".to_owned())?;
        let centroid_bytes = usize::from(dimensions)
            .checked_mul(size_of::<f32>())
            .ok_or_else(|| "ec_spire routing centroid byte length overflow".to_owned())?;
        let child_bytes = ROUTING_CHILD_ENTRY_FIXED_BYTES
            .checked_add(centroid_bytes)
            .ok_or_else(|| "ec_spire routing child byte length overflow".to_owned())?;
        let expected_tail_len = child_count
            .checked_mul(child_bytes)
            .and_then(|children_bytes| children_bytes.checked_add(ROUTING_OBJECT_BODY_PREFIX_BYTES))
            .ok_or_else(|| "ec_spire routing partition object length overflow".to_owned())?;
        if tail.len() != expected_tail_len {
            return Err(format!(
                "ec_spire routing partition object length mismatch: got {}, expected {}",
                tail.len(),
                expected_tail_len
            ));
        }

        let centroid_capacity = child_count
            .checked_mul(usize::from(dimensions))
            .ok_or_else(|| "ec_spire routing centroid component count overflow".to_owned())?;
        let mut centroid_ordinals = Vec::with_capacity(child_count);
        let mut child_pids = Vec::with_capacity(child_count);
        let mut centroids = Vec::with_capacity(centroid_capacity);
        let mut cursor = ROUTING_OBJECT_BODY_PREFIX_BYTES;
        for _ in 0..child_count {
            let centroid_index =
                u32::from_le_bytes(tail[cursor..cursor + 4].try_into().expect("centroid index"));
            cursor += 4;
            let child_pid =
                u64::from_le_bytes(tail[cursor..cursor + 8].try_into().expect("child pid"));
            cursor += 8;
            centroid_ordinals.push(centroid_index);
            child_pids.push(child_pid);
            for _ in 0..dimensions {
                centroids.push(f32::from_le_bytes(
                    tail[cursor..cursor + 4]
                        .try_into()
                        .expect("centroid component"),
                ));
                cursor += 4;
            }
        }

        let object = Self {
            header,
            dimensions,
            centroid_ordinals,
            child_pids,
            centroids,
        };
        object.validate()?;
        Ok(object)
    }

    fn validate(&self) -> Result<(), String> {
        self.header
            .validate_for_format_version(PARTITION_OBJECT_FORMAT_VERSION_V1)?;
        match self.header.kind {
            SpirePartitionObjectKind::Root => {
                if self.header.parent_pid != 0 {
                    return Err("ec_spire root routing object parent_pid must be 0".to_owned());
                }
            }
            SpirePartitionObjectKind::Internal => {
                if self.header.parent_pid == 0 {
                    return Err(
                        "ec_spire internal routing object parent_pid 0 is invalid".to_owned()
                    );
                }
            }
            other => {
                return Err(format!(
                    "ec_spire routing partition object kind must be Root or Internal, got {other:?}"
                ));
            }
        }
        if self.header.level == 0 {
            return Err("ec_spire routing partition object level 0 is invalid".to_owned());
        }
        if self.header.assignment_count != 0 {
            return Err(format!(
                "ec_spire routing partition object assignment_count must be 0, got {}",
                self.header.assignment_count
            ));
        }
        let child_count = u32::try_from(self.child_count())
            .map_err(|_| "ec_spire routing child count exceeds u32".to_owned())?;
        if self.header.child_count != child_count {
            return Err(format!(
                "ec_spire routing child count mismatch: header {}, children {child_count}",
                self.header.child_count
            ));
        }
        if self.dimensions == 0 {
            return Err("ec_spire routing partition object dimensions 0 is invalid".to_owned());
        }
        if self.child_pids.is_empty() {
            return Err("ec_spire routing partition object requires at least one child".to_owned());
        }

        let dimensions = usize::from(self.dimensions);
        if self.centroid_ordinals.len() != self.child_pids.len() {
            return Err(format!(
                "ec_spire routing centroid ordinal count {} does not match child pid count {}",
                self.centroid_ordinals.len(),
                self.child_pids.len()
            ));
        }
        let expected_centroid_components = self
            .child_pids
            .len()
            .checked_mul(dimensions)
            .ok_or_else(|| "ec_spire routing centroid component count overflow".to_owned())?;
        if self.centroids.len() != expected_centroid_components {
            return Err(format!(
                "ec_spire routing centroid component count mismatch: got {}, expected {expected_centroid_components}",
                self.centroids.len()
            ));
        }

        for (expected_index, child) in self.children().enumerate() {
            let expected_index = u32::try_from(expected_index)
                .map_err(|_| "ec_spire routing child centroid index exceeds u32".to_owned())?;
            if child.centroid_index != expected_index {
                return Err(format!(
                    "ec_spire routing child centroid index mismatch: got {}, expected {expected_index}",
                    child.centroid_index
                ));
            }
            if child.child_pid == 0 {
                return Err("ec_spire routing child pid 0 is invalid".to_owned());
            }
            if child
                .centroid
                .iter()
                .any(|component| !component.is_finite())
            {
                return Err(format!(
                    "ec_spire routing child centroid {} must be finite",
                    child.centroid_index
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireDeltaPartitionObject {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) assignments: Vec<SpireLeafAssignmentRow>,
}

impl SpireDeltaPartitionObject {
    pub(super) fn new(
        pid: u64,
        object_version: u64,
        base_pid: u64,
        assignments: Vec<SpireLeafAssignmentRow>,
    ) -> Result<Self, String> {
        if base_pid == 0 {
            return Err("ec_spire delta partition object base_pid 0 is invalid".to_owned());
        }
        let assignment_count = u32::try_from(assignments.len())
            .map_err(|_| "ec_spire delta assignment count exceeds u32".to_owned())?;
        let object = Self {
            header: SpirePartitionObjectHeader {
                kind: SpirePartitionObjectKind::Delta,
                pid,
                object_version,
                published_epoch_backref: 0,
                level: 0,
                parent_pid: base_pid,
                child_count: 0,
                assignment_count,
                flags: 0,
            },
            assignments,
        };
        object.validate_header()?;
        Ok(object)
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate_header()?;

        let mut out = self
            .header
            .encode_after_validation(PARTITION_OBJECT_FORMAT_VERSION_V1);
        for assignment in &self.assignments {
            out.extend_from_slice(&assignment.encode_after_validation());
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
            validate_delta_assignment(&assignment)?;
            object.assignments.push(assignment);
            tail = next_tail;
        }
        if !tail.is_empty() {
            return Err(format!(
                "ec_spire delta partition object has {} trailing bytes",
                tail.len()
            ));
        }
        object.validate_header()?;
        Ok(object)
    }

    fn validate_header(&self) -> Result<(), String> {
        let assignment_count = u32::try_from(self.assignments.len())
            .map_err(|_| "ec_spire delta assignment count exceeds u32".to_owned())?;
        if self.header.assignment_count != assignment_count {
            return Err(format!(
                "ec_spire delta assignment count mismatch: header {}, rows {assignment_count}",
                self.header.assignment_count
            ));
        }
        self.validate_header_without_assignment_len()?;
        validate_delta_assignments(&self.assignments)?;
        Ok(())
    }

    fn validate_header_without_assignment_len(&self) -> Result<(), String> {
        self.header
            .validate_for_format_version(PARTITION_OBJECT_FORMAT_VERSION_V1)?;
        if self.header.kind != SpirePartitionObjectKind::Delta {
            return Err(format!(
                "ec_spire delta partition object header kind must be Delta, got {:?}",
                self.header.kind
            ));
        }
        if self.header.parent_pid == 0 {
            return Err("ec_spire delta partition object base_pid 0 is invalid".to_owned());
        }
        if self.header.child_count != 0 {
            return Err(format!(
                "ec_spire delta partition object child_count must be 0, got {}",
                self.header.child_count
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(super) struct SpireLocalObjectStore {
    store_relid: u32,
    pages: DataPageChain,
}

impl SpireLocalObjectStore {
    pub(super) fn with_default_page_size(store_relid: u32) -> Result<Self, String> {
        Self::new(store_relid, DEFAULT_PAGE_SIZE)
    }

    pub(super) fn new(store_relid: u32, page_size: usize) -> Result<Self, String> {
        if store_relid == 0 {
            return Err("ec_spire local object store relid 0 is invalid".to_owned());
        }
        if page_size == 0 {
            return Err("ec_spire local object store page size 0 is invalid".to_owned());
        }
        Ok(Self {
            store_relid,
            pages: DataPageChain::new(page_size),
        })
    }

    pub(super) fn page_count(&self) -> usize {
        self.pages.pages().len()
    }

    pub(super) fn insert_leaf_object(
        &mut self,
        epoch: u64,
        object: &SpireLeafPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire local object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = self.pages.insert_raw_tuple(encoded)?;
        let placement = SpirePlacementEntry::local_single_store_available(
            epoch,
            durable_object.header.pid,
            self.store_relid,
            durable_object.header.object_version,
            object_tid,
            object_bytes,
        );
        placement.encode()?;
        Ok(placement)
    }

    pub(super) fn insert_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignments: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire local object store epoch 0 is invalid".to_owned());
        }
        validate_leaf_assignments(assignments)?;
        let assignment_count = u32::try_from(assignments.len())
            .map_err(|_| "ec_spire leaf V2 assignment count exceeds u32".to_owned())?;
        let (payload_format, payload_stride) = leaf_v2_payload_layout(assignments)?;
        let max_segment_rows = leaf_v2_max_segment_rows(
            self.pages.page_size(),
            payload_stride,
            LEAF_V2_LOCAL_VEC_ID_STRIDE,
        )?;
        let segment_count = if assignments.is_empty() {
            0
        } else {
            let count = assignments
                .len()
                .checked_add(max_segment_rows - 1)
                .and_then(|value| value.checked_div(max_segment_rows))
                .ok_or_else(|| "ec_spire leaf V2 segment count overflow".to_owned())?;
            u32::try_from(count)
                .map_err(|_| "ec_spire leaf V2 segment count exceeds u32".to_owned())?
        };

        let provisional_first_segment = if assignments.is_empty() {
            ItemPointer::INVALID
        } else {
            ItemPointer {
                block_number: 1,
                offset_number: 1,
            }
        };
        let provisional_meta = SpireLeafPartitionObjectV2Meta::new(
            pid,
            object_version,
            parent_pid,
            assignment_count,
            payload_format,
            u32::try_from(payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds u32".to_owned())?,
            segment_count,
            provisional_first_segment,
            1,
            epoch,
        )?;

        let mut next_segment_locator = ItemPointer::INVALID;
        let mut segment_bytes_total = 0_u64;
        for segment_index in (0..usize::try_from(segment_count).unwrap_or(0)).rev() {
            let row_base = segment_index
                .checked_mul(max_segment_rows)
                .ok_or_else(|| "ec_spire leaf V2 row_base overflow".to_owned())?;
            let rows_end = assignments.len().min(row_base + max_segment_rows);
            let segment = SpireLeafPartitionObjectV2Segment::new(
                &provisional_meta,
                u32::try_from(segment_index)
                    .map_err(|_| "ec_spire leaf V2 segment index exceeds u32".to_owned())?,
                u32::try_from(row_base)
                    .map_err(|_| "ec_spire leaf V2 row_base exceeds u32".to_owned())?,
                next_segment_locator,
                &assignments[row_base..rows_end],
            )?;
            let encoded_segment = segment.encode(&provisional_meta)?;
            segment_bytes_total =
                segment_bytes_total
                    .checked_add(u64::try_from(encoded_segment.len()).map_err(|_| {
                        "ec_spire leaf V2 segment byte length exceeds u64".to_owned()
                    })?)
                    .ok_or_else(|| "ec_spire leaf V2 object byte length overflow".to_owned())?;
            next_segment_locator = self.pages.insert_raw_tuple(encoded_segment)?;
        }

        let first_segment_locator = if assignments.is_empty() {
            ItemPointer::INVALID
        } else {
            next_segment_locator
        };
        let meta_bytes_len = PARTITION_OBJECT_HEADER_BYTES
            .checked_add(LEAF_V2_META_BODY_BYTES)
            .ok_or_else(|| "ec_spire leaf V2 meta byte length overflow".to_owned())?;
        let object_bytes_total = segment_bytes_total
            .checked_add(
                u64::try_from(meta_bytes_len)
                    .map_err(|_| "ec_spire leaf V2 meta byte length exceeds u64".to_owned())?,
            )
            .ok_or_else(|| "ec_spire leaf V2 object byte length overflow".to_owned())?;
        let object_bytes = u32::try_from(object_bytes_total)
            .map_err(|_| "ec_spire leaf V2 object length exceeds u32".to_owned())?;
        let meta = SpireLeafPartitionObjectV2Meta::new(
            pid,
            object_version,
            parent_pid,
            assignment_count,
            payload_format,
            u32::try_from(payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds u32".to_owned())?,
            segment_count,
            first_segment_locator,
            object_bytes_total,
            epoch,
        )?;
        let encoded_meta = meta.encode()?;
        let meta_tid = self.pages.insert_raw_tuple(encoded_meta)?;
        let placement = SpirePlacementEntry::local_single_store_available(
            epoch,
            pid,
            self.store_relid,
            object_version,
            meta_tid,
            object_bytes,
        );
        placement.encode()?;
        Ok(placement)
    }

    pub(super) fn insert_delta_object(
        &mut self,
        epoch: u64,
        object: &SpireDeltaPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire local object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = self.pages.insert_raw_tuple(encoded)?;
        let placement = SpirePlacementEntry::local_single_store_available(
            epoch,
            durable_object.header.pid,
            self.store_relid,
            durable_object.header.object_version,
            object_tid,
            object_bytes,
        );
        placement.encode()?;
        Ok(placement)
    }

    pub(super) fn insert_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire local object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = self.pages.insert_raw_tuple(encoded)?;
        let placement = SpirePlacementEntry::local_single_store_available(
            epoch,
            durable_object.header.pid,
            self.store_relid,
            durable_object.header.object_version,
            object_tid,
            object_bytes,
        );
        placement.encode()?;
        Ok(placement)
    }

    pub(super) fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String> {
        let raw = self.read_object_bytes(placement)?;
        let object = SpireLeafPartitionObject::decode(raw)?;
        if object.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match object pid {}",
                placement.pid, object.header.pid
            ));
        }
        if object.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match object version {}",
                placement.object_version, object.header.object_version
            ));
        }
        if object.header.published_epoch_backref == 0
            || object.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                object.header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(object)
    }

    pub(super) fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String> {
        self.validate_local_available_placement(placement)?;
        let raw_meta = self.read_raw_tuple(placement.object_tid)?;
        let meta = SpireLeafPartitionObjectV2Meta::decode(raw_meta)?;
        if meta.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match leaf V2 pid {}",
                placement.pid, meta.header.pid
            ));
        }
        if meta.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match leaf V2 version {}",
                placement.object_version, meta.header.object_version
            ));
        }
        if meta.header.published_epoch_backref == 0
            || meta.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire leaf V2 published epoch backref {} is not valid for placement epoch {}",
                meta.header.published_epoch_backref, placement.epoch
            ));
        }
        if u64::from(placement.object_bytes) != meta.object_bytes_total {
            return Err(format!(
                "ec_spire placement object_bytes {} does not match leaf V2 total {}",
                placement.object_bytes, meta.object_bytes_total
            ));
        }

        let segment_count = usize::try_from(meta.segment_count)
            .map_err(|_| "ec_spire leaf V2 segment count exceeds usize".to_owned())?;
        let mut segments = Vec::with_capacity(segment_count);
        let mut next_locator = meta.first_segment_locator;
        for _ in 0..segment_count {
            if next_locator == ItemPointer::INVALID {
                return Err("ec_spire leaf V2 segment chain ended early".to_owned());
            }
            let raw_segment = self.read_raw_tuple(next_locator)?;
            let segment = SpireLeafPartitionObjectV2Segment::decode(raw_segment, &meta)?;
            next_locator = segment.next_segment_locator;
            segments.push(segment);
        }
        if next_locator != ItemPointer::INVALID {
            return Err("ec_spire leaf V2 segment chain has trailing locator".to_owned());
        }
        SpireLeafPartitionObjectV2::new(meta, segments)
    }

    pub(super) fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String> {
        self.validate_local_available_placement(placement)?;
        let raw = self.read_raw_tuple(placement.object_tid)?;
        let (mut header, format_version, _) =
            SpirePartitionObjectHeader::decode_prefix_with_format_version(raw)?;
        match format_version {
            PARTITION_OBJECT_FORMAT_VERSION_V1 => {
                let expected_len = usize::try_from(placement.object_bytes)
                    .map_err(|_| "ec_spire placement object_bytes exceeds usize".to_owned())?;
                if raw.len() != expected_len {
                    return Err(format!(
                        "ec_spire object byte length mismatch: placement {}, tuple {}",
                        placement.object_bytes,
                        raw.len()
                    ));
                }
            }
            PARTITION_OBJECT_FORMAT_VERSION_V2 => {
                let meta = SpireLeafPartitionObjectV2Meta::decode(raw)?;
                if u64::from(placement.object_bytes) != meta.object_bytes_total {
                    return Err(format!(
                        "ec_spire placement object_bytes {} does not match leaf V2 total {}",
                        placement.object_bytes, meta.object_bytes_total
                    ));
                }
                header = meta.header;
            }
            other => {
                return Err(format!(
                    "ec_spire unsupported partition object format version: {other}"
                ));
            }
        }
        if header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match object pid {}",
                placement.pid, header.pid
            ));
        }
        if header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match object version {}",
                placement.object_version, header.object_version
            ));
        }
        if header.published_epoch_backref == 0 || header.published_epoch_backref > placement.epoch {
            return Err(format!(
                "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(header)
    }

    pub(super) fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String> {
        let raw = self.read_object_bytes(placement)?;
        let object = SpireRoutingPartitionObject::decode(raw)?;
        if object.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match object pid {}",
                placement.pid, object.header.pid
            ));
        }
        if object.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match object version {}",
                placement.object_version, object.header.object_version
            ));
        }
        if object.header.published_epoch_backref == 0
            || object.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                object.header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(object)
    }

    pub(super) fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String> {
        let raw = self.read_object_bytes(placement)?;
        let object = SpireDeltaPartitionObject::decode(raw)?;
        if object.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match object pid {}",
                placement.pid, object.header.pid
            ));
        }
        if object.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match object version {}",
                placement.object_version, object.header.object_version
            ));
        }
        if object.header.published_epoch_backref == 0
            || object.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                object.header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(object)
    }

    fn read_object_bytes(&self, placement: &SpirePlacementEntry) -> Result<&[u8], String> {
        self.validate_local_available_placement(placement)?;
        let raw = self.read_raw_tuple(placement.object_tid)?;
        let expected_len = usize::try_from(placement.object_bytes)
            .map_err(|_| "ec_spire placement object_bytes exceeds usize".to_owned())?;
        if raw.len() != expected_len {
            return Err(format!(
                "ec_spire object byte length mismatch: placement {}, tuple {}",
                placement.object_bytes,
                raw.len()
            ));
        }
        Ok(raw)
    }

    fn read_raw_tuple(&self, tid: ItemPointer) -> Result<&[u8], String> {
        let page = self
            .pages
            .get_page(tid.block_number)
            .ok_or_else(|| format!("ec_spire object block {} not found", tid.block_number))?;
        page.raw_tuple(tid)
    }

    fn validate_local_available_placement(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<(), String> {
        if placement.node_id != SPIRE_LOCAL_NODE_ID {
            return Err(format!(
                "ec_spire local object store cannot read node_id {}",
                placement.node_id
            ));
        }
        if placement.local_store_id != SPIRE_SINGLE_LOCAL_STORE_ID {
            return Err(format!(
                "ec_spire local object store cannot read local_store_id {}",
                placement.local_store_id
            ));
        }
        if placement.store_relid != self.store_relid {
            return Err(format!(
                "ec_spire placement store_relid {} does not match local store relid {}",
                placement.store_relid, self.store_relid
            ));
        }
        if placement.state != SpirePlacementState::Available {
            return Err(format!(
                "ec_spire local object store cannot read {:?} placement",
                placement.state
            ));
        }
        Ok(())
    }
}

impl SpireObjectReader for SpireLocalObjectStore {
    fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String> {
        SpireLocalObjectStore::read_object_header(self, placement)
    }

    fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String> {
        SpireLocalObjectStore::read_routing_object(self, placement)
    }

    fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String> {
        SpireLocalObjectStore::read_leaf_object(self, placement)
    }

    fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String> {
        SpireLocalObjectStore::read_leaf_object_v2(self, placement)
    }

    fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String> {
        SpireLocalObjectStore::read_delta_object(self, placement)
    }
}

pub(super) struct SpireRelationObjectStore {
    index_relation: pg_sys::Relation,
    store_relid: u32,
}

impl SpireRelationObjectStore {
    pub(super) unsafe fn for_index_relation(
        index_relation: pg_sys::Relation,
    ) -> Result<Self, String> {
        if index_relation.is_null() {
            return Err("ec_spire relation object store needs a valid relation".to_owned());
        }
        let relation_oid = unsafe { (*index_relation).rd_id };
        if relation_oid == pg_sys::InvalidOid {
            return Err("ec_spire relation object store relid is invalid".to_owned());
        }
        let store_relid = relation_oid.into();
        Ok(Self {
            index_relation,
            store_relid,
        })
    }

    pub(super) unsafe fn insert_routing_object(
        &self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire relation object store epoch 0 is invalid".to_owned());
        }
        let mut durable_object = object.clone();
        durable_object.header.published_epoch_backref = epoch;
        let encoded = durable_object.encode()?;
        let object_bytes = u32::try_from(encoded.len())
            .map_err(|_| "ec_spire partition object length exceeds u32".to_owned())?;
        let object_tid = unsafe { page::append_object_tuple(self.index_relation, &encoded)? };
        let placement = SpirePlacementEntry::local_single_store_available(
            epoch,
            durable_object.header.pid,
            self.store_relid,
            durable_object.header.object_version,
            object_tid,
            object_bytes,
        );
        placement.encode()?;
        Ok(placement)
    }

    pub(super) unsafe fn insert_leaf_object_v2_from_rows(
        &self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignments: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        if epoch == 0 {
            return Err("ec_spire relation object store epoch 0 is invalid".to_owned());
        }
        validate_leaf_assignments(assignments)?;
        let assignment_count = u32::try_from(assignments.len())
            .map_err(|_| "ec_spire leaf V2 assignment count exceeds u32".to_owned())?;
        let (payload_format, payload_stride) = leaf_v2_payload_layout(assignments)?;
        let max_segment_rows = leaf_v2_max_segment_rows(
            pg_sys::BLCKSZ as usize,
            payload_stride,
            LEAF_V2_LOCAL_VEC_ID_STRIDE,
        )?;
        let segment_count = if assignments.is_empty() {
            0
        } else {
            let count = assignments
                .len()
                .checked_add(max_segment_rows - 1)
                .and_then(|value| value.checked_div(max_segment_rows))
                .ok_or_else(|| "ec_spire leaf V2 segment count overflow".to_owned())?;
            u32::try_from(count)
                .map_err(|_| "ec_spire leaf V2 segment count exceeds u32".to_owned())?
        };

        let provisional_first_segment = if assignments.is_empty() {
            ItemPointer::INVALID
        } else {
            ItemPointer {
                block_number: 1,
                offset_number: 1,
            }
        };
        let provisional_meta = SpireLeafPartitionObjectV2Meta::new(
            pid,
            object_version,
            parent_pid,
            assignment_count,
            payload_format,
            u32::try_from(payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds u32".to_owned())?,
            segment_count,
            provisional_first_segment,
            1,
            epoch,
        )?;

        let mut next_segment_locator = ItemPointer::INVALID;
        let mut segment_bytes_total = 0_u64;
        for segment_index in (0..usize::try_from(segment_count).unwrap_or(0)).rev() {
            let row_base = segment_index
                .checked_mul(max_segment_rows)
                .ok_or_else(|| "ec_spire leaf V2 row_base overflow".to_owned())?;
            let rows_end = assignments.len().min(row_base + max_segment_rows);
            let segment = SpireLeafPartitionObjectV2Segment::new(
                &provisional_meta,
                u32::try_from(segment_index)
                    .map_err(|_| "ec_spire leaf V2 segment index exceeds u32".to_owned())?,
                u32::try_from(row_base)
                    .map_err(|_| "ec_spire leaf V2 row_base exceeds u32".to_owned())?,
                next_segment_locator,
                &assignments[row_base..rows_end],
            )?;
            let encoded_segment = segment.encode(&provisional_meta)?;
            segment_bytes_total =
                segment_bytes_total
                    .checked_add(u64::try_from(encoded_segment.len()).map_err(|_| {
                        "ec_spire leaf V2 segment byte length exceeds u64".to_owned()
                    })?)
                    .ok_or_else(|| "ec_spire leaf V2 object byte length overflow".to_owned())?;
            next_segment_locator =
                unsafe { page::append_object_tuple(self.index_relation, &encoded_segment)? };
        }

        let first_segment_locator = if assignments.is_empty() {
            ItemPointer::INVALID
        } else {
            next_segment_locator
        };
        let meta_bytes_len = PARTITION_OBJECT_HEADER_BYTES
            .checked_add(LEAF_V2_META_BODY_BYTES)
            .ok_or_else(|| "ec_spire leaf V2 meta byte length overflow".to_owned())?;
        let object_bytes_total = segment_bytes_total
            .checked_add(
                u64::try_from(meta_bytes_len)
                    .map_err(|_| "ec_spire leaf V2 meta byte length exceeds u64".to_owned())?,
            )
            .ok_or_else(|| "ec_spire leaf V2 object byte length overflow".to_owned())?;
        let object_bytes = u32::try_from(object_bytes_total)
            .map_err(|_| "ec_spire leaf V2 object length exceeds u32".to_owned())?;
        let meta = SpireLeafPartitionObjectV2Meta::new(
            pid,
            object_version,
            parent_pid,
            assignment_count,
            payload_format,
            u32::try_from(payload_stride)
                .map_err(|_| "ec_spire leaf V2 payload stride exceeds u32".to_owned())?,
            segment_count,
            first_segment_locator,
            object_bytes_total,
            epoch,
        )?;
        let encoded_meta = meta.encode()?;
        let meta_tid = unsafe { page::append_object_tuple(self.index_relation, &encoded_meta)? };
        let placement = SpirePlacementEntry::local_single_store_available(
            epoch,
            pid,
            self.store_relid,
            object_version,
            meta_tid,
            object_bytes,
        );
        placement.encode()?;
        Ok(placement)
    }

    pub(super) unsafe fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String> {
        let raw = unsafe { self.read_object_bytes(placement)? };
        let object = SpireRoutingPartitionObject::decode(&raw)?;
        if object.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match object pid {}",
                placement.pid, object.header.pid
            ));
        }
        if object.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match object version {}",
                placement.object_version, object.header.object_version
            ));
        }
        if object.header.published_epoch_backref == 0
            || object.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                object.header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(object)
    }

    pub(super) unsafe fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String> {
        self.validate_local_available_placement(placement)?;
        let raw_meta =
            unsafe { page::read_object_tuple(self.index_relation, placement.object_tid)? };
        let meta = SpireLeafPartitionObjectV2Meta::decode(&raw_meta)?;
        if meta.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match leaf V2 pid {}",
                placement.pid, meta.header.pid
            ));
        }
        if meta.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match leaf V2 version {}",
                placement.object_version, meta.header.object_version
            ));
        }
        if meta.header.published_epoch_backref == 0
            || meta.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire leaf V2 published epoch backref {} is not valid for placement epoch {}",
                meta.header.published_epoch_backref, placement.epoch
            ));
        }
        if u64::from(placement.object_bytes) != meta.object_bytes_total {
            return Err(format!(
                "ec_spire placement object_bytes {} does not match leaf V2 total {}",
                placement.object_bytes, meta.object_bytes_total
            ));
        }

        let segment_count = usize::try_from(meta.segment_count)
            .map_err(|_| "ec_spire leaf V2 segment count exceeds usize".to_owned())?;
        let mut segments = Vec::with_capacity(segment_count);
        let mut next_locator = meta.first_segment_locator;
        for _ in 0..segment_count {
            if next_locator == ItemPointer::INVALID {
                return Err("ec_spire leaf V2 segment chain ended early".to_owned());
            }
            let raw_segment =
                unsafe { page::read_object_tuple(self.index_relation, next_locator)? };
            let segment = SpireLeafPartitionObjectV2Segment::decode(&raw_segment, &meta)?;
            next_locator = segment.next_segment_locator;
            segments.push(segment);
        }
        if next_locator != ItemPointer::INVALID {
            return Err("ec_spire leaf V2 segment chain has trailing locator".to_owned());
        }
        SpireLeafPartitionObjectV2::new(meta, segments)
    }

    pub(super) unsafe fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String> {
        self.validate_local_available_placement(placement)?;
        let raw = unsafe { page::read_object_tuple(self.index_relation, placement.object_tid)? };
        let (mut header, format_version, _) =
            SpirePartitionObjectHeader::decode_prefix_with_format_version(&raw)?;
        match format_version {
            PARTITION_OBJECT_FORMAT_VERSION_V1 => {
                let expected_len = usize::try_from(placement.object_bytes)
                    .map_err(|_| "ec_spire placement object_bytes exceeds usize".to_owned())?;
                if raw.len() != expected_len {
                    return Err(format!(
                        "ec_spire object byte length mismatch: placement {}, tuple {}",
                        placement.object_bytes,
                        raw.len()
                    ));
                }
            }
            PARTITION_OBJECT_FORMAT_VERSION_V2 => {
                let meta = SpireLeafPartitionObjectV2Meta::decode(&raw)?;
                if u64::from(placement.object_bytes) != meta.object_bytes_total {
                    return Err(format!(
                        "ec_spire placement object_bytes {} does not match leaf V2 total {}",
                        placement.object_bytes, meta.object_bytes_total
                    ));
                }
                header = meta.header;
            }
            other => {
                return Err(format!(
                    "ec_spire unsupported partition object format version: {other}"
                ));
            }
        }
        if header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match object pid {}",
                placement.pid, header.pid
            ));
        }
        if header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match object version {}",
                placement.object_version, header.object_version
            ));
        }
        if header.published_epoch_backref == 0 || header.published_epoch_backref > placement.epoch {
            return Err(format!(
                "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(header)
    }

    pub(super) unsafe fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String> {
        let raw = unsafe { self.read_object_bytes(placement)? };
        let object = SpireLeafPartitionObject::decode(&raw)?;
        if object.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match object pid {}",
                placement.pid, object.header.pid
            ));
        }
        if object.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match object version {}",
                placement.object_version, object.header.object_version
            ));
        }
        if object.header.published_epoch_backref == 0
            || object.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                object.header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(object)
    }

    pub(super) unsafe fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String> {
        let raw = unsafe { self.read_object_bytes(placement)? };
        let object = SpireDeltaPartitionObject::decode(&raw)?;
        if object.header.pid != placement.pid {
            return Err(format!(
                "ec_spire placement pid {} does not match object pid {}",
                placement.pid, object.header.pid
            ));
        }
        if object.header.object_version != placement.object_version {
            return Err(format!(
                "ec_spire placement object_version {} does not match object version {}",
                placement.object_version, object.header.object_version
            ));
        }
        if object.header.published_epoch_backref == 0
            || object.header.published_epoch_backref > placement.epoch
        {
            return Err(format!(
                "ec_spire object published epoch backref {} is not valid for placement epoch {}",
                object.header.published_epoch_backref, placement.epoch
            ));
        }
        Ok(object)
    }

    unsafe fn read_object_bytes(&self, placement: &SpirePlacementEntry) -> Result<Vec<u8>, String> {
        self.validate_local_available_placement(placement)?;
        let raw = unsafe { page::read_object_tuple(self.index_relation, placement.object_tid)? };
        let expected_len = usize::try_from(placement.object_bytes)
            .map_err(|_| "ec_spire placement object_bytes exceeds usize".to_owned())?;
        if raw.len() != expected_len {
            return Err(format!(
                "ec_spire object byte length mismatch: placement {}, tuple {}",
                placement.object_bytes,
                raw.len()
            ));
        }
        Ok(raw)
    }

    fn validate_local_available_placement(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<(), String> {
        if placement.node_id != SPIRE_LOCAL_NODE_ID {
            return Err(format!(
                "ec_spire relation object store cannot read node_id {}",
                placement.node_id
            ));
        }
        if placement.local_store_id != SPIRE_SINGLE_LOCAL_STORE_ID {
            return Err(format!(
                "ec_spire relation object store cannot read local_store_id {}",
                placement.local_store_id
            ));
        }
        if placement.store_relid != self.store_relid {
            return Err(format!(
                "ec_spire placement store_relid {} does not match relation store relid {}",
                placement.store_relid, self.store_relid
            ));
        }
        if placement.state != SpirePlacementState::Available {
            return Err(format!(
                "ec_spire relation object store cannot read {:?} placement",
                placement.state
            ));
        }
        Ok(())
    }
}

impl SpireObjectReader for SpireRelationObjectStore {
    fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String> {
        unsafe { SpireRelationObjectStore::read_object_header(self, placement) }
    }

    fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String> {
        unsafe { SpireRelationObjectStore::read_routing_object(self, placement) }
    }

    fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String> {
        unsafe { SpireRelationObjectStore::read_leaf_object(self, placement) }
    }

    fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String> {
        unsafe { SpireRelationObjectStore::read_leaf_object_v2(self, placement) }
    }

    fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String> {
        unsafe { SpireRelationObjectStore::read_delta_object(self, placement) }
    }
}

fn validate_vec_id_bytes(bytes: &[u8]) -> Result<(), String> {
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
            let local_vec_seq =
                u64::from_le_bytes(bytes[1..].try_into().expect("local vec_id sequence bytes"));
            if local_vec_seq == 0 {
                return Err("ec_spire local vec_id sequence 0 is invalid".to_owned());
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
    Ok(())
}

pub(super) fn is_visible_primary_assignment_flags(flags: u16) -> bool {
    let blocked_flags = SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA
        | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
        | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
        | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
    flags & SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 && flags & blocked_flags == 0
}

pub(super) fn is_visible_primary_assignment(assignment: &SpireLeafAssignmentRow) -> bool {
    is_visible_primary_assignment_flags(assignment.flags)
}

pub(super) fn is_visible_primary_assignment_ref(
    assignment: &SpireLeafAssignmentRowRef<'_>,
) -> bool {
    is_visible_primary_assignment_flags(assignment.flags)
}

pub(super) fn is_delete_delta_assignment_flags(flags: u16) -> bool {
    flags & SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE != 0
}

pub(super) fn is_delete_delta_assignment(assignment: &SpireLeafAssignmentRow) -> bool {
    is_delete_delta_assignment_flags(assignment.flags)
}

fn validate_leaf_v2_header(
    header: &SpirePartitionObjectHeader,
    expected_flag: u32,
) -> Result<(), String> {
    header.validate_for_format_version(PARTITION_OBJECT_FORMAT_VERSION_V2)?;
    if header.kind != SpirePartitionObjectKind::Leaf {
        return Err(format!(
            "ec_spire leaf V2 header kind must be Leaf, got {:?}",
            header.kind
        ));
    }
    if header.child_count != 0 {
        return Err(format!(
            "ec_spire leaf V2 child_count must be 0, got {}",
            header.child_count
        ));
    }
    if header.flags != expected_flag {
        return Err(format!(
            "ec_spire leaf V2 header flags mismatch: got {:#x}, expected {expected_flag:#x}",
            header.flags
        ));
    }
    Ok(())
}

fn validate_leaf_v2_locator(locator: ItemPointer, label: &str) -> Result<(), String> {
    if locator == ItemPointer::INVALID {
        return Ok(());
    }
    if locator.offset_number == 0 {
        return Err(format!(
            "ec_spire leaf V2 {label} locator offset 0 is invalid"
        ));
    }
    if locator.block_number == u32::MAX || locator.offset_number == u16::MAX {
        return Err(format!(
            "ec_spire leaf V2 {label} locator is partially invalid"
        ));
    }
    Ok(())
}

fn encode_leaf_v2_local_vec_id(vec_id: &SpireVecId, out: &mut Vec<u8>) -> Result<(), String> {
    let Some(local_vec_seq) = vec_id.local_sequence() else {
        return Err("ec_spire leaf V2 Phase 1 requires local vec_id rows".to_owned());
    };
    out.push(SPIRE_LOCAL_VEC_ID_DISCRIMINATOR);
    out.extend_from_slice(&local_vec_seq.to_le_bytes());
    out.resize(
        out.len() + (LEAF_V2_LOCAL_VEC_ID_STRIDE - 1 - size_of::<u64>()),
        0,
    );
    Ok(())
}

fn decode_leaf_v2_local_vec_id(input: &[u8]) -> Result<u64, String> {
    if input.len() != LEAF_V2_LOCAL_VEC_ID_STRIDE {
        return Err(format!(
            "ec_spire leaf V2 local vec_id stride mismatch: got {}, expected {LEAF_V2_LOCAL_VEC_ID_STRIDE}",
            input.len()
        ));
    }
    if input[0] != SPIRE_LOCAL_VEC_ID_DISCRIMINATOR {
        return Err(format!(
            "ec_spire leaf V2 local vec_id discriminator mismatch: got {:#x}",
            input[0]
        ));
    }
    if input[1 + size_of::<u64>()..].iter().any(|byte| *byte != 0) {
        return Err("ec_spire leaf V2 local vec_id padding must be zero".to_owned());
    }
    let local_vec_seq = u64::from_le_bytes(
        input[1..1 + size_of::<u64>()]
            .try_into()
            .expect("local vec_id bytes"),
    );
    if local_vec_seq == 0 {
        return Err("ec_spire leaf V2 local vec_id sequence 0 is invalid".to_owned());
    }
    Ok(local_vec_seq)
}

fn leaf_v2_payload_layout(assignments: &[SpireLeafAssignmentRow]) -> Result<(u8, usize), String> {
    let Some(first) = assignments.first() else {
        return Ok((SPIRE_PAYLOAD_FORMAT_NONE, 0));
    };
    validate_leaf_assignment(first)?;
    if first.payload_format == SPIRE_PAYLOAD_FORMAT_NONE {
        return Err("ec_spire non-empty leaf V2 payload format must not be NONE".to_owned());
    }
    let payload_format = first.payload_format;
    let payload_stride = first.encoded_payload.len();
    if payload_stride == 0 {
        return Err("ec_spire non-empty leaf V2 payload stride 0 is invalid".to_owned());
    }
    for assignment in assignments {
        validate_leaf_assignment(assignment)?;
        if assignment.payload_format != payload_format {
            return Err(format!(
                "ec_spire leaf V2 requires one payload format per object: got {}, expected {payload_format}",
                assignment.payload_format
            ));
        }
        if assignment.encoded_payload.len() != payload_stride {
            return Err(format!(
                "ec_spire leaf V2 requires one payload stride per object: got {}, expected {payload_stride}",
                assignment.encoded_payload.len()
            ));
        }
        if assignment.vec_id.local_sequence().is_none() {
            return Err("ec_spire leaf V2 Phase 1 requires local vec_id rows".to_owned());
        }
    }
    Ok((payload_format, payload_stride))
}

fn leaf_v2_max_segment_rows(
    page_size: usize,
    payload_stride: usize,
    vec_id_stride: usize,
) -> Result<usize, String> {
    let row_bytes = size_of::<u16>()
        .checked_add(vec_id_stride)
        .and_then(|len| len.checked_add(ITEM_POINTER_BYTES))
        .and_then(|len| len.checked_add(size_of::<f32>()))
        .and_then(|len| len.checked_add(payload_stride))
        .ok_or_else(|| "ec_spire leaf V2 row byte length overflow".to_owned())?;
    if row_bytes == 0 {
        return Ok(usize::MAX);
    }
    let fixed_bytes = PARTITION_OBJECT_HEADER_BYTES
        .checked_add(LEAF_V2_SEGMENT_PREFIX_BYTES)
        .ok_or_else(|| "ec_spire leaf V2 segment fixed byte length overflow".to_owned())?;
    let usable_bytes = usable_page_bytes(page_size);
    if fixed_bytes >= usable_bytes {
        return Err(format!(
            "ec_spire leaf V2 segment fixed bytes {fixed_bytes} exceed page usable bytes {usable_bytes}"
        ));
    }
    let mut rows = (usable_bytes - fixed_bytes) / row_bytes;
    while rows > 0
        && !element_or_neighbor_tuple_fits(fixed_bytes + row_bytes.saturating_mul(rows), page_size)
    {
        rows -= 1;
    }
    if rows == 0 {
        return Err(format!(
            "ec_spire leaf V2 row bytes {row_bytes} do not fit page size {page_size}"
        ));
    }
    Ok(rows)
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

fn validate_assignment_payload_format(payload_format: u8) -> Result<(), String> {
    match payload_format {
        SPIRE_PAYLOAD_FORMAT_NONE
        | SPIRE_PAYLOAD_FORMAT_TURBOQUANT
        | SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN
        | SPIRE_PAYLOAD_FORMAT_RABITQ => Ok(()),
        other => Err(format!(
            "ec_spire unknown assignment payload_format: {other}"
        )),
    }
}

fn validate_scored_assignment_payload(assignment: &SpireLeafAssignmentRow) -> Result<(), String> {
    if assignment.payload_format == SPIRE_PAYLOAD_FORMAT_NONE {
        return Err("ec_spire scored assignment payload_format must not be 0".to_owned());
    }
    if assignment.encoded_payload.is_empty() {
        return Err("ec_spire scored assignment payload must not be empty".to_owned());
    }
    Ok(())
}

fn validate_delta_assignment(assignment: &SpireLeafAssignmentRow) -> Result<(), String> {
    assignment.validate_wire_shape()?;
    let is_insert = assignment.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0;
    let is_delete = assignment.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE != 0;
    if is_insert == is_delete {
        return Err(
            "ec_spire delta assignment must set exactly one insert/delete delta flag".to_owned(),
        );
    }
    if assignment.flags & SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0 {
        return Err("ec_spire delta assignment cannot be a boundary replica in Phase 1".to_owned());
    }
    if assignment.flags & SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR != 0 {
        return Err("ec_spire delta assignment cannot be a stale locator".to_owned());
    }
    if is_insert && assignment.flags & SPIRE_ASSIGNMENT_FLAG_PRIMARY == 0 {
        return Err("ec_spire insert delta assignment must be primary".to_owned());
    }
    if is_insert && assignment.flags & SPIRE_ASSIGNMENT_FLAG_TOMBSTONE != 0 {
        return Err("ec_spire insert delta assignment cannot be tombstoned".to_owned());
    }
    if is_insert {
        validate_scored_assignment_payload(assignment)?;
    }
    if is_delete && assignment.flags & SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 {
        return Err("ec_spire delete delta assignment cannot be primary".to_owned());
    }
    if is_delete && assignment.flags & SPIRE_ASSIGNMENT_FLAG_TOMBSTONE == 0 {
        return Err("ec_spire delete delta assignment must be tombstoned".to_owned());
    }
    if is_delete && assignment.payload_format != SPIRE_PAYLOAD_FORMAT_NONE {
        return Err("ec_spire delete delta assignment payload format must be 0".to_owned());
    }
    if is_delete && assignment.gamma != 0.0 {
        return Err("ec_spire delete delta assignment gamma must be 0".to_owned());
    }
    if is_delete && !assignment.encoded_payload.is_empty() {
        return Err("ec_spire delete delta assignment payload must be empty".to_owned());
    }
    Ok(())
}

fn validate_leaf_assignments(assignments: &[SpireLeafAssignmentRow]) -> Result<(), String> {
    let mut seen_vec_ids = HashSet::new();
    for assignment in assignments {
        validate_leaf_assignment(assignment)?;
        if !seen_vec_ids.insert(assignment.vec_id.clone()) {
            return Err(
                "ec_spire leaf partition object contains duplicate vec_id assignments".to_owned(),
            );
        }
    }
    Ok(())
}

fn validate_leaf_assignment(assignment: &SpireLeafAssignmentRow) -> Result<(), String> {
    assignment.validate_wire_shape()?;
    if assignment.flags & (SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE)
        != 0
    {
        return Err("ec_spire leaf partition object assignment cannot set delta flags".to_owned());
    }
    let role_flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY
        | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA
        | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
        | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
    if assignment.flags & role_flags == 0 {
        return Err("ec_spire leaf partition object assignment must set a role flag".to_owned());
    }
    if assignment.flags & (SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA)
        != 0
    {
        validate_scored_assignment_payload(assignment)?;
    }
    Ok(())
}

fn validate_delta_assignments(assignments: &[SpireLeafAssignmentRow]) -> Result<(), String> {
    let mut seen_vec_ids = HashSet::new();
    for assignment in assignments {
        validate_delta_assignment(assignment)?;
        if !seen_vec_ids.insert(assignment.vec_id.clone()) {
            return Err(
                "ec_spire delta partition object contains duplicate vec_id assignments".to_owned(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::meta::SpirePlacementState;
    use super::{
        decode_leaf_v2_local_vec_id, is_delete_delta_assignment, is_visible_primary_assignment,
        is_visible_primary_assignment_ref, SpireDeltaPartitionObject, SpireLeafAssignmentRow,
        SpireLeafPartitionObject, SpireLocalObjectStore, SpirePartitionObjectHeader,
        SpirePartitionObjectKind, SpireRoutingChildEntry, SpireRoutingPartitionObject, SpireVecId,
        SpireVecIdKind, SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
        SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR, SPIRE_LOCAL_VEC_ID_DISCRIMINATOR,
        SPIRE_PAYLOAD_FORMAT_NONE, SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
        SPIRE_PAYLOAD_FORMAT_TURBOQUANT, SPIRE_VEC_ID_MAX_BYTES,
    };
    use crate::storage::page::{ItemPointer, ITEM_POINTER_BYTES};

    fn routing_children() -> Vec<SpireRoutingChildEntry> {
        vec![
            SpireRoutingChildEntry {
                centroid_index: 0,
                child_pid: 17,
                centroid: vec![1.0, 0.0],
            },
            SpireRoutingChildEntry {
                centroid_index: 1,
                child_pid: 18,
                centroid: vec![-1.0, 0.0],
            },
        ]
    }

    fn leaf_v2_assignment(local_vec_seq: u64, payload_len: usize) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(local_vec_seq),
            heap_tid: ItemPointer {
                block_number: 100 + local_vec_seq as u32,
                offset_number: local_vec_seq as u16,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: local_vec_seq as f32 / 10.0,
            encoded_payload: vec![local_vec_seq as u8; payload_len],
        }
    }

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
        assert!(SpireVecId::from_bytes(SpireVecId::local(0).as_bytes()).is_err());
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
            published_epoch_backref: 7,
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
            published_epoch_backref: 7,
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
    fn partition_object_constructors_reject_invalid_header_identity() {
        let row = leaf_v2_assignment(1, 8);

        assert!(SpireLeafPartitionObject::new(0, 3, 0, vec![row.clone()]).is_err());
        assert!(SpireLeafPartitionObject::new(17, 0, 0, vec![row]).is_err());
        assert!(SpireDeltaPartitionObject::new(0, 4, 17, Vec::new()).is_err());
        assert!(SpireDeltaPartitionObject::new(19, 0, 17, Vec::new()).is_err());
        assert!(SpireRoutingPartitionObject::root(0, 3, 2, routing_children()).is_err());
        assert!(SpireRoutingPartitionObject::root(11, 0, 2, routing_children()).is_err());
    }

    #[test]
    fn routing_partition_object_round_trips_root_children() {
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();

        let decoded = SpireRoutingPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.kind, SpirePartitionObjectKind::Root);
        assert_eq!(decoded.header.level, 1);
        assert_eq!(decoded.header.parent_pid, 0);
        assert_eq!(decoded.header.child_count, 2);
        assert_eq!(decoded.header.assignment_count, 0);
        assert_eq!(decoded.child_pids[0], 17);
        assert_eq!(decoded.child_centroid(1).unwrap(), &[-1.0, 0.0]);
    }

    #[test]
    fn routing_partition_object_round_trips_internal_children() {
        let object =
            SpireRoutingPartitionObject::internal(12, 4, 2, 11, 2, routing_children()).unwrap();

        let decoded = SpireRoutingPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.kind, SpirePartitionObjectKind::Internal);
        assert_eq!(decoded.header.level, 2);
        assert_eq!(decoded.header.parent_pid, 11);
    }

    #[test]
    fn routing_partition_object_rejects_invalid_shape() {
        assert!(SpireRoutingPartitionObject::root(11, 3, 0, routing_children()).is_err());
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, Vec::new()).is_err());
        assert!(SpireRoutingPartitionObject::internal(12, 4, 2, 0, 2, routing_children()).is_err());

        let mut children = routing_children();
        children[1].centroid_index = 7;
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, children).is_err());

        let mut children = routing_children();
        children[0].child_pid = 0;
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, children).is_err());

        let mut children = routing_children();
        children[0].centroid = vec![1.0];
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, children).is_err());

        let mut children = routing_children();
        children[0].centroid = vec![f32::NAN, 0.0];
        assert!(SpireRoutingPartitionObject::root(11, 3, 2, children).is_err());
    }

    #[test]
    fn routing_partition_object_rejects_corrupt_header_and_body() {
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let mut wrong_count = object.clone();
        wrong_count.header.child_count = 1;
        assert!(wrong_count.encode().is_err());

        let mut wrong_kind = object.clone();
        wrong_kind.header.kind = SpirePartitionObjectKind::Leaf;
        assert!(wrong_kind.encode().is_err());

        let mut encoded = object.encode().unwrap();
        encoded.truncate(encoded.len() - 1);
        assert!(SpireRoutingPartitionObject::decode(&encoded).is_err());

        let mut encoded = object.encode().unwrap();
        encoded[48] = 1;
        assert!(SpireRoutingPartitionObject::decode(&encoded).is_err());
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
            payload_format: SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
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
            payload_format: SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
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
    fn assignment_row_ref_prefix_decoder_borrows_payload() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(123),
            heap_tid: ItemPointer {
                block_number: 44,
                offset_number: 7,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN,
            gamma: 1.25,
            encoded_payload: vec![4, 5, 6],
        };
        let mut encoded = row.encode().unwrap();
        encoded.extend_from_slice(&[9, 8]);

        let (row_ref, tail) = SpireLeafAssignmentRow::decode_prefix_ref(&encoded).unwrap();

        assert_eq!(row_ref.flags, row.flags);
        assert_eq!(row_ref.vec_id.local_sequence(), Some(123));
        assert_eq!(row_ref.heap_tid, row.heap_tid);
        assert_eq!(row_ref.payload_format, row.payload_format);
        assert_eq!(row_ref.gamma, row.gamma);
        assert_eq!(row_ref.encoded_payload, row.encoded_payload.as_slice());
        assert_eq!(row_ref.to_owned(), row);
        assert_eq!(tail, &[9, 8]);
    }

    #[test]
    fn assignment_visibility_helpers_match_primary_and_delta_semantics() {
        let mut row = leaf_v2_assignment(1, 8);
        assert!(is_visible_primary_assignment(&row));
        let encoded = row.encode().unwrap();
        let (row_ref, _) =
            SpireLeafAssignmentRow::decode_prefix_ref(&encoded).expect("row ref decodes");
        assert!(is_visible_primary_assignment_ref(&row_ref));

        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
        assert!(!is_visible_primary_assignment(&row));
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE;
        assert!(!is_visible_primary_assignment(&row));
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
        assert!(!is_visible_primary_assignment(&row));

        row.flags = SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        row.payload_format = SPIRE_PAYLOAD_FORMAT_NONE;
        row.gamma = 0.0;
        row.encoded_payload.clear();
        assert!(is_delete_delta_assignment(&row));
        assert!(!is_visible_primary_assignment(&row));
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
            payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        };
        assert!(row.encode().is_err());

        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY;
        row.heap_tid = ItemPointer::INVALID;
        assert!(row.encode().is_err());
    }

    #[test]
    fn assignment_row_rejects_unknown_payload_format() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 1,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        };

        let mut invalid = row.clone();
        invalid.payload_format = 99;
        assert!(invalid.encode().is_err());

        let mut encoded = row.encode().unwrap();
        let payload_format_offset = 3 + row.vec_id.as_bytes().len() + ITEM_POINTER_BYTES;
        encoded[payload_format_offset] = 99;
        assert!(SpireLeafAssignmentRow::decode(&encoded).is_err());
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
            payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
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
    fn leaf_partition_object_v2_store_segments_large_leaf() {
        let mut store = SpireLocalObjectStore::new(99, 512).unwrap();
        let assignments = (1..=13)
            .map(|local_vec_seq| leaf_v2_assignment(local_vec_seq, 64))
            .collect::<Vec<_>>();

        let placement = store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &assignments)
            .unwrap();
        let decoded = store.read_leaf_object_v2(&placement).unwrap();
        let header = store.read_object_header(&placement).unwrap();

        assert_eq!(header.kind, SpirePartitionObjectKind::Leaf);
        assert_eq!(header.pid, 17);
        assert_eq!(header.object_version, 3);
        assert_eq!(header.published_epoch_backref, 7);
        assert_eq!(header.parent_pid, 5);
        assert_eq!(header.assignment_count, assignments.len() as u32);
        assert_eq!(decoded.meta.header.pid, 17);
        assert_eq!(decoded.meta.header.object_version, 3);
        assert_eq!(decoded.meta.header.published_epoch_backref, 7);
        assert_eq!(decoded.meta.header.parent_pid, 5);
        assert_eq!(
            decoded.meta.header.assignment_count,
            assignments.len() as u32
        );
        assert_eq!(
            decoded.meta.object_bytes_total,
            u64::from(placement.object_bytes)
        );
        assert!(decoded.meta.segment_count > 1);
        assert_ne!(decoded.meta.first_segment_locator, ItemPointer::INVALID);
        assert!(store.page_count() > 1);

        let mut decoded_row_count = 0_usize;
        for segment in &decoded.segments {
            decoded_row_count += segment.flags.len();
            assert_eq!(segment.flags.len(), segment.heap_tids.len());
            assert_eq!(segment.flags.len(), segment.gammas.len());
            assert_eq!(segment.vec_ids.len(), segment.flags.len() * 16);
            assert_eq!(segment.payloads.len(), segment.flags.len() * 64);
        }
        assert_eq!(decoded_row_count, assignments.len());

        let column_segments = decoded.column_segments().unwrap();
        assert_eq!(column_segments.len(), decoded.segments.len());
        assert_eq!(
            column_segments[0].payload_format,
            SPIRE_PAYLOAD_FORMAT_TURBOQUANT
        );
        assert_eq!(column_segments[0].payload_stride, 64);
        assert_eq!(column_segments[0].vec_id_kind, SpireVecIdKind::LocalU64);
        assert_eq!(column_segments[0].vec_id_stride, 16);
        let first_row = column_segments[0].row(0).unwrap();
        assert_eq!(first_row.row_index, 0);
        assert_eq!(first_row.flags, SPIRE_ASSIGNMENT_FLAG_PRIMARY);
        assert_eq!(first_row.local_vec_seq().unwrap(), 1);
        assert_eq!(first_row.heap_tid, assignments[0].heap_tid);
        assert_eq!(first_row.gamma, assignments[0].gamma);
        assert_eq!(first_row.encoded_payload, assignments[0].encoded_payload);

        let first_vec_id = decode_leaf_v2_local_vec_id(&decoded.segments[0].vec_ids[0..16])
            .expect("first local vec_id decodes");
        assert_eq!(first_vec_id, 1);
        let last = decoded.segments.last().expect("segments are present");
        let last_columns = column_segments.last().expect("column segments are present");
        let last_row = last_columns.row(last_columns.row_count() - 1).unwrap();
        assert_eq!(last_row.local_vec_seq().unwrap(), 13);
        assert_eq!(last_row.heap_tid, assignments[12].heap_tid);
        assert!(last_columns.row(last_columns.row_count()).is_err());
        let last_vec_id_start = (last.flags.len() - 1) * 16;
        let last_vec_id =
            decode_leaf_v2_local_vec_id(&last.vec_ids[last_vec_id_start..last_vec_id_start + 16])
                .expect("last local vec_id decodes");
        assert_eq!(last_vec_id, 13);
        assert_eq!(last.next_segment_locator, ItemPointer::INVALID);
    }

    #[test]
    fn leaf_partition_object_v2_store_preserves_empty_leaf_without_segments() {
        let mut store = SpireLocalObjectStore::new(99, 512).unwrap();

        let placement = store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &[])
            .unwrap();
        let decoded = store.read_leaf_object_v2(&placement).unwrap();

        assert_eq!(decoded.meta.header.assignment_count, 0);
        assert_eq!(decoded.meta.segment_count, 0);
        assert_eq!(decoded.meta.first_segment_locator, ItemPointer::INVALID);
        assert_eq!(decoded.meta.payload_format, SPIRE_PAYLOAD_FORMAT_NONE);
        assert!(decoded.segments.is_empty());
        assert!(decoded.column_segments().unwrap().is_empty());
    }

    #[test]
    fn leaf_partition_object_v2_rejects_mixed_payload_or_global_vec_id() {
        let mut store = SpireLocalObjectStore::new(99, 512).unwrap();
        let mut mixed_stride = vec![leaf_v2_assignment(1, 8), leaf_v2_assignment(2, 16)];
        assert!(store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &mixed_stride)
            .is_err());

        mixed_stride[1] = leaf_v2_assignment(2, 8);
        mixed_stride[1].payload_format = SPIRE_PAYLOAD_FORMAT_PQ_FASTSCAN;
        assert!(store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &mixed_stride)
            .is_err());

        let mut global_row = leaf_v2_assignment(1, 8);
        global_row.vec_id = SpireVecId::global(&[9, 9, 9]).unwrap();
        assert!(store
            .insert_leaf_object_v2_from_rows(7, 17, 3, 5, &[global_row])
            .is_err());
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
    fn leaf_partition_object_rejects_delta_flags() {
        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };

        assert!(SpireLeafPartitionObject::new(17, 3, 0, vec![row.clone()]).is_err());

        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 17,
            object_version: 3,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 0,
            child_count: 0,
            assignment_count: 1,
            flags: 0,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&row.encode().unwrap());

        assert!(SpireLeafPartitionObject::decode(&encoded).is_err());
    }

    #[test]
    fn leaf_partition_object_rejects_missing_role_flags() {
        let row = SpireLeafAssignmentRow {
            flags: 0,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };

        assert!(SpireLeafPartitionObject::new(17, 3, 0, vec![row.clone()]).is_err());

        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 17,
            object_version: 3,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 0,
            child_count: 0,
            assignment_count: 1,
            flags: 0,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&row.encode().unwrap());

        assert!(SpireLeafPartitionObject::decode(&encoded).is_err());
    }

    #[test]
    fn leaf_partition_object_rejects_scored_assignments_without_payload() {
        let valid_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_TURBOQUANT,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };

        let mut row = valid_row.clone();
        row.payload_format = SPIRE_PAYLOAD_FORMAT_NONE;
        assert!(SpireLeafPartitionObject::new(17, 3, 0, vec![row]).is_err());

        row = valid_row;
        row.encoded_payload.clear();
        assert!(SpireLeafPartitionObject::new(17, 3, 0, vec![row]).is_err());
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

    #[test]
    fn leaf_partition_object_rejects_duplicate_vec_ids() {
        let primary_row = SpireLeafAssignmentRow {
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
        let boundary_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 2,
            },
            payload_format: 1,
            gamma: 0.75,
            encoded_payload: vec![3, 4],
        };

        assert!(SpireLeafPartitionObject::new(
            17,
            3,
            0,
            vec![primary_row.clone(), boundary_row.clone()],
        )
        .is_err());

        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Leaf,
            pid: 17,
            object_version: 3,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 0,
            child_count: 0,
            assignment_count: 2,
            flags: 0,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&primary_row.encode().unwrap());
        encoded.extend_from_slice(&boundary_row.encode().unwrap());

        assert!(SpireLeafPartitionObject::decode(&encoded).is_err());
    }

    #[test]
    fn delta_partition_object_round_trips_insert_and_delete_rows() {
        let object = SpireDeltaPartitionObject::new(
            19,
            4,
            17,
            vec![
                SpireLeafAssignmentRow {
                    flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
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
                    flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
                    vec_id: SpireVecId::local(2),
                    heap_tid: ItemPointer {
                        block_number: 10,
                        offset_number: 2,
                    },
                    payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
                    gamma: 0.0,
                    encoded_payload: Vec::new(),
                },
            ],
        )
        .unwrap();

        let decoded = SpireDeltaPartitionObject::decode(&object.encode().unwrap()).unwrap();

        assert_eq!(decoded, object);
        assert_eq!(decoded.header.kind, SpirePartitionObjectKind::Delta);
        assert_eq!(decoded.header.pid, 19);
        assert_eq!(decoded.header.parent_pid, 17);
        assert_eq!(decoded.header.assignment_count, 2);
    }

    #[test]
    fn delta_partition_object_rejects_invalid_header_shape() {
        assert!(SpireDeltaPartitionObject::new(19, 4, 0, Vec::new()).is_err());

        let row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };
        let mut object = SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).unwrap();

        object.header.kind = SpirePartitionObjectKind::Leaf;
        assert!(object.encode().is_err());

        object.header.kind = SpirePartitionObjectKind::Delta;
        object.header.child_count = 1;
        assert!(object.encode().is_err());

        object.header.child_count = 0;
        object.header.assignment_count = 2;
        assert!(object.encode().is_err());
    }

    #[test]
    fn delta_partition_object_rejects_invalid_delta_flags() {
        let valid_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };

        let mut row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY
            | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
            | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.flags = SPIRE_ASSIGNMENT_FLAG_PRIMARY
            | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
            | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row.clone();
        row.payload_format = SPIRE_PAYLOAD_FORMAT_NONE;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_row;
        row.encoded_payload.clear();
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());
    }

    #[test]
    fn delta_partition_object_rejects_delete_payloads() {
        let valid_delete_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        };

        let mut row = valid_delete_row.clone();
        row.payload_format = 1;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_delete_row.clone();
        row.gamma = 0.5;
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());

        row = valid_delete_row;
        row.encoded_payload = vec![1, 2];
        assert!(SpireDeltaPartitionObject::new(19, 4, 17, vec![row]).is_err());
    }

    #[test]
    fn delta_partition_object_rejects_duplicate_vec_ids() {
        let insert_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2],
        };
        let delete_row = SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            vec_id: SpireVecId::local(1),
            heap_tid: ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            payload_format: SPIRE_PAYLOAD_FORMAT_NONE,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        };

        assert!(SpireDeltaPartitionObject::new(
            19,
            4,
            17,
            vec![insert_row.clone(), delete_row.clone()],
        )
        .is_err());

        let header = SpirePartitionObjectHeader {
            kind: SpirePartitionObjectKind::Delta,
            pid: 19,
            object_version: 4,
            published_epoch_backref: 7,
            level: 0,
            parent_pid: 17,
            child_count: 0,
            assignment_count: 2,
            flags: 0,
        };
        let mut encoded = header.encode().unwrap();
        encoded.extend_from_slice(&insert_row.encode().unwrap());
        encoded.extend_from_slice(&delete_row.encode().unwrap());

        assert!(SpireDeltaPartitionObject::decode(&encoded).is_err());
    }

    #[test]
    fn local_object_store_writes_and_reads_leaf_object() {
        let object = SpireLeafPartitionObject::new(
            17,
            3,
            0,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(1),
                heap_tid: ItemPointer {
                    block_number: 10,
                    offset_number: 1,
                },
                payload_format: 1,
                gamma: 0.5,
                encoded_payload: vec![1, 2],
            }],
        )
        .unwrap();
        let expected_bytes = object.encode().unwrap().len() as u32;
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let placement = store.insert_leaf_object(7, &object).unwrap();
        let decoded = store.read_leaf_object(&placement).unwrap();
        let mut expected = object.clone();
        expected.header.published_epoch_backref = 7;

        assert_eq!(decoded, expected);
        assert_eq!(placement.epoch, 7);
        assert_eq!(placement.pid, 17);
        assert_eq!(placement.object_version, 3);
        assert_eq!(placement.store_relid, 12345);
        assert_eq!(placement.node_id, 0);
        assert_eq!(placement.local_store_id, 0);
        assert_eq!(placement.object_bytes, expected_bytes);
        assert_eq!(store.page_count(), 1);
    }

    #[test]
    fn local_object_store_writes_and_reads_delta_object() {
        let object = SpireDeltaPartitionObject::new(
            19,
            4,
            17,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
                vec_id: SpireVecId::local(1),
                heap_tid: ItemPointer {
                    block_number: 10,
                    offset_number: 1,
                },
                payload_format: 1,
                gamma: 0.5,
                encoded_payload: vec![1, 2],
            }],
        )
        .unwrap();
        let expected_bytes = object.encode().unwrap().len() as u32;
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let placement = store.insert_delta_object(7, &object).unwrap();
        let decoded = store.read_delta_object(&placement).unwrap();
        let mut expected = object.clone();
        expected.header.published_epoch_backref = 7;

        assert_eq!(decoded, expected);
        assert_eq!(placement.epoch, 7);
        assert_eq!(placement.pid, 19);
        assert_eq!(placement.object_version, 4);
        assert_eq!(placement.store_relid, 12345);
        assert_eq!(placement.node_id, 0);
        assert_eq!(placement.local_store_id, 0);
        assert_eq!(placement.object_bytes, expected_bytes);
        assert_eq!(store.page_count(), 1);
    }

    #[test]
    fn local_object_store_writes_and_reads_routing_object() {
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let expected_bytes = object.encode().unwrap().len() as u32;
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let placement = store.insert_routing_object(7, &object).unwrap();
        let decoded = store.read_routing_object(&placement).unwrap();
        let mut expected = object.clone();
        expected.header.published_epoch_backref = 7;

        assert_eq!(decoded, expected);
        assert_eq!(placement.epoch, 7);
        assert_eq!(placement.pid, 11);
        assert_eq!(placement.object_version, 3);
        assert_eq!(placement.store_relid, 12345);
        assert_eq!(placement.node_id, 0);
        assert_eq!(placement.local_store_id, 0);
        assert_eq!(placement.object_bytes, expected_bytes);
        assert_eq!(store.page_count(), 1);
    }

    #[test]
    fn local_object_store_reads_object_headers_for_dispatch() {
        let leaf = SpireLeafPartitionObject::new(17, 3, 0, Vec::new()).unwrap();
        let delta = SpireDeltaPartitionObject::new(19, 4, 17, Vec::new()).unwrap();
        let root = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        let leaf_placement = store.insert_leaf_object(7, &leaf).unwrap();
        let delta_placement = store.insert_delta_object(7, &delta).unwrap();
        let root_placement = store.insert_routing_object(7, &root).unwrap();
        let leaf_header = store.read_object_header(&leaf_placement).unwrap();
        let delta_header = store.read_object_header(&delta_placement).unwrap();
        let root_header = store.read_object_header(&root_placement).unwrap();

        assert_eq!(leaf_header.kind, SpirePartitionObjectKind::Leaf);
        assert_eq!(leaf_header.pid, 17);
        assert_eq!(leaf_header.object_version, 3);
        assert_eq!(leaf_header.published_epoch_backref, 7);
        assert_eq!(delta_header.kind, SpirePartitionObjectKind::Delta);
        assert_eq!(delta_header.pid, 19);
        assert_eq!(delta_header.object_version, 4);
        assert_eq!(delta_header.published_epoch_backref, 7);
        assert_eq!(root_header.kind, SpirePartitionObjectKind::Root);
        assert_eq!(root_header.pid, 11);
        assert_eq!(root_header.object_version, 3);
        assert_eq!(root_header.published_epoch_backref, 7);
    }

    #[test]
    fn local_object_store_rejects_invalid_store_and_epoch() {
        assert!(SpireLocalObjectStore::with_default_page_size(0).is_err());
        let object = SpireLeafPartitionObject::new(17, 3, 0, Vec::new()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

        assert!(store.insert_leaf_object(0, &object).is_err());
        assert_eq!(store.page_count(), 1);

        let delta = SpireDeltaPartitionObject::new(19, 4, 17, Vec::new()).unwrap();
        assert!(store.insert_delta_object(0, &delta).is_err());

        let root = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        assert!(store.insert_routing_object(0, &root).is_err());
    }

    #[test]
    fn local_object_store_rejects_mismatched_placement() {
        let object = SpireLeafPartitionObject::new(17, 3, 0, Vec::new()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let placement = store.insert_leaf_object(7, &object).unwrap();

        let mut wrong_store = placement;
        wrong_store.store_relid = 54321;
        assert!(store.read_leaf_object(&wrong_store).is_err());

        let mut stale = placement;
        stale.state = SpirePlacementState::Stale;
        assert!(store.read_leaf_object(&stale).is_err());

        let mut wrong_pid = placement;
        wrong_pid.pid = 99;
        assert!(store.read_leaf_object(&wrong_pid).is_err());

        let mut wrong_bytes = placement;
        wrong_bytes.object_bytes += 1;
        assert!(store.read_leaf_object(&wrong_bytes).is_err());

        let mut wrong_epoch = placement;
        wrong_epoch.epoch = 6;
        assert!(store.read_leaf_object(&wrong_epoch).is_err());
    }

    #[test]
    fn local_object_store_rejects_mismatched_delta_placement() {
        let object = SpireDeltaPartitionObject::new(19, 4, 17, Vec::new()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let placement = store.insert_delta_object(7, &object).unwrap();

        let mut wrong_store = placement;
        wrong_store.store_relid = 54321;
        assert!(store.read_delta_object(&wrong_store).is_err());

        let mut stale = placement;
        stale.state = SpirePlacementState::Stale;
        assert!(store.read_delta_object(&stale).is_err());

        let mut wrong_pid = placement;
        wrong_pid.pid = 99;
        assert!(store.read_delta_object(&wrong_pid).is_err());

        let mut wrong_version = placement;
        wrong_version.object_version = 99;
        assert!(store.read_delta_object(&wrong_version).is_err());

        let mut wrong_bytes = placement;
        wrong_bytes.object_bytes += 1;
        assert!(store.read_delta_object(&wrong_bytes).is_err());
    }

    #[test]
    fn local_object_store_rejects_mismatched_routing_placement() {
        let object = SpireRoutingPartitionObject::root(11, 3, 2, routing_children()).unwrap();
        let mut store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let placement = store.insert_routing_object(7, &object).unwrap();

        let mut wrong_store = placement;
        wrong_store.store_relid = 54321;
        assert!(store.read_routing_object(&wrong_store).is_err());

        let mut stale = placement;
        stale.state = SpirePlacementState::Stale;
        assert!(store.read_routing_object(&stale).is_err());

        let mut wrong_pid = placement;
        wrong_pid.pid = 99;
        assert!(store.read_routing_object(&wrong_pid).is_err());

        let mut wrong_bytes = placement;
        wrong_bytes.object_bytes += 1;
        assert!(store.read_routing_object(&wrong_bytes).is_err());
    }
}
