pub(super) fn plan_leaf_replacement_pids(
    mode: SpireLeafReplacementMode,
    affected_leaf_pids: &[u64],
    replacement_leaf_count: usize,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireLeafReplacementPidPlan, String> {
    validate_affected_leaf_pids(affected_leaf_pids)?;
    if replacement_leaf_count == 0 {
        return Err("ec_spire leaf replacement requires at least one replacement leaf".to_owned());
    }

    let mut cursor = *pid_allocator;
    for pid in affected_leaf_pids {
        cursor.observe(*pid)?;
    }

    let (replacement_pids, reuses_existing_pid) = match mode {
        SpireLeafReplacementMode::Split => {
            if affected_leaf_pids.len() != 1 {
                return Err(
                    "ec_spire split replacement requires exactly one affected leaf pid".to_owned(),
                );
            }
            if replacement_leaf_count < 2 {
                return Err(
                    "ec_spire split replacement requires at least two replacement leaves"
                        .to_owned(),
                );
            }
            (
                allocate_replacement_pids(&mut cursor, replacement_leaf_count)?,
                false,
            )
        }
        SpireLeafReplacementMode::Merge => {
            if replacement_leaf_count != 1 {
                return Err(
                    "ec_spire merge replacement publishes exactly one replacement leaf".to_owned(),
                );
            }
            (vec![cursor.allocate()?], false)
        }
        SpireLeafReplacementMode::Rebalance {
            parent_centroid_byte_equal,
        } => {
            if affected_leaf_pids.len() != 1 || replacement_leaf_count != 1 {
                return Err(
                    "ec_spire rebalance replacement requires exactly one affected leaf and one replacement leaf"
                        .to_owned(),
                );
            }
            if !parent_centroid_byte_equal {
                return Err(
                    "ec_spire rebalance may reuse a pid only when the parent routing centroid is byte-equal"
                        .to_owned(),
                );
            }
            (vec![affected_leaf_pids[0]], true)
        }
    };

    let plan = SpireLeafReplacementPidPlan {
        replacement_pids,
        reuses_existing_pid,
        next_pid: cursor.next_pid(),
    };
    *pid_allocator = cursor;
    Ok(plan)
}

pub(super) fn plan_scheduled_leaf_replacement_pids(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_allocator: &mut SpirePidAllocator,
) -> Result<SpireLeafReplacementPidPlan, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    let mode = match decision.mode {
        SpireLeafReplacementScheduleMode::Split => SpireLeafReplacementMode::Split,
        SpireLeafReplacementScheduleMode::Merge => SpireLeafReplacementMode::Merge,
    };
    plan_leaf_replacement_pids(
        mode,
        &decision.affected_leaf_pids,
        decision.replacement_leaf_count,
        pid_allocator,
    )
}

pub(super) fn choose_leaf_replacement_schedule(
    rows: &[SpireIndexLeafSnapshotRow],
) -> Result<Option<SpireLeafReplacementScheduleDecision>, String> {
    validate_leaf_replacement_schedule_rows(rows)?;
    if let Some(row) = rows
        .iter()
        .filter(|row| row.split_recommended)
        .max_by_key(|row| {
            (
                row.effective_assignment_count,
                std::cmp::Reverse(row.leaf_pid),
            )
        })
    {
        return Ok(Some(SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Split,
            active_epoch: row.active_epoch,
            replaced_parent_pid: row.parent_pid,
            affected_leaf_pids: vec![row.leaf_pid],
            replacement_leaf_count: 2,
            reason: "largest_split_candidate",
        }));
    }

    let mut merge_candidates_by_parent: HashMap<u64, Vec<&SpireIndexLeafSnapshotRow>> =
        HashMap::new();
    for row in rows.iter().filter(|row| row.merge_recommended) {
        merge_candidates_by_parent
            .entry(row.parent_pid)
            .or_default()
            .push(row);
    }
    let mut best_pair: Option<(&SpireIndexLeafSnapshotRow, &SpireIndexLeafSnapshotRow)> = None;
    for candidates in merge_candidates_by_parent.values_mut() {
        if candidates.len() < 2 {
            continue;
        }
        candidates.sort_by_key(|row| (row.effective_assignment_count, row.leaf_pid));
        let pair = (candidates[0], candidates[1]);
        let replace = match best_pair {
            Some(best) => merge_pair_sort_key(pair) < merge_pair_sort_key(best),
            None => true,
        };
        if replace {
            best_pair = Some(pair);
        }
    }
    if let Some((left, right)) = best_pair {
        let mut affected_leaf_pids = vec![left.leaf_pid, right.leaf_pid];
        affected_leaf_pids.sort_unstable();
        return Ok(Some(SpireLeafReplacementScheduleDecision {
            mode: SpireLeafReplacementScheduleMode::Merge,
            active_epoch: left.active_epoch,
            replaced_parent_pid: left.parent_pid,
            affected_leaf_pids,
            replacement_leaf_count: 1,
            reason: "sparsest_same_parent_merge_pair",
        }));
    }

    Ok(None)
}

pub(super) fn recheck_leaf_replacement_schedule_decision(
    rows: &[SpireIndexLeafSnapshotRow],
    expected: &SpireLeafReplacementScheduleDecision,
) -> Result<(), String> {
    validate_leaf_replacement_schedule_decision_shape(expected)?;
    // Keep this recheck in lockstep with the selector: scheduler execution
    // treats selector tie-breaks as part of the publish-lock consistency
    // contract, not just as advisory ranking.
    let Some(observed) = choose_leaf_replacement_schedule(rows)? else {
        return Err("ec_spire replacement scheduler decision is no longer recommended".to_owned());
    };
    if !leaf_replacement_schedule_decisions_match(&observed, expected) {
        return Err(format!(
            "ec_spire replacement scheduler decision changed under publish lock: expected {:?} for pids {:?}, observed {:?} for pids {:?}",
            expected.mode, expected.affected_leaf_pids, observed.mode, observed.affected_leaf_pids
        ));
    }
    Ok(())
}
