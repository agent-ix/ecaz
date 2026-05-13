const SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE: &str =
    "SELECT * FROM ec_spire_remote_search($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text)";
const SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE: &str =
    "SELECT * FROM ec_spire_remote_search_local_heap_candidates($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text)";
const SPIRE_REMOTE_SEARCH_LIBPQ_TYPED_TUPLE_PAYLOAD_SQL_TEMPLATE: &str =
    "SELECT requested_epoch, served_epoch, node_id, pid, object_version, row_index, \
            assignment_flags, vec_id, row_locator, heap_block, heap_offset, score, \
            payload_attnums, payload_names, payload_type_oids::text[] AS payload_type_oids, \
            payload_typmods, payload_collations::text[] AS payload_collations, \
            payload_nulls, \
            ARRAY(SELECT encode(payload_value, 'hex') FROM unnest(payload_values) AS payload_value)::text[] AS payload_values_hex, \
            payload_formats, tuple_payload_missing, payload_key, payload_column_count, \
            tuple_transport, tuple_transport_status, status \
       FROM ec_spire_remote_search_tuple_payload_typed($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text, $7::text[])";
const SPIRE_REMOTE_SEARCH_ENDPOINT_IDENTITY_SQL_TEMPLATE: &str =
    "SELECT * FROM ec_spire_remote_search_endpoint_identity($1::oid)";
const SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT: u64 = 6;
const SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR: &str = "validate_remote_search_candidate_batch";
const SPIRE_REMOTE_SEARCH_MERGE_FUNCTION: &str =
    "merge_validated_remote_search_candidate_batches";

fn remote_search_result_column_count() -> u64 {
    u64::try_from(remote_search_libpq_result_contract_rows().len())
        .expect("remote search result contract row count should fit in u64")
}

