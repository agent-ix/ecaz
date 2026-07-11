// Task 179 registry privilege and SECURITY DEFINER search-path regressions.
// This stays separate from the lifecycle fixture so the attacker path remains
// small and reviewable.

#[pg_test]
fn test_distann_registry_security_definer_path_and_acl() {
    const ROLE: &str = "ec_distann_registry_unprivileged";
    const ATTACKER_SCHEMA: &str = "ec_distann_registry_attacker";
    const SECRET_NAME: &str = "DISTANN_REGISTRY_SECURITY";
    const SECRET_KEY: &str = "EC_SPIRE_REMOTE_CONNINFO_DISTANN_REGISTRY_SECURITY";

    let _env_lock = env_var_test_lock();
    let _conninfo_secret = ScopedEnvVar::set(SECRET_KEY, "host=/unused dbname=unused");
    let coordinator =
        create_distann_physical_generation_fixture("ec_distann_security_coordinator", 0x81);
    let participant =
        create_distann_physical_generation_fixture("ec_distann_security_participant", 0x82);
    configure_distann_participant_identity(&participant, "security/node-19");

    let extension_schema = Spi::get_one::<String>(
        "SELECT pg_catalog.quote_ident(n.nspname)
           FROM pg_catalog.pg_extension e
           JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace
          WHERE e.extname = 'ecaz'",
    )
    .unwrap()
    .expect("extension schema should resolve");

    Spi::run(&format!(
        "CREATE ROLE {ROLE} NOLOGIN;
         CREATE SCHEMA {ATTACKER_SCHEMA};
         GRANT USAGE ON SCHEMA {extension_schema}, {ATTACKER_SCHEMA} TO {ROLE}"
    ))
    .unwrap();

    let endpoint_denial = expect_pg_error_rolled_back(|| {
        Spi::run(&format!(
            "SET LOCAL ROLE {ROLE};
             SELECT {extension_schema}.ec_distann_control_identity(
                 {}::pg_catalog.oid::pg_catalog.regclass
             )",
            u32::from(participant.index_oid),
        ))
        .expect("ungranted control identity call must fail");
    });
    assert!(
        endpoint_denial.contains("permission denied")
            && endpoint_denial.contains("ec_distann_control_identity"),
        "unexpected endpoint ACL error: {endpoint_denial}"
    );

    let catalog_denial = expect_pg_error_rolled_back(|| {
        Spi::run(&format!(
            "SET LOCAL ROLE {ROLE};
             SELECT count(*) FROM {extension_schema}.ec_distann_node_descriptor"
        ))
        .expect("direct desired-roster read must fail");
    });
    assert!(
        catalog_denial.contains("permission denied")
            && catalog_denial.contains("ec_distann_node_descriptor"),
        "unexpected catalog ACL error: {catalog_denial}"
    );

    Spi::run(&format!(
        "CREATE FUNCTION {ATTACKER_SCHEMA}.ec_distann_control_identity(
             pg_catalog.regclass
         ) RETURNS TABLE (
             logical_index_uuid pg_catalog.uuid,
             index_format_version pg_catalog.int4,
             distributed_control pg_catalog.bool,
             compatibility_digest pg_catalog.bytea,
             endpoint_identity pg_catalog.text,
             canonical_index_regclass pg_catalog.text
         ) LANGUAGE plpgsql AS $spoof$
         BEGIN
             RAISE EXCEPTION 'SPOOF_CALLED';
         END
         $spoof$;
         CREATE TEMP TABLE uuid (trap pg_catalog.int4);
         CREATE TEMP TABLE oid (trap pg_catalog.int4);
         CREATE TEMP TABLE text (trap pg_catalog.int4);
         CREATE TEMP TABLE integer (trap pg_catalog.int4);
         CREATE TEMP TABLE ec_distann_registry_state (trap pg_catalog.int4);
         CREATE TEMP TABLE ec_distann_node_descriptor (trap pg_catalog.int4);
         GRANT EXECUTE ON FUNCTION {extension_schema}.ec_distann_register_node_descriptor(
             pg_catalog.regclass, pg_catalog.int4, pg_catalog.int4,
             pg_catalog.text, pg_catalog.text, pg_catalog.text, pg_catalog.bool
         ) TO {ROLE};
         GRANT EXECUTE ON FUNCTION {extension_schema}.ec_distann_unregister_node_descriptor(
             pg_catalog.regclass, pg_catalog.int4
         ) TO {ROLE}"
    ))
    .unwrap();

    Spi::run(&format!(
        "SET LOCAL ROLE {ROLE};
         SET LOCAL search_path TO {ATTACKER_SCHEMA}, pg_catalog, {extension_schema}"
    ))
    .unwrap();
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "SELECT {extension_schema}.ec_distann_register_node_descriptor(
                         $1::pg_catalog.oid::pg_catalog.regclass,
                         $2::pg_catalog.int4, $3::pg_catalog.int4,
                         $4::pg_catalog.text, $5::pg_catalog.text,
                         $6::pg_catalog.text, $7::pg_catalog.bool
                     )"
                ),
                None,
                &[
                    coordinator.index_oid.into(),
                    0_i32.into(),
                    19_i32.into(),
                    "security/node-19".to_owned().into(),
                    SECRET_NAME.to_owned().into(),
                    participant.canonical_index_regclass.clone().into(),
                    true.into(),
                ],
            )
            .expect("granted registry call must ignore caller/temp shadows");
    });
    Spi::run("RESET ROLE; RESET search_path").unwrap();

    let descriptor_catalog = distann_catalog_name("ec_distann_node_descriptor");
    assert_eq!(
        Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM {descriptor_catalog}
              WHERE index_oid = {} AND logical_index_uuid = '{}'::pg_catalog.uuid
                AND roster_ordinal = 0 AND node_id = 19",
            u32::from(coordinator.index_oid),
            coordinator.logical_index_uuid,
        ))
        .unwrap(),
        Some(1),
        "the extension-owned desired roster must receive the granted mutation"
    );
    assert_eq!(
        Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM pg_catalog.pg_locks
              WHERE pid = pg_catalog.pg_backend_pid()
                AND relation = {}::pg_catalog.oid
                AND mode = 'ShareRowExclusiveLock'
                AND granted",
            u32::from(coordinator.index_oid),
        ))
        .unwrap(),
        Some(1),
        "the coordinator relation lock must survive the SQL call until transaction end"
    );

    let safely_configured_endpoints = Spi::get_one::<i64>(&format!(
        "SELECT count(*)
           FROM pg_catalog.pg_proc p
           JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
           JOIN pg_catalog.pg_extension e ON e.extnamespace = n.oid
          WHERE e.extname = 'ecaz'
            AND p.proname = ANY (ARRAY[
                'ec_distann_control_identity',
                'ec_distann_configure_participant_identity',
                'ec_distann_register_node_descriptor',
                'ec_distann_unregister_node_descriptor',
                'ec_distann_begin_epoch_handoff',
                'ec_distann_abort_epoch_handoff',
                'ec_distann_list_unpublished_generations',
                'ec_distann_catalog_index_cleanup'
            ]::pg_catalog.text[])
            AND pg_catalog.format(
                    'search_path=pg_catalog, %I, pg_temp', n.nspname
                ) = ANY (p.proconfig)"
    ))
    .unwrap()
    .unwrap();
    assert_eq!(
        safely_configured_endpoints, 8,
        "every current DistANN definer endpoint must put explicit pg_temp last"
    );

    Spi::run(&format!(
        "SET LOCAL ROLE {ROLE};
         SET LOCAL search_path TO {ATTACKER_SCHEMA}, pg_catalog, {extension_schema}"
    ))
    .unwrap();
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "SELECT {extension_schema}.ec_distann_unregister_node_descriptor(
                         $1::pg_catalog.oid::pg_catalog.regclass,
                         $2::pg_catalog.int4
                     )"
                ),
                None,
                &[coordinator.index_oid.into(), 0_i32.into()],
            )
            .expect("granted unregister must ignore caller/temp shadows");
    });
    Spi::run("RESET ROLE; RESET search_path").unwrap();
    assert_eq!(
        Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM {descriptor_catalog}
              WHERE index_oid = {} AND logical_index_uuid = '{}'::pg_catalog.uuid",
            u32::from(coordinator.index_oid),
            coordinator.logical_index_uuid,
        ))
        .unwrap(),
        Some(0)
    );
}
