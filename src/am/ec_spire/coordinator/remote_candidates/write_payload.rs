fn coordinator_insert_remote_tuple_payload_sql(
    remote_index_regclass: &str,
    row_payload_json: &str,
    requested_columns: &[String],
) -> Result<String, String> {
    if requested_columns.is_empty() {
        return Err("ec_spire coordinator insert tuple payload column list is empty".to_owned());
    }
    let column_literals = requested_columns
        .iter()
        .map(|column| {
            if column.is_empty() {
                return Err(
                    "ec_spire coordinator insert tuple payload column name is empty".to_owned(),
                );
            }
            Ok(quote_sql_literal(column))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(", ");
    Ok(format!(
        "SELECT * FROM ec_spire_remote_insert_tuple_payload(\
             {}::regclass, {}::jsonb, ARRAY[{}]::text[])",
        quote_sql_literal(remote_index_regclass),
        quote_sql_literal(row_payload_json),
        column_literals
    ))
}

fn coordinator_update_remote_tuple_payload_sql(
    remote_index_regclass: &str,
    pk_column: &str,
    pk_value: &[u8],
    row_payload_json: &str,
    updated_columns: &[String],
) -> Result<String, String> {
    if pk_column.is_empty() {
        return Err("ec_spire coordinator update pk column is empty".to_owned());
    }
    if updated_columns.is_empty() {
        return Err("ec_spire coordinator update column list is empty".to_owned());
    }
    let column_literals = updated_columns
        .iter()
        .map(|column| {
            if column.is_empty() {
                return Err("ec_spire coordinator update column name is empty".to_owned());
            }
            Ok(quote_sql_literal(column))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(", ");
    Ok(format!(
        "SELECT * FROM ec_spire_remote_update_tuple_payload(\
             {}::regclass, {}::text, decode({}, 'hex'), {}::jsonb, ARRAY[{}]::text[])",
        quote_sql_literal(remote_index_regclass),
        quote_sql_literal(pk_column),
        quote_sql_literal(&hex::encode(pk_value)),
        quote_sql_literal(row_payload_json),
        column_literals
    ))
}

fn coordinator_delete_remote_tuple_payload_sql(
    remote_index_regclass: &str,
    pk_column: &str,
    pk_value: &[u8],
) -> Result<String, String> {
    if pk_column.is_empty() {
        return Err("ec_spire coordinator delete pk column is empty".to_owned());
    }
    if pk_value.is_empty() {
        return Err("ec_spire coordinator delete pk_value is empty".to_owned());
    }
    Ok(format!(
        "SELECT * FROM ec_spire_remote_delete_tuple_payload(\
             {}::regclass, {}::text, decode({}, 'hex'))",
        quote_sql_literal(remote_index_regclass),
        quote_sql_literal(pk_column),
        quote_sql_literal(&hex::encode(pk_value))
    ))
}

fn coordinator_select_remote_tuple_payload_sql(
    remote_index_regclass: &str,
    pk_column: &str,
    pk_value: &[u8],
    requested_columns: &[String],
) -> Result<String, String> {
    if pk_column.is_empty() {
        return Err("ec_spire coordinator select pk column is empty".to_owned());
    }
    if pk_value.is_empty() {
        return Err("ec_spire coordinator select pk_value is empty".to_owned());
    }
    if requested_columns.is_empty() {
        return Err("ec_spire coordinator select column list is empty".to_owned());
    }
    let column_literals = requested_columns
        .iter()
        .map(|column| {
            if column.is_empty() {
                return Err("ec_spire coordinator select column name is empty".to_owned());
            }
            Ok(quote_sql_literal(column))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(", ");
    Ok(format!(
        "SELECT * FROM ec_spire_remote_select_tuple_payload(\
             {}::regclass, {}::text, decode({}, 'hex'), ARRAY[{}]::text[])",
        quote_sql_literal(remote_index_regclass),
        quote_sql_literal(pk_column),
        quote_sql_literal(&hex::encode(pk_value)),
        column_literals
    ))
}

fn coordinator_insert_remote_descriptor_metadata(
    client: &mut postgres::Client,
    node_id: u32,
    remote_index_regclass: &str,
) -> Result<(u64, Vec<u8>, String), String> {
    let row = client
        .query_one(
            "SELECT h.active_epoch::bigint AS active_epoch, \
                    e.protocol_version, e.extension_version, e.opclass_identity, \
                    e.storage_format, e.assignment_payload_format, e.quantizer_profile, \
                    e.scoring_profile, e.profile_fingerprint, e.status, e.recommendation \
               FROM ec_spire_index_hierarchy_snapshot($1::text::regclass) h \
              CROSS JOIN ec_spire_remote_search_endpoint_identity($1::text::regclass::oid) e",
            &[&remote_index_regclass],
        )
        .map_err(|error| {
            format!(
                "ec_spire coordinator insert remote descriptor metadata query failed for node_id {node_id}: {error}"
            )
        })?;
    let active_epoch = row
        .try_get::<_, i64>("active_epoch")
        .map_err(|_| {
            "ec_spire coordinator insert remote descriptor active_epoch decode failed".to_owned()
        })
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire coordinator insert remote descriptor active_epoch is negative".to_owned()
            })
        })?;
    let profile_fingerprint = row
        .try_get::<_, String>("profile_fingerprint")
        .map_err(|_| {
            "ec_spire coordinator insert remote descriptor profile_fingerprint decode failed"
                .to_owned()
        })?;
    let remote_index_identity = remote_search_endpoint_profile_fingerprint_bytes(&profile_fingerprint)?;
    let extension_version = row
        .try_get::<_, String>("extension_version")
        .map_err(|_| {
            "ec_spire coordinator insert remote descriptor extension_version decode failed".to_owned()
        })?;
    if extension_version.is_empty() {
        return Err(
            "ec_spire coordinator insert remote descriptor extension_version is empty".to_owned(),
        );
    }

    Ok((active_epoch, remote_index_identity, extension_version))
}

async fn coordinator_insert_remote_descriptor_metadata_async(
    client: &tokio_postgres::Client,
    node_id: u32,
    remote_index_regclass: &str,
) -> Result<(u64, Vec<u8>, String), String> {
    let row = client
        .query_one(
            "SELECT h.active_epoch::bigint AS active_epoch, \
                    e.protocol_version, e.extension_version, e.opclass_identity, \
                    e.storage_format, e.assignment_payload_format, e.quantizer_profile, \
                    e.scoring_profile, e.profile_fingerprint, e.status, e.recommendation \
               FROM ec_spire_index_hierarchy_snapshot($1::text::regclass) h \
              CROSS JOIN ec_spire_remote_search_endpoint_identity($1::text::regclass::oid) e",
            &[&remote_index_regclass],
        )
        .await
        .map_err(|error| {
            format!(
                "ec_spire coordinator insert remote descriptor metadata query failed for node_id {node_id}: {error}"
            )
        })?;
    let active_epoch = row
        .try_get::<_, i64>("active_epoch")
        .map_err(|_| {
            "ec_spire coordinator insert remote descriptor active_epoch decode failed".to_owned()
        })
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire coordinator insert remote descriptor active_epoch is negative".to_owned()
            })
        })?;
    let profile_fingerprint = row
        .try_get::<_, String>("profile_fingerprint")
        .map_err(|_| {
            "ec_spire coordinator insert remote descriptor profile_fingerprint decode failed"
                .to_owned()
        })?;
    let remote_index_identity =
        remote_search_endpoint_profile_fingerprint_bytes(&profile_fingerprint)?;
    let extension_version = row
        .try_get::<_, String>("extension_version")
        .map_err(|_| {
            "ec_spire coordinator insert remote descriptor extension_version decode failed".to_owned()
        })?;
    if extension_version.is_empty() {
        return Err(
            "ec_spire coordinator insert remote descriptor extension_version is empty".to_owned(),
        );
    }

    Ok((active_epoch, remote_index_identity, extension_version))
}

