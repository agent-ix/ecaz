//! Root/control metadata, epoch, and placement-map codecs.

use super::assign::{SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID};
use crate::storage::page::{ItemPointer, ITEM_POINTER_BYTES};

pub(super) const SPIRE_LOCAL_NODE_ID: u32 = 0;
pub(super) const SPIRE_SINGLE_LOCAL_STORE_ID: u32 = 0;
pub(super) const SPIRE_MIN_EPOCH_RETENTION_SECS: u32 = 10 * 60;
pub(super) const SPIRE_FAILED_EPOCH_RETENTION_SECS: u32 = 60 * 60;
pub(super) const SPIRE_MAX_RETAINED_RETIRED_EPOCHS: u16 = 2;

const META_FORMAT_VERSION: u16 = 1;
const ROOT_CONTROL_MAGIC: u32 = 0x4352_5345; // "ESRC" as little-endian bytes.
const ROOT_CONTROL_STATE_BYTES: usize = 4 + 2 + 2 + 8 + 8 + 8 + ITEM_POINTER_BYTES * 3;
const PLACEMENT_DIRECTORY_MAGIC: u32 = 0x4450_5345; // "ESPD" as little-endian bytes.
const PLACEMENT_DIRECTORY_HEADER_BYTES: usize = 4 + 2 + 2 + 8 + 4;
const OBJECT_MANIFEST_MAGIC: u32 = 0x4d4f_5345; // "ESOM" as little-endian bytes.
const OBJECT_MANIFEST_HEADER_BYTES: usize = 4 + 2 + 2 + 8 + 4;
const PLACEMENT_ENTRY_BYTES: usize = 2 + 1 + 1 + 8 + 8 + 4 + 4 + 4 + 8 + ITEM_POINTER_BYTES + 4;
const EPOCH_MANIFEST_BYTES: usize = 2 + 1 + 1 + 8 + 8 + 8 + 8;
const MANIFEST_ENTRY_BYTES: usize = 2 + 2 + 8 + 8 + 8 + ITEM_POINTER_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireRootControlState {
    pub(super) active_epoch: u64,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
    pub(super) epoch_manifest_tid: ItemPointer,
    pub(super) object_manifest_tid: ItemPointer,
    pub(super) placement_directory_tid: ItemPointer,
}

impl SpireRootControlState {
    pub(super) fn empty() -> Self {
        Self {
            active_epoch: 0,
            next_pid: SPIRE_FIRST_PID,
            next_local_vec_seq: SPIRE_FIRST_LOCAL_VEC_SEQ,
            epoch_manifest_tid: ItemPointer::INVALID,
            object_manifest_tid: ItemPointer::INVALID,
            placement_directory_tid: ItemPointer::INVALID,
        }
    }

