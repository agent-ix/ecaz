use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use super::meta::{SpireConsistencyMode, SpirePlacementState, SpirePublishedEpochSnapshot};
use super::quantizer::{SpireAssignmentPayloadFormat, SpirePreparedAssignmentScorer};
use super::storage::{
    SpireLeafAssignmentRow, SpireLocalObjectStore, SpirePartitionObjectKind,
    SpireRoutingPartitionObject, SpireVecId, SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
    SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE, SPIRE_ASSIGNMENT_FLAG_PRIMARY,
    SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR, SPIRE_ASSIGNMENT_FLAG_TOMBSTONE,
};
use crate::storage::page::ItemPointer;
use pgrx::pg_sys;

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
    pub(super) root_pid: u64,
    pub(super) leaf_pid: u64,
    pub(super) rows: Vec<SpireLeafScanRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SpireScoredScanCandidate {
    pub(super) pid: u64,
    pub(super) object_version: u64,
    pub(super) row_index: u32,
    pub(super) vec_id: SpireVecId,
    pub(super) heap_tid: ItemPointer,
    pub(super) score: f32,
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

    pub(super) fn reset(&mut self, candidates: Vec<SpireScoredScanCandidate>) {
        *self = Self::new(candidates);
    }
}

pub(super) fn collect_snapshot_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
) -> Result<Vec<SpireLeafScanRow>, String> {
    SpirePublishedEpochSnapshot::new(
        snapshot.epoch_manifest,
        snapshot.object_manifest,
        snapshot.placement_directory,
    )?;

    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest.entries {
        let placement = snapshot
            .placement_directory
            .get(manifest_entry.pid)
            .ok_or_else(|| {
                format!(
                    "ec_spire scan snapshot missing placement for pid {}",
                    manifest_entry.pid
                )
            })?;

        if should_skip_placement(snapshot.epoch_manifest.consistency_mode, placement.state)? {
            continue;
        }

        let header = object_store.read_object_header(placement)?;
        if header.kind != SpirePartitionObjectKind::Leaf {
            continue;
        }

        let leaf_object = object_store.read_leaf_object(placement)?;
        for (row_index, assignment) in leaf_object.assignments.into_iter().enumerate() {
            let row_index = u32::try_from(row_index)
                .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
            rows.push(SpireLeafScanRow {
                pid: manifest_entry.pid,
                object_version: manifest_entry.object_version,
                row_index,
                assignment,
            });
        }
    }
    Ok(rows)
}

pub(super) fn collect_snapshot_routed_leaf_rows(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
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
    object_store: &SpireLocalObjectStore,
    query_vector: &[f32],
    nprobe: u32,
) -> Result<Vec<SpireRoutedLeafScanRows>, String> {
    let (root_pid, root_object) = load_snapshot_root_routing_object(snapshot, object_store)?;
    let leaf_pids = route_root_object_to_leaf_pids(&root_object, query_vector, nprobe)?;

    let mut routed = Vec::with_capacity(leaf_pids.len());
    for leaf_pid in leaf_pids {
        let rows = collect_snapshot_leaf_rows_for_pid(snapshot, object_store, leaf_pid, root_pid)?;
        routed.push(SpireRoutedLeafScanRows {
            root_pid,
            leaf_pid,
            rows,
        });
    }
    Ok(routed)
}

pub(super) fn collect_ranked_routed_probe_candidates<F>(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
    query_vector: &[f32],
    nprobe: u32,
    score_ip: F,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireLeafAssignmentRow) -> Result<f32, String>,
{
    let routed_rows =
        collect_snapshot_routed_probe_leaf_rows(snapshot, object_store, query_vector, nprobe)?;
    rank_routed_leaf_rows_by_ip(routed_rows, score_ip, limit)
}

