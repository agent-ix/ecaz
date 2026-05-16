impl SpireBuildObjectStore for SpireLocalObjectStore {
    fn write_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_routing_object(epoch, object)
    }

    fn write_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_leaf_object_v2_from_rows(epoch, pid, object_version, parent_pid, rows)
    }

    fn write_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_top_graph_object(epoch, object)
    }
}

impl SpireBuildObjectStore for SpireLocalObjectStoreSet {
    fn write_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_routing_object(epoch, object)
    }

    fn write_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_leaf_object_v2_from_rows(epoch, pid, object_version, parent_pid, rows)
    }

    fn write_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_top_graph_object(epoch, object)
    }
}

impl SpireBuildObjectStore for SpireRelationObjectStore {
    fn write_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe { self.insert_routing_object(epoch, object) }
    }

    fn write_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        unsafe {
            self.insert_leaf_object_v2_from_rows(epoch, pid, object_version, parent_pid, rows)
        }
    }

    fn write_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe { self.insert_top_graph_object(epoch, object) }
    }
}

impl SpireBuildObjectStore for SpireRelationObjectStoreSet {
    fn write_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe { self.insert_routing_object(epoch, object) }
    }

    fn write_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String> {
        unsafe {
            self.insert_leaf_object_v2_from_rows(epoch, pid, object_version, parent_pid, rows)
        }
    }

    fn write_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        unsafe { self.insert_top_graph_object(epoch, object) }
    }
}
