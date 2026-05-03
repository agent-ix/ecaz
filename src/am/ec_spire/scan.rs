use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::ptr;

use super::meta::{
    SpireConsistencyMode, SpireEpochManifest, SpireObjectManifest, SpirePlacementDirectory,
    SpirePlacementState, SpirePublishedEpochSnapshot, SpireRootControlState,
    SpireValidatedEpochSnapshot,
};
use super::options::{
    relation_options, resolve_single_level_scan_plan, EcSpireOptions, SpireCandidateDedupeMode,
    SpireSingleLevelScanPlan,
};
use super::page;
use super::quantizer::{SpireAssignmentPayloadFormat, SpirePreparedAssignmentScorer};
use super::storage::{
    is_delete_delta_assignment, is_visible_primary_assignment, is_visible_primary_assignment_flags,
    SpireLeafAssignmentRow, SpireLeafObjectColumns, SpireLeafPartitionObject, SpireObjectReader,
    SpirePartitionObjectKind, SpireRelationObjectStore, SpireRoutingPartitionObject, SpireVecId,
    SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
};
use crate::storage::page::ItemPointer;
use pgrx::{pg_sys, FromDatum, IntoDatum, PgBox, PgMemoryContexts};

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

#[derive(Debug, Clone, Copy)]
struct SpireRouteCandidate {
    centroid_index: u32,
    child_pid: u64,
    ip_score: f32,
}

#[derive(Debug, Clone, Copy)]
struct SpireRouteCandidateHeapEntry {
    candidate: SpireRouteCandidate,
}

impl PartialEq for SpireRouteCandidateHeapEntry {
    fn eq(&self, other: &Self) -> bool {
        route_candidate_cmp(&self.candidate, &other.candidate) == Ordering::Equal
    }
}

impl Eq for SpireRouteCandidateHeapEntry {}

impl PartialOrd for SpireRouteCandidateHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SpireRouteCandidateHeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        route_candidate_cmp(&self.candidate, &other.candidate)
    }
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
    // Cached for the scan descriptor lifetime. Future snapshot loading can
    // invalidate this if it observes a newer active_epoch during a rescan.
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
        if let Some(root_control) = self.root_control {
            return root_control;
        }
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        self.root_control = Some(root_control);
        root_control
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

pub(super) fn collect_snapshot_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    collect_validated_snapshot_leaf_rows(&snapshot, object_store)
}

fn collect_validated_snapshot_leaf_rows(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "scan leaf row collection")?;
        let placement = lookup.placement;

        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Leaf {
            continue;
        }

        rows.extend(read_leaf_scan_rows(
            object_store,
            placement,
            manifest_entry.pid,
            manifest_entry.object_version,
        )?);
    }
    Ok(rows)
}

pub(super) fn collect_snapshot_routed_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
) -> Result<SpireRoutedLeafScanRows, String> {
    let mut routed =
        collect_snapshot_routed_probe_leaf_rows(snapshot, object_store, query_vector, 1)?;
    routed
        .pop()
        .ok_or_else(|| "ec_spire routed scan found no leaf route".to_owned())
}

pub(super) fn collect_snapshot_routed_probe_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<SpireRoutedLeafScanRows>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let (root_pid, root_object) = load_snapshot_root_routing_object(&snapshot, object_store)?;
    let leaf_pids = route_root_object_to_leaf_pids(&root_object, query_vector, nprobe)?;
    let epoch = snapshot.epoch_manifest().epoch;

    let mut routed = Vec::with_capacity(leaf_pids.len());
    for leaf_pid in leaf_pids {
        let rows = collect_snapshot_leaf_rows_for_pid(&snapshot, object_store, leaf_pid, root_pid)?;
        routed.push(SpireRoutedLeafScanRows {
            epoch,
            root_pid,
            leaf_pid,
            rows,
        });
    }
    Ok(routed)
}

pub(super) fn count_snapshot_single_level_leaf_pids(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<u32, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let (_, root_object) = load_snapshot_root_routing_object(&snapshot, object_store)?;
    u32::try_from(root_object.child_count())
        .map_err(|_| "ec_spire scan root child count exceeds u32".to_owned())
}

pub(super) fn collect_ranked_routed_probe_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe: u32,
    score_ip: F,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireLeafAssignmentRow) -> Result<f32, String>,
{
    let routed_rows =
        collect_snapshot_routed_probe_leaf_rows(snapshot, object_store, query_vector, nprobe)?;
    rank_routed_leaf_rows_by_ip(routed_rows, score_ip, dedupe_mode, limit)
}

pub(super) fn collect_quantized_routed_probe_candidates(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe: u32,
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let scorer =
        SpirePreparedAssignmentScorer::prepare(payload_format, query_vector.len(), query_vector)?;
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let (root_pid, root_object) = load_snapshot_root_routing_object(&snapshot, object_store)?;
    let leaf_pids = route_root_object_to_leaf_pids(&root_object, query_vector, nprobe)?;
    if limit == Some(0) {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    let mut candidates_by_vec_id = match dedupe_mode {
        SpireCandidateDedupeMode::NoReplicaDedupeDisabled => None,
        SpireCandidateDedupeMode::VecIdDedupeEnabled => Some(HashMap::new()),
    };
    for leaf_pid in leaf_pids {
        append_quantized_leaf_candidates_for_pid(
            &snapshot,
            object_store,
            leaf_pid,
            root_pid,
            &scorer,
            &mut candidates,
            &mut candidates_by_vec_id,
        )?;
    }

    if let Some(candidates_by_vec_id) = candidates_by_vec_id {
        candidates.extend(candidates_by_vec_id.into_values());
    }

    Ok(rank_bounded_scored_candidates(candidates, limit))
}

pub(super) fn collect_reranked_quantized_routed_probe_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    nprobe: u32,
    payload_format: SpireAssignmentPayloadFormat,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
    rerank_width: usize,
    exact_score_ip: F,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<f32, String>,
{
    let mut candidates = collect_quantized_routed_probe_candidates(
        snapshot,
        object_store,
        query_vector,
        nprobe,
        payload_format,
        dedupe_mode,
        limit,
    )?;
    rerank_scored_candidates_by_ip(&mut candidates, rerank_width, exact_score_ip)?;
    Ok(candidates)
}

pub(super) fn collect_single_level_scan_plan_reranked_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query_vector: &[f32],
    scan_plan: SpireSingleLevelScanPlan,
    exact_score_ip: F,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<f32, String>,
{
    if scan_plan.nprobe == 0 {
        return Ok(Vec::new());
    }

    collect_reranked_quantized_routed_probe_candidates(
        snapshot,
        object_store,
        query_vector,
        scan_plan.nprobe,
        scan_plan.payload_format,
        scan_plan.dedupe_mode,
        scan_plan.candidate_limit,
        scan_plan.rerank_width,
        exact_score_ip,
    )
}

pub(super) fn prepare_single_level_snapshot_scan_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    query: &SpireScanQuery,
    options: EcSpireOptions,
    exact_score_ip: F,
) -> Result<SpirePreparedScanCandidates, String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<f32, String>,
{
    let leaf_count = count_snapshot_single_level_leaf_pids(snapshot, object_store)?;
    let scan_plan = resolve_single_level_scan_plan(leaf_count, options)?;
    let candidates = collect_single_level_scan_plan_reranked_candidates(
        snapshot,
        object_store,
        query.values(),
        scan_plan,
        exact_score_ip,
    )?;

    Ok(SpirePreparedScanCandidates {
        scan_plan,
        candidates,
    })
}

pub(super) fn rerank_scored_candidates_by_ip<F>(
    candidates: &mut Vec<SpireScoredScanCandidate>,
    rerank_width: usize,
    mut exact_score_ip: F,
) -> Result<(), String>
where
    F: FnMut(&SpireScoredScanCandidate) -> Result<f32, String>,
{
    let rerank_len = if rerank_width == 0 {
        candidates.len()
    } else {
        rerank_width.min(candidates.len())
    };

    for candidate in candidates.iter_mut().take(rerank_len) {
        let ip = exact_score_ip(candidate)?;
        if !ip.is_finite() {
            return Err(
                "ec_spire routed candidate reranker returned a non-finite score".to_owned(),
            );
        }
        candidate.score = -ip;
    }

    candidates[..rerank_len].sort_by(scored_candidate_cmp);
    if rerank_width > 0 {
        candidates.truncate(rerank_len);
    }
    Ok(())
}

pub(super) fn collect_snapshot_delta_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireDeltaScanRow>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    collect_validated_snapshot_delta_rows(&snapshot, object_store)
}

