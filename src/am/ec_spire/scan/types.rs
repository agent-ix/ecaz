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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireRecursiveLeafRoute {
    leaf_pid: u64,
    parent_pid: u64,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireLeafObjectReadRoute {
    leaf_pid: u64,
    parent_pid: u64,
    placement: SpirePlacementEntry,
    object_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub(super) scanned_pid_count: usize,
    pub(super) leaf_pid_count: usize,
    pub(super) delta_pid_count: usize,
    pub(super) candidate_row_count: usize,
    pub(super) leaf_candidate_row_count: usize,
    pub(super) delta_candidate_row_count: usize,
    pub(super) primary_candidate_row_count: usize,
    pub(super) boundary_replica_candidate_row_count: usize,
    pub(super) deduped_candidate_row_count: usize,
    pub(super) deduped_primary_candidate_row_count: usize,
    pub(super) deduped_boundary_replica_candidate_row_count: usize,
    pub(super) truncated_candidate_row_count: usize,
    pub(super) truncated_primary_candidate_row_count: usize,
    pub(super) truncated_boundary_replica_candidate_row_count: usize,
    pub(super) candidate_winner_count: usize,
    pub(super) primary_candidate_winner_count: usize,
    pub(super) boundary_replica_candidate_winner_count: usize,
    pub(super) delete_delta_row_count: usize,
    pub(super) dropped_unselected_delta_route_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireScanPlacementDiagnostics {
    pub(super) scan_plan: SpireSingleLevelScanPlan,
    pub(super) stores: Vec<SpireStoreScanDiagnostics>,
}

trait SpireRoutedScanObserver {
    fn routed_leaf(&mut self, _epoch: u64, _placement: &SpirePlacementEntry) {}

    fn routed_delta(&mut self, _epoch: u64, _placement: &SpirePlacementEntry) {}

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

    fn candidate_winner(
        &mut self,
        _epoch: u64,
        _placement: &SpirePlacementEntry,
        _assignment_flags: u16,
    ) {
    }
}

struct SpireNoopRoutedScanObserver;

impl SpireRoutedScanObserver for SpireNoopRoutedScanObserver {}

struct SpireScanPlacementDiagnosticsObserver {
    by_store: BTreeMap<(u32, u32), SpireStoreScanDiagnostics>,
}

impl SpireScanPlacementDiagnosticsObserver {
    fn new() -> Self {
        Self {
            by_store: BTreeMap::new(),
        }
    }

    fn into_stores(self) -> Vec<SpireStoreScanDiagnostics> {
        self.by_store.into_values().collect()
    }

    fn entry(
        &mut self,
        epoch: u64,
        placement: &SpirePlacementEntry,
    ) -> &mut SpireStoreScanDiagnostics {
        self.by_store
            .entry((placement.node_id, placement.local_store_id))
            .or_insert_with(|| SpireStoreScanDiagnostics {
                epoch,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                route_count: 0,
                leaf_route_count: 0,
                delta_route_count: 0,
                prefetched_object_count: 0,
                scanned_pid_count: 0,
                leaf_pid_count: 0,
                delta_pid_count: 0,
                candidate_row_count: 0,
                leaf_candidate_row_count: 0,
                delta_candidate_row_count: 0,
                primary_candidate_row_count: 0,
                boundary_replica_candidate_row_count: 0,
                deduped_candidate_row_count: 0,
                deduped_primary_candidate_row_count: 0,
                deduped_boundary_replica_candidate_row_count: 0,
                truncated_candidate_row_count: 0,
                truncated_primary_candidate_row_count: 0,
                truncated_boundary_replica_candidate_row_count: 0,
                candidate_winner_count: 0,
                primary_candidate_winner_count: 0,
                boundary_replica_candidate_winner_count: 0,
                delete_delta_row_count: 0,
                dropped_unselected_delta_route_count: 0,
            })
    }
}

impl SpireRoutedScanObserver for SpireScanPlacementDiagnosticsObserver {
    fn routed_leaf(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        let entry = self.entry(epoch, placement);
        entry.route_count += 1;
        entry.leaf_route_count += 1;
    }

    fn routed_delta(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        let entry = self.entry(epoch, placement);
        entry.route_count += 1;
        entry.delta_route_count += 1;
    }

    fn prefetched_object(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        self.entry(epoch, placement).prefetched_object_count += 1;
    }

    fn scanned_leaf(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        let entry = self.entry(epoch, placement);
        entry.scanned_pid_count += 1;
        entry.leaf_pid_count += 1;
    }

    fn scanned_delta(&mut self, epoch: u64, placement: &SpirePlacementEntry) {
        let entry = self.entry(epoch, placement);
        entry.scanned_pid_count += 1;
        entry.delta_pid_count += 1;
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
    cursor: SpireScanCandidateCursor,
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
            cursor: SpireScanCandidateCursor::default(),
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
        self.rescan_called = true;
        self.query = Some(query);
        self.scan_plan = Some(scan_plan);
        self.cursor.reset(candidates);
    }

    fn clear_scan_work(&mut self) {
        self.rescan_called = false;
        self.query = None;
        self.scan_plan = None;
        self.cursor.reset(Vec::new());
    }

    unsafe fn root_control_for_rescan(
        &mut self,
        index_relation: pg_sys::Relation,
    ) -> SpireRootControlState {
        let observed = unsafe { page::read_root_control_page(index_relation) };
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
