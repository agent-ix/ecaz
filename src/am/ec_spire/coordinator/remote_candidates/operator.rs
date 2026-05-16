pub(crate) unsafe fn remote_search_libpq_secret_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqSecretPlanRow> {
    let dispatch_rows = unsafe {
        remote_search_libpq_dispatch_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };

    remote_search_libpq_secret_plan_rows_from_dispatch(&dispatch_rows)
}

fn remote_search_libpq_secret_plan_rows_from_dispatch(
    dispatch_rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
) -> Vec<SpireRemoteSearchLibpqSecretPlanRow> {
    dispatch_rows
        .iter()
        .map(|row| {
            if row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
                let next_executor_step = remote_search_pre_dispatch_blocker_step(row.status);
                let recommendation =
                    remote_search_pre_dispatch_blocker_recommendation(row.status);
                return SpireRemoteSearchLibpqSecretPlanRow {
                    requested_epoch: row.requested_epoch,
                    node_id: row.node_id,
                    selected_pids: row.selected_pids.clone(),
                    pid_count: row.pid_count,
                    conninfo_secret_name: row.conninfo_secret_name.clone(),
                    provider_lookup_key: SPIRE_REMOTE_NONE.to_owned(),
                    resolved_conninfo_bytes: 0,
                    raw_conninfo_exposed: false,
                    secret_resolution_action: SPIRE_REMOTE_NONE,
                    next_executor_step,
                    status: row.status,
                    recommendation,
                };
            }

            let secret_status =
                remote_conninfo_secret_resolution_status_row(&row.conninfo_secret_name);
            let (secret_resolution_action, next_executor_step) =
                if secret_status.status == SPIRE_REMOTE_CONNINFO_RESOLVED {
                    ("resolved_conninfo_secret_reference", "open_libpq_connection")
                } else {
                    (
                        "resolve_conninfo_secret_reference",
                        SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
                    )
                };

            SpireRemoteSearchLibpqSecretPlanRow {
                requested_epoch: row.requested_epoch,
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                provider_lookup_key: secret_status.provider_lookup_key,
                resolved_conninfo_bytes: secret_status.resolved_conninfo_bytes,
                raw_conninfo_exposed: secret_status.raw_conninfo_exposed,
                secret_resolution_action,
                next_executor_step,
                status: secret_status.status,
                recommendation: secret_status.recommendation,
            }
        })
        .collect()
}

pub(crate) unsafe fn remote_search_libpq_secret_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqSecretSummaryRow {
    let rows = unsafe {
        remote_search_libpq_secret_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_secret_summary_from_plan_rows(requested_epoch, &rows)
        .unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_secret_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqSecretPlanRow],
) -> Result<SpireRemoteSearchLibpqSecretSummaryRow, String> {
    let mut resolved_secret_count = 0_u64;
    let mut blocked_secret_count = 0_u64;
    let mut pre_secret_blocked_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut blocked_pid_count = 0_u64;
    let mut first_pre_secret_blocked_status = SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR;
    let mut first_pre_secret_blocked_step = SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR;

    for row in rows {
        add_remote_count(
            &mut remote_pid_count,
            row.pid_count,
            "remote search libpq secret summary",
            "remote PID",
        )?;
        if row.status == SPIRE_REMOTE_CONNINFO_RESOLVED {
            add_remote_count(
                &mut resolved_secret_count,
                1,
                "remote search libpq secret summary",
                "resolved secret",
            )?;
        } else {
            add_remote_count(
                &mut blocked_pid_count,
                row.pid_count,
                "remote search libpq secret summary",
                "blocked PID",
            )?;
            if row.next_executor_step == SPIRE_REMOTE_EXECUTOR_STEP_SECRET {
                add_remote_count(
                    &mut blocked_secret_count,
                    1,
                    "remote search libpq secret summary",
                    "blocked secret",
                )?;
            } else {
                if pre_secret_blocked_count == 0 {
                    first_pre_secret_blocked_status = row.status;
                    first_pre_secret_blocked_step = row.next_executor_step;
                }
                add_remote_count(
                    &mut pre_secret_blocked_count,
                    1,
                    "remote search libpq secret summary",
                    "pre-secret-blocked row",
                )?;
            }
        }
    }

    let secret_count = u64::try_from(rows.len())
        .map_err(|_| "remote search libpq secret count exceeds u64")?;
    let (next_executor_step, status) = if secret_count == 0 {
        (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY)
    } else if pre_secret_blocked_count > 0 {
        (first_pre_secret_blocked_step, first_pre_secret_blocked_status)
    } else if blocked_secret_count > 0 {
        (
            SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
        )
    } else {
        ("open_libpq_connection", SPIRE_REMOTE_CONNINFO_RESOLVED)
    };

    Ok(SpireRemoteSearchLibpqSecretSummaryRow {
        requested_epoch,
        secret_count,
        resolved_secret_count,
        blocked_secret_count,
        remote_pid_count,
        blocked_pid_count,
        next_executor_step,
        status,
    })
}