fn insert_step_observed_local_cancel<T>(
    result: &Result<SpireCoordinatorInsertAsyncStep<T>, String>,
) -> bool {
    result
        .as_ref()
        .map(|step| step.local_cancel_observed)
        .unwrap_or(false)
}

const SPIRE_PREPARED_TRANSACTION_CAPACITY_HINT: &str =
    "SPIRE requires max_prepared_transactions > 0 and enough free prepared \
     transaction slots on every remote PostgreSQL instance; increase \
     max_prepared_transactions, restart the remote, and size it for peak \
     concurrent coordinator-routed SPIRE writes plus any non-SPIRE prepared \
     transactions";

fn postgres_prepare_transaction_capacity_failure(
    sqlstate: Option<&str>,
    message: &str,
) -> bool {
    if sqlstate == Some("55000") {
        return true;
    }
    let message = message.to_ascii_lowercase();
    let capacity_message = message.contains("prepared transactions are disabled")
        || message.contains("maximum number of prepared transactions")
        || message.contains("max_prepared_transactions");
    capacity_message && matches!(sqlstate, Some("53300" | "53400") | None)
}

fn spire_remote_prepare_transaction_error(
    operation: &str,
    node_id: u32,
    error: &postgres::Error,
) -> String {
    let base = format!(
        "ec_spire coordinator {operation} remote PREPARE TRANSACTION failed for node_id {node_id}: {error}"
    );
    let (sqlstate, message) = error
        .as_db_error()
        .map(|db_error| (Some(db_error.code().code()), db_error.message()))
        .unwrap_or((None, base.as_str()));
    if postgres_prepare_transaction_capacity_failure(sqlstate, message) {
        format!("{base}; {SPIRE_PREPARED_TRANSACTION_CAPACITY_HINT}")
    } else {
        base
    }
}

