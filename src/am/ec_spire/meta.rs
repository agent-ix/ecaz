//! Root/control metadata, epoch, and placement-map codecs.

use crate::storage::page::{ItemPointer, ITEM_POINTER_BYTES};

pub(super) const SPIRE_LOCAL_NODE_ID: u32 = 0;
pub(super) const SPIRE_SINGLE_LOCAL_STORE_ID: u32 = 0;
pub(super) const SPIRE_MIN_EPOCH_RETENTION_SECS: u32 = 10 * 60;
pub(super) const SPIRE_FAILED_EPOCH_RETENTION_SECS: u32 = 60 * 60;
pub(super) const SPIRE_MAX_RETAINED_RETIRED_EPOCHS: u16 = 2;

const META_FORMAT_VERSION: u16 = 1;
const PLACEMENT_ENTRY_BYTES: usize = 2 + 1 + 1 + 8 + 8 + 4 + 4 + 4 + 8 + ITEM_POINTER_BYTES + 4;
const EPOCH_MANIFEST_BYTES: usize = 2 + 1 + 1 + 8 + 8 + 8 + 8;
const MANIFEST_ENTRY_BYTES: usize = 2 + 2 + 8 + 8 + 8 + ITEM_POINTER_BYTES;

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
        SpirePlacementEntry, SpirePlacementState, SPIRE_FAILED_EPOCH_RETENTION_SECS,
        SPIRE_LOCAL_NODE_ID, SPIRE_MAX_RETAINED_RETIRED_EPOCHS, SPIRE_MIN_EPOCH_RETENTION_SECS,
        SPIRE_SINGLE_LOCAL_STORE_ID,
    };
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    #[test]
    fn retention_defaults_match_phase0_design() {
        assert_eq!(SPIRE_MIN_EPOCH_RETENTION_SECS, 600);
        assert_eq!(SPIRE_FAILED_EPOCH_RETENTION_SECS, 3600);
        assert_eq!(SPIRE_MAX_RETAINED_RETIRED_EPOCHS, 2);
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
}
