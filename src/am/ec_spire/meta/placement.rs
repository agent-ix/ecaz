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
        Self::local_store_available_by_id(
            epoch,
            pid,
            store.local_store_id,
            store.store_relid,
            object_version,
            object_tid,
            object_bytes,
        )
    }

    pub(super) fn local_store_available_by_id(
        epoch: u64,
        pid: u64,
        local_store_id: u32,
        store_relid: u32,
        object_version: u64,
        object_tid: ItemPointer,
        object_bytes: u32,
    ) -> Self {
        Self {
            epoch,
            pid,
            node_id: SPIRE_LOCAL_NODE_ID,
            local_store_id,
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

