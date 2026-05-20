//! Careful-side scaffold for
//! `src/am/ec_spire/coordinator/diagnostics_helpers.rs`.
//!
//! The helpers file under production lives in the coordinator scope
//! where `meta::*`, `storage::*`, and `quantizer::SpireAssignmentPayloadFormat`
//! are already in scope via the `mod.rs` include chain. The careful
//! crate doesn't include the full coordinator (it would require
//! shimming most of `coordinator/types.rs`), so this scaffold sets up
//! a minimal compatible scope and `include!`s the same helpers
//! verbatim so coverage attributes back to the production file.

#![allow(dead_code, non_snake_case)]

mod scaffold {
    // Local shims of the production enums and constants the helpers
    // file references. The production builds get the canonical
    // definitions via mod.rs's include chain; this shadow copy lets the
    // helpers compile inside the careful crate without pulling the full
    // coordinator/types.rs and quantizer/mod.rs scaffolds.

    pub(super) mod meta {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum SpireConsistencyMode {
            Strict,
            Degraded,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum SpireEpochState {
            Building,
            Published,
            Retired,
            Failed,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum SpirePlacementState {
            Available,
            Stale,
            Unavailable,
            Skipped,
        }

        // Shim of `src/am/ec_spire/meta/epoch.rs::SpireEpochManifest`.
        // The helpers only read `state`, `active_query_count`, and
        // `retain_until_micros` from the manifest.
        #[derive(Debug, Clone, Copy)]
        pub struct SpireEpochManifest {
            pub epoch: u64,
            pub state: SpireEpochState,
            pub consistency_mode: SpireConsistencyMode,
            pub published_at_micros: i64,
            pub retain_until_micros: i64,
            pub active_query_count: u64,
        }
    }

    pub(super) mod storage {
        pub const SPIRE_LOCAL_VEC_ID_DISCRIMINATOR: u8 = 0x01;
        pub const SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR: u8 = 0x02;

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum SpirePartitionObjectKind {
            Root,
            Internal,
            Leaf,
            Delta,
            TopGraph,
        }
    }

    // Shim of `coordinator/types.rs::SpireActiveSnapshotDiagnostics`
    // used by health_snapshot_from_diagnostics.
    #[derive(Debug, Clone)]
    pub(super) struct SpireActiveSnapshotDiagnostics {
        pub(super) active_epoch: u64,
        pub(super) next_pid: u64,
        pub(super) next_local_vec_seq: u64,
        pub(super) consistency_mode: &'static str,
        pub(super) object_count: u64,
        pub(super) placement_count: u64,
        pub(super) local_store_count: u64,
        pub(super) available_placement_count: u64,
        pub(super) stale_placement_count: u64,
        pub(super) unavailable_placement_count: u64,
        pub(super) skipped_placement_count: u64,
        pub(super) root_object_count: u64,
        pub(super) internal_object_count: u64,
        pub(super) leaf_object_count: u64,
        pub(super) delta_object_count: u64,
        pub(super) routing_child_count: u64,
        pub(super) leaf_assignment_count: u64,
        pub(super) delta_assignment_count: u64,
        pub(super) available_object_bytes: u64,
        pub(super) routing_object_bytes: u64,
        pub(super) leaf_object_bytes: u64,
        pub(super) delta_object_bytes: u64,
    }

    // Shim of `coordinator/types.rs::SpireIndexHealthSnapshot` produced
    // by health_snapshot_from_diagnostics.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(super) struct SpireIndexHealthSnapshot {
        pub(super) active_epoch: u64,
        pub(super) consistency_mode: &'static str,
        pub(super) status: &'static str,
        pub(super) healthy: bool,
        pub(super) recommendation: &'static str,
        pub(super) compaction_recommended: bool,
        pub(super) object_count: u64,
        pub(super) leaf_assignment_count: u64,
        pub(super) delta_assignment_count: u64,
        pub(super) delta_object_count: u64,
        pub(super) available_placement_count: u64,
        pub(super) stale_placement_count: u64,
        pub(super) unavailable_placement_count: u64,
        pub(super) skipped_placement_count: u64,
    }