fn spire_remote_prepare_transaction_async_error(
    operation: &str,
    node_id: u32,
    error: &tokio_postgres::Error,
) -> String {
    let base = format!(
        "ec_spire coordinator {operation} remote PREPARE TRANSACTION failed for node_id {node_id}: {error}"
    );
    let (sqlstate, message) = error
        .as_db_error()
        .map(|db_error| (Some(db_error.code().code()), db_error.message()))
        .unwrap_or((None, base.as_str()));
    if postgres_prepare_transaction_capacity_failure(sqlstate, message) {
        format!("{base}; {SPIRE_PREPARED_TRANSACTION_CAPACITY_HINT}")
    } else {
        base
    }
}

fn postgres_async_error_message_with_detail(error: &tokio_postgres::Error) -> String {
    let Some(db_error) = error.as_db_error() else {
        return error.to_string();
    };
    let mut message = db_error.message().to_owned();
    if let Some(detail) = db_error.detail() {
        message.push_str("; detail: ");
        message.push_str(detail);
    }
    if let Some(hint) = db_error.hint() {
        message.push_str("; hint: ");
        message.push_str(hint);
    }
    message
}

fn postgres_error_message_with_detail(error: &postgres::Error) -> String {
    let Some(db_error) = error.as_db_error() else {
        return error.to_string();
    };
    let mut message = format!("{} (SQLSTATE {})", db_error.message(), db_error.code().code());
    if let Some(detail) = db_error.detail() {
        if !detail.is_empty() {
            message.push_str("; DETAIL: ");
            message.push_str(detail);
        }
    }
    if let Some(hint) = db_error.hint() {
        if !hint.is_empty() {
            message.push_str("; HINT: ");
            message.push_str(hint);
        }
    }
    message
}

fn coordinator_write_current_shape_fingerprint(index_oid: pg_sys::Oid) -> Result<String, String> {
    let sql = format!(
        "SELECT ec_spire_coordinator_index_shape_fingerprint('{}'::oid::regclass) AS fingerprint",
        u32::from(index_oid)
    );
    Spi::get_one::<String>(sql.as_str())
        .map_err(|e| format!("ec_spire coordinator write shape fingerprint read failed: {e}"))?
        .ok_or_else(|| {
            "ec_spire coordinator write shape fingerprint returned no row for index".to_owned()
        })
}

fn remote_write_current_shape_fingerprint(
    conninfo: &str,
    node_id: u32,
    remote_index_regclass: &str,
) -> Result<String, String> {
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        conninfo,
        node_id,
        "remote write shape fingerprint",
    )?;
    let row = client
        .query_one(
            "SELECT ec_spire_remote_index_shape_fingerprint(to_regclass($1))::text AS fingerprint",
            &[&remote_index_regclass],
        )
        .map_err(|error| {
            format!(
                "ec_spire remote write shape fingerprint query failed for node_id {node_id}: {}",
                postgres_error_message_with_detail(&error)
            )
        })?;
    row.try_get::<_, Option<String>>("fingerprint")
        .map_err(|error| {
            format!(
                "ec_spire remote write shape fingerprint decode failed for node_id {node_id}: {error}"
            )
        })?
        .ok_or_else(|| {
            format!(
                "ec_spire remote write shape fingerprint remote index {remote_index_regclass} did not resolve for node_id {node_id}"
            )
        })
}