pub(crate) unsafe fn remote_search_libpq_connection_open_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLibpqConnectionOpenPlanRow> {
    let secret_rows = unsafe {
        remote_search_libpq_secret_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };

    remote_search_libpq_connection_open_plan_rows_from_secrets(&secret_rows)
}

fn remote_search_libpq_connection_open_plan_rows_from_secrets(
    secret_rows: &[SpireRemoteSearchLibpqSecretPlanRow],
) -> Vec<SpireRemoteSearchLibpqConnectionOpenPlanRow> {
    secret_rows
        .iter()
        .map(|row| {
            let (connection_action, next_executor_step, status, recommendation) =
                if row.status == SPIRE_REMOTE_CONNINFO_RESOLVED {
                    (
                        "open_libpq_connection",
                        "enter_libpq_pipeline_mode",
                        SPIRE_REMOTE_EXECUTOR_REQUIRED,
                        "open per-query libpq connection with executor-owned resolved conninfo",
                    )
                } else {
                    (
                        "blocked_before_connection",
                        row.next_executor_step,
                        row.status,
                        row.recommendation,
                    )
                };

            SpireRemoteSearchLibpqConnectionOpenPlanRow {
                requested_epoch: row.requested_epoch,
                node_id: row.node_id,
                selected_pids: row.selected_pids.clone(),
                pid_count: row.pid_count,
                conninfo_secret_name: row.conninfo_secret_name.clone(),
                provider_lookup_key: row.provider_lookup_key.clone(),
                resolved_conninfo_bytes: row.resolved_conninfo_bytes,
                connection_lifecycle_policy: "per_query",
                pooling_policy: "no_pooling_v1",
                connection_action,
                next_executor_step,
                status,
                recommendation,
            }
        })
        .collect()
}

pub(crate) unsafe fn remote_search_libpq_connection_open_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqConnectionOpenSummaryRow {
    let rows = unsafe {
        remote_search_libpq_connection_open_plan_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_libpq_connection_open_summary_from_plan_rows(requested_epoch, &rows)
        .unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_connection_open_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqConnectionOpenPlanRow],
) -> Result<SpireRemoteSearchLibpqConnectionOpenSummaryRow, String> {
    let mut ready_connection_count = 0_u64;
    let mut blocked_connection_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut blocked_pid_count = 0_u64;
    let mut first_blocked_step = SPIRE_REMOTE_NONE;
    let mut first_blocked_status = SPIRE_REMOTE_STATUS_READY;

    for row in rows {
        add_remote_count(
            &mut remote_pid_count,
            row.pid_count,
            "remote search libpq connection-open summary",
            "remote PID",
        )?;
        if row.connection_action == "open_libpq_connection" {
            add_remote_count(
                &mut ready_connection_count,
                1,
                "remote search libpq connection-open summary",
                "ready connection",
            )?;
        } else {
            if blocked_connection_count == 0 {
                first_blocked_step = row.next_executor_step;
                first_blocked_status = row.status;
            }
            add_remote_count(
                &mut blocked_connection_count,
                1,
                "remote search libpq connection-open summary",
                "blocked connection",
            )?;
            add_remote_count(
                &mut blocked_pid_count,
                row.pid_count,
                "remote search libpq connection-open summary",
                "blocked PID",
            )?;
        }
    }

    let connection_count = u64::try_from(rows.len())
        .map_err(|_| "remote search libpq connection-open count exceeds u64")?;
    let (next_executor_step, status) = if connection_count == 0 {
        (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY)
    } else if blocked_connection_count > 0 {
        (first_blocked_step, first_blocked_status)
    } else {
        ("enter_libpq_pipeline_mode", SPIRE_REMOTE_EXECUTOR_REQUIRED)
    };

    Ok(SpireRemoteSearchLibpqConnectionOpenSummaryRow {
        requested_epoch,
        connection_count,
        ready_connection_count,
        blocked_connection_count,
        remote_pid_count,
        blocked_pid_count,
        next_executor_step,
        status,
    })
}

