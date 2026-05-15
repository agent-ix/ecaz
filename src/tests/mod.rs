    use super::*;
    use std::sync::{Mutex, OnceLock};

    struct ScopedEnvVar {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl ScopedEnvVar {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for ScopedEnvVar {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_ref() {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    struct ScopedPgQueryCancelFlags {
        interrupt_pending_ptr: *mut std::ffi::c_int,
        query_cancel_pending_ptr: *mut std::ffi::c_int,
        previous_interrupt_pending: std::ffi::c_int,
        previous_query_cancel_pending: std::ffi::c_int,
    }

    impl ScopedPgQueryCancelFlags {
        unsafe fn set_pending() -> Option<Self> {
            unsafe extern "C" {
                fn dlsym(
                    handle: *mut std::ffi::c_void,
                    symbol: *const std::ffi::c_char,
                ) -> *mut std::ffi::c_void;
            }

            let interrupt_pending_ptr =
                unsafe { dlsym(std::ptr::null_mut(), b"InterruptPending\0".as_ptr().cast()) }
                    .cast::<std::ffi::c_int>();
            let query_cancel_pending_ptr = unsafe {
                dlsym(
                    std::ptr::null_mut(),
                    b"QueryCancelPending\0".as_ptr().cast(),
                )
            }
            .cast::<std::ffi::c_int>();
            if interrupt_pending_ptr.is_null() || query_cancel_pending_ptr.is_null() {
                return None;
            }
            let previous_interrupt_pending = unsafe { *interrupt_pending_ptr };
            let previous_query_cancel_pending = unsafe { *query_cancel_pending_ptr };
            unsafe {
                *interrupt_pending_ptr = 1;
                *query_cancel_pending_ptr = 1;
            }
            Some(Self {
                interrupt_pending_ptr,
                query_cancel_pending_ptr,
                previous_interrupt_pending,
                previous_query_cancel_pending,
            })
        }

        unsafe fn clear_pending_for_test() {
            unsafe extern "C" {
                fn dlsym(
                    handle: *mut std::ffi::c_void,
                    symbol: *const std::ffi::c_char,
                ) -> *mut std::ffi::c_void;
            }

            let interrupt_pending_ptr =
                unsafe { dlsym(std::ptr::null_mut(), b"InterruptPending\0".as_ptr().cast()) }
                    .cast::<std::ffi::c_int>();
            let query_cancel_pending_ptr = unsafe {
                dlsym(
                    std::ptr::null_mut(),
                    b"QueryCancelPending\0".as_ptr().cast(),
                )
            }
            .cast::<std::ffi::c_int>();
            if !interrupt_pending_ptr.is_null() {
                unsafe {
                    *interrupt_pending_ptr = 0;
                }
            }
            if !query_cancel_pending_ptr.is_null() {
                unsafe {
                    *query_cancel_pending_ptr = 0;
                }
            }
        }
    }

    impl Drop for ScopedPgQueryCancelFlags {
        fn drop(&mut self) {
            unsafe {
                *self.interrupt_pending_ptr = self.previous_interrupt_pending;
                *self.query_cancel_pending_ptr = self.previous_query_cancel_pending;
            }
        }
    }

    type PgTestEnableTimeoutAfter = unsafe extern "C" fn(std::ffi::c_int, std::ffi::c_int);
    type PgTestDisableTimeout = unsafe extern "C" fn(std::ffi::c_int, bool);
    type PgTestGetTimeoutIndicator = unsafe extern "C" fn(std::ffi::c_int, bool) -> bool;

    const PG_TEST_STATEMENT_TIMEOUT_ID: std::ffi::c_int = 3;

    struct ScopedPgStatementTimeoutSignal {
        interrupt_pending_ptr: *mut std::ffi::c_int,
        query_cancel_pending_ptr: *mut std::ffi::c_int,
        previous_interrupt_pending: std::ffi::c_int,
        previous_query_cancel_pending: std::ffi::c_int,
        disable_timeout: PgTestDisableTimeout,
        get_timeout_indicator: PgTestGetTimeoutIndicator,
    }

    impl ScopedPgStatementTimeoutSignal {
        unsafe fn trigger_after_ms(delay_ms: std::ffi::c_int) -> Option<Self> {
            unsafe extern "C" {
                fn dlsym(
                    handle: *mut std::ffi::c_void,
                    symbol: *const std::ffi::c_char,
                ) -> *mut std::ffi::c_void;
            }

            let interrupt_pending_ptr =
                unsafe { dlsym(std::ptr::null_mut(), b"InterruptPending\0".as_ptr().cast()) }
                    .cast::<std::ffi::c_int>();
            let query_cancel_pending_ptr = unsafe {
                dlsym(
                    std::ptr::null_mut(),
                    b"QueryCancelPending\0".as_ptr().cast(),
                )
            }
            .cast::<std::ffi::c_int>();
            let enable_timeout_after_ptr = unsafe {
                dlsym(
                    std::ptr::null_mut(),
                    b"enable_timeout_after\0".as_ptr().cast(),
                )
            };
            let disable_timeout_ptr =
                unsafe { dlsym(std::ptr::null_mut(), b"disable_timeout\0".as_ptr().cast()) };
            let get_timeout_indicator_ptr = unsafe {
                dlsym(
                    std::ptr::null_mut(),
                    b"get_timeout_indicator\0".as_ptr().cast(),
                )
            };
            if interrupt_pending_ptr.is_null()
                || query_cancel_pending_ptr.is_null()
                || enable_timeout_after_ptr.is_null()
                || disable_timeout_ptr.is_null()
                || get_timeout_indicator_ptr.is_null()
            {
                return None;
            }
            let enable_timeout_after: PgTestEnableTimeoutAfter =
                unsafe { std::mem::transmute(enable_timeout_after_ptr) };
            let disable_timeout: PgTestDisableTimeout =
                unsafe { std::mem::transmute(disable_timeout_ptr) };
            let get_timeout_indicator: PgTestGetTimeoutIndicator =
                unsafe { std::mem::transmute(get_timeout_indicator_ptr) };
            let previous_interrupt_pending = unsafe { *interrupt_pending_ptr };
            let previous_query_cancel_pending = unsafe { *query_cancel_pending_ptr };
            let guard = Self {
                interrupt_pending_ptr,
                query_cancel_pending_ptr,
                previous_interrupt_pending,
                previous_query_cancel_pending,
                disable_timeout,
                get_timeout_indicator,
            };

            unsafe {
                (guard.get_timeout_indicator)(PG_TEST_STATEMENT_TIMEOUT_ID, true);
                enable_timeout_after(PG_TEST_STATEMENT_TIMEOUT_ID, delay_ms.max(1));
            }
            for _ in 0..50 {
                if guard.statement_timeout_pending() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            Some(guard)
        }

        fn statement_timeout_pending(&self) -> bool {
            unsafe { (self.get_timeout_indicator)(PG_TEST_STATEMENT_TIMEOUT_ID, false) }
        }
    }

    impl Drop for ScopedPgStatementTimeoutSignal {
        fn drop(&mut self) {
            unsafe {
                (self.disable_timeout)(PG_TEST_STATEMENT_TIMEOUT_ID, false);
                (self.get_timeout_indicator)(PG_TEST_STATEMENT_TIMEOUT_ID, true);
                *self.interrupt_pending_ptr = self.previous_interrupt_pending;
                *self.query_cancel_pending_ptr = self.previous_query_cancel_pending;
            }
        }
    }

    fn env_var_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env-var test lock should not be poisoned")
    }

    // Test-schema helper only: external loopback backends run in a different
    // process from the pg_test runner, so env-var based conninfo secrets must
    // be installed in that backend before trigger-dispatch fixtures run.
    #[pg_extern]
    fn ec_spire_test_set_env_var(key: String, value: String) -> bool {
        std::env::set_var(key, value);
        true
    }

    #[pg_extern]
    fn ec_spire_test_rewrite_placement_node(
        index_oid: pg_sys::Oid,
        pid: i64,
        node_id: i32,
    ) -> bool {
        let pid = u64::try_from(pid)
            .unwrap_or_else(|_| pgrx::error!("test placement rewrite pid must be non-negative"));
        let node_id = u32::try_from(node_id).unwrap_or_else(|_| {
            pgrx::error!("test placement rewrite node_id must be non-negative")
        });
        unsafe { am::debug_spire_rewrite_placement_node(index_oid, pid, node_id) };
        true
    }

    #[pg_extern]
    fn ec_spire_test_rewrite_placement_nodes(
        index_oid: pg_sys::Oid,
        pids: Vec<i64>,
        node_ids: Vec<i32>,
    ) -> bool {
        if pids.len() != node_ids.len() {
            pgrx::error!("test placement rewrite pids and node_ids lengths must match");
        }
        let rewrites = pids
            .into_iter()
            .zip(node_ids)
            .map(|(pid, node_id)| {
                let pid = u64::try_from(pid).unwrap_or_else(|_| {
                    pgrx::error!("test placement rewrite pid must be non-negative")
                });
                let node_id = u32::try_from(node_id).unwrap_or_else(|_| {
                    pgrx::error!("test placement rewrite node_id must be non-negative")
                });
                (pid, node_id)
            })
            .collect::<Vec<_>>();
        unsafe { am::debug_spire_rewrite_placement_nodes(index_oid, &rewrites) };
        true
    }

    #[pg_extern]
    fn ec_spire_test_rewrite_consistency_mode(index_oid: pg_sys::Oid, mode: String) -> bool {
        unsafe { am::debug_spire_rewrite_consistency_mode(index_oid, &mode) };
        true
    }

    fn ec_spire_test_transport_probe_requests(
        function_name: &str,
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        slow_node_id: i32,
    ) -> Vec<am::SpireRemoteProductionTransportProbeRequest> {
        if node_ids.len() != conninfo_secret_names.len() {
            pgrx::error!("{function_name} node_ids and conninfo_secret_names lengths must match");
        }
        let slow_node_id = u32::try_from(slow_node_id)
            .unwrap_or_else(|_| pgrx::error!("{function_name} slow_node_id must be non-negative"));
        node_ids
            .into_iter()
            .zip(conninfo_secret_names)
            .map(|(node_id, conninfo_secret_name)| {
                let node_id = u32::try_from(node_id).unwrap_or_else(|_| {
                    pgrx::error!("{function_name} node_id must be non-negative")
                });
                let provider_lookup_key =
                    am::spire_remote_conninfo_secret_provider_lookup_key(&conninfo_secret_name)
                        .unwrap_or_else(|e| pgrx::error!("{function_name} {e}"));
                let conninfo = std::env::var(&provider_lookup_key).unwrap_or_else(|_| {
                    pgrx::error!("{function_name} missing conninfo secret {conninfo_secret_name}")
                });
                if conninfo.is_empty() {
                    pgrx::error!("{function_name} empty conninfo secret {conninfo_secret_name}");
                }
                am::SpireRemoteProductionTransportProbeRequest {
                    node_id,
                    conninfo,
                    sql: if node_id == slow_node_id {
                        "SELECT pg_sleep(0.30)"
                    } else {
                        "SELECT 1"
                    },
                }
            })
            .collect::<Vec<_>>()
    }

    fn ec_spire_test_transport_probe_case_requests(
        function_name: &str,
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        fault_node_id: i32,
        fault_case: &str,
    ) -> Vec<am::SpireRemoteProductionTransportProbeRequest> {
        if node_ids.len() != conninfo_secret_names.len() {
            pgrx::error!("{function_name} node_ids and conninfo_secret_names lengths must match");
        }
        let fault_node_id = u32::try_from(fault_node_id)
            .unwrap_or_else(|_| pgrx::error!("{function_name} fault_node_id must be non-negative"));
        let fault_sql = match fault_case {
            "connection_reset_mid_batch" => {
                "SELECT CASE WHEN g = 1 THEN 1 ELSE CASE WHEN pg_terminate_backend(pg_backend_pid()) THEN 2 ELSE 3 END END FROM generate_series(1, 2) AS g"
            }
            "remote_oom" => {
                "DO $$ BEGIN RAISE EXCEPTION 'simulated remote out of memory' USING ERRCODE = '53200'; END $$"
            }
            "remote_statement_timeout" => "SELECT pg_sleep(0.30)",
            "remote_backend_termination" => "SELECT pg_terminate_backend(pg_backend_pid())",
            other => pgrx::error!("{function_name} unsupported fault_case {other}"),
        };
        node_ids
            .into_iter()
            .zip(conninfo_secret_names)
            .map(|(node_id, conninfo_secret_name)| {
                let node_id = u32::try_from(node_id).unwrap_or_else(|_| {
                    pgrx::error!("{function_name} node_id must be non-negative")
                });
                let provider_lookup_key =
                    am::spire_remote_conninfo_secret_provider_lookup_key(&conninfo_secret_name)
                        .unwrap_or_else(|e| pgrx::error!("{function_name} {e}"));
                let conninfo = std::env::var(&provider_lookup_key).unwrap_or_else(|_| {
                    pgrx::error!("{function_name} missing conninfo secret {conninfo_secret_name}")
                });
                if conninfo.is_empty() {
                    pgrx::error!("{function_name} empty conninfo secret {conninfo_secret_name}");
                }
                am::SpireRemoteProductionTransportProbeRequest {
                    node_id,
                    conninfo,
                    sql: if node_id == fault_node_id {
                        fault_sql
                    } else {
                        "SELECT 1"
                    },
                }
            })
            .collect::<Vec<_>>()
    }

    fn ec_spire_test_transport_probe_local_cancel_requests(
        function_name: &str,
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
    ) -> Vec<am::SpireRemoteProductionTransportProbeRequest> {
        if node_ids.len() != conninfo_secret_names.len() {
            pgrx::error!("{function_name} node_ids and conninfo_secret_names lengths must match");
        }
        node_ids
            .into_iter()
            .zip(conninfo_secret_names)
            .map(|(node_id, conninfo_secret_name)| {
                let node_id = u32::try_from(node_id).unwrap_or_else(|_| {
                    pgrx::error!("{function_name} node_id must be non-negative")
                });
                let provider_lookup_key =
                    am::spire_remote_conninfo_secret_provider_lookup_key(&conninfo_secret_name)
                        .unwrap_or_else(|e| pgrx::error!("{function_name} {e}"));
                let conninfo = std::env::var(&provider_lookup_key).unwrap_or_else(|_| {
                    pgrx::error!("{function_name} missing conninfo secret {conninfo_secret_name}")
                });
                if conninfo.is_empty() {
                    pgrx::error!("{function_name} empty conninfo secret {conninfo_secret_name}");
                }
                am::SpireRemoteProductionTransportProbeRequest {
                    node_id,
                    conninfo,
                    sql: "SELECT pg_sleep(0.30)",
                }
            })
            .collect::<Vec<_>>()
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_production_transport_probe(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        slow_node_id: i32,
    ) -> TableIterator<
        'static,
        (
            name!(node_id, i64),
            name!(started_after_ms, i64),
            name!(completed_after_ms, i64),
            name!(elapsed_ms, i64),
            name!(row_count, i64),
            name!(status, &'static str),
            name!(failure_category, &'static str),
        ),
    > {
        let requests = ec_spire_test_transport_probe_requests(
            "ec_spire_test_production_transport_probe",
            node_ids,
            conninfo_secret_names,
            slow_node_id,
        );
        let rows = am::spire_remote_search_production_transport_probe_for_test(requests);

        TableIterator::new(rows.into_iter().map(|row| {
            (
                i64::from(row.node_id),
                i64::try_from(row.started_after_ms).expect("started_after_ms should fit in i64"),
                i64::try_from(row.completed_after_ms)
                    .expect("completed_after_ms should fit in i64"),
                i64::try_from(row.elapsed_ms).expect("elapsed_ms should fit in i64"),
                i64::try_from(row.row_count).expect("row count should fit in i64"),
                row.status,
                row.failure_category,
            )
        }))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_production_transport_probe_case(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        fault_node_id: i32,
        fault_case: String,
    ) -> TableIterator<
        'static,
        (
            name!(node_id, i64),
            name!(started_after_ms, i64),
            name!(completed_after_ms, i64),
            name!(elapsed_ms, i64),
            name!(row_count, i64),
            name!(status, &'static str),
            name!(failure_category, &'static str),
        ),
    > {
        let requests = ec_spire_test_transport_probe_case_requests(
            "ec_spire_test_production_transport_probe_case",
            node_ids,
            conninfo_secret_names,
            fault_node_id,
            &fault_case,
        );
        let rows = am::spire_remote_search_production_transport_probe_for_test(requests);

        TableIterator::new(rows.into_iter().map(|row| {
            (
                i64::from(row.node_id),
                i64::try_from(row.started_after_ms).expect("started_after_ms should fit in i64"),
                i64::try_from(row.completed_after_ms)
                    .expect("completed_after_ms should fit in i64"),
                i64::try_from(row.elapsed_ms).expect("elapsed_ms should fit in i64"),
                i64::try_from(row.row_count).expect("row count should fit in i64"),
                row.status,
                row.failure_category,
            )
        }))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_production_transport_probe_summary(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        slow_node_id: i32,
        consistency_mode: String,
    ) -> TableIterator<
        'static,
        (
            name!(state_model, &'static str),
            name!(dispatch_count, i64),
            name!(transport_sent_dispatch_count, i64),
            name!(transport_ready_dispatch_count, i64),
            name!(transport_failed_dispatch_count, i64),
            name!(first_transport_failure_category, &'static str),
            name!(candidate_receive_pending_dispatch_count, i64),
            name!(degraded_skipped_dispatch_count, i64),
            name!(first_degraded_skip_category, &'static str),
            name!(next_executor_step, &'static str),
            name!(status, &'static str),
        ),
    > {
        let requests = ec_spire_test_transport_probe_requests(
            "ec_spire_test_production_transport_probe_summary",
            node_ids,
            conninfo_secret_names,
            slow_node_id,
        );
        let row = am::spire_remote_search_production_transport_probe_summary_for_test(
            requests,
            &consistency_mode,
        );

        TableIterator::once((
            row.state_model,
            i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
            i64::try_from(row.transport_sent_dispatch_count)
                .expect("transport sent count should fit in i64"),
            i64::try_from(row.transport_ready_dispatch_count)
                .expect("transport ready count should fit in i64"),
            i64::try_from(row.transport_failed_dispatch_count)
                .expect("transport failed count should fit in i64"),
            row.first_transport_failure_category,
            i64::try_from(row.candidate_receive_pending_dispatch_count)
                .expect("candidate receive pending count should fit in i64"),
            i64::try_from(row.degraded_skipped_dispatch_count)
                .expect("degraded skipped count should fit in i64"),
            row.first_degraded_skip_category,
            row.next_executor_step,
            row.status,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_production_transport_probe_case_summary(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        fault_node_id: i32,
        fault_case: String,
        consistency_mode: String,
    ) -> TableIterator<
        'static,
        (
            name!(state_model, &'static str),
            name!(dispatch_count, i64),
            name!(transport_sent_dispatch_count, i64),
            name!(transport_ready_dispatch_count, i64),
            name!(transport_failed_dispatch_count, i64),
            name!(first_transport_failure_category, &'static str),
            name!(candidate_receive_pending_dispatch_count, i64),
            name!(degraded_skipped_dispatch_count, i64),
            name!(first_degraded_skip_category, &'static str),
            name!(next_executor_step, &'static str),
            name!(status, &'static str),
        ),
    > {
        let requests = ec_spire_test_transport_probe_case_requests(
            "ec_spire_test_production_transport_probe_case_summary",
            node_ids,
            conninfo_secret_names,
            fault_node_id,
            &fault_case,
        );
        let row = am::spire_remote_search_production_transport_probe_summary_for_test(
            requests,
            &consistency_mode,
        );

        TableIterator::once((
            row.state_model,
            i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
            i64::try_from(row.transport_sent_dispatch_count)
                .expect("transport sent count should fit in i64"),
            i64::try_from(row.transport_ready_dispatch_count)
                .expect("transport ready count should fit in i64"),
            i64::try_from(row.transport_failed_dispatch_count)
                .expect("transport failed count should fit in i64"),
            row.first_transport_failure_category,
            i64::try_from(row.candidate_receive_pending_dispatch_count)
                .expect("candidate receive pending count should fit in i64"),
            i64::try_from(row.degraded_skipped_dispatch_count)
                .expect("degraded skipped count should fit in i64"),
            row.first_degraded_skip_category,
            row.next_executor_step,
            row.status,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_production_transport_probe_local_cancel(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        local_cancel_after_ms: i64,
    ) -> TableIterator<
        'static,
        (
            name!(node_id, i64),
            name!(started_after_ms, i64),
            name!(completed_after_ms, i64),
            name!(elapsed_ms, i64),
            name!(row_count, i64),
            name!(status, &'static str),
            name!(failure_category, &'static str),
        ),
    > {
        let local_cancel_after_ms = u64::try_from(local_cancel_after_ms)
            .unwrap_or_else(|_| pgrx::error!("local_cancel_after_ms must be non-negative"));
        let requests = ec_spire_test_transport_probe_local_cancel_requests(
            "ec_spire_test_production_transport_probe_local_cancel",
            node_ids,
            conninfo_secret_names,
        );
        let rows = am::spire_remote_search_production_transport_probe_with_local_cancel_for_test(
            requests,
            local_cancel_after_ms,
        );

        TableIterator::new(rows.into_iter().map(|row| {
            (
                i64::from(row.node_id),
                i64::try_from(row.started_after_ms).expect("started_after_ms should fit in i64"),
                i64::try_from(row.completed_after_ms)
                    .expect("completed_after_ms should fit in i64"),
                i64::try_from(row.elapsed_ms).expect("elapsed_ms should fit in i64"),
                i64::try_from(row.row_count).expect("row count should fit in i64"),
                row.status,
                row.failure_category,
            )
        }))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_production_transport_probe_local_cancel_summary(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        local_cancel_after_ms: i64,
        consistency_mode: String,
    ) -> TableIterator<
        'static,
        (
            name!(state_model, &'static str),
            name!(dispatch_count, i64),
            name!(transport_sent_dispatch_count, i64),
            name!(transport_ready_dispatch_count, i64),
            name!(transport_failed_dispatch_count, i64),
            name!(candidate_receive_pending_dispatch_count, i64),
            name!(cancelled_dispatch_count, i64),
            name!(first_cancellation_category, &'static str),
            name!(degraded_skipped_dispatch_count, i64),
            name!(first_degraded_skip_category, &'static str),
            name!(next_executor_step, &'static str),
            name!(status, &'static str),
        ),
    > {
        let local_cancel_after_ms = u64::try_from(local_cancel_after_ms)
            .unwrap_or_else(|_| pgrx::error!("local_cancel_after_ms must be non-negative"));
        let requests = ec_spire_test_transport_probe_local_cancel_requests(
            "ec_spire_test_production_transport_probe_local_cancel_summary",
            node_ids,
            conninfo_secret_names,
        );
        let row =
            am::spire_remote_search_production_transport_probe_with_local_cancel_summary_for_test(
                requests,
                local_cancel_after_ms,
                &consistency_mode,
            );

        TableIterator::once((
            row.state_model,
            i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
            i64::try_from(row.transport_sent_dispatch_count)
                .expect("transport sent count should fit in i64"),
            i64::try_from(row.transport_ready_dispatch_count)
                .expect("transport ready count should fit in i64"),
            i64::try_from(row.transport_failed_dispatch_count)
                .expect("transport failed count should fit in i64"),
            i64::try_from(row.candidate_receive_pending_dispatch_count)
                .expect("candidate receive pending count should fit in i64"),
            i64::try_from(row.cancelled_dispatch_count).expect("cancelled count should fit in i64"),
            row.first_cancellation_category,
            i64::try_from(row.degraded_skipped_dispatch_count)
                .expect("degraded skipped count should fit in i64"),
            row.first_degraded_skip_category,
            row.next_executor_step,
            row.status,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_prod_transport_stmt_timeout(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        statement_timeout_after_ms: i32,
    ) -> TableIterator<
        'static,
        (
            name!(node_id, i64),
            name!(started_after_ms, i64),
            name!(completed_after_ms, i64),
            name!(elapsed_ms, i64),
            name!(row_count, i64),
            name!(status, &'static str),
            name!(failure_category, &'static str),
        ),
    > {
        let _interrupt_lock = env_var_test_lock();
        let timeout_signal =
            unsafe { ScopedPgStatementTimeoutSignal::trigger_after_ms(statement_timeout_after_ms) }
                .unwrap_or_else(|| {
                    pgrx::error!(
                        "ec_spire_test_prod_transport_stmt_timeout could not resolve PostgreSQL timeout symbols"
                    )
                });
        if !timeout_signal.statement_timeout_pending() {
            pgrx::error!(
                "ec_spire_test_prod_transport_stmt_timeout did not observe a pending statement timeout"
            );
        }
        let requests = ec_spire_test_transport_probe_local_cancel_requests(
            "ec_spire_test_prod_transport_stmt_timeout",
            node_ids,
            conninfo_secret_names,
        );
        let rows = am::spire_remote_search_production_transport_probe_for_test(requests);

        TableIterator::new(rows.into_iter().map(|row| {
            (
                i64::from(row.node_id),
                i64::try_from(row.started_after_ms).expect("started_after_ms should fit in i64"),
                i64::try_from(row.completed_after_ms)
                    .expect("completed_after_ms should fit in i64"),
                i64::try_from(row.elapsed_ms).expect("elapsed_ms should fit in i64"),
                i64::try_from(row.row_count).expect("row count should fit in i64"),
                row.status,
                row.failure_category,
            )
        }))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_prod_transport_stmt_timeout_summary(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        statement_timeout_after_ms: i32,
        consistency_mode: String,
    ) -> TableIterator<
        'static,
        (
            name!(state_model, &'static str),
            name!(dispatch_count, i64),
            name!(transport_sent_dispatch_count, i64),
            name!(transport_ready_dispatch_count, i64),
            name!(transport_failed_dispatch_count, i64),
            name!(candidate_receive_pending_dispatch_count, i64),
            name!(cancelled_dispatch_count, i64),
            name!(first_cancellation_category, &'static str),
            name!(degraded_skipped_dispatch_count, i64),
            name!(first_degraded_skip_category, &'static str),
            name!(next_executor_step, &'static str),
            name!(status, &'static str),
        ),
    > {
        let _interrupt_lock = env_var_test_lock();
        let timeout_signal = unsafe {
            ScopedPgStatementTimeoutSignal::trigger_after_ms(statement_timeout_after_ms)
        }
        .unwrap_or_else(|| {
            pgrx::error!(
                "ec_spire_test_prod_transport_stmt_timeout_summary could not resolve PostgreSQL timeout symbols"
            )
        });
        if !timeout_signal.statement_timeout_pending() {
            pgrx::error!(
                "ec_spire_test_prod_transport_stmt_timeout_summary did not observe a pending statement timeout"
            );
        }
        let requests = ec_spire_test_transport_probe_local_cancel_requests(
            "ec_spire_test_prod_transport_stmt_timeout_summary",
            node_ids,
            conninfo_secret_names,
        );
        let row = am::spire_remote_search_production_transport_probe_summary_for_test(
            requests,
            &consistency_mode,
        );

        TableIterator::once((
            row.state_model,
            i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
            i64::try_from(row.transport_sent_dispatch_count)
                .expect("transport sent count should fit in i64"),
            i64::try_from(row.transport_ready_dispatch_count)
                .expect("transport ready count should fit in i64"),
            i64::try_from(row.transport_failed_dispatch_count)
                .expect("transport failed count should fit in i64"),
            i64::try_from(row.candidate_receive_pending_dispatch_count)
                .expect("candidate receive pending count should fit in i64"),
            i64::try_from(row.cancelled_dispatch_count).expect("cancelled count should fit in i64"),
            row.first_cancellation_category,
            i64::try_from(row.degraded_skipped_dispatch_count)
                .expect("degraded skipped count should fit in i64"),
            row.first_degraded_skip_category,
            row.next_executor_step,
            row.status,
        ))
    }

    fn ec_spire_test_candidate_receive_requests(
        function_name: &str,
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        remote_index_regclasses: Vec<String>,
        remote_index_identity_hexes: Vec<String>,
        selected_pids: Vec<i64>,
        requested_epoch: i64,
        query: Vec<f32>,
        top_k: i32,
        consistency_mode: String,
    ) -> Vec<am::SpireRemoteProductionCandidateReceiveRequest> {
        if node_ids.len() != conninfo_secret_names.len()
            || node_ids.len() != remote_index_regclasses.len()
            || node_ids.len() != remote_index_identity_hexes.len()
        {
            pgrx::error!("{function_name} request arrays must have matching lengths");
        }
        let requested_epoch = u64::try_from(requested_epoch).unwrap_or_else(|_| {
            pgrx::error!("{function_name} requested_epoch must be non-negative")
        });
        let top_k = usize::try_from(top_k)
            .unwrap_or_else(|_| pgrx::error!("{function_name} top_k must be non-negative"));
        let selected_pids = selected_pids
            .into_iter()
            .map(|selected_pid| {
                u64::try_from(selected_pid).unwrap_or_else(|_| {
                    pgrx::error!("{function_name} selected_pid must be non-negative")
                })
            })
            .collect::<Vec<_>>();
        node_ids
            .into_iter()
            .zip(conninfo_secret_names)
            .zip(remote_index_regclasses)
            .zip(remote_index_identity_hexes)
            .map(
                |(((node_id, conninfo_secret_name), remote_index_regclass), identity_hex)| {
                    let node_id = u32::try_from(node_id).unwrap_or_else(|_| {
                        pgrx::error!("{function_name} node_id must be non-negative")
                    });
                    let provider_lookup_key =
                        am::spire_remote_conninfo_secret_provider_lookup_key(&conninfo_secret_name)
                            .unwrap_or_else(|e| pgrx::error!("{function_name} {e}"));
                    let conninfo = std::env::var(&provider_lookup_key).unwrap_or_else(|_| {
                        pgrx::error!(
                            "{function_name} missing conninfo secret {conninfo_secret_name}"
                        )
                    });
                    if conninfo.is_empty() {
                        pgrx::error!(
                            "{function_name} empty conninfo secret {conninfo_secret_name}"
                        );
                    }
                    let remote_index_identity = hex::decode(&identity_hex).unwrap_or_else(|e| {
                        pgrx::error!("{function_name} remote_index_identity hex decode failed: {e}")
                    });
                    am::SpireRemoteProductionCandidateReceiveRequest {
                        node_id,
                        conninfo,
                        remote_index_regclass,
                        remote_index_identity,
                        requested_epoch,
                        query: query.clone(),
                        selected_pids: selected_pids.clone(),
                        top_k,
                        consistency_mode: consistency_mode.clone(),
                    }
                },
            )
            .collect::<Vec<_>>()
    }

    fn ec_spire_test_conninfo_from_secret(
        function_name: &str,
        conninfo_secret_name: &str,
    ) -> String {
        let provider_lookup_key =
            am::spire_remote_conninfo_secret_provider_lookup_key(conninfo_secret_name)
                .unwrap_or_else(|e| pgrx::error!("{function_name} {e}"));
        let conninfo = std::env::var(&provider_lookup_key).unwrap_or_else(|_| {
            pgrx::error!("{function_name} missing conninfo secret {conninfo_secret_name}")
        });
        if conninfo.is_empty() {
            pgrx::error!("{function_name} empty conninfo secret {conninfo_secret_name}");
        }
        conninfo
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_remote_conninfo_tls_probe(
        conninfo: String,
    ) -> TableIterator<
        'static,
        (
            name!(connection_status, String),
            name!(ssl, bool),
            name!(tls_version, String),
            name!(error, String),
        ),
    > {
        let mut client = match am::spire_remote_search_libpq_connect_with_session_timeouts(
            &conninfo,
            1,
            "test remote TLS probe",
        ) {
            Ok(client) => client,
            Err(error) => {
                return TableIterator::once((
                    "connect_failed".to_owned(),
                    false,
                    String::new(),
                    error,
                ));
            }
        };
        let row = match client.query_one(
            "SELECT ssl, coalesce(version, '') \
               FROM pg_stat_ssl \
              WHERE pid = pg_backend_pid()",
            &[],
        ) {
            Ok(row) => row,
            Err(error) => {
                return TableIterator::once((
                    "probe_failed".to_owned(),
                    false,
                    String::new(),
                    error.to_string(),
                ));
            }
        };

        let ssl = row.try_get::<_, bool>(0).unwrap_or(false);
        let tls_version = row.try_get::<_, String>(1).unwrap_or_default();
        TableIterator::once(("connected".to_owned(), ssl, tls_version, String::new()))
    }

    fn ec_spire_test_run_remote_sql_after_request_build(
        function_name: &str,
        conninfo_secret_name: &str,
        remote_sql: &str,
    ) {
        let conninfo = ec_spire_test_conninfo_from_secret(function_name, conninfo_secret_name);
        let mut client = postgres::Client::connect(&conninfo, postgres::NoTls)
            .unwrap_or_else(|e| pgrx::error!("{function_name} remote SQL connection failed: {e}"));
        client
            .batch_execute(remote_sql)
            .unwrap_or_else(|e| pgrx::error!("{function_name} remote SQL execution failed: {e}"));
    }

    fn ec_spire_test_remote_endpoint_identity_hex(
        function_name: &str,
        conninfo_secret_name: &str,
        remote_index_regclass: &str,
    ) -> String {
        let conninfo = ec_spire_test_conninfo_from_secret(function_name, conninfo_secret_name);
        let mut client =
            postgres::Client::connect(&conninfo, postgres::NoTls).unwrap_or_else(|e| {
                pgrx::error!("{function_name} remote endpoint identity connection failed: {e}")
            });
        let remote_index_regclass = remote_index_regclass.replace('\'', "''");
        client
            .query_one(
                &format!(
                    "SELECT profile_fingerprint \
                       FROM ec_spire_remote_search_endpoint_identity('{remote_index_regclass}'::regclass)"
                ),
                &[],
            )
            .and_then(|row| row.try_get::<_, String>(0))
            .unwrap_or_else(|e| {
                pgrx::error!("{function_name} remote endpoint identity query failed: {e}")
            })
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_production_candidate_receive(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        remote_index_regclasses: Vec<String>,
        remote_index_identity_hexes: Vec<String>,
        selected_pids: Vec<i64>,
        requested_epoch: i64,
        query: Vec<f32>,
        top_k: i32,
        consistency_mode: String,
    ) -> TableIterator<
        'static,
        (
            name!(node_id, i64),
            name!(started_after_ms, i64),
            name!(completed_after_ms, i64),
            name!(elapsed_ms, i64),
            name!(candidate_count, i64),
            name!(status, &'static str),
            name!(failure_category, &'static str),
        ),
    > {
        let requests = ec_spire_test_candidate_receive_requests(
            "ec_spire_test_production_candidate_receive",
            node_ids,
            conninfo_secret_names,
            remote_index_regclasses,
            remote_index_identity_hexes,
            selected_pids,
            requested_epoch,
            query,
            top_k,
            consistency_mode,
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(requests);

        TableIterator::new(rows.into_iter().map(|row| {
            (
                i64::from(row.node_id),
                i64::try_from(row.started_after_ms).expect("started_after_ms should fit in i64"),
                i64::try_from(row.completed_after_ms)
                    .expect("completed_after_ms should fit in i64"),
                i64::try_from(row.elapsed_ms).expect("elapsed_ms should fit in i64"),
                i64::try_from(row.candidate_count).expect("candidate count should fit in i64"),
                row.status,
                row.failure_category,
            )
        }))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_prod_receive_after_remote_sql(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        remote_index_regclasses: Vec<String>,
        remote_index_identity_hexes: Vec<String>,
        selected_pids: Vec<i64>,
        requested_epoch: i64,
        query: Vec<f32>,
        top_k: i32,
        consistency_mode: String,
        remote_sql_conninfo_secret_name: String,
        remote_sql: String,
    ) -> TableIterator<
        'static,
        (
            name!(node_id, i64),
            name!(started_after_ms, i64),
            name!(completed_after_ms, i64),
            name!(elapsed_ms, i64),
            name!(candidate_count, i64),
            name!(status, &'static str),
            name!(failure_category, &'static str),
        ),
    > {
        let requests = ec_spire_test_candidate_receive_requests(
            "ec_spire_test_prod_receive_after_remote_sql",
            node_ids,
            conninfo_secret_names,
            remote_index_regclasses,
            remote_index_identity_hexes,
            selected_pids,
            requested_epoch,
            query,
            top_k,
            consistency_mode,
        );
        ec_spire_test_run_remote_sql_after_request_build(
            "ec_spire_test_prod_receive_after_remote_sql",
            &remote_sql_conninfo_secret_name,
            &remote_sql,
        );
        let rows = am::spire_remote_search_production_candidate_receive_for_test(requests);

        TableIterator::new(rows.into_iter().map(|row| {
            (
                i64::from(row.node_id),
                i64::try_from(row.started_after_ms).expect("started_after_ms should fit in i64"),
                i64::try_from(row.completed_after_ms)
                    .expect("completed_after_ms should fit in i64"),
                i64::try_from(row.elapsed_ms).expect("elapsed_ms should fit in i64"),
                i64::try_from(row.candidate_count).expect("candidate count should fit in i64"),
                row.status,
                row.failure_category,
            )
        }))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_production_candidate_receive_summary(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        remote_index_regclasses: Vec<String>,
        remote_index_identity_hexes: Vec<String>,
        selected_pids: Vec<i64>,
        requested_epoch: i64,
        query: Vec<f32>,
        top_k: i32,
        consistency_mode: String,
    ) -> TableIterator<
        'static,
        (
            name!(state_model, &'static str),
            name!(dispatch_count, i64),
            name!(candidate_receive_sent_dispatch_count, i64),
            name!(candidate_receive_ready_dispatch_count, i64),
            name!(candidate_receive_failed_dispatch_count, i64),
            name!(first_candidate_receive_failure_category, &'static str),
            name!(candidate_row_count, i64),
            name!(degraded_skipped_dispatch_count, i64),
            name!(first_degraded_skip_category, &'static str),
            name!(next_executor_step, &'static str),
            name!(status, &'static str),
        ),
    > {
        let requests = ec_spire_test_candidate_receive_requests(
            "ec_spire_test_production_candidate_receive_summary",
            node_ids,
            conninfo_secret_names,
            remote_index_regclasses,
            remote_index_identity_hexes,
            selected_pids,
            requested_epoch,
            query,
            top_k,
            consistency_mode.clone(),
        );
        let row = am::spire_remote_search_production_candidate_receive_summary_for_test(
            requests,
            &consistency_mode,
        );

        TableIterator::once((
            row.state_model,
            i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
            i64::try_from(row.candidate_receive_sent_dispatch_count)
                .expect("candidate receive sent count should fit in i64"),
            i64::try_from(row.candidate_receive_ready_dispatch_count)
                .expect("candidate receive ready count should fit in i64"),
            i64::try_from(row.candidate_receive_failed_dispatch_count)
                .expect("candidate receive failed count should fit in i64"),
            row.first_candidate_receive_failure_category,
            i64::try_from(row.candidate_row_count).expect("candidate row count should fit in i64"),
            i64::try_from(row.degraded_skipped_dispatch_count)
                .expect("degraded skipped count should fit in i64"),
            row.first_degraded_skip_category,
            row.next_executor_step,
            row.status,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_spire_test_prod_receive_after_remote_sql_summary(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        remote_index_regclasses: Vec<String>,
        remote_index_identity_hexes: Vec<String>,
        selected_pids: Vec<i64>,
        requested_epoch: i64,
        query: Vec<f32>,
        top_k: i32,
        consistency_mode: String,
        remote_sql_conninfo_secret_name: String,
        remote_sql: String,
    ) -> TableIterator<
        'static,
        (
            name!(state_model, &'static str),
            name!(dispatch_count, i64),
            name!(candidate_receive_sent_dispatch_count, i64),
            name!(candidate_receive_ready_dispatch_count, i64),
            name!(candidate_receive_failed_dispatch_count, i64),
            name!(first_candidate_receive_failure_category, &'static str),
            name!(candidate_row_count, i64),
            name!(degraded_skipped_dispatch_count, i64),
            name!(first_degraded_skip_category, &'static str),
            name!(next_executor_step, &'static str),
            name!(status, &'static str),
        ),
    > {
        let requests = ec_spire_test_candidate_receive_requests(
            "ec_spire_test_prod_receive_after_remote_sql_summary",
            node_ids,
            conninfo_secret_names,
            remote_index_regclasses,
            remote_index_identity_hexes,
            selected_pids,
            requested_epoch,
            query,
            top_k,
            consistency_mode.clone(),
        );
        ec_spire_test_run_remote_sql_after_request_build(
            "ec_spire_test_prod_receive_after_remote_sql_summary",
            &remote_sql_conninfo_secret_name,
            &remote_sql,
        );
        let row = am::spire_remote_search_production_candidate_receive_summary_for_test(
            requests,
            &consistency_mode,
        );

        TableIterator::once((
            row.state_model,
            i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
            i64::try_from(row.candidate_receive_sent_dispatch_count)
                .expect("candidate receive sent count should fit in i64"),
            i64::try_from(row.candidate_receive_ready_dispatch_count)
                .expect("candidate receive ready count should fit in i64"),
            i64::try_from(row.candidate_receive_failed_dispatch_count)
                .expect("candidate receive failed count should fit in i64"),
            row.first_candidate_receive_failure_category,
            i64::try_from(row.candidate_row_count).expect("candidate row count should fit in i64"),
            i64::try_from(row.degraded_skipped_dispatch_count)
                .expect("degraded skipped count should fit in i64"),
            row.first_degraded_skip_category,
            row.next_executor_step,
            row.status,
        ))
    }

    #[pg_extern]
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn ec_spire_test_prod_receive_after_remote_descriptor_summary(
        node_ids: Vec<i32>,
        conninfo_secret_names: Vec<String>,
        remote_index_regclasses: Vec<String>,
        remote_index_identity_hexes: Vec<String>,
        selected_pids: Vec<i64>,
        requested_epoch: i64,
        query: Vec<f32>,
        top_k: i32,
        consistency_mode: String,
        remote_sql_conninfo_secret_name: String,
        remote_sql: String,
        descriptor_index_oid: pg_sys::Oid,
        descriptor_node_id: i32,
        descriptor_generation: i64,
        descriptor_conninfo_secret_name: String,
        descriptor_remote_index_regclass: String,
        descriptor_state: String,
        descriptor_last_served_epoch: i64,
        descriptor_min_retained_epoch: i64,
        descriptor_extension_version: String,
        descriptor_last_error: String,
    ) -> TableIterator<
        'static,
        (
            name!(state_model, &'static str),
            name!(dispatch_count, i64),
            name!(candidate_receive_sent_dispatch_count, i64),
            name!(candidate_receive_ready_dispatch_count, i64),
            name!(candidate_receive_failed_dispatch_count, i64),
            name!(first_candidate_receive_failure_category, &'static str),
            name!(candidate_row_count, i64),
            name!(degraded_skipped_dispatch_count, i64),
            name!(first_degraded_skip_category, &'static str),
            name!(next_executor_step, &'static str),
            name!(status, &'static str),
        ),
    > {
        let function_name = "ec_spire_test_prod_receive_after_remote_descriptor_summary";
        let requests = ec_spire_test_candidate_receive_requests(
            function_name,
            node_ids,
            conninfo_secret_names,
            remote_index_regclasses,
            remote_index_identity_hexes,
            selected_pids,
            requested_epoch,
            query,
            top_k,
            consistency_mode.clone(),
        );
        ec_spire_test_run_remote_sql_after_request_build(
            function_name,
            &remote_sql_conninfo_secret_name,
            &remote_sql,
        );
        let descriptor_identity_hex = ec_spire_test_remote_endpoint_identity_hex(
            function_name,
            &remote_sql_conninfo_secret_name,
            &descriptor_remote_index_regclass,
        );
        let descriptor_identity = hex::decode(&descriptor_identity_hex).unwrap_or_else(|e| {
            pgrx::error!("{function_name} descriptor identity decode failed: {e}")
        });
        ec_spire_register_remote_node_descriptor(
            descriptor_index_oid,
            descriptor_node_id,
            descriptor_generation,
            descriptor_conninfo_secret_name,
            descriptor_identity,
            descriptor_remote_index_regclass,
            descriptor_state,
            descriptor_last_served_epoch,
            descriptor_min_retained_epoch,
            descriptor_extension_version,
            descriptor_last_error,
        );
        let row = am::spire_remote_search_production_candidate_receive_summary_for_test(
            requests,
            &consistency_mode,
        );

        TableIterator::once((
            row.state_model,
            i64::try_from(row.dispatch_count).expect("dispatch count should fit in i64"),
            i64::try_from(row.candidate_receive_sent_dispatch_count)
                .expect("candidate receive sent count should fit in i64"),
            i64::try_from(row.candidate_receive_ready_dispatch_count)
                .expect("candidate receive ready count should fit in i64"),
            i64::try_from(row.candidate_receive_failed_dispatch_count)
                .expect("candidate receive failed count should fit in i64"),
            row.first_candidate_receive_failure_category,
            i64::try_from(row.candidate_row_count).expect("candidate row count should fit in i64"),
            i64::try_from(row.degraded_skipped_dispatch_count)
                .expect("degraded skipped count should fit in i64"),
            row.first_degraded_skip_category,
            row.next_executor_step,
            row.status,
        ))
    }

    fn current_pg_test_loopback_conninfo() -> String {
        let socket_dirs = Spi::get_one::<String>("SHOW unix_socket_directories")
            .expect("socket directory query should succeed")
            .expect("socket directory should exist");
        let socket_dir = socket_dirs
            .split(',')
            .next()
            .expect("at least one socket directory should exist")
            .trim();
        let port = Spi::get_one::<String>("SHOW port")
            .expect("port query should succeed")
            .expect("port should exist");
        let database = Spi::get_one::<String>("SELECT current_database()::text")
            .expect("database query should succeed")
            .expect("database should exist");
        let user = Spi::get_one::<String>("SELECT current_user::text")
            .expect("user query should succeed")
            .expect("user should exist");

        format!("host={socket_dir} port={port} dbname={database} user={user} connect_timeout=1")
    }

    struct TestCoordinatorInsertPrepareResult {
        prepared_gid: String,
        status: &'static str,
        next_step: &'static str,
    }

    fn assert_stable_spire_prepared_gid(
        gid: &str,
        index_oid: pg_sys::Oid,
        node_id: i32,
        served_epoch: i64,
    ) {
        let gid_parts = gid.split('_').collect::<Vec<_>>();
        assert_eq!(
            gid_parts.len(),
            7,
            "SPIRE prepared GID should omit volatile backend pid: {gid}"
        );
        assert_eq!(&gid_parts[0..3], ["ec", "spire", "insert"]);
        assert_eq!(gid_parts[3], u32::from(index_oid).to_string());
        assert_eq!(gid_parts[4], node_id.to_string());
        assert_eq!(gid_parts[5], served_epoch.to_string());
        let top_xid = gid_parts[6]
            .parse::<u32>()
            .expect("prepared GID top transaction id should be numeric");
        assert!(top_xid > 0, "prepared GID should carry a real top xid");
    }

    #[allow(clippy::too_many_arguments)]
    fn test_prepare_coordinator_insert_remote_sql(
        index_oid: pg_sys::Oid,
        pk_value: Vec<u8>,
        node_id: i32,
        centroid_id: i64,
        served_epoch: i64,
        source_identity: Vec<u8>,
        remote_sql: &str,
    ) -> TestCoordinatorInsertPrepareResult {
        let node_id_u32 = u32::try_from(node_id).expect("node_id should fit u32");
        let served_epoch_u64 = u64::try_from(served_epoch).expect("served_epoch should fit u64");
        let index_relation = unsafe {
            open_valid_ec_spire_index(index_oid, "test_prepare_coordinator_insert_remote_sql")
        };
        let row = unsafe {
            am::spire_coordinator_insert_prepare_remote_sql(
                index_relation,
                node_id_u32,
                served_epoch_u64,
                remote_sql,
            )
        }
        .expect("remote insert prepare should succeed");
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

        Spi::connect_mut(|client| {
            client
                .update(
                    "INSERT INTO ec_spire_placement \
                         (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
                     VALUES ($1::oid, $2::bytea, $3::integer, $4::bigint, $5::bigint, $6::bytea)",
                    None,
                    &[
                        index_oid.into(),
                        pk_value.into(),
                        node_id.into(),
                        centroid_id.into(),
                        served_epoch.into(),
                        source_identity.into(),
                    ],
                )
                .expect("placement insert should succeed");
        });

        TestCoordinatorInsertPrepareResult {
            prepared_gid: row.prepared_gid,
            status: row.status,
            next_step: row.next_step,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn test_prepare_coordinator_insert_remote_tuple_payload(
        index_oid: pg_sys::Oid,
        pk_value: Vec<u8>,
        node_id: i32,
        centroid_id: i64,
        served_epoch: i64,
        source_identity: Vec<u8>,
        row_payload_json: &str,
        requested_columns: Vec<String>,
    ) -> TestCoordinatorInsertPrepareResult {
        let node_id_u32 = u32::try_from(node_id).expect("node_id should fit u32");
        let served_epoch_u64 = u64::try_from(served_epoch).expect("served_epoch should fit u64");
        let index_relation = unsafe {
            open_valid_ec_spire_index(
                index_oid,
                "test_prepare_coordinator_insert_remote_tuple_payload",
            )
        };
        let row = unsafe {
            am::spire_coordinator_insert_prepare_remote_tuple_payload(
                index_relation,
                node_id_u32,
                served_epoch_u64,
                row_payload_json,
                &requested_columns,
            )
        }
        .expect("remote tuple payload insert prepare should succeed");
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

        Spi::connect_mut(|client| {
            client
                .update(
                    "INSERT INTO ec_spire_placement \
                         (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity) \
                     VALUES ($1::oid, $2::bytea, $3::integer, $4::bigint, $5::bigint, $6::bytea)",
                    None,
                    &[
                        index_oid.into(),
                        pk_value.into(),
                        node_id.into(),
                        centroid_id.into(),
                        served_epoch.into(),
                        source_identity.into(),
                    ],
                )
                .expect("placement insert should succeed");
        });

        TestCoordinatorInsertPrepareResult {
            prepared_gid: row.prepared_gid,
            status: row.status,
            next_step: row.next_step,
        }
    }

    fn loopback_remote_index_identity_bytes(
        client: &mut postgres::Client,
        remote_index_regclass: &str,
    ) -> Vec<u8> {
        client
            .query_one(
                "SELECT decode(profile_fingerprint, 'hex') \
                   FROM ec_spire_remote_search_endpoint_identity(to_regclass($1)::oid)",
                &[&remote_index_regclass],
            )
            .expect("remote endpoint identity query should succeed")
            .try_get::<_, Vec<u8>>(0)
            .expect("remote endpoint identity should decode")
    }

    fn assert_governance_lock_released(conninfo: &str, class_id: i32, object_id: i32, label: &str) {
        let mut lock_probe = postgres::Client::connect(conninfo, postgres::NoTls)
            .expect("governance lock probe connection should succeed");
        let acquired = lock_probe
            .query_one(
                "SELECT pg_try_advisory_lock($1::integer, $2::integer)",
                &[&class_id, &object_id],
            )
            .expect("governance lock probe should succeed")
            .try_get::<_, bool>(0)
            .expect("governance lock probe should decode");
        assert!(acquired, "{label} governance lock should be released");
        let unlocked = lock_probe
            .query_one(
                "SELECT pg_advisory_unlock($1::integer, $2::integer)",
                &[&class_id, &object_id],
            )
            .expect("governance lock unlock should succeed")
            .try_get::<_, bool>(0)
            .expect("governance lock unlock should decode");
        assert!(unlocked, "{label} governance lock should unlock");
    }

    fn set_remote_governance_test_namespace(namespace: i32) {
        Spi::run(&format!(
            "SET LOCAL ec_spire.remote_search_governance_test_namespace = {namespace}"
        ))
        .expect("governance test namespace SET should succeed");
    }

    fn loopback_remote_index_identity_hex(
        client: &mut postgres::Client,
        remote_index_regclass: &str,
    ) -> String {
        let identity = loopback_remote_index_identity_bytes(client, remote_index_regclass);
        identity
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    }

    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::collections::{HashMap, HashSet};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Output, Stdio};
    use std::time::{Duration, Instant};

    const RECALL_BITS: i32 = 4;
    const RECALL_SEED: i64 = 42;
    const RECALL_DIM: usize = 1536;
    const RECALL_CORPUS_SIZE: usize = 10_000;
    const RECALL_QUERY_COUNT: usize = 100;
    const RECALL_K: usize = 10;
    const RECALL_EF_CONSTRUCTION: i32 = 128;
    const RECALL_INSERT_BATCH_SIZE: usize = 32;
    const PQ_FASTSCAN_BINARY_RUNTIME_WORD_COUNT: i32 = ((RECALL_DIM as i32) + 63) / 64;
    const SCORE_ASSERT_EPSILON: f32 = 1e-5;
    const RECALL_GATE_CONFIGS: [(i32, i32, Option<f32>); 4] = [
        (8, 40, None),
        (8, 128, Some(0.89_f32)),
        (8, 200, None),
        (16, 200, None),
    ];

    // External oracle from the Qdrant `vector-db-benchmark` published results,
    // setup `qdrant-m-16-ef-128` (m=16, ef_construct=128, hnsw_ef=128) on
    // `dbpedia-openai-1M-1536-angular`. See `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md`.
    // Source: https://qdrant.tech/benchmarks/results-1-100-thread-2024-06-15.json
    const ANN_BENCHMARKS_ANCHOR_PUBLISHED_RECALL_AT_10: f32 = 0.96082_f32;
    const ANN_BENCHMARKS_ANCHOR_TOLERANCE: f32 = 0.02_f32;

    fn setup_rescan_scaffold_index(name: &str) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE TABLE {name} (id bigint primary key, embedding ecvector)"
        ))
        .expect("table creation should succeed");
        Spi::run(&format!(
            "INSERT INTO {name} VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))"
        ))
        .expect("seed insert should succeed");
        Spi::run(&format!(
            "CREATE INDEX {name}_idx ON {name} USING ec_hnsw (embedding ecvector_ip_ops)"
        ))
        .expect("index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{name}_idx'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist")
    }

    fn random_unit_vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut vectors = Vec::with_capacity(n);

        for _ in 0..n {
            let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0f32..1.0f32)).collect();
            let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
            for value in &mut values {
                *value /= norm.max(f32::EPSILON);
            }
            vectors.push(values);
        }

        vectors
    }

    fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    fn assert_f32_close(observed: f32, expected: f32, label: &str) {
        assert!(
            (observed - expected).abs() <= SCORE_ASSERT_EPSILON,
            "{label}: observed {observed}, expected {expected}",
        );
    }

    fn brute_force_top_k(corpus: &[Vec<f32>], query: &[f32], k: usize) -> Vec<usize> {
        let mut scores: Vec<(usize, f32)> = corpus
            .iter()
            .enumerate()
            .map(|(i, vector)| (i, dot_product(query, vector)))
            .collect();
        scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        scores.truncate(k);
        scores.into_iter().map(|(i, _)| i).collect()
    }

    fn encoded_code_bytes(encoded: crate::bench_api::EncodedTq) -> Vec<u8> {
        let mut code_bytes = encoded.mse_packed;
        code_bytes.extend_from_slice(&encoded.qjl_packed);
        code_bytes
    }

    fn parse_ctid(ctid: &str) -> am::page::ItemPointer {
        let trimmed = ctid.trim();
        let inner = trimmed
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .expect("ctid should use (block,offset) formatting");
        let (block_number, offset_number) = inner
            .split_once(',')
            .expect("ctid should contain block and offset");
        am::page::ItemPointer {
            block_number: block_number
                .trim()
                .parse()
                .expect("ctid block number should parse"),
            offset_number: offset_number
                .trim()
                .parse()
                .expect("ctid offset number should parse"),
        }
    }

    fn heap_tid_for_row(table_name: &str, id: i64) -> am::page::ItemPointer {
        let ctid = Spi::get_one::<String>(&format!(
            "SELECT ctid::text FROM {table_name} WHERE id = {id}"
        ))
        .expect("SPI query should succeed")
        .expect("table row should exist");
        parse_ctid(&ctid)
    }

    type ScalarElementsAndNeighbors = (
        am::page::MetadataPage,
        Vec<(am::page::ItemPointer, am::page::TqElementTuple)>,
        HashMap<am::page::ItemPointer, am::page::TqNeighborTuple>,
    );

    type GroupedElementsAndNeighbors = (
        am::page::MetadataPage,
        am::graph::PqFastScanLayout,
        Vec<(am::page::ItemPointer, am::page::TqGroupedHotTuple)>,
        HashMap<am::page::ItemPointer, am::page::TqNeighborTuple>,
    );

    fn is_turboquant_element_tag(tag: Option<u8>) -> bool {
        matches!(
            tag,
            Some(am::page::TQ_ELEMENT_TAG) | Some(am::page::TQ_TURBO_HOT_TAG)
        )
    }

    fn turboquant_v3_binary_word_count(dim: usize, bits: u8) -> usize {
        if bits == 4 && crate::quant::rotation::tile_dim(dim).is_some() {
            dim.div_ceil(64)
        } else {
            0
        }
    }

    fn turboquant_v3_triplet_storage_bytes(
        level: u8,
        m: u16,
        code_len: usize,
        binary_word_count: usize,
    ) -> usize {
        am::page::raw_tuple_storage_bytes(am::page::neighbor_tuple_encoded_len(level, m))
            + am::page::raw_tuple_storage_bytes(am::page::TqRerankTuple::encoded_len(code_len))
            + am::page::raw_tuple_storage_bytes(am::page::TqTurboHotTuple::encoded_len(
                binary_word_count,
            ))
    }

    fn decode_turboquant_elements_from_pages(
        metadata: &am::page::MetadataPage,
        data_pages: &[am::DebugIndexDataPage],
        code_len: usize,
    ) -> Vec<(am::page::ItemPointer, am::page::TqElementTuple)> {
        let turbo_layout = match am::graph::GraphStorageDescriptor::from_metadata(metadata)
            .expect("metadata should decode into a graph storage descriptor")
        {
            am::graph::GraphStorageDescriptor::TurboQuant { .. } => None,
            am::graph::GraphStorageDescriptor::TurboQuantHotCold(layout) => Some(layout),
            am::graph::GraphStorageDescriptor::PqFastScan(_) => {
                panic!("turboquant decode helper requires a turboquant index")
            }
        };
        let mut elements = Vec::new();
        let mut hot_elements = Vec::new();
        let mut rerank_payloads = HashMap::new();

        for page in data_pages {
            for (idx, tuple) in page.tuples.iter().enumerate() {
                let tid = am::page::ItemPointer {
                    block_number: page.block_number,
                    offset_number: u16::try_from(idx + 1)
                        .expect("page tuple offset should fit in u16"),
                };
                match tuple.first().copied() {
                    Some(am::page::TQ_ELEMENT_TAG) => elements.push((
                        tid,
                        am::page::TqElementTuple::decode(tuple, code_len)
                            .expect("element tuple should decode"),
                    )),
                    Some(am::page::TQ_TURBO_HOT_TAG) => {
                        let layout = turbo_layout
                            .expect("turbo hot tuple should only appear in V3 turboquant pages");
                        hot_elements.push((
                            tid,
                            am::page::TqTurboHotTuple::decode(tuple, layout.binary_word_count)
                                .expect("turbo hot tuple should decode"),
                        ));
                    }
                    Some(am::page::TQ_RERANK_TAG) => {
                        rerank_payloads.insert(
                            tid,
                            am::page::TqRerankTuple::decode(tuple, code_len)
                                .expect("rerank tuple should decode"),
                        );
                    }
                    _ => {}
                }
            }
        }

        elements.extend(hot_elements.into_iter().map(|(tid, hot)| {
            let rerank = rerank_payloads.remove(&hot.reranktid).unwrap_or_else(|| {
                panic!(
                    "turbo hot tuple {}:{} should reference a decodable rerank payload",
                    hot.reranktid.block_number, hot.reranktid.offset_number
                )
            });
            (
                tid,
                am::page::TqElementTuple {
                    level: hot.level,
                    deleted: hot.deleted,
                    heaptids: hot.heaptids,
                    gamma: rerank.gamma,
                    neighbortid: hot.neighbortid,
                    code: rerank.code,
                    binary_words: hot.binary_words,
                },
            )
        }));
        elements
    }

    fn decode_index_elements_and_neighbors(
        index_oid: pg_sys::Oid,
        code_len: usize,
    ) -> ScalarElementsAndNeighbors {
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let mut neighbors = HashMap::new();

        for page in &data_pages {
            for (idx, tuple) in page.tuples.iter().enumerate() {
                let tid = am::page::ItemPointer {
                    block_number: page.block_number,
                    offset_number: u16::try_from(idx + 1)
                        .expect("page tuple offset should fit in u16"),
                };
                if let Some(am::page::TQ_NEIGHBOR_TAG) = tuple.first().copied() {
                    neighbors.insert(
                        tid,
                        am::page::TqNeighborTuple::decode(tuple)
                            .expect("neighbor tuple should decode"),
                    );
                }
            }
        }

        let elements = decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len);
        (metadata, elements, neighbors)
    }

    fn decode_grouped_index_elements_and_neighbors(
        index_oid: pg_sys::Oid,
    ) -> GroupedElementsAndNeighbors {
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let layout = match am::graph::GraphStorageDescriptor::from_metadata(&metadata).unwrap() {
            am::graph::GraphStorageDescriptor::PqFastScan(layout) => layout,
            am::graph::GraphStorageDescriptor::TurboQuant { .. }
            | am::graph::GraphStorageDescriptor::TurboQuantHotCold(_) => {
                panic!("grouped decode helper requires a PqFastScan index")
            }
        };
        let mut elements = Vec::new();
        let mut neighbors = HashMap::new();

        for page in data_pages {
            for (idx, tuple) in page.tuples.iter().enumerate() {
                let tid = am::page::ItemPointer {
                    block_number: page.block_number,
                    offset_number: u16::try_from(idx + 1)
                        .expect("page tuple offset should fit in u16"),
                };
                match tuple.first().copied() {
                    Some(am::page::TQ_GROUPED_HOT_TAG) => {
                        elements.push((
                            tid,
                            am::page::TqGroupedHotTuple::decode(
                                tuple,
                                layout.binary_word_count,
                                layout.search_code_len,
                            )
                            .expect("grouped hot tuple should decode"),
                        ));
                    }
                    Some(am::page::TQ_NEIGHBOR_TAG) => {
                        neighbors.insert(
                            tid,
                            am::page::TqNeighborTuple::decode(tuple)
                                .expect("neighbor tuple should decode"),
                        );
                    }
                    _ => {}
                }
            }
        }

        (metadata, layout, elements, neighbors)
    }

    fn find_element_for_heap_tid(
        elements: &[(am::page::ItemPointer, am::page::TqElementTuple)],
        heap_tid: am::page::ItemPointer,
    ) -> (am::page::ItemPointer, &am::page::TqElementTuple) {
        let (element_tid, element) = elements
            .iter()
            .find(|(_, element)| element.heaptids.contains(&heap_tid))
            .expect("element should be discoverable by heap tid");
        (*element_tid, element)
    }

    fn find_grouped_element_for_heap_tid(
        elements: &[(am::page::ItemPointer, am::page::TqGroupedHotTuple)],
        heap_tid: am::page::ItemPointer,
    ) -> (am::page::ItemPointer, &am::page::TqGroupedHotTuple) {
        let (element_tid, element) = elements
            .iter()
            .find(|(_, element)| element.heaptids.contains(&heap_tid))
            .expect("grouped element should be discoverable by heap tid");
        (*element_tid, element)
    }

    fn layer_neighbor_slice(
        neighbor_tids: &[am::page::ItemPointer],
        m: usize,
        layer: u8,
    ) -> &[am::page::ItemPointer] {
        let (start, end) = if layer == 0 {
            (0, (m * 2).min(neighbor_tids.len()))
        } else {
            let start = (m * 2) + (usize::from(layer) - 1) * m;
            if start >= neighbor_tids.len() {
                return &neighbor_tids[0..0];
            }
            (start, (start + m).min(neighbor_tids.len()))
        };
        &neighbor_tids[start..end]
    }

    fn count_neighbor_refs(
        neighbors: &HashMap<am::page::ItemPointer, am::page::TqNeighborTuple>,
        target_tid: am::page::ItemPointer,
    ) -> usize {
        neighbors
            .values()
            .map(|neighbor| {
                neighbor
                    .tids
                    .iter()
                    .filter(|tid| **tid == target_tid)
                    .count()
            })
            .sum()
    }

    fn encode_recall_query_code(query: &[f32]) -> Vec<u8> {
        let quantizer = ProdQuantizer::cached(
            RECALL_DIM,
            u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
            RECALL_SEED as u64,
        );
        encoded_code_bytes(quantizer.encode(query))
    }

    fn encode_recall_corpus_codes(corpus: &[Vec<f32>]) -> Vec<Vec<u8>> {
        let quantizer = ProdQuantizer::cached(
            RECALL_DIM,
            u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
            RECALL_SEED as u64,
        );
        corpus
            .iter()
            .map(|vector| encoded_code_bytes(quantizer.encode(vector)))
            .collect()
    }

    fn brute_force_top_k_code_inner_product(
        corpus_codes: &[Vec<u8>],
        query_code: &[u8],
        k: usize,
    ) -> Vec<usize> {
        let mut scores = corpus_codes
            .iter()
            .enumerate()
            .map(|(i, code)| {
                (
                    i,
                    score_code_inner_product(
                        RECALL_DIM,
                        u8::try_from(RECALL_BITS).expect("recall bits should fit into u8"),
                        RECALL_SEED as u64,
                        query_code,
                        code,
                    ),
                )
            })
            .collect::<Vec<_>>();
        scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        scores.truncate(k);
        scores.into_iter().map(|(i, _)| i).collect()
    }

    fn create_recall_table(table_name: &str) {
        Spi::run(&format!(
            "CREATE TABLE {table_name} (id bigint primary key, embedding ecvector)"
        ))
        .expect("recall benchmark table creation should succeed");
    }

    fn create_recall_table_with_source(table_name: &str) {
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key,
                source real[] NOT NULL,
                embedding ecvector
            )"
        ))
        .expect("recall benchmark source table creation should succeed");
    }

    fn insert_recall_corpus(table_name: &str, corpus: &[Vec<f32>]) {
        for batch in corpus.chunks(RECALL_INSERT_BATCH_SIZE).enumerate() {
            let (batch_index, embeddings) = batch;
            let batch_offset = batch_index * RECALL_INSERT_BATCH_SIZE;
            let values_sql = embeddings
                .iter()
                .enumerate()
                .map(|(batch_row, embedding)| {
                    format!(
                        "({}, encode_to_ecvector({}, {RECALL_BITS}, {RECALL_SEED}))",
                        batch_offset + batch_row,
                        format_recall_vector_sql_literal(embedding),
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO {table_name} (id, embedding) VALUES {values_sql}"
            ))
            .expect("recall benchmark batch insert should succeed");
        }
    }

    fn insert_recall_corpus_with_source(table_name: &str, corpus: &[Vec<f32>]) {
        for batch in corpus.chunks(RECALL_INSERT_BATCH_SIZE).enumerate() {
            let (batch_index, embeddings) = batch;
            let batch_offset = batch_index * RECALL_INSERT_BATCH_SIZE;
            let values_sql = embeddings
                .iter()
                .enumerate()
                .map(|(batch_row, embedding)| {
                    let source = format_recall_vector_sql_literal(embedding);
                    format!(
                        "({}, {}, encode_to_ecvector({}, {RECALL_BITS}, {RECALL_SEED}))",
                        batch_offset + batch_row,
                        source,
                        source,
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO {table_name} (id, source, embedding) VALUES {values_sql}"
            ))
            .expect("recall benchmark source batch insert should succeed");
        }
    }

    fn format_recall_vector_sql_literal(embedding: &[f32]) -> String {
        format!(
            "ARRAY[{}]::real[]",
            embedding
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn ctid_id_map(table_name: &str) -> HashMap<(u32, u16), usize> {
        Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT
                            split_part(trim(both '()' from ctid::text), ',', 1)::int4 AS block_number,
                            split_part(trim(both '()' from ctid::text), ',', 2)::int2 AS offset_number,
                            id
                         FROM {table_name}"
                    ),
                    None,
                    &[],
                )
                .expect("ctid/id map query should succeed")
                .map(|row| {
                    let block_number = row["block_number"]
                        .value::<i32>()
                        .expect("block number should decode")
                        .expect("block number should be non-null");
                    let offset_number = row["offset_number"]
                        .value::<i16>()
                        .expect("offset number should decode")
                        .expect("offset number should be non-null");
                    let id = row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null");
                    (
                        (
                            u32::try_from(block_number)
                                .expect("block number should be non-negative"),
                            u16::try_from(offset_number)
                                .expect("offset number should be positive"),
                        ),
                        usize::try_from(id).expect("id should fit into usize"),
                    )
                })
                .collect::<HashMap<_, _>>()
        })
    }

    fn exact_ecvector_top_k_ids(table_name: &str, query: &[f32], k: usize) -> Vec<usize> {
        let query_literal = format_recall_vector_sql_literal(query);
        Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT id
                         FROM {table_name}
                         ORDER BY ecvector_negative_query_inner_product(embedding, {query_literal}), id
                         LIMIT {k}"
                    ),
                    None,
                    &[],
                )
                .expect("exact ecvector top-k query should succeed")
                .map(|row| {
                    let id = row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null");
                    usize::try_from(id).expect("id should fit into usize")
                })
                .collect()
        })
    }

    fn ivf_debug_output_ids(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        ctid_to_id: &HashMap<(u32, u16), usize>,
        k: usize,
    ) -> Vec<usize> {
        let (outputs, _orderby_cleared) =
            unsafe { am::debug_ec_ivf_gettuple_outputs(index_oid, query) };
        outputs
            .into_iter()
            .take(k)
            .map(|(block_number, offset_number, _score)| {
                *ctid_to_id
                    .get(&(block_number, offset_number))
                    .expect("IVF emitted heap tid should map back to a row id")
            })
            .collect()
    }

    fn hnsw_debug_output_ids(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        ctid_to_id: &HashMap<(u32, u16), usize>,
        k: usize,
    ) -> Vec<usize> {
        unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query) }
            .into_iter()
            .take(k)
            .map(|heap_tid| {
                *ctid_to_id
                    .get(&heap_tid)
                    .expect("HNSW emitted heap tid should map back to a row id")
            })
            .collect()
    }

    fn create_ivf_recall_index(
        table_name: &str,
        index_name: &str,
        nlists: i32,
        nprobe: i32,
        training_sample_rows: i32,
    ) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_ivf \
             (embedding ecvector_ip_ops) \
             WITH (nlists = {nlists}, nprobe = {nprobe}, training_sample_rows = {training_sample_rows})"
        ))
        .expect("IVF recall index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("IVF recall index oid query should succeed")
            .expect("IVF recall index oid should exist")
    }

    fn create_recall_index(table_name: &str, index_name: &str, m: i32) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = {m}, ef_construction = {RECALL_EF_CONSTRUCTION})"
        ))
        .expect("recall benchmark index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("recall benchmark index oid query should succeed")
            .expect("recall benchmark index oid should exist")
    }

    fn create_recall_index_with_source_build(
        table_name: &str,
        index_name: &str,
        m: i32,
    ) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (
                 m = {m},
                 ef_construction = {RECALL_EF_CONSTRUCTION},
                 build_source_column = 'source'
             )"
        ))
        .expect("recall benchmark source-build index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("recall benchmark source-build index oid query should succeed")
            .expect("recall benchmark source-build index oid should exist")
    }

    fn recall_index_block_count(index_oid: pg_sys::Oid, caller_name: &'static str) -> i32 {
        let index_relation = unsafe { open_valid_ec_hnsw_index(index_oid, caller_name) };
        let index_block_count = unsafe {
            i32::try_from(pg_sys::RelationGetNumberOfBlocksInFork(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
            ))
            .expect("block count should fit into int")
        };
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        index_block_count
    }

    fn recall_fixture_ident(label: &str) -> String {
        assert!(!label.is_empty(), "recall fixture names must not be empty");
        assert!(
            label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
            "recall fixture names must be ASCII alphanumeric or underscore only"
        );
        label.to_owned()
    }

    fn reset_graph_scan_recall_fixture(fixture_name: &str, m: i32, corpus_size: usize) -> i32 {
        assert!(corpus_size >= RECALL_K);

        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);

        Spi::run(&format!("DROP TABLE IF EXISTS {fixture_name} CASCADE"))
            .expect("recall fixture cleanup should succeed");
        create_recall_table(&fixture_name);
        insert_recall_corpus(&fixture_name, &corpus);
        let index_oid = create_recall_index(&fixture_name, &index_name, m);
        recall_index_block_count(index_oid, "reset_graph_scan_recall_fixture")
    }

    fn gate_fixture_already_exists(
        table_name: &str,
        fixture_prefix: &str,
        corpus_size: usize,
    ) -> Option<Vec<(i32, i32)>> {
        let table_exists = Spi::get_one::<bool>(&format!(
            "SELECT EXISTS (
                 SELECT 1
                 FROM pg_class
                 WHERE relname = '{table_name}'
                   AND relkind = 'r'
             )"
        ))
        .expect("table existence check should succeed")
        .unwrap_or(false);
        if !table_exists {
            return None;
        }

        let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) FROM {table_name}"))
            .expect("row count query should succeed")
            .unwrap_or(0);
        if row_count != i64::try_from(corpus_size).expect("corpus size should fit into i64") {
            return None;
        }

        let mut results = Vec::new();
        for m in [8, 16] {
            let index_name = format!("{fixture_prefix}_m{m}_idx");
            let expected_m = format!("m={m}");
            let expected_ef = format!("ef_construction={RECALL_EF_CONSTRUCTION}");
            let index_ok = Spi::get_one::<bool>(&format!(
                "SELECT EXISTS (
                     SELECT 1
                     FROM pg_class
                     WHERE relname = '{index_name}'
                       AND relkind = 'i'
                       AND reloptions @> ARRAY['{expected_m}', '{expected_ef}']
                 )"
            ))
            .expect("index existence check should succeed")
            .unwrap_or(false);
            if !index_ok {
                return None;
            }

            let index_oid =
                Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                    .expect("index oid query should succeed")
                    .expect("index oid should exist");
            let block_count = recall_index_block_count(index_oid, "gate_fixture_already_exists");
            results.push((m, block_count));
        }

        Some(results)
    }

    fn reset_graph_scan_recall_gate_fixtures(
        fixture_prefix: &str,
        corpus_size: usize,
    ) -> Vec<(i32, i32)> {
        let fixture_prefix = recall_fixture_ident(fixture_prefix);
        let table_name = format!("{fixture_prefix}_corpus");

        if let Some(existing) =
            gate_fixture_already_exists(&table_name, &fixture_prefix, corpus_size)
        {
            pgrx::log!("fixture already exists, skipping rebuild: {table_name}");
            return existing;
        }

        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);

        Spi::run(&format!("DROP TABLE IF EXISTS {table_name} CASCADE"))
            .expect("recall gate fixture cleanup should succeed");
        create_recall_table(&table_name);
        insert_recall_corpus(&table_name, &corpus);

        [8, 16]
            .into_iter()
            .map(|m| {
                let index_name = format!("{fixture_prefix}_m{m}_idx");
                let index_oid = create_recall_index(&table_name, &index_name, m);
                let index_block_count =
                    recall_index_block_count(index_oid, "reset_graph_scan_recall_gate_fixtures");
                (m, index_block_count)
            })
            .collect()
    }

    fn reset_graph_scan_recall_gate_source_fixtures(
        fixture_prefix: &str,
        corpus_size: usize,
    ) -> Vec<(i32, i32)> {
        let fixture_prefix = recall_fixture_ident(fixture_prefix);
        let table_name = format!("{fixture_prefix}_corpus");

        if let Some(existing) =
            gate_fixture_already_exists(&table_name, &fixture_prefix, corpus_size)
        {
            pgrx::log!("fixture already exists, skipping rebuild: {table_name}");
            return existing;
        }

        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);

        Spi::run(&format!("DROP TABLE IF EXISTS {table_name} CASCADE"))
            .expect("recall gate source fixture cleanup should succeed");
        create_recall_table_with_source(&table_name);
        insert_recall_corpus_with_source(&table_name, &corpus);

        [8, 16]
            .into_iter()
            .map(|m| {
                let index_name = format!("{fixture_prefix}_m{m}_idx");
                let index_oid = create_recall_index_with_source_build(&table_name, &index_name, m);
                let index_block_count = recall_index_block_count(
                    index_oid,
                    "reset_graph_scan_recall_gate_source_fixtures",
                );
                (m, index_block_count)
            })
            .collect()
    }

    fn measure_graph_scan_recall(
        index_oid: pg_sys::Oid,
        ctid_to_id: &HashMap<(u32, u16), usize>,
        queries: &[Vec<f32>],
        ground_truth: &[Vec<usize>],
        ef_search: i32,
    ) -> f32 {
        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let hits = queries
            .iter()
            .zip(ground_truth.iter())
            .map(|(query, true_top_k)| {
                let predicted =
                    unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) };
                let predicted_top_k: HashSet<usize> = predicted
                    .iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        *ctid_to_id
                            .get(heap_tid)
                            .expect("emitted heap tid should map back to a benchmark row id")
                    })
                    .collect();

                true_top_k
                    .iter()
                    .filter(|id| predicted_top_k.contains(id))
                    .count()
            })
            .sum::<usize>();

        hits as f32 / (queries.len() * RECALL_K) as f32
    }

    include!("type_registration.rs");




    include!("cost_and_planner.rs");
    include!("spire_cost_tuning.rs");

    include!("placement.rs");

    include!("scan.rs");
    include!("spire_recall.rs");
    include!("insert.rs");
    include!("insert_remote_trigger.rs");

    include!("diagnostics.rs");
    include!("diagnostics_reindex.rs");

    include!("build.rs");


    include!("vacuum.rs");





    include!("remote_search/mod.rs");



    include!("dml_frontdoor.rs");

    include!("dml_schema_drift.rs");

    include!("dml_frontdoor_select.rs");

    include!("dml_frontdoor_delete.rs");

    include!("dml_concurrency.rs");

    include!("data_shape.rs");

    include!("custom_scan_execution.rs");

    include!("custom_scan_schema_drift.rs");

    include!("custom_scan_tuple_transport.rs");

    include!("custom_scan_timeout.rs");

    include!("custom_scan_concurrency.rs");

    include!("custom_scan_lifecycle.rs");




    unsafe fn analyzed_query(sql: &str) -> *mut pg_sys::Query {
        let sql = CString::new(sql).expect("test SQL should not contain NUL");
        let raw_parses = unsafe { pg_sys::pg_parse_query(sql.as_ptr()) };
        assert!(
            !raw_parses.is_null(),
            "parse should return a raw statement list"
        );
        let raw_stmt = unsafe { pg_sys::list_nth(raw_parses, 0) }.cast::<pg_sys::RawStmt>();
        let queries = unsafe {
            pg_sys::pg_analyze_and_rewrite_fixedparams(
                raw_stmt,
                sql.as_ptr(),
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
            )
        };
        assert!(!queries.is_null(), "analyze should return a query list");
        unsafe { pg_sys::list_nth(queries, 0) }.cast::<pg_sys::Query>()
    }

    include!("custom_scan.rs");
    include!("custom_scan_planner.rs");
    include!("custom_scan_fanout.rs");



    fn index_oid(index_name: &str) -> pg_sys::Oid {
        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist")
    }

    fn ec_spire_active_snapshot_i64(index_name: &str, column_name: &str) -> i64 {
        Spi::get_one::<i64>(&format!(
            "SELECT {column_name} FROM \
             ec_spire_index_active_snapshot_diagnostics('{index_name}'::regclass)"
        ))
        .expect("SPI query should succeed")
        .expect("diagnostics row should exist")
    }

    fn ec_ivf_index_oid(index_name: &str) -> pg_sys::Oid {
        index_oid(index_name)
    }

    fn ec_ivf_index_blocks(index_name: &str) -> i64 {
        Spi::get_one::<i64>(&format!(
            "SELECT (pg_relation_size('{index_name}') \
             / current_setting('block_size')::int)::bigint"
        ))
        .expect("SPI query should succeed")
        .expect("relation size should exist")
    }

    include!("psql_helpers.rs");

    include!("ec_ivf.rs");

    include!("ec_hnsw_build.rs");
    include!("ec_hnsw_runtime_profiles.rs");
    include!("ec_hnsw_runtime_comparisons.rs");
    include!("ec_hnsw_storage_lifecycle.rs");
    include!("ec_hnsw_graph_lifecycle.rs");
    include!("ec_hnsw_scan_gettuple.rs");

    include!("ec_hnsw_recall_helpers.rs");
    include!("ec_hnsw_recall_debug_exports.rs");
    include!("ec_hnsw_recall_tests.rs");
    include!("hnsw_misc.rs");
