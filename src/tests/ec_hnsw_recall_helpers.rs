    fn run_graph_scan_recall_gate() -> Vec<(i32, i32, f32, Option<f32>, bool)> {
        let corpus = random_unit_vectors(RECALL_CORPUS_SIZE, RECALL_DIM, RECALL_SEED as u64);
        let queries = random_unit_vectors(
            RECALL_QUERY_COUNT,
            RECALL_DIM,
            (RECALL_SEED as u64) + 1_000_000,
        );
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();

        create_recall_table("ec_hnsw_graph_scan_recall_gate");
        insert_recall_corpus("ec_hnsw_graph_scan_recall_gate", &corpus);
        let ctid_to_id = ctid_id_map("ec_hnsw_graph_scan_recall_gate");

        let mut results = Vec::new();

        for m in [8, 16] {
            let index_name = format!("ec_hnsw_graph_scan_recall_gate_m{m}_idx");
            let index_oid = create_recall_index("ec_hnsw_graph_scan_recall_gate", &index_name, m);

            for (config_m, ef_search, target) in RECALL_GATE_CONFIGS
                .iter()
                .copied()
                .filter(|(cfg_m, _, _)| *cfg_m == m)
            {
                let recall = measure_graph_scan_recall(
                    index_oid,
                    &ctid_to_id,
                    &queries,
                    &ground_truth,
                    ef_search,
                );
                let passed = target.map(|gate| recall >= gate).unwrap_or(true);
                results.push((config_m, ef_search, recall, target, passed));
            }

            Spi::run(&format!("DROP INDEX {index_name}"))
                .expect("recall benchmark index cleanup should succeed");
        }

        Spi::run("DROP TABLE ec_hnsw_graph_scan_recall_gate")
            .expect("recall benchmark table cleanup should succeed");

        results
    }

    fn run_graph_scan_recall_gate_from_fixtures(
        fixture_prefix: &str,
        query_count: usize,
    ) -> Vec<(i32, i32, f32, Option<f32>, bool)> {
        assert!(query_count > 0, "query_count must be positive");

        let fixture_prefix = recall_fixture_ident(fixture_prefix);
        let table_name = format!("{fixture_prefix}_corpus");
        RECALL_GATE_CONFIGS
            .iter()
            .copied()
            .map(|(m, ef_search, target)| {
                let index_name = format!("{fixture_prefix}_m{m}_idx");
                let (_, _, _, graph_recall_at_10, _, _, _, _, _, _, _) =
                    probe_graph_scan_recall_fixture_summary_for_relation(
                        &table_name,
                        &index_name,
                        m,
                        ef_search,
                        query_count,
                    );
                let passed = target
                    .map(|gate| graph_recall_at_10 >= gate)
                    .unwrap_or(true);
                (m, ef_search, graph_recall_at_10, target, passed)
            })
            .collect()
    }

    type GraphScanRecallProbeRow = (i32, i32, i32, i32, bool, Vec<i64>, Vec<i64>, Vec<i64>);
    type GraphScanRecallFrontierTranscriptRow = (
        i32,
        i32,
        i32,
        i32,
        bool,
        i32,
        i32,
        Vec<i64>,
        Vec<i64>,
        Vec<i64>,
        Option<String>,
        Vec<String>,
        Vec<String>,
    );
    type GraphScanRecallProbeRanksRow = (
        i32,
        i32,
        i32,
        i32,
        bool,
        Vec<i64>,
        Vec<i64>,
        Vec<i64>,
        Vec<i32>,
        Vec<i32>,
    );
    type GraphScanRecallScoreAuditRow = (i32, i32, Vec<i64>, Vec<f32>, Vec<i32>, Vec<f32>);
    type GraphScanRecallFixtureQueryOverlapRow = (i32, i32, i32, i32, i32, i32, i32);
    type GraphScanRecallFixtureSummaryRow = (i32, i32, i32, f32, f32, f32, i32, i32, i32, i32, i32);
    type GraphScanRecallExternalSummaryRow = (
        i32, // m
        i32, // ef_search
        i32, // corpus_rows
        i32, // query_count
        f32, // graph_recall_at_10
        f32, // graph_recall_at_100
        f32, // ndcg_at_10
        f32, // mean_abs_score_error
        f32, // spearman_rho_at_10
        f32, // exact_quantized_recall_at_10
        i32, // graph_below_exact_queries
        i32, // worst_exact_gap
    );
    type GraphScanRecallAnnBenchmarksReferenceRow = (
        i32,  // m
        i32,  // ef_search
        f32,  // recall_at_10
        f32,  // published_recall_at_10
        f32,  // absolute_delta
        bool, // within_two_percent
    );
    type GraphScanRecallTopLevelOracleSummaryRow = (i32, i32, i32, f32, f32, f32, i32, i32, i32);
    type GraphScanRecallTopLevelOracleKSummaryRow =
        (i32, i32, i32, i32, f32, f32, f32, i32, i32, i32);
    type GraphScanRecallLayerOracleKCarrydownSummaryRow =
        (i32, i32, i32, i32, i32, f32, f32, f32, i32, i32, i32);
    type GraphScanRecallLayerNeighborCoverageSummaryRow =
        (i32, i32, i32, i32, i32, f32, f32, f32, i32, i32, i32);
    type GraphScanRecallTopLevelSeedCoverageRow = (
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        f32,
        i32,
        Vec<i64>,
        Vec<i32>,
    );
    type GraphScanRecallExactSeedSummaryRow = (i32, i32, i32, f32, f32, f32, f32, i32, i32, i32);
    type GraphScanRecallHistogramRow = (
        i32, // recall_bucket (0..=10)
        i32, // query_count
        f32, // query_fraction
    );
    type GraphScanRecallEfSweepRow = (
        i32, // m
        i32, // ef_search
        f32, // recall_at_10
        f32, // exact_quantized_recall_at_10
        f32, // mean_abs_score_error
        f32, // mean_query_latency_ms
    );
    type GraphScanRecallFailureBreakdownRow = (
        i32,      // query_index
        i32,      // graph_recall_at_10
        i32,      // exact_quantized_recall_at_10
        Vec<i64>, // missed_ids
    );

    fn recall_top_k_overlap(left: &[i64], right: &[i64]) -> i32 {
        i32::try_from(left.iter().filter(|id| right.contains(id)).count())
            .expect("top-k overlap should fit into int")
    }

    fn format_heap_tid_coords((block_number, offset_number): (u32, u16)) -> String {
        format!("{block_number}:{offset_number}")
    }

    fn format_frontier_provenance_slot(
        (valid, node, source, score): (bool, (u32, u16), (u32, u16), f32),
    ) -> Option<String> {
        if !valid {
            return None;
        }

        let source = if source == (u32::MAX, u16::MAX) {
            "-".to_owned()
        } else {
            format_heap_tid_coords(source)
        };
        Some(format!(
            "{}<-{}@{score:.6}",
            format_heap_tid_coords(node),
            source
        ))
    }

    fn probe_graph_scan_recall_fixture(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_index: usize,
        query_count: usize,
    ) -> GraphScanRecallProbeRow {
        assert!(query_count > query_index);

        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {fixture_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let query = queries
            .get(query_index)
            .expect("query index should be within the generated query set");
        let ctid_to_id = ctid_id_map(&fixture_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");
        let index_block_count =
            recall_index_block_count(index_oid, "build_graph_scan_recall_probe_with_sizes");
        let truth = brute_force_top_k(
            &random_unit_vectors(
                usize::try_from(corpus).expect("fixture corpus size should fit usize"),
                RECALL_DIM,
                RECALL_SEED as u64,
            ),
            query,
            RECALL_K,
        )
        .into_iter()
        .map(|id| i64::try_from(id).expect("truth id should fit into bigint"))
        .collect::<Vec<_>>();

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let (prefill_found, _, _, _, _, _, _, _) =
            unsafe { am::debug_gettuple_current_result_state(index_oid, query.clone()) };
        let predicted_heap_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) };
        let predicted_ids = predicted_heap_tids
            .iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(heap_tid)
                        .expect("probe heap tid should map back to a benchmark row id"),
                )
                .expect("predicted id should fit into bigint")
            })
            .collect::<Vec<_>>();
        let exact_quantized_ids = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT id
                         FROM {fixture_name}
                         ORDER BY embedding <#> $1
                         LIMIT 10"
                    ),
                    None,
                    &[query.clone().into()],
                )
                .expect("exact quantized probe query should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        (
            m,
            ef_search,
            index_block_count,
            i32::try_from(predicted_heap_tids.len()).expect("row count should fit into int"),
            prefill_found,
            truth,
            predicted_ids,
            exact_quantized_ids,
        )
    }

    fn probe_graph_scan_recall_fixture_transcript(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_index: usize,
        query_count: usize,
    ) -> GraphScanRecallFrontierTranscriptRow {
        let probe =
            probe_graph_scan_recall_fixture(fixture_name, m, ef_search, query_index, query_count);
        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");
        let query = random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000)
            .into_iter()
            .nth(query_index)
            .expect("query index should be within the generated query set");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let (frontier_head, _, _, frontier_provenance, expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, query) };

        (
            probe.0,
            probe.1,
            probe.2,
            probe.3,
            probe.4,
            recall_top_k_overlap(&probe.5, &probe.6),
            recall_top_k_overlap(&probe.5, &probe.7),
            probe.5,
            probe.6,
            probe.7,
            frontier_head.map(format_heap_tid_coords),
            frontier_provenance
                .into_iter()
                .filter_map(format_frontier_provenance_slot)
                .collect::<Vec<_>>(),
            expanded_sources
                .into_iter()
                .map(format_heap_tid_coords)
                .collect::<Vec<_>>(),
        )
    }

    fn probe_graph_scan_recall_fixture_ranks(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_index: usize,
        query_count: usize,
    ) -> GraphScanRecallProbeRanksRow {
        assert!(query_count > query_index);

        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {fixture_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let query = queries
            .get(query_index)
            .expect("query index should be within the generated query set");
        let ctid_to_id = ctid_id_map(&fixture_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");
        let index_block_count =
            recall_index_block_count(index_oid, "probe_graph_scan_recall_fixture_ranks");
        let truth = brute_force_top_k(
            &random_unit_vectors(
                usize::try_from(corpus).expect("fixture corpus size should fit usize"),
                RECALL_DIM,
                RECALL_SEED as u64,
            ),
            query,
            RECALL_K,
        )
        .into_iter()
        .map(|id| i64::try_from(id).expect("truth id should fit into bigint"))
        .collect::<Vec<_>>();

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let (prefill_found, _, _, _, _, _, _, _) =
            unsafe { am::debug_gettuple_current_result_state(index_oid, query.clone()) };
        let predicted_heap_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) };
        let predicted_ids_full = predicted_heap_tids
            .iter()
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(heap_tid)
                        .expect("probe heap tid should map back to a benchmark row id"),
                )
                .expect("predicted id should fit into bigint")
            })
            .collect::<Vec<_>>();
        let predicted_top10_ids = predicted_ids_full
            .iter()
            .copied()
            .take(RECALL_K)
            .collect::<Vec<_>>();
        let exact_quantized_ids = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT id
                         FROM {fixture_name}
                         ORDER BY embedding <#> $1
                         LIMIT 10"
                    ),
                    None,
                    &[query.clone().into()],
                )
                .expect("exact quantized probe query should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        let truth_ranks = truth
            .iter()
            .map(|id| {
                predicted_ids_full
                    .iter()
                    .position(|candidate| candidate == id)
                    .map(|rank| i32::try_from(rank).expect("rank should fit into int"))
                    .unwrap_or(-1)
            })
            .collect::<Vec<_>>();
        let exact_ranks = exact_quantized_ids
            .iter()
            .map(|id| {
                predicted_ids_full
                    .iter()
                    .position(|candidate| candidate == id)
                    .map(|rank| i32::try_from(rank).expect("rank should fit into int"))
                    .unwrap_or(-1)
            })
            .collect::<Vec<_>>();

        (
            m,
            ef_search,
            index_block_count,
            i32::try_from(predicted_ids_full.len()).expect("row count should fit into int"),
            prefill_found,
            truth,
            predicted_top10_ids,
            exact_quantized_ids,
            truth_ranks,
            exact_ranks,
        )
    }

    fn probe_graph_scan_recall_fixture_score_audit(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_index: usize,
        query_count: usize,
    ) -> GraphScanRecallScoreAuditRow {
        assert!(query_count > query_index);

        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let query = queries
            .get(query_index)
            .expect("query index should be within the generated query set")
            .clone();
        let ctid_to_id = ctid_id_map(&fixture_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let predicted_with_scores =
            unsafe { am::debug_gettuple_scan_heap_tids_with_scores(index_oid, query.clone()) };
        let predicted_id_scores = predicted_with_scores
            .into_iter()
            .map(|(heap_tid, score)| {
                (
                    i64::try_from(
                        *ctid_to_id
                            .get(&heap_tid)
                            .expect("probe heap tid should map back to a benchmark row id"),
                    )
                    .expect("predicted id should fit into bigint"),
                    score,
                )
            })
            .collect::<Vec<_>>();

        let exact_ids = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT id
                         FROM {fixture_name}
                         ORDER BY embedding <#> $1
                         LIMIT 10"
                    ),
                    None,
                    &[query.clone().into()],
                )
                .expect("exact quantized probe query should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        let exact_scores = exact_ids
            .iter()
            .map(|id| {
                Spi::get_one::<f32>(&format!(
                    "SELECT embedding <#> ARRAY[{}]::real[] FROM {fixture_name} WHERE id = {id}",
                    query
                        .iter()
                        .map(|value| value.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ))
                .expect("exact score query should succeed")
                .expect("exact score should exist")
            })
            .collect::<Vec<_>>();
        let emitted_ranks = exact_ids
            .iter()
            .map(|id| {
                predicted_id_scores
                    .iter()
                    .position(|(candidate_id, _)| candidate_id == id)
                    .map(|rank| i32::try_from(rank).expect("rank should fit into int"))
                    .unwrap_or(-1)
            })
            .collect::<Vec<_>>();
        let emitted_scores = exact_ids
            .iter()
            .map(|id| {
                predicted_id_scores
                    .iter()
                    .find_map(|(candidate_id, score)| (*candidate_id == *id).then_some(*score))
                    .unwrap_or(f32::NAN)
            })
            .collect::<Vec<_>>();

        (
            m,
            ef_search,
            exact_ids,
            exact_scores,
            emitted_ranks,
            emitted_scores,
        )
    }

    fn collect_graph_scan_recall_fixture_query_overlaps_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> Vec<GraphScanRecallFixtureQueryOverlapRow> {
        assert!(query_count > 0);

        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus_size = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus_size).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let corpus_codes = encode_recall_corpus_codes(&corpus);
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("recall fixture index oid query should succeed")
                .expect("recall fixture index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let mut rows = Vec::with_capacity(query_count);

        for (query_index, (query, truth)) in queries.iter().zip(ground_truth.iter()).enumerate() {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("probe heap tid should map back to a benchmark row id"),
                        )
                        .expect("predicted id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized probe query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });
            let build_code_ids = brute_force_top_k_code_inner_product(
                &corpus_codes,
                &encode_recall_query_code(query),
                RECALL_K,
            )
            .into_iter()
            .map(|id| i64::try_from(id).expect("build-code id should fit into bigint"))
            .collect::<Vec<_>>();

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);
            let build_code_overlap = recall_top_k_overlap(&truth_ids, &build_code_ids);

            rows.push((
                m,
                ef_search,
                i32::try_from(query_count).expect("query count should fit into int"),
                i32::try_from(query_index).expect("query index should fit into int"),
                graph_overlap,
                exact_overlap,
                build_code_overlap,
            ));
        }

        rows
    }

    fn collect_graph_scan_recall_fixture_query_overlaps(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> Vec<GraphScanRecallFixtureQueryOverlapRow> {
        let fixture_name = recall_fixture_ident(fixture_name);
        let index_name = format!("{fixture_name}_idx");
        collect_graph_scan_recall_fixture_query_overlaps_for_relation(
            &fixture_name,
            &index_name,
            m,
            ef_search,
            query_count,
        )
    }

    fn probe_graph_scan_recall_fixture_summary(
        fixture_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallFixtureSummaryRow {
        let rows = collect_graph_scan_recall_fixture_query_overlaps(
            fixture_name,
            m,
            ef_search,
            query_count,
        );
        summarize_graph_scan_recall_fixture_query_overlaps(rows, m, ef_search, query_count)
    }

    fn probe_graph_scan_recall_fixture_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallFixtureSummaryRow {
        let rows = collect_graph_scan_recall_fixture_query_overlaps_for_relation(
            table_name,
            index_name,
            m,
            ef_search,
            query_count,
        );
        summarize_graph_scan_recall_fixture_query_overlaps(rows, m, ef_search, query_count)
    }

    /// Read `(id, source)` rows from a corpus / query table loaded by
    /// `scripts/load_real_corpus.py`. The returned vectors preserve the row
    /// order returned by Postgres so that ground-truth indices stay stable
    /// across reruns.
    fn load_external_recall_relation(table_name: &str) -> (Vec<i64>, Vec<Vec<f32>>) {
        let table_name = recall_fixture_ident(table_name);
        Spi::connect(|client| {
            let mut ids: Vec<i64> = Vec::new();
            let mut vectors: Vec<Vec<f32>> = Vec::new();
            let rows = client
                .select(
                    &format!("SELECT id, source FROM {table_name} ORDER BY id"),
                    None,
                    &[],
                )
                .expect("external recall relation query should succeed");
            for row in rows {
                let id = row["id"]
                    .value::<i64>()
                    .expect("id should decode")
                    .expect("id should be non-null");
                let source = row["source"]
                    .value::<Vec<f32>>()
                    .expect("source real[] should decode")
                    .expect("source real[] should be non-null");
                ids.push(id);
                vectors.push(source);
            }
            (ids, vectors)
        })
    }

    struct ExternalRecallContext {
        corpus_ids: Vec<i64>,
        corpus: Vec<Vec<f32>>,
        queries: Vec<Vec<f32>>,
        ground_truth_top_k: Vec<Vec<(usize, f32)>>,
        exact_quantized_row_indices_top10: Option<Vec<Vec<i64>>>,
        ctid_to_row_index: HashMap<(u32, u16), usize>,
    }

    fn build_external_recall_context(
        corpus_table: &str,
        query_table: &str,
        include_exact_quantized_top10: bool,
    ) -> ExternalRecallContext {
        let corpus_table_ident = recall_fixture_ident(corpus_table);
        let (corpus_ids, corpus) = load_external_recall_relation(corpus_table);
        let (_query_ids, queries) = load_external_recall_relation(query_table);

        assert!(
            !corpus.is_empty(),
            "external recall corpus {corpus_table_ident} must contain at least one row"
        );
        assert!(
            !queries.is_empty(),
            "external recall query table must contain at least one row"
        );

        let recall_k_wide = RECALL_K * 10;
        let ground_truth_top_k: Vec<Vec<(usize, f32)>> = queries
            .iter()
            .map(|query| {
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
                scores.truncate(recall_k_wide);
                scores
            })
            .collect();

        let id_to_row_index: HashMap<i64, usize> = corpus_ids
            .iter()
            .enumerate()
            .map(|(idx, id)| (*id, idx))
            .collect();
        let ctid_to_row_index: HashMap<(u32, u16), usize> = ctid_id_map(&corpus_table_ident)
            .into_iter()
            .map(|(ctid, id)| {
                let id_i64 = i64::try_from(id).expect("ctid id should fit into bigint");
                let row_index = *id_to_row_index
                    .get(&id_i64)
                    .expect("ctid id should map back to a corpus row index");
                (ctid, row_index)
            })
            .collect();
        let exact_quantized_row_indices_top10 = include_exact_quantized_top10.then(|| {
            Spi::connect(|client| {
                // The "exact quantized" baseline must come from a table-level
                // sort over persisted embeddings, not whichever ec_hnsw index
                // the planner happens to prefer on a multi-index corpus table.
                Spi::run("SET LOCAL enable_indexscan = off")
                    .expect("disabling index scans for exact quantized baseline should succeed");
                Spi::run("SET LOCAL enable_indexonlyscan = off").expect(
                    "disabling index-only scans for exact quantized baseline should succeed",
                );
                Spi::run("SET LOCAL enable_bitmapscan = off")
                    .expect("disabling bitmap scans for exact quantized baseline should succeed");
                queries
                    .iter()
                    .map(|query| {
                        client
                            .select(
                                &format!(
                                    "SELECT id
                                     FROM {corpus_table_ident}
                                     ORDER BY embedding <#> $1
                                     LIMIT 10"
                                ),
                                None,
                                &[query.clone().into()],
                            )
                            .expect("exact quantized external recall query should succeed")
                            .map(|row| {
                                let id = row["id"]
                                    .value::<i64>()
                                    .expect("id should decode")
                                    .expect("id should be non-null");
                                i64::try_from(*id_to_row_index.get(&id).expect(
                                    "exact quantized id should map back to a corpus row index",
                                ))
                                .expect("row index should fit into bigint")
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
        });

        ExternalRecallContext {
            corpus_ids,
            corpus,
            queries,
            ground_truth_top_k,
            exact_quantized_row_indices_top10,
            ctid_to_row_index,
        }
    }

    fn ndcg_at_k_external(true_top_k: &[(usize, f32)], pred_ids: &[i64], k: usize) -> f32 {
        let relevance: HashMap<usize, f32> =
            true_top_k.iter().take(k).map(|(i, s)| (*i, *s)).collect();

        let dcg: f32 = pred_ids
            .iter()
            .take(k)
            .enumerate()
            .map(|(rank, idx)| {
                let rel = *idx as usize;
                let score = relevance.get(&rel).copied().unwrap_or(0.0).max(0.0);
                score / ((rank as f32 + 2.0).ln() / 2.0_f32.ln())
            })
            .sum();

        let idcg: f32 = true_top_k
            .iter()
            .take(k)
            .enumerate()
            .map(|(rank, (_, score))| {
                let rel = score.max(0.0);
                rel / ((rank as f32 + 2.0).ln() / 2.0_f32.ln())
            })
            .sum();

        if idcg == 0.0 {
            0.0
        } else {
            dcg / idcg
        }
    }

    fn spearman_rank_correlation_external(true_top_k: &[(usize, f32)], pred_ids: &[i64]) -> f32 {
        let n = true_top_k.len().min(pred_ids.len());
        if n < 2 {
            return 0.0;
        }

        let pred_rank: HashMap<usize, usize> = pred_ids
            .iter()
            .enumerate()
            .take(n)
            .map(|(rank, idx)| (*idx as usize, rank))
            .collect();

        let mut d_squared_sum = 0.0_f64;
        for (true_rank, (idx, _)) in true_top_k.iter().enumerate().take(n) {
            let pred_r = pred_rank.get(idx).copied().unwrap_or(n);
            let d = true_rank as f64 - pred_r as f64;
            d_squared_sum += d * d;
        }

        let n = n as f64;
        1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0))) as f32
    }

    fn probe_graph_scan_recall_external_summary_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        m: i32,
        ef_search: i32,
    ) -> GraphScanRecallExternalSummaryRow {
        let index_name_ident = recall_fixture_ident(index_name);
        let recall_k_wide = RECALL_K * 10;

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name_ident}'::regclass::oid"))
                .expect("external recall index oid query should succeed")
                .expect("external recall index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let query_count = context.queries.len();
        let mut graph_top_10_hits = 0_i32;
        let mut graph_top_100_hits = 0_i32;
        let mut exact_top_10_hits = 0_i32;
        let mut graph_below_exact_queries = 0_i32;
        let mut worst_exact_gap = 0_i32;
        let mut ndcg_sum = 0.0_f32;
        let mut mae_sum = 0.0_f32;
        let mut spearman_sum = 0.0_f32;
        let exact_quantized_row_indices_top10 = context
            .exact_quantized_row_indices_top10
            .as_ref()
            .expect("summary context should include exact quantized top-10 rows");

        for ((query, truth), exact_quantized_row_indices) in context
            .queries
            .iter()
            .zip(context.ground_truth_top_k.iter())
            .zip(exact_quantized_row_indices_top10.iter())
        {
            // Graph scan: returns heap tids plus operator-facing `<#>` scores.
            let predicted_row_indices_with_scores: Vec<(usize, f32)> =
                unsafe { am::debug_gettuple_scan_heap_tids_with_scores(index_oid, query.clone()) }
                    .into_iter()
                    .map(|(heap_tid, operator_score)| {
                        let row_index = *context
                            .ctid_to_row_index
                            .get(&heap_tid)
                            .expect("graph heap tid should map back to a corpus row index");
                        (row_index, operator_score)
                    })
                    .collect();
            let predicted_row_indices: Vec<i64> = predicted_row_indices_with_scores
                .iter()
                .map(|(row_index, _)| {
                    i64::try_from(*row_index).expect("row index should fit into bigint")
                })
                .collect();

            // Top-10 graph recall vs fp32 truth (row-index space).
            let truth_top_10_ids: Vec<i64> = truth
                .iter()
                .take(RECALL_K)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_10_ids: Vec<i64> = predicted_row_indices
                .iter()
                .take(RECALL_K)
                .copied()
                .collect();
            let graph_overlap_10 = recall_top_k_overlap(&truth_top_10_ids, &predicted_top_10_ids);
            graph_top_10_hits += graph_overlap_10;

            // Top-100 graph recall vs the wider truth band.
            let truth_top_100_ids: Vec<i64> = truth
                .iter()
                .take(recall_k_wide)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_100_ids: Vec<i64> = predicted_row_indices
                .iter()
                .take(recall_k_wide)
                .copied()
                .collect();
            graph_top_100_hits += recall_top_k_overlap(&truth_top_100_ids, &predicted_top_100_ids);

            let exact_overlap_10 =
                recall_top_k_overlap(&truth_top_10_ids, exact_quantized_row_indices);
            exact_top_10_hits += exact_overlap_10;

            if graph_overlap_10 < exact_overlap_10 {
                graph_below_exact_queries += 1;
                worst_exact_gap = worst_exact_gap.max(exact_overlap_10 - graph_overlap_10);
            }

            ndcg_sum += ndcg_at_k_external(truth, &predicted_top_10_ids, RECALL_K);
            spearman_sum += spearman_rank_correlation_external(
                &truth.iter().take(RECALL_K).copied().collect::<Vec<_>>(),
                &predicted_top_10_ids,
            );

            // NFR-003 MAE: per predicted item, compare the graph's approximate
            // inner product estimate against the true fp32 inner product for
            // that same item. The operator-facing `<#>` score is ascending
            // negative inner product, so negate it back into similarity space.
            let predicted_top_10_score_errors: Vec<f32> = predicted_row_indices_with_scores
                .iter()
                .take(RECALL_K)
                .map(|(row_index, operator_score)| {
                    let approx_inner_product = -*operator_score;
                    let true_inner_product = dot_product(query, &context.corpus[*row_index]);
                    (approx_inner_product - true_inner_product).abs()
                })
                .collect();
            if !predicted_top_10_score_errors.is_empty() {
                mae_sum += predicted_top_10_score_errors.iter().sum::<f32>()
                    / predicted_top_10_score_errors.len() as f32;
            }
        }

        let recall_10_denom = (query_count as f32) * (RECALL_K as f32);
        let recall_100_denom = (query_count as f32) * (recall_k_wide as f32);
        (
            m,
            ef_search,
            i32::try_from(context.corpus_ids.len()).expect("corpus row count should fit into int"),
            i32::try_from(query_count).expect("query count should fit into int"),
            graph_top_10_hits as f32 / recall_10_denom,
            graph_top_100_hits as f32 / recall_100_denom,
            ndcg_sum / query_count as f32,
            mae_sum / query_count as f32,
            spearman_sum / query_count as f32,
            exact_top_10_hits as f32 / recall_10_denom,
            graph_below_exact_queries,
            worst_exact_gap,
        )
    }

    fn probe_graph_scan_recall_external_summary_for_relation(
        corpus_table: &str,
        query_table: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
    ) -> GraphScanRecallExternalSummaryRow {
        let context = build_external_recall_context(corpus_table, query_table, true);
        probe_graph_scan_recall_external_summary_for_context(&context, index_name, m, ef_search)
    }

    fn probe_graph_scan_recall_external_gate_row_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        m: i32,
        ef_search: i32,
        target: Option<f32>,
    ) -> (i32, i32, f32, Option<f32>, bool) {
        let index_name_ident = recall_fixture_ident(index_name);

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name_ident}'::regclass::oid"))
                .expect("external recall index oid query should succeed")
                .expect("external recall index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_top_10_hits = 0_i32;
        for (query, truth) in context
            .queries
            .iter()
            .zip(context.ground_truth_top_k.iter())
        {
            let predicted_row_indices: Vec<i64> =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .map(|heap_tid| {
                        let row_index = *context
                            .ctid_to_row_index
                            .get(&heap_tid)
                            .expect("graph heap tid should map back to a corpus row index");
                        i64::try_from(row_index).expect("row index should fit into bigint")
                    })
                    .collect();

            let truth_top_10_ids: Vec<i64> = truth
                .iter()
                .take(RECALL_K)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_10_ids: Vec<i64> = predicted_row_indices
                .iter()
                .take(RECALL_K)
                .copied()
                .collect();
            graph_top_10_hits += recall_top_k_overlap(&truth_top_10_ids, &predicted_top_10_ids);
        }

        let recall_at_10 =
            graph_top_10_hits as f32 / ((context.queries.len() as f32) * (RECALL_K as f32));
        let passed = target.map(|gate| recall_at_10 >= gate).unwrap_or(true);
        (m, ef_search, recall_at_10, target, passed)
    }

    // One-shot oracle: re-uses the external recall context machinery and
    // compares the measured `recall@10` against the published anchor recorded
    // in `docs/RECALL_ANN_BENCHMARKS_ANCHOR.md`. This is intentionally not a
    // sweep — anchor diagnostics live in the histogram / ef_sweep surfaces.
    fn probe_graph_scan_recall_ann_benchmarks_reference_for_relation(
        corpus_table: &str,
        query_table: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
    ) -> GraphScanRecallAnnBenchmarksReferenceRow {
        let summary = probe_graph_scan_recall_external_summary_for_relation(
            corpus_table,
            query_table,
            index_name,
            m,
            ef_search,
        );
        let measured_recall_at_10 = summary.4;
        let absolute_delta = measured_recall_at_10 - ANN_BENCHMARKS_ANCHOR_PUBLISHED_RECALL_AT_10;
        let within_two_percent = absolute_delta.abs() <= ANN_BENCHMARKS_ANCHOR_TOLERANCE;
        (
            m,
            ef_search,
            measured_recall_at_10,
            ANN_BENCHMARKS_ANCHOR_PUBLISHED_RECALL_AT_10,
            absolute_delta,
            within_two_percent,
        )
    }

    fn run_graph_scan_recall_gate_from_external(
        corpus_table: &str,
        query_table: &str,
        fixture_prefix: &str,
    ) -> Vec<(i32, i32, f32, Option<f32>, bool)> {
        let fixture_prefix = recall_fixture_ident(fixture_prefix);
        let context = build_external_recall_context(corpus_table, query_table, false);
        RECALL_GATE_CONFIGS
            .iter()
            .copied()
            .map(|(m, ef_search, target)| {
                let index_name = format!("{fixture_prefix}_m{m}_idx");
                probe_graph_scan_recall_external_gate_row_for_context(
                    &context,
                    &index_name,
                    m,
                    ef_search,
                    target,
                )
            })
            .collect()
    }

    /// Returns one row per top-10 recall bucket (`0..=10`). Buckets with no
    /// queries are still emitted so the output is always 11 rows. Builds the
    /// graph scan top-10 for each query in the supplied context and bins by
    /// the count of correct items vs the precomputed fp32 ground truth.
    fn build_graph_scan_recall_histogram_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        ef_search: i32,
    ) -> Vec<GraphScanRecallHistogramRow> {
        let index_name_ident = recall_fixture_ident(index_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name_ident}'::regclass::oid"))
                .expect("histogram index oid query should succeed")
                .expect("histogram index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut buckets = [0_i32; RECALL_K + 1];
        for (query, truth) in context
            .queries
            .iter()
            .zip(context.ground_truth_top_k.iter())
        {
            let truth_top_10_ids: Vec<i64> = truth
                .iter()
                .take(RECALL_K)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_10_ids: Vec<i64> =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        let row_index = *context
                            .ctid_to_row_index
                            .get(&heap_tid)
                            .expect("graph heap tid should map back to a corpus row index");
                        i64::try_from(row_index).expect("row index should fit into bigint")
                    })
                    .collect();
            let overlap = recall_top_k_overlap(&truth_top_10_ids, &predicted_top_10_ids);
            let bucket = usize::try_from(overlap)
                .expect("overlap should be non-negative")
                .min(RECALL_K);
            buckets[bucket] += 1;
        }

        let total_queries = context.queries.len() as f32;
        (0..=RECALL_K)
            .map(|bucket| {
                let count = buckets[bucket];
                let fraction = if total_queries > 0.0 {
                    count as f32 / total_queries
                } else {
                    0.0
                };
                (
                    i32::try_from(bucket).expect("bucket index should fit into int"),
                    count,
                    fraction,
                )
            })
            .collect()
    }

    /// Sweeps a list of `ef_search` values against a single fixture, building
    /// the external recall context exactly once and reusing it for every probe.
    /// Per-row latency is the wall clock spent inside
    /// `probe_graph_scan_recall_external_summary_for_context` for that
    /// `ef_search`, divided by the query count — it includes the small per-row
    /// overhead of NDCG/MAE/Spearman bookkeeping but is dominated by the graph
    /// scan itself.
    fn run_graph_scan_recall_ef_sweep_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        m: i32,
        ef_values: &[i32],
    ) -> Vec<GraphScanRecallEfSweepRow> {
        let query_count = context.queries.len();
        ef_values
            .iter()
            .copied()
            .map(|ef_search| {
                let started = Instant::now();
                let summary = probe_graph_scan_recall_external_summary_for_context(
                    context, index_name, m, ef_search,
                );
                let elapsed = started.elapsed();
                let mean_query_latency_ms = if query_count > 0 {
                    (elapsed.as_secs_f64() * 1000.0 / query_count as f64) as f32
                } else {
                    0.0
                };
                (
                    m,
                    ef_search,
                    summary.4, // graph_recall_at_10
                    summary.9, // exact_quantized_recall_at_10
                    summary.7, // mean_abs_score_error
                    mean_query_latency_ms,
                )
            })
            .collect()
    }

    /// Lists every query whose top-10 graph recall is strictly less than
    /// `recall_threshold`, alongside the exact-quantized recall on the same
    /// query and the corpus ids that neither retrieval surface managed to
    /// find. Rows are emitted in `query_index` order so the output is
    /// deterministic for diffing.
    fn run_graph_scan_recall_failure_breakdown_for_context(
        context: &ExternalRecallContext,
        index_name: &str,
        ef_search: i32,
        recall_threshold: i32,
    ) -> Vec<GraphScanRecallFailureBreakdownRow> {
        let index_name_ident = recall_fixture_ident(index_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name_ident}'::regclass::oid"))
                .expect("failure breakdown index oid query should succeed")
                .expect("failure breakdown index oid should exist");
        let exact_quantized_row_indices_top10 = context
            .exact_quantized_row_indices_top10
            .as_ref()
            .expect("failure breakdown context should include exact quantized top-10 rows");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut rows = Vec::new();
        for (query_index, ((query, truth), exact_quantized_row_indices)) in context
            .queries
            .iter()
            .zip(context.ground_truth_top_k.iter())
            .zip(exact_quantized_row_indices_top10.iter())
            .enumerate()
        {
            let truth_top_10_row_indices: Vec<i64> = truth
                .iter()
                .take(RECALL_K)
                .map(|(idx, _)| *idx as i64)
                .collect();
            let predicted_top_10_row_indices: Vec<i64> =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        let row_index = *context
                            .ctid_to_row_index
                            .get(&heap_tid)
                            .expect("graph heap tid should map back to a corpus row index");
                        i64::try_from(row_index).expect("row index should fit into bigint")
                    })
                    .collect();
            let graph_recall =
                recall_top_k_overlap(&truth_top_10_row_indices, &predicted_top_10_row_indices);
            if graph_recall >= recall_threshold {
                continue;
            }
            let exact_recall =
                recall_top_k_overlap(&truth_top_10_row_indices, exact_quantized_row_indices);

            // Missed = truth_top_10 \ (graph_top_10 ∪ exact_quantized_top_10),
            // mapped from row indices back to corpus ids so the output is
            // human-actionable.
            let missed_ids: Vec<i64> = truth_top_10_row_indices
                .iter()
                .filter(|row_index| {
                    !predicted_top_10_row_indices.contains(row_index)
                        && !exact_quantized_row_indices.contains(row_index)
                })
                .map(|row_index| {
                    let idx = usize::try_from(*row_index)
                        .expect("missed row index should be non-negative");
                    context.corpus_ids[idx]
                })
                .collect();

            rows.push((
                i32::try_from(query_index).expect("query index should fit into int"),
                graph_recall,
                exact_recall,
                missed_ids,
            ));
        }
        rows
    }

    fn probe_graph_scan_recall_top_level_oracle_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallTopLevelOracleSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("oracle summary fixture index oid query should succeed")
                .expect("oracle summary fixture index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut oracle_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_oracle_queries = 0_i32;
        let mut oracle_below_exact_queries = 0_i32;
        let mut worst_oracle_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let oracle_ids = unsafe {
                am::debug_top_level_oracle_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("oracle heap tid should map back to a benchmark row id"),
                )
                .expect("oracle id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized oracle summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let oracle_overlap = recall_top_k_overlap(&truth_ids, &oracle_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            oracle_hits += oracle_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < oracle_overlap {
                graph_below_oracle_queries += 1;
                worst_oracle_gap = worst_oracle_gap.max(oracle_overlap - graph_overlap);
            }
            if oracle_overlap < exact_overlap {
                oracle_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            graph_hits as f32 / recall_denominator,
            oracle_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_oracle_queries,
            oracle_below_exact_queries,
            worst_oracle_gap,
        )
    }

    fn probe_graph_scan_recall_top_level_oracle_k_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
        seed_count: usize,
    ) -> GraphScanRecallTopLevelOracleKSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("oracle-k summary fixture index oid query should succeed")
                .expect("oracle-k summary fixture index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut oracle_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_oracle_queries = 0_i32;
        let mut oracle_below_exact_queries = 0_i32;
        let mut worst_oracle_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let oracle_ids = unsafe {
                am::debug_top_level_oracle_k_seed_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                    seed_count,
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("oracle-k heap tid should map back to a benchmark row id"),
                )
                .expect("oracle-k id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized oracle-k summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let oracle_overlap = recall_top_k_overlap(&truth_ids, &oracle_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            oracle_hits += oracle_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < oracle_overlap {
                graph_below_oracle_queries += 1;
                worst_oracle_gap = worst_oracle_gap.max(oracle_overlap - graph_overlap);
            }
            if oracle_overlap < exact_overlap {
                oracle_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            i32::try_from(seed_count).expect("seed count should fit into int"),
            graph_hits as f32 / recall_denominator,
            oracle_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_oracle_queries,
            oracle_below_exact_queries,
            worst_oracle_gap,
        )
    }

    fn probe_graph_scan_recall_layer_oracle_k_carrydown_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
        layer: u8,
        seed_count: usize,
    ) -> GraphScanRecallLayerOracleKCarrydownSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("layer-oracle summary fixture index oid query should succeed")
                .expect("layer-oracle summary fixture index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut oracle_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_oracle_queries = 0_i32;
        let mut oracle_below_exact_queries = 0_i32;
        let mut worst_oracle_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let oracle_ids = unsafe {
                am::debug_layer_oracle_k_carrydown_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                    layer,
                    seed_count,
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("layer-oracle heap tid should map back to a benchmark row id"),
                )
                .expect("layer-oracle id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized layer-oracle summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let oracle_overlap = recall_top_k_overlap(&truth_ids, &oracle_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            oracle_hits += oracle_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < oracle_overlap {
                graph_below_oracle_queries += 1;
                worst_oracle_gap = worst_oracle_gap.max(oracle_overlap - graph_overlap);
            }
            if oracle_overlap < exact_overlap {
                oracle_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::from(layer),
            i32::try_from(query_count).expect("query count should fit into int"),
            i32::try_from(seed_count).expect("seed count should fit into int"),
            graph_hits as f32 / recall_denominator,
            oracle_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_oracle_queries,
            oracle_below_exact_queries,
            worst_oracle_gap,
        )
    }

    fn probe_graph_scan_recall_layer_neighbor_coverage_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
        layer: u8,
        seed_count: usize,
    ) -> GraphScanRecallLayerNeighborCoverageSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("layer-neighbor summary fixture index oid query should succeed")
                .expect("layer-neighbor summary fixture index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut neighbor_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_neighbor_queries = 0_i32;
        let mut neighbor_below_exact_queries = 0_i32;
        let mut worst_neighbor_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let neighbor_ids = unsafe {
                am::debug_layer_oracle_k_seed_layer0_neighbor_heap_tids(
                    index_oid,
                    query.clone(),
                    layer,
                    seed_count,
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("layer-neighbor heap tid should map back to a benchmark row id"),
                )
                .expect("layer-neighbor id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.clone().into()],
                    )
                    .expect("exact quantized layer-neighbor summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let neighbor_overlap = recall_top_k_overlap(&truth_ids, &neighbor_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            neighbor_hits += neighbor_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < neighbor_overlap {
                graph_below_neighbor_queries += 1;
                worst_neighbor_gap = worst_neighbor_gap.max(neighbor_overlap - graph_overlap);
            }
            if neighbor_overlap < exact_overlap {
                neighbor_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::from(layer),
            i32::try_from(query_count).expect("query count should fit into int"),
            i32::try_from(seed_count).expect("seed count should fit into int"),
            graph_hits as f32 / recall_denominator,
            neighbor_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_neighbor_queries,
            neighbor_below_exact_queries,
            worst_neighbor_gap,
        )
    }

    fn probe_graph_scan_recall_top_level_seed_coverage_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
        seed_count: usize,
    ) -> GraphScanRecallTopLevelSeedCoverageRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ctid_to_id = ctid_id_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("seed coverage fixture index oid query should succeed")
                .expect("seed coverage fixture index oid should exist");

        let all_top_level_ids = unsafe { am::debug_all_top_level_heap_tids(index_oid) }
            .into_iter()
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("top-level heap tid should map back to a benchmark row id"),
                )
                .expect("top-level id should fit into bigint")
            })
            .collect::<std::collections::HashSet<_>>();
        let reachable_top_level_ids = unsafe { am::debug_top_level_reachable_heap_tids(index_oid) }
            .into_iter()
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("reachable heap tid should map back to a benchmark row id"),
                )
                .expect("reachable id should fit into bigint")
            })
            .collect::<std::collections::HashSet<_>>();

        let mut oracle_seed_frequency = std::collections::HashMap::<i64, i32>::new();
        let mut reachable_seed_slots = 0_i32;
        let mut total_seed_slots = 0_i32;
        let mut fully_reachable_queries = 0_i32;

        for query in &queries {
            let oracle_seed_ids = unsafe {
                am::debug_top_level_oracle_k_seed_heap_tids(index_oid, query.clone(), seed_count)
            }
            .into_iter()
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("oracle seed heap tid should map back to a benchmark row id"),
                )
                .expect("oracle seed id should fit into bigint")
            })
            .collect::<Vec<_>>();

            total_seed_slots +=
                i32::try_from(oracle_seed_ids.len()).expect("oracle seed slot count should fit");
            let reachable_for_query = oracle_seed_ids
                .iter()
                .filter(|id| reachable_top_level_ids.contains(id))
                .count();
            reachable_seed_slots +=
                i32::try_from(reachable_for_query).expect("reachable query count should fit");
            if reachable_for_query == oracle_seed_ids.len() {
                fully_reachable_queries += 1;
            }
            for id in oracle_seed_ids {
                *oracle_seed_frequency.entry(id).or_insert(0) += 1;
            }
        }

        let unique_oracle_seed_ids =
            i32::try_from(oracle_seed_frequency.len()).expect("unique oracle seed ids should fit");
        let reachable_unique_oracle_seed_ids = i32::try_from(
            oracle_seed_frequency
                .keys()
                .filter(|id| reachable_top_level_ids.contains(id))
                .count(),
        )
        .expect("reachable unique oracle seed ids should fit");
        let mut frequent_oracle_seeds = oracle_seed_frequency.into_iter().collect::<Vec<_>>();
        frequent_oracle_seeds
            .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        frequent_oracle_seeds.truncate(seed_count.max(10));
        let (top_seed_ids, top_seed_query_counts): (Vec<_>, Vec<_>) =
            frequent_oracle_seeds.into_iter().unzip();

        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            i32::try_from(seed_count).expect("seed count should fit into int"),
            i32::try_from(all_top_level_ids.len()).expect("top-level node count should fit"),
            i32::try_from(reachable_top_level_ids.len())
                .expect("reachable top-level node count should fit"),
            unique_oracle_seed_ids,
            reachable_unique_oracle_seed_ids,
            if total_seed_slots == 0 {
                0.0
            } else {
                reachable_seed_slots as f32 / total_seed_slots as f32
            },
            fully_reachable_queries,
            top_seed_ids,
            top_seed_query_counts,
        )
    }

    fn id_heap_tid_map(table_name: &str) -> HashMap<i64, (u32, u16)> {
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
                .expect("id/ctid map query should succeed")
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
                        id,
                        (
                            u32::try_from(block_number)
                                .expect("block number should be non-negative"),
                            u16::try_from(offset_number)
                                .expect("offset number should be positive"),
                        ),
                    )
                })
                .collect::<HashMap<_, _>>()
        })
    }

    fn probe_graph_scan_recall_exact_seed_summary_for_relation(
        table_name: &str,
        index_name: &str,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallExactSeedSummaryRow {
        let table_name = recall_fixture_ident(table_name);
        let index_name = recall_fixture_ident(index_name);
        let corpus = Spi::connect(|client| {
            client
                .select(
                    &format!("SELECT count(*) AS count FROM {table_name}"),
                    None,
                    &[],
                )
                .expect("fixture row count query should succeed")
                .next()
                .expect("fixture row count should return one row")["count"]
                .value::<i64>()
                .expect("fixture row count should decode")
                .expect("fixture row count should be non-null")
        });
        let corpus = random_unit_vectors(
            usize::try_from(corpus).expect("fixture corpus size should fit usize"),
            RECALL_DIM,
            RECALL_SEED as u64,
        );
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let ground_truth = queries
            .iter()
            .map(|query| brute_force_top_k(&corpus, query, RECALL_K))
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(&table_name);
        let id_to_heap_tid = id_heap_tid_map(&table_name);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("exact-seed fixture index oid query should succeed")
                .expect("exact-seed fixture index oid should exist");

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");

        let mut graph_hits = 0_i32;
        let mut exact_seed1_hits = 0_i32;
        let mut exact_seed10_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut graph_below_exact_seed10_queries = 0_i32;
        let mut exact_seed10_below_exact_queries = 0_i32;
        let mut worst_exact_seed10_gap = 0_i32;

        for (query, truth) in queries.iter().zip(ground_truth.iter()) {
            let truth_ids = truth
                .iter()
                .map(|id| i64::try_from(*id).expect("truth id should fit into bigint"))
                .collect::<Vec<_>>();
            let predicted_ids =
                unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) }
                    .into_iter()
                    .take(RECALL_K)
                    .map(|heap_tid| {
                        i64::try_from(
                            *ctid_to_id
                                .get(&heap_tid)
                                .expect("graph heap tid should map back to a benchmark row id"),
                        )
                        .expect("graph id should fit into bigint")
                    })
                    .collect::<Vec<_>>();
            let exact_quantized_ids = Spi::connect(|client| {
                client
                    .select(
                        &format!(
                            "SELECT id
                             FROM {table_name}
                             ORDER BY embedding <#> $1
                             LIMIT 10"
                        ),
                        None,
                        &[query.to_vec().into()],
                    )
                    .expect("exact quantized exact-seed summary query should succeed")
                    .map(|row| {
                        row["id"]
                            .value::<i64>()
                            .expect("id should decode")
                            .expect("id should be non-null")
                    })
                    .collect::<Vec<_>>()
            });
            let exact_seed_heap_tids = exact_quantized_ids
                .iter()
                .filter_map(|id| id_to_heap_tid.get(id))
                .copied()
                .collect::<Vec<_>>();
            let exact_seed1_input = exact_seed_heap_tids
                .iter()
                .copied()
                .take(1)
                .collect::<Vec<_>>();
            let exact_seed1_ids = unsafe {
                am::debug_exact_seed_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    exact_seed1_input,
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("exact-seed1 heap tid should map back to a benchmark row id"),
                )
                .expect("exact-seed1 id should fit into bigint")
            })
            .collect::<Vec<_>>();
            let exact_seed10_ids = unsafe {
                am::debug_exact_seed_scan_heap_tids(
                    index_oid,
                    query.clone(),
                    exact_seed_heap_tids,
                    usize::try_from(ef_search).expect("ef_search should fit into usize"),
                )
            }
            .into_iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("exact-seed10 heap tid should map back to a benchmark row id"),
                )
                .expect("exact-seed10 id should fit into bigint")
            })
            .collect::<Vec<_>>();

            let graph_overlap = recall_top_k_overlap(&truth_ids, &predicted_ids);
            let exact_seed1_overlap = recall_top_k_overlap(&truth_ids, &exact_seed1_ids);
            let exact_seed10_overlap = recall_top_k_overlap(&truth_ids, &exact_seed10_ids);
            let exact_overlap = recall_top_k_overlap(&truth_ids, &exact_quantized_ids);

            graph_hits += graph_overlap;
            exact_seed1_hits += exact_seed1_overlap;
            exact_seed10_hits += exact_seed10_overlap;
            exact_hits += exact_overlap;

            if graph_overlap < exact_seed10_overlap {
                graph_below_exact_seed10_queries += 1;
                worst_exact_seed10_gap =
                    worst_exact_seed10_gap.max(exact_seed10_overlap - graph_overlap);
            }
            if exact_seed10_overlap < exact_overlap {
                exact_seed10_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            graph_hits as f32 / recall_denominator,
            exact_seed1_hits as f32 / recall_denominator,
            exact_seed10_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            graph_below_exact_seed10_queries,
            exact_seed10_below_exact_queries,
            worst_exact_seed10_gap,
        )
    }

    fn summarize_graph_scan_recall_fixture_query_overlaps(
        rows: Vec<GraphScanRecallFixtureQueryOverlapRow>,
        m: i32,
        ef_search: i32,
        query_count: usize,
    ) -> GraphScanRecallFixtureSummaryRow {
        let mut graph_hits = 0_i32;
        let mut exact_hits = 0_i32;
        let mut build_code_hits = 0_i32;
        let mut graph_below_exact_queries = 0_i32;
        let mut graph_below_build_code_queries = 0_i32;
        let mut build_code_below_exact_queries = 0_i32;
        let mut worst_exact_gap = 0_i32;
        let mut worst_build_code_gap = 0_i32;

        for (_, _, _, _, graph_overlap, exact_overlap, build_code_overlap) in &rows {
            graph_hits += *graph_overlap;
            exact_hits += *exact_overlap;
            build_code_hits += *build_code_overlap;

            if *graph_overlap < *exact_overlap {
                graph_below_exact_queries += 1;
                worst_exact_gap = worst_exact_gap.max(*exact_overlap - *graph_overlap);
            }
            if *graph_overlap < *build_code_overlap {
                graph_below_build_code_queries += 1;
                worst_build_code_gap =
                    worst_build_code_gap.max(*build_code_overlap - *graph_overlap);
            }
            if *build_code_overlap < *exact_overlap {
                build_code_below_exact_queries += 1;
            }
        }

        let recall_denominator = (query_count as f32) * (RECALL_K as f32);
        (
            m,
            ef_search,
            i32::try_from(query_count).expect("query count should fit into int"),
            graph_hits as f32 / recall_denominator,
            exact_hits as f32 / recall_denominator,
            build_code_hits as f32 / recall_denominator,
            graph_below_exact_queries,
            graph_below_build_code_queries,
            build_code_below_exact_queries,
            worst_exact_gap,
            worst_build_code_gap,
        )
    }

    fn build_graph_scan_recall_probe_with_sizes(
        m: i32,
        ef_search: i32,
        query_index: usize,
        corpus_size: usize,
        query_count: usize,
    ) -> GraphScanRecallProbeRow {
        assert!(corpus_size >= RECALL_K);
        assert!(query_count > query_index);

        let corpus = random_unit_vectors(corpus_size, RECALL_DIM, RECALL_SEED as u64);
        let queries =
            random_unit_vectors(query_count, RECALL_DIM, (RECALL_SEED as u64) + 1_000_000);
        let query = queries
            .get(query_index)
            .expect("query index should be within the generated query set");
        let truth = brute_force_top_k(&corpus, query, RECALL_K)
            .into_iter()
            .map(|id| i64::try_from(id).expect("truth id should fit into bigint"))
            .collect::<Vec<_>>();

        create_recall_table("ec_hnsw_graph_scan_recall_probe");
        insert_recall_corpus("ec_hnsw_graph_scan_recall_probe", &corpus);
        let ctid_to_id = ctid_id_map("ec_hnsw_graph_scan_recall_probe");
        let index_oid = create_recall_index(
            "ec_hnsw_graph_scan_recall_probe",
            "ec_hnsw_graph_scan_recall_probe_idx",
            m,
        );
        let index_relation =
            unsafe { open_valid_ec_hnsw_index(index_oid, "ec_hnsw_graph_scan_recall_probe") };
        let index_block_count = unsafe {
            i32::try_from(pg_sys::RelationGetNumberOfBlocksInFork(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
            ))
            .expect("block count should fit into int")
        };
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

        Spi::run(&format!("SET LOCAL ec_hnsw.ef_search = {ef_search}"))
            .expect("setting ef_search should succeed");
        let (prefill_found, _, _, _, _, _, _, _) =
            unsafe { am::debug_gettuple_current_result_state(index_oid, query.clone()) };
        let predicted_heap_tids =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query.clone()) };
        let predicted_ids = predicted_heap_tids
            .iter()
            .take(RECALL_K)
            .map(|heap_tid| {
                i64::try_from(
                    *ctid_to_id
                        .get(heap_tid)
                        .expect("probe heap tid should map back to a benchmark row id"),
                )
                .expect("predicted id should fit into bigint")
            })
            .collect::<Vec<_>>();
        let exact_quantized_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id
                     FROM ec_hnsw_graph_scan_recall_probe
                     ORDER BY embedding <#> $1
                     LIMIT 10",
                    None,
                    &[query.clone().into()],
                )
                .expect("exact quantized probe query should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        Spi::run("DROP INDEX ec_hnsw_graph_scan_recall_probe_idx")
            .expect("probe index cleanup should succeed");
        Spi::run("DROP TABLE ec_hnsw_graph_scan_recall_probe")
            .expect("probe table cleanup should succeed");

        (
            m,
            ef_search,
            index_block_count,
            i32::try_from(predicted_heap_tids.len()).expect("row count should fit into int"),
            prefill_found,
            truth,
            predicted_ids,
            exact_quantized_ids,
        )
    }

    fn build_graph_scan_recall_probe(
        m: i32,
        ef_search: i32,
        query_index: usize,
    ) -> GraphScanRecallProbeRow {
        build_graph_scan_recall_probe_with_sizes(
            m,
            ef_search,
            query_index,
            RECALL_CORPUS_SIZE,
            RECALL_QUERY_COUNT,
        )
    }

