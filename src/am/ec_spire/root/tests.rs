#[cfg(test)]
mod tests {
    use super::*;

    fn tid(block_number: u32, offset_number: u16) -> crate::storage::page::ItemPointer {
        crate::storage::page::ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn published_epoch_manifest(epoch: u64) -> meta::SpireEpochManifest {
        meta::SpireEpochManifest {
            epoch,
            state: meta::SpireEpochState::Published,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 1,
            active_query_count: 0,
        }
    }

    fn retired_epoch_manifest(epoch: u64) -> meta::SpireEpochManifest {
        meta::SpireEpochManifest {
            epoch,
            state: meta::SpireEpochState::Retired,
            consistency_mode: meta::SpireConsistencyMode::Strict,
            published_at_micros: 1,
            retain_until_micros: 1,
            active_query_count: 0,
        }
    }

    fn manifest_entry_for(placement: &meta::SpirePlacementEntry) -> meta::SpireManifestEntry {
        meta::SpireManifestEntry {
            epoch: placement.epoch,
            pid: placement.pid,
            object_version: placement.object_version,
            placement_tid: placement.object_tid,
        }
    }

    fn empty_leaf_row(
        store: &mut storage::SpireLocalObjectStore,
        pid: u64,
        parent_pid: u64,
    ) -> meta::SpirePlacementEntry {
        store
            .insert_leaf_object_v2_from_rows(1, pid, 1, parent_pid, &[])
            .expect("empty leaf object should store")
    }

    fn remote_local_vec_id(local_vec_seq: u64) -> Vec<u8> {
        storage::SpireVecId::local(local_vec_seq).as_bytes().to_vec()
    }

    fn remote_global_vec_id(payload: &[u8]) -> Vec<u8> {
        storage::SpireVecId::global(payload)
            .expect("global vec_id payload should be valid")
            .as_bytes()
            .to_vec()
    }

    fn remote_candidate(
        node_id: u32,
        pid: u64,
        row_index: u32,
        vec_id: impl AsRef<[u8]>,
        score: f32,
        assignment_flags: u16,
    ) -> SpireRemoteSearchCandidateRow {
        SpireRemoteSearchCandidateRow {
            served_epoch: 7,
            node_id,
            pid,
            object_version: 11,
            row_index,
            assignment_flags,
            vec_id: vec_id.as_ref().to_vec(),
            row_locator: vec![node_id as u8, row_index as u8],
            score,
        }
    }

    fn remote_heap_candidate(
        node_id: u32,
        pid: u64,
        row_index: u32,
        vec_id: impl AsRef<[u8]>,
        score: f32,
        heap_lookup_owner: &'static str,
    ) -> SpireRemoteSearchLocalHeapCandidateRow {
        SpireRemoteSearchLocalHeapCandidateRow {
            requested_epoch: 7,
            served_epoch: 7,
            node_id,
            pid,
            object_version: 11,
            row_index,
            assignment_flags: storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            vec_id: vec_id.as_ref().to_vec(),
            row_locator: vec![node_id as u8, row_index as u8],
            heap_block: 10 + node_id,
            heap_offset: 1 + row_index as u16,
            score,
            heap_lookup_owner,
            status: SPIRE_REMOTE_STATUS_READY,
        }
    }

    fn fanout_placement(
        pid: u64,
        node_id: u32,
        state: meta::SpirePlacementState,
    ) -> meta::SpirePlacementEntry {
        meta::SpirePlacementEntry {
            epoch: 7,
            pid,
            node_id,
            local_store_id: node_id,
            store_relid: 10_000 + node_id,
            object_version: 11,
            object_tid: tid(90, pid as u16),
            object_bytes: 4096,
            state,
        }
    }

    #[test]
    fn scan_sanity_status_reports_empty_approximate_and_full_scan() {
        assert_eq!(
            scan_sanity_status(0, false, false),
            (
                "empty",
                "none",
                "build or insert rows to publish the first SPIRE epoch"
            )
        );
        assert_eq!(
            scan_sanity_status(1, false, false),
            (
                "approximate_leaf_coverage",
                "bounded_leaf_probe",
                "increase nprobe to active_leaf_count for exact leaf coverage sanity checks"
            )
        );
        assert_eq!(
            scan_sanity_status(1, true, false),
            (
                "exact_leaf_coverage_bounded_rerank",
                "bounded_rerank",
                "set rerank_width = 0 and max_candidate_rows high enough for full-frontier recall sanity checks"
            )
        );
        assert_eq!(
            scan_sanity_status(1, true, true),
            (
                "exact_leaf_and_frontier_coverage",
                "full_scan",
                "use this configuration only when max_candidate_rows covers the expected frontier"
            )
        );
    }

    #[test]
    fn remote_candidate_merge_dedupes_vec_ids_and_keeps_best_ranked_row() {
        let same = remote_global_vec_id(b"same");
        let other = remote_global_vec_id(b"other");
        let boundary = remote_candidate(
            2,
            20,
            0,
            &same,
            1.0,
            storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
        );
        let primary = remote_candidate(1, 10, 0, &same, 1.0, storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY);
        let better_score =
            remote_candidate(3, 30, 0, &other, 0.5, storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY);

        let merged = merge_remote_search_candidates(
            vec![boundary, primary.clone(), better_score.clone()],
            None,
        )
        .expect("remote candidates should merge");

        assert_eq!(merged.input_count, 3);
        assert_eq!(merged.duplicate_vec_id_count, 1);
        assert_eq!(merged.candidates, vec![better_score, primary]);
    }

    #[test]
    fn remote_candidate_merge_applies_top_k_after_global_dedupe() {
        let a = remote_global_vec_id(b"a");
        let b = remote_global_vec_id(b"b");
        let c = remote_global_vec_id(b"c");
        let duplicate_worse =
            remote_candidate(1, 10, 0, &a, 3.0, storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY);
        let duplicate_best =
            remote_candidate(1, 11, 0, &a, 0.3, storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY);
        let second = remote_candidate(1, 12, 0, &b, 0.4, storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY);
        let truncated = remote_candidate(1, 13, 0, &c, 0.5, storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY);

        let merged = merge_remote_search_candidates(
            vec![
                duplicate_worse,
                truncated,
                second.clone(),
                duplicate_best.clone(),
            ],
            Some(2),
        )
        .expect("remote candidates should merge");

        assert_eq!(merged.input_count, 4);
        assert_eq!(merged.duplicate_vec_id_count, 1);
        assert_eq!(merged.candidates, vec![duplicate_best, second]);
    }

    #[test]
    fn remote_candidate_merge_rejects_invalid_candidate_envelope() {
        let nan_error = merge_remote_search_candidates(
            vec![remote_candidate(
                1,
                10,
                0,
                remote_global_vec_id(b"a"),
                f32::NAN,
                storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            )],
            None,
        )
        .expect_err("nan scores should fail");
        assert!(nan_error.contains("non-finite score"));

        let empty_vec_id_error =
            merge_remote_search_candidates(
                vec![remote_candidate(1, 10, 0, b"", 1.0, storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY)],
                None,
            )
            .expect_err("empty vec_id should fail");
        assert!(empty_vec_id_error.contains("invalid vec_id"));
    }

    #[test]
    fn remote_candidate_merge_scopes_local_vec_ids_by_node() {
        let local = remote_local_vec_id(7);
        let node_two = remote_candidate(
            2,
            10,
            0,
            &local,
            0.4,
            storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        );
        let node_three = remote_candidate(
            3,
            20,
            0,
            &local,
            0.3,
            storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        );

        let merged =
            merge_remote_search_candidates(vec![node_two.clone(), node_three.clone()], None)
                .expect("node-local vec_ids should merge without cross-node dedupe");

        assert_eq!(merged.duplicate_vec_id_count, 0);
        assert_eq!(merged.candidates, vec![node_three, node_two]);
    }

    #[test]
    fn remote_candidate_merge_dedupes_global_vec_ids_across_nodes() {
        let global = remote_global_vec_id(b"global-7");
        let local_best = remote_candidate(
            2,
            10,
            0,
            &global,
            0.2,
            storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        );
        let remote_replica = remote_candidate(
            3,
            20,
            0,
            &global,
            0.4,
            storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
        );

        let merged =
            merge_remote_search_candidates(vec![remote_replica, local_best.clone()], None)
                .expect("global vec_ids should dedupe across nodes");

        assert_eq!(merged.duplicate_vec_id_count, 1);
        assert_eq!(merged.candidates, vec![local_best]);
    }

    #[test]
    fn remote_candidate_dedupe_key_prefixes_do_not_overlap_vec_id_discriminators() {
        assert_eq!(SPIRE_REMOTE_VEC_ID_KEY_GLOBAL, 0xA0);
        assert_eq!(SPIRE_REMOTE_VEC_ID_KEY_NODE_LOCAL, 0xA1);
        assert_ne!(
            SPIRE_REMOTE_VEC_ID_KEY_GLOBAL,
            storage::SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR
        );
        assert_ne!(
            SPIRE_REMOTE_VEC_ID_KEY_NODE_LOCAL,
            storage::SPIRE_LOCAL_VEC_ID_DISCRIMINATOR
        );
    }

    #[test]
    fn remote_heap_candidate_result_merge_scopes_local_vec_ids_by_node() {
        let local = remote_local_vec_id(7);
        let node_two = remote_heap_candidate(
            2,
            10,
            0,
            &local,
            0.4,
            SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
        );
        let node_three = remote_heap_candidate(
            3,
            20,
            0,
            &local,
            0.3,
            SPIRE_REMOTE_HEAP_RESOLUTION,
        );

        let merged = merge_remote_search_heap_candidates_for_result(
            vec![node_two.clone(), node_three.clone()],
            10,
        )
        .expect("node-local heap candidates should merge without cross-node dedupe");

        assert_eq!(merged, vec![node_three, node_two]);
    }

    #[test]
    fn remote_heap_candidate_result_merge_dedupes_global_vec_ids_across_nodes() {
        let global = remote_global_vec_id(b"global-heap");
        let local_best = remote_heap_candidate(
            2,
            10,
            0,
            &global,
            0.2,
            SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
        );
        let remote_replica =
            remote_heap_candidate(3, 20, 0, &global, 0.4, SPIRE_REMOTE_HEAP_RESOLUTION);

        let merged = merge_remote_search_heap_candidates_for_result(
            vec![remote_replica, local_best.clone()],
            10,
        )
        .expect("global heap candidates should dedupe across nodes");

        assert_eq!(merged, vec![local_best]);
    }

    #[test]
    fn remote_candidate_batch_validation_accepts_expected_envelope() {
        let candidates = vec![
            remote_candidate(
                2,
                10,
                0,
                remote_local_vec_id(1),
                0.5,
                storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            ),
            remote_candidate(
                2,
                11,
                0,
                remote_local_vec_id(2),
                0.6,
                storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
            ),
        ];

        validate_remote_search_candidate_batch(7, 2, &[10, 11], &candidates)
            .expect("candidate batch should validate");
    }

    #[test]
    fn remote_candidate_batch_validation_accepts_leaf_derived_delta_rows() {
        let candidates = vec![
            remote_candidate(
                2,
                10,
                0,
                remote_local_vec_id(1),
                0.5,
                storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
            ),
            remote_candidate(
                2,
                12,
                0,
                remote_local_vec_id(2),
                0.6,
                storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY
                    | storage::SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT,
            ),
        ];

        validate_remote_search_candidate_batch(7, 2, &[10], &candidates)
            .expect("leaf-derived delta candidate PID should validate");
    }

    #[test]
    fn remote_candidate_batch_validation_rejects_receive_contract_drift() {
        let wrong_epoch = remote_candidate(
            2,
            10,
            0,
            remote_local_vec_id(1),
            0.5,
            storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        );
        let mut wrong_epoch = SpireRemoteSearchCandidateRow {
            served_epoch: 8,
            ..wrong_epoch
        };
        let error = validate_remote_search_candidate_batch(7, 2, &[10], &[wrong_epoch.clone()])
            .expect_err("wrong served epoch should fail");
        assert!(error.contains("served epoch 8"));

        wrong_epoch.served_epoch = 7;
        wrong_epoch.node_id = 3;
        let error = validate_remote_search_candidate_batch(7, 2, &[10], &[wrong_epoch.clone()])
            .expect_err("wrong node_id should fail");
        assert!(error.contains("does not match expected node_id"));

        wrong_epoch.node_id = 2;
        wrong_epoch.pid = 12;
        let error = validate_remote_search_candidate_batch(7, 2, &[10], &[wrong_epoch.clone()])
            .expect_err("unselected pid should fail");
        assert!(error.contains("was not selected"));

        wrong_epoch.pid = 10;
        wrong_epoch.object_version = 0;
        let error = validate_remote_search_candidate_batch(7, 2, &[10], &[wrong_epoch.clone()])
            .expect_err("zero object version should fail");
        assert!(error.contains("object_version 0"));

        wrong_epoch.object_version = 11;
        wrong_epoch.assignment_flags = storage::SPIRE_ASSIGNMENT_FLAG_TOMBSTONE;
        let error = validate_remote_search_candidate_batch(7, 2, &[10], &[wrong_epoch.clone()])
            .expect_err("non-visible assignment flags should fail");
        assert!(error.contains("non-visible assignment_flags"));

        wrong_epoch.assignment_flags = storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY;
        wrong_epoch.row_locator.clear();
        let error = validate_remote_search_candidate_batch(7, 2, &[10], &[wrong_epoch])
            .expect_err("empty row locator should fail");
        assert!(error.contains("empty row_locator"));
    }

    #[test]
    fn remote_candidate_batch_merge_validates_batches_before_merge() {
        let first = remote_candidate(
            2,
            10,
            0,
            remote_local_vec_id(1),
            0.4,
            storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        );
        let dup = remote_global_vec_id(b"dup");
        let duplicate_best = remote_candidate(
            3,
            20,
            0,
            &dup,
            0.3,
            storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        );
        let duplicate_worse = remote_candidate(
            3,
            21,
            0,
            &dup,
            0.9,
            storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA,
        );
        let batches = vec![
            SpireRemoteSearchCandidateBatch {
                node_id: 2,
                selected_pids: vec![10],
                candidates: vec![first.clone()],
            },
            SpireRemoteSearchCandidateBatch {
                node_id: 3,
                selected_pids: vec![20, 21],
                candidates: vec![duplicate_worse, duplicate_best.clone()],
            },
        ];

        let merged = merge_validated_remote_search_candidate_batches(7, batches, Some(2))
            .expect("validated candidate batches should merge");

        assert_eq!(merged.input_count, 3);
        assert_eq!(merged.duplicate_vec_id_count, 1);
        assert_eq!(merged.candidates, vec![duplicate_best, first]);
    }

    #[test]
    fn remote_candidate_batch_merge_rejects_invalid_batch_before_merge() {
        let mut invalid = remote_candidate(
            2,
            10,
            0,
            remote_local_vec_id(1),
            0.4,
            storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        );
        invalid.node_id = 9;
        let batches = vec![SpireRemoteSearchCandidateBatch {
            node_id: 2,
            selected_pids: vec![10],
            candidates: vec![invalid],
        }];

        let error = merge_validated_remote_search_candidate_batches(7, batches, Some(1))
            .expect_err("invalid candidate batch should fail before merge");

        assert!(error.contains("does not match expected node_id"));
    }

    #[test]
    fn remote_local_heap_locator_decode_error_includes_candidate_context() {
        let candidate = remote_candidate(
            2,
            10,
            7,
            remote_local_vec_id(1),
            0.4,
            storage::SPIRE_ASSIGNMENT_FLAG_PRIMARY,
        );
        let error = decode_remote_search_local_heap_locator(&candidate, "unit test")
            .expect_err("short locator should fail to decode");

        assert!(error.contains("unit test"));
        assert!(error.contains("pid 10"));
        assert!(error.contains("row_index 7"));
        assert!(error.contains("vec_id 010100000000000000"));
    }

    #[test]
    fn remote_search_fanout_groups_selected_pids_by_local_and_remote_node() {
        let placements = vec![
            fanout_placement(
                11,
                meta::SPIRE_LOCAL_NODE_ID,
                meta::SpirePlacementState::Available,
            ),
            fanout_placement(12, 2, meta::SpirePlacementState::Available),
            fanout_placement(13, 2, meta::SpirePlacementState::Available),
            fanout_placement(14, 7, meta::SpirePlacementState::Available),
        ];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory =
            meta::SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let epoch_manifest = published_epoch_manifest(7);
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let plan = plan_remote_search_fanout(&snapshot, &[13, 11, 14, 12]).unwrap();

        assert_eq!(plan.requested_epoch, 7);
        assert_eq!(plan.local_selected_pids, vec![11]);
        assert_eq!(
            plan.remote_targets,
            vec![
                SpireRemoteSearchFanoutTarget {
                    node_id: 2,
                    selected_pids: vec![13, 12],
                },
                SpireRemoteSearchFanoutTarget {
                    node_id: 7,
                    selected_pids: vec![14],
                },
            ]
        );
        assert!(plan.skipped_placements.is_empty());
    }

    #[test]
    fn remote_search_fanout_records_degraded_skipped_placements() {
        let placements = vec![
            fanout_placement(11, 2, meta::SpirePlacementState::Available),
            fanout_placement(12, 2, meta::SpirePlacementState::Unavailable),
            fanout_placement(
                13,
                meta::SPIRE_LOCAL_NODE_ID,
                meta::SpirePlacementState::Skipped,
            ),
        ];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory =
            meta::SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let epoch_manifest = meta::SpireEpochManifest {
            consistency_mode: meta::SpireConsistencyMode::Degraded,
            ..published_epoch_manifest(7)
        };
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let plan = plan_remote_search_fanout(&snapshot, &[11, 12, 13]).unwrap();

        assert!(plan.local_selected_pids.is_empty());
        assert_eq!(
            plan.remote_targets,
            vec![SpireRemoteSearchFanoutTarget {
                node_id: 2,
                selected_pids: vec![11],
            }]
        );
        assert_eq!(
            plan.skipped_placements,
            vec![
                SpireRemoteSearchSkippedPlacement {
                    node_id: 2,
                    pid: 12,
                    state: "unavailable",
                },
                SpireRemoteSearchSkippedPlacement {
                    node_id: meta::SPIRE_LOCAL_NODE_ID,
                    pid: 13,
                    state: "skipped",
                },
            ]
        );
    }

    #[test]
    fn remote_degradation_policy_contract_matches_fanout_skip_decisions() {
        for row in remote_degradation_policy_contract_rows() {
            let consistency_mode = match row.consistency_mode {
                "strict" => meta::SpireConsistencyMode::Strict,
                "degraded" => meta::SpireConsistencyMode::Degraded,
                other => panic!("unexpected consistency mode in contract: {other}"),
            };
            let placement_state = match row.placement_state {
                "available" => meta::SpirePlacementState::Available,
                "stale" => meta::SpirePlacementState::Stale,
                "unavailable" => meta::SpirePlacementState::Unavailable,
                "skipped" => meta::SpirePlacementState::Skipped,
                other => panic!("unexpected placement state in contract: {other}"),
            };

            let search_action = match fanout_should_skip_placement(consistency_mode, placement_state)
            {
                Ok(false) => "dispatch",
                Ok(true) => "skip_and_report",
                Err(_) => "fail_closed",
            };

            assert_eq!(
                row.search_action, search_action,
                "degradation policy contract drift for {} {}",
                row.consistency_mode, row.placement_state
            );
        }
    }

    #[test]
    fn remote_summary_status_helper_preserves_precedence() {
        let mut rollup = SpireRemoteCountRollup {
            remote_count: 1,
            skipped_count: 1,
            ..Default::default()
        };
        assert_eq!(
            rollup.summary_status(0, SpireRemoteSummaryStatusMode::RequestPlan),
            SPIRE_REMOTE_STATUS_EMPTY_TOP_K
        );
        assert_eq!(
            rollup.summary_status(1, SpireRemoteSummaryStatusMode::RequestPlan),
            SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
        );

        rollup.remote_count = 0;
        assert_eq!(
            rollup.summary_status(1, SpireRemoteSummaryStatusMode::RequestPlan),
            SPIRE_REMOTE_STATUS_DEGRADED_READY
        );

        rollup.missing_descriptor_count = 1;
        rollup.transport_count = 1;
        assert_eq!(
            rollup.summary_status(1, SpireRemoteSummaryStatusMode::Readiness),
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
        );

        rollup.missing_descriptor_count = 0;
        assert_eq!(
            rollup.summary_status(1, SpireRemoteSummaryStatusMode::Readiness),
            SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
        );

        rollup.transport_count = 0;
        rollup.degraded_skipped_count = 1;
        assert_eq!(
            rollup.summary_status(1, SpireRemoteSummaryStatusMode::Execution),
            SPIRE_REMOTE_STATUS_DEGRADED_READY
        );
    }

    #[test]
    fn remote_search_fanout_rejects_duplicate_selected_pids() {
        let placements = vec![fanout_placement(
            11,
            meta::SPIRE_LOCAL_NODE_ID,
            meta::SpirePlacementState::Available,
        )];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            7,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .unwrap();
        let placement_directory =
            meta::SpirePlacementDirectory::from_entries(7, placements).unwrap();
        let epoch_manifest = published_epoch_manifest(7);
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .unwrap();

        let error = plan_remote_search_fanout(&snapshot, &[11, 11]).unwrap_err();

        assert!(error.contains("appears more than once"));
    }

    #[test]
    fn epoch_snapshot_partial_retired_residue_keeps_root_manifest_authoritative() {
        let active_tid = tid(10, 1);
        let retired_residue_tid = tid(10, 2);
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, active_tid, tid(10, 3), tid(10, 4))
                .expect("root/control should build");

        let rows = epoch_snapshot_rows_from_manifests(
            root_control,
            vec![
                (active_tid, published_epoch_manifest(7)),
                (retired_residue_tid, retired_epoch_manifest(7)),
            ],
            2,
        )
        .expect("epoch snapshot rows should build");

        assert_eq!(rows.len(), 2);
        let active_row = rows
            .iter()
            .find(|row| row.manifest_offset == active_tid.offset_number)
            .expect("active root row should exist");
        let retired_residue_row = rows
            .iter()
            .find(|row| row.manifest_offset == retired_residue_tid.offset_number)
            .expect("retired residue row should exist");

        assert_eq!(active_row.state, "published");
        assert!(active_row.is_active_root_manifest);
        assert!(!active_row.cleanup_eligible_now);
        assert_eq!(active_row.cleanup_blocked_reason, "active_root_manifest");
        assert_eq!(retired_residue_row.state, "retired");
        assert!(!retired_residue_row.is_active_root_manifest);
        assert!(!retired_residue_row.cleanup_eligible_now);
        assert_eq!(
            retired_residue_row.cleanup_blocked_reason,
            "retained_retired_epoch"
        );
    }

