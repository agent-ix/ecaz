impl SpireActiveSnapshotDiagnostics {
    fn empty(root_control: meta::SpireRootControlState) -> Self {
        Self {
            active_epoch: root_control.active_epoch,
            next_pid: root_control.next_pid,
            next_local_vec_seq: root_control.next_local_vec_seq,
            consistency_mode: "none",
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
}

fn health_snapshot_from_diagnostics(
    diagnostics: &SpireActiveSnapshotDiagnostics,
) -> SpireIndexHealthSnapshot {
    let has_no_active_epoch = diagnostics.active_epoch == 0;
    let (status, healthy, recommendation, compaction_recommended) = if has_no_active_epoch {
        (
            "empty",
            true,
            "build or insert rows to publish the first SPIRE epoch",
            false,
        )
    } else if diagnostics.unavailable_placement_count > 0 {
        (
            "unavailable_placements",
            false,
            "restore unavailable local placements before relying on this index",
            false,
        )
    } else if diagnostics.stale_placement_count > 0 {
        (
            "stale_placements",
            false,
            "publish a cleanup epoch to remove stale placements",
            false,
        )
    } else if diagnostics.skipped_placement_count > 0 {
        (
            "skipped_placements",
            false,
            "inspect skipped placements before enabling degraded reads",
            false,
        )
    } else if diagnostics.delta_object_count > 0 {
        (
            "maintenance_recommended",
            true,
            "run VACUUM to compact active delta objects into V2 base leaves",
            true,
        )
    } else if diagnostics.consistency_mode == "degraded" {
        (
            "degraded_consistency",
            true,
            "verify degraded-read policy before relying on strict local semantics",
            false,
        )
    } else {
        ("ok", true, "none", false)
    };

    SpireIndexHealthSnapshot {
        active_epoch: diagnostics.active_epoch,
        consistency_mode: diagnostics.consistency_mode,
        status,
        healthy,
        recommendation,
        compaction_recommended,
        object_count: diagnostics.object_count,
        leaf_assignment_count: diagnostics.leaf_assignment_count,
        delta_assignment_count: diagnostics.delta_assignment_count,
        delta_object_count: diagnostics.delta_object_count,
        available_placement_count: diagnostics.available_placement_count,
        stale_placement_count: diagnostics.stale_placement_count,
        unavailable_placement_count: diagnostics.unavailable_placement_count,
        skipped_placement_count: diagnostics.skipped_placement_count,
    }
}

fn assignment_payload_format_name(format: quantizer::SpireAssignmentPayloadFormat) -> &'static str {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant => "turboquant",
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => "pq_fastscan",
        quantizer::SpireAssignmentPayloadFormat::RaBitQ => "rabitq",
    }
}

const SPIRE_ASSIGNMENT_PAYLOAD_STATUS_SUPPORTED: &str = "supported";
const SPIRE_ASSIGNMENT_PAYLOAD_STATUS_DEFERRED_MODEL_METADATA: &str = "deferred_model_metadata";

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

fn count_snapshot_options_leaf_pids(
    snapshot: &meta::SpirePublishedEpochSnapshot<'_>,
    object_store: &impl storage::SpireObjectReader,
    recursive_build_enabled: bool,
) -> Result<u32, String> {
    if recursive_build_enabled {
        scan::count_snapshot_recursive_leaf_pids(snapshot, object_store)
    } else {
        scan::count_snapshot_single_level_leaf_pids(snapshot, object_store)
    }
}

fn level_target_fanout(
    level: u16,
    relation_options: &options::EcSpireOptions,
    observed_child_count: u64,
) -> Result<u32, String> {
    if level == 1 && relation_options.nlists > 0 {
        return u32::try_from(relation_options.nlists)
            .map_err(|_| "ec_spire nlists reloption must be non-negative".to_owned());
    }
    if level > 1 {
        if let Some(recursive_fanout) = relation_options.recursive_fanout() {
            return Ok(recursive_fanout);
        }
    }
    u32::try_from(observed_child_count)
        .map_err(|_| "ec_spire observed routing child count exceeds u32".to_owned())
}

