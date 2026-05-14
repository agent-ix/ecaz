    #[pg_test]
    fn test_ec_spire_remote_search_tuple_payload_side_channel() {
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_sql (id, title, embedding) VALUES \
             (1, 'alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_tuple_payload_sql_idx \
             ON ec_spire_tuple_payload_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_tuple_payload_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_tuple_payload_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let payload_from = format!(
            "FROM ec_spire_remote_search_tuple_payload(\
             'ec_spire_tuple_payload_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY['id', 'title']::text[])",
            selected_pids[0], selected_pids[1],
        );
        let payload_count = Spi::get_one::<i64>(&format!("SELECT count(*) {payload_from}"))
            .expect("tuple payload count query should succeed")
            .expect("tuple payload count should exist");
        let key_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} WHERE payload_key = 'node_id_vec_id'"
        ))
        .expect("tuple payload key query should succeed")
        .expect("tuple payload key count should exist");
        let column_count =
            Spi::get_one::<i32>(&format!("SELECT min(payload_column_count) {payload_from}"))
                .expect("tuple payload column count query should succeed")
                .expect("tuple payload column count should exist");
        let exact_projection_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} \
             WHERE tuple_payload ? 'id' \
               AND tuple_payload ? 'title' \
               AND NOT tuple_payload ? 'embedding'"
        ))
        .expect("tuple payload projection query should succeed")
        .expect("tuple payload projection count should exist");
        let alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} \
             WHERE tuple_payload ->> 'id' = '1' \
               AND tuple_payload ->> 'title' = 'alpha'"
        ))
        .expect("tuple payload value query should succeed")
        .expect("tuple payload value count should exist");
        let missing_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} \
             WHERE tuple_payload_missing"
        ))
        .expect("tuple payload missing query should succeed")
        .expect("tuple payload missing count should exist");
        let ready_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) {payload_from} \
             WHERE status = 'ready'"
        ))
        .expect("tuple payload status query should succeed")
        .expect("tuple payload ready count should exist");

        assert_eq!(payload_count, 2);
        assert_eq!(key_count, payload_count);
        assert_eq!(column_count, 2);
        assert_eq!(exact_projection_count, payload_count);
        assert_eq!(alpha_count, 1);
        assert_eq!(missing_count, 0);
        assert_eq!(ready_count, payload_count);
    }

    #[pg_test]
    fn test_ec_spire_typed_tuple_payload_scalar_parity_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_typed_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_typed_sql (id, title, embedding) VALUES \
             (1, 'alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'beta', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_tuple_payload_typed_sql_idx \
             ON ec_spire_tuple_payload_typed_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_tuple_payload_typed_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_tuple_payload_typed_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let endpoint_args = format!(
            "'ec_spire_tuple_payload_typed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY['id', 'title']::text[]",
            selected_pids[0], selected_pids[1],
        );
        let json_alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload({endpoint_args}) \
              WHERE tuple_payload ->> 'id' = '1' \
                AND tuple_payload ->> 'title' = 'alpha' \
                AND NOT tuple_payload ? 'embedding' \
                AND status = 'ready'"
        ))
        .expect("JSON tuple payload parity query should succeed")
        .expect("JSON tuple payload parity count should exist");
        let typed_summary = Spi::get_one::<String>(&format!(
            "SELECT count(*)::text || '|' || \
                    min(payload_column_count)::text || '|' || \
                    count(*) FILTER (WHERE payload_key = 'node_id_vec_id')::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport = 'pg_binary_attr_v1')::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport_status = 'ready')::text || '|' || \
                    count(*) FILTER (WHERE status = 'ready')::text \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args})"
        ))
        .expect("typed tuple payload summary query should succeed")
        .expect("typed tuple payload summary should exist");
        let typed_alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args}) \
              WHERE payload_attnums = ARRAY[1, 2]::int2[] \
                AND payload_names = ARRAY['id', 'title']::text[] \
                AND payload_type_oids = ARRAY['int8'::regtype::oid, 'text'::regtype::oid]::oid[] \
                AND payload_typmods = ARRAY[-1, -1]::int4[] \
                AND payload_nulls = ARRAY[false, false]::boolean[] \
                AND payload_formats = ARRAY['pg_binary_attr_v1', 'pg_binary_attr_v1']::text[] \
                AND payload_values[1] = int8send(1::bigint)::bytea \
                AND payload_values[2] = textsend('alpha'::text)::bytea \
                AND NOT tuple_payload_missing \
                AND tuple_transport_status = 'ready' \
                AND status = 'ready'"
        ))
        .expect("typed tuple payload scalar value query should succeed")
        .expect("typed tuple payload scalar value count should exist");
        let empty_projection_args = format!(
            "'ec_spire_tuple_payload_typed_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY[]::text[]",
            selected_pids[0], selected_pids[1],
        );
        let empty_projection_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload_typed({empty_projection_args}) \
              WHERE payload_column_count = 0 \
                AND payload_attnums = ARRAY[]::int2[] \
                AND payload_names = ARRAY[]::text[] \
                AND payload_type_oids = ARRAY[]::oid[] \
                AND payload_typmods = ARRAY[]::int4[] \
                AND payload_collations = ARRAY[]::oid[] \
                AND payload_nulls = ARRAY[]::boolean[] \
                AND payload_values = ARRAY[]::bytea[] \
                AND payload_formats = ARRAY[]::text[] \
                AND NOT tuple_payload_missing \
                AND tuple_transport = 'pg_binary_attr_v1' \
                AND tuple_transport_status = 'ready' \
                AND status = 'ready'"
        ))
        .expect("typed tuple payload empty projection query should succeed")
        .expect("typed tuple payload empty projection count should exist");

        assert_eq!(json_alpha_count, 1);
        assert_eq!(typed_summary, "2|2|2|2|2|2");
        assert_eq!(typed_alpha_count, 1);
        assert_eq!(empty_projection_count, 2);
    }

    #[pg_test]
    fn test_ec_spire_typed_tuple_payload_null_array_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_typed_array_sql \
             (id bigint primary key, title text, tags text[] not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_typed_array_sql (id, title, tags, embedding) VALUES \
             (1, NULL, ARRAY['red', 'blue']::text[], encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'beta', ARRAY['green']::text[], encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_tuple_payload_typed_array_idx \
             ON ec_spire_tuple_payload_typed_array_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_tuple_payload_typed_array_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_tuple_payload_typed_array_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let endpoint_args = format!(
            "'ec_spire_tuple_payload_typed_array_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY['id', 'title', 'tags']::text[]",
            selected_pids[0], selected_pids[1],
        );
        let typed_summary = Spi::get_one::<String>(&format!(
            "SELECT count(*)::text || '|' || \
                    min(payload_column_count)::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport = 'pg_binary_attr_v1')::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport_status = 'ready')::text || '|' || \
                    count(*) FILTER (WHERE status = 'ready')::text \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args})"
        ))
        .expect("typed tuple payload summary query should succeed")
        .expect("typed tuple payload summary should exist");
        let alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args}) \
              WHERE payload_attnums = ARRAY[1, 2, 3]::int2[] \
                AND payload_names = ARRAY['id', 'title', 'tags']::text[] \
                AND payload_type_oids = ARRAY[\
                    'int8'::regtype::oid, \
                    'text'::regtype::oid, \
                    'text[]'::regtype::oid]::oid[] \
                AND payload_nulls = ARRAY[false, true, false]::boolean[] \
                AND payload_formats = ARRAY[\
                    'pg_binary_attr_v1', \
                    'pg_binary_attr_v1', \
                    'pg_binary_attr_v1']::text[] \
                AND payload_values[1] = int8send(1::bigint)::bytea \
                AND payload_values[2] = ''::bytea \
                AND payload_values[3] = array_send(ARRAY['red', 'blue']::text[])::bytea \
                AND NOT tuple_payload_missing \
                AND tuple_transport_status = 'ready' \
                AND status = 'ready'"
        ))
        .expect("typed tuple payload NULL/array query should succeed")
        .expect("typed tuple payload NULL/array count should exist");

        assert_eq!(typed_summary, "2|3|2|2|2");
        assert_eq!(alpha_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_typed_tuple_payload_domain_composite_sql() {
        Spi::run(
            "CREATE DOMAIN ec_spire_typed_label_domain AS text \
             CHECK (VALUE <> 'blocked')",
        )
        .expect("domain creation should succeed");
        Spi::run("CREATE TYPE ec_spire_typed_pair AS (code int4, label text)")
            .expect("composite type creation should succeed");
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_typed_record_sql \
             (id bigint primary key, \
              label ec_spire_typed_label_domain not null, \
              pair ec_spire_typed_pair not null, \
              embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_typed_record_sql \
             (id, label, pair, embedding) VALUES \
             (1, 'alpha'::ec_spire_typed_label_domain, \
              ROW(7, 'left')::ec_spire_typed_pair, \
              encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, 'beta'::ec_spire_typed_label_domain, \
              ROW(9, 'right')::ec_spire_typed_pair, \
              encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_tuple_payload_typed_record_idx \
             ON ec_spire_tuple_payload_typed_record_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_tuple_payload_typed_record_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_tuple_payload_typed_record_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        let endpoint_args = format!(
            "'ec_spire_tuple_payload_typed_record_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict', ARRAY['id', 'label', 'pair']::text[]",
            selected_pids[0], selected_pids[1],
        );
        let typed_summary = Spi::get_one::<String>(&format!(
            "SELECT count(*)::text || '|' || \
                    min(payload_column_count)::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport = 'pg_binary_attr_v1')::text || '|' || \
                    count(*) FILTER (WHERE tuple_transport_status = 'ready')::text || '|' || \
                    count(*) FILTER (WHERE status = 'ready')::text \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args})"
        ))
        .expect("typed tuple payload summary query should succeed")
        .expect("typed tuple payload summary should exist");
        let alpha_count = Spi::get_one::<i64>(&format!(
            "SELECT count(*) \
               FROM ec_spire_remote_search_tuple_payload_typed({endpoint_args}) \
              WHERE payload_attnums = ARRAY[1, 2, 3]::int2[] \
                AND payload_names = ARRAY['id', 'label', 'pair']::text[] \
                AND payload_type_oids = ARRAY[\
                    'int8'::regtype::oid, \
                    'ec_spire_typed_label_domain'::regtype::oid, \
                    'ec_spire_typed_pair'::regtype::oid]::oid[] \
                AND payload_nulls = ARRAY[false, false, false]::boolean[] \
                AND payload_formats = ARRAY[\
                    'pg_binary_attr_v1', \
                    'pg_binary_attr_v1', \
                    'pg_binary_attr_v1']::text[] \
                AND payload_values[1] = int8send(1::bigint)::bytea \
                AND payload_values[2] = textsend(\
                    'alpha'::ec_spire_typed_label_domain::text)::bytea \
                AND payload_values[3] = record_send(\
                    ROW(7, 'left')::ec_spire_typed_pair)::bytea \
                AND NOT tuple_payload_missing \
                AND tuple_transport_status = 'ready' \
                AND status = 'ready'"
        ))
        .expect("typed tuple payload domain/composite query should succeed")
        .expect("typed tuple payload domain/composite count should exist");

        assert_eq!(typed_summary, "2|3|2|2|2");
        assert_eq!(alpha_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_insert_tuple_payload_endpoint_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_insert_payload_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_insert_payload_sql_idx \
             ON ec_spire_remote_insert_payload_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let insert_status = Spi::get_one::<String>(
            "SELECT status || ':' || inserted_count::text || ':' || payload_column_count::text \
               FROM ec_spire_remote_insert_tuple_payload(\
                    'ec_spire_remote_insert_payload_sql_idx'::regclass, \
                    jsonb_build_object(\
                        'id', 101, \
                        'title', 'remote payload', \
                        'embedding', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)::text), \
                    ARRAY['id', 'title', 'embedding']::text[])",
        )
        .expect("remote insert tuple payload status query should succeed")
        .expect("remote insert tuple payload status should exist");
        let inserted_row = Spi::get_one::<String>(
            "SELECT id::text || ':' || title \
               FROM ec_spire_remote_insert_payload_sql \
              WHERE id = 101",
        )
        .expect("inserted row query should succeed")
        .expect("inserted row should exist");

        assert_eq!(insert_status, "ready:1:3");
        assert_eq!(inserted_row, "101:remote payload");
    }

    #[pg_test]
    fn test_ec_spire_remote_search_tuple_payload_missing_ctid_signal() {
        Spi::run(
            "CREATE TABLE ec_spire_tuple_payload_missing_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_tuple_payload_missing_sql (id, title, embedding) VALUES \
             (1, 'alpha', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        let table_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_tuple_payload_missing_sql'::regclass::oid",
        )
        .expect("table oid query should succeed")
        .expect("table oid should exist");
        let heap_relation_regclass = ec_spire_relation_regclass_text(table_oid)
            .expect("heap relation regclass lookup should succeed");
        let requested_columns = vec!["id".to_owned(), "title".to_owned()];
        let missing_ctid = "(999,1)".to_owned();

        let payloads = ec_spire_remote_search_tuple_payloads_for_ctids(
            &heap_relation_regclass,
            &requested_columns,
            &[missing_ctid.clone(), missing_ctid],
        )
        .expect("tuple payload batch fetch should succeed");
        assert_eq!(payloads.len(), 2);
        for (tuple_payload, tuple_payload_missing) in payloads {
            assert!(tuple_payload_missing);
            assert!(tuple_payload
                .0
                .as_object()
                .expect("missing tuple payload should be a JSON object")
                .is_empty());
        }
    }

    #[pg_test]
    fn test_ec_spire_remote_search_local_heap_degraded_skip_status() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_local_heap_degraded_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_local_heap_degraded_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_local_heap_degraded_sql_idx \
             ON ec_spire_remote_local_heap_degraded_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_local_heap_degraded_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_local_heap_degraded_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_local_heap_degraded_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe {
            am::debug_spire_rewrite_consistency_mode(index_oid, "degraded");
            am::debug_spire_rewrite_placement_state(index_oid, selected_pids[1] as u64, "skipped");
        }
        let args = format!(
            "'ec_spire_remote_local_heap_degraded_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'degraded'",
            selected_pids[0], selected_pids[1],
        );
        let merge_from = format!("FROM ec_spire_remote_search_merge_input_summary({args})");
        let final_from = format!("FROM ec_spire_remote_search_finalization_summary({args})");
        let heap_from = format!("FROM ec_spire_remote_search_heap_resolution_summary({args})");
        let candidate_from = format!("FROM ec_spire_remote_search_local_heap_candidates({args})");
        let candidate_summary_from =
            format!("FROM ec_spire_remote_search_local_heap_candidate_summary({args})");
        let result_summary_from =
            format!("FROM ec_spire_remote_search_coordinator_result_summary({args})");

        let merge_status = Spi::get_one::<String>(&format!("SELECT status {merge_from}"))
            .expect("degraded merge status query should succeed")
            .expect("degraded merge status should exist");
        let skipped_batch_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_batch_count {merge_from}"))
                .expect("degraded merge skipped query should succeed")
                .expect("degraded merge skipped count should exist");
        let local_batch_count =
            Spi::get_one::<i64>(&format!("SELECT local_batch_count {merge_from}"))
                .expect("degraded merge local query should succeed")
                .expect("degraded merge local count should exist");
        let final_status = Spi::get_one::<String>(&format!("SELECT status {final_from}"))
            .expect("degraded final status query should succeed")
            .expect("degraded final status should exist");
        let final_heap_fetch_status =
            Spi::get_one::<String>(&format!("SELECT final_heap_fetch_status {final_from}"))
                .expect("degraded final heap status query should succeed")
                .expect("degraded final heap status should exist");
        let heap_status = Spi::get_one::<String>(&format!("SELECT status {heap_from}"))
            .expect("degraded heap status query should succeed")
            .expect("degraded heap status should exist");
        let local_heap_resolution_status =
            Spi::get_one::<String>(&format!("SELECT local_heap_resolution_status {heap_from}"))
                .expect("degraded local heap resolution query should succeed")
                .expect("degraded local heap resolution status should exist");
        let decoded_local_locator_count =
            Spi::get_one::<i64>(&format!("SELECT decoded_local_locator_count {heap_from}"))
                .expect("degraded decoded locator query should succeed")
                .expect("degraded decoded locator count should exist");
        let candidate_count = Spi::get_one::<i64>(&format!("SELECT count(*) {candidate_from}"))
            .expect("degraded local candidate count query should succeed")
            .expect("degraded local candidate count should exist");
        let returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {candidate_summary_from}"
        ))
        .expect("degraded candidate summary count query should succeed")
        .expect("degraded candidate summary count should exist");
        let candidate_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {candidate_summary_from}"))
                .expect("degraded candidate summary status query should succeed")
                .expect("degraded candidate summary status should exist");
        let result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {result_summary_from}"))
                .expect("degraded result source query should succeed")
                .expect("degraded result source should exist");
        let result_receive_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_receive_count {result_summary_from}"))
                .expect("degraded result receive count query should succeed")
                .expect("degraded result receive count should exist");
        let result_receive_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_receive_status {result_summary_from}"
        ))
        .expect("degraded result receive status query should succeed")
        .expect("degraded result receive status should exist");
        let result_status = Spi::get_one::<String>(&format!("SELECT status {result_summary_from}"))
            .expect("degraded result status query should succeed")
            .expect("degraded result status should exist");
        let result_skipped_pid_count =
            Spi::get_one::<i64>(&format!("SELECT skipped_pid_count {result_summary_from}"))
                .expect("degraded result skipped pid query should succeed")
                .expect("degraded result skipped pid count should exist");

        assert_eq!(merge_status, "degraded_ready");
        assert_eq!(skipped_batch_count, 1);
        assert_eq!(local_batch_count, 1);
        assert_eq!(final_status, "degraded_ready");
        assert_eq!(final_heap_fetch_status, "local_ready");
        assert_eq!(heap_status, "degraded_ready");
        assert_eq!(local_heap_resolution_status, "ready");
        assert_eq!(decoded_local_locator_count, 1);
        assert_eq!(candidate_count, 1);
        assert_eq!(returned_candidate_count, 1);
        assert_eq!(candidate_summary_status, "degraded_ready");
        assert_eq!(result_source, "local_heap_candidates");
        assert_eq!(result_receive_count, 0);
        assert_eq!(result_receive_status, "ready");
        assert_eq!(result_status, "degraded_ready");
        assert_eq!(result_skipped_pid_count, 1);
    }

    #[pg_test]
    fn test_ec_spire_remote_heap_resolution_summary_blocks_remote() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_heap_res_summary_sql \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_heap_res_summary_sql (id, embedding) VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_heap_res_summary_sql_idx \
             ON ec_spire_remote_heap_res_summary_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 2)",
        )
        .expect("ec_spire index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_spire_remote_heap_res_summary_sql_idx'::regclass::oid",
        )
        .expect("index oid query should succeed")
        .expect("index oid should exist");
        let active_epoch = Spi::get_one::<i64>(
            "SELECT active_epoch FROM \
             ec_spire_index_hierarchy_snapshot('ec_spire_remote_heap_res_summary_sql_idx'::regclass)",
        )
        .expect("hierarchy snapshot query should succeed")
        .expect("active epoch should exist");
        let selected_pids = Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(leaf_pid ORDER BY leaf_pid) FROM \
             ec_spire_index_leaf_snapshot('ec_spire_remote_heap_res_summary_sql_idx'::regclass)",
        )
        .expect("leaf snapshot query should succeed")
        .expect("leaf pids should exist");
        assert_eq!(selected_pids.len(), 2);

        unsafe { am::debug_spire_rewrite_placement_node(index_oid, selected_pids[1] as u64, 2) };
        let summary_from = format!(
            "FROM ec_spire_remote_search_heap_resolution_summary(\
             'ec_spire_remote_heap_res_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let candidate_summary_from = format!(
            "FROM ec_spire_remote_search_local_heap_candidate_summary(\
             'ec_spire_remote_heap_res_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let result_summary_from = format!(
            "FROM ec_spire_remote_search_coordinator_result_summary(\
             'ec_spire_remote_heap_res_summary_sql_idx'::regclass, \
             {active_epoch}, ARRAY[1.0, 0.0]::real[], \
             ARRAY[{}, {}]::bigint[], 2, 'strict')",
            selected_pids[0], selected_pids[1],
        );
        let status = Spi::get_one::<String>(&format!("SELECT status {summary_from}"))
            .expect("remote heap summary status query should succeed")
            .expect("remote heap summary status should exist");
        let remote_plan_count =
            Spi::get_one::<i64>(&format!("SELECT remote_plan_count {summary_from}"))
                .expect("remote heap summary remote plan query should succeed")
                .expect("remote heap summary remote plan count should exist");
        let remote_pid_count =
            Spi::get_one::<i64>(&format!("SELECT remote_pid_count {summary_from}"))
                .expect("remote heap summary remote pid query should succeed")
                .expect("remote heap summary remote pid count should exist");
        let decoded_local_locator_count = Spi::get_one::<i64>(&format!(
            "SELECT decoded_local_locator_count {summary_from}"
        ))
        .expect("remote heap summary decoded count query should succeed")
        .expect("remote heap summary decoded count should exist");
        let local_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT local_heap_resolution_status {summary_from}"
        ))
        .expect("remote heap summary local status query should succeed")
        .expect("remote heap summary local status should exist");
        let remote_resolution_status = Spi::get_one::<String>(&format!(
            "SELECT remote_heap_resolution_status {summary_from}"
        ))
        .expect("remote heap summary remote status query should succeed")
        .expect("remote heap summary remote status should exist");
        let candidate_summary_status =
            Spi::get_one::<String>(&format!("SELECT status {candidate_summary_from}"))
                .expect("remote heap candidate summary status query should succeed")
                .expect("remote heap candidate summary status should exist");
        let returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {candidate_summary_from}"
        ))
        .expect("remote heap candidate summary return query should succeed")
        .expect("remote heap candidate summary return count should exist");
        let result_source =
            Spi::get_one::<String>(&format!("SELECT result_source {result_summary_from}"))
                .expect("remote result source query should succeed")
                .expect("remote result source should exist");
        let result_receive_count =
            Spi::get_one::<i64>(&format!("SELECT libpq_receive_count {result_summary_from}"))
                .expect("remote result receive count query should succeed")
                .expect("remote result receive count should exist");
        let result_receive_status = Spi::get_one::<String>(&format!(
            "SELECT libpq_receive_status {result_summary_from}"
        ))
        .expect("remote result receive status query should succeed")
        .expect("remote result receive status should exist");
        let result_next_blocker =
            Spi::get_one::<String>(&format!("SELECT next_blocker {result_summary_from}"))
                .expect("remote result blocker query should succeed")
                .expect("remote result blocker should exist");
        let result_returned_candidate_count = Spi::get_one::<i64>(&format!(
            "SELECT returned_candidate_count {result_summary_from}"
        ))
        .expect("remote result returned count query should succeed")
        .expect("remote result returned count should exist");

        assert_eq!(status, "requires_remote_node_descriptor");
        assert_eq!(remote_plan_count, 1);
        assert_eq!(remote_pid_count, 1);
        assert_eq!(decoded_local_locator_count, 0);
        assert_eq!(local_resolution_status, "planned");
        assert_eq!(remote_resolution_status, "requires_remote_node_descriptor");
        assert_eq!(candidate_summary_status, "requires_remote_node_descriptor");
        assert_eq!(returned_candidate_count, 0);
        assert_eq!(result_source, "blocked");
        assert_eq!(result_receive_count, 1);
        assert_eq!(result_receive_status, "requires_remote_node_descriptor");
        assert_eq!(result_next_blocker, "remote_node_descriptor");
        assert_eq!(result_returned_candidate_count, 0);
    }