fn collect_validated_snapshot_delta_rows(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireDeltaScanRow>, String> {
    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "scan delta row collection")?;
        let placement = lookup.placement;

        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Delta {
            continue;
        }

        let delta_object = object_store.read_delta_object(placement)?;
        for (row_index, assignment) in delta_object.assignments.into_iter().enumerate() {
            let row_index = u32::try_from(row_index)
                .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
            rows.push(SpireDeltaScanRow {
                pid: manifest_entry.pid,
                object_version: manifest_entry.object_version,
                row_index,
                assignment,
            });
        }
    }
    Ok(rows)
}

pub(super) fn collect_snapshot_visible_primary_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    collect_validated_snapshot_visible_primary_rows(&snapshot, object_store)
}

pub(super) fn collect_validated_snapshot_visible_primary_rows(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let delta_rows = collect_validated_snapshot_delta_rows(snapshot, object_store)?;
    let deleted_vec_ids: HashSet<_> = delta_rows
        .iter()
        .filter(|row| is_delete_delta_assignment(&row.assignment))
        .map(|row| row.assignment.vec_id.clone())
        .collect();

    let mut visible_rows = Vec::new();
    visible_rows.extend(
        collect_validated_snapshot_leaf_rows(snapshot, object_store)?
            .into_iter()
            .filter(|row| {
                is_visible_primary_assignment(&row.assignment)
                    && !deleted_vec_ids.contains(&row.assignment.vec_id)
            }),
    );
    visible_rows.extend(delta_rows.into_iter().filter_map(|row| {
        if is_visible_primary_assignment(&row.assignment)
            && !deleted_vec_ids.contains(&row.assignment.vec_id)
        {
            Some(SpireLeafScanRow {
                pid: row.pid,
                object_version: row.object_version,
                row_index: row.row_index,
                assignment: row.assignment,
            })
        } else {
            None
        }
    }));

    let mut visible_vec_ids = HashSet::new();
    for row in &visible_rows {
        if !visible_vec_ids.insert(row.assignment.vec_id.clone()) {
            return Err(
                "ec_spire visible snapshot contains duplicate primary vec_id assignments"
                    .to_owned(),
            );
        }
    }

    Ok(visible_rows)
}

fn append_quantized_leaf_candidates_for_pid(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    leaf_pid: u64,
    root_pid: u64,
    scorer: &SpirePreparedAssignmentScorer,
    candidates: &mut Vec<SpireScoredScanCandidate>,
    candidates_by_vec_id: &mut Option<HashMap<SpireVecId, SpireScoredScanCandidate>>,
) -> Result<(), String> {
    let lookup = snapshot.require_lookup(leaf_pid, "quantized routed scan leaf")?;
    let manifest_entry = lookup.manifest_entry;
    let placement = lookup.placement;
    if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
        return Ok(());
    }

    let header = object_store.read_object_header(placement)?;
    if header.kind != SpirePartitionObjectKind::Leaf {
        return Err(format!(
            "ec_spire quantized routed scan pid {leaf_pid} is not a leaf object"
        ));
    }
    if header.parent_pid != root_pid {
        return Err(format!(
            "ec_spire quantized routed scan leaf pid {leaf_pid} parent {} does not match root pid {root_pid}",
            header.parent_pid
        ));
    }

    match object_store.read_leaf_object_v2(placement) {
        Ok(leaf_object) => {
            for columns in leaf_object.column_segments()? {
                append_quantized_v2_column_candidates(
                    columns,
                    snapshot.epoch_manifest().epoch,
                    leaf_pid,
                    manifest_entry.object_version,
                    scorer,
                    candidates,
                    candidates_by_vec_id,
                )?;
            }
            Ok(())
        }
        Err(v2_error) => {
            let leaf_object = object_store.read_leaf_object(placement).map_err(|v1_error| {
                format!(
                    "ec_spire quantized scan could not read leaf pid {leaf_pid} as V2 or V1: V2 error: {v2_error}; V1 error: {v1_error}"
                )
            })?;
            append_quantized_v1_leaf_candidates(
                leaf_object,
                snapshot.epoch_manifest().epoch,
                leaf_pid,
                manifest_entry.object_version,
                scorer,
                candidates,
                candidates_by_vec_id,
            )
        }
    }
}

fn append_quantized_v2_column_candidates(
    columns: SpireLeafObjectColumns<'_>,
    epoch: u64,
    pid: u64,
    object_version: u64,
    scorer: &SpirePreparedAssignmentScorer,
    candidates: &mut Vec<SpireScoredScanCandidate>,
    candidates_by_vec_id: &mut Option<HashMap<SpireVecId, SpireScoredScanCandidate>>,
) -> Result<(), String> {
    let column_format = SpireAssignmentPayloadFormat::from_tag(columns.payload_format)?;
    if column_format != scorer.payload_format() {
        return Err(format!(
            "ec_spire leaf V2 payload format {:?} does not match prepared scorer {:?}",
            column_format,
            scorer.payload_format()
        ));
    }

    let mut scores = vec![0.0; columns.row_count()];
    scorer.score_batch_ip(
        columns.payload_stride,
        columns.payloads,
        columns.gammas,
        &mut scores,
    )?;

    for (row_offset, ip) in scores.into_iter().enumerate() {
        if !is_visible_primary_assignment_flags(columns.flags[row_offset]) {
            continue;
        }
        if !ip.is_finite() {
            return Err(
                "ec_spire routed candidate batch scorer returned a non-finite score".to_owned(),
            );
        }

        let row = columns.row(row_offset)?;
        let candidate = SpireScoredScanCandidate {
            epoch,
            pid,
            object_version,
            row_index: row.row_index,
            assignment_flags: row.flags,
            vec_id: SpireVecId::local(row.local_vec_seq()?),
            heap_tid: row.heap_tid,
            score: -ip,
        };
        append_scored_candidate(candidate, candidates, candidates_by_vec_id);
    }
    Ok(())
}

fn append_quantized_v1_leaf_candidates(
    leaf_object: SpireLeafPartitionObject,
    epoch: u64,
    pid: u64,
    object_version: u64,
    scorer: &SpirePreparedAssignmentScorer,
    candidates: &mut Vec<SpireScoredScanCandidate>,
    candidates_by_vec_id: &mut Option<HashMap<SpireVecId, SpireScoredScanCandidate>>,
) -> Result<(), String> {
    for (row_index, assignment) in leaf_object.assignments.into_iter().enumerate() {
        if !is_visible_primary_assignment(&assignment) {
            continue;
        }
        let ip = scorer.score_assignment_ip(&assignment)?;
        if !ip.is_finite() {
            return Err("ec_spire routed candidate scorer returned a non-finite score".to_owned());
        }
        let row_index = u32::try_from(row_index)
            .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
        let candidate = SpireScoredScanCandidate {
            epoch,
            pid,
            object_version,
            row_index,
            assignment_flags: assignment.flags,
            vec_id: assignment.vec_id,
            heap_tid: assignment.heap_tid,
            score: -ip,
        };
        append_scored_candidate(candidate, candidates, candidates_by_vec_id);
    }
    Ok(())
}

