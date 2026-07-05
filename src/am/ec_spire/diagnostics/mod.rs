use std::collections::{BTreeMap, HashSet};

use super::assign::{
    SpireAllocatorExhaustionDiagnostics, SpireLocalVecIdAllocator, SpirePidAllocator,
};
use super::meta::{
    SpireConsistencyMode, SpirePlacementEntry, SpirePlacementState, SpirePublishedEpochSnapshot,
    SpireRootControlState, SpireValidatedEpochSnapshot, SPIRE_LOCAL_NODE_ID,
};
use super::storage::{SpireObjectReader, SpirePartitionObjectKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireSnapshotDiagnostics {
    pub(super) epoch: u64,
    pub(super) consistency_mode: SpireConsistencyMode,
    pub(super) object_count: usize,
    pub(super) placement_count: usize,
    pub(super) local_store_count: usize,
    pub(super) available_placement_count: usize,
    pub(super) stale_placement_count: usize,
    pub(super) unavailable_placement_count: usize,
    pub(super) skipped_placement_count: usize,
    pub(super) root_object_count: usize,
    pub(super) internal_object_count: usize,
    pub(super) leaf_object_count: usize,
    pub(super) delta_object_count: usize,
    pub(super) routing_child_count: usize,
    pub(super) leaf_assignment_count: usize,
    pub(super) delta_assignment_count: usize,
    pub(super) available_object_bytes: u64,
    pub(super) routing_object_bytes: u64,
    pub(super) leaf_object_bytes: u64,
    pub(super) delta_object_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpireStorePlacementDiagnostics {
    pub(super) epoch: u64,
    pub(super) node_id: u32,
    pub(super) local_store_id: u32,
    pub(super) store_relid: u32,
    pub(super) placement_count: usize,
    pub(super) available_placement_count: usize,
    pub(super) stale_placement_count: usize,
    pub(super) unavailable_placement_count: usize,
    pub(super) skipped_placement_count: usize,
    pub(super) object_count: usize,
    pub(super) root_object_count: usize,
    pub(super) internal_object_count: usize,
    pub(super) leaf_object_count: usize,
    pub(super) delta_object_count: usize,
    pub(super) routing_child_count: usize,
    pub(super) assignment_count: usize,
    pub(super) placement_object_bytes: u64,
    pub(super) available_object_bytes: u64,
    pub(super) routing_object_bytes: u64,
    pub(super) leaf_object_bytes: u64,
    pub(super) delta_object_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SpireAllocatorDiagnostics {
    pub(super) pid: SpireAllocatorExhaustionDiagnostics,
    pub(super) local_vec_id: SpireAllocatorExhaustionDiagnostics,
}

pub(super) fn collect_allocator_diagnostics(
    root_control: &SpireRootControlState,
    warn_within: u64,
) -> Result<SpireAllocatorDiagnostics, String> {
    let pid_allocator = SpirePidAllocator::new(root_control.next_pid)?;
    let local_vec_id_allocator = SpireLocalVecIdAllocator::new(root_control.next_local_vec_seq)?;
    Ok(SpireAllocatorDiagnostics {
        pid: pid_allocator.exhaustion_diagnostics(warn_within),
        local_vec_id: local_vec_id_allocator.exhaustion_diagnostics(warn_within),
    })
}

pub(super) fn collect_snapshot_diagnostics(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_reader: &impl SpireObjectReader,
) -> Result<SpireSnapshotDiagnostics, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;

    let mut diagnostics = SpireSnapshotDiagnostics {
        epoch: snapshot.epoch_manifest().epoch,
        consistency_mode: snapshot.epoch_manifest().consistency_mode,
        object_count: snapshot.object_manifest().entries.len(),
        placement_count: snapshot.placement_directory().entries.len(),
        local_store_count: 0,
        available_placement_count: 0,
        stale_placement_count: 0,
        unavailable_placement_count: 0,
        skipped_placement_count: 0,
        root_object_count: 0,
        internal_object_count: 0,
        leaf_object_count: 0,
        delta_object_count: 0,
        routing_child_count: 0,
        leaf_assignment_count: 0,
        delta_assignment_count: 0,
        available_object_bytes: 0,
        routing_object_bytes: 0,
        leaf_object_bytes: 0,
        delta_object_bytes: 0,
    };
    let mut local_stores = HashSet::new();

    for placement in &snapshot.placement_directory().entries {
        local_stores.insert((placement.node_id, placement.local_store_id));
        match placement.state {
            SpirePlacementState::Available => {
                diagnostics.available_placement_count += 1;
            }
            SpirePlacementState::Stale => {
                // Published snapshots reject Stale placements today; this
                // branch stays defensive for future retained-placement
                // diagnostics that may expose stale local stores.
                diagnostics.stale_placement_count += 1;
                continue;
            }
            SpirePlacementState::Unavailable => {
                diagnostics.unavailable_placement_count += 1;
                continue;
            }
            SpirePlacementState::Skipped => {
                diagnostics.skipped_placement_count += 1;
                continue;
            }
        }

        let object_bytes = u64::from(placement.object_bytes);
        diagnostics.available_object_bytes = diagnostics
            .available_object_bytes
            .checked_add(object_bytes)
            .ok_or_else(|| "ec_spire diagnostics object byte count overflow".to_owned())?;
        let metadata_placement = coordinator_metadata_read_placement(placement);
        let header = object_reader.read_object_header(&metadata_placement)?;
        match header.kind {
            SpirePartitionObjectKind::Root => {
                diagnostics.root_object_count += 1;
                diagnostics.routing_object_bytes = diagnostics
                    .routing_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| "ec_spire diagnostics routing byte count overflow".to_owned())?;
                let object = object_reader.read_routing_object(&metadata_placement)?;
                diagnostics.routing_child_count += object.child_count();
            }
            SpirePartitionObjectKind::Internal => {
                diagnostics.internal_object_count += 1;
                diagnostics.routing_object_bytes = diagnostics
                    .routing_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| "ec_spire diagnostics routing byte count overflow".to_owned())?;
                let object = object_reader.read_routing_object(&metadata_placement)?;
                diagnostics.routing_child_count += object.child_count();
            }
            SpirePartitionObjectKind::Leaf => {
                diagnostics.leaf_object_count += 1;
                diagnostics.leaf_object_bytes = diagnostics
                    .leaf_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| "ec_spire diagnostics leaf byte count overflow".to_owned())?;
                let assignment_count = usize::try_from(header.assignment_count).map_err(|_| {
                    "ec_spire diagnostics leaf assignment count exceeds usize".to_owned()
                })?;
                diagnostics.leaf_assignment_count = diagnostics
                    .leaf_assignment_count
                    .checked_add(assignment_count)
                    .ok_or_else(|| {
                        "ec_spire diagnostics leaf assignment count overflow".to_owned()
                    })?;
            }
            SpirePartitionObjectKind::Delta => {
                diagnostics.delta_object_count += 1;
                diagnostics.delta_object_bytes = diagnostics
                    .delta_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| "ec_spire diagnostics delta byte count overflow".to_owned())?;
                let object = object_reader.read_delta_object(&metadata_placement)?;
                diagnostics.delta_assignment_count += object.assignments.len();
            }
            SpirePartitionObjectKind::TopGraph => {}
        }
    }

    diagnostics.local_store_count = local_stores.len();
    Ok(diagnostics)
}

