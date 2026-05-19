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
