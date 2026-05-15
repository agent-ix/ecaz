    #[pg_test]
    fn test_ec_spire_relation_storage_snapshot_during_reindex_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_storage_reindex_snapshot_sql \
                 (id bigint primary key, embedding ecvector); \
             INSERT INTO ec_spire_storage_reindex_snapshot_sql (id, embedding) VALUES \
                 (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                 (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
                 (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)), \
                 (4, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42)); \
             CREATE INDEX ec_spire_storage_reindex_snapshot_idx \
                 ON ec_spire_storage_reindex_snapshot_sql USING ec_spire \
                 (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("relation storage REINDEX snapshot fixture should be created");

        let conninfo = current_pg_test_loopback_conninfo();
        let mut lock_client = postgres::Client::connect(&conninfo, postgres::NoTls)
            .expect("relation storage REINDEX lock client should connect");
        lock_client
            .batch_execute(
                "BEGIN; \
                 LOCK TABLE ec_spire_storage_reindex_snapshot_sql \
                 IN ACCESS EXCLUSIVE MODE",
            )
            .expect("relation storage REINDEX blocker should hold table lock");

        let reindex_conninfo = format!(
            "{conninfo} application_name=ec_spire_storage_snapshot_reindex"
        );
        let reindex_handle = std::thread::spawn(move || -> Result<(), String> {
            let mut client = postgres::Client::connect(&reindex_conninfo, postgres::NoTls)
                .map_err(|error| error.to_string())?;
            client
                .batch_execute("SET statement_timeout = '5s'")
                .map_err(|error| error.to_string())?;
            client
                .batch_execute("REINDEX INDEX ec_spire_storage_reindex_snapshot_idx")
                .map_err(|error| error.to_string())
        });

        let mut monitor_client = postgres::Client::connect(&conninfo, postgres::NoTls)
            .expect("relation storage REINDEX monitor client should connect");
        let wait_started = std::time::Instant::now();
        loop {
            let waiting = monitor_client
                .query_one(
                    "SELECT count(*)::bigint \
                       FROM pg_stat_activity \
                      WHERE application_name = 'ec_spire_storage_snapshot_reindex' \
                        AND wait_event_type = 'Lock'",
                    &[],
                )
                .expect("relation storage REINDEX wait probe should succeed")
                .try_get::<_, i64>(0)
                .expect("relation storage REINDEX wait count should decode");
            if waiting > 0 {
                break;
            }
            assert!(
                wait_started.elapsed() < std::time::Duration::from_secs(2),
                "timed out waiting for REINDEX to enter lock wait"
            );
            std::thread::sleep(std::time::Duration::from_millis(25));
        }

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_storage_reindex_snapshot_idx'::regclass)",
        )
        .expect("relation storage snapshot during REINDEX should succeed")
        .expect("relation storage snapshot during REINDEX should return a row");
        let tuple_count = Spi::get_one::<i64>(
            "SELECT relation_object_tuple_count FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_storage_reindex_snapshot_idx'::regclass)",
        )
        .expect("relation storage tuple count during REINDEX should succeed")
        .expect("relation storage tuple count during REINDEX should return a row");
        let cleanup_supported = Spi::get_one::<bool>(
            "SELECT physical_cleanup_supported FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_storage_reindex_snapshot_idx'::regclass)",
        )
        .expect("relation storage cleanup flag during REINDEX should succeed")
        .expect("relation storage cleanup flag during REINDEX should return a row");

        assert!(active_epoch > 0);
        assert!(tuple_count > 0);
        assert!(cleanup_supported);

        lock_client
            .batch_execute("ROLLBACK")
            .expect("relation storage REINDEX blocker should release table lock");
        reindex_handle
            .join()
            .expect("relation storage REINDEX thread should not panic")
            .expect("relation storage REINDEX should complete after blocker releases");

        let post_reindex_active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_relation_storage_snapshot(\
                 'ec_spire_storage_reindex_snapshot_idx'::regclass)",
        )
        .expect("relation storage snapshot after REINDEX should succeed")
        .expect("relation storage snapshot after REINDEX should return a row");
        assert!(post_reindex_active_epoch > 0);
    }