fn coordinator_metadata_read_placement(placement: &SpirePlacementEntry) -> SpirePlacementEntry {
    let mut placement = *placement;
    placement.node_id = SPIRE_LOCAL_NODE_ID;
    placement
}

fn placement_routing_byte_count_overflow() -> String {
    "ec_spire placement diagnostics routing byte count overflow".to_owned()
}

pub(super) fn collect_store_placement_diagnostics(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_reader: &impl SpireObjectReader,
) -> Result<Vec<SpireStorePlacementDiagnostics>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let epoch = snapshot.epoch_manifest().epoch;
    let mut by_store = BTreeMap::<(u32, u32, u32), SpireStorePlacementDiagnostics>::new();

    for placement in &snapshot.placement_directory().entries {
        let entry = by_store
            .entry((
                placement.node_id,
                placement.local_store_id,
                placement.store_relid,
            ))
            .or_insert_with(|| SpireStorePlacementDiagnostics {
                epoch,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
                store_relid: placement.store_relid,
                placement_count: 0,
                available_placement_count: 0,
                stale_placement_count: 0,
                unavailable_placement_count: 0,
                skipped_placement_count: 0,
                object_count: 0,
                root_object_count: 0,
                internal_object_count: 0,
                leaf_object_count: 0,
                delta_object_count: 0,
                routing_child_count: 0,
                assignment_count: 0,
                placement_object_bytes: 0,
                available_object_bytes: 0,
                routing_object_bytes: 0,
                leaf_object_bytes: 0,
                delta_object_bytes: 0,
            });

        entry.placement_count += 1;
        entry.placement_object_bytes = entry
            .placement_object_bytes
            .checked_add(u64::from(placement.object_bytes))
            .ok_or_else(|| "ec_spire placement diagnostics byte count overflow".to_owned())?;

        match placement.state {
            SpirePlacementState::Available => {
                entry.available_placement_count += 1;
            }
            SpirePlacementState::Stale => {
                // Published snapshots reject Stale placements today; this
                // branch stays defensive for future retained-placement
                // diagnostics that may expose stale local stores.
                entry.stale_placement_count += 1;
                continue;
            }
            SpirePlacementState::Unavailable => {
                entry.unavailable_placement_count += 1;
                continue;
            }
            SpirePlacementState::Skipped => {
                entry.skipped_placement_count += 1;
                continue;
            }
        }

        let object_bytes = u64::from(placement.object_bytes);
        entry.object_count += 1;
        entry.available_object_bytes = entry
            .available_object_bytes
            .checked_add(object_bytes)
            .ok_or_else(|| {
                "ec_spire placement diagnostics available byte count overflow".to_owned()
            })?;
        let metadata_placement = coordinator_metadata_read_placement(placement);
        let header = object_reader.read_object_header(&metadata_placement)?;
        match header.kind {
            SpirePartitionObjectKind::Root => {
                entry.root_object_count += 1;
                entry.routing_object_bytes = entry
                    .routing_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(placement_routing_byte_count_overflow)?;
                let object = object_reader.read_routing_object(&metadata_placement)?;
                entry.routing_child_count += object.child_count();
            }
            SpirePartitionObjectKind::Internal => {
                entry.internal_object_count += 1;
                entry.routing_object_bytes = entry
                    .routing_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(placement_routing_byte_count_overflow)?;
                let object = object_reader.read_routing_object(&metadata_placement)?;
                entry.routing_child_count += object.child_count();
            }
            SpirePartitionObjectKind::Leaf => {
                entry.leaf_object_count += 1;
                entry.leaf_object_bytes = entry
                    .leaf_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| {
                        "ec_spire placement diagnostics leaf byte count overflow".to_owned()
                    })?;
                let assignment_count = usize::try_from(header.assignment_count).map_err(|_| {
                    "ec_spire placement diagnostics assignment count exceeds usize".to_owned()
                })?;
                entry.assignment_count = entry
                    .assignment_count
                    .checked_add(assignment_count)
                    .ok_or_else(|| {
                        "ec_spire placement diagnostics assignment count overflow".to_owned()
                    })?;
            }
            SpirePartitionObjectKind::Delta => {
                entry.delta_object_count += 1;
                entry.delta_object_bytes = entry
                    .delta_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| {
                        "ec_spire placement diagnostics delta byte count overflow".to_owned()
                    })?;
                let object = object_reader.read_delta_object(&metadata_placement)?;
                entry.assignment_count = entry
                    .assignment_count
                    .checked_add(object.assignments.len())
                    .ok_or_else(|| {
                        "ec_spire placement diagnostics assignment count overflow".to_owned()
                    })?;
            }
            SpirePartitionObjectKind::TopGraph => {}
        }
    }

    Ok(by_store.into_values().collect())
}

include!("tests.rs");
