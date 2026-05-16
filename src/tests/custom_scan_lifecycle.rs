    #[pg_test]
    fn test_ec_spire_customscan_uses_cic_refreshed_descriptor_sql() {
        let _env_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _conninfo_secret = ScopedEnvVar::set(
            "EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_CIC_REFRESH",
            &loopback_conninfo,
        );
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");

        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_customscan_cic_refresh_remote_sql; \
                 CREATE TABLE ec_spire_customscan_cic_refresh_remote_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 INSERT INTO ec_spire_customscan_cic_refresh_remote_sql \
                     (id, title, embedding) VALUES \
                 (9101, 'refreshed descriptor alpha', \
                  encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                 (9102, 'refreshed descriptor beta', \
                  encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_customscan_cic_refresh_old_idx \
                     ON ec_spire_customscan_cic_refresh_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
            )
            .expect("loopback CIC-refresh remote fixture should be created");

        Spi::run(
            "CREATE TABLE ec_spire_customscan_cic_refresh_coord_sql \
                 (id bigint primary key, title text not null, embedding ecvector); \
             INSERT INTO ec_spire_customscan_cic_refresh_coord_sql \
                 (id, title, embedding) VALUES \
             (1, 'coord alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'coord beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
             CREATE INDEX ec_spire_customscan_cic_refresh_coord_idx \
                 ON ec_spire_customscan_cic_refresh_coord_sql USING ec_spire \
                 (embedding ecvector_spire_ip_ops) \
                 WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
        )
        .expect("coordinator CIC-refresh fixture should be created");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_customscan_cic_refresh_coord_idx'::regclass::oid",
        )
        .expect("CIC-refresh coordinator index oid query should succeed")
        .expect("CIC-refresh coordinator index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot(\
                 'ec_spire_customscan_cic_refresh_coord_idx'::regclass)",
        )
        .expect("CIC-refresh active epoch query should succeed")
        .expect("CIC-refresh active epoch should exist");
        let coord_leaf_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot(\
                 'ec_spire_customscan_cic_refresh_coord_idx'::regclass)",
        )
        .expect("CIC-refresh coordinator leaf pid query should succeed")
        .expect("CIC-refresh coordinator leaf pids should exist");
        let old_remote_leaf_pids = loopback_client
            .query_one(
                "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot(\
                     'ec_spire_customscan_cic_refresh_old_idx'::regclass)",
                &[],
            )
            .expect("CIC-refresh old remote leaf pid query should succeed")
            .try_get::<_, Vec<i64>>(0)
            .expect("CIC-refresh old remote leaf pids should decode");
        assert_eq!(old_remote_leaf_pids, coord_leaf_pids);

        unsafe {
            for pid in &coord_leaf_pids {
                am::debug_spire_rewrite_placement_node(index_oid, *pid as u64, 91);
            }
        }

        let old_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_customscan_cic_refresh_old_idx",
        );
        let old_register = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 91, 51, 'spire/remote/customscan/cic_refresh', \
                     decode('{old_identity_hex}', 'hex'), \
                     'ec_spire_customscan_cic_refresh_old_idx', 'active', \
                     {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("old CIC-refresh descriptor registration should succeed")
        .expect("old CIC-refresh descriptor registration result should exist");
        assert!(old_register);

        loopback_client
            .batch_execute(
                "CREATE INDEX CONCURRENTLY ec_spire_customscan_cic_refresh_new_idx \
                     ON ec_spire_customscan_cic_refresh_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq')",
            )
            .expect("loopback CIC-refresh new remote index should be created");
        let new_remote_leaf_pids = loopback_client
            .query_one(
                "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot(\
                     'ec_spire_customscan_cic_refresh_new_idx'::regclass)",
                &[],
            )
            .expect("CIC-refresh new remote leaf pid query should succeed")
            .try_get::<_, Vec<i64>>(0)
            .expect("CIC-refresh new remote leaf pids should decode");
        assert_eq!(new_remote_leaf_pids, coord_leaf_pids);
        let new_identity_hex = loopback_remote_index_identity_hex(
            &mut loopback_client,
            "ec_spire_customscan_cic_refresh_new_idx",
        );
        assert_ne!(old_identity_hex, new_identity_hex);

        let new_register = Spi::get_one::<bool>(&format!(
            "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 91, 52, 'spire/remote/customscan/cic_refresh', \
                     decode('{new_identity_hex}', 'hex'), \
                     'ec_spire_customscan_cic_refresh_new_idx', 'active', \
                     {active_epoch}, {active_epoch}, '{}', 'none')",
            u32::from(index_oid),
            env!("CARGO_PKG_VERSION")
        ))
        .expect("new CIC-refresh descriptor registration should succeed")
        .expect("new CIC-refresh descriptor registration result should exist");
        assert!(new_register);
        let descriptor_row = Spi::get_one::<String>(&format!(
            "SELECT descriptor_generation::text || ':' || remote_index_regclass || ':' || \
                    encode(remote_index_identity, 'hex') \
               FROM ec_spire_remote_node_descriptor \
              WHERE coordinator_index_oid = '{}'::oid AND node_id = 91",
            u32::from(index_oid)
        ))
        .expect("CIC-refresh descriptor row query should succeed")
        .expect("CIC-refresh descriptor row should exist");
        assert_eq!(
            descriptor_row,
            format!("52:ec_spire_customscan_cic_refresh_new_idx:{new_identity_hex}")
        );

        loopback_client
            .batch_execute("DROP INDEX ec_spire_customscan_cic_refresh_old_idx")
            .expect("old CIC-refresh remote index drop should succeed");

        Spi::run("SET LOCAL enable_seqscan = off").expect("disable seqscan should succeed");
        Spi::run("SET LOCAL enable_indexscan = off").expect("disable indexscan should succeed");
        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id, title FROM ec_spire_customscan_cic_refresh_coord_sql \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1",
                    None,
                    &[],
                )
                .expect("CIC-refresh CustomScan explain should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("CIC-refresh plan row should decode")
                        .expect("CIC-refresh plan row should not be NULL"),
                );
            }
            lines.join("\n")
        });
        let custom_scan_row = Spi::get_one::<String>(
            "SELECT id::text || ':' || title \
               FROM ec_spire_customscan_cic_refresh_coord_sql \
              ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1",
        )
        .expect("CIC-refresh CustomScan query should succeed")
        .expect("CIC-refresh CustomScan should return one row");

        assert!(
            plan.contains("node: EcSpireDistributedScan"),
            "expected refreshed descriptor CustomScan plan:\n{plan}"
        );
        assert_eq!(custom_scan_row, "9101:refreshed descriptor alpha");
    }
