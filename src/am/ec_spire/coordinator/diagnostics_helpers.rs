// Pure helpers extracted from `diagnostics.rs` so they can be
// `include!`'d into both production (via diagnostics.rs → mod.rs) and
// the hardening shadow crate, where the same logic is exercised under
// `cargo test --manifest-path hardening/careful/Cargo.toml --lib`.
//
// Functions in this file MUST stay free of pgrx-FFI side effects and
// `pg_sys` calls so the careful shim can host them without extra
// scaffolding. Anything that needs an open relation, a snapshot, or
// SPI stays in `diagnostics.rs`.

const SPIRE_LEAF_SPLIT_AVERAGE_MULTIPLIER: u64 = 4;
const SPIRE_LEAF_SPLIT_MIN_ASSIGNMENTS: u64 = 32;
const SPIRE_LEAF_MERGE_AVERAGE_DIVISOR: u64 = 4;

const SPIRE_ASSIGNMENT_PAYLOAD_STATUS_SUPPORTED: &str = "supported";
const SPIRE_ASSIGNMENT_PAYLOAD_STATUS_DEFERRED_MODEL_METADATA: &str = "deferred_model_metadata";

fn assignment_payload_format_name(format: quantizer::SpireAssignmentPayloadFormat) -> &'static str {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant => "turboquant",
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => "pq_fastscan",
        quantizer::SpireAssignmentPayloadFormat::RaBitQ => "rabitq",
    }
}

fn assignment_payload_scannability(
    format: quantizer::SpireAssignmentPayloadFormat,
) -> (bool, &'static str, &'static str) {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant
        | quantizer::SpireAssignmentPayloadFormat::RaBitQ => (
            true,
            SPIRE_ASSIGNMENT_PAYLOAD_STATUS_SUPPORTED,
            "format can be used for current SPIRE scans",
        ),
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => (
            false,
            SPIRE_ASSIGNMENT_PAYLOAD_STATUS_DEFERRED_MODEL_METADATA,
            "persist grouped-PQ model metadata before using pq_fastscan with SPIRE",
        ),
    }
}

fn boundary_replica_identity_scope(vec_id: &[u8]) -> &'static str {
    match vec_id.first().copied() {
        Some(storage::SPIRE_GLOBAL_VEC_ID_DISCRIMINATOR) => "global",
        Some(storage::SPIRE_LOCAL_VEC_ID_DISCRIMINATOR) => "node_local",
        _ => "invalid",
    }
}

fn boundary_replica_identity_status(
    vec_id_scope: &'static str,
    primary_assignment_count: u64,
    boundary_replica_assignment_count: u64,
    node_count: u64,
) -> (&'static str, &'static str) {
    if primary_assignment_count == 0 {
        (
            "missing_primary_assignment",
            "boundary replica identity requires one primary assignment for each replicated vec_id",
        )
    } else if primary_assignment_count > 1 {
        (
            "duplicate_primary_assignment",
            "inspect boundary routing because one replicated vec_id has multiple primary assignments",
        )
    } else if boundary_replica_assignment_count == 0 {
        (
            "missing_boundary_replica",
            "no boundary replica assignment is present for this vec_id",
        )
    } else if vec_id_scope == "global" {
        (
            "ready",
            "global vec_id is shared by the primary and boundary replica assignments",
        )
    } else if vec_id_scope == "node_local" && node_count <= 1 {
        (
            "local_scope_only",
            "node-local vec_id can dedupe local boundary replicas but is not safe for cross-node replica dedupe",
        )
    } else {
        (
            "requires_global_vec_id",
            "enable source_identity = 'include' before using cross-node boundary replica dedupe",
        )
    }
}