fn rank_routed_leaf_rows_by_ip<F>(
    routed_rows: Vec<SpireRoutedLeafScanRows>,
    mut score_ip: F,
    dedupe_mode: SpireCandidateDedupeMode,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireLeafAssignmentRow) -> Result<f32, String>,
{
    if limit == Some(0) {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    let mut candidates_by_vec_id = match dedupe_mode {
        SpireCandidateDedupeMode::NoReplicaDedupeDisabled => None,
        SpireCandidateDedupeMode::VecIdDedupeEnabled => Some(HashMap::new()),
    };
    for routed in routed_rows {
        for row in routed.rows {
            if !is_visible_primary_assignment(&row.assignment) {
                continue;
            }
            let ip = score_ip(&row.assignment)?;
            if !ip.is_finite() {
                return Err(
                    "ec_spire routed candidate scorer returned a non-finite score".to_owned(),
                );
            }
            let candidate = SpireScoredScanCandidate {
                epoch: routed.epoch,
                pid: row.pid,
                object_version: row.object_version,
                row_index: row.row_index,
                assignment_flags: row.assignment.flags,
                vec_id: row.assignment.vec_id.clone(),
                heap_tid: row.assignment.heap_tid,
                score: -ip,
            };
            append_scored_candidate(candidate, &mut candidates, &mut candidates_by_vec_id);
        }
    }

    if let Some(candidates_by_vec_id) = candidates_by_vec_id {
        candidates.extend(candidates_by_vec_id.into_values());
    }

    Ok(rank_bounded_scored_candidates(candidates, limit))
}

fn append_scored_candidate(
    candidate: SpireScoredScanCandidate,
    candidates: &mut Vec<SpireScoredScanCandidate>,
    candidates_by_vec_id: &mut Option<HashMap<SpireVecId, SpireScoredScanCandidate>>,
) {
    if let Some(candidates_by_vec_id) = candidates_by_vec_id.as_mut() {
        match candidates_by_vec_id.entry(candidate.vec_id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                if scored_candidate_cmp(&candidate, entry.get()) == Ordering::Less {
                    *entry.get_mut() = candidate;
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
        }
    } else {
        candidates.push(candidate);
    }
}

fn scored_candidate_cmp(
    left: &SpireScoredScanCandidate,
    right: &SpireScoredScanCandidate,
) -> Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| right.epoch.cmp(&left.epoch))
        .then_with(|| {
            candidate_assignment_role_rank(left).cmp(&candidate_assignment_role_rank(right))
        })
        .then_with(|| left.heap_tid.block_number.cmp(&right.heap_tid.block_number))
        .then_with(|| {
            left.heap_tid
                .offset_number
                .cmp(&right.heap_tid.offset_number)
        })
        .then_with(|| left.pid.cmp(&right.pid))
        .then_with(|| left.row_index.cmp(&right.row_index))
        .then_with(|| left.vec_id.as_bytes().cmp(right.vec_id.as_bytes()))
}

fn candidate_assignment_role_rank(candidate: &SpireScoredScanCandidate) -> u8 {
    u8::from(candidate.assignment_flags & SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0)
}

fn rank_bounded_scored_candidates<I>(
    candidates: I,
    limit: Option<usize>,
) -> Vec<SpireScoredScanCandidate>
where
    I: IntoIterator<Item = SpireScoredScanCandidate>,
{
    let Some(limit) = limit else {
        let mut ranked = candidates.into_iter().collect::<Vec<_>>();
        ranked.sort_by(scored_candidate_cmp);
        return ranked;
    };

    if limit == 0 {
        return Vec::new();
    }

    let mut heap = BinaryHeap::with_capacity(limit);
    for candidate in candidates {
        let entry = SpireScoredScanCandidateHeapEntry { candidate };
        if heap.len() < limit {
            heap.push(entry);
            continue;
        }

        if heap
            .peek()
            .is_some_and(|worst| scored_candidate_cmp(&entry.candidate, &worst.candidate).is_lt())
        {
            heap.pop();
            heap.push(entry);
        }
    }

    let mut ranked = heap
        .into_iter()
        .map(|entry| entry.candidate)
        .collect::<Vec<_>>();
    ranked.sort_by(scored_candidate_cmp);
    ranked
}

fn load_snapshot_root_routing_object(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
) -> Result<(u64, SpireRoutingPartitionObject), String> {
    let mut root = None;
    for manifest_entry in &snapshot.object_manifest().entries {
        let lookup = snapshot.require_lookup(manifest_entry.pid, "scan root routing load")?;
        let placement = lookup.placement;
        if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Root {
            continue;
        }
        if root.is_some() {
            return Err("ec_spire scan snapshot contains multiple root routing objects".to_owned());
        }
        root = Some((
            manifest_entry.pid,
            object_store.read_routing_object(placement)?,
        ));
    }

    root.ok_or_else(|| "ec_spire scan snapshot has no available root routing object".to_owned())
}

fn route_root_object_to_leaf_pids(
    root_object: &SpireRoutingPartitionObject,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<u64>, String> {
    if root_object.header.kind != SpirePartitionObjectKind::Root {
        return Err("ec_spire scan routing requires a root routing object".to_owned());
    }
    if nprobe == 0 {
        return Err("ec_spire routed scan requires nprobe > 0".to_owned());
    }
    validate_routing_query_vector(query_vector, usize::from(root_object.dimensions))?;

    let requested = usize::try_from(nprobe)
        .map_err(|_| "ec_spire routed scan nprobe exceeds usize".to_owned())?;

    let mut heap = BinaryHeap::with_capacity(requested.min(root_object.child_count()));
    for child in root_object.children() {
        let entry = SpireRouteCandidateHeapEntry {
            candidate: SpireRouteCandidate {
                centroid_index: child.centroid_index,
                child_pid: child.child_pid,
                ip_score: inner_product(query_vector, child.centroid),
            },
        };
        if heap.len() < requested {
            heap.push(entry);
            continue;
        }

        if heap
            .peek()
            .is_some_and(|worst| route_candidate_cmp(&entry.candidate, &worst.candidate).is_lt())
        {
            heap.pop();
            heap.push(entry);
        }
    }

    let mut scored_children = heap
        .into_iter()
        .map(|entry| entry.candidate)
        .collect::<Vec<_>>();
    scored_children.sort_by(route_candidate_cmp);

    Ok(scored_children
        .into_iter()
        .map(|candidate| candidate.child_pid)
        .collect())
}

fn route_candidate_cmp(left: &SpireRouteCandidate, right: &SpireRouteCandidate) -> Ordering {
    right
        .ip_score
        .total_cmp(&left.ip_score)
        .then_with(|| left.centroid_index.cmp(&right.centroid_index))
        .then_with(|| left.child_pid.cmp(&right.child_pid))
}

fn validate_routing_query_vector(query_vector: &[f32], dimensions: usize) -> Result<(), String> {
    if query_vector.len() != dimensions {
        return Err(format!(
            "ec_spire vector dimensions mismatch: got {}, expected {dimensions}",
            query_vector.len()
        ));
    }
    if query_vector.iter().any(|value| !value.is_finite()) {
        return Err("ec_spire vector contains a non-finite value".to_owned());
    }
    let norm_sq = query_vector
        .iter()
        .map(|value| (*value as f64) * (*value as f64))
        .sum::<f64>();
    if norm_sq <= f64::EPSILON {
        return Err("ec_spire spherical routing requires non-zero vectors".to_owned());
    }
    Ok(())
}

fn inner_product(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| left * right)
        .sum()
}

fn collect_snapshot_leaf_rows_for_pid(
    snapshot: &SpireValidatedEpochSnapshot<'_>,
    object_store: &impl SpireObjectReader,
    leaf_pid: u64,
    root_pid: u64,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let lookup = snapshot.require_lookup(leaf_pid, "routed scan leaf")?;
    let manifest_entry = lookup.manifest_entry;
    let placement = lookup.placement;
    if should_skip_placement(snapshot.epoch_manifest().consistency_mode, placement.state)? {
        return Ok(Vec::new());
    }

    let header = object_store.read_object_header(placement)?;
    if header.kind != SpirePartitionObjectKind::Leaf {
        return Err(format!(
            "ec_spire routed scan pid {leaf_pid} is not a leaf object"
        ));
    }
    if header.parent_pid != root_pid {
        return Err(format!(
            "ec_spire routed scan leaf pid {leaf_pid} parent {} does not match root pid {root_pid}",
            header.parent_pid
        ));
    }

    read_leaf_scan_rows(
        object_store,
        placement,
        leaf_pid,
        manifest_entry.object_version,
    )
}

fn read_leaf_scan_rows(
    object_store: &impl SpireObjectReader,
    placement: &super::meta::SpirePlacementEntry,
    pid: u64,
    object_version: u64,
) -> Result<Vec<SpireLeafScanRow>, String> {
    match object_store.read_leaf_object(placement) {
        Ok(leaf_object) => {
            let mut rows = Vec::with_capacity(leaf_object.assignments.len());
            for (row_index, assignment) in leaf_object.assignments.into_iter().enumerate() {
                let row_index = u32::try_from(row_index)
                    .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
                rows.push(SpireLeafScanRow {
                    pid,
                    object_version,
                    row_index,
                    assignment,
                });
            }
            Ok(rows)
        }
        Err(v1_error) => {
            let leaf_object = object_store.read_leaf_object_v2(placement).map_err(|v2_error| {
                format!(
                    "ec_spire scan could not read leaf pid {pid} as V1 or V2: V1 error: {v1_error}; V2 error: {v2_error}"
                )
            })?;
            let assignments = leaf_object.assignment_rows()?;
            let mut rows = Vec::with_capacity(assignments.len());
            for (row_index, assignment) in assignments.into_iter().enumerate() {
                let row_index = u32::try_from(row_index)
                    .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
                rows.push(SpireLeafScanRow {
                    pid,
                    object_version,
                    row_index,
                    assignment,
                });
            }
            Ok(rows)
        }
    }
}

