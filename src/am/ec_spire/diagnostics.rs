use std::collections::HashSet;

use super::meta::{
    SpireConsistencyMode, SpirePlacementState, SpirePublishedEpochSnapshot,
    SpireValidatedEpochSnapshot,
};
use super::storage::{SpireLocalObjectStore, SpirePartitionObjectKind};

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
}

pub(super) fn collect_snapshot_diagnostics(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    object_store: &SpireLocalObjectStore,
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

        diagnostics.available_object_bytes = diagnostics
            .available_object_bytes
            .checked_add(u64::from(placement.object_bytes))
            .ok_or_else(|| "ec_spire diagnostics object byte count overflow".to_owned())?;
        match object_store.read_object_header(placement)?.kind {
            SpirePartitionObjectKind::Root => {
                diagnostics.root_object_count += 1;
                let object = object_store.read_routing_object(placement)?;
                diagnostics.routing_child_count += object.children.len();
            }
            SpirePartitionObjectKind::Internal => {
                diagnostics.internal_object_count += 1;
                let object = object_store.read_routing_object(placement)?;
                diagnostics.routing_child_count += object.children.len();
            }
            SpirePartitionObjectKind::Leaf => {
                diagnostics.leaf_object_count += 1;
                let object = object_store.read_leaf_object(placement)?;
                diagnostics.leaf_assignment_count += object.assignments.len();
            }
            SpirePartitionObjectKind::Delta => {
                diagnostics.delta_object_count += 1;
                let object = object_store.read_delta_object(placement)?;
                diagnostics.delta_assignment_count += object.assignments.len();
            }
        }
    }

    diagnostics.local_store_count = local_stores.len();
    Ok(diagnostics)
}

#[cfg(test)]
mod tests {
    use super::collect_snapshot_diagnostics;
    use crate::am::ec_spire::assign::{
        SpireLeafAssignmentInput, SpireLocalVecIdAllocator, SpirePidAllocator, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::build::{
        build_partitioned_single_level_leaf_epoch_draft, SpirePartitionedSingleLevelBuildInput,
        SpireSingleLevelCentroidPlan,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpirePlacementDirectory,
        SpirePlacementState, SpirePublishedEpochSnapshot,
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
    }
}
