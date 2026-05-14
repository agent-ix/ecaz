    #[pg_extern]
    fn ec_hnsw_debug_pack_f32_bytea(values: Vec<f32>) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(values.len() * std::mem::size_of::<f32>());
        for value in values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    #[pg_extern]
    fn ec_hnsw_debug_parallel_build_workers_launched() -> i32 {
        am::debug_last_parallel_build_workers_launched()
    }

    #[pg_extern]
    fn ec_hnsw_debug_parallel_graph_build_workers_launched() -> i32 {
        am::debug_last_parallel_graph_build_workers_launched()
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_last_build_timing() -> TableIterator<
        'static,
        (
            name!(requested_workers, i64),
            name!(workers_launched, i64),
            name!(heap_workers_launched, i64),
            name!(graph_workers_launched, i64),
            name!(heap_tuples, i64),
            name!(index_tuples, i64),
            name!(heap_ingest_us, i64),
            name!(parallel_begin_us, i64),
            name!(parallel_drain_us, i64),
            name!(parallel_sort_push_us, i64),
            name!(flush_total_us, i64),
            name!(graph_us, i64),
            name!(stage_us, i64),
            name!(write_us, i64),
        ),
    > {
        let timing = am::debug_last_build_timing();
        TableIterator::once((
            timing.requested_workers as i64,
            timing.workers_launched as i64,
            timing.heap_workers_launched as i64,
            timing.graph_workers_launched as i64,
            timing.heap_tuples as i64,
            timing.index_tuples as i64,
            timing.heap_ingest_us as i64,
            timing.parallel_begin_us as i64,
            timing.parallel_drain_us as i64,
            timing.parallel_sort_push_us as i64,
            timing.flush_total_us as i64,
            timing.graph_us as i64,
            timing.stage_us as i64,
            timing.write_us as i64,
        ))
    }

    fn create_pq_fastscan_runtime_fixture_internal(
        table_name: &str,
        index_name: &str,
        include_source_raw: bool,
        m: i32,
        persisted_rerank_source_column: Option<&str>,
    ) -> pg_sys::Oid {
        let source_raw_column = if include_source_raw {
            ",\n                source_raw bytea"
        } else {
            ""
        };
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key,
                source real[]{source_raw_column},
                embedding ecvector
            )"
        ))
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 41 + dim) as f32) * 0.03).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let insert_sql = if include_source_raw {
                format!(
                    "INSERT INTO {table_name} VALUES \
                     ({id}, ARRAY[{source}]::real[], tests.ec_hnsw_debug_pack_f32_bytea(ARRAY[{source}]::real[]), \
                     encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
                )
            } else {
                format!(
                    "INSERT INTO {table_name} VALUES \
                     ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
                )
            };
            Spi::run(&insert_sql).expect("insert should succeed");
        }

        let rerank_source_reloption = persisted_rerank_source_column
            .map(|column| format!(", rerank_source_column = '{column}'"))
            .unwrap_or_default();
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = {m}, ef_construction = 80, build_source_column = 'source'{rerank_source_reloption}, storage_format = 'pq_fastscan')"
        ))
        .expect("index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist")
    }

    fn create_pq_fastscan_runtime_fixture(table_name: &str, index_name: &str) -> pg_sys::Oid {
        create_pq_fastscan_runtime_fixture_internal(table_name, index_name, false, 6, None)
    }

    fn create_pq_fastscan_runtime_fixture_with_source_raw(
        table_name: &str,
        index_name: &str,
    ) -> pg_sys::Oid {
        create_pq_fastscan_runtime_fixture_internal(table_name, index_name, true, 6, None)
    }

    fn create_pq_fastscan_runtime_fixture_with_persisted_source_raw(
        table_name: &str,
        index_name: &str,
    ) -> pg_sys::Oid {
        create_pq_fastscan_runtime_fixture_internal(
            table_name,
            index_name,
            true,
            6,
            Some("source_raw"),
        )
    }

    fn create_pq_fastscan_runtime_fixture_with_m(
        table_name: &str,
        index_name: &str,
        m: i32,
    ) -> pg_sys::Oid {
        create_pq_fastscan_runtime_fixture_internal(table_name, index_name, false, m, None)
    }

    fn create_pq_fastscan_binary_runtime_fixture(
        table_name: &str,
        index_name: &str,
    ) -> pg_sys::Oid {
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key,
                source real[],
                embedding ecvector
            )"
        ))
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = format_recall_vector_sql_literal(&pq_fastscan_binary_runtime_source(id));
            let embedding =
                format_recall_vector_sql_literal(&pq_fastscan_binary_runtime_embedding(id));
            Spi::run(&format!(
                "INSERT INTO {table_name} VALUES \
                 ({id}, {source}, encode_to_ecvector({embedding}, 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')"
        ))
        .expect("index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist")
    }

    fn create_turboquant_runtime_fixture_internal(
        table_name: &str,
        index_name: &str,
        include_source: bool,
    ) -> pg_sys::Oid {
        let source_column = if include_source {
            ",\n                source real[]"
        } else {
            ""
        };
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key{source_column},
                embedding ecvector
            )"
        ))
        .expect("table creation should succeed");

        for id in 1..=16 {
            let embedding = format_recall_vector_sql_literal(&runtime_fixture_embedding(id));
            let insert_sql = if include_source {
                let source = format_recall_vector_sql_literal(&pq_fastscan_runtime_source(id));
                format!(
                    "INSERT INTO {table_name} VALUES \
                     ({id}, {source}, encode_to_ecvector({embedding}, 4, 42))"
                )
            } else {
                format!(
                    "INSERT INTO {table_name} VALUES \
                     ({id}, encode_to_ecvector({embedding}, 4, 42))"
                )
            };
            Spi::run(&insert_sql).expect("insert should succeed");
        }

        let build_source_column = if include_source {
            ", build_source_column = 'source'"
        } else {
            ""
        };
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80{build_source_column}, storage_format = 'turboquant')"
        ))
        .expect("index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist")
    }

    fn create_turboquant_runtime_fixture(table_name: &str, index_name: &str) -> pg_sys::Oid {
        create_turboquant_runtime_fixture_internal(table_name, index_name, false)
    }

    fn create_turboquant_runtime_fixture_with_source(
        table_name: &str,
        index_name: &str,
    ) -> pg_sys::Oid {
        create_turboquant_runtime_fixture_internal(table_name, index_name, true)
    }

    fn create_turboquant_binary_runtime_fixture_internal(
        table_name: &str,
        index_name: &str,
        include_source: bool,
        persisted_rerank_source_column: Option<&str>,
    ) -> pg_sys::Oid {
        let include_source_raw = persisted_rerank_source_column.is_some();
        let source_column = match (include_source, include_source_raw) {
            (false, false) => "",
            (true, false) => ",\n                source real[]",
            (true, true) => ",\n                source real[],\n                source_raw bytea",
            (false, true) => {
                panic!("persisted TurboQuant rerank-source fixtures require source real[]")
            }
        };
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key{source_column},
                embedding ecvector
            )"
        ))
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = include_source
                .then(|| format_recall_vector_sql_literal(&pq_fastscan_binary_runtime_source(id)));
            let source_raw = include_source_raw.then(|| {
                format_recall_vector_sql_literal(&turboquant_binary_runtime_rerank_source(id))
            });
            let embedding =
                format_recall_vector_sql_literal(&pq_fastscan_binary_runtime_embedding(id));
            let insert_sql = match (source, source_raw) {
                (Some(source), Some(source_raw)) => format!(
                    "INSERT INTO {table_name} VALUES \
                     ({id}, {source}, tests.ec_hnsw_debug_pack_f32_bytea({source_raw}), encode_to_ecvector({embedding}, 4, 42))"
                ),
                (Some(source), None) => format!(
                    "INSERT INTO {table_name} VALUES \
                     ({id}, {source}, encode_to_ecvector({embedding}, 4, 42))"
                ),
                (None, None) => format!(
                    "INSERT INTO {table_name} VALUES \
                     ({id}, encode_to_ecvector({embedding}, 4, 42))"
                ),
                (None, Some(_)) => unreachable!(
                    "persisted TurboQuant rerank-source fixtures require source real[]"
                ),
            };
            Spi::run(&insert_sql).expect("insert should succeed");
        }

        let build_source_column = if include_source {
            ", build_source_column = 'source'"
        } else {
            ""
        };
        let rerank_source_column = persisted_rerank_source_column
            .map(|column| format!(", rerank_source_column = '{column}'"))
            .unwrap_or_default();
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80{build_source_column}{rerank_source_column}, storage_format = 'turboquant')"
        ))
        .expect("index creation should succeed");

        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist")
    }

    fn create_turboquant_binary_runtime_fixture(table_name: &str, index_name: &str) -> pg_sys::Oid {
        create_turboquant_binary_runtime_fixture_internal(table_name, index_name, false, None)
    }

    fn create_turboquant_binary_runtime_fixture_with_source(
        table_name: &str,
        index_name: &str,
    ) -> pg_sys::Oid {
        create_turboquant_binary_runtime_fixture_internal(table_name, index_name, true, None)
    }

    fn create_turboquant_binary_runtime_fixture_with_persisted_source_raw(
        table_name: &str,
        index_name: &str,
    ) -> pg_sys::Oid {
        create_turboquant_binary_runtime_fixture_internal(
            table_name,
            index_name,
            true,
            Some("source_raw"),
        )
    }

    fn pq_fastscan_runtime_query() -> Vec<f32> {
        vec![
            0.12_f32, 0.22, 0.32, 0.42, 0.52, 0.62, 0.72, 0.82, 0.92, 1.02, 1.12, 1.22, 1.32, 1.42,
            1.52, 1.62,
        ]
    }

    fn pq_fastscan_binary_runtime_query() -> Vec<f32> {
        (0..RECALL_DIM)
            .map(|dim| {
                let dim = dim as i64;
                (((dim * 17 + 5) as f32) * 0.0013).sin() + (((dim * 29 + 11) as f32) * 0.0009).cos()
            })
            .collect()
    }

    fn runtime_fixture_embedding(id: i64) -> Vec<f32> {
        (0..16)
            .map(|dim| (((id * 29 + dim) as f32) * 0.02).sin())
            .collect()
    }

    fn pq_fastscan_runtime_source(id: i64) -> Vec<f32> {
        (0..16)
            .map(|dim| (((id * 41 + dim) as f32) * 0.03).cos())
            .collect()
    }

    fn pq_fastscan_binary_runtime_embedding(id: i64) -> Vec<f32> {
        (0..RECALL_DIM)
            .map(|dim| {
                let dim = dim as i64;
                (((id * 29 + dim * 7) as f32) * 0.0011).sin()
                    + (((id * 13 + dim * 3) as f32) * 0.0007).cos()
            })
            .collect()
    }

    fn pq_fastscan_binary_runtime_source(id: i64) -> Vec<f32> {
        (0..RECALL_DIM)
            .map(|dim| {
                let dim = dim as i64;
                (((id * 41 + dim * 11) as f32) * 0.0012).cos()
                    + (((id * 19 + dim * 5) as f32) * 0.0008).sin()
            })
            .collect()
    }

    fn turboquant_binary_runtime_rerank_source(id: i64) -> Vec<f32> {
        pq_fastscan_binary_runtime_source(id)
            .into_iter()
            .enumerate()
            .map(|(dim, value)| (-0.5 * value) + (dim as f32 * 0.002) - 0.25)
            .collect()
    }

    fn fetch_pq_fastscan_index_runtime_text(
        index_oid: pg_sys::Oid,
        column: &str,
    ) -> Option<String> {
        Spi::get_one::<String>(&format!(
            "SELECT {column} FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings_for_index({index_oid})"
        ))
        .expect("index runtime settings probe should succeed")
    }

    fn fetch_pq_fastscan_index_runtime_i32(index_oid: pg_sys::Oid, column: &str) -> Option<i32> {
        Spi::get_one::<i32>(&format!(
            "SELECT {column} FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings_for_index({index_oid})"
        ))
        .expect("index runtime settings probe should succeed")
    }

    fn observed_heap_tids_for_query(index_oid: pg_sys::Oid, query: Vec<f32>) -> Vec<(u32, u16)> {
        unsafe { am::debug_gettuple_scan_heap_tids_with_scores(index_oid, query) }
            .into_iter()
            .map(|(heap_tid, _score)| heap_tid)
            .collect()
    }

    fn observed_ids_for_query(
        index_oid: pg_sys::Oid,
        table_name: &str,
        query: Vec<f32>,
    ) -> Vec<usize> {
        let ctid_to_id = ctid_id_map(table_name);
        observed_heap_tids_for_query(index_oid, query)
            .into_iter()
            .map(|heap_tid| {
                *ctid_to_id
                    .get(&heap_tid)
                    .expect("observed heap tid should map back to a table row")
            })
            .collect()
    }

    fn first_self_ranked_runtime_fixture_id(
        index_oid: pg_sys::Oid,
        table_name: &str,
        candidate_ids: std::ops::RangeInclusive<i64>,
    ) -> i64 {
        candidate_ids
            .into_iter()
            .find(|id| {
                observed_ids_for_query(index_oid, table_name, runtime_fixture_embedding(*id))
                    .first()
                    .copied()
                    == Some(usize::try_from(*id).expect("runtime fixture id should fit in usize"))
            })
            .expect("fixture should expose at least one row that ranks first for its own embedding")
    }

    fn simulate_grouped_live_window_order(
        baseline_rows: &[DebugGroupedComparisonRow],
        window_size: usize,
    ) -> Vec<(u32, u16)> {
        let mut buffered_rows = Vec::with_capacity(window_size);
        let mut next_idx = 0usize;
        let mut expected_order = Vec::with_capacity(baseline_rows.len());
        while expected_order.len() < baseline_rows.len() {
            while buffered_rows.len() < window_size && next_idx < baseline_rows.len() {
                buffered_rows.push(baseline_rows[next_idx]);
                next_idx += 1;
            }
            let (selected_idx, _) = buffered_rows
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    left.3
                        .unwrap_or(left.2)
                        .total_cmp(&right.3.unwrap_or(right.2))
                        .then_with(|| left.1.cmp(&right.1))
                })
                .expect("windowed grouped simulation should always have a buffered row");
            let (heap_tid, _approx_rank, _approx_score, _comparison_score, _exact_rank, _shift) =
                buffered_rows.remove(selected_idx);
            expected_order.push(heap_tid);
        }
        expected_order
    }

    fn pq_fastscan_exact_traversal_runtime_observed_scores(
        table_name: &str,
        index_name: &str,
        scope: Option<&str>,
    ) -> (Vec<DebugScanComparisonRow>, HashMap<(u32, u16), f32>) {
        let _lock = env_var_test_lock();
        let _exact_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL", "1");
        let _scope_guard = scope
            .map(|value| ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE", value));
        let index_oid = create_pq_fastscan_runtime_fixture(table_name, index_name);
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(
                index_oid,
                pq_fastscan_runtime_query(),
            )
        };
        let emitted_scores = unsafe {
            am::debug_gettuple_scan_heap_tids_with_scores(index_oid, pq_fastscan_runtime_query())
        }
        .into_iter()
        .collect::<HashMap<_, _>>();
        (observed, emitted_scores)
    }

    fn pq_fastscan_rerank_runtime_observed_scores(
        table_name: &str,
        index_name: &str,
        rerank_mode: Option<&str>,
        rerank_source_column: Option<&str>,
        include_source_raw: bool,
    ) -> (Vec<DebugScanComparisonRow>, HashMap<(u32, u16), f32>) {
        let _lock = env_var_test_lock();
        let _rerank_guard =
            rerank_mode.map(|value| ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", value));
        let _source_guard = rerank_source_column
            .map(|value| ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN", value));
        let index_oid = if include_source_raw {
            create_pq_fastscan_runtime_fixture_with_source_raw(table_name, index_name)
        } else {
            create_pq_fastscan_runtime_fixture(table_name, index_name)
        };
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(
                index_oid,
                pq_fastscan_runtime_query(),
            )
        };
        let emitted_scores = unsafe {
            am::debug_gettuple_scan_heap_tids_with_scores(index_oid, pq_fastscan_runtime_query())
        }
        .into_iter()
        .collect::<HashMap<_, _>>();
        (observed, emitted_scores)
    }

    #[pg_test]
    fn test_pq_fastscan_exact_traversal_emits_exact_scores() {
        let (observed, emitted_scores) = pq_fastscan_exact_traversal_runtime_observed_scores(
            "ec_hnsw_pq_fastscan_runtime_exact_traversal",
            "ec_hnsw_pq_fastscan_runtime_exact_traversal_idx",
            None,
        );

        assert!(
            !observed.is_empty(),
            "exact grouped traversal runtime should still emit ordered results"
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score
                .expect("exact grouped traversal runtime should still attach comparison scores");
            assert_f32_close(
                emitted_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("exact traversal should emit an order-by score for every observed heap tid"),
                comparison_score,
                "exact grouped traversal should emit the same exact rerank score it records as the comparison sidecar",
            );
        }
    }

    #[pg_test]
    fn test_ech_debug_runtime_settings_reflect_controls() {
        let _lock = env_var_test_lock();
        let _window_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW", "8");
        let _score_mode_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE", "binary");
        let _rerank_mode_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "heap_f32");
        let _rerank_source_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN", "source_raw");
        let _exact_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL", "1");
        let _scope_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE", "all");
        let _strategy_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_STRATEGY", "expansion");
        let _limit_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT", "1");

        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT pq_fastscan_build_enabled
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed"),
            Some(true),
            "the runtime settings probe should surface that pq_fastscan build selection is always available via reloptions",
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT pq_fastscan_scan_enabled
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed"),
            Some(true),
            "the runtime settings probe should surface that pq_fastscan scan selection is always available",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_scan_window
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("8"),
            "the runtime settings probe should surface the configured pq_fastscan scan window",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_traversal_score_mode
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("binary"),
            "the runtime settings probe should surface the configured pq_fastscan traversal score mode",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_rerank_mode
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("heap_f32"),
            "the runtime settings probe should surface the configured pq_fastscan rerank mode",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_rerank_source_column
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("source_raw"),
            "the runtime settings probe should surface the configured pq_fastscan rerank source column",
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT pq_fastscan_exact_traversal_enabled
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed"),
            Some(true),
            "the runtime settings probe should surface the pq_fastscan exact traversal gate",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_exact_traversal_scope
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("all"),
            "the runtime settings probe should surface the pq_fastscan exact traversal scope",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_exact_traversal_strategy
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("expansion"),
            "the runtime settings probe should surface the pq_fastscan exact traversal strategy",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_exact_traversal_limit
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("1"),
            "the runtime settings probe should surface the pq_fastscan exact traversal limit",
        );
    }

    #[pg_test]
    fn test_ech_debug_runtime_settings_surface_effective_defaults() {
        let _lock = env_var_test_lock();

        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_scan_window
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("64"),
            "the runtime settings probe should surface the effective default pq_fastscan scan window",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_traversal_score_mode
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("binary"),
            "the runtime settings probe should surface the effective default pq_fastscan traversal score mode",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_rerank_mode
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("heap_f32"),
            "the runtime settings probe should surface the effective default pq_fastscan rerank mode",
        );
        assert_eq!(
            Spi::get_one::<String>(
                "SELECT pq_fastscan_rerank_source_column
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed")
            .as_deref(),
            Some("build_source_column"),
            "the runtime settings probe should surface the source-backed pq_fastscan rerank column label by default",
        );
        assert_eq!(
            Spi::get_one::<bool>(
                "SELECT pq_fastscan_exact_traversal_enabled
                 FROM tests.ec_hnsw_debug_pq_fastscan_runtime_settings()"
            )
            .expect("runtime settings probe should succeed"),
            Some(false),
            "the runtime settings probe should surface that exact traversal stays disabled by default",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_index_runtime_settings_report_binary_default() {
        let _lock = env_var_test_lock();
        let index_oid = create_pq_fastscan_binary_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_settings_binary_default",
            "ec_hnsw_pq_fastscan_runtime_settings_binary_default_idx",
        );

        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_traversal_score_mode")
                .as_deref(),
            Some("binary"),
            "the index-aware runtime settings helper should surface the effective traversal mode for a binary-sidecar pq_fastscan index",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(
                index_oid,
                "pq_fastscan_traversal_score_mode_resolution",
            )
            .as_deref(),
            Some("default_binary_with_binary_sidecar"),
            "the index-aware runtime settings helper should explain when binary traversal comes from the persisted binary sidecar default",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_i32(index_oid, "pq_fastscan_layout_binary_word_count"),
            Some(PQ_FASTSCAN_BINARY_RUNTIME_WORD_COUNT),
            "the index-aware runtime settings helper should surface the layout binary word count used to pick the default traversal path",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode").as_deref(),
            Some("heap_f32"),
            "the index-aware runtime settings helper should surface the effective heap_f32 rerank default for source-backed pq_fastscan indexes",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode_resolution")
                .as_deref(),
            Some("default_heap_f32_with_build_source_column"),
            "the index-aware runtime settings helper should explain when heap_f32 rerank came from the persisted build_source_column default",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_source_column")
                .as_deref(),
            Some("source"),
            "the index-aware runtime settings helper should surface the effective rerank source column name",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_index_runtime_settings_report_binary_fallback() {
        let _lock = env_var_test_lock();
        let index_oid = create_pq_fastscan_binary_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_settings_binary_fallback",
            "ec_hnsw_pq_fastscan_runtime_settings_binary_fallback_idx",
        );

        let (_block_count, _m, _ef_construction, mut metadata) =
            unsafe { am::debug_index_metadata(index_oid) };
        metadata.payload_flags &= !am::page::PAYLOAD_FLAG_BINARY_SIDECAR;
        unsafe { am::debug_update_index_metadata(index_oid, metadata) };

        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_traversal_score_mode")
                .as_deref(),
            Some("pq"),
            "the index-aware runtime settings helper should surface the actual grouped-pq fallback when a pq_fastscan index lacks a persisted binary sidecar",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(
                index_oid,
                "pq_fastscan_traversal_score_mode_resolution",
            )
            .as_deref(),
            Some("fallback_grouped_pq_missing_binary_sidecar"),
            "the index-aware runtime settings helper should explain when the binary default fell back because the index metadata no longer advertises a binary sidecar",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_i32(index_oid, "pq_fastscan_layout_binary_word_count"),
            Some(0),
            "the index-aware runtime settings helper should surface that the fallback path came from a zero-word binary layout",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_index_runtime_settings_report_env_override() {
        let _lock = env_var_test_lock();
        let _score_mode_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE", "pq");
        let _rerank_mode_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "quantized");
        let index_oid = create_pq_fastscan_binary_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_settings_env_override",
            "ec_hnsw_pq_fastscan_runtime_settings_env_override_idx",
        );

        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_traversal_score_mode")
                .as_deref(),
            Some("pq"),
            "the index-aware runtime settings helper should surface the env-selected traversal mode",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(
                index_oid,
                "pq_fastscan_traversal_score_mode_resolution",
            )
            .as_deref(),
            Some("env_override"),
            "the index-aware runtime settings helper should report that an explicit env override won over the layout default",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_i32(index_oid, "pq_fastscan_layout_binary_word_count"),
            Some(PQ_FASTSCAN_BINARY_RUNTIME_WORD_COUNT),
            "the index-aware runtime settings helper should still surface the persisted binary layout when an env override changes the selected traversal mode",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode").as_deref(),
            Some("quantized"),
            "the index-aware runtime settings helper should surface the env-selected rerank mode",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode_resolution")
                .as_deref(),
            Some("env_override"),
            "the index-aware runtime settings helper should report that an explicit rerank env override won over the source-backed default",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_source_column")
                .as_deref(),
            None,
            "the index-aware runtime settings helper should omit the rerank source column when quantized rerank is selected",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_index_runtime_settings_report_heap_override() {
        let _lock = env_var_test_lock();
        let _rerank_mode_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "heap_f32");
        let _rerank_source_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN", "source_raw");
        let index_oid = create_pq_fastscan_runtime_fixture_with_source_raw(
            "ec_hnsw_pq_fastscan_runtime_settings_heap_source_override",
            "ec_hnsw_pq_fastscan_runtime_settings_heap_source_override_idx",
        );

        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode").as_deref(),
            Some("heap_f32"),
            "the index-aware runtime settings helper should surface the env-selected heap_f32 rerank mode",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode_resolution")
                .as_deref(),
            Some("env_override"),
            "the index-aware runtime settings helper should report that the heap rerank mode came from an explicit env override",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_source_column")
                .as_deref(),
            Some("source_raw"),
            "the index-aware runtime settings helper should surface the effective rerank source override column name",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_runtime_settings_report_persisted_source() {
        let _lock = env_var_test_lock();
        let index_oid = create_pq_fastscan_runtime_fixture_with_persisted_source_raw(
            "ec_hnsw_pq_fastscan_runtime_settings_persisted_heap_source",
            "ec_hnsw_pq_fastscan_runtime_settings_persisted_heap_source_idx",
        );

        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode").as_deref(),
            Some("heap_f32"),
            "the index-aware runtime settings helper should surface the persisted heap_f32 default when rerank_source_column is present",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode_resolution")
                .as_deref(),
            Some("default_heap_f32_with_rerank_source_column"),
            "the index-aware runtime settings helper should explain when heap_f32 came from a persisted rerank_source_column",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_source_column")
                .as_deref(),
            Some("source_raw"),
            "the index-aware runtime settings helper should surface the persisted rerank source column name",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_indexed_ecvector_ignores_tqvector_sibling() {
        let _lock = env_var_test_lock();

        let table_name = "ec_hnsw_pq_fastscan_indexed_ecvector_tqvector_sibling";
        let index_name = "ec_hnsw_pq_fastscan_indexed_ecvector_tqvector_sibling_idx";
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key,
                artifact tqvector,
                embedding ecvector
            )"
        ))
        .expect("table creation should succeed");

        for id in 1..=16 {
            let artifact =
                format_recall_vector_sql_literal(&turboquant_binary_runtime_rerank_source(id));
            let embedding = format_recall_vector_sql_literal(&runtime_fixture_embedding(id));
            Spi::run(&format!(
                "INSERT INTO {table_name} VALUES \
                 ({id}, encode_to_tqvector({artifact}, 4, 42), encode_to_ecvector({embedding}, 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, storage_format = 'pq_fastscan')"
        ))
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode").as_deref(),
            Some("heap_f32"),
            "indexed ecvector pq_fastscan indexes should still default to heap_f32 rerank even when the table carries a tqvector sibling",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_mode_resolution")
                .as_deref(),
            Some("default_heap_f32_with_indexed_column"),
            "the runtime settings helper should explain that heap_f32 came from the indexed ecvector column, not the tqvector sibling",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(index_oid, "pq_fastscan_rerank_source_column")
                .as_deref(),
            None,
            "the indexed ecvector default should not resolve a sibling tqvector column as the rerank source",
        );

        let query = pq_fastscan_runtime_query();
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
        };
        let emitted_scores =
            unsafe { am::debug_gettuple_scan_heap_tids_with_scores(index_oid, query.clone()) }
                .into_iter()
                .collect::<HashMap<_, _>>();
        let exact_scores = (1..=16)
            .map(|id| {
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &runtime_fixture_embedding(id)),
                )
            })
            .collect::<HashMap<_, _>>();

        assert!(
            !observed.is_empty(),
            "indexed ecvector pq_fastscan output should still emit ordered results when a sibling tqvector column is present",
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score.expect(
                "indexed ecvector pq_fastscan scans should attach exact comparison scores by default",
            );
            let expected = exact_scores.get(&heap_tid).copied().expect(
                "every emitted heap tid should map back to an exact indexed-ecvector score",
            );
            assert_f32_close(
                comparison_score,
                expected,
                "the default indexed-ecvector heap rerank should score against the indexed ecvector column, not a sibling tqvector artifact",
            );
            assert_f32_close(
                emitted_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("emitted score should be present for every observed heap tid"),
                expected,
                "the order-by score should match the indexed ecvector exact comparison score",
            );
        }
    }

    #[pg_test]
    fn test_pq_fastscan_default_source_rerank_emits_heap_scores() {
        let table_name = "ec_hnsw_pq_fastscan_runtime_source_backed_default_rerank";
        let index_name = "ec_hnsw_pq_fastscan_runtime_source_backed_default_rerank_idx";
        let (observed, emitted_scores) =
            pq_fastscan_rerank_runtime_observed_scores(table_name, index_name, None, None, false);
        let query = pq_fastscan_runtime_query();
        let exact_scores = (1..=16)
            .map(|id| {
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &pq_fastscan_runtime_source(id)),
                )
            })
            .collect::<HashMap<_, _>>();

        assert!(
            !observed.is_empty(),
            "source-backed default PqFastScan rerank should still emit ordered results"
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score.expect(
                "source-backed default PqFastScan rerank should attach exact heap comparison scores",
            );
            let expected = exact_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact heap score");
            assert_f32_close(
                comparison_score,
                expected,
                "source-backed default PqFastScan rerank should use the raw heap f32 inner product",
            );
            assert_f32_close(
                emitted_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("emitted score should be present for every observed heap tid"),
                expected,
                "source-backed default PqFastScan rerank should emit the exact heap comparison score as the order-by score",
            );
        }
    }

    #[pg_test]
    fn test_pq_fastscan_default_rerank_matches_explicit_heap() {
        let _lock = env_var_test_lock();
        let default_index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_default_rerank_parity",
            "ec_hnsw_pq_fastscan_runtime_default_rerank_parity_idx",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(default_index_oid, "pq_fastscan_rerank_mode")
                .as_deref(),
            Some("heap_f32"),
            "the default source-backed fixture should resolve to heap_f32 rerank",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(
                default_index_oid,
                "pq_fastscan_rerank_mode_resolution",
            )
            .as_deref(),
            Some("default_heap_f32_with_build_source_column"),
            "the default source-backed fixture should report that heap_f32 came from build_source_column",
        );
        let default_observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(
                default_index_oid,
                pq_fastscan_runtime_query(),
            )
        };

        let _rerank_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "heap_f32");
        let explicit_index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_explicit_heap_rerank_parity",
            "ec_hnsw_pq_fastscan_runtime_explicit_heap_rerank_parity_idx",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(explicit_index_oid, "pq_fastscan_rerank_mode")
                .as_deref(),
            Some("heap_f32"),
            "the explicit heap override fixture should still resolve to heap_f32 rerank",
        );
        assert_eq!(
            fetch_pq_fastscan_index_runtime_text(
                explicit_index_oid,
                "pq_fastscan_rerank_mode_resolution",
            )
            .as_deref(),
            Some("env_override"),
            "the explicit heap override fixture should report that heap_f32 came from the env override",
        );
        let explicit_heap_observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(
                explicit_index_oid,
                pq_fastscan_runtime_query(),
            )
        };

        let default_scores = default_observed
            .into_iter()
            .map(
                |(_heap_tid, emitted_score, comparison_score, approx_rank)| {
                    (
                        emitted_score,
                        comparison_score
                            .expect("default heap rerank should emit comparison scores"),
                        approx_rank,
                    )
                },
            )
            .collect::<Vec<_>>();
        let explicit_heap_scores = explicit_heap_observed
            .into_iter()
            .map(
                |(_heap_tid, emitted_score, comparison_score, approx_rank)| {
                    (
                        emitted_score,
                        comparison_score
                            .expect("explicit heap rerank should emit comparison scores"),
                        approx_rank,
                    )
                },
            )
            .collect::<Vec<_>>();

        assert_eq!(
            default_scores, explicit_heap_scores,
            "source-backed default rerank should match the explicit heap_f32 override exactly"
        );
    }

    #[pg_test]
    fn test_pq_fastscan_heap_rerank_emits_heap_exact_scores() {
        let table_name = "ec_hnsw_pq_fastscan_runtime_heap_rerank";
        let index_name = "ec_hnsw_pq_fastscan_runtime_heap_rerank_idx";
        let (observed, emitted_scores) = pq_fastscan_rerank_runtime_observed_scores(
            table_name,
            index_name,
            Some("heap_f32"),
            None,
            false,
        );
        let query = pq_fastscan_runtime_query();
        let exact_scores = (1..=16)
            .map(|id| {
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &pq_fastscan_runtime_source(id)),
                )
            })
            .collect::<HashMap<_, _>>();

        assert!(
            !observed.is_empty(),
            "PqFastScan heap rerank should still emit ordered results"
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score
                .expect("PqFastScan heap rerank should attach exact heap comparison scores");
            let expected = exact_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact heap score");
            assert_f32_close(
                comparison_score,
                expected,
                "PqFastScan heap rerank comparison score should match the raw heap f32 inner product",
            );
            assert_f32_close(
                emitted_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("emitted score should be present for every observed heap tid"),
                expected,
                "PqFastScan heap rerank should emit the exact heap comparison score as the order-by score",
            );
        }
    }

    #[pg_test]
    fn test_pq_fastscan_heap_rerank_bytea_source_emits_exact_scores() {
        let table_name = "ec_hnsw_pq_fastscan_runtime_heap_rerank_bytea";
        let index_name = "ec_hnsw_pq_fastscan_runtime_heap_rerank_bytea_idx";
        let (observed, emitted_scores) = pq_fastscan_rerank_runtime_observed_scores(
            table_name,
            index_name,
            Some("heap_f32"),
            Some("source_raw"),
            true,
        );
        let query = pq_fastscan_runtime_query();
        let exact_scores = (1..=16)
            .map(|id| {
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &pq_fastscan_runtime_source(id)),
                )
            })
            .collect::<HashMap<_, _>>();

        assert!(
            !observed.is_empty(),
            "PqFastScan heap rerank should still emit ordered results with a bytea source override"
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score.expect(
                "PqFastScan heap rerank with a bytea source override should attach exact heap comparison scores",
            );
            let expected = exact_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact heap score");
            assert_f32_close(
                comparison_score,
                expected,
                "PqFastScan heap rerank bytea comparison score should match the raw heap f32 inner product",
            );
            assert_f32_close(
                emitted_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("emitted score should be present for every observed heap tid"),
                expected,
                "PqFastScan heap rerank with a bytea source override should emit the exact heap comparison score as the order-by score",
            );
        }
    }

    #[pg_test]
    fn test_pq_fastscan_persisted_bytea_rerank_emits_scores() {
        let table_name = "ec_hnsw_pq_fastscan_runtime_persisted_heap_rerank_bytea";
        let index_name = "ec_hnsw_pq_fastscan_runtime_persisted_heap_rerank_bytea_idx";
        let index_oid =
            create_pq_fastscan_runtime_fixture_with_persisted_source_raw(table_name, index_name);
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(
                index_oid,
                pq_fastscan_runtime_query(),
            )
        };
        let emitted_scores = unsafe {
            am::debug_gettuple_scan_heap_tids_with_scores(index_oid, pq_fastscan_runtime_query())
        }
        .into_iter()
        .collect::<HashMap<_, _>>();
        let query = pq_fastscan_runtime_query();
        let exact_scores = (1..=16)
            .map(|id| {
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &pq_fastscan_runtime_source(id)),
                )
            })
            .collect::<HashMap<_, _>>();

        assert!(
            !observed.is_empty(),
            "PqFastScan should still emit ordered results when rerank_source_column persists a bytea source"
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score.expect(
                "a persisted bytea rerank_source_column should attach exact heap comparison scores",
            );
            let expected = exact_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact heap score");
            assert_f32_close(
                comparison_score,
                expected,
                "a persisted bytea rerank_source_column should use the raw heap f32 inner product",
            );
            assert_f32_close(
                emitted_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("emitted score should be present for every observed heap tid"),
                expected,
                "a persisted bytea rerank_source_column should emit the exact heap comparison score as the order-by score",
            );
        }
    }

    #[pg_test]
    fn test_pq_fastscan_profile_exact_counters_zero_without_gate() {
        let _lock = env_var_test_lock();
        let index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_profile_approx",
            "ec_hnsw_pq_fastscan_runtime_profile_approx_idx",
        );
        let (
            _rescan_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            _rescan_phase,
            _rescan_current_result,
            _rescan_ordered_slots,
            _rescan_pending_heap_tids,
            _rescan_visited_elements,
            _rescan_expanded_sources,
            _rescan_emitted_elements,
            _rescan_bootstrap_expansions,
            _rescan_bootstrap_pages_read,
            _rescan_quantizer_cache_hit,
            _result_count,
            _final_phase,
            _final_ordered_slots,
            _total_bootstrap_expansions,
            _total_bootstrap_pages_read,
            _total_linear_pages_read,
            _total_elements_scored,
            _total_elements_skipped,
            _total_heap_tids_returned,
            _total_quantizer_cache_hit,
            _total_emitted_elements,
            _rescan_amrescan_total_elapsed_us,
            _rescan_query_decode_elapsed_us,
            _rescan_scan_setup_elapsed_us,
            _rescan_store_query_elapsed_us,
            _rescan_prepare_query_elapsed_us,
            _rescan_reset_state_elapsed_us,
            _rescan_initialize_entry_elapsed_us,
            _rescan_upper_layer_seed_elapsed_us,
            _rescan_layer0_seed_elapsed_us,
            _rescan_stage_ordered_results_elapsed_us,
            _rescan_initial_prefetch_elapsed_us,
            _rescan_frontier_consume_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _graph_element_cache_hits,
            _graph_element_cache_misses,
            _graph_element_load_elapsed_us,
            _graph_neighbor_cache_hits,
            _graph_neighbor_cache_misses,
            _graph_neighbor_load_elapsed_us,
            _candidate_score_calls,
            _candidate_score_elapsed_us,
            _score_cache_hits,
            _score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, pq_fastscan_runtime_query()) };

        assert!(
            grouped_traversal_approx_score_calls > 0
                && grouped_traversal_approx_score_elapsed_us >= 0,
            "grouped approximate scans should surface grouped approximate traversal scoring work",
        );
        assert_eq!(
            (
                grouped_traversal_exact_score_calls,
                grouped_traversal_exact_score_elapsed_us,
                grouped_traversal_budgeted_expansions,
                grouped_traversal_budgeted_candidates,
                grouped_traversal_budgeted_exact_candidates,
            ),
            (0, 0, 0, 0, 0),
            "grouped approximate scans should leave grouped exact traversal counters inert",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_binary_score_mode_bypasses_grouped_pq_scoring() {
        let _lock = env_var_test_lock();
        let _score_mode_guard =
            ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE", "binary");
        let index_oid = create_pq_fastscan_binary_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_profile_binary_score_mode",
            "ec_hnsw_pq_fastscan_runtime_profile_binary_score_mode_idx",
        );
        let (
            _rescan_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            _rescan_phase,
            _rescan_current_result,
            _rescan_ordered_slots,
            _rescan_pending_heap_tids,
            _rescan_visited_elements,
            _rescan_expanded_sources,
            _rescan_emitted_elements,
            _rescan_bootstrap_expansions,
            _rescan_bootstrap_pages_read,
            _rescan_quantizer_cache_hit,
            result_count,
            _final_phase,
            _final_ordered_slots,
            _total_bootstrap_expansions,
            _total_bootstrap_pages_read,
            _total_linear_pages_read,
            _total_elements_scored,
            _total_elements_skipped,
            _total_heap_tids_returned,
            _total_quantizer_cache_hit,
            _total_emitted_elements,
            _rescan_amrescan_total_elapsed_us,
            _rescan_query_decode_elapsed_us,
            _rescan_scan_setup_elapsed_us,
            _rescan_store_query_elapsed_us,
            _rescan_prepare_query_elapsed_us,
            _rescan_reset_state_elapsed_us,
            _rescan_initialize_entry_elapsed_us,
            _rescan_upper_layer_seed_elapsed_us,
            _rescan_layer0_seed_elapsed_us,
            _rescan_stage_ordered_results_elapsed_us,
            _rescan_initial_prefetch_elapsed_us,
            _rescan_frontier_consume_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _graph_element_cache_hits,
            _graph_element_cache_misses,
            _graph_element_load_elapsed_us,
            _graph_neighbor_cache_hits,
            _graph_neighbor_cache_misses,
            _graph_neighbor_load_elapsed_us,
            _candidate_score_calls,
            _candidate_score_elapsed_us,
            _score_cache_hits,
            _score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe {
            am::debug_profile_ordered_scan(index_oid, pq_fastscan_binary_runtime_query())
        };

        assert!(
            result_count > 0,
            "binary grouped traversal score mode should still emit ordered results",
        );
        assert_eq!(
            (
                grouped_traversal_approx_score_calls,
                grouped_traversal_approx_score_elapsed_us,
                grouped_traversal_exact_score_calls,
                grouped_traversal_exact_score_elapsed_us,
                grouped_traversal_budgeted_expansions,
                grouped_traversal_budgeted_candidates,
                grouped_traversal_budgeted_exact_candidates,
            ),
            (0, 0, 0, 0, 0, 0, 0),
            "binary grouped traversal score mode should bypass grouped PQ scoring and leave exact-traversal counters inert without the exact gate",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_quantized_rerank_profile_quantized_only() {
        let _lock = env_var_test_lock();
        let _window_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW", "8");
        let _rerank_mode_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "quantized");
        let index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_quantized_rerank_profile",
            "ec_hnsw_pq_fastscan_runtime_quantized_rerank_profile_idx",
        );
        let (
            _rescan_amrescan_total_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            result_count,
            grouped_rerank_quantized_score_calls,
            grouped_rerank_quantized_score_elapsed_us,
            grouped_rerank_heap_score_calls,
            grouped_rerank_heap_score_elapsed_us,
            grouped_rerank_heap_rows_fetched,
            grouped_rerank_heap_fetch_elapsed_us,
            grouped_rerank_heap_decode_elapsed_us,
            grouped_rerank_heap_dot_elapsed_us,
        ) = unsafe { am::debug_grouped_rerank_profile(index_oid, pq_fastscan_runtime_query(), 10) };

        assert!(
            result_count > 0,
            "quantized grouped rerank profile should still emit ordered results",
        );
        assert!(
            grouped_rerank_quantized_score_calls > 0
                && grouped_rerank_quantized_score_elapsed_us >= 0,
            "quantized grouped rerank profile should surface quantized comparison work",
        );
        assert_eq!(
            (
                grouped_rerank_heap_score_calls,
                grouped_rerank_heap_score_elapsed_us,
                grouped_rerank_heap_rows_fetched,
                grouped_rerank_heap_fetch_elapsed_us,
                grouped_rerank_heap_decode_elapsed_us,
                grouped_rerank_heap_dot_elapsed_us,
            ),
            (0, 0, 0, 0, 0, 0),
            "quantized grouped rerank profile should leave heap rerank counters inert",
        );
    }

    #[pg_test]
    fn test_pq_fastscan_heap_rerank_profile_reports_heap_only() {
        let _lock = env_var_test_lock();
        let _window_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW", "8");
        let _rerank_mode_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "heap_f32");
        let index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_heap_rerank_profile",
            "ec_hnsw_pq_fastscan_runtime_heap_rerank_profile_idx",
        );
        let (
            _rescan_amrescan_total_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            result_count,
            grouped_rerank_quantized_score_calls,
            grouped_rerank_quantized_score_elapsed_us,
            grouped_rerank_heap_score_calls,
            grouped_rerank_heap_score_elapsed_us,
            grouped_rerank_heap_rows_fetched,
            grouped_rerank_heap_fetch_elapsed_us,
            grouped_rerank_heap_decode_elapsed_us,
            grouped_rerank_heap_dot_elapsed_us,
        ) = unsafe { am::debug_grouped_rerank_profile(index_oid, pq_fastscan_runtime_query(), 10) };

        assert!(
            result_count > 0,
            "heap-f32 grouped rerank profile should still emit ordered results",
        );
        assert_eq!(
            (
                grouped_rerank_quantized_score_calls,
                grouped_rerank_quantized_score_elapsed_us,
            ),
            (0, 0),
            "heap-f32 grouped rerank profile should bypass quantized rerank counters",
        );
        assert!(
            grouped_rerank_heap_score_calls > 0 && grouped_rerank_heap_score_elapsed_us >= 0,
            "heap-f32 grouped rerank profile should surface per-element heap rerank work",
        );
        assert!(
            grouped_rerank_heap_rows_fetched >= grouped_rerank_heap_score_calls
                && grouped_rerank_heap_fetch_elapsed_us >= 0
                && grouped_rerank_heap_decode_elapsed_us >= 0
                && grouped_rerank_heap_dot_elapsed_us >= 0,
            "heap-f32 grouped rerank profile should surface heap fetch, decode, and dot-product work for survivor rows",
        );
    }

    #[pg_test]
    fn test_turboquant_quantized_rerank_profile_reports_quantized_only() {
        let _lock = env_var_test_lock();
        let index_oid = create_turboquant_binary_runtime_fixture(
            "ec_hnsw_turboquant_runtime_quantized_rerank_profile",
            "ec_hnsw_turboquant_runtime_quantized_rerank_profile_idx",
        );
        let (
            _rescan_amrescan_total_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            result_count,
            grouped_rerank_quantized_score_calls,
            grouped_rerank_quantized_score_elapsed_us,
            grouped_rerank_heap_score_calls,
            grouped_rerank_heap_score_elapsed_us,
            grouped_rerank_heap_rows_fetched,
            grouped_rerank_heap_fetch_elapsed_us,
            grouped_rerank_heap_decode_elapsed_us,
            grouped_rerank_heap_dot_elapsed_us,
        ) = unsafe {
            am::debug_grouped_rerank_profile(index_oid, pq_fastscan_binary_runtime_query(), 10)
        };

        assert!(
            result_count > 0,
            "turboquant quantized rerank profile should still emit ordered results",
        );
        assert!(
            grouped_rerank_quantized_score_calls > 0
                && grouped_rerank_quantized_score_elapsed_us >= 0,
            "turboquant quantized rerank profile should surface deferred quantized comparison work",
        );
        assert_eq!(
            (
                grouped_rerank_heap_score_calls,
                grouped_rerank_heap_score_elapsed_us,
                grouped_rerank_heap_rows_fetched,
                grouped_rerank_heap_fetch_elapsed_us,
                grouped_rerank_heap_decode_elapsed_us,
                grouped_rerank_heap_dot_elapsed_us,
            ),
            (0, 0, 0, 0, 0, 0),
            "turboquant quantized rerank profile should leave heap rerank counters inert",
        );
    }

    #[pg_test]
    fn test_turboquant_heap_rerank_profile_reports_heap_only() {
        let _lock = env_var_test_lock();
        let _rerank_mode_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "heap_f32");
        let index_oid = create_turboquant_binary_runtime_fixture_with_source(
            "ec_hnsw_turboquant_runtime_heap_rerank_profile",
            "ec_hnsw_turboquant_runtime_heap_rerank_profile_idx",
        );
        let (
            _rescan_amrescan_total_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            result_count,
            grouped_rerank_quantized_score_calls,
            grouped_rerank_quantized_score_elapsed_us,
            grouped_rerank_heap_score_calls,
            grouped_rerank_heap_score_elapsed_us,
            grouped_rerank_heap_rows_fetched,
            grouped_rerank_heap_fetch_elapsed_us,
            grouped_rerank_heap_decode_elapsed_us,
            grouped_rerank_heap_dot_elapsed_us,
        ) = unsafe {
            am::debug_grouped_rerank_profile(index_oid, pq_fastscan_binary_runtime_query(), 10)
        };

        assert!(
            result_count > 0,
            "turboquant heap-f32 rerank profile should still emit ordered results",
        );
        assert_eq!(
            (
                grouped_rerank_quantized_score_calls,
                grouped_rerank_quantized_score_elapsed_us,
            ),
            (0, 0),
            "turboquant heap-f32 rerank profile should bypass quantized rerank counters",
        );
        assert!(
            grouped_rerank_heap_score_calls > 0 && grouped_rerank_heap_score_elapsed_us >= 0,
            "turboquant heap-f32 rerank profile should surface per-element heap rerank work",
        );
        assert!(
            grouped_rerank_heap_rows_fetched >= grouped_rerank_heap_score_calls
                && grouped_rerank_heap_fetch_elapsed_us >= 0
                && grouped_rerank_heap_decode_elapsed_us >= 0
                && grouped_rerank_heap_dot_elapsed_us >= 0,
            "turboquant heap-f32 rerank profile should surface heap fetch, decode, and dot-product work for survivor rows",
        );
    }

    #[pg_test]
    fn test_turboquant_source_backed_default_rerank_stays_quantized() {
        let _lock = env_var_test_lock();
        let index_oid = create_turboquant_binary_runtime_fixture_with_source(
            "ec_hnsw_turboquant_runtime_source_backed_default_quantized",
            "ec_hnsw_turboquant_runtime_source_backed_default_quantized_idx",
        );
        let (
            _rescan_amrescan_total_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            result_count,
            grouped_rerank_quantized_score_calls,
            grouped_rerank_quantized_score_elapsed_us,
            grouped_rerank_heap_score_calls,
            grouped_rerank_heap_score_elapsed_us,
            grouped_rerank_heap_rows_fetched,
            grouped_rerank_heap_fetch_elapsed_us,
            grouped_rerank_heap_decode_elapsed_us,
            grouped_rerank_heap_dot_elapsed_us,
        ) = unsafe {
            am::debug_grouped_rerank_profile(index_oid, pq_fastscan_binary_runtime_query(), 10)
        };

        assert!(
            result_count > 0,
            "source-backed turboquant default rerank should still emit ordered results",
        );
        assert!(
            grouped_rerank_quantized_score_calls > 0
                && grouped_rerank_quantized_score_elapsed_us >= 0,
            "source-backed turboquant default rerank should stay on quantized comparisons",
        );
        assert_eq!(
            (
                grouped_rerank_heap_score_calls,
                grouped_rerank_heap_score_elapsed_us,
                grouped_rerank_heap_rows_fetched,
                grouped_rerank_heap_fetch_elapsed_us,
                grouped_rerank_heap_decode_elapsed_us,
                grouped_rerank_heap_dot_elapsed_us,
            ),
            (0, 0, 0, 0, 0, 0),
            "source-backed turboquant default rerank should leave heap rerank counters inert until explicitly overridden",
        );
    }

    #[pg_test]
    fn test_turboquant_persisted_rerank_source_default_stays_quantized() {
        let _lock = env_var_test_lock();
        let index_oid = create_turboquant_binary_runtime_fixture_with_persisted_source_raw(
            "ec_hnsw_tq_persisted_rerank_default_q",
            "ec_hnsw_tq_persisted_rerank_default_q_idx",
        );
        let (
            _rescan_amrescan_total_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            result_count,
            grouped_rerank_quantized_score_calls,
            grouped_rerank_quantized_score_elapsed_us,
            grouped_rerank_heap_score_calls,
            grouped_rerank_heap_score_elapsed_us,
            grouped_rerank_heap_rows_fetched,
            grouped_rerank_heap_fetch_elapsed_us,
            grouped_rerank_heap_decode_elapsed_us,
            grouped_rerank_heap_dot_elapsed_us,
        ) = unsafe {
            am::debug_grouped_rerank_profile(index_oid, pq_fastscan_binary_runtime_query(), 10)
        };

        assert!(
            result_count > 0,
            "persisted-rerank-source turboquant default rerank should still emit ordered results",
        );
        assert!(
            grouped_rerank_quantized_score_calls > 0
                && grouped_rerank_quantized_score_elapsed_us >= 0,
            "persisted-rerank-source turboquant default rerank should stay on quantized comparisons",
        );
        assert_eq!(
            (
                grouped_rerank_heap_score_calls,
                grouped_rerank_heap_score_elapsed_us,
                grouped_rerank_heap_rows_fetched,
                grouped_rerank_heap_fetch_elapsed_us,
                grouped_rerank_heap_decode_elapsed_us,
                grouped_rerank_heap_dot_elapsed_us,
            ),
            (0, 0, 0, 0, 0, 0),
            "persisted-rerank-source turboquant default rerank should leave heap rerank counters inert until explicitly overridden",
        );
    }

    #[pg_test]
    fn test_turboquant_persisted_bytea_rerank_emits_scores() {
        let _lock = env_var_test_lock();
        let _rerank_mode_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "heap_f32");
        let table_name = "ec_hnsw_tq_persisted_heap_rerank_bytea";
        let index_name = "ec_hnsw_tq_persisted_heap_rerank_bytea_idx";
        let index_oid = create_turboquant_binary_runtime_fixture_with_persisted_source_raw(
            table_name, index_name,
        );
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(
                index_oid,
                pq_fastscan_binary_runtime_query(),
            )
        };
        let emitted_scores = unsafe {
            am::debug_gettuple_scan_heap_tids_with_scores(
                index_oid,
                pq_fastscan_binary_runtime_query(),
            )
        }
        .into_iter()
        .collect::<HashMap<_, _>>();
        let query = pq_fastscan_binary_runtime_query();
        let rerank_scores = (1..=16)
            .map(|id| {
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &turboquant_binary_runtime_rerank_source(id)),
                )
            })
            .collect::<HashMap<_, _>>();
        let build_source_scores = (1..=16)
            .map(|id| {
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &pq_fastscan_binary_runtime_source(id)),
                )
            })
            .collect::<HashMap<_, _>>();

        assert!(
            !observed.is_empty(),
            "TurboQuant heap rerank should still emit ordered results when rerank_source_column persists a bytea source",
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score.expect(
                "a persisted TurboQuant bytea rerank_source_column should attach exact heap comparison scores",
            );
            let expected = rerank_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact rerank-source score");
            let build_source_expected = build_source_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact build-source score");
            assert!(
                (expected - build_source_expected).abs() > 1.0e-3,
                "the persisted TurboQuant rerank source should differ materially from build_source_column for emitted rows",
            );
            assert_f32_close(
                comparison_score,
                expected,
                "a persisted TurboQuant bytea rerank_source_column should use the raw heap f32 inner product from source_raw",
            );
            assert_f32_close(
                emitted_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("emitted score should be present for every observed heap tid"),
                expected,
                "a persisted TurboQuant bytea rerank_source_column should emit the exact heap comparison score as the order-by score",
            );
        }
    }

    #[pg_test]
    fn test_turboquant_rerank_source_reloption_reset_round_trip() {
        let _lock = env_var_test_lock();
        let _rerank_mode_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_RERANK_MODE", "heap_f32");
        let table_name = "ec_hnsw_tq_rerank_source_reloption_round_trip";
        let index_name = "ec_hnsw_tq_rerank_source_reloption_round_trip_idx";
        let index_oid = create_turboquant_binary_runtime_fixture_with_persisted_source_raw(
            table_name, index_name,
        );
        let query = pq_fastscan_binary_runtime_query();
        let rerank_scores = (1..=16)
            .map(|id| {
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &turboquant_binary_runtime_rerank_source(id)),
                )
            })
            .collect::<HashMap<_, _>>();
        let build_source_scores = (1..=16)
            .map(|id| {
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &pq_fastscan_binary_runtime_source(id)),
                )
            })
            .collect::<HashMap<_, _>>();
        let assert_matches_expected = |observed: Vec<DebugScanComparisonRow>,
                                       expected_scores: &HashMap<(u32, u16), f32>,
                                       alternate_scores: &HashMap<(u32, u16), f32>,
                                       message_prefix: &str| {
            assert!(
                !observed.is_empty(),
                "{message_prefix} should still emit ordered results",
            );
            for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
                let comparison_score =
                    comparison_score.expect("heap_f32 rerank should attach comparison scores");
                let expected = expected_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("every emitted heap tid should map back to an exact source score");
                let alternate = alternate_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("every emitted heap tid should map back to the alternate source score");
                assert!(
                    (expected - alternate).abs() > 1.0e-3,
                    "{message_prefix} fixture should keep rerank and build-source scores materially different",
                );
                assert_f32_close(
                    comparison_score,
                    expected,
                    &format!("{message_prefix} should use the expected exact heap source"),
                );
            }
        };
        let reloptions_for_index = || {
            Spi::get_one::<Vec<String>>(&format!(
                "SELECT reloptions FROM pg_class WHERE oid = '{index_name}'::regclass"
            ))
            .expect("reloptions query should succeed")
            .expect("reloptions should exist")
        };

        assert!(reloptions_for_index().contains(&"rerank_source_column=source_raw".to_string()));
        assert_matches_expected(
            unsafe {
                am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
            },
            &rerank_scores,
            &build_source_scores,
            "persisted TurboQuant rerank_source_column",
        );

        Spi::run(&format!(
            "ALTER INDEX {index_name} RESET (rerank_source_column)"
        ))
        .expect("ALTER INDEX RESET should clear the persisted rerank source reloption");
        assert!(
            !reloptions_for_index().contains(&"rerank_source_column=source_raw".to_string()),
            "RESET should remove rerank_source_column from reloptions",
        );
        assert_matches_expected(
            unsafe {
                am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
            },
            &build_source_scores,
            &rerank_scores,
            "reset TurboQuant rerank_source_column",
        );

        Spi::run(&format!(
            "ALTER INDEX {index_name} SET (rerank_source_column = 'source_raw')"
        ))
        .expect("ALTER INDEX SET should restore the persisted rerank source reloption");
        assert!(reloptions_for_index().contains(&"rerank_source_column=source_raw".to_string()));
        assert_matches_expected(
            unsafe { am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query) },
            &rerank_scores,
            &build_source_scores,
            "restored TurboQuant rerank_source_column",
        );
    }

    #[pg_test]
    fn test_ech_debug_pq_fastscan_rerank_profile_sql_surface() {
        let _lock = env_var_test_lock();
        let _window_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW", "8");
        let index_name = "ec_hnsw_pq_fastscan_rerank_profile_sql_surface_idx";
        let _index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_rerank_profile_sql_surface",
            index_name,
        );
        let query_literal = format_recall_vector_sql_literal(&pq_fastscan_runtime_query());
        let (
            result_count,
            pq_fastscan_rerank_quantized_score_calls,
            pq_fastscan_rerank_quantized_score_elapsed_us,
            pq_fastscan_rerank_heap_score_calls,
            pq_fastscan_rerank_heap_score_elapsed_us,
            pq_fastscan_rerank_heap_rows_fetched,
            pq_fastscan_rerank_heap_fetch_elapsed_us,
            pq_fastscan_rerank_heap_decode_elapsed_us,
            pq_fastscan_rerank_heap_dot_elapsed_us,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            result_count,
                            pq_fastscan_rerank_quantized_score_calls,
                            pq_fastscan_rerank_quantized_score_elapsed_us,
                            pq_fastscan_rerank_heap_score_calls,
                            pq_fastscan_rerank_heap_score_elapsed_us,
                            pq_fastscan_rerank_heap_rows_fetched,
                            pq_fastscan_rerank_heap_fetch_elapsed_us,
                            pq_fastscan_rerank_heap_decode_elapsed_us,
                            pq_fastscan_rerank_heap_dot_elapsed_us
                         FROM tests.ec_hnsw_debug_pq_fastscan_rerank_profile(
                            '{index_name}'::regclass::oid,
                            {query_literal},
                            10
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("pq fastscan rerank profile query should succeed")
                .next()
                .expect("pq fastscan rerank profile should return one row");
            (
                row["result_count"]
                    .value::<i32>()
                    .expect("result count should decode")
                    .expect("result count should be non-null"),
                row["pq_fastscan_rerank_quantized_score_calls"]
                    .value::<i32>()
                    .expect("quantized score call count should decode")
                    .expect("quantized score call count should be non-null"),
                row["pq_fastscan_rerank_quantized_score_elapsed_us"]
                    .value::<i64>()
                    .expect("quantized score elapsed should decode")
                    .expect("quantized score elapsed should be non-null"),
                row["pq_fastscan_rerank_heap_score_calls"]
                    .value::<i32>()
                    .expect("heap score call count should decode")
                    .expect("heap score call count should be non-null"),
                row["pq_fastscan_rerank_heap_score_elapsed_us"]
                    .value::<i64>()
                    .expect("heap score elapsed should decode")
                    .expect("heap score elapsed should be non-null"),
                row["pq_fastscan_rerank_heap_rows_fetched"]
                    .value::<i32>()
                    .expect("heap rows fetched should decode")
                    .expect("heap rows fetched should be non-null"),
                row["pq_fastscan_rerank_heap_fetch_elapsed_us"]
                    .value::<i64>()
                    .expect("heap fetch elapsed should decode")
                    .expect("heap fetch elapsed should be non-null"),
                row["pq_fastscan_rerank_heap_decode_elapsed_us"]
                    .value::<i64>()
                    .expect("heap decode elapsed should decode")
                    .expect("heap decode elapsed should be non-null"),
                row["pq_fastscan_rerank_heap_dot_elapsed_us"]
                    .value::<i64>()
                    .expect("heap dot elapsed should decode")
                    .expect("heap dot elapsed should be non-null"),
            )
        });

        assert!(result_count > 0);
        assert!(
            pq_fastscan_rerank_heap_score_calls > 0
                && pq_fastscan_rerank_heap_score_elapsed_us >= 0
                && pq_fastscan_rerank_heap_rows_fetched >= pq_fastscan_rerank_heap_score_calls
                && pq_fastscan_rerank_heap_fetch_elapsed_us >= 0
                && pq_fastscan_rerank_heap_decode_elapsed_us >= 0
                && pq_fastscan_rerank_heap_dot_elapsed_us >= 0,
            "canonical pq fastscan rerank profile should surface heap rerank work on the source-backed default path",
        );
        assert_eq!(
            (
                pq_fastscan_rerank_quantized_score_calls,
                pq_fastscan_rerank_quantized_score_elapsed_us,
            ),
            (0, 0),
            "canonical pq fastscan rerank profile should bypass quantized rerank counters on the source-backed default path",
        );
    }

    #[pg_test]
    fn test_ech_debug_turboquant_scan_stage_profile_sql_surface() {
        let index_name = "ec_hnsw_turboquant_scan_stage_profile_sql_surface_idx";
        let _index_oid = create_turboquant_binary_runtime_fixture(
            "ec_hnsw_turboquant_scan_stage_profile_sql_surface",
            index_name,
        );
        let query_literal = format_recall_vector_sql_literal(&pq_fastscan_binary_runtime_query());
        let (
            rescan_amrescan_total_elapsed_us,
            turboquant_traversal_residual_elapsed_us,
            turboquant_binary_prefilter_score_calls,
            turboquant_binary_prefilter_score_elapsed_us,
            turboquant_binary_prefilter_survivor_candidates,
            turboquant_exact_score_calls,
            turboquant_exact_score_elapsed_us,
            turboquant_rerank_score_calls,
            turboquant_rerank_score_elapsed_us,
            turboquant_exact_score_mode,
            turboquant_exact_score_uses_lut,
            turboquant_exact_score_uses_qjl,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            rescan_amrescan_total_elapsed_us,
                            turboquant_traversal_residual_elapsed_us,
                            turboquant_binary_prefilter_score_calls,
                            turboquant_binary_prefilter_score_elapsed_us,
                            turboquant_binary_prefilter_survivor_candidates,
                            turboquant_exact_score_calls,
                            turboquant_exact_score_elapsed_us,
                            turboquant_rerank_score_calls,
                            turboquant_rerank_score_elapsed_us,
                            turboquant_exact_score_mode,
                            turboquant_exact_score_uses_lut,
                            turboquant_exact_score_uses_qjl
                         FROM tests.ec_hnsw_debug_turboquant_scan_stage_profile(
                            '{index_name}'::regclass::oid,
                            {query_literal}
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("turboquant scan stage profile query should succeed")
                .next()
                .expect("turboquant scan stage profile should return one row");
            (
                row["rescan_amrescan_total_elapsed_us"]
                    .value::<i64>()
                    .expect("rescan elapsed should decode")
                    .expect("rescan elapsed should be non-null"),
                row["turboquant_traversal_residual_elapsed_us"]
                    .value::<i64>()
                    .expect("traversal residual should decode")
                    .expect("traversal residual should be non-null"),
                row["turboquant_binary_prefilter_score_calls"]
                    .value::<i32>()
                    .expect("binary prefilter calls should decode")
                    .expect("binary prefilter calls should be non-null"),
                row["turboquant_binary_prefilter_score_elapsed_us"]
                    .value::<i64>()
                    .expect("binary prefilter elapsed should decode")
                    .expect("binary prefilter elapsed should be non-null"),
                row["turboquant_binary_prefilter_survivor_candidates"]
                    .value::<i32>()
                    .expect("binary prefilter survivors should decode")
                    .expect("binary prefilter survivors should be non-null"),
                row["turboquant_exact_score_calls"]
                    .value::<i32>()
                    .expect("exact score calls should decode")
                    .expect("exact score calls should be non-null"),
                row["turboquant_exact_score_elapsed_us"]
                    .value::<i64>()
                    .expect("exact score elapsed should decode")
                    .expect("exact score elapsed should be non-null"),
                row["turboquant_rerank_score_calls"]
                    .value::<i32>()
                    .expect("rerank calls should decode")
                    .expect("rerank calls should be non-null"),
                row["turboquant_rerank_score_elapsed_us"]
                    .value::<i64>()
                    .expect("rerank elapsed should decode")
                    .expect("rerank elapsed should be non-null"),
                row["turboquant_exact_score_mode"]
                    .value::<String>()
                    .expect("exact score mode should decode")
                    .expect("exact score mode should be non-null"),
                row["turboquant_exact_score_uses_lut"]
                    .value::<bool>()
                    .expect("exact score uses lut should decode")
                    .expect("exact score uses lut should be non-null"),
                row["turboquant_exact_score_uses_qjl"]
                    .value::<bool>()
                    .expect("exact score uses qjl should decode")
                    .expect("exact score uses qjl should be non-null"),
            )
        });

        assert!(rescan_amrescan_total_elapsed_us >= 0);
        assert!(turboquant_traversal_residual_elapsed_us >= 0);
        assert!(
            turboquant_binary_prefilter_score_calls > 0
                && turboquant_binary_prefilter_score_elapsed_us >= 0
                && turboquant_binary_prefilter_survivor_candidates > 0
                && turboquant_binary_prefilter_survivor_candidates
                    <= turboquant_binary_prefilter_score_calls,
            "turboquant scan stage profile should surface binary-prefilter work on the no-QJL 4-bit lane",
        );
        assert!(
            turboquant_exact_score_calls >= 0 && turboquant_exact_score_elapsed_us >= 0,
            "turboquant scan stage profile should keep exact-score counters well-formed even after deferring most scalar rescoring out of traversal",
        );
        assert!(
            turboquant_rerank_score_calls > 0 && turboquant_rerank_score_elapsed_us >= 0,
            "turboquant scan stage profile should surface deferred rerank work once traversal stops exact-scoring every binary-prefilter survivor",
        );
        assert!(
            turboquant_exact_score_calls < turboquant_binary_prefilter_survivor_candidates,
            "turboquant scan stage profile should exact-score fewer candidates than the binary-prefilter survivor set once rerank owns the deferred comparison pass",
        );
        assert_eq!(
            turboquant_exact_score_mode, "mse_no_qjl_4bit",
            "1536-dim 4-bit turboquant should report the tiled no-QJL exact-score lane",
        );
        assert!(
            !turboquant_exact_score_uses_lut && !turboquant_exact_score_uses_qjl,
            "the serious turboquant lane should not claim LUT or QJL exact work once the no-QJL path is active",
        );
    }

    fn assert_turboquant_scan_stage_profile_mode(
        env_value: &str,
        expected_mode: &str,
        expected_uses_lut: bool,
        expected_uses_qjl: bool,
    ) {
        let _lock = env_var_test_lock();
        let _score_mode_guard =
            ScopedEnvVar::set("TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE", env_value);
        let index_name = "ec_hnsw_turboquant_scan_stage_profile_sql_surface_int8_idx";
        let _index_oid = create_turboquant_binary_runtime_fixture(
            "ec_hnsw_turboquant_scan_stage_profile_sql_surface_int8",
            index_name,
        );
        let query_literal = format_recall_vector_sql_literal(&pq_fastscan_binary_runtime_query());
        let (
            turboquant_binary_prefilter_score_calls,
            turboquant_binary_prefilter_survivor_candidates,
            turboquant_exact_score_calls,
            turboquant_exact_score_elapsed_us,
            turboquant_rerank_score_calls,
            turboquant_rerank_score_elapsed_us,
            turboquant_exact_score_mode,
            turboquant_exact_score_uses_lut,
            turboquant_exact_score_uses_qjl,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            turboquant_binary_prefilter_score_calls,
                            turboquant_binary_prefilter_survivor_candidates,
                            turboquant_exact_score_calls,
                            turboquant_exact_score_elapsed_us,
                            turboquant_rerank_score_calls,
                            turboquant_rerank_score_elapsed_us,
                            turboquant_exact_score_mode,
                            turboquant_exact_score_uses_lut,
                            turboquant_exact_score_uses_qjl
                         FROM tests.ec_hnsw_debug_turboquant_scan_stage_profile(
                            '{index_name}'::regclass::oid,
                            {query_literal}
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("turboquant scan stage profile query should succeed")
                .next()
                .expect("turboquant scan stage profile should return one row");
            (
                row["turboquant_binary_prefilter_score_calls"]
                    .value::<i32>()
                    .expect("binary prefilter calls should decode")
                    .expect("binary prefilter calls should be non-null"),
                row["turboquant_binary_prefilter_survivor_candidates"]
                    .value::<i32>()
                    .expect("binary prefilter survivors should decode")
                    .expect("binary prefilter survivors should be non-null"),
                row["turboquant_exact_score_calls"]
                    .value::<i32>()
                    .expect("exact score calls should decode")
                    .expect("exact score calls should be non-null"),
                row["turboquant_exact_score_elapsed_us"]
                    .value::<i64>()
                    .expect("exact score elapsed should decode")
                    .expect("exact score elapsed should be non-null"),
                row["turboquant_rerank_score_calls"]
                    .value::<i32>()
                    .expect("rerank calls should decode")
                    .expect("rerank calls should be non-null"),
                row["turboquant_rerank_score_elapsed_us"]
                    .value::<i64>()
                    .expect("rerank elapsed should decode")
                    .expect("rerank elapsed should be non-null"),
                row["turboquant_exact_score_mode"]
                    .value::<String>()
                    .expect("exact score mode should decode")
                    .expect("exact score mode should be non-null"),
                row["turboquant_exact_score_uses_lut"]
                    .value::<bool>()
                    .expect("exact score uses lut should decode")
                    .expect("exact score uses lut should be non-null"),
                row["turboquant_exact_score_uses_qjl"]
                    .value::<bool>()
                    .expect("exact score uses qjl should decode")
                    .expect("exact score uses qjl should be non-null"),
            )
        });

        assert!(
            turboquant_binary_prefilter_score_calls > 0
                && turboquant_binary_prefilter_survivor_candidates > 0
                && turboquant_binary_prefilter_survivor_candidates
                    <= turboquant_binary_prefilter_score_calls,
            "turboquant int8 exact-score mode should leave the binary prefilter active",
        );
        assert!(
            turboquant_exact_score_calls >= 0 && turboquant_exact_score_elapsed_us >= 0,
            "turboquant int8 exact-score mode should keep exact-score counters well-formed",
        );
        assert!(
            turboquant_rerank_score_calls > 0 && turboquant_rerank_score_elapsed_us >= 0,
            "non-default turboquant exact-score modes should still surface deferred rerank work",
        );
        assert_eq!(
            turboquant_exact_score_mode, expected_mode,
            "the stage profile should expose the requested opt-in turboquant exact-score experiment",
        );
        assert!(
            turboquant_exact_score_uses_lut == expected_uses_lut
                && turboquant_exact_score_uses_qjl == expected_uses_qjl,
            "the stage profile should report the expected LUT/QJL shape for the requested turboquant exact-score mode",
        );
    }

    #[pg_test]
    fn test_turboquant_scan_stage_profile_full_lut_mode() {
        assert_turboquant_scan_stage_profile_mode("full_lut", "full_lut_no_qjl_4bit", true, false);
    }

    #[pg_test]
    fn test_turboquant_scan_stage_profile_tiled_lut_mode() {
        assert_turboquant_scan_stage_profile_mode(
            "tiled_lut",
            "tiled_lut_no_qjl_4bit",
            true,
            false,
        );
    }

    #[pg_test]
    fn test_turboquant_scan_stage_profile_int8_mode() {
        assert_turboquant_scan_stage_profile_mode(
            "int8_approx",
            "int8_approx_no_qjl_4bit",
            false,
            false,
        );
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw TurboQuant exact score mode must be one of [exact, full_lut, tiled_lut, int8_approx], got \"bogus\""
    )]
    fn test_turboquant_exact_score_mode_rejects_invalid_env() {
        let _lock = env_var_test_lock();
        let _score_mode_guard = ScopedEnvVar::set("TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE", "bogus");
        let index_oid = create_turboquant_binary_runtime_fixture(
            "ec_hnsw_turboquant_invalid_exact_score_mode",
            "ec_hnsw_turboquant_invalid_exact_score_mode_idx",
        );

        let _ = unsafe {
            am::debug_profile_ordered_scan(index_oid, pq_fastscan_binary_runtime_query())
        };
    }

    #[pg_test]
    fn test_pq_fastscan_runtime_profile_budgeted_exact_counters() {
        let _lock = env_var_test_lock();
        let _exact_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL", "1");
        let _limit_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT", "1");
        let index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_runtime_profile_budgeted_exact",
            "ec_hnsw_pq_fastscan_runtime_profile_budgeted_exact_idx",
        );
        let (
            _rescan_elapsed_us,
            _emit_elapsed_us,
            _total_elapsed_us,
            _rescan_phase,
            _rescan_current_result,
            _rescan_ordered_slots,
            _rescan_pending_heap_tids,
            _rescan_visited_elements,
            _rescan_expanded_sources,
            _rescan_emitted_elements,
            _rescan_bootstrap_expansions,
            _rescan_bootstrap_pages_read,
            _rescan_quantizer_cache_hit,
            _result_count,
            _final_phase,
            _final_ordered_slots,
            _total_bootstrap_expansions,
            _total_bootstrap_pages_read,
            _total_linear_pages_read,
            _total_elements_scored,
            _total_elements_skipped,
            _total_heap_tids_returned,
            _total_quantizer_cache_hit,
            _total_emitted_elements,
            _rescan_amrescan_total_elapsed_us,
            _rescan_query_decode_elapsed_us,
            _rescan_scan_setup_elapsed_us,
            _rescan_store_query_elapsed_us,
            _rescan_prepare_query_elapsed_us,
            _rescan_reset_state_elapsed_us,
            _rescan_initialize_entry_elapsed_us,
            _rescan_upper_layer_seed_elapsed_us,
            _rescan_layer0_seed_elapsed_us,
            _rescan_stage_ordered_results_elapsed_us,
            _rescan_initial_prefetch_elapsed_us,
            _rescan_frontier_consume_elapsed_us,
            _rescan_graph_result_materialize_elapsed_us,
            _graph_element_cache_hits,
            _graph_element_cache_misses,
            _graph_element_load_elapsed_us,
            _graph_neighbor_cache_hits,
            _graph_neighbor_cache_misses,
            _graph_neighbor_load_elapsed_us,
            _candidate_score_calls,
            _candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, pq_fastscan_runtime_query()) };

        assert!(
            grouped_traversal_approx_score_calls > 0
                && grouped_traversal_approx_score_elapsed_us >= 0,
            "budgeted grouped exact traversal should still score grouped approximate candidates first",
        );
        assert!(
            grouped_traversal_exact_score_calls > 0
                && grouped_traversal_exact_score_elapsed_us >= 0,
            "budgeted grouped exact traversal should surface exact rescoring work",
        );
        assert!(
            score_cache_hits > 0 && score_cache_misses > 0,
            "budgeted grouped exact traversal should reuse cached exact scores after the first miss path",
        );
        assert!(
            grouped_traversal_budgeted_expansions > 0
                && grouped_traversal_budgeted_candidates
                    >= grouped_traversal_budgeted_exact_candidates,
            "budgeted grouped exact traversal should report the candidate sets it exact-rescored",
        );
        assert!(
            grouped_traversal_exact_score_calls >= grouped_traversal_budgeted_exact_candidates,
            "grouped exact traversal should include at least the budgeted exact rescoring calls, even if entry or seed scoring adds more",
        );
        assert_eq!(
            grouped_traversal_budgeted_expansions, grouped_traversal_budgeted_exact_candidates,
            "limit=1 should exact-rescore one grouped candidate per budgeted expansion",
        );
    }

    #[pg_test]
    fn test_ech_debug_pq_fastscan_scan_hot_path_profile_sql_surface() {
        let _lock = env_var_test_lock();
        let _exact_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL", "1");
        let _limit_guard = ScopedEnvVar::set("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT", "1");
        let index_name = "ec_hnsw_pq_fastscan_hot_path_profile_sql_surface_idx";
        let _index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_pq_fastscan_hot_path_profile_sql_surface",
            index_name,
        );
        let query_literal = format_recall_vector_sql_literal(&pq_fastscan_runtime_query());
        let (
            score_cache_hits,
            score_cache_misses,
            pq_fastscan_traversal_approx_score_calls,
            pq_fastscan_traversal_approx_score_elapsed_us,
            pq_fastscan_traversal_exact_score_calls,
            pq_fastscan_traversal_exact_score_elapsed_us,
            pq_fastscan_traversal_budgeted_expansions,
            pq_fastscan_traversal_budgeted_candidates,
            pq_fastscan_traversal_budgeted_exact_candidates,
        ) = Spi::connect(|client| {
            let row = client
                .select(
                    &format!(
                        "SELECT
                            score_cache_hits,
                            score_cache_misses,
                            pq_fastscan_traversal_approx_score_calls,
                            pq_fastscan_traversal_approx_score_elapsed_us,
                            pq_fastscan_traversal_exact_score_calls,
                            pq_fastscan_traversal_exact_score_elapsed_us,
                            pq_fastscan_traversal_budgeted_expansions,
                            pq_fastscan_traversal_budgeted_candidates,
                            pq_fastscan_traversal_budgeted_exact_candidates
                         FROM tests.ec_hnsw_debug_pq_fastscan_scan_hot_path_profile(
                            '{index_name}'::regclass::oid,
                            {query_literal}
                         )"
                    ),
                    None,
                    &[],
                )
                .expect("pq fastscan hot path profile query should succeed")
                .next()
                .expect("pq fastscan hot path profile should return one row");
            (
                row["score_cache_hits"]
                    .value::<i32>()
                    .expect("score cache hits should decode")
                    .expect("score cache hits should be non-null"),
                row["score_cache_misses"]
                    .value::<i32>()
                    .expect("score cache misses should decode")
                    .expect("score cache misses should be non-null"),
                row["pq_fastscan_traversal_approx_score_calls"]
                    .value::<i32>()
                    .expect("approx score call count should decode")
                    .expect("approx score call count should be non-null"),
                row["pq_fastscan_traversal_approx_score_elapsed_us"]
                    .value::<i64>()
                    .expect("approx score elapsed should decode")
                    .expect("approx score elapsed should be non-null"),
                row["pq_fastscan_traversal_exact_score_calls"]
                    .value::<i32>()
                    .expect("exact score call count should decode")
                    .expect("exact score call count should be non-null"),
                row["pq_fastscan_traversal_exact_score_elapsed_us"]
                    .value::<i64>()
                    .expect("exact score elapsed should decode")
                    .expect("exact score elapsed should be non-null"),
                row["pq_fastscan_traversal_budgeted_expansions"]
                    .value::<i32>()
                    .expect("budgeted expansion count should decode")
                    .expect("budgeted expansion count should be non-null"),
                row["pq_fastscan_traversal_budgeted_candidates"]
                    .value::<i32>()
                    .expect("budgeted candidate count should decode")
                    .expect("budgeted candidate count should be non-null"),
                row["pq_fastscan_traversal_budgeted_exact_candidates"]
                    .value::<i32>()
                    .expect("budgeted exact candidate count should decode")
                    .expect("budgeted exact candidate count should be non-null"),
            )
        });

        assert!(
            pq_fastscan_traversal_approx_score_calls > 0
                && pq_fastscan_traversal_approx_score_elapsed_us >= 0,
            "canonical pq fastscan hot path profile should surface approximate traversal scoring",
        );
        assert!(
            pq_fastscan_traversal_exact_score_calls > 0
                && pq_fastscan_traversal_exact_score_elapsed_us >= 0,
            "canonical pq fastscan hot path profile should surface exact traversal rescoring",
        );
        assert!(
            score_cache_hits >= 0 && score_cache_misses > 0,
            "canonical pq fastscan hot path profile should surface exact-score cache activity",
        );
        assert!(
            pq_fastscan_traversal_budgeted_expansions > 0
                && pq_fastscan_traversal_budgeted_candidates
                    >= pq_fastscan_traversal_budgeted_exact_candidates,
            "canonical pq fastscan hot path profile should report the budgeted exact traversal candidate sets",
        );
        assert!(
            pq_fastscan_traversal_exact_score_calls
                >= pq_fastscan_traversal_budgeted_exact_candidates,
            "canonical pq fastscan hot path profile should include at least the budgeted exact rescoring calls",
        );
    }

