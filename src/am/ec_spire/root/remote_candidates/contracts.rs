#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireRemoteSearchLibpqExecutorBudgetLimits {
    max_nodes: u64,
    max_pids: u64,
    max_pids_per_node: u64,
    max_concurrent_dispatches: u64,
    max_concurrent_dispatches_per_node: u64,
    connect_timeout_ms: u64,
    statement_timeout_ms: u64,
}

impl SpireRemoteSearchLibpqExecutorBudgetLimits {
    fn from_session() -> Self {
        Self {
            max_nodes: session_limit_to_u64(options::current_session_remote_search_max_nodes()),
            max_pids: session_limit_to_u64(options::current_session_remote_search_max_pids()),
            max_pids_per_node: session_limit_to_u64(
                options::current_session_remote_search_max_pids_per_node(),
            ),
            max_concurrent_dispatches: session_limit_to_u64(
                options::current_session_remote_search_max_concurrent_dispatches(),
            ),
            max_concurrent_dispatches_per_node: session_limit_to_u64(
                options::current_session_remote_search_max_concurrent_dispatches_per_node(),
            ),
            connect_timeout_ms: session_limit_to_u64(
                options::current_session_remote_search_connect_timeout_ms(),
            ),
            statement_timeout_ms: session_limit_to_u64(
                options::current_session_remote_search_statement_timeout_ms(),
            ),
        }
    }

    fn has_node_cap(self) -> bool {
        self.max_nodes > 0
    }

    fn has_pid_cap(self) -> bool {
        self.max_pids > 0
    }

    fn has_pid_per_node_cap(self) -> bool {
        self.max_pids_per_node > 0
    }

    fn has_concurrent_dispatch_cap(self) -> bool {
        self.max_concurrent_dispatches > 0
    }

    fn has_concurrent_dispatch_per_node_cap(self) -> bool {
        self.max_concurrent_dispatches_per_node > 0
    }
}

fn session_limit_to_u64(value: i32) -> u64 {
    u64::try_from(value.max(0)).expect("non-negative session limit should fit in u64")
}

fn session_limit_to_usize(value: i32) -> usize {
    usize::try_from(value.max(0)).expect("non-negative session limit should fit in usize")
}

fn remote_search_pre_dispatch_blocker_step(status: &str) -> &'static str {
    match status {
        SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR => SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
        SPIRE_REMOTE_STATUS_STALE_EPOCH | SPIRE_REMOTE_STATUS_RETENTION_GAP => {
            SPIRE_REMOTE_EXECUTOR_STEP_EPOCH_WINDOW
        }
        SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION => {
            SPIRE_REMOTE_EXECUTOR_STEP_EXTENSION_VERSION
        }
        SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD => SPIRE_REMOTE_EXECUTOR_STEP_BUDGET,
        _ => SPIRE_REMOTE_NONE,
    }
}

fn remote_search_pre_dispatch_blocker_recommendation(status: &str) -> &'static str {
    match status {
        SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR => {
            "register active or draining remote node descriptors before libpq executor startup"
        }
        SPIRE_REMOTE_STATUS_STALE_EPOCH | SPIRE_REMOTE_STATUS_RETENTION_GAP => {
            "refresh remote node served epoch window before libpq fanout execution"
        }
        SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION => {
            "upgrade remote node extension before libpq fanout execution"
        }
        SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD => {
            "raise ec_spire remote-search executor budgets or reduce remote fanout before libpq dispatch"
        }
        _ => SPIRE_REMOTE_NONE,
    }
}