pub(crate) fn remote_write_shape_fingerprint_from_secret(
    conninfo_secret_name: &str,
    node_id: u32,
    remote_index_regclass: &str,
) -> Result<String, String> {
    let conninfo = remote_conninfo_secret_value(conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire remote write shape fingerprint conninfo secret for node_id {node_id} is not resolved: {status}"
        )
    })?;
    remote_write_current_shape_fingerprint(&conninfo, node_id, remote_index_regclass)
}

fn validate_coordinator_write_shape_fingerprint(
    operation: &str,
    index_oid: pg_sys::Oid,
    descriptor_fingerprint: &str,
) -> Result<String, String> {
    if descriptor_fingerprint == SPIRE_REMOTE_NONE || descriptor_fingerprint == "unset" {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: schema drift guard is missing coordinator descriptor fingerprint; hint: refresh remote node descriptors before coordinator-routed writes"
        ));
    }
    let current_fingerprint = coordinator_write_current_shape_fingerprint(index_oid)?;
    if current_fingerprint != descriptor_fingerprint {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: coordinator side drifted for index_oid {}: descriptor coordinator fingerprint {} does not match current coordinator fingerprint {}; hint: pause writes, apply matching DDL on every remote, refresh descriptors, then retry",
            u32::from(index_oid),
            descriptor_fingerprint,
            current_fingerprint
        ));
    }
    Ok(current_fingerprint)
}

fn validate_remote_write_shape_fingerprint(
    operation: &str,
    node_id: u32,
    conninfo: &str,
    remote_index_regclass: &str,
    coordinator_fingerprint: &str,
    descriptor_remote_fingerprint: &str,
) -> Result<(), String> {
    if descriptor_remote_fingerprint == SPIRE_REMOTE_NONE || descriptor_remote_fingerprint == "unset"
    {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: schema drift guard is missing remote descriptor fingerprint for node_id {node_id}; hint: refresh the remote node descriptor after verifying coordinator and remote DDL match"
        ));
    }
    let current_remote_fingerprint =
        remote_write_current_shape_fingerprint(conninfo, node_id, remote_index_regclass)?;
    if current_remote_fingerprint != descriptor_remote_fingerprint {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: remote side drifted for node_id {node_id}: descriptor remote fingerprint {descriptor_remote_fingerprint} does not match current remote fingerprint {current_remote_fingerprint}; hint: pause writes, apply matching DDL on the remote, refresh the descriptor, then retry"
        ));
    }
    if current_remote_fingerprint != coordinator_fingerprint {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: coordinator and remote schema fingerprints differ for node_id {node_id}: coordinator fingerprint {coordinator_fingerprint}, remote fingerprint {current_remote_fingerprint}; hint: pause writes, apply matching DDL on the side that drifted, refresh descriptors, then retry"
        ));
    }
    Ok(())
}

