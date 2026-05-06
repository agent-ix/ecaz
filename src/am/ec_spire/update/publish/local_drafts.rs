pub(super) fn build_local_scheduled_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    publish_plan: &SpireScheduledReplacementPublishPlan,
    input: SpireLocalScheduledReplacementExecutionInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_local_scheduled_replacement_execution_publish_plan(
        publish_plan,
        pid_plan,
        decision,
        &input,
    )?;
    validate_scheduled_replacement_execution_snapshot(snapshot, decision, publish_plan)?;
    let replacement_object_placements = write_local_scheduled_replacement_objects(
        input.epoch,
        &input.replacement_parent,
        decision,
        &input.replacement_children,
        input.leaf_object_version,
        input.leaf_inputs,
        object_store,
    )?;
    validate_scheduled_replacement_pid_plan_output(
        decision,
        pid_plan,
        &replacement_object_placements,
        pid_plan.next_pid,
    )?;
    build_scheduled_replacement_epoch_draft_from_object_placements(
        snapshot,
        object_store,
        decision,
        SpireScheduledReplacementEpochObjectPlacementInput {
            epoch: input.epoch,
            published_at_micros: input.published_at_micros,
            retain_until_micros: input.retain_until_micros,
            consistency_mode: input.consistency_mode,
            replacement_object_placements,
            placement_write_evidence: input.placement_write_evidence,
            next_pid: pid_plan.next_pid,
            next_local_vec_seq: input.next_local_vec_seq,
        },
    )
}

pub(super) fn build_local_selected_scheduled_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    input: SpireLocalScheduledReplacementExecutionInput,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    validate_local_selected_scheduled_replacement_draft_inputs(snapshot, selected, &input)?;
    build_local_scheduled_replacement_epoch_draft(
        snapshot,
        &selected.decision,
        &selected.lock_plan.pid_plan,
        &selected.lock_plan.publish_plan,
        input,
        object_store,
    )
}

pub(super) fn build_local_selected_scheduled_split_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let input = build_local_selected_scheduled_split_replacement_execution_input(
        selected,
        parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_selected_scheduled_replacement_epoch_draft(snapshot, selected, input, object_store)
}

pub(super) fn build_local_selected_scheduled_split_replacement_epoch_draft_from_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    centroids: Vec<Vec<f32>>,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    build_local_selected_scheduled_split_replacement_epoch_draft(
        snapshot,
        selected,
        &parent,
        centroids,
        routed_leaf_inputs,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
        object_store,
    )
}

pub(super) fn build_local_selected_scheduled_merge_replacement_epoch_draft(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    parent: &SpireRoutingPartitionObject,
    rows: &[SpireIndexLeafSnapshotRow],
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let input = build_local_selected_scheduled_merge_replacement_execution_input(
        selected,
        parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
    )?;
    build_local_selected_scheduled_replacement_epoch_draft(snapshot, selected, input, object_store)
}

pub(super) fn build_local_selected_scheduled_merge_replacement_epoch_draft_from_snapshot(
    snapshot: &SpirePublishedEpochSnapshot<'_>,
    selected: &SpireSelectedScheduledReplacementPublishLockPlan,
    rows: &[SpireIndexLeafSnapshotRow],
    parent_object_version: u64,
    leaf_object_version: u64,
    published_at_micros: i64,
    retain_until_micros: i64,
    placement_write_evidence: Vec<SpirePublishPlacementWriteEvidence>,
    object_store: &mut SpireLocalObjectStore,
) -> Result<SpireReplacementEpochDraft, String> {
    let parent =
        load_selected_scheduled_replacement_parent_routing(snapshot, object_store, selected)?;
    let replacement_leaf_rows =
        collect_selected_scheduled_replacement_leaf_rows(snapshot, object_store, selected)?;
    build_local_selected_scheduled_merge_replacement_epoch_draft(
        snapshot,
        selected,
        &parent,
        rows,
        replacement_leaf_rows,
        parent_object_version,
        leaf_object_version,
        published_at_micros,
        retain_until_micros,
        placement_write_evidence,
        object_store,
    )
}
