    #[pg_test]
    fn test_ec_spire_production_transport_probe_overlaps_ready_remotes() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo.clone(),
                sql: "SELECT pg_sleep(0.30)",
            },
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 3,
                conninfo: loopback_conninfo,
                sql: "SELECT 1",
            },
        ]);
        let slow = rows
            .iter()
            .find(|row| row.node_id == 2)
            .expect("slow row should exist");
        let fast = rows
            .iter()
            .find(|row| row.node_id == 3)
            .expect("fast row should exist");

        assert_eq!(slow.status, "ready");
        assert_eq!(fast.status, "ready");
        assert!(
            fast.completed_after_ms < slow.completed_after_ms,
            "fast remote should complete before slow remote when transport overlaps work: \
             fast={}ms slow={}ms",
            fast.completed_after_ms,
            slow.completed_after_ms
        );
        assert!(
            slow.elapsed_ms >= 250,
            "slow probe should actually exercise delayed remote work: {}ms",
            slow.elapsed_ms
        );
    }

    #[pg_test]
    fn test_ec_spire_production_transport_probe_isolates_node_failure() {
        Spi::run("SET LOCAL ec_spire.remote_search_connect_timeout_ms = 25")
            .expect("connect timeout SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: "host=/tmp/ecaz_missing_pg_socket_30725 dbname=postgres user=postgres"
                    .to_owned(),
                sql: "SELECT 1",
            },
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 3,
                conninfo: loopback_conninfo,
                sql: "SELECT 1",
            },
        ]);
        let failed = rows
            .iter()
            .find(|row| row.node_id == 2)
            .expect("failed row should exist");
        let ready = rows
            .iter()
            .find(|row| row.node_id == 3)
            .expect("ready row should exist");

        assert_eq!(rows.len(), 2);
        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "connect_failed");
        assert_eq!(ready.status, "ready");
        assert_eq!(ready.failure_category, "none");
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_network_partition_matrix_actions() {
        Spi::run("SET LOCAL ec_spire.remote_search_connect_timeout_ms = 25")
            .expect("connect timeout SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let requests = vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: "host=/tmp/ecaz_missing_pg_socket_network_partition \
                           dbname=postgres user=postgres"
                    .to_owned(),
                sql: "SELECT 1",
            },
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 3,
                conninfo: loopback_conninfo,
                sql: "SELECT 1",
            },
        ];

        let strict = am::spire_remote_search_production_transport_probe_summary_for_test(
            requests.clone(),
            "strict",
        );
        assert_eq!(strict.transport_sent_dispatch_count, 2);
        assert_eq!(strict.transport_ready_dispatch_count, 1);
        assert_eq!(strict.transport_failed_dispatch_count, 1);
        assert_eq!(strict.first_transport_failure_category, "connect_failed");
        assert_eq!(strict.degraded_skipped_dispatch_count, 0);
        assert_eq!(
            strict.next_executor_step,
            "production_transport_adapter"
        );
        assert_eq!(strict.status, "remote_transport_failed");

        let degraded = am::spire_remote_search_production_transport_probe_summary_for_test(
            requests,
            "degraded",
        );
        assert_eq!(degraded.transport_sent_dispatch_count, 1);
        assert_eq!(degraded.transport_ready_dispatch_count, 1);
        assert_eq!(degraded.transport_failed_dispatch_count, 0);
        assert_eq!(degraded.first_transport_failure_category, "none");
        assert_eq!(degraded.degraded_skipped_dispatch_count, 1);
        assert_eq!(degraded.first_degraded_skip_category, "connect_failed");
        assert_eq!(degraded.next_executor_step, "remote_heap_resolution");
        assert_eq!(degraded.status, "degraded_ready");
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_remote_stmt_timeout() {
        Spi::run("SET LOCAL ec_spire.remote_search_statement_timeout_ms = 25")
            .expect("statement timeout SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_sleep(0.30)",
            },
        ]);
        let failed = rows.first().expect("timeout row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "remote_statement_timeout");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_remote_oom() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "DO $$ BEGIN RAISE EXCEPTION 'simulated remote out of memory' USING ERRCODE = '53200'; END $$",
            },
        ]);
        let failed = rows.first().expect("remote OOM row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "remote_query_failed");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_remote_oom_matrix_actions() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let requests = vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo.clone(),
                sql: "DO $$ BEGIN RAISE EXCEPTION 'simulated remote out of memory' USING ERRCODE = '53200'; END $$",
            },
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 3,
                conninfo: loopback_conninfo,
                sql: "SELECT 1",
            },
        ];

        let strict = am::spire_remote_search_production_transport_probe_summary_for_test(
            requests.clone(),
            "strict",
        );
        assert_eq!(strict.transport_sent_dispatch_count, 2);
        assert_eq!(strict.transport_ready_dispatch_count, 1);
        assert_eq!(strict.transport_failed_dispatch_count, 1);
        assert_eq!(strict.first_transport_failure_category, "remote_query_failed");
        assert_eq!(strict.degraded_skipped_dispatch_count, 0);
        assert_eq!(strict.first_degraded_skip_category, "none");
        assert_eq!(
            strict.next_executor_step,
            "production_transport_adapter"
        );
        assert_eq!(strict.status, "remote_transport_failed");

        let degraded = am::spire_remote_search_production_transport_probe_summary_for_test(
            requests,
            "degraded",
        );
        assert_eq!(degraded.transport_sent_dispatch_count, 1);
        assert_eq!(degraded.transport_ready_dispatch_count, 1);
        assert_eq!(degraded.transport_failed_dispatch_count, 0);
        assert_eq!(degraded.first_transport_failure_category, "none");
        assert_eq!(degraded.degraded_skipped_dispatch_count, 1);
        assert_eq!(degraded.first_degraded_skip_category, "remote_query_failed");
        assert_eq!(degraded.next_executor_step, "remote_heap_resolution");
        assert_eq!(degraded.status, "degraded_ready");
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_backend_terminated() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_terminate_backend(pg_backend_pid())",
            },
        ]);
        let failed = rows.first().expect("termination row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "remote_backend_terminated");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_remote_query_cancelled() {
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_cancel_backend(pg_backend_pid()), pg_sleep(0.30)",
            },
        ]);
        let failed = rows.first().expect("cancel row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "remote_query_cancelled");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_governance_overload() {
        set_remote_governance_test_namespace(6601);
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

        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: "invalid_conninfo_before_transport_open".to_owned(),
                sql: "SELECT 1",
            },
        ]);
        let failed = rows.first().expect("governance overload row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "remote_executor_overload");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_local_cancel_remote_cancel() {
        set_remote_governance_test_namespace(6602);
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches = 1")
            .expect("global governance cap SET should succeed");
        Spi::run("SET LOCAL ec_spire.remote_search_max_concurrent_dispatches_per_node = 1")
            .expect("per-node governance cap SET should succeed");
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let (global_class_id, global_object_id) =
            am::remote_search_libpq_global_governance_advisory_key_for_test(0);
        let (node_class_id, node_object_id) =
            am::remote_search_libpq_node_governance_advisory_key_for_test(2, 0);
        let rows = am::spire_remote_search_production_transport_probe_with_local_cancel_for_test(
            vec![am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_sleep(0.30)",
            }],
            25,
        );
        let failed = rows.first().expect("local cancel row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "local_query_cancelled");
        assert_eq!(failed.row_count, 0);
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        assert_governance_lock_released(
            &loopback_conninfo,
            global_class_id,
            global_object_id,
            "global transport local-cancel",
        );
        assert_governance_lock_released(
            &loopback_conninfo,
            node_class_id,
            node_object_id,
            "per-node transport local-cancel",
        );
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_pg_interrupt_bridge_cancel() {
        let _interrupt_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let _flags = unsafe { ScopedPgQueryCancelFlags::set_pending() }
            .expect("PostgreSQL query-cancel flags should resolve inside pg_test backend");
        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_sleep(0.30)",
            },
        ]);
        let failed = rows.first().expect("pg interrupt cancel row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "local_query_cancelled");
        assert_eq!(failed.row_count, 0);
    }

    #[pg_test]
    fn test_ec_spire_prod_transport_pg_statement_timeout_bridge_cancel() {
        let _interrupt_lock = env_var_test_lock();
        let loopback_conninfo = current_pg_test_loopback_conninfo();
        let timeout_signal = unsafe { ScopedPgStatementTimeoutSignal::trigger_after_ms(1) }
            .expect("PostgreSQL timeout symbols should resolve inside pg_test backend");
        assert!(timeout_signal.statement_timeout_pending());

        let rows = am::spire_remote_search_production_transport_probe_for_test(vec![
            am::SpireRemoteProductionTransportProbeRequest {
                node_id: 2,
                conninfo: loopback_conninfo,
                sql: "SELECT pg_sleep(0.30)",
            },
        ]);
        let failed = rows
            .first()
            .expect("pg statement-timeout cancel row should exist");

        assert_eq!(failed.status, "remote_transport_failed");
        assert_eq!(failed.failure_category, "local_statement_timeout");
        assert_eq!(failed.row_count, 0);
    }