    pub(super) fn published(
        active_epoch: u64,
        next_pid: u64,
        next_local_vec_seq: u64,
        epoch_manifest_tid: ItemPointer,
        object_manifest_tid: ItemPointer,
        placement_directory_tid: ItemPointer,
    ) -> Result<Self, String> {
        let state = Self {
            active_epoch,
            next_pid,
            next_local_vec_seq,
            epoch_manifest_tid,
            object_manifest_tid,
            placement_directory_tid,
        };
        state.validate()?;
        Ok(state)
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = Vec::with_capacity(ROOT_CONTROL_STATE_BYTES);
        out.extend_from_slice(&ROOT_CONTROL_MAGIC.to_le_bytes());
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.active_epoch.to_le_bytes());
        out.extend_from_slice(&self.next_pid.to_le_bytes());
        out.extend_from_slice(&self.next_local_vec_seq.to_le_bytes());
        self.epoch_manifest_tid.encode_into(&mut out);
        self.object_manifest_tid.encode_into(&mut out);
        self.placement_directory_tid.encode_into(&mut out);
        debug_assert_eq!(out.len(), ROOT_CONTROL_STATE_BYTES);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != ROOT_CONTROL_STATE_BYTES {
            return Err(format!(
                "ec_spire root/control state length mismatch: got {}, expected {ROOT_CONTROL_STATE_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("root magic bytes"));
        if magic != ROOT_CONTROL_MAGIC {
            return Err(format!(
                "ec_spire invalid root/control state magic: {magic:#x}"
            ));
        }
        validate_format_version(&input[4..6])?;
        let reserved = u16::from_le_bytes(input[6..8].try_into().expect("reserved bytes"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire root/control state reserved bytes must be zero, got {reserved}"
            ));
        }

        let state = Self {
            active_epoch: u64::from_le_bytes(input[8..16].try_into().expect("active epoch bytes")),
            next_pid: u64::from_le_bytes(input[16..24].try_into().expect("next pid bytes")),
            next_local_vec_seq: u64::from_le_bytes(
                input[24..32].try_into().expect("next local vec seq bytes"),
            ),
            epoch_manifest_tid: ItemPointer::decode(&input[32..38])?,
            object_manifest_tid: ItemPointer::decode(&input[38..44])?,
            placement_directory_tid: ItemPointer::decode(&input[44..50])?,
        };
        state.validate()?;
        Ok(state)
    }

    fn validate(&self) -> Result<(), String> {
        if self.next_pid == 0 {
            return Err("ec_spire root/control next_pid 0 is invalid".to_owned());
        }
        if self.next_local_vec_seq == 0 {
            return Err("ec_spire root/control next_local_vec_seq 0 is invalid".to_owned());
        }
        if self.active_epoch == 0 {
            if self.epoch_manifest_tid != ItemPointer::INVALID
                || self.object_manifest_tid != ItemPointer::INVALID
                || self.placement_directory_tid != ItemPointer::INVALID
            {
                return Err(
                    "ec_spire empty root/control state must not reference active manifests"
                        .to_owned(),
                );
            }
            return Ok(());
        }
        if self.epoch_manifest_tid == ItemPointer::INVALID {
            return Err("ec_spire active root/control state needs an epoch manifest".to_owned());
        }
        if self.object_manifest_tid == ItemPointer::INVALID {
            return Err("ec_spire active root/control state needs an object manifest".to_owned());
        }
        if self.placement_directory_tid == ItemPointer::INVALID {
            return Err(
                "ec_spire active root/control state needs a placement directory".to_owned(),
            );
        }
        Ok(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpirePlacementState {
    Available = 1,
    Stale = 2,
    Unavailable = 3,
    Skipped = 4,
}

impl SpirePlacementState {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Available),
            2 => Ok(Self::Stale),
            3 => Ok(Self::Unavailable),
            4 => Ok(Self::Skipped),
            other => Err(format!("ec_spire invalid placement state: {other}")),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireEpochState {
    Building = 1,
    Published = 2,
    Retired = 3,
    Failed = 4,
}

impl SpireEpochState {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Building),
            2 => Ok(Self::Published),
            3 => Ok(Self::Retired),
            4 => Ok(Self::Failed),
            other => Err(format!("ec_spire invalid epoch state: {other}")),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireConsistencyMode {
    Strict = 1,
    Degraded = 2,
}

impl SpireConsistencyMode {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Strict),
            2 => Ok(Self::Degraded),
            other => Err(format!("ec_spire invalid consistency mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePlacementEntry {
    pub(super) epoch: u64,
    pub(super) pid: u64,
    pub(super) node_id: u32,
    pub(super) local_store_id: u32,
    pub(super) store_relid: u32,
    pub(super) object_version: u64,
    pub(super) object_tid: ItemPointer,
    pub(super) object_bytes: u32,
    pub(super) state: SpirePlacementState,
}

impl SpirePlacementEntry {
    pub(super) fn local_single_store(
        epoch: u64,
        pid: u64,
        store_relid: u32,
        object_version: u64,
        object_tid: ItemPointer,
        object_bytes: u32,
    ) -> Self {
        Self {
            epoch,
            pid,
            node_id: SPIRE_LOCAL_NODE_ID,
            local_store_id: SPIRE_SINGLE_LOCAL_STORE_ID,
            store_relid,
            object_version,
            object_tid,
            object_bytes,
            state: SpirePlacementState::Available,
        }
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = Vec::with_capacity(PLACEMENT_ENTRY_BYTES);
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.push(self.state as u8);
        out.push(0);
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&self.pid.to_le_bytes());
        out.extend_from_slice(&self.node_id.to_le_bytes());
        out.extend_from_slice(&self.local_store_id.to_le_bytes());
        out.extend_from_slice(&self.store_relid.to_le_bytes());
        out.extend_from_slice(&self.object_version.to_le_bytes());
        self.object_tid.encode_into(&mut out);
        out.extend_from_slice(&self.object_bytes.to_le_bytes());
        debug_assert_eq!(out.len(), PLACEMENT_ENTRY_BYTES);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != PLACEMENT_ENTRY_BYTES {
            return Err(format!(
                "ec_spire placement entry length mismatch: got {}, expected {PLACEMENT_ENTRY_BYTES}",
                input.len()
            ));
        }
        validate_format_version(&input[0..2])?;
        if input[3] != 0 {
            return Err(format!(
                "ec_spire placement entry reserved byte must be zero, got {}",
                input[3]
            ));
        }

        let entry = Self {
            state: SpirePlacementState::decode(input[2])?,
            epoch: u64::from_le_bytes(input[4..12].try_into().expect("epoch bytes")),
            pid: u64::from_le_bytes(input[12..20].try_into().expect("pid bytes")),
            node_id: u32::from_le_bytes(input[20..24].try_into().expect("node id bytes")),
            local_store_id: u32::from_le_bytes(
                input[24..28].try_into().expect("local store id bytes"),
            ),
            store_relid: u32::from_le_bytes(input[28..32].try_into().expect("store relid bytes")),
            object_version: u64::from_le_bytes(
                input[32..40].try_into().expect("object version bytes"),
            ),
            object_tid: ItemPointer::decode(&input[40..46])?,
            object_bytes: u32::from_le_bytes(
                input[46..50].try_into().expect("object bytes length"),
            ),
        };
        entry.validate()?;
        Ok(entry)
    }

    fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 {
            return Err("ec_spire placement entry epoch 0 is invalid".to_owned());
        }
        if self.pid == 0 {
            return Err("ec_spire placement entry pid 0 is invalid".to_owned());
        }
        if self.store_relid == 0 {
            return Err("ec_spire placement entry store_relid 0 is invalid".to_owned());
        }
        if self.object_version == 0 {
            return Err("ec_spire placement entry object_version 0 is invalid".to_owned());
        }
        if self.object_tid == ItemPointer::INVALID {
            return Err("ec_spire placement entry object_tid must be valid".to_owned());
        }
        if self.object_bytes == 0 {
            return Err("ec_spire placement entry object_bytes must be non-zero".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpirePlacementDirectory {
    pub(super) epoch: u64,
    pub(super) entries: Vec<SpirePlacementEntry>,
}

impl SpirePlacementDirectory {
    pub(super) fn from_entries(
        epoch: u64,
        mut entries: Vec<SpirePlacementEntry>,
    ) -> Result<Self, String> {
        if epoch == 0 {
            return Err("ec_spire placement directory epoch 0 is invalid".to_owned());
        }
        entries.sort_by_key(|entry| entry.pid);
        let directory = Self { epoch, entries };
        directory.validate()?;
        Ok(directory)
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let entry_count = u32::try_from(self.entries.len())
            .map_err(|_| "ec_spire placement directory entry count exceeds u32".to_owned())?;

        let mut out = Vec::with_capacity(
            PLACEMENT_DIRECTORY_HEADER_BYTES + self.entries.len() * PLACEMENT_ENTRY_BYTES,
        );
        out.extend_from_slice(&PLACEMENT_DIRECTORY_MAGIC.to_le_bytes());
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&entry_count.to_le_bytes());
        for entry in &self.entries {
            out.extend_from_slice(&entry.encode()?);
        }
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() < PLACEMENT_DIRECTORY_HEADER_BYTES {
            return Err(format!(
                "ec_spire placement directory too short: got {}, expected at least {PLACEMENT_DIRECTORY_HEADER_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("directory magic bytes"));
        if magic != PLACEMENT_DIRECTORY_MAGIC {
            return Err(format!(
                "ec_spire invalid placement directory magic: {magic:#x}"
            ));
        }
        validate_format_version(&input[4..6])?;
        let reserved = u16::from_le_bytes(input[6..8].try_into().expect("reserved bytes"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire placement directory reserved bytes must be zero, got {reserved}"
            ));
        }
        let epoch = u64::from_le_bytes(input[8..16].try_into().expect("directory epoch bytes"));
        let entry_count =
            u32::from_le_bytes(input[16..20].try_into().expect("directory count bytes")) as usize;
        let expected_len = entry_count
            .checked_mul(PLACEMENT_ENTRY_BYTES)
            .and_then(|entry_bytes| entry_bytes.checked_add(PLACEMENT_DIRECTORY_HEADER_BYTES))
            .ok_or_else(|| "ec_spire placement directory length overflow".to_owned())?;
        if input.len() != expected_len {
            return Err(format!(
                "ec_spire placement directory length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }

        let mut entries = Vec::with_capacity(entry_count);
        let mut cursor = PLACEMENT_DIRECTORY_HEADER_BYTES;
        for _ in 0..entry_count {
            let entry =
                SpirePlacementEntry::decode(&input[cursor..cursor + PLACEMENT_ENTRY_BYTES])?;
            entries.push(entry);
            cursor += PLACEMENT_ENTRY_BYTES;
        }
        Self::from_entries(epoch, entries)
    }

    pub(super) fn get(&self, pid: u64) -> Option<&SpirePlacementEntry> {
        self.entries
            .binary_search_by_key(&pid, |entry| entry.pid)
            .ok()
            .map(|index| &self.entries[index])
    }

    fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 {
            return Err("ec_spire placement directory epoch 0 is invalid".to_owned());
        }
        let mut previous_pid = None;
        for entry in &self.entries {
            entry.validate()?;
            if entry.epoch != self.epoch {
                return Err(format!(
                    "ec_spire placement directory epoch mismatch: directory {}, entry {}",
                    self.epoch, entry.epoch
                ));
            }
            if let Some(previous_pid) = previous_pid {
                if entry.pid == previous_pid {
                    return Err(format!(
                        "ec_spire placement directory duplicate pid: {}",
                        entry.pid
                    ));
                }
                if entry.pid < previous_pid {
                    return Err("ec_spire placement directory entries must be sorted".to_owned());
                }
            }
            previous_pid = Some(entry.pid);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireEpochManifest {
    pub(super) epoch: u64,
    pub(super) state: SpireEpochState,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) active_query_count: u64,
}

impl SpireEpochManifest {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = Vec::with_capacity(EPOCH_MANIFEST_BYTES);
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.push(self.state as u8);
        out.push(self.consistency_mode as u8);
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&self.published_at_micros.to_le_bytes());
        out.extend_from_slice(&self.retain_until_micros.to_le_bytes());
        out.extend_from_slice(&self.active_query_count.to_le_bytes());
        debug_assert_eq!(out.len(), EPOCH_MANIFEST_BYTES);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != EPOCH_MANIFEST_BYTES {
            return Err(format!(
                "ec_spire epoch manifest length mismatch: got {}, expected {EPOCH_MANIFEST_BYTES}",
                input.len()
            ));
        }
        validate_format_version(&input[0..2])?;

        let manifest = Self {
            state: SpireEpochState::decode(input[2])?,
            consistency_mode: SpireConsistencyMode::decode(input[3])?,
            epoch: u64::from_le_bytes(input[4..12].try_into().expect("epoch bytes")),
            published_at_micros: i64::from_le_bytes(
                input[12..20].try_into().expect("published_at bytes"),
            ),
            retain_until_micros: i64::from_le_bytes(
                input[20..28].try_into().expect("retain_until bytes"),
            ),
            active_query_count: u64::from_le_bytes(
                input[28..36].try_into().expect("active query count bytes"),
            ),
        };
        manifest.validate()?;
        Ok(manifest)
    }

    fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 {
            return Err("ec_spire epoch manifest epoch 0 is invalid".to_owned());
        }
        if matches!(
            self.state,
            SpireEpochState::Published | SpireEpochState::Retired
        ) && self.published_at_micros <= 0
        {
            return Err(
                "ec_spire published or retired epoch must have a publish timestamp".to_owned(),
            );
        }
        if matches!(self.state, SpireEpochState::Retired)
            && self.retain_until_micros < self.published_at_micros
        {
            return Err("ec_spire retired epoch retain_until precedes published_at".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireManifestEntry {
    pub(super) epoch: u64,
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) placement_tid: ItemPointer,
}

impl SpireManifestEntry {
    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = Vec::with_capacity(MANIFEST_ENTRY_BYTES);
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&self.pid.to_le_bytes());
        out.extend_from_slice(&self.object_version.to_le_bytes());
        self.placement_tid.encode_into(&mut out);
        debug_assert_eq!(out.len(), MANIFEST_ENTRY_BYTES);
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != MANIFEST_ENTRY_BYTES {
            return Err(format!(
                "ec_spire manifest entry length mismatch: got {}, expected {MANIFEST_ENTRY_BYTES}",
                input.len()
            ));
        }
        validate_format_version(&input[0..2])?;
        let reserved = u16::from_le_bytes(input[2..4].try_into().expect("reserved bytes"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire manifest entry reserved bytes must be zero, got {reserved}"
            ));
        }

        let entry = Self {
            epoch: u64::from_le_bytes(input[4..12].try_into().expect("epoch bytes")),
            pid: u64::from_le_bytes(input[12..20].try_into().expect("pid bytes")),
            object_version: u64::from_le_bytes(
                input[20..28].try_into().expect("object version bytes"),
            ),
            placement_tid: ItemPointer::decode(&input[28..34])?,
        };
        entry.validate()?;
        Ok(entry)
    }

    fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 {
            return Err("ec_spire manifest entry epoch 0 is invalid".to_owned());
        }
        if self.pid == 0 {
            return Err("ec_spire manifest entry pid 0 is invalid".to_owned());
        }
        if self.object_version == 0 {
            return Err("ec_spire manifest entry object_version 0 is invalid".to_owned());
        }
        if self.placement_tid == ItemPointer::INVALID {
            return Err("ec_spire manifest entry placement_tid must be valid".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireObjectManifest {
    pub(super) epoch: u64,
    pub(super) entries: Vec<SpireManifestEntry>,
}

impl SpireObjectManifest {
    pub(super) fn from_entries(
        epoch: u64,
        mut entries: Vec<SpireManifestEntry>,
    ) -> Result<Self, String> {
        if epoch == 0 {
            return Err("ec_spire object manifest epoch 0 is invalid".to_owned());
        }
        entries.sort_by_key(|entry| entry.pid);
        let manifest = Self { epoch, entries };
        manifest.validate()?;
        Ok(manifest)
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let entry_count = u32::try_from(self.entries.len())
            .map_err(|_| "ec_spire object manifest entry count exceeds u32".to_owned())?;

        let mut out = Vec::with_capacity(
            OBJECT_MANIFEST_HEADER_BYTES + self.entries.len() * MANIFEST_ENTRY_BYTES,
        );
        out.extend_from_slice(&OBJECT_MANIFEST_MAGIC.to_le_bytes());
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.epoch.to_le_bytes());
        out.extend_from_slice(&entry_count.to_le_bytes());
        for entry in &self.entries {
            out.extend_from_slice(&entry.encode()?);
        }
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() < OBJECT_MANIFEST_HEADER_BYTES {
            return Err(format!(
                "ec_spire object manifest too short: got {}, expected at least {OBJECT_MANIFEST_HEADER_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("manifest magic bytes"));
        if magic != OBJECT_MANIFEST_MAGIC {
            return Err(format!(
                "ec_spire invalid object manifest magic: {magic:#x}"
            ));
        }
        validate_format_version(&input[4..6])?;
        let reserved = u16::from_le_bytes(input[6..8].try_into().expect("reserved bytes"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire object manifest reserved bytes must be zero, got {reserved}"
            ));
        }
        let epoch = u64::from_le_bytes(input[8..16].try_into().expect("manifest epoch bytes"));
        let entry_count =
            u32::from_le_bytes(input[16..20].try_into().expect("manifest count bytes")) as usize;
        let expected_len = entry_count
            .checked_mul(MANIFEST_ENTRY_BYTES)
            .and_then(|entry_bytes| entry_bytes.checked_add(OBJECT_MANIFEST_HEADER_BYTES))
            .ok_or_else(|| "ec_spire object manifest length overflow".to_owned())?;
        if input.len() != expected_len {
            return Err(format!(
                "ec_spire object manifest length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }

        let mut entries = Vec::with_capacity(entry_count);
        let mut cursor = OBJECT_MANIFEST_HEADER_BYTES;
        for _ in 0..entry_count {
            let entry = SpireManifestEntry::decode(&input[cursor..cursor + MANIFEST_ENTRY_BYTES])?;
            entries.push(entry);
            cursor += MANIFEST_ENTRY_BYTES;
        }
        Self::from_entries(epoch, entries)
    }

    pub(super) fn get(&self, pid: u64) -> Option<&SpireManifestEntry> {
        self.entries
            .binary_search_by_key(&pid, |entry| entry.pid)
            .ok()
            .map(|index| &self.entries[index])
    }

    fn validate(&self) -> Result<(), String> {
        if self.epoch == 0 {
            return Err("ec_spire object manifest epoch 0 is invalid".to_owned());
        }
        let mut previous_pid = None;
        for entry in &self.entries {
            entry.validate()?;
            if entry.epoch != self.epoch {
                return Err(format!(
                    "ec_spire object manifest epoch mismatch: manifest {}, entry {}",
                    self.epoch, entry.epoch
                ));
            }
            if let Some(previous_pid) = previous_pid {
                if entry.pid == previous_pid {
                    return Err(format!(
                        "ec_spire object manifest duplicate pid: {}",
                        entry.pid
                    ));
                }
                if entry.pid < previous_pid {
                    return Err("ec_spire object manifest entries must be sorted".to_owned());
                }
            }
            previous_pid = Some(entry.pid);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpirePublishedEpochSnapshot<'a> {
    pub(super) epoch_manifest: &'a SpireEpochManifest,
    pub(super) object_manifest: &'a SpireObjectManifest,
    pub(super) placement_directory: &'a SpirePlacementDirectory,
}

impl<'a> SpirePublishedEpochSnapshot<'a> {
    pub(super) fn new(
        epoch_manifest: &'a SpireEpochManifest,
        object_manifest: &'a SpireObjectManifest,
        placement_directory: &'a SpirePlacementDirectory,
    ) -> Result<Self, String> {
        let snapshot = Self {
            epoch_manifest,
            object_manifest,
            placement_directory,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    fn validate(&self) -> Result<(), String> {
        self.epoch_manifest.validate()?;
        self.object_manifest.validate()?;
        self.placement_directory.validate()?;

        if self.epoch_manifest.state != SpireEpochState::Published {
            return Err("ec_spire published snapshot requires a published epoch".to_owned());
        }

        let epoch = self.epoch_manifest.epoch;
        if self.object_manifest.epoch != epoch {
            return Err(format!(
                "ec_spire published snapshot object manifest epoch mismatch: epoch {}, manifest {}",
                epoch, self.object_manifest.epoch
            ));
        }
        if self.placement_directory.epoch != epoch {
            return Err(format!(
                "ec_spire published snapshot placement directory epoch mismatch: epoch {}, directory {}",
                epoch, self.placement_directory.epoch
            ));
        }

        for entry in &self.object_manifest.entries {
            let placement = self.placement_directory.get(entry.pid).ok_or_else(|| {
                format!(
                    "ec_spire published snapshot missing placement for pid {}",
                    entry.pid
                )
            })?;

            if placement.object_version != entry.object_version {
                return Err(format!(
                    "ec_spire published snapshot object_version mismatch for pid {}: manifest {}, placement {}",
                    entry.pid, entry.object_version, placement.object_version
                ));
            }

            match (self.epoch_manifest.consistency_mode, placement.state) {
                (SpireConsistencyMode::Strict, SpirePlacementState::Available)
                | (SpireConsistencyMode::Degraded, SpirePlacementState::Available)
                | (SpireConsistencyMode::Degraded, SpirePlacementState::Unavailable)
                | (SpireConsistencyMode::Degraded, SpirePlacementState::Skipped) => {}
                (SpireConsistencyMode::Strict, state) => {
                    return Err(format!(
                        "ec_spire strict published snapshot requires available placement for pid {}: got {:?}",
                        entry.pid, state
                    ));
                }
                (SpireConsistencyMode::Degraded, SpirePlacementState::Stale) => {
                    return Err(format!(
                        "ec_spire degraded published snapshot cannot use stale placement for pid {}",
                        entry.pid
                    ));
                }
            }
        }

        Ok(())
    }
}

fn validate_format_version(input: &[u8]) -> Result<(), String> {
    let format_version = u16::from_le_bytes(input.try_into().expect("format version bytes"));
    if format_version != META_FORMAT_VERSION {
        return Err(format!(
            "ec_spire unsupported metadata format version: {format_version}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
        SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState,
        SpirePublishedEpochSnapshot, SpireRootControlState, SPIRE_FAILED_EPOCH_RETENTION_SECS,
        SPIRE_LOCAL_NODE_ID, SPIRE_MAX_RETAINED_RETIRED_EPOCHS, SPIRE_MIN_EPOCH_RETENTION_SECS,
        SPIRE_SINGLE_LOCAL_STORE_ID,
    };
    use crate::am::ec_spire::assign::{SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID};
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn published_epoch(epoch: u64, consistency_mode: SpireConsistencyMode) -> SpireEpochManifest {
        SpireEpochManifest {
            epoch,
            state: SpireEpochState::Published,
            consistency_mode,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        }
    }

    fn object_manifest(epoch: u64, pid: u64, object_version: u64) -> SpireObjectManifest {
        SpireObjectManifest::from_entries(
            epoch,
            vec![SpireManifestEntry {
                epoch,
                pid,
                object_version,
                placement_tid: tid(55, 4),
            }],
        )
        .unwrap()
    }

    fn placement_directory(
        epoch: u64,
        pid: u64,
        object_version: u64,
        state: SpirePlacementState,
    ) -> SpirePlacementDirectory {
        let mut placement = SpirePlacementEntry::local_single_store(
            epoch,
            pid,
            12345,
            object_version,
            tid(44, 2),
            4096,
        );
        placement.state = state;
        SpirePlacementDirectory::from_entries(epoch, vec![placement]).unwrap()
    }

    #[test]
    fn retention_defaults_match_phase0_design() {
        assert_eq!(SPIRE_MIN_EPOCH_RETENTION_SECS, 600);
        assert_eq!(SPIRE_FAILED_EPOCH_RETENTION_SECS, 3600);
        assert_eq!(SPIRE_MAX_RETAINED_RETIRED_EPOCHS, 2);
    }

    #[test]
    fn root_control_empty_state_round_trips() {
        let state = SpireRootControlState::empty();

        assert_eq!(state.active_epoch, 0);
        assert_eq!(state.next_pid, SPIRE_FIRST_PID);
        assert_eq!(state.next_local_vec_seq, SPIRE_FIRST_LOCAL_VEC_SEQ);
        assert_eq!(state.epoch_manifest_tid, ItemPointer::INVALID);
        assert_eq!(
            SpireRootControlState::decode(&state.encode().unwrap()).unwrap(),
            state
        );
    }

    #[test]
    fn root_control_published_state_round_trips() {
        let state =
            SpireRootControlState::published(7, 12, 100, tid(50, 1), tid(50, 2), tid(50, 3))
                .unwrap();

        assert_eq!(
            SpireRootControlState::decode(&state.encode().unwrap()).unwrap(),
            state
        );
    }

    #[test]
    fn root_control_rejects_invalid_cursors_and_manifest_refs() {
        assert!(
            SpireRootControlState::published(7, 0, 100, tid(50, 1), tid(50, 2), tid(50, 3))
                .is_err()
        );
        assert!(
            SpireRootControlState::published(7, 12, 0, tid(50, 1), tid(50, 2), tid(50, 3)).is_err()
        );
        assert!(SpireRootControlState::published(
            7,
            12,
            100,
            ItemPointer::INVALID,
            tid(50, 2),
            tid(50, 3),
        )
        .is_err());

        let mut empty = SpireRootControlState::empty();
        empty.epoch_manifest_tid = tid(50, 1);
        assert!(empty.encode().is_err());
    }

    #[test]
    fn root_control_rejects_corrupt_header() {
        let state = SpireRootControlState::empty();
        let mut encoded = state.encode().unwrap();

        encoded[0] = 0;
        assert!(SpireRootControlState::decode(&encoded).is_err());

        encoded = state.encode().unwrap();
        encoded[6] = 1;
        assert!(SpireRootControlState::decode(&encoded).is_err());
    }

    #[test]
    fn local_single_store_placement_uses_default_ids() {
        let entry = SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096);

        assert_eq!(entry.node_id, SPIRE_LOCAL_NODE_ID);
        assert_eq!(entry.local_store_id, SPIRE_SINGLE_LOCAL_STORE_ID);
        assert_eq!(entry.state, SpirePlacementState::Available);
    }

    #[test]
    fn placement_entry_round_trips() {
        let entry = SpirePlacementEntry {
            epoch: 7,
            pid: 11,
            node_id: 0,
            local_store_id: 2,
            store_relid: 12345,
            object_version: 3,
            object_tid: tid(44, 2),
            object_bytes: 8192,
            state: SpirePlacementState::Stale,
        };

        assert_eq!(
            SpirePlacementEntry::decode(&entry.encode().unwrap()).unwrap(),
            entry
        );
    }

    #[test]
    fn placement_entry_rejects_invalid_identity_and_locator() {
        let mut entry = SpirePlacementEntry::local_single_store(0, 11, 12345, 3, tid(44, 2), 4096);
        assert!(entry.encode().is_err());

        entry.epoch = 7;
        entry.pid = 0;
        assert!(entry.encode().is_err());

        entry.pid = 11;
        entry.store_relid = 0;
        assert!(entry.encode().is_err());

        entry.store_relid = 12345;
        entry.object_version = 0;
        assert!(entry.encode().is_err());

        entry.object_version = 3;
        entry.object_tid = ItemPointer::INVALID;
        assert!(entry.encode().is_err());

        entry.object_tid = tid(44, 2);
        entry.object_bytes = 0;
        assert!(entry.encode().is_err());
    }

    #[test]
    fn placement_entry_rejects_unknown_state_and_format() {
        let entry = SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096);
        let mut encoded = entry.encode().unwrap();

        encoded[2] = 99;
        assert!(SpirePlacementEntry::decode(&encoded).is_err());

        encoded[2] = SpirePlacementState::Available as u8;
        encoded[0] = 2;
        assert!(SpirePlacementEntry::decode(&encoded).is_err());
    }

    #[test]
    fn placement_directory_sorts_and_round_trips_entries() {
        let directory = SpirePlacementDirectory::from_entries(
            7,
            vec![
                SpirePlacementEntry::local_single_store(7, 21, 12345, 4, tid(45, 2), 4096),
                SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096),
            ],
        )
        .unwrap();

        let decoded = SpirePlacementDirectory::decode(&directory.encode().unwrap()).unwrap();

        assert_eq!(decoded, directory);
        assert_eq!(decoded.entries[0].pid, 11);
        assert_eq!(decoded.entries[1].pid, 21);
        assert_eq!(decoded.get(21).unwrap().object_version, 4);
        assert!(decoded.get(99).is_none());
    }

    #[test]
    fn placement_directory_rejects_duplicate_pid_and_epoch_mismatch() {
        assert!(SpirePlacementDirectory::from_entries(
            7,
            vec![
                SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096),
                SpirePlacementEntry::local_single_store(7, 11, 12345, 4, tid(45, 2), 4096),
            ],
        )
        .is_err());

        assert!(SpirePlacementDirectory::from_entries(
            7,
            vec![SpirePlacementEntry::local_single_store(
                8,
                11,
                12345,
                3,
                tid(44, 2),
                4096,
            )],
        )
        .is_err());
    }

    #[test]
    fn placement_directory_rejects_corrupt_header_and_length() {
        let directory = SpirePlacementDirectory::from_entries(
            7,
            vec![SpirePlacementEntry::local_single_store(
                7,
                11,
                12345,
                3,
                tid(44, 2),
                4096,
            )],
        )
        .unwrap();
        let mut encoded = directory.encode().unwrap();

        encoded[0] = 0;
        assert!(SpirePlacementDirectory::decode(&encoded).is_err());

        encoded = directory.encode().unwrap();
        encoded[6] = 1;
        assert!(SpirePlacementDirectory::decode(&encoded).is_err());

        encoded = directory.encode().unwrap();
        encoded.push(99);
        assert!(SpirePlacementDirectory::decode(&encoded).is_err());
    }

    #[test]
    fn epoch_manifest_round_trips() {
        let manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 2,
        };

        assert_eq!(
            SpireEpochManifest::decode(&manifest.encode().unwrap()).unwrap(),
            manifest
        );
    }

    #[test]
    fn epoch_manifest_allows_building_without_publish_timestamp() {
        let manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Building,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };

        assert_eq!(
            SpireEpochManifest::decode(&manifest.encode().unwrap()).unwrap(),
            manifest
        );
    }

    #[test]
    fn epoch_manifest_rejects_invalid_state_and_consistency_mode() {
        let manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Failed,
            consistency_mode: SpireConsistencyMode::Degraded,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };
        let mut encoded = manifest.encode().unwrap();

        encoded[2] = 99;
        assert!(SpireEpochManifest::decode(&encoded).is_err());

        encoded[2] = SpireEpochState::Failed as u8;
        encoded[3] = 99;
        assert!(SpireEpochManifest::decode(&encoded).is_err());
    }

    #[test]
    fn epoch_manifest_rejects_invalid_publish_timing() {
        let mut manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };
        assert!(manifest.encode().is_err());

        manifest.state = SpireEpochState::Retired;
        manifest.published_at_micros = 2000;
        manifest.retain_until_micros = 1000;
        assert!(manifest.encode().is_err());
    }

    #[test]
    fn manifest_entry_round_trips() {
        let entry = SpireManifestEntry {
            epoch: 7,
            pid: 11,
            object_version: 3,
            placement_tid: tid(55, 4),
        };

        assert_eq!(
            SpireManifestEntry::decode(&entry.encode().unwrap()).unwrap(),
            entry
        );
    }

    #[test]
    fn manifest_entry_rejects_invalid_identity_locator_and_reserved_bytes() {
        let mut entry = SpireManifestEntry {
            epoch: 0,
            pid: 11,
            object_version: 3,
            placement_tid: tid(55, 4),
        };
        assert!(entry.encode().is_err());

        entry.epoch = 7;
        entry.pid = 0;
        assert!(entry.encode().is_err());

        entry.pid = 11;
        entry.object_version = 0;
        assert!(entry.encode().is_err());

        entry.object_version = 3;
        entry.placement_tid = ItemPointer::INVALID;
        assert!(entry.encode().is_err());

        entry.placement_tid = tid(55, 4);
        let mut encoded = entry.encode().unwrap();
        encoded[2] = 1;
        assert!(SpireManifestEntry::decode(&encoded).is_err());
    }

    #[test]
    fn object_manifest_sorts_and_round_trips_entries() {
        let manifest = SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 21,
                    object_version: 4,
                    placement_tid: tid(45, 2),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 3,
                    placement_tid: tid(44, 2),
                },
            ],
        )
        .unwrap();

        let decoded = SpireObjectManifest::decode(&manifest.encode().unwrap()).unwrap();

        assert_eq!(decoded, manifest);
        assert_eq!(decoded.entries[0].pid, 11);
        assert_eq!(decoded.entries[1].pid, 21);
        assert_eq!(decoded.get(21).unwrap().object_version, 4);
        assert!(decoded.get(99).is_none());
    }

    #[test]
    fn object_manifest_rejects_duplicate_pid_and_epoch_mismatch() {
        assert!(SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 3,
                    placement_tid: tid(44, 2),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 4,
                    placement_tid: tid(45, 2),
                },
            ],
        )
        .is_err());

        assert!(SpireObjectManifest::from_entries(
            7,
            vec![SpireManifestEntry {
                epoch: 8,
                pid: 11,
                object_version: 3,
                placement_tid: tid(44, 2),
            }],
        )
        .is_err());
    }

    #[test]
    fn object_manifest_rejects_corrupt_header_and_length() {
        let manifest = SpireObjectManifest::from_entries(
            7,
            vec![SpireManifestEntry {
                epoch: 7,
                pid: 11,
                object_version: 3,
                placement_tid: tid(44, 2),
            }],
        )
        .unwrap();
        let mut encoded = manifest.encode().unwrap();

        encoded[0] = 0;
        assert!(SpireObjectManifest::decode(&encoded).is_err());

        encoded = manifest.encode().unwrap();
        encoded[6] = 1;
        assert!(SpireObjectManifest::decode(&encoded).is_err());

        encoded = manifest.encode().unwrap();
        encoded.push(99);
        assert!(SpireObjectManifest::decode(&encoded).is_err());
    }

    #[test]
    fn published_epoch_snapshot_accepts_strict_available_placement() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let manifest = object_manifest(7, 11, 3);
        let directory = placement_directory(7, 11, 3, SpirePlacementState::Available);

        let snapshot = SpirePublishedEpochSnapshot::new(&epoch, &manifest, &directory).unwrap();

        assert_eq!(snapshot.epoch_manifest.epoch, 7);
        assert_eq!(snapshot.object_manifest.get(11).unwrap().object_version, 3);
        assert_eq!(
            snapshot.placement_directory.get(11).unwrap().state,
            SpirePlacementState::Available
        );
    }

    #[test]
    fn published_epoch_snapshot_rejects_non_published_epoch() {
        let mut epoch = published_epoch(7, SpireConsistencyMode::Strict);
        epoch.state = SpireEpochState::Building;
        epoch.published_at_micros = 0;
        let manifest = object_manifest(7, 11, 3);
        let directory = placement_directory(7, 11, 3, SpirePlacementState::Available);

        assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &directory).is_err());
    }

    #[test]
    fn published_epoch_snapshot_rejects_epoch_mismatch() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let wrong_manifest = object_manifest(8, 11, 3);
        let directory = placement_directory(7, 11, 3, SpirePlacementState::Available);

        assert!(SpirePublishedEpochSnapshot::new(&epoch, &wrong_manifest, &directory).is_err());

        let manifest = object_manifest(7, 11, 3);
        let wrong_directory = placement_directory(8, 11, 3, SpirePlacementState::Available);

        assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &wrong_directory).is_err());
    }

    #[test]
    fn published_epoch_snapshot_rejects_missing_or_version_mismatched_placement() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let manifest = object_manifest(7, 11, 3);
        let wrong_pid_directory = placement_directory(7, 12, 3, SpirePlacementState::Available);

        assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &wrong_pid_directory).is_err());

