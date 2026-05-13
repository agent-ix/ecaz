pub(crate) fn remote_search_endpoint_contract_rows(
) -> Vec<SpireRemoteSearchEndpointContractRow> {
    vec![
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 1,
            contract_item: "endpoint_function",
            contract_value: SPIRE_REMOTE_ENDPOINT_SEARCH,
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_be_registered_strict_pg_extern_endpoint",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 2,
            contract_item: "protocol_version",
            contract_value: SPIRE_REMOTE_CANDIDATE_FORMAT_V1,
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_match_libpq_candidate_format_v1",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 3,
            contract_item: "request_contract",
            contract_value: "remote_index_oid,requested_epoch,query,selected_pids,top_k,consistency_mode",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_match_remote_search_libpq_parameter_contract",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 4,
            contract_item: "response_contract",
            contract_value: "served_epoch,node_id,pid,object_version,row_index,assignment_flags,vec_id,row_locator,score,protocol_version,extension_version,opclass_identity,storage_format,assignment_payload_format,quantizer_profile,scoring_profile,profile_fingerprint,endpoint_status",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_match_remote_search_libpq_result_contract",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 5,
            contract_item: "tuple_transport_capabilities",
            contract_value: "pg_binary_attr_v1",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_advertise_pg_binary_attr_v1_before_custom_scan_typed_receive",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 6,
            contract_item: "tuple_transport_default",
            contract_value: "pg_binary_attr_v1",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_prefer_typed_tuple_transport_when_advertised",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 7,
            contract_item: "tuple_transport_status",
            contract_value: "ready",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_be_ready_before_typed_custom_scan_receive",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 8,
            contract_item: "selected_pid_semantics",
            contract_value: "selected_leaf_pid_set_with_leaf_derived_delta_rows",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "candidate_pid_must_be_selected_leaf_or_leaf_derived_delta_pid",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 9,
            contract_item: "quantizer_family",
            contract_value: "rabitq_only_pq_and_pqfastscan_reserved",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_reject_unsupported_quantizer_families_until_implemented",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 10,
            contract_item: "extension_version_binding",
            contract_value: env!("CARGO_PKG_VERSION"),
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "remote_node_capability_plan_must_match_required_extension_version",
            recommendation: SPIRE_REMOTE_NONE,
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 11,
            contract_item: "scoring_option_binding",
            contract_value: "fixed_index_profile_explicit_request_options_pending",
            status: SPIRE_REMOTE_STATUS_REQUIRES_SCORING_OPTION_BINDING,
            validator: "request_must_bind_scoring_and_rerank_options_before_production_merge",
            recommendation: "add explicit scoring/rerank option fields or a stable served scoring profile binding",
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 12,
            contract_item: "quantizer_index_fingerprint_binding",
            contract_value: "rabitq_profile,code_length,training_stat_fingerprint,storage_format",
            status: SPIRE_REMOTE_STATUS_REQUIRES_FINGERPRINT_BINDING,
            validator: "candidate_batch_must_bind_served_quantizer_index_fingerprint",
            recommendation: "bind fingerprint fields before accepting cross-node remote scores",
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 13,
            contract_item: "opclass_binary_binding",
            contract_value: "opclass_identity_and_binary_score_semantics",
            status: SPIRE_REMOTE_STATUS_REQUIRES_OPCLASS_BINDING,
            validator: "candidate_batch_must_bind_opclass_score_semantics",
            recommendation: "bind opclass identity before accepting remote scores from mixed binaries",
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 14,
            contract_item: "direct_sql_endpoint_status_policy",
            contract_value: "ec_spire_remote_search_exposes_non_ready_endpoint_rows_for_diagnostics_libpq_receive_accepts_ready_only",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_not_treat_direct_sql_rows_as_mergeable_without_libpq_receive_validation",
            recommendation: "use libpq executor readiness surfaces before production remote merge",
        },
        SpireRemoteSearchEndpointContractRow {
            contract_ordinal: 15,
            contract_item: "remote_heap_candidate_endpoint_identity_preflight",
            contract_value: "libpq_heap_candidate_executor_validates_ec_spire_remote_search_endpoint_identity_before_origin_node_heap_rows",
            status: SPIRE_REMOTE_STATUS_READY,
            validator: "must_validate_ready_endpoint_identity_before_remote_heap_candidate_merge",
            recommendation: SPIRE_REMOTE_NONE,
        },
    ]
}

