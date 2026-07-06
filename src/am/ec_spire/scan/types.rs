#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafScanRow {
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) row_index: u32,
    pub(super) assignment: SpireLeafAssignmentRow,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireDeltaScanRow {
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) row_index: u32,
    pub(super) assignment: SpireLeafAssignmentRow,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireRoutedLeafScanRows {
    pub(super) epoch: u64,
    pub(super) root_pid: u64,
    pub(super) leaf_pid: u64,
    pub(super) rows: Vec<SpireLeafScanRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireScoredScanCandidate {
    pub(super) epoch: u64,
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) row_index: u32,
    pub(super) assignment_flags: u16,
    pub(super) vec_id: SpireVecId,
    pub(super) heap_tid: ItemPointer,
    pub(super) score: f32,
}

#[derive(Debug, Clone)]
struct SpireScoredScanCandidateHeapEntry {
    candidate: SpireScoredScanCandidate,
}

impl PartialEq for SpireScoredScanCandidateHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        scored_candidate_cmp(&self.candidate, &other.candidate) == Ordering::Equal
    }
}

impl Eq for SpireScoredScanCandidateHeapEntry {}

impl PartialOrd for SpireScoredScanCandidateHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SpireScoredScanCandidateHeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        scored_candidate_cmp(&self.candidate, &other.candidate)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SpireLoadedRoutingHierarchy {
    root_pid: u64,
    root_object: SpireRoutingPartitionObject,
    internal_objects_by_pid: HashMap<u64, SpireRoutingPartitionObject>,
    leaf_assignment_counts_by_pid: HashMap<u64, usize>,
    top_graph: Option<(u64, SpireTopGraphPartitionObject)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireRoutingHierarchyCacheKey {
    pub(super) index_relid: u32,
    pub(super) active_epoch: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireResolvedScanPlanSelection {
    pub(super) scan_plan: SpireSingleLevelScanPlan,
    pub(super) selected_leaf_pids: Vec<u64>,
    pub(super) routing_hierarchy_load_count: u64,
    pub(super) top_graph_load_count: u64,
    pub(super) leaf_count_elapsed_us: u64,
    pub(super) route_select_elapsed_us: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpireRecursiveLeafRoute {
    leaf_pid: u64,
    parent_pid: u64,
    route_score: f32,
    leaf_score: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireLeafRouteSelection {
    routes: Vec<SpireRecursiveLeafRoute>,
    stopped_by_row_budget: bool,
    stopped_by_max_leaf_routes: bool,
    stopped_by_probe_distance_ratio: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpireRoutingChildRoute {
    centroid_index: u32,
    child_pid: u64,
    score: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireRecursiveParentRoute {
    parent: SpireRoutingPartitionObject,
    path_score: f32,
}

// Recursive routing accumulates inner-product scores across levels. Top-graph
// routes enter this same contract by converting their search distance back to a
// score before the recursive descent continues.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SpireRecursiveScoredChildRoute {
    parent_pid: u64,
    parent_level: u16,
    child_pid: u64,
    centroid_index: u32,
    path_score: f32,
    score: f32,
}

impl SpireRecursiveScoredChildRoute {
    fn total_score(&self) -> f32 {
        self.path_score + self.score
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireDeltaObjectRoute {
    delta_pid: u64,
    parent_leaf_pid: u64,
    placement: SpirePlacementEntry,
    object_version: u64,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireLoadedDeltaObjectRoute {
    route: SpireDeltaObjectRoute,
    rows: Vec<SpireDeltaScanRow>,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireLoadedQuantizedLeafRoute {
    route: SpireLeafObjectReadRoute,
    leaf_object: SpireLeafPartitionObjectV2,
    selected_row_ranges: Option<Vec<SpireLeafBlockRowRange>>,
    loaded_delta_routes: Vec<SpireLoadedDeltaObjectRoute>,
    deleted_vec_ids: HashSet<SpireVecId>,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireLoadedQuantizedLeafSummaryRoute {
    route: SpireLeafObjectReadRoute,
    leaf_summaries: SpireLeafPartitionObjectV2Summaries,
    selected_row_ranges: Option<Vec<SpireLeafBlockRowRange>>,
    loaded_delta_routes: Vec<SpireLoadedDeltaObjectRoute>,
    deleted_vec_ids: HashSet<SpireVecId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpireLeafObjectReadRoute {
    leaf_pid: u64,
    parent_pid: u64,
    route_score: f32,
    placement: SpirePlacementEntry,
    object_version: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireLeafBlockRowRange {
    row_base: u32,
    row_end: u32,
    row_segment_locator: ItemPointer,
}

#[derive(Debug, Clone, PartialEq)]
struct SpireStoreObjectReadGroup {
    node_id: u32,
    local_store_id: u32,
    leaf_routes: Vec<SpireLeafObjectReadRoute>,
    delta_routes: Vec<SpireDeltaObjectRoute>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SpireScanOutput {
    pub(super) heap_tid: ItemPointer,
    pub(super) orderby_score: f32,
}

impl From<&SpireScoredScanCandidate> for SpireScanOutput {
    fn from(candidate: &SpireScoredScanCandidate) -> Self {
        Self {
            heap_tid: candidate.heap_tid,
            orderby_score: candidate.score,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpirePreparedScanCandidates {
    pub(super) scan_plan: SpireSingleLevelScanPlan,
    pub(super) candidates: Vec<SpireScoredScanCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireRoutingLevelDiagnostics {
    pub(super) level: u16,
    pub(super) effective_nprobe: u32,
    pub(super) effective_nprobe_source: &'static str,
    pub(super) adaptive_nprobe_decision: &'static str,
    pub(super) input_frontier_width: usize,
    pub(super) expanded_parent_count: usize,
    pub(super) selected_child_count: usize,
    pub(super) deduped_route_count: usize,
    pub(super) truncation_reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireScanRoutingDiagnostics {
    pub(super) scan_plan: SpireSingleLevelScanPlan,
    pub(super) levels: Vec<SpireRoutingLevelDiagnostics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireStoreScanDiagnostics {
    pub(super) epoch: u64,
    pub(super) node_id: u32,
    pub(super) local_store_id: u32,
    pub(super) route_count: usize,
    pub(super) leaf_route_count: usize,
    pub(super) delta_route_count: usize,
    pub(super) prefetched_object_count: usize,
    pub(super) prefetched_object_bytes: u64,
    pub(super) read_batch_count: usize,
    pub(super) scanned_pid_count: usize,
    pub(super) leaf_pid_count: usize,
    pub(super) delta_pid_count: usize,
    pub(super) delta_decode_count: usize,
    pub(super) candidate_row_count: usize,
    pub(super) leaf_candidate_row_count: usize,
    pub(super) delta_candidate_row_count: usize,
    pub(super) leaf_block_available_count: usize,
    pub(super) leaf_block_selected_count: usize,
    pub(super) leaf_block_skipped_count: usize,
    pub(super) leaf_summary_object_bytes: u64,
    pub(super) leaf_row_object_bytes: u64,
    pub(super) leaf_row_segment_read_count: usize,
    pub(super) leaf_row_segment_read_bytes: u64,
    pub(super) primary_candidate_row_count: usize,
    pub(super) boundary_replica_candidate_row_count: usize,
    pub(super) deduped_candidate_row_count: usize,
    pub(super) deduped_primary_candidate_row_count: usize,
    pub(super) deduped_boundary_replica_candidate_row_count: usize,
    pub(super) truncated_candidate_row_count: usize,
    pub(super) pre_materialization_pruned_candidate_row_count: usize,
    pub(super) truncated_primary_candidate_row_count: usize,
    pub(super) truncated_boundary_replica_candidate_row_count: usize,
    pub(super) candidate_winner_count: usize,
    pub(super) primary_candidate_winner_count: usize,
    pub(super) boundary_replica_candidate_winner_count: usize,
    pub(super) delete_delta_row_count: usize,
    pub(super) dropped_unselected_delta_route_count: usize,
    pub(super) leaf_object_read_nanos: u64,
    pub(super) leaf_summary_score_nanos: u64,
    pub(super) leaf_row_score_nanos: u64,
    pub(super) candidate_score_nanos: u64,
    pub(super) candidate_materialize_nanos: u64,
    pub(super) candidate_heap_append_nanos: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireLeafScanDiagnostics {
    pub(super) epoch: u64,
    pub(super) pid: u64,
    pub(super) node_id: u32,
    pub(super) local_store_id: u32,
    pub(super) object_version: u64,
    pub(super) object_bytes: u32,
    pub(super) route_count: usize,
    pub(super) scanned_count: usize,
    pub(super) candidate_row_count: usize,
    pub(super) leaf_block_available_count: usize,
    pub(super) leaf_block_selected_count: usize,
    pub(super) leaf_block_skipped_count: usize,
    pub(super) leaf_summary_object_bytes: u64,
    pub(super) leaf_row_object_bytes: u64,
    pub(super) leaf_row_segment_read_count: usize,
    pub(super) leaf_row_segment_read_bytes: u64,
    pub(super) primary_candidate_row_count: usize,
    pub(super) boundary_replica_candidate_row_count: usize,
    pub(super) deduped_candidate_row_count: usize,
    pub(super) truncated_candidate_row_count: usize,
    pub(super) pre_materialization_pruned_candidate_row_count: usize,
    pub(super) candidate_winner_count: usize,
    pub(super) leaf_object_read_nanos: u64,
    pub(super) leaf_summary_score_nanos: u64,
    pub(super) leaf_row_score_nanos: u64,
    pub(super) candidate_score_nanos: u64,
    pub(super) candidate_materialize_nanos: u64,
    pub(super) candidate_heap_append_nanos: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireRerankLocalityDiagnostics {
    pub(crate) active_epoch: u64,
    pub(crate) effective_nprobe: u32,
    pub(crate) effective_nprobe_source: &'static str,
    pub(crate) effective_rerank_width: u64,
    pub(crate) effective_rerank_width_source: &'static str,
    pub(crate) candidate_count: u64,
    pub(crate) rerank_prefix_count: u64,
    pub(crate) unique_heap_block_count: u64,
    pub(crate) heap_block_transition_count: u64,
    pub(crate) heap_block_span: u64,
    pub(crate) heap_block_jump_sum: u64,
    pub(crate) heap_block_jump_max: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireScanPlacementDiagnostics {
    pub(super) scan_plan: SpireSingleLevelScanPlan,
    pub(super) stores: Vec<SpireStoreScanDiagnostics>,
    pub(super) leaves: Vec<SpireLeafScanDiagnostics>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireSelectedLeafScanProfile {
    pub(crate) served_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) selected_pid_count: u64,
    pub(crate) scanned_pid_count: u64,
    pub(crate) leaf_candidate_row_count: u64,
    pub(crate) deduped_candidate_row_count: u64,
    pub(crate) truncated_candidate_row_count: u64,
    pub(crate) pre_materialization_pruned_candidate_row_count: u64,
    pub(crate) candidate_winner_count: u64,
    pub(crate) leaf_block_available_count: u64,
    pub(crate) leaf_block_selected_count: u64,
    pub(crate) leaf_block_skipped_count: u64,
    pub(crate) sound_upper_bound_available_count: u64,
    pub(crate) sound_upper_bound_missing_count: u64,
    pub(crate) leaf_summary_score_nanos: u64,
    pub(crate) leaf_row_score_nanos: u64,
    pub(crate) candidate_score_nanos: u64,
    pub(crate) local_kth_score: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireSelectedLeafThresholdProfile {
    pub(crate) served_epoch: u64,
    pub(crate) node_id: u32,
    pub(crate) selected_pid_count: u64,
    pub(crate) evaluated_pid_count: u64,
    pub(crate) threshold_score: f32,
    pub(crate) threshold_ip: f32,
    pub(crate) sound_upper_bound_available_count: u64,
    pub(crate) sound_upper_bound_missing_count: u64,
    pub(crate) threshold_block_available_count: u64,
    pub(crate) threshold_block_selected_count: u64,
    pub(crate) threshold_block_skipped_count: u64,
    pub(crate) threshold_row_available_count: u64,
    pub(crate) threshold_row_selected_count: u64,
    pub(crate) threshold_row_skipped_count: u64,
    pub(crate) leaf_summary_score_nanos: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireLeafBlockRankSnapshotRow {
    pub(super) active_epoch: u64,
    pub(super) effective_nprobe: u32,
    pub(super) effective_nprobe_source: &'static str,
    pub(super) effective_rerank_width: u64,
    pub(super) effective_rerank_width_source: &'static str,
    pub(super) target_ordinal: u64,
    pub(super) target_local_sequence: u64,
    pub(super) status: &'static str,
    pub(super) max_global_blocks: u64,
    pub(super) radius_weight: f64,
    pub(super) scored_block_count: u64,
    pub(super) block_rank: Option<u64>,
    pub(super) selected_by_global_cap: Option<bool>,
    pub(super) pid: Option<u64>,
    pub(super) node_id: Option<u32>,
    pub(super) local_store_id: Option<u32>,
    pub(super) object_version: Option<u64>,
    pub(super) row_index: Option<u32>,
    pub(super) row_base: Option<u32>,
    pub(super) row_end: Option<u32>,
    pub(super) row_count: Option<u32>,
    pub(super) block_ip: Option<f32>,
    pub(super) cap_block_ip: Option<f32>,
    pub(super) block_ip_margin_to_cap: Option<f32>,
    pub(super) route_rank: Option<u64>,
    pub(super) route_score: Option<f32>,
    pub(super) assignment_flags: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireTargetCandidateRankSnapshotRow {
    pub(super) active_epoch: u64,
    pub(super) effective_nprobe: u32,
    pub(super) effective_nprobe_source: &'static str,
    pub(super) effective_rerank_width: u64,
    pub(super) effective_rerank_width_source: &'static str,
    pub(super) target_ordinal: u64,
    pub(super) target_local_sequence: u64,
    pub(super) status: &'static str,
    pub(super) approximate_candidate_count: u64,
    pub(super) rerank_prefix_count: u64,
    pub(super) approximate_rank: Option<u64>,
    pub(super) selected_by_rerank_prefix: Option<bool>,
    pub(super) pid: Option<u64>,
    pub(super) node_id: Option<u32>,
    pub(super) local_store_id: Option<u32>,
    pub(super) object_version: Option<u64>,
    pub(super) row_index: Option<u32>,
    pub(super) assignment_flags: Option<u16>,
    pub(super) approximate_score: Option<f32>,
    pub(super) heap_block: Option<u32>,
    pub(super) heap_offset: Option<u16>,
}

trait SpireRoutedScanObserver {
    fn wants_candidate_timing(&self) -> bool {
        false
    }

    fn routed_leaf(&mut self, _epoch: u64, _placement: &SpirePlacementEntry) {}

    fn routed_delta(&mut self, _epoch: u64, _placement: &SpirePlacementEntry) {}

    fn store_read_batch(&mut self, _epoch: u64, _node_id: u32, _local_store_id: u32) {}

    fn prefetched_object(&mut self, _epoch: u64, _placement: &SpirePlacementEntry) {}

    fn scanned_leaf(&mut self, _epoch: u64, _placement: &SpirePlacementEntry) {}

    fn scanned_delta(&mut self, _epoch: u64, _placement: &SpirePlacementEntry) {}

    fn delete_delta_row(&mut self, _epoch: u64, _placement: &SpirePlacementEntry) {}

    fn dropped_unselected_delta_route(&mut self, _epoch: u64, _placement: &SpirePlacementEntry) {}

    fn visible_leaf_candidate(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _assignment_flags: u16,
    ) {
    }

    fn visible_delta_candidate(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _assignment_flags: u16,
    ) {
    }

    fn deduped_candidate(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _assignment_flags: u16,
    ) {
    }

    fn truncated_candidate(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _assignment_flags: u16,
    ) {
    }

    fn pre_materialization_pruned_candidate(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        assignment_flags: u16,
    ) {
        self.truncated_candidate(epoch, placement, assignment_flags);
    }

    fn candidate_winner(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _assignment_flags: u16,
    ) {
    }

    fn leaf_object_read_time(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _nanos: u64,
    ) {
    }

    fn candidate_score_time(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _nanos: u64,
    ) {
    }

    fn leaf_summary_score_time(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        nanos: u64,
    ) {
        self.candidate_score_time(epoch, placement, nanos);
    }

    fn leaf_row_score_time(&mut self, epoch: u64, placement: &SpirePlacementEntry, nanos: u64) {
        self.candidate_score_time(epoch, placement, nanos);
    }

    fn leaf_block_storage(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _summary_object_bytes: u64,
        _row_object_bytes: u64,
    ) {
    }

    fn leaf_row_segments_read(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _segment_count: usize,
        _segment_bytes: u64,
    ) {
    }

    fn leaf_block_selection(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _available_blocks: usize,
        _selected_blocks: usize,
    ) {
    }

    fn candidate_materialize_time(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _nanos: u64,
    ) {
    }

    fn candidate_heap_append_time(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _nanos: u64,
    ) {
    }
}

struct SpireNoopRoutedScanObserver;

impl SpireRoutedScanObserver for SpireNoopRoutedScanObserver {}

struct SpireScanPlacementDiagnosticsObserver {
    by_store: BTreeMap<(u32, u32), SpireStoreScanDiagnostics>,
    by_leaf: BTreeMap<u64, SpireLeafScanDiagnostics>,
}

impl SpireScanPlacementDiagnosticsObserver {
    fn new() -> Self {
        Self {
            by_store: BTreeMap::new(),
            by_leaf: BTreeMap::new(),
        }
    }

    fn into_diagnostics(self) -> (Vec<SpireStoreScanDiagnostics>, Vec<SpireLeafScanDiagnostics>) {
        (
            self.by_store.into_values().collect(),
            self.by_leaf.into_values().collect(),
        )
    }

    #[cfg(test)]
    fn into_stores(self) -> Vec<SpireStoreScanDiagnostics> {
        self.by_store.into_values().collect()
    }

    fn entry(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
    ) -> &mut SpireStoreScanDiagnostics {
        self.entry_by_store(epoch, placement.node_id, placement.local_store_id)
    }

    fn entry_by_store(
        &mut self,
        epoch: u64,
        node_id: u32,
        local_store_id: u32,
    ) -> &mut SpireStoreScanDiagnostics {
        self.by_store
            .entry((node_id, local_store_id))
            .or_insert_with(|| SpireStoreScanDiagnostics {
                epoch,
                node_id,
                local_store_id,
                route_count: 0,
                leaf_route_count: 0,
                delta_route_count: 0,
                prefetched_object_count: 0,
                prefetched_object_bytes: 0,
                read_batch_count: 0,
                scanned_pid_count: 0,
                leaf_pid_count: 0,
                delta_pid_count: 0,
                delta_decode_count: 0,
                candidate_row_count: 0,
                leaf_candidate_row_count: 0,
                delta_candidate_row_count: 0,
                leaf_block_available_count: 0,
                leaf_block_selected_count: 0,
                leaf_block_skipped_count: 0,
                leaf_summary_object_bytes: 0,
                leaf_row_object_bytes: 0,
                leaf_row_segment_read_count: 0,
                leaf_row_segment_read_bytes: 0,
                primary_candidate_row_count: 0,
                boundary_replica_candidate_row_count: 0,
                deduped_candidate_row_count: 0,
                deduped_primary_candidate_row_count: 0,
                deduped_boundary_replica_candidate_row_count: 0,
                truncated_candidate_row_count: 0,
                pre_materialization_pruned_candidate_row_count: 0,
                truncated_primary_candidate_row_count: 0,
                truncated_boundary_replica_candidate_row_count: 0,
                candidate_winner_count: 0,
                primary_candidate_winner_count: 0,
                boundary_replica_candidate_winner_count: 0,
                delete_delta_row_count: 0,
                dropped_unselected_delta_route_count: 0,
                leaf_object_read_nanos: 0,
                leaf_summary_score_nanos: 0,
                leaf_row_score_nanos: 0,
                candidate_score_nanos: 0,
                candidate_materialize_nanos: 0,
                candidate_heap_append_nanos: 0,
            })
    }

    fn leaf_entry(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
    ) -> &mut SpireLeafScanDiagnostics {
        self.by_leaf
            .entry(placement.pid)
            .or_insert_with(|| SpireLeafScanDiagnostics {
                epoch,
                pid: placement.pid,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                object_version: placement.object_version,
                object_bytes: placement.object_bytes,
                route_count: 0,
                scanned_count: 0,
                candidate_row_count: 0,
                leaf_block_available_count: 0,
                leaf_block_selected_count: 0,
                leaf_block_skipped_count: 0,
                leaf_summary_object_bytes: 0,
                leaf_row_object_bytes: 0,
                leaf_row_segment_read_count: 0,
                leaf_row_segment_read_bytes: 0,
                primary_candidate_row_count: 0,
                boundary_replica_candidate_row_count: 0,
                deduped_candidate_row_count: 0,
                truncated_candidate_row_count: 0,
                pre_materialization_pruned_candidate_row_count: 0,
                candidate_winner_count: 0,
                leaf_object_read_nanos: 0,
                leaf_summary_score_nanos: 0,
                leaf_row_score_nanos: 0,
                candidate_score_nanos: 0,
                candidate_materialize_nanos: 0,
                candidate_heap_append_nanos: 0,
            })
    }

    fn leaf_entry_if_routed(
        &mut self,
        placement: &SpirePlacementEntry,
    ) -> Option<&mut SpireLeafScanDiagnostics> {
        self.by_leaf.get_mut(&placement.pid)
    }
}

impl SpireRoutedScanObserver for SpireScanPlacementDiagnosticsObserver {
    fn wants_candidate_timing(&self) -> bool {
        true
    }

    fn routed_leaf(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        let entry = self.entry(epoch, placement);
        entry.route_count += 1;
        entry.leaf_route_count += 1;
        self.leaf_entry(epoch, placement).route_count += 1;
    }

    fn routed_delta(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        let entry = self.entry(epoch, placement);
        entry.route_count += 1;
        entry.delta_route_count += 1;
    }

    fn store_read_batch(&mut self, epoch: u64, node_id: u32, local_store_id: u32) {
        self.entry_by_store(epoch, node_id, local_store_id)
            .read_batch_count += 1;
    }

    fn prefetched_object(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        let entry = self.entry(epoch, placement);
        entry.prefetched_object_count += 1;
        entry.prefetched_object_bytes += u64::from(placement.object_bytes);
    }

    fn scanned_leaf(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        let entry = self.entry(epoch, placement);
        entry.scanned_pid_count += 1;
        entry.leaf_pid_count += 1;
        self.leaf_entry(epoch, placement).scanned_count += 1;
    }

    fn scanned_delta(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        let entry = self.entry(epoch, placement);
        entry.scanned_pid_count += 1;
        entry.delta_pid_count += 1;
        entry.delta_decode_count += 1;
    }

    fn delete_delta_row(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        self.entry(epoch, placement).delete_delta_row_count += 1;
    }

    fn dropped_unselected_delta_route(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        self.entry(epoch, placement)
            .dropped_unselected_delta_route_count += 1;
    }

    fn visible_leaf_candidate(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        assignment_flags: u16,
    ) {
        let entry = self.entry(epoch, placement);
        entry.candidate_row_count += 1;
        entry.leaf_candidate_row_count += 1;
        count_candidate_role(
            assignment_flags,
            &mut entry.primary_candidate_row_count,
            &mut entry.boundary_replica_candidate_row_count,
        );
        let leaf = self.leaf_entry(epoch, placement);
        leaf.candidate_row_count += 1;
        count_candidate_role(
            assignment_flags,
            &mut leaf.primary_candidate_row_count,
            &mut leaf.boundary_replica_candidate_row_count,
        );
    }

    fn visible_delta_candidate(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        assignment_flags: u16,
    ) {
        let entry = self.entry(epoch, placement);
        entry.candidate_row_count += 1;
        entry.delta_candidate_row_count += 1;
        count_candidate_role(
            assignment_flags,
            &mut entry.primary_candidate_row_count,
            &mut entry.boundary_replica_candidate_row_count,
        );
    }

    fn deduped_candidate(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        assignment_flags: u16,
    ) {
        let entry = self.entry(epoch, placement);
        entry.deduped_candidate_row_count += 1;
        count_candidate_role(
            assignment_flags,
            &mut entry.deduped_primary_candidate_row_count,
            &mut entry.deduped_boundary_replica_candidate_row_count,
        );
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.deduped_candidate_row_count += 1;
        }
    }

    fn truncated_candidate(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        assignment_flags: u16,
    ) {
        let entry = self.entry(epoch, placement);
        entry.truncated_candidate_row_count += 1;
        count_candidate_role(
            assignment_flags,
            &mut entry.truncated_primary_candidate_row_count,
            &mut entry.truncated_boundary_replica_candidate_row_count,
        );
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.truncated_candidate_row_count += 1;
        }
    }

    fn pre_materialization_pruned_candidate(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        assignment_flags: u16,
    ) {
        let entry = self.entry(epoch, placement);
        entry.truncated_candidate_row_count += 1;
        entry.pre_materialization_pruned_candidate_row_count += 1;
        count_candidate_role(
            assignment_flags,
            &mut entry.truncated_primary_candidate_row_count,
            &mut entry.truncated_boundary_replica_candidate_row_count,
        );
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.truncated_candidate_row_count += 1;
            leaf.pre_materialization_pruned_candidate_row_count += 1;
        }
    }

    fn candidate_winner(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        assignment_flags: u16,
    ) {
        let entry = self.entry(epoch, placement);
        entry.candidate_winner_count += 1;
        count_candidate_role(
            assignment_flags,
            &mut entry.primary_candidate_winner_count,
            &mut entry.boundary_replica_candidate_winner_count,
        );
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.candidate_winner_count += 1;
        }
    }

    fn leaf_object_read_time(&mut self, epoch: u64, placement: &SpirePlacementEntry, nanos: u64) {
        self.entry(epoch, placement).leaf_object_read_nanos += nanos;
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.leaf_object_read_nanos += nanos;
        }
    }

    fn candidate_score_time(&mut self, epoch: u64, placement: &SpirePlacementEntry, nanos: u64) {
        self.entry(epoch, placement).candidate_score_nanos += nanos;
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.candidate_score_nanos += nanos;
        }
    }

    fn leaf_summary_score_time(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        nanos: u64,
    ) {
        let entry = self.entry(epoch, placement);
        entry.leaf_summary_score_nanos += nanos;
        entry.candidate_score_nanos += nanos;
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.leaf_summary_score_nanos += nanos;
            leaf.candidate_score_nanos += nanos;
        }
    }

    fn leaf_row_score_time(&mut self, epoch: u64, placement: &SpirePlacementEntry, nanos: u64) {
        let entry = self.entry(epoch, placement);
        entry.leaf_row_score_nanos += nanos;
        entry.candidate_score_nanos += nanos;
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.leaf_row_score_nanos += nanos;
            leaf.candidate_score_nanos += nanos;
        }
    }

    fn leaf_block_storage(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        summary_object_bytes: u64,
        row_object_bytes: u64,
    ) {
        let entry = self.entry(epoch, placement);
        entry.leaf_summary_object_bytes += summary_object_bytes;
        entry.leaf_row_object_bytes += row_object_bytes;
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.leaf_summary_object_bytes += summary_object_bytes;
            leaf.leaf_row_object_bytes += row_object_bytes;
        }
    }

    fn leaf_row_segments_read(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        segment_count: usize,
        segment_bytes: u64,
    ) {
        let entry = self.entry(epoch, placement);
        entry.leaf_row_segment_read_count += segment_count;
        entry.leaf_row_segment_read_bytes += segment_bytes;
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.leaf_row_segment_read_count += segment_count;
            leaf.leaf_row_segment_read_bytes += segment_bytes;
        }
    }

    fn leaf_block_selection(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        available_blocks: usize,
        selected_blocks: usize,
    ) {
        let selected_blocks = selected_blocks.min(available_blocks);
        let skipped_blocks = available_blocks - selected_blocks;
        let entry = self.entry(epoch, placement);
        entry.leaf_block_available_count += available_blocks;
        entry.leaf_block_selected_count += selected_blocks;
        entry.leaf_block_skipped_count += skipped_blocks;
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.leaf_block_available_count += available_blocks;
            leaf.leaf_block_selected_count += selected_blocks;
            leaf.leaf_block_skipped_count += skipped_blocks;
        }
    }

    fn candidate_materialize_time(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        nanos: u64,
    ) {
        self.entry(epoch, placement).candidate_materialize_nanos += nanos;
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.candidate_materialize_nanos += nanos;
        }
    }

    fn candidate_heap_append_time(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
        nanos: u64,
    ) {
        self.entry(epoch, placement).candidate_heap_append_nanos += nanos;
        if let Some(leaf) = self.leaf_entry_if_routed(placement) {
            leaf.candidate_heap_append_nanos += nanos;
        }
    }
}

fn count_candidate_role(flags: u16, primary_count: &mut usize, replica_count: &mut usize) {
    if flags & SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0 {
        *replica_count += 1;
    } else {
        *primary_count += 1;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireScanQuery {
    pub(super) dimensions: u16,
    values: Vec<f32>,
}

impl SpireScanQuery {
    pub(super) fn new(values: Vec<f32>) -> Result<Self, String> {
        if values.is_empty() {
            return Err("ec_spire scan query must not be empty".to_owned());
        }
        let dimensions = u16::try_from(values.len()).map_err(|_| {
            format!(
                "ec_spire scan query dimension {} exceeds maximum {}",
                values.len(),
                u16::MAX
            )
        })?;
        if values.iter().any(|value| !value.is_finite()) {
            return Err("ec_spire scan query contains a non-finite value".to_owned());
        }
        let norm_sq = values
            .iter()
            .map(|value| (*value as f64) * (*value as f64))
            .sum::<f64>();
        if norm_sq <= f64::EPSILON {
            return Err("ec_spire scan query requires a non-zero vector".to_owned());
        }

        Ok(Self { dimensions, values })
    }

    pub(super) fn values(&self) -> &[f32] {
        &self.values
    }
}

#[derive(Debug)]
struct SpireScanOpaque {
    rescan_called: bool,
    query: Option<SpireScanQuery>,
    scan_plan: Option<SpireSingleLevelScanPlan>,
    cursor: SpireScanOutputCursor,
    // Cached for diagnostics and tests; every rescan replaces this with the
    // root/control page just read so scan-side cursor fields cannot go stale.
    root_control: Option<SpireRootControlState>,
}

impl Default for SpireScanOpaque {
    fn default() -> Self {
        Self {
            rescan_called: false,
            query: None,
            scan_plan: None,
            cursor: SpireScanOutputCursor::default(),
            root_control: None,
        }
    }
}

impl SpireScanOpaque {
    fn reset_for_candidates(
        &mut self,
        query: SpireScanQuery,
        scan_plan: SpireSingleLevelScanPlan,
        candidates: Vec<SpireScoredScanCandidate>,
    ) {
        let outputs = candidates.iter().map(SpireScanOutput::from).collect();
        self.reset_for_outputs(query, Some(scan_plan), outputs);
    }

    fn reset_for_outputs(
        &mut self,
        query: SpireScanQuery,
        scan_plan: Option<SpireSingleLevelScanPlan>,
        outputs: Vec<SpireScanOutput>,
    ) {
        self.rescan_called = true;
        self.query = Some(query);
        self.scan_plan = scan_plan;
        self.cursor.reset(outputs);
    }

    fn clear_scan_work(&mut self) {
        self.rescan_called = false;
        self.query = None;
        self.scan_plan = None;
        self.cursor.reset(Vec::new());
    }

    /// Read and cache the SPIRE root-control state observed at rescan
    /// time.
    ///
    /// # Safety
    ///
    /// `index_relation` must be the live SPIRE index relation that the AM
    /// scan rescan callback is operating on; PostgreSQL guarantees the
    /// relation is open and locked for the duration of the rescan call.
    /// The helper only reads the root-control page; it must not be called
    /// concurrently with a publish that mutates that page (the AM scan
    /// state machine ensures this).
    unsafe fn root_control_for_rescan(
        &mut self,
        index_relation: pg_sys::Relation,
    ) -> SpireRootControlState {
        // SAFETY: AM rescan invokes this with the live scan index relation, and
        // this helper only reads the root-control page before caching the result.
        let observed = page::read_root_control_page(index_relation);
        self.observe_root_control_for_rescan(observed)
    }

    fn observe_root_control_for_rescan(
        &mut self,
        observed: SpireRootControlState,
    ) -> SpireRootControlState {
        self.root_control = Some(observed);
        observed
    }

    fn next_output(&mut self) -> Option<SpireScanOutput> {
        self.cursor.next_output()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireScanOutputCursor {
    outputs: Vec<SpireScanOutput>,
    next_index: usize,
}

impl SpireScanOutputCursor {
    pub(super) fn new(outputs: Vec<SpireScanOutput>) -> Self {
        Self {
            outputs,
            next_index: 0,
        }
    }

    pub(super) fn remaining(&self) -> usize {
        self.outputs.len().saturating_sub(self.next_index)
    }

    pub(super) fn is_exhausted(&self) -> bool {
        self.remaining() == 0
    }

    pub(super) fn next_output(&mut self) -> Option<SpireScanOutput> {
        if self.next_index >= self.outputs.len() {
            return None;
        }
        let output_index = self.next_index;
        self.next_index += 1;
        self.outputs.get(output_index).copied()
    }

    pub(super) fn reset(&mut self, outputs: Vec<SpireScanOutput>) {
        *self = Self::new(outputs);
    }
}

impl Default for SpireScanOutputCursor {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireScanCandidateCursor {
    candidates: Vec<SpireScoredScanCandidate>,
    next_index: usize,
}

impl SpireScanCandidateCursor {
    pub(super) fn new(candidates: Vec<SpireScoredScanCandidate>) -> Self {
        debug_assert!(candidates
            .windows(2)
            .all(|window| scored_candidate_cmp(&window[0], &window[1]) != Ordering::Greater));
        Self {
            candidates,
            next_index: 0,
        }
    }

    pub(super) fn remaining(&self) -> usize {
        self.candidates.len().saturating_sub(self.next_index)
    }

    pub(super) fn is_exhausted(&self) -> bool {
        self.remaining() == 0
    }

    pub(super) fn next_candidate(&mut self) -> Option<&SpireScoredScanCandidate> {
        if self.next_index >= self.candidates.len() {
            return None;
        }
        let candidate_index = self.next_index;
        self.next_index += 1;
        self.candidates.get(candidate_index)
    }

    pub(super) fn next_output(&mut self) -> Option<SpireScanOutput> {
        self.next_candidate().map(SpireScanOutput::from)
    }

    pub(super) fn reset(&mut self, candidates: Vec<SpireScoredScanCandidate>) {
        *self = Self::new(candidates);
    }
}

impl Default for SpireScanCandidateCursor {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

fn production_scan_output_is_am_deliverable(
    output: &super::SpireRemoteProductionScanOutputRow,
) -> bool {
    matches!(
        output.heap_lookup_owner,
        super::SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION
    )
}

fn production_scan_output_to_am_output(
    output: &super::SpireRemoteProductionScanOutputRow,
) -> Result<SpireScanOutput, String> {
    if !production_scan_output_is_am_deliverable(output) {
        return Err(format!(
            "ec_spire production scan output for node_id {} cannot be delivered as coordinator xs_heaptid; next step is {}",
            output.node_id,
            super::SPIRE_REMOTE_EXECUTOR_STEP_CUSTOM_SCAN_TUPLE_DELIVERY
        ));
    }

    Ok(SpireScanOutput {
        heap_tid: ItemPointer {
            block_number: output.heap_block,
            offset_number: output.heap_offset,
        },
        orderby_score: output.score,
    })
}

fn production_scan_result_stream_am_outputs(
    stream: &super::SpireRemoteProductionScanResultStream,
) -> Result<Vec<SpireScanOutput>, String> {
    if stream.am_delivery.next_blocker != super::SPIRE_REMOTE_NONE {
        return Err(format!(
            "ec_spire production scan AM tuple delivery blocked: status {}, next_blocker {}, recommendation {}",
            stream.am_delivery.status,
            stream.am_delivery.next_blocker,
            stream.am_delivery.recommendation
        ));
    }
    if stream.am_delivery.remote_origin_output_count != 0 {
        return Err(format!(
            "ec_spire production scan AM tuple delivery requires {} for {} remote-origin output(s)",
            super::SPIRE_REMOTE_EXECUTOR_STEP_CUSTOM_SCAN_TUPLE_DELIVERY,
            stream.am_delivery.remote_origin_output_count
        ));
    }

    stream
        .outputs
        .iter()
        .map(production_scan_output_to_am_output)
        .collect()
}