pub(crate) fn remote_operator_entrypoint_contract_rows(
) -> Vec<SpireRemoteOperatorEntrypointContractRow> {
    vec![
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 1,
            entrypoint_name: "ec_spire_remote_search_coordinator_gate_summary",
            area: "search",
            operator_use: "pre_result_gate",
            status_source: "status,next_blocker,libpq_executor_next_step",
            next_action: "resolve_reported_blocker_before_expect_result_rows",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 2,
            entrypoint_name: "ec_spire_remote_search_coordinator_result_summary",
            area: "search",
            operator_use: "final_result_source",
            status_source: "result_source,status,next_blocker",
            next_action: "consume_local_heap_candidates_or_resolve_blocker",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 3,
            entrypoint_name: "ec_spire_remote_node_snapshot",
            area: "descriptor",
            operator_use: "node_inventory",
            status_source: "descriptor_state,status,recommendation",
            next_action: "register_or_refresh_remote_node_descriptor",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 4,
            entrypoint_name: "ec_spire_remote_node_descriptor_readiness_summary",
            area: "descriptor",
            operator_use: "descriptor_field_gate",
            status_source: "ready_field_count,blocked_field_count,status",
            next_action: "fill_required_descriptor_fields_before_transport",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 5,
            entrypoint_name: "ec_spire_remote_node_capability_summary",
            area: "capability",
            operator_use: "remote_serving_window_gate",
            status_source: "ready_node_count,blocked_node_count,status",
            next_action: "refresh_served_epoch_or_remote_index_identity",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 6,
            entrypoint_name: "ec_spire_remote_epoch_publish_gate_summary",
            area: "manifest",
            operator_use: "distributed_publish_gate",
            status_source: "publish_decision,status,next_blocker",
            next_action: "persist_manifest_only_after_publish_gate_ready",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 7,
            entrypoint_name: "ec_spire_remote_epoch_manifest_catalog_summary",
            area: "manifest",
            operator_use: "manifest_persistence_gate",
            status_source: "catalog_status,persisted_manifest_count,persisted_entry_mismatch_count",
            next_action: "persist_or_refresh_remote_epoch_manifest_catalog",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 8,
            entrypoint_name: "ec_spire_remote_epoch_manifest_publication_result_summary",
            area: "publication",
            operator_use: "manifest_publication_result",
            status_source: "result_source,status,next_blocker",
            next_action: "run_libpq_executor_or_resolve_publication_blocker",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 9,
            entrypoint_name: "ec_spire_remote_search_libpq_secret_summary",
            area: "search",
            operator_use: "search_conninfo_secret_gate",
            status_source: "status,next_executor_step,blocked_secret_count",
            next_action: "resolve_missing_conninfo_secrets_before_opening_libpq_connections",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 10,
            entrypoint_name: "ec_spire_remote_conninfo_secret_resolution_status",
            area: "secret",
            operator_use: "single_conninfo_secret_probe",
            status_source: "status,provider_lookup_key,resolved_conninfo_bytes",
            next_action: "set_executor_owned_secret_provider_value_without_exposing_raw_conninfo",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 11,
            entrypoint_name: "ec_spire_remote_search_production_executor_state_summary",
            area: "search",
            operator_use: "production_executor_dry_state",
            status_source: "state_model,dispatch_count,next_executor_step,status",
            next_action: "use_dry_state_for_planning_before_async_pipeline_transport_lands",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 12,
            entrypoint_name: "ec_spire_remote_pipeline_steps",
            area: "search",
            operator_use: "consolidated_remote_pipeline_steps_dry",
            status_source: "step_name,status,item_count,next_blocker",
            next_action: "inspect_first_non_ready_step_before_live_probe_or_narrow_surfaces",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 13,
            entrypoint_name: "ec_spire_remote_pipeline_steps_live",
            area: "search",
            operator_use: "consolidated_remote_pipeline_steps_live_probe",
            status_source: "step_name,status,item_count,next_blocker",
            next_action: "run_only_when_connection_and_remote_executor_probe_cost_is_expected",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 14,
            entrypoint_name: "ec_spire_remote_search_vector_identity_contract",
            area: "search",
            operator_use: "remote_dedupe_identity_contract",
            status_source: "contract_item,contract_value,status",
            next_action: "require_global_vec_ids_before_cross_node_replica_dedupe",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 15,
            entrypoint_name: "ec_spire_remote_search_endpoint_contract",
            area: "search",
            operator_use: "remote_endpoint_contract_gate",
            status_source: "contract_item,contract_value,status,validator",
            next_action: "resolve_non_ready_endpoint_contract_rows_before_production_remote_merge",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 16,
            entrypoint_name: "ec_spire_remote_search_endpoint_identity",
            area: "search",
            operator_use: "remote_endpoint_identity_gate",
            status_source: "protocol_version,opclass_identity,profile_fingerprint,status",
            next_action: "require_ready_endpoint_identity_before_accepting_remote_candidate_scores",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 17,
            entrypoint_name: "ec_spire_remote_search_libpq_executor_receive_attempts",
            area: "search",
            operator_use: "per_node_remote_receive_attempt_diagnostics",
            status_source: "node_id,status,next_blocker,failure_action,failure_reason",
            next_action: "use_strict_fail_closed_or_degraded_skip_reason_before_merge",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 18,
            entrypoint_name: "ec_spire_remote_search_libpq_executor_budget_summary",
            area: "search",
            operator_use: "remote_executor_resource_governance",
            status_source: "status,next_executor_step,max_nodes,max_pids,max_pids_per_node",
            next_action: "tune_remote_executor_budgets_or_reduce_fanout_before_dispatch",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 19,
            entrypoint_name: "ec_spire_remote_search_production_scan_handoff_summary",
            area: "search",
            operator_use: "production_am_scan_candidate_handoff",
            status_source: "selected_pid_count,candidate_row_count,status,next_blocker",
            next_action: "resolve remote heap rows before allowing remote SQL tuple return",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 20,
            entrypoint_name: "ec_spire_remote_search_stage_e_fault_matrix",
            area: "search",
            operator_use: "local_multi_instance_fault_fixture_contract",
            status_source: "fault_case,failure_category,strict_action,degraded_action,counter_delta",
            next_action: "implement_stage_e_fault_fixture_against_each_named_case",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 21,
            entrypoint_name: "ec_spire_remote_search_operator_diagnostics",
            area: "search",
            operator_use: "packet_friendly_production_readiness_rollup",
            status_source: "remote_readiness_status,candidate_batch_count,final_heap_fetch_status,am_delivery_status,next_blocker",
            next_action: "inspect_single_rollup_before_running_multi_instance_fault_fixture",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 22,
            entrypoint_name: "ec_spire_remote_search_stage_e_lifecycle_matrix",
            area: "search",
            operator_use: "local_multi_instance_lifecycle_fixture_contract",
            status_source: "lifecycle_case,strict_action,degraded_action,required_detection,next_executor_step",
            next_action: "implement_drop_reindex_create_concurrently_fixture_against_each_named_case",
        },
        SpireRemoteOperatorEntrypointContractRow {
            entrypoint_ordinal: 23,
            entrypoint_name: "ec_spire_remote_epoch_manifest_freshness",
            area: "manifest",
            operator_use: "stage_e_manifest_freshness_assertion",
            status_source: "node_id,freshness_status,persisted_entry_matches,next_action",
            next_action: "persist_or_refresh_manifest_before_stage_e_fixture_execution",
        },
    ]
}

