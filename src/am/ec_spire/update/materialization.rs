pub(super) fn build_merge_replacement_leaf_object_input(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    replacement_leaf_rows: Vec<SpireReplacementLeafRows>,
) -> Result<SpireReplacementLeafObjectInput, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Merge {
        return Err("ec_spire merge replacement leaf input requires a merge decision".to_owned());
    }
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire merge replacement leaf input requires fresh replacement pid".to_owned(),
        );
    }
    let [replacement_pid] = pid_plan.replacement_pids.as_slice() else {
        return Err(
            "ec_spire merge replacement leaf input requires exactly one replacement pid".to_owned(),
        );
    };
    if *replacement_pid >= pid_plan.next_pid {
        return Err(format!(
            "ec_spire merge replacement leaf input pid plan next_pid {} does not advance past replacement pid {replacement_pid}",
            pid_plan.next_pid
        ));
    }

    let affected: HashSet<u64> = decision.affected_leaf_pids.iter().copied().collect();
    let mut rows_by_base_pid = HashMap::new();
    for leaf_rows in replacement_leaf_rows {
        if !affected.contains(&leaf_rows.base_pid) {
            return Err(format!(
                "ec_spire merge replacement leaf input got rows for unselected base pid {}",
                leaf_rows.base_pid
            ));
        }
        if rows_by_base_pid
            .insert(leaf_rows.base_pid, leaf_rows.rows)
            .is_some()
        {
            return Err(format!(
                "ec_spire merge replacement leaf input got duplicate rows for base pid {}",
                leaf_rows.base_pid
            ));
        }
    }

    let mut rows = Vec::new();
    for base_pid in &decision.affected_leaf_pids {
        let Some(mut leaf_rows) = rows_by_base_pid.remove(base_pid) else {
            return Err(format!(
                "ec_spire merge replacement leaf input missing rows for base pid {base_pid}"
            ));
        };
        rows.append(&mut leaf_rows);
    }
    let input = SpireReplacementLeafObjectInput {
        pid: *replacement_pid,
        rows,
    };
    // Leaf-input validation only needs the replacement PID; centroid shape is
    // checked later when scheduler-built routing children are available.
    validate_replacement_leaf_object_inputs(
        &[SpireRoutingReplacementChild {
            child_pid: input.pid,
            centroid: Vec::new(),
        }],
        std::slice::from_ref(&input),
    )?;
    Ok(input)
}

pub(super) fn build_split_replacement_leaf_object_inputs(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    routed_leaf_inputs: Vec<SpireReplacementLeafObjectInput>,
) -> Result<Vec<SpireReplacementLeafObjectInput>, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Split {
        return Err("ec_spire split replacement leaf inputs require a split decision".to_owned());
    }
    if pid_plan.reuses_existing_pid {
        return Err(
            "ec_spire split replacement leaf inputs require fresh replacement pids".to_owned(),
        );
    }
    if pid_plan.replacement_pids.len() != decision.replacement_leaf_count {
        return Err(format!(
            "ec_spire split replacement leaf input pid count {} does not match decision replacement count {}",
            pid_plan.replacement_pids.len(),
            decision.replacement_leaf_count
        ));
    }
    if let Some(unadvanced_pid) = pid_plan
        .replacement_pids
        .iter()
        .copied()
        .find(|pid| *pid >= pid_plan.next_pid)
    {
        return Err(format!(
            "ec_spire split replacement leaf input pid plan next_pid {} does not advance past replacement pid {unadvanced_pid}",
            pid_plan.next_pid
        ));
    }
    if routed_leaf_inputs.len() != pid_plan.replacement_pids.len() {
        return Err(format!(
            "ec_spire split replacement leaf input count {} does not match replacement pid count {}",
            routed_leaf_inputs.len(),
            pid_plan.replacement_pids.len()
        ));
    }

    let children = pid_plan
        .replacement_pids
        .iter()
        .map(|pid| SpireRoutingReplacementChild {
            child_pid: *pid,
            // Leaf-input validation only needs the replacement PID; centroid
            // shape is checked later against scheduler-built routing children.
            centroid: Vec::new(),
        })
        .collect::<Vec<_>>();
    validate_replacement_leaf_object_inputs(&children, &routed_leaf_inputs)?;

    let mut inputs_by_pid = routed_leaf_inputs
        .into_iter()
        .map(|input| (input.pid, input))
        .collect::<HashMap<_, _>>();
    let mut ordered = Vec::with_capacity(pid_plan.replacement_pids.len());
    for pid in &pid_plan.replacement_pids {
        let input = inputs_by_pid.remove(pid).ok_or_else(|| {
            format!("ec_spire split replacement leaf input missing replacement pid {pid}")
        })?;
        ordered.push(input);
    }
    Ok(ordered)
}

