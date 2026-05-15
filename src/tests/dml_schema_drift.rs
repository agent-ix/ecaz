    #[derive(Clone, Copy)]
    enum EcSpireDmlSchemaDriftOperation {
        Update,
        Delete,
    }

    impl EcSpireDmlSchemaDriftOperation {
        fn name(self) -> &'static str {
            match self {
                Self::Update => "update",
                Self::Delete => "delete",
            }
        }

        fn invoke_sql(self, coord_index: &str, pk: i64) -> String {
            match self {
                Self::Update => format!(
                    "SELECT * FROM ec_spire_forward_coordinator_update_tuple_payload(\
                         '{coord_index}'::regclass, \
                         'id', \
                         int8send({pk}::bigint)::bytea, \
                         jsonb_build_object('title', 'after drift'), \
                         ARRAY['title']::text[])"
                ),
                Self::Delete => format!(
                    "SELECT * FROM ec_spire_prepare_coordinator_delete_tuple_payload(\
                         '{coord_index}'::regclass, \
                         'id', \
                         int8send({pk}::bigint)::bytea)"
                ),
            }
        }
    }

    #[derive(Clone, Copy)]
    enum EcSpireDmlSchemaDriftVariant {
        CoordinatorOnly,
        RemoteOnly,
        BothSides,
    }

    impl EcSpireDmlSchemaDriftVariant {
        fn suffix(self) -> &'static str {
            match self {
                Self::CoordinatorOnly => "coord_only",
                Self::RemoteOnly => "remote_only",
                Self::BothSides => "both_sides",
            }
        }

        fn expected_message(self) -> &'static str {
            match self {
                Self::CoordinatorOnly => "coordinator side drifted",
                Self::RemoteOnly => "remote side drifted",
                Self::BothSides => "coordinator and remote schema fingerprints differ",
            }
        }
    }

    const DML_SCHEMA_DRIFT_SECRET_ENV: &str =
        "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_DML_SCHEMA_DRIFT_VARIANTS";
    const DML_SCHEMA_DRIFT_SECRET_NAME: &str = "spire/remote/dml_schema_drift_variants";
    const DML_SCHEMA_DRIFT_SOURCE_IDENTITY: &str = "9192939495969798999a9b9c9d9e9fa0";

    fn assert_ec_spire_dml_schema_drift_variant_sql(
        loopback_client: &mut postgres::Client,
        operation: EcSpireDmlSchemaDriftOperation,
        variant: EcSpireDmlSchemaDriftVariant,
        pk: i64,
        node_id: i32,
    ) {
        let suffix = format!("{}_{}", operation.name(), variant.suffix());
        let remote_table = format!("ec_spire_dml_schema_drift_{suffix}_remote");
        let remote_index = format!("ec_spire_dml_schema_drift_{suffix}_remote_idx");
        let coord_table = format!("ec_spire_dml_schema_drift_{suffix}_coord");
        let coord_index = format!("ec_spire_dml_schema_drift_{suffix}_coord_idx");

        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS {remote_table}; \
                 DROP TABLE IF EXISTS {coord_table}; \
                 CREATE TABLE {remote_table} \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO {remote_table} \
                     (id, title, embedding, source_identity) VALUES \
                     ({pk}, 'before drift', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                      decode('{DML_SCHEMA_DRIFT_SOURCE_IDENTITY}', 'hex')); \
                 CREATE INDEX {remote_index} \
                     ON {remote_table} USING ec_spire \
                     (embedding ecvector_spire_ip_ops); \
                 CREATE TABLE {coord_table} \
                     (id bigint primary key, title text not null, embedding ecvector, \
                      source_identity bytea not null); \
                 INSERT INTO {coord_table} \
                     (id, title, embedding, source_identity) VALUES \
                     (1, 'coordinator seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42), \
                      decode('a1a2a3a4a5a6a7a8a9aaabacadaeafb0', 'hex')); \
                 CREATE INDEX {coord_index} \
                     ON {coord_table} USING ec_spire \
                     (embedding ecvector_spire_ip_ops);"
            ))
            .expect("loopback DML schema drift fixture should be created");

        let active_epoch = loopback_client
            .query_one(
                &format!(
                    "SELECT active_epoch \
                       FROM ec_spire_index_hierarchy_snapshot('{coord_index}'::regclass)"
                ),
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let index_oid = loopback_client
            .query_one(
                &format!("SELECT '{coord_index}'::regclass::oid::bigint"),
                &[],
            )
            .expect("coordinator index oid query should succeed")
            .try_get::<_, i64>(0)
            .expect("coordinator index oid should decode");
        let remote_identity_hex = loopback_client
            .query_one(
                &format!(
                    "SELECT profile_fingerprint \
                       FROM ec_spire_remote_search_endpoint_identity(\
                            '{remote_index}'::regclass::oid)"
                ),
                &[],
            )
            .expect("remote identity query should succeed")
            .try_get::<_, String>(0)
            .expect("remote identity should decode");

        register_ec_spire_dml_schema_drift_descriptor(
            loopback_client,
            &coord_index,
            &remote_index,
            &remote_identity_hex,
            active_epoch,
            node_id,
            31,
        );
        loopback_client
            .batch_execute(&format!(
                "INSERT INTO ec_spire_placement \
                     (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
                 VALUES ('{coord_index}'::regclass, int8send({pk}::bigint)::bytea, \
                         {node_id}, 2, {active_epoch}, \
                         decode('{DML_SCHEMA_DRIFT_SOURCE_IDENTITY}', 'hex'))"
            ))
            .expect("DML schema drift placement row should be inserted");

        match variant {
            EcSpireDmlSchemaDriftVariant::CoordinatorOnly => loopback_client
                .batch_execute(&format!("ALTER TABLE {coord_table} ADD COLUMN coord_only text"))
                .expect("coordinator-only DDL should succeed"),
            EcSpireDmlSchemaDriftVariant::RemoteOnly => loopback_client
                .batch_execute(&format!("ALTER TABLE {remote_table} ADD COLUMN remote_only text"))
                .expect("remote-only DDL should succeed"),
            EcSpireDmlSchemaDriftVariant::BothSides => {
                loopback_client
                    .batch_execute(&format!(
                        "ALTER TABLE {coord_table} ADD COLUMN coord_side text; \
                         ALTER TABLE {remote_table} ADD COLUMN remote_side integer"
                    ))
                    .expect("both-sides DDL should succeed");
                register_ec_spire_dml_schema_drift_descriptor(
                    loopback_client,
                    &coord_index,
                    &remote_index,
                    &remote_identity_hex,
                    active_epoch,
                    node_id,
                    32,
                );
            }
        }

        let error = loopback_client
            .batch_execute(&operation.invoke_sql(&coord_index, pk))
            .expect_err("DML schema drift should fail before remote dispatch");
        let message = error
            .as_db_error()
            .map(|db_error| db_error.message().to_owned())
            .unwrap_or_else(|| error.to_string());
        assert!(
            message.contains(&format!(
                "ec_spire coordinator {} status schema_drift",
                operation.name()
            )),
            "{message}"
        );
        assert!(message.contains(variant.expected_message()), "{message}");

        let remote_summary = loopback_client
            .query_one(
                &format!(
                    "SELECT title || '|' || count(*)::text \
                       FROM {remote_table} \
                      WHERE id = {pk} \
                      GROUP BY title"
                ),
                &[],
            )
            .expect("remote DML schema drift summary query should succeed")
            .try_get::<_, String>(0)
            .expect("remote DML schema drift summary should decode");
        let prepared_count = loopback_client
            .query_one(
                "SELECT count(*)::bigint \
                   FROM pg_prepared_xacts \
                  WHERE gid LIKE $1",
                &[&format!(
                    "ec_spire_insert_{index_oid}_{node_id}_{active_epoch}_%"
                )],
            )
            .expect("prepared xact count query should succeed")
            .try_get::<_, i64>(0)
            .expect("prepared xact count should decode");
        let placement_count = loopback_client
            .query_one(
                &format!(
                    "SELECT count(*)::bigint \
                       FROM ec_spire_placement \
                      WHERE index_oid = '{coord_index}'::regclass \
                        AND pk_value = int8send({pk}::bigint)::bytea"
                ),
                &[],
            )
            .expect("placement count query should succeed")
            .try_get::<_, i64>(0)
            .expect("placement count should decode");

        assert_eq!(remote_summary, "before drift|1");
        assert_eq!(prepared_count, 0);
        assert_eq!(placement_count, 1);
    }

    fn register_ec_spire_dml_schema_drift_descriptor(
        loopback_client: &mut postgres::Client,
        coord_index: &str,
        remote_index: &str,
        remote_identity_hex: &str,
        active_epoch: i64,
        node_id: i32,
        descriptor_generation: i32,
    ) {
        loopback_client
            .batch_execute(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                     '{coord_index}'::regclass, \
                     {node_id}, {descriptor_generation}, '{DML_SCHEMA_DRIFT_SECRET_NAME}', \
                     decode('{remote_identity_hex}', 'hex'), \
                     '{remote_index}', \
                     'active', {active_epoch}, {active_epoch}, '{}', '')",
                env!("CARGO_PKG_VERSION")
            ))
            .expect("remote descriptor registration should succeed");
    }

    #[pg_test]
    fn test_ec_spire_update_schema_drift_variants_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .execute(
                "SELECT tests.ec_spire_test_set_env_var($1, $2)",
                &[&DML_SCHEMA_DRIFT_SECRET_ENV, &loopback_conninfo],
            )
            .expect("loopback backend should receive DML schema drift conninfo secret env var");

        assert_ec_spire_dml_schema_drift_variant_sql(
            &mut loopback_client,
            EcSpireDmlSchemaDriftOperation::Update,
            EcSpireDmlSchemaDriftVariant::CoordinatorOnly,
            7701,
            71,
        );
        assert_ec_spire_dml_schema_drift_variant_sql(
            &mut loopback_client,
            EcSpireDmlSchemaDriftOperation::Update,
            EcSpireDmlSchemaDriftVariant::RemoteOnly,
            7702,
            72,
        );
        assert_ec_spire_dml_schema_drift_variant_sql(
            &mut loopback_client,
            EcSpireDmlSchemaDriftOperation::Update,
            EcSpireDmlSchemaDriftVariant::BothSides,
            7703,
            73,
        );
    }

    #[pg_test]
    fn test_ec_spire_delete_schema_drift_variants_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .execute(
                "SELECT tests.ec_spire_test_set_env_var($1, $2)",
                &[&DML_SCHEMA_DRIFT_SECRET_ENV, &loopback_conninfo],
            )
            .expect("loopback backend should receive DML schema drift conninfo secret env var");

        assert_ec_spire_dml_schema_drift_variant_sql(
            &mut loopback_client,
            EcSpireDmlSchemaDriftOperation::Delete,
            EcSpireDmlSchemaDriftVariant::CoordinatorOnly,
            7801,
            81,
        );
        assert_ec_spire_dml_schema_drift_variant_sql(
            &mut loopback_client,
            EcSpireDmlSchemaDriftOperation::Delete,
            EcSpireDmlSchemaDriftVariant::RemoteOnly,
            7802,
            82,
        );
        assert_ec_spire_dml_schema_drift_variant_sql(
            &mut loopback_client,
            EcSpireDmlSchemaDriftOperation::Delete,
            EcSpireDmlSchemaDriftVariant::BothSides,
            7803,
            83,
        );
    }