pub(crate) fn validate_read_shape_fingerprints_before_remote_dispatch(
    index_oid: pg_sys::Oid,
    node_id: u32,
    conninfo: &str,
    remote_index_regclass: &str,
    descriptor_coordinator_fingerprint: &str,
    descriptor_remote_fingerprint: &str,
) -> Result<(), String> {
    let operation = "read";
    if descriptor_coordinator_fingerprint == SPIRE_REMOTE_NONE
        || descriptor_coordinator_fingerprint == "unset"
    {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: schema drift guard is missing coordinator descriptor fingerprint; hint: refresh remote node descriptors before coordinator-routed reads"
        ));
    }
    if descriptor_remote_fingerprint == SPIRE_REMOTE_NONE || descriptor_remote_fingerprint == "unset"
    {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: schema drift guard is missing remote descriptor fingerprint for node_id {node_id}; hint: refresh the remote node descriptor after verifying coordinator and remote DDL match"
        ));
    }

    let current_coordinator_fingerprint = coordinator_write_current_shape_fingerprint(index_oid)?;
    let current_remote_fingerprint =
        remote_write_current_shape_fingerprint(conninfo, node_id, remote_index_regclass)?;
    let coordinator_drifted =
        current_coordinator_fingerprint != descriptor_coordinator_fingerprint;
    let remote_drifted = current_remote_fingerprint != descriptor_remote_fingerprint;

    if coordinator_drifted && remote_drifted {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: coordinator and remote schema fingerprints differ for node_id {node_id}: descriptor coordinator fingerprint {descriptor_coordinator_fingerprint}, current coordinator fingerprint {current_coordinator_fingerprint}, descriptor remote fingerprint {descriptor_remote_fingerprint}, current remote fingerprint {current_remote_fingerprint}; hint: pause reads, apply matching DDL on both sides, refresh descriptors, then retry"
        ));
    }
    if coordinator_drifted {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: coordinator side drifted for index_oid {}: descriptor coordinator fingerprint {} does not match current coordinator fingerprint {}; hint: pause reads, apply matching DDL on every remote, refresh descriptors, then retry",
            u32::from(index_oid),
            descriptor_coordinator_fingerprint,
            current_coordinator_fingerprint
        ));
    }
    if remote_drifted {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: remote side drifted for node_id {node_id}: descriptor remote fingerprint {descriptor_remote_fingerprint} does not match current remote fingerprint {current_remote_fingerprint}; hint: pause reads, apply matching DDL on the remote, refresh the descriptor, then retry"
        ));
    }
    if current_remote_fingerprint != current_coordinator_fingerprint {
        return Err(format!(
            "ec_spire coordinator {operation} status {SPIRE_REMOTE_STATUS_SCHEMA_DRIFT}: coordinator and remote schema fingerprints differ for node_id {node_id}: coordinator fingerprint {current_coordinator_fingerprint}, remote fingerprint {current_remote_fingerprint}; hint: pause reads, apply matching DDL on the side that drifted, refresh descriptors, then retry"
        ));
    }

    Ok(())
}

fn validate_write_shape_fingerprints_before_remote_dispatch(
    operation: &str,
    dispatch: &SpireCoordinatorInsertDispatchPlanRow,
    conninfo: &str,
) -> Result<(), String> {
    let coordinator_fingerprint = validate_coordinator_write_shape_fingerprint(
        operation,
        dispatch.index_oid,
        &dispatch.coordinator_insert_shape_fingerprint,
    )?;
    validate_remote_write_shape_fingerprint(
        operation,
        dispatch.node_id,
        conninfo,
        &dispatch.remote_index_regclass,
        &coordinator_fingerprint,
        &dispatch.remote_insert_shape_fingerprint,
    )
}

fn coordinator_insert_prepare_row_from_async_result(
    dispatch: &SpireCoordinatorInsertDispatchPlanRow,
    result: &SpireCoordinatorInsertRemotePrepareResult,
) -> SpireCoordinatorInsertRemotePrepareRow {
    SpireCoordinatorInsertRemotePrepareRow {
        node_id: result.node_id,
        prepared_gid: result.prepared_gid.clone(),
        remote_insert_sent: true,
        remote_prepared: true,
        descriptor_generation: dispatch.descriptor_generation.saturating_add(1),
        remote_index_identity: result.remote_index_identity.clone(),
        remote_last_served_epoch: result.remote_last_served_epoch,
        remote_min_retained_epoch: result.remote_last_served_epoch,
        remote_extension_version: result.remote_extension_version.clone(),
        status: SPIRE_COORDINATOR_INSERT_PREPARED_STATUS,
        next_step: SPIRE_COORDINATOR_INSERT_NEXT_STEP_LOCAL_PLACEMENT,
    }
}