pub(super) fn build_split_replacement_source_rows(
    decision: &SpireLeafReplacementScheduleDecision,
    replacement_rows: Vec<SpireReplacementLeafRows>,
    fetched_sources: Vec<SpireSplitReplacementFetchedSourceVector>,
) -> Result<Vec<SpireSplitReplacementSourceRow>, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Split {
        return Err("ec_spire split replacement source rows require a split decision".to_owned());
    }
    let [affected_pid] = decision.affected_leaf_pids.as_slice() else {
        return Err(
            "ec_spire split replacement source rows require one affected leaf pid".to_owned(),
        );
    };
    let [replacement_row_group] = replacement_rows.as_slice() else {
        return Err(
            "ec_spire split replacement source rows require one affected row group".to_owned(),
        );
    };
    if replacement_row_group.base_pid != *affected_pid {
        return Err(format!(
            "ec_spire split replacement source rows got row group for unselected base pid {}",
            replacement_row_group.base_pid
        ));
    }
    if replacement_row_group.rows.is_empty() {
        return Err("ec_spire split replacement source rows require assignment rows".to_owned());
    }

    let mut sources_by_heap_tid = HashMap::with_capacity(fetched_sources.len());
    for fetched in fetched_sources {
        if fetched.heap_tid == ItemPointer::INVALID {
            return Err(
                "ec_spire split replacement source rows require valid heap tids".to_owned(),
            );
        }
        if sources_by_heap_tid
            .insert(fetched.heap_tid, fetched.source_vector)
            .is_some()
        {
            return Err(format!(
                "ec_spire split replacement source rows got duplicate source heap tid {}:{}",
                fetched.heap_tid.block_number, fetched.heap_tid.offset_number
            ));
        }
    }

    let mut seen_assignment_heap_tids = HashSet::new();
    let mut source_rows = Vec::with_capacity(replacement_row_group.rows.len());
    for assignment in &replacement_row_group.rows {
        if !seen_assignment_heap_tids.insert(assignment.heap_tid) {
            return Err(format!(
                "ec_spire split replacement source rows got duplicate assignment heap tid {}:{}",
                assignment.heap_tid.block_number, assignment.heap_tid.offset_number
            ));
        }
        let Some(source_vector) = sources_by_heap_tid.remove(&assignment.heap_tid) else {
            return Err(format!(
                "ec_spire split replacement source rows missing source vector for heap tid {}:{}",
                assignment.heap_tid.block_number, assignment.heap_tid.offset_number
            ));
        };
        source_rows.push(SpireSplitReplacementSourceRow {
            base_pid: replacement_row_group.base_pid,
            assignment: assignment.clone(),
            source_vector,
        });
    }

    if !sources_by_heap_tid.is_empty() {
        let mut unused_heap_tids = sources_by_heap_tid.keys().copied().collect::<Vec<_>>();
        unused_heap_tids.sort_by_key(|tid| (tid.block_number, tid.offset_number));
        let first = unused_heap_tids[0];
        return Err(format!(
            "ec_spire split replacement source rows got unused source vector for heap tid {}:{}",
            first.block_number, first.offset_number
        ));
    }

    Ok(source_rows)
}