pub(crate) fn remote_libpq_connection_lifecycle_contract_rows(
) -> Vec<SpireRemoteLibpqConnectionLifecycleContractRow> {
    vec![
        SpireRemoteLibpqConnectionLifecycleContractRow {
            surface: "ec_spire_remote_search_libpq_executor",
            connection_lifecycle_policy: "per_query",
            pooling_policy: "no_pooling_v1",
            secret_resolution_policy: "conninfo_secret_name_resolved_by_executor",
            conninfo_exposure_policy: "never_expose_raw_conninfo_in_sql",
            failure_policy: "fail_closed_no_implicit_retry",
            resource_limit_policy: "bounded_by_ec_spire_remote_search_session_caps",
            validator: "must_close_connection_before_coordinator_returns",
            recommendation: "enforce remote executor budgets before secret lookup or socket open",
        },
        SpireRemoteLibpqConnectionLifecycleContractRow {
            surface: "ec_spire_remote_epoch_manifest_publication_libpq_executor",
            connection_lifecycle_policy: "per_query",
            pooling_policy: "no_pooling_v1",
            secret_resolution_policy: "conninfo_secret_name_resolved_by_executor",
            conninfo_exposure_policy: "never_expose_raw_conninfo_in_sql",
            failure_policy: "fail_closed_no_implicit_retry",
            resource_limit_policy: "one_connection_per_ready_remote_node_per_publication",
            validator: "must_close_connection_before_publication_result_returns",
            recommendation: "share executor secret provider with remote search publication transport",
        },
    ]
}

