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
        })?
        .into_iter()
        .map(|oid| {
            oid.parse::<u32>()
                .map(pg_sys::Oid::from)
                .map_err(|_| "ec_spire remote heap executor typed payload_type_oid is invalid".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let payload_typmods = row
        .try_get::<_, Vec<i32>>("payload_typmods")
        .map_err(|_| "ec_spire remote heap executor typed payload_typmods decode failed".to_owned())?;
    let payload_collations = match row.try_get::<_, Vec<String>>("payload_collations") {
        Ok(collations) => collations
            .into_iter()
            .map(|oid| {
                oid.parse::<u32>()
                    .map(pg_sys::Oid::from)
                    .map_err(|_| {
                        "ec_spire remote heap executor typed payload_collation is invalid"
                            .to_owned()
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
        Err(_) => vec![pg_sys::InvalidOid; payload_attnums.len()],
    };
    let payload_nulls = row
        .try_get::<_, Vec<bool>>("payload_nulls")
        .map_err(|_| "ec_spire remote heap executor typed payload_nulls decode failed".to_owned())?;
    let payload_values_hex = row
        .try_get::<_, Vec<String>>("payload_values_hex")
        .map_err(|_| {
            "ec_spire remote heap executor typed payload_values_hex decode failed".to_owned()
        })?;
    validate_remote_payload_row_bytes(
        typed_payload_hex_decoded_bytes(&payload_values_hex)?,
        "remote typed tuple payload",
    )?;
    let payload_values = payload_values_hex
        .iter()
        .map(|value| {
            hex::decode(value).map_err(|_| {
                "ec_spire remote heap executor typed payload_values_hex is invalid".to_owned()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
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
    let payload_width = payload_attnums.len();
    for (label, width) in [
        ("payload_names", payload_names.len()),
        ("payload_type_oids", payload_type_oids.len()),
        ("payload_typmods", payload_typmods.len()),
        ("payload_collations", payload_collations.len()),
        ("payload_nulls", payload_nulls.len()),
        ("payload_values", payload_values.len()),
        ("payload_formats", payload_formats.len()),
    ] {
        if width != payload_width {
            return Err(format!(
                "ec_spire remote heap executor typed {label} width {width} does not match attnum width {payload_width}"
            ));
        }
    }
    if tuple_transport != SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1 {
        return Err(format!(
            "ec_spire remote heap executor unsupported tuple transport {tuple_transport}"
        ));
    }
    if tuple_transport_status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire remote heap executor tuple transport status {tuple_transport_status} is not ready"
        ));
    }
    if payload_formats
        .iter()
        .any(|format| format != SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1)
    {
        return Err(
            "ec_spire remote heap executor typed payload format mismatch".to_owned()
        );
    }

    Ok(Some(SpireRemoteTypedTuplePayload {
        payload_attnums,
        payload_names,
        payload_type_oids,
        payload_typmods,
        payload_collations,
        payload_nulls,
        payload_values,
        payload_formats,
        tuple_transport: SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
        tuple_transport_status: SPIRE_REMOTE_STATUS_READY,
    }))
}