fn should_skip_placement(
    consistency_mode: SpireConsistencyMode,
    state: SpirePlacementState,
) -> Result<bool, String> {
    match (consistency_mode, state) {
        (_, SpirePlacementState::Available) => Ok(false),
        (SpireConsistencyMode::Degraded, SpirePlacementState::Unavailable)
        | (SpireConsistencyMode::Degraded, SpirePlacementState::Skipped) => Ok(true),
        (SpireConsistencyMode::Strict, state) => Err(format!(
            "ec_spire strict scan cannot skip {:?} placement",
            state
        )),
        (SpireConsistencyMode::Degraded, SpirePlacementState::Stale) => {
            Err("ec_spire degraded scan cannot use stale placement".to_owned())
        }
    }
}

pub(super) fn set_scan_heap_tid(scan: pg_sys::IndexScanDesc, heap_tid: ItemPointer) {
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut (*scan).xs_heaptid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
    }
}

pub(super) fn set_scan_orderby_score(scan: pg_sys::IndexScanDesc, score: f32) {
    unsafe {
        if (*scan).xs_orderbyvals.is_null() {
            (*scan).xs_orderbyvals =
                pg_sys::palloc0(std::mem::size_of::<pg_sys::Datum>()).cast::<pg_sys::Datum>();
        }
        if (*scan).xs_orderbynulls.is_null() {
            (*scan).xs_orderbynulls = pg_sys::palloc0(std::mem::size_of::<bool>()).cast::<bool>();
        }

        *(*scan).xs_orderbyvals = score.into_datum().expect("score should convert to datum");
        *(*scan).xs_orderbynulls = false;
    }
}

pub(super) fn clear_scan_orderby_output(scan: pg_sys::IndexScanDesc) {
    unsafe {
        if !(*scan).xs_orderbynulls.is_null() {
            *(*scan).xs_orderbynulls = true;
        }
    }
}

unsafe fn load_relation_epoch_manifests(
    index_relation: pg_sys::Relation,
    root_control: SpireRootControlState,
) -> Result<
    (
        SpireEpochManifest,
        SpireObjectManifest,
        SpirePlacementDirectory,
    ),
    String,
> {
    if root_control.active_epoch == 0 {
        return Err("ec_spire cannot load manifests for empty active epoch".to_owned());
    }
    let epoch_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.epoch_manifest_tid)? };
    let object_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.object_manifest_tid)? };
    let placement_bytes =
        unsafe { page::read_object_tuple(index_relation, root_control.placement_directory_tid)? };
    let epoch_manifest = SpireEpochManifest::decode(&epoch_bytes)?;
    let object_manifest = SpireObjectManifest::decode(&object_bytes)?;
    let placement_directory = SpirePlacementDirectory::decode(&placement_bytes)?;
    if epoch_manifest.epoch != root_control.active_epoch {
        return Err(format!(
            "ec_spire root/control active epoch {} does not match epoch manifest {}",
            root_control.active_epoch, epoch_manifest.epoch
        ));
    }
    SpireValidatedEpochSnapshot::new(&epoch_manifest, &object_manifest, &placement_directory)?;
    Ok((epoch_manifest, object_manifest, placement_directory))
}

unsafe fn decode_scan_orderby_query(orderbys: pg_sys::ScanKey) -> Result<SpireScanQuery, String> {
    if orderbys.is_null() {
        return Err("ec_spire amrescan received null order-by scan keys".to_owned());
    }

    let orderby = unsafe { &*orderbys };
    if (orderby.sk_flags as u32) & pg_sys::SK_ISNULL != 0 {
        return Err("ec_spire scan query must not be NULL".to_owned());
    }

    let values =
        Vec::<f32>::from_polymorphic_datum(orderby.sk_argument, false, pg_sys::FLOAT4ARRAYOID)
            .ok_or_else(|| "ec_spire scan requires a real[] ORDER BY query".to_owned())?;
    SpireScanQuery::new(values)
}

