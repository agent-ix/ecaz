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
    fn diagnostics_count_delta_objects_and_assignments() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_partitioned_single_level_leaf_epoch_draft(
            partitioned_build_input(),
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
            SpireDeltaEpochInput {
                epoch: 8,
                object_version: 1,
                published_at_micros: 3000,
                retain_until_micros: 4000,
                consistency_mode: SpireConsistencyMode::Strict,
                base_pid: SPIRE_FIRST_PID + 1,
                placement_tid: tid(80, 1),
                insert_assignments: vec![assignment_input(20, 1)],
                delete_assignments: Vec::new(),
            },
            &base_snapshot,
            &mut pid_allocator,
            &mut local_vec_id_allocator,
            &mut object_store,
        )
        .unwrap();
        let delta_snapshot = SpirePublishedEpochSnapshot::new(
            &delta_draft.epoch_manifest,
            &delta_draft.object_manifest,
            &delta_draft.placement_directory,
        )
        .unwrap();

        let snapshot_diagnostics =
            collect_snapshot_diagnostics(&delta_snapshot, &object_store).unwrap();
        let store_diagnostics =
            collect_store_placement_diagnostics(&delta_snapshot, &object_store).unwrap();

        assert_eq!(snapshot_diagnostics.object_count, 4);
        assert_eq!(snapshot_diagnostics.root_object_count, 1);
        assert_eq!(snapshot_diagnostics.leaf_object_count, 2);
        assert_eq!(snapshot_diagnostics.delta_object_count, 1);
        assert_eq!(snapshot_diagnostics.leaf_assignment_count, 2);
        assert_eq!(snapshot_diagnostics.delta_assignment_count, 1);
        assert!(snapshot_diagnostics.delta_object_bytes > 0);
        assert_eq!(
            snapshot_diagnostics.available_object_bytes,
            snapshot_diagnostics.routing_object_bytes
                + snapshot_diagnostics.leaf_object_bytes
                + snapshot_diagnostics.delta_object_bytes
        );

        assert_eq!(store_diagnostics.len(), 1);
        let store = &store_diagnostics[0];
        assert_eq!(store.placement_count, 4);
        assert_eq!(store.object_count, 4);
        assert_eq!(store.root_object_count, 1);
        assert_eq!(store.leaf_object_count, 2);
        assert_eq!(store.delta_object_count, 1);
        assert_eq!(store.assignment_count, 3);
        assert!(store.delta_object_bytes > 0);
        assert_eq!(
            store.available_object_bytes,
            store.routing_object_bytes + store.leaf_object_bytes + store.delta_object_bytes
        );
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

    #[test]
    fn store_placement_diagnostics_counts_degraded_unavailable_and_skipped_without_reading_them() {
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
            .find(|placement| placement.pid == SPIRE_FIRST_PID + 1)
            .unwrap()
            .state = SpirePlacementState::Unavailable;
        placements
            .iter_mut()
            .find(|placement| placement.pid == SPIRE_FIRST_PID + 2)
            .unwrap()
            .state = SpirePlacementState::Skipped;
        let placement_directory =
            SpirePlacementDirectory::from_entries(draft.epoch_manifest.epoch, placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &draft.object_manifest,
            &placement_directory,
        )
        .unwrap();

        let diagnostics = collect_store_placement_diagnostics(&snapshot, &object_store).unwrap();

        assert_eq!(diagnostics.len(), 1);
        let store = &diagnostics[0];
        assert_eq!(store.placement_count, 3);
        assert_eq!(store.available_placement_count, 1);
        assert_eq!(store.stale_placement_count, 0);
        assert_eq!(store.unavailable_placement_count, 1);
        assert_eq!(store.skipped_placement_count, 1);
        assert_eq!(store.object_count, 1);
        assert_eq!(store.root_object_count, 1);
        assert_eq!(store.leaf_object_count, 0);
        assert_eq!(store.routing_child_count, 2);
        assert_eq!(store.assignment_count, 0);
        assert!(store.placement_object_bytes > store.available_object_bytes);
        assert_eq!(store.available_object_bytes, store.routing_object_bytes);
        assert_eq!(store.leaf_object_bytes, 0);
        assert_eq!(store.delta_object_bytes, 0);
    }

    #[test]
    fn store_placement_diagnostics_cannot_collect_stale_published_snapshot() {
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
            .find(|placement| placement.pid == SPIRE_FIRST_PID)
            .unwrap()
            .state = SpirePlacementState::Stale;
        let placement_directory =
            SpirePlacementDirectory::from_entries(draft.epoch_manifest.epoch, placements).unwrap();

        let error = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &draft.object_manifest,
            &placement_directory,
        )
        .unwrap_err();

        assert_eq!(
            error,
            "ec_spire degraded published snapshot cannot use stale placement for pid 1"
        );
    }
}
