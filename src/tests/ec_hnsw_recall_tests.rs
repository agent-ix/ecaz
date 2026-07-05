    #[pg_test]
    fn test_ech_graph_scan_recall_gate() {
        if std::env::var_os("TQVECTOR_RUN_RECALL_GATE").is_none() {
            return;
        }

        let results = run_graph_scan_recall_gate();
        let gate_recall = results
            .iter()
            .find(|(m, ef_search, _, _, _)| *m == 8 && *ef_search == 128)
            .map(|(_, _, recall, _, _)| *recall)
            .expect("A4 gate config should have been measured");

        assert!(
            gate_recall >= 0.89,
            "A4 recall gate failed: Recall@10 at m=8 ef=128 was {:.2}% (required >= 89%)",
            gate_recall * 100.0
        );
    }

    #[pg_test]
    #[ignore]
    fn test_ech_graph_scan_recall_fixture_summary_1k_tiled_fwht() {
        let fixture_name = "ec_hnsw_graph_scan_recall_tiled_1k";
        let index_blocks = reset_graph_scan_recall_fixture(fixture_name, 8, 1_000);
        let (
            _m,
            _ef_search,
            query_count,
            graph_recall_at_10,
            exact_quantized_recall_at_10,
            build_code_recall_at_10,
            graph_below_exact_queries,
            graph_below_build_code_queries,
            build_code_below_exact_queries,
            worst_exact_gap,
            worst_build_code_gap,
        ) = probe_graph_scan_recall_fixture_summary(fixture_name, 8, 128, 50);

        println!(
            "1k tiled fixture: blocks={index_blocks} queries={query_count} graph={graph_recall_at_10:.4} exact={exact_quantized_recall_at_10:.4} build_code={build_code_recall_at_10:.4} graph_below_exact={graph_below_exact_queries} graph_below_build_code={graph_below_build_code_queries} build_code_below_exact={build_code_below_exact_queries} worst_exact_gap={worst_exact_gap} worst_build_code_gap={worst_build_code_gap}"
        );

        assert!(
            exact_quantized_recall_at_10 >= 0.70,
            "expected tiled 1536 quantizer path to keep exact Recall@10 above 70% on the 1k fixture, got {:.2}%",
            exact_quantized_recall_at_10 * 100.0
        );
        assert!(
            graph_recall_at_10 >= 0.70,
            "expected live graph-first Recall@10 above 70% on the 1k tiled fixture, got {:.2}% (exact {:.2}%, build-code {:.2}%)",
            graph_recall_at_10 * 100.0,
            exact_quantized_recall_at_10 * 100.0,
            build_code_recall_at_10 * 100.0
        );
    }

    #[pg_test]
    #[ignore]
    fn test_ech_graph_scan_recall_fixture_gate_10k_tiled_fwht() {
        let fixture_prefix = "ec_hnsw_graph_scan_recall_gate_tiled_10k";

        let reset_started = Instant::now();
        let reset_rows = reset_graph_scan_recall_gate_fixtures(fixture_prefix, 10_000);
        let reset_elapsed = reset_started.elapsed();

        let first_started = Instant::now();
        let first = run_graph_scan_recall_gate_from_fixtures(fixture_prefix, 100);
        let first_elapsed = first_started.elapsed();

        let second_started = Instant::now();
        let second = run_graph_scan_recall_gate_from_fixtures(fixture_prefix, 100);
        let second_elapsed = second_started.elapsed();

        println!(
            "10k fixture gate reuse: reset={reset_elapsed:?} fixtures={reset_rows:?} first={first_elapsed:?} second={second_elapsed:?} results={first:?}"
        );

        assert_eq!(
            first, second,
            "fixture-backed gate report should be stable across reruns"
        );
    }

    #[pg_test]
    #[ignore]
    fn test_ech_graph_scan_recall_source_gate_10k() {
        let fixture_prefix = "ec_hnsw_graph_scan_recall_gate_source_10k";
        let query_count = 25;

        let reset_started = Instant::now();
        let reset_rows = reset_graph_scan_recall_gate_source_fixtures(fixture_prefix, 10_000);
        let reset_elapsed = reset_started.elapsed();

        let first_started = Instant::now();
        let first = run_graph_scan_recall_gate_from_fixtures(fixture_prefix, query_count);
        let first_elapsed = first_started.elapsed();

        let second_started = Instant::now();
        let second = run_graph_scan_recall_gate_from_fixtures(fixture_prefix, query_count);
        let second_elapsed = second_started.elapsed();

        println!(
            "10k source fixture gate reuse: reset={reset_elapsed:?} fixtures={reset_rows:?} queries={query_count} first={first_elapsed:?} second={second_elapsed:?} results={first:?}"
        );

        assert_eq!(
            first, second,
            "source-build fixture-backed gate report should be stable across reruns"
        );
    }

    fn external_recall_index_prefix(prefix: &str, storage_format: Option<&str>) -> String {
        match storage_format {
            Some(storage_format) => format!("{prefix}_{storage_format}"),
            None => prefix.to_string(),
        }
    }

    fn external_recall_index_name(prefix: &str, storage_format: Option<&str>, m: i32) -> String {
        format!(
            "{}_m{m}_idx",
            external_recall_index_prefix(prefix, storage_format)
        )
    }

    fn external_recall_storage_format_clause(storage_format: Option<&str>) -> String {
        match storage_format {
            Some(storage_format) => format!(", storage_format = '{storage_format}'"),
            None => String::new(),
        }
    }

    /// Helper that materializes the external corpus / query table layout
    /// described in `docs/RECALL_REAL_CORPUS.md` for a small synthetic dataset.
    /// Index families are created separately so the smoke test can attach
    /// multiple storage formats to the same staged tables.
    fn create_external_recall_smoke_tables(prefix: &str, corpus_size: usize, query_count: usize) {
        let corpus_table = format!("{prefix}_corpus");
        let queries_table = format!("{prefix}_queries");

        Spi::run(&format!("DROP TABLE IF EXISTS {corpus_table} CASCADE"))
            .expect("smoke fixture corpus drop should succeed");
        Spi::run(&format!("DROP TABLE IF EXISTS {queries_table} CASCADE"))
            .expect("smoke fixture queries drop should succeed");

        Spi::run(&format!(
            "CREATE TABLE {corpus_table} (
                id bigint primary key,
                source real[] NOT NULL,
                embedding ecvector
            )"
        ))
        .expect("smoke fixture corpus create should succeed");
        Spi::run(&format!(
            "CREATE TABLE {queries_table} (
                id bigint primary key,
                source real[] NOT NULL
            )"
        ))
        .expect("smoke fixture queries create should succeed");

        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);

        // Seed both tables with a single multi-row INSERT each instead of
        // one statement per row. pgrx 0.17 does not expose a stable
        // SPI-level COPY FROM STDIN API, so a batched INSERT is the fastest
        // transport available without adding a Postgres client crate. Row
        // order, ids, vector floats, and the encode call are preserved
        // byte-for-byte from the previous per-row path so the recall summary
        // remains deterministic.
        let corpus_values = corpus
            .iter()
            .enumerate()
            .map(|(id, vector)| {
                let source = format_recall_vector_sql_literal(vector);
                format!(
                    "({id}, {source}, encode_to_ecvector({source}, {RECALL_BITS}, {RECALL_SEED}))"
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        Spi::run(&format!(
            "INSERT INTO {corpus_table} (id, source, embedding) VALUES {corpus_values}"
        ))
        .expect("smoke fixture corpus batch insert should succeed");

        let query_values = queries
            .iter()
            .enumerate()
            .map(|(id, vector)| {
                let source = format_recall_vector_sql_literal(vector);
                format!("({id}, {source})")
            })
            .collect::<Vec<_>>()
            .join(", ");
        Spi::run(&format!(
            "INSERT INTO {queries_table} (id, source) VALUES {query_values}"
        ))
        .expect("smoke fixture query batch insert should succeed");
    }

    fn create_external_recall_smoke_indexes(prefix: &str, storage_format: Option<&str>) {
        let corpus_table = format!("{prefix}_corpus");
        let storage_format_clause = external_recall_storage_format_clause(storage_format);
        for m in [8_i32, 16_i32] {
            let index_name = external_recall_index_name(prefix, storage_format, m);
            Spi::run(&format!(
                "CREATE INDEX {index_name} ON {corpus_table} \
                 USING ec_hnsw (embedding ecvector_ip_ops) \
                 WITH (m = {m}, ef_construction = {RECALL_EF_CONSTRUCTION}, \
                       build_source_column = 'source'{storage_format_clause})"
            ))
            .expect("smoke fixture index create should succeed");
        }
    }

    fn assert_external_recall_smoke_probe(prefix: &str, storage_format: Option<&str>) {
        let corpus_table = format!("{prefix}_corpus");
        let queries_table = format!("{prefix}_queries");
        let index_prefix = external_recall_index_prefix(prefix, storage_format);
        let m8_index = external_recall_index_name(prefix, storage_format, 8);
        let storage_label = storage_format.unwrap_or("default");

        let summary = probe_graph_scan_recall_external_summary_for_relation(
            &corpus_table,
            &queries_table,
            &m8_index,
            8,
            128,
        );
        let (
            m,
            ef_search,
            corpus_rows,
            query_count,
            graph_recall_at_10,
            graph_recall_at_100,
            ndcg_at_10,
            mean_abs_score_error,
            spearman_rho_at_10,
            exact_quantized_recall_at_10,
            graph_below_exact_queries,
            worst_exact_gap,
        ) = summary;

        println!(
            "external smoke 500 ({storage_label}): m={m} ef={ef_search} corpus={corpus_rows} queries={query_count} \
             graph@10={graph_recall_at_10:.4} graph@100={graph_recall_at_100:.4} \
             ndcg@10={ndcg_at_10:.4} mae={mean_abs_score_error:.6} \
             spearman={spearman_rho_at_10:.4} exact@10={exact_quantized_recall_at_10:.4} \
             graph_below_exact={graph_below_exact_queries} worst_gap={worst_exact_gap}"
        );

        assert_eq!(m, 8);
        assert_eq!(ef_search, 128);
        assert_eq!(corpus_rows, 500);
        assert_eq!(query_count, 25);
        assert!((0.0..=1.0).contains(&graph_recall_at_10));
        assert!((0.0..=1.0).contains(&graph_recall_at_100));
        assert!((0.0..=1.0).contains(&exact_quantized_recall_at_10));
        assert!((-1.0..=1.0).contains(&spearman_rho_at_10));
        assert!(ndcg_at_10 >= 0.0);
        assert!(mean_abs_score_error >= 0.0);

        let summary_two = probe_graph_scan_recall_external_summary_for_relation(
            &corpus_table,
            &queries_table,
            &m8_index,
            8,
            128,
        );
        assert_eq!(
            summary, summary_two,
            "external recall summary should be deterministic across reruns for {storage_label}"
        );

        let gate =
            run_graph_scan_recall_gate_from_external(&corpus_table, &queries_table, &index_prefix);
        assert_eq!(
            gate.len(),
            RECALL_GATE_CONFIGS.len(),
            "gate report should emit one row per A4 config for {storage_label}"
        );
        for ((m, ef_search, recall, target, passed), expected) in
            gate.iter().zip(RECALL_GATE_CONFIGS.iter())
        {
            assert_eq!(*m, expected.0);
            assert_eq!(*ef_search, expected.1);
            assert_eq!(*target, expected.2);
            assert!((0.0..=1.0).contains(recall));
            if expected.2.is_none() {
                assert!(*passed);
            }
        }
    }

    #[pg_test]
    // Ignored because it requires the `pg_test` cargo feature and a scratch
    // pgrx test cluster to run, not because of long seeding. Seeding is
    // batched in `create_external_recall_smoke_tables`; the remaining
    // wall-clock cost lives in the probe / gate phases below.
    #[ignore]
    fn test_ech_recall_external_smoke_500_formats() {
        // Smoke test for the external corpus / query / index probe path. The
        // real DBpedia corpus is staged out-of-band by
        // `scripts/load_real_corpus.py`; here we substitute a tiny synthetic
        // dataset that the loader's schema accepts so we can exercise the
        // Rust probe surface end-to-end, including coexisting explicit
        // storage-format index families on one shared corpus/query table pair.
        let prefix = "ec_hnsw_recall_external_smoke";
        create_external_recall_smoke_tables(prefix, 500, 25);
        create_external_recall_smoke_indexes(prefix, None);
        create_external_recall_smoke_indexes(prefix, Some("turboquant"));
        create_external_recall_smoke_indexes(prefix, Some("pq_fastscan"));

        assert_external_recall_smoke_probe(prefix, None);
        assert_external_recall_smoke_probe(prefix, Some("turboquant"));
        assert_external_recall_smoke_probe(prefix, Some("pq_fastscan"));
    }

    #[pg_test]
    fn test_ech_external_summary_exact_baseline_multiidx() {
        let prefix = "ec_hnsw_recall_external_summary_multiidx";
        create_external_recall_smoke_tables(prefix, 64, 8);
        create_external_recall_smoke_indexes(prefix, None);
        create_external_recall_smoke_indexes(prefix, Some("turboquant"));
        create_external_recall_smoke_indexes(prefix, Some("pq_fastscan"));

        let corpus_table = format!("{prefix}_corpus");
        let queries_table = format!("{prefix}_queries");
        let turboquant_m8_index = external_recall_index_name(prefix, Some("turboquant"), 8);

        let summary = probe_graph_scan_recall_external_summary_for_relation(
            &corpus_table,
            &queries_table,
            &turboquant_m8_index,
            8,
            128,
        );

        assert_eq!(summary.0, 8);
        assert_eq!(summary.1, 128);
        assert_eq!(summary.2, 64);
        assert_eq!(summary.3, 8);
        assert!((0.0..=1.0).contains(&summary.9));
    }