    pub(super) mod quantizer {
        // Minimal shim of the production enum at
        // `src/am/ec_spire/quantizer/mod.rs::SpireAssignmentPayloadFormat`.
        // The helpers only need the three variants and `match` exhaustiveness.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum SpireAssignmentPayloadFormat {
            TurboQuant,
            PqFastScan,
            RaBitQ,
        }
    }

    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../src/am/ec_spire/coordinator/diagnostics_helpers.rs"
    ));

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn miri_assignment_payload_format_name_covers_every_variant() {
            assert_eq!(
                assignment_payload_format_name(quantizer::SpireAssignmentPayloadFormat::TurboQuant),
                "turboquant",
            );
            assert_eq!(
                assignment_payload_format_name(quantizer::SpireAssignmentPayloadFormat::PqFastScan),
                "pq_fastscan",
            );
            assert_eq!(
                assignment_payload_format_name(quantizer::SpireAssignmentPayloadFormat::RaBitQ),
                "rabitq",
            );
        }

        #[test]
        fn miri_assignment_payload_scannability_reports_status_per_variant() {
            let (scannable, status, _rec) =
                assignment_payload_scannability(quantizer::SpireAssignmentPayloadFormat::TurboQuant);
            assert!(scannable);
            assert_eq!(status, "supported");

            let (scannable, status, _rec) =
                assignment_payload_scannability(quantizer::SpireAssignmentPayloadFormat::RaBitQ);
            assert!(scannable);
            assert_eq!(status, "supported");

            let (scannable, status, rec) =
                assignment_payload_scannability(quantizer::SpireAssignmentPayloadFormat::PqFastScan);
            assert!(!scannable);
            assert_eq!(status, "deferred_model_metadata");
            assert!(rec.contains("grouped-PQ"));
        }

        #[test]
        fn miri_boundary_replica_identity_scope_distinguishes_global_local_invalid() {
            assert_eq!(
                boundary_replica_identity_scope(&[storage::SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR, 9, 9]),
                "global",
            );
            assert_eq!(
                boundary_replica_identity_scope(&[storage::SPIRE_LOCAL_VEC_ID_DISCRIMINATOR, 1, 0, 0, 0, 0, 0, 0, 0]),
                "node_local",
            );
            assert_eq!(boundary_replica_identity_scope(&[0xff, 1, 2]), "invalid");
            assert_eq!(boundary_replica_identity_scope(&[]), "invalid");
        }

        #[test]
        fn miri_boundary_replica_identity_status_walks_every_branch() {
            // primary_assignment_count == 0
            let (status, _) = boundary_replica_identity_status("global", 0, 1, 1);
            assert_eq!(status, "missing_primary_assignment");
            // primary_assignment_count > 1
            let (status, _) = boundary_replica_identity_status("global", 2, 1, 1);
            assert_eq!(status, "duplicate_primary_assignment");
            // boundary_replica_assignment_count == 0
            let (status, _) = boundary_replica_identity_status("global", 1, 0, 1);
            assert_eq!(status, "missing_boundary_replica");
            // global vec_id
            let (status, _) = boundary_replica_identity_status("global", 1, 1, 3);
            assert_eq!(status, "ready");
            // node_local with one node
            let (status, _) = boundary_replica_identity_status("node_local", 1, 1, 1);
            assert_eq!(status, "local_scope_only");
            // node_local with multiple nodes
            let (status, _) = boundary_replica_identity_status("node_local", 1, 1, 3);
            assert_eq!(status, "requires_global_vec_id");
        }

        #[test]
        fn miri_boundary_replica_placement_status_walks_every_branch() {
            let (status, decision, _) = boundary_replica_placement_status(0, 1, 0, 0, 0);
            assert_eq!(status, "missing_primary_assignment");
            assert_eq!(decision, "fail_closed");
            let (status, decision, _) = boundary_replica_placement_status(1, 0, 0, 0, 0);
            assert_eq!(status, "missing_boundary_replica");
            assert_eq!(decision, "fail_closed");
            let (status, decision, _) = boundary_replica_placement_status(1, 1, 1, 0, 0);
            assert_eq!(status, "stale_boundary_replica");
            assert_eq!(decision, "fail_closed");
            let (status, decision, _) = boundary_replica_placement_status(1, 1, 0, 1, 0);
            assert_eq!(status, "unavailable_boundary_replica");
            assert_eq!(decision, "skip_and_report");
            let (status, decision, _) = boundary_replica_placement_status(1, 1, 0, 0, 1);
            assert_eq!(status, "skipped_boundary_replica");
            assert_eq!(decision, "skip_and_report");
            let (status, decision, _) = boundary_replica_placement_status(1, 1, 0, 0, 0);
            assert_eq!(status, "ready");
            assert_eq!(decision, "serve_or_dedupe");
        }

        #[test]
        fn miri_scan_sanity_status_picks_configuration_per_input() {
            let (status, scan_mode, _) = scan_sanity_status(0, false, false);
            assert_eq!(status, "empty");
            assert_eq!(scan_mode, "none");

            let (status, scan_mode, _) = scan_sanity_status(7, true, true);
            assert_eq!(status, "exact_leaf_and_frontier_coverage");
            assert_eq!(scan_mode, "full_scan");

            let (status, scan_mode, _) = scan_sanity_status(7, true, false);
            assert_eq!(status, "exact_leaf_coverage_bounded_rerank");
            assert_eq!(scan_mode, "bounded_rerank");

            let (status, scan_mode, _) = scan_sanity_status(7, false, true);
            assert_eq!(status, "approximate_leaf_coverage");
            assert_eq!(scan_mode, "bounded_leaf_probe");
        }

        #[test]
        fn miri_meta_name_helpers_cover_every_enum_variant() {
            assert_eq!(consistency_mode_name(meta::SpireConsistencyMode::Strict), "strict");
            assert_eq!(consistency_mode_name(meta::SpireConsistencyMode::Degraded), "degraded");

            assert_eq!(epoch_state_name(meta::SpireEpochState::Building), "building");
            assert_eq!(epoch_state_name(meta::SpireEpochState::Published), "published");
            assert_eq!(epoch_state_name(meta::SpireEpochState::Retired), "retired");
            assert_eq!(epoch_state_name(meta::SpireEpochState::Failed), "failed");

            assert_eq!(placement_state_name(meta::SpirePlacementState::Available), "available");
            assert_eq!(placement_state_name(meta::SpirePlacementState::Stale), "stale");
            assert_eq!(placement_state_name(meta::SpirePlacementState::Unavailable), "unavailable");
            assert_eq!(placement_state_name(meta::SpirePlacementState::Skipped), "skipped");

            assert_eq!(partition_object_kind_name(storage::SpirePartitionObjectKind::Root), "root");
            assert_eq!(partition_object_kind_name(storage::SpirePartitionObjectKind::Internal), "internal");
            assert_eq!(partition_object_kind_name(storage::SpirePartitionObjectKind::Leaf), "leaf");
            assert_eq!(partition_object_kind_name(storage::SpirePartitionObjectKind::Delta), "delta");
            assert_eq!(partition_object_kind_name(storage::SpirePartitionObjectKind::TopGraph), "top_graph");
        }

        fn empty_diagnostics() -> SpireActiveSnapshotDiagnostics {
            SpireActiveSnapshotDiagnostics {
                active_epoch: 7,
                next_pid: 1,
                next_local_vec_seq: 1,
                consistency_mode: "strict",
                object_count: 0,
                placement_count: 0,
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
            }
        }

        #[test]
        fn miri_health_snapshot_walks_every_status_branch() {
            // active_epoch == 0 short-circuits to "empty".
            let mut diag = empty_diagnostics();
            diag.active_epoch = 0;
            let snap = health_snapshot_from_diagnostics(&diag);
            assert_eq!(snap.status, "empty");
            assert!(snap.healthy);
            assert!(!snap.compaction_recommended);

            // unavailable placements take priority over stale/skipped.
            let mut diag = empty_diagnostics();
            diag.unavailable_placement_count = 1;
            diag.stale_placement_count = 1;
            assert_eq!(
                health_snapshot_from_diagnostics(&diag).status,
                "unavailable_placements",
            );

            // stale placements.
            let mut diag = empty_diagnostics();
            diag.stale_placement_count = 1;
            assert_eq!(
                health_snapshot_from_diagnostics(&diag).status,
                "stale_placements",
            );

            // skipped placements.
            let mut diag = empty_diagnostics();
            diag.skipped_placement_count = 1;
            assert_eq!(
                health_snapshot_from_diagnostics(&diag).status,
                "skipped_placements",
            );

            // delta objects → maintenance_recommended + compaction.
            let mut diag = empty_diagnostics();
            diag.delta_object_count = 5;
            let snap = health_snapshot_from_diagnostics(&diag);
            assert_eq!(snap.status, "maintenance_recommended");
            assert!(snap.compaction_recommended);
            assert!(snap.healthy);

            // degraded consistency.
            let mut diag = empty_diagnostics();
            diag.consistency_mode = "degraded";
            assert_eq!(
                health_snapshot_from_diagnostics(&diag).status,
                "degraded_consistency",
            );

            // ok path with all clean counters.
            let diag = empty_diagnostics();
            let snap = health_snapshot_from_diagnostics(&diag);
            assert_eq!(snap.status, "ok");
            assert!(snap.healthy);
            assert_eq!(snap.recommendation, "none");
        }

        #[test]
        fn miri_epoch_cleanup_blocked_reason_walks_every_branch() {
            let base = meta::SpireEpochManifest {
                epoch: 5,
                state: meta::SpireEpochState::Retired,
                consistency_mode: meta::SpireConsistencyMode::Strict,
                published_at_micros: 100,
                retain_until_micros: 1_000,
                active_query_count: 0,
            };

            // cleanup_eligible short-circuits.
            assert_eq!(
                epoch_cleanup_blocked_reason(&base, 500, false, false, true),
                "cleanup_eligible",
            );
            // is_active_root_manifest takes priority over state.
            assert_eq!(
                epoch_cleanup_blocked_reason(&base, 500, true, false, false),
                "active_root_manifest",
            );
            // Building → state_not_cleanup_eligible.
            let mut building = base;
            building.state = meta::SpireEpochState::Building;
            assert_eq!(
                epoch_cleanup_blocked_reason(&building, 500, false, false, false),
                "state_not_cleanup_eligible",
            );
            // Published → state_not_cleanup_eligible.
            let mut published = base;
            published.state = meta::SpireEpochState::Published;
            assert_eq!(
                epoch_cleanup_blocked_reason(&published, 500, false, false, false),
                "state_not_cleanup_eligible",
            );
            // Retired with active queries → active_queries.
            let mut active = base;
            active.active_query_count = 3;
            assert_eq!(
                epoch_cleanup_blocked_reason(&active, 500, false, false, false),
                "active_queries",
            );
            // Retired and retained_retired → retained_retired_epoch.
            assert_eq!(
                epoch_cleanup_blocked_reason(&base, 500, false, true, false),
                "retained_retired_epoch",
            );
            // Retired within retention window → retention_window.
            assert_eq!(
                epoch_cleanup_blocked_reason(&base, 500, false, false, false),
                "retention_window",
            );
            // Retired past retention window → cleanup_plan_retained.
            assert_eq!(
                epoch_cleanup_blocked_reason(&base, 2_000, false, false, false),
                "cleanup_plan_retained",
            );
            // Failed within retention → retention_window.
            let mut failed = base;
            failed.state = meta::SpireEpochState::Failed;
            assert_eq!(
                epoch_cleanup_blocked_reason(&failed, 500, false, false, false),
                "retention_window",
            );
            // Failed past retention → cleanup_plan_retained.
            assert_eq!(
                epoch_cleanup_blocked_reason(&failed, 2_000, false, false, false),
                "cleanup_plan_retained",
            );
        }

        #[test]
        fn miri_leaf_maintenance_thresholds_and_labels_round_trip() {
            // leaf_count == 0 short-circuits to (0, 0).
            assert_eq!(leaf_maintenance_thresholds(100, 0), (0, 0));

            // Balanced: 1000 effective / 10 leaves → avg 100; split = max(32, 100*4) = 400; merge = 25.
            let (split, merge) = leaf_maintenance_thresholds(1000, 10);
            assert_eq!(split, 400);
            assert_eq!(merge, 25);

            // Above split: split_recommended.
            let (split_rec, merge_rec, action, _reason) =
                leaf_maintenance_labels(500, split, merge);
            assert!(split_rec);
            assert!(!merge_rec);
            assert_eq!(action, "split_candidate");

            // Below merge: merge_recommended.
            let (split_rec, merge_rec, action, _reason) =
                leaf_maintenance_labels(10, split, merge);
            assert!(!split_rec);
            assert!(merge_rec);
            assert_eq!(action, "merge_candidate");

            // Between thresholds: neither.
            let (split_rec, merge_rec, action, _reason) =
                leaf_maintenance_labels(100, split, merge);
            assert!(!split_rec);
            assert!(!merge_rec);
            assert_eq!(action, "none");
        }
    }
}
