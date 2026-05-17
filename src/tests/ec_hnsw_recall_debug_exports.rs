    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_gate_report() -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(recall_at_10, f32),
            name!(gate_recall_at_10, Option<f32>),
            name!(passes_gate, bool),
        ),
    > {
        TableIterator::new(run_graph_scan_recall_gate())
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_fixture_gate_reset(
        fixture_prefix: String,
        corpus_size: i32,
    ) -> TableIterator<'static, (name!(m, i32), name!(index_block_count, i32))> {
        TableIterator::new(reset_graph_scan_recall_gate_fixtures(
            &fixture_prefix,
            usize::try_from(corpus_size).expect("corpus size should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_fixture_gate_source_build_reset(
        fixture_prefix: String,
        corpus_size: i32,
    ) -> TableIterator<'static, (name!(m, i32), name!(index_block_count, i32))> {
        TableIterator::new(reset_graph_scan_recall_gate_source_fixtures(
            &fixture_prefix,
            usize::try_from(corpus_size).expect("corpus size should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_fixture_gate_report(
        fixture_prefix: String,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(recall_at_10, f32),
            name!(gate_recall_at_10, Option<f32>),
            name!(passes_gate, bool),
        ),
    > {
        TableIterator::new(run_graph_scan_recall_gate_from_fixtures(
            &fixture_prefix,
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_external_summary(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_search: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(corpus_rows, i32),
            name!(query_count, i32),
            name!(graph_recall_at_10, f32),
            name!(graph_recall_at_100, f32),
            name!(ndcg_at_10, f32),
            name!(mean_abs_score_error, f32),
            name!(spearman_rho_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_exact_queries, i32),
            name!(worst_exact_gap, i32),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_external_summary_for_relation(
            &corpus_table,
            &query_table,
            &index_name,
            m,
            ef_search,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_external_gate_report(
        corpus_table: String,
        query_table: String,
        fixture_prefix: String,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(recall_at_10, f32),
            name!(gate_recall_at_10, Option<f32>),
            name!(passes_gate, bool),
        ),
    > {
        TableIterator::new(run_graph_scan_recall_gate_from_external(
            &corpus_table,
            &query_table,
            &fixture_prefix,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_ann_benchmarks_reference(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_search: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(recall_at_10, f32),
            name!(published_recall_at_10, f32),
            name!(absolute_delta, f32),
            name!(within_two_percent, bool),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_ann_benchmarks_reference_for_relation(
                &corpus_table,
                &query_table,
                &index_name,
                m,
                ef_search,
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_histogram(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_search: i32,
    ) -> TableIterator<
        'static,
        (
            name!(recall_bucket, i32),
            name!(query_count, i32),
            name!(query_fraction, f32),
        ),
    > {
        // `m` is part of the SQL signature for parity with the other recall
        // diagnostics; the histogram itself is fully determined by `index_name`
        // and `ef_search`.
        let _ = m;
        let context = build_external_recall_context(&corpus_table, &query_table, false);
        TableIterator::new(build_graph_scan_recall_histogram_for_context(
            &context,
            &index_name,
            ef_search,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_ef_sweep(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_values: Vec<i32>,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(mean_abs_score_error, f32),
            name!(mean_query_latency_ms, f32),
        ),
    > {
        let context = build_external_recall_context(&corpus_table, &query_table, true);
        TableIterator::new(run_graph_scan_recall_ef_sweep_for_context(
            &context,
            &index_name,
            m,
            &ef_values,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_failure_breakdown(
        corpus_table: String,
        query_table: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        recall_threshold: i32,
    ) -> TableIterator<
        'static,
        (
            name!(query_index, i32),
            name!(graph_recall_at_10, i32),
            name!(exact_quantized_recall_at_10, i32),
            name!(missed_ids, Vec<i64>),
        ),
    > {
        // `m` is part of the SQL signature for parity with the other recall
        // diagnostics; the breakdown itself is fully determined by `index_name`
        // and `ef_search`.
        let _ = m;
        let context = build_external_recall_context(&corpus_table, &query_table, true);
        TableIterator::new(run_graph_scan_recall_failure_breakdown_for_context(
            &context,
            &index_name,
            ef_search,
            recall_threshold,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_probe(
        m: i32,
        ef_search: i32,
        query_index: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
        ),
    > {
        TableIterator::once(build_graph_scan_recall_probe(
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_probe_sized(
        m: i32,
        ef_search: i32,
        query_index: i32,
        corpus_size: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
        ),
    > {
        TableIterator::once(build_graph_scan_recall_probe_with_sizes(
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(corpus_size).expect("corpus size should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    fn ec_hnsw_graph_scan_recall_fixture_reset(
        fixture_name: String,
        m: i32,
        corpus_size: i32,
    ) -> i32 {
        reset_graph_scan_recall_fixture(
            &fixture_name,
            m,
            usize::try_from(corpus_size).expect("corpus size should be non-negative"),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_fixture_probe(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_index: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_fixture_transcript(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_index: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(graph_overlap, i32),
            name!(exact_overlap, i32),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
            name!(frontier_head, Option<String>),
            name!(frontier_provenance, Vec<String>),
            name!(expanded_sources, Vec<String>),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture_transcript(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_fixture_ranks(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_index: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(index_block_count, i32),
            name!(predicted_count, i32),
            name!(prefill_found, bool),
            name!(truth_top10_ids, Vec<i64>),
            name!(predicted_top10_ids, Vec<i64>),
            name!(exact_quantized_top10_ids, Vec<i64>),
            name!(truth_ranks_in_predicted, Vec<i32>),
            name!(exact_ranks_in_predicted, Vec<i32>),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture_ranks(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_fixture_score_audit(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_index: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(exact_quantized_top10_ids, Vec<i64>),
            name!(exact_quantized_scores, Vec<f32>),
            name!(emitted_ranks, Vec<i32>),
            name!(emitted_scores, Vec<f32>),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture_score_audit(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_index).expect("query index should be non-negative"),
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_fixture_summary(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(graph_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(build_code_recall_at_10, f32),
            name!(graph_below_exact_queries, i32),
            name!(graph_below_build_code_queries, i32),
            name!(build_code_below_exact_queries, i32),
            name!(worst_exact_gap, i32),
            name!(worst_build_code_gap, i32),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_fixture_summary(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_top_level_oracle_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(graph_recall_at_10, f32),
            name!(oracle_top_level_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_oracle_queries, i32),
            name!(oracle_below_exact_queries, i32),
            name!(worst_oracle_gap, i32),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_top_level_oracle_summary_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_top_level_oracle_k_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
        seed_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(seed_count, i32),
            name!(graph_recall_at_10, f32),
            name!(oracle_top_level_k_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_oracle_queries, i32),
            name!(oracle_below_exact_queries, i32),
            name!(worst_oracle_gap, i32),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_top_level_oracle_k_summary_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
                usize::try_from(seed_count).expect("seed count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_layer_oracle_k_carrydown_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        layer: i32,
        query_count: i32,
        seed_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(layer, i32),
            name!(query_count, i32),
            name!(seed_count, i32),
            name!(graph_recall_at_10, f32),
            name!(oracle_layer_k_carrydown_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_oracle_queries, i32),
            name!(oracle_below_exact_queries, i32),
            name!(worst_oracle_gap, i32),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_layer_oracle_k_carrydown_summary_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
                u8::try_from(layer).expect("layer should fit in u8"),
                usize::try_from(seed_count).expect("seed count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_layer_neighbor_coverage_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        layer: i32,
        query_count: i32,
        seed_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(layer, i32),
            name!(query_count, i32),
            name!(seed_count, i32),
            name!(graph_recall_at_10, f32),
            name!(oracle_seed_layer0_neighbor_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_neighbor_queries, i32),
            name!(neighbor_below_exact_queries, i32),
            name!(worst_neighbor_gap, i32),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_layer_neighbor_coverage_summary_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
                u8::try_from(layer).expect("layer should fit in u8"),
                usize::try_from(seed_count).expect("seed count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_top_level_seed_coverage_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
        seed_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(seed_count, i32),
            name!(top_level_node_count, i32),
            name!(reachable_top_level_node_count, i32),
            name!(unique_oracle_seed_id_count, i32),
            name!(reachable_unique_oracle_seed_id_count, i32),
            name!(reachable_oracle_seed_slot_fraction, f32),
            name!(fully_reachable_queries, i32),
            name!(top_oracle_seed_ids, Vec<i64>),
            name!(top_oracle_seed_query_counts, Vec<i32>),
        ),
    > {
        TableIterator::once(
            probe_graph_scan_recall_top_level_seed_coverage_for_relation(
                &table_name,
                &index_name,
                m,
                ef_search,
                usize::try_from(query_count).expect("query count should be non-negative"),
                usize::try_from(seed_count).expect("seed count should be non-negative"),
            ),
        )
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_exact_seed_summary_rel(
        table_name: String,
        index_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(graph_recall_at_10, f32),
            name!(exact_seed1_recall_at_10, f32),
            name!(exact_seed10_recall_at_10, f32),
            name!(exact_quantized_recall_at_10, f32),
            name!(graph_below_exact_seed10_queries, i32),
            name!(exact_seed10_below_exact_queries, i32),
            name!(worst_exact_seed10_gap, i32),
        ),
    > {
        TableIterator::once(probe_graph_scan_recall_exact_seed_summary_for_relation(
            &table_name,
            &index_name,
            m,
            ef_search,
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_scan_recall_fixture_query_overlaps(
        fixture_name: String,
        m: i32,
        ef_search: i32,
        query_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(m, i32),
            name!(ef_search, i32),
            name!(query_count, i32),
            name!(query_index, i32),
            name!(graph_overlap, i32),
            name!(exact_overlap, i32),
            name!(build_code_overlap, i32),
        ),
    > {
        TableIterator::new(collect_graph_scan_recall_fixture_query_overlaps(
            &fixture_name,
            m,
            ef_search,
            usize::try_from(query_count).expect("query count should be non-negative"),
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_graph_hierarchy_summary(
        index_oid: pg_sys::Oid,
    ) -> TableIterator<
        'static,
        (
            name!(level, i32),
            name!(node_count, i32),
            name!(avg_neighbor_count, f64),
            name!(min_neighbor_count, i32),
            name!(max_neighbor_count, i32),
            name!(expected_max_neighbors, i32),
        ),
    > {
        drop(open_valid_ec_hnsw_index_guard(
            index_oid,
            "ec_hnsw_graph_hierarchy_summary",
        ));

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let m = metadata.m as usize;
        let code_len = code_len(metadata.dimensions as usize, metadata.bits);

        // Build a map of neighbor TID -> decoded neighbor tuple
        let neighbor_map: HashMap<am::page::ItemPointer, am::page::TqNeighborTuple> = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples
                    .iter()
                    .enumerate()
                    .filter_map(move |(idx, tuple)| {
                        if tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG) {
                            Some((
                                am::page::ItemPointer {
                                    block_number: page.block_number,
                                    offset_number: (idx + 1) as u16,
                                },
                                am::page::TqNeighborTuple::decode(tuple)
                                    .expect("neighbor tuple should decode"),
                            ))
                        } else {
                            None
                        }
                    })
            })
            .collect();

        struct LevelStats {
            node_count: usize,
            total_neighbors: usize,
            min_neighbors: usize,
            max_neighbors: usize,
        }

        let mut level_stats: HashMap<u8, LevelStats> = HashMap::new();

        for (_, element) in decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len)
        {
            let neighbor = neighbor_map
                .get(&element.neighbortid)
                .expect("element neighbor TID should resolve");

            // For each layer this element participates in, count valid neighbors
            for layer in 0..=element.level {
                let (start, end) = if layer == 0 {
                    (0, m * 2)
                } else {
                    let s = m * 2 + (usize::from(layer) - 1) * m;
                    (s, s + m)
                };

                let valid_count = neighbor
                    .tids
                    .iter()
                    .skip(start)
                    .take(end.saturating_sub(start))
                    .filter(|tid| **tid != am::page::ItemPointer::INVALID)
                    .count();

                let stats = level_stats.entry(layer).or_insert(LevelStats {
                    node_count: 0,
                    total_neighbors: 0,
                    min_neighbors: usize::MAX,
                    max_neighbors: 0,
                });
                stats.node_count += 1;
                stats.total_neighbors += valid_count;
                if valid_count < stats.min_neighbors {
                    stats.min_neighbors = valid_count;
                }
                if valid_count > stats.max_neighbors {
                    stats.max_neighbors = valid_count;
                }
            }
        }

        let mut rows: Vec<(i32, i32, f64, i32, i32, i32)> = level_stats
            .iter()
            .map(|(&level, stats)| {
                let avg = if stats.node_count > 0 {
                    stats.total_neighbors as f64 / stats.node_count as f64
                } else {
                    0.0
                };
                let expected_max = if level == 0 { (m * 2) as i32 } else { m as i32 };
                (
                    i32::from(level),
                    stats.node_count as i32,
                    avg,
                    stats.min_neighbors as i32,
                    stats.max_neighbors as i32,
                    expected_max,
                )
            })
            .collect();
        rows.sort_by_key(|(level, _, _, _, _, _)| *level);

        TableIterator::new(rows)
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_scan_profile(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_elapsed_us, i64),
            name!(emit_elapsed_us, i64),
            name!(total_elapsed_us, i64),
            name!(rescan_phase, String),
            name!(rescan_current_result, bool),
            name!(rescan_ordered_slots, i32),
            name!(rescan_pending_heap_tids, i32),
            name!(rescan_visited_elements, i32),
            name!(rescan_expanded_sources, i32),
            name!(rescan_emitted_elements, i32),
            name!(rescan_bootstrap_expansions, i32),
            name!(rescan_bootstrap_pages_read, i32),
            name!(rescan_quantizer_cache_hit, bool),
            name!(result_count, i32),
            name!(final_phase, String),
            name!(final_ordered_slots, i32),
            name!(total_bootstrap_expansions, i32),
            name!(total_bootstrap_pages_read, i32),
            name!(total_linear_pages_read, i32),
            name!(total_elements_scored, i32),
            name!(total_elements_skipped, i32),
            name!(total_heap_tids_returned, i32),
            name!(total_quantizer_cache_hit, bool),
            name!(total_emitted_elements, i32),
        ),
    > {
        drop(open_valid_ec_hnsw_index_guard(
            index_oid,
            "tests.ec_hnsw_debug_scan_profile",
        ));

        let (
            rescan_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            rescan_phase,
            rescan_current_result,
            rescan_ordered_slots,
            rescan_pending_heap_tids,
            rescan_visited_elements,
            rescan_expanded_sources,
            rescan_emitted_elements,
            rescan_bootstrap_expansions,
            rescan_bootstrap_pages_read,
            rescan_quantizer_cache_hit,
            result_count,
            final_phase,
            final_ordered_slots,
            total_bootstrap_expansions,
            total_bootstrap_pages_read,
            total_linear_pages_read,
            total_elements_scored,
            total_elements_skipped,
            total_heap_tids_returned,
            total_quantizer_cache_hit,
            total_emitted_elements,
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
            _grouped_traversal_approx_score_calls,
            _grouped_traversal_approx_score_elapsed_us,
            _grouped_traversal_exact_score_calls,
            _grouped_traversal_exact_score_elapsed_us,
            _grouped_traversal_budgeted_expansions,
            _grouped_traversal_budgeted_candidates,
            _grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, query) };

        TableIterator::once((
            rescan_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            rescan_phase,
            rescan_current_result,
            rescan_ordered_slots,
            rescan_pending_heap_tids,
            rescan_visited_elements,
            rescan_expanded_sources,
            rescan_emitted_elements,
            rescan_bootstrap_expansions,
            rescan_bootstrap_pages_read,
            rescan_quantizer_cache_hit,
            result_count,
            final_phase,
            final_ordered_slots,
            total_bootstrap_expansions,
            total_bootstrap_pages_read,
            total_linear_pages_read,
            total_elements_scored,
            total_elements_skipped,
            total_heap_tids_returned,
            total_quantizer_cache_hit,
            total_emitted_elements,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_scan_profile_limited(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        limit_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_elapsed_us, i64),
            name!(emit_elapsed_us, i64),
            name!(total_elapsed_us, i64),
            name!(rescan_phase, String),
            name!(rescan_current_result, bool),
            name!(rescan_ordered_slots, i32),
            name!(rescan_pending_heap_tids, i32),
            name!(rescan_visited_elements, i32),
            name!(rescan_expanded_sources, i32),
            name!(rescan_emitted_elements, i32),
            name!(rescan_bootstrap_expansions, i32),
            name!(rescan_bootstrap_pages_read, i32),
            name!(rescan_quantizer_cache_hit, bool),
            name!(result_count, i32),
            name!(final_phase, String),
            name!(final_ordered_slots, i32),
            name!(total_bootstrap_expansions, i32),
            name!(total_bootstrap_pages_read, i32),
            name!(total_linear_pages_read, i32),
            name!(total_elements_scored, i32),
            name!(total_elements_skipped, i32),
            name!(total_heap_tids_returned, i32),
            name!(total_quantizer_cache_hit, bool),
            name!(total_emitted_elements, i32),
        ),
    > {
        if limit_count < 0 {
            pgrx::error!("limit_count must be non-negative");
        }

        drop(open_valid_ec_hnsw_index_guard(
            index_oid,
            "tests.ec_hnsw_debug_scan_profile_limited",
        ));

        let (
            rescan_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            rescan_phase,
            rescan_current_result,
            rescan_ordered_slots,
            rescan_pending_heap_tids,
            rescan_visited_elements,
            rescan_expanded_sources,
            rescan_emitted_elements,
            rescan_bootstrap_expansions,
            rescan_bootstrap_pages_read,
            rescan_quantizer_cache_hit,
            result_count,
            final_phase,
            final_ordered_slots,
            total_bootstrap_expansions,
            total_bootstrap_pages_read,
            total_linear_pages_read,
            total_elements_scored,
            total_elements_skipped,
            total_heap_tids_returned,
            total_quantizer_cache_hit,
            total_emitted_elements,
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
            _grouped_traversal_approx_score_calls,
            _grouped_traversal_approx_score_elapsed_us,
            _grouped_traversal_exact_score_calls,
            _grouped_traversal_exact_score_elapsed_us,
            _grouped_traversal_budgeted_expansions,
            _grouped_traversal_budgeted_candidates,
            _grouped_traversal_budgeted_exact_candidates,
        ) = unsafe {
            am::debug_profile_ordered_scan_with_limit(
                index_oid,
                query,
                Some(usize::try_from(limit_count).expect("limit count should fit in usize")),
            )
        };

        TableIterator::once((
            rescan_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            rescan_phase,
            rescan_current_result,
            rescan_ordered_slots,
            rescan_pending_heap_tids,
            rescan_visited_elements,
            rescan_expanded_sources,
            rescan_emitted_elements,
            rescan_bootstrap_expansions,
            rescan_bootstrap_pages_read,
            rescan_quantizer_cache_hit,
            result_count,
            final_phase,
            final_ordered_slots,
            total_bootstrap_expansions,
            total_bootstrap_pages_read,
            total_linear_pages_read,
            total_elements_scored,
            total_elements_skipped,
            total_heap_tids_returned,
            total_quantizer_cache_hit,
            total_emitted_elements,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_scan_heap_fetch_profile(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        limit_count: i32,
        project_attnum: i32,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_elapsed_us, i64),
            name!(emit_elapsed_us, i64),
            name!(total_elapsed_us, i64),
            name!(slot_fetch_elapsed_us, i64),
            name!(projection_elapsed_us, i64),
            name!(result_count, i32),
            name!(slot_fetch_count, i32),
            name!(projected_count, i32),
        ),
    > {
        if limit_count < 0 {
            pgrx::error!("limit_count must be non-negative");
        }
        if project_attnum < 0 {
            pgrx::error!("project_attnum must be non-negative");
        }

        drop(open_valid_ec_hnsw_index_guard(
            index_oid,
            "tests.ec_hnsw_debug_scan_heap_fetch_profile",
        ));

        let (
            rescan_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            slot_fetch_elapsed_us,
            projection_elapsed_us,
            result_count,
            slot_fetch_count,
            projected_count,
        ) = unsafe {
            am::debug_profile_ordered_scan_with_heap_fetch(
                index_oid,
                query,
                usize::try_from(limit_count).expect("limit count should fit in usize"),
                (project_attnum > 0).then_some(project_attnum),
            )
        };

        TableIterator::once((
            rescan_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            slot_fetch_elapsed_us,
            projection_elapsed_us,
            result_count,
            slot_fetch_count,
            projected_count,
        ))
    }

    struct PqFastScanRuntimeSettings {
        build_enabled: bool,
        scan_enabled: bool,
        scan_window: Option<String>,
        traversal_score_mode: Option<String>,
        rerank_mode: Option<String>,
        rerank_source_column: Option<String>,
        exact_traversal_enabled: bool,
        exact_traversal_scope: Option<String>,
        exact_traversal_strategy: Option<String>,
        exact_traversal_limit: Option<String>,
    }

    fn current_pq_fastscan_runtime_settings() -> PqFastScanRuntimeSettings {
        let env_string = |canonical: &str, legacy: &str| {
            std::env::var_os(canonical)
                .or_else(|| std::env::var_os(legacy))
                .map(|value| value.to_string_lossy().into_owned())
        };
        PqFastScanRuntimeSettings {
            build_enabled: true,
            scan_enabled: true,
            scan_window: Some(
                env_string(
                    "TQVECTOR_PQ_FASTSCAN_SCAN_WINDOW",
                    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW",
                )
                .unwrap_or_else(|| crate::am::PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW.to_string()),
            ),
            traversal_score_mode: Some(
                env_string(
                    "TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE",
                    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_GROUPED_SCORE_MODE",
                )
                .unwrap_or_else(|| {
                    crate::am::PQ_FASTSCAN_DEFAULT_TRAVERSAL_SCORE_MODE_NAME.to_owned()
                }),
            ),
            rerank_mode: Some(
                env_string(
                    "TQVECTOR_PQ_FASTSCAN_RERANK_MODE",
                    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_MODE",
                )
                .unwrap_or_else(|| crate::am::PQ_FASTSCAN_DEFAULT_RERANK_MODE_NAME.to_owned()),
            ),
            rerank_source_column: Some(
                env_string(
                    "TQVECTOR_PQ_FASTSCAN_RERANK_SOURCE_COLUMN",
                    "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_SOURCE_COLUMN",
                )
                .unwrap_or_else(|| "build_source_column".to_owned()),
            ),
            exact_traversal_enabled: std::env::var_os("TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL")
                .or_else(|| {
                    std::env::var_os("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL")
                })
                .is_some(),
            exact_traversal_scope: env_string(
                "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_SCOPE",
                "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_SCOPE",
            ),
            exact_traversal_strategy: env_string(
                "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_STRATEGY",
                "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_STRATEGY",
            ),
            exact_traversal_limit: env_string(
                "TQVECTOR_PQ_FASTSCAN_EXACT_TRAVERSAL_LIMIT",
                "TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_EXACT_TRAVERSAL_LIMIT",
            ),
        }
    }

    struct PqFastScanIndexRuntimeSettings {
        base: PqFastScanRuntimeSettings,
        traversal_score_mode_resolution: Option<String>,
        rerank_mode_resolution: Option<String>,
        layout_binary_word_count: Option<i32>,
    }

    fn current_pq_fastscan_runtime_settings_for_index(
        index_oid: pg_sys::Oid,
    ) -> PqFastScanIndexRuntimeSettings {
        let index_relation = open_valid_ec_hnsw_index_guard(
            index_oid,
            "tests.ec_hnsw_debug_pq_fastscan_runtime_settings_for_index",
        );
        let (_block_count, _m, _ef_construction, metadata) =
            unsafe { am::debug_index_metadata(index_oid) };
        let storage = unsafe {
            am::graph::GraphStorageDescriptor::from_index_relation(
                index_relation.as_ptr(),
                &metadata,
            )
        }
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        let layout = match storage {
            am::graph::GraphStorageDescriptor::PqFastScan(layout) => layout,
            am::graph::GraphStorageDescriptor::TurboQuant { .. }
            | am::graph::GraphStorageDescriptor::TurboQuantHotCold(_) => {
                drop(index_relation);
                pgrx::error!(
                    "tests.ec_hnsw_debug_pq_fastscan_runtime_settings_for_index requires a pq_fastscan index"
                );
            }
        };
        let traversal = am::resolve_pq_fastscan_traversal_score_mode_decision(storage);
        let rerank =
            unsafe { am::resolve_pq_fastscan_rerank_mode_decision(index_relation.as_ptr(), storage) };
        drop(index_relation);
        let mut base = current_pq_fastscan_runtime_settings();
        base.traversal_score_mode = Some(traversal.mode_name().to_owned());
        base.rerank_mode = Some(rerank.mode_name().to_owned());
        base.rerank_source_column = rerank.source_column;

        PqFastScanIndexRuntimeSettings {
            base,
            traversal_score_mode_resolution: Some(traversal.resolution.as_str().to_owned()),
            rerank_mode_resolution: Some(rerank.resolution.as_str().to_owned()),
            layout_binary_word_count: Some(
                i32::try_from(layout.binary_word_count)
                    .expect("binary-word count should fit in i32"),
            ),
        }
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_pq_fastscan_runtime_settings() -> TableIterator<
        'static,
        (
            name!(pq_fastscan_build_enabled, bool),
            name!(pq_fastscan_scan_enabled, bool),
            name!(pq_fastscan_scan_window, Option<String>),
            name!(pq_fastscan_traversal_score_mode, Option<String>),
            name!(pq_fastscan_rerank_mode, Option<String>),
            name!(pq_fastscan_rerank_source_column, Option<String>),
            name!(pq_fastscan_exact_traversal_enabled, bool),
            name!(pq_fastscan_exact_traversal_scope, Option<String>),
            name!(pq_fastscan_exact_traversal_strategy, Option<String>),
            name!(pq_fastscan_exact_traversal_limit, Option<String>),
        ),
    > {
        let settings = current_pq_fastscan_runtime_settings();
        TableIterator::once((
            settings.build_enabled,
            settings.scan_enabled,
            settings.scan_window,
            settings.traversal_score_mode,
            settings.rerank_mode,
            settings.rerank_source_column,
            settings.exact_traversal_enabled,
            settings.exact_traversal_scope,
            settings.exact_traversal_strategy,
            settings.exact_traversal_limit,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_pq_fastscan_runtime_settings_for_index(
        index_oid: pg_sys::Oid,
    ) -> TableIterator<
        'static,
        (
            name!(pq_fastscan_build_enabled, bool),
            name!(pq_fastscan_scan_enabled, bool),
            name!(pq_fastscan_scan_window, Option<String>),
            name!(pq_fastscan_traversal_score_mode, Option<String>),
            name!(pq_fastscan_traversal_score_mode_resolution, Option<String>),
            name!(pq_fastscan_layout_binary_word_count, Option<i32>),
            name!(pq_fastscan_rerank_mode, Option<String>),
            name!(pq_fastscan_rerank_mode_resolution, Option<String>),
            name!(pq_fastscan_rerank_source_column, Option<String>),
            name!(pq_fastscan_exact_traversal_enabled, bool),
            name!(pq_fastscan_exact_traversal_scope, Option<String>),
            name!(pq_fastscan_exact_traversal_strategy, Option<String>),
            name!(pq_fastscan_exact_traversal_limit, Option<String>),
        ),
    > {
        let settings = current_pq_fastscan_runtime_settings_for_index(index_oid);
        TableIterator::once((
            settings.base.build_enabled,
            settings.base.scan_enabled,
            settings.base.scan_window,
            settings.base.traversal_score_mode,
            settings.traversal_score_mode_resolution,
            settings.layout_binary_word_count,
            settings.base.rerank_mode,
            settings.rerank_mode_resolution,
            settings.base.rerank_source_column,
            settings.base.exact_traversal_enabled,
            settings.base.exact_traversal_scope,
            settings.base.exact_traversal_strategy,
            settings.base.exact_traversal_limit,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_adr030_runtime_settings() -> TableIterator<
        'static,
        (
            name!(grouped_build_enabled, bool),
            name!(grouped_scan_enabled, bool),
            name!(grouped_scan_window, Option<String>),
            name!(grouped_scan_score_mode, Option<String>),
            name!(grouped_scan_rerank_mode, Option<String>),
            name!(grouped_scan_rerank_source_column, Option<String>),
            name!(grouped_exact_traversal_enabled, bool),
            name!(grouped_exact_traversal_scope, Option<String>),
            name!(grouped_exact_traversal_strategy, Option<String>),
            name!(grouped_exact_traversal_limit, Option<String>),
        ),
    > {
        let settings = current_pq_fastscan_runtime_settings();
        TableIterator::once((
            settings.build_enabled,
            settings.scan_enabled,
            settings.scan_window,
            settings.traversal_score_mode,
            settings.rerank_mode,
            settings.rerank_source_column,
            settings.exact_traversal_enabled,
            settings.exact_traversal_scope,
            settings.exact_traversal_strategy,
            settings.exact_traversal_limit,
        ))
    }

    fn validate_debug_index(index_oid: pg_sys::Oid, helper_name: &'static str) {
        drop(open_valid_ec_hnsw_index_guard(index_oid, helper_name));
    }

    type PqFastScanScanOrderDriftSummaryValues = (
        i32,
        i32,
        i32,
        f64,
        i32,
        f64,
        Option<i32>,
        Option<i32>,
        bool,
        bool,
        bool,
        bool,
    );
    type PqFastScanScanWindowedRowValues = (
        i64,
        i32,
        i32,
        i32,
        f32,
        Option<f32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
    );
    type PqFastScanScanWindowedSummaryValues = (
        i32,
        i32,
        i32,
        i32,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        f64,
        f64,
        i32,
        i32,
        f64,
        f64,
    );
    type PqFastScanScanComparisonRowValues =
        (i64, i32, i32, f32, Option<f32>, Option<i32>, Option<i32>);
    type PqFastScanScanComparisonSummaryValues = (i32, i32, i32, i32, f64, f32, f64);
    type DebugScanHotPathProfileValues = (
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i64,
        i32,
        i32,
        i64,
        i32,
        i32,
        i64,
        i32,
        i64,
        i32,
        i32,
        i32,
        i64,
        i32,
        i64,
        i32,
        i32,
        i32,
    );
    type PqFastScanRerankProfileValues = (
        i64,
        i64,
        i64,
        i64,
        i32,
        i32,
        i64,
        i32,
        i64,
        i32,
        i64,
        i64,
        i64,
    );
    type TurboQuantScanStageProfileValues = (
        i64,
        i64,
        i32,
        i64,
        i32,
        i32,
        i64,
        i32,
        i64,
        String,
        bool,
        bool,
    );

    fn pq_fastscan_scan_order_drift_summary_values(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> PqFastScanScanOrderDriftSummaryValues {
        unsafe { am::debug_grouped_scan_order_drift_summary(index_oid, query) }
    }

    fn pq_fastscan_scan_windowed_rows_values(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        window_size: i32,
    ) -> Vec<PqFastScanScanWindowedRowValues> {
        unsafe { am::debug_grouped_scan_windowed_rows(index_oid, query, window_size) }
            .into_iter()
            .map(
                |(
                    (block_number, offset_number),
                    approx_rank,
                    windowed_rank,
                    approx_score,
                    comparison_score,
                    exact_rank,
                    exact_rank_shift,
                    windowed_rank_shift,
                )| {
                    (
                        i64::from(block_number),
                        i32::from(offset_number),
                        approx_rank,
                        windowed_rank,
                        approx_score,
                        comparison_score,
                        exact_rank,
                        exact_rank_shift,
                        windowed_rank_shift,
                    )
                },
            )
            .collect::<Vec<_>>()
    }

    fn pq_fastscan_scan_windowed_summary_values(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        window_size: i32,
    ) -> PqFastScanScanWindowedSummaryValues {
        unsafe { am::debug_grouped_scan_windowed_summary(index_oid, query, window_size) }
    }

    fn pq_fastscan_scan_comparison_rows_values(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> Vec<PqFastScanScanComparisonRowValues> {
        unsafe { am::debug_grouped_scan_comparison_rows(index_oid, query) }
            .into_iter()
            .map(
                |(
                    (block_number, offset_number),
                    approx_rank,
                    approx_score,
                    comparison_score,
                    exact_rank,
                    exact_rank_shift,
                )| {
                    (
                        i64::from(block_number),
                        i32::from(offset_number),
                        approx_rank,
                        approx_score,
                        comparison_score,
                        exact_rank,
                        exact_rank_shift,
                    )
                },
            )
            .collect::<Vec<_>>()
    }

    fn pq_fastscan_scan_comparison_summary_values(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> PqFastScanScanComparisonSummaryValues {
        unsafe { am::debug_grouped_scan_comparison_summary(index_oid, query) }
    }

    fn debug_scan_hot_path_profile_values(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> DebugScanHotPathProfileValues {
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
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = unsafe { am::debug_profile_ordered_scan(index_oid, query) };

        (
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        )
    }

    fn pq_fastscan_rerank_profile_values(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        limit_count: i32,
    ) -> PqFastScanRerankProfileValues {
        unsafe { am::debug_grouped_rerank_profile(index_oid, query, limit_count) }
    }

    fn turboquant_scan_stage_profile_values(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TurboQuantScanStageProfileValues {
        unsafe { am::debug_turboquant_scan_stage_profile(index_oid, query) }
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_scan_hot_path_profile(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_amrescan_total_elapsed_us, i64),
            name!(rescan_query_decode_elapsed_us, i64),
            name!(rescan_scan_setup_elapsed_us, i64),
            name!(rescan_store_query_elapsed_us, i64),
            name!(rescan_prepare_query_elapsed_us, i64),
            name!(rescan_reset_state_elapsed_us, i64),
            name!(rescan_initialize_entry_elapsed_us, i64),
            name!(rescan_upper_layer_seed_elapsed_us, i64),
            name!(rescan_layer0_seed_elapsed_us, i64),
            name!(rescan_stage_ordered_results_elapsed_us, i64),
            name!(rescan_initial_prefetch_elapsed_us, i64),
            name!(rescan_frontier_consume_elapsed_us, i64),
            name!(rescan_graph_result_materialize_elapsed_us, i64),
            name!(graph_element_cache_hits, i32),
            name!(graph_element_cache_misses, i32),
            name!(graph_element_load_elapsed_us, i64),
            name!(graph_neighbor_cache_hits, i32),
            name!(graph_neighbor_cache_misses, i32),
            name!(graph_neighbor_load_elapsed_us, i64),
            name!(candidate_score_calls, i32),
            name!(candidate_score_elapsed_us, i64),
            name!(score_cache_hits, i32),
            name!(score_cache_misses, i32),
            name!(grouped_traversal_approx_score_calls, i32),
            name!(grouped_traversal_approx_score_elapsed_us, i64),
            name!(grouped_traversal_exact_score_calls, i32),
            name!(grouped_traversal_exact_score_elapsed_us, i64),
            name!(grouped_traversal_budgeted_expansions, i32),
            name!(grouped_traversal_budgeted_candidates, i32),
            name!(grouped_traversal_budgeted_exact_candidates, i32),
        ),
    > {
        validate_debug_index(index_oid, "tests.ec_hnsw_debug_scan_hot_path_profile");

        let (
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ) = debug_scan_hot_path_profile_values(index_oid, query);

        TableIterator::once((
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            grouped_traversal_approx_score_calls,
            grouped_traversal_approx_score_elapsed_us,
            grouped_traversal_exact_score_calls,
            grouped_traversal_exact_score_elapsed_us,
            grouped_traversal_budgeted_expansions,
            grouped_traversal_budgeted_candidates,
            grouped_traversal_budgeted_exact_candidates,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_pq_fastscan_scan_hot_path_profile(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_amrescan_total_elapsed_us, i64),
            name!(rescan_query_decode_elapsed_us, i64),
            name!(rescan_scan_setup_elapsed_us, i64),
            name!(rescan_store_query_elapsed_us, i64),
            name!(rescan_prepare_query_elapsed_us, i64),
            name!(rescan_reset_state_elapsed_us, i64),
            name!(rescan_initialize_entry_elapsed_us, i64),
            name!(rescan_upper_layer_seed_elapsed_us, i64),
            name!(rescan_layer0_seed_elapsed_us, i64),
            name!(rescan_stage_ordered_results_elapsed_us, i64),
            name!(rescan_initial_prefetch_elapsed_us, i64),
            name!(rescan_frontier_consume_elapsed_us, i64),
            name!(rescan_graph_result_materialize_elapsed_us, i64),
            name!(graph_element_cache_hits, i32),
            name!(graph_element_cache_misses, i32),
            name!(graph_element_load_elapsed_us, i64),
            name!(graph_neighbor_cache_hits, i32),
            name!(graph_neighbor_cache_misses, i32),
            name!(graph_neighbor_load_elapsed_us, i64),
            name!(candidate_score_calls, i32),
            name!(candidate_score_elapsed_us, i64),
            name!(score_cache_hits, i32),
            name!(score_cache_misses, i32),
            name!(pq_fastscan_traversal_approx_score_calls, i32),
            name!(pq_fastscan_traversal_approx_score_elapsed_us, i64),
            name!(pq_fastscan_traversal_exact_score_calls, i32),
            name!(pq_fastscan_traversal_exact_score_elapsed_us, i64),
            name!(pq_fastscan_traversal_budgeted_expansions, i32),
            name!(pq_fastscan_traversal_budgeted_candidates, i32),
            name!(pq_fastscan_traversal_budgeted_exact_candidates, i32),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_pq_fastscan_scan_hot_path_profile",
        );

        let (
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            pq_fastscan_traversal_approx_score_calls,
            pq_fastscan_traversal_approx_score_elapsed_us,
            pq_fastscan_traversal_exact_score_calls,
            pq_fastscan_traversal_exact_score_elapsed_us,
            pq_fastscan_traversal_budgeted_expansions,
            pq_fastscan_traversal_budgeted_candidates,
            pq_fastscan_traversal_budgeted_exact_candidates,
        ) = debug_scan_hot_path_profile_values(index_oid, query);

        TableIterator::once((
            rescan_amrescan_total_elapsed_us,
            rescan_query_decode_elapsed_us,
            rescan_scan_setup_elapsed_us,
            rescan_store_query_elapsed_us,
            rescan_prepare_query_elapsed_us,
            rescan_reset_state_elapsed_us,
            rescan_initialize_entry_elapsed_us,
            rescan_upper_layer_seed_elapsed_us,
            rescan_layer0_seed_elapsed_us,
            rescan_stage_ordered_results_elapsed_us,
            rescan_initial_prefetch_elapsed_us,
            rescan_frontier_consume_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            graph_element_cache_hits,
            graph_element_cache_misses,
            graph_element_load_elapsed_us,
            graph_neighbor_cache_hits,
            graph_neighbor_cache_misses,
            graph_neighbor_load_elapsed_us,
            candidate_score_calls,
            candidate_score_elapsed_us,
            score_cache_hits,
            score_cache_misses,
            pq_fastscan_traversal_approx_score_calls,
            pq_fastscan_traversal_approx_score_elapsed_us,
            pq_fastscan_traversal_exact_score_calls,
            pq_fastscan_traversal_exact_score_elapsed_us,
            pq_fastscan_traversal_budgeted_expansions,
            pq_fastscan_traversal_budgeted_candidates,
            pq_fastscan_traversal_budgeted_exact_candidates,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_pq_fastscan_rerank_profile(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        limit_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_amrescan_total_elapsed_us, i64),
            name!(rescan_graph_result_materialize_elapsed_us, i64),
            name!(emit_elapsed_us, i64),
            name!(total_elapsed_us, i64),
            name!(result_count, i32),
            name!(pq_fastscan_rerank_quantized_score_calls, i32),
            name!(pq_fastscan_rerank_quantized_score_elapsed_us, i64),
            name!(pq_fastscan_rerank_heap_score_calls, i32),
            name!(pq_fastscan_rerank_heap_score_elapsed_us, i64),
            name!(pq_fastscan_rerank_heap_rows_fetched, i32),
            name!(pq_fastscan_rerank_heap_fetch_elapsed_us, i64),
            name!(pq_fastscan_rerank_heap_decode_elapsed_us, i64),
            name!(pq_fastscan_rerank_heap_dot_elapsed_us, i64),
        ),
    > {
        validate_debug_index(index_oid, "tests.ec_hnsw_debug_pq_fastscan_rerank_profile");

        let (
            rescan_amrescan_total_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            result_count,
            pq_fastscan_rerank_quantized_score_calls,
            pq_fastscan_rerank_quantized_score_elapsed_us,
            pq_fastscan_rerank_heap_score_calls,
            pq_fastscan_rerank_heap_score_elapsed_us,
            pq_fastscan_rerank_heap_rows_fetched,
            pq_fastscan_rerank_heap_fetch_elapsed_us,
            pq_fastscan_rerank_heap_decode_elapsed_us,
            pq_fastscan_rerank_heap_dot_elapsed_us,
        ) = pq_fastscan_rerank_profile_values(index_oid, query, limit_count);

        TableIterator::once((
            rescan_amrescan_total_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            result_count,
            pq_fastscan_rerank_quantized_score_calls,
            pq_fastscan_rerank_quantized_score_elapsed_us,
            pq_fastscan_rerank_heap_score_calls,
            pq_fastscan_rerank_heap_score_elapsed_us,
            pq_fastscan_rerank_heap_rows_fetched,
            pq_fastscan_rerank_heap_fetch_elapsed_us,
            pq_fastscan_rerank_heap_decode_elapsed_us,
            pq_fastscan_rerank_heap_dot_elapsed_us,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_turboquant_scan_stage_profile(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_amrescan_total_elapsed_us, i64),
            name!(turboquant_traversal_residual_elapsed_us, i64),
            name!(turboquant_binary_prefilter_score_calls, i32),
            name!(turboquant_binary_prefilter_score_elapsed_us, i64),
            name!(turboquant_binary_prefilter_survivor_candidates, i32),
            name!(turboquant_exact_score_calls, i32),
            name!(turboquant_exact_score_elapsed_us, i64),
            name!(turboquant_rerank_score_calls, i32),
            name!(turboquant_rerank_score_elapsed_us, i64),
            name!(turboquant_exact_score_mode, String),
            name!(turboquant_exact_score_uses_lut, bool),
            name!(turboquant_exact_score_uses_qjl, bool),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_turboquant_scan_stage_profile",
        );

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
        ) = turboquant_scan_stage_profile_values(index_oid, query);

        TableIterator::once((
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
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_grouped_rerank_profile(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        limit_count: i32,
    ) -> TableIterator<
        'static,
        (
            name!(rescan_amrescan_total_elapsed_us, i64),
            name!(rescan_graph_result_materialize_elapsed_us, i64),
            name!(emit_elapsed_us, i64),
            name!(total_elapsed_us, i64),
            name!(result_count, i32),
            name!(grouped_rerank_quantized_score_calls, i32),
            name!(grouped_rerank_quantized_score_elapsed_us, i64),
            name!(grouped_rerank_heap_score_calls, i32),
            name!(grouped_rerank_heap_score_elapsed_us, i64),
            name!(grouped_rerank_heap_rows_fetched, i32),
            name!(grouped_rerank_heap_fetch_elapsed_us, i64),
            name!(grouped_rerank_heap_decode_elapsed_us, i64),
            name!(grouped_rerank_heap_dot_elapsed_us, i64),
        ),
    > {
        validate_debug_index(index_oid, "tests.ec_hnsw_debug_grouped_rerank_profile");

        let (
            rescan_amrescan_total_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            result_count,
            grouped_rerank_quantized_score_calls,
            grouped_rerank_quantized_score_elapsed_us,
            grouped_rerank_heap_score_calls,
            grouped_rerank_heap_score_elapsed_us,
            grouped_rerank_heap_rows_fetched,
            grouped_rerank_heap_fetch_elapsed_us,
            grouped_rerank_heap_decode_elapsed_us,
            grouped_rerank_heap_dot_elapsed_us,
        ) = pq_fastscan_rerank_profile_values(index_oid, query, limit_count);

        TableIterator::once((
            rescan_amrescan_total_elapsed_us,
            rescan_graph_result_materialize_elapsed_us,
            emit_elapsed_us,
            total_elapsed_us,
            result_count,
            grouped_rerank_quantized_score_calls,
            grouped_rerank_quantized_score_elapsed_us,
            grouped_rerank_heap_score_calls,
            grouped_rerank_heap_score_elapsed_us,
            grouped_rerank_heap_rows_fetched,
            grouped_rerank_heap_fetch_elapsed_us,
            grouped_rerank_heap_decode_elapsed_us,
            grouped_rerank_heap_dot_elapsed_us,
        ))
    }

    #[pg_extern]
    fn ec_hnsw_debug_scan_result_count(index_oid: pg_sys::Oid, query: Vec<f32>) -> i32 {
        drop(open_valid_ec_hnsw_index_guard(
            index_oid,
            "tests.ec_hnsw_debug_scan_result_count",
        ));

        i32::try_from(unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query) }.len())
            .expect("debug scan result count should fit in i32")
    }

    #[pg_extern]
    fn ec_hnsw_debug_scan_heap_tids(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<'static, (name!(block_number, i64), name!(offset_number, i32))> {
        drop(open_valid_ec_hnsw_index_guard(
            index_oid,
            "tests.ec_hnsw_debug_scan_heap_tids",
        ));

        let rows = unsafe { am::debug_gettuple_scan_heap_tids(index_oid, query) }
            .into_iter()
            .map(|(block_number, offset_number)| {
                (i64::from(block_number), i32::from(offset_number))
            })
            .collect::<Vec<_>>();
        TableIterator::new(rows)
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_pq_fastscan_scan_order_drift_summary(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(emitted_result_count, i32),
            name!(pq_fastscan_result_count, i32),
            name!(compared_result_count, i32),
            name!(mean_abs_rank_shift, f64),
            name!(max_abs_rank_shift, i32),
            name!(spearman_rank_correlation, f64),
            name!(exact_best_approx_rank, Option<i32>),
            name!(exact_top4_max_approx_rank, Option<i32>),
            name!(window_1_contains_exact_best, bool),
            name!(window_2_contains_exact_best, bool),
            name!(window_4_contains_exact_best, bool),
            name!(window_8_contains_exact_best, bool),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_pq_fastscan_scan_order_drift_summary",
        );

        let (
            emitted_result_count,
            pq_fastscan_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ) = pq_fastscan_scan_order_drift_summary_values(index_oid, query);

        TableIterator::once((
            emitted_result_count,
            pq_fastscan_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_grouped_scan_order_drift_summary(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(emitted_result_count, i32),
            name!(grouped_result_count, i32),
            name!(compared_result_count, i32),
            name!(mean_abs_rank_shift, f64),
            name!(max_abs_rank_shift, i32),
            name!(spearman_rank_correlation, f64),
            name!(exact_best_approx_rank, Option<i32>),
            name!(exact_top4_max_approx_rank, Option<i32>),
            name!(window_1_contains_exact_best, bool),
            name!(window_2_contains_exact_best, bool),
            name!(window_4_contains_exact_best, bool),
            name!(window_8_contains_exact_best, bool),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_grouped_scan_order_drift_summary",
        );

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ) = pq_fastscan_scan_order_drift_summary_values(index_oid, query);

        TableIterator::once((
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            mean_abs_rank_shift,
            max_abs_rank_shift,
            spearman_rank_correlation,
            exact_best_approx_rank,
            exact_top4_max_approx_rank,
            window_1_contains_exact_best,
            window_2_contains_exact_best,
            window_4_contains_exact_best,
            window_8_contains_exact_best,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_pq_fastscan_scan_windowed_rows(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        window_size: i32,
    ) -> TableIterator<
        'static,
        (
            name!(block_number, i64),
            name!(offset_number, i32),
            name!(approx_rank, i32),
            name!(windowed_rank, i32),
            name!(approx_score, f32),
            name!(comparison_score, Option<f32>),
            name!(exact_rank, Option<i32>),
            name!(exact_rank_shift, Option<i32>),
            name!(windowed_rank_shift, Option<i32>),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_pq_fastscan_scan_windowed_rows",
        );
        TableIterator::new(pq_fastscan_scan_windowed_rows_values(
            index_oid,
            query,
            window_size,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_grouped_scan_windowed_rows(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        window_size: i32,
    ) -> TableIterator<
        'static,
        (
            name!(block_number, i64),
            name!(offset_number, i32),
            name!(approx_rank, i32),
            name!(windowed_rank, i32),
            name!(approx_score, f32),
            name!(comparison_score, Option<f32>),
            name!(exact_rank, Option<i32>),
            name!(exact_rank_shift, Option<i32>),
            name!(windowed_rank_shift, Option<i32>),
        ),
    > {
        validate_debug_index(index_oid, "tests.ec_hnsw_debug_grouped_scan_windowed_rows");
        TableIterator::new(pq_fastscan_scan_windowed_rows_values(
            index_oid,
            query,
            window_size,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_pq_fastscan_scan_windowed_summary(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        window_size: i32,
    ) -> TableIterator<
        'static,
        (
            name!(emitted_result_count, i32),
            name!(pq_fastscan_result_count, i32),
            name!(compared_result_count, i32),
            name!(window_size, i32),
            name!(exact_best_approx_rank, Option<i32>),
            name!(exact_best_windowed_rank, Option<i32>),
            name!(exact_top4_max_approx_rank, Option<i32>),
            name!(exact_top4_max_windowed_rank, Option<i32>),
            name!(mean_abs_rank_shift_before, f64),
            name!(mean_abs_rank_shift_after, f64),
            name!(max_abs_rank_shift_before, i32),
            name!(max_abs_rank_shift_after, i32),
            name!(spearman_rank_correlation_before, f64),
            name!(spearman_rank_correlation_after, f64),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_pq_fastscan_scan_windowed_summary",
        );

        let (
            emitted_result_count,
            pq_fastscan_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_rank_correlation_before,
            spearman_rank_correlation_after,
        ) = pq_fastscan_scan_windowed_summary_values(index_oid, query, window_size);

        TableIterator::once((
            emitted_result_count,
            pq_fastscan_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_rank_correlation_before,
            spearman_rank_correlation_after,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_grouped_scan_windowed_summary(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
        window_size: i32,
    ) -> TableIterator<
        'static,
        (
            name!(emitted_result_count, i32),
            name!(grouped_result_count, i32),
            name!(compared_result_count, i32),
            name!(window_size, i32),
            name!(exact_best_approx_rank, Option<i32>),
            name!(exact_best_windowed_rank, Option<i32>),
            name!(exact_top4_max_approx_rank, Option<i32>),
            name!(exact_top4_max_windowed_rank, Option<i32>),
            name!(mean_abs_rank_shift_before, f64),
            name!(mean_abs_rank_shift_after, f64),
            name!(max_abs_rank_shift_before, i32),
            name!(max_abs_rank_shift_after, i32),
            name!(spearman_rank_correlation_before, f64),
            name!(spearman_rank_correlation_after, f64),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_grouped_scan_windowed_summary",
        );

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_rank_correlation_before,
            spearman_rank_correlation_after,
        ) = pq_fastscan_scan_windowed_summary_values(index_oid, query, window_size);

        TableIterator::once((
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            window_size,
            exact_best_approx_rank,
            exact_best_windowed_rank,
            exact_top4_max_approx_rank,
            exact_top4_max_windowed_rank,
            mean_abs_rank_shift_before,
            mean_abs_rank_shift_after,
            max_abs_rank_shift_before,
            max_abs_rank_shift_after,
            spearman_rank_correlation_before,
            spearman_rank_correlation_after,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_pq_fastscan_scan_comparison_rows(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(block_number, i64),
            name!(offset_number, i32),
            name!(approx_rank, i32),
            name!(approx_score, f32),
            name!(comparison_score, Option<f32>),
            name!(exact_rank, Option<i32>),
            name!(exact_rank_shift, Option<i32>),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_pq_fastscan_scan_comparison_rows",
        );
        TableIterator::new(pq_fastscan_scan_comparison_rows_values(index_oid, query))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_grouped_scan_comparison_rows(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(block_number, i64),
            name!(offset_number, i32),
            name!(approx_rank, i32),
            name!(approx_score, f32),
            name!(comparison_score, Option<f32>),
            name!(exact_rank, Option<i32>),
            name!(exact_rank_shift, Option<i32>),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_grouped_scan_comparison_rows",
        );
        TableIterator::new(pq_fastscan_scan_comparison_rows_values(index_oid, query))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_pq_fastscan_scan_comparison_summary(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(emitted_result_count, i32),
            name!(pq_fastscan_result_count, i32),
            name!(compared_result_count, i32),
            name!(missing_comparison_count, i32),
            name!(mean_abs_score_delta, f64),
            name!(max_abs_score_delta, f32),
            name!(mean_signed_score_delta, f64),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_pq_fastscan_scan_comparison_summary",
        );

        let (
            emitted_result_count,
            pq_fastscan_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ) = pq_fastscan_scan_comparison_summary_values(index_oid, query);

        TableIterator::once((
            emitted_result_count,
            pq_fastscan_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ))
    }

    #[pg_extern]
    #[allow(clippy::type_complexity)]
    fn ec_hnsw_debug_grouped_scan_comparison_summary(
        index_oid: pg_sys::Oid,
        query: Vec<f32>,
    ) -> TableIterator<
        'static,
        (
            name!(emitted_result_count, i32),
            name!(grouped_result_count, i32),
            name!(compared_result_count, i32),
            name!(missing_comparison_count, i32),
            name!(mean_abs_score_delta, f64),
            name!(max_abs_score_delta, f32),
            name!(mean_signed_score_delta, f64),
        ),
    > {
        validate_debug_index(
            index_oid,
            "tests.ec_hnsw_debug_grouped_scan_comparison_summary",
        );

        let (
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ) = pq_fastscan_scan_comparison_summary_values(index_oid, query);

        TableIterator::once((
            emitted_result_count,
            grouped_result_count,
            compared_result_count,
            missing_comparison_count,
            mean_abs_score_delta,
            max_abs_score_delta,
            mean_signed_score_delta,
        ))
    }

    #[pg_extern]
    fn ec_hnsw_debug_reachable_live_element_count(index_oid: pg_sys::Oid) -> i32 {
        drop(open_valid_ec_hnsw_index_guard(
            index_oid,
            "tests.ec_hnsw_debug_reachable_live_element_count",
        ));

        i32::try_from(unsafe { am::debug_layer0_reachable_live_element_tids(index_oid) }.len())
            .expect("debug reachable live element count should fit in i32")
    }