pub(crate) unsafe fn coordinator_insert_prepare_remote_sql_batch(
    index_relation: pg_sys::Relation,
    requests: Vec<(u32, u64, String)>,
) -> Result<Vec<SpireCoordinatorInsertRemotePrepareRow>, String> {
    let mut dispatches = Vec::with_capacity(requests.len());
    let mut prepare_requests = Vec::with_capacity(requests.len());
    for (node_id, served_epoch, remote_sql) in requests {
        let dispatch =
            unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
        if dispatch.status != SPIRE_REMOTE_STATUS_READY {
            return Err(format!(
                "ec_spire coordinator insert remote dispatch for node_id {} is blocked with status {}",
                node_id, dispatch.status
            ));
        }
        let conninfo =
            remote_conninfo_secret_value(&dispatch.conninfo_secret_name).map_err(|status| {
                format!(
                    "ec_spire coordinator insert conninfo secret for node_id {node_id} is not resolved: {status}"
                )
            })?;
        validate_write_shape_fingerprints_before_remote_dispatch("insert", &dispatch, &conninfo)?;
        let prepared_gid =
            coordinator_insert_prepared_gid(dispatch.index_oid, node_id, served_epoch);
        coordinator_prepared_xact_intent_record_prepare_requested(
            dispatch.index_oid,
            node_id,
            served_epoch,
            &prepared_gid,
        )?;
        prepare_requests.push(SpireCoordinatorInsertRemotePrepareRequest {
            node_id,
            conninfo,
            remote_index_regclass: dispatch.remote_index_regclass.clone(),
            remote_sql,
            prepared_gid,
        });
        dispatches.push(dispatch);
    }

    let prepare_results =
        SpireRemoteProductionTransportAdapter::run_insert_prepare_requests(prepare_requests)?;
    if prepare_results.len() != dispatches.len() {
        return Err(format!(
            "ec_spire coordinator insert remote prepare returned {} rows for {} dispatches",
            prepare_results.len(),
            dispatches.len()
        ));
    }
    let mut rows = Vec::with_capacity(prepare_results.len());
    for (dispatch, result) in dispatches.iter().zip(&prepare_results) {
        if dispatch.node_id != result.node_id {
            return Err(format!(
                "ec_spire coordinator insert remote prepare result for node_id {} does not match planned node_id {}",
                result.node_id, dispatch.node_id
            ));
        }
        rows.push(coordinator_insert_prepare_row_from_async_result(
            dispatch,
            result,
        ));
        coordinator_prepared_xact_intent_mark(
            &result.prepared_gid,
            SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED,
            SpirePreparedXactIntentTransitionContext::RemotePrepareAck,
        )?;
        coordinator_prepared_xact_intent_mark(
            &result.prepared_gid,
            SPIRE_PREPARED_XACT_INTENT_COMMIT_LOCAL,
            SpirePreparedXactIntentTransitionContext::LocalCommitRecorded,
        )?;
    }

    for result in prepare_results {
        let commit_conninfo = result.conninfo.clone();
        let rollback_conninfo = result.conninfo;
        let node_id = result.node_id;
        let commit_gid = result.prepared_gid.clone();
        let rollback_gid = result.prepared_gid;
        pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Commit, move || {
            coordinator_insert_resolve_remote_prepared(commit_conninfo, node_id, commit_gid, true);
        });
        pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Abort, move || {
            coordinator_insert_resolve_remote_prepared(rollback_conninfo, node_id, rollback_gid, false);
        });
    }

    Ok(rows)
}

pub(crate) unsafe fn coordinator_insert_prepare_remote_sql(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    remote_sql: &str,
) -> Result<SpireCoordinatorInsertRemotePrepareRow, String> {
    unsafe {
        coordinator_insert_prepare_remote_sql_batch(
            index_relation,
            vec![(node_id, served_epoch, remote_sql.to_owned())],
        )
    }?
    .into_iter()
    .next()
    .ok_or_else(|| "ec_spire coordinator insert remote prepare returned no row".to_owned())
}

pub(crate) unsafe fn coordinator_insert_prepare_remote_tuple_payload(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    row_payload_json: &str,
    requested_columns: &[String],
) -> Result<SpireCoordinatorInsertRemotePrepareRow, String> {
    let dispatch =
        unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
    if dispatch.status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire coordinator insert remote dispatch for node_id {} is blocked with status {}",
            node_id, dispatch.status
        ));
    }
    let remote_sql = coordinator_insert_remote_tuple_payload_sql(
        &dispatch.remote_index_regclass,
        row_payload_json,
        requested_columns,
    )?;
    unsafe {
        coordinator_insert_prepare_remote_sql(index_relation, node_id, served_epoch, &remote_sql)
    }
}

