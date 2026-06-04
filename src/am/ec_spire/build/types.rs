
fn elapsed_ms(duration: Duration) -> u128 {
    duration.as_millis()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireIndexedVectorKind {
    Ecvector,
    Tqvector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpireSourceIdentityDatumKind {
    Uuid,
    Bytea16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireSourceIdentityAttribute {
    pub(super) index_attr_offset: usize,
    pub(super) heap_attnum: i32,
    pub(super) datum_kind: SpireSourceIdentityDatumKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireIndexedTupleLayout {
    pub(super) vector_kind: SpireIndexedVectorKind,
    pub(super) source_identity: Option<SpireSourceIdentityAttribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireBuildTuple {
    pub(super) heap_tid: ItemPointer,
    pub(super) dimensions: u16,
    pub(super) assignment: SpireLeafAssignmentInput,
    pub(super) vec_id_source_identity: SpireVecIdSourceIdentity,
    pub(super) source_vector: Vec<f32>,
}

struct SpireBuildState {
    options: options::EcSpireOptions,
    tuple_layout: SpireIndexedTupleLayout,
    scanned_tuples: usize,
    tuples: Vec<SpireBuildTuple>,
    dimensions: Option<u16>,
}

#[derive(Debug, Default)]
struct SpireBuildTiming {
    setup_ms: u128,
    heap_scan_ms: u128,
    sample_collect_ms: u128,
    kmeans_ms: u128,
    kmeans_calls: usize,
    assignment_ms: u128,
    recursive_kmeans_ms: u128,
    recursive_kmeans_calls: usize,
    recursive_kmeans_max_level: u16,
    recursive_assignment_ms: u128,
    recursive_routing_initial_children: usize,
    recursive_routing_final_children: usize,
    recursive_routing_iterations: usize,
    draft_ms: u128,
    draft_total_ms: u128,
    draft_input_clone_ms: u128,
    draft_pid_alloc_ms: u128,
    draft_recursive_routing_ms: u128,
    draft_route_map_ms: u128,
    draft_leaf_rows_ms: u128,
    draft_leaf_inputs_ms: u128,
    draft_validation_ms: u128,
    top_graph_ms: u128,
    object_store_ms: u128,
    object_store_total_ms: u128,
    publish_ms: u128,
}

impl SpireBuildTiming {
    fn add_setup(&mut self, duration: Duration) {
        self.setup_ms += elapsed_ms(duration);
    }

    fn add_heap_scan(&mut self, duration: Duration) {
        self.heap_scan_ms += elapsed_ms(duration);
    }

    fn add_sample_collect(&mut self, duration: Duration) {
        self.sample_collect_ms += elapsed_ms(duration);
    }

    fn add_kmeans(&mut self, duration: Duration) {
        self.kmeans_ms += elapsed_ms(duration);
        self.kmeans_calls += 1;
    }

    fn add_assignment(&mut self, duration: Duration) {
        self.assignment_ms += elapsed_ms(duration);
    }

    fn add_recursive_kmeans(&mut self, level: u16, duration: Duration) {
        self.recursive_kmeans_ms += elapsed_ms(duration);
        self.recursive_kmeans_calls += 1;
        self.recursive_kmeans_max_level = self.recursive_kmeans_max_level.max(level);
        self.recompute_exclusive_draft();
    }

    fn add_recursive_assignment(&mut self, duration: Duration) {
        self.recursive_assignment_ms += elapsed_ms(duration);
        self.recompute_exclusive_draft();
    }

    fn record_recursive_routing_children(
        &mut self,
        initial_children: usize,
        final_children: usize,
        iterations: usize,
    ) {
        self.recursive_routing_initial_children = initial_children;
        self.recursive_routing_final_children = final_children;
        self.recursive_routing_iterations = iterations;
    }

    fn add_draft_total(&mut self, duration: Duration) {
        self.draft_total_ms += elapsed_ms(duration);
        self.recompute_exclusive_draft();
    }

    fn add_draft_input_clone(&mut self, duration: Duration) {
        self.draft_input_clone_ms += elapsed_ms(duration);
        self.recompute_exclusive_draft();
    }

    fn add_draft_pid_alloc(&mut self, duration: Duration) {
        self.draft_pid_alloc_ms += elapsed_ms(duration);
        self.recompute_exclusive_draft();
    }

    fn add_draft_recursive_routing(&mut self, duration: Duration) {
        self.draft_recursive_routing_ms += elapsed_ms(duration);
        self.recompute_exclusive_draft();
    }

    fn add_draft_route_map(&mut self, duration: Duration) {
        self.draft_route_map_ms += elapsed_ms(duration);
        self.recompute_exclusive_draft();
    }

    fn add_draft_leaf_rows(&mut self, duration: Duration) {
        self.draft_leaf_rows_ms += elapsed_ms(duration);
        self.recompute_exclusive_draft();
    }

    fn add_draft_leaf_inputs(&mut self, duration: Duration) {
        self.draft_leaf_inputs_ms += elapsed_ms(duration);
        self.recompute_exclusive_draft();
    }

    fn add_draft_validation(&mut self, duration: Duration) {
        self.draft_validation_ms += elapsed_ms(duration);
        self.recompute_exclusive_draft();
    }

    fn recompute_exclusive_draft(&mut self) {
        self.draft_ms = self
            .draft_total_ms
            .saturating_sub(self.recursive_kmeans_ms)
            .saturating_sub(self.recursive_assignment_ms);
    }

    fn add_top_graph(&mut self, duration: Duration) {
        self.top_graph_ms += elapsed_ms(duration);
        self.recompute_exclusive_object_store();
    }

    fn add_object_store_total(&mut self, duration: Duration) {
        self.object_store_total_ms += elapsed_ms(duration);
        self.recompute_exclusive_object_store();
    }

    fn recompute_exclusive_object_store(&mut self) {
        self.object_store_ms = self.object_store_total_ms.saturating_sub(self.top_graph_ms);
    }

    fn add_publish(&mut self, duration: Duration) {
        self.publish_ms += elapsed_ms(duration);
    }
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
    pub(super) assignments: Vec<SpireLeafAssignmentIdentityInput>,
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
    pub(super) summaries: Vec<SpireLeafBlockSummary>,
    pub(super) summary_block_rows: u32,
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
pub(super) struct SpireRecursiveTopGraphEpochInput {
    pub(super) epoch_input: SpireRecursiveRoutingEpochInput,
    pub(super) top_graph_params: SpireTopGraphBuildParams,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRecursiveRoutingEpochDraft {
    pub(super) epoch_manifest: SpireEpochManifest,
    pub(super) object_manifest: SpireObjectManifest,
    pub(super) placement_directory: SpirePlacementDirectory,
    pub(super) root_pid: u64,
    pub(super) routing_objects: Vec<SpireRoutingPartitionObject>,
    pub(super) top_graph_object: Option<SpireTopGraphPartitionObject>,
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

    fn write_leaf_object_v3_from_rows_and_summaries(
        &mut self,
        epoch: u64,
        pid: u64,
        object_version: u64,
        parent_pid: u64,
        rows: &[SpireLeafAssignmentRow],
        summaries: &[SpireLeafBlockSummary],
        summary_block_rows: u32,
    ) -> Result<SpirePlacementEntry, String>;

    fn write_top_graph_object(
        &mut self,
        epoch: u64,
        object: &SpireTopGraphPartitionObject,
    ) -> Result<SpirePlacementEntry, String>;
}
