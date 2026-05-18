use ecaz::bench_api::{
    validate_custom_wal_record_format_version, ECAZ_CUSTOM_WAL_RECORDS_ENABLED,
    ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION, ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION_OFFSET,
};

#[test]
fn custom_wal_policy_states_no_current_custom_payloads() {
    assert!(!ECAZ_CUSTOM_WAL_RECORDS_ENABLED);
    assert_eq!(ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION, 1);
    assert_eq!(ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION_OFFSET, 0);
}

#[test]
fn custom_wal_record_version_validator_rejects_missing_or_unknown_tags() {
    assert_eq!(
        validate_custom_wal_record_format_version(&[]),
        Err("missing ECAZ custom WAL record format version")
    );
    assert_eq!(
        validate_custom_wal_record_format_version(&[ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION + 1]),
        Err("unknown ECAZ custom WAL record format version")
    );
    assert!(validate_custom_wal_record_format_version(&[
        ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION,
        0xaa,
    ])
    .is_ok());
}
