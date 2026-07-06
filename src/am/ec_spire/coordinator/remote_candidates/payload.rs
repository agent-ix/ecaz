fn decode_remote_search_candidate_pg_row(
    row: &postgres::Row,
    expected_node_id: u32,
    validate_endpoint_identity: bool,
    expected_remote_index_identity: Option<&[u8]>,
) -> Result<SpireRemoteSearchCandidateRow, String> {
    let served_epoch = row
        .try_get::<_, i64>("served_epoch")
        .map_err(|_| "ec_spire remote search executor served_epoch decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| "ec_spire remote search executor served_epoch is negative".to_owned())
        })?;
    let remote_node_id = row
        .try_get::<_, i64>("node_id")
        .map_err(|_| "ec_spire remote search executor node_id decode failed".to_owned())
        .and_then(|value| {
            u32::try_from(value)
                .map_err(|_| "ec_spire remote search executor node_id is invalid".to_owned())
        })?;
    let node_id = if remote_node_id == meta::SPIRE_LOCAL_NODE_ID {
        expected_node_id
    } else {
        remote_node_id
    };
    let pid = row
        .try_get::<_, i64>("pid")
        .map_err(|_| "ec_spire remote search executor pid decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| "ec_spire remote search executor pid is negative".to_owned())
        })?;
    let object_version = row
        .try_get::<_, i64>("object_version")
        .map_err(|_| "ec_spire remote search executor object_version decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire remote search executor object_version is negative".to_owned()
            })
        })?;
    let row_index = row
        .try_get::<_, i64>("row_index")
        .map_err(|_| "ec_spire remote search executor row_index decode failed".to_owned())
        .and_then(|value| {
            u32::try_from(value)
                .map_err(|_| "ec_spire remote search executor row_index is invalid".to_owned())
        })?;
    let assignment_flags = row
        .try_get::<_, i16>("assignment_flags")
        .map_err(|_| "ec_spire remote search executor assignment_flags decode failed".to_owned())
        .and_then(|value| {
            u16::try_from(value).map_err(|_| {
                "ec_spire remote search executor assignment_flags is negative".to_owned()
            })
        })?;
    let vec_id = row
        .try_get::<_, Vec<u8>>("vec_id")
        .map_err(|_| "ec_spire remote search executor vec_id decode failed".to_owned())?;
    let row_locator = row
        .try_get::<_, Vec<u8>>("row_locator")
        .map_err(|_| "ec_spire remote search executor row_locator decode failed".to_owned())?;
    let score = row
        .try_get::<_, f32>("score")
        .map_err(|_| "ec_spire remote search executor score decode failed".to_owned())?;
    if validate_endpoint_identity {
        let profile_fingerprint_bytes = validate_remote_search_candidate_endpoint_identity(row)?;
        if let Some(expected_remote_index_identity) = expected_remote_index_identity {
            if profile_fingerprint_bytes.as_slice() != expected_remote_index_identity {
                return Err(
                    "ec_spire remote search executor remote_index_identity does not match candidate profile_fingerprint"
                        .to_owned(),
                );
            }
        }
    }

    Ok(SpireRemoteSearchCandidateRow {
        served_epoch,
        node_id,
        pid,
        object_version,
        row_index,
        assignment_flags,
        vec_id,
        row_locator,
        score,
    })
}

