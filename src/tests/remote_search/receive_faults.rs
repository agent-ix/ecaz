    #[pg_test]
    fn test_ec_spire_production_candidate_receive_loopback() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_production_candidate_receive_remote_sql; \
                 CREATE TABLE ec_spire_production_candidate_receive_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_production_candidate_receive_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_production_candidate_receive_remote_idx \
                     ON ec_spire_production_candidate_receive_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')",
            )
            .expect("loopback candidate receive fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_production_candidate_receive_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_production_candidate_receive_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let selected_pid = u64::try_from(selected_pid).expect("leaf pid should fit u64");
        let requested_epoch = u64::try_from(active_epoch).expect("active epoch should fit u64");
        let remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_production_candidate_receive_remote_idx",
        );

        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                remote_index_regclass: "ec_spire_production_candidate_receive_remote_idx"
                    .to_owned(),
                remote_index_identity,
                requested_epoch,
                query: vec![1.0, 0.0],
                selected_pids: vec![selected_pid],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let receive = rows.first().expect("receive row should exist");
        let batch = receive
            .batch
            .as_ref()
            .expect("candidate batch should exist");
        let candidate = batch
            .candidates
            .first()
            .expect("candidate row should exist");

        assert_eq!(receive.status, "ready");
        assert_eq!(receive.failure_category, "none");
        assert_eq!(receive.candidate_count, 1);
        assert_eq!(batch.node_id, 2);
        assert_eq!(batch.selected_pids, vec![selected_pid]);
        assert_eq!(candidate.node_id, 2);
        assert_eq!(candidate.served_epoch, requested_epoch);
        assert_eq!(candidate.pid, selected_pid);
        assert!(!candidate.row_locator.is_empty());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_top_k_zero() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_top_k_zero_sql; \
                 CREATE TABLE ec_spire_candidate_receive_top_k_zero_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_top_k_zero_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_top_k_zero_idx \
                     ON ec_spire_candidate_receive_top_k_zero_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')",
            )
            .expect("loopback top-k-zero fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_top_k_zero_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_top_k_zero_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_candidate_receive_top_k_zero_idx",
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_top_k_zero_idx".to_owned(),
                remote_index_identity,
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 0,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let receive = rows.first().expect("top-k-zero row should exist");
        let batch = receive
            .batch
            .as_ref()
            .expect("top-k-zero ready batch should exist");

        assert_eq!(receive.status, "ready");
        assert_eq!(receive.failure_category, "none");
        assert_eq!(receive.candidate_count, 0);
        assert!(batch.candidates.is_empty());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_drop_remote_index_before_dispatch() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_lifecycle_drop_pre_dispatch_sql; \
                 CREATE TABLE ec_spire_lifecycle_drop_pre_dispatch_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_lifecycle_drop_pre_dispatch_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_lifecycle_drop_pre_dispatch_ready_idx \
                     ON ec_spire_lifecycle_drop_pre_dispatch_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE INDEX ec_spire_lifecycle_drop_pre_dispatch_dropped_idx \
                     ON ec_spire_lifecycle_drop_pre_dispatch_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')",
            )
            .expect("loopback drop-before-dispatch fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot(\
                     'ec_spire_lifecycle_drop_pre_dispatch_ready_idx'::regclass)",
                &[],
            )
            .expect("drop-before-dispatch active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("drop-before-dispatch active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot(\
                     'ec_spire_lifecycle_drop_pre_dispatch_ready_idx'::regclass)",
                &[],
            )
            .expect("drop-before-dispatch leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("drop-before-dispatch leaf pid should decode");
        let ready_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_lifecycle_drop_pre_dispatch_ready_idx",
        );
        let dropped_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_lifecycle_drop_pre_dispatch_dropped_idx",
        );
        loopback_client
            .batch_execute("DROP INDEX ec_spire_lifecycle_drop_pre_dispatch_dropped_idx")
            .expect("drop-before-dispatch remote index drop should succeed");

        let request = |node_id: u32,
                       remote_index_regclass: &str,
                       remote_index_identity: Vec<u8>,
                       consistency_mode: &str| {
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id,
                conninfo: loopback_conninfo.clone(),
                remote_index_regclass: remote_index_regclass.to_owned(),
                remote_index_identity,
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: consistency_mode.to_owned(),
            }
        };

        let strict = am::spire_remote_search_production_candidate_receive_summary_for_test(
            vec![
                request(
                    2,
                    "ec_spire_lifecycle_drop_pre_dispatch_dropped_idx",
                    dropped_identity.clone(),
                    "strict",
                ),
                request(
                    3,
                    "ec_spire_lifecycle_drop_pre_dispatch_ready_idx",
                    ready_identity.clone(),
                    "strict",
                ),
            ],
            "strict",
        );
        assert_eq!(strict.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(strict.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(strict.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(strict.endpoint_identity_query_count, 0);
        assert_eq!(
            strict.first_candidate_receive_failure_category,
            "remote_index_unavailable"
        );
        assert_eq!(strict.degraded_skipped_dispatch_count, 0);
        assert_eq!(strict.next_executor_step, "compact_candidate_receive");
        assert_eq!(strict.status, "remote_candidate_receive_failed");

        let degraded = am::spire_remote_search_production_candidate_receive_summary_for_test(
            vec![
                request(
                    2,
                    "ec_spire_lifecycle_drop_pre_dispatch_dropped_idx",
                    dropped_identity,
                    "degraded",
                ),
                request(
                    3,
                    "ec_spire_lifecycle_drop_pre_dispatch_ready_idx",
                    ready_identity,
                    "degraded",
                ),
            ],
            "degraded",
        );
        assert_eq!(degraded.candidate_receive_sent_dispatch_count, 1);
        assert_eq!(degraded.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(degraded.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(degraded.endpoint_identity_query_count, 0);
        assert_eq!(degraded.first_candidate_receive_failure_category, "none");
        assert_eq!(degraded.degraded_skipped_dispatch_count, 1);
        assert_eq!(
            degraded.first_degraded_skip_category,
            "remote_index_unavailable"
        );
        assert_eq!(degraded.next_executor_step, "remote_heap_resolution");
        assert_eq!(degraded.status, "degraded_ready");
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_drop_index_in_flight() {
        let run_mode = |consistency_mode: &str| {
            let loopback_conninfo = current_pg_test_loopback_conninfo();
            let mut loopback_client =
                postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
                    .expect("loopback client connection should succeed");
            let fixture_suffix = format!(
                "ec_spire_lc_drop_flight_{}",
                consistency_mode
            );
            let table_name = format!("{fixture_suffix}_sql");
            let ready_index_name = format!("{fixture_suffix}_ready_idx");
            let dropped_index_name = format!("{fixture_suffix}_drop_idx");

            loopback_client
                .batch_execute(&format!(
                    "DROP TABLE IF EXISTS {table_name}; \
                     CREATE TABLE {table_name} \
                         (id bigint primary key, embedding ecvector); \
                     INSERT INTO {table_name} (id, embedding) VALUES \
                         (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                         (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                     CREATE INDEX {ready_index_name} \
                         ON {table_name} USING ec_spire \
                         (embedding ecvector_spire_ip_ops) \
                         WITH (nlists = 2, storage_format = 'rabitq'); \
                     CREATE INDEX {dropped_index_name} \
                         ON {table_name} USING ec_spire \
                         (embedding ecvector_spire_ip_ops) \
                         WITH (nlists = 2, storage_format = 'rabitq')",
                ))
                .expect("loopback drop-in-flight fixture should be created");
            let active_epoch = loopback_client
                .query_one(
                    &format!(
                        "SELECT active_epoch FROM \
                         ec_spire_index_hierarchy_snapshot('{ready_index_name}'::regclass)"
                    ),
                    &[],
                )
                .expect("drop-in-flight active epoch query should succeed")
                .try_get::<_, i64>(0)
                .expect("drop-in-flight active epoch should decode");
            let selected_pid = loopback_client
                .query_one(
                    &format!(
                        "SELECT min(leaf_pid) FROM \
                         ec_spire_index_leaf_snapshot('{ready_index_name}'::regclass)"
                    ),
                    &[],
                )
                .expect("drop-in-flight leaf pid query should succeed")
                .try_get::<_, i64>(0)
                .expect("drop-in-flight leaf pid should decode");
            let ready_identity =
                loopback_remote_index_identity_bytes(&mut loopback_client, &ready_index_name);
            let dropped_identity =
                loopback_remote_index_identity_bytes(&mut loopback_client, &dropped_index_name);
            let request = |node_id: u32, remote_index_regclass: &str, remote_index_identity: Vec<u8>| {
                am::SpireRemoteProductionCandidateReceiveRequest {
                    node_id,
                    conninfo: loopback_conninfo.clone(),
                    remote_index_regclass: remote_index_regclass.to_owned(),
                    remote_index_identity,
                    requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                    query: vec![1.0, 0.0],
                    selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                    top_k: 1,
                    consistency_mode: consistency_mode.to_owned(),
                }
            };
            let requests = vec![
                request(2, &dropped_index_name, dropped_identity),
                request(3, &ready_index_name, ready_identity),
            ];

            loopback_client
                .batch_execute(&format!("DROP INDEX {dropped_index_name}"))
                .expect("drop-in-flight remote index drop should succeed");
            let dropped_is_absent = loopback_client
                .query_one(
                    &format!("SELECT to_regclass('{dropped_index_name}') IS NULL"),
                    &[],
                )
                .expect("drop-in-flight dropped index check should succeed")
                .try_get::<_, bool>(0)
                .expect("drop-in-flight dropped index check should decode");
            assert!(dropped_is_absent);

            am::spire_remote_search_production_candidate_receive_summary_for_test(
                requests,
                consistency_mode,
            )
        };

        let strict = run_mode("strict");
        assert_eq!(strict.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(strict.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(strict.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(
            strict.first_candidate_receive_failure_category,
            "remote_index_unavailable"
        );
        assert_eq!(strict.degraded_skipped_dispatch_count, 0);
        assert_eq!(strict.next_executor_step, "compact_candidate_receive");
        assert_eq!(strict.status, "remote_candidate_receive_failed");

        let degraded = run_mode("degraded");
        assert_eq!(degraded.candidate_receive_sent_dispatch_count, 1);
        assert_eq!(degraded.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(degraded.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(degraded.first_candidate_receive_failure_category, "none");
        assert_eq!(degraded.degraded_skipped_dispatch_count, 1);
        assert_eq!(
            degraded.first_degraded_skip_category,
            "remote_index_unavailable"
        );
        assert_eq!(degraded.next_executor_step, "remote_heap_resolution");
        assert_eq!(degraded.status, "degraded_ready");
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_reindex_before_dispatch() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_lifecycle_reindex_pre_dispatch_sql; \
                 CREATE TABLE ec_spire_lifecycle_reindex_pre_dispatch_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_lifecycle_reindex_pre_dispatch_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_lifecycle_reindex_pre_dispatch_ready_idx \
                     ON ec_spire_lifecycle_reindex_pre_dispatch_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE INDEX ec_spire_lifecycle_reindex_pre_dispatch_reindexed_idx \
                     ON ec_spire_lifecycle_reindex_pre_dispatch_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')",
            )
            .expect("loopback reindex-before-dispatch fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot(\
                     'ec_spire_lifecycle_reindex_pre_dispatch_ready_idx'::regclass)",
                &[],
            )
            .expect("reindex-before-dispatch active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("reindex-before-dispatch active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot(\
                     'ec_spire_lifecycle_reindex_pre_dispatch_ready_idx'::regclass)",
                &[],
            )
            .expect("reindex-before-dispatch leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("reindex-before-dispatch leaf pid should decode");
        let ready_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_lifecycle_reindex_pre_dispatch_ready_idx",
        );
        let stale_reindexed_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_lifecycle_reindex_pre_dispatch_reindexed_idx",
        );
        loopback_client
            .batch_execute("REINDEX INDEX ec_spire_lifecycle_reindex_pre_dispatch_reindexed_idx")
            .expect("reindex-before-dispatch remote index reindex should succeed");
        let current_reindexed_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_lifecycle_reindex_pre_dispatch_reindexed_idx",
        );
        assert_ne!(stale_reindexed_identity, current_reindexed_identity);

        let request = |node_id: u32,
                       remote_index_regclass: &str,
                       remote_index_identity: Vec<u8>,
                       consistency_mode: &str| {
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id,
                conninfo: loopback_conninfo.clone(),
                remote_index_regclass: remote_index_regclass.to_owned(),
                remote_index_identity,
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: consistency_mode.to_owned(),
            }
        };

        let strict = am::spire_remote_search_production_candidate_receive_summary_for_test(
            vec![
                request(
                    2,
                    "ec_spire_lifecycle_reindex_pre_dispatch_reindexed_idx",
                    stale_reindexed_identity.clone(),
                    "strict",
                ),
                request(
                    3,
                    "ec_spire_lifecycle_reindex_pre_dispatch_ready_idx",
                    ready_identity.clone(),
                    "strict",
                ),
            ],
            "strict",
        );
        assert_eq!(strict.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(strict.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(strict.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(
            strict.first_candidate_receive_failure_category,
            "endpoint_identity_mismatch"
        );
        assert_eq!(strict.degraded_skipped_dispatch_count, 0);
        assert_eq!(strict.next_executor_step, "compact_candidate_receive");
        assert_eq!(strict.status, "remote_candidate_receive_failed");

        let degraded = am::spire_remote_search_production_candidate_receive_summary_for_test(
            vec![
                request(
                    2,
                    "ec_spire_lifecycle_reindex_pre_dispatch_reindexed_idx",
                    stale_reindexed_identity,
                    "degraded",
                ),
                request(
                    3,
                    "ec_spire_lifecycle_reindex_pre_dispatch_ready_idx",
                    ready_identity,
                    "degraded",
                ),
            ],
            "degraded",
        );
        assert_eq!(degraded.candidate_receive_sent_dispatch_count, 1);
        assert_eq!(degraded.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(degraded.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(degraded.first_candidate_receive_failure_category, "none");
        assert_eq!(degraded.degraded_skipped_dispatch_count, 1);
        assert_eq!(
            degraded.first_degraded_skip_category,
            "endpoint_identity_mismatch"
        );
        assert_eq!(degraded.next_executor_step, "remote_heap_resolution");
        assert_eq!(degraded.status, "degraded_ready");
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_reindex_in_flight() {
        let run_mode = |consistency_mode: &str| {
            let loopback_conninfo = current_pg_test_loopback_conninfo();
            let mut loopback_client =
                postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
                    .expect("loopback client connection should succeed");
            let fixture_suffix = format!(
                "ec_spire_lc_reindex_flight_{}",
                consistency_mode
            );
            let table_name = format!("{fixture_suffix}_sql");
            let ready_index_name = format!("{fixture_suffix}_ready_idx");
            let reindexed_index_name = format!("{fixture_suffix}_re_idx");

            loopback_client
                .batch_execute(&format!(
                    "DROP TABLE IF EXISTS {table_name}; \
                     CREATE TABLE {table_name} \
                         (id bigint primary key, embedding ecvector); \
                     INSERT INTO {table_name} (id, embedding) VALUES \
                         (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                         (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                     CREATE INDEX {ready_index_name} \
                         ON {table_name} USING ec_spire \
                         (embedding ecvector_spire_ip_ops) \
                         WITH (nlists = 2, storage_format = 'rabitq'); \
                     CREATE INDEX {reindexed_index_name} \
                         ON {table_name} USING ec_spire \
                         (embedding ecvector_spire_ip_ops) \
                         WITH (nlists = 2, storage_format = 'rabitq')",
                ))
                .expect("loopback reindex-in-flight fixture should be created");
            let active_epoch = loopback_client
                .query_one(
                    &format!(
                        "SELECT active_epoch FROM \
                         ec_spire_index_hierarchy_snapshot('{ready_index_name}'::regclass)"
                    ),
                    &[],
                )
                .expect("reindex-in-flight active epoch query should succeed")
                .try_get::<_, i64>(0)
                .expect("reindex-in-flight active epoch should decode");
            let selected_pid = loopback_client
                .query_one(
                    &format!(
                        "SELECT min(leaf_pid) FROM \
                         ec_spire_index_leaf_snapshot('{ready_index_name}'::regclass)"
                    ),
                    &[],
                )
                .expect("reindex-in-flight leaf pid query should succeed")
                .try_get::<_, i64>(0)
                .expect("reindex-in-flight leaf pid should decode");
            let ready_identity =
                loopback_remote_index_identity_bytes(&mut loopback_client, &ready_index_name);
            let stale_reindexed_identity =
                loopback_remote_index_identity_bytes(&mut loopback_client, &reindexed_index_name);
            let request = |node_id: u32, remote_index_regclass: &str, remote_index_identity: Vec<u8>| {
                am::SpireRemoteProductionCandidateReceiveRequest {
                    node_id,
                    conninfo: loopback_conninfo.clone(),
                    remote_index_regclass: remote_index_regclass.to_owned(),
                    remote_index_identity,
                    requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                    query: vec![1.0, 0.0],
                    selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                    top_k: 1,
                    consistency_mode: consistency_mode.to_owned(),
                }
            };
            let requests = vec![
                request(2, &reindexed_index_name, stale_reindexed_identity.clone()),
                request(3, &ready_index_name, ready_identity),
            ];

            loopback_client
                .batch_execute(&format!("REINDEX INDEX {reindexed_index_name}"))
                .expect("reindex-in-flight remote index reindex should succeed");
            let current_reindexed_identity = loopback_remote_index_identity_bytes(
                &mut loopback_client,
                &reindexed_index_name,
            );
            assert_ne!(stale_reindexed_identity, current_reindexed_identity);

            am::spire_remote_search_production_candidate_receive_summary_for_test(
                requests,
                consistency_mode,
            )
        };

        let strict = run_mode("strict");
        assert_eq!(strict.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(strict.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(strict.candidate_receive_failed_dispatch_count, 1);
        assert_eq!(
            strict.first_candidate_receive_failure_category,
            "endpoint_identity_mismatch"
        );
        assert_eq!(strict.degraded_skipped_dispatch_count, 0);
        assert_eq!(strict.next_executor_step, "compact_candidate_receive");
        assert_eq!(strict.status, "remote_candidate_receive_failed");

        let degraded = run_mode("degraded");
        assert_eq!(degraded.candidate_receive_sent_dispatch_count, 1);
        assert_eq!(degraded.candidate_receive_ready_dispatch_count, 1);
        assert_eq!(degraded.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(degraded.first_candidate_receive_failure_category, "none");
        assert_eq!(degraded.degraded_skipped_dispatch_count, 1);
        assert_eq!(
            degraded.first_degraded_skip_category,
            "endpoint_identity_mismatch"
        );
        assert_eq!(degraded.next_executor_step, "remote_heap_resolution");
        assert_eq!(degraded.status, "degraded_ready");
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_cic_new_descriptor_deferred() {
        let run_mode = |consistency_mode: &str, descriptor_generation: i64| {
            let loopback_conninfo = current_pg_test_loopback_conninfo();
            let mut loopback_client =
                postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
                    .expect("loopback client connection should succeed");
            let fixture_suffix = format!("ec_spire_lc_cic_new_{consistency_mode}");
            let table_name = format!("{fixture_suffix}_sql");
            let old_index_name = format!("{fixture_suffix}_old_idx");
            let ready_index_name = format!("{fixture_suffix}_ready_idx");
            let new_index_name = format!("{fixture_suffix}_new_idx");

            loopback_client
                .batch_execute(&format!(
                    "DROP TABLE IF EXISTS {table_name}; \
                     CREATE TABLE {table_name} \
                         (id bigint primary key, embedding ecvector); \
                     INSERT INTO {table_name} (id, embedding) VALUES \
                         (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                         (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                     CREATE INDEX {old_index_name} \
                         ON {table_name} USING ec_spire \
                         (embedding ecvector_spire_ip_ops) \
                         WITH (nlists = 2, storage_format = 'rabitq'); \
                     CREATE INDEX {ready_index_name} \
                         ON {table_name} USING ec_spire \
                         (embedding ecvector_spire_ip_ops) \
                         WITH (nlists = 2, storage_format = 'rabitq')",
                ))
                .expect("loopback CIC new-descriptor fixture should be created");
            let active_epoch = loopback_client
                .query_one(
                    &format!(
                        "SELECT active_epoch FROM \
                         ec_spire_index_hierarchy_snapshot('{ready_index_name}'::regclass)"
                    ),
                    &[],
                )
                .expect("CIC new-descriptor active epoch query should succeed")
                .try_get::<_, i64>(0)
                .expect("CIC new-descriptor active epoch should decode");
            let selected_pids = loopback_client
                .query(
                    &format!(
                        "SELECT DISTINCT leaf_pid FROM (\
                             SELECT leaf_pid FROM ec_spire_index_leaf_snapshot('{old_index_name}'::regclass) \
                             UNION ALL \
                             SELECT leaf_pid FROM ec_spire_index_leaf_snapshot('{ready_index_name}'::regclass)\
                         ) p ORDER BY leaf_pid"
                    ),
                    &[],
                )
                .expect("CIC new-descriptor selected PID query should succeed")
                .into_iter()
                .map(|row| {
                    u64::try_from(
                        row.try_get::<_, i64>(0)
                            .expect("CIC new-descriptor PID should decode"),
                    )
                    .expect("CIC new-descriptor PID should fit u64")
                })
                .collect::<Vec<_>>();
            assert!(!selected_pids.is_empty());
            let ready_identity =
                loopback_remote_index_identity_bytes(&mut loopback_client, &ready_index_name);
            let old_identity =
                loopback_remote_index_identity_bytes(&mut loopback_client, &old_index_name);
            let old_identity_hex = hex::encode(&old_identity);
            let coordinator_index_oid = Spi::get_one::<pg_sys::Oid>(&format!(
                "SELECT '{ready_index_name}'::regclass::oid"
            ))
            .expect("CIC new-descriptor coordinator index OID query should succeed")
            .expect("CIC new-descriptor coordinator index OID should exist");
            let old_register = Spi::get_one::<bool>(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, {}, 'spire/remote/lifecycle/cic-new', \
                     decode('{}', 'hex'), '{}', 'active', {}, {}, '{}', 'none')",
                u32::from(coordinator_index_oid),
                descriptor_generation - 1,
                old_identity_hex,
                old_index_name,
                active_epoch,
                active_epoch,
                env!("CARGO_PKG_VERSION")
            ))
            .expect("CIC old descriptor registration should succeed")
            .expect("CIC old descriptor registration result should exist");
            assert!(old_register);

            let request = |node_id: u32, remote_index_regclass: &str, remote_index_identity: Vec<u8>| {
                am::SpireRemoteProductionCandidateReceiveRequest {
                    node_id,
                    conninfo: loopback_conninfo.clone(),
                    remote_index_regclass: remote_index_regclass.to_owned(),
                    remote_index_identity,
                    requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                    query: vec![1.0, 0.0],
                    selected_pids: selected_pids.clone(),
                    top_k: 1,
                    consistency_mode: consistency_mode.to_owned(),
                }
            };
            let requests = vec![
                request(2, &old_index_name, old_identity.clone()),
                request(3, &ready_index_name, ready_identity),
            ];

            loopback_client
                .batch_execute(&format!(
                    "CREATE INDEX CONCURRENTLY {new_index_name} \
                     ON {table_name} USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq')"
                ))
                .expect("CIC new remote index creation should succeed");
            let new_identity =
                loopback_remote_index_identity_bytes(&mut loopback_client, &new_index_name);
            assert_ne!(old_identity, new_identity);
            let new_identity_hex = hex::encode(&new_identity);
            let new_register = Spi::get_one::<bool>(&format!(
                "SELECT ec_spire_register_remote_node_descriptor(\
                     '{}'::oid, 2, {}, 'spire/remote/lifecycle/cic-new', \
                     decode('{}', 'hex'), '{}', 'active', {}, {}, '{}', 'none')",
                u32::from(coordinator_index_oid),
                descriptor_generation,
                new_identity_hex,
                new_index_name,
                active_epoch,
                active_epoch,
                env!("CARGO_PKG_VERSION")
            ))
            .expect("CIC new descriptor registration should succeed")
            .expect("CIC new descriptor registration result should exist");
            assert!(new_register);
            let descriptor_row = Spi::get_one::<String>(&format!(
                "SELECT descriptor_generation::text || ':' || remote_index_regclass || ':' || \
                        encode(remote_index_identity, 'hex') \
                 FROM ec_spire_remote_node_descriptor \
                 WHERE coordinator_index_oid = '{}'::oid AND node_id = 2",
                u32::from(coordinator_index_oid)
            ))
            .expect("CIC descriptor row query should succeed")
            .expect("CIC descriptor row should exist");
            assert_eq!(
                descriptor_row,
                format!("{descriptor_generation}:{new_index_name}:{new_identity_hex}")
            );

            am::spire_remote_search_production_candidate_receive_summary_for_test(
                requests,
                consistency_mode,
            )
        };

        let strict = run_mode("strict", 31);
        assert_eq!(strict.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(strict.candidate_receive_ready_dispatch_count, 2);
        assert_eq!(strict.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(strict.first_candidate_receive_failure_category, "none");
        assert_eq!(strict.degraded_skipped_dispatch_count, 0);
        assert_eq!(strict.first_degraded_skip_category, "none");
        assert_eq!(strict.next_executor_step, "remote_heap_resolution");
        assert_eq!(strict.status, "requires_remote_heap_resolution");

        let degraded = run_mode("degraded", 41);
        assert_eq!(degraded.candidate_receive_sent_dispatch_count, 2);
        assert_eq!(degraded.candidate_receive_ready_dispatch_count, 2);
        assert_eq!(degraded.candidate_receive_failed_dispatch_count, 0);
        assert_eq!(degraded.first_candidate_receive_failure_category, "none");
        assert_eq!(degraded.degraded_skipped_dispatch_count, 0);
        assert_eq!(degraded.first_degraded_skip_category, "none");
        assert_eq!(degraded.next_executor_step, "remote_heap_resolution");
        assert_eq!(degraded.status, "requires_remote_heap_resolution");
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_remote_stmt_timeout() {
        Spi::run("SET LOCAL ec_spire.remote_search_statement_timeout_ms = 25")
            .expect("statement timeout SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_timeout_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_timeout CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_timeout_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_timeout_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_timeout_remote_idx \
                     ON ec_spire_candidate_receive_timeout_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_timeout; \
                 CREATE FUNCTION ec_spire_candidate_receive_timeout.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                     FROM pg_sleep(0.30) \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback timeout fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_timeout_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_timeout_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_candidate_receive_timeout_remote_idx",
        );
        let timeout_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_timeout,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: timeout_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_timeout_remote_idx".to_owned(),
                remote_index_identity,
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("timeout row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "remote_statement_timeout");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_remote_query_cancelled() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_cancel_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_cancel CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_cancel_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_cancel_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_cancel_remote_idx \
                     ON ec_spire_candidate_receive_cancel_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_cancel; \
                 CREATE FUNCTION ec_spire_candidate_receive_cancel.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                     FROM pg_cancel_backend(pg_backend_pid()), pg_sleep(0.30) \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback cancel fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_cancel_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_cancel_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_candidate_receive_cancel_remote_idx",
        );
        let cancel_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_cancel,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: cancel_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_cancel_remote_idx".to_owned(),
                remote_index_identity,
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("cancel row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "remote_query_cancelled");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_local_cancel_remote_cancel() {
        set_remote_governance_test_namespace(6603);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 1")
            .expect("global governance cap SET should succeed");
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches_per_node = 1")
            .expect("per-node governance cap SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let (global_class_id, global_object_id) =
            am::remote_search_libpq_global_governance_advisory_key_for_test(0);
        let (node_class_id, node_object_id) =
            am::remote_search_libpq_node_governance_advisory_key_for_test(2, 0);
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_local_cancel_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_local_cancel CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_local_cancel_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_local_cancel_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_local_cancel_remote_idx \
                     ON ec_spire_candidate_receive_local_cancel_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_local_cancel; \
                 CREATE FUNCTION ec_spire_candidate_receive_local_cancel.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                     FROM pg_sleep(0.30) \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback local cancel fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_local_cancel_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_local_cancel_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_candidate_receive_local_cancel_remote_idx",
        );
        let local_cancel_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_local_cancel,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_with_local_cancel_for_test(
            vec![am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: local_cancel_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_local_cancel_remote_idx"
                    .to_owned(),
                remote_index_identity,
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            }],
            25,
        );
        let failed = rows.first().expect("local cancel row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "local_query_cancelled");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
        assert_governance_lock_released(
            &loopback_conninfo,
            global_class_id,
            global_object_id,
            "global receive local-cancel",
        );
        assert_governance_lock_released(
            &loopback_conninfo,
            node_class_id,
            node_object_id,
            "per-node receive local-cancel",
        );
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_governance_overload() {
        set_remote_governance_test_namespace(6604);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 1")
            .expect("global governance cap SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let (class_id, object_id) =
            am::remote_search_libpq_global_governance_advisory_key_for_test(0);
        let mut lock_holder = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback lock-holder connection should succeed");
        lock_holder
            .batch_execute(&format!("SELECT pg_advisory_lock({class_id}, {object_id})"))
            .expect("global governance advisory lock should be held by separate backend");

        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: "invalid_conninfo_before_candidate_receive_open".to_owned(),
                remote_index_regclass: "ec_spire_missing_remote_idx".to_owned(),
                remote_index_identity: vec![0xaa],
                requested_epoch: 1,
                query: vec![1.0, 0.0],
                selected_pids: vec![1],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("governance overload row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "remote_executor_overload");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_identity_mismatch() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_identity_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_identity CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_identity_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_identity_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_identity_remote_idx \
                     ON ec_spire_candidate_receive_identity_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_identity; \
                 CREATE FUNCTION ec_spire_candidate_receive_identity.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'bb', 'ready' \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback identity mismatch fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_identity_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_identity_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let identity_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_identity,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: identity_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_identity_remote_idx".to_owned(),
                remote_index_identity: vec![0xaa],
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("identity mismatch row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "endpoint_identity_mismatch");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_stale_epoch() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_stale_epoch_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_stale_epoch CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_stale_epoch_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_stale_epoch_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_stale_epoch_idx \
                     ON ec_spire_candidate_receive_stale_epoch_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_stale_epoch; \
                 CREATE FUNCTION ec_spire_candidate_receive_stale_epoch.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2 - 1, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
	                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
	                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
	                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
	                            'inner_product_score_v1', \
	                            (SELECT profile_fingerprint \
	                               FROM ec_spire_remote_search_endpoint_identity(\
	                                   'ec_spire_candidate_receive_stale_epoch_idx'::regclass)), \
	                            'ready' \
	                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback stale epoch fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_stale_epoch_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_stale_epoch_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_candidate_receive_stale_epoch_idx",
        );
        let stale_epoch_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_stale_epoch,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: stale_epoch_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_stale_epoch_idx".to_owned(),
                remote_index_identity,
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("stale epoch row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "served_epoch_mismatch");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_backend_terminated() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_candidate_receive_terminate_remote_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_terminate CASCADE; \
                 CREATE TABLE ec_spire_candidate_receive_terminate_remote_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_candidate_receive_terminate_remote_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_candidate_receive_terminate_remote_idx \
                     ON ec_spire_candidate_receive_terminate_remote_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_terminate; \
                 CREATE FUNCTION ec_spire_candidate_receive_terminate.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 2::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
                            'inner_product_score_v1', 'aa', 'ready' \
                     FROM pg_terminate_backend(pg_backend_pid()) \
                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback terminate fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_candidate_receive_terminate_remote_idx'::regclass)",
                &[],
            )
            .expect("active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_candidate_receive_terminate_remote_idx'::regclass)",
                &[],
            )
            .expect("leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("leaf pid should decode");
        let remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_candidate_receive_terminate_remote_idx",
        );
        let terminate_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_terminate,public'"
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id: 2,
                conninfo: terminate_conninfo,
                remote_index_regclass: "ec_spire_candidate_receive_terminate_remote_idx".to_owned(),
                remote_index_identity,
                requested_epoch: u64::try_from(active_epoch).expect("epoch should fit u64"),
                query: vec![1.0, 0.0],
                selected_pids: vec![u64::try_from(selected_pid).expect("pid should fit u64")],
                top_k: 1,
                consistency_mode: "strict".to_owned(),
            },
        ]);
        let failed = rows.first().expect("termination row should exist");

        assert_eq!(failed.status, "remote_candidate_receive_failed");
        assert_eq!(failed.failure_category, "remote_backend_terminated");
        assert_eq!(failed.candidate_count, 0);
        assert!(failed.batch.is_none());
    }

    #[pg_test]
    fn test_ec_spire_prod_receive_isolates_node_failures() {
        Spi::run("SET LOCAL ec_spire.remote_search_connect_timeout_ms = 25")
            .expect("connect timeout SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
            .expect("loopback client connection should succeed");
        loopback_client
            .batch_execute(&format!(
                "DROP TABLE IF EXISTS ec_spire_production_candidate_receive_ready_sql; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_decode_fail CASCADE; \
                 DROP SCHEMA IF EXISTS ec_spire_candidate_receive_validation_fail CASCADE; \
                 CREATE TABLE ec_spire_production_candidate_receive_ready_sql \
                     (id bigint primary key, embedding ecvector); \
                 INSERT INTO ec_spire_production_candidate_receive_ready_sql (id, embedding) VALUES \
                     (10, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)), \
                     (20, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_production_candidate_receive_ready_idx \
                     ON ec_spire_production_candidate_receive_ready_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) \
                     WITH (nlists = 2, storage_format = 'rabitq'); \
                 CREATE SCHEMA ec_spire_candidate_receive_decode_fail; \
                 CREATE FUNCTION ec_spire_candidate_receive_decode_fail.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score text, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 1::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
	                            decode('01', 'hex'), decode('02', 'hex'), 'not-a-real'::text, \
	                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
	                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
	                            'inner_product_score_v1', \
	                            (SELECT profile_fingerprint \
	                               FROM ec_spire_remote_search_endpoint_identity(\
	                                   'ec_spire_production_candidate_receive_ready_idx'::regclass)), \
	                            'ready' \
	                 $function$; \
                 CREATE SCHEMA ec_spire_candidate_receive_validation_fail; \
                 CREATE FUNCTION ec_spire_candidate_receive_validation_fail.ec_spire_remote_search(\
                     oid, bigint, real[], bigint[], integer, text) \
                 RETURNS TABLE (\
                     served_epoch bigint, node_id bigint, pid bigint, \
                     object_version bigint, row_index bigint, assignment_flags smallint, \
                     vec_id bytea, row_locator bytea, score real, \
                     protocol_version text, extension_version text, opclass_identity text, \
                     storage_format text, assignment_payload_format text, \
                     quantizer_profile text, scoring_profile text, \
                     profile_fingerprint text, endpoint_status text) \
                 LANGUAGE sql AS $function$ \
                     SELECT $2, 999::bigint, $4[1], \
                            1::bigint, 0::bigint, 1::smallint, \
	                            decode('01', 'hex'), decode('02', 'hex'), 1.0::real, \
	                            'ec_spire_remote_search_v1', '{extension_version}', 'test-opclass', \
	                            'rabitq', 'rabitq_v1', 'rabitq_quantizer_v1', \
	                            'inner_product_score_v1', \
	                            (SELECT profile_fingerprint \
	                               FROM ec_spire_remote_search_endpoint_identity(\
	                                   'ec_spire_production_candidate_receive_ready_idx'::regclass)), \
	                            'ready' \
	                 $function$",
                extension_version = env!("CARGO_PKG_VERSION")
            ))
            .expect("loopback multi-node candidate receive fixture should be created");
        let active_epoch = loopback_client
            .query_one(
                "SELECT active_epoch FROM \
                 ec_spire_index_hierarchy_snapshot('ec_spire_production_candidate_receive_ready_idx'::regclass)",
                &[],
            )
            .expect("ready active epoch query should succeed")
            .try_get::<_, i64>(0)
            .expect("ready active epoch should decode");
        let selected_pid = loopback_client
            .query_one(
                "SELECT min(leaf_pid) FROM \
                 ec_spire_index_leaf_snapshot('ec_spire_production_candidate_receive_ready_idx'::regclass)",
                &[],
            )
            .expect("ready leaf pid query should succeed")
            .try_get::<_, i64>(0)
            .expect("ready leaf pid should decode");
        let selected_pid = u64::try_from(selected_pid).expect("ready leaf pid should fit u64");
        let requested_epoch = u64::try_from(active_epoch).expect("ready epoch should fit u64");
        let ready_remote_index_identity = loopback_remote_index_identity_bytes(
            &mut loopback_client,
            "ec_spire_production_candidate_receive_ready_idx",
        );
        let decode_fail_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_decode_fail,public'"
        );
        let validation_fail_conninfo = format!(
            "{loopback_conninfo} options='-c search_path=ec_spire_candidate_receive_validation_fail,public'"
        );

        let request = |node_id: u32,
                       conninfo: String,
                       remote_index_regclass: &str,
                       remote_index_identity: Vec<u8>,
                       requested_epoch: u64,
                       query: Vec<f32>,
                       selected_pids: Vec<u64>,
                       consistency_mode: &str| {
            am::SpireRemoteProductionCandidateReceiveRequest {
                node_id,
                conninfo,
                remote_index_regclass: remote_index_regclass.to_owned(),
                remote_index_identity,
                requested_epoch,
                query,
                selected_pids,
                top_k: 1,
                consistency_mode: consistency_mode.to_owned(),
            }
        };
        let rows = am::spire_remote_search_production_candidate_receive_for_test(vec![
            request(
                2,
                loopback_conninfo.clone(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                3,
                loopback_conninfo.clone(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![u64::MAX],
                "strict",
            ),
            request(
                4,
                "port=not-a-number dbname=postgres".to_owned(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                5,
                "host=/tmp/ecaz_missing_pg_socket_30729 port=6543 dbname=postgres user=postgres connect_timeout=1"
                    .to_owned(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                6,
                loopback_conninfo.clone(),
                "ec_spire_missing_candidate_receive_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                7,
                loopback_conninfo.clone(),
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                8,
                decode_fail_conninfo,
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity.clone(),
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
            request(
                9,
                validation_fail_conninfo,
                "ec_spire_production_candidate_receive_ready_idx",
                ready_remote_index_identity,
                requested_epoch,
                vec![1.0, 0.0],
                vec![selected_pid],
                "strict",
            ),
        ]);
        let ready = rows
            .iter()
            .find(|row| row.node_id == 2)
            .expect("ready row should exist");
        let ready_batch = ready.batch.as_ref().expect("ready batch should exist");
        let expected_failures = [
            (3, "candidate_invalid_parameters"),
            (4, "conninfo_parse_failed"),
            (5, "connect_failed"),
            (6, "remote_index_unavailable"),
            (7, "remote_query_failed"),
            (8, "candidate_decode_failed"),
            (9, "candidate_batch_validation_failed"),
        ];

        assert_eq!(rows.len(), 8);
        assert_eq!(ready.status, "ready");
        assert_eq!(ready.failure_category, "none");
        assert_eq!(ready.candidate_count, 1);
        assert_eq!(ready_batch.node_id, 2);
        assert_eq!(ready_batch.selected_pids, vec![selected_pid]);
        assert_eq!(ready_batch.candidates.len(), 1);
        assert!(ready_batch
            .candidates
            .iter()
            .all(|candidate| candidate.node_id == 2));
        for (node_id, failure_category) in expected_failures {
            let failed = rows
                .iter()
                .find(|row| row.node_id == node_id)
                .expect("failed row should exist");
            assert_eq!(failed.status, "remote_candidate_receive_failed");
            assert_eq!(failed.failure_category, failure_category);
            assert_eq!(failed.candidate_count, 0);
            assert!(
                failed.batch.is_none(),
                "failed node {node_id} should not return a candidate batch"
            );
        }
    }