pub(crate) fn remote_conninfo_secret_resolution_contract_rows(
) -> Vec<SpireRemoteConninfoSecretResolutionContractRow> {
    vec![
        SpireRemoteConninfoSecretResolutionContractRow {
            provider_ordinal: 1,
            provider_policy: "external_executor_secret_provider",
            provider_status: "selected_v1",
            secret_reference_field: "conninfo_secret_name",
            sql_storage_policy: "descriptor_catalog_stores_secret_reference_only",
            raw_conninfo_allowed: false,
            executor_action: "resolve_conninfo_secret_reference",
            failure_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_resolve_to_libpq_conninfo_without_sql_exposure",
            recommendation: "configure the executor secret provider before enabling remote transport",
        },
        SpireRemoteConninfoSecretResolutionContractRow {
            provider_ordinal: 2,
            provider_policy: "postgres_fdw_user_mapping",
            provider_status: "not_selected_v1",
            secret_reference_field: "conninfo_secret_name",
            sql_storage_policy: "no_postgres_fdw_dependency_for_v1",
            raw_conninfo_allowed: false,
            executor_action: "not_used",
            failure_status: "unsupported_secret_provider",
            validator: "must_not_assume_fdw_user_mapping_exists",
            recommendation: "keep FDW-style mapping as a future integration option",
        },
        SpireRemoteConninfoSecretResolutionContractRow {
            provider_ordinal: 3,
            provider_policy: "in_extension_conninfo_table",
            provider_status: "rejected_v1",
            secret_reference_field: "conninfo_secret_name",
            sql_storage_policy: "never_store_raw_conninfo_in_extension_catalog",
            raw_conninfo_allowed: false,
            executor_action: "not_used",
            failure_status: "unsupported_secret_provider",
            validator: "must_not_persist_raw_conninfo_in_ec_spire_catalogs",
            recommendation: "use external secret storage instead of an extension-owned conninfo table",
        },
    ]
}

pub(crate) fn remote_conninfo_secret_provider_lookup_key(
    conninfo_secret_name: &str,
) -> Result<String, String> {
    if conninfo_secret_name.is_empty() {
        return Err("conninfo_secret_name must be nonempty".to_owned());
    }

    let mut key = String::from(SPIRE_REMOTE_CONNINFO_ENV_PREFIX);
    for byte in conninfo_secret_name.bytes() {
        if byte.is_ascii_alphanumeric() {
            key.push(char::from(byte).to_ascii_uppercase());
        } else {
            key.push('_');
        }
    }
    Ok(key)
}

pub(crate) fn remote_conninfo_secret_resolution_status_row(
    conninfo_secret_name: &str,
) -> SpireRemoteConninfoSecretResolutionStatusRow {
    let provider_lookup_key = remote_conninfo_secret_provider_lookup_key(conninfo_secret_name)
        .unwrap_or_else(|e| pgrx::error!("ec_spire remote conninfo secret reference invalid: {e}"));

    match std::env::var(&provider_lookup_key) {
        Ok(conninfo) if !conninfo.is_empty() => SpireRemoteConninfoSecretResolutionStatusRow {
            provider_policy: "external_executor_secret_provider",
            conninfo_secret_name: conninfo_secret_name.to_owned(),
            provider_lookup_key,
            resolved_conninfo_bytes: u64::try_from(conninfo.len())
                .expect("conninfo byte length should fit in u64"),
            raw_conninfo_exposed: false,
            status: SPIRE_REMOTE_CONNINFO_RESOLVED,
            recommendation: "open libpq connection with executor-owned resolved conninfo",
        },
        Ok(_) => SpireRemoteConninfoSecretResolutionStatusRow {
            provider_policy: "external_executor_secret_provider",
            conninfo_secret_name: conninfo_secret_name.to_owned(),
            provider_lookup_key,
            resolved_conninfo_bytes: 0,
            raw_conninfo_exposed: false,
            status: SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
            recommendation: "configure a nonempty conninfo value in the external secret provider",
        },
        Err(_) => SpireRemoteConninfoSecretResolutionStatusRow {
            provider_policy: "external_executor_secret_provider",
            conninfo_secret_name: conninfo_secret_name.to_owned(),
            provider_lookup_key,
            resolved_conninfo_bytes: 0,
            raw_conninfo_exposed: false,
            status: SPIRE_REMOTE_STATUS_REQUIRES_SECRET,
            recommendation: "configure the external secret provider entry for conninfo_secret_name",
        },
    }
}