fn decode_remote_search_scan_profile_pg_row(
    row: &postgres::Row,
    expected_requested_epoch: u64,
    expected_node_id: u32,
) -> Result<scan::SpireSelectedLeafScanProfile, String> {
    let served_epoch = row
        .try_get::<_, i64>("served_epoch")
        .map_err(|_| "ec_spire remote scan profile served_epoch decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| "ec_spire remote scan profile served_epoch is negative".to_owned())
        })?;
    if served_epoch != expected_requested_epoch {
        return Err(format!(
            "ec_spire remote scan profile served_epoch {served_epoch} does not match requested epoch {expected_requested_epoch}"
        ));
    }
    let selected_pid_count = decode_nonnegative_i64_metric(row, "selected_pid_count")?;
    let scanned_pid_count = decode_nonnegative_i64_metric(row, "scanned_pid_count")?;
    let leaf_candidate_row_count = decode_nonnegative_i64_metric(row, "leaf_candidate_row_count")?;
    let deduped_candidate_row_count =
        decode_nonnegative_i64_metric(row, "deduped_candidate_row_count")?;
    let truncated_candidate_row_count =
        decode_nonnegative_i64_metric(row, "truncated_candidate_row_count")?;
    let pre_materialization_pruned_candidate_row_count =
        decode_nonnegative_i64_metric(row, "pre_materialization_pruned_candidate_row_count")?;
    let candidate_winner_count = decode_nonnegative_i64_metric(row, "candidate_winner_count")?;
    let leaf_block_available_count =
        decode_nonnegative_i64_metric(row, "leaf_block_available_count")?;
    let leaf_block_selected_count =
        decode_nonnegative_i64_metric(row, "leaf_block_selected_count")?;
    let leaf_block_skipped_count = decode_nonnegative_i64_metric(row, "leaf_block_skipped_count")?;
    let sound_upper_bound_available_count =
        decode_nonnegative_i64_metric(row, "sound_upper_bound_available_count")?;
    let sound_upper_bound_missing_count =
        decode_nonnegative_i64_metric(row, "sound_upper_bound_missing_count")?;
    let leaf_summary_score_nanos =
        decode_nonnegative_i64_metric(row, "leaf_summary_score_nanos")?;
    let leaf_row_score_nanos = decode_nonnegative_i64_metric(row, "leaf_row_score_nanos")?;
    let candidate_score_nanos = decode_nonnegative_i64_metric(row, "candidate_score_nanos")?;
    let local_kth_score = row
        .try_get::<_, Option<f32>>("local_kth_score")
        .map_err(|_| "ec_spire remote scan profile local_kth_score decode failed".to_owned())?;

    Ok(scan::SpireSelectedLeafScanProfile {
        served_epoch,
        node_id: expected_node_id,
        selected_pid_count,
        scanned_pid_count,
        leaf_candidate_row_count,
        deduped_candidate_row_count,
        truncated_candidate_row_count,
        pre_materialization_pruned_candidate_row_count,
        candidate_winner_count,
        leaf_block_available_count,
        leaf_block_selected_count,
        leaf_block_skipped_count,
        sound_upper_bound_available_count,
        sound_upper_bound_missing_count,
        leaf_summary_score_nanos,
        leaf_row_score_nanos,
        candidate_score_nanos,
        local_kth_score,
    })
}

fn decode_remote_search_threshold_profile_pg_row(
    row: &postgres::Row,
    expected_requested_epoch: u64,
    expected_node_id: u32,
) -> Result<scan::SpireSelectedLeafThresholdProfile, String> {
    let served_epoch = row
        .try_get::<_, i64>("served_epoch")
        .map_err(|_| "ec_spire remote threshold profile served_epoch decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire remote threshold profile served_epoch is negative".to_owned()
            })
        })?;
    if served_epoch != expected_requested_epoch {
        return Err(format!(
            "ec_spire remote threshold profile served_epoch {served_epoch} does not match requested epoch {expected_requested_epoch}"
        ));
    }
    let selected_pid_count = decode_nonnegative_i64_metric(row, "selected_pid_count")?;
    let evaluated_pid_count = decode_nonnegative_i64_metric(row, "evaluated_pid_count")?;
    let threshold_score = row
        .try_get::<_, f32>("threshold_score")
        .map_err(|_| "ec_spire remote threshold profile threshold_score decode failed".to_owned())?;
    let threshold_ip = row
        .try_get::<_, f32>("threshold_ip")
        .map_err(|_| "ec_spire remote threshold profile threshold_ip decode failed".to_owned())?;
    let sound_upper_bound_available_count =
        decode_nonnegative_i64_metric(row, "sound_upper_bound_available_count")?;
    let sound_upper_bound_missing_count =
        decode_nonnegative_i64_metric(row, "sound_upper_bound_missing_count")?;
    let threshold_block_available_count =
        decode_nonnegative_i64_metric(row, "threshold_block_available_count")?;
    let threshold_block_selected_count =
        decode_nonnegative_i64_metric(row, "threshold_block_selected_count")?;
    let threshold_block_skipped_count =
        decode_nonnegative_i64_metric(row, "threshold_block_skipped_count")?;
    let threshold_row_available_count =
        decode_nonnegative_i64_metric(row, "threshold_row_available_count")?;
    let threshold_row_selected_count =
        decode_nonnegative_i64_metric(row, "threshold_row_selected_count")?;
    let threshold_row_skipped_count =
        decode_nonnegative_i64_metric(row, "threshold_row_skipped_count")?;
    let leaf_summary_score_nanos =
        decode_nonnegative_i64_metric(row, "leaf_summary_score_nanos")?;

    Ok(scan::SpireSelectedLeafThresholdProfile {
        served_epoch,
        node_id: expected_node_id,
        selected_pid_count,
        evaluated_pid_count,
        threshold_score,
        threshold_ip,
        sound_upper_bound_available_count,
        sound_upper_bound_missing_count,
        threshold_block_available_count,
        threshold_block_selected_count,
        threshold_block_skipped_count,
        threshold_row_available_count,
        threshold_row_selected_count,
        threshold_row_skipped_count,
        leaf_summary_score_nanos,
    })
}

fn decode_nonnegative_i64_metric(row: &postgres::Row, column: &str) -> Result<u64, String> {
    row.try_get::<_, i64>(column)
        .map_err(|_| format!("ec_spire remote scan profile {column} decode failed"))
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| format!("ec_spire remote scan profile {column} is negative"))
        })
}