pub(crate) unsafe fn remote_search_libpq_executor_readiness_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqExecutorReadinessRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqExecutorReadinessRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq executor readiness top_k exceeds u64")?;
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let dispatch_summary = remote_search_libpq_dispatch_summary_from_plan_rows(
            requested_epoch,
            &dispatch_rows,
            query_for_empty_plan,
            top_k_for_empty_plan,
            consistency_mode,
        )?;
        let secret_rows = remote_search_libpq_secret_plan_rows_from_dispatch(&dispatch_rows);
        let secret_summary =
            remote_search_libpq_secret_summary_from_plan_rows(requested_epoch, &secret_rows)?;

        Ok(remote_search_libpq_executor_readiness_from_summaries(
            requested_epoch,
            &dispatch_summary,
            &secret_summary,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_executor_readiness_from_summaries(
    requested_epoch: u64,
    dispatch_summary: &SpireRemoteSearchLibpqDispatchSummaryRow,
    secret_summary: &SpireRemoteSearchLibpqSecretSummaryRow,
) -> SpireRemoteSearchLibpqExecutorReadinessRow {
    let blocked_dispatch_count = dispatch_summary
        .dispatch_count
        .saturating_sub(dispatch_summary.pipeline_dispatch_count);

    let (
        secret_resolution_action,
        connection_action,
        pipeline_action,
        send_action,
        receive_action,
        merge_action,
        next_executor_step,
        status,
        recommendation,
    ) = if dispatch_summary.status == SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD {
        (
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_EXECUTOR_STEP_BUDGET,
            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            remote_search_pre_dispatch_blocker_recommendation(
                SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            ),
        )
    } else if dispatch_summary.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
        (
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            "register active or draining remote node descriptors before libpq executor startup",
        )
    } else if dispatch_summary.pipeline_dispatch_count > 0
        && secret_summary.status == SPIRE_REMOTE_CONNINFO_RESOLVED
    {
        (
            "resolve_conninfo_secret_reference",
            "open_libpq_connection",
            "enter_libpq_pipeline_mode",
            "send_remote_search_request",
            SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
            "open_libpq_connection",
            SPIRE_REMOTE_EXECUTOR_REQUIRED,
            "open executor-owned libpq connections before remote dispatch",
        )
    } else if dispatch_summary.pipeline_dispatch_count > 0 {
        (
            "resolve_conninfo_secret_reference",
            "open_libpq_connection",
            "enter_libpq_pipeline_mode",
            "send_remote_search_request",
            SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
            SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            SPIRE_REMOTE_EXECUTOR_REQUIRED,
            "implement conninfo secret resolution and libpq pipeline execution before remote dispatch",
        )
    } else {
        let next_executor_step = remote_search_pre_dispatch_blocker_step(dispatch_summary.status);
        let recommendation =
            remote_search_pre_dispatch_blocker_recommendation(dispatch_summary.status);
        (
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            next_executor_step,
            dispatch_summary.status,
            recommendation,
        )
    };

    SpireRemoteSearchLibpqExecutorReadinessRow {
        requested_epoch,
        dispatch_count: dispatch_summary.dispatch_count,
        pipeline_dispatch_count: dispatch_summary.pipeline_dispatch_count,
        blocked_dispatch_count,
        secret_resolution_action,
        connection_action,
        pipeline_action,
        send_action,
        receive_action,
        merge_action,
        next_executor_step,
        status,
        recommendation,
    }
}

pub(crate) fn remote_search_libpq_executor_step_contract_rows(
) -> Vec<SpireRemoteSearchLibpqExecutorStepContractRow> {
    vec![
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 1,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
            executor_action: "resolve_remote_node_descriptor",
            input_contract: "remote_leaf_pid_placement",
            output_contract: "active_or_draining_remote_node_descriptor",
            blocking_status: SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            validator: "descriptor_state_must_allow_pipeline_dispatch",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 2,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_EPOCH_WINDOW,
            executor_action: "verify_remote_served_epoch_window",
            input_contract: "remote_node_descriptor.last_served_epoch,min_retained_epoch",
            output_contract: "descriptor_epoch_window_covers_requested_epoch",
            blocking_status: SPIRE_REMOTE_STATUS_STALE_EPOCH,
            validator: "descriptor_epoch_window_must_cover_requested_epoch",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 3,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_EXTENSION_VERSION,
            executor_action: "verify_remote_extension_version",
            input_contract: "remote_node_descriptor.extension_version",
            output_contract: "extension_version_matches_coordinator",
            blocking_status: SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION,
            validator: "descriptor_extension_version_must_match_coordinator",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 4,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_BUDGET,
            executor_action: "enforce_remote_executor_budget",
            input_contract: "ready_remote_dispatch_rows_and_session_budget_gucs",
            output_contract: "budget_admitted_remote_dispatch_rows",
            blocking_status: SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD,
            validator: "must_block_over_budget_rows_before_secret_lookup_or_socket_open",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 5,
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            executor_action: "resolve_conninfo_secret_reference",
            input_contract: "conninfo_secret_name",
            output_contract: SPIRE_REMOTE_CONNINFO_READY,
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_resolve_secret_without_exposing_raw_conninfo",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 6,
            step_name: "open_libpq_connection",
            executor_action: "open_libpq_connection",
            input_contract: SPIRE_REMOTE_CONNINFO_READY,
            output_contract: "libpq_connection",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_target_registered_remote_index",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 7,
            step_name: "enter_libpq_pipeline_mode",
            executor_action: "enter_libpq_pipeline_mode",
            input_contract: "libpq_connection",
            output_contract: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_enter_pipeline_before_sending_remote_search",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 8,
            step_name: "send_remote_search_request",
            executor_action: "send_remote_search_request",
            input_contract: "ec_spire_remote_search_libpq_request_plan",
            output_contract: "pending_remote_search_result",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_bind_libpq_parameter_contract",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 9,
            step_name: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            executor_action: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            input_contract: "remote_search_result_batch",
            output_contract: "validated_remote_candidate_batch",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_match_libpq_result_contract",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 10,
            step_name: SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
            executor_action: SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
            input_contract: "validated_remote_candidate_batches",
            output_contract: "coordinator_ranked_candidate_batch",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_preserve_merge_order_contract",
        },
    ]
}

