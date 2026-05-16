    #[pg_test]
    fn test_ec_spire_placement_directory_catalog_sql() {
        let table_exists =
            Spi::get_one::<bool>("SELECT to_regclass('ec_spire_placement') IS NOT NULL")
                .expect("placement directory table lookup should succeed")
                .expect("placement directory table lookup should return a row");
        let identity_index_exists = Spi::get_one::<bool>(
            "SELECT to_regclass('ec_spire_placement_by_identity') IS NOT NULL",
        )
        .expect("placement identity index lookup should succeed")
        .expect("placement identity index lookup should return a row");
        let index_oid_index_exists = Spi::get_one::<bool>(
            "SELECT to_regclass('ec_spire_placement_by_index_oid') IS NOT NULL",
        )
        .expect("placement index_oid index lookup should succeed")
        .expect("placement index_oid index lookup should return a row");
        let primary_key_columns = Spi::get_one::<String>(
            "SELECT string_agg(a.attname, ',' ORDER BY k.ord) \
               FROM pg_index i \
               JOIN pg_class c ON c.oid = i.indrelid \
               JOIN unnest(i.indkey) WITH ORDINALITY AS k(attnum, ord) ON true \
               JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = k.attnum \
              WHERE c.relname = 'ec_spire_placement' AND i.indisprimary",
        )
        .expect("placement primary key query should succeed")
        .expect("placement primary key columns should exist");
        let source_identity_check_count = Spi::get_one::<i64>(
            "SELECT count(*) \
               FROM pg_constraint \
              WHERE conrelid = 'ec_spire_placement'::regclass \
                AND contype = 'c' \
                AND pg_get_constraintdef(oid) LIKE '%octet_length(source_identity) = 16%'",
        )
        .expect("placement source identity check query should succeed")
        .expect("placement source identity check count should exist");
        let node_id_check_count = Spi::get_one::<i64>(
            "SELECT count(*) \
               FROM pg_constraint \
              WHERE conrelid = 'ec_spire_placement'::regclass \
                AND contype = 'c' \
                AND pg_get_constraintdef(oid) LIKE '%node_id >= 0%'",
        )
        .expect("placement node id check query should succeed")
        .expect("placement node id check count should exist");

        Spi::run(
            "INSERT INTO ec_spire_placement \
             (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES \
                 ('4294967291'::oid, decode('01', 'hex'), 2, 7, 5, \
                  decode('000102030405060708090a0b0c0d0e0f', 'hex')), \
                 ('4294967291'::oid, decode('02', 'hex'), 0, 7, 5, \
                  decode('101112131415161718191a1b1c1d1e1f', 'hex'))",
        )
        .expect("placement directory insert should succeed");
        let stored_row = Spi::get_one::<String>(
            "SELECT string_agg( \
                    encode(pk_value, 'hex') || ':' || node_id::text || ':' || \
                    centroid_id::text || ':' || served_epoch::text, \
                    ',' ORDER BY pk_value) \
               FROM ec_spire_placement \
              WHERE index_oid = '4294967291'::oid",
        )
        .expect("placement directory stored row query should succeed")
        .expect("placement directory stored row should exist");

        assert!(table_exists);
        assert!(identity_index_exists);
        assert!(index_oid_index_exists);
        assert_eq!(primary_key_columns, "index_oid,pk_value");
        assert_eq!(source_identity_check_count, 1);
        assert_eq!(node_id_check_count, 1);
        assert_eq!(stored_row, "01:2:7:5,02:0:7:5");
    }
    #[pg_test]
    fn test_ec_spire_placement_index_oid_lookup_uses_index_sql() {
        Spi::run(
            "INSERT INTO ec_spire_placement \
                 (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             SELECT (4294960000 - (g % 97))::oid, \
                    int8send(g::bigint)::bytea, \
                    2, 7, 5, decode(md5(g::text), 'hex') \
               FROM generate_series(1, 512) AS g",
        )
        .expect("unrelated placement rows should insert");
        Spi::run(
            "INSERT INTO ec_spire_placement \
                 (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
             VALUES ('4294967285'::oid, int8send(9999::bigint)::bytea, \
                     2, 7, 5, decode('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'hex'))",
        )
        .expect("target placement row should insert");
        Spi::run("SET LOCAL enable_seqscan = off").expect("disabling seqscan should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT 1 FROM ec_spire_placement \
                      WHERE index_oid = '4294967285'::oid \
                      LIMIT 1",
                    None,
                    &[],
                )
                .expect("placement index_oid lookup explain should succeed");
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("placement lookup plan row should decode")
                        .expect("placement lookup plan row should not be NULL"),
                );
            }
            lines.join("\n")
        });
        let target_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_placement \
              WHERE index_oid = '4294967285'::oid",
        )
        .expect("target placement count should succeed")
        .expect("target placement count should exist");

        assert_eq!(target_count, 1);
        assert!(
            plan.contains("ec_spire_placement_by_index_oid"),
            "planner should use bounded index_oid lookup, got:\n{plan}"
        );
    }

    #[pg_test]
    fn test_ec_spire_register_placement_batch_sql() {
        let empty_registered_count = Spi::get_one::<i64>(
            "SELECT ec_spire_register_placement_batch( \
                 '4294967289'::oid, \
                 ARRAY[]::ec_spire_placement_entry[] \
             )",
        )
        .expect("empty placement batch registration should succeed")
        .expect("empty placement batch registration count should exist");
        let registered_count = Spi::get_one::<i64>(
            "SELECT ec_spire_register_placement_batch( \
                 '4294967290'::oid, \
                 ARRAY[ \
                   ROW(decode('01', 'hex'), 2, 7, 5, \
                       decode('000102030405060708090a0b0c0d0e0f', 'hex'))::ec_spire_placement_entry, \
                   ROW(decode('02', 'hex'), 0, 8, 5, \
                       decode('101112131415161718191a1b1c1d1e1f', 'hex'))::ec_spire_placement_entry \
                 ] \
             )",
        )
        .expect("placement batch registration should succeed")
        .expect("placement batch registration count should exist");
        let stored_rows = Spi::get_one::<String>(
            "SELECT string_agg( \
                    encode(pk_value, 'hex') || ':' || node_id::text || ':' || \
                    centroid_id::text || ':' || served_epoch::text || ':' || \
                    encode(source_identity, 'hex'), \
                    ',' ORDER BY pk_value) \
               FROM ec_spire_placement \
              WHERE index_oid = '4294967290'::oid",
        )
        .expect("placement batch rows query should succeed")
        .expect("placement batch rows should exist");

        assert_eq!(empty_registered_count, 0);
        assert_eq!(registered_count, 2);
        assert_eq!(
            stored_rows,
            "01:2:7:5:000102030405060708090a0b0c0d0e0f,\
             02:0:8:5:101112131415161718191a1b1c1d1e1f"
        );
    }

    #[pg_test]
    #[should_panic(expected = "entries[1] is NULL")]
    fn test_ec_spire_register_placement_batch_rejects_null_entry_sql() {
        Spi::run(
            "SELECT ec_spire_register_placement_batch( \
                 '4294967288'::oid, \
                 ARRAY[ \
                   NULL::ec_spire_placement_entry, \
                   ROW(decode('01', 'hex'), 2, 7, 5, \
                       decode('000102030405060708090a0b0c0d0e0f', 'hex'))::ec_spire_placement_entry \
                 ] \
             )",
        )
        .expect("placement batch registration should reject null entries");
    }

    #[pg_test]
    #[should_panic(expected = "ec_spire_placement_pkey")]
    fn test_ec_spire_register_placement_batch_rejects_duplicate_pk_sql() {
        Spi::run(
            "SELECT ec_spire_register_placement_batch( \
                 '4294967287'::oid, \
                 ARRAY[ \
                   ROW(decode('01', 'hex'), 2, 7, 5, \
                       decode('000102030405060708090a0b0c0d0e0f', 'hex'))::ec_spire_placement_entry, \
                   ROW(decode('01', 'hex'), 3, 8, 5, \
                       decode('101112131415161718191a1b1c1d1e1f', 'hex'))::ec_spire_placement_entry \
                 ] \
             )",
        )
        .expect("placement batch registration should reject duplicate primary keys");
    }

    #[pg_test]
    #[should_panic(expected = "ec_spire_placement_source_identity_check")]
    fn test_ec_spire_register_placement_batch_rejects_invalid_sql() {
        Spi::run(
            "SELECT ec_spire_register_placement_batch( \
                 '4294967286'::oid, \
                 ARRAY[ \
                   ROW(decode('01', 'hex'), 2, 7, 5, decode('0001', 'hex')) \
                       ::ec_spire_placement_entry \
                 ] \
             )",
        )
        .expect("placement batch registration should reject invalid entries");
    }

    #[pg_test]
    fn test_ec_spire_placement_snapshot_sql() {
        Spi::run("CREATE TABLE ec_spire_place_sql (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_place_empty_idx ON ec_spire_place_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops)",
        )
        .expect("empty ec_spire index creation should succeed");
        let empty_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_empty_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("count should exist");
        assert_eq!(empty_rows, 0);

        Spi::run("DROP INDEX ec_spire_place_empty_idx").expect("drop index should succeed");
        Spi::run(
            "INSERT INTO ec_spire_place_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)), \
             (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_place_sql_idx ON ec_spire_place_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("populated ec_spire index creation should succeed");

        let row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("count should exist");
        let placement_count = Spi::get_one::<i64>(
            "SELECT placement_count FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");
        let root_object_count = Spi::get_one::<i64>(
            "SELECT root_object_count FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");
        let leaf_object_count = Spi::get_one::<i64>(
            "SELECT leaf_object_count FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");
        let assignment_count = Spi::get_one::<i64>(
            "SELECT assignment_count FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");
        let available_object_bytes = Spi::get_one::<i64>(
            "SELECT available_object_bytes FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");
        let placement_object_bytes = Spi::get_one::<i64>(
            "SELECT placement_object_bytes FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");

        assert_eq!(row_count, 1);
        assert_eq!(placement_count, 3);
        assert_eq!(root_object_count, 1);
        assert_eq!(leaf_object_count, 2);
        assert_eq!(assignment_count, 3);
        assert!(available_object_bytes > 0);
        assert_eq!(placement_object_bytes, available_object_bytes);

        Spi::run(
            "INSERT INTO ec_spire_place_sql (id, embedding) VALUES \
             (4, encode_to_ecvector(ARRAY[0.9, 0.1], 4, 42))",
        )
        .expect("post-build insert should publish delta placement");

        let post_insert_placement_count = Spi::get_one::<i64>(
            "SELECT placement_count FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");
        let post_insert_delta_object_count = Spi::get_one::<i64>(
            "SELECT delta_object_count FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");
        let post_insert_assignment_count = Spi::get_one::<i64>(
            "SELECT assignment_count FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");
        let delta_object_bytes = Spi::get_one::<i64>(
            "SELECT delta_object_bytes FROM \
             ec_spire_index_placement_snapshot('ec_spire_place_sql_idx'::regclass)",
        )
        .expect("placement query should succeed")
        .expect("placement row should exist");

        assert_eq!(post_insert_placement_count, 4);
        assert_eq!(post_insert_delta_object_count, 1);
        assert_eq!(post_insert_assignment_count, 4);
        assert!(delta_object_bytes > 0);
    }

    #[pg_test]
    fn test_ec_spire_selected_pid_placement_snapshot_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_selected_pid_place_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_selected_pid_place_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_selected_pid_place_idx \
             ON ec_spire_selected_pid_place_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_spire_selected_pid_place_idx'::regclass::oid")
                .expect("index oid query should succeed")
                .expect("index oid should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_selected_pid_place_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pid array should exist");
        assert_eq!(selected_pids.len(), 2);
        unsafe {
            am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2);
        }

        let placement_map = Spi::get_one::<String>(&format!(
            "SELECT string_agg(\
                 pid::text || ':' || node_id::text || ':' || local_store_id::text || ':' || placement_state, \
                 ',' ORDER BY selection_ordinal) \
             FROM ec_spire_index_selected_pid_placement_snapshot(\
                 'ec_spire_selected_pid_place_idx'::regclass, ARRAY[{}, {}]::bigint[])",
            selected_pids[0], selected_pids[1]
        ))
        .expect("selected PID placement snapshot query should succeed")
        .expect("selected PID placement map should exist");
        let object_bytes_positive = Spi::get_one::<bool>(&format!(
            "SELECT bool_and(object_bytes > 0) \
             FROM ec_spire_index_selected_pid_placement_snapshot(\
                 'ec_spire_selected_pid_place_idx'::regclass, ARRAY[{}, {}]::bigint[])",
            selected_pids[0], selected_pids[1]
        ))
        .expect("selected PID placement object bytes query should succeed")
        .expect("selected PID placement object bytes result should exist");

        assert_eq!(
            placement_map,
            format!(
                "{}:0:0:available,{}:2:2:available",
                selected_pids[0], selected_pids[1]
            )
        );
        assert!(object_bytes_positive);
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_ec_spire_placement_write_contention_distinct_pk_dml() {
        const TABLE_NAME: &str = "ec_spire_placement_contention_dml";
        const INDEX_NAME: &str = "ec_spire_placement_contention_dml_idx";
        // Test-unique advisory-lock id; conventionally `<review-packet>0`.
        const BARRIER_KEY: i64 = 309_690;
        const WRITER_COUNT: usize = 8;
        const P99_THRESHOLD: Duration = Duration::from_secs(20);

        let connection = pg_test_psql_connection();
        run_psql_script(
            &connection,
            "ec_spire placement contention setup",
            &format!(
                "DROP TABLE IF EXISTS {TABLE_NAME};
                 CREATE TABLE {TABLE_NAME} (id bigint primary key, embedding ecvector);
                 INSERT INTO {TABLE_NAME} (id, embedding)
                 SELECT gs, encode_to_ecvector(
                     ARRAY[1.0, (gs::real / 100.0)]::real[], 4, 42)
                   FROM generate_series(1, {WRITER_COUNT}) AS gs;
                 CREATE INDEX {INDEX_NAME} ON {TABLE_NAME} USING ec_spire
                   (embedding ecvector_spire_ip_ops)
                   WITH (nlists = 1, nprobe = 1, training_sample_rows = {WRITER_COUNT});
                 WITH active AS (
                     SELECT active_epoch
                       FROM ec_spire_index_hierarchy_snapshot('{INDEX_NAME}'::regclass)
                 )
                 INSERT INTO ec_spire_placement
                     (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity)
                 SELECT '{INDEX_NAME}'::regclass,
                        int8send(gs::bigint)::bytea,
                        0,
                        2,
                        active.active_epoch,
                        decode(md5(gs::text), 'hex')
                   FROM generate_series(1, {WRITER_COUNT}) AS gs
                   CROSS JOIN active;",
            ),
        );
        let deadlocks_before = Spi::get_one::<i64>(
            "SELECT deadlocks FROM pg_stat_database WHERE datname = current_database()",
        )
        .expect("deadlock stats query should succeed")
        .expect("deadlock stats should exist");
        let active_epoch = Spi::get_one::<i64>(&format!(
            "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('{INDEX_NAME}'::regclass)"
        ))
        .expect("active epoch query should succeed")
        .expect("active epoch should exist");

        Spi::run(&format!("SELECT pg_advisory_lock({BARRIER_KEY})"))
            .expect("barrier lock should be acquired");
        let mut workers = Vec::with_capacity(WRITER_COUNT);
        for worker_idx in 0..WRITER_COUNT {
            let delete_id = i64::try_from(worker_idx + 1).expect("worker index should fit i64");
            let insert_id = 101_i64 + i64::try_from(worker_idx).expect("worker index fits i64");
            let inserted_tail = (worker_idx + 1) as f32 / 10.0;
            // Keep app-table and placement writes in one transaction: that is
            // the coordinator INSERT trigger production shape this fixture
            // is meant to pressure.
            let worker_sql = format!(
                "SET lock_timeout = '10s';
                 SET statement_timeout = '30s';
                 SELECT pg_advisory_lock_shared({BARRIER_KEY});
                 SELECT pg_advisory_unlock_shared({BARRIER_KEY});
                 BEGIN;
                 INSERT INTO {TABLE_NAME} (id, embedding)
                 VALUES ({insert_id}, encode_to_ecvector(
                     ARRAY[1.0, {inserted_tail}]::real[], 4, 42));
                 INSERT INTO ec_spire_placement
                     (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity)
                 VALUES ('{INDEX_NAME}'::regclass,
                         int8send({insert_id}::bigint)::bytea,
                         0,
                         2,
                         {active_epoch},
                         decode(md5({insert_id}::text), 'hex'));
                 DO $worker$
                 DECLARE row_count bigint;
                 BEGIN
                     DELETE FROM {TABLE_NAME} WHERE id = {delete_id};
                     GET DIAGNOSTICS row_count = ROW_COUNT;
                     IF row_count != 1 THEN
                         RAISE EXCEPTION 'unexpected placement contention delete row count %',
                             row_count;
                     END IF;
                 END
                 $worker$;
                 DELETE FROM ec_spire_placement
                  WHERE index_oid = '{INDEX_NAME}'::regclass
                    AND pk_value = int8send({delete_id}::bigint)::bytea;
                 COMMIT;",
            );
            let label = format!("spire placement contention worker {}", worker_idx + 1);
            let child = spawn_psql_script(&connection, &label, &worker_sql);
            workers.push((label, child));
        }
        wait_for_advisory_lock_waiters(BARRIER_KEY, WRITER_COUNT as i64);
        let released_at = Instant::now();
        Spi::run(&format!("SELECT pg_advisory_unlock({BARRIER_KEY})"))
            .expect("barrier lock should be released");

        let handles = workers
            .into_iter()
            .map(|(label, child)| {
                std::thread::spawn(move || {
                    let output = child
                        .wait_with_output()
                        .unwrap_or_else(|e| panic!("{label} wait failed: {e}"));
                    (label, released_at.elapsed(), output)
                })
            })
            .collect::<Vec<_>>();
        let mut durations = Vec::with_capacity(WRITER_COUNT);
        for handle in handles {
            let (label, duration, output) = handle
                .join()
                .expect("placement contention worker join should succeed");
            assert_psql_success(&label, output);
            durations.push(duration);
        }
        durations.sort_unstable();
        let p99 = durations[durations.len() - 1];
        let point_lookup_count = |ids: std::ops::RangeInclusive<i64>| -> i64 {
            ids.map(|id| {
                Spi::get_one::<i64>(&format!(
                    "SELECT count(*) FROM {TABLE_NAME} WHERE id = {id}"
                ))
                .expect("point lookup count query should succeed")
                .expect("point lookup count should exist")
            })
            .sum()
        };

        let inserted_count = point_lookup_count(101..=108);
        let placement_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM ec_spire_placement WHERE index_oid = '{INDEX_NAME}'::regclass"
        ))
        .expect("placement count query should succeed")
        .expect("placement count should exist");
        let placement_waiter_count = Spi::get_one::<i64>(
            "SELECT count(*)::bigint
               FROM pg_locks
              WHERE relation = 'ec_spire_placement'::regclass
                AND NOT granted",
        )
        .expect("placement waiter count query should succeed")
        .expect("placement waiter count should exist");
        let deadlocks_after = Spi::get_one::<i64>(
            "SELECT deadlocks FROM pg_stat_database WHERE datname = current_database()",
        )
        .expect("post deadlock stats query should succeed")
        .expect("post deadlock stats should exist");

        assert_eq!(inserted_count, WRITER_COUNT as i64);
        assert_eq!(placement_count, WRITER_COUNT as i64);
        assert_eq!(placement_waiter_count, 0);
        assert_eq!(deadlocks_after, deadlocks_before);
        assert!(
            p99 <= P99_THRESHOLD,
            "placement contention p99 {:?} exceeded {:?}; samples={durations:?}",
            p99,
            P99_THRESHOLD
        );
    }