        let wrong_version_directory = placement_directory(7, 11, 4, SpirePlacementState::Available);

        assert!(
            SpirePublishedEpochSnapshot::new(&epoch, &manifest, &wrong_version_directory).is_err()
        );
    }

    #[test]
    fn published_epoch_snapshot_rejects_non_available_placement_in_strict_mode() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let manifest = object_manifest(7, 11, 3);

        for state in [
            SpirePlacementState::Stale,
            SpirePlacementState::Unavailable,
            SpirePlacementState::Skipped,
        ] {
            let directory = placement_directory(7, 11, 3, state);
            assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &directory).is_err());
        }
    }

    #[test]
    fn published_epoch_snapshot_degraded_mode_allows_unavailable_or_skipped_placement() {
        let epoch = published_epoch(7, SpireConsistencyMode::Degraded);
        let manifest = object_manifest(7, 11, 3);

        for state in [
            SpirePlacementState::Unavailable,
            SpirePlacementState::Skipped,
        ] {
            let directory = placement_directory(7, 11, 3, state);
            assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &directory).is_ok());
        }

        let stale_directory = placement_directory(7, 11, 3, SpirePlacementState::Stale);
        assert!(SpirePublishedEpochSnapshot::new(&epoch, &manifest, &stale_directory).is_err());
    }
}