pub(crate) unsafe fn coordinator_insert_prepare_remote_tuple_payload_batch(
    index_relation: pg_sys::Relation,
    rows: Vec<(u32, u64, String)>,
    requested_columns: &[String],
) -> Result<Vec<SpireCoordinatorInsertRemotePrepareRow>, String> {
    let mut requests = Vec::with_capacity(rows.len());
    for (node_id, served_epoch, row_payload_json) in rows {
        let dispatch =
            unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
        if dispatch.status != SPIRE_REMOTE_STATUS_READY {
            return Err(format!(
                "ec_spire coordinator insert remote dispatch for node_id {} is blocked with status {}",
                node_id, dispatch.status
            ));
        }
        let remote_sql = coordinator_insert_remote_tuple_payload_sql(
            &dispatch.remote_index_regclass,
            &row_payload_json,
            requested_columns,
        )?;
        requests.push((node_id, served_epoch, remote_sql));
    }
    unsafe { coordinator_insert_prepare_remote_sql_batch(index_relation, requests) }
}

pub(crate) unsafe fn coordinator_update_remote_tuple_payload(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    pk_column: &str,
    pk_value: &[u8],
    row_payload_json: &str,
    updated_columns: &[String],
) -> Result<SpireCoordinatorUpdateRemoteRow, String> {
    let dispatch =
        unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
    if dispatch.status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire coordinator update remote dispatch for node_id {} is blocked with status {}",
            node_id, dispatch.status
        ));
    }
    let remote_sql = coordinator_update_remote_tuple_payload_sql(
        &dispatch.remote_index_regclass,
        pk_column,
        pk_value,
        row_payload_json,
        updated_columns,
    )?;

    let _governance_permit = remote_search_libpq_executor_governance_permit_for_node(node_id)?;
    let conninfo = remote_conninfo_secret_value(&dispatch.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire coordinator update conninfo secret for node_id {node_id} is not resolved: {status}"
        )
    })?;
    validate_write_shape_fingerprints_before_remote_dispatch("update", &dispatch, &conninfo)?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "coordinator update remote dispatch",
    )?;
    let row = client.query_one(remote_sql.as_str(), &[]).map_err(|error| {
        format!("ec_spire coordinator update remote SQL failed for node_id {node_id}: {error}")
    })?;
    let remote_updated_count = row
        .try_get::<_, i64>("updated_count")
        .map_err(|_| "ec_spire coordinator update remote updated_count decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| "ec_spire coordinator update remote updated_count is negative".to_owned())
        })?;

    Ok(SpireCoordinatorUpdateRemoteRow {
        node_id,
        remote_update_sent: true,
        remote_updated_count,
        status: "remote_update_applied",
        next_step: "done",
    })
}