pub(super) fn build_split_replacement_leaf_materialization_from_rows(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    replacement_rows: Vec<SpireReplacementLeafRows>,
    fetched_sources: Vec<SpireSplitReplacementFetchedSourceVector>,
    boundary_replica_count: u32,
    dimensions: usize,
    seed: u64,
    max_iterations: usize,
) -> Result<SpireSplitReplacementMaterialization, String> {
    let source_rows =
        build_split_replacement_source_rows(decision, replacement_rows, fetched_sources)?;
    build_split_replacement_leaf_materialization(
        decision,
        pid_plan,
        source_rows,
        boundary_replica_count,
        dimensions,
        seed,
        max_iterations,
    )
}

fn filter_split_replacement_rows_to_fetched_sources(
    replacement_rows: Vec<SpireReplacementLeafRows>,
    fetched_sources: &[SpireSplitReplacementFetchedSourceVector],
) -> Result<Vec<SpireReplacementLeafRows>, String> {
    let mut fetched_heap_tids = HashSet::with_capacity(fetched_sources.len());
    for fetched in fetched_sources {
        if fetched.heap_tid == ItemPointer::INVALID {
            return Err(
                "ec_spire split replacement fetched source rows require valid heap tids".to_owned(),
            );
        }
        if !fetched_heap_tids.insert(fetched.heap_tid) {
            return Err(format!(
                "ec_spire split replacement fetched source rows got duplicate heap tid {}:{}",
                fetched.heap_tid.block_number, fetched.heap_tid.offset_number
            ));
        }
    }

    Ok(replacement_rows
        .into_iter()
        .map(|row_group| SpireReplacementLeafRows {
            base_pid: row_group.base_pid,
            rows: row_group
                .rows
                .into_iter()
                .filter(|assignment| fetched_heap_tids.contains(&assignment.heap_tid))
                .collect(),
        })
        .collect())
}

pub(super) unsafe fn fetch_split_replacement_source_vectors(
    heap_relation: pgrx::pg_sys::Relation,
    snapshot: pgrx::pg_sys::Snapshot,
    slot: *mut pgrx::pg_sys::TupleTableSlot,
    indexed_attribute: source::IndexedVectorAttribute,
    replacement_rows: &[SpireReplacementLeafRows],
) -> Result<Vec<SpireSplitReplacementFetchedSourceVector>, String> {
    let row_count = replacement_rows
        .iter()
        .map(|row_group| row_group.rows.len())
        .sum();
    let mut fetched_sources = Vec::with_capacity(row_count);
    for row_group in replacement_rows {
        for assignment in &row_group.rows {
            let Some(source_vector) = unsafe {
                load_indexed_source_vector_from_heap_row(
                    heap_relation,
                    snapshot,
                    slot,
                    indexed_attribute,
                    assignment.heap_tid,
                    "ec_spire split replacement source vector",
                )
            }?
            else {
                // Heap rows that are no longer visible are omitted here; the
                // heap-source wrapper drops their assignment rows before
                // materialization so exact source coverage still holds for
                // the live row set.
                continue;
            };
            fetched_sources.push(SpireSplitReplacementFetchedSourceVector {
                heap_tid: assignment.heap_tid,
                source_vector,
            });
        }
    }
    Ok(fetched_sources)
}