pub(crate) fn remote_search_libpq_parameter_contract_rows(
) -> Vec<SpireRemoteSearchLibpqParameterContractRow> {
    vec![
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 1,
            parameter_name: "remote_index_oid",
            pg_type: "oid",
            semantic_role: "remote_index_identity",
            validator: "must_resolve_to_ec_spire_index_on_remote_node",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 2,
            parameter_name: "requested_epoch",
            pg_type: "bigint",
            semantic_role: "served_epoch_gate",
            validator: "must_be_positive_and_served_by_remote_node",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 3,
            parameter_name: "query",
            pg_type: "real[]",
            semantic_role: "query_vector",
            validator: "must_match_index_dimensions",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 4,
            parameter_name: "selected_pids",
            pg_type: "bigint[]",
            semantic_role: "selected_leaf_pid_set",
            validator: "must_be_nonempty_positive_unique_remote_leaf_pids_delta_rows_are_leaf_derived",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 5,
            parameter_name: "top_k",
            pg_type: "integer",
            semantic_role: "candidate_budget",
            validator: "must_be_non_negative",
        },
        SpireRemoteSearchLibpqParameterContractRow {
            parameter_ordinal: 6,
            parameter_name: "consistency_mode",
            pg_type: "text",
            semantic_role: "strict_or_degraded_policy",
            validator: "must_match_active_remote_epoch_consistency_mode",
        },
    ]
}