pub(super) fn collect_quantized_routed_probe_candidates(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
    query_vector: &[f32],
    nprobe: u32,
    payload_format: SpireAssignmentPayloadFormat,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String> {
    let scorer =
        SpirePreparedAssignmentScorer::prepare(payload_format, query_vector.len(), query_vector)?;
    collect_ranked_routed_probe_candidates(
        snapshot,
        object_store,
        query_vector,
        nprobe,
        |row| scorer.score_assignment_ip(row),
        limit,
    )
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
    object_store: &SpireLocalObjectStore,
) -> Result<Vec<SpireDeltaScanRow>, String> {
    SpirePublishedEpochSnapshot::new(
        snapshot.epoch_manifest,
        snapshot.object_manifest,
        snapshot.placement_directory,
    )?;

    let mut rows = Vec::new();
    for manifest_entry in &snapshot.object_manifest.entries {
        let placement = snapshot
            .placement_directory
            .get(manifest_entry.pid)
            .ok_or_else(|| {
                format!(
                    "ec_spire scan snapshot missing placement for pid {}",
                    manifest_entry.pid
                )
            })?;

        if should_skip_placement(snapshot.epoch_manifest.consistency_mode, placement.state)? {
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
    object_store: &SpireLocalObjectStore,
) -> Result<Vec<SpireLeafScanRow>, String> {
    let delta_rows = collect_snapshot_delta_rows(snapshot, object_store)?;
    let deleted_vec_ids: HashSet<_> = delta_rows
        .iter()
        .filter(|row| is_delete_delta_assignment(&row.assignment))
        .map(|row| row.assignment.vec_id.clone())
        .collect();

    let mut visible_rows = Vec::new();
    visible_rows.extend(
        collect_snapshot_leaf_rows(snapshot, object_store)?
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

fn rank_routed_leaf_rows_by_ip<F>(
    routed_rows: Vec<SpireRoutedLeafScanRows>,
    mut score_ip: F,
    limit: Option<usize>,
) -> Result<Vec<SpireScoredScanCandidate>, String>
where
    F: FnMut(&SpireLeafAssignmentRow) -> Result<f32, String>,
{
    if limit == Some(0) {
        return Ok(Vec::new());
    }

    let mut candidates_by_vec_id = HashMap::new();
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
                pid: row.pid,
                object_version: row.object_version,
                row_index: row.row_index,
                vec_id: row.assignment.vec_id.clone(),
                heap_tid: row.assignment.heap_tid,
                score: -ip,
            };
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
        }
    }

    let mut candidates = candidates_by_vec_id.into_values().collect::<Vec<_>>();
    candidates.sort_by(scored_candidate_cmp);
    if let Some(limit) = limit {
        candidates.truncate(limit);
    }
    Ok(candidates)
}

fn scored_candidate_cmp(
    left: &SpireScoredScanCandidate,
    right: &SpireScoredScanCandidate,
) -> Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| left.heap_tid.block_number.cmp(&right.heap_tid.block_number))
        .then_with(|| {
            left.heap_tid
                .offset_number
                .cmp(&right.heap_tid.offset_number)
        })
        .then_with(|| left.pid.cmp(&right.pid))
        .then_with(|| left.row_index.cmp(&right.row_index))
}