pub(crate) unsafe fn coordinator_delete_prepare_remote_tuple_payload(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    pk_column: &str,
    pk_value: &[u8],
) -> Result<SpireCoordinatorDeleteRemotePrepareRow, String> {
    let dispatch =
        unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
    if dispatch.status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire coordinator delete remote dispatch for node_id {} is blocked with status {}",
            node_id, dispatch.status
        ));
    }
    let remote_sql =
        coordinator_delete_remote_tuple_payload_sql(&dispatch.remote_index_regclass, pk_column, pk_value)?;

    let _governance_permit = remote_search_libpq_executor_governance_permit_for_node(node_id)?;
    let conninfo = remote_conninfo_secret_value(&dispatch.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire coordinator delete conninfo secret for node_id {node_id} is not resolved: {status}"
        )
    })?;
    validate_write_shape_fingerprints_before_remote_dispatch("delete", &dispatch, &conninfo)?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "coordinator delete remote prepare",
    )?;
    let prepared_gid = coordinator_insert_prepared_gid(dispatch.index_oid, node_id, served_epoch);
    coordinator_prepared_xact_intent_record_prepare_requested(
        dispatch.index_oid,
        node_id,
        served_epoch,
        &prepared_gid,
    )?;
    client.batch_execute("BEGIN").map_err(|_| {
        format!(
            "ec_spire coordinator delete failed to begin remote transaction for node_id {node_id}"
        )
    })?;
    let row = match client.query_one(remote_sql.as_str(), &[]) {
        Ok(row) => row,
        Err(error) => {
            let _ = client.batch_execute("ROLLBACK");
            return Err(format!(
                "ec_spire coordinator delete remote SQL failed for node_id {node_id}: {error}"
            ));
        }
    };
    let remote_deleted_count = row
        .try_get::<_, i64>("deleted_count")
        .map_err(|_| "ec_spire coordinator delete remote deleted_count decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value)
                .map_err(|_| "ec_spire coordinator delete remote deleted_count is negative".to_owned())
        })?;
    client
        .batch_execute(&format!(
            "PREPARE TRANSACTION {}",
            quote_sql_literal(&prepared_gid)
        ))
        .map_err(|error| {
            spire_remote_prepare_transaction_error("delete", node_id, &error)
        })?;
    coordinator_prepared_xact_intent_mark(
        &prepared_gid,
        SPIRE_PREPARED_XACT_INTENT_PREPARE_ACKED,
        SpirePreparedXactIntentTransitionContext::RemotePrepareAck,
    )?;
    coordinator_prepared_xact_intent_mark(
        &prepared_gid,
        SPIRE_PREPARED_XACT_INTENT_COMMIT_LOCAL,
        SpirePreparedXactIntentTransitionContext::LocalCommitRecorded,
    )?;

    let commit_conninfo = conninfo.clone();
    let commit_gid = prepared_gid.clone();
    let rollback_gid = prepared_gid.clone();
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Commit, move || {
        coordinator_insert_resolve_remote_prepared(commit_conninfo, node_id, commit_gid, true);
    });
    pgrx::register_xact_callback(pgrx::PgXactCallbackEvent::Abort, move || {
        coordinator_insert_resolve_remote_prepared(conninfo, node_id, rollback_gid, false);
    });

    Ok(SpireCoordinatorDeleteRemotePrepareRow {
        node_id,
        prepared_gid,
        remote_delete_sent: true,
        remote_prepared: true,
        remote_deleted_count,
        status: "remote_delete_prepared",
        next_step: "local_placement_directory_delete",
    })
}

pub(crate) unsafe fn coordinator_select_remote_tuple_payload(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
    pk_column: &str,
    pk_value: &[u8],
    requested_columns: &[String],
) -> Result<SpireCoordinatorSelectRemoteRow, String> {
    let dispatch =
        unsafe { coordinator_insert_dispatch_plan_row(index_relation, node_id, served_epoch) };
    if dispatch.status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire coordinator select remote dispatch for node_id {} is blocked with status {}",
            node_id, dispatch.status
        ));
    }
    let remote_sql = coordinator_select_remote_tuple_payload_sql(
        &dispatch.remote_index_regclass,
        pk_column,
        pk_value,
        requested_columns,
    )?;

    let _governance_permit = remote_search_libpq_executor_governance_permit_for_node(node_id)?;
    let conninfo = remote_conninfo_secret_value(&dispatch.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire coordinator select conninfo secret for node_id {node_id} is not resolved: {status}"
        )
    })?;
    validate_read_shape_fingerprints_before_remote_dispatch(
        dispatch.index_oid,
        dispatch.node_id,
        &conninfo,
        &dispatch.remote_index_regclass,
        &dispatch.coordinator_insert_shape_fingerprint,
        &dispatch.remote_insert_shape_fingerprint,
    )?;
    let mut client = remote_search_libpq_connect_with_session_timeouts(
        &conninfo,
        node_id,
        "coordinator select remote dispatch",
    )?;
    let row = client.query_one(remote_sql.as_str(), &[]).map_err(|error| {
        format!("ec_spire coordinator select remote SQL failed for node_id {node_id}: {error}")
    })?;
    let remote_selected_count = row
        .try_get::<_, i64>("selected_count")
        .map_err(|_| "ec_spire coordinator select remote selected_count decode failed".to_owned())
        .and_then(|value| {
            u64::try_from(value).map_err(|_| {
                "ec_spire coordinator select remote selected_count is negative".to_owned()
            })
        })?;
    let tuple_payload_json = row
        .try_get::<_, Option<String>>("tuple_payload_json")
        .map_err(|_| "ec_spire coordinator select remote tuple payload decode failed".to_owned())?;

    Ok(SpireCoordinatorSelectRemoteRow {
        node_id,
        remote_select_sent: true,
        remote_selected_count,
        tuple_payload_json,
        status: "remote_select_ready",
        next_step: "done",
    })
}