pub(super) unsafe extern "C-unwind" fn ec_spire_ambeginscan(
    index_relation: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let scan = pg_sys::RelationGetIndexScan(index_relation, nkeys, norderbys);
            if scan.is_null() {
                pgrx::error!("ec_spire failed to allocate scan descriptor");
            }

            let opaque = PgBox::<SpireScanOpaque>::alloc_in_context(PgMemoryContexts::For(
                pg_sys::CurrentMemoryContext,
            ));
            ptr::write(opaque.as_ptr(), SpireScanOpaque::default());
            (*scan).parallel_scan = ptr::null_mut();
            (*scan).opaque = opaque.into_pg().cast();
            scan
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amrescan(
    scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    orderbys: pg_sys::ScanKey,
    norderbys: std::ffi::c_int,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_spire amrescan received a null scan descriptor");
            }
            if nkeys != 0 {
                pgrx::error!("ec_spire scan does not support index quals yet");
            }
            if norderbys != 1 {
                pgrx::error!("ec_spire scan currently requires exactly one ORDER BY query");
            }

            let opaque_ptr = (*scan).opaque.cast::<SpireScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_spire amrescan missing scan opaque state");
            }
            let opaque = &mut *opaque_ptr;
            opaque.clear_scan_work();
            let query = decode_scan_orderby_query(orderbys).unwrap_or_else(|e| pgrx::error!("{e}"));
            (*scan).xs_recheck = false;
            (*scan).xs_recheckorderby = false;
            (*scan).xs_orderbyvals = ptr::null_mut();
            (*scan).xs_orderbynulls = ptr::null_mut();

            let root_control = opaque.root_control_for_rescan((*scan).indexRelation);
            if root_control.active_epoch == 0 {
                let scan_plan =
                    resolve_single_level_scan_plan(0, relation_options((*scan).indexRelation))
                        .unwrap_or_else(|e| pgrx::error!("{e}"));
                opaque.reset_for_candidates(query, scan_plan, Vec::new());
                return;
            }

            let (epoch_manifest, object_manifest, placement_directory) =
                load_relation_epoch_manifests((*scan).indexRelation, root_control)
                    .unwrap_or_else(|e| pgrx::error!("{e}"));
            let snapshot = SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )
            .unwrap_or_else(|e| pgrx::error!("{e}"));
            let object_store = SpireRelationObjectStore::for_index_relation((*scan).indexRelation)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            let prepared = prepare_single_level_snapshot_scan_candidates(
                &snapshot,
                &object_store,
                &query,
                relation_options((*scan).indexRelation),
                |candidate| Ok(-candidate.score),
            )
            .unwrap_or_else(|e| pgrx::error!("{e}"));
            opaque.reset_for_candidates(query, prepared.scan_plan, prepared.candidates);
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amgettuple(
    scan: pg_sys::IndexScanDesc,
    direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_spire amgettuple received a null scan descriptor");
            }
            if direction != pg_sys::ScanDirection::ForwardScanDirection {
                pgrx::error!("ec_spire amgettuple only supports forward scan direction");
            }
            let opaque_ptr = (*scan).opaque.cast::<SpireScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_spire amgettuple missing scan opaque state");
            }
            let opaque = &mut *opaque_ptr;
            if !opaque.rescan_called {
                pgrx::error!("ec_spire amgettuple requires amrescan before scan execution");
            }

            match opaque.next_output() {
                Some(output) => {
                    set_scan_heap_tid(scan, output.heap_tid);
                    set_scan_orderby_score(scan, output.orderby_score);
                    (*scan).xs_recheck = false;
                    (*scan).xs_recheckorderby = false;
                    true
                }
                None => {
                    clear_scan_orderby_output(scan);
                    false
                }
            }
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amendscan(scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                return;
            }

            let opaque_ptr = (*scan).opaque.cast::<SpireScanOpaque>();
            if !opaque_ptr.is_null() {
                ptr::drop_in_place(opaque_ptr);
                pg_sys::pfree(opaque_ptr.cast());
                (*scan).opaque = ptr::null_mut();
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_quantized_routed_probe_candidates, collect_ranked_routed_probe_candidates,
        collect_reranked_quantized_routed_probe_candidates,
        collect_single_level_scan_plan_reranked_candidates, collect_snapshot_delta_rows,
        collect_snapshot_leaf_rows, collect_snapshot_routed_leaf_rows,
        collect_snapshot_routed_probe_leaf_rows, collect_snapshot_visible_primary_rows,
        count_snapshot_single_level_leaf_pids, prepare_single_level_snapshot_scan_candidates,
        rank_routed_leaf_rows_by_ip, rerank_scored_candidates_by_ip,
        route_root_object_to_leaf_pids, SpireLeafScanRow, SpireRoutedLeafScanRows,
        SpireScanCandidateCursor, SpireScanOpaque, SpireScanOutput, SpireScanQuery,
        SpireScoredScanCandidate,
    };
    use crate::am::ec_spire::assign::{
        SpireDeleteDeltaInput, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
        SpirePidAllocator, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::build::{
        build_partitioned_single_level_leaf_epoch_draft, build_single_level_leaf_epoch_draft,
        SpirePartitionedSingleLevelBuildInput, SpireSingleLevelBuildInput,
        SpireSingleLevelCentroidPlan,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
        SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState,
        SpirePublishedEpochSnapshot, SpireRootControlState,
    };
    use crate::am::ec_spire::options::{
        EcSpireOptions, SpireCandidateDedupeMode, SpireSingleLevelScanPlan, SpireStorageFormat,
    };
    use crate::am::ec_spire::quantizer::{
        encode_assignment_input, SpireAssignmentPayloadFormat, SpirePreparedAssignmentScorer,
    };
    use crate::am::ec_spire::storage::SpireLocalObjectStore;
    use crate::am::ec_spire::storage::{
        SpireDeltaPartitionObject, SpireLeafAssignmentRow, SpireLeafPartitionObject,
        SpireRoutingChildEntry, SpireRoutingPartitionObject, SpireVecId,
        SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA, SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
        SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
    };
    use crate::am::ec_spire::update::{
        build_delta_epoch_draft_from_snapshot, SpireDeltaEpochInput,
    };
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn assignment_input(block_number: u32, offset_number: u16) -> SpireLeafAssignmentInput {
        assignment_input_with_payload(block_number, offset_number, vec![1, 2, 3])
    }

    fn quantized_assignment_input(
        block_number: u32,
        offset_number: u16,
        payload_format: SpireAssignmentPayloadFormat,
        source_vector: &[f32],
    ) -> SpireLeafAssignmentInput {
        encode_assignment_input(
            payload_format,
            tid(block_number, offset_number),
            source_vector,
        )
        .unwrap()
    }

    fn assignment_input_with_payload(
        block_number: u32,
        offset_number: u16,
        encoded_payload: Vec<u8>,
    ) -> SpireLeafAssignmentInput {
        SpireLeafAssignmentInput {
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload,
        }
    }

    fn build_input(assignments: Vec<SpireLeafAssignmentInput>) -> SpireSingleLevelBuildInput {
        SpireSingleLevelBuildInput {
            epoch: 7,
            object_version: 1,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            placement_tid: tid(60, 1),
            assignments,
        }
    }

    fn partitioned_build_input(
        assignments: Vec<SpireLeafAssignmentInput>,
        assignment_indexes: Vec<u32>,
    ) -> SpirePartitionedSingleLevelBuildInput {
        SpirePartitionedSingleLevelBuildInput {
            epoch: 7,
            object_version: 1,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            root_placement_tid: tid(60, 3),
            placement_tids: vec![tid(60, 1), tid(60, 2)],
            assignments,
            centroid_plan: SpireSingleLevelCentroidPlan {
                dimensions: 2,
                centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
                assignment_indexes,
            },
        }
    }

    fn delta_input(
        insert_assignments: Vec<SpireLeafAssignmentInput>,
        delete_assignments: Vec<SpireDeleteDeltaInput>,
    ) -> SpireDeltaEpochInput {
        SpireDeltaEpochInput {
            epoch: 8,
            object_version: 3,
            published_at_micros: 2000,
            retain_until_micros: 3000,
            consistency_mode: SpireConsistencyMode::Strict,
            base_pid: SPIRE_FIRST_PID,
            placement_tid: tid(80, 1),
            insert_assignments,
            delete_assignments,
        }
    }

    fn delete_delta_input(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireDeleteDeltaInput {
        SpireDeleteDeltaInput {
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
        }
    }

    fn assignment_row(flags: u16, offset_number: u16) -> SpireLeafAssignmentRow {
        assignment_row_with_payload(
            flags,
            u64::from(offset_number),
            10,
            offset_number,
            vec![1, 2, 3],
        )
    }

    fn assignment_row_with_payload(
        flags: u16,
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
        encoded_payload: Vec<u8>,
    ) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload,
        }
    }

    fn delete_assignment_row(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
    ) -> SpireLeafAssignmentRow {
        SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            payload_format: 0,
            gamma: 0.0,
            encoded_payload: Vec::new(),
        }
    }

    fn scored_candidate(
        vec_seq: u64,
        block_number: u32,
        offset_number: u16,
        score: f32,
    ) -> SpireScoredScanCandidate {
        SpireScoredScanCandidate {
            epoch: 1,
            pid: SPIRE_FIRST_PID + vec_seq,
            object_version: 1,
            row_index: u32::from(offset_number),
            assignment_flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            score,
        }
    }

    fn routing_child(
        centroid_index: u32,
        child_pid: u64,
        centroid: Vec<f32>,
    ) -> SpireRoutingChildEntry {
        SpireRoutingChildEntry {
            centroid_index,
            child_pid,
            centroid,
        }
    }

    fn snapshot_for_placement<'a>(
        epoch_manifest: &'a SpireEpochManifest,
        object_manifest: &'a SpireObjectManifest,
        placement_directory: &'a SpirePlacementDirectory,
    ) -> SpirePublishedEpochSnapshot<'a> {
        SpirePublishedEpochSnapshot::new(epoch_manifest, object_manifest, placement_directory)
            .unwrap()
    }

    #[test]
    fn collect_snapshot_leaf_rows_returns_available_leaf_assignments() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_single_level_leaf_epoch_draft(
            build_input(vec![assignment_input(10, 1), assignment_input(10, 2)]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        let rows = collect_snapshot_leaf_rows(&snapshot, &object_store).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pid, SPIRE_FIRST_PID);
        assert_eq!(rows[0].object_version, 1);
        assert_eq!(rows[0].row_index, 0);
        assert_eq!(rows[0].assignment.heap_tid, tid(10, 1));
        assert_eq!(rows[1].row_index, 1);
        assert_eq!(rows[1].assignment.heap_tid, tid(10, 2));
    }

    #[test]
    fn collect_snapshot_leaf_rows_skips_degraded_unavailable_or_skipped_placements() {
        for state in [
            SpirePlacementState::Unavailable,
            SpirePlacementState::Skipped,
        ] {
            let epoch_manifest = SpireEpochManifest {
                epoch: 7,
                state: SpireEpochState::Published,
                consistency_mode: SpireConsistencyMode::Degraded,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                active_query_count: 0,
            };
            let object_manifest = SpireObjectManifest::from_entries(
                7,
                vec![SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 1,
                    placement_tid: tid(60, 1),
                }],
            )
            .unwrap();
            let mut placement =
                SpirePlacementEntry::local_single_store(7, 11, 12345, 1, tid(44, 2), 4096);
            placement.state = state;
            let placement_directory =
                SpirePlacementDirectory::from_entries(7, vec![placement]).unwrap();
            let snapshot = SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )
            .unwrap();
            let object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();

            assert!(collect_snapshot_leaf_rows(&snapshot, &object_store)
                .unwrap()
                .is_empty());
        }
    }

    #[test]
    fn collect_snapshot_visible_primary_rows_filters_non_output_assignments() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let object = SpireLeafPartitionObject::new(
            11,
            1,
            0,
            vec![
                assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 1),
                assignment_row(
                    SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
                    2,
                ),
                assignment_row(
                    SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
                    3,
                ),
                assignment_row(
                    SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR,
                    4,
                ),
            ],
        )
        .unwrap();
        let leaf_placement = object_store.insert_leaf_object(7, &object).unwrap();
        let delta_object =
            SpireDeltaPartitionObject::new(12, 1, 11, vec![delete_assignment_row(6, 10, 6)])
                .unwrap();
        let delta_placement = object_store.insert_delta_object(7, &delta_object).unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 1,
                    placement_tid: tid(60, 1),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 12,
                    object_version: 1,
                    placement_tid: tid(60, 2),
                },
            ],
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(7, vec![leaf_placement, delta_placement])
                .unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);

        let rows = collect_snapshot_visible_primary_rows(&snapshot, &object_store).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pid, 11);
        assert_eq!(rows[0].row_index, 0);
        assert_eq!(rows[0].assignment.heap_tid, tid(10, 1));
    }

    #[test]
    fn collect_snapshot_visible_primary_rows_rejects_duplicate_primary_vec_ids() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let first = SpireLeafPartitionObject::new(
            11,
            1,
            0,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(1),
                heap_tid: tid(10, 1),
                payload_format: 1,
                gamma: 0.5,
                encoded_payload: vec![1, 2, 3],
            }],
        )
        .unwrap();
        let second = SpireLeafPartitionObject::new(
            12,
            1,
            0,
            vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(1),
                heap_tid: tid(20, 1),
                payload_format: 1,
                gamma: 0.75,
                encoded_payload: vec![4, 5, 6],
            }],
        )
        .unwrap();
        let first_placement = object_store.insert_leaf_object(7, &first).unwrap();
        let second_placement = object_store.insert_leaf_object(7, &second).unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            vec![
                SpireManifestEntry {
                    epoch: 7,
                    pid: 11,
                    object_version: 1,
                    placement_tid: tid(60, 1),
                },
                SpireManifestEntry {
                    epoch: 7,
                    pid: 12,
                    object_version: 1,
                    placement_tid: tid(60, 2),
                },
            ],
        )
        .unwrap();
        let placement_directory =
            SpirePlacementDirectory::from_entries(7, vec![first_placement, second_placement])
                .unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);

        assert!(collect_snapshot_visible_primary_rows(&snapshot, &object_store).is_err());
    }

    #[test]
    fn collect_snapshot_rows_dispatches_leaf_and_delta_objects() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_single_level_leaf_epoch_draft(
            build_input(vec![assignment_input(10, 1)]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let base_snapshot = SpirePublishedEpochSnapshot::new(
            &base_draft.epoch_manifest,
            &base_draft.object_manifest,
            &base_draft.placement_directory,
        )
        .unwrap();
        let delta_draft = build_delta_epoch_draft_from_snapshot(
            delta_input(
                vec![assignment_input(20, 1)],
                vec![delete_delta_input(1, 10, 1)],
            ),
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &delta_draft.epoch_manifest,
            &delta_draft.object_manifest,
            &delta_draft.placement_directory,
        )
        .unwrap();

        let leaf_rows = collect_snapshot_leaf_rows(&snapshot, &object_store).unwrap();
        let delta_rows = collect_snapshot_delta_rows(&snapshot, &object_store).unwrap();
        let visible_rows = collect_snapshot_visible_primary_rows(&snapshot, &object_store).unwrap();

        assert_eq!(leaf_rows.len(), 1);
        assert_eq!(leaf_rows[0].pid, SPIRE_FIRST_PID);
        assert_eq!(leaf_rows[0].assignment.heap_tid, tid(10, 1));
        assert_eq!(delta_rows.len(), 2);
        assert_eq!(delta_rows[0].pid, SPIRE_FIRST_PID + 1);
        assert_eq!(
            delta_rows[0].assignment.flags,
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT
        );
        assert_eq!(
            delta_rows[1].assignment.flags,
            SPIRE_ASSIGNMENT_FLAG_TOMBSTONE | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
        );
        assert_eq!(visible_rows.len(), 1);
        assert_eq!(visible_rows[0].pid, SPIRE_FIRST_PID + 1);
        assert_eq!(visible_rows[0].assignment.heap_tid, tid(20, 1));
        assert_eq!(visible_rows[0].assignment.vec_id.local_sequence(), Some(2));
    }

    #[test]
    fn collect_snapshot_routed_leaf_rows_routes_query_to_leaf_pid() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        let positive_rows =
            collect_snapshot_routed_leaf_rows(&snapshot, &object_store, &[1.0, 0.0]).unwrap();
        let negative_rows =
            collect_snapshot_routed_leaf_rows(&snapshot, &object_store, &[-1.0, 0.0]).unwrap();

        assert_eq!(positive_rows.root_pid, SPIRE_FIRST_PID);
        assert_eq!(positive_rows.leaf_pid, SPIRE_FIRST_PID + 1);
        assert_eq!(positive_rows.rows.len(), 1);
        assert_eq!(positive_rows.rows[0].assignment.heap_tid, tid(10, 1));
        assert_eq!(negative_rows.root_pid, SPIRE_FIRST_PID);
        assert_eq!(negative_rows.leaf_pid, SPIRE_FIRST_PID + 2);
        assert_eq!(negative_rows.rows.len(), 1);
        assert_eq!(negative_rows.rows[0].assignment.heap_tid, tid(10, 2));
    }

    #[test]
    fn collect_snapshot_routed_probe_leaf_rows_routes_top_nprobe_leaf_pids() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        let routed =
            collect_snapshot_routed_probe_leaf_rows(&snapshot, &object_store, &[1.0, 0.0], 2)
                .unwrap();

        assert_eq!(routed.len(), 2);
        assert_eq!(routed[0].leaf_pid, SPIRE_FIRST_PID + 1);
        assert_eq!(routed[0].rows[0].assignment.heap_tid, tid(10, 1));
        assert_eq!(routed[1].leaf_pid, SPIRE_FIRST_PID + 2);
        assert_eq!(routed[1].rows[0].assignment.heap_tid, tid(10, 2));
    }

    #[test]
    fn count_snapshot_single_level_leaf_pids_uses_root_routing_children() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            SpirePartitionedSingleLevelBuildInput {
                epoch: 7,
                object_version: 1,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                root_placement_tid: tid(60, 3),
                placement_tids: vec![tid(60, 1), tid(60, 2), tid(60, 4)],
                assignments: vec![assignment_input(10, 1), assignment_input(10, 2)],
                centroid_plan: SpireSingleLevelCentroidPlan {
                    dimensions: 2,
                    centroids: vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![-1.0, 0.0]],
                    assignment_indexes: vec![0, 2],
                },
            },
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        assert_eq!(
            count_snapshot_single_level_leaf_pids(&snapshot, &object_store).unwrap(),
            3
        );
    }

    #[test]
    fn collect_ranked_routed_probe_candidates_scores_and_limits() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![
                    assignment_input_with_payload(10, 1, vec![1]),
                    assignment_input_with_payload(10, 2, vec![9]),
                ],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        let candidates = collect_ranked_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(1),
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].pid, SPIRE_FIRST_PID + 2);
        assert_eq!(candidates[0].object_version, 1);
        assert_eq!(candidates[0].row_index, 0);
        assert_eq!(candidates[0].heap_tid, tid(10, 2));
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -9.0);
    }

    #[test]
    fn collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer() {
        for payload_format in [
            SpireAssignmentPayloadFormat::TurboQuant,
            SpireAssignmentPayloadFormat::RaBitQ,
        ] {
            let mut pid_allocator = SpirePidAllocator::default();
            let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
            let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
            let query = [1.0, 0.0];
            let draft = build_partitioned_single_level_leaf_epoch_draft(
                partitioned_build_input(
                    vec![
                        quantized_assignment_input(10, 1, payload_format, &[1.0, 0.0]),
                        quantized_assignment_input(10, 2, payload_format, &[-1.0, 0.0]),
                    ],
                    vec![0, 1],
                ),
                &mut pid_allocator,
                &mut local_vec_id_allocator,
                &mut object_store,
            )
            .unwrap();
            let snapshot = SpirePublishedEpochSnapshot::new(
                &draft.epoch_manifest,
                &draft.object_manifest,
                &draft.placement_directory,
            )
            .unwrap();
            let scorer =
                SpirePreparedAssignmentScorer::prepare(payload_format, query.len(), &query)
                    .unwrap();
            let expected = collect_ranked_routed_probe_candidates(
                &snapshot,
                &object_store,
                &query,
                2,
                |row| scorer.score_assignment_ip(row),
                SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
                Some(2),
            )
            .unwrap();

            let observed = collect_quantized_routed_probe_candidates(
                &snapshot,
                &object_store,
                &query,
                2,
                payload_format,
                SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
                Some(2),
            )
            .unwrap();

            assert_eq!(observed, expected);
            assert_eq!(observed.len(), 2);
        }
    }

    #[test]
    fn collect_quantized_routed_probe_candidates_rejects_deferred_and_bad_payloads() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![
                    assignment_input_with_payload(10, 1, vec![1]),
                    assignment_input_with_payload(10, 2, vec![2]),
                ],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        assert!(collect_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            SpireAssignmentPayloadFormat::PqFastScan,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
        )
        .unwrap_err()
        .contains("PQ-FastScan"));
        assert!(collect_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            SpireAssignmentPayloadFormat::TurboQuant,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
        )
        .unwrap_err()
        .contains("payload stride mismatch"));
    }

    #[test]
    fn collect_reranked_quantized_routed_probe_candidates_rescores_prefix() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![
                    quantized_assignment_input(
                        10,
                        1,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[1.0, 0.0],
                    ),
                    quantized_assignment_input(
                        10,
                        2,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[-1.0, 0.0],
                    ),
                ],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        let candidates = collect_reranked_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            SpireAssignmentPayloadFormat::TurboQuant,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
            2,
            |candidate| {
                Ok(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 1.0,
                    2 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                })
            },
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -10.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(1));
        assert_eq!(candidates[1].score, -1.0);
    }

    #[test]
    fn collect_single_level_scan_plan_reranked_candidates_uses_plan_knobs() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![
                    quantized_assignment_input(
                        10,
                        1,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[1.0, 0.0],
                    ),
                    quantized_assignment_input(
                        10,
                        2,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[-1.0, 0.0],
                    ),
                ],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 2,
            nprobe: 2,
            nprobe_source: "relation",
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 2,
            rerank_width_source: "relation",
            candidate_limit: Some(2),
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };

        let candidates = collect_single_level_scan_plan_reranked_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            scan_plan,
            |candidate| {
                Ok(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 1.0,
                    2 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                })
            },
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -10.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(1));
        assert_eq!(candidates[1].score, -1.0);
    }

    #[test]
    fn prepare_single_level_snapshot_scan_candidates_resolves_plan_and_candidates() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![
                    quantized_assignment_input(
                        10,
                        1,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[1.0, 0.0],
                    ),
                    quantized_assignment_input(
                        10,
                        2,
                        SpireAssignmentPayloadFormat::TurboQuant,
                        &[-1.0, 0.0],
                    ),
                ],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();
        let options = EcSpireOptions {
            nlists: 2,
            nprobe: 2,
            rerank_width: 2,
            training_sample_rows: 0,
            seed: 0,
            pq_group_size: 0,
            storage_format: SpireStorageFormat::TurboQuant,
        };
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();

        let prepared = prepare_single_level_snapshot_scan_candidates(
            &snapshot,
            &object_store,
            &query,
            options,
            |candidate| {
                Ok(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 1.0,
                    2 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                })
            },
        )
        .unwrap();

        assert_eq!(prepared.scan_plan.leaf_count, 2);
        assert_eq!(prepared.scan_plan.nprobe, 2);
        assert_eq!(prepared.scan_plan.nprobe_source, "relation");
        assert_eq!(prepared.candidates.len(), 2);
        assert_eq!(prepared.candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(prepared.candidates[0].score, -10.0);
        assert_eq!(prepared.candidates[1].vec_id.local_sequence(), Some(1));
        assert_eq!(prepared.candidates[1].score, -1.0);
    }

    #[test]
    fn collect_single_level_scan_plan_reranked_candidates_allows_empty_plan() {
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let object_manifest = SpireObjectManifest::from_entries(7, Vec::new()).unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, Vec::new()).unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);
        let object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 0,
            nprobe: 0,
            nprobe_source: "none",
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 0,
            rerank_width_source: "relation",
            candidate_limit: None,
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };

        let candidates = collect_single_level_scan_plan_reranked_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            scan_plan,
            |_| panic!("empty scan plan should not call exact scorer"),
        )
        .unwrap();

        assert!(candidates.is_empty());
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_keeps_best_visible_vec_id_candidate() {
        let same_vec_id_low_score =
            assignment_row_with_payload(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 7, 20, 2, vec![1]);
        let same_vec_id_high_score =
            assignment_row_with_payload(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 7, 10, 1, vec![9]);
        let boundary_replica = assignment_row_with_payload(
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
            8,
            30,
            3,
            vec![100],
        );
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 0,
                    assignment: same_vec_id_low_score,
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 2,
                    object_version: 1,
                    row_index: 0,
                    assignment: same_vec_id_high_score,
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 3,
                    object_version: 1,
                    row_index: 0,
                    assignment: boundary_replica,
                },
            ],
        }];

        let candidates = rank_routed_leaf_rows_by_ip(
            routed,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::VecIdDedupeEnabled,
            None,
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(7));
        assert_eq!(candidates[0].pid, SPIRE_FIRST_PID + 2);
        assert_eq!(candidates[0].heap_tid, tid(10, 1));
        assert_eq!(candidates[0].score, -9.0);
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_can_skip_vec_id_dedupe() {
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 0,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        7,
                        20,
                        2,
                        vec![1],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 2,
                    object_version: 1,
                    row_index: 0,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        7,
                        10,
                        1,
                        vec![9],
                    ),
                },
            ],
        }];

        let candidates = rank_routed_leaf_rows_by_ip(
            routed,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            None,
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(7));
        assert_eq!(candidates[0].score, -9.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(7));
        assert_eq!(candidates[1].score, -1.0);
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_keeps_bounded_best_candidates() {
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 0,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        1,
                        10,
                        1,
                        vec![3],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 1,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        2,
                        10,
                        2,
                        vec![10],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 2,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        3,
                        10,
                        3,
                        vec![5],
                    ),
                },
                SpireLeafScanRow {
                    pid: SPIRE_FIRST_PID + 1,
                    object_version: 1,
                    row_index: 3,
                    assignment: assignment_row_with_payload(
                        SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                        4,
                        10,
                        4,
                        vec![7],
                    ),
                },
            ],
        }];

        let candidates = rank_routed_leaf_rows_by_ip(
            routed,
            |row| Ok(f32::from(row.encoded_payload[0])),
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -10.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(4));
        assert_eq!(candidates[1].score, -7.0);
    }

    #[test]
    fn scored_candidate_tie_break_prefers_newer_epoch_then_primary_role() {
        let older_primary = scored_candidate(1, 10, 1, 1.0);
        let mut newer_replica = scored_candidate(2, 10, 2, 1.0);
        newer_replica.epoch = 2;
        newer_replica.assignment_flags =
            SPIRE_ASSIGNMENT_FLAG_PRIMARY | SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA;
        let mut newer_primary = scored_candidate(3, 10, 3, 1.0);
        newer_primary.epoch = 2;

        let ranked = super::rank_bounded_scored_candidates(
            vec![older_primary, newer_replica, newer_primary],
            None,
        );

        assert_eq!(ranked[0].vec_id.local_sequence(), Some(3));
        assert_eq!(ranked[1].vec_id.local_sequence(), Some(2));
        assert_eq!(ranked[2].vec_id.local_sequence(), Some(1));
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_rejects_non_finite_scores() {
        let routed = vec![SpireRoutedLeafScanRows {
            epoch: 1,
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![SpireLeafScanRow {
                pid: SPIRE_FIRST_PID + 1,
                object_version: 1,
                row_index: 0,
                assignment: assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 1),
            }],
        }];

        assert!(rank_routed_leaf_rows_by_ip(
            routed,
            |_| Ok(f32::NAN),
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            None
        )
        .unwrap_err()
        .contains("non-finite"));
    }

    #[test]
    fn route_root_object_to_leaf_pids_keeps_bounded_best_routes() {
        let root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 9, vec![-2.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 1, vec![1.0, 1.0]),
                routing_child(2, SPIRE_FIRST_PID + 2, vec![1.0, 0.0]),
                routing_child(3, SPIRE_FIRST_PID + 4, vec![2.0, 0.0]),
                routing_child(4, SPIRE_FIRST_PID + 7, vec![0.25, 0.0]),
            ],
        )
        .unwrap();

        assert_eq!(
            route_root_object_to_leaf_pids(&root, &[1.0, 0.0], 3).unwrap(),
            vec![
                SPIRE_FIRST_PID + 4,
                SPIRE_FIRST_PID + 1,
                SPIRE_FIRST_PID + 2
            ]
        );
    }

    #[test]
    fn rerank_scored_candidates_by_ip_rescores_prefix_and_truncates() {
        let mut candidates = vec![
            scored_candidate(1, 10, 1, -5.0),
            scored_candidate(2, 10, 2, -4.0),
            scored_candidate(3, 10, 3, -3.0),
        ];

        rerank_scored_candidates_by_ip(&mut candidates, 2, |candidate| {
            Ok(match candidate.vec_id.local_sequence().unwrap() {
                1 => 1.0,
                2 => 10.0,
                other => panic!("unexpected rerank candidate {other}"),
            })
        })
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[0].score, -10.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(1));
        assert_eq!(candidates[1].score, -1.0);
    }

    #[test]
    fn rerank_scored_candidates_by_ip_zero_width_rescores_all() {
        let mut candidates = vec![
            scored_candidate(1, 10, 1, -5.0),
            scored_candidate(2, 10, 2, -4.0),
            scored_candidate(3, 10, 3, -3.0),
        ];

        rerank_scored_candidates_by_ip(&mut candidates, 0, |candidate| {
            Ok(candidate.heap_tid.offset_number as f32)
        })
        .unwrap();

        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].heap_tid, tid(10, 3));
        assert_eq!(candidates[0].score, -3.0);
        assert_eq!(candidates[1].heap_tid, tid(10, 2));
        assert_eq!(candidates[1].score, -2.0);
        assert_eq!(candidates[2].heap_tid, tid(10, 1));
        assert_eq!(candidates[2].score, -1.0);
    }

    #[test]
    fn rerank_scored_candidates_by_ip_rejects_non_finite_scores() {
        let mut candidates = vec![scored_candidate(1, 10, 1, -5.0)];

        assert!(
            rerank_scored_candidates_by_ip(&mut candidates, 0, |_| Ok(f32::INFINITY))
                .unwrap_err()
                .contains("non-finite")
        );
    }

    #[test]
    fn scan_candidate_cursor_emits_ranked_candidates_once() {
        let mut cursor = SpireScanCandidateCursor::new(vec![
            scored_candidate(2, 10, 2, -10.0),
            scored_candidate(1, 10, 1, -1.0),
        ]);

        assert_eq!(cursor.remaining(), 2);
        assert!(!cursor.is_exhausted());
        let first = cursor.next_candidate().unwrap();
        assert_eq!(first.vec_id.local_sequence(), Some(2));
        assert_eq!(first.heap_tid, tid(10, 2));
        assert_eq!(first.score, -10.0);

        assert_eq!(cursor.remaining(), 1);
        let second = cursor.next_candidate().unwrap();
        assert_eq!(second.vec_id.local_sequence(), Some(1));
        assert_eq!(second.heap_tid, tid(10, 1));
        assert_eq!(second.score, -1.0);

        assert_eq!(cursor.remaining(), 0);
        assert!(cursor.is_exhausted());
        assert!(cursor.next_candidate().is_none());
        assert!(cursor.next_candidate().is_none());
    }

    #[test]
    fn scan_candidate_cursor_reset_replaces_candidate_set() {
        let mut cursor = SpireScanCandidateCursor::new(vec![
            scored_candidate(2, 10, 2, -10.0),
            scored_candidate(1, 10, 1, -1.0),
        ]);
        assert_eq!(
            cursor.next_candidate().unwrap().vec_id.local_sequence(),
            Some(2)
        );

        cursor.reset(vec![scored_candidate(3, 20, 3, -3.0)]);

        assert_eq!(cursor.remaining(), 1);
        let candidate = cursor.next_candidate().unwrap();
        assert_eq!(candidate.vec_id.local_sequence(), Some(3));
        assert_eq!(candidate.heap_tid, tid(20, 3));
        assert!(cursor.is_exhausted());
    }

    #[test]
    fn scan_candidate_cursor_next_output_returns_amgettuple_shape() {
        let mut cursor = SpireScanCandidateCursor::new(vec![scored_candidate(7, 40, 3, -7.5)]);

        assert_eq!(
            cursor.next_output(),
            Some(SpireScanOutput {
                heap_tid: tid(40, 3),
                orderby_score: -7.5,
            })
        );
        assert!(cursor.next_output().is_none());
    }

    #[test]
    fn scan_query_accepts_nonzero_finite_vectors() {
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();

        assert_eq!(query.dimensions, 2);
        assert_eq!(query.values(), &[1.0, 0.0]);
    }

    #[test]
    fn scan_query_rejects_empty_zero_and_non_finite_vectors() {
        assert!(SpireScanQuery::new(Vec::new())
            .unwrap_err()
            .contains("must not be empty"));
        assert!(SpireScanQuery::new(vec![0.0, 0.0])
            .unwrap_err()
            .contains("non-zero"));
        assert!(SpireScanQuery::new(vec![1.0, f32::NAN])
            .unwrap_err()
            .contains("non-finite"));
    }

    #[test]
    fn scan_opaque_reset_stores_query_plan_and_candidate_cursor() {
        let mut opaque = SpireScanOpaque::default();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 1,
            nprobe: 1,
            nprobe_source: "relation",
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 1,
            rerank_width_source: "relation",
            candidate_limit: Some(1),
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };

        opaque.reset_for_candidates(
            SpireScanQuery::new(vec![1.0, 0.0]).unwrap(),
            scan_plan,
            vec![scored_candidate(9, 50, 4, -9.0)],
        );

        assert!(opaque.rescan_called);
        assert_eq!(opaque.query.as_ref().unwrap().values(), &[1.0, 0.0]);
        assert_eq!(opaque.scan_plan, Some(scan_plan));
        assert_eq!(
            opaque.next_output(),
            Some(SpireScanOutput {
                heap_tid: tid(50, 4),
                orderby_score: -9.0,
            })
        );
        assert!(opaque.next_output().is_none());
    }

    #[test]
    fn scan_opaque_clear_scan_work_drops_rescan_state() {
        let mut opaque = SpireScanOpaque::default();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 1,
            nprobe: 1,
            nprobe_source: "relation",
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 1,
            rerank_width_source: "relation",
            candidate_limit: Some(1),
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };
        opaque.reset_for_candidates(
            SpireScanQuery::new(vec![1.0, 0.0]).unwrap(),
            scan_plan,
            vec![scored_candidate(9, 50, 4, -9.0)],
        );
        opaque.root_control = Some(SpireRootControlState::empty());

        opaque.clear_scan_work();

        assert!(!opaque.rescan_called);
        assert_eq!(opaque.query, None);
        assert_eq!(opaque.scan_plan, None);
        assert_eq!(opaque.root_control, Some(SpireRootControlState::empty()));
        assert!(opaque.next_output().is_none());
    }

    #[test]
    fn collect_snapshot_routed_probe_leaf_rows_rejects_invalid_nprobe_and_query() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        assert!(
            collect_snapshot_routed_probe_leaf_rows(&snapshot, &object_store, &[1.0, 0.0], 0)
                .unwrap_err()
                .contains("nprobe > 0")
        );
        assert!(
            collect_snapshot_routed_probe_leaf_rows(&snapshot, &object_store, &[1.0], 1)
                .unwrap_err()
                .contains("dimensions mismatch")
        );
        assert!(
            collect_snapshot_routed_probe_leaf_rows(&snapshot, &object_store, &[0.0, 0.0], 1)
                .unwrap_err()
                .contains("non-zero")
        );
    }

    #[test]
    fn collect_snapshot_routed_leaf_rows_rejects_missing_root() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_single_level_leaf_epoch_draft(
            build_input(vec![assignment_input(10, 1)]),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &draft.epoch_manifest,
            &draft.object_manifest,
            &draft.placement_directory,
        )
        .unwrap();

        assert!(
            collect_snapshot_routed_leaf_rows(&snapshot, &object_store, &[1.0, 0.0])
                .unwrap_err()
                .contains("no available root")
        );
    }

    #[test]
    fn collect_snapshot_routed_leaf_rows_skips_degraded_unavailable_leaf() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(
                vec![assignment_input(10, 1), assignment_input(10, 2)],
                vec![0, 1],
            ),
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: draft.epoch_manifest.epoch,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Degraded,
            published_at_micros: draft.epoch_manifest.published_at_micros,
            retain_until_micros: draft.epoch_manifest.retain_until_micros,
            active_query_count: 0,
        };
        let mut placements = draft.placement_directory.entries.clone();
        placements
            .iter_mut()
            .find(|placement| placement.pid == SPIRE_FIRST_PID + 1)
            .unwrap()
            .state = SpirePlacementState::Unavailable;
        let placement_directory =
            SpirePlacementDirectory::from_entries(draft.epoch_manifest.epoch, placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &draft.object_manifest,
            &placement_directory,
        )
        .unwrap();

        let routed =
            collect_snapshot_routed_leaf_rows(&snapshot, &object_store, &[1.0, 0.0]).unwrap();

        assert_eq!(routed.root_pid, SPIRE_FIRST_PID);
        assert_eq!(routed.leaf_pid, SPIRE_FIRST_PID + 1);
        assert!(routed.rows.is_empty());
    }
}
