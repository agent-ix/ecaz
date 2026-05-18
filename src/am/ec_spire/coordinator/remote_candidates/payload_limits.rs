fn current_remote_payload_bytes_per_row_limit() -> usize {
    session_limit_to_usize(options::current_session_max_remote_payload_bytes_per_row())
}

fn current_remote_payload_rows_per_batch_limit() -> usize {
    session_limit_to_usize(options::current_session_max_remote_payload_rows_per_batch())
}

fn validate_remote_payload_batch_row_count(row_count: usize, context: &str) -> Result<(), String> {
    validate_remote_payload_batch_row_count_with_limit(
        row_count,
        current_remote_payload_rows_per_batch_limit(),
        context,
    )
}

fn validate_remote_payload_batch_row_count_with_limit(
    row_count: usize,
    limit: usize,
    context: &str,
) -> Result<(), String> {
    if row_count > limit {
        return Err(format!(
            "{SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE}: {context} row count {row_count} exceeds ec_spire.max_remote_payload_rows_per_batch {limit}; {SPIRE_REMOTE_PAYLOAD_TOO_LARGE_HINT}"
        ));
    }
    Ok(())
}

fn validate_remote_payload_row_bytes(row_bytes: usize, context: &str) -> Result<(), String> {
    validate_remote_payload_row_bytes_with_limit(
        row_bytes,
        current_remote_payload_bytes_per_row_limit(),
        context,
    )
}

fn validate_remote_payload_row_bytes_with_limit(
    row_bytes: usize,
    limit: usize,
    context: &str,
) -> Result<(), String> {
    if row_bytes > limit {
        return Err(format!(
            "{SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE}: {context} payload bytes {row_bytes} exceeds ec_spire.max_remote_payload_bytes_per_row {limit}; {SPIRE_REMOTE_PAYLOAD_TOO_LARGE_HINT}"
        ));
    }
    Ok(())
}

fn typed_payload_hex_decoded_bytes(payload_values_hex: &[String]) -> Result<usize, String> {
    let mut row_bytes = 0_usize;
    for value in payload_values_hex {
        if value.len() % 2 != 0 {
            return Err(
                "ec_spire remote heap executor typed payload_values_hex is invalid".to_owned(),
            );
        }
        row_bytes = row_bytes.checked_add(value.len() / 2).ok_or_else(|| {
            format!(
                "{SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE}: remote typed tuple payload byte count overflow; {SPIRE_REMOTE_PAYLOAD_TOO_LARGE_HINT}"
            )
        })?;
    }
    Ok(row_bytes)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteTypedTuplePayloadFields {
    payload_attnums: Vec<i16>,
    payload_names: Vec<String>,
    payload_type_oids: Vec<String>,
    payload_typmods: Vec<i32>,
    payload_collations: Option<Vec<String>>,
    payload_nulls: Vec<bool>,
    payload_values_hex: Vec<String>,
    payload_formats: Vec<String>,
    tuple_transport: String,
    tuple_transport_status: String,
}

fn parse_remote_typed_payload_oid(oid_text: &str, label: &str) -> Result<pg_sys::Oid, String> {
    oid_text
        .parse::<u32>()
        .map(pg_sys::Oid::from)
        .map_err(|_| format!("ec_spire remote heap executor typed {label} is invalid"))
}

fn decode_remote_typed_tuple_payload_fields(
    fields: RemoteTypedTuplePayloadFields,
) -> Result<SpireRemoteTypedTuplePayload, String> {
    decode_remote_typed_tuple_payload_fields_with_limit(
        fields,
        current_remote_payload_bytes_per_row_limit(),
    )
}

fn decode_remote_typed_tuple_payload_fields_with_limit(
    fields: RemoteTypedTuplePayloadFields,
    row_byte_limit: usize,
) -> Result<SpireRemoteTypedTuplePayload, String> {
    let payload_width = fields.payload_attnums.len();
    for (label, width) in [
        ("payload_names", fields.payload_names.len()),
        ("payload_type_oids", fields.payload_type_oids.len()),
        ("payload_typmods", fields.payload_typmods.len()),
        ("payload_nulls", fields.payload_nulls.len()),
        ("payload_values", fields.payload_values_hex.len()),
        ("payload_formats", fields.payload_formats.len()),
    ] {
        if width != payload_width {
            return Err(format!(
                "ec_spire remote heap executor typed {label} width {width} does not match attnum width {payload_width}"
            ));
        }
    }
    let payload_collations = fields
        .payload_collations
        .unwrap_or_else(|| vec![pg_sys::InvalidOid.to_string(); payload_width]);
    if payload_collations.len() != payload_width {
        return Err(format!(
            "ec_spire remote heap executor typed payload_collations width {} does not match attnum width {payload_width}",
            payload_collations.len()
        ));
    }
    if fields.payload_attnums.iter().any(|attnum| *attnum <= 0) {
        return Err("ec_spire remote heap executor typed payload_attnum is invalid".to_owned());
    }
    let payload_type_oids = fields
        .payload_type_oids
        .iter()
        .map(|oid| parse_remote_typed_payload_oid(oid, "payload_type_oid"))
        .collect::<Result<Vec<_>, _>>()?;
    if payload_type_oids
        .iter()
        .any(|oid| *oid == pg_sys::InvalidOid)
    {
        return Err("ec_spire remote heap executor typed payload_type_oid is invalid".to_owned());
    }
    let payload_collations = payload_collations
        .iter()
        .map(|oid| parse_remote_typed_payload_oid(oid, "payload_collation"))
        .collect::<Result<Vec<_>, _>>()?;
    validate_remote_payload_row_bytes_with_limit(
        typed_payload_hex_decoded_bytes(&fields.payload_values_hex)?,
        row_byte_limit,
        "remote typed tuple payload",
    )?;
    let payload_values = fields
        .payload_values_hex
        .iter()
        .map(|value| {
            hex::decode(value).map_err(|_| {
                "ec_spire remote heap executor typed payload_values_hex is invalid".to_owned()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if fields.tuple_transport != SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1 {
        return Err(format!(
            "ec_spire remote heap executor unsupported tuple transport {}",
            fields.tuple_transport
        ));
    }
    if fields.tuple_transport_status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire remote heap executor tuple transport status {} is not ready",
            fields.tuple_transport_status
        ));
    }
    if fields
        .payload_formats
        .iter()
        .any(|format| format != SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1)
    {
        return Err("ec_spire remote heap executor typed payload format mismatch".to_owned());
    }

    Ok(SpireRemoteTypedTuplePayload {
        payload_attnums: fields.payload_attnums,
        payload_names: fields.payload_names,
        payload_type_oids,
        payload_typmods: fields.payload_typmods,
        payload_collations,
        payload_nulls: fields.payload_nulls,
        payload_values,
        payload_formats: fields.payload_formats,
        tuple_transport: SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
        tuple_transport_status: SPIRE_REMOTE_STATUS_READY,
    })
}