    #[test]
    fn epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative() {
        let active_tid = tid(10, 1);
        let retired_residue_tid = tid(10, 2);
        let bundle_residue_tid = tid(10, 3);
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, active_tid, tid(10, 4), tid(10, 5))
                .expect("root/control should build");

        let rows = epoch_snapshot_rows_from_manifests(
            root_control,
            vec![
                (active_tid, published_epoch_manifest(7)),
                (retired_residue_tid, retired_epoch_manifest(7)),
                (bundle_residue_tid, published_epoch_manifest(8)),
            ],
            2,
        )
        .expect("epoch snapshot rows should build");

        assert_eq!(rows.len(), 3);
        let active_row = rows
            .iter()
            .find(|row| row.manifest_offset == active_tid.offset_number)
            .expect("active root row should exist");
        let bundle_residue_row = rows
            .iter()
            .find(|row| row.epoch == 8)
            .expect("bundle residue row should exist");

        assert_eq!(active_row.epoch, 7);
        assert_eq!(active_row.state, "published");
        assert!(active_row.is_active_root_manifest);
        assert!(!active_row.cleanup_eligible_now);
        assert_eq!(active_row.cleanup_blocked_reason, "active_root_manifest");
        assert_eq!(bundle_residue_row.state, "published");
        assert!(!bundle_residue_row.is_active_root_manifest);
        assert!(!bundle_residue_row.cleanup_eligible_now);
        assert_eq!(
            bundle_residue_row.cleanup_blocked_reason,
            "state_not_cleanup_eligible"
        );
    }

    #[test]
    fn leaf_maintenance_thresholds_use_named_split_merge_policy() {
        assert_eq!(leaf_maintenance_thresholds(0, 0), (0, 0));
        assert_eq!(leaf_maintenance_thresholds(2, 3), (32, 0));
        assert_eq!(leaf_maintenance_thresholds(120, 3), (160, 10));
    }

    fn root_for_child(pid: u64, child_pid: u64) -> storage::SpireRoutingPartitionObject {
        storage::SpireRoutingPartitionObject::root(
            pid,
            1,
            2,
            vec![storage::SpireRoutingChildEntry {
                centroid_index: 0,
                child_pid,
                centroid: vec![1.0, 0.0],
            }],
        )
        .expect("root routing object should build")
    }

    fn hierarchy_summary(
        pid: u64,
        kind: storage::SpirePartitionObjectKind,
        level: u16,
        parent_pid: u64,
        child_pids: Vec<u64>,
    ) -> SpireHierarchyObjectSummary {
        SpireHierarchyObjectSummary {
            pid,
            kind,
            level,
            parent_pid,
            child_pids,
        }
    }

    #[test]
    fn recursive_hierarchy_shape_accepts_single_level_root_to_leaves() {
        let objects = vec![
            hierarchy_summary(
                1,
                storage::SpirePartitionObjectKind::Root,
                1,
                0,
                vec![11, 12],
            ),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                1,
                Vec::new(),
            ),
            hierarchy_summary(
                12,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                1,
                Vec::new(),
            ),
        ];

        let has_internal =
            validate_recursive_hierarchy_shape(&objects).expect("shape should validate");

        assert!(!has_internal);
    }

    #[test]
    fn recursive_hierarchy_shape_accepts_internal_level_between_root_and_leaves() {
        let objects = vec![
            hierarchy_summary(1, storage::SpirePartitionObjectKind::Root, 2, 0, vec![10]),
            hierarchy_summary(
                10,
                storage::SpirePartitionObjectKind::Internal,
                1,
                1,
                vec![11, 12],
            ),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                10,
                Vec::new(),
            ),
            hierarchy_summary(
                12,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                10,
                Vec::new(),
            ),
        ];

        let has_internal =
            validate_recursive_hierarchy_shape(&objects).expect("shape should validate");

        assert!(has_internal);
    }

    #[test]
    fn recursive_hierarchy_shape_rejects_level_skip_to_leaf() {
        let objects = vec![
            hierarchy_summary(1, storage::SpirePartitionObjectKind::Root, 2, 0, vec![11]),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                1,
                Vec::new(),
            ),
        ];

        let err = validate_recursive_hierarchy_shape(&objects).unwrap_err();

        assert!(err.contains("child pid 11 has kind Leaf level 0"));
    }

    #[test]
    fn recursive_hierarchy_shape_rejects_orphan_leaf_parent_link() {
        let objects = vec![
            hierarchy_summary(1, storage::SpirePartitionObjectKind::Root, 1, 0, vec![11]),
            hierarchy_summary(
                11,
                storage::SpirePartitionObjectKind::Leaf,
                0,
                99,
                Vec::new(),
            ),
        ];

        let err = validate_recursive_hierarchy_shape(&objects).unwrap_err();

        assert!(err.contains("parent_pid 99 does not match routing pid 1"));
    }

    fn maintenance_leaf_row(
        leaf_pid: u64,
        parent_pid: u64,
        effective_assignment_count: u64,
        split_recommended: bool,
        merge_recommended: bool,
    ) -> SpireIndexLeafSnapshotRow {
        SpireIndexLeafSnapshotRow {
            active_epoch: 7,
            leaf_pid,
            parent_pid,
            object_version: 1,
            node_id: meta::SPIRE_LOCAL_NODE_ID,
            local_store_id: meta::SPIRE_SINGLE_LOCAL_STORE_ID,
            placement_state: "available",
            base_assignment_count: effective_assignment_count,
            base_primary_assignment_count: effective_assignment_count,
            base_boundary_replica_assignment_count: 0,
            delta_object_count: 0,
            delta_insert_assignment_count: 0,
            delta_boundary_replica_insert_assignment_count: 0,
            delta_delete_assignment_count: 0,
            effective_assignment_count,
            effective_boundary_replica_assignment_count: 0,
            split_assignment_threshold: 32,
            merge_assignment_threshold: 1,
            split_recommended,
            merge_recommended,
            maintenance_action: "none",
            maintenance_reason: "test",
            leaf_object_bytes: 1,
            delta_object_bytes: 0,
        }
    }

    #[test]
    fn maintenance_plan_snapshot_reports_selected_split_plan() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 100, true, false),
        ];

        let snapshot =
            maintenance_plan_snapshot_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance plan should build");

        assert_eq!(snapshot.active_epoch, 7);
        assert_eq!(snapshot.planner_status, "planned");
        assert_eq!(snapshot.planned_action, "split");
        assert_eq!(snapshot.planned_reason, "largest_split_candidate");
        assert_eq!(snapshot.replaced_parent_pid, 1);
        assert_eq!(snapshot.affected_leaf_pids, vec![12]);
        assert_eq!(snapshot.replacement_leaf_count, 2);
        assert_eq!(snapshot.replacement_leaf_pids, vec![40, 41]);
        assert_eq!(snapshot.publish_epoch, 8);
        assert_eq!(snapshot.next_pid, 42);
        assert_eq!(snapshot.next_local_vec_seq, 100);
    }

    #[test]
    fn maintenance_plan_snapshot_reports_no_action_without_candidate() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 11, false, false),
        ];

        let snapshot =
            maintenance_plan_snapshot_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance plan should build");

        assert_eq!(snapshot.active_epoch, 7);
        assert_eq!(snapshot.planner_status, "no_action");
        assert_eq!(snapshot.planned_action, "none");
        assert_eq!(snapshot.planned_reason, "no_candidate");
        assert_eq!(snapshot.replacement_leaf_pids, Vec::<u64>::new());
        assert_eq!(snapshot.next_pid, 40);
        assert_eq!(snapshot.next_local_vec_seq, 100);
    }

    #[test]
    fn maintenance_plan_snapshot_reports_selected_merge_plan() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 3, false, true),
            maintenance_leaf_row(12, 1, 1, false, true),
            maintenance_leaf_row(13, 2, 20, false, false),
        ];

        let snapshot =
            maintenance_plan_snapshot_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance plan should build");

        assert_eq!(snapshot.active_epoch, 7);
        assert_eq!(snapshot.planner_status, "planned");
        assert_eq!(snapshot.planned_action, "merge");
        assert_eq!(snapshot.planned_reason, "sparsest_same_parent_merge_pair");
        assert_eq!(snapshot.replaced_parent_pid, 1);
        assert_eq!(snapshot.affected_leaf_pids, vec![11, 12]);
        assert_eq!(snapshot.replacement_leaf_count, 1);
        assert_eq!(snapshot.replacement_leaf_pids, vec![40]);
        assert_eq!(snapshot.publish_epoch, 8);
        assert_eq!(snapshot.next_pid, 41);
        assert_eq!(snapshot.next_local_vec_seq, 100);
    }

    fn selected_split_maintenance_plan() -> update::SpireSelectedScheduledReplacementPublishLockPlan
    {
        update::SpireSelectedScheduledReplacementPublishLockPlan {
            decision: update::SpireLeafReplacementScheduleDecision {
                mode: update::SpireLeafReplacementScheduleMode::Split,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![12],
                replacement_leaf_count: 2,
                reason: "largest_split_candidate",
            },
            lock_plan: update::SpireScheduledReplacementPublishLockPlan {
                pid_plan: update::SpireLeafReplacementPidPlan {
                    replacement_pids: vec![40, 41],
                    reuses_existing_pid: false,
                    next_pid: 42,
                },
                publish_plan: update::SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: meta::SpireConsistencyMode::Strict,
                    next_pid: 42,
                    next_local_vec_seq: 100,
                },
            },
        }
    }

    #[test]
    fn maintenance_run_result_reports_no_action() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");

        let result = no_maintenance_run_result(
            root_control,
            7,
            "no_candidate",
            "active leaves are within split/merge thresholds",
        );

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "no_action");
        assert_eq!(result.planned_action, "none");
        assert_eq!(result.planned_reason, "no_candidate");
        assert_eq!(result.replacement_leaf_pids, Vec::<u64>::new());
        assert_eq!(result.publish_epoch, 0);
        assert_eq!(result.next_pid, 40);
        assert_eq!(result.next_local_vec_seq, 100);
        assert!(!result.published);
    }

    #[test]
    fn maintenance_run_result_reports_projected_selected_plan() {
        let result = selected_maintenance_run_result(
            selected_split_maintenance_plan(),
            "planned",
            false,
            "scheduled replacement selected; no epoch was published",
        )
        .expect("maintenance run result should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "planned");
        assert_eq!(result.planned_action, "split");
        assert_eq!(result.planned_reason, "largest_split_candidate");
        assert_eq!(result.replaced_parent_pid, 1);
        assert_eq!(result.affected_leaf_pids, vec![12]);
        assert_eq!(result.replacement_leaf_count, 2);
        assert_eq!(result.replacement_leaf_pids, vec![40, 41]);
        assert_eq!(result.publish_epoch, 8);
        assert_eq!(result.next_pid, 42);
        assert_eq!(result.next_local_vec_seq, 100);
        assert!(!result.published);
    }

    #[test]
    fn maintenance_run_result_reports_published_selected_plan() {
        let result = selected_maintenance_run_result(
            selected_split_maintenance_plan(),
            "published",
            true,
            "scheduled replacement epoch was published",
        )
        .expect("maintenance run result should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 8);
        assert_eq!(result.maintenance_status, "published");
        assert_eq!(result.planned_action, "split");
        assert_eq!(result.publish_epoch, 8);
        assert!(result.published);
    }

    #[test]
    fn maintenance_run_plan_from_rows_reports_selected_split_without_publishing() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 100, true, false),
        ];

        let result =
            maintenance_run_result_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance run plan should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "planned");
        assert_eq!(result.planned_action, "split");
        assert_eq!(result.planned_reason, "largest_split_candidate");
        assert_eq!(result.replacement_leaf_pids, vec![40, 41]);
        assert_eq!(result.publish_epoch, 8);
        assert_eq!(result.next_pid, 42);
        assert!(!result.published);
    }

    #[test]
    fn maintenance_run_plan_from_rows_reports_no_candidate() {
        let root_control =
            meta::SpireRootControlState::published(7, 40, 100, tid(1, 1), tid(1, 2), tid(1, 3))
                .expect("root control should build");
        let rows = vec![
            maintenance_leaf_row(11, 1, 10, false, false),
            maintenance_leaf_row(12, 1, 11, false, false),
        ];

        let result =
            maintenance_run_result_from_rows(root_control, &published_epoch_manifest(7), &rows)
                .expect("maintenance run plan should build");

        assert_eq!(result.active_epoch_before, 7);
        assert_eq!(result.active_epoch_after, 7);
        assert_eq!(result.maintenance_status, "no_action");
        assert_eq!(result.planned_action, "none");
        assert_eq!(result.planned_reason, "no_candidate");
        assert_eq!(result.replacement_leaf_pids, Vec::<u64>::new());
        assert_eq!(result.next_pid, 40);
        assert!(!result.published);
    }

    #[test]
    fn scheduled_replacement_object_version_plan_uses_successor_versions() {
        let selected = selected_split_maintenance_plan();
        let mut unaffected = maintenance_leaf_row(11, 1, 10, false, false);
        unaffected.object_version = 9;
        let mut affected = maintenance_leaf_row(12, 1, 100, true, false);
        affected.object_version = 3;

        let plan = scheduled_replacement_object_version_plan(&selected, 4, &[unaffected, affected])
            .expect("object version plan should build");

        assert_eq!(plan.parent_object_version, 5);
        assert_eq!(plan.leaf_object_version, 4);
    }

    #[test]
    fn scheduled_replacement_object_version_plan_uses_max_affected_leaf_successor() {
        let selected = update::SpireSelectedScheduledReplacementPublishLockPlan {
            decision: update::SpireLeafReplacementScheduleDecision {
                mode: update::SpireLeafReplacementScheduleMode::Merge,
                active_epoch: 7,
                replaced_parent_pid: 1,
                affected_leaf_pids: vec![11, 12],
                replacement_leaf_count: 1,
                reason: "sparsest_same_parent_merge_pair",
            },
            lock_plan: update::SpireScheduledReplacementPublishLockPlan {
                pid_plan: update::SpireLeafReplacementPidPlan {
                    replacement_pids: vec![40],
                    reuses_existing_pid: false,
                    next_pid: 41,
                },
                publish_plan: update::SpireScheduledReplacementPublishPlan {
                    epoch: 8,
                    consistency_mode: meta::SpireConsistencyMode::Strict,
                    next_pid: 41,
                    next_local_vec_seq: 100,
                },
            },
        };
        let mut first = maintenance_leaf_row(11, 1, 3, false, true);
        first.object_version = 2;
        let mut second = maintenance_leaf_row(12, 1, 1, false, true);
        second.object_version = 5;

        let plan = scheduled_replacement_object_version_plan(&selected, 4, &[first, second])
            .expect("object version plan should build");

        assert_eq!(plan.parent_object_version, 5);
        assert_eq!(plan.leaf_object_version, 6);
    }

    #[test]
    fn scheduled_replacement_object_version_plan_rejects_missing_affected_leaf() {
        let selected = selected_split_maintenance_plan();
        let rows = vec![maintenance_leaf_row(11, 1, 10, false, false)];

        let err = scheduled_replacement_object_version_plan(&selected, 4, &rows).unwrap_err();

        assert!(err.contains("missing affected leaf rows"));
    }

    #[test]
    fn leaf_snapshot_base_row_preserves_prior_delta_counts() {
        let mut rows_by_leaf_pid = HashMap::new();
        rows_by_leaf_pid.insert(
            20,
            SpireIndexLeafSnapshotRow {
                active_epoch: 7,
                leaf_pid: 20,
                parent_pid: 0,
                object_version: 0,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                local_store_id: meta::SPIRE_SINGLE_LOCAL_STORE_ID,
                placement_state: "missing_base_leaf",
                base_assignment_count: 0,
                base_primary_assignment_count: 0,
                base_boundary_replica_assignment_count: 0,
                delta_object_count: 2,
                delta_insert_assignment_count: 3,
                delta_boundary_replica_insert_assignment_count: 0,
                delta_delete_assignment_count: 1,
                effective_assignment_count: 0,
                effective_boundary_replica_assignment_count: 0,
                split_assignment_threshold: 0,
                merge_assignment_threshold: 0,
                split_recommended: false,
                merge_recommended: false,
                maintenance_action: "none",
                maintenance_reason: "missing_base_leaf",
                leaf_object_bytes: 0,
                delta_object_bytes: 44,
            },
        );
        let header = storage::SpirePartitionObjectHeader {
            kind: storage::SpirePartitionObjectKind::Leaf,
            pid: 20,
            object_version: 9,
            published_epoch_backref: 7,
            level: 1,
            parent_pid: 10,
            child_count: 0,
            assignment_count: 5,
            flags: 0,
        };
        let placement = meta::SpirePlacementEntry::local_single_store_available(
            7,
            20,
            12345,
            9,
            crate::storage::page::ItemPointer {
                block_number: 30,
                offset_number: 4,
            },
            88,
        );

        apply_leaf_snapshot_base_row(&mut rows_by_leaf_pid, 7, &header, &placement, 4, 1);

        let row = rows_by_leaf_pid.get(&20).expect("leaf row should exist");
        assert_eq!(row.parent_pid, 10);
        assert_eq!(row.object_version, 9);
        assert_eq!(row.base_assignment_count, 5);
        assert_eq!(row.base_primary_assignment_count, 4);
        assert_eq!(row.base_boundary_replica_assignment_count, 1);
        assert_eq!(row.leaf_object_bytes, 88);
        assert_eq!(row.placement_state, "available");
        assert_eq!(row.maintenance_reason, "not_evaluated");
        assert_eq!(row.delta_object_count, 2);
        assert_eq!(row.delta_insert_assignment_count, 3);
        assert_eq!(row.delta_delete_assignment_count, 1);
        assert_eq!(row.delta_object_bytes, 44);
    }

    #[test]
    fn root_routing_snapshot_rejects_active_manifest_without_root() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 20, 10);
        let epoch_manifest = published_epoch_manifest(1);
        let object_manifest =
            meta::SpireObjectManifest::from_entries(1, vec![manifest_entry_for(&leaf)])
                .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, vec![leaf])
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let err = collect_root_routing_snapshot_rows(&snapshot, &store)
            .expect_err("rootless active snapshot should fail");

        assert_eq!(
            err,
            "ec_spire root routing snapshot found no active root object"
        );
    }

    #[test]
    fn root_routing_snapshot_rejects_active_manifest_with_multiple_roots() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 20, 10);
        let first_root = store
            .insert_routing_object(1, &root_for_child(10, 20))
            .expect("first root should store");
        let second_root = store
            .insert_routing_object(1, &root_for_child(11, 20))
            .expect("second root should store");
        let epoch_manifest = published_epoch_manifest(1);
        let placements = vec![first_root, second_root, leaf];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            1,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, placements)
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let err = collect_root_routing_snapshot_rows(&snapshot, &store)
            .expect_err("multi-root active snapshot should fail");

        assert_eq!(
            err,
            "ec_spire root routing snapshot found multiple root objects"
        );
    }

    #[test]
    fn recursive_maintenance_guard_rejects_recursive_hierarchy() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 30, 20);
        let internal = store
            .insert_routing_object(
                1,
                &storage::SpireRoutingPartitionObject::internal(
                    20,
                    1,
                    1,
                    10,
                    2,
                    vec![storage::SpireRoutingChildEntry {
                        centroid_index: 0,
                        child_pid: 30,
                        centroid: vec![1.0, 0.0],
                    }],
                )
                .expect("internal routing object should build"),
            )
            .expect("internal should store");
        let root = store
            .insert_routing_object(
                1,
                &storage::SpireRoutingPartitionObject::root_at_level(
                    10,
                    1,
                    2,
                    2,
                    vec![storage::SpireRoutingChildEntry {
                        centroid_index: 0,
                        child_pid: 20,
                        centroid: vec![1.0, 0.0],
                    }],
                )
                .expect("root routing object should build"),
            )
            .expect("root should store");
        let epoch_manifest = published_epoch_manifest(1);
        let placements = vec![root, internal, leaf];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            1,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, placements)
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let err = reject_recursive_maintenance_until_update_propagation(&snapshot, &store)
            .expect_err("recursive hierarchy should reject maintenance");

        assert_eq!(err, RECURSIVE_MAINTENANCE_DEFERRED_MESSAGE);
    }

    #[test]
    fn root_routing_snapshot_reports_child_rows_from_local_store() {
        let mut store = storage::SpireLocalObjectStore::with_default_page_size(12345)
            .expect("local store should build");
        let leaf = empty_leaf_row(&mut store, 20, 10);
        let root = store
            .insert_routing_object(1, &root_for_child(10, 20))
            .expect("root should store");
        let epoch_manifest = published_epoch_manifest(1);
        let placements = vec![root, leaf];
        let object_manifest = meta::SpireObjectManifest::from_entries(
            1,
            placements.iter().map(manifest_entry_for).collect(),
        )
        .expect("object manifest should build");
        let placement_directory = meta::SpirePlacementDirectory::from_entries(1, placements)
            .expect("placement directory should build");
        let snapshot = meta::SpireValidatedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )
        .expect("snapshot should validate");

        let rows = collect_root_routing_snapshot_rows(&snapshot, &store)
            .expect("root routing rows should collect");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].root_pid, 10);
        assert_eq!(rows[0].child_pid, 20);
        assert_eq!(rows[0].child_kind, "leaf");
        assert_eq!(rows[0].child_store_relid, 12345);
    }
}
