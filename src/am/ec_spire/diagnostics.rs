use std::collections::{BTreeMap, HashSet};

use super::assign::{
    SpireAllocatorExhaustionDiagnostics, SpireLocalVecIdAllocator, SpirePidAllocator,
};
use super::meta::{
    SpireConsistencyMode, SpirePlacementState, SpirePublishedEpochSnapshot, SpireRootControlState,
    SpireValidatedEpochSnapshot,
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
        let header = object_reader.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Root => {
                diagnostics.root_object_count += 1;
                diagnostics.routing_object_bytes = diagnostics
                    .routing_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| "ec_spire diagnostics routing byte count overflow".to_owned())?;
                let object = object_reader.read_routing_object(placement)?;
                diagnostics.routing_child_count += object.child_count();
            }
            SpirePartitionObjectKind::Internal => {
                diagnostics.internal_object_count += 1;
                diagnostics.routing_object_bytes = diagnostics
                    .routing_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| "ec_spire diagnostics routing byte count overflow".to_owned())?;
                let object = object_reader.read_routing_object(placement)?;
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
                let object = object_reader.read_delta_object(placement)?;
                diagnostics.delta_assignment_count += object.assignments.len();
            }
        }
    }

    diagnostics.local_store_count = local_stores.len();
    Ok(diagnostics)
}

