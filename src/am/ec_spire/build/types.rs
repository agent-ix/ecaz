
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireIndexedVectorKind {
    Ecvector,
    Tqvector,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireBuildTuple {
    pub(super) heap_tid: ItemPointer,
    pub(super) dimensions: u16,
    pub(super) assignment: SpireLeafAssignmentInput,
    pub(super) source_vector: Vec<f32>,
}

struct SpireBuildState {
    options: options::EcSpireOptions,
    indexed_vector_kind: SpireIndexedVectorKind,
    scanned_tuples: usize,
    tuples: Vec<SpireBuildTuple>,
    dimensions: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelBuildInput {
    pub(super) epoch: u64,
    pub(super) object_version: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) placement_tid: ItemPointer,
    pub(super) assignments: Vec<SpireLeafAssignmentInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireSingleLevelBuildDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) leaf_object: SpireLeafPartitionObject,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpirePartitionedSingleLevelBuildInput {
    pub(super) epoch: u64,
    pub(super) object_version: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) root_placement_tid: ItemPointer,
    pub(super) placement_tids: Vec<ItemPointer>,
    pub(super) assignments: Vec<SpireLeafAssignmentInput>,
    pub(super) centroid_plan: SpireSingleLevelCentroidPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpirePartitionedSingleLevelBuildDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) route_map: SpireSingleLevelRouteMap,
    pub(super) root_pid: u64,
    pub(super) routing_object: SpireRoutingPartitionObject,
    pub(super) centroid_pids: Vec<u64>,
    pub(super) leaf_objects: Vec<SpireLeafPartitionObject>,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingChildInput {
    pub(super) child_pid: u64,
    pub(super) child_level: u16,
    pub(super) centroid: Vec<f32>,
    pub(super) source_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingBuildInput {
    pub(super) object_version: u64,
    pub(super) dimensions: u16,
    pub(super) target_fanout: u32,
    pub(super) seed: u64,
    pub(super) children: Vec<SpireRecursiveRoutingChildInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingBuildDraft {
    pub(super) root_pid: u64,
    pub(super) root_level: u16,
    pub(super) routing_objects: Vec<SpireRoutingPartitionObject>,
    pub(super) centroid_records: Vec<SpireRecursiveCentroidRecord>,
    pub(super) next_pid: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveCentroidRecord {
    pub(super) parent_pid: u64,
    pub(super) child_pid: u64,
    pub(super) child_level: u16,
    pub(super) centroid_ordinal: u32,
    pub(super) dimensions: u16,
    pub(super) centroid: Vec<f32>,
    pub(super) source_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveBuildCoordinatorInput {
    pub(super) epoch: u64,
    pub(super) object_version: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) target_fanout: u32,
    pub(super) seed: u64,
    pub(super) boundary_replica_count: u32,
    pub(super) assignments: Vec<SpireLeafAssignmentInput>,
    pub(super) source_vectors: Vec<Vec<f32>>,
    pub(super) centroid_plan: SpireSingleLevelCentroidPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveBuildCoordinatorDraft {
    pub(super) epoch_input: SpireRecursiveRoutingEpochObjectInput,
    pub(super) leaf_pids: Vec<u64>,
    pub(super) next_pid: u64,
    pub(super) next_local_vec_seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveLeafObjectInput {
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) parent_pid: u64,
    pub(super) rows: Vec<SpireLeafAssignmentRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingEpochObjectInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) routing_draft: SpireRecursiveRoutingBuildDraft,
    pub(super) leaf_inputs: Vec<SpireRecursiveLeafObjectInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingEpochInput {
    pub(super) epoch: u64,
    pub(super) published_at_micros: i64,
    pub(super) retain_until_micros: i64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) routing_draft: SpireRecursiveRoutingBuildDraft,
    pub(super) leaf_placements: Vec<SpirePlacementEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingEpochDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) root_pid: u64,
    pub(super) routing_objects: Vec<SpireRoutingPartitionObject>,
    // TODO: these are not persisted separately; diagnostics rebuild them with
    // centroid_records_from_routing until durable centroid objects land.
    pub(super) centroid_records: Vec<SpireRecursiveCentroidRecord>,
    pub(super) next_pid: u64,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireRecursiveDraftInvariants {
    leaf_parent_pids: HashMap<u64, u64>,
}

pub(super) trait SpireBuildObjectStore: SpireObjectReader {
    fn write_routing_object(
        &mut self,
        epoch: u64,
        object: &SpireRoutingPartitionObject,
    ) -> Result<SpirePlacementEntry, String>;

    fn write_leaf_object_v2_from_rows(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
    ) -> Result<SpirePlacementEntry, String>;
}
