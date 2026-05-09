fn remote_candidate_assignment_role_rank(candidate: &SpireRemoteSearchCandidateRow) -> u8 {
    u8::from(candidate.assignment_flags & storage::SPIRE_ASSIGNMENT_FLAG_BOUNDARY_REPLICA != 0)
}

fn remote_search_candidate_cmp(
    left: &SpireRemoteSearchCandidateRow,
    right: &SpireRemoteSearchCandidateRow,
) -> std::cmp::Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| {
            remote_candidate_assignment_role_rank(left)
                .cmp(&remote_candidate_assignment_role_rank(right))
        })
        .then_with(|| right.served_epoch.cmp(&left.served_epoch))
        .then_with(|| left.node_id.cmp(&right.node_id))
        .then_with(|| left.pid.cmp(&right.pid))
        .then_with(|| right.object_version.cmp(&left.object_version))
        .then_with(|| left.row_index.cmp(&right.row_index))
        .then_with(|| left.row_locator.cmp(&right.row_locator))
}

const SPIRE_REMOTE_TARGET_LOCAL: &str = "local";
const SPIRE_REMOTE_TARGET_REMOTE: &str = "remote";
const SPIRE_REMOTE_TARGET_SKIPPED: &str = "skipped";
const SPIRE_REMOTE_STATUS_READY: &str = "ready";
const SPIRE_REMOTE_STATUS_EMPTY_TOP_K: &str = "empty_top_k";
const SPIRE_REMOTE_STATUS_DEGRADED_READY: &str = "degraded_ready";
const SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED: &str = "degraded_skipped";
const SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR: &str = "requires_remote_node_descriptor";
const SPIRE_REMOTE_STATUS_REQUIRES_SECRET: &str = "requires_conninfo_secret_resolution";
const SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ: &str = "requires_libpq_transport";
const SPIRE_REMOTE_STATUS_MISSING_DESCRIPTOR: &str = "missing_descriptor";
const SPIRE_REMOTE_STATUS_OPTIONAL_DESCRIPTOR_MISSING: &str = "optional_descriptor_missing";
const SPIRE_REMOTE_TRANSPORT_LOCAL_DIRECT: &str = "local_direct";
const SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE: &str = "libpq_pipeline";
const SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION: &str = "open_pipeline_and_send_remote_search";
const SPIRE_REMOTE_DISPATCH_BLOCKED_ACTION: &str = "blocked_before_dispatch";
const SPIRE_REMOTE_NONE: &str = "none";
const SPIRE_REMOTE_EXECUTOR_REQUIRED: &str = "requires_libpq_executor";
const SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR: &str = "remote_node_descriptor";
const SPIRE_REMOTE_EXECUTOR_STEP_SECRET: &str = "conninfo_secret_resolution";
const SPIRE_REMOTE_ENDPOINT_SEARCH: &str = "ec_spire_remote_search";
const SPIRE_REMOTE_INDEX_SOURCE_LOCAL_OID: &str = "local_index_oid";
const SPIRE_REMOTE_DESCRIPTOR_SOURCE: &str = "remote_node_descriptor";
const SPIRE_REMOTE_CONNINFO_READY: &str = "secret_reference_ready";
const SPIRE_REMOTE_CONNINFO_RESOLVED: &str = "resolved_conninfo";
const SPIRE_REMOTE_CANDIDATE_FORMAT_LOCAL: &str = "local";
const SPIRE_REMOTE_CANDIDATE_FORMAT_V1: &str = "ec_spire_remote_search_v1";
const SPIRE_REMOTE_ROW_LOCATOR_POLICY: &str = "opaque_origin_node_bytes";
const SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION: &str = "coordinator_local_heap";
const SPIRE_REMOTE_HEAP_RESOLUTION: &str = "origin_node_row_locator";
const SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY: &str = "local_ready";
const SPIRE_REMOTE_FINAL_STATUS_REMOTE_READY: &str = "remote_ready";
const SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES: &str = "no_candidate_batches";
const SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP: &str = "requires_remote_heap_resolution";
const SPIRE_REMOTE_FINAL_STATUS_BLOCKED: &str = "blocked";
const SPIRE_REMOTE_FINAL_STATUS_PLANNED: &str = "planned";
const SPIRE_REMOTE_RESULT_SOURCE_LOCAL_HEAP_CANDIDATES: &str = "local_heap_candidates";
const SPIRE_REMOTE_RESULT_SOURCE_REMOTE_HEAP_CANDIDATES: &str = "remote_heap_candidates";
const SPIRE_REMOTE_RESULT_SOURCE_BLOCKED: &str = "blocked";
const SPIRE_REMOTE_DESCRIPTOR_STATE_ACTIVE: &str = "active";
const SPIRE_REMOTE_DESCRIPTOR_STATE_DRAINING: &str = "draining";
const SPIRE_REMOTE_DESCRIPTOR_STATE_DISABLED: &str = "disabled";
const SPIRE_REMOTE_DESCRIPTOR_STATE_FAILED: &str = "failed";
const SPIRE_REMOTE_DESCRIPTOR_STATE_MISSING: &str = "missing";
const SPIRE_REMOTE_CONNINFO_ENV_PREFIX: &str = "EC_SPIRE_REMOTE_CONNINFO_";

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
            resource_limit_policy: "one_connection_per_ready_remote_node_per_query",
            validator: "must_close_connection_before_coordinator_returns",
            recommendation: "implement executor-owned secret provider before opening libpq sockets",
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
                } else if self.transport_count > 0 {
                    SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
                } else {
                    SPIRE_REMOTE_STATUS_READY
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteSearchMergeResult {
    pub(crate) candidates: Vec<SpireRemoteSearchCandidateRow>,
    pub(crate) input_count: u64,
    pub(crate) duplicate_vec_id_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpireRemoteSearchCandidateBatch {
    pub(crate) node_id: u32,
    pub(crate) selected_pids: Vec<u64>,
    pub(crate) candidates: Vec<SpireRemoteSearchCandidateRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteSearchFanoutTarget {
    node_id: u32,
    selected_pids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteSearchSkippedPlacement {
    node_id: u32,
    pid: u64,
    state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteSearchFanoutPlan {
    requested_epoch: u64,
    local_selected_pids: Vec<u64>,
    remote_targets: Vec<SpireRemoteSearchFanoutTarget>,
    skipped_placements: Vec<SpireRemoteSearchSkippedPlacement>,
}

fn plan_remote_search_fanout(
    snapshot: &meta::SpirePublishedEpochSnapshot<'_>,
    selected_leaf_pids: &[u64],
) -> Result<SpireRemoteSearchFanoutPlan, String> {
    let snapshot = meta::SpireValidatedEpochSnapshot::from_snapshot(*snapshot)?;
    if selected_leaf_pids.is_empty() {
        return Ok(SpireRemoteSearchFanoutPlan {
            requested_epoch: snapshot.epoch_manifest().epoch,
            local_selected_pids: Vec::new(),
            remote_targets: Vec::new(),
            skipped_placements: Vec::new(),
        });
    }

    let mut seen = HashSet::new();
    let mut local_selected_pids = Vec::new();
    let mut remote_by_node = BTreeMap::<u32, Vec<u64>>::new();
    let mut skipped_placements = Vec::new();

    for &pid in selected_leaf_pids {
        if pid == 0 {
            return Err("ec_spire remote search fanout selected PID 0 is invalid".to_owned());
        }
        if !seen.insert(pid) {
            return Err(format!(
                "ec_spire remote search fanout selected PID {pid} appears more than once"
            ));
        }

        let lookup = snapshot.require_lookup(pid, "remote search fanout selected leaf")?;
        if fanout_should_skip_placement(
            snapshot.epoch_manifest().consistency_mode,
            lookup.placement.state,
        )? {
            skipped_placements.push(SpireRemoteSearchSkippedPlacement {
                node_id: lookup.placement.node_id,
                pid,
                state: fanout_placement_state_name(lookup.placement.state),
            });
            continue;
        }

        if lookup.placement.node_id == meta::SPIRE_LOCAL_NODE_ID {
            local_selected_pids.push(pid);
        } else {
            remote_by_node
                .entry(lookup.placement.node_id)
                .or_default()
                .push(pid);
        }
    }

    let remote_targets = remote_by_node
        .into_iter()
        .map(|(node_id, selected_pids)| SpireRemoteSearchFanoutTarget {
            node_id,
            selected_pids,
        })
        .collect();

    Ok(SpireRemoteSearchFanoutPlan {
        requested_epoch: snapshot.epoch_manifest().epoch,
        local_selected_pids,
        remote_targets,
        skipped_placements,
    })
}

fn fanout_should_skip_placement(
    consistency_mode: meta::SpireConsistencyMode,
    state: meta::SpirePlacementState,
) -> Result<bool, String> {
    match (consistency_mode, state) {
        (_, meta::SpirePlacementState::Available) => Ok(false),
        (meta::SpireConsistencyMode::Degraded, meta::SpirePlacementState::Unavailable)
        | (meta::SpireConsistencyMode::Degraded, meta::SpirePlacementState::Skipped) => Ok(true),
        (meta::SpireConsistencyMode::Strict, state) => Err(format!(
            "ec_spire strict remote search fanout cannot skip {:?} placement",
            state
        )),
        (meta::SpireConsistencyMode::Degraded, meta::SpirePlacementState::Stale) => {
            Err("ec_spire degraded remote search fanout cannot use stale placement".to_owned())
        }
    }
}

fn fanout_placement_state_name(state: meta::SpirePlacementState) -> &'static str {
    match state {
        meta::SpirePlacementState::Available => "available",
        meta::SpirePlacementState::Stale => "stale",
        meta::SpirePlacementState::Unavailable => "unavailable",
        meta::SpirePlacementState::Skipped => "skipped",
    }
}

pub(crate) unsafe fn remote_search_fanout_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    selected_pids: Vec<u64>,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchFanoutPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchFanoutPlanRow>, String> {
        if requested_epoch == 0 {
            return Err(
                "ec_spire remote search fanout requested_epoch must be greater than 0".to_owned(),
            );
        }
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote search fanout requested epoch {requested_epoch} does not match active epoch {}",
                root_control.active_epoch
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) =
            unsafe { scan::load_relation_epoch_manifests(index_relation, root_control)? };
        if epoch_manifest.consistency_mode != requested_consistency_mode {
            return Err(format!(
                "ec_spire remote search fanout requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
                consistency_mode_name(epoch_manifest.consistency_mode)
            ));
        }
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let plan = plan_remote_search_fanout(&snapshot, &selected_pids)?;
        let validated_snapshot = meta::SpireValidatedEpochSnapshot::from_snapshot(snapshot)?;
        let mut rows = Vec::with_capacity(
            plan.local_selected_pids.len()
                + plan
                    .remote_targets
                    .iter()
                    .map(|target| target.selected_pids.len())
                    .sum::<usize>()
                + plan.skipped_placements.len(),
        );
        for pid in plan.local_selected_pids {
            let placement_state = fanout_placement_state_name(
                validated_snapshot
                    .require_lookup(pid, "remote search fanout local row")?
                    .placement
                    .state,
            );
            rows.push(SpireRemoteSearchFanoutPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_LOCAL,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                pid,
                placement_state,
            });
        }
        for target in plan.remote_targets {
            for pid in target.selected_pids {
                let placement_state = fanout_placement_state_name(
                    validated_snapshot
                        .require_lookup(pid, "remote search fanout remote row")?
                        .placement
                        .state,
                );
                rows.push(SpireRemoteSearchFanoutPlanRow {
                    requested_epoch: plan.requested_epoch,
                    target_kind: SPIRE_REMOTE_TARGET_REMOTE,
                    node_id: target.node_id,
                    pid,
                    placement_state,
                });
            }
        }
        rows.extend(plan.skipped_placements.into_iter().map(|skipped| {
            SpireRemoteSearchFanoutPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_SKIPPED,
                node_id: skipped.node_id,
                pid: skipped.pid,
                placement_state: skipped.state,
            }
        }));
        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_target_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    selected_pids: Vec<u64>,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchTargetPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchTargetPlanRow>, String> {
        if requested_epoch == 0 {
            return Err(
                "ec_spire remote search target plan requested_epoch must be greater than 0"
                    .to_owned(),
            );
        }
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        if root_control.active_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote search target plan requested epoch {requested_epoch} does not match active epoch {}",
                root_control.active_epoch
            ));
        }

        let (epoch_manifest, object_manifest, placement_directory) = unsafe {
            load_relation_epoch_manifests_for_coordinator_fanout(index_relation, root_control)?
        };
        if epoch_manifest.consistency_mode != requested_consistency_mode {
            return Err(format!(
                "ec_spire remote search target plan requested consistency_mode '{consistency_mode}' does not match active epoch consistency mode '{}'",
                consistency_mode_name(epoch_manifest.consistency_mode)
            ));
        }
        let snapshot = meta::SpirePublishedEpochSnapshot::new(
            &epoch_manifest,
            &object_manifest,
            &placement_directory,
        )?;
        let plan = plan_remote_search_fanout(&snapshot, &selected_pids)?;
        let mut rows = Vec::new();
        if !plan.local_selected_pids.is_empty() {
            let pid_count = u64::try_from(plan.local_selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan local PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_LOCAL,
                node_id: meta::SPIRE_LOCAL_NODE_ID,
                selected_pids: plan.local_selected_pids,
                pid_count,
                placement_state: "available",
                status: SPIRE_REMOTE_STATUS_READY,
            });
        }
        for target in plan.remote_targets {
            let pid_count = u64::try_from(target.selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan remote PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_REMOTE,
                node_id: target.node_id,
                selected_pids: target.selected_pids,
                pid_count,
                placement_state: "available",
                status: SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
            });
        }

        let mut skipped_by_node_state = BTreeMap::<(u32, &'static str), Vec<u64>>::new();
        for skipped in plan.skipped_placements {
            skipped_by_node_state
                .entry((skipped.node_id, skipped.state))
                .or_default()
                .push(skipped.pid);
        }
        for ((node_id, placement_state), selected_pids) in skipped_by_node_state {
            let pid_count = u64::try_from(selected_pids.len())
                .map_err(|_| "ec_spire remote search target plan skipped PID count exceeds u64")?;
            rows.push(SpireRemoteSearchTargetPlanRow {
                requested_epoch: plan.requested_epoch,
                target_kind: SPIRE_REMOTE_TARGET_SKIPPED,
                node_id,
                selected_pids,
                pid_count,
                placement_state,
                status: SPIRE_REMOTE_STATUS_DEGRADED_SKIPPED,
            });
        }

        Ok(rows)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_target_readiness_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    selected_pids: Vec<u64>,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchTargetReadinessRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchTargetReadinessRow>, String> {
        let target_rows = unsafe {
            remote_search_target_plan_rows(
                index_relation,
                requested_epoch,
                selected_pids,
                consistency_mode,
            )
        };
        let node_rows = unsafe { remote_node_snapshot(index_relation) }
            .into_iter()
            .map(|row| (row.node_id, row))
            .collect::<BTreeMap<_, _>>();

        target_rows
            .into_iter()
            .map(|target| {
                let node = node_rows.get(&target.node_id).ok_or_else(|| {
                    format!(
                        "ec_spire remote search target readiness missing node snapshot for node_id {}",
                        target.node_id
                    )
                })?;
                let status = if target.target_kind == SPIRE_REMOTE_TARGET_SKIPPED {
                    target.status
                } else if matches!(
                    node.descriptor_state,
                    SPIRE_REMOTE_DESCRIPTOR_STATE_DISABLED | SPIRE_REMOTE_DESCRIPTOR_STATE_FAILED
                ) {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                } else if node.status != SPIRE_REMOTE_STATUS_READY {
                    node.status
                } else {
                    target.status
                };
                Ok(SpireRemoteSearchTargetReadinessRow {
                    requested_epoch: target.requested_epoch,
                    target_kind: target.target_kind,
                    node_id: target.node_id,
                    selected_pids: target.selected_pids,
                    pid_count: target.pid_count,
                    placement_state: target.placement_state,
                    node_kind: node.node_kind,
                    descriptor_state: node.descriptor_state,
                    node_status: node.status,
                    status,
                })
            })
            .collect()
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_request_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchRequestPlanRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchRequestPlanRow>, String> {
        let query = scan::SpireScanQuery::new(query)?;
        let query_dimension = u64::try_from(query.values().len())
            .map_err(|_| "ec_spire remote search request plan query dimension exceeds u64")?;
        let top_k = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search request plan top_k exceeds u64")?;
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let rows = unsafe {
            remote_search_target_plan_rows(
                index_relation,
                requested_epoch,
                selected_pids,
                consistency_mode,
            )
        };
        Ok(rows
            .into_iter()
            .map(|row| SpireRemoteSearchRequestPlanRow {
                requested_epoch: row.requested_epoch,
                target_kind: row.target_kind,
                node_id: row.node_id,
                selected_pids: row.selected_pids,
                pid_count: row.pid_count,
                query_dimension,
                top_k,
                consistency_mode: consistency_mode_name(requested_consistency_mode),
                endpoint_function: if row.target_kind == SPIRE_REMOTE_TARGET_SKIPPED {
                    SPIRE_REMOTE_NONE
                } else {
                    SPIRE_REMOTE_ENDPOINT_SEARCH
                },
                status: row.status,
            })
            .collect())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_request_readiness_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchRequestReadinessRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchRequestReadinessRow>, String> {
        let query = scan::SpireScanQuery::new(query)?;
        let query_dimension = u64::try_from(query.values().len())
            .map_err(|_| "ec_spire remote search request readiness query dimension exceeds u64")?;
        let top_k = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search request readiness top_k exceeds u64")?;
        let requested_consistency_mode = parse_remote_search_consistency_mode(consistency_mode)?;
        let rows = unsafe {
            remote_search_target_readiness_rows(
                index_relation,
                requested_epoch,
                selected_pids,
                consistency_mode,
            )
        };
        Ok(rows
            .into_iter()
            .map(|row| SpireRemoteSearchRequestReadinessRow {
                requested_epoch: row.requested_epoch,
                target_kind: row.target_kind,
                node_id: row.node_id,
                selected_pids: row.selected_pids,
                pid_count: row.pid_count,
                query_dimension,
                top_k,
                consistency_mode: consistency_mode_name(requested_consistency_mode),
                endpoint_function: if row.target_kind == SPIRE_REMOTE_TARGET_SKIPPED {
                    SPIRE_REMOTE_NONE
                } else {
                    SPIRE_REMOTE_ENDPOINT_SEARCH
                },
                node_kind: row.node_kind,
                descriptor_state: row.descriptor_state,
                node_status: row.node_status,
                status: row.status,
            })
            .collect())
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_request_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchRequestSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchRequestSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search request summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_request_plan_rows(
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
            rollup.record_target(row.target_kind, row.pid_count, "remote search request summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search request summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let request_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search request summary request count exceeds u64")?;
        let executable_pid_count = rollup.executable_pid_count("remote search request summary")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::RequestPlan);

        Ok(SpireRemoteSearchRequestSummaryRow {
            requested_epoch,
            request_count,
            local_request_count: rollup.local_count,
            remote_request_count: rollup.remote_count,
            skipped_request_count: rollup.skipped_count,
            executable_pid_count,
            local_pid_count: rollup.local_pid_count,
            remote_pid_count: rollup.remote_pid_count,
            skipped_pid_count: rollup.skipped_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_readiness_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchReadinessSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchReadinessSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search readiness summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_request_readiness_rows(
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
            rollup.record_target(row.target_kind, row.pid_count, "remote search readiness summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search readiness summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search readiness summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let request_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search readiness summary request count exceeds u64")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::Readiness);

        Ok(SpireRemoteSearchReadinessSummaryRow {
            requested_epoch,
            request_count,
            ready_request_count: rollup.ready_count,
            blocked_request_count: rollup.blocked_count,
            local_request_count: rollup.local_count,
            remote_request_count: rollup.remote_count,
            skipped_request_count: rollup.skipped_count,
            executable_pid_count: rollup.executable_pid_count("remote search readiness summary")?,
            ready_pid_count: rollup.ready_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            skipped_pid_count: rollup.skipped_pid_count,
            missing_descriptor_request_count: rollup.missing_descriptor_count,
            missing_descriptor_pid_count: rollup.missing_descriptor_pid_count,
            transport_request_count: rollup.transport_count,
            transport_pid_count: rollup.transport_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_execution_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchExecutionPlanRow> {
    let rows = unsafe {
        remote_search_request_readiness_rows(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    rows.into_iter()
        .map(|row| {
            remote_search_execution_plan_row_from_readiness(row)
        })
        .collect()
}

fn remote_search_execution_plan_row_from_readiness(
    row: SpireRemoteSearchRequestReadinessRow,
) -> SpireRemoteSearchExecutionPlanRow {
    let (execution_transport, remote_index_source, conninfo_source, candidate_format) =
        match row.target_kind {
            SPIRE_REMOTE_TARGET_LOCAL => (
                SPIRE_REMOTE_TRANSPORT_LOCAL_DIRECT,
                SPIRE_REMOTE_INDEX_SOURCE_LOCAL_OID,
                SPIRE_REMOTE_CANDIDATE_FORMAT_LOCAL,
                SPIRE_REMOTE_CANDIDATE_FORMAT_LOCAL,
            ),
            SPIRE_REMOTE_TARGET_REMOTE => (
                SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
                SPIRE_REMOTE_DESCRIPTOR_SOURCE,
                SPIRE_REMOTE_DESCRIPTOR_SOURCE,
                SPIRE_REMOTE_CANDIDATE_FORMAT_V1,
            ),
            SPIRE_REMOTE_TARGET_SKIPPED => (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_NONE,
            ),
            _ => ("unknown", "unknown", "unknown", "unknown"),
        };
    SpireRemoteSearchExecutionPlanRow {
        requested_epoch: row.requested_epoch,
        target_kind: row.target_kind,
        node_id: row.node_id,
        selected_pids: row.selected_pids,
        pid_count: row.pid_count,
        query_dimension: row.query_dimension,
        top_k: row.top_k,
        consistency_mode: row.consistency_mode,
        execution_transport,
        endpoint_function: row.endpoint_function,
        remote_index_source,
        conninfo_source,
        candidate_format,
        status: row.status,
    }
}

pub(crate) unsafe fn remote_search_execution_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchExecutionSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchExecutionSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search execution summary top_k exceeds u64")?;
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
        remote_search_execution_summary_from_plan_rows(
            requested_epoch,
            &rows,
            query_for_empty_plan,
            top_k_for_empty_plan,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_execution_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchExecutionPlanRow],
    query_for_empty_plan: Vec<f32>,
    top_k_for_empty_plan: u64,
    consistency_mode: &str,
) -> Result<SpireRemoteSearchExecutionSummaryRow, String> {
        let mut rollup = SpireRemoteCountRollup::default();
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_target(row.target_kind, row.pid_count, "remote search execution summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search execution summary")?;
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search execution summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let plan_count = u64::try_from(rows.len())
            .map_err(|_| "ec_spire remote search execution summary plan count exceeds u64")?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::Execution);

        Ok(SpireRemoteSearchExecutionSummaryRow {
            requested_epoch,
            plan_count,
            local_plan_count: rollup.local_count,
            remote_plan_count: rollup.remote_count,
            skipped_plan_count: rollup.skipped_count,
            ready_plan_count: rollup.ready_count,
            blocked_plan_count: rollup.blocked_count,
            degraded_skipped_plan_count: rollup.degraded_skipped_count,
            local_pid_count: rollup.local_pid_count,
            remote_pid_count: rollup.remote_pid_count,
            skipped_pid_count: rollup.skipped_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
}

const SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE: &str =
    "SELECT * FROM ec_spire_remote_search($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text)";
const SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE: &str =
    "SELECT * FROM ec_spire_remote_search_local_heap_candidates($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text)";
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
    conninfo_secret_name: String,
    remote_index_regclass: String,
    remote_index_identity_bytes: u64,
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
                conninfo_secret_name, \
                remote_index_identity, \
                remote_index_regclass \
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

                Ok((
                    node_id,
                    SpireRemoteLibpqConnectionDescriptorRow {
                        conninfo_secret_name,
                        remote_index_regclass,
                        remote_index_identity_bytes,
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
                remote_index_identity_bytes: descriptor
                    .map(|row| row.remote_index_identity_bytes)
                    .unwrap_or(0),
                conninfo_resolution: if descriptor.is_some() {
                    SPIRE_REMOTE_CONNINFO_READY
                } else {
                    SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
                },
                pipeline_mode: if descriptor.is_some() {
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
    connection_rows
        .iter()
        .map(|row| {
            let dispatch_action = if row.pipeline_mode == SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE {
                SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION
            } else {
                SPIRE_REMOTE_DISPATCH_BLOCKED_ACTION
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
                pipeline_mode: row.pipeline_mode,
                dispatch_action,
                receive_validator: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
                status: row.status,
            }
        })
        .collect()
}

pub(crate) unsafe fn remote_search_libpq_dispatch_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchLibpqDispatchSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchLibpqDispatchSummaryRow, String> {
        let query_for_empty_plan = query.clone();
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire remote search libpq dispatch summary top_k exceeds u64")?;
        let rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        remote_search_libpq_dispatch_summary_from_plan_rows(
            requested_epoch,
            &rows,
            query_for_empty_plan,
            top_k_for_empty_plan,
            consistency_mode,
        )
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_libpq_dispatch_summary_from_plan_rows(
    requested_epoch: u64,
    rows: &[SpireRemoteSearchLibpqDispatchPlanRow],
    query_for_empty_plan: Vec<f32>,
    top_k_for_empty_plan: u64,
    consistency_mode: &str,
) -> Result<SpireRemoteSearchLibpqDispatchSummaryRow, String> {
        let mut rollup = SpireRemoteCountRollup::default();
        let mut pipeline_dispatch_count = 0_u64;
        let mut missing_descriptor_dispatch_count = 0_u64;
        let mut query_dimension = 0_u64;
        let mut top_k = 0_u64;
        let mut parsed_consistency_mode = "";

        for row in rows {
            query_dimension = row.query_dimension;
            top_k = row.top_k;
            parsed_consistency_mode = row.consistency_mode;
            rollup.record_remote_target(row.pid_count, "remote search libpq dispatch summary")?;
            rollup.record_status(row.status, row.pid_count, "remote search libpq dispatch summary")?;
            if row.dispatch_action == SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
                pipeline_dispatch_count = pipeline_dispatch_count.checked_add(1).ok_or_else(|| {
                    "ec_spire remote search libpq dispatch summary pipeline count overflow".to_owned()
                })?;
            }
            if row.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
                missing_descriptor_dispatch_count =
                    missing_descriptor_dispatch_count.checked_add(1).ok_or_else(|| {
                        "ec_spire remote search libpq dispatch summary missing descriptor count overflow"
                            .to_owned()
                    })?;
            }
        }

        if rows.is_empty() {
            let query = scan::SpireScanQuery::new(query_for_empty_plan)?;
            query_dimension = u64::try_from(query.values().len()).map_err(|_| {
                "ec_spire remote search libpq dispatch summary query dimension exceeds u64"
            })?;
            top_k = top_k_for_empty_plan;
            parsed_consistency_mode =
                consistency_mode_name(parse_remote_search_consistency_mode(consistency_mode)?);
        }

        let dispatch_count = u64::try_from(rows.len()).map_err(|_| {
            "ec_spire remote search libpq dispatch summary dispatch count exceeds u64"
        })?;
        let status = rollup.summary_status(top_k, SpireRemoteSummaryStatusMode::LibpqRequest);

        Ok(SpireRemoteSearchLibpqDispatchSummaryRow {
            requested_epoch,
            dispatch_count,
            pipeline_dispatch_count,
            missing_descriptor_dispatch_count,
            remote_pid_count: rollup.remote_pid_count,
            blocked_pid_count: rollup.blocked_pid_count,
            query_dimension,
            top_k,
            consistency_mode: parsed_consistency_mode,
            status,
        })
}

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
                    next_executor_step: SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR,
                    status: row.status,
                    recommendation: "resolve descriptor gate before conninfo secret lookup",
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
    let mut descriptor_blocked_count = 0_u64;
    let mut remote_pid_count = 0_u64;
    let mut blocked_pid_count = 0_u64;
    let mut first_descriptor_status = SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR;

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
                if descriptor_blocked_count == 0 {
                    first_descriptor_status = row.status;
                }
                add_remote_count(
                    &mut descriptor_blocked_count,
                    1,
                    "remote search libpq secret summary",
                    "descriptor-blocked row",
                )?;
            }
        }
    }

    let secret_count = u64::try_from(rows.len())
        .map_err(|_| "remote search libpq secret count exceeds u64")?;
    let (next_executor_step, status) = if secret_count == 0 {
        (SPIRE_REMOTE_NONE, SPIRE_REMOTE_STATUS_READY)
    } else if descriptor_blocked_count > 0 {
        (SPIRE_REMOTE_EXECUTOR_STEP_DESCRIPTOR, first_descriptor_status)
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
    ) = if dispatch_summary.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
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
        (
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            SPIRE_REMOTE_NONE,
            dispatch_summary.status,
            SPIRE_REMOTE_NONE,
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
            step_name: SPIRE_REMOTE_EXECUTOR_STEP_SECRET,
            executor_action: "resolve_conninfo_secret_reference",
            input_contract: "conninfo_secret_name",
            output_contract: SPIRE_REMOTE_CONNINFO_READY,
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_resolve_secret_without_exposing_raw_conninfo",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 3,
            step_name: "open_libpq_connection",
            executor_action: "open_libpq_connection",
            input_contract: SPIRE_REMOTE_CONNINFO_READY,
            output_contract: "libpq_connection",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_target_registered_remote_index",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 4,
            step_name: "enter_libpq_pipeline_mode",
            executor_action: "enter_libpq_pipeline_mode",
            input_contract: "libpq_connection",
            output_contract: SPIRE_REMOTE_TRANSPORT_LIBPQ_PIPELINE,
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_enter_pipeline_before_sending_remote_search",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 5,
            step_name: "send_remote_search_request",
            executor_action: "send_remote_search_request",
            input_contract: "ec_spire_remote_search_libpq_request_plan",
            output_contract: "pending_remote_search_result",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_bind_libpq_parameter_contract",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 6,
            step_name: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            executor_action: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            input_contract: "remote_search_result_batch",
            output_contract: "validated_remote_candidate_batch",
            blocking_status: SPIRE_REMOTE_EXECUTOR_REQUIRED,
            validator: "must_match_libpq_result_contract",
        },
        SpireRemoteSearchLibpqExecutorStepContractRow {
            step_ordinal: 7,
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
            validator: "must_be_nonempty_positive_unique_remote_leaf_pids",
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
            validator: "must_be_selected_pid",
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
    ]
}

fn remote_conninfo_secret_value(conninfo_secret_name: &str) -> Result<String, String> {
    let provider_lookup_key = remote_conninfo_secret_provider_lookup_key(conninfo_secret_name)?;
    match std::env::var(provider_lookup_key) {
        Ok(conninfo) if !conninfo.is_empty() => Ok(conninfo),
        Ok(_) => Err("conninfo_secret_empty".to_owned()),
        Err(_) => Err("conninfo_secret_missing".to_owned()),
    }
}

fn decode_remote_search_candidate_pg_row(
    row: &postgres::Row,
    expected_node_id: u32,
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
    let candidate = decode_remote_search_candidate_pg_row(row, expected_node_id)?;
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
        status: SPIRE_REMOTE_STATUS_READY,
    })
}

fn remote_search_libpq_executor_candidates_for_dispatch(
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
) -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
    if row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
        return Err(format!(
            "ec_spire remote search libpq executor dispatch for node_id {} is blocked with status {}",
            row.node_id, row.status
        ));
    }

    let conninfo = remote_conninfo_secret_value(&row.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire remote search libpq executor conninfo secret for node_id {} is not resolved: {status}",
            row.node_id
        )
    })?;
    let mut client = postgres::Client::connect(&conninfo, postgres::NoTls).map_err(|_| {
        format!(
            "ec_spire remote search libpq executor failed to open connection for node_id {}",
            row.node_id
        )
    })?;
    let remote_index_oid = client
        .query_one(
            "SELECT to_regclass($1)::oid",
            &[&row.remote_index_regclass.as_str()],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor failed to resolve remote index for node_id {}",
                row.node_id
            )
        })?
        .try_get::<_, Option<u32>>(0)
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor remote index oid decode failed for node_id {}",
                row.node_id
            )
        })?
        .ok_or_else(|| {
            format!(
                "ec_spire remote search libpq executor remote index is missing for node_id {}",
                row.node_id
            )
        })?;
    let requested_epoch = i64::try_from(row.requested_epoch)
        .map_err(|_| "ec_spire remote search libpq executor requested_epoch exceeds i64")?;
    let selected_pids = row
        .selected_pids
        .iter()
        .map(|pid| {
            i64::try_from(*pid)
                .map_err(|_| "ec_spire remote search libpq executor selected PID exceeds i64")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let top_k = i32::try_from(top_k)
        .map_err(|_| "ec_spire remote search libpq executor top_k exceeds i32")?;

    let result_rows = client
        .query(
            SPIRE_REMOTE_SEARCH_LIBPQ_SQL_TEMPLATE,
            &[
                &remote_index_oid,
                &requested_epoch,
                &query,
                &selected_pids,
                &top_k,
                &consistency_mode,
            ],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote search libpq executor remote search query failed for node_id {}",
                row.node_id
            )
        })?;
    let candidates = result_rows
        .iter()
        .map(|candidate_row| decode_remote_search_candidate_pg_row(candidate_row, row.node_id))
        .collect::<Result<Vec<_>, _>>()?;
    validate_remote_search_candidate_batch(
        row.requested_epoch,
        row.node_id,
        &row.selected_pids,
        &candidates,
    )?;

    Ok(candidates)
}