fn decode_remote_search_heap_candidate_pg_row(
    row: &postgres::Row,
    expected_requested_epoch: u64,
    expected_node_id: u32,
) -> Result<SpireRemoteSearchLocalHeapCandidateRow, String> {
    let requested_epoch = row
        .try_get::<_, i64>("requested_epoch")
        .map_err(|_| {
            "ec_spire remote heap executor requested_epoch decode failed".to_owned()
        })
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire remote heap executor requested_epoch is negative".to_owned()
            })
        })?;
    if requested_epoch != expected_requested_epoch {
        return Err(format!(
            "ec_spire remote heap executor requested_epoch {requested_epoch} does not match expected epoch {expected_requested_epoch}"
        ));
    }
    let candidate = decode_remote_search_candidate_pg_row(row, expected_node_id, false, None)?;
    let heap_block = row
        .try_get::<_, i64>("heap_block")
        .map_err(|_| "ec_spire remote heap executor heap_block decode failed".to_owned())
        .and_then(|value| {
            u32::try_from(value)
                .map_err(|_| "ec_spire remote heap executor heap_block is invalid".to_owned())
        })?;
    let heap_offset = row
        .try_get::<_, i32>("heap_offset")
        .map_err(|_| "ec_spire remote heap executor heap_offset decode failed".to_owned())
        .and_then(|value| {
            u16::try_from(value)
                .map_err(|_| "ec_spire remote heap executor heap_offset is invalid".to_owned())
        })?;
    let status = row
        .try_get::<_, String>("status")
        .map_err(|_| "ec_spire remote heap executor status decode failed".to_owned())?;
    if status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire remote heap executor returned non-ready heap candidate status {status}"
        ));
    }

    Ok(SpireRemoteSearchLocalHeapCandidateRow {
        requested_epoch,
        served_epoch: candidate.served_epoch,
        node_id: candidate.node_id,
        pid: candidate.pid,
        object_version: candidate.object_version,
        row_index: candidate.row_index,
        assignment_flags: candidate.assignment_flags,
        vec_id: candidate.vec_id,
        row_locator: candidate.row_locator,
        heap_block,
        heap_offset,
        score: candidate.score,
        heap_lookup_owner: SPIRE_REMOTE_HEAP_RESOLUTION,
        tuple_payload_json: row.try_get::<_, String>("tuple_payload_text").ok(),
        typed_tuple_payload: decode_remote_search_typed_tuple_payload_pg_row(row)?,
        tuple_payload_missing: row
            .try_get::<_, bool>("tuple_payload_missing")
            .unwrap_or(false),
        status: SPIRE_REMOTE_STATUS_READY,
    })
}

fn decode_remote_search_typed_tuple_payload_pg_row(
    row: &postgres::Row,
) -> Result<Option<SpireRemoteTypedTuplePayload>, String> {
    let Ok(payload_attnums) = row.try_get::<_, Vec<i16>>("payload_attnums") else {
        return Ok(None);
    };
    let payload_names = row
        .try_get::<_, Vec<String>>("payload_names")
        .map_err(|_| "ec_spire remote heap executor typed payload_names decode failed".to_owned())?;
    let payload_type_oids = row
        .try_get::<_, Vec<String>>("payload_type_oids")
        .map_err(|_| {
            "ec_spire remote heap executor typed payload_type_oids decode failed".to_owned()
        })?;
    let payload_typmods = row
        .try_get::<_, Vec<i32>>("payload_typmods")
        .map_err(|_| "ec_spire remote heap executor typed payload_typmods decode failed".to_owned())?;
    let payload_collations = row.try_get::<_, Vec<String>>("payload_collations").ok();
    let payload_nulls = row
        .try_get::<_, Vec<bool>>("payload_nulls")
        .map_err(|_| "ec_spire remote heap executor typed payload_nulls decode failed".to_owned())?;
    let payload_values_hex = row
        .try_get::<_, Vec<String>>("payload_values_hex")
        .map_err(|_| {
            "ec_spire remote heap executor typed payload_values_hex decode failed".to_owned()
        })?;
    let payload_formats = row
        .try_get::<_, Vec<String>>("payload_formats")
        .map_err(|_| {
            "ec_spire remote heap executor typed payload_formats decode failed".to_owned()
        })?;
    let tuple_transport = row
        .try_get::<_, String>("tuple_transport")
        .map_err(|_| "ec_spire remote heap executor tuple_transport decode failed".to_owned())?;
    let tuple_transport_status = row
        .try_get::<_, String>("tuple_transport_status")
        .map_err(|_| {
            "ec_spire remote heap executor tuple_transport_status decode failed".to_owned()
        })?;

    Ok(Some(decode_remote_typed_tuple_payload_fields(
        RemoteTypedTuplePayloadFields {
            payload_attnums,
            payload_names,
            payload_type_oids,
            payload_typmods,
            payload_collations,
            payload_nulls,
            payload_values_hex,
            payload_formats,
            tuple_transport,
            tuple_transport_status,
        },
    )?))
}
