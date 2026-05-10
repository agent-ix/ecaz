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
    fn rerank_scored_candidates_by_ip_prefetches_prefix_before_fetching() {
        let mut candidates = vec![
            scored_candidate(1, 10, 1, -5.0),
            scored_candidate(2, 20, 2, -4.0),
            scored_candidate(3, 30, 3, -3.0),
        ];
        let events = RefCell::new(Vec::new());

        rerank_scored_candidates_by_ip_with_prefetch(
            &mut candidates,
            2,
            |prefetch_candidates| {
                let sequences = prefetch_candidates
                    .iter()
                    .map(|candidate| candidate.vec_id.local_sequence().unwrap())
                    .collect::<Vec<_>>();
                events.borrow_mut().push(format!("prefetch:{sequences:?}"));
                Ok(())
            },
            |candidate| {
                let sequence = candidate.vec_id.local_sequence().unwrap();
                events.borrow_mut().push(format!("score:{sequence}"));
                Ok(Some(sequence as f32))
            },
        )
        .unwrap();

        assert_eq!(
            events.into_inner(),
            vec![
                "prefetch:[1, 2]".to_owned(),
                "score:1".to_owned(),
                "score:2".to_owned(),
            ]
        );
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].vec_id.local_sequence(), Some(2));
        assert_eq!(candidates[1].vec_id.local_sequence(), Some(1));
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
    fn heap_rerank_prefetch_block_numbers_dedupes_and_sorts_blocks() {
        let candidates = vec![
            scored_candidate(1, 30, 1, -5.0),
            scored_candidate(2, 10, 2, -4.0),
            scored_candidate(3, 30, 3, -3.0),
            scored_candidate(4, 20, 4, -2.0),
        ];

        assert_eq!(
            heap_rerank_prefetch_block_numbers(&candidates),
            vec![10, 20, 30]
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
    fn scan_output_cursor_emits_am_outputs_once() {
        let mut cursor = SpireScanOutputCursor::new(vec![
            SpireScanOutput {
                heap_tid: tid(41, 1),
                orderby_score: -4.1,
            },
            SpireScanOutput {
                heap_tid: tid(42, 2),
                orderby_score: -4.2,
            },
        ]);

        assert_eq!(cursor.remaining(), 2);
        assert_eq!(
            cursor.next_output(),
            Some(SpireScanOutput {
                heap_tid: tid(41, 1),
                orderby_score: -4.1,
            })
        );
        assert_eq!(cursor.remaining(), 1);
        assert_eq!(
            cursor.next_output(),
            Some(SpireScanOutput {
                heap_tid: tid(42, 2),
                orderby_score: -4.2,
            })
        );
        assert!(cursor.is_exhausted());
        assert!(cursor.next_output().is_none());
    }

    #[test]
    fn production_scan_result_stream_am_outputs_accepts_local_heap_tid_rows() {
        let stream = production_scan_stream_for_am(
            SpireRemoteProductionScanAmDeliverySummaryRow {
                requested_epoch: 1,
                output_count: 1,
                local_heap_tid_output_count: 1,
                remote_origin_output_count: 0,
                am_deliverable_output_count: 1,
                status: SPIRE_REMOTE_STATUS_READY,
                next_blocker: SPIRE_REMOTE_NONE,
                recommendation: SPIRE_REMOTE_NONE,
            },
            vec![SpireRemoteProductionScanOutputRow {
                requested_epoch: 1,
                served_epoch: 1,
                node_id: super::super::meta::SPIRE_LOCAL_NODE_ID,
                heap_block: 50,
                heap_offset: 3,
                score: -1.25,
                heap_lookup_owner: SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
                vec_id: vec![1],
                row_locator: vec![2],
            }],
        );

        assert_eq!(
            production_scan_result_stream_am_outputs(&stream).unwrap(),
            vec![SpireScanOutput {
                heap_tid: tid(50, 3),
                orderby_score: -1.25,
            }]
        );
    }

    #[test]
    fn production_scan_result_stream_am_outputs_blocks_remote_origin_rows() {
        let stream = production_scan_stream_for_am(
            SpireRemoteProductionScanAmDeliverySummaryRow {
                requested_epoch: 1,
                output_count: 1,
                local_heap_tid_output_count: 0,
                remote_origin_output_count: 1,
                am_deliverable_output_count: 0,
                status: SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_ROW_MATERIALIZATION,
                next_blocker: SPIRE_REMOTE_EXECUTOR_STEP_REMOTE_ROW_MATERIALIZATION,
                recommendation: "materialize first",
            },
            vec![SpireRemoteProductionScanOutputRow {
                requested_epoch: 1,
                served_epoch: 1,
                node_id: 9,
                heap_block: 60,
                heap_offset: 4,
                score: -1.5,
                heap_lookup_owner: "origin_node_heap",
                vec_id: vec![1],
                row_locator: vec![2],
            }],
        );

        let error = production_scan_result_stream_am_outputs(&stream)
            .expect_err("remote-origin rows should block AM delivery");

        assert!(error.contains("remote_row_materialization"));
        assert!(error.contains("blocked"));
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
            recursive_nprobe_policy: SpireRecursiveNprobePolicy::conservative(1).unwrap(),
            recursive_route_budget: SpireRecursiveRouteBudget::unbounded(),
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
            recursive_nprobe_policy: SpireRecursiveNprobePolicy::conservative(1).unwrap(),
            recursive_route_budget: SpireRecursiveRouteBudget::unbounded(),
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
    fn local_heap_delivery_gate_accepts_local_placements() {
        let placement = SpirePlacementEntry::local_single_store_available(
            1,
            SPIRE_FIRST_PID,
            42,
            1,
            tid(10, 1),
            128,
        );
        let directory = SpirePlacementDirectory::from_entries(1, vec![placement]).unwrap();

        ensure_local_heap_placement_directory_is_deliverable(&directory)
            .expect("local placements should be deliverable through xs_heaptid");
    }

    #[test]
    fn local_heap_delivery_gate_blocks_remote_placements() {
        let mut placement = SpirePlacementEntry::local_single_store_available(
            1,
            SPIRE_FIRST_PID + 3,
            42,
            1,
            tid(10, 2),
            128,
        );
        placement.node_id = 9;
        let directory = SpirePlacementDirectory::from_entries(1, vec![placement]).unwrap();

        let error = ensure_local_heap_placement_directory_is_deliverable(&directory)
            .expect_err("remote placements should require materialization");

        assert!(error.contains("remote_row_materialization"));
        assert!(error.contains("1 remote placement"));
        assert!(error.contains("node_id 9"));
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
