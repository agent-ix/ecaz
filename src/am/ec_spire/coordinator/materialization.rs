#[derive(Debug, Clone)]
struct SpireRemoteLeafMaterializationInputRow {
    leaf_pid: u64,
    parent_pid: u64,
    object_version: u64,
    row_index: u32,
    assignment: storage::SpireLeafAssignmentRow,
}

pub(crate) fn materialize_static_remote_leaf_assignments(
    index_oid: pg_sys::Oid,
    target_epoch: u64,
    leaf_pids: Vec<u64>,
    parent_pids: Vec<u64>,
    object_versions: Vec<u64>,
    row_indices: Vec<u32>,
    assignment_flags: Vec<u16>,
    vec_ids: Vec<Vec<u8>>,
    heap_blocks: Vec<u32>,
    heap_offsets: Vec<u16>,
    payload_formats: Vec<u8>,
    gammas: Vec<f32>,
    encoded_payloads: Vec<Vec<u8>>,
) -> SpireRemoteLeafMaterializationSummaryRow {
    let lockmode = pg_sys::RowExclusiveLock as pg_sys::LOCKMODE;
    let index_relation = crate::storage::relation_guard::IndexRelationGuard::open(
        index_oid,
        lockmode,
        "ec_spire static remote leaf materialization",
    );
    let result = unsafe {
        (|| -> Result<SpireRemoteLeafMaterializationSummaryRow, String> {
            let rows = static_remote_leaf_materialization_rows(
                leaf_pids,
                parent_pids,
                object_versions,
                row_indices,
                assignment_flags,
                vec_ids,
                heap_blocks,
                heap_offsets,
                payload_formats,
                gammas,
                encoded_payloads,
            )?;
            let root_control = page::read_root_control_page(index_relation.as_ptr());
            if target_epoch == 0 {
                return Err("ec_spire remote leaf materialization target_epoch must be greater than zero".to_owned());
            }
            let previous_epoch = if root_control.active_epoch == 0 {
                None
            } else {
                let index = live_index_relation(index_relation.as_ptr());
                let (epoch_manifest, _object_manifest, _placement_directory) =
                    load_relation_epoch_manifests_for_coordinator_fanout(index, root_control)?;
                Some(epoch_manifest)
            };
            if root_control.active_epoch > target_epoch {
                return Err(format!(
                    "ec_spire remote leaf materialization target_epoch {target_epoch} is behind remote active epoch {}",
                    root_control.active_epoch
                ));
            }
            let publish_epoch = target_epoch;
            let store =
                storage::SpireRelationObjectStore::for_index_relation(index_relation.as_ptr())?;

            let mut grouped: BTreeMap<u64, Vec<SpireRemoteLeafMaterializationInputRow>> =
                BTreeMap::new();
            for row in rows {
                grouped.entry(row.leaf_pid).or_default().push(row);
            }

            let mut placements = Vec::with_capacity(grouped.len());
            let mut assignment_count = 0_u64;
            let mut max_pid = 0_u64;
            let mut max_local_vec_seq = 0_u64;
            for (leaf_pid, mut leaf_rows) in grouped {
                leaf_rows.sort_by_key(|row| row.row_index);
                let parent_pid = leaf_rows[0].parent_pid;
                let object_version = leaf_rows[0].object_version;
                for (expected_index, row) in leaf_rows.iter().enumerate() {
                    let expected_index = u32::try_from(expected_index).map_err(|_| {
                        "ec_spire remote leaf materialization row index exceeds u32".to_owned()
                    })?;
                    if row.row_index != expected_index {
                        return Err(format!(
                            "ec_spire remote leaf materialization leaf pid {leaf_pid} row_index {} is not contiguous from zero at {expected_index}",
                            row.row_index
                        ));
                    }
                    if row.parent_pid != parent_pid {
                        return Err(format!(
                            "ec_spire remote leaf materialization leaf pid {leaf_pid} has mixed parent_pid values"
                        ));
                    }
                    if row.object_version != object_version {
                        return Err(format!(
                            "ec_spire remote leaf materialization leaf pid {leaf_pid} has mixed object_version values"
                        ));
                    }
                    if let Some(local_seq) = row.assignment.vec_id.local_sequence() {
                        max_local_vec_seq = max_local_vec_seq.max(local_seq);
                    }
                }

                let assignments = leaf_rows
                    .into_iter()
                    .map(|row| row.assignment)
                    .collect::<Vec<_>>();
                assignment_count = assignment_count
                    .checked_add(u64::try_from(assignments.len()).map_err(|_| {
                        "ec_spire remote leaf materialization assignment count exceeds u64"
                            .to_owned()
                    })?)
                    .ok_or_else(|| {
                        "ec_spire remote leaf materialization assignment count overflow".to_owned()
                    })?;
                max_pid = max_pid.max(leaf_pid);
                placements.push(store.insert_leaf_object_v2_from_rows(
                    publish_epoch,
                    leaf_pid,
                    object_version,
                    parent_pid,
                    &assignments,
                )?);
            }

            let placement_directory =
                meta::SpirePlacementDirectory::from_entries(publish_epoch, placements)?;
            let placement_evidence = build::write_placement_entries_to_relation(
                index_relation.as_ptr(),
                &placement_directory,
            )?;
            let object_manifest = build::object_manifest_from_placement_writes(
                publish_epoch,
                &placement_directory,
                &placement_evidence,
            )?;
            let (published_at_micros, retain_until_micros) = build::current_epoch_publish_times()?;
            let epoch_manifest = meta::SpireEpochManifest {
                epoch: publish_epoch,
                state: meta::SpireEpochState::Published,
                consistency_mode: meta::SpireConsistencyMode::Strict,
                published_at_micros,
                retain_until_micros,
                active_query_count: 0,
            };
            let next_pid = root_control
                .next_pid
                .max(max_pid.checked_add(1).ok_or_else(|| {
                    "ec_spire remote leaf materialization next_pid overflow".to_owned()
                })?)
                .max(assign::SPIRE_FIRST_PID);
            let next_local_vec_seq = root_control
                .next_local_vec_seq
                .max(max_local_vec_seq.checked_add(1).ok_or_else(|| {
                    "ec_spire remote leaf materialization next_local_vec_seq overflow".to_owned()
                })?)
                .max(assign::SPIRE_FIRST_LOCAL_VEC_SEQ);
            let local_store_config = meta::SpireLocalStoreConfig::from_placement_directory(
                publish_epoch,
                &placement_directory,
            )?;
            let input = build::SpirePublishCoordinatorInput {
                epoch_manifest: &epoch_manifest,
                object_manifest: &object_manifest,
                placement_directory: &placement_directory,
                local_store_config,
                next_pid,
                next_local_vec_seq,
            };
            if let Some(previous_epoch) = previous_epoch {
                build::publish_replacement_epoch_to_relation(
                    index_relation.as_ptr(),
                    previous_epoch,
                    input,
                )?;
            } else {
                let manifests = build::encode_manifest_bundle_for_publish(input.clone())?;
                let locators =
                    build::write_manifest_bundle_to_relation(index_relation.as_ptr(), &manifests)?;
                let root_control = build::root_control_state_for_publish(input, locators)?;
                page::initialize_root_control_page(index_relation.as_ptr(), root_control);
            }

            Ok(SpireRemoteLeafMaterializationSummaryRow {
                active_epoch: publish_epoch,
                leaf_count: u64::try_from(placement_directory.entries.len()).map_err(|_| {
                    "ec_spire remote leaf materialization leaf count exceeds u64".to_owned()
                })?,
                assignment_count,
                next_pid,
                next_local_vec_seq,
                status: "materialized",
            })
        })()
    };
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn static_remote_leaf_materialization_rows(
    leaf_pids: Vec<u64>,
    parent_pids: Vec<u64>,
    object_versions: Vec<u64>,
    row_indices: Vec<u32>,
    assignment_flags: Vec<u16>,
    vec_ids: Vec<Vec<u8>>,
    heap_blocks: Vec<u32>,
    heap_offsets: Vec<u16>,
    payload_formats: Vec<u8>,
    gammas: Vec<f32>,
    encoded_payloads: Vec<Vec<u8>>,
) -> Result<Vec<SpireRemoteLeafMaterializationInputRow>, String> {
    let row_count = leaf_pids.len();
    if row_count == 0 {
        return Err(
            "ec_spire remote leaf materialization requires at least one assignment row".to_owned(),
        );
    }
    let lengths = [
        ("parent_pids", parent_pids.len()),
        ("object_versions", object_versions.len()),
        ("row_indices", row_indices.len()),
        ("assignment_flags", assignment_flags.len()),
        ("vec_ids", vec_ids.len()),
        ("heap_blocks", heap_blocks.len()),
        ("heap_offsets", heap_offsets.len()),
        ("payload_formats", payload_formats.len()),
        ("gammas", gammas.len()),
        ("encoded_payloads", encoded_payloads.len()),
    ];
    for (name, len) in lengths {
        if len != row_count {
            return Err(format!(
                "ec_spire remote leaf materialization {name} length {len} does not match leaf_pids length {row_count}"
            ));
        }
    }

    let mut rows = Vec::with_capacity(row_count);
    for row_no in 0..row_count {
        let leaf_pid = leaf_pids[row_no];
        if leaf_pid == 0 {
            return Err(format!(
                "ec_spire remote leaf materialization row {row_no} leaf_pid must be greater than zero"
            ));
        }
        let object_version = object_versions[row_no];
        if object_version == 0 {
            return Err(format!(
                "ec_spire remote leaf materialization row {row_no} object_version must be greater than zero"
            ));
        }
        let gamma = gammas[row_no];
        if !gamma.is_finite() {
            return Err(format!(
                "ec_spire remote leaf materialization row {row_no} gamma must be finite"
            ));
        }
        rows.push(SpireRemoteLeafMaterializationInputRow {
            leaf_pid,
            parent_pid: parent_pids[row_no],
            object_version,
            row_index: row_indices[row_no],
            assignment: storage::SpireLeafAssignmentRow {
                flags: assignment_flags[row_no],
                vec_id: storage::SpireVecId::from_bytes(&vec_ids[row_no])?,
                heap_tid: crate::storage::page::ItemPointer {
                    block_number: heap_blocks[row_no],
                    offset_number: heap_offsets[row_no],
                },
                payload_format: payload_formats[row_no],
                gamma,
                encoded_payload: encoded_payloads[row_no].clone(),
            },
        });
    }
    Ok(rows)
}
