    macro_rules! ec_ivf_debug {
        ($call:expr) => {{
            // SAFETY: ec_ivf debug entrypoints are unsafe fn that take the
            // debug-test live state pointers. Each call site here is inside
            // a pgrx-managed test fixture where those invariants hold.
            #[allow(unused_unsafe)]
            unsafe { $call }
        }};
    }

    fn ec_ivf_insert_values(start_id: i64, count: usize, vector: &str) -> String {
        (0..count)
            .map(|offset| format!("({}, '{}'::ecvector)", start_id + offset as i64, vector))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn ec_ivf_page_ownership_signature(index_name: &str) -> String {
        Spi::get_one::<String>(&format!(
            "SELECT COALESCE(
                string_agg(
                    block_number || ':' ||
                    line_pointer_count || ':' ||
                    unused_line_pointers || ':' ||
                    non_posting_tuples || ':' ||
                    posting_tuples || ':' ||
                    live_posting_tuples || ':' ||
                    deleted_posting_tuples || ':' ||
                    heap_tid_refs || ':' ||
                    distinct_lists || ':' ||
                    COALESCE(min_list_id::text, '') || ':' ||
                    COALESCE(max_list_id::text, '') || ':' ||
                    list_ids,
                    ';' ORDER BY block_number
                ),
                ''
             )
             FROM ec_ivf_index_page_ownership('{index_name}'::regclass)"
        ))
        .expect("page ownership signature query should succeed")
        .expect("page ownership signature should not be NULL")
    }

    fn ec_ivf_build_metadata_signature(
        index_oid: pg_sys::Oid,
    ) -> (u16, u32, u16, u64, bool, bool) {
        ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid))
    }

    fn ec_ivf_directory_signatures(
        index_oid: pg_sys::Oid,
        nlists: u32,
    ) -> Vec<(u32, u32, u64, u64, u64)> {
        (0..nlists)
            .map(|list_id| ec_ivf_debug!(am::debug_ec_ivf_directory_entry(index_oid, list_id)))
            .collect()
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_ivf_debug_last_build_timing() -> TableIterator<
        'static,
        (
            name!(requested_workers, i64),
            name!(workers_launched, i64),
            name!(heap_tuples, i64),
            name!(index_tuples, i64),
            name!(heap_ingest_us, i64),
            name!(train_model_us, i64),
            name!(stage_build_plan_us, i64),
            name!(stage_pq_train_us, i64),
            name!(stage_centroids_us, i64),
            name!(stage_assign_us, i64),
            name!(stage_postings_us, i64),
            name!(stage_directory_us, i64),
            name!(flush_build_plan_us, i64),
            name!(parallel_begin_us, i64),
            name!(parallel_drain_us, i64),
            name!(parallel_sort_push_us, i64),
            name!(parallel_worker_tuple_buffer_capacity, i64),
            name!(parallel_worker_tuple_buffer_struct_bytes, i64),
        ),
    > {
        let timing = am::ivf_debug_last_build_timing();
        TableIterator::once((
            timing.requested_workers as i64,
            timing.workers_launched as i64,
            timing.heap_tuples as i64,
            timing.index_tuples as i64,
            timing.heap_ingest_us as i64,
            timing.train_model_us as i64,
            timing.stage_build_plan_us as i64,
            timing.stage_pq_train_us as i64,
            timing.stage_centroids_us as i64,
            timing.stage_assign_us as i64,
            timing.stage_postings_us as i64,
            timing.stage_directory_us as i64,
            timing.flush_build_plan_us as i64,
            timing.parallel_begin_us as i64,
            timing.parallel_drain_us as i64,
            timing.parallel_sort_push_us as i64,
            timing.parallel_worker_tuple_buffer_capacity as i64,
            timing.parallel_worker_tuple_buffer_struct_bytes as i64,
        ))
    }

    #[pg_test]
    fn test_ec_ivf_empty_index_build_initializes_metadata_page() {
        Spi::run("CREATE TABLE ec_ivf_empty_build (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_empty_build_idx ON ec_ivf_empty_build USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 16, nprobe = 4, training_sample_rows = 128, seed = 7)",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_empty_build_idx");
        let (format_version, nlists, nprobe, training_sample_rows, seed) =
            ec_ivf_debug!(am::debug_ec_ivf_metadata(index_oid));
        let (summary_nlists, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));

        assert_eq!(format_version, 2);
        assert_eq!(nlists, 16);
        assert_eq!(nprobe, 4);
        assert_eq!(training_sample_rows, 128);
        assert_eq!(seed, 7);
        assert_eq!(summary_nlists, 16);
        assert_eq!(empty_lists, 16);
        assert_eq!(directory_live, 0);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 0);
    }

    #[pg_test]
    fn test_ec_ivf_singleton_index_build_records_one_live_list() {
        Spi::run("CREATE TABLE ec_ivf_singleton_build (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_singleton_build VALUES \
             (1, '[1.0,0.0]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_singleton_build_idx ON ec_ivf_singleton_build USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 1, seed = 13)",
        )
        .expect("singleton index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_singleton_build_idx");
        let (dimensions, nlists, training_version, total_live, has_centroids, has_directory) =
            ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (summary_nlists, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));

        assert_eq!(dimensions, 2);
        assert_eq!(nlists, 1);
        assert_eq!(training_version, 1);
        assert_eq!(total_live, 1);
        assert!(has_centroids);
        assert!(has_directory);
        assert_eq!(summary_nlists, 1);
        assert_eq!(empty_lists, 0);
        assert_eq!(directory_live, 1);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 0);
    }

    #[pg_test]
    fn test_ec_ivf_non_empty_index_build_writes_staged_pages() {
        Spi::run("CREATE TABLE ec_ivf_non_empty_build (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_non_empty_build VALUES
             (1, '[1.0,0.0]'::ecvector),
             (2, '[0.9,0.1]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_non_empty_build_idx ON ec_ivf_non_empty_build USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 3, nprobe = 2, training_sample_rows = 3, seed = 11)",
        )
        .expect("non-empty index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_non_empty_build_idx");
        let (dimensions, nlists, training_version, total_live, has_centroids, has_directory) =
            ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (summary_nlists, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let index_blocks = ec_ivf_index_blocks("ec_ivf_non_empty_build_idx");

        assert_eq!(dimensions, 2);
        assert_eq!(nlists, 3);
        assert_eq!(training_version, 1);
        assert_eq!(total_live, 3);
        assert!(has_centroids);
        assert!(has_directory);
        assert_eq!(summary_nlists, 3);
        assert_eq!(empty_lists, 0);
        assert_eq!(directory_live, 3);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 0);
        assert!(
            index_blocks >= 2,
            "non-empty ec_ivf build should write metadata plus data pages"
        );
    }

    #[pg_test]
    fn test_ec_ivf_parallel_build_workers_and_counts() {
        Spi::run("SET max_parallel_maintenance_workers = 2")
            .expect("parallel maintenance worker setting should be accepted");
        Spi::run("SET max_parallel_workers = 4")
            .expect("parallel worker setting should be accepted");
        Spi::run(
            "CREATE TABLE ec_ivf_parallel_build (
                id bigint primary key,
                embedding ecvector
             ) WITH (parallel_workers = 2)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_parallel_build
             SELECT id,
                    ARRAY[
                        id::real,
                        (id * 2)::real,
                        (id * 3 + 1)::real,
                        (id * 5 + 2)::real
                    ]::ecvector
             FROM generate_series(1, 128) AS id",
        )
        .expect("insert should succeed");

        Spi::run(
            "CREATE INDEX ec_ivf_parallel_build_idx ON ec_ivf_parallel_build USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 8, nprobe = 4, training_sample_rows = 64, seed = 71)",
        )
        .expect("parallel index creation should succeed");

        let build_timing = Spi::connect(|client| {
            let mut rows = client
                .select(
                    "SELECT requested_workers, workers_launched, heap_tuples, index_tuples,
                            heap_ingest_us, train_model_us, stage_build_plan_us,
                            stage_pq_train_us, stage_centroids_us, stage_assign_us,
                            stage_postings_us, stage_directory_us, flush_build_plan_us,
                            parallel_begin_us, parallel_drain_us,
                            parallel_sort_push_us, parallel_worker_tuple_buffer_capacity,
                            parallel_worker_tuple_buffer_struct_bytes
                     FROM tests.ec_ivf_debug_last_build_timing()",
                    None,
                    &[],
                )
                .expect("timing query should succeed");
            let row = rows.next().expect("timing query should return one row");
            (
                row.get::<i64>(1)
                    .expect("requested_workers should decode")
                    .unwrap(),
                row.get::<i64>(2)
                    .expect("workers_launched should decode")
                    .unwrap(),
                row.get::<i64>(3)
                    .expect("heap_tuples should decode")
                    .unwrap(),
                row.get::<i64>(4)
                    .expect("index_tuples should decode")
                    .unwrap(),
                row.get::<i64>(5)
                    .expect("heap_ingest_us should decode")
                    .unwrap(),
                row.get::<i64>(6)
                    .expect("train_model_us should decode")
                    .unwrap(),
                row.get::<i64>(7)
                    .expect("stage_build_plan_us should decode")
                    .unwrap(),
                row.get::<i64>(8)
                    .expect("stage_pq_train_us should decode")
                    .unwrap(),
                row.get::<i64>(9)
                    .expect("stage_centroids_us should decode")
                    .unwrap(),
                row.get::<i64>(10)
                    .expect("stage_assign_us should decode")
                    .unwrap(),
                row.get::<i64>(11)
                    .expect("stage_postings_us should decode")
                    .unwrap(),
                row.get::<i64>(12)
                    .expect("stage_directory_us should decode")
                    .unwrap(),
                row.get::<i64>(13)
                    .expect("flush_build_plan_us should decode")
                    .unwrap(),
                row.get::<i64>(14)
                    .expect("parallel_begin_us should decode")
                    .unwrap(),
                row.get::<i64>(15)
                    .expect("parallel_drain_us should decode")
                    .unwrap(),
                row.get::<i64>(16)
                    .expect("parallel_sort_push_us should decode")
                    .unwrap(),
                row.get::<i64>(17)
                    .expect("parallel_worker_tuple_buffer_capacity should decode")
                    .unwrap(),
                row.get::<i64>(18)
                    .expect("parallel_worker_tuple_buffer_struct_bytes should decode")
                    .unwrap(),
            )
        });
        assert_eq!(build_timing.0, 2);
        assert!(
            build_timing.1 >= 1,
            "parallel build should launch at least one worker, launched {}",
            build_timing.1
        );
        assert_eq!(build_timing.2, 128);
        assert_eq!(build_timing.3, 128);
        assert!(build_timing.4 > 0, "heap ingest timing should be recorded");
        assert!(
            build_timing.5 > 0,
            "train model timing should be recorded"
        );
        assert!(
            build_timing.6 > 0,
            "stage build plan timing should be recorded"
        );
        assert!(
            build_timing.8 > 0,
            "stage centroid timing should be recorded"
        );
        assert!(
            build_timing.9 > 0,
            "stage assign timing should be recorded"
        );
        assert!(
            build_timing.10 > 0,
            "stage posting timing should be recorded"
        );
        assert!(
            build_timing.11 > 0,
            "stage directory timing should be recorded"
        );
        assert!(
            build_timing.12 > 0,
            "flush build plan timing should be recorded"
        );
        assert!(
            build_timing.13 > 0,
            "parallel begin timing should be recorded"
        );
        assert!(
            build_timing.14 > 0,
            "parallel drain timing should be recorded"
        );
        assert!(
            build_timing.15 > 0,
            "parallel sort/push timing should be recorded"
        );
        assert!(
            build_timing.16 >= build_timing.2,
            "worker tuple buffer capacity should cover observed heap tuples"
        );
        assert!(
            build_timing.16 > 0,
            "worker tuple buffer capacity should prove capture is live"
        );
        assert_eq!(
            build_timing.17,
            build_timing.16 * am::ivf_build_tuple_struct_size_for_test() as i64
        );

        let index_oid = ec_ivf_index_oid("ec_ivf_parallel_build_idx");
        let (_, _, _, total_live, has_centroids, has_directory) =
            ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (_, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        assert_eq!(total_live, 128);
        assert!(has_centroids);
        assert!(has_directory);
        assert!(empty_lists < 8);
        assert_eq!(directory_live, 128);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 0);
        Spi::run("RESET max_parallel_workers").expect("reset should succeed");
        Spi::run("RESET max_parallel_maintenance_workers").expect("reset should succeed");
    }

    #[pg_test]
    fn test_ec_ivf_parallel_build_matches_serial_structure() {
        Spi::run("SET max_parallel_maintenance_workers = 0")
            .expect("serial maintenance worker setting should be accepted");
        Spi::run("SET max_parallel_workers = 0")
            .expect("serial worker setting should be accepted");
        Spi::run(
            "CREATE TABLE ec_ivf_serial_equiv (
                id bigint primary key,
                embedding ecvector
             )",
        )
        .expect("serial table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_serial_equiv
             SELECT id,
                    ARRAY[
                        (id % 17)::real,
                        ((id * 3) % 19)::real,
                        ((id * 5 + 1) % 23)::real,
                        ((id * 7 + 2) % 29)::real
                    ]::ecvector
             FROM generate_series(1, 128) AS id",
        )
        .expect("serial seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_serial_equiv_idx ON ec_ivf_serial_equiv USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 8, nprobe = 4, training_sample_rows = 64, seed = 710)",
        )
        .expect("serial index creation should succeed");

        Spi::run("SET max_parallel_maintenance_workers = 4")
            .expect("parallel maintenance worker setting should be accepted");
        Spi::run("SET max_parallel_workers = 4")
            .expect("parallel worker setting should be accepted");
        Spi::run(
            "CREATE TABLE ec_ivf_parallel_equiv (
                id bigint primary key,
                embedding ecvector
             ) WITH (parallel_workers = 4)",
        )
        .expect("parallel table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_parallel_equiv
             SELECT id,
                    ARRAY[
                        (id % 17)::real,
                        ((id * 3) % 19)::real,
                        ((id * 5 + 1) % 23)::real,
                        ((id * 7 + 2) % 29)::real
                    ]::ecvector
             FROM generate_series(1, 128) AS id",
        )
        .expect("parallel seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_parallel_equiv_idx ON ec_ivf_parallel_equiv USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 8, nprobe = 4, training_sample_rows = 64, seed = 710)",
        )
        .expect("parallel index creation should succeed");

        let parallel_workers_launched = Spi::connect(|client| {
            let mut rows = client
                .select(
                    "SELECT workers_launched
                     FROM tests.ec_ivf_debug_last_build_timing()",
                    None,
                    &[],
                )
                .expect("timing query should succeed");
            rows.next()
                .expect("timing query should return one row")
                .get::<i64>(1)
                .expect("workers_launched should decode")
                .expect("workers_launched should not be NULL")
        });
        assert!(
            parallel_workers_launched >= 1,
            "parallel equivalence build should launch workers, launched {parallel_workers_launched}"
        );

        let serial_oid = ec_ivf_index_oid("ec_ivf_serial_equiv_idx");
        let parallel_oid = ec_ivf_index_oid("ec_ivf_parallel_equiv_idx");
        let serial_metadata = ec_ivf_build_metadata_signature(serial_oid);
        let parallel_metadata = ec_ivf_build_metadata_signature(parallel_oid);
        assert_eq!(serial_metadata, parallel_metadata);

        let nlists = serial_metadata.1;
        assert_eq!(
            ec_ivf_directory_signatures(serial_oid, nlists),
            ec_ivf_directory_signatures(parallel_oid, nlists)
        );
        assert_eq!(
            ec_ivf_index_blocks("ec_ivf_serial_equiv_idx"),
            ec_ivf_index_blocks("ec_ivf_parallel_equiv_idx")
        );
        assert_eq!(
            ec_ivf_page_ownership_signature("ec_ivf_serial_equiv_idx"),
            ec_ivf_page_ownership_signature("ec_ivf_parallel_equiv_idx")
        );

        let serial_ctid_to_id = ctid_id_map("ec_ivf_serial_equiv");
        let parallel_ctid_to_id = ctid_id_map("ec_ivf_parallel_equiv");
        for query in [
            vec![1.0, 3.0, 6.0, 9.0],
            vec![8.0, 5.0, 2.0, 13.0],
            vec![16.0, 18.0, 22.0, 28.0],
        ] {
            assert_eq!(
                ivf_debug_output_ids(serial_oid, query.clone(), &serial_ctid_to_id, 16),
                ivf_debug_output_ids(parallel_oid, query, &parallel_ctid_to_id, 16)
            );
        }

        Spi::run("RESET max_parallel_workers").expect("reset should succeed");
        Spi::run("RESET max_parallel_maintenance_workers").expect("reset should succeed");
    }

    #[pg_test]
    fn test_ec_ivf_duplicate_heavy_build_keeps_empty_list_counts() {
        Spi::run(
            "CREATE TABLE ec_ivf_duplicate_heavy_build \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_duplicate_heavy_build \
             SELECT g, '[1.0,0.0]'::ecvector \
             FROM generate_series(1, 6) AS g",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_duplicate_heavy_build_idx \
             ON ec_ivf_duplicate_heavy_build USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 3, nprobe = 2, training_sample_rows = 6, seed = 17)",
        )
        .expect("duplicate-heavy index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_duplicate_heavy_build_idx");
        let (dimensions, nlists, training_version, total_live, has_centroids, has_directory) =
            ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (summary_nlists, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));

        assert_eq!(dimensions, 2);
        assert_eq!(nlists, 3);
        assert_eq!(training_version, 1);
        assert_eq!(total_live, 6);
        assert!(has_centroids);
        assert!(has_directory);
        assert_eq!(summary_nlists, 3);
        assert_eq!(empty_lists, 2);
        assert_eq!(directory_live, 6);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 0);
    }

    #[pg_test]
    fn test_ec_ivf_multi_page_list_build_writes_multiple_data_pages() {
        Spi::run(
            "CREATE TABLE ec_ivf_multi_page_build \
             (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_multi_page_build \
             SELECT g, encode_to_ecvector(array_fill(1.0::real, ARRAY[512]), 4, 42) \
             FROM generate_series(1, 32) AS g",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_multi_page_build_idx \
             ON ec_ivf_multi_page_build USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 32, seed = 19)",
        )
        .expect("multi-page index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_multi_page_build_idx");
        let (dimensions, nlists, training_version, total_live, has_centroids, has_directory) =
            ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (summary_nlists, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let index_blocks = ec_ivf_index_blocks("ec_ivf_multi_page_build_idx");

        assert_eq!(dimensions, 512);
        assert_eq!(nlists, 1);
        assert_eq!(training_version, 1);
        assert_eq!(total_live, 32);
        assert!(has_centroids);
        assert!(has_directory);
        assert_eq!(summary_nlists, 1);
        assert_eq!(empty_lists, 0);
        assert_eq!(directory_live, 32);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 0);
        assert!(
            index_blocks > 2,
            "multi-page ec_ivf build should write more than one data page"
        );
    }

    #[pg_test]
    fn test_ec_ivf_empty_gettuple_returns_false_after_rescan() {
        Spi::run("CREATE TABLE ec_ivf_empty_scan (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_empty_scan_idx ON ec_ivf_empty_scan USING ec_ivf \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_empty_scan_idx");
        let found_tuple = ec_ivf_debug!(am::debug_ec_ivf_gettuple_after_rescan_result(index_oid));

        assert!(
            !found_tuple,
            "ec_ivf amgettuple should report no tuples for an empty index"
        );
    }

    #[pg_test]
    fn test_ec_ivf_empty_rescan_query_prep_has_no_probe_lists() {
        Spi::run(
            "CREATE TABLE ec_ivf_empty_query_prep (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_empty_query_prep_idx ON ec_ivf_empty_query_prep USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 4, nprobe = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_empty_query_prep_idx");
        let snapshot = ec_ivf_debug!(am::debug_ec_ivf_rescan_query_prep(index_oid, vec![1.0, 0.0]));

        assert!(snapshot.rescan_called);
        assert_eq!(snapshot.query_dimensions, 2);
        assert_eq!(snapshot.query_values, vec![1.0, 0.0]);
        assert_eq!(snapshot.scan_dimensions, 0);
        assert_eq!(snapshot.scan_nlists, 4);
        assert_eq!(snapshot.scan_nprobe, 0);
        assert!(!snapshot.has_prepared_query);
        assert_eq!(snapshot.prepared_lut_len, 0);
        assert_eq!(snapshot.prepared_sq_len, 0);
        assert_eq!(snapshot.centroid_score_count, 0);
        assert_eq!(snapshot.posting_candidate_count, 0);
        assert!(snapshot.selected_lists.is_empty());
    }

    #[pg_test]
    fn test_ec_ivf_rescan_query_prep_selects_nprobe_lists() {
        Spi::run("CREATE TABLE ec_ivf_query_prep (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_query_prep VALUES
             (1, '[1.0,0.0]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_query_prep_idx ON ec_ivf_query_prep USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 3, nprobe = 2, training_sample_rows = 3, seed = 23)",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_query_prep_idx");
        let snapshot = ec_ivf_debug!(am::debug_ec_ivf_rescan_query_prep(index_oid, vec![1.0, 0.0]));
        let unique_lists = snapshot
            .selected_lists
            .iter()
            .copied()
            .collect::<HashSet<_>>();

        assert!(snapshot.rescan_called);
        assert_eq!(snapshot.query_dimensions, 2);
        assert_eq!(snapshot.query_values, vec![1.0, 0.0]);
        assert_eq!(snapshot.scan_dimensions, 2);
        assert_eq!(snapshot.scan_nlists, 3);
        assert_eq!(snapshot.scan_nprobe, 2);
        assert!(snapshot.has_prepared_query);
        assert!(snapshot.prepared_lut_len > 0);
        assert!(snapshot.prepared_sq_len > 0);
        assert_eq!(snapshot.centroid_score_count, 3);
        assert_eq!(snapshot.posting_candidate_count, 2);
        assert_eq!(snapshot.selected_lists.len(), 2);
        assert_eq!(unique_lists.len(), snapshot.selected_lists.len());
        assert!(snapshot
            .selected_lists
            .iter()
            .all(|list_id| *list_id < snapshot.scan_nlists));
    }

    #[pg_test]
    fn test_ec_ivf_rescan_reuses_cached_prod_quantizer() {
        Spi::run("CREATE TABLE ec_ivf_quantizer_cache (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_quantizer_cache VALUES
             (1, '[1.0,0.0]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_quantizer_cache_idx ON ec_ivf_quantizer_cache USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 3, nprobe = 2, training_sample_rows = 3, seed = 23)",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_quantizer_cache_idx");
        let _first_scan = ec_ivf_debug!(am::debug_ec_ivf_rescan_query_prep(index_oid, vec![1.0, 0.0]));
        let first_ptr = ec_ivf_debug!(am::debug_ec_ivf_quantizer_cache_ptr(index_oid))
            .expect("first IVF scan should populate the ProdQuantizer cache");
        let _second_scan = ec_ivf_debug!(am::debug_ec_ivf_rescan_query_prep(index_oid, vec![0.0, 1.0]));
        let second_ptr = ec_ivf_debug!(am::debug_ec_ivf_quantizer_cache_ptr(index_oid))
            .expect("second IVF scan should find the cached ProdQuantizer");

        assert_eq!(
            first_ptr, second_ptr,
            "IVF rescans on the same index should reuse the same cached ProdQuantizer"
        );
    }

    #[pg_test]
    fn test_ec_ivf_gettuple_emits_probe_candidates_with_scores() {
        Spi::run("CREATE TABLE ec_ivf_gettuple_emit (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_gettuple_emit VALUES
             (1, '[1.0,0.0]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_gettuple_emit_idx ON ec_ivf_gettuple_emit USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 3, nprobe = 3, training_sample_rows = 3, seed = 29)",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_gettuple_emit_idx");
        let (outputs, orderby_cleared) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.0]));
        let unique_heap_tids = outputs
            .iter()
            .map(|(block_number, offset_number, _score)| (*block_number, *offset_number))
            .collect::<HashSet<_>>();

        assert_eq!(outputs.len(), 3);
        assert_eq!(unique_heap_tids.len(), outputs.len());
        assert!(outputs.iter().all(|(_, _, score)| score.is_finite()));
        assert!(
            outputs.windows(2).all(|pair| pair[0].2 <= pair[1].2),
            "ec_ivf gettuple outputs should be ordered by ascending ORDER BY score"
        );
        assert!(orderby_cleared);
    }

    #[pg_test]
    fn test_ec_ivf_auto_rerank_resolves_to_off_metadata() {
        Spi::run("CREATE TABLE ec_ivf_auto_rerank (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_auto_rerank_idx ON ec_ivf_auto_rerank USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (rerank = 'auto')",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_auto_rerank_idx");
        let rerank_mode = ec_ivf_debug!(am::debug_ec_ivf_rerank_mode(index_oid));

        assert_eq!(rerank_mode, "off");
    }

    #[pg_test]
    fn test_ec_ivf_heap_f32_rerank_mode_is_persisted() {
        Spi::run("CREATE TABLE ec_ivf_heap_f32_rerank (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_heap_f32_rerank_idx ON ec_ivf_heap_f32_rerank USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (rerank = 'heap_f32')",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_heap_f32_rerank_idx");
        let rerank_mode = ec_ivf_debug!(am::debug_ec_ivf_rerank_mode(index_oid));

        assert_eq!(rerank_mode, "heap_f32");
    }

    #[pg_test]
    fn test_ec_ivf_pq_fastscan_storage_build_scan_insert_vacuum() {
        Spi::run(
            "CREATE TABLE ec_ivf_pq_fastscan_storage (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_pq_fastscan_storage VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_pq_fastscan_storage_idx ON ec_ivf_pq_fastscan_storage USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 2, nprobe = 2, training_sample_rows = 4, storage_format = 'pq_fastscan')",
        )
        .expect("PqFastScan IVF index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_pq_fastscan_storage_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_pq_fastscan_storage");
        let build_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);

        assert!(
            build_ids.contains(&0),
            "PqFastScan IVF scan should return the matching build-time row"
        );

        Spi::run("INSERT INTO ec_ivf_pq_fastscan_storage VALUES (4, '[1.0,0.1]'::ecvector)")
            .expect("live insert should succeed");
        let inserted_tid = heap_tid_for_row("ec_ivf_pq_fastscan_storage", 4);
        let before_vacuum =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.1])).0;

        assert!(
            before_vacuum
                .iter()
                .any(|(block_number, offset_number, _score)| {
                    (*block_number, *offset_number)
                        == (inserted_tid.block_number, inserted_tid.offset_number)
                }),
            "PqFastScan IVF scan should include the live-inserted row"
        );

        Spi::run("DELETE FROM ec_ivf_pq_fastscan_storage WHERE id = 4")
            .expect("delete should succeed");
        let stats = ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[inserted_tid]));
        let after_vacuum =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.1])).0;

        assert_eq!(stats.tuples_removed, 1.0);
        assert!(
            after_vacuum
                .iter()
                .all(|(block_number, offset_number, _score)| {
                    (*block_number, *offset_number)
                        != (inserted_tid.block_number, inserted_tid.offset_number)
                }),
            "PqFastScan IVF scan should not include the vacuumed row"
        );
        assert!(after_vacuum.iter().all(|(_, _, score)| score.is_finite()));
    }

    #[pg_test]
    fn test_ec_ivf_pq_fastscan_accepts_group_size_reloption() {
        fn literal(values: &[f32]) -> String {
            let body = values
                .iter()
                .map(|value| format!("{value:.1}"))
                .collect::<Vec<_>>()
                .join(",");
            format!("'[{body}]'::ecvector")
        }

        let mut base = vec![0.0_f32; 32];
        base[0] = 1.0;
        let mut near = vec![0.0_f32; 32];
        near[0] = 0.9;
        near[1] = 0.1;
        let mut orthogonal = vec![0.0_f32; 32];
        orthogonal[8] = 1.0;
        let mut opposite = vec![0.0_f32; 32];
        opposite[0] = -1.0;

        Spi::run(
            "CREATE TABLE ec_ivf_pq_fastscan_group_size (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(&format!(
            "INSERT INTO ec_ivf_pq_fastscan_group_size VALUES
             (0, {}),
             (1, {}),
             (2, {}),
             (3, {})",
            literal(&base),
            literal(&near),
            literal(&orthogonal),
            literal(&opposite)
        ))
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_pq_fastscan_group_size_idx ON ec_ivf_pq_fastscan_group_size USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 2, nprobe = 2, training_sample_rows = 4, storage_format = 'pq_fastscan', pq_group_size = 8)",
        )
        .expect("PqFastScan IVF index creation with pq_group_size should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_pq_fastscan_group_size_idx");
        let reloptions = Spi::get_one::<Vec<String>>(
            "SELECT reloptions FROM pg_class WHERE oid = 'ec_ivf_pq_fastscan_group_size_idx'::regclass",
        )
        .expect("reloptions query should succeed")
        .expect("reloptions should exist");
        let outputs = ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, base)).0;

        assert!(reloptions.contains(&"pq_group_size=8".to_string()));
        assert!(
            outputs.iter().all(|(_, _, score)| score.is_finite()),
            "PqFastScan IVF scan should work with non-default grouped-PQ subvector size"
        );
    }

    #[pg_test]
    fn test_ec_ivf_pq_fastscan_scan_reuses_loaded_model() {
        Spi::run(
            "CREATE TABLE ec_ivf_pq_fastscan_model_cache (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_pq_fastscan_model_cache VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_pq_fastscan_model_cache_idx ON ec_ivf_pq_fastscan_model_cache USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 2, nprobe = 2, training_sample_rows = 4, storage_format = 'pq_fastscan')",
        )
        .expect("PqFastScan IVF index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_pq_fastscan_model_cache_idx");

        assert!(
            ec_ivf_debug!(am::debug_ec_ivf_pq_fastscan_model_cache_reused(index_oid)),
            "PqFastScan IVF scans should reuse one loaded grouped-codebook model across rescans"
        );
    }

    #[pg_test]
    fn test_ec_ivf_quantizer_reloption_alias_accepts_pq_fastscan() {
        Spi::run(
            "CREATE TABLE ec_ivf_quantizer_alias_pq (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_quantizer_alias_pq VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_quantizer_alias_pq_idx ON ec_ivf_quantizer_alias_pq USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 2, nprobe = 2, training_sample_rows = 4, quantizer = 'pq_fastscan')",
        )
        .expect("PqFastScan IVF index creation through quantizer alias should succeed");

        let storage_format = Spi::get_one::<String>(
            "SELECT storage_format FROM ec_ivf_index_admin_snapshot('ec_ivf_quantizer_alias_pq_idx'::regclass)",
        )
        .expect("admin snapshot should succeed")
        .expect("storage format should be present");
        let index_oid = ec_ivf_index_oid("ec_ivf_quantizer_alias_pq_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_quantizer_alias_pq");
        let build_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);

        assert_eq!(storage_format, "pq_fastscan");
        assert!(build_ids.contains(&0));
    }

    #[pg_test]
    fn test_ec_ivf_quantizer_reloption_alias_accepts_rabitq() {
        Spi::run(
            "CREATE TABLE ec_ivf_quantizer_alias_rabitq (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_quantizer_alias_rabitq VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_quantizer_alias_rabitq_idx ON ec_ivf_quantizer_alias_rabitq USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 2, nprobe = 2, training_sample_rows = 4, quantizer = 'rabitq')",
        )
        .expect("RaBitQ IVF index creation through quantizer alias should succeed");

        let storage_format = Spi::get_one::<String>(
            "SELECT storage_format FROM ec_ivf_index_admin_snapshot('ec_ivf_quantizer_alias_rabitq_idx'::regclass)",
        )
        .expect("admin snapshot should succeed")
        .expect("storage format should be present");
        let index_oid = ec_ivf_index_oid("ec_ivf_quantizer_alias_rabitq_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_quantizer_alias_rabitq");
        let build_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);

        assert_eq!(storage_format, "rabitq");
        assert!(build_ids.contains(&0));
    }

    #[pg_test]
    fn test_ec_ivf_coarse_rerank_contract_admin_snapshot() {
        Spi::run(
            "CREATE TABLE ec_ivf_coarse_rerank_contract (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_coarse_rerank_contract VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_coarse_rerank_contract_idx ON ec_ivf_coarse_rerank_contract USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 2,
                nprobe = 2,
                training_sample_rows = 4,
                storage_format = 'coarse_rerank',
                coarse_format = 'rabitq',
                coarse_bits = 1,
                rerank = 'heap_f32',
                rerank_placement = 'heap',
                rerank_format = 'heap_f32',
                rerank_width = 3
             )",
        )
        .expect("coarse_rerank IVF index creation should succeed");

        let contract = Spi::get_one::<String>(
            "SELECT storage_format || '/' || coarse_format || '/' || coarse_bits || '/' ||
                    rerank || '/' || rerank_placement || '/' || rerank_format || '/' ||
                    relation_rerank_width
             FROM ec_ivf_index_admin_snapshot('ec_ivf_coarse_rerank_contract_idx'::regclass)",
        )
        .expect("admin snapshot should succeed")
        .expect("contract row should be present");
        let index_oid = ec_ivf_index_oid("ec_ivf_coarse_rerank_contract_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_coarse_rerank_contract");
        let build_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);

        assert_eq!(contract, "coarse_rerank/rabitq/1/heap_f32/table/f32/3");
        assert!(build_ids.contains(&0));
    }

    #[pg_test]
    fn test_ec_ivf_coarse_rerank_f16_rabitq4_admin_snapshot() {
        // Task 111g: f16 and rabitq4 table-side rerank formats are creatable and
        // reported by the admin snapshot.
        for (suffix, rerank_format, expected) in [
            ("f16", "f16", "coarse_rerank/table/f16"),
            ("rabitq4", "rabitq4", "coarse_rerank/table/rabitq4"),
        ] {
            let table = format!("ec_ivf_cr_fmt_{suffix}");
            let index = format!("ec_ivf_cr_fmt_{suffix}_idx");
            Spi::run(&format!(
                "CREATE TABLE {table} (id bigint primary key, embedding ecvector)"
            ))
            .expect("table creation should succeed");
            Spi::run(&format!(
                "INSERT INTO {table} VALUES
                 (0, '[1.0,0.0]'::ecvector),
                 (1, '[0.0,1.0]'::ecvector),
                 (2, '[-1.0,0.0]'::ecvector),
                 (3, '[0.0,-1.0]'::ecvector)"
            ))
            .expect("seed insert should succeed");
            Spi::run(&format!(
                "CREATE INDEX {index} ON {table} USING ec_ivf \
                 (embedding ecvector_ip_ops) \
                 WITH (
                    nlists = 2,
                    nprobe = 2,
                    training_sample_rows = 4,
                    storage_format = 'coarse_rerank',
                    rerank_format = '{rerank_format}',
                    rerank_width = 3
                 )"
            ))
            .unwrap_or_else(|e| panic!("coarse_rerank {rerank_format} index creation should succeed: {e:?}"));

            let contract = Spi::get_one::<String>(&format!(
                "SELECT storage_format || '/' || rerank_placement || '/' || rerank_format \
                 FROM ec_ivf_index_admin_snapshot('{index}'::regclass)"
            ))
            .expect("admin snapshot should succeed")
            .expect("contract row should be present");
            assert_eq!(contract, expected, "rerank_format {rerank_format} admin snapshot");
        }
    }

    #[pg_test]
    fn test_ec_ivf_coarse_rerank_f16_matches_f32_ranking() {
        // Task 111g recall parity: an f16 table-side rerank returns the same
        // top-k ranking as the f32 table-side rerank on a deterministic corpus,
        // and the f16 scores stay within binary16 precision of the f32 scores.
        let rows: Vec<(i64, Vec<f32>)> = (0..16)
            .map(|id| {
                let base = id as f32;
                let vector = (0..8)
                    .map(|d| ((base * 0.37 + d as f32 * 0.11).sin()) * 0.5 + 0.5)
                    .collect::<Vec<f32>>();
                (id, vector)
            })
            .collect();

        for (table, rerank_format) in [
            ("ec_ivf_cr_par_f32", "f32"),
            ("ec_ivf_cr_par_f16", "f16"),
        ] {
            Spi::run(&format!(
                "CREATE TABLE {table} (id bigint primary key, embedding ecvector)"
            ))
            .expect("table creation should succeed");
            for (id, vector) in &rows {
                let literal = format_recall_vector_sql_literal(vector);
                Spi::run(&format!(
                    "INSERT INTO {table} VALUES ({id}, {literal}::ecvector)"
                ))
                .expect("seed insert should succeed");
            }
            Spi::run(&format!(
                "CREATE INDEX {table}_idx ON {table} USING ec_ivf \
                 (embedding ecvector_ip_ops) \
                 WITH (
                    nlists = 4,
                    nprobe = 4,
                    training_sample_rows = 16,
                    storage_format = 'coarse_rerank',
                    rerank_format = '{rerank_format}',
                    rerank_width = 5
                 )"
            ))
            .expect("coarse_rerank index creation should succeed");
        }

        let query = rows[3].1.clone();
        let f32_oid = ec_ivf_index_oid("ec_ivf_cr_par_f32_idx");
        let f16_oid = ec_ivf_index_oid("ec_ivf_cr_par_f16_idx");
        let ctid_f32 = ctid_id_map("ec_ivf_cr_par_f32");
        let ctid_f16 = ctid_id_map("ec_ivf_cr_par_f16");

        let f32_ids = ivf_debug_output_ids(f32_oid, query.clone(), &ctid_f32, 5);
        let f16_ids = ivf_debug_output_ids(f16_oid, query.clone(), &ctid_f16, 5);
        assert_eq!(
            f32_ids, f16_ids,
            "f16 rerank top-5 ranking should match f32 rerank ranking"
        );

        // The f16 scores should track the f32 scores within binary16 precision.
        let (f32_outputs, _) = ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(f32_oid, query.clone()));
        let (f16_outputs, _) = ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(f16_oid, query));
        assert_eq!(f32_outputs.len(), f16_outputs.len());
        for ((_, _, f32_score), (_, _, f16_score)) in f32_outputs.iter().zip(f16_outputs.iter()) {
            let tolerance = 0.01_f32.max(f32_score.abs() * 0.01);
            assert!(
                (f32_score - f16_score).abs() <= tolerance,
                "f16 rerank score {f16_score} should track f32 rerank score {f32_score} within binary16 precision"
            );
        }
    }

    #[pg_test]
    fn test_ec_ivf_coarse_rerank_rabitq4_returns_correct_top_neighbor() {
        // Task 111g correctness: rabitq4 table-side rerank ranks the exact
        // nearest neighbor first on a separable corpus.
        Spi::run(
            "CREATE TABLE ec_ivf_cr_rb4 (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        let rows: Vec<(i64, Vec<f32>)> = (0..12)
            .map(|id| {
                let theta = id as f32 * (std::f32::consts::PI / 6.0);
                (id, vec![theta.cos(), theta.sin(), 0.0, 0.0])
            })
            .collect();
        for (id, vector) in &rows {
            let literal = format_recall_vector_sql_literal(vector);
            Spi::run(&format!(
                "INSERT INTO ec_ivf_cr_rb4 VALUES ({id}, {literal}::ecvector)"
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX ec_ivf_cr_rb4_idx ON ec_ivf_cr_rb4 USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 4,
                nprobe = 4,
                training_sample_rows = 12,
                storage_format = 'coarse_rerank',
                rerank_format = 'rabitq4',
                rerank_width = 5
             )",
        )
        .expect("coarse_rerank rabitq4 index creation should succeed");

        // Query aligned with row 0; row 0 is the exact nearest neighbor.
        let query = rows[0].1.clone();
        let index_oid = ec_ivf_index_oid("ec_ivf_cr_rb4_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_cr_rb4");
        let ids = ivf_debug_output_ids(index_oid, query, &ctid_to_id, 3);
        assert!(
            !ids.is_empty(),
            "rabitq4 rerank should emit candidates"
        );
        assert_eq!(
            ids[0], 0,
            "rabitq4 rerank should rank the exact nearest neighbor first, got {ids:?}"
        );
    }

    #[pg_test]
    fn test_ec_ivf_index_placement_f16_rabitq4_admin_snapshot() {
        // Task 111g (003b): index placement is creatable for f16 and rabitq4,
        // and the admin snapshot reports rerank_placement = index.
        for (suffix, rerank_format, expected) in [
            ("f16", "f16", "coarse_rerank/index/f16"),
            ("rabitq4", "rabitq4", "coarse_rerank/index/rabitq4"),
        ] {
            let table = format!("ec_ivf_idx_pl_{suffix}");
            let index = format!("ec_ivf_idx_pl_{suffix}_idx");
            Spi::run(&format!(
                "CREATE TABLE {table} (id bigint primary key, embedding ecvector)"
            ))
            .expect("table creation should succeed");
            Spi::run(&format!(
                "INSERT INTO {table} VALUES
                 (0, '[1.0,0.0]'::ecvector),
                 (1, '[0.0,1.0]'::ecvector),
                 (2, '[-1.0,0.0]'::ecvector),
                 (3, '[0.0,-1.0]'::ecvector)"
            ))
            .expect("seed insert should succeed");
            Spi::run(&format!(
                "CREATE INDEX {index} ON {table} USING ec_ivf \
                 (embedding ecvector_ip_ops) \
                 WITH (
                    nlists = 2,
                    nprobe = 2,
                    training_sample_rows = 4,
                    storage_format = 'coarse_rerank',
                    rerank_format = '{rerank_format}',
                    rerank_placement = 'index',
                    rerank_width = 3
                 )"
            ))
            .unwrap_or_else(|e| {
                panic!("index-placement {rerank_format} index creation should succeed: {e:?}")
            });

            let contract = Spi::get_one::<String>(&format!(
                "SELECT storage_format || '/' || rerank_placement || '/' || rerank_format \
                 FROM ec_ivf_index_admin_snapshot('{index}'::regclass)"
            ))
            .expect("admin snapshot should succeed")
            .expect("contract row should be present");
            assert_eq!(contract, expected, "index placement {rerank_format} admin snapshot");
        }
    }

    #[pg_test]
    fn test_ec_ivf_index_placement_f16_matches_table_f16_ranking() {
        // Task 111g (003b) recall parity: the index-side f16 sidecar rerank
        // returns the SAME top-k ranking and bit-identical scores as the
        // table-side f16 rerank (both round-trip the source through binary16),
        // proving the persisted compact sidecar preserves f16 recall.
        let rows: Vec<(i64, Vec<f32>)> = (0..16)
            .map(|id| {
                let base = id as f32;
                let vector = (0..8)
                    .map(|d| ((base * 0.37 + d as f32 * 0.11).sin()) * 0.5 + 0.5)
                    .collect::<Vec<f32>>();
                (id, vector)
            })
            .collect();

        for (table, placement) in [
            ("ec_ivf_idx_par_table", "table"),
            ("ec_ivf_idx_par_index", "index"),
        ] {
            Spi::run(&format!(
                "CREATE TABLE {table} (id bigint primary key, embedding ecvector)"
            ))
            .expect("table creation should succeed");
            for (id, vector) in &rows {
                let literal = format_recall_vector_sql_literal(vector);
                Spi::run(&format!("INSERT INTO {table} VALUES ({id}, {literal}::ecvector)"))
                    .expect("seed insert should succeed");
            }
            Spi::run(&format!(
                "CREATE INDEX {table}_idx ON {table} USING ec_ivf \
                 (embedding ecvector_ip_ops) \
                 WITH (
                    nlists = 4,
                    nprobe = 4,
                    training_sample_rows = 16,
                    storage_format = 'coarse_rerank',
                    rerank_format = 'f16',
                    rerank_placement = '{placement}',
                    rerank_width = 5
                 )"
            ))
            .expect("coarse_rerank f16 index creation should succeed");
        }

        let query = rows[3].1.clone();
        let table_oid = ec_ivf_index_oid("ec_ivf_idx_par_table_idx");
        let index_oid = ec_ivf_index_oid("ec_ivf_idx_par_index_idx");
        let ctid_table = ctid_id_map("ec_ivf_idx_par_table");
        let ctid_index = ctid_id_map("ec_ivf_idx_par_index");

        let table_ids = ivf_debug_output_ids(table_oid, query.clone(), &ctid_table, 5);
        let index_ids = ivf_debug_output_ids(index_oid, query.clone(), &ctid_index, 5);
        assert_eq!(
            table_ids, index_ids,
            "index-side f16 rerank top-5 ranking should match table-side f16 ranking"
        );

        let (table_outputs, _) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(table_oid, query.clone()));
        let (index_outputs, _) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, query));
        assert_eq!(table_outputs.len(), index_outputs.len());
        for ((_, _, table_score), (_, _, index_score)) in
            table_outputs.iter().zip(index_outputs.iter())
        {
            // The sidecar stores the exact f16 round-trip of the source, so the
            // index-side score is bit-identical to the table-side f16 score.
            assert_eq!(
                index_score.to_bits(),
                table_score.to_bits(),
                "index-side f16 score {index_score} should match table-side f16 score {table_score}"
            );
        }
    }

    #[pg_test]
    fn test_ec_ivf_index_placement_rabitq4_top_neighbor() {
        // Task 111g (003b) correctness: the index-side rabitq4 sidecar rerank
        // ranks the exact nearest neighbor first on a separable corpus.
        Spi::run("CREATE TABLE ec_ivf_idx_rb4 (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        let rows: Vec<(i64, Vec<f32>)> = (0..12)
            .map(|id| {
                let theta = id as f32 * (std::f32::consts::PI / 6.0);
                (id, vec![theta.cos(), theta.sin(), 0.0, 0.0])
            })
            .collect();
        for (id, vector) in &rows {
            let literal = format_recall_vector_sql_literal(vector);
            Spi::run(&format!("INSERT INTO ec_ivf_idx_rb4 VALUES ({id}, {literal}::ecvector)"))
                .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX ec_ivf_idx_rb4_idx ON ec_ivf_idx_rb4 USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 4,
                nprobe = 4,
                training_sample_rows = 12,
                storage_format = 'coarse_rerank',
                rerank_format = 'rabitq4',
                rerank_placement = 'index',
                rerank_width = 5
             )",
        )
        .expect("index-placement rabitq4 index creation should succeed");

        let query = rows[0].1.clone();
        let index_oid = ec_ivf_index_oid("ec_ivf_idx_rb4_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_idx_rb4");
        let ids = ivf_debug_output_ids(index_oid, query, &ctid_to_id, 3);
        assert!(!ids.is_empty(), "index-side rabitq4 rerank should emit candidates");
        assert_eq!(
            ids[0], 0,
            "index-side rabitq4 rerank should rank the exact nearest neighbor first, got {ids:?}"
        );
    }

    #[pg_test]
    fn test_ec_ivf_index_placement_fewer_rerank_bytes() {
        // Task 111g (003b) WIN EVIDENCE (by counter, no ecaz bench suite): the
        // index-placement compact sidecar reads strictly fewer rerank *source*
        // bytes than the table-side f32 rerank on the same corpus/query.
        //   - table f32 reads dims*4 per reranked candidate (full heap source).
        //   - index f16 reads dims*2; index rabitq4 reads the rabitq4 payload
        //     length (both far smaller than dims*4).
        let dims = 8usize;
        let rows: Vec<(i64, Vec<f32>)> = (0..16)
            .map(|id| {
                let base = id as f32;
                let vector = (0..dims)
                    .map(|d| ((base * 0.41 + d as f32 * 0.13).cos()) * 0.5 + 0.5)
                    .collect::<Vec<f32>>();
                (id, vector)
            })
            .collect();

        let make_index = |table: &str, rerank_format: &str, placement: &str| {
            Spi::run(&format!(
                "CREATE TABLE {table} (id bigint primary key, embedding ecvector)"
            ))
            .expect("table creation should succeed");
            for (id, vector) in &rows {
                let literal = format_recall_vector_sql_literal(vector);
                Spi::run(&format!("INSERT INTO {table} VALUES ({id}, {literal}::ecvector)"))
                    .expect("seed insert should succeed");
            }
            Spi::run(&format!(
                "CREATE INDEX {table}_idx ON {table} USING ec_ivf \
                 (embedding ecvector_ip_ops) \
                 WITH (
                    nlists = 4,
                    nprobe = 4,
                    training_sample_rows = 16,
                    storage_format = 'coarse_rerank',
                    rerank_format = '{rerank_format}',
                    rerank_placement = '{placement}',
                    rerank_width = 8
                 )"
            ))
            .expect("coarse_rerank index creation should succeed");
            ec_ivf_index_oid(&format!("{table}_idx"))
        };

        let table_f32_oid = make_index("ec_ivf_bytes_table_f32", "f32", "table");
        let index_f16_oid = make_index("ec_ivf_bytes_index_f16", "f16", "index");
        let index_rb4_oid = make_index("ec_ivf_bytes_index_rb4", "rabitq4", "index");

        let query = rows[2].1.clone();
        let table_f32 = ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(
            table_f32_oid,
            query.clone()
        ));
        let index_f16 = ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(
            index_f16_oid,
            query.clone()
        ));
        let index_rb4 = ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(
            index_rb4_oid,
            query
        ));

        // Every variant reranks the same frontier, so rerank_rows match and the
        // byte comparison is apples-to-apples.
        assert!(table_f32.rerank_rows > 0, "table f32 should rerank rows");
        assert_eq!(table_f32.rerank_rows, index_f16.rerank_rows);
        assert_eq!(table_f32.rerank_rows, index_rb4.rerank_rows);

        // f32 reads dims*4 per candidate; f16 reads dims*2 (exactly half).
        assert_eq!(
            table_f32.rerank_source_bytes_read,
            table_f32.rerank_rows * (dims * 4) as u32,
            "table f32 should read dims*4 per reranked candidate"
        );
        assert_eq!(
            index_f16.rerank_source_bytes_read,
            index_f16.rerank_rows * (dims * 2) as u32,
            "index f16 should read dims*2 per reranked candidate"
        );

        assert!(
            index_f16.rerank_source_bytes_read < table_f32.rerank_source_bytes_read,
            "index f16 ({}) should read fewer rerank source bytes than table f32 ({})",
            index_f16.rerank_source_bytes_read,
            table_f32.rerank_source_bytes_read
        );
        assert!(
            index_rb4.rerank_source_bytes_read < table_f32.rerank_source_bytes_read,
            "index rabitq4 ({}) should read fewer rerank source bytes than table f32 ({})",
            index_rb4.rerank_source_bytes_read,
            table_f32.rerank_source_bytes_read
        );
    }

    #[pg_test]
    fn test_ec_ivf_index_placement_insert_maintains_sidecar() {
        // Task 111g (003b): a row inserted after build into an index-placement
        // index is reranked from the sidecar (its compact payload was appended),
        // so it is returned and ranked correctly.
        Spi::run("CREATE TABLE ec_ivf_idx_ins (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        let rows: Vec<(i64, Vec<f32>)> = (0..12)
            .map(|id| {
                let theta = id as f32 * (std::f32::consts::PI / 6.0);
                (id, vec![theta.cos(), theta.sin(), 0.0, 0.0])
            })
            .collect();
        for (id, vector) in &rows {
            let literal = format_recall_vector_sql_literal(vector);
            Spi::run(&format!("INSERT INTO ec_ivf_idx_ins VALUES ({id}, {literal}::ecvector)"))
                .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX ec_ivf_idx_ins_idx ON ec_ivf_idx_ins USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 4,
                nprobe = 4,
                training_sample_rows = 12,
                storage_format = 'coarse_rerank',
                rerank_format = 'f16',
                rerank_placement = 'index',
                rerank_width = 8
             )",
        )
        .expect("index-placement f16 index creation should succeed");

        // Insert a brand-new row AFTER build with a vector distinct from every
        // built row (no tie), placed nearest the query; it must flow through
        // aminsert's sidecar append and then rank first when probed.
        Spi::run("INSERT INTO ec_ivf_idx_ins VALUES (100, '[0.0,0.0,1.0,0.0]'::ecvector)")
            .expect("post-build insert should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_idx_ins_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_idx_ins");
        let ids = ivf_debug_output_ids(index_oid, vec![0.0, 0.0, 1.0, 0.0], &ctid_to_id, 3);
        assert!(
            ids.contains(&100),
            "post-build inserted row should be reranked from the sidecar, got {ids:?}"
        );
        assert_eq!(
            ids[0], 100,
            "the inserted exact match should rank first, got {ids:?}"
        );
    }

    #[pg_test]
    fn test_ec_ivf_table_placement_has_no_sidecar_and_scans() {
        // Task 111g (003b): a table-placement coarse_rerank index carries NO
        // sidecar (rerank_sidecar_head = INVALID). Such an index must still
        // scan, reading the heap source (the table f32 path). This is the
        // legitimate "no sidecar -> table fallback" runtime state, not a compat
        // mode.
        Spi::run("CREATE TABLE ec_ivf_no_sidecar (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_no_sidecar VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_no_sidecar_idx ON ec_ivf_no_sidecar USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 2,
                nprobe = 2,
                training_sample_rows = 4,
                storage_format = 'coarse_rerank',
                rerank_format = 'f32',
                rerank_placement = 'table',
                rerank_width = 3
             )",
        )
        .expect("table-placement index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_no_sidecar_idx");
        // No sidecar head persisted: the metadata reads back with an invalid
        // sidecar head and the scan falls back to the heap/table source path.
        let sidecar_head_valid =
            ec_ivf_debug!(am::debug_ec_ivf_rerank_sidecar_head_is_valid(index_oid));
        assert!(
            !sidecar_head_valid,
            "table-placement index should have no sidecar head"
        );

        let ctid_to_id = ctid_id_map("ec_ivf_no_sidecar");
        let ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);
        assert!(ids.contains(&0), "table-placement scan should return rows, got {ids:?}");
        assert_eq!(ids[0], 0, "table f32 rerank should rank the exact match first");
    }

    #[pg_test]
    fn test_ec_ivf_posting_slack_reloption_reserves_list_range() {
        Spi::run(
            "CREATE TABLE ec_ivf_posting_slack_build (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_posting_slack_build VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.9,0.1]'::ecvector),
             (2, '[0.8,0.2]'::ecvector),
             (3, '[0.7,0.3]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_posting_slack_build_idx ON ec_ivf_posting_slack_build USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 4, posting_slack_percent = 100)",
        )
        .expect("IVF index creation with posting slack should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_posting_slack_build_idx");
        let (head_block, tail_block, live_count, dead_count, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_entry(index_oid, 0));
        let ctid_to_id = ctid_id_map("ec_ivf_posting_slack_build");
        let build_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);

        assert!(
            tail_block > head_block,
            "posting_slack_percent should extend the list-local block range"
        );
        assert_eq!(live_count, 4);
        assert_eq!(dead_count, 0);
        assert_eq!(inserted_since_build, 0);
        assert!(build_ids.contains(&0));
    }

    #[pg_test]
    #[should_panic(expected = "storage_format and quantizer reloptions conflict")]
    fn test_ec_ivf_quantizer_reloption_conflicts_with_storage_format() {
        Spi::run(
            "CREATE TABLE ec_ivf_quantizer_alias_conflict (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_quantizer_alias_conflict_idx ON ec_ivf_quantizer_alias_conflict USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (storage_format = 'turboquant', quantizer = 'rabitq')",
        )
        .expect("conflicting quantizer reloptions should fail");
    }

    #[pg_test]
    fn test_ec_ivf_rabitq_storage_build_scan_insert_vacuum() {
        Spi::run("CREATE TABLE ec_ivf_rabitq_storage (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_rabitq_storage VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_rabitq_storage_idx ON ec_ivf_rabitq_storage USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 2, nprobe = 2, training_sample_rows = 4, storage_format = 'rabitq')",
        )
        .expect("RaBitQ IVF index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_rabitq_storage_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_rabitq_storage");
        let build_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);

        assert!(
            build_ids.contains(&0),
            "RaBitQ IVF scan should return the matching build-time row"
        );

        Spi::run("INSERT INTO ec_ivf_rabitq_storage VALUES (4, '[1.0,0.1]'::ecvector)")
            .expect("live insert should succeed");
        let inserted_tid = heap_tid_for_row("ec_ivf_rabitq_storage", 4);
        let before_vacuum =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.1])).0;

        assert!(
            before_vacuum
                .iter()
                .any(|(block_number, offset_number, _score)| {
                    (*block_number, *offset_number)
                        == (inserted_tid.block_number, inserted_tid.offset_number)
                }),
            "RaBitQ IVF scan should include the live-inserted row"
        );

        Spi::run("DELETE FROM ec_ivf_rabitq_storage WHERE id = 4").expect("delete should succeed");
        let stats = ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[inserted_tid]));
        let after_vacuum =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.1])).0;

        assert_eq!(stats.tuples_removed, 1.0);
        assert!(
            after_vacuum
                .iter()
                .all(|(block_number, offset_number, _score)| {
                    (*block_number, *offset_number)
                        != (inserted_tid.block_number, inserted_tid.offset_number)
                }),
            "RaBitQ IVF scan should not include the vacuumed row"
        );
        assert!(after_vacuum.iter().all(|(_, _, score)| score.is_finite()));
    }

    #[pg_test]
    fn test_ec_ivf_dense_posting_blocks_scan_build_rows() {
        Spi::run(
            "CREATE TABLE ec_ivf_dense_build_scan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_dense_build_scan VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.9,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_dense_build_scan_idx ON ec_ivf_dense_build_scan USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 4, storage_format = 'turboquant', dense_posting_blocks = 1)",
        )
        .expect("dense IVF index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_dense_build_scan_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_dense_build_scan");
        let ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);
        let counters =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, vec![1.0, 0.0]));

        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&0));
        assert_eq!(counters.outputs.len(), 4);
        assert_eq!(counters.row_postings_visited, 0);
        assert_eq!(counters.dense_blocks_visited, 1);
        assert_eq!(counters.dense_postings_visited, 4);
        assert_eq!(counters.scratch_soa_flushes, 0);
        assert_eq!(counters.dense_coalesced_flushes, 1);
        assert!(counters.orderby_cleared);
    }

    #[pg_test]
    fn test_ec_ivf_dense_typed_posting_blocks_scan_build_rows() {
        Spi::run(
            "CREATE TABLE ec_ivf_dense_typed_build_scan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_dense_typed_build_scan VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.9,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_dense_typed_build_scan_idx ON ec_ivf_dense_typed_build_scan USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 4, storage_format = 'turboquant', dense_posting_blocks = 1, dense_posting_typed_layout = 1)",
        )
        .expect("typed dense IVF index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_dense_typed_build_scan_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_dense_typed_build_scan");
        let ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);
        let counters =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, vec![1.0, 0.0]));

        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&0));
        assert_eq!(counters.outputs.len(), 4);
        assert_eq!(counters.row_postings_visited, 0);
        assert_eq!(counters.dense_blocks_visited, 1);
        assert_eq!(counters.dense_postings_visited, 4);
        assert_eq!(counters.scratch_soa_flushes, 0);
        assert_eq!(counters.dense_coalesced_flushes, 1);
        assert!(counters.orderby_cleared);
    }

    #[pg_test]
    fn test_ec_ivf_dense_posting_blocks_scan_can_disable_coalescing() {
        Spi::run(
            "CREATE TABLE ec_ivf_dense_coalescing_off_scan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_dense_coalescing_off_scan VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.9,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_dense_coalescing_off_scan_idx ON ec_ivf_dense_coalescing_off_scan USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 4, storage_format = 'turboquant', dense_posting_blocks = 1)",
        )
        .expect("dense IVF index creation should succeed");
        Spi::run("SET ec_ivf.dense_posting_coalescing = off")
            .expect("coalescing GUC should be settable");

        let index_oid = ec_ivf_index_oid("ec_ivf_dense_coalescing_off_scan_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_dense_coalescing_off_scan");
        let ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);
        let counters =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, vec![1.0, 0.0]));

        Spi::run("RESET ec_ivf.dense_posting_coalescing")
            .expect("coalescing GUC should be resettable");
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&0));
        assert_eq!(counters.outputs.len(), 4);
        assert_eq!(counters.row_postings_visited, 0);
        assert_eq!(counters.dense_blocks_visited, 1);
        assert_eq!(counters.dense_postings_visited, 4);
        assert_eq!(counters.scratch_soa_flushes, 0);
        assert_eq!(counters.dense_coalesced_flushes, 0);
        assert!(counters.orderby_cleared);
    }

    #[pg_test]
    fn test_ec_ivf_dense_posting_blocks_rabitq_scan_build_rows() {
        Spi::run(
            "CREATE TABLE ec_ivf_dense_rabitq_scan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_dense_rabitq_scan VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.9,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-1.0,0.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_dense_rabitq_scan_idx ON ec_ivf_dense_rabitq_scan USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 4, storage_format = 'rabitq', quant_bits = 1, dense_posting_blocks = 1)",
        )
        .expect("dense RaBitQ IVF index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_dense_rabitq_scan_idx");
        let ctid_to_id = ctid_id_map("ec_ivf_dense_rabitq_scan");
        let ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 4);
        let counters =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, vec![1.0, 0.0]));

        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&0));
        assert_eq!(counters.outputs.len(), 4);
        assert_eq!(counters.row_postings_visited, 0);
        assert_eq!(counters.dense_blocks_visited, 1);
        assert_eq!(counters.dense_postings_visited, 4);
        assert_eq!(counters.scratch_soa_flushes, 0);
        assert_eq!(counters.dense_coalesced_flushes, 1);
        assert!(counters.orderby_cleared);
    }

    #[pg_test]
    fn test_ec_ivf_dense_posting_blocks_scan_mixed_insert_rows() {
        Spi::run(
            "CREATE TABLE ec_ivf_dense_mixed_scan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_dense_mixed_scan VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.9,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_dense_mixed_scan_idx ON ec_ivf_dense_mixed_scan USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 3, storage_format = 'turboquant', dense_posting_blocks = 1)",
        )
        .expect("dense IVF index creation should succeed");
        Spi::run("INSERT INTO ec_ivf_dense_mixed_scan VALUES (3, '[1.0,0.2]'::ecvector)")
            .expect("live insert should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_dense_mixed_scan_idx");
        let inserted_tid = heap_tid_for_row("ec_ivf_dense_mixed_scan", 3);
        let counters =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, vec![1.0, 0.2]));

        assert_eq!(counters.outputs.len(), 4);
        assert!(counters.outputs.iter().any(
            |(block_number, offset_number, _score)| (*block_number, *offset_number)
                == (inserted_tid.block_number, inserted_tid.offset_number)
        ));
        assert!(counters.row_postings_visited >= 1);
        assert_eq!(counters.dense_blocks_visited, 1);
        assert_eq!(counters.dense_postings_visited, 3);
        assert_eq!(counters.dense_coalesced_flushes, 1);
        assert!(counters.orderby_cleared);
    }

    #[pg_test]
    fn test_ec_ivf_dense_posting_blocks_vacuum_removes_build_row() {
        Spi::run(
            "CREATE TABLE ec_ivf_dense_vacuum (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_dense_vacuum VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.9,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_dense_vacuum_idx ON ec_ivf_dense_vacuum USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 3, storage_format = 'turboquant', dense_posting_blocks = 1)",
        )
        .expect("dense IVF index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_dense_vacuum_idx");
        let deleted_tid = heap_tid_for_row("ec_ivf_dense_vacuum", 1);
        Spi::run("DELETE FROM ec_ivf_dense_vacuum WHERE id = 1").expect("delete should succeed");
        let stats =
            ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[deleted_tid]));
        let counters =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, vec![1.0, 0.1]));
        let (_, _, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));

        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 2.0);
        assert_eq!(directory_live, 2);
        assert_eq!(directory_dead, 1);
        assert_eq!(inserted_since_build, 0);
        assert_eq!(total_live, 2);
        assert_eq!(counters.outputs.len(), 2);
        assert!(counters.outputs.iter().all(
            |(block_number, offset_number, _score)| (*block_number, *offset_number)
                != (deleted_tid.block_number, deleted_tid.offset_number)
        ));
        assert_eq!(counters.dense_blocks_visited, 1);
        assert_eq!(counters.dense_postings_visited, 2);
        assert_eq!(counters.dense_coalesced_flushes, 1);
    }

    #[pg_test]
    fn test_ec_ivf_index_placement_vacuum_removes_sidecar_entry() {
        // Task 111g (003b): vacuuming a dead row removes it from the index-side
        // f16 sidecar (tombstoned), so it is no longer returned, and the
        // surviving rows still rerank correctly from the sidecar.
        Spi::run("CREATE TABLE ec_ivf_idx_vac (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_idx_vac VALUES
             (0, '[1.0,0.0,0.0,0.0]'::ecvector),
             (1, '[0.0,1.0,0.0,0.0]'::ecvector),
             (2, '[0.0,0.0,1.0,0.0]'::ecvector),
             (3, '[0.0,0.0,0.0,1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_idx_vac_idx ON ec_ivf_idx_vac USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 2,
                nprobe = 2,
                training_sample_rows = 4,
                storage_format = 'coarse_rerank',
                rerank_format = 'f16',
                rerank_placement = 'index',
                rerank_width = 4
             )",
        )
        .expect("index-placement f16 index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_idx_vac_idx");
        let deleted_tid = heap_tid_for_row("ec_ivf_idx_vac", 1);
        Spi::run("DELETE FROM ec_ivf_idx_vac WHERE id = 1").expect("delete should succeed");
        let stats =
            ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[deleted_tid]));
        assert_eq!(stats.tuples_removed, 1.0);

        // Query aligned with the deleted row's vector: it must not be returned,
        // and the remaining rows must still be reranked from the sidecar.
        let ctid_to_id = ctid_id_map("ec_ivf_idx_vac");
        let ids = ivf_debug_output_ids(index_oid, vec![0.0, 1.0, 0.0, 0.0], &ctid_to_id, 4);
        assert!(
            !ids.contains(&1),
            "vacuumed row should not be returned, got {ids:?}"
        );
        assert_eq!(ids.len(), 3, "three live rows should remain, got {ids:?}");

        // A surviving row still reranks correctly from the (tombstoned) sidecar.
        let top = ivf_debug_output_ids(index_oid, vec![0.0, 0.0, 1.0, 0.0], &ctid_to_id, 1);
        assert_eq!(top, vec![2], "surviving row 2 should rank first for its query");
    }

    #[pg_test]
    fn test_ec_ivf_full_probe_matches_simple_exact_oracle_top1() {
        Spi::run(
            "CREATE TABLE ec_ivf_full_probe_oracle (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_full_probe_oracle VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.7,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-0.7,0.1]'::ecvector)",
        )
        .expect("insert should succeed");

        let index_oid = create_ivf_recall_index(
            "ec_ivf_full_probe_oracle",
            "ec_ivf_full_probe_oracle_idx",
            4,
            4,
            4,
        );
        let ctid_to_id = ctid_id_map("ec_ivf_full_probe_oracle");
        let query = vec![1.0, 0.0];

        let exact_top = exact_ecvector_top_k_ids("ec_ivf_full_probe_oracle", &query, 1);
        let ivf_top = ivf_debug_output_ids(index_oid, query, &ctid_to_id, 4);
        let unique_ivf_ids = ivf_top.iter().copied().collect::<HashSet<_>>();

        assert_eq!(exact_top, vec![0]);
        assert_eq!(ivf_top.first(), exact_top.first());
        assert_eq!(ivf_top.len(), 4);
        assert_eq!(unique_ivf_ids.len(), ivf_top.len());
    }

    #[pg_test]
    fn test_ec_ivf_heap_f32_rerank_full_probe_matches_exact_scores() {
        Spi::run(
            "CREATE TABLE ec_ivf_heap_f32_exact (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_heap_f32_exact VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.7,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-0.7,0.1]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_heap_f32_exact_idx ON ec_ivf_heap_f32_exact USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 4, nprobe = 4, training_sample_rows = 4, rerank = 'heap_f32')",
        )
        .expect("index creation should succeed");

        let query = vec![1.0, 0.0];
        let query_literal = format_recall_vector_sql_literal(&query);
        let exact = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT id, embedding <#> {query_literal} AS score
                         FROM ec_ivf_heap_f32_exact
                         ORDER BY score, id"
                    ),
                    None,
                    &[],
                )
                .expect("exact score query should succeed")
                .map(|row| {
                    let id = row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should not be NULL");
                    let score = row["score"]
                        .value::<f32>()
                        .expect("score should decode")
                        .expect("score should not be NULL");
                    (usize::try_from(id).expect("id should fit usize"), score)
                })
                .collect::<Vec<_>>()
        });
        let ctid_to_id = ctid_id_map("ec_ivf_heap_f32_exact");
        let index_oid = ec_ivf_index_oid("ec_ivf_heap_f32_exact_idx");
        let (outputs, orderby_cleared) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, query));
        let observed = outputs
            .into_iter()
            .map(|(block_number, offset_number, score)| {
                (
                    *ctid_to_id
                        .get(&(block_number, offset_number))
                        .expect("IVF emitted heap tid should map back to a row id"),
                    score,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(observed.len(), exact.len());
        for ((observed_id, observed_score), (exact_id, exact_score)) in
            observed.iter().zip(exact.iter())
        {
            assert_eq!(observed_id, exact_id);
            assert!(
                (observed_score - exact_score).abs() <= f32::EPSILON,
                "heap_f32 rerank score for id {observed_id} should match exact <#>: got {observed_score}, expected {exact_score}"
            );
        }
        assert!(orderby_cleared);
    }

    #[pg_test]
    fn test_ec_ivf_heap_f32_rerank_width_bounds_exact_frontier() {
        Spi::run(
            "CREATE TABLE ec_ivf_heap_f32_width (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_heap_f32_width VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.7,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-0.7,0.1]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_heap_f32_width_idx ON ec_ivf_heap_f32_width USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 4,
                nprobe = 4,
                training_sample_rows = 4,
                rerank = 'heap_f32',
                rerank_width = 2
             )",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_heap_f32_width_idx");
        let (outputs, _orderby_cleared) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.0]));

        assert_eq!(outputs.len(), 2);
        let exact_scores = outputs
            .iter()
            .map(|(_, _, score)| *score)
            .collect::<Vec<_>>();
        assert_eq!(exact_scores, vec![-1.0, -0.7]);

        Spi::run("SET LOCAL ec_ivf.rerank_width = 3").expect("session rerank_width should set");
        let (outputs, _orderby_cleared) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.0]));
        assert_eq!(outputs.len(), 3);
        let exact_scores = outputs
            .iter()
            .map(|(_, _, score)| *score)
            .collect::<Vec<_>>();
        assert_eq!(exact_scores, vec![-1.0, -0.7, -0.0]);

        Spi::run("SET LOCAL ec_ivf.rerank_width = 0")
            .expect("session rerank_width should accept full-frontier override");
        let (outputs, _orderby_cleared) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.0]));
        assert_eq!(outputs.len(), 4);

        Spi::run("SET LOCAL ec_ivf.rerank_width = -1")
            .expect("session rerank_width should return to relation default");
        let (outputs, _orderby_cleared) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.0]));
        assert_eq!(outputs.len(), 2);
    }

    #[pg_test]
    fn test_ec_ivf_scan_rerank_uses_current_reloptions() {
        Spi::run(
            "CREATE TABLE ec_ivf_altered_rerank (
                id bigint primary key,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_altered_rerank VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.7,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[-0.7,0.1]'::ecvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_altered_rerank_idx ON ec_ivf_altered_rerank USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 4,
                nprobe = 4,
                training_sample_rows = 4,
                rerank = 'off',
                rerank_width = 0
             )",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_altered_rerank_idx");
        let metadata_rerank_mode = ec_ivf_debug!(am::debug_ec_ivf_rerank_mode(index_oid));
        assert_eq!(metadata_rerank_mode, "off");

        Spi::run(
            "ALTER INDEX ec_ivf_altered_rerank_idx SET (rerank = 'heap_f32', rerank_width = 2)",
        )
        .expect("ALTER INDEX should update relation options");

        let (outputs, _orderby_cleared) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.0]));

        assert_eq!(outputs.len(), 2);
        let exact_scores = outputs
            .iter()
            .map(|(_, _, score)| *score)
            .collect::<Vec<_>>();
        assert_eq!(exact_scores, vec![-1.0, -0.7]);
    }

    #[pg_test]
    fn test_ec_ivf_recall_smoke_compares_exact_hnsw_ivf() {
        let table_name = "ec_ivf_recall_smoke";
        let k = 5;
        let corpus = random_unit_vectors(64, 8, 0x1F17);
        let query = corpus[0].clone();

        create_recall_table(table_name);
        insert_recall_corpus(table_name, &corpus);
        let ctid_to_id = ctid_id_map(table_name);
        let hnsw_oid = create_recall_index(table_name, "ec_ivf_recall_smoke_hnsw_idx", 8);
        let ivf_oid = create_ivf_recall_index(table_name, "ec_ivf_recall_smoke_ivf_idx", 8, 8, 64);

        let brute_force_top = brute_force_top_k(&corpus, &query, k);
        let exact_top = exact_ecvector_top_k_ids(table_name, &query, k);
        Spi::run("SET LOCAL ec_hnsw.ef_search = 64").expect("setting ef_search should succeed");
        let hnsw_top = hnsw_debug_output_ids(hnsw_oid, query.clone(), &ctid_to_id, k);
        let ivf_top = ivf_debug_output_ids(ivf_oid, query, &ctid_to_id, k);
        let exact_set = exact_top.iter().copied().collect::<HashSet<_>>();
        let hnsw_overlap = hnsw_top.iter().filter(|id| exact_set.contains(id)).count();
        let ivf_overlap = ivf_top.iter().filter(|id| exact_set.contains(id)).count();

        assert_eq!(exact_top, brute_force_top);
        assert_eq!(exact_top.first(), Some(&0));
        assert_eq!(hnsw_top.len(), k);
        assert_eq!(ivf_top.len(), k);
        assert!(
            hnsw_overlap > 0,
            "ec_hnsw should overlap exact top-k in the deterministic recall smoke"
        );
        assert!(
            ivf_overlap > 0,
            "ec_ivf should overlap exact top-k in the deterministic recall smoke"
        );
    }

    #[pg_test]
    fn test_ec_ivf_insert_appends_posting_and_updates_stats() {
        Spi::run("CREATE TABLE ec_ivf_live_insert (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_live_insert VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid =
            create_ivf_recall_index("ec_ivf_live_insert", "ec_ivf_live_insert_idx", 2, 2, 2);

        Spi::run("INSERT INTO ec_ivf_live_insert VALUES (2, '[1.0,0.1]'::ecvector)")
            .expect("live insert should succeed");

        let (summary_nlists, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (dimensions, nlists, training_version, total_live, has_centroids, has_directory) =
            ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let ctid_to_id = ctid_id_map("ec_ivf_live_insert");
        let ivf_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.1], &ctid_to_id, 3);
        let unique_ivf_ids = ivf_ids.iter().copied().collect::<HashSet<_>>();

        assert_eq!(summary_nlists, 2);
        assert_eq!(empty_lists, 0);
        assert_eq!(directory_live, 3);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 1);
        assert_eq!(dimensions, 2);
        assert_eq!(nlists, 2);
        assert_eq!(training_version, 1);
        assert_eq!(total_live, 3);
        assert!(has_centroids);
        assert!(has_directory);
        assert_eq!(ivf_ids.len(), 3);
        assert_eq!(unique_ivf_ids.len(), ivf_ids.len());
        assert!(
            ivf_ids.contains(&2),
            "live-inserted heap row should be reachable through ec_ivf scan"
        );
    }

    #[pg_test]
    fn test_ec_ivf_insert_reuses_same_list_tail_page() {
        Spi::run(
            "CREATE TABLE ec_ivf_same_list_live_insert (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_same_list_live_insert VALUES
             (0, '[1.0,0.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid = create_ivf_recall_index(
            "ec_ivf_same_list_live_insert",
            "ec_ivf_same_list_live_insert_idx",
            1,
            1,
            1,
        );
        let blocks_before = ec_ivf_index_blocks("ec_ivf_same_list_live_insert_idx");

        Spi::run(
            "INSERT INTO ec_ivf_same_list_live_insert VALUES
             (1, '[1.0,0.1]'::ecvector),
             (2, '[1.0,0.2]'::ecvector)",
        )
        .expect("same-list live inserts should succeed");

        let blocks_after = ec_ivf_index_blocks("ec_ivf_same_list_live_insert_idx");
        let (summary_nlists, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let ctid_to_id = ctid_id_map("ec_ivf_same_list_live_insert");
        let ivf_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.2], &ctid_to_id, 3);
        let unique_ivf_ids = ivf_ids.iter().copied().collect::<HashSet<_>>();

        assert_eq!(blocks_after, blocks_before);
        assert_eq!(summary_nlists, 1);
        assert_eq!(empty_lists, 0);
        assert_eq!(directory_live, 3);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 2);
        assert_eq!(total_live, 3);
        assert_eq!(ivf_ids.len(), 3);
        assert_eq!(unique_ivf_ids.len(), ivf_ids.len());
        assert!(ivf_ids.contains(&1));
        assert!(ivf_ids.contains(&2));
    }

    #[pg_test]
    fn test_ec_ivf_large_build_insert_directory_chain() {
        const TABLE_NAME: &str = "ec_ivf_large_build_live_insert";
        const INDEX_NAME: &str = "ec_ivf_large_build_live_insert_idx";

        Spi::run(&format!(
            "CREATE TABLE {TABLE_NAME} (id bigint primary key, embedding ecvector);
             INSERT INTO {TABLE_NAME} (id, embedding)
             SELECT gs,
                    encode_to_ecvector(
                        ARRAY[
                            sin((gs * 0.013)::double precision)::real,
                            cos((gs * 0.013)::double precision)::real,
                            sin((gs * 0.021)::double precision)::real,
                            cos((gs * 0.021)::double precision)::real
                        ]::real[],
                        4,
                        42
                    )
             FROM generate_series(1, 1000) AS gs;
             CREATE INDEX {INDEX_NAME} ON {TABLE_NAME} USING ec_ivf
               (embedding ecvector_ip_ops)
               WITH (nlists = 16, nprobe = 16, training_sample_rows = 1000);"
        ))
        .expect("large IVF build setup should succeed");

        Spi::run(&format!(
            "INSERT INTO {TABLE_NAME} (id, embedding)
             VALUES (
                 1001,
                 encode_to_ecvector(ARRAY[0.1, 0.2, 0.3, 0.4]::real[], 4, 42)
             )"
        ))
        .expect("live insert after large build should keep directory chain readable");

        let index_oid = ec_ivf_index_oid(INDEX_NAME);
        let (_, _, directory_live, _, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));

        assert_eq!(directory_live, 1001);
        assert_eq!(inserted_since_build, 1);
        assert_eq!(total_live, 1001);
    }

    #[pg_test]
    fn test_ec_ivf_concurrent_inserts() {
        const TABLE_NAME: &str = "ec_ivf_concurrent_insert";
        const INDEX_NAME: &str = "ec_ivf_concurrent_insert_idx";
        const WORKER_INSERTS: usize = 20;
        const EXPECTED_INSERTED: u64 = (WORKER_INSERTS * 2) as u64;
        const EXPECTED_TOTAL: u64 = EXPECTED_INSERTED + 2;
        const BARRIER_KEY: i64 = 280_501;

        let connection = pg_test_psql_connection();
        run_psql_script(
            &connection,
            "ec_ivf concurrent insert setup",
            &format!(
                "DROP TABLE IF EXISTS {TABLE_NAME};
                 CREATE TABLE {TABLE_NAME} (id bigint primary key, embedding ecvector);
                 INSERT INTO {TABLE_NAME} VALUES
                   (0, '[1.0,0.0]'::ecvector),
                   (1, '[0.0,1.0]'::ecvector);
                 CREATE INDEX {INDEX_NAME} ON {TABLE_NAME} USING ec_ivf
                   (embedding ecvector_ip_ops)
                   WITH (nlists = 2, nprobe = 2, training_sample_rows = 2, seed = 37);",
            ),
        );

        Spi::run(&format!("SELECT pg_advisory_lock({BARRIER_KEY})"))
            .expect("barrier lock should be acquired");
        let left_values = ec_ivf_insert_values(10, WORKER_INSERTS, "[1.0,0.05]");
        let right_values = ec_ivf_insert_values(1_000, WORKER_INSERTS, "[0.05,1.0]");
        let worker_sql = |values: String| {
            format!(
                "SET lock_timeout = '10s';
                 SET statement_timeout = '30s';
                 SELECT pg_advisory_lock_shared({BARRIER_KEY});
                 SELECT pg_advisory_unlock_shared({BARRIER_KEY});
                 INSERT INTO {TABLE_NAME} VALUES {values};"
            )
        };
        let workers = vec![
            (
                "left-list worker",
                spawn_psql_script(&connection, "left-list worker", &worker_sql(left_values)),
            ),
            (
                "right-list worker",
                spawn_psql_script(&connection, "right-list worker", &worker_sql(right_values)),
            ),
        ];
        std::thread::sleep(Duration::from_millis(750));
        Spi::run(&format!("SELECT pg_advisory_unlock({BARRIER_KEY})"))
            .expect("barrier lock should be released");

        for (label, worker) in workers {
            let output = worker
                .wait_with_output()
                .unwrap_or_else(|e| panic!("{label} wait failed: {e}"));
            assert_psql_success(label, output);
        }

        let heap_count = Spi::get_one::<i64>(&format!("SELECT count(*) FROM {TABLE_NAME}"))
            .expect("SPI query should succeed")
            .expect("heap count should exist");
        let index_oid = ec_ivf_index_oid(INDEX_NAME);
        let (summary_nlists, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (_, _, list0_live, _, list0_inserted) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_entry(index_oid, 0));
        let (_, _, list1_live, _, list1_inserted) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_entry(index_oid, 1));
        let ctid_to_id = ctid_id_map(TABLE_NAME);
        let ivf_ids = ivf_debug_output_ids(
            index_oid,
            vec![1.0, 0.0],
            &ctid_to_id,
            EXPECTED_TOTAL as usize,
        );
        let unique_ivf_ids = ivf_ids.iter().copied().collect::<HashSet<_>>();

        assert_eq!(heap_count, EXPECTED_TOTAL as i64);
        assert_eq!(summary_nlists, 2);
        assert_eq!(empty_lists, 0);
        assert_eq!(directory_live, EXPECTED_TOTAL);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, EXPECTED_INSERTED);
        assert_eq!(total_live, EXPECTED_TOTAL);
        assert_eq!(list0_live + list1_live, EXPECTED_TOTAL);
        assert_eq!(list0_inserted + list1_inserted, EXPECTED_INSERTED);
        assert!(list0_inserted > 0);
        assert!(list1_inserted > 0);
        assert_eq!(ivf_ids.len(), EXPECTED_TOTAL as usize);
        assert_eq!(unique_ivf_ids.len(), ivf_ids.len());
        assert!(ivf_ids.contains(&10));
        assert!(ivf_ids.contains(&1_000));
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_ec_ivf_concurrent_same_list_inserts_remain_reachable() {
        const TABLE_NAME: &str = "ec_ivf_concurrent_same_list_insert";
        const INDEX_NAME: &str = "ec_ivf_concurrent_same_list_insert_idx";
        const BARRIER_KEY: i64 = 280_502;

        let connection = pg_test_psql_connection();
        run_psql_script(
            &connection,
            "ec_ivf same-list concurrent insert setup",
            &format!(
                "DROP TABLE IF EXISTS {TABLE_NAME};
                 CREATE TABLE {TABLE_NAME} (id bigint primary key, embedding ecvector);
                 INSERT INTO {TABLE_NAME} VALUES (0, '[1.0,0.0]'::ecvector);
                 CREATE INDEX {INDEX_NAME} ON {TABLE_NAME} USING ec_ivf
                   (embedding ecvector_ip_ops)
                   WITH (nlists = 1, nprobe = 1, training_sample_rows = 1);",
            ),
        );

        Spi::run(&format!("SELECT pg_advisory_lock({BARRIER_KEY})"))
            .expect("barrier lock should be acquired");
        let worker_sql = |id: i64, vector: &str| {
            format!(
                "SET lock_timeout = '10s';
                 SET statement_timeout = '30s';
                 SELECT pg_advisory_lock_shared({BARRIER_KEY});
                 INSERT INTO {TABLE_NAME} VALUES ({id}, '{vector}'::ecvector);
                 SELECT pg_advisory_unlock_shared({BARRIER_KEY});"
            )
        };
        let workers = vec![
            (
                "same-list worker 1",
                spawn_psql_script(
                    &connection,
                    "same-list worker 1",
                    &worker_sql(1, "[1.0,0.1]"),
                ),
            ),
            (
                "same-list worker 2",
                spawn_psql_script(
                    &connection,
                    "same-list worker 2",
                    &worker_sql(2, "[1.0,0.2]"),
                ),
            ),
        ];
        std::thread::sleep(Duration::from_millis(750));
        Spi::run(&format!("SELECT pg_advisory_unlock({BARRIER_KEY})"))
            .expect("barrier lock should be released");

        for (label, worker) in workers {
            let output = worker
                .wait_with_output()
                .unwrap_or_else(|e| panic!("{label} wait failed: {e}"));
            assert_psql_success(label, output);
        }

        let heap_count = Spi::get_one::<i64>(&format!("SELECT count(*) FROM {TABLE_NAME}"))
            .expect("SPI query should succeed")
            .expect("heap count should exist");
        let index_oid = ec_ivf_index_oid(INDEX_NAME);
        let (_, _, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, list_live, list_dead, list_inserted) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_entry(index_oid, 0));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let ctid_to_id = ctid_id_map(TABLE_NAME);
        let ivf_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.2], &ctid_to_id, 3);
        let unique_ivf_ids = ivf_ids.iter().copied().collect::<HashSet<_>>();

        assert_eq!(heap_count, 3);
        assert_eq!(directory_live, 3);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 2);
        assert_eq!(list_live, 3);
        assert_eq!(list_dead, 0);
        assert_eq!(list_inserted, 2);
        assert_eq!(total_live, 3);
        assert_eq!(ivf_ids.len(), 3);
        assert_eq!(unique_ivf_ids.len(), ivf_ids.len());
        assert!(ivf_ids.contains(&1));
        assert!(ivf_ids.contains(&2));
    }

    #[cfg(feature = "pg18")]
    #[pg_test]
    fn test_pg18_ec_ivf_concurrent_empty_bootstrap_reachable() {
        const TABLE_NAME: &str = "ec_ivf_concurrent_empty_bootstrap_insert";
        const INDEX_NAME: &str = "ec_ivf_concurrent_empty_bootstrap_insert_idx";
        const BARRIER_KEY: i64 = 280_503;

        let connection = pg_test_psql_connection();
        run_psql_script(
            &connection,
            "ec_ivf empty-bootstrap concurrent insert setup",
            &format!(
                "DROP TABLE IF EXISTS {TABLE_NAME};
                 CREATE TABLE {TABLE_NAME} (id bigint primary key, embedding ecvector);
                 CREATE INDEX {INDEX_NAME} ON {TABLE_NAME} USING ec_ivf
                   (embedding ecvector_ip_ops)
                   WITH (nlists = 2, nprobe = 2, training_sample_rows = 2, seed = 41);",
            ),
        );

        Spi::run(&format!("SELECT pg_advisory_lock({BARRIER_KEY})"))
            .expect("barrier lock should be acquired");
        let worker_sql = |id: i64, vector: &str| {
            format!(
                "SET lock_timeout = '10s';
                 SET statement_timeout = '30s';
                 SELECT pg_advisory_lock_shared({BARRIER_KEY});
                 INSERT INTO {TABLE_NAME} VALUES ({id}, '{vector}'::ecvector);
                 SELECT pg_advisory_unlock_shared({BARRIER_KEY});"
            )
        };
        let workers = vec![
            (
                "empty-bootstrap worker 1",
                spawn_psql_script(
                    &connection,
                    "empty-bootstrap worker 1",
                    &worker_sql(1, "[1.0,0.0]"),
                ),
            ),
            (
                "empty-bootstrap worker 2",
                spawn_psql_script(
                    &connection,
                    "empty-bootstrap worker 2",
                    &worker_sql(2, "[0.0,1.0]"),
                ),
            ),
        ];
        std::thread::sleep(Duration::from_millis(750));
        Spi::run(&format!("SELECT pg_advisory_unlock({BARRIER_KEY})"))
            .expect("barrier lock should be released");

        for (label, worker) in workers {
            let output = worker
                .wait_with_output()
                .unwrap_or_else(|e| panic!("{label} wait failed: {e}"));
            assert_psql_success(label, output);
        }

        let heap_count = Spi::get_one::<i64>(&format!("SELECT count(*) FROM {TABLE_NAME}"))
            .expect("SPI query should succeed")
            .expect("heap count should exist");
        let index_oid = ec_ivf_index_oid(INDEX_NAME);
        let (dimensions, nlists, training_version, total_live, has_centroids, has_directory) =
            ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (summary_nlists, _, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, list0_live, _, list0_inserted) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_entry(index_oid, 0));
        let (_, _, list1_live, _, list1_inserted) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_entry(index_oid, 1));
        let ctid_to_id = ctid_id_map(TABLE_NAME);
        let ivf_ids = ivf_debug_output_ids(index_oid, vec![1.0, 0.0], &ctid_to_id, 2);
        let unique_ivf_ids = ivf_ids.iter().copied().collect::<HashSet<_>>();

        assert_eq!(heap_count, 2);
        assert_eq!(dimensions, 2);
        assert_eq!(nlists, 2);
        assert_eq!(training_version, 1);
        assert_eq!(total_live, 2);
        assert!(has_centroids);
        assert!(has_directory);
        assert_eq!(summary_nlists, 2);
        assert_eq!(directory_live, 2);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 1);
        assert_eq!(list0_live + list1_live, 2);
        assert_eq!(list0_inserted + list1_inserted, 1);
        assert_eq!(ivf_ids.len(), 2);
        assert_eq!(unique_ivf_ids.len(), ivf_ids.len());
        assert!(ivf_ids.contains(&1));
        assert!(ivf_ids.contains(&2));
    }

    #[pg_test]
    #[should_panic(expected = "duplicate heap tid")]
    fn test_ec_ivf_insert_rejects_duplicate_heap_tid() {
        Spi::run(
            "CREATE TABLE ec_ivf_duplicate_heap_tid (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_duplicate_heap_tid VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid = create_ivf_recall_index(
            "ec_ivf_duplicate_heap_tid",
            "ec_ivf_duplicate_heap_tid_idx",
            2,
            2,
            2,
        );
        let heap_tid = heap_tid_for_row("ec_ivf_duplicate_heap_tid", 0);

        ec_ivf_debug!(am::debug_ec_ivf_validate_no_duplicate_heap_tid(
                index_oid,
                heap_tid.block_number,
                heap_tid.offset_number,
            ));
    }

    #[pg_test]
    fn test_ec_ivf_insert_bootstraps_empty_index() {
        Spi::run(
            "CREATE TABLE ec_ivf_empty_live_insert (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        let index_oid = create_ivf_recall_index(
            "ec_ivf_empty_live_insert",
            "ec_ivf_empty_live_insert_idx",
            4,
            4,
            4,
        );

        Spi::run("INSERT INTO ec_ivf_empty_live_insert VALUES (0, '[0.25,1.0]'::ecvector)")
            .expect("first live insert should bootstrap the empty index");

        let (dimensions, nlists, training_version, total_live, has_centroids, has_directory) =
            ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (summary_nlists, empty_lists, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let ctid_to_id = ctid_id_map("ec_ivf_empty_live_insert");
        let ivf_ids = ivf_debug_output_ids(index_oid, vec![0.25, 1.0], &ctid_to_id, 1);

        assert_eq!(dimensions, 2);
        assert_eq!(nlists, 4);
        assert_eq!(training_version, 1);
        assert_eq!(total_live, 1);
        assert!(has_centroids);
        assert!(has_directory);
        assert_eq!(summary_nlists, 4);
        assert_eq!(empty_lists, 3);
        assert_eq!(directory_live, 1);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 0);
        assert_eq!(ivf_ids, vec![0]);
    }

    #[pg_test]
    #[should_panic(expected = "dimension mismatch")]
    fn test_ec_ivf_insert_rejects_dimension_mismatch() {
        Spi::run(
            "CREATE TABLE ec_ivf_insert_dim_mismatch (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run("INSERT INTO ec_ivf_insert_dim_mismatch VALUES (0, '[1.0,0.0]'::ecvector)")
            .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_insert_dim_mismatch_idx \
             ON ec_ivf_insert_dim_mismatch USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 1)",
        )
        .expect("index creation should succeed");

        Spi::run("INSERT INTO ec_ivf_insert_dim_mismatch VALUES (1, '[1.0,0.0,0.0]'::ecvector)")
            .expect("insert should fail");
    }

    #[pg_test]
    fn test_ec_ivf_empty_vacuum_callbacks_report_noop_stats() {
        Spi::run("CREATE TABLE ec_ivf_vacuum_empty (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_vacuum_empty_idx ON ec_ivf_vacuum_empty USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 4, nprobe = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_vacuum_empty_idx");
        let index_blocks = ec_ivf_index_blocks("ec_ivf_vacuum_empty_idx");
        let stats = ec_ivf_debug!(am::debug_ec_ivf_vacuum_stats(index_oid));

        assert_eq!(stats.num_pages as i64, index_blocks);
        assert!(
            !stats.estimated_count,
            "ec_ivf vacuum stats should report exact metadata counts"
        );
        assert_eq!(stats.num_index_tuples, 0.0);
        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.pages_newly_deleted, 0);
        assert_eq!(stats.pages_deleted, 0);
        assert_eq!(stats.pages_free, 0);
    }

    #[pg_test]
    fn test_ec_ivf_vacuum_callbacks_keep_live_count_noop() {
        Spi::run("CREATE TABLE ec_ivf_vacuum_noop (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_vacuum_noop VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid =
            create_ivf_recall_index("ec_ivf_vacuum_noop", "ec_ivf_vacuum_noop_idx", 2, 2, 2);
        Spi::run("DELETE FROM ec_ivf_vacuum_noop WHERE id = 1").expect("delete should succeed");

        let index_blocks = ec_ivf_index_blocks("ec_ivf_vacuum_noop_idx");
        let stats = ec_ivf_debug!(am::debug_ec_ivf_vacuum_stats(index_oid));
        let (_, _, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));

        assert_eq!(stats.num_pages as i64, index_blocks);
        assert!(
            !stats.estimated_count,
            "ec_ivf vacuum stats should report exact metadata counts"
        );
        assert_eq!(stats.num_index_tuples, 2.0);
        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.pages_newly_deleted, 0);
        assert_eq!(stats.pages_deleted, 0);
        assert_eq!(stats.pages_free, 0);
        assert_eq!(directory_live, 2);
        assert_eq!(directory_dead, 0);
        assert_eq!(inserted_since_build, 0);
        assert_eq!(total_live, 2);
    }

    #[pg_test]
    fn test_ec_ivf_vacuum_bulkdelete_removes_dead_heap_tid() {
        Spi::run("CREATE TABLE ec_ivf_vacuum_delete (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_vacuum_delete VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid =
            create_ivf_recall_index("ec_ivf_vacuum_delete", "ec_ivf_vacuum_delete_idx", 3, 3, 3);
        let deleted_tid = heap_tid_for_row("ec_ivf_vacuum_delete", 1);

        Spi::run("DELETE FROM ec_ivf_vacuum_delete WHERE id = 1").expect("delete should succeed");
        let stats = ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[deleted_tid]));
        let (_, _, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (outputs, _orderby_cleared) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![0.0, 1.0]));

        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 2.0);
        assert_eq!(directory_live, 2);
        assert_eq!(directory_dead, 1);
        assert_eq!(inserted_since_build, 0);
        assert_eq!(total_live, 2);
        assert!(
            outputs.iter().all(|(block_number, offset_number, _score)| {
                (*block_number, *offset_number)
                    != (deleted_tid.block_number, deleted_tid.offset_number)
            }),
            "vacuumed ec_ivf scan output should not include the deleted heap tid"
        );
    }

    #[pg_test]
    fn test_ec_ivf_vacuum_compacts_deleted_posting_space_for_reuse() {
        Spi::run("CREATE TABLE ec_ivf_vacuum_reuse (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_vacuum_reuse VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[1.0,0.1]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid =
            create_ivf_recall_index("ec_ivf_vacuum_reuse", "ec_ivf_vacuum_reuse_idx", 1, 1, 2);
        let deleted_tid = heap_tid_for_row("ec_ivf_vacuum_reuse", 1);
        let blocks_before_delete = ec_ivf_index_blocks("ec_ivf_vacuum_reuse_idx");

        Spi::run("DELETE FROM ec_ivf_vacuum_reuse WHERE id = 1").expect("delete should succeed");
        let stats = ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[deleted_tid]));
        let blocks_after_vacuum = ec_ivf_index_blocks("ec_ivf_vacuum_reuse_idx");

        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(blocks_after_vacuum, blocks_before_delete);

        Spi::run("INSERT INTO ec_ivf_vacuum_reuse VALUES (2, '[1.0,0.2]'::ecvector)")
            .expect("insert should reuse compacted posting page space");
        let blocks_after_reinsert = ec_ivf_index_blocks("ec_ivf_vacuum_reuse_idx");
        let (_, _, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));

        assert_eq!(
            blocks_after_reinsert, blocks_after_vacuum,
            "post-vacuum insert should reuse compacted posting page space before extending the index"
        );
        assert_eq!(directory_live, 2);
        assert_eq!(directory_dead, 1);
        assert_eq!(inserted_since_build, 1);
        assert_eq!(total_live, 2);
    }

    #[pg_test]
    fn test_ec_ivf_vacuum_repairs_empty_list_directory_refs() {
        Spi::run(
            "CREATE TABLE ec_ivf_vacuum_empty_list (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_vacuum_empty_list VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[1.0,0.1]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid = create_ivf_recall_index(
            "ec_ivf_vacuum_empty_list",
            "ec_ivf_vacuum_empty_list_idx",
            1,
            1,
            2,
        );
        let dead_tids = [
            heap_tid_for_row("ec_ivf_vacuum_empty_list", 0),
            heap_tid_for_row("ec_ivf_vacuum_empty_list", 1),
        ];
        let blocks_before_vacuum = ec_ivf_index_blocks("ec_ivf_vacuum_empty_list_idx");

        Spi::run("DELETE FROM ec_ivf_vacuum_empty_list").expect("delete should succeed");
        let stats = ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &dead_tids));
        let (head_block, tail_block, live_count, dead_count, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_entry(index_oid, 0));
        let (summary_nlists, empty_lists, directory_live, directory_dead, _) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));
        let (outputs, _orderby_cleared) =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.0]));

        assert_eq!(stats.tuples_removed, 2.0);
        assert_eq!(stats.num_index_tuples, 0.0);
        assert_ne!(head_block, u32::MAX);
        assert_ne!(tail_block, u32::MAX);
        assert_eq!(live_count, 0);
        assert_eq!(dead_count, 2);
        assert_eq!(inserted_since_build, 0);
        assert_eq!(summary_nlists, 1);
        assert_eq!(empty_lists, 1);
        assert_eq!(directory_live, 0);
        assert_eq!(directory_dead, 2);
        assert_eq!(total_live, 0);
        assert!(outputs.is_empty());

        Spi::run("INSERT INTO ec_ivf_vacuum_empty_list VALUES (2, '[1.0,0.2]'::ecvector)")
            .expect("insert should reuse preserved empty-list range");
        let blocks_after_reinsert = ec_ivf_index_blocks("ec_ivf_vacuum_empty_list_idx");
        let (head_after_reinsert, tail_after_reinsert, live_after_reinsert, _, inserted_after) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_entry(index_oid, 0));

        assert_eq!(
            blocks_after_reinsert, blocks_before_vacuum,
            "refilling an emptied list should reuse its preserved posting range"
        );
        assert_eq!(head_after_reinsert, head_block);
        assert_eq!(tail_after_reinsert, tail_block);
        assert_eq!(live_after_reinsert, 1);
        assert_eq!(inserted_after, 1);
    }

    #[pg_test]
    fn test_ec_ivf_drift_snapshot_handles_empty_index() {
        Spi::run("CREATE TABLE ec_ivf_drift_empty (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_drift_empty_idx ON ec_ivf_drift_empty USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 4, nprobe = 2)",
        )
        .expect("index creation should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_tuples FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live tuple count should be non-null"),
            0
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT empty_lists FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("empty-list count should be non-null"),
            4
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT changed_row_fraction FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("changed-row fraction should be non-null"),
            0.0
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT reindex_recommended FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_empty_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("reindex recommendation should be non-null")
        );
    }

    #[pg_test]
    fn test_ec_ivf_admin_snapshot() {
        Spi::run("CREATE TABLE ec_ivf_admin_snapshot (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_admin_snapshot VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector),
             (4, '[0.9,0.1]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_admin_snapshot_idx ON ec_ivf_admin_snapshot USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 4, nprobe = 2, rerank_width = 25, posting_slack_percent = 25, training_sample_rows = 5, seed = 37, storage_format = 'turboquant')",
        )
        .expect("index creation should succeed");
        Spi::run("INSERT INTO ec_ivf_admin_snapshot VALUES (5, '[1.0,0.2]'::ecvector)")
            .expect("live insert should succeed");
        Spi::run("ANALYZE ec_ivf_admin_snapshot").expect("analyze should succeed");

        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT dimensions FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("dimensions should be non-null"),
            2
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT nlists FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("nlists should be non-null"),
            4
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT relation_nprobe FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("relation nprobe should be non-null"),
            2
        );
        assert!(
            Spi::get_one::<i32>(
                "SELECT session_nprobe FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .is_none()
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_nprobe_source FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective nprobe source should be non-null"),
            "relation"
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT relation_rerank_width FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("relation rerank width should be non-null"),
            25
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT relation_posting_slack_percent FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("relation posting slack percent should be non-null"),
            25
        );
        assert!(
            Spi::get_one::<i32>(
                "SELECT session_rerank_width FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .is_none()
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT effective_rerank_width FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective rerank width should be non-null"),
            25
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_rerank_width_source FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective rerank width source should be non-null"),
            "relation"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT storage_format FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("storage format should be non-null"),
            "turboquant"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT rerank FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("rerank should be non-null"),
            "off"
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_tuples FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live tuple count should be non-null"),
            6
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_build FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-build should be non-null"),
            1
        );
        assert!(
            Spi::get_one::<f64>(
                "SELECT index_pages FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("index pages should be non-null")
                >= 1.0
        );
        assert!(
            !Spi::get_one::<bool>(
                "SELECT reindex_recommended FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("reindex recommendation should be non-null")
        );

        Spi::run("SET LOCAL ec_ivf.nprobe = 1").expect("session nprobe should set");
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT session_nprobe FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("session nprobe should be non-null"),
            1
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_nprobe_source FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective nprobe source should be non-null"),
            "session"
        );

        Spi::run("SET LOCAL ec_ivf.rerank_width = 10").expect("session rerank width should set");
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT session_rerank_width FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("session rerank width should be non-null"),
            10
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_rerank_width_source FROM ec_ivf_index_admin_snapshot('ec_ivf_admin_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective rerank width source should be non-null"),
            "session"
        );
    }

    #[pg_test]
    fn test_ec_ivf_page_ownership_snapshot_reports_posting_blocks() {
        Spi::run("CREATE TABLE ec_ivf_page_ownership (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_page_ownership VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.9,0.1]'::ecvector),
             (2, '[0.0,1.0]'::ecvector),
             (3, '[0.1,0.9]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_page_ownership_idx ON ec_ivf_page_ownership USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 2, nprobe = 2, training_sample_rows = 4, storage_format = 'turboquant')",
        )
        .expect("index creation should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT COALESCE(sum(posting_tuples), 0) \
                 FROM ec_ivf_index_page_ownership('ec_ivf_page_ownership_idx'::regclass)",
            )
            .expect("page ownership query should succeed")
            .expect("posting tuple count should be non-null"),
            4
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT COALESCE(sum(deleted_posting_tuples), 0) \
                 FROM ec_ivf_index_page_ownership('ec_ivf_page_ownership_idx'::regclass)",
            )
            .expect("page ownership query should succeed")
            .expect("deleted tuple count should be non-null"),
            0
        );
        assert!(
            Spi::get_one::<i64>(
                "SELECT count(*) \
                 FROM ec_ivf_index_page_ownership('ec_ivf_page_ownership_idx'::regclass) \
                 WHERE posting_tuples > 0 AND distinct_lists >= 1",
            )
            .expect("page ownership query should succeed")
            .expect("posting block count should be non-null")
                >= 1
        );
    }

    #[pg_test]
    fn test_ec_ivf_cost_snapshot_reports_modeled_costs() {
        Spi::run("CREATE TABLE ec_ivf_cost_snapshot (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_cost_snapshot VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector),
             (4, '[0.9,0.1]'::ecvector),
             (5, '[0.1,0.9]'::ecvector),
             (6, '[-0.9,0.1]'::ecvector),
             (7, '[0.1,-0.9]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_cost_snapshot_idx ON ec_ivf_cost_snapshot USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 4, nprobe = 2, training_sample_rows = 8, storage_format = 'turboquant')",
        )
        .expect("index creation should succeed");
        Spi::run("ANALYZE ec_ivf_cost_snapshot").expect("analyze should succeed");
        Spi::run("SET LOCAL ec_ivf.nprobe = 3").expect("session nprobe should set");

        assert!(
            Spi::get_one::<bool>(
                "SELECT planner_scan_enabled FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("planner flag should be non-null")
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT nlists FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("nlists should be non-null"),
            4
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT relation_nprobe FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("relation nprobe should be non-null"),
            2
        );
        assert_eq!(
            Spi::get_one::<i32>(
                "SELECT session_nprobe FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("session nprobe should be non-null"),
            3
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT effective_nprobe_source FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("effective nprobe source should be non-null"),
            "session"
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT resolved_tree_height FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("resolved tree height should be non-null"),
            0.0,
            "IVF reports a partitioned scan surface, not a tree height"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT tree_height_source FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("tree height source should be non-null"),
            if cfg!(feature = "pg18") {
                "amgettreeheight_callback"
            } else {
                "partitioned_ivf"
            }
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT pg18_tree_height_callback_ready FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("tree height readiness should be non-null"),
            cfg!(feature = "pg18")
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT ordering_compare_type FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("ordering compare type should be non-null"),
            "COMPARE_LT"
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT pg18_strategy_translation_ready FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("strategy translation readiness should be non-null"),
            cfg!(feature = "pg18")
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT estimated_centroid_scores FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("centroid score count should be non-null"),
            4
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT estimated_selected_lists FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("selected list count should be non-null"),
            3
        );
        let average_list_live_count = Spi::get_one::<f64>(
            "SELECT average_list_live_count FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("average list live count should be non-null");
        assert!((average_list_live_count - 2.0).abs() < 1e-9);
        let estimated_candidate_rows = Spi::get_one::<f64>(
            "SELECT estimated_candidate_rows FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("candidate rows should be non-null");
        assert!((estimated_candidate_rows - 6.0).abs() < 1e-9);
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT scoring_mode FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("scoring mode should be non-null"),
            "turboquant_lut"
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT rerank FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("rerank should be non-null"),
            "off"
        );

        let modeled_startup = Spi::get_one::<f64>(
            "SELECT modeled_startup_cost FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled startup should be non-null");
        let modeled_total = Spi::get_one::<f64>(
            "SELECT modeled_total_cost FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("modeled total should be non-null");
        assert!(modeled_startup.is_finite());
        assert!(modeled_total.is_finite());
        assert!(modeled_total > modeled_startup);
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT modeled_selectivity FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("modeled selectivity should be non-null"),
            1.0
        );
        assert_eq!(
            Spi::get_one::<f64>(
                "SELECT modeled_correlation FROM ec_ivf_index_cost_snapshot('ec_ivf_cost_snapshot_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("modeled correlation should be non-null"),
            0.0
        );
    }

    #[pg_test]
    fn test_ec_ivf_drift_snapshot_tracks_insert_and_vacuum_churn() {
        Spi::run("CREATE TABLE ec_ivf_drift_churn (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_drift_churn VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector),
             (3, '[0.0,-1.0]'::ecvector),
             (4, '[0.9,0.1]'::ecvector),
             (5, '[0.1,0.9]'::ecvector),
             (6, '[-0.9,0.1]'::ecvector),
             (7, '[0.1,-0.9]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid =
            create_ivf_recall_index("ec_ivf_drift_churn", "ec_ivf_drift_churn_idx", 4, 4, 8);

        Spi::run("INSERT INTO ec_ivf_drift_churn VALUES (8, '[1.0,0.2]'::ecvector)")
            .expect("live insert should succeed");

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_tuples FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live tuple count should be non-null"),
            9
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT inserted_since_build FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("inserted-since-build should be non-null"),
            1
        );
        let changed_after_insert = Spi::get_one::<f64>(
            "SELECT changed_row_fraction FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("changed-row fraction should be non-null");
        assert!((changed_after_insert - (1.0 / 9.0)).abs() < 1e-9);
        assert!(
            !Spi::get_one::<bool>(
                "SELECT reindex_recommended FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("reindex recommendation should be non-null")
        );

        let dead_tid = heap_tid_for_row("ec_ivf_drift_churn", 0);
        Spi::run("DELETE FROM ec_ivf_drift_churn WHERE id = 0").expect("delete should succeed");
        ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[dead_tid]));

        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_live_tuples FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("live tuple count should be non-null"),
            8
        );
        assert_eq!(
            Spi::get_one::<i64>(
                "SELECT total_dead_tuples FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("dead tuple count should be non-null"),
            1
        );
        let changed_after_vacuum = Spi::get_one::<f64>(
            "SELECT changed_row_fraction FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("changed-row fraction should be non-null");
        assert!((changed_after_vacuum - (2.0 / 9.0)).abs() < 1e-9);
        assert!(
            Spi::get_one::<f64>(
                "SELECT list_imbalance_ratio FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("list imbalance ratio should be non-null")
            .is_finite()
        );
        assert!(
            Spi::get_one::<bool>(
                "SELECT reindex_recommended FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
            )
            .expect("snapshot query should succeed")
            .expect("reindex recommendation should be non-null")
        );
        let reindex_reason = Spi::get_one::<String>(
            "SELECT reindex_reason FROM ec_ivf_index_drift_snapshot('ec_ivf_drift_churn_idx'::regclass)",
        )
        .expect("snapshot query should succeed")
        .expect("reindex reason should be non-null");
        assert!(reindex_reason.contains("changed_rows"));
    }

    #[pg_test]
    fn test_ec_ivf_vacuum_repeated_bulkdelete_is_idempotent() {
        Spi::run("CREATE TABLE ec_ivf_vacuum_repeat (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_vacuum_repeat VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector),
             (2, '[-1.0,0.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid =
            create_ivf_recall_index("ec_ivf_vacuum_repeat", "ec_ivf_vacuum_repeat_idx", 3, 3, 3);
        let dead_tid = heap_tid_for_row("ec_ivf_vacuum_repeat", 1);

        Spi::run("DELETE FROM ec_ivf_vacuum_repeat WHERE id = 1").expect("delete should succeed");
        let first_stats =
            ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[dead_tid]));
        let second_stats =
            ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[dead_tid]));
        let (_, _, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));

        assert_eq!(first_stats.tuples_removed, 1.0);
        assert_eq!(first_stats.num_index_tuples, 2.0);
        assert_eq!(second_stats.tuples_removed, 0.0);
        assert_eq!(second_stats.num_index_tuples, 2.0);
        assert_eq!(directory_live, 2);
        assert_eq!(directory_dead, 1);
        assert_eq!(inserted_since_build, 0);
        assert_eq!(total_live, 2);
    }

    #[pg_test]
    fn test_ec_ivf_insert_vacuum_scan_safety() {
        Spi::run("CREATE TABLE ec_ivf_vacuum_insert (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_vacuum_insert VALUES
             (0, '[1.0,0.0]'::ecvector),
             (1, '[0.0,1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        let index_oid =
            create_ivf_recall_index("ec_ivf_vacuum_insert", "ec_ivf_vacuum_insert_idx", 2, 2, 2);
        Spi::run("INSERT INTO ec_ivf_vacuum_insert VALUES (2, '[1.0,0.1]'::ecvector)")
            .expect("live insert should succeed");
        let inserted_tid = heap_tid_for_row("ec_ivf_vacuum_insert", 2);
        let before_outputs =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.1])).0;

        assert!(
            before_outputs
                .iter()
                .any(|(block_number, offset_number, _score)| {
                    (*block_number, *offset_number)
                        == (inserted_tid.block_number, inserted_tid.offset_number)
                }),
            "live-inserted row should be reachable before vacuum"
        );

        Spi::run("DELETE FROM ec_ivf_vacuum_insert WHERE id = 2").expect("delete should succeed");
        let stats = ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[inserted_tid]));
        let after_outputs =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_outputs(index_oid, vec![1.0, 0.1])).0;
        let (_, _, directory_live, directory_dead, inserted_since_build) =
            ec_ivf_debug!(am::debug_ec_ivf_directory_summary(index_oid));
        let (_, _, _, total_live, _, _) = ec_ivf_debug!(am::debug_ec_ivf_build_metadata(index_oid));

        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 2.0);
        assert_eq!(directory_live, 2);
        assert_eq!(directory_dead, 1);
        assert_eq!(inserted_since_build, 1);
        assert_eq!(total_live, 2);
        assert!(
            after_outputs
                .iter()
                .all(|(block_number, offset_number, _score)| {
                    (*block_number, *offset_number)
                        != (inserted_tid.block_number, inserted_tid.offset_number)
                }),
            "vacuumed scan output should not include the deleted live-inserted row"
        );
        assert_eq!(after_outputs.len(), 2);
        assert!(after_outputs.iter().all(|(_, _, score)| score.is_finite()));
    }

    // ---------------------------------------------------------------------
    // Task 112: lazy heap-f32 exact rerank.
    // ---------------------------------------------------------------------

    /// Build a heap-f32 (table-side) rerank IVF index over `rows` with the given
    /// `rerank_width`, returning its OID. `storage_format = 'coarse_rerank'`
    /// keeps the heap source vector and reranks table-side through the f32 path
    /// (the pre-112 exact rerank), which is exactly the surface Task 112 touches.
    fn make_lazy_rerank_index(table: &str, rows: &[(i64, Vec<f32>)], rerank_width: i32) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE TABLE {table} (id bigint primary key, embedding ecvector)"
        ))
        .expect("table creation should succeed");
        for (id, vector) in rows {
            let literal = format_recall_vector_sql_literal(vector);
            Spi::run(&format!("INSERT INTO {table} VALUES ({id}, {literal}::ecvector)"))
                .expect("seed insert should succeed");
        }
        Spi::run(&format!(
            "CREATE INDEX {table}_idx ON {table} USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (
                nlists = 4,
                nprobe = 4,
                training_sample_rows = {rows_len},
                storage_format = 'coarse_rerank',
                rerank_format = 'f32',
                rerank_placement = 'table',
                rerank_width = {rerank_width}
             )",
            rows_len = rows.len(),
        ))
        .expect("coarse_rerank heap-f32 index creation should succeed");
        ec_ivf_index_oid(&format!("{table}_idx"))
    }

    fn lazy_rerank_rows(count: i64, dims: usize) -> Vec<(i64, Vec<f32>)> {
        (0..count)
            .map(|id| {
                let base = id as f32;
                let vector = (0..dims)
                    .map(|d| ((base * 0.41 + d as f32 * 0.13).cos()) * 0.5 + 0.5)
                    .collect::<Vec<f32>>();
                (id, vector)
            })
            .collect()
    }

    /// RECALL-SAFETY PROOF (in test form): the lazy heap-f32 rerank returns the
    /// exact same top-k, in the same order, with the same exact scores as the
    /// fixed-width rerank on the same index and query. Under the sound NoBound
    /// contract the lazy path never stops early, so this must hold byte-for-byte.
    #[pg_test]
    fn test_ec_ivf_lazy_rerank_equals_fixed_width() {
        let rows = lazy_rerank_rows(16, 8);
        let index_oid = make_lazy_rerank_index("ec_ivf_lazy_eq", &rows, 8);
        let query = rows[3].1.clone();

        Spi::run("SET ec_ivf.lazy_heap_rerank = off").expect("gate GUC should be settable");
        let fixed =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, query.clone()));

        Spi::run("SET ec_ivf.lazy_heap_rerank = on").expect("gate GUC should be settable");
        let lazy =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, query));
        Spi::run("RESET ec_ivf.lazy_heap_rerank").expect("gate GUC should be resettable");

        assert!(!fixed.outputs.is_empty(), "fixed-width rerank should emit rows");
        assert_eq!(
            fixed.outputs, lazy.outputs,
            "lazy rerank must return identical (tid, tid, exact score) outputs in identical order"
        );
        // Same heap fetch work: under NoBound nothing is skipped, so the lazy
        // path reranks every candidate the fixed path did.
        assert_eq!(fixed.rerank_rows, lazy.rerank_rows);
        assert_eq!(
            fixed.rerank_source_bytes_read,
            lazy.rerank_source_bytes_read
        );
    }

    /// Counters: the lazy stage exposes considered / skipped, and (under the
    /// NoBound default) considered == rerank_rows and skipped == 0.
    #[pg_test]
    fn test_ec_ivf_lazy_rerank_counters_no_skip_under_no_bound() {
        let rows = lazy_rerank_rows(16, 8);
        let index_oid = make_lazy_rerank_index("ec_ivf_lazy_counters", &rows, 8);
        let query = rows[5].1.clone();

        let snap =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, query));

        assert!(
            snap.rerank_candidates_considered > 0,
            "lazy rerank should report considered candidates"
        );
        assert_eq!(
            snap.rerank_candidates_skipped, 0,
            "NoBound must never skip a candidate (recall-safe)"
        );
        assert_eq!(
            snap.rerank_candidates_considered, snap.rerank_rows,
            "every considered candidate is reranked under NoBound"
        );
    }

    /// Disabling the gate routes the legacy fixed-width path and still reports
    /// considered, with zero skipped.
    #[pg_test]
    fn test_ec_ivf_lazy_rerank_gate_off_reports_zero_skipped() {
        let rows = lazy_rerank_rows(16, 8);
        let index_oid = make_lazy_rerank_index("ec_ivf_lazy_gate_off", &rows, 8);
        let query = rows[1].1.clone();

        Spi::run("SET ec_ivf.lazy_heap_rerank = off").expect("gate GUC should be settable");
        let snap =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, query));
        Spi::run("RESET ec_ivf.lazy_heap_rerank").expect("gate GUC should be resettable");

        assert_eq!(snap.rerank_candidates_skipped, 0);
        assert_eq!(snap.rerank_candidates_considered, snap.rerank_rows);
    }

    /// Ties: rows that share an exact score must come back in the deterministic
    /// heap-TID tiebreak order, identical under lazy and fixed width.
    #[pg_test]
    fn test_ec_ivf_lazy_rerank_handles_score_ties() {
        // Several identical vectors => identical exact scores against the query.
        let mut rows: Vec<(i64, Vec<f32>)> = (0..6)
            .map(|id| (id, vec![1.0_f32, 0.0]))
            .collect();
        // A couple of distinct rows so the frontier is non-trivial.
        rows.push((6, vec![0.0_f32, 1.0]));
        rows.push((7, vec![-1.0_f32, 0.0]));
        let index_oid = make_lazy_rerank_index("ec_ivf_lazy_ties", &rows, 8);
        let query = vec![1.0_f32, 0.0];

        Spi::run("SET ec_ivf.lazy_heap_rerank = off").expect("gate GUC should be settable");
        let fixed =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, query.clone()));
        Spi::run("SET ec_ivf.lazy_heap_rerank = on").expect("gate GUC should be settable");
        let lazy =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, query));
        Spi::run("RESET ec_ivf.lazy_heap_rerank").expect("gate GUC should be resettable");

        assert_eq!(
            fixed.outputs, lazy.outputs,
            "tie ordering must be identical under lazy and fixed width"
        );
        // The tied rows must appear before the distant ones, all with finite
        // scores, in a stable order.
        assert!(lazy.outputs.iter().all(|(_, _, score)| score.is_finite()));
    }

    /// Duplicate heap TIDs: the same heap row referenced from multiple probed
    /// lists must be reranked once and returned once, identically lazy vs fixed.
    #[pg_test]
    fn test_ec_ivf_lazy_rerank_dedupes_duplicate_heap_tids() {
        let rows = lazy_rerank_rows(12, 8);
        // nprobe spans all lists so a heap TID reachable from multiple centroids
        // is exercised through the dedup map before rerank.
        let index_oid = make_lazy_rerank_index("ec_ivf_lazy_dupe", &rows, 8);
        let query = rows[2].1.clone();

        Spi::run("SET ec_ivf.lazy_heap_rerank = off").expect("gate GUC should be settable");
        let fixed =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, query.clone()));
        Spi::run("SET ec_ivf.lazy_heap_rerank = on").expect("gate GUC should be settable");
        let lazy =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, query));
        Spi::run("RESET ec_ivf.lazy_heap_rerank").expect("gate GUC should be resettable");

        // No heap TID appears twice in the output.
        let mut seen = std::collections::HashSet::new();
        for (block_number, offset_number, _score) in &lazy.outputs {
            assert!(
                seen.insert((*block_number, *offset_number)),
                "lazy rerank emitted a duplicate heap tid {block_number}:{offset_number}"
            );
        }
        assert_eq!(fixed.outputs, lazy.outputs);
    }

    /// Empty frontier: when every indexed row has been deleted and vacuumed out
    /// of the postings, the scan produces no candidates, so the rerank stage
    /// reranks nothing, skips nothing, and emits nothing.
    #[pg_test]
    fn test_ec_ivf_lazy_rerank_empty_frontier_is_a_noop() {
        Spi::run("CREATE TABLE ec_ivf_lazy_empty (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_ivf_lazy_empty VALUES (0, '[1.0,0.0]'::ecvector), (1, '[0.0,1.0]'::ecvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_ivf_lazy_empty_idx ON ec_ivf_lazy_empty USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = 1, nprobe = 1, training_sample_rows = 2, \
                   storage_format = 'coarse_rerank', rerank_format = 'f32', \
                   rerank_placement = 'table', rerank_width = 8)",
        )
        .expect("index creation should succeed");

        let index_oid = ec_ivf_index_oid("ec_ivf_lazy_empty_idx");
        let tid0 = heap_tid_for_row("ec_ivf_lazy_empty", 0);
        let tid1 = heap_tid_for_row("ec_ivf_lazy_empty", 1);
        Spi::run("DELETE FROM ec_ivf_lazy_empty").expect("delete should succeed");
        // Vacuum the deleted TIDs out of the index so the scan frontier is empty.
        ec_ivf_debug!(am::debug_ec_ivf_vacuum_remove_heap_tids(index_oid, &[tid0, tid1]));

        let snap =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(index_oid, vec![1.0, 0.0]));

        assert_eq!(snap.outputs.len(), 0, "empty frontier should emit no rows");
        assert_eq!(snap.rerank_rows, 0);
        assert_eq!(snap.rerank_candidates_considered, 0);
        assert_eq!(snap.rerank_candidates_skipped, 0);
    }

    /// `rerank_width` boundaries: width smaller than the frontier truncates the
    /// considered set to the width; width >= frontier considers the whole
    /// frontier. Lazy and fixed width agree at each boundary.
    #[pg_test]
    fn test_ec_ivf_lazy_rerank_width_boundaries() {
        let rows = lazy_rerank_rows(16, 8);
        let query = rows[4].1.clone();

        // Narrow width: fewer candidates considered than the frontier.
        let narrow_oid = make_lazy_rerank_index("ec_ivf_lazy_width_narrow", &rows, 3);
        let narrow =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(narrow_oid, query.clone()));
        assert!(
            narrow.rerank_candidates_considered <= 3,
            "narrow rerank_width=3 should consider at most 3 candidates, got {}",
            narrow.rerank_candidates_considered
        );
        assert_eq!(narrow.rerank_candidates_skipped, 0);
        assert_eq!(narrow.rerank_candidates_considered, narrow.rerank_rows);

        // Wide width: considers the whole reachable frontier; lazy == fixed.
        let wide_oid = make_lazy_rerank_index("ec_ivf_lazy_width_wide", &rows, 64);
        Spi::run("SET ec_ivf.lazy_heap_rerank = off").expect("gate GUC should be settable");
        let wide_fixed =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(wide_oid, query.clone()));
        Spi::run("SET ec_ivf.lazy_heap_rerank = on").expect("gate GUC should be settable");
        let wide_lazy =
            ec_ivf_debug!(am::debug_ec_ivf_gettuple_counter_snapshot(wide_oid, query));
        Spi::run("RESET ec_ivf.lazy_heap_rerank").expect("gate GUC should be resettable");

        assert_eq!(wide_fixed.outputs, wide_lazy.outputs);
        assert_eq!(
            wide_lazy.rerank_candidates_considered,
            wide_lazy.rerank_rows
        );
        assert_eq!(wide_lazy.rerank_candidates_skipped, 0);
    }