fn load_snapshot_root_routing_object(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
) -> Result<(u64, SpireRoutingPartitionObject), String> {
    SpirePublishedEpochSnapshot::new(
        snapshot.epoch_manifest,
        snapshot.object_manifest,
        snapshot.placement_directory,
    )?;

    let mut root = None;
    for manifest_entry in &snapshot.object_manifest.entries {
        let placement = snapshot
            .placement_directory
            .get(manifest_entry.pid)
            .ok_or_else(|| {
                format!(
                    "ec_spire scan snapshot missing placement for pid {}",
                    manifest_entry.pid
                )
            })?;
        if should_skip_placement(snapshot.epoch_manifest.consistency_mode, placement.state)? {
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

    let mut scored_children = root_object
        .children
        .iter()
        .map(|child| {
            (
                child.centroid_index,
                child.child_pid,
                inner_product(query_vector, &child.centroid),
            )
        })
        .collect::<Vec<_>>();
    scored_children.sort_by(|left, right| {
        right
            .2
            .total_cmp(&left.2)
            .then_with(|| left.0.cmp(&right.0))
    });

    let requested = usize::try_from(nprobe)
        .map_err(|_| "ec_spire routed scan nprobe exceeds usize".to_owned())?;
    Ok(scored_children
        .into_iter()
        .take(requested)
        .map(|(_, child_pid, _)| child_pid)
        .collect())
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
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
    leaf_pid: u64,
    root_pid: u64,
) -> Result<Vec<SpireLeafScanRow>, String> {
    SpirePublishedEpochSnapshot::new(
        snapshot.epoch_manifest,
        snapshot.object_manifest,
        snapshot.placement_directory,
    )?;

    let manifest_entry = snapshot.object_manifest.get(leaf_pid).ok_or_else(|| {
        format!("ec_spire routed scan missing object manifest entry for leaf pid {leaf_pid}")
    })?;
    let placement = snapshot
        .placement_directory
        .get(leaf_pid)
        .ok_or_else(|| format!("ec_spire routed scan missing placement for leaf pid {leaf_pid}"))?;
    if should_skip_placement(snapshot.epoch_manifest.consistency_mode, placement.state)? {
        return Ok(Vec::new());
    }

    let header = object_store.read_object_header(placement)?;
    if header.kind != SpirePartitionObjectKind::Leaf {
        return Err(format!(
            "ec_spire routed scan pid {leaf_pid} is not a leaf object"
        ));
    }
    let leaf_object = object_store.read_leaf_object(placement)?;
    if leaf_object.header.parent_pid != root_pid {
        return Err(format!(
            "ec_spire routed scan leaf pid {leaf_pid} parent {} does not match root pid {root_pid}",
            leaf_object.header.parent_pid
        ));
    }

    let mut rows = Vec::with_capacity(leaf_object.assignments.len());
    for (row_index, assignment) in leaf_object.assignments.into_iter().enumerate() {
        let row_index = u32::try_from(row_index)
            .map_err(|_| "ec_spire scan row index exceeds u32".to_owned())?;
        rows.push(SpireLeafScanRow {
            pid: leaf_pid,
            object_version: manifest_entry.object_version,
            row_index,
            assignment,
        });
    }
    Ok(rows)
}

fn is_visible_primary_assignment(assignment: &SpireLeafAssignmentRow) -> bool {
    let blocked_flags = SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA
        | SPIRE_ASSIGNMENT_FLAG_TOMBSTONE
        | SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE
        | SPIRE_ASSIGNMENT_FLAG_STALE_LOCATOR;
    assignment.flags & SPIRE_ASSIGNMENT_FLAG_PRIMARY != 0 && assignment.flags & blocked_flags == 0
}

fn is_delete_delta_assignment(assignment: &SpireLeafAssignmentRow) -> bool {
    assignment.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_DELETE != 0
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

pub(super) unsafe extern "C-unwind" fn ec_spire_ambeginscan(
    _index_relation: pg_sys::Relation,
    _nkeys: std::ffi::c_int,
    _norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("ambeginscan")) }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amrescan(
    _scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    _nkeys: std::ffi::c_int,
    _orderbys: pg_sys::ScanKey,
    _norderbys: std::ffi::c_int,
) {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amrescan")) }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amgettuple(
    _scan: pg_sys::IndexScanDesc,
    _direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amgettuple")) }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amendscan(_scan: pg_sys::IndexScanDesc) {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amendscan")) }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_quantized_routed_probe_candidates, collect_ranked_routed_probe_candidates,
        collect_snapshot_delta_rows, collect_snapshot_leaf_rows, collect_snapshot_routed_leaf_rows,
        collect_snapshot_routed_probe_leaf_rows, collect_snapshot_visible_primary_rows,
        rank_routed_leaf_rows_by_ip, rerank_scored_candidates_by_ip, SpireLeafScanRow,
        SpireRoutedLeafScanRows, SpireScanCandidateCursor, SpireScoredScanCandidate,
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
        SpirePublishedEpochSnapshot,
    };
    use crate::am::ec_spire::quantizer::{
        encode_assignment_payload, SpireAssignmentPayloadFormat, SpirePreparedAssignmentScorer,
    };
    use crate::am::ec_spire::storage::SpireLocalObjectStore;
    use crate::am::ec_spire::storage::{
        SpireDeltaPartitionObject, SpireLeafAssignmentRow, SpireLeafPartitionObject, SpireVecId,
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
        let (dimensions, gamma, encoded_payload) =
            encode_assignment_payload(payload_format, source_vector).unwrap();
        assert_eq!(usize::from(dimensions), source_vector.len());
        SpireLeafAssignmentInput {
            heap_tid: tid(block_number, offset_number),
            payload_format: payload_format.tag(),
            gamma,
            encoded_payload,
        }
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
            pid: SPIRE_FIRST_PID + vec_seq,
            object_version: 1,
            row_index: u32::from(offset_number),
            vec_id: SpireVecId::local(vec_seq),
            heap_tid: tid(block_number, offset_number),
            score,
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
                Some(2),
            )
            .unwrap();

            let observed = collect_quantized_routed_probe_candidates(
                &snapshot,
                &object_store,
                &query,
                2,
                payload_format,
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
            Some(2),
        )
        .unwrap_err()
        .contains("payload length mismatch"));
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

        let candidates =
            rank_routed_leaf_rows_by_ip(routed, |row| Ok(f32::from(row.encoded_payload[0])), None)
                .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(7));
        assert_eq!(candidates[0].pid, SPIRE_FIRST_PID + 2);
        assert_eq!(candidates[0].heap_tid, tid(10, 1));
        assert_eq!(candidates[0].score, -9.0);
    }

    #[test]
    fn rank_routed_leaf_rows_by_ip_rejects_non_finite_scores() {
        let routed = vec![SpireRoutedLeafScanRows {
            root_pid: SPIRE_FIRST_PID,
            leaf_pid: SPIRE_FIRST_PID + 1,
            rows: vec![SpireLeafScanRow {
                pid: SPIRE_FIRST_PID + 1,
                object_version: 1,
                row_index: 0,
                assignment: assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 1),
            }],
        }];

        assert!(rank_routed_leaf_rows_by_ip(routed, |_| Ok(f32::NAN), None)
            .unwrap_err()
            .contains("non-finite"));
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