fn remote_search_assignment_payload_format_name(
    format: quantizer::SpireAssignmentPayloadFormat,
) -> &'static str {
    match format {
        quantizer::SpireAssignmentPayloadFormat::TurboQuant => "turboquant",
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => "pq_fastscan",
        quantizer::SpireAssignmentPayloadFormat::RaBitQ => "rabitq",
    }
}

fn remote_search_endpoint_quantizer_profile(
    format: quantizer::SpireAssignmentPayloadFormat,
) -> &'static str {
    match format {
        quantizer::SpireAssignmentPayloadFormat::RaBitQ => "rabitq_v1",
        quantizer::SpireAssignmentPayloadFormat::TurboQuant => "unsupported_turboquant",
        quantizer::SpireAssignmentPayloadFormat::PqFastScan => "unsupported_pq_fastscan",
    }
}

fn remote_search_endpoint_opclass_identity(index_relid: pg_sys::Oid) -> Result<String, String> {
    let sql = format!(
        "SELECT opc.opcname::text AS opclass_identity \
           FROM pg_index idx \
           JOIN pg_opclass opc ON opc.oid = idx.indclass[0] \
          WHERE idx.indexrelid = '{}'::oid",
        u32::from(index_relid)
    );

    Spi::connect(|client| {
        client
            .select(sql.as_str(), None, &[])
            .map_err(|e| format!("ec_spire endpoint identity opclass read failed: {e}"))?
            .map(|row| {
                row["opclass_identity"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire endpoint identity opclass decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire endpoint identity opclass is null".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or_else(|| "unknown".to_owned()))
    })
}

fn remote_search_endpoint_generation_identity(index_relid: pg_sys::Oid) -> Result<String, String> {
    let sql = format!(
        "SELECT pg_relation_filenode('{}'::oid)::text AS generation_identity",
        u32::from(index_relid)
    );

    Spi::connect(|client| {
        client
            .select(sql.as_str(), None, &[])
            .map_err(|e| format!("ec_spire endpoint identity generation read failed: {e}"))?
            .map(|row| {
                row["generation_identity"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire endpoint identity generation decode failed: {e}")
                    })?
                    .ok_or_else(|| {
                        "ec_spire endpoint identity generation is null".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or_else(|| "unknown".to_owned()))
    })
}

fn remote_search_stable_fingerprint(parts: &[String]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in parts {
        for byte in part.as_bytes().iter().copied().chain(std::iter::once(0)) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

fn remote_search_candidate_endpoint_text(
    row: &postgres::Row,
    column: &str,
) -> Result<String, String> {
    row.try_get::<_, String>(column).map_err(|_| {
        format!("ec_spire remote search executor {column} endpoint identity decode failed")
    })
}

fn validate_remote_search_endpoint_identity_fields(
    protocol_version: &str,
    extension_version: &str,
    opclass_identity: &str,
    storage_format: &str,
    assignment_payload_format: &str,
    quantizer_profile: &str,
    scoring_profile: &str,
    profile_fingerprint: &str,
    endpoint_status: &str,
) -> Result<(), String> {
    if protocol_version != SPIRE_REMOTE_CANDIDATE_FORMAT_V1 {
        return Err(format!(
            "ec_spire remote search executor protocol_version {protocol_version} does not match {}",
            SPIRE_REMOTE_CANDIDATE_FORMAT_V1
        ));
    }

    if extension_version != env!("CARGO_PKG_VERSION") {
        return Err(format!(
            "ec_spire remote search executor extension_version {extension_version} does not match {}",
            env!("CARGO_PKG_VERSION")
        ));
    }

    for (column, value) in [
        ("opclass_identity", opclass_identity),
        ("storage_format", storage_format),
        ("assignment_payload_format", assignment_payload_format),
        ("quantizer_profile", quantizer_profile),
        ("scoring_profile", scoring_profile),
        ("profile_fingerprint", profile_fingerprint),
    ] {
        if value.is_empty() {
            return Err(format!(
                "ec_spire remote search executor {column} endpoint identity is empty"
            ));
        }
    }

    if endpoint_status != SPIRE_REMOTE_STATUS_READY {
        return Err(format!(
            "ec_spire remote search executor endpoint_status {endpoint_status} is not ready"
        ));
    }

    Ok(())
}

fn validate_remote_search_candidate_endpoint_identity(
    row: &postgres::Row,
) -> Result<Vec<u8>, String> {
    let protocol_version = remote_search_candidate_endpoint_text(row, "protocol_version")?;
    let extension_version = remote_search_candidate_endpoint_text(row, "extension_version")?;
    let opclass_identity = remote_search_candidate_endpoint_text(row, "opclass_identity")?;
    let storage_format = remote_search_candidate_endpoint_text(row, "storage_format")?;
    let assignment_payload_format =
        remote_search_candidate_endpoint_text(row, "assignment_payload_format")?;
    let quantizer_profile = remote_search_candidate_endpoint_text(row, "quantizer_profile")?;
    let scoring_profile = remote_search_candidate_endpoint_text(row, "scoring_profile")?;
    let profile_fingerprint = remote_search_candidate_endpoint_text(row, "profile_fingerprint")?;
    let endpoint_status = remote_search_candidate_endpoint_text(row, "endpoint_status")?;

    validate_remote_search_endpoint_identity_fields(
        &protocol_version,
        &extension_version,
        &opclass_identity,
        &storage_format,
        &assignment_payload_format,
        &quantizer_profile,
        &scoring_profile,
        &profile_fingerprint,
        &endpoint_status,
    )?;
    remote_search_endpoint_profile_fingerprint_bytes(&profile_fingerprint)
}

fn remote_search_endpoint_profile_fingerprint_bytes(
    profile_fingerprint: &str,
) -> Result<Vec<u8>, String> {
    if profile_fingerprint.len() % 2 != 0 {
        return Err(
            "ec_spire remote search executor profile_fingerprint endpoint identity has invalid hex length"
                .to_owned(),
        );
    }

    (0..profile_fingerprint.len())
        .step_by(2)
        .map(|offset| {
            u8::from_str_radix(&profile_fingerprint[offset..offset + 2], 16).map_err(|_| {
                "ec_spire remote search executor profile_fingerprint endpoint identity is not hex"
                    .to_owned()
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpireRemoteValidatedEndpointIdentity {
    protocol_version: String,
    extension_version: String,
    opclass_identity: String,
    storage_format: String,
    assignment_payload_format: String,
    quantizer_profile: String,
    scoring_profile: String,
    tuple_transport_capabilities: Vec<String>,
    tuple_transport_default: String,
    tuple_transport_status: String,
    profile_fingerprint: String,
    profile_fingerprint_bytes: Vec<u8>,
}

impl SpireRemoteValidatedEndpointIdentity {
    fn prefers_typed_tuple_transport(&self) -> bool {
        remote_endpoint_prefers_typed_tuple_transport(
            self.tuple_transport_status.as_str(),
            self.tuple_transport_default.as_str(),
            &self.tuple_transport_capabilities,
            options::current_session_remote_tuple_transport(),
        )
    }
}

fn remote_tuple_payload_production_sql(
    endpoint_identity: &SpireRemoteValidatedEndpointIdentity,
) -> Result<&'static str, &'static str> {
    if endpoint_identity.prefers_typed_tuple_transport() {
        Ok(SPIRE_REMOTE_SEARCH_LIBPQ_TYPED_TUPLE_PAYLOAD_SQL_TEMPLATE)
    } else {
        Err(SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED)
    }
}

fn remote_endpoint_prefers_typed_tuple_transport(
    tuple_transport_status: &str,
    tuple_transport_default: &str,
    tuple_transport_capabilities: &[String],
    session_transport: options::SpireRemoteTupleTransportGuc,
) -> bool {
    if tuple_transport_status != SPIRE_REMOTE_STATUS_READY
        || !tuple_transport_capabilities
            .iter()
            .any(|capability| capability == SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1)
    {
        return false;
    }
    match session_transport {
        options::SpireRemoteTupleTransportGuc::Auto => {
            tuple_transport_default == SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1
        }
        options::SpireRemoteTupleTransportGuc::JsonTuplePayloadV1 => false,
        options::SpireRemoteTupleTransportGuc::PgBinaryAttrV1 => true,
    }
}

#[cfg(test)]
mod remote_tuple_transport_tests {
    use super::*;

    fn pg_binary_capabilities() -> Vec<String> {
        vec![SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1.to_owned()]
    }

    fn endpoint_identity(
        tuple_transport_default: &str,
        tuple_transport_capabilities: Vec<String>,
    ) -> SpireRemoteValidatedEndpointIdentity {
        SpireRemoteValidatedEndpointIdentity {
            protocol_version: "1".to_owned(),
            extension_version: "0.1.2".to_owned(),
            opclass_identity: "ec_spire".to_owned(),
            storage_format: "rabitq".to_owned(),
            assignment_payload_format: "leaf-pid-v1".to_owned(),
            quantizer_profile: "rabitq".to_owned(),
            scoring_profile: "l2".to_owned(),
            tuple_transport_capabilities,
            tuple_transport_default: tuple_transport_default.to_owned(),
            tuple_transport_status: SPIRE_REMOTE_STATUS_READY.to_owned(),
            profile_fingerprint: "00".to_owned(),
            profile_fingerprint_bytes: vec![0],
        }
    }

    #[test]
    fn remote_tuple_transport_auto_uses_endpoint_default() {
        assert!(remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
            &pg_binary_capabilities(),
            options::SpireRemoteTupleTransportGuc::Auto,
        ));
        assert!(!remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            "json_tuple_payload_v1",
            &pg_binary_capabilities(),
            options::SpireRemoteTupleTransportGuc::Auto,
        ));
    }

    #[test]
    fn remote_tuple_transport_session_override_keeps_capability_gate() {
        assert!(!remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
            &pg_binary_capabilities(),
            options::SpireRemoteTupleTransportGuc::JsonTuplePayloadV1,
        ));
        assert!(remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            "json_tuple_payload_v1",
            &pg_binary_capabilities(),
            options::SpireRemoteTupleTransportGuc::PgBinaryAttrV1,
        ));
        assert!(!remote_endpoint_prefers_typed_tuple_transport(
            SPIRE_REMOTE_STATUS_READY,
            "json_tuple_payload_v1",
            &[],
            options::SpireRemoteTupleTransportGuc::PgBinaryAttrV1,
        ));
    }

    #[test]
    fn remote_tuple_payload_production_sql_requires_typed_transport() {
        let typed_identity = endpoint_identity(
            SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
            pg_binary_capabilities(),
        );
        assert_eq!(
            remote_tuple_payload_production_sql(&typed_identity),
            Ok(SPIRE_REMOTE_SEARCH_LIBPQ_TYPED_TUPLE_PAYLOAD_SQL_TEMPLATE)
        );

        let json_identity = endpoint_identity("json_tuple_payload_v1", pg_binary_capabilities());
        assert_eq!(
            remote_tuple_payload_production_sql(&json_identity),
            Err(SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED)
        );

        let missing_capability_identity =
            endpoint_identity(SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1, Vec::new());
        assert_eq!(
            remote_tuple_payload_production_sql(&missing_capability_identity),
            Err(SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED)
        );
    }
}

fn validate_remote_search_endpoint_identity_row(
    row: &postgres::Row,
) -> Result<SpireRemoteValidatedEndpointIdentity, String> {
    let protocol_version = remote_search_candidate_endpoint_text(row, "protocol_version")?;
    let extension_version = remote_search_candidate_endpoint_text(row, "extension_version")?;
    let opclass_identity = remote_search_candidate_endpoint_text(row, "opclass_identity")?;
    let storage_format = remote_search_candidate_endpoint_text(row, "storage_format")?;
    let assignment_payload_format =
        remote_search_candidate_endpoint_text(row, "assignment_payload_format")?;
    let quantizer_profile = remote_search_candidate_endpoint_text(row, "quantizer_profile")?;
    let scoring_profile = remote_search_candidate_endpoint_text(row, "scoring_profile")?;
    let profile_fingerprint = remote_search_candidate_endpoint_text(row, "profile_fingerprint")?;
    let endpoint_status = remote_search_candidate_endpoint_text(row, "status")?;
    let tuple_transport_capabilities = row
        .try_get::<_, Vec<String>>("tuple_transport_capabilities")
        .unwrap_or_default();
    let tuple_transport_default = row
        .try_get::<_, String>("tuple_transport_default")
        .unwrap_or_else(|_| "json_tuple_payload_v1".to_owned());
    let tuple_transport_status = row
        .try_get::<_, String>("tuple_transport_status")
        .unwrap_or_else(|_| SPIRE_REMOTE_STATUS_READY.to_owned());

    validate_remote_search_endpoint_identity_fields(
        &protocol_version,
        &extension_version,
        &opclass_identity,
        &storage_format,
        &assignment_payload_format,
        &quantizer_profile,
        &scoring_profile,
        &profile_fingerprint,
        &endpoint_status,
    )?;
    let profile_fingerprint_bytes =
        remote_search_endpoint_profile_fingerprint_bytes(&profile_fingerprint)?;
    Ok(SpireRemoteValidatedEndpointIdentity {
        protocol_version,
        extension_version,
        opclass_identity,
        storage_format,
        assignment_payload_format,
        quantizer_profile,
        scoring_profile,
        tuple_transport_capabilities,
        tuple_transport_default,
        tuple_transport_status,
        profile_fingerprint,
        profile_fingerprint_bytes,
    })
}

pub(crate) unsafe fn remote_search_endpoint_identity_row(
    index_relation: pg_sys::Relation,
) -> SpireRemoteSearchEndpointIdentityRow {
    let result = (|| -> Result<SpireRemoteSearchEndpointIdentityRow, String> {
        let relation_options = unsafe { options::relation_options(index_relation) };
        let assignment_payload_format = relation_options.assignment_payload_format();
        let assignment_payload_format_name =
            remote_search_assignment_payload_format_name(assignment_payload_format);
        let quantizer_profile =
            remote_search_endpoint_quantizer_profile(assignment_payload_format);
        let opclass_identity =
            remote_search_endpoint_opclass_identity(unsafe { (*index_relation).rd_id })?;
        let generation_identity =
            remote_search_endpoint_generation_identity(unsafe { (*index_relation).rd_id })?;
        let root_control = unsafe { page::read_root_control_page(index_relation) };
        let scoring_profile = "inner_product_score_v1";
        let storage_format = relation_options.storage_format.reloption_name();
        let profile_fingerprint = remote_search_stable_fingerprint(&[
            SPIRE_REMOTE_CANDIDATE_FORMAT_V1.to_owned(),
            env!("CARGO_PKG_VERSION").to_owned(),
            opclass_identity.clone(),
            storage_format.to_owned(),
            assignment_payload_format_name.to_owned(),
            quantizer_profile.to_owned(),
            scoring_profile.to_owned(),
            relation_options.nlists.to_string(),
            relation_options.recursive_fanout.to_string(),
            relation_options.training_sample_rows.to_string(),
            relation_options.seed.to_string(),
            relation_options.pq_group_size.to_string(),
            root_control.active_epoch.to_string(),
            generation_identity,
        ]);

        let (status, recommendation) =
            if assignment_payload_format == quantizer::SpireAssignmentPayloadFormat::RaBitQ {
                (SPIRE_REMOTE_STATUS_READY, SPIRE_REMOTE_NONE)
            } else {
                (
                    SPIRE_REMOTE_STATUS_REQUIRES_RABITQ_STORAGE_FORMAT,
                    "create or reindex the remote-serving SPIRE index with storage_format = 'rabitq'",
                )
            };

        Ok(SpireRemoteSearchEndpointIdentityRow {
            protocol_version: SPIRE_REMOTE_CANDIDATE_FORMAT_V1,
            extension_version: env!("CARGO_PKG_VERSION"),
            opclass_identity,
            storage_format,
            assignment_payload_format: assignment_payload_format_name,
            quantizer_profile,
            scoring_profile,
            tuple_transport_capabilities: vec![
                SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1.to_owned(),
            ],
            tuple_transport_default: SPIRE_REMOTE_TUPLE_TRANSPORT_PG_BINARY_ATTR_V1,
            tuple_transport_status: SPIRE_REMOTE_STATUS_READY,
            profile_fingerprint,
            status,
            recommendation,
        })
    })();
    result.unwrap_or_else(|e| pgrx::error!("{e}"))
}

