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
            validator: "must_be_selected_leaf_pid_or_leaf_derived_delta_pid",
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

pub(crate) fn remote_search_vector_identity_contract_rows(
) -> Vec<SpireRemoteSearchVectorIdentityContractRow> {
    vec![
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "global_vec_id_format",
            contract_value: "0x02 || stable_global_payload_bytes",
            status: "active_contract",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "local_vec_id_format",
            contract_value: "0x01 || little_endian_u64",
            status: "compatibility_contract",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "remote_merge_dedupe_key",
            contract_value: SPIRE_REMOTE_VEC_ID_DEDUPE_KEY,
            status: "active_contract",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "local_vec_id_remote_scope",
            contract_value: "node_id || local_vec_id_bytes",
            status: "compatibility_fallback",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "boundary_replica_identity",
            contract_value: "primary_and_boundary_replica_rows_share_identical_vec_id_bytes",
            status: "active_contract",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "cross_node_replica_dedupe",
            contract_value: "requires_global_vec_id_format",
            status: "global_id_required",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "writer_identity_allocation_hook",
            contract_value: "SpireVecIdSourceIdentity",
            status: "phase11_2_landed",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "writer_global_source_identity",
            contract_value: "fixed_16_byte_source_identity_required_not_heap_tid",
            status: "phase11_2_contract_defined",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "writer_global_base_storage",
            contract_value: "leaf_v2_global_bytes_fixed_width_per_object",
            status: "phase11_2_landed",
        },
        SpireRemoteSearchVectorIdentityContractRow {
            contract_item: "writer_global_delta_storage",
            contract_value: "row_encoded_delta_assignments_accept_global_vec_id",
            status: "phase11_2_landed",
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