pub(crate) fn remote_catalog_lifecycle_contract_rows(
) -> Vec<SpireRemoteCatalogLifecycleContractRow> {
    vec![
        SpireRemoteCatalogLifecycleContractRow {
            lifecycle_ordinal: 1,
            lifecycle_event: "pg_dump_restore",
            oid_stability: "oids_reassigned",
            catalog_risk: "catalog_rows_reference_dump_source_oids",
            operator_action: "run_orphan_cleanup_and_reregister_descriptors",
            cleanup_surface: "ec_spire_remote_catalog_orphan_summary,ec_spire_remote_catalog_orphan_cleanup",
            migration_surface: "bootstrap_remote_catalog_tables",
            status: "requires_operator_reregistration",
            recommendation: "after logical restore, clean orphaned remote catalog rows and re-register remote node descriptors for restored coordinator indexes",
        },
        SpireRemoteCatalogLifecycleContractRow {
            lifecycle_ordinal: 2,
            lifecycle_event: "drop_index",
            oid_stability: "coordinator_oid_removed",
            catalog_risk: "remote_catalog_orphans",
            operator_action: "event_trigger_runs_remote_catalog_index_cleanup",
            cleanup_surface: "ec_spire_remote_catalog_index_cleanup,ec_spire_remote_catalog_orphan_cleanup",
            migration_surface: "ec_spire_remote_catalog_drop_index_cleanup",
            status: "automatic_event_trigger_cleanup",
            recommendation: "DROP INDEX automatically removes matching remote catalog rows; use orphan cleanup for restore-era sweeps",
        },
        SpireRemoteCatalogLifecycleContractRow {
            lifecycle_ordinal: 3,
            lifecycle_event: "basebackup_wal_replay",
            oid_stability: "oids_stable",
            catalog_risk: "catalog_rows_replay_with_database",
            operator_action: "no_reregistration_required",
            cleanup_surface: "none",
            migration_surface: "physical_backup",
            status: "supported",
            recommendation: "physical backups preserve coordinator index OIDs and remote catalog rows through WAL replay",
        },
        SpireRemoteCatalogLifecycleContractRow {
            lifecycle_ordinal: 4,
            lifecycle_event: "extension_upgrade_0_1_0_to_0_1_1",
            oid_stability: "oids_stable",
            catalog_risk: "remote_catalog_tables_absent_before_upgrade",
            operator_action: "apply_extension_upgrade_before_remote_transport",
            cleanup_surface: "none",
            migration_surface: "ecaz--0.1.0--0.1.1.sql",
            status: "supported_after_upgrade_script",
            recommendation: "extension upgrade creates remote catalog tables before remote transport is enabled",
        },
    ]
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct SpireRemoteCountRollup {
    local_count: u64,
    remote_count: u64,
    skipped_count: u64,
    ready_count: u64,
    blocked_count: u64,
    degraded_skipped_count: u64,
    missing_descriptor_count: u64,
    transport_count: u64,
    local_pid_count: u64,
    remote_pid_count: u64,
    skipped_pid_count: u64,
    ready_pid_count: u64,
    blocked_pid_count: u64,
    missing_descriptor_pid_count: u64,
    transport_pid_count: u64,
    first_remote_blocked_status: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpireRemoteSummaryStatusMode {
    RequestPlan,
    Readiness,
    Execution,
    LibpqRequest,
}

fn add_remote_count(value: &mut u64, amount: u64, context: &str, field: &str) -> Result<(), String> {
    *value = value
        .checked_add(amount)
        .ok_or_else(|| format!("ec_spire {context} {field} count overflowed"))?;
    Ok(())
}

impl SpireRemoteCountRollup {
    fn record_target(
        &mut self,
        target_kind: &str,
        pid_count: u64,
        context: &str,
    ) -> Result<(), String> {
        match target_kind {
            SPIRE_REMOTE_TARGET_LOCAL => {
                add_remote_count(&mut self.local_count, 1, context, "local")?;
                add_remote_count(&mut self.local_pid_count, pid_count, context, "local PID")?;
            }
            SPIRE_REMOTE_TARGET_REMOTE => {
                add_remote_count(&mut self.remote_count, 1, context, "remote")?;
                add_remote_count(&mut self.remote_pid_count, pid_count, context, "remote PID")?;
            }
            SPIRE_REMOTE_TARGET_SKIPPED => {
                add_remote_count(&mut self.skipped_count, 1, context, "skipped")?;
                add_remote_count(&mut self.skipped_pid_count, pid_count, context, "skipped PID")?;
            }
            target_kind => {
                return Err(format!(
                    "ec_spire {context} found unknown target_kind '{target_kind}'"
                ));
            }
        }
        Ok(())
    }

    fn record_remote_target(&mut self, pid_count: u64, context: &str) -> Result<(), String> {
        add_remote_count(&mut self.remote_count, 1, context, "remote")?;
        add_remote_count(&mut self.remote_pid_count, pid_count, context, "remote PID")
    }

    fn record_status(
        &mut self,
        status: &str,
        pid_count: u64,
        context: &str,
    ) -> Result<(), String> {
        match status {
            SPIRE_REMOTE_STATUS_READY => {
                add_remote_count(&mut self.ready_count, 1, context, "ready")?;
                add_remote_count(&mut self.ready_pid_count, pid_count, context, "ready PID")?;
            }
            SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED => {
                add_remote_count(&mut self.degraded_skipped_count, 1, context, "degraded skipped")?;
            }
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR => {
                add_remote_count(&mut self.blocked_count, 1, context, "blocked")?;
                add_remote_count(
                    &mut self.missing_descriptor_count,
                    1,
                    context,
                    "missing descriptor",
                )?;
                add_remote_count(&mut self.blocked_pid_count, pid_count, context, "blocked PID")?;
                add_remote_count(
                    &mut self.missing_descriptor_pid_count,
                    pid_count,
                    context,
                    "missing descriptor PID",
                )?;
            }
            SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ => {
                add_remote_count(&mut self.blocked_count, 1, context, "blocked")?;
                add_remote_count(&mut self.transport_count, 1, context, "transport")?;
                add_remote_count(&mut self.blocked_pid_count, pid_count, context, "blocked PID")?;
                add_remote_count(&mut self.transport_pid_count, pid_count, context, "transport PID")?;
            }
            SPIRE_REMOTE_STATUS_STALE_EPOCH
            | SPIRE_REMOTE_STATUS_RETENTION_GAP
            | SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION
            | SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD => {
                add_remote_count(&mut self.blocked_count, 1, context, "blocked")?;
                add_remote_count(&mut self.blocked_pid_count, pid_count, context, "blocked PID")?;
                if self.first_remote_blocked_status.is_none() {
                    self.first_remote_blocked_status = Some(match status {
                        SPIRE_REMOTE_STATUS_STALE_EPOCH => SPIRE_REMOTE_STATUS_STALE_EPOCH,
                        SPIRE_REMOTE_STATUS_RETENTION_GAP => SPIRE_REMOTE_STATUS_RETENTION_GAP,
                        SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION => {
                            SPIRE_REMOTE_STATUS_INCOMPATIBLE_EXTENSION_VERSION
                        }
                        SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD => {
                            SPIRE_REMOTE_STATUS_EXECUTOR_OVERLOAD
                        }
                        _ => unreachable!("remote blocker status match already narrowed"),
                    });
                }
            }
            status => {
                return Err(format!("ec_spire {context} found unknown status '{status}'"));
            }
        }
        Ok(())
    }

    fn executable_pid_count(&self, context: &str) -> Result<u64, String> {
        self.local_pid_count
            .checked_add(self.remote_pid_count)
            .ok_or_else(|| format!("ec_spire {context} executable PID count overflowed"))
    }

    fn summary_status(&self, top_k: u64, mode: SpireRemoteSummaryStatusMode) -> &'static str {
        if top_k == 0 {
            return SPIRE_REMOTE_STATUS_EMPTY_TOP_K;
        }

        match mode {
            SpireRemoteSummaryStatusMode::RequestPlan => {
                if self.remote_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
                } else if self.skipped_count > 0 {
                    SPIRE_REMOTE_STATUS_DEGRADED_READY
                } else {
                    SPIRE_REMOTE_STATUS_READY
                }
            }
            SpireRemoteSummaryStatusMode::Readiness => {
                if self.missing_descriptor_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                } else if let Some(status) = self.first_remote_blocked_status {
                    status
                } else if self.transport_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
                } else if self.skipped_count > 0 {
                    SPIRE_REMOTE_STATUS_DEGRADED_READY
                } else {
                    SPIRE_REMOTE_STATUS_READY
                }
            }
            SpireRemoteSummaryStatusMode::Execution => {
                if self.missing_descriptor_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                } else if let Some(status) = self.first_remote_blocked_status {
                    status
                } else if self.transport_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
                } else if self.degraded_skipped_count > 0 {
                    SPIRE_REMOTE_STATUS_DEGRADED_READY
                } else {
                    SPIRE_REMOTE_STATUS_READY
                }
            }
            SpireRemoteSummaryStatusMode::LibpqRequest => {
                if self.missing_descriptor_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                } else if let Some(status) = self.first_remote_blocked_status {
                    status
                } else if self.transport_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
                } else {
                    SPIRE_REMOTE_STATUS_READY
                }
            }
        }
    }
}

