// FR-082 shared-memory scan-token registry and heavyweight-fence coverage.

#[pg_test]
fn test_ec_distann_scan_registry_contract_and_gucs() {
    let settings = Spi::get_two::<String, String>(
        "SELECT
             (SELECT context FROM pg_settings WHERE name = 'ec_distann.max_scan_pins'),
             (SELECT context FROM pg_settings WHERE name = 'ec_distann.max_retire_fences')",
    )
    .expect("scan-registry GUC inspection should succeed");
    assert_eq!(settings.0.as_deref(), Some("postmaster"));
    assert_eq!(settings.1.as_deref(), Some("postmaster"));

    let logical = pgrx::datum::Uuid::from_bytes([
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x41, 0x11, 0x81, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
        0x11,
    ]);
    let token = pgrx::datum::Uuid::from_bytes([
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x42, 0x22, 0x82, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22,
    ]);
    let fingerprint = [0x33; 34];

    let shared_preload = Spi::get_one::<String>("SHOW shared_preload_libraries")
        .expect("shared_preload_libraries inspection should succeed")
        .unwrap_or_default();
    let is_preloaded = shared_preload
        .split(',')
        .any(|library| library.trim() == "ecaz");
    let registration =
        crate::am::ec_distann::register_scan_token_for_test(logical, fingerprint, token);
    if is_preloaded {
        assert_eq!(
            registration,
            Ok(crate::am::ec_distann::ScanRegisterOutcome::Registered)
        );
        assert_eq!(
            crate::am::ec_distann::release_scan_token_for_test(logical, fingerprint, token),
            Ok(true)
        );
    } else {
        assert_eq!(
            registration,
            Err(crate::am::ec_distann::ScanRegistryError::Unavailable)
        );
    }
}

#[pg_test]
fn test_ec_distann_scan_registry_two_backend_retirement_contention() {
    let conninfo = current_pg_test_loopback_conninfo();
    let mut holder = postgres::Client::connect(&conninfo, postgres::NoTls)
        .expect("retirement-fence holder should connect");
    let mut contender = postgres::Client::connect(&conninfo, postgres::NoTls)
        .expect("scan-token contender should connect");
    let extension_schema = holder
        .query_one(
            "SELECT pg_catalog.quote_ident(n.nspname)
               FROM pg_catalog.pg_extension e
               JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace
              WHERE e.extname = 'ecaz'",
            &[],
        )
        .expect("extension schema should resolve")
        .get::<_, String>(0);
    for client in [&mut holder, &mut contender] {
        client
            .batch_execute(&format!("SET search_path = {extension_schema}, public"))
            .expect("test backend search_path should set");
    }

    let logical = "11111111-1111-4111-8111-111111111111";
    let token = "22222222-2222-4222-8222-222222222222";
    holder
        .batch_execute(&format!(
            "BEGIN;
             SELECT ec_distann_test_scan_registry_acquire_retirement('{logical}'::uuid)"
        ))
        .expect("first backend should hold the transaction retirement fence");
    contender
        .batch_execute("SET statement_timeout = '250ms'")
        .expect("contender timeout should set");
    let blocked = contender
        .query_one(
            &format!(
                "SELECT ec_distann_test_scan_registry_register(
                     '{logical}'::uuid, decode(repeat('33', 34), 'hex'), '{token}'::uuid)"
            ),
            &[],
        )
        .expect_err("a second backend must block behind the retirement fence");
    assert_eq!(
        blocked.code(),
        Some(&postgres::error::SqlState::QUERY_CANCELED),
        "contention must be a real heavyweight-lock wait interrupted by statement_timeout: {blocked}"
    );

    holder
        .batch_execute("COMMIT")
        .expect("retirement fence should release at commit");
    contender
        .batch_execute("RESET statement_timeout")
        .expect("contender timeout should reset");
    let registered = contender
        .query_one(
            &format!(
                "SELECT ec_distann_test_scan_registry_register(
                     '{logical}'::uuid, decode(repeat('33', 34), 'hex'), '{token}'::uuid)"
            ),
            &[],
        )
        .expect("registration should succeed after the other backend commits")
        .get::<_, String>(0);
    assert_eq!(registered, "Registered");
    let count = holder
        .query_one(
            &format!(
                "SELECT ec_distann_test_scan_registry_count(
                     '{logical}'::uuid, decode(repeat('33', 34), 'hex'))"
            ),
            &[],
        )
        .expect("the first backend should observe the second backend's shared token")
        .get::<_, i64>(0);
    assert_eq!(count, 1);
    assert!(contender
        .query_one(
            &format!(
                "SELECT ec_distann_test_scan_registry_release(
                         '{logical}'::uuid, decode(repeat('33', 34), 'hex'), '{token}'::uuid)"
            ),
            &[],
        )
        .expect("token release should succeed")
        .get::<_, bool>(0));
}
