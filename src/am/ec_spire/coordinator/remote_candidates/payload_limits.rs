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
