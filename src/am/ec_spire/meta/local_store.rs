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

    pub(super) fn from_placement_directory_with_tablespaces(
        generation: u64,
        placement_directory: &SpirePlacementDirectory,
        mut tablespace_oid_for_relid: impl FnMut(u32) -> Result<u32, String>,
    ) -> Result<Self, String> {
        let mut relid_by_store_id = BTreeMap::<u32, u32>::new();
        for placement in &placement_directory.entries {
            if let Some(existing_relid) =
                relid_by_store_id.insert(placement.local_store_id, placement.store_relid)
            {
                if existing_relid != placement.store_relid {
                    return Err(format!(
                        "ec_spire placement directory maps local_store_id {} to relids {} and {}",
                        placement.local_store_id, existing_relid, placement.store_relid
                    ));
                }
            }
        }
        if relid_by_store_id.is_empty() {
            return Err("ec_spire local store config needs at least one placement".to_owned());
        }

        let mut stores = Vec::with_capacity(relid_by_store_id.len());
        for (local_store_id, store_relid) in relid_by_store_id {
            stores.push(SpireLocalStoreDescriptor::available(
                local_store_id,
                store_relid,
                tablespace_oid_for_relid(store_relid)?,
            )?);
        }
        Self::from_stores(generation, stores)
    }

    pub(super) fn from_placement_directory(
        generation: u64,
        placement_directory: &SpirePlacementDirectory,
    ) -> Result<Self, String> {
        Self::from_placement_directory_with_tablespaces(generation, placement_directory, |_| Ok(0))
    }

    pub(super) fn validate_placement_directory(
        &self,
        placement_directory: &SpirePlacementDirectory,
    ) -> Result<(), String> {
        for placement in &placement_directory.entries {
            self.validate_placement(placement)?;
        }
        Ok(())
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
        // Store count is part of built-index placement. Changing it remaps
        // existing object PIDs and requires REINDEX or an explicit rewrite.
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

// SplitMix64 finalizer over the PID. This is durable placement format; do not
// replace it without migration coverage and updated stable-value tests.
pub(super) fn spire_pid_hash(pid: u64) -> u64 {
    let mut value = pid;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
