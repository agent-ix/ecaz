#[derive(Debug, Clone)]
pub(super) struct SpireLocalObjectStoreSet {
    config: SpireLocalStoreConfig,
    stores: Vec<SpireLocalObjectStore>,
    store_indexes_by_id: HashMap<u32, usize>,
}

impl SpireLocalObjectStoreSet {
    pub(super) fn from_config(
        config: SpireLocalStoreConfig,
        page_size: usize,
    ) -> Result<Self, String> {
        let mut stores = Vec::with_capacity(config.stores.len());
        let mut store_indexes_by_id = HashMap::with_capacity(config.stores.len());
        for descriptor in &config.stores {
            let store_index = stores.len();
            if store_indexes_by_id
                .insert(descriptor.local_store_id, store_index)
                .is_some()
            {
                return Err(format!(
                    "ec_spire local object store set duplicate local_store_id {}",
                    descriptor.local_store_id
                ));
            }
            stores.push(SpireLocalObjectStore::for_store_descriptor(
                descriptor, page_size,
            )?);
        }
        Ok(Self {
            config,
            stores,
            store_indexes_by_id,
        })
    }

    pub(super) fn insert_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.store_mut_for_pid(object.header.pid)?
            .insert_routing_object(epoch, object)
    }

    pub(super) fn insert_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        assignments: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        self.store_mut_for_pid(pid)?
            .insert_leaf_object_v2_from_rows(epoch, pid, object_version, parent_pid, assignments)
    }

    pub(super) fn insert_delta_object(
        &mut self,
        epoch: u64,
        object: &SpireDeltaPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.store_mut_for_pid(object.header.pid)?
            .insert_delta_object(epoch, object)
    }

    pub(super) fn insert_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.store_mut_for_pid(object.header.pid)?
            .insert_top_graph_object(epoch, object)
    }

    fn store_mut_for_pid(&mut self, pid: u64) -> Result<&mut SpireLocalObjectStore, String> {
        let local_store_id = self.config.store_for_pid(pid)?.local_store_id;
        let store_index = *self
            .store_indexes_by_id
            .get(&local_store_id)
            .ok_or_else(|| {
                format!(
                    "ec_spire local object store set is missing local_store_id {}",
                    local_store_id
                )
            })?;
        self.stores.get_mut(store_index).ok_or_else(|| {
            format!("ec_spire local object store set has stale index for local_store_id {local_store_id}")
        })
    }

    fn store_for_placement(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<&SpireLocalObjectStore, String> {
        self.config.validate_placement(placement)?;
        let store_index = *self
            .store_indexes_by_id
            .get(&placement.local_store_id)
            .ok_or_else(|| {
                format!(
                    "ec_spire local object store set is missing local_store_id {}",
                    placement.local_store_id
                )
            })?;
        self.stores.get(store_index).ok_or_else(|| {
            format!(
                "ec_spire local object store set has stale index for local_store_id {}",
                placement.local_store_id
            )
        })
    }
}

impl SpireObjectReader for SpireLocalObjectStoreSet {
    fn read_object_header(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpirePartitionObjectHeader, String> {
        self.store_for_placement(placement)?
            .read_object_header(placement)
    }

    fn read_routing_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireRoutingPartitionObject, String> {
        self.store_for_placement(placement)?
            .read_routing_object(placement)
    }

    fn read_leaf_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObject, String> {
        self.store_for_placement(placement)?
            .read_leaf_object(placement)
    }

    fn read_leaf_object_v2(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireLeafPartitionObjectV2, String> {
        self.store_for_placement(placement)?
            .read_leaf_object_v2(placement)
    }

    fn read_delta_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireDeltaPartitionObject, String> {
        self.store_for_placement(placement)?
            .read_delta_object(placement)
    }

    fn read_top_graph_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireTopGraphPartitionObject, String> {
        self.store_for_placement(placement)?
            .read_top_graph_object(placement)
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

    fn read_top_graph_object(
        &self,
        placement: &SpirePlacementEntry,
    ) -> Result<SpireTopGraphPartitionObject, String> {
        SpireLocalObjectStore::read_top_graph_object(self, placement)
    }
}
