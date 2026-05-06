//! Root/control metadata, epoch, and placement-map codecs.

use std::collections::HashMap;

use super::assign::{SPIRE_FIRST_LOCAL_VEC_SEQ, SPIRE_FIRST_PID};
use crate::storage::page::{ItemPointer, ITEM_POINTER_BYTES};

pub(super) const SPIRE_LOCAL_NODE_ID: u32 = 0;
pub(super) const SPIRE_SINGLE_LOCAL_STORE_ID: u32 = 0;
pub(super) const SPIRE_DEFAULT_LOCAL_STORE_GENERATION: u64 = 0;
pub(super) const SPIRE_MIN_EPOCH_RETENTION_SECS: u32 = 10 * 60;
pub(super) const SPIRE_FAILED_EPOCH_RETENTION_SECS: u32 = 60 * 60;
pub(super) const SPIRE_MAX_RETAINED_RETIRED_EPOCHS: u16 = 2;

const META_FORMAT_VERSION: u16 = 1;
const ROOT_CONTROL_MAGIC: u32 = 0x4352_5345; // "ESRC" as little-endian bytes.
const ROOT_CONTROL_STATE_BYTES: usize = 4 + 2 + 2 + 8 + 8 + 8 + ITEM_POINTER_BYTES * 3;
const LOCAL_STORE_CONFIG_MAGIC: u32 = 0x534c_5345; // "ESLS" as little-endian bytes.
const LOCAL_STORE_CONFIG_HEADER_BYTES: usize = 4 + 2 + 2 + 8 + 4;
const LOCAL_STORE_DESCRIPTOR_BYTES: usize = 4 + 4 + 4 + 1 + 3;
const PLACEMENT_DIRECTORY_MAGIC: u32 = 0x4450_5345; // "ESPD" as little-endian bytes.
const PLACEMENT_DIRECTORY_HEADER_BYTES: usize = 4 + 2 + 2 + 8 + 4;
const OBJECT_MANIFEST_MAGIC: u32 = 0x4d4f_5345; // "ESOM" as little-endian bytes.
const OBJECT_MANIFEST_HEADER_BYTES: usize = 4 + 2 + 2 + 8 + 4;
const EPOCH_MANIFEST_MAGIC: u32 = 0x454d_5345; // "ESME" as little-endian bytes.
const PLACEMENT_ENTRY_BYTES: usize = 2 + 1 + 1 + 8 + 8 + 4 + 4 + 4 + 8 + ITEM_POINTER_BYTES + 4;
const EPOCH_MANIFEST_BYTES: usize = 4 + 2 + 1 + 1 + 8 + 8 + 8 + 8;
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
    pub(super) fn encoded_len() -> usize {
        ROOT_CONTROL_STATE_BYTES
    }

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
pub(super) enum SpireLocalStoreState {
    Available = 1,
    Unavailable = 2,
}