fn remote_search_libpq_executor_heap_candidates_for_dispatch(
    row: &SpireRemoteSearchLibpqDispatchPlanRow,
    query: &[f32],
    top_k: usize,
    consistency_mode: &str,
) -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
    if row.dispatch_action != SPIRE_REMOTE_DISPATCH_PIPELINE_ACTION {
        return Err(format!(
            "ec_spire remote heap executor dispatch for node_id {} is blocked with status {}",
            row.node_id, row.status
        ));
    }

    let conninfo = remote_conninfo_secret_value(&row.conninfo_secret_name).map_err(|status| {
        format!(
            "ec_spire remote heap executor conninfo secret for node_id {} is not resolved: {status}",
            row.node_id
        )
    })?;
    let mut client = postgres::Client::connect(&conninfo, postgres::NoTls).map_err(|_| {
        format!(
            "ec_spire remote heap executor failed to open connection for node_id {}",
            row.node_id
        )
    })?;
    let remote_index_oid = client
        .query_one(
            "SELECT to_regclass($1)::oid",
            &[&row.remote_index_regclass.as_str()],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote heap executor failed to resolve remote index for node_id {}",
                row.node_id
            )
        })?
        .try_get::<_, Option<u32>>(0)
        .map_err(|_| {
            format!(
                "ec_spire remote heap executor remote index oid decode failed for node_id {}",
                row.node_id
            )
        })?
        .ok_or_else(|| {
            format!(
                "ec_spire remote heap executor remote index is missing for node_id {}",
                row.node_id
            )
        })?;
    let requested_epoch = i64::try_from(row.requested_epoch)
        .map_err(|_| "ec_spire remote heap executor requested_epoch exceeds i64")?;
    let selected_pids = row
        .selected_pids
        .iter()
        .map(|pid| {
            i64::try_from(*pid)
                .map_err(|_| "ec_spire remote heap executor selected PID exceeds i64")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let top_k =
        i32::try_from(top_k).map_err(|_| "ec_spire remote heap executor top_k exceeds i32")?;

    let result_rows = client
        .query(
            SPIRE_REMOTE_SEARCH_LIBPQ_HEAP_SQL_TEMPLATE,
            &[
                &remote_index_oid,
                &requested_epoch,
                &query,
                &selected_pids,
                &top_k,
                &consistency_mode,
            ],
        )
        .map_err(|_| {
            format!(
                "ec_spire remote heap executor remote heap query failed for node_id {}",
                row.node_id
            )
        })?;
    let candidates = result_rows
        .iter()
        .map(|candidate_row| {
            decode_remote_search_heap_candidate_pg_row(
                candidate_row,
                row.requested_epoch,
                row.node_id,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let merge_candidates = candidates
        .iter()
        .map(|candidate| SpireRemoteSearchCandidateRow {
            served_epoch: candidate.served_epoch,
            node_id: candidate.node_id,
            pid: candidate.pid,
            object_version: candidate.object_version,
            row_index: candidate.row_index,
            assignment_flags: candidate.assignment_flags,
            vec_id: candidate.vec_id.clone(),
            row_locator: candidate.row_locator.clone(),
            score: candidate.score,
        })
        .collect::<Vec<_>>();
    validate_remote_search_candidate_batch(
        row.requested_epoch,
        row.node_id,
        &row.selected_pids,
        &merge_candidates,
    )?;

    Ok(candidates)
}

pub(crate) unsafe fn remote_search_libpq_executor_candidate_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchCandidateRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchCandidateRow>, String> {
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut candidates = Vec::new();
        for row in &dispatch_rows {
            candidates.extend(remote_search_libpq_executor_candidates_for_dispatch(
                row,
                &query,
                top_k,
                consistency_mode,
            )?);
        }
        Ok(candidates)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_libpq_executor_heap_candidate_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchLocalHeapCandidateRow> {
    let result = (|| -> Result<Vec<SpireRemoteSearchLocalHeapCandidateRow>, String> {
        let dispatch_rows = unsafe {
            remote_search_libpq_dispatch_plan_rows(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let mut candidates = Vec::new();
        for row in &dispatch_rows {
            candidates.extend(remote_search_libpq_executor_heap_candidates_for_dispatch(
                row,
                &query,
                top_k,
                consistency_mode,
            )?);
        }
        candidates.sort_by(|left, right| {
            remote_search_candidate_cmp(
                &SpireRemoteSearchCandidateRow {
                    served_epoch: left.served_epoch,
                    node_id: left.node_id,
                    pid: left.pid,
                    object_version: left.object_version,
                    row_index: left.row_index,
                    assignment_flags: left.assignment_flags,
                    vec_id: left.vec_id.clone(),
                    row_locator: left.row_locator.clone(),
                    score: left.score,
                },
                &SpireRemoteSearchCandidateRow {
                    served_epoch: right.served_epoch,
                    node_id: right.node_id,
                    pid: right.pid,
                    object_version: right.object_version,
                    row_index: right.row_index,
                    assignment_flags: right.assignment_flags,
                    vec_id: right.vec_id.clone(),
                    row_locator: right.row_locator.clone(),
                    score: right.score,
                },
            )
        });
        candidates.truncate(top_k);
        Ok(candidates)
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

pub(crate) unsafe fn remote_search_receive_plan_rows(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> Vec<SpireRemoteSearchReceivePlanRow> {
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
    remote_search_receive_plan_rows_from_requests(&rows)
}

fn remote_search_receive_plan_rows_from_requests(
    rows: &[SpireRemoteSearchLibpqRequestPlanRow],
) -> Vec<SpireRemoteSearchReceivePlanRow> {
    rows.iter()
        .map(|row| SpireRemoteSearchReceivePlanRow {
            requested_epoch: row.requested_epoch,
            node_id: row.node_id,
            selected_pids: row.selected_pids.clone(),
            pid_count: row.pid_count,
            expected_candidate_format: row.candidate_format,
            expected_result_column_count: row.result_column_count,
            validator_function: SPIRE_REMOTE_SEARCH_RECEIVE_VALIDATOR,
            row_locator_policy: SPIRE_REMOTE_ROW_LOCATOR_POLICY,
            status: row.status,
        })
        .collect()
}

pub(crate) unsafe fn remote_search_merge_input_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchMergeInputSummaryRow {
    let result = (|| -> Result<SpireRemoteSearchMergeInputSummaryRow, String> {
        let execution_summary = unsafe {
            remote_search_execution_summary_row(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        Ok(remote_search_merge_input_summary_from_execution(
            &execution_summary,
        ))
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn remote_search_merge_input_summary_from_execution(
    execution_summary: &SpireRemoteSearchExecutionSummaryRow,
) -> SpireRemoteSearchMergeInputSummaryRow {
    let remote_batch_count = execution_summary.remote_plan_count;
    let local_batch_count = execution_summary.local_plan_count;
    let skipped_batch_count = execution_summary.skipped_plan_count;
    let ready_batch_count = execution_summary.ready_plan_count;
    let blocked_batch_count = execution_summary.blocked_plan_count;
    let status = if execution_summary.top_k == 0 {
        SPIRE_REMOTE_STATUS_EMPTY_TOP_K
    } else if blocked_batch_count > 0 {
        execution_summary.status
    } else if execution_summary.degraded_skipped_plan_count > 0 {
        SPIRE_REMOTE_STATUS_DEGRADED_READY
    } else if remote_batch_count > 0 || local_batch_count > 0 {
        SPIRE_REMOTE_STATUS_READY
    } else if skipped_batch_count > 0 {
        SPIRE_REMOTE_STATUS_DEGRADED_READY
    } else {
        SPIRE_REMOTE_STATUS_READY
    };

    SpireRemoteSearchMergeInputSummaryRow {
        requested_epoch: execution_summary.requested_epoch,
        remote_batch_count,
        local_batch_count,
        skipped_batch_count,
        ready_batch_count,
        blocked_batch_count,
        remote_pid_count: execution_summary.remote_pid_count,
        local_pid_count: execution_summary.local_pid_count,
        skipped_pid_count: execution_summary.skipped_pid_count,
        merge_function: SPIRE_REMOTE_SEARCH_MERGE_FUNCTION,
        dedupe_key: "vec_id",
        tie_breaker: "score_then_assignment_role_then_epoch_desc_then_node_pid_version_row_locator",
        top_k: execution_summary.top_k,
        status,
    }
}

pub(crate) fn remote_search_merge_order_contract_rows(
) -> Vec<SpireRemoteSearchMergeOrderContractRow> {
    vec![
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 1,
            order_key: "score",
            direction: "ascending",
            semantic_role: "nearest_candidate_first",
            validator: "must_be_finite",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 2,
            order_key: "assignment_role",
            direction: "primary_before_boundary_replica",
            semantic_role: "prefer_primary_placement_on_tie",
            validator: "must_include_visible_assignment_role",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 3,
            order_key: "served_epoch",
            direction: "descending",
            semantic_role: "newer_epoch_wins_tie",
            validator: "must_equal_requested_epoch",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 4,
            order_key: "node_id",
            direction: "ascending",
            semantic_role: "deterministic_node_tie_breaker",
            validator: "must_equal_origin_node",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 5,
            order_key: "pid",
            direction: "ascending",
            semantic_role: "deterministic_partition_tie_breaker",
            validator: "must_be_selected_pid",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 6,
            order_key: "object_version",
            direction: "descending",
            semantic_role: "newer_object_wins_tie",
            validator: "must_be_positive",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 7,
            order_key: "row_index",
            direction: "ascending",
            semantic_role: "deterministic_row_tie_breaker",
            validator: "must_fit_u32",
        },
        SpireRemoteSearchMergeOrderContractRow {
            order_ordinal: 8,
            order_key: "row_locator",
            direction: "lexicographic_ascending",
            semantic_role: "final_stable_tie_breaker",
            validator: "must_be_nonempty_and_opaque",
        },
    ]
}

pub(crate) fn remote_search_row_locator_contract_rows(
) -> Vec<SpireRemoteSearchRowLocatorContractRow> {
    vec![
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "locator_scope",
            contract_value: "origin_node",
            status: "active_contract",
        },
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "coordinator_interpretation",
            contract_value: "opaque_bytes",
            status: "active_contract",
        },
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "receive_validation",
            contract_value: "nonempty_only",
            status: "active_contract",
        },
        SpireRemoteSearchRowLocatorContractRow {
            contract_item: "remote_heap_resolution",
            contract_value: "requires_origin_node_resolution",
            status: "deferred_until_remote_heap_fetch",
        },
    ]
}

pub(crate) fn remote_search_heap_resolution_contract_rows(
) -> Vec<SpireRemoteSearchHeapResolutionContractRow> {
    vec![
        SpireRemoteSearchHeapResolutionContractRow {
            resolution_scope: "local",
            candidate_source: "coordinator_local_candidate_batch",
            heap_lookup_owner: SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
            row_locator_policy: SPIRE_REMOTE_ROW_LOCATOR_POLICY,
            status: SPIRE_REMOTE_STATUS_READY,
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchHeapResolutionContractRow {
            resolution_scope: "remote",
            candidate_source: "libpq_candidate_batch",
            heap_lookup_owner: SPIRE_REMOTE_HEAP_RESOLUTION,
            row_locator_policy: SPIRE_REMOTE_ROW_LOCATOR_POLICY,
            status: SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
            recommendation: "resolve remote row locators on the origin storage node",
        },
    ]
}

pub(crate) unsafe fn remote_search_finalization_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchFinalizationSummaryRow {
    let merge_summary = unsafe {
        remote_search_merge_input_summary_row(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    };
    remote_search_finalization_summary_from_merge(&merge_summary)
}

fn remote_search_finalization_summary_from_merge(
    merge_summary: &SpireRemoteSearchMergeInputSummaryRow,
) -> SpireRemoteSearchFinalizationSummaryRow {
    let (final_heap_fetch_status, status, recommendation) = if merge_summary.status
        == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
    {
        (
            SPIRE_REMOTE_FINAL_STATUS_BLOCKED,
            SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
            "register remote node descriptors before remote candidate finalization",
        )
    } else if merge_summary.remote_batch_count > 0 {
        (
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
            SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
            "add origin-node row locator resolution before returning remote heap rows",
        )
    } else if merge_summary.local_batch_count > 0 {
        (
            SPIRE_REMOTE_FINAL_STATUS_LOCAL_READY,
            merge_summary.status,
            SPIRE_REMOTE_NONE,
        )
    } else {
        (
            SPIRE_REMOTE_FINAL_STATUS_NO_BATCHES,
            merge_summary.status,
            SPIRE_REMOTE_NONE,
        )
    };

    SpireRemoteSearchFinalizationSummaryRow {
        requested_epoch: merge_summary.requested_epoch,
        remote_batch_count: merge_summary.remote_batch_count,
        local_batch_count: merge_summary.local_batch_count,
        skipped_batch_count: merge_summary.skipped_batch_count,
        merge_status: merge_summary.status,
        row_locator_policy: SPIRE_REMOTE_ROW_LOCATOR_POLICY,
        local_heap_resolution: SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION,
        remote_heap_resolution: SPIRE_REMOTE_HEAP_RESOLUTION,
        final_heap_fetch_status,
        status,
        recommendation,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireCoordinatorPipeline {
    execution_summary: SpireRemoteSearchExecutionSummaryRow,
    dispatch_summary: SpireRemoteSearchLibpqDispatchSummaryRow,
    receive_rows: Vec<SpireRemoteSearchReceivePlanRow>,
    finalization_summary: SpireRemoteSearchFinalizationSummaryRow,
    executor_readiness: SpireRemoteSearchLibpqExecutorReadinessRow,
}

impl SpireCoordinatorPipeline {
    unsafe fn execute_once(
        index_relation: pg_sys::Relation,
        requested_epoch: u64,
        query: Vec<f32>,
        selected_pids: Vec<u64>,
        top_k: usize,
        consistency_mode: &str,
    ) -> Result<Self, String> {
        let top_k_for_empty_plan = u64::try_from(top_k)
            .map_err(|_| "ec_spire coordinator pipeline top_k exceeds u64")?;
        let query_for_summary_fallback = query.clone();
        let readiness_rows = unsafe {
            remote_search_request_readiness_rows(
                index_relation,
                requested_epoch,
                query.clone(),
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        let execution_rows = readiness_rows
            .into_iter()
            .map(remote_search_execution_plan_row_from_readiness)
            .collect::<Vec<_>>();
        let execution_summary = remote_search_execution_summary_from_plan_rows(
            requested_epoch,
            &execution_rows,
            query_for_summary_fallback.clone(),
            top_k_for_empty_plan,
            consistency_mode,
        )?;
        let request_rows = remote_search_libpq_request_plan_rows_from_execution(&execution_rows);
        let connection_rows = remote_search_libpq_connection_plan_rows_from_requests(
            unsafe { (*index_relation).rd_id },
            &request_rows,
        )?;
        let dispatch_rows = remote_search_libpq_dispatch_plan_rows_from_connections(&connection_rows);
        let dispatch_summary = remote_search_libpq_dispatch_summary_from_plan_rows(
            requested_epoch,
            &dispatch_rows,
            query_for_summary_fallback,
            top_k_for_empty_plan,
            consistency_mode,
        )?;
        let receive_rows = remote_search_receive_plan_rows_from_requests(&request_rows);
        let merge_summary = remote_search_merge_input_summary_from_execution(&execution_summary);
        let finalization_summary = remote_search_finalization_summary_from_merge(&merge_summary);
        let secret_rows = remote_search_libpq_secret_plan_rows_from_dispatch(&dispatch_rows);
        let secret_summary =
            remote_search_libpq_secret_summary_from_plan_rows(requested_epoch, &secret_rows)?;
        let executor_readiness = remote_search_libpq_executor_readiness_from_summaries(
            requested_epoch,
            &dispatch_summary,
            &secret_summary,
        );

        Ok(Self {
            execution_summary,
            dispatch_summary,
            receive_rows,
            finalization_summary,
            executor_readiness,
        })
    }
}

pub(crate) unsafe fn remote_search_coordinator_gate_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchCoordinatorGateSummaryRow {
    let pipeline = unsafe {
        SpireCoordinatorPipeline::execute_once(
            index_relation,
            requested_epoch,
            query,
            selected_pids,
            top_k,
            consistency_mode,
        )
    }
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let execution_summary = &pipeline.execution_summary;
    let dispatch_summary = &pipeline.dispatch_summary;
    let receive_rows = &pipeline.receive_rows;
    let finalization_summary = &pipeline.finalization_summary;
    let executor_readiness = &pipeline.executor_readiness;
    let libpq_receive_count =
        u64::try_from(receive_rows.len()).expect("receive row count should fit in u64");
    let libpq_receive_status = receive_rows
        .iter()
        .find(|row| row.status != SPIRE_REMOTE_STATUS_READY)
        .map(|row| row.status)
        .unwrap_or(SPIRE_REMOTE_STATUS_READY);

    let (next_blocker, status, recommendation) =
        if execution_summary.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR {
            (
                "remote_node_descriptor",
                SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR,
                "register remote node descriptors before coordinator execution",
            )
        } else if executor_readiness.status == SPIRE_REMOTE_EXECUTOR_REQUIRED {
            (
                executor_readiness.next_executor_step,
                SPIRE_REMOTE_EXECUTOR_REQUIRED,
                executor_readiness.recommendation,
            )
        } else if execution_summary.status == SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ {
            (
                "libpq_transport",
                SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ,
                "add libpq pipeline execution before remote coordinator dispatch",
            )
        } else if finalization_summary.final_heap_fetch_status
            == SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
        {
            (
                "remote_heap_resolution",
                SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP,
                "add origin-node row locator resolution before returning remote heap rows",
            )
        } else if finalization_summary.status == SPIRE_REMOTE_STATUS_EMPTY_TOP_K {
            (
                SPIRE_REMOTE_NONE,
                SPIRE_REMOTE_STATUS_EMPTY_TOP_K,
                SPIRE_REMOTE_NONE,
            )
        } else {
            (SPIRE_REMOTE_NONE, finalization_summary.status, SPIRE_REMOTE_NONE)
        };

    SpireRemoteSearchCoordinatorGateSummaryRow {
        requested_epoch,
        local_plan_count: execution_summary.local_plan_count,
        remote_plan_count: execution_summary.remote_plan_count,
        skipped_plan_count: execution_summary.skipped_plan_count,
        local_pid_count: execution_summary.local_pid_count,
        remote_pid_count: execution_summary.remote_pid_count,
        skipped_pid_count: execution_summary.skipped_pid_count,
        execution_status: execution_summary.status,
        libpq_dispatch_count: dispatch_summary.dispatch_count,
        libpq_dispatch_status: dispatch_summary.status,
        libpq_executor_status: executor_readiness.status,
        libpq_executor_next_step: executor_readiness.next_executor_step,
        libpq_receive_count,
        libpq_receive_status,
        merge_status: finalization_summary.merge_status,
        final_heap_fetch_status: finalization_summary.final_heap_fetch_status,
        next_blocker,
        status,
        recommendation,
    }
}

pub(crate) unsafe fn remote_search_heap_resolution_summary_row(
    index_relation: pg_sys::Relation,
    requested_epoch: u64,
    query: Vec<f32>,
    selected_pids: Vec<u64>,
    top_k: usize,
    consistency_mode: &str,
) -> SpireRemoteSearchHeapResolutionSummaryRow {
    let gate = unsafe {
        remote_search_coordinator_gate_summary_row(
            index_relation,
            requested_epoch,
            query.clone(),
            selected_pids.clone(),
            top_k,
            consistency_mode,
        )
    };

    let decoded_local_locator_count = if gate.remote_plan_count == 0
        && gate.status != SPIRE_REMOTE_STATUS_EMPTY_TOP_K
    {
        let rows = unsafe {
            remote_search_local_heap_resolution_plan_rows(
                index_relation,
                requested_epoch,
                query,
                selected_pids,
                top_k,
                consistency_mode,
            )
        };
        u64::try_from(rows.len())
            .unwrap_or_else(|_| pgrx::error!("ec_spire local heap resolution row count overflow"))
    } else {
        0
    };

    let local_heap_resolution_status = if gate.local_plan_count == 0 {
        SPIRE_REMOTE_NONE
    } else if gate.status == SPIRE_REMOTE_STATUS_EMPTY_TOP_K {
        SPIRE_REMOTE_STATUS_EMPTY_TOP_K
    } else if gate.remote_plan_count == 0 && remote_search_status_allows_local_heap_rows(gate.status)
    {
        SPIRE_REMOTE_STATUS_READY
    } else {
        SPIRE_REMOTE_FINAL_STATUS_PLANNED
    };
    let remote_heap_resolution_status = if gate.remote_plan_count == 0 {
        SPIRE_REMOTE_NONE
    } else if gate.status == SPIRE_REMOTE_STATUS_REQUIRES_DESCRIPTOR
        || gate.status == SPIRE_REMOTE_STATUS_REQUIRES_LIBPQ
    {
        gate.status
    } else {
        SPIRE_REMOTE_FINAL_STATUS_REQUIRES_REMOTE_HEAP
    };

    SpireRemoteSearchHeapResolutionSummaryRow {
        requested_epoch,
        local_plan_count: gate.local_plan_count,
        remote_plan_count: gate.remote_plan_count,
        skipped_plan_count: gate.skipped_plan_count,
        local_pid_count: gate.local_pid_count,
        remote_pid_count: gate.remote_pid_count,
        decoded_local_locator_count,
        local_heap_resolution_status,
        remote_heap_resolution_status,
        status: gate.status,
        recommendation: gate.recommendation,
    }
}

/// Validates one target-scoped remote candidate receive batch.
///
/// The batch must match the requested epoch, expected node, selected PID set,
/// visible assignment flags, nonempty vec_id, nonempty opaque row_locator, and
/// finite score contract before candidates can enter the merge path.
pub(crate) fn validate_remote_search_candidate_batch(
    requested_epoch: u64,
    expected_node_id: u32,
    selected_pids: &[u64],
    candidates: &[SpireRemoteSearchCandidateRow],
) -> Result<(), String> {
    if requested_epoch == 0 {
        return Err(
            "ec_spire remote candidate batch requested_epoch must be greater than 0".to_owned(),
        );
    }

    let mut selected = HashSet::new();
    for &pid in selected_pids {
        if pid == 0 {
            return Err("ec_spire remote candidate batch selected PID 0 is invalid".to_owned());
        }
        if !selected.insert(pid) {
            return Err(format!(
                "ec_spire remote candidate batch selected PID {pid} appears more than once"
            ));
        }
    }

    for candidate in candidates {
        if candidate.served_epoch != requested_epoch {
            return Err(format!(
                "ec_spire remote candidate batch served epoch {} does not match requested epoch {requested_epoch}",
                candidate.served_epoch
            ));
        }
        if candidate.node_id != expected_node_id {
            return Err(format!(
                "ec_spire remote candidate batch node_id {} does not match expected node_id {expected_node_id}",
                candidate.node_id
            ));
        }
        if candidate.pid == 0 {
            return Err("ec_spire remote candidate batch candidate PID 0 is invalid".to_owned());
        }
        if !selected.contains(&candidate.pid) {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} was not selected for node_id {expected_node_id}",
                candidate.pid
            ));
        }
        if candidate.object_version == 0 {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} has object_version 0",
                candidate.pid
            ));
        }
        if !storage::is_visible_scored_assignment_flags(candidate.assignment_flags) {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} has non-visible assignment_flags {}",
                candidate.pid, candidate.assignment_flags
            ));
        }
        if candidate.vec_id.is_empty() {
            return Err("ec_spire remote candidate batch received empty vec_id".to_owned());
        }
        if candidate.row_locator.is_empty() {
            return Err(format!(
                "ec_spire remote candidate batch candidate PID {} has empty row_locator",
                candidate.pid
            ));
        }
        if !candidate.score.is_finite() {
            return Err("ec_spire remote candidate batch received non-finite score".to_owned());
        }
    }

    Ok(())
}

/// Merges candidates that share one coordinator-scoped `vec_id` namespace.
///
/// Current local SPIRE writers allocate node-local vec-id bytes. Until the
/// global vec-id format lands, multi-node callers must only use this helper
/// when they can prove the input vec-id bytes are globally unique by
/// construction.
pub(crate) fn merge_remote_search_candidates<I>(
    candidates: I,
    limit: Option<usize>,
) -> Result<SpireRemoteSearchMergeResult, String>
where
    I: IntoIterator<Item = SpireRemoteSearchCandidateRow>,
{
    let mut input_count = 0_u64;
    let mut duplicate_vec_id_count = 0_u64;
    let mut best_by_vec_id: HashMap<Vec<u8>, SpireRemoteSearchCandidateRow> = HashMap::new();

    for candidate in candidates {
        input_count = input_count
            .checked_add(1)
            .ok_or_else(|| "ec_spire remote candidate merge input count overflow".to_owned())?;
        if !candidate.score.is_finite() {
            return Err("ec_spire remote candidate merge received non-finite score".to_owned());
        }
        if candidate.vec_id.is_empty() {
            return Err("ec_spire remote candidate merge received empty vec_id".to_owned());
        }

        match best_by_vec_id.entry(candidate.vec_id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                duplicate_vec_id_count =
                    duplicate_vec_id_count.checked_add(1).ok_or_else(|| {
                        "ec_spire remote candidate merge duplicate count overflow".to_owned()
                    })?;
                if remote_search_candidate_cmp(&candidate, entry.get()).is_lt() {
                    *entry.get_mut() = candidate;
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(candidate);
            }
        }
    }

    let mut candidates = best_by_vec_id.into_values().collect::<Vec<_>>();
    candidates.sort_by(remote_search_candidate_cmp);
    if let Some(limit) = limit {
        candidates.truncate(limit);
    }

    Ok(SpireRemoteSearchMergeResult {
        candidates,
        input_count,
        duplicate_vec_id_count,
    })
}

/// Validates each target-scoped receive batch before global candidate merge.
///
/// The same global-vec-id precondition as `merge_remote_search_candidates`
/// applies when batches span more than one node.
pub(crate) fn merge_validated_remote_search_candidate_batches(
    requested_epoch: u64,
    batches: Vec<SpireRemoteSearchCandidateBatch>,
    limit: Option<usize>,
) -> Result<SpireRemoteSearchMergeResult, String> {
    for batch in &batches {
        validate_remote_search_candidate_batch(
            requested_epoch,
            batch.node_id,
            &batch.selected_pids,
            &batch.candidates,
        )?;
    }

    merge_remote_search_candidates(
        batches.into_iter().flat_map(|batch| batch.candidates),
        limit,
    )
}