pub(crate) unsafe fn remote_search_libpq_request_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqRequestPlanRow> {
    let rows = unsafe {
        remote_search_execution_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_request_plan_rows_from_execution(&rows)
}

fn remote_search_libpq_request_plan_rows_from_execution(
    rows: &[SpireRemoteSearchExecutionPlanRow],
) -> Vec<SpireRemoteSearchLibpqRequestPlanRow> {
    rows.iter()
        .filter(|row| row.target_kind == SPIRE_REMOTE_TARGET_REMOTE)
        .map(|row| SpireRemoteSearchLibpqRequestPlanRow {
            requested_epoch: row.requested_epoch,
            node_id: row.node_id,
            selected_pids: row.selected_pids.clone(),
            pid_count: row.pid_count,
            query_dimension: row.query_dimension,
            top_k: row.top_k,
            consistency_mode: row.consistency_mode,
            execution_transport: row.execution_transport,
            sql_template: SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
            parameter_count: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
            result_column_count: remote_search_result_column_count(),
            remote_index_source: row.remote_index_source,
            conninfo_source: row.conninfo_source,
            candidate_format: row.candidate_format,
            status: row.status,
        })
        .collect()
}

pub(crate) unsafe fn remote_search_libpq_request_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqRequestSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqRequestSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq request summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_libpq_request_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut rollup = SpireRemoteCountRollup::default();
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_remote_target(row.pid_count, "remote search libpq request summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search libpq request summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search libpq request summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let request_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search libpq request summary request count exceeds u64")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::LibpqRequest);

        Ok(SpireRemoteSearchLibpqRequestSummaryRow {
            requested_epoch,
            request_count,
            ready_request_count: rollup.ready_count,
            blocked_request_count: rollup.blocked_count,
            remote_pid_count: rollup.remote_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            parameter_count_per_request: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
            result_column_count: remote_search_result_column_count(),
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteLibpqConnectionDescriptorRow {
    descriptor_generation: u64,
    conninfo_secret_name: String,
    remote_index_regclass: String,
    remote_index_identity: Vec<u8>,
    remote_index_identity_bytes: u64,
    coordinator_insert_shape_fingerprint: String,
    remote_insert_shape_fingerprint: String,
    last_served_epoch: u64,
    min_retained_epoch: u64,
}

fn load_remote_libpq_connection_descriptors(
    index_relid: pg_sys::Oid,
    remote_node_ids: &[u32],
) -> Result<HashMap<u32, SpireRemoteLibpqConnectionDescriptorRow>, String> {
    if remote_node_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let node_id_list = remote_node_ids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT node_id::int4, \
                descriptor_generation::bigint, \
                conninfo_secret_name, \
                remote_index_identity, \
                remote_index_regclass, \
                coordinator_insert_shape_fingerprint, \
                remote_insert_shape_fingerprint, \
                last_served_epoch::bigint, \
                min_retained_epoch::bigint \
           FROM ec_spire_remote_node_descriptor \
          WHERE coordinator_index_oid = '{}'::oid \
            AND node_id = ANY (ARRAY[{}]::integer[]) \
            AND descriptor_state IN ('{}', '{}')",
        u32::from(index_relid),
        node_id_list,
        SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE,
        SPIRE_REMOTE_DESCRIPTOR_STATE_DRAINING
    );

    Spi::connect(|client| {
        client
            .select(sql.as_str(), None, &[])
            .map_err(|e| format!("ec_spire libpq connection descriptor read failed: {e}"))?
            .map(|row| {
                let node_id = row["node_id"]
                    .value::<i32>()
                    .map_err(|e| format!("ec_spire libpq connection node_id decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire libpq connection node_id is null".to_owned())
                    .and_then(|value| {
                        u32::try_from(value)
                            .map_err(|_| "ec_spire libpq connection node_id is negative".to_owned())
                    })?;
                let descriptor_generation = row["descriptor_generation"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!(
                            "ec_spire libpq connection descriptor generation decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection descriptor generation is null".to_owned()
                    })
                    .and_then(|value| {
                        u64::try_from(value).map_err(|_| {
                            "ec_spire libpq connection descriptor generation is negative"
                                .to_owned()
                        })
                    })?;
                let conninfo_secret_name = row["conninfo_secret_name"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection conninfo secret decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection conninfo secret is null".to_owned()
                    })?;
                let remote_index_identity = row["remote_index_identity"]
                    .value::<Vec<u8>>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection remote identity decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection remote identity is null".to_owned()
                    })?;
                let remote_index_regclass = row["remote_index_regclass"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection remote regclass decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection remote regclass is null".to_owned()
                    })?;
                let remote_index_identity_bytes = u64::try_from(remote_index_identity.len())
                    .map_err(|_| {
                        "ec_spire libpq connection remote identity length exceeds u64".to_owned()
                    })?;
                let coordinator_insert_shape_fingerprint = row
                    ["coordinator_insert_shape_fingerprint"]
                    .value::<String>()
                    .map_err(|e| {
                        format!(
                            "ec_spire libpq connection coordinator insert shape fingerprint decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection coordinator insert shape fingerprint is null"
                            .to_owned()
                    })?;
                let remote_insert_shape_fingerprint = row["remote_insert_shape_fingerprint"]
                    .value::<String>()
                    .map_err(|e| {
                        format!(
                            "ec_spire libpq connection remote insert shape fingerprint decode failed: {e}"
                        )
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection remote insert shape fingerprint is null"
                            .to_owned()
                    })?;
                let last_served_epoch = row["last_served_epoch"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection last served epoch decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection last served epoch is null".to_owned()
                    })
                    .and_then(|value| {
                        u64::try_from(value).map_err(|_| {
                            "ec_spire libpq connection last served epoch is negative".to_owned()
                        })
                    })?;
                let min_retained_epoch = row["min_retained_epoch"]
                    .value::<i64>()
                    .map_err(|e| {
                        format!("ec_spire libpq connection min retained epoch decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire libpq connection min retained epoch is null".to_owned()
                    })
                    .and_then(|value| {
                        u64::try_from(value).map_err(|_| {
                            "ec_spire libpq connection min retained epoch is negative".to_owned()
                        })
                    })?;

                Ok((
                    node_id,
                    SpireRemoteLibpqConnectionDescriptorRow {
                        descriptor_generation,
                        conninfo_secret_name,
                        remote_index_regclass,
                        remote_index_identity,
                        remote_index_identity_bytes,
                        coordinator_insert_shape_fingerprint,
                        remote_insert_shape_fingerprint,
                        last_served_epoch,
                        min_retained_epoch,
                    },
                ))
            })
            .collect::<Result<HashMap<_, _>, String>>()
    })
}

