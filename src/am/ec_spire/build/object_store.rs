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

    fn write_leaf_object_v3_from_rows_and_summaries(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
        summaries: &[SpireLeafBlockSummary],
        summary_block_rows: u32,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_leaf_object_v3_from_rows_and_summaries(
            epoch,
            pid,
            object_version,
            parent_pid,
            rows,
            summaries,
            summary_block_rows,
        )
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

    fn write_leaf_object_v3_from_rows_and_summaries(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
        summaries: &[SpireLeafBlockSummary],
        summary_block_rows: u32,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_leaf_object_v3_from_rows_and_summaries(
            epoch,
            pid,
            object_version,
            parent_pid,
            rows,
            summaries,
            summary_block_rows,
        )
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

    fn write_leaf_object_v3_from_rows_and_summaries(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
        summaries: &[SpireLeafBlockSummary],
        summary_block_rows: u32,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_leaf_object_v3_from_rows_and_summaries(
            epoch,
            pid,
            object_version,
            parent_pid,
            rows,
            summaries,
            summary_block_rows,
        )
    }

    fn write_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_top_graph_object(epoch, object)
    }
}

impl SpireBuildObjectStore for SpireRelationObjectStoreSet {
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

    fn write_leaf_object_v3_from_rows_and_summaries(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
        summaries: &[SpireLeafBlockSummary],
        summary_block_rows: u32,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_leaf_object_v3_from_rows_and_summaries(
            epoch,
            pid,
            object_version,
            parent_pid,
            rows,
            summaries,
            summary_block_rows,
        )
    }

    fn write_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String> {
        self.insert_top_graph_object(epoch, object)
    }
}
