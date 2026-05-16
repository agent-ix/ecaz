pub(super) fn build_single_level_leaf_epoch_draft(
    input: SpireSingleLevelBuildInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut impl SpireBuildObjectStore,
) -> Result<SpireSingleLevelBuildDraft, String> {
    let epoch_manifest = SpireEpochManifest {
        epoch: input.epoch,
        state: SpireEpochState::Published,
        consistency_mode: input.consistency_mode,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    let mut pid_cursor = *pid_allocator;
    let mut local_vec_id_cursor = *local_vec_id_allocator;
    let pid = pid_cursor.allocate()?;
    let object_manifest = SpireObjectManifest::from_entries(
        input.epoch,
        vec![SpireManifestEntry {
            epoch: input.epoch,
            pid,
            object_version: input.object_version,
            placement_tid: input.placement_tid,
        }],
    )?;
    let assignments = build_primary_leaf_assignments(&mut local_vec_id_cursor, input.assignments)?;
    let leaf_object = SpireLeafPartitionObject::new(pid, input.object_version, 0, assignments)?;
    let placement = object_store.write_leaf_object_v2_from_rows(
        input.epoch,
        leaf_object.header.pid,
        leaf_object.header.object_version,
        leaf_object.header.parent_pid,
        &leaf_object.assignments,
    )?;
    let placement_directory = SpirePlacementDirectory::from_entries(input.epoch, vec![placement])?;

    let draft = SpireSingleLevelBuildDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        leaf_object,
        next_pid: pid_cursor.next_pid(),
        next_local_vec_seq: local_vec_id_cursor.next_local_vec_seq(),
    };
    SpireValidatedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;

    *pid_allocator = pid_cursor;
    *local_vec_id_allocator = local_vec_id_cursor;
    Ok(draft)
}