impl SpireLocalStoreState {
    fn decode(value: u8) -> Result<Self, String> {
        match value {
            1 => Ok(Self::Available),
            2 => Ok(Self::Unavailable),
            other => Err(format!("ec_spire invalid local store state: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireLocalStoreDescriptor {
    pub(super) local_store_id: u32,
    pub(super) store_relid: u32,
    pub(super) tablespace_oid: u32,
    pub(super) state: SpireLocalStoreState,
}

impl SpireLocalStoreDescriptor {
    pub(super) fn available(
        local_store_id: u32,
        store_relid: u32,
        tablespace_oid: u32,
    ) -> Result<Self, String> {
        let descriptor = Self {
            local_store_id,
            store_relid,
            tablespace_oid,
            state: SpireLocalStoreState::Available,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    pub(super) fn embedded_single_store(
        store_relid: u32,
        tablespace_oid: u32,
    ) -> Result<Self, String> {
        Self::available(SPIRE_SINGLE_LOCAL_STORE_ID, store_relid, tablespace_oid)
    }

    fn encode_into(&self, out: &mut Vec<u8>) -> Result<(), String> {
        self.validate()?;
        out.extend_from_slice(&self.local_store_id.to_le_bytes());
        out.extend_from_slice(&self.store_relid.to_le_bytes());
        out.extend_from_slice(&self.tablespace_oid.to_le_bytes());
        out.push(self.state as u8);
        out.extend_from_slice(&[0_u8; 3]);
        Ok(())
    }

    fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() != LOCAL_STORE_DESCRIPTOR_BYTES {
            return Err(format!(
                "ec_spire local store descriptor length mismatch: got {}, expected {LOCAL_STORE_DESCRIPTOR_BYTES}",
                input.len()
            ));
        }
        if input[13..16] != [0_u8; 3] {
            return Err("ec_spire local store descriptor reserved bytes must be zero".to_owned());
        }
        let descriptor = Self {
            local_store_id: u32::from_le_bytes(
                input[0..4].try_into().expect("local store id bytes"),
            ),
            store_relid: u32::from_le_bytes(input[4..8].try_into().expect("store relid bytes")),
            tablespace_oid: u32::from_le_bytes(
                input[8..12].try_into().expect("tablespace oid bytes"),
            ),
            state: SpireLocalStoreState::decode(input[12])?,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }

    fn validate(&self) -> Result<(), String> {
        if self.store_relid == 0 {
            return Err("ec_spire local store descriptor store_relid 0 is invalid".to_owned());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireLocalStoreConfig {
    pub(super) generation: u64,
    pub(super) stores: Vec<SpireLocalStoreDescriptor>,
}

impl SpireLocalStoreConfig {
    pub(super) fn from_stores(
        generation: u64,
        mut stores: Vec<SpireLocalStoreDescriptor>,
    ) -> Result<Self, String> {
        stores.sort_by_key(|store| store.local_store_id);
        let config = Self { generation, stores };
        config.validate()?;
        Ok(config)
    }

    pub(super) fn embedded_single_store(
        store_relid: u32,
        tablespace_oid: u32,
    ) -> Result<Self, String> {
        Self::from_stores(
            SPIRE_DEFAULT_LOCAL_STORE_GENERATION,
            vec![SpireLocalStoreDescriptor::embedded_single_store(
                store_relid,
                tablespace_oid,
            )?],
        )
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let store_count = u32::try_from(self.stores.len())
            .map_err(|_| "ec_spire local store count exceeds u32".to_owned())?;
        let mut out = Vec::with_capacity(
            LOCAL_STORE_CONFIG_HEADER_BYTES + self.stores.len() * LOCAL_STORE_DESCRIPTOR_BYTES,
        );
        out.extend_from_slice(&LOCAL_STORE_CONFIG_MAGIC.to_le_bytes());
        out.extend_from_slice(&META_FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&self.generation.to_le_bytes());
        out.extend_from_slice(&store_count.to_le_bytes());
        for store in &self.stores {
            store.encode_into(&mut out)?;
        }
        Ok(out)
    }

    pub(super) fn decode(input: &[u8]) -> Result<Self, String> {
        if input.len() < LOCAL_STORE_CONFIG_HEADER_BYTES {
            return Err(format!(
                "ec_spire local store config too short: got {}, expected at least {LOCAL_STORE_CONFIG_HEADER_BYTES}",
                input.len()
            ));
        }
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("store config magic bytes"));
        if magic != LOCAL_STORE_CONFIG_MAGIC {
            return Err(format!(
                "ec_spire invalid local store config magic: {magic:#x}"
            ));
        }
        validate_format_version(&input[4..6])?;
        let reserved = u16::from_le_bytes(input[6..8].try_into().expect("reserved bytes"));
        if reserved != 0 {
            return Err(format!(
                "ec_spire local store config reserved bytes must be zero, got {reserved}"
            ));
        }
        let generation =
            u64::from_le_bytes(input[8..16].try_into().expect("store generation bytes"));
        let store_count =
            u32::from_le_bytes(input[16..20].try_into().expect("store count bytes")) as usize;
        let expected_len = store_count
            .checked_mul(LOCAL_STORE_DESCRIPTOR_BYTES)
            .and_then(|store_bytes| store_bytes.checked_add(LOCAL_STORE_CONFIG_HEADER_BYTES))
            .ok_or_else(|| "ec_spire local store config length overflow".to_owned())?;
        if input.len() != expected_len {
            return Err(format!(
                "ec_spire local store config length mismatch: got {}, expected {expected_len}",
                input.len()
            ));
        }

        let mut stores = Vec::with_capacity(store_count);
        let mut cursor = LOCAL_STORE_CONFIG_HEADER_BYTES;
        for _ in 0..store_count {
            stores.push(SpireLocalStoreDescriptor::decode(
                &input[cursor..cursor + LOCAL_STORE_DESCRIPTOR_BYTES],
            )?);
            cursor += LOCAL_STORE_DESCRIPTOR_BYTES;
        }
        Self::from_stores(generation, stores)
    }

    pub(super) fn get(&self, local_store_id: u32) -> Option<&SpireLocalStoreDescriptor> {
        self.stores
            .binary_search_by_key(&local_store_id, |store| store.local_store_id)
            .ok()
            .map(|index| &self.stores[index])
    }

    pub(super) fn store_for_pid(&self, pid: u64) -> Result<&SpireLocalStoreDescriptor, String> {
        if pid == 0 {
            return Err("ec_spire cannot place pid 0 in a local store".to_owned());
        }
        let store_count = u64::try_from(self.stores.len())
            .map_err(|_| "ec_spire local store count exceeds u64".to_owned())?;
        let store_index = usize::try_from(spire_pid_hash(pid) % store_count)
            .map_err(|_| "ec_spire local store index exceeds usize".to_owned())?;
        self.stores
            .get(store_index)
            .ok_or_else(|| "ec_spire local store hash index out of bounds".to_owned())
    }

    pub(super) fn validate_placement(&self, placement: &SpirePlacementEntry) -> Result<(), String> {
        if placement.node_id != SPIRE_LOCAL_NODE_ID {
            return Err(format!(
                "ec_spire local store config cannot validate remote node_id {}",
                placement.node_id
            ));
        }
        let store = self.get(placement.local_store_id).ok_or_else(|| {
            format!(
                "ec_spire placement local_store_id {} is not in active local store config",
                placement.local_store_id
            )
        })?;
        if store.store_relid != placement.store_relid {
            return Err(format!(
                "ec_spire placement store_relid {} does not match local_store_id {} relid {}",
                placement.store_relid, placement.local_store_id, store.store_relid
            ));
        }
        if store.state != SpireLocalStoreState::Available
            && placement.state == SpirePlacementState::Available
        {
            return Err(format!(
                "ec_spire placement local_store_id {} is available but store state is {:?}",
                placement.local_store_id, store.state
            ));
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.stores.is_empty() {
            return Err("ec_spire local store config must include at least one store".to_owned());
        }
        let mut previous_store_id = None;
        for store in &self.stores {
            store.validate()?;
            if let Some(previous_store_id) = previous_store_id {
                if store.local_store_id == previous_store_id {
                    return Err(format!(
                        "ec_spire local store config duplicate local_store_id: {}",
                        store.local_store_id
                    ));
                }
                if store.local_store_id < previous_store_id {
                    return Err("ec_spire local store config entries must be sorted".to_owned());
                }
            }
            previous_store_id = Some(store.local_store_id);
        }
        Ok(())
    }
}

pub(super) fn spire_pid_hash(pid: u64) -> u64 {
    let mut value = pid;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
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
        Self::local_single_store_available(
            epoch,
            pid,
            store_relid,
            object_version,
            object_tid,
            object_bytes,
        )
    }

    pub(super) fn local_single_store_available(
        epoch: u64,
        pid: u64,
        store_relid: u32,
        object_version: u64,
        object_tid: ItemPointer,
        object_bytes: u32,
    ) -> Self {
        Self::local_single_store_with_state(
            epoch,
            pid,
            store_relid,
            object_version,
            object_tid,
            object_bytes,
            SpirePlacementState::Available,
        )
    }

    pub(super) fn local_single_store_stale(
        epoch: u64,
        pid: u64,
        store_relid: u32,
        object_version: u64,
        object_tid: ItemPointer,
        object_bytes: u32,
    ) -> Self {
        Self::local_single_store_with_state(
            epoch,
            pid,
            store_relid,
            object_version,
            object_tid,
            object_bytes,
            SpirePlacementState::Stale,
        )
    }

    pub(super) fn local_single_store_unavailable(
        epoch: u64,
        pid: u64,
        store_relid: u32,
        object_version: u64,
        object_tid: ItemPointer,
        object_bytes: u32,
    ) -> Self {
        Self::local_single_store_with_state(
            epoch,
            pid,
            store_relid,
            object_version,
            object_tid,
            object_bytes,
            SpirePlacementState::Unavailable,
        )
    }

    pub(super) fn local_single_store_skipped(
        epoch: u64,
        pid: u64,
        store_relid: u32,
        object_version: u64,
        object_tid: ItemPointer,
        object_bytes: u32,
    ) -> Self {
        Self::local_single_store_with_state(
            epoch,
            pid,
            store_relid,
            object_version,
            object_tid,
            object_bytes,
            SpirePlacementState::Skipped,
        )
    }

    pub(super) fn local_single_store_with_state(
        epoch: u64,
        pid: u64,
        store_relid: u32,
        object_version: u64,
        object_tid: ItemPointer,
        object_bytes: u32,
        state: SpirePlacementState,
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
            state,
        }
    }

    pub(super) fn local_store_available(
        epoch: u64,
        pid: u64,
        store: &SpireLocalStoreDescriptor,
        object_version: u64,
        object_tid: ItemPointer,
        object_bytes: u32,
    ) -> Self {
        Self {
            epoch,
            pid,
            node_id: SPIRE_LOCAL_NODE_ID,
            local_store_id: store.local_store_id,
            store_relid: store.store_relid,
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
    pub(super) fn encoded_len() -> usize {
        EPOCH_MANIFEST_BYTES
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;

        let mut out = Vec::with_capacity(EPOCH_MANIFEST_BYTES);
        out.extend_from_slice(&EPOCH_MANIFEST_MAGIC.to_le_bytes());
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
        let magic = u32::from_le_bytes(input[0..4].try_into().expect("epoch magic bytes"));
        if magic != EPOCH_MANIFEST_MAGIC {
            return Err(format!("ec_spire invalid epoch manifest magic: {magic:#x}"));
        }
        validate_format_version(&input[4..6])?;

        let manifest = Self {
            state: SpireEpochState::decode(input[6])?,
            consistency_mode: SpireConsistencyMode::decode(input[7])?,
            epoch: u64::from_le_bytes(input[8..16].try_into().expect("epoch bytes")),
            published_at_micros: i64::from_le_bytes(
                input[16..24].try_into().expect("published_at bytes"),
            ),
            retain_until_micros: i64::from_le_bytes(
                input[24..32].try_into().expect("retain_until bytes"),
            ),
            active_query_count: u64::from_le_bytes(
                input[32..40].try_into().expect("active query count bytes"),
            ),
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub(super) fn validate(&self) -> Result<(), String> {
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

    pub(super) fn cleanup_eligible_at(&self, now_micros: i64) -> bool {
        match self.state {
            SpireEpochState::Building | SpireEpochState::Published => false,
            SpireEpochState::Retired => {
                self.active_query_count == 0 && now_micros >= self.retain_until_micros
            }
            SpireEpochState::Failed => now_micros >= self.retain_until_micros,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireEpochCleanupPlan {
    pub(super) cleanup_epochs: Vec<u64>,
    pub(super) retained_retired_epochs: Vec<u64>,
}

pub(super) fn plan_epoch_cleanup(
    manifests: &[SpireEpochManifest],
    active_epoch: u64,
    now_micros: i64,
) -> Result<SpireEpochCleanupPlan, String> {
    let mut epochs = Vec::with_capacity(manifests.len());
    for manifest in manifests {
        manifest.validate()?;
        if epochs.contains(&manifest.epoch) {
            return Err(format!(
                "ec_spire cleanup plan duplicate epoch: {}",
                manifest.epoch
            ));
        }
        epochs.push(manifest.epoch);
    }

    let mut retained_retired_epochs: Vec<u64> = manifests
        .iter()
        .filter(|manifest| manifest.state == SpireEpochState::Retired)
        .map(|manifest| manifest.epoch)
        .collect();
    retained_retired_epochs.sort_unstable_by(|left, right| right.cmp(left));
    retained_retired_epochs.truncate(usize::from(SPIRE_MAX_RETAINED_RETIRED_EPOCHS));
    retained_retired_epochs.sort_unstable();

    let mut cleanup_epochs = Vec::new();
    for manifest in manifests {
        if manifest.epoch == active_epoch {
            continue;
        }
        if !manifest.cleanup_eligible_at(now_micros) {
            continue;
        }
        if manifest.state == SpireEpochState::Retired
            && retained_retired_epochs.contains(&manifest.epoch)
        {
            continue;
        }
        cleanup_epochs.push(manifest.epoch);
    }
    cleanup_epochs.sort_unstable();

    Ok(SpireEpochCleanupPlan {
        cleanup_epochs,
        retained_retired_epochs,
    })
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

        for placement in &self.placement_directory.entries {
            if self.object_manifest.get(placement.pid).is_none() {
                return Err(format!(
                    "ec_spire published snapshot has placement without object manifest entry for pid {}",
                    placement.pid
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireSnapshotPidLookup<'a> {
    pub(super) manifest_entry: &'a SpireManifestEntry,
    pub(super) placement: &'a SpirePlacementEntry,
}

#[derive(Debug, Clone)]
pub(super) struct SpireValidatedEpochSnapshot<'a> {
    snapshot: SpirePublishedEpochSnapshot<'a>,
    pid_index: HashMap<u64, SpireSnapshotPidLookup<'a>>,
}

impl<'a> SpireValidatedEpochSnapshot<'a> {
    pub(super) fn new(
        epoch_manifest: &'a SpireEpochManifest,
        object_manifest: &'a SpireObjectManifest,
        placement_directory: &'a SpirePlacementDirectory,
    ) -> Result<Self, String> {
        Self::from_snapshot(SpirePublishedEpochSnapshot::new(
            epoch_manifest,
            object_manifest,
            placement_directory,
        )?)
    }

    pub(super) fn from_snapshot(snapshot: SpirePublishedEpochSnapshot<'a>) -> Result<Self, String> {
        snapshot.validate()?;
        let mut pid_index = HashMap::with_capacity(snapshot.object_manifest.entries.len());
        for manifest_entry in &snapshot.object_manifest.entries {
            let placement = snapshot
                .placement_directory
                .get(manifest_entry.pid)
                .ok_or_else(|| {
                    format!(
                        "ec_spire validated snapshot missing placement for pid {}",
                        manifest_entry.pid
                    )
                })?;
            pid_index.insert(
                manifest_entry.pid,
                SpireSnapshotPidLookup {
                    manifest_entry,
                    placement,
                },
            );
        }
        Ok(Self {
            snapshot,
            pid_index,
        })
    }

    pub(super) fn snapshot(&self) -> SpirePublishedEpochSnapshot<'a> {
        self.snapshot
    }

    pub(super) fn epoch_manifest(&self) -> &'a SpireEpochManifest {
        self.snapshot.epoch_manifest
    }

    pub(super) fn object_manifest(&self) -> &'a SpireObjectManifest {
        self.snapshot.object_manifest
    }

    pub(super) fn placement_directory(&self) -> &'a SpirePlacementDirectory {
        self.snapshot.placement_directory
    }

    pub(super) fn lookup(&self, pid: u64) -> Option<SpireSnapshotPidLookup<'a>> {
        self.pid_index.get(&pid).copied()
    }

    pub(super) fn require_lookup(
        &self,
        pid: u64,
        context: &str,
    ) -> Result<SpireSnapshotPidLookup<'a>, String> {
        self.lookup(pid)
            .ok_or_else(|| format!("ec_spire {context} missing snapshot lookup for pid {pid}"))
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
        plan_epoch_cleanup, spire_pid_hash, SpireConsistencyMode, SpireEpochManifest,
        SpireEpochState, SpireLocalStoreConfig, SpireLocalStoreDescriptor, SpireLocalStoreState,
        SpireManifestEntry, SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry,
        SpirePlacementState, SpirePublishedEpochSnapshot, SpireRootControlState,
        SpireValidatedEpochSnapshot, SPIRE_DEFAULT_LOCAL_STORE_GENERATION,
        SPIRE_FAILED_EPOCH_RETENTION_SECS, SPIRE_LOCAL_NODE_ID, SPIRE_MAX_RETAINED_RETIRED_EPOCHS,
        SPIRE_MIN_EPOCH_RETENTION_SECS, SPIRE_SINGLE_LOCAL_STORE_ID,
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

    fn retired_epoch(
        epoch: u64,
        retain_until_micros: i64,
        active_query_count: u64,
    ) -> SpireEpochManifest {
        SpireEpochManifest {
            epoch,
            state: SpireEpochState::Retired,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros,
            active_query_count,
        }
    }

    fn failed_epoch(epoch: u64, retain_until_micros: i64) -> SpireEpochManifest {
        SpireEpochManifest {
            epoch,
            state: SpireEpochState::Failed,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros,
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
    fn embedded_single_store_config_preserves_current_store_shape() {
        let config = SpireLocalStoreConfig::embedded_single_store(12345, 0)
            .expect("default tablespace oid 0 should be allowed");

        assert_eq!(config.generation, SPIRE_DEFAULT_LOCAL_STORE_GENERATION);
        assert_eq!(config.stores.len(), 1);
        assert_eq!(config.stores[0].local_store_id, SPIRE_SINGLE_LOCAL_STORE_ID);
        assert_eq!(config.stores[0].store_relid, 12345);
        assert_eq!(config.stores[0].tablespace_oid, 0);
        assert_eq!(config.stores[0].state, SpireLocalStoreState::Available);

        let decoded = SpireLocalStoreConfig::decode(&config.encode().unwrap()).unwrap();
        assert_eq!(decoded, config);
    }

    #[test]
    fn local_store_config_allows_repeated_tablespaces_for_baselines() {
        let config = SpireLocalStoreConfig::from_stores(
            2,
            vec![
                SpireLocalStoreDescriptor::available(1, 12346, 987).unwrap(),
                SpireLocalStoreDescriptor::available(0, 12345, 987).unwrap(),
            ],
        )
        .expect("repeated tablespace oid should be accepted");

        assert_eq!(config.stores[0].local_store_id, 0);
        assert_eq!(config.stores[1].local_store_id, 1);
        assert_eq!(
            config.stores[0].tablespace_oid,
            config.stores[1].tablespace_oid
        );

        let decoded = SpireLocalStoreConfig::decode(&config.encode().unwrap()).unwrap();
        assert_eq!(decoded, config);
    }

    #[test]
    fn local_store_config_rejects_empty_duplicate_or_invalid_store_relid() {
        assert!(SpireLocalStoreConfig::from_stores(1, Vec::new()).is_err());
        assert!(SpireLocalStoreDescriptor::available(0, 0, 42).is_err());
        assert!(SpireLocalStoreConfig::from_stores(
            1,
            vec![
                SpireLocalStoreDescriptor::available(0, 12345, 42).unwrap(),
                SpireLocalStoreDescriptor::available(0, 12346, 43).unwrap(),
            ],
        )
        .is_err());
    }

    #[test]
    fn local_store_config_validates_placements_against_active_store_set() {
        let store = SpireLocalStoreDescriptor::available(2, 12347, 987).unwrap();
        let config = SpireLocalStoreConfig::from_stores(4, vec![store]).unwrap();
        let placement =
            SpirePlacementEntry::local_store_available(7, 11, &store, 3, tid(44, 2), 4096);

        config.validate_placement(&placement).unwrap();

        let mut wrong_store_id = placement;
        wrong_store_id.local_store_id = 3;
        assert!(config.validate_placement(&wrong_store_id).is_err());

        let mut wrong_relid = placement;
        wrong_relid.store_relid = 99999;
        assert!(config.validate_placement(&wrong_relid).is_err());

        let unavailable_config = SpireLocalStoreConfig::from_stores(
            4,
            vec![SpireLocalStoreDescriptor {
                state: SpireLocalStoreState::Unavailable,
                ..store
            }],
        )
        .unwrap();
        assert!(unavailable_config.validate_placement(&placement).is_err());
    }

    #[test]
    fn spire_pid_hash_has_stable_cross_platform_values() {
        assert_eq!(spire_pid_hash(1), 0x5692_161d_100b_05e5);
        assert_eq!(spire_pid_hash(2), 0xdbd2_3897_3a2b_148a);
        assert_eq!(spire_pid_hash(11), 0x3462_d848_f53a_bb6d);
        assert_eq!(spire_pid_hash(123_456_789), 0xf21c_87d4_233f_fd60);
    }

    #[test]
    fn local_store_config_places_pid_by_stable_hash_mod_store_count() {
        let config = SpireLocalStoreConfig::from_stores(
            1,
            vec![
                SpireLocalStoreDescriptor::available(0, 12345, 900).unwrap(),
                SpireLocalStoreDescriptor::available(1, 12346, 901).unwrap(),
                SpireLocalStoreDescriptor::available(2, 12347, 902).unwrap(),
                SpireLocalStoreDescriptor::available(3, 12348, 903).unwrap(),
            ],
        )
        .unwrap();

        assert_eq!(config.store_for_pid(1).unwrap().local_store_id, 1);
        assert_eq!(config.store_for_pid(2).unwrap().local_store_id, 2);
        assert_eq!(config.store_for_pid(3).unwrap().local_store_id, 0);
        assert_eq!(config.store_for_pid(11).unwrap().local_store_id, 1);
        assert!(config.store_for_pid(0).is_err());
    }

    #[test]
    fn local_single_store_placement_uses_default_ids() {
        let entry = SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096);

        assert_eq!(entry.node_id, SPIRE_LOCAL_NODE_ID);
        assert_eq!(entry.local_store_id, SPIRE_SINGLE_LOCAL_STORE_ID);
        assert_eq!(entry.state, SpirePlacementState::Available);
    }

    #[test]
    fn local_single_store_state_constructors_make_state_explicit() {
        let available =
            SpirePlacementEntry::local_single_store_available(7, 11, 12345, 3, tid(44, 2), 4096);
        let stale =
            SpirePlacementEntry::local_single_store_stale(7, 11, 12345, 3, tid(44, 2), 4096);
        let unavailable =
            SpirePlacementEntry::local_single_store_unavailable(7, 11, 12345, 3, tid(44, 2), 4096);
        let skipped =
            SpirePlacementEntry::local_single_store_skipped(7, 11, 12345, 3, tid(44, 2), 4096);

        assert_eq!(available.state, SpirePlacementState::Available);
        assert_eq!(stale.state, SpirePlacementState::Stale);
        assert_eq!(unavailable.state, SpirePlacementState::Unavailable);
        assert_eq!(skipped.state, SpirePlacementState::Skipped);
        assert_eq!(stale.node_id, SPIRE_LOCAL_NODE_ID);
        assert_eq!(stale.local_store_id, SPIRE_SINGLE_LOCAL_STORE_ID);
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
        assert_eq!(
            manifest.encode().unwrap().len(),
            SpireEpochManifest::encoded_len()
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

        encoded[6] = 99;
        assert!(SpireEpochManifest::decode(&encoded).is_err());

        encoded[6] = SpireEpochState::Failed as u8;
        encoded[7] = 99;
        assert!(SpireEpochManifest::decode(&encoded).is_err());
    }

    #[test]
    fn epoch_manifest_rejects_invalid_magic_and_format() {
        let manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Failed,
            consistency_mode: SpireConsistencyMode::Degraded,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };
        let mut encoded = manifest.encode().unwrap();

        encoded[0] = 0;
        assert!(SpireEpochManifest::decode(&encoded).is_err());

        encoded = manifest.encode().unwrap();
        encoded[4] = 2;
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
    fn cleanup_eligibility_keeps_building_and_published_epochs() {
        let building = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Building,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros: 0,
            active_query_count: 0,
        };
        let published = published_epoch(8, SpireConsistencyMode::Strict);

        assert!(!building.cleanup_eligible_at(i64::MAX));
        assert!(!published.cleanup_eligible_at(i64::MAX));
    }

    #[test]
    fn cleanup_eligibility_keeps_retired_epochs_until_retention_and_queries_clear() {
        let mut retired = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Retired,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 1,
        };

        assert!(!retired.cleanup_eligible_at(1999));
        assert!(!retired.cleanup_eligible_at(2000));

        retired.active_query_count = 0;
        assert!(!retired.cleanup_eligible_at(1999));
        assert!(retired.cleanup_eligible_at(2000));
    }

    #[test]
    fn cleanup_eligibility_uses_failed_epoch_retain_until() {
        let failed = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Failed,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 0,
            retain_until_micros: 2000,
            active_query_count: 99,
        };

        assert!(!failed.cleanup_eligible_at(1999));
        assert!(failed.cleanup_eligible_at(2000));
    }

    #[test]
    fn cleanup_plan_keeps_active_epoch_and_newest_retired_epochs() {
        let manifests = vec![
            published_epoch(10, SpireConsistencyMode::Strict),
            retired_epoch(9, 1000, 0),
            retired_epoch(8, 1000, 0),
            retired_epoch(7, 1000, 0),
            failed_epoch(6, 1000),
        ];

        let plan = plan_epoch_cleanup(&manifests, 10, 2000).unwrap();

        assert_eq!(plan.retained_retired_epochs, vec![8, 9]);
        assert_eq!(plan.cleanup_epochs, vec![6, 7]);
    }

    #[test]
    fn cleanup_plan_waits_for_retention_and_active_queries() {
        let manifests = vec![
            retired_epoch(9, 3000, 0),
            retired_epoch(8, 1000, 1),
            retired_epoch(7, 1000, 0),
            failed_epoch(6, 3000),
        ];

        let plan = plan_epoch_cleanup(&manifests, 0, 2000).unwrap();

        assert_eq!(plan.retained_retired_epochs, vec![8, 9]);
        assert_eq!(plan.cleanup_epochs, vec![7]);
    }

    #[test]
    fn cleanup_plan_rejects_duplicate_epochs() {
        let manifests = vec![retired_epoch(7, 1000, 0), failed_epoch(7, 1000)];

        assert!(plan_epoch_cleanup(&manifests, 0, 2000).is_err());
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
    fn validated_epoch_snapshot_builds_pid_lookup_cache() {
        let epoch = published_epoch(7, SpireConsistencyMode::Strict);
        let manifest = SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 3,
                    placement_tid: tid(55, 4),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 12,
                    object_version: 4,
                    placement_tid: tid(56, 4),
                },
            ],
        )
        .unwrap();
        let directory = SpirePlacementDirectory::from_entries(
            7,
            vec![
                SpirePlacementEntry::local_single_store(7, 12, 12345, 4, tid(45, 2), 4096),
                SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 2048),
            ],
        )
        .unwrap();

        let snapshot = SpireValidatedEpochSnapshot::new(&epoch, &manifest, &directory).unwrap();
        let lookup = snapshot.require_lookup(12, "test").unwrap();

        assert_eq!(snapshot.snapshot().epoch_manifest.epoch, 7);
        assert_eq!(lookup.manifest_entry.object_version, 4);
        assert_eq!(lookup.placement.object_tid, tid(45, 2));
        assert!(snapshot.lookup(99).is_none());
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

        let orphan_placement_directory = SpirePlacementDirectory::from_entries(
            7,
            vec![
                SpirePlacementEntry::local_single_store(7, 11, 12345, 3, tid(44, 2), 4096),
                SpirePlacementEntry::local_single_store(7, 12, 12345, 4, tid(45, 2), 4096),
            ],
        )
        .unwrap();

        assert!(
            SpirePublishedEpochSnapshot::new(&epoch, &manifest, &orphan_placement_directory)
                .is_err()
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