pub(crate) fn remote_search_libpq_result_contract_rows(
) -> Vec<SpireRemoteSearchLibpqResultContractRow> {
    vec![
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 1,
            column_name: "served_epoch",
            pg_type: "bigint",
            semantic_role: "candidate_epoch",
            nullable: false,
            validator: "must_equal_requested_epoch",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 2,
            column_name: "node_id",
            pg_type: "bigint",
            semantic_role: "candidate_node",
            nullable: false,
            validator: "must_equal_expected_node_id",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 3,
            column_name: "pid",
            pg_type: "bigint",
            semantic_role: "partition_object",
            nullable: false,
            validator: "must_be_selected_leaf_pid_or_leaf_derived_delta_pid",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 4,
            column_name: "object_version",
            pg_type: "bigint",
            semantic_role: "partition_object_version",
            nullable: false,
            validator: "must_be_positive",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 5,
            column_name: "row_index",
            pg_type: "bigint",
            semantic_role: "candidate_row_index",
            nullable: false,
            validator: "must_fit_u32",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 6,
            column_name: "assignment_flags",
            pg_type: "smallint",
            semantic_role: "candidate_assignment_flags",
            nullable: false,
            validator: "must_include_primary_or_boundary_replica",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 7,
            column_name: "vec_id",
            pg_type: "bytea",
            semantic_role: "dedupe_key",
            nullable: false,
            validator: "must_be_nonempty",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 8,
            column_name: "row_locator",
            pg_type: "bytea",
            semantic_role: "origin_node_locator",
            nullable: false,
            validator: "must_be_nonempty_and_opaque",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 9,
            column_name: "score",
            pg_type: "real",
            semantic_role: "candidate_score",
            nullable: false,
            validator: "must_be_finite",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 10,
            column_name: "protocol_version",
            pg_type: "text",
            semantic_role: "remote_endpoint_protocol",
            nullable: false,
            validator: "must_equal_ec_spire_remote_search_v1",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 11,
            column_name: "extension_version",
            pg_type: "text",
            semantic_role: "remote_extension_version",
            nullable: false,
            validator: "must_match_required_extension_version",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 12,
            column_name: "opclass_identity",
            pg_type: "text",
            semantic_role: "remote_opclass_identity",
            nullable: false,
            validator: "must_be_nonempty",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 13,
            column_name: "storage_format",
            pg_type: "text",
            semantic_role: "remote_storage_format",
            nullable: false,
            validator: "must_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 14,
            column_name: "assignment_payload_format",
            pg_type: "text",
            semantic_role: "remote_assignment_payload_format",
            nullable: false,
            validator: "must_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 15,
            column_name: "quantizer_profile",
            pg_type: "text",
            semantic_role: "remote_quantizer_profile",
            nullable: false,
            validator: "must_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 16,
            column_name: "scoring_profile",
            pg_type: "text",
            semantic_role: "remote_scoring_profile",
            nullable: false,
            validator: "must_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 17,
            column_name: "profile_fingerprint",
            pg_type: "text",
            semantic_role: "remote_quantizer_index_fingerprint",
            nullable: false,
            validator: "must_be_nonempty_and_match_served_endpoint_identity",
        },
        SpireRemoteSearchLibpqResultContractRow {
            column_ordinal: 18,
            column_name: "endpoint_status",
            pg_type: "text",
            semantic_role: "remote_endpoint_identity_status",
            nullable: false,
            validator: "must_be_ready_before_production_merge",
        },
    ]
}