pub(super) fn collect_store_placement_diagnostics(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_reader: &impl SpireObjectReader,
) -> Result<Vec<SpireStorePlacementDiagnostics>, String> {
    let snapshot = SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    let epoch = snapshot.epoch_manifest().epoch;
    let mut by_store = BTreeMap::<(u32, u32), SpireStorePlacementDiagnostics>::new();

    for placement in &snapshot.placement_directory().entries {
        let entry = by_store
            .entry((placement.node_id, placement.local_store_id))
            .or_insert_with(|| SpireStorePlacementDiagnostics {
                epoch,
                node_id: placement.node_id,
                local_store_id: placement.local_store_id,
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
        let header = object_reader.read_object_header(placement)?;
        match header.kind {
            SpirePartitionObjectKind::Root => {
                entry.root_object_count += 1;
                let routing_object_bytes = entry
                    .routing_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| {
                        "ec_spire placement diagnostics routing byte count overflow".to_owned()
                    })?;
                entry.routing_object_bytes = routing_object_bytes;
                let object = object_reader.read_routing_object(placement)?;
                entry.routing_child_count += object.child_count();
            }
            SpirePartitionObjectKind::Internal => {
                entry.internal_object_count += 1;
                let routing_object_bytes = entry
                    .routing_object_bytes
                    .checked_add(object_bytes)
                    .ok_or_else(|| {
                        "ec_spire placement diagnostics routing byte count overflow".to_owned()
                    })?;
                entry.routing_object_bytes = routing_object_bytes;
                let object = object_reader.read_routing_object(placement)?;
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
                let object = object_reader.read_delta_object(placement)?;
                entry.assignment_count = entry
                    .assignment_count
                    .checked_add(object.assignments.len())
                    .ok_or_else(|| {
                        "ec_spire placement diagnostics assignment count overflow".to_owned()
                    })?;
            }
        }
    }

    Ok(by_store.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::{
        collect_allocator_diagnostics, collect_snapshot_diagnostics,
        collect_store_placement_diagnostics,
    };
    use crate::am::ec_spire::assign::{
        SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::build::{
        build_partitioned_single_level_leaf_epoch_draft, SpirePartitionedSingleLevelBuildInput,
        SpireSingleLevelCentroidPlan,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpirePlacementDirectory,
        SpirePlacementState, SpirePublishedEpochSnapshot, SpireRootControlState,
    };
    use crate::am::ec_spire::storage::SpireLocalObjectStore;
    use crate::storage::page::ItemPointer;

    fn tid(block_number: u32, offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn assignment_input(block_number: u32, offset_number: u16) -> SpireLeafAssignmentInput {
        SpireLeafAssignmentInput {
            heap_tid: tid(block_number, offset_number),
            payload_format: 1,
            gamma: 0.5,
            encoded_payload: vec![1, 2, 3],
        }
    }

    fn partitioned_build_input() -> SpirePartitionedSingleLevelBuildInput {
        SpirePartitionedSingleLevelBuildInput {
            epoch: 7,
            object_version: 1,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            consistency_mode: SpireConsistencyMode::Strict,
            root_placement_tid: tid(60, 3),
            placement_tids: vec![tid(60, 1), tid(60, 2)],
            assignments: vec![assignment_input(10, 1), assignment_input(10, 2)],
            centroid_plan: SpireSingleLevelCentroidPlan {
                dimensions: 2,
                centroids: vec![vec![1.0, 0.0], vec![-1.0, 0.0]],
                assignment_indexes: vec![0, 1],
            },
        }
    }

    #[test]
    fn allocator_diagnostics_uses_root_control_cursors() {
        let root_control = SpireRootControlState::published(
            7,
            u64::MAX - 3,
            u64::MAX - 2,
            tid(70, 1),
            tid(70, 2),
            tid(70, 3),
        )
        .unwrap();

        let diagnostics = collect_allocator_diagnostics(&root_control, 3).unwrap();

        assert_eq!(diagnostics.pid.next_value, u64::MAX - 3);
        assert_eq!(diagnostics.pid.remaining_allocations, 3);
        assert!(diagnostics.pid.near_exhaustion);
        assert_eq!(diagnostics.local_vec_id.next_value, u64::MAX - 2);
        assert_eq!(diagnostics.local_vec_id.remaining_allocations, 2);
        assert!(diagnostics.local_vec_id.near_exhaustion);
    }

    #[test]
    fn snapshot_diagnostics_counts_partition_objects_and_assignments() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(),
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

        let diagnostics = collect_snapshot_diagnostics(&snapshot, &object_store).unwrap();

        assert_eq!(diagnostics.epoch, 7);
        assert_eq!(diagnostics.consistency_mode, SpireConsistencyMode::Strict);
        assert_eq!(diagnostics.object_count, 3);
        assert_eq!(diagnostics.placement_count, 3);
        assert_eq!(diagnostics.local_store_count, 1);
        assert_eq!(diagnostics.available_placement_count, 3);
        assert_eq!(diagnostics.root_object_count, 1);
        assert_eq!(diagnostics.internal_object_count, 0);
        assert_eq!(diagnostics.leaf_object_count, 2);
        assert_eq!(diagnostics.delta_object_count, 0);
        assert_eq!(diagnostics.routing_child_count, 2);
        assert_eq!(diagnostics.leaf_assignment_count, 2);
        assert_eq!(diagnostics.delta_assignment_count, 0);
        assert!(diagnostics.available_object_bytes > 0);
        assert!(diagnostics.routing_object_bytes > 0);
        assert!(diagnostics.leaf_object_bytes > 0);
        assert_eq!(diagnostics.delta_object_bytes, 0);
        assert_eq!(
            diagnostics.available_object_bytes,
            diagnostics.routing_object_bytes
                + diagnostics.leaf_object_bytes
                + diagnostics.delta_object_bytes
        );
    }

    #[test]
    fn snapshot_diagnostics_counts_degraded_unavailable_without_reading_object() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(),
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
            .find(|placement| placement.pid == SPIRE_FIRST_PID + 2)
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

        let diagnostics = collect_snapshot_diagnostics(&snapshot, &object_store).unwrap();

        assert_eq!(diagnostics.consistency_mode, SpireConsistencyMode::Degraded);
        assert_eq!(diagnostics.object_count, 3);
        assert_eq!(diagnostics.available_placement_count, 2);
        assert_eq!(diagnostics.unavailable_placement_count, 1);
        assert_eq!(diagnostics.root_object_count, 1);
        assert_eq!(diagnostics.leaf_object_count, 1);
        assert_eq!(diagnostics.leaf_assignment_count, 1);
        assert_eq!(diagnostics.routing_child_count, 2);
        assert_eq!(
            diagnostics.available_object_bytes,
            diagnostics.routing_object_bytes + diagnostics.leaf_object_bytes
        );
        assert_eq!(diagnostics.delta_object_bytes, 0);
    }

    #[test]
    fn store_placement_diagnostics_groups_available_objects_by_store() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(),
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

        let diagnostics = collect_store_placement_diagnostics(&snapshot, &object_store).unwrap();

        assert_eq!(diagnostics.len(), 1);
        let store = &diagnostics[0];
        assert_eq!(store.epoch, 7);
        assert_eq!(store.node_id, 0);
        assert_eq!(store.local_store_id, 0);
        assert_eq!(store.placement_count, 3);
        assert_eq!(store.available_placement_count, 3);
        assert_eq!(store.object_count, 3);
        assert_eq!(store.root_object_count, 1);
        assert_eq!(store.leaf_object_count, 2);
        assert_eq!(store.routing_child_count, 2);
        assert_eq!(store.assignment_count, 2);
        assert!(store.placement_object_bytes > 0);
        assert_eq!(store.placement_object_bytes, store.available_object_bytes);
        assert_eq!(
            store.available_object_bytes,
            store.routing_object_bytes + store.leaf_object_bytes + store.delta_object_bytes
        );
    }
}
