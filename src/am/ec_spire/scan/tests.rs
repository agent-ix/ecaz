#[cfg(test)]
mod tests {
    use super::{
        collect_quantized_routed_probe_candidates, collect_ranked_routed_probe_candidates,
        collect_reranked_quantized_routed_probe_candidates,
        collect_single_level_scan_plan_placement_diagnostics,
        collect_single_level_scan_plan_reranked_candidates, collect_snapshot_delta_rows,
        collect_snapshot_leaf_rows, collect_snapshot_routed_leaf_rows,
        collect_snapshot_routed_probe_leaf_rows, collect_snapshot_visible_primary_rows,
        count_snapshot_recursive_leaf_pids, count_snapshot_single_level_leaf_pids,
        group_leaf_and_delta_reads_by_local_store, group_leaf_routes_by_local_store,
        load_snapshot_routing_hierarchy, prepare_single_level_snapshot_scan_candidates,
        rank_routed_leaf_rows_by_ip, rerank_scored_candidates_by_ip,
        route_recursive_routing_objects_to_leaf_pids, route_root_object_to_leaf_pids,
        route_routing_object_to_child_pids, SpireDeltaObjectRoute, SpireLeafScanRow,
        SpireRecursiveLeafRoute, SpireRoutedLeafScanRows, SpireScanCandidateCursor,
        SpireScanOpaque, SpireScanOutput, SpireScanQuery, SpireScoredScanCandidate,
    };
    use crate::am::ec_spire::assign::{
        SpireDeleteDeltaInput, SpireLeafAssignmentInput, SpireLocalVecIdAllocator,
        SpirePidAllocator, SPIRE_FIRST_PID,
    };
    use crate::am::ec_spire::build::{
        build_local_recursive_routing_epoch_draft, build_partitioned_single_level_leaf_epoch_draft,
        build_recursive_routing_hierarchy_draft, build_single_level_leaf_epoch_draft,
        SpirePartitionedSingleLevelBuildInput, SpireRecursiveRoutingBuildInput,
        SpireRecursiveRoutingChildInput, SpireRecursiveRoutingEpochInput,
        SpireSingleLevelBuildInput, SpireSingleLevelCentroidPlan,
    };
    use crate::am::ec_spire::meta::{
        SpireConsistencyMode, SpireEpochManifest, SpireEpochState, SpireManifestEntry,
        SpireObjectManifest, SpirePlacementDirectory, SpirePlacementEntry, SpirePlacementState,
        SpirePublishedEpochSnapshot, SpireRootControlState, SpireValidatedEpochSnapshot,
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
    use std::collections::HashMap;

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

    fn manifest_entry_for(placement: &SpirePlacementEntry) -> SpireManifestEntry {
        SpireManifestEntry {
            epoch: placement.epoch,
            pid: placement.pid,
            object_version: placement.object_version,
            placement_tid: placement.object_tid,
        }
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
    fn collect_snapshot_routed_probe_leaf_rows_accepts_recursive_leaf_parent() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root_pid = SPIRE_FIRST_PID;
        let internal_pid = SPIRE_FIRST_PID + 1;
        let first_leaf_pid = SPIRE_FIRST_PID + 2;
        let second_leaf_pid = SPIRE_FIRST_PID + 3;
        let root = SpireRoutingPartitionObject::root_at_level(
            root_pid,
            1,
            2,
            2,
            vec![routing_child(0, internal_pid, vec![1.0, 0.0])],
        )
        .unwrap();
        let internal = SpireRoutingPartitionObject::internal(
            internal_pid,
            1,
            1,
            root_pid,
            2,
            vec![
                routing_child(0, first_leaf_pid, vec![0.5, 0.0]),
                routing_child(1, second_leaf_pid, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let first_leaf_rows = vec![assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 1)];
        let second_leaf_rows = vec![assignment_row(SPIRE_ASSIGNMENT_FLAG_PRIMARY, 2)];
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store.insert_routing_object(7, &internal).unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    first_leaf_pid,
                    1,
                    internal_pid,
                    &first_leaf_rows,
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    second_leaf_pid,
                    1,
                    internal_pid,
                    &second_leaf_rows,
                )
                .unwrap(),
        ];
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
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let routed =
            collect_snapshot_routed_probe_leaf_rows(&snapshot, &object_store, &[1.0, 0.0], 1)
                .unwrap();

        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].root_pid, root_pid);
        assert_eq!(routed[0].leaf_pid, second_leaf_pid);
        assert_eq!(routed[0].rows.len(), 1);
        assert_eq!(routed[0].rows[0].assignment.heap_tid, tid(10, 2));
    }

    #[test]
    fn collect_scan_placement_diagnostics_counts_routed_store_rows() {
        let mut pid_allocator = SpirePidAllocator::default();
        let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let base_draft = build_partitioned_single_level_leaf_epoch_draft(
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
        let base_snapshot = SpirePublishedEpochSnapshot::new(
            &base_draft.epoch_manifest,
            &base_draft.object_manifest,
            &base_draft.placement_directory,
        )
        .unwrap();
        let mut delta = delta_input(
            vec![quantized_assignment_input(
                30,
                1,
                SpireAssignmentPayloadFormat::TurboQuant,
                &[1.0, 0.0],
            )],
            vec![delete_delta_input(1, 10, 1)],
        );
        delta.base_pid = SPIRE_FIRST_PID + 1;
        let delta_draft = build_delta_epoch_draft_from_snapshot(
            delta,
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
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 2,
            nprobe: 1,
            nprobe_source: "relation",
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 10,
            rerank_width_source: "relation",
            candidate_limit: Some(10),
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };

        let diagnostics = collect_single_level_scan_plan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            scan_plan,
        )
        .unwrap();

        assert_eq!(diagnostics.scan_plan.leaf_count, 2);
        assert_eq!(diagnostics.scan_plan.nprobe, 1);
        assert_eq!(diagnostics.scan_plan.nprobe_source, "relation");
        assert_eq!(diagnostics.stores.len(), 1);
        let store = &diagnostics.stores[0];
        assert_eq!(store.epoch, 8);
        assert_eq!(store.node_id, 0);
        assert_eq!(store.local_store_id, 0);
        assert_eq!(store.scanned_pid_count, 2);
        assert_eq!(store.leaf_pid_count, 1);
        assert_eq!(store.delta_pid_count, 1);
        assert_eq!(store.candidate_row_count, 1);
        assert_eq!(store.leaf_candidate_row_count, 0);
        assert_eq!(store.delta_candidate_row_count, 1);
        assert_eq!(store.delete_delta_row_count, 1);

        let zero_nprobe_plan = SpireSingleLevelScanPlan {
            nprobe: 0,
            ..scan_plan
        };
        let zero_nprobe_diagnostics = collect_single_level_scan_plan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            zero_nprobe_plan,
        )
        .unwrap();
        assert_eq!(zero_nprobe_diagnostics.scan_plan.nprobe, 0);
        assert!(zero_nprobe_diagnostics.stores.is_empty());

        let stale_leaf_count_plan = SpireSingleLevelScanPlan {
            leaf_count: 3,
            ..scan_plan
        };
        let error = collect_single_level_scan_plan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            stale_leaf_count_plan,
        )
        .unwrap_err();
        assert!(error.contains("does not match snapshot leaf_count 2"));
    }

    #[test]
    fn collect_scan_placement_diagnostics_skips_degraded_unavailable_leaf() {
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
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();
        let scan_plan = SpireSingleLevelScanPlan {
            leaf_count: 2,
            nprobe: 2,
            nprobe_source: "relation",
            payload_format: SpireAssignmentPayloadFormat::TurboQuant,
            rerank_width: 10,
            rerank_width_source: "relation",
            candidate_limit: Some(10),
            dedupe_mode: SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
        };

        let diagnostics = collect_single_level_scan_plan_placement_diagnostics(
            &snapshot,
            &object_store,
            &query,
            scan_plan,
        )
        .unwrap();

        assert_eq!(diagnostics.stores.len(), 1);
        let store = &diagnostics.stores[0];
        assert_eq!(store.epoch, 7);
        assert_eq!(store.scanned_pid_count, 1);
        assert_eq!(store.leaf_pid_count, 1);
        assert_eq!(store.delta_pid_count, 0);
        assert_eq!(store.candidate_row_count, 1);
        assert_eq!(store.leaf_candidate_row_count, 1);
        assert_eq!(store.delta_candidate_row_count, 0);
        assert_eq!(store.delete_delta_row_count, 0);
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
    fn count_snapshot_recursive_leaf_pids_counts_leaf_level_children() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root_pid = SPIRE_FIRST_PID;
        let first_internal_pid = SPIRE_FIRST_PID + 1;
        let second_internal_pid = SPIRE_FIRST_PID + 2;
        let root = SpireRoutingPartitionObject::root_at_level(
            root_pid,
            1,
            2,
            2,
            vec![
                routing_child(0, first_internal_pid, vec![1.0, 0.0]),
                routing_child(1, second_internal_pid, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let first_internal = SpireRoutingPartitionObject::internal(
            first_internal_pid,
            1,
            1,
            root_pid,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![0.25, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 11, vec![0.75, 0.0]),
            ],
        )
        .unwrap();
        let second_internal = SpireRoutingPartitionObject::internal(
            second_internal_pid,
            1,
            1,
            root_pid,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0])],
        )
        .unwrap();
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store
                .insert_routing_object(7, &first_internal)
                .unwrap(),
            object_store
                .insert_routing_object(7, &second_internal)
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 10,
                    1,
                    first_internal_pid,
                    &[],
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 11,
                    1,
                    first_internal_pid,
                    &[],
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    SPIRE_FIRST_PID + 20,
                    1,
                    second_internal_pid,
                    &[],
                )
                .unwrap(),
        ];
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
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        assert_eq!(
            count_snapshot_single_level_leaf_pids(&snapshot, &object_store).unwrap(),
            2
        );
        assert_eq!(
            count_snapshot_recursive_leaf_pids(&snapshot, &object_store).unwrap(),
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
    fn collect_quantized_routed_probe_candidates_accepts_recursive_leaf_parent() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root_pid = SPIRE_FIRST_PID;
        let internal_pid = SPIRE_FIRST_PID + 1;
        let first_leaf_pid = SPIRE_FIRST_PID + 2;
        let second_leaf_pid = SPIRE_FIRST_PID + 3;
        let payload_format = SpireAssignmentPayloadFormat::TurboQuant;
        let root = SpireRoutingPartitionObject::root_at_level(
            root_pid,
            1,
            2,
            2,
            vec![routing_child(0, internal_pid, vec![1.0, 0.0])],
        )
        .unwrap();
        let internal = SpireRoutingPartitionObject::internal(
            internal_pid,
            1,
            1,
            root_pid,
            2,
            vec![
                routing_child(0, first_leaf_pid, vec![0.5, 0.0]),
                routing_child(1, second_leaf_pid, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let first_input = quantized_assignment_input(10, 1, payload_format, &[-1.0, 0.0]);
        let second_input = quantized_assignment_input(10, 2, payload_format, &[1.0, 0.0]);
        let first_leaf_rows = vec![SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(1),
            heap_tid: first_input.heap_tid,
            payload_format: first_input.payload_format,
            gamma: first_input.gamma,
            encoded_payload: first_input.encoded_payload,
        }];
        let second_leaf_rows = vec![SpireLeafAssignmentRow {
            flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: SpireVecId::local(2),
            heap_tid: second_input.heap_tid,
            payload_format: second_input.payload_format,
            gamma: second_input.gamma,
            encoded_payload: second_input.encoded_payload,
        }];
        let placements = vec![
            object_store.insert_routing_object(7, &root).unwrap(),
            object_store.insert_routing_object(7, &internal).unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    first_leaf_pid,
                    1,
                    internal_pid,
                    &first_leaf_rows,
                )
                .unwrap(),
            object_store
                .insert_leaf_object_v2_from_rows(
                    7,
                    second_leaf_pid,
                    1,
                    internal_pid,
                    &second_leaf_rows,
                )
                .unwrap(),
        ];
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
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let observed = collect_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            1,
            payload_format,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(1),
        )
        .unwrap();

        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].pid, second_leaf_pid);
        assert_eq!(observed[0].heap_tid, tid(10, 2));
        assert_eq!(observed[0].vec_id.local_sequence(), Some(2));
    }

    #[test]
    fn group_leaf_routes_by_local_store_orders_stores_and_preserves_route_order() {
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let placements = vec![
            SpirePlacementEntry::local_store_available_by_id(
                7,
                SPIRE_FIRST_PID + 3,
                1,
                501,
                1,
                tid(60, 3),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                SPIRE_FIRST_PID + 1,
                0,
                500,
                1,
                tid(60, 1),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                SPIRE_FIRST_PID + 2,
                1,
                501,
                1,
                tid(60, 2),
                100,
            ),
        ];
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);
        let snapshot = SpireValidatedEpochSnapshot::from_snapshot(snapshot).unwrap();

        let groups = group_leaf_routes_by_local_store(
            &snapshot,
            vec![
                SpireRecursiveLeafRoute {
                    leaf_pid: SPIRE_FIRST_PID + 3,
                    parent_pid: SPIRE_FIRST_PID,
                },
                SpireRecursiveLeafRoute {
                    leaf_pid: SPIRE_FIRST_PID + 1,
                    parent_pid: SPIRE_FIRST_PID,
                },
                SpireRecursiveLeafRoute {
                    leaf_pid: SPIRE_FIRST_PID + 2,
                    parent_pid: SPIRE_FIRST_PID,
                },
            ],
        )
        .unwrap();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].node_id, 0);
        assert_eq!(groups[0].local_store_id, 0);
        assert_eq!(
            groups[0]
                .routes
                .iter()
                .map(|route| route.leaf_pid)
                .collect::<Vec<_>>(),
            vec![SPIRE_FIRST_PID + 1]
        );
        assert_eq!(groups[1].node_id, 0);
        assert_eq!(groups[1].local_store_id, 1);
        assert_eq!(
            groups[1]
                .routes
                .iter()
                .map(|route| route.leaf_pid)
                .collect::<Vec<_>>(),
            vec![SPIRE_FIRST_PID + 3, SPIRE_FIRST_PID + 2]
        );
    }

    #[test]
    fn group_leaf_and_delta_reads_by_local_store_groups_deltas_by_own_store() {
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let selected_leaf_pid = SPIRE_FIRST_PID + 1;
        let other_leaf_pid = SPIRE_FIRST_PID + 2;
        let selected_delta_pid = SPIRE_FIRST_PID + 11;
        let other_delta_pid = SPIRE_FIRST_PID + 12;
        let placements = vec![
            SpirePlacementEntry::local_store_available_by_id(
                7,
                selected_leaf_pid,
                0,
                500,
                1,
                tid(60, 1),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                other_leaf_pid,
                1,
                501,
                1,
                tid(60, 2),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                selected_delta_pid,
                2,
                502,
                1,
                tid(60, 3),
                100,
            ),
            SpirePlacementEntry::local_store_available_by_id(
                7,
                other_delta_pid,
                1,
                501,
                1,
                tid(60, 4),
                100,
            ),
        ];
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot =
            snapshot_for_placement(&epoch_manifest, &object_manifest, &placement_directory);
        let snapshot = SpireValidatedEpochSnapshot::from_snapshot(snapshot).unwrap();

        let groups = group_leaf_and_delta_reads_by_local_store(
            &snapshot,
            vec![SpireRecursiveLeafRoute {
                leaf_pid: selected_leaf_pid,
                parent_pid: SPIRE_FIRST_PID,
            }],
            vec![
                SpireDeltaObjectRoute {
                    delta_pid: selected_delta_pid,
                    parent_leaf_pid: selected_leaf_pid,
                },
                SpireDeltaObjectRoute {
                    delta_pid: other_delta_pid,
                    parent_leaf_pid: other_leaf_pid,
                },
            ],
        )
        .unwrap();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].local_store_id, 0);
        assert_eq!(groups[0].leaf_routes.len(), 1);
        assert_eq!(groups[0].leaf_routes[0].leaf_pid, selected_leaf_pid);
        assert!(groups[0].delta_routes.is_empty());
        assert_eq!(groups[1].local_store_id, 2);
        assert!(groups[1].leaf_routes.is_empty());
        assert_eq!(groups[1].delta_routes.len(), 1);
        assert_eq!(groups[1].delta_routes[0].delta_pid, selected_delta_pid);
        assert_eq!(groups[1].delta_routes[0].parent_leaf_pid, selected_leaf_pid);
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
                Ok(Some(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 1.0,
                    2 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                }))
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
                Ok(Some(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 1.0,
                    2 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                }))
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
            recursive_fanout: 0,
            local_store_count: 1,
            nprobe: 2,
            rerank_width: 2,
            training_sample_rows: 0,
            seed: 0,
            pq_group_size: 0,
            storage_format: SpireStorageFormat::TurboQuant,
            local_store_tablespaces: None,
        };
        let query = SpireScanQuery::new(vec![1.0, 0.0]).unwrap();

        let prepared = prepare_single_level_snapshot_scan_candidates(
            &snapshot,
            &object_store,
            &query,
            options,
            |candidate| {
                Ok(Some(match candidate.vec_id.local_sequence().unwrap() {
                    1 => 1.0,
                    2 => 10.0,
                    other => panic!("unexpected rerank candidate {other}"),
                }))
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
    fn route_routing_object_to_child_pids_routes_internal_level() {
        let internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 1, vec![0.0, 1.0]),
                routing_child(1, SPIRE_FIRST_PID + 2, vec![1.0, 0.0]),
                routing_child(2, SPIRE_FIRST_PID + 3, vec![0.5, 0.0]),
            ],
        )
        .unwrap();

        assert_eq!(
            route_routing_object_to_child_pids(&internal, &[1.0, 0.0], 2).unwrap(),
            vec![SPIRE_FIRST_PID + 2, SPIRE_FIRST_PID + 3]
        );
    }

    #[test]
    fn route_root_object_to_leaf_pids_still_rejects_internal_parent() {
        let internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0])],
        )
        .unwrap();

        let error = route_root_object_to_leaf_pids(&internal, &[1.0, 0.0], 1).unwrap_err();

        assert!(error.contains("root routing object"));
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_descends_to_leaf_level() {
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let internal_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let internal_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 22, vec![-0.5, 0.0]),
            ],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::from([
            (internal_a.header.pid, internal_a),
            (internal_b.header.pid, internal_b),
        ]);

        assert_eq!(
            route_recursive_routing_objects_to_leaf_pids(
                &root,
                &routing_objects_by_pid,
                &[1.0, 0.0],
                1
            )
            .unwrap(),
            vec![SPIRE_FIRST_PID + 12]
        );
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_rejects_missing_internal_child() {
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0])],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::new();

        let error = route_recursive_routing_objects_to_leaf_pids(
            &root,
            &routing_objects_by_pid,
            &[1.0, 0.0],
            1,
        )
        .unwrap_err();

        assert!(error.contains("missing internal routing child"));
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_rejects_wrong_child_level() {
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0])],
        )
        .unwrap();
        let wrong_level_child = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            2,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 11, vec![1.0, 0.0])],
        )
        .unwrap();
        let routing_objects_by_pid =
            HashMap::from([(wrong_level_child.header.pid, wrong_level_child)]);

        let error = route_recursive_routing_objects_to_leaf_pids(
            &root,
            &routing_objects_by_pid,
            &[1.0, 0.0],
            1,
        )
        .unwrap_err();

        assert!(error.contains("is not one below parent level"));
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_uses_conservative_upper_level_nprobe() {
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let internal_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let internal_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 21, vec![-0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 22, vec![-1.5, 0.0]),
            ],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::from([
            (internal_a.header.pid, internal_a),
            (internal_b.header.pid, internal_b),
        ]);

        let leaf_pids = route_recursive_routing_objects_to_leaf_pids(
            &root,
            &routing_objects_by_pid,
            &[1.0, 0.0],
            2,
        )
        .unwrap();

        assert_eq!(leaf_pids, vec![SPIRE_FIRST_PID + 12, SPIRE_FIRST_PID + 11]);
    }

    #[test]
    fn route_recursive_routing_objects_to_leaf_pids_descends_three_levels_conservatively() {
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            3,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 100, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 200, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let level_2_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 100,
            1,
            2,
            SPIRE_FIRST_PID,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 110, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 120, vec![0.4, 0.0]),
            ],
        )
        .unwrap();
        let level_2_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 200,
            1,
            2,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 210, vec![-0.5, 0.0])],
        )
        .unwrap();
        let level_1_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 110,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 111, vec![2.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 112, vec![1.0, 0.0]),
            ],
        )
        .unwrap();
        let level_1_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 120,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 121, vec![3.0, 0.0])],
        )
        .unwrap();
        let level_1_c = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 210,
            1,
            1,
            SPIRE_FIRST_PID + 200,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 211, vec![-2.0, 0.0])],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::from([
            (level_2_a.header.pid, level_2_a),
            (level_2_b.header.pid, level_2_b),
            (level_1_a.header.pid, level_1_a),
            (level_1_b.header.pid, level_1_b),
            (level_1_c.header.pid, level_1_c),
        ]);

        let leaf_pids = route_recursive_routing_objects_to_leaf_pids(
            &root,
            &routing_objects_by_pid,
            &[1.0, 0.0],
            2,
        )
        .unwrap();

        assert_eq!(
            leaf_pids,
            vec![SPIRE_FIRST_PID + 111, SPIRE_FIRST_PID + 112]
        );
    }

    #[test]
    fn load_snapshot_routing_hierarchy_loads_root_and_internal_objects() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 1, vec![1.0, 0.0])],
        )
        .unwrap();
        let internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 1,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 2, vec![1.0, 0.0])],
        )
        .unwrap();
        let root_placement = object_store.insert_routing_object(7, &root).unwrap();
        let internal_placement = object_store.insert_routing_object(7, &internal).unwrap();
        let leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 2, 1, SPIRE_FIRST_PID + 1, &[])
            .unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let placements = vec![root_placement, internal_placement, leaf_placement];
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let hierarchy = load_snapshot_routing_hierarchy(&snapshot, &object_store)
            .expect("routing hierarchy should load");

        assert_eq!(hierarchy.root_pid, SPIRE_FIRST_PID);
        assert_eq!(hierarchy.root_object.header.level, 2);
        assert_eq!(hierarchy.internal_objects_by_pid.len(), 1);
        assert_eq!(
            hierarchy
                .internal_objects_by_pid
                .get(&(SPIRE_FIRST_PID + 1))
                .unwrap()
                .header
                .parent_pid,
            SPIRE_FIRST_PID
        );
    }

    #[test]
    fn recursive_routed_leaf_rows_skip_degraded_unavailable_unselected_internal() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let available_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 11, vec![1.0, 0.0])],
        )
        .unwrap();
        let unavailable_internal = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.0, 0.0])],
        )
        .unwrap();
        let root_placement = object_store.insert_routing_object(7, &root).unwrap();
        let available_internal_placement = object_store
            .insert_routing_object(7, &available_internal)
            .unwrap();
        let mut unavailable_internal_placement = object_store
            .insert_routing_object(7, &unavailable_internal)
            .unwrap();
        unavailable_internal_placement.state = SpirePlacementState::Unavailable;
        let available_leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 11, 1, SPIRE_FIRST_PID + 10, &[])
            .unwrap();
        let unavailable_leaf_placement = object_store
            .insert_leaf_object_v2_from_rows(7, SPIRE_FIRST_PID + 21, 1, SPIRE_FIRST_PID + 20, &[])
            .unwrap();
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Degraded,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let placements = vec![
            root_placement,
            available_internal_placement,
            unavailable_internal_placement,
            available_leaf_placement,
            unavailable_leaf_placement,
        ];
        let object_manifest = SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let routed =
            collect_snapshot_routed_probe_leaf_rows(&snapshot, &object_store, &[1.0, 0.0], 1)
                .expect("degraded recursive route should skip unselected unavailable internal");

        assert_eq!(routed.len(), 1);
        assert_eq!(routed[0].root_pid, SPIRE_FIRST_PID);
        assert_eq!(routed[0].leaf_pid, SPIRE_FIRST_PID + 11);
    }

    #[test]
    fn load_snapshot_routing_hierarchy_rejects_multiple_roots() {
        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let first_root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0])],
        )
        .unwrap();
        let second_root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID + 1,
            1,
            2,
            vec![routing_child(0, SPIRE_FIRST_PID + 11, vec![-1.0, 0.0])],
        )
        .unwrap();
        let placements = vec![
            object_store.insert_routing_object(7, &first_root).unwrap(),
            object_store.insert_routing_object(7, &second_root).unwrap(),
        ];
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
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory = SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let snapshot = SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let error = load_snapshot_routing_hierarchy(&snapshot, &object_store).unwrap_err();

        assert!(error.contains("multiple root routing objects"));
    }

    #[test]
    fn recursive_route_matches_flat_single_level_on_small_hierarchy() {
        let flat_root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![1.5, 0.0]),
                routing_child(2, SPIRE_FIRST_PID + 21, vec![-1.5, 0.0]),
                routing_child(3, SPIRE_FIRST_PID + 22, vec![-0.5, 0.0]),
            ],
        )
        .unwrap();
        let recursive_root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID + 100,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let internal_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let internal_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            SPIRE_FIRST_PID + 100,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 22, vec![-0.5, 0.0]),
            ],
        )
        .unwrap();
        let routing_objects_by_pid = HashMap::from([
            (internal_a.header.pid, internal_a),
            (internal_b.header.pid, internal_b),
        ]);

        let query = [1.0, 0.0];
        let flat_best = route_root_object_to_leaf_pids(&flat_root, &query, 1).unwrap();
        let recursive_best = route_recursive_routing_objects_to_leaf_pids(
            &recursive_root,
            &routing_objects_by_pid,
            &query,
            1,
        )
        .unwrap();

        assert_eq!(flat_best, vec![SPIRE_FIRST_PID + 12]);
        assert_eq!(recursive_best, flat_best);
    }

    #[test]
    fn recursive_quantized_candidates_match_flat_single_level_on_small_hierarchy() {
        let payload_format = SpireAssignmentPayloadFormat::TurboQuant;
        let leaf_specs = [
            (SPIRE_FIRST_PID + 11, 1, tid(10, 1), [0.5, 0.0]),
            (SPIRE_FIRST_PID + 12, 2, tid(10, 2), [1.5, 0.0]),
            (SPIRE_FIRST_PID + 21, 3, tid(10, 3), [-1.5, 0.0]),
            (SPIRE_FIRST_PID + 22, 4, tid(10, 4), [-0.5, 0.0]),
        ];
        let mut flat_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let flat_root = SpireRoutingPartitionObject::root(
            SPIRE_FIRST_PID,
            1,
            2,
            leaf_specs
                .iter()
                .enumerate()
                .map(|(centroid_index, (pid, _, _, centroid))| {
                    routing_child(
                        u32::try_from(centroid_index).unwrap(),
                        *pid,
                        centroid.to_vec(),
                    )
                })
                .collect(),
        )
        .unwrap();
        let mut flat_placements = vec![flat_store.insert_routing_object(7, &flat_root).unwrap()];
        for (pid, vec_seq, heap_tid, source_vector) in &leaf_specs {
            let input = encode_assignment_input(payload_format, *heap_tid, source_vector).unwrap();
            let rows = vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(*vec_seq),
                heap_tid: *heap_tid,
                payload_format: input.payload_format,
                gamma: input.gamma,
                encoded_payload: input.encoded_payload,
            }];
            flat_placements.push(
                flat_store
                    .insert_leaf_object_v2_from_rows(7, *pid, 1, flat_root.header.pid, &rows)
                    .unwrap(),
            );
        }
        let epoch_manifest = SpireEpochManifest {
            epoch: 7,
            state: SpireEpochState::Published,
            consistency_mode: SpireConsistencyMode::Strict,
            published_at_micros: 1000,
            retain_until_micros: 2000,
            active_query_count: 0,
        };
        let flat_object_manifest = SpireObjectManifest::from_entries(
            7,
            flat_placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let flat_placement_directory =
            SpirePlacementDirectory::from_entries(7, flat_placements).unwrap();
        let flat_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &flat_object_manifest,
            &flat_placement_directory,
        )
        .unwrap();

        let mut recursive_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let recursive_root = SpireRoutingPartitionObject::root_at_level(
            SPIRE_FIRST_PID + 100,
            1,
            2,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 10, vec![1.0, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 20, vec![-1.0, 0.0]),
            ],
        )
        .unwrap();
        let internal_a = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 10,
            1,
            1,
            recursive_root.header.pid,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 11, vec![0.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 12, vec![1.5, 0.0]),
            ],
        )
        .unwrap();
        let internal_b = SpireRoutingPartitionObject::internal(
            SPIRE_FIRST_PID + 20,
            1,
            1,
            recursive_root.header.pid,
            2,
            vec![
                routing_child(0, SPIRE_FIRST_PID + 21, vec![-1.5, 0.0]),
                routing_child(1, SPIRE_FIRST_PID + 22, vec![-0.5, 0.0]),
            ],
        )
        .unwrap();
        let mut recursive_placements = vec![
            recursive_store
                .insert_routing_object(7, &recursive_root)
                .unwrap(),
            recursive_store
                .insert_routing_object(7, &internal_a)
                .unwrap(),
            recursive_store
                .insert_routing_object(7, &internal_b)
                .unwrap(),
        ];
        for (pid, vec_seq, heap_tid, source_vector) in &leaf_specs {
            let input = encode_assignment_input(payload_format, *heap_tid, source_vector).unwrap();
            let rows = vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(*vec_seq),
                heap_tid: *heap_tid,
                payload_format: input.payload_format,
                gamma: input.gamma,
                encoded_payload: input.encoded_payload,
            }];
            let parent_pid = if *pid < SPIRE_FIRST_PID + 20 {
                internal_a.header.pid
            } else {
                internal_b.header.pid
            };
            recursive_placements.push(
                recursive_store
                    .insert_leaf_object_v2_from_rows(7, *pid, 1, parent_pid, &rows)
                    .unwrap(),
            );
        }
        let recursive_object_manifest = SpireObjectManifest::from_entries(
            7,
            recursive_placements
                .iter()
                .map(manifest_entry_for)
                .collect(),
        )
        .unwrap();
        let recursive_placement_directory =
            SpirePlacementDirectory::from_entries(7, recursive_placements).unwrap();
        let recursive_snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &recursive_object_manifest,
            &recursive_placement_directory,
        )
        .unwrap();

        let flat_candidates = collect_quantized_routed_probe_candidates(
            &flat_snapshot,
            &flat_store,
            &[1.0, 0.0],
            1,
            payload_format,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(1),
        )
        .unwrap();
        let recursive_candidates = collect_quantized_routed_probe_candidates(
            &recursive_snapshot,
            &recursive_store,
            &[1.0, 0.0],
            1,
            payload_format,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(1),
        )
        .unwrap();

        assert_eq!(flat_candidates.len(), 1);
        assert_eq!(recursive_candidates, flat_candidates);
        assert_eq!(recursive_candidates[0].pid, SPIRE_FIRST_PID + 12);
        assert_eq!(recursive_candidates[0].heap_tid, tid(10, 2));
    }

    #[test]
    fn materialized_recursive_routing_epoch_scans_quantized_candidates() {
        let payload_format = SpireAssignmentPayloadFormat::TurboQuant;
        let leaf_specs = [
            (SPIRE_FIRST_PID + 20, 1, tid(10, 1), [0.5, 0.0]),
            (SPIRE_FIRST_PID + 21, 2, tid(10, 2), [1.5, 0.0]),
            (SPIRE_FIRST_PID + 22, 3, tid(10, 3), [-1.5, 0.0]),
            (SPIRE_FIRST_PID + 23, 4, tid(10, 4), [-0.5, 0.0]),
        ];
        let mut pid_allocator = SpirePidAllocator::default();
        let routing_draft = build_recursive_routing_hierarchy_draft(
            SpireRecursiveRoutingBuildInput {
                object_version: 1,
                dimensions: 2,
                target_fanout: 2,
                seed: 42,
                children: leaf_specs
                    .iter()
                    .map(|(pid, _, _, centroid)| SpireRecursiveRoutingChildInput {
                        child_pid: *pid,
                        child_level: 0,
                        centroid: centroid.to_vec(),
                        source_count: 1,
                    })
                    .collect(),
            },
            &mut pid_allocator,
        )
        .unwrap();
        let mut leaf_parent_pids = HashMap::new();
        for object in routing_draft
            .routing_objects
            .iter()
            .filter(|object| object.header.level == 1)
        {
            for child in object.children() {
                leaf_parent_pids.insert(child.child_pid, object.header.pid);
            }
        }

        let mut object_store = SpireLocalObjectStore::with_default_page_size(12345).unwrap();
        let mut leaf_placements = Vec::new();
        for (pid, vec_seq, heap_tid, source_vector) in &leaf_specs {
            let input = encode_assignment_input(payload_format, *heap_tid, source_vector).unwrap();
            let rows = vec![SpireLeafAssignmentRow {
                flags: SPIRE_ASSIGNMENT_FLAG_PRIMARY,
                vec_id: SpireVecId::local(*vec_seq),
                heap_tid: *heap_tid,
                payload_format: input.payload_format,
                gamma: input.gamma,
                encoded_payload: input.encoded_payload,
            }];
            leaf_placements.push(
                object_store
                    .insert_leaf_object_v2_from_rows(
                        7,
                        *pid,
                        1,
                        *leaf_parent_pids.get(pid).unwrap(),
                        &rows,
                    )
                    .unwrap(),
            );
        }
        let epoch_draft = build_local_recursive_routing_epoch_draft(
            SpireRecursiveRoutingEpochInput {
                epoch: 7,
                published_at_micros: 1000,
                retain_until_micros: 2000,
                consistency_mode: SpireConsistencyMode::Strict,
                routing_draft,
                leaf_placements,
            },
            &mut object_store,
        )
        .unwrap();
        let snapshot = SpirePublishedEpochSnapshot::new(
            &epoch_draft.epoch_manifest,
            &epoch_draft.object_manifest,
            &epoch_draft.placement_directory,
        )
        .unwrap();

        let candidates = collect_quantized_routed_probe_candidates(
            &snapshot,
            &object_store,
            &[1.0, 0.0],
            2,
            payload_format,
            SpireCandidateDedupeMode::NoReplicaDedupeDisabled,
            Some(2),
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].pid, SPIRE_FIRST_PID + 21);
        assert_eq!(candidates[0].heap_tid, tid(10, 2));
        assert_eq!(candidates[1].pid, SPIRE_FIRST_PID + 20);
        assert_eq!(candidates[1].heap_tid, tid(10, 1));
    }

    #[test]
    fn rerank_scored_candidates_by_ip_rescores_prefix_and_truncates() {
        let mut candidates = vec![
            scored_candidate(1, 10, 1, -5.0),
            scored_candidate(2, 10, 2, -4.0),
            scored_candidate(3, 10, 3, -3.0),
        ];

        rerank_scored_candidates_by_ip(&mut candidates, 2, |candidate| {
            Ok(Some(match candidate.vec_id.local_sequence().unwrap() {
                1 => 1.0,
                2 => 10.0,
                other => panic!("unexpected rerank candidate {other}"),
            }))
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
            Ok(Some(candidate.heap_tid.offset_number as f32))
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
    fn rerank_scored_candidates_by_ip_drops_invisible_candidates() {
        let mut candidates = vec![
            scored_candidate(1, 10, 1, -5.0),
            scored_candidate(2, 10, 2, -4.0),
            scored_candidate(3, 10, 3, -3.0),
        ];

        rerank_scored_candidates_by_ip(&mut candidates, 0, |candidate| {
            if candidate.vec_id.local_sequence() == Some(2) {
                Ok(None)
            } else {
                Ok(Some(candidate.heap_tid.offset_number as f32))
            }
        })
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(3));
        assert_eq!(candidates[0].score, -3.0);
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(1));
        assert_eq!(candidates[1].score, -1.0);
    }

    #[test]
    fn rerank_scored_candidates_by_ip_rejects_non_finite_scores() {
        let mut candidates = vec![scored_candidate(1, 10, 1, -5.0)];

        assert!(
            rerank_scored_candidates_by_ip(&mut candidates, 0, |_| Ok(Some(f32::INFINITY)))
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
    fn scan_opaque_refreshes_root_control_on_every_rescan_observation() {
        let mut opaque = SpireScanOpaque::default();
        let epoch_one =
            SpireRootControlState::published(1, 4, 3, tid(10, 1), tid(10, 2), tid(10, 3)).unwrap();
        let same_epoch_newer_cursors =
            SpireRootControlState::published(1, 5, 4, tid(20, 1), tid(20, 2), tid(20, 3)).unwrap();
        let epoch_two =
            SpireRootControlState::published(2, 5, 4, tid(20, 1), tid(20, 2), tid(20, 3)).unwrap();

        assert_eq!(opaque.root_control, None);
        assert_eq!(opaque.observe_root_control_for_rescan(epoch_one), epoch_one);
        assert_eq!(opaque.root_control, Some(epoch_one));
        assert_eq!(
            opaque.observe_root_control_for_rescan(same_epoch_newer_cursors),
            same_epoch_newer_cursors
        );
        assert_eq!(opaque.root_control, Some(same_epoch_newer_cursors));
        assert_eq!(opaque.observe_root_control_for_rescan(epoch_two), epoch_two);
        assert_eq!(opaque.root_control, Some(epoch_two));
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