fn boundary_replica_placement_status(
    primary_assignment_count: u64,
    boundary_replica_assignment_count: u64,
    stale_boundary_replica_count: u64,
    unavailable_boundary_replica_count: u64,
    skipped_boundary_replica_count: u64,
) -> (&'static str, &'static str, &'static str) {
    if primary_assignment_count == 0 {
        (
            "missing_primary_assignment",
            "fail_closed",
            "boundary replica diagnostics require a primary assignment for each replicated vec_id",
        )
    } else if boundary_replica_assignment_count == 0 {
        (
            "missing_boundary_replica",
            "fail_closed",
            "restore boundary replica assignment coverage before relying on degraded replica reads",
        )
    } else if stale_boundary_replica_count > 0 {
        (
            "stale_boundary_replica",
            "fail_closed",
            "do not serve stale boundary replica placements in degraded mode",
        )
    } else if unavailable_boundary_replica_count > 0 {
        (
            "unavailable_boundary_replica",
            "skip_and_report",
            "report unavailable boundary replica placements in degraded search diagnostics",
        )
    } else if skipped_boundary_replica_count > 0 {
        (
            "skipped_boundary_replica",
            "skip_and_report",
            "report skipped boundary replica placements in degraded search diagnostics",
        )
    } else {
        (
            "ready",
            "serve_or_dedupe",
            "boundary replica placement coverage is available",
        )
    }
}

fn scan_sanity_status(
    active_epoch: u64,
    exact_leaf_coverage: bool,
    full_frontier_rerank: bool,
) -> (&'static str, &'static str, &'static str) {
    if active_epoch == 0 {
        return (
            "empty",
            "none",
            "build or insert rows to publish the first SPIRE epoch",
        );
    }
    if exact_leaf_coverage && full_frontier_rerank {
        return (
            "exact_leaf_and_frontier_coverage",
            "full_scan",
            "use this configuration only when max_candidate_rows covers the expected frontier",
        );
    }
    if exact_leaf_coverage {
        return (
            "exact_leaf_coverage_bounded_rerank",
            "bounded_rerank",
            "set rerank_width = 0 and max_candidate_rows high enough for full-frontier recall sanity checks",
        );
    }
    (
        "approximate_leaf_coverage",
        "bounded_leaf_probe",
        "increase nprobe to active_leaf_count for exact leaf coverage sanity checks",
    )
}

fn consistency_mode_name(mode: meta::SpireConsistencyMode) -> &'static str {
    match mode {
        meta::SpireConsistencyMode::Strict => "strict",
        meta::SpireConsistencyMode::Degraded => "degraded",
    }
}

fn epoch_state_name(state: meta::SpireEpochState) -> &'static str {
    match state {
        meta::SpireEpochState::Building => "building",
        meta::SpireEpochState::Published => "published",
        meta::SpireEpochState::Retired => "retired",
        meta::SpireEpochState::Failed => "failed",
    }
}

fn placement_state_name(state: meta::SpirePlacementState) -> &'static str {
    match state {
        meta::SpirePlacementState::Available => "available",
        meta::SpirePlacementState::Stale => "stale",
        meta::SpirePlacementState::Unavailable => "unavailable",
        meta::SpirePlacementState::Skipped => "skipped",
    }
}

fn partition_object_kind_name(kind: storage::SpirePartitionObjectKind) -> &'static str {
    match kind {
        storage::SpirePartitionObjectKind::Root => "root",
        storage::SpirePartitionObjectKind::Internal => "internal",
        storage::SpirePartitionObjectKind::Leaf => "leaf",
        storage::SpirePartitionObjectKind::Delta => "delta",
        storage::SpirePartitionObjectKind::TopGraph => "top_graph",
    }
}

fn leaf_maintenance_thresholds(effective_total: u64, leaf_count: u64) -> (u64, u64) {
    if leaf_count == 0 {
        return (0, 0);
    }
    let average = effective_total.div_ceil(leaf_count);
    let split_threshold = average
        .saturating_mul(SPIRE_LEAF_SPLIT_AVERAGE_MULTIPLIER)
        .max(SPIRE_LEAF_SPLIT_MIN_ASSIGNMENTS);
    let merge_threshold = average / SPIRE_LEAF_MERGE_AVERAGE_DIVISOR;
    (split_threshold, merge_threshold)
}

fn leaf_maintenance_labels(
    effective_assignment_count: u64,
    split_threshold: u64,
    merge_threshold: u64,
) -> (bool, bool, &'static str, &'static str) {
    if effective_assignment_count >= split_threshold && split_threshold > 0 {
        return (
            true,
            false,
            "split_candidate",
            "effective_assignments_at_or_above_split_threshold",
        );
    }
    if effective_assignment_count <= merge_threshold {
        return (
            false,
            true,
            "merge_candidate",
            "effective_assignments_at_or_below_merge_threshold",
        );
    }
    (false, false, "none", "within_distribution_thresholds")
}