pub(crate) unsafe fn remote_search_libpq_connection_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqConnectionPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLibpqConnectionPlanRow>, String> {
        let request_rows = unsafe {
            remote_search_libpq_request_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_libpq_connection_plan_rows_from_requests(
            unsafe { (*index_relation).rd_id },
            &request_rows,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_connection_plan_rows_from_requests(
    index_relid: pg_sys::Oid,
    request_rows: &[SpireRemoteSearchLibpqRequestPlanRow],
) -> Result<Vec<SpireRemoteSearchLibpqConnectionPlanRow>, String> {
    let remote_node_ids = request_rows
        .iter()
        .map(|row| row.node_id)
        .collect::<Vec<_>>();
    let descriptors = load_remote_libpq_connection_descriptors(index_relid, &remote_node_ids)?;

    request_rows
        .iter()
        .map(|row| {
            let descriptor = descriptors.get(&row.node_id);
            let pipeline_ready =
                descriptor.is_some() && row.status == SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ;
            Ok(SpireRemoteSearchLibpqConnectionPlanRow {
                requested_epoch: row.requested_epoch,
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                query_dimension: row.query_dimension,
                top_k: row.top_k,
                consistency_mode: row.consistency_mode,
                execution_transport: row.execution_transport,
                conninfo_secret_name: descriptor
                    .map(|row| row.conninfo_secret_name.clone())
                    .unwrap_or_else(|| SPIRE_REMOTE_NONE.to_owned()),
                remote_index_regclass: descriptor
                    .map(|row| row.remote_index_regclass.clone())
                    .unwrap_or_else(|| SPIRE_REMOTE_NONE.to_owned()),
                descriptor_generation: descriptor
                    .map(|row| row.descriptor_generation)
                    .unwrap_or(0),
                remote_index_identity: descriptor
                    .map(|row| row.remote_index_identity.clone())
                    .unwrap_or_default(),
                remote_index_identity_bytes: descriptor
                    .map(|row| row.remote_index_identity_bytes)
                    .unwrap_or(0),
                conninfo_resolution: if descriptor.is_some() {
                    SPIRE_REMOTE_CONNINFO_READY
                } else {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                },
                pipeline_mode: if pipeline_ready {
                    SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE
                } else {
                    SPIRE_REMOTE_NONE
                },
                status: row.status,
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

pub(crate) unsafe fn remote_search_libpq_connection_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqConnectionSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqConnectionSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq connection summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_libpq_connection_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut rollup = SpireRemoteCountRollup::default();
        let mut descriptor_resolved_connection_count = 0_u64;
        let mut missing_descriptor_connection_count = 0_u64;
        let mut pipeline_connection_count = 0_u64;
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in &rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_remote_target(row.pid_count, "remote search libpq connection summary")?;
            rollup
                .record_status(row.status, row.pid_count, "remote search libpq connection summary")?;
            if row.conninfo_resolution == SPIRE_REMOTE_CONNINFO_READY {
                descriptor_resolved_connection_count =
                    descriptor_resolved_connection_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire remote search libpq connection summary resolved count overflow"
                                .to_owned()
                        })?;
            }
            if row.conninfo_resolution == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
                missing_descriptor_connection_count =
                    missing_descriptor_connection_count
                        .checked_add(1)
                        .ok_or_else(|| {
                            "ec_spire remote search libpq connection summary missing descriptor count overflow"
                                .to_owned()
                        })?;
            }
            if row.pipeline_mode == SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE {
                pipeline_connection_count =
                    pipeline_connection_count.checked_add(1).ok_or_else(|| {
                        "ec_spire remote search libpq connection summary pipeline count overflow"
                            .to_owned()
                    })?;
            }
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search libpq connection summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let connection_count = u64::try_from(rows.len()).map_err(|_| {
            "ec_spire remote search libpq connection summary connection count exceeds u64"
        })?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::LibpqRequest);

        Ok(SpireRemoteSearchLibpqConnectionSummaryRow {
            requested_epoch,
            connection_count,
            descriptor_resolved_connection_count,
            missing_descriptor_connection_count,
            pipeline_connection_count,
            remote_pid_count: rollup.remote_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_libpq_dispatch_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqDispatchPlanRow> {
    let connection_rows = unsafe {
        remote_search_libpq_connection_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };

    remote_search_libpq_dispatch_plan_rows_from_connections(&connection_rows)
}

fn remote_search_libpq_dispatch_plan_rows_from_connections(
    connection_rows: &[SpireRemoteSearchLibpqConnectionPlanRow],
) -> Vec<SpireRemoteSearchLibpqDispatchPlanRow> {
    let budget_limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let mut admitted_node_count = 0_u64;
    let mut admitted_pid_count = 0_u64;

    connection_rows
        .iter()
        .map(|row| {
            let budget_blocked = remote_search_libpq_dispatch_budget_blocked(
                row,
                budget_limits,
                admitted_node_count,
                admitted_pid_count,
            );
            if row.pipeline_mode == SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE && !budget_blocked {
                admitted_node_count = admitted_node_count.saturating_add(1);
                admitted_pid_count = admitted_pid_count.saturating_add(row.pid_count);
            }

            let pipeline_mode = if budget_blocked {
                SPIRE_REMOTE_NONE
            } else {
                row.pipeline_mode
            };
            let dispatch_action = if pipeline_mode == SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE {
                SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION
            } else {
                SPIRE_REMOTE_DISPATCH_BLOCKED_ACTION
            };
            let status = if budget_blocked {
                SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD
            } else {
                row.status
            };

            SpireRemoteSearchLibpqDispatchPlanRow {
                requested_epoch: row.requested_epoch,
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                query_dimension: row.query_dimension,
                top_k: row.top_k,
                consistency_mode: row.consistency_mode,
                sql_template: SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
                parameter_count: SPIRE_REMOTE_SEARCH_LIBPQ_PARAMETER_COUNT,
                result_column_count: remote_search_result_column_count(),
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                remote_index_regclass: row.remote_index_regclass.clone(),
                descriptor_generation: row.descriptor_generation,
                remote_index_identity: row.remote_index_identity.clone(),
                pipeline_mode,
                dispatch_action,
                receive_validator: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
                status,
            }
        })
        .collect()
}

pub(crate) unsafe fn coordinator_insert_dispatch_plan_row(
    index_relation: pg_sys::Relation,
    node_id: u32,
    served_epoch: u64,
) -> SpireCoordinatorInsertDispatchPlanRow {
    let index_oid = unsafe { (*index_relation).rd_id };
    let result = (|| -> Result<SpireCoordinatorInsertDispatchPlanRow, String> {
        let descriptors = load_remote_libpq_connection_descriptors(index_oid, &[node_id])?;
        let Some(descriptor) = descriptors.get(&node_id) else {
            return Ok(SpireCoordinatorInsertDispatchPlanRow {
                index_oid,
                node_id,
                served_epoch,
                dispatch_transport: SPIRE_COORDINATOR_INSERT_DISPATCH_TRANSPORT_LIBPQ,
                transaction_protocol: SPIRE_COORDINATOR_INSERT_TRANSACTION_PROTOCOL_2PC,
                conninfo_secret_name: SPIRE_REMOTE_NONE.to_owned(),
                conninfo_provider_lookup_key: SPIRE_REMOTE_NONE.to_owned(),
                remote_index_regclass: SPIRE_REMOTE_NONE.to_owned(),
                descriptor_generation: 0,
                remote_index_identity_bytes: 0,
                coordinator_insert_shape_fingerprint: SPIRE_REMOTE_NONE.to_owned(),
                remote_insert_shape_fingerprint: SPIRE_REMOTE_NONE.to_owned(),
                dispatch_action: SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_BLOCKED,
                status: SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
                next_step: SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
            });
        };

        let secret_status =
            remote_conninfo_secret_resolution_status_row(&descriptor.conninfo_secret_name);
        let epoch_status = if served_epoch > descriptor.last_served_epoch {
            Some(SPIRE_REMOTE_STATUS_STALE_EPOCH)
        } else if served_epoch < descriptor.min_retained_epoch {
            Some(SPIRE_REMOTE_STATUS_RETENTION_GAP)
        } else {
            None
        };
        let (dispatch_action, status, next_step) = if let Some(epoch_status) = epoch_status {
            (
                SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_BLOCKED,
                epoch_status,
                SPIRE_REMOTE_EXECUTOR_STEP_EPOCH_WINDOW,
            )
        } else if secret_status.status == SPIRE_REMOTE_CONNINFO_RESOLVED {
            (
                SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_PREPARE,
                SPIRE_REMOTE_STATUS_READY,
                SPIRE_COORDINATOR_INSERT_NEXT_STEP_PREPARE,
            )
        } else {
            (
                SPIRE_COORDINATOR_INSERT_DISPATCH_ACTION_BLOCKED,
                secret_status.status,
                SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            )
        };

        Ok(SpireCoordinatorInsertDispatchPlanRow {
            index_oid,
            node_id,
            served_epoch,
            dispatch_transport: SPIRE_COORDINATOR_INSERT_DISPATCH_TRANSPORT_LIBPQ,
            transaction_protocol: SPIRE_COORDINATOR_INSERT_TRANSACTION_PROTOCOL_2PC,
            conninfo_secret_name: descriptor.conninfo_secret_name.clone(),
            conninfo_provider_lookup_key: secret_status.provider_lookup_key,
            remote_index_regclass: descriptor.remote_index_regclass.clone(),
            descriptor_generation: descriptor.descriptor_generation,
            remote_index_identity_bytes: descriptor.remote_index_identity_bytes,
            coordinator_insert_shape_fingerprint: descriptor
                .coordinator_insert_shape_fingerprint
                .clone(),
            remote_insert_shape_fingerprint: descriptor.remote_insert_shape_fingerprint.clone(),
            dispatch_action,
            status,
            next_step,
        })
    })();

    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