pub(super) fn build_split_replacement_leaf_materialization(
    decision: &SpireLeafReplacementScheduleDecision,
    pid_plan: &SpireLeafReplacementPidPlan,
    source_rows: Vec<SpireSplitReplacementSourceRow>,
    boundary_replica_count: u32,
    dimensions: usize,
    seed: u64,
    max_iterations: usize,
) -> Result<SpireSplitReplacementMaterialization, String> {
    validate_leaf_replacement_schedule_decision_shape(decision)?;
    if decision.mode != SpireLeafReplacementScheduleMode::Split {
        return Err(
            "ec_spire split replacement materialization requires a split decision".to_owned(),
        );
    }
    let [affected_pid] = decision.affected_leaf_pids.as_slice() else {
        return Err(
            "ec_spire split replacement materialization requires one affected leaf pid".to_owned(),
        );
    };
    if dimensions == 0 {
        return Err("ec_spire split replacement materialization dimensions must be > 0".to_owned());
    }
    if max_iterations == 0 {
        return Err(
            "ec_spire split replacement materialization requires at least one training iteration"
                .to_owned(),
        );
    }
    if source_rows.is_empty() {
        return Err("ec_spire split replacement materialization requires source rows".to_owned());
    }

    for source_row in &source_rows {
        if source_row.base_pid != *affected_pid {
            return Err(format!(
                "ec_spire split replacement materialization got source row for unselected base pid {}",
                source_row.base_pid
            ));
        }
        if !is_visible_primary_assignment(&source_row.assignment) {
            return Err(
                "ec_spire split replacement materialization requires visible primary rows"
                    .to_owned(),
            );
        }
        if source_row.assignment.flags & SPIRE_ASSIGNMENT_FLAG_DELTA_INSERT != 0 {
            return Err(
                "ec_spire split replacement materialization requires normalized base rows"
                    .to_owned(),
            );
        }
    }

    let source_refs = source_rows
        .iter()
        .map(|row| row.source_vector.as_slice())
        .collect::<Vec<_>>();
    let model = common_training::train_spherical_kmeans(
        "ec_spire split replacement materialization",
        &source_refs,
        dimensions,
        decision.replacement_leaf_count,
        seed,
        max_iterations,
    )?;

    let mut routed_inputs = pid_plan
        .replacement_pids
        .iter()
        .map(|pid| SpireReplacementLeafObjectInput {
            pid: *pid,
            rows: Vec::new(),
        })
        .collect::<Vec<_>>();
    for source_row in source_rows {
        let routed_indexes = route_split_replacement_boundary_indexes(
            &model.centroids,
            &pid_plan.replacement_pids,
            &source_row.source_vector,
            boundary_replica_count,
        )?;
        for (route_offset, centroid_index) in routed_indexes.into_iter().enumerate() {
            let Some(input) = routed_inputs.get_mut(centroid_index) else {
                return Err(
                    "ec_spire split replacement materialization centroid index out of bounds"
                        .to_owned(),
                );
            };
            let flags = if route_offset == 0 {
                SPIRE_ASSIGNMENT_FLAG_PRIMARY
            } else {
                SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA
            };
            input
                .rows
                .push(split_replacement_assignment_with_flags(&source_row.assignment, flags)?);
        }
    }

    let leaf_inputs =
        build_split_replacement_leaf_object_inputs(decision, pid_plan, routed_inputs)?;
    Ok(SpireSplitReplacementMaterialization {
        centroids: model.centroids,
        leaf_inputs,
    })
}

fn route_split_replacement_boundary_indexes(
    centroids: &[Vec<f32>],
    replacement_pids: &[u64],
    source_vector: &[f32],
    boundary_replica_count: u32,
) -> Result<Vec<usize>, String> {
    if centroids.is_empty() {
        return Err(
            "ec_spire split replacement boundary routing requires centroids".to_owned(),
        );
    }
    if replacement_pids.len() != centroids.len() {
        return Err(format!(
            "ec_spire split replacement boundary routing pid count {} does not match centroid count {}",
            replacement_pids.len(),
            centroids.len()
        ));
    }
    if centroids.len() > u32::MAX as usize {
        return Err(
            "ec_spire split replacement boundary routing centroid count exceeds u32".to_owned(),
        );
    }
    let ranked = rank_centroid_routes_by_ip(
        "ec_spire split replacement boundary routing",
        source_vector,
        source_vector.len(),
        centroids
        .iter()
        .enumerate()
        .zip(replacement_pids.iter().copied())
        .map(|((index, centroid), pid)| SpireCentroidRouteInput {
            centroid_index: index as u32,
            pid,
            centroid,
        }),
    )?;
    let limit = usize::try_from(boundary_replica_count)
        .unwrap_or(usize::MAX)
        .saturating_add(1)
        .min(ranked.len());
    let selected = ranked
        .into_iter()
        .take(limit)
        .map(|route| {
            usize::try_from(route.centroid_index).map_err(|_| {
                "ec_spire split replacement boundary routing centroid index exceeds usize"
                    .to_owned()
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(selected)
}

fn split_replacement_assignment_with_flags(
    assignment: &SpireLeafAssignmentRow,
    flags: u16,
) -> Result<SpireLeafAssignmentRow, String> {
    let mut row = assignment.clone();
    row.flags = flags;
    Ok(row)
}