pub(super) fn build_partitioned_single_level_leaf_epoch_draft(
    input: SpirePartitionedSingleLevelBuildInput,
    pid_allocator: &mut SpirePidAllocator,
    local_vec_id_allocator: &mut SpireLocalVecIdAllocator,
    object_store: &mut impl SpireBuildObjectStore,
) -> Result<SpirePartitionedSingleLevelBuildDraft, String> {
    let epoch_manifest = SpireEpochManifest {
        epoch: input.epoch,
        state: SpireEpochState::Published,
        consistency_mode: input.consistency_mode,
        published_at_micros: input.published_at_micros,
        retain_until_micros: input.retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    input.centroid_plan.validate()?;
    let centroid_count = input.centroid_plan.centroid_count();
    if input.assignments.len() != input.centroid_plan.assignment_indexes.len() {
        return Err(format!(
            "ec_spire partitioned build assignment count {} does not match centroid assignment count {}",
            input.assignments.len(),
            input.centroid_plan.assignment_indexes.len()
        ));
    }
    if input.placement_tids.len() != centroid_count {
        return Err(format!(
            "ec_spire partitioned build placement count {} does not match centroid count {centroid_count}",
            input.placement_tids.len()
        ));
    }

    let assignments_by_centroid = group_assignments_by_centroid(
        input.assignments,
        &input.centroid_plan.assignment_indexes,
        centroid_count,
    )?;

    let mut pid_cursor = *pid_allocator;
    let mut local_vec_id_cursor = *local_vec_id_allocator;
    let root_pid = pid_cursor.allocate()?;
    let mut centroid_pids = Vec::with_capacity(centroid_count);
    for _ in 0..centroid_count {
        centroid_pids.push(pid_cursor.allocate()?);
    }
    let route_map =
        SpireSingleLevelRouteMap::from_centroid_plan(&input.centroid_plan, &centroid_pids)?;

    let routing_object = SpireRoutingPartitionObject::root(
        root_pid,
        input.object_version,
        input.centroid_plan.dimensions,
        route_map
            .entries
            .iter()
            .map(|entry| SpireRoutingChildEntry {
                centroid_index: entry.centroid_index,
                child_pid: entry.pid,
                centroid: entry.centroid.clone(),
            })
            .collect(),
    )?;

    let mut manifest_entries = Vec::with_capacity(centroid_count + 1);
    manifest_entries.push(SpireManifestEntry {
        epoch: input.epoch,
        pid: root_pid,
        object_version: input.object_version,
        placement_tid: input.root_placement_tid,
    });
    manifest_entries.extend(centroid_pids.iter().zip(input.placement_tids.iter()).map(
        |(&pid, &placement_tid)| SpireManifestEntry {
            epoch: input.epoch,
            pid,
            object_version: input.object_version,
            placement_tid,
        },
    ));
    let object_manifest = SpireObjectManifest::from_entries(input.epoch, manifest_entries)?;

    let mut leaf_objects = Vec::with_capacity(centroid_count);
    for (pid, assignments) in centroid_pids
        .iter()
        .copied()
        .zip(assignments_by_centroid.into_iter())
    {
        let assignments = build_primary_leaf_assignments(&mut local_vec_id_cursor, assignments)?;
        let leaf_object =
            SpireLeafPartitionObject::new(pid, input.object_version, root_pid, assignments)?;
        leaf_objects.push(leaf_object);
    }
    let mut placements = Vec::with_capacity(centroid_count + 1);
    placements.push(object_store.write_routing_object(input.epoch, &routing_object)?);
    for leaf_object in &leaf_objects {
        placements.push(object_store.write_leaf_object_v2_from_rows(
            input.epoch,
            leaf_object.header.pid,
            leaf_object.header.object_version,
            leaf_object.header.parent_pid,
            &leaf_object.assignments,
        )?);
    }
    let placement_directory = SpirePlacementDirectory::from_entries(input.epoch, placements)?;

    let draft = SpirePartitionedSingleLevelBuildDraft {
        epoch_manifest,
        object_manifest,
        placement_directory,
        route_map,
        root_pid,
        routing_object,
        centroid_pids,
        leaf_objects,
        next_pid: pid_cursor.next_pid(),
        next_local_vec_seq: local_vec_id_cursor.next_local_vec_seq(),
    };
    SpireValidatedEpochSnapshot::new(
        &draft.epoch_manifest,
        &draft.object_manifest,
        &draft.placement_directory,
    )?;

    *pid_allocator = pid_cursor;
    *local_vec_id_allocator = local_vec_id_cursor;
    Ok(draft)
}

fn group_assignments_by_centroid(
    assignments: Vec<SpireLeafAssignmentInput>,
    assignment_indexes: &[u32],
    centroid_count: usize,
) -> Result<Vec<Vec<SpireLeafAssignmentInput>>, String> {
    if assignments.len() != assignment_indexes.len() {
        return Err(format!(
            "ec_spire centroid assignment count {} does not match assignment index count {}",
            assignments.len(),
            assignment_indexes.len()
        ));
    }
    let mut assignments_by_centroid = vec![Vec::new(); centroid_count];
    for (assignment, assignment_index) in assignments.into_iter().zip(assignment_indexes.iter()) {
        let centroid_index = usize::try_from(*assignment_index)
            .map_err(|_| "ec_spire centroid assignment index exceeds usize".to_owned())?;
        let assignments = assignments_by_centroid.get_mut(centroid_index).ok_or_else(|| {
            format!(
                "ec_spire centroid assignment index {centroid_index} exceeds centroid count {centroid_count}"
            )
        })?;
        assignments.push(assignment);
    }
    Ok(assignments_by_centroid)
}

unsafe fn publish_relation_partitioned_single_level_build(
    index_relation: pg_sys::Relation,
    state: &SpireBuildState,
    local_store_config: SpireLocalStoreConfig,
) -> Result<usize, String> {
    if state.scanned_tuples == 0 {
        return Ok(0);
    }

    let (published_at_micros, retain_until_micros) = unsafe { current_epoch_publish_times()? };
    let epoch_manifest = SpireEpochManifest {
        epoch: SPIRE_INITIAL_EPOCH,
        state: SpireEpochState::Published,
        consistency_mode: SpireConsistencyMode::Strict,
        published_at_micros,
        retain_until_micros,
        active_query_count: 0,
    };
    epoch_manifest.validate()?;

    let centroid_plan = state.train_centroid_plan()?;
    let assignments = state.assignment_inputs();
    let centroid_count = centroid_plan.centroid_count();
    if assignments.len() != centroid_plan.assignment_indexes.len() {
        return Err(format!(
            "ec_spire populated build assignment count {} does not match centroid assignment count {}",
            assignments.len(),
            centroid_plan.assignment_indexes.len()
        ));
    }

    let mut pid_allocator = SpirePidAllocator::default();
    let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
    let root_pid = pid_allocator.allocate()?;
    let mut centroid_pids = Vec::with_capacity(centroid_count);
    for _ in 0..centroid_count {
        centroid_pids.push(pid_allocator.allocate()?);
    }
    let route_map = SpireSingleLevelRouteMap::from_centroid_plan(&centroid_plan, &centroid_pids)?;
    let routing_object = SpireRoutingPartitionObject::root(
        root_pid,
        SPIRE_INITIAL_OBJECT_VERSION,
        centroid_plan.dimensions,
        route_map
            .entries
            .iter()
            .map(|entry| SpireRoutingChildEntry {
                centroid_index: entry.centroid_index,
                child_pid: entry.pid,
                centroid: entry.centroid.clone(),
            })
            .collect(),
    )?;

    let mut leaf_assignments_by_centroid = Vec::with_capacity(centroid_count);
    leaf_assignments_by_centroid.resize_with(centroid_count, Vec::new);
    let boundary_replica_count = u32::try_from(state.options.boundary_replica_count)
        .map_err(|_| "ec_spire boundary_replica_count reloption must be non-negative".to_owned())?;
    for placement in build_boundary_leaf_assignment_placements_with_identity(
        &mut local_vec_id_allocator,
        plan_boundary_assignment_identity_inputs(state, &route_map, boundary_replica_count)?,
    )? {
        let centroid_index = centroid_pids
            .iter()
            .position(|pid| *pid == placement.pid)
            .ok_or_else(|| {
                format!(
                    "ec_spire boundary assignment resolved unknown leaf pid {}",
                    placement.pid
                )
            })?;
        leaf_assignments_by_centroid[centroid_index].push(placement.row);
    }

    let mut store = unsafe {
        SpireRelationObjectStoreSet::for_index_relation_and_config(
            index_relation,
            local_store_config.clone(),
            pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        )?
    };
    let mut placements = Vec::with_capacity(centroid_count + 1);
    placements.push(store.insert_routing_object(SPIRE_INITIAL_EPOCH, &routing_object)?);
    for (pid, assignments) in centroid_pids
        .iter()
        .copied()
        .zip(leaf_assignments_by_centroid.iter())
    {
        placements.push(store.insert_leaf_object_v2_from_rows(
            SPIRE_INITIAL_EPOCH,
            pid,
            SPIRE_INITIAL_OBJECT_VERSION,
            root_pid,
            assignments,
        )?);
    }
    let placement_directory =
        SpirePlacementDirectory::from_entries(SPIRE_INITIAL_EPOCH, placements)?;
    let placement_evidence =
        unsafe { write_placement_entries_to_relation(index_relation, &placement_directory)? };
    let object_manifest = object_manifest_from_placement_writes(
        SPIRE_INITIAL_EPOCH,
        &placement_directory,
        &placement_evidence,
    )?;

    let input = SpirePublishCoordinatorInput {
        epoch_manifest: &epoch_manifest,
        object_manifest: &object_manifest,
        placement_directory: &placement_directory,
        local_store_config,
        next_pid: pid_allocator.next_pid(),
        next_local_vec_seq: local_vec_id_allocator.next_local_vec_seq(),
    };
    let manifests = encode_manifest_bundle_for_publish(input.clone())?;
    let locators = unsafe { write_manifest_bundle_to_relation(index_relation, &manifests)? };
    let root_control = root_control_state_for_publish(input, locators)?;
    unsafe { page::initialize_root_control_page(index_relation, root_control) };
    Ok(state.scanned_tuples)
}

fn plan_boundary_assignment_identity_inputs(
    state: &SpireBuildState,
    route_map: &SpireSingleLevelRouteMap,
    boundary_replica_count: u32,
) -> Result<Vec<SpireBoundaryLeafAssignmentIdentityInput>, String> {
    state
        .tuples
        .iter()
        .map(|tuple| {
            let plan = route_map.route_boundary_assignment_for_vector(
                &tuple.source_vector,
                boundary_replica_count,
            )?;
            Ok(SpireBoundaryLeafAssignmentIdentityInput {
                primary_pid: plan.primary_pid,
                replica_pids: plan.replica_pids,
                assignment: SpireLeafAssignmentIdentityInput {
                    assignment: tuple.assignment.clone(),
                    vec_id_source_identity: tuple.vec_id_source_identity.clone(),
                },
            })
        })
        .collect()
}

unsafe fn publish_relation_recursive_routing_build(
    index_relation: pg_sys::Relation,
    state: &SpireBuildState,
    target_fanout: u32,
    local_store_config: SpireLocalStoreConfig,
) -> Result<usize, String> {
    if state.scanned_tuples == 0 {
        return Ok(0);
    }

    let (published_at_micros, retain_until_micros) = unsafe { current_epoch_publish_times()? };
    let centroid_plan = state.train_centroid_plan()?;
    let mut pid_allocator = SpirePidAllocator::default();
    let mut local_vec_id_allocator = SpireLocalVecIdAllocator::default();
    let coordinator = build_recursive_epoch_input_from_centroid_plan(
        SpireRecursiveBuildCoordinatorInput {
            epoch: SPIRE_INITIAL_EPOCH,
            object_version: SPIRE_INITIAL_OBJECT_VERSION,
            published_at_micros,
            retain_until_micros,
            consistency_mode: SpireConsistencyMode::Strict,
            target_fanout,
            seed: state.options.seed as u64,
            boundary_replica_count: u32::try_from(state.options.boundary_replica_count).map_err(
                |_| "ec_spire boundary_replica_count reloption must be non-negative".to_owned(),
            )?,
            assignments: state.assignment_identity_inputs(),
            source_vectors: state.source_vectors(),
            centroid_plan,
        },
        &mut pid_allocator,
        &mut local_vec_id_allocator,
    )?;
    let mut store = unsafe {
        SpireRelationObjectStoreSet::for_index_relation_and_config(
            index_relation,
            local_store_config.clone(),
            pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        )?
    };
    let top_graph_plan = state.options.top_graph_plan()?;
    let expected_next_pid = if top_graph_plan.enabled {
        coordinator
            .next_pid
            .checked_add(1)
            .ok_or_else(|| "ec_spire recursive top graph next_pid overflow".to_owned())?
    } else {
        coordinator.next_pid
    };
    let draft = if top_graph_plan.enabled {
        build_recursive_top_graph_epoch_from_leaf_inputs_with_store(
            coordinator.epoch_input,
            SpireTopGraphBuildParams {
                graph_degree: top_graph_plan.graph_degree,
                build_list_size: top_graph_plan.build_list_size,
                alpha: top_graph_plan.alpha,
                seed: state.options.seed as u64,
            },
            &mut store,
        )?
    } else {
        build_recursive_routing_epoch_from_leaf_inputs_with_store(
            coordinator.epoch_input,
            &mut store,
        )?
    };
    if draft.next_pid != expected_next_pid {
        return Err(format!(
            "ec_spire recursive relation build next_pid {} does not match expected next_pid {}",
            draft.next_pid, expected_next_pid
        ));
    }
    unsafe {
        publish_relation_recursive_routing_epoch_draft(
            index_relation,
            &draft,
            coordinator.next_local_vec_seq,
            local_store_config,
        )?
    };
    Ok(state.scanned_tuples)
}

pub(super) unsafe fn current_epoch_publish_times() -> Result<(i64, i64), String> {
    let published_at_micros = unsafe { pg_sys::GetCurrentTimestamp() };
    let retention_micros = i64::from(SPIRE_MIN_EPOCH_RETENTION_SECS)
        .checked_mul(MICROS_PER_SECOND)
        .ok_or_else(|| "ec_spire epoch retention micros overflow".to_owned())?;
    let retain_until_micros = published_at_micros
        .checked_add(retention_micros)
        .ok_or_else(|| "ec_spire epoch retain_until timestamp overflow".to_owned())?;
    Ok((published_at_micros, retain_until_micros))
}

pub(super) unsafe fn build_spire_index_tuple(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: ItemPointer,
    tuple_layout: SpireIndexedTupleLayout,
    payload_format: SpireAssignmentPayloadFormat,
    context: &str,
) -> SpireBuildTuple {
    if values.is_null() || isnull.is_null() {
        pgrx::error!("ec_spire {context} received null tuple value arrays");
    }
    if unsafe { *isnull } {
        pgrx::error!("ec_spire does not support NULL indexed values");
    }

    let datum = unsafe { *values };
    if datum.is_null() {
        pgrx::error!("ec_spire {context} received a null indexed datum");
    }

    let bytes = unsafe { detoasted_varlena_bytes(datum, "indexed vector column") };
    let vec_id_source_identity = unsafe {
        build_source_identity_from_tuple_values(values, isnull, tuple_layout.source_identity, context)
    };
    match tuple_layout.vector_kind {
        SpireIndexedVectorKind::Ecvector => {
            build_spire_ecvector_tuple(
                heap_tid,
                &bytes,
                payload_format,
                vec_id_source_identity,
                context,
            )
        }
        SpireIndexedVectorKind::Tqvector => {
            build_spire_tqvector_tuple(
                heap_tid,
                &bytes,
                payload_format,
                vec_id_source_identity,
                context,
            )
        }
    }
}

unsafe fn build_source_identity_from_tuple_values(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    source_identity: Option<SpireSourceIdentityAttribute>,
    context: &str,
) -> SpireVecIdSourceIdentity {
    let Some(source_identity) = source_identity else {
        return SpireVecIdSourceIdentity::AllocateLocal;
    };
    let offset = source_identity.index_attr_offset;
    if unsafe { *isnull.add(offset) } {
        pgrx::error!("ec_spire {context} source_identity INCLUDE column must not be NULL");
    }
    let datum = unsafe { *values.add(offset) };
    if datum.is_null() {
        pgrx::error!("ec_spire {context} received a null source_identity datum");
    }

    let payload = match source_identity.datum_kind {
        SpireSourceIdentityDatumKind::Uuid => unsafe { uuid_source_identity_payload(datum) },
        SpireSourceIdentityDatumKind::Bytea16 => unsafe {
            let bytes = detoasted_varlena_bytes(datum, "source_identity INCLUDE column");
            <[u8; SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES]>::try_from(bytes.as_slice())
                .unwrap_or_else(|_| {
                    pgrx::error!(
                        "ec_spire {context} source_identity bytea payload length {} must be {} bytes",
                        bytes.len(),
                        SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES
                    )
                })
        },
    };
    SpireVecIdSourceIdentity::stable_fixed_global_payload(payload)
}

unsafe fn uuid_source_identity_payload(
    datum: pg_sys::Datum,
) -> [u8; SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES] {
    let bytes = unsafe {
        std::slice::from_raw_parts(
            datum.cast_mut_ptr::<u8>(),
            SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES,
        )
    };
    let mut payload = [0_u8; SPIRE_STABLE_GLOBAL_SOURCE_ID_PAYLOAD_BYTES];
    payload.copy_from_slice(bytes);
    payload
}