fn level_nprobe_resolution(
    level: u16,
    leaf_count: u32,
    relation_options: &options::EcSpireOptions,
) -> Result<(Option<i32>, u32, &'static str, &'static str), String> {
    let relation_nprobe = u32::try_from(relation_options.nprobe)
        .map_err(|_| "ec_spire nprobe reloption must be non-negative".to_owned())?;
    if level <= 1 {
        let resolved = options::resolve_scan_nprobe(leaf_count, relation_nprobe);
        let session_nprobe = resolved
            .session_nprobe
            .map(|value| i32::try_from(value).expect("session nprobe should fit in i32"));
        return Ok((
            session_nprobe,
            resolved.effective_nprobe,
            resolved.source,
            "relation_or_session_leaf_level",
        ));
    }
    Ok((
        None,
        1,
        "conservative_upper_level",
        "one_child_above_level_1",
    ))
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
            "use this configuration only for recall sanity checks or small indexes",
        );
    }
    if exact_leaf_coverage {
        return (
            "exact_leaf_coverage_bounded_rerank",
            "bounded_rerank",
            "set rerank_width = 0 for full-frontier exact recall sanity checks",
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

fn epoch_cleanup_blocked_reason(
    manifest: &meta::SpireEpochManifest,
    now_micros: i64,
    is_active_root_manifest: bool,
    retained_retired: bool,
    cleanup_eligible_now: bool,
) -> &'static str {
    if cleanup_eligible_now {
        return "cleanup_eligible";
    }
    if is_active_root_manifest {
        return "active_root_manifest";
    }
    match manifest.state {
        meta::SpireEpochState::Building | meta::SpireEpochState::Published => {
            "state_not_cleanup_eligible"
        }
        meta::SpireEpochState::Retired if manifest.active_query_count > 0 => "active_queries",
        meta::SpireEpochState::Retired if retained_retired => "retained_retired_epoch",
        meta::SpireEpochState::Retired | meta::SpireEpochState::Failed
            if now_micros < manifest.retain_until_micros =>
        {
            "retention_window"
        }
        meta::SpireEpochState::Retired | meta::SpireEpochState::Failed => "cleanup_plan_retained",
    }
}

fn epoch_snapshot_rows_from_manifests(
    root_control: meta::SpireRootControlState,
    mut manifests: Vec<(crate::storage::page::ItemPointer, meta::SpireEpochManifest)>,
    now_micros: i64,
) -> Result<Vec<SpireIndexEpochSnapshotRow>, String> {
    manifests.sort_by_key(|(tid, manifest)| (manifest.epoch, tid.block_number, tid.offset_number));

    let mut latest_manifest_tid_by_epoch = HashMap::new();
    for (tid, manifest) in &manifests {
        latest_manifest_tid_by_epoch
            .entry(manifest.epoch)
            .and_modify(|latest_tid: &mut crate::storage::page::ItemPointer| {
                if (tid.block_number, tid.offset_number)
                    > (latest_tid.block_number, latest_tid.offset_number)
                {
                    *latest_tid = *tid;
                }
            })
            .or_insert(*tid);
    }
    let latest_manifests = manifests
        .iter()
        .filter_map(|(tid, manifest)| {
            let latest_tid = latest_manifest_tid_by_epoch.get(&manifest.epoch)?;
            if latest_tid == tid {
                Some(*manifest)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let cleanup_plan =
        meta::plan_epoch_cleanup(&latest_manifests, root_control.active_epoch, now_micros)?;
    let cleanup_epochs: HashSet<u64> = cleanup_plan.cleanup_epochs.into_iter().collect();
    let retained_retired_epochs: HashSet<u64> =
        cleanup_plan.retained_retired_epochs.into_iter().collect();

    Ok(manifests
        .into_iter()
        .map(|(tid, manifest)| {
            let is_latest_manifest = latest_manifest_tid_by_epoch
                .get(&manifest.epoch)
                .is_some_and(|latest_tid| latest_tid == &tid);
            let is_active_root_manifest = root_control.active_epoch == manifest.epoch
                && root_control.epoch_manifest_tid == tid;
            let cleanup_eligible_now =
                is_latest_manifest && cleanup_epochs.contains(&manifest.epoch);
            let retained_retired = retained_retired_epochs.contains(&manifest.epoch);
            let cleanup_blocked_reason = if is_active_root_manifest {
                "active_root_manifest"
            } else if is_latest_manifest {
                epoch_cleanup_blocked_reason(
                    &manifest,
                    now_micros,
                    false,
                    retained_retired,
                    cleanup_eligible_now,
                )
            } else {
                "superseded_manifest"
            };
            SpireIndexEpochSnapshotRow {
                active_epoch: root_control.active_epoch,
                epoch: manifest.epoch,
                state: epoch_state_name(manifest.state),
                consistency_mode: consistency_mode_name(manifest.consistency_mode),
                published_at_micros: manifest.published_at_micros,
                retain_until_micros: manifest.retain_until_micros,
                active_query_count: manifest.active_query_count,
                manifest_block: tid.block_number,
                manifest_offset: tid.offset_number,
                is_active_root_manifest,
                cleanup_eligible_now,
                cleanup_blocked_reason,
            }
        })
        .collect())
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

fn apply_leaf_snapshot_base_row(
    rows_by_leaf_pid: &mut HashMap<u64, SpireIndexLeafSnapshotRow>,
    active_epoch: u64,
    header: &storage::SpirePartitionObjectHeader,
    placement: &meta::SpirePlacementEntry,
) {
    let row = rows_by_leaf_pid
        .entry(header.pid)
        .or_insert_with(|| SpireIndexLeafSnapshotRow {
            active_epoch,
            leaf_pid: header.pid,
            parent_pid: header.parent_pid,
            object_version: header.object_version,
            node_id: placement.node_id,
            local_store_id: placement.local_store_id,
            placement_state: placement_state_name(placement.state),
            base_assignment_count: 0,
            delta_object_count: 0,
            delta_insert_assignment_count: 0,
            delta_delete_assignment_count: 0,
            effective_assignment_count: 0,
            split_assignment_threshold: 0,
            merge_assignment_threshold: 0,
            split_recommended: false,
            merge_recommended: false,
            maintenance_action: "none",
            maintenance_reason: "not_evaluated",
            leaf_object_bytes: 0,
            delta_object_bytes: 0,
        });

    row.active_epoch = active_epoch;
    row.leaf_pid = header.pid;
    row.parent_pid = header.parent_pid;
    row.object_version = header.object_version;
    row.node_id = placement.node_id;
    row.local_store_id = placement.local_store_id;
    row.placement_state = placement_state_name(placement.state);
    row.base_assignment_count = u64::from(header.assignment_count);
    row.effective_assignment_count = u64::from(header.assignment_count);
    row.maintenance_action = "none";
    row.maintenance_reason = "not_evaluated";
    row.leaf_object_bytes = u64::from(placement.object_bytes);
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
    }
}
