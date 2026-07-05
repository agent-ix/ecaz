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
