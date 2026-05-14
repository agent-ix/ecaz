    #[pg_test]
    fn test_raw_source_build_coalesces_duplicate_vectors() {
        Spi::run(
            "CREATE TABLE ec_hnsw_duplicate_source_build (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_duplicate_source_build VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], encode_to_ecvector(ARRAY[0.5, 0.2, 0.1, 0.0], 4, 42)),
             (2, ARRAY[0.0, 1.0, 0.0, 0.0], encode_to_ecvector(ARRAY[0.5, 0.2, 0.1, 0.0], 4, 42)),
             (3, ARRAY[-1.0, 0.0, 0.0, 0.0], encode_to_ecvector(ARRAY[-0.6, -0.1, 0.0, 0.3], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_duplicate_source_build_idx ON ec_hnsw_duplicate_source_build USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_duplicate_source_build_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);

        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4))
                .into_iter()
                .map(|(_, element)| element)
                .collect::<Vec<_>>();

        assert_eq!(
            elements.len(),
            2,
            "duplicate encoded vectors should share one element tuple"
        );
        let mut heaptid_counts = elements
            .iter()
            .map(|element| element.heaptids.len())
            .collect::<Vec<_>>();
        heaptid_counts.sort_unstable();
        assert_eq!(heaptid_counts, vec![1, 2]);
    }

    #[pg_test]
    #[should_panic(expected = "does not name a user column")]
    fn test_raw_source_rejects_missing_column() {
        Spi::run(
            "CREATE TABLE ec_hnsw_bad_source_column (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_bad_source_column VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_bad_source_column_idx ON ec_hnsw_bad_source_column USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'missing')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "must be real[] or ecvector")]
    fn test_raw_source_rejects_wrong_type() {
        Spi::run(
            "CREATE TABLE ec_hnsw_bad_source_type (
                id bigint primary key,
                source double precision[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_bad_source_type VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0]::double precision[], encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_bad_source_type_idx ON ec_hnsw_bad_source_type USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "must be real[], bytea, or ecvector")]
    fn test_turboquant_rerank_source_rejects_wrong_type() {
        Spi::run(
            "CREATE TABLE ec_hnsw_bad_rerank_source_type_turboquant (
                id bigint primary key,
                source real[],
                source_raw text,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_bad_rerank_source_type_turboquant VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], 'not-bytea', encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_bad_rerank_source_type_turboquant_idx ON ec_hnsw_bad_rerank_source_type_turboquant USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source', rerank_source_column = 'source_raw')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "does not name a user column")]
    fn test_pq_fastscan_rerank_source_rejects_missing_column() {
        Spi::run(
            "CREATE TABLE ec_hnsw_bad_rerank_source_column (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_bad_rerank_source_column VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_bad_rerank_source_column_idx ON ec_hnsw_bad_rerank_source_column USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source', rerank_source_column = 'missing', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "must be real[], bytea, or ecvector")]
    fn test_pq_fastscan_rerank_source_rejects_wrong_type() {
        Spi::run(
            "CREATE TABLE ec_hnsw_bad_rerank_source_type (
                id bigint primary key,
                source real[],
                source_raw text,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_bad_rerank_source_type VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], 'not-bytea', encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_bad_rerank_source_type_idx ON ec_hnsw_bad_rerank_source_type USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source', rerank_source_column = 'source_raw', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    fn test_pq_fastscan_build_source_accepts_ecvector() {
        let table_name = "ec_hnsw_pq_fastscan_runtime_ecvector_source";
        let index_name = "ec_hnsw_pq_fastscan_runtime_ecvector_source_idx";
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key,
                source ecvector,
                embedding ecvector
            )"
        ))
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| (((id * 31 + dim) as f32) * 0.03).cos())
                .collect::<Vec<_>>();
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 19 + dim) as f32) * 0.02).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let source = format_recall_vector_sql_literal(&source);
            Spi::run(&format!(
                "INSERT INTO {table_name} VALUES \
                 ({id}, ({source})::ecvector, encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', storage_format = 'pq_fastscan')"
        ))
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let query = vec![
            0.1_f32, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6,
        ];
        let observed = unsafe {
            am::debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query.clone())
        };
        let exact_scores = (1..=16)
            .map(|id| {
                let source = (0..16)
                    .map(|dim| (((id * 31 + dim) as f32) * 0.03).cos())
                    .collect::<Vec<_>>();
                let heap_tid = heap_tid_for_row(table_name, id);
                (
                    (heap_tid.block_number, heap_tid.offset_number),
                    -dot_product(&query, &source),
                )
            })
            .collect::<HashMap<_, _>>();

        assert!(
            !observed.is_empty(),
            "PqFastScan build_source_column should accept ecvector and emit ordered results",
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score
                .expect("ecvector build_source_column scans should attach exact comparison scores");
            let expected = exact_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact source score");
            assert_f32_close(
                comparison_score,
                expected,
                "ecvector build_source_column should preserve the exact heap comparison score",
            );
        }
    }

    #[pg_test]
    fn test_pq_fastscan_persisted_ecvector_rerank_emits_scores() {
        let table_name = "ec_hnsw_pq_fastscan_runtime_persisted_heap_rerank_ecvector";
        let index_name = "ec_hnsw_pq_fastscan_runtime_persisted_heap_rerank_ecvector_idx";
        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key,
                source real[],
                source_raw ecvector,
                embedding ecvector
            )"
        ))
        .expect("table creation should succeed");

        for id in 1..=16 {
            let source = format_recall_vector_sql_literal(&pq_fastscan_binary_runtime_source(id));
            let rerank_source =
                format_recall_vector_sql_literal(&turboquant_binary_runtime_rerank_source(id));
            let embedding =
                format_recall_vector_sql_literal(&pq_fastscan_binary_runtime_embedding(id));
            Spi::run(&format!(
                "INSERT INTO {table_name} VALUES \
                 ({id}, {source}, ({rerank_source})::ecvector, encode_to_ecvector({embedding}, 4, 42))"
            ))
            .expect("insert should succeed");
        }

        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 6, ef_construction = 80, build_source_column = 'source', rerank_source_column = 'source_raw', storage_format = 'pq_fastscan')"
        ))
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("SPI query should succeed")
                .expect("index oid should exist");
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

        assert!(
            !observed.is_empty(),
            "PqFastScan should accept persisted ecvector rerank sources and emit ordered results",
        );
        for (heap_tid, _approx_score, comparison_score, _approx_rank) in observed {
            let comparison_score = comparison_score.expect(
                "a persisted ecvector rerank_source_column should attach exact heap comparison scores",
            );
            let expected = rerank_scores
                .get(&heap_tid)
                .copied()
                .expect("every emitted heap tid should map back to an exact rerank-source score");
            assert_f32_close(
                comparison_score,
                expected,
                "a persisted ecvector rerank_source_column should use the raw heap f32 inner product",
            );
            assert_f32_close(
                emitted_scores
                    .get(&heap_tid)
                    .copied()
                    .expect("emitted score should be present for every observed heap tid"),
                expected,
                "a persisted ecvector rerank_source_column should emit the exact heap comparison score as the order-by score",
            );
        }
    }

    #[pg_test]
    #[should_panic(expected = "dimension mismatch")]
    fn test_raw_source_rejects_dimension_mismatch() {
        Spi::run(
            "CREATE TABLE ec_hnsw_bad_source_dim (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_bad_source_dim VALUES
             (1, ARRAY[1.0, 0.0, 0.0], encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_bad_source_dim_idx ON ec_hnsw_bad_source_dim USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "does not support NULL ec_hnsw build_source_column")]
    fn test_raw_source_rejects_null_value() {
        Spi::run(
            "CREATE TABLE ec_hnsw_null_source (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_null_source VALUES
             (1, NULL, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_null_source_idx ON ec_hnsw_null_source USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "does not support expression indexes yet")]
    fn test_raw_source_rejects_expression_index() {
        Spi::run(
            "CREATE TABLE ec_hnsw_expression_source (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_expression_source VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_expression_source_idx ON ec_hnsw_expression_source USING ec_hnsw \
             (((embedding::text)::ecvector) ecvector_ip_ops) WITH (build_source_column = 'source')",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "does not support partial indexes yet")]
    fn test_raw_source_rejects_partial_index() {
        Spi::run(
            "CREATE TABLE ec_hnsw_partial_source (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_partial_source VALUES
             (1, ARRAY[1.0, 0.0, 0.0, 0.0], encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, ARRAY[0.0, 1.0, 0.0, 0.0], encode_to_ecvector(ARRAY[0.0, 1.0, 0.0, 0.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_partial_source_idx ON ec_hnsw_partial_source USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source') WHERE id > 1",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    fn test_non_empty_index_build_spans_multiple_data_pages() {
        Spi::run(
            "CREATE TABLE ec_hnsw_multipage_build (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");

        let dim = 256_usize;
        let bits = 4_u8;
        let payload_len = code_len(dim, bits);
        for id in 1..=128 {
            let code = (0..payload_len)
                .map(|offset| ((id * 17 + offset as i32) & 0xff) as u8)
                .collect::<Vec<_>>();
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_multipage_build VALUES \
                 ({id}, '[dim={dim},bits={bits},seed=42,gamma=0.5]:{payload}'::tqvector)",
                payload = hex::encode(code),
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_multipage_build_idx ON ec_hnsw_multipage_build USING ec_hnsw \
             (embedding tqvector_ip_ops) WITH (m = 4, ef_construction = 64)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_multipage_build_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert!(block_count > 2, "build should span more than one data page");
        assert_eq!(metadata.dimensions, dim as u16);
        assert_eq!(metadata.bits, bits);
        assert_eq!(metadata.seed, 42);
        assert!(metadata.max_level <= am::page::default_max_level_cap(metadata.m));

        let page_tuples = data_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().map(move |(idx, tuple)| {
                    (
                        am::page::ItemPointer {
                            block_number: page.block_number,
                            offset_number: (idx + 1) as u16,
                        },
                        tuple.as_slice(),
                    )
                })
            })
            .collect::<Vec<_>>();

        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(dim, bits));
        let neighbors = page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                if tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG) {
                    Some((
                        *tid,
                        am::page::TqNeighborTuple::decode(tuple)
                            .expect("neighbor tuple should decode"),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let element_tids = elements.iter().map(|(tid, _)| *tid).collect::<Vec<_>>();
        let covered_heap_tids = elements
            .iter()
            .map(|(_, element)| element.heaptids.len())
            .sum::<usize>();
        let entry_element = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("entry point should identify an element tuple");

        assert!(elements.len() > 1);
        assert_eq!(neighbors.len(), elements.len());
        assert_eq!(covered_heap_tids, 128);
        assert_eq!(entry_element.1.level, metadata.max_level);
        assert!(
            data_pages
                .iter()
                .filter(|page| !page.tuples.is_empty())
                .count()
                > 1,
            "more than one populated data page should exist"
        );

        let neighbor_map = neighbors
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();
        for (_, element) in &elements {
            let neighbor = neighbor_map
                .get(&element.neighbortid)
                .expect("neighbor tuple should exist");
            assert_eq!(neighbor.count as usize, neighbor.tids.len());
            assert!(neighbor.tids.len() <= am::page::neighbor_slots(element.level, metadata.m));
            assert!(neighbor.tids.iter().all(|tid| {
                *tid == am::page::ItemPointer::INVALID || element_tids.contains(tid)
            }));
        }
    }

    #[pg_test]
    fn test_build_keeps_element_neighbor_local() {
        Spi::run("CREATE TABLE ec_hnsw_build_locality (id bigint primary key, embedding tqvector)")
            .expect("table creation should succeed");

        let dim = 256_usize;
        let bits = 4_u8;
        let payload_len = code_len(dim, bits);
        for id in 1..=128 {
            let code = (0..payload_len)
                .map(|offset| ((id * 29 + offset as i32) & 0xff) as u8)
                .collect::<Vec<_>>();
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_build_locality VALUES \
                 ({id}, '[dim={dim},bits={bits},seed=42,gamma=0.5]:{payload}'::tqvector)",
                payload = hex::encode(code),
            ))
            .expect("insert should succeed");
        }

        Spi::run(
            "CREATE INDEX ec_hnsw_build_locality_idx ON ec_hnsw_build_locality USING ec_hnsw \
             (embedding tqvector_ip_ops) WITH (m = 4, ef_construction = 64)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_build_locality_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(dim, bits));

        assert!(
            !elements.is_empty(),
            "build should persist at least one element tuple"
        );
        assert!(elements
            .iter()
            .any(|(tid, element)| { element.neighbortid.block_number == tid.block_number }));
        assert!(elements.iter().all(|(tid, element)| {
            element.neighbortid.block_number <= tid.block_number
                && tid.block_number - element.neighbortid.block_number <= 1
        }),
        "build should keep each element tuple on the same page as its neighbor tuple or on the immediately following page");
    }

    #[pg_test]
    fn test_non_empty_index_build_coalesces_duplicate_vectors() {
        Spi::run(
            "CREATE TABLE ec_hnsw_duplicate_build (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_duplicate_build VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, -2.0, -3.0, -4.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_duplicate_build_idx ON ec_hnsw_duplicate_build USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_duplicate_build_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);

        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4))
                .into_iter()
                .map(|(_, element)| element)
                .collect::<Vec<_>>();

        assert_eq!(
            elements.len(),
            2,
            "duplicate encoded vectors should share one element tuple"
        );
        let mut heaptid_counts = elements
            .iter()
            .map(|element| element.heaptids.len())
            .collect::<Vec<_>>();
        heaptid_counts.sort_unstable();
        assert_eq!(heaptid_counts, vec![1, 2]);
    }

    #[pg_test]
    fn test_non_empty_index_build_keeps_gamma_distinct() {
        Spi::run(
            "CREATE TABLE ec_hnsw_duplicate_build_gamma (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_duplicate_build_gamma VALUES
             (1, '[dim=4,bits=4,seed=42,gamma=0.5]:112233'::tqvector),
             (2, '[dim=4,bits=4,seed=42,gamma=1.5]:112233'::tqvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_duplicate_build_gamma_idx ON ec_hnsw_duplicate_build_gamma USING ec_hnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_duplicate_build_gamma_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);

        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4))
                .into_iter()
                .map(|(_, element)| element)
                .collect::<Vec<_>>();

        assert_eq!(
            elements.len(),
            2,
            "same-code build inputs with distinct persisted gamma values must not coalesce"
        );
        assert!(elements.iter().all(|element| element.heaptids.len() == 1));
        let mut gammas = elements
            .iter()
            .map(|element| element.gamma.to_bits())
            .collect::<Vec<_>>();
        gammas.sort_unstable();
        assert_eq!(
            gammas,
            vec![0.5_f32.to_bits(), 1.5_f32.to_bits()],
            "build should persist element gamma values alongside same-code distinct tuples"
        );
    }

    #[pg_test]
    fn test_ech_insert_appends_new_element_tuple() {
        Spi::run("CREATE TABLE ec_hnsw_insert_append (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_append VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_append_idx ON ec_hnsw_insert_append USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_append VALUES
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("insert should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_insert_append_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);

        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4))
                .into_iter()
                .map(|(_, element)| element)
                .collect::<Vec<_>>();

        assert_eq!(elements.len(), 2);
        assert!(elements
            .iter()
            .all(|element| element.level == 0 || element.level <= metadata.max_level));
        assert!(elements.iter().any(|element| element.heaptids.len() == 1));
    }

    #[pg_test]
    fn test_ech_insert_reuses_tail_page_when_space_remains() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_tail_reuse (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_tail_reuse VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_tail_reuse_idx ON ec_hnsw_insert_tail_reuse USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_insert_tail_reuse_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (before_block_count, _metadata, _data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            before_block_count, 2,
            "seed build should fit on one data page"
        );

        Spi::run(
            "INSERT INTO ec_hnsw_insert_tail_reuse VALUES
             (4, encode_to_ecvector(ARRAY[0.5, -0.5, 0.1, 0.2], 4, 42))",
        )
        .expect("insert should succeed");

        let (after_block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            after_block_count, before_block_count,
            "insert should reuse existing tail page"
        );
        assert_eq!(metadata.seed, 42);

        let tuple_count = data_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();
        assert_eq!(
            tuple_count, 12,
            "three build tuples plus one inserted tuple should store four hot/rerank/neighbor triplets"
        );
    }

    #[pg_test]
    fn test_ech_insert_allocates_new_page_when_tail_is_full() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_new_page (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");

        let default_m = 8_u16;
        let large_dim = (1_u16..=u16::MAX)
            .rev()
            .find(|dim| {
                let code_len = code_len(*dim as usize, 4);
                let binary_word_count = turboquant_v3_binary_word_count(*dim as usize, 4);
                let required_bytes =
                    turboquant_v3_triplet_storage_bytes(0, default_m, code_len, binary_word_count);
                if required_bytes
                    > (pg_sys::BLCKSZ as usize).saturating_sub(am::page::PAGE_HEADER_BYTES)
                {
                    return false;
                }
                let mut staged_page = am::page::DataPage::new(
                    am::page::FIRST_DATA_BLOCK_NUMBER,
                    pg_sys::BLCKSZ as usize,
                );
                let neighbor_slot_count = am::page::neighbor_slots(0, default_m);
                let neighbor = am::page::TqNeighborTuple {
                    count: neighbor_slot_count as u16,
                    tids: vec![am::page::ItemPointer::INVALID; neighbor_slot_count],
                };
                let code = vec![0x11_u8; code_len];
                let rerank = am::page::TqRerankTuple {
                    gamma: 0.5,
                    code: code.clone(),
                };
                let hot = am::page::TqTurboHotTuple {
                    level: 0,
                    deleted: false,
                    heaptids: vec![am::page::ItemPointer {
                        block_number: 1,
                        offset_number: 1,
                    }],
                    neighbortid: am::page::ItemPointer::INVALID,
                    reranktid: am::page::ItemPointer::INVALID,
                    binary_words: vec![0_u64; binary_word_count],
                };
                staged_page.insert_neighbor(&neighbor).is_ok()
                    && staged_page.insert_rerank(&rerank).is_ok()
                    && staged_page.insert_turbo_hot(&hot).is_ok()
                    && staged_page.free_bytes() < required_bytes
            })
            .expect("should find a dimension that saturates one data page");
        let large_code_len = code_len(large_dim as usize, 4);
        let first_code = vec![0x11_u8; large_code_len];
        let second_code = vec![0x22_u8; large_code_len];
        Spi::run(&format!(
            "INSERT INTO ec_hnsw_insert_new_page VALUES
             (1, '[dim={large_dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
            hex::encode(first_code),
        ))
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_new_page_idx ON ec_hnsw_insert_new_page USING ec_hnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_insert_new_page_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (before_block_count, metadata, _data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            before_block_count, 2,
            "seed build should occupy one data page"
        );
        assert_eq!(metadata.dimensions, large_dim);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);

        Spi::run(&format!(
            "INSERT INTO ec_hnsw_insert_new_page VALUES
             (2, '[dim={large_dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
            hex::encode(second_code),
        ))
        .expect("insert should succeed");

        let (after_block_count, _metadata, data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert!(
            after_block_count > before_block_count,
            "insert should allocate a new data page when the tail page is full"
        );
        assert_eq!(data_pages.len(), 2, "index should now have two data pages");
    }

    #[pg_test]
    fn test_ech_insert_reuses_new_tail_page_after_rollover() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_rollover_reuse (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");

        let m = 2_u16;
        let (dim, elements_per_page) = (1_u16..=u16::MAX)
            .rev()
            .find_map(|dim| {
                let code_len = code_len(dim as usize, 4);
                let binary_word_count = turboquant_v3_binary_word_count(dim as usize, 4);
                if turboquant_v3_triplet_storage_bytes(0, m, code_len, binary_word_count)
                    > (pg_sys::BLCKSZ as usize).saturating_sub(am::page::PAGE_HEADER_BYTES)
                {
                    return None;
                }
                let mut staged_page = am::page::DataPage::new(
                    am::page::FIRST_DATA_BLOCK_NUMBER,
                    pg_sys::BLCKSZ as usize,
                );
                let insert_triplet_fits = |page: &mut am::page::DataPage, offset_number: u16| {
                    let heap_tid = am::page::ItemPointer {
                        block_number: 0,
                        offset_number,
                    };
                    let level = am::debug_insert_level_for_heap_tid(m, 42, heap_tid, code_len);
                    let neighbor_slots = am::page::neighbor_slots(level, m);
                    let neighbor = am::page::TqNeighborTuple {
                        count: u16::try_from(neighbor_slots)
                            .expect("neighbor slot count should fit in u16"),
                        tids: vec![am::page::ItemPointer::INVALID; neighbor_slots],
                    };
                    let code = vec![0x11_u8; code_len];
                    let rerank = am::page::TqRerankTuple {
                        gamma: 0.5,
                        code: code.clone(),
                    };
                    let hot = am::page::TqTurboHotTuple {
                        level,
                        deleted: false,
                        heaptids: vec![heap_tid],
                        neighbortid: am::page::ItemPointer::INVALID,
                        reranktid: am::page::ItemPointer::INVALID,
                        binary_words: vec![0_u64; binary_word_count],
                    };
                    page.insert_neighbor(&neighbor).is_ok()
                        && page.insert_rerank(&rerank).is_ok()
                        && page.insert_turbo_hot(&hot).is_ok()
                };
                let mut elements = 0_usize;
                while insert_triplet_fits(
                    &mut staged_page,
                    u16::try_from(elements + 1).expect("offset should fit in u16"),
                ) {
                    elements += 1;
                }

                let mut next_page = am::page::DataPage::new(
                    am::page::FIRST_DATA_BLOCK_NUMBER + 1,
                    pg_sys::BLCKSZ as usize,
                );
                let next_offset = u16::try_from(elements + 1).expect("offset should fit in u16");
                let reuse_offset = u16::try_from(elements + 2).expect("offset should fit in u16");
                if elements >= 2
                    && insert_triplet_fits(&mut next_page, next_offset)
                    && insert_triplet_fits(&mut next_page, reuse_offset)
                {
                    Some((dim, elements))
                } else {
                    None
                }
            })
            .expect("should find a dimension where one page fits multiple turboquant triplets");
        let code_len = code_len(dim as usize, 4);

        Spi::run(&format!(
            "CREATE INDEX ec_hnsw_insert_rollover_reuse_idx ON ec_hnsw_insert_rollover_reuse USING ec_hnsw \
             (embedding tqvector_ip_ops) WITH (m = {m})"
        ))
        .expect("index creation should succeed");

        for id in 1..=elements_per_page {
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_rollover_reuse VALUES
                 ({id}, '[dim={dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
                hex::encode(vec![id as u8; code_len]),
            ))
            .expect("live insert should succeed");
        }

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_rollover_reuse_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_block_count, metadata, before_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.dimensions, dim);
        assert!(
            !before_pages.is_empty(),
            "live inserts should create at least one data page"
        );

        Spi::run(&format!(
            "INSERT INTO ec_hnsw_insert_rollover_reuse VALUES
             ({}, '[dim={dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
            elements_per_page + 1,
            hex::encode(vec![0xaa_u8; code_len]),
        ))
        .expect("rollover insert should succeed");

        let (after_rollover_block_count, _metadata, after_rollover_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert!(
            after_rollover_block_count > before_block_count,
            "insert should allocate a new page once the original tail page is full"
        );
        assert_eq!(after_rollover_pages.len(), 2);

        Spi::run(&format!(
            "INSERT INTO ec_hnsw_insert_rollover_reuse VALUES
             ({}, '[dim={dim},bits=4,seed=42,gamma=0.5]:{}'::tqvector)",
            elements_per_page + 2,
            hex::encode(vec![0xbb_u8; code_len]),
        ))
        .expect("post-rollover insert should succeed");

        let (after_reuse_block_count, _metadata, after_reuse_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            after_reuse_block_count, after_rollover_block_count,
            "insert after rollover should reuse the new tail page when space remains"
        );
        assert_eq!(after_reuse_pages.len(), 2);
    }

    #[pg_test]
    fn test_ech_insert_coalesces_duplicate_vectors() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_duplicate (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_duplicate VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[-1.0, -2.0, -3.0, -4.0], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_duplicate_idx ON ec_hnsw_insert_duplicate USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_insert_duplicate_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (before_block_count, metadata, data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(metadata.seed, 42);
        let before_tuple_count = data_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();

        Spi::run(
            "INSERT INTO ec_hnsw_insert_duplicate VALUES
             (3, encode_to_ecvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))",
        )
        .expect("duplicate insert should succeed");

        let (after_block_count, _metadata, data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            after_block_count, before_block_count,
            "duplicate insert should not allocate a new block"
        );
        let after_tuple_count = data_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();
        assert_eq!(
            after_tuple_count, before_tuple_count,
            "duplicate insert should not add a new tuple pair"
        );

        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4))
                .into_iter()
                .map(|(_, element)| element)
                .collect::<Vec<_>>();
        let mut heaptid_counts = elements
            .iter()
            .map(|element| element.heaptids.len())
            .collect::<Vec<_>>();
        heaptid_counts.sort_unstable();
        assert_eq!(heaptid_counts, vec![1, 2]);
    }

    #[pg_test]
    fn test_ech_insert_keeps_gamma_distinct() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_duplicate_gamma (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_duplicate_gamma VALUES
             (1, '[dim=4,bits=4,seed=42,gamma=0.5]:112233'::tqvector)",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_duplicate_gamma_idx ON ec_hnsw_insert_duplicate_gamma USING ec_hnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_duplicate_gamma_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (before_block_count, metadata, before_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        let before_tuple_count = before_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();

        Spi::run(
            "INSERT INTO ec_hnsw_insert_duplicate_gamma VALUES
             (2, '[dim=4,bits=4,seed=42,gamma=1.5]:112233'::tqvector)",
        )
        .expect("gamma-distinct insert should succeed");

        let (after_block_count, _metadata, data_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(
            after_block_count, before_block_count,
            "gamma-distinct same-code inserts should stay on the current tail page in this narrow test"
        );
        let after_tuple_count = data_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();
        assert_eq!(
            after_tuple_count,
            before_tuple_count + 3,
            "gamma-distinct same-code inserts should append a fresh hot/rerank/neighbor triplet"
        );

        let elements =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4))
                .into_iter()
                .map(|(_, element)| element)
                .collect::<Vec<_>>();

        assert_eq!(
            elements.len(),
            2,
            "same-code inserts with distinct persisted gamma values must not coalesce"
        );
        assert!(elements.iter().all(|element| element.heaptids.len() == 1));
        let mut gammas = elements
            .iter()
            .map(|element| element.gamma.to_bits())
            .collect::<Vec<_>>();
        gammas.sort_unstable();
        assert_eq!(
            gammas,
            vec![0.5_f32.to_bits(), 1.5_f32.to_bits()],
            "live insert should persist element gamma values alongside same-code distinct tuples"
        );
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw does not support non-finite gamma values")]
    fn test_non_empty_index_build_rejects_non_finite_gamma() {
        Spi::run(
            "CREATE TABLE ec_hnsw_build_nan_gamma (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_build_nan_gamma VALUES
             (1, '[dim=4,bits=4,seed=42,gamma=NaN]:112233'::tqvector)",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_build_nan_gamma_idx ON ec_hnsw_build_nan_gamma USING ec_hnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should fail");
    }

    #[pg_test]
    #[should_panic(expected = "ec_hnsw does not support non-finite gamma values")]
    fn test_ech_insert_rejects_non_finite_gamma() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_nan_gamma (id bigint primary key, embedding tqvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_nan_gamma_idx ON ec_hnsw_insert_nan_gamma USING ec_hnsw \
             (embedding tqvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_nan_gamma VALUES
             (1, '[dim=4,bits=4,seed=42,gamma=NaN]:112233'::tqvector)",
        )
        .expect("insert should fail");
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw aminsert supports at most 10 duplicate heap tids per encoded vector"
    )]
    fn test_ech_insert_rejects_duplicate_heaptid_overflow() {
        Spi::run("CREATE TABLE ec_hnsw_insert_duplicate_overflow (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_duplicate_overflow VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_duplicate_overflow_idx ON ec_hnsw_insert_duplicate_overflow USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        for id in 2..=10 {
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_duplicate_overflow VALUES
                 ({id}, encode_to_ecvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))"
            ))
            .expect("duplicate insert should succeed until inline heap tid capacity is exhausted");
        }

        Spi::run(
            "INSERT INTO ec_hnsw_insert_duplicate_overflow VALUES
             (11, encode_to_ecvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))",
        )
        .expect("insert should fail once duplicate heap tid capacity is exhausted");
    }

    #[pg_test]
    fn test_ech_insert_supports_build_source_column_index() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_source_live (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_source_live VALUES
             (1, ARRAY[1.0, 0.0, 0.5, -1.0], encode_to_ecvector(ARRAY[0.2, 0.1, 0.0, -0.2], 4, 42)),
             (2, ARRAY[0.9, 0.1, 0.4, -0.8], encode_to_ecvector(ARRAY[-0.1, 0.9, 0.2, -0.3], 4, 42)),
             (3, ARRAY[0.8, 0.2, 0.3, -0.6], encode_to_ecvector(ARRAY[0.4, 0.1, -0.2, 0.3], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_source_live_idx ON ec_hnsw_insert_source_live USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source', m = 2)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_source_live VALUES
             (4, ARRAY[0.7, 0.3, 0.2, -0.4], encode_to_ecvector(ARRAY[0.1, -0.3, 0.7, 0.2], 4, 42))",
        )
        .expect("live insert should succeed on build_source_column indexes");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_insert_source_live_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let inserted_heap_tid = heap_tid_for_row("ec_hnsw_insert_source_live", 4);
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (_element_tid, inserted_element) =
            find_element_for_heap_tid(&elements, inserted_heap_tid);

        assert!(
            metadata.inserted_since_rebuild > 0,
            "live inserts should still advance drift tracking on build_source_column indexes",
        );
        assert!(!inserted_element.deleted);
        assert!(
            inserted_element.neighbortid != am::page::ItemPointer::INVALID,
            "live insert should persist a neighbor tuple for the new element",
        );
        let neighbors = neighbors
            .get(&inserted_element.neighbortid)
            .expect("newly inserted source-backed element should keep a neighbor tuple");
        assert_eq!(neighbors.count as usize, neighbors.tids.len());
    }

    #[pg_test]
    fn test_ech_insert_supports_ecvector_build_source_column_index() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_ecvector_source_live (
                id bigint primary key,
                source ecvector,
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_ecvector_source_live VALUES
             (1, ('[1.0,0.0,0.5,-1.0]')::ecvector, encode_to_ecvector(ARRAY[0.2, 0.1, 0.0, -0.2], 4, 42)),
             (2, ('[0.9,0.1,0.4,-0.8]')::ecvector, encode_to_ecvector(ARRAY[-0.1, 0.9, 0.2, -0.3], 4, 42)),
             (3, ('[0.8,0.2,0.3,-0.6]')::ecvector, encode_to_ecvector(ARRAY[0.4, 0.1, -0.2, 0.3], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_ecvector_source_live_idx ON ec_hnsw_insert_ecvector_source_live USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source', m = 2)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_ecvector_source_live VALUES
             (4, ('[0.7,0.3,0.2,-0.4]')::ecvector, encode_to_ecvector(ARRAY[0.1, -0.3, 0.7, 0.2], 4, 42))",
        )
        .expect("live insert should succeed on ecvector build_source_column indexes");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_ecvector_source_live_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let inserted_heap_tid = heap_tid_for_row("ec_hnsw_insert_ecvector_source_live", 4);
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (_element_tid, inserted_element) =
            find_element_for_heap_tid(&elements, inserted_heap_tid);

        assert!(
            metadata.inserted_since_rebuild > 0,
            "live inserts should still advance drift tracking on ecvector build_source_column indexes",
        );
        assert!(!inserted_element.deleted);
        assert!(
            inserted_element.neighbortid != am::page::ItemPointer::INVALID,
            "live insert should persist a neighbor tuple for the new ecvector-backed element",
        );
        let neighbors = neighbors
            .get(&inserted_element.neighbortid)
            .expect("newly inserted ecvector-backed element should keep a neighbor tuple");
        assert_eq!(neighbors.count as usize, neighbors.tids.len());
    }

    #[pg_test]
    fn test_ech_insert_bootstraps_empty_pq_fastscan_index() {
        let _lock = env_var_test_lock();
        let inserted_query = vec![
            0.2_f32, 0.1, 0.0, -0.1, -0.2, -0.3, -0.4, -0.5, 0.5, 0.4, 0.3, 0.2, 0.1, 0.0, -0.1,
            -0.2,
        ];

        Spi::run(
            "CREATE TABLE ec_hnsw_insert_empty_pq_fastscan_bootstrap (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_empty_pq_fastscan_bootstrap_idx ON ec_hnsw_insert_empty_pq_fastscan_bootstrap USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_empty_pq_fastscan_bootstrap VALUES
             (17,
              ARRAY[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8,
                    0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6]::real[],
              encode_to_ecvector(
                  ARRAY[0.2, 0.1, 0.0, -0.1, -0.2, -0.3, -0.4, -0.5,
                        0.5, 0.4, 0.3, 0.2, 0.1, 0.0, -0.1, -0.2]::real[],
                  4,
                  42
              ))",
        )
        .expect("empty-index bootstrap insert should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_empty_pq_fastscan_bootstrap_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_metadata, layout, elements, neighbors) =
            decode_grouped_index_elements_and_neighbors(index_oid);
        assert_eq!(elements.len(), 1);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(elements[0].1.search_code.len(), layout.search_code_len);
        assert_eq!(elements[0].1.binary_words.len(), layout.binary_word_count);
        assert!(
            elements[0].1.reranktid != am::page::ItemPointer::INVALID,
            "empty-index bootstrap should persist a rerank tuple",
        );

        let ctid_to_id = ctid_id_map("ec_hnsw_insert_empty_pq_fastscan_bootstrap");
        let observed_ids =
            unsafe { am::debug_gettuple_scan_heap_tids_with_scores(index_oid, inserted_query) }
                .into_iter()
                .map(|(heap_tid, _score)| {
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("bootstrap scan heap tid should map back to a table row")
                })
                .collect::<Vec<_>>();
        assert_eq!(
            observed_ids,
            vec![17],
            "bootstrap-created PqFastScan index should emit the inserted row in ordered scan",
        );
    }

    #[pg_test]
    fn test_ech_insert_appends_to_built_pq_fastscan_index() {
        let _lock = env_var_test_lock();
        let inserted_query = vec![
            0.2_f32, 0.1, 0.0, -0.1, -0.2, -0.3, -0.4, -0.5, 0.5, 0.4, 0.3, 0.2, 0.1, 0.0, -0.1,
            -0.2,
        ];

        Spi::run(
            "CREATE TABLE ec_hnsw_insert_pq_fastscan_live (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 29 + dim) as f32) * 0.05).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 17 + dim) as f32) * 0.04).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_pq_fastscan_live VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_pq_fastscan_live_idx ON ec_hnsw_insert_pq_fastscan_live USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_pq_fastscan_live_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_before_block_count, before_metadata, before_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        let before_page_tuples = before_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().map(move |(idx, tuple)| {
                    (
                        am::page::ItemPointer {
                            block_number: page.block_number,
                            offset_number: (idx + 1) as u16,
                        },
                        tuple.as_slice(),
                    )
                })
            })
            .collect::<Vec<_>>();
        let before_grouped_hot_tids = before_page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                (tuple.first().copied() == Some(am::page::TQ_GROUPED_HOT_TAG)).then_some(*tid)
            })
            .collect::<Vec<_>>();
        let before_rerank_count = before_page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_RERANK_TAG))
            .count();
        let before_neighbor_count = before_page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG))
            .count();

        Spi::run(
            "INSERT INTO ec_hnsw_insert_pq_fastscan_live VALUES
             (17,
              ARRAY[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8,
                    0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6]::real[],
              encode_to_ecvector(
                  ARRAY[0.2, 0.1, 0.0, -0.1, -0.2, -0.3, -0.4, -0.5,
                        0.5, 0.4, 0.3, 0.2, 0.1, 0.0, -0.1, -0.2]::real[],
                  4,
                  42
              ))",
        )
        .expect("insert should succeed on a built PqFastScan index");

        let (_after_block_count, after_metadata, after_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        let after_page_tuples = after_pages
            .iter()
            .flat_map(|page| {
                page.tuples.iter().enumerate().map(move |(idx, tuple)| {
                    (
                        am::page::ItemPointer {
                            block_number: page.block_number,
                            offset_number: (idx + 1) as u16,
                        },
                        tuple.as_slice(),
                    )
                })
            })
            .collect::<Vec<_>>();
        let after_grouped_hot_tids = after_page_tuples
            .iter()
            .filter_map(|(tid, tuple)| {
                (tuple.first().copied() == Some(am::page::TQ_GROUPED_HOT_TAG)).then_some(*tid)
            })
            .collect::<Vec<_>>();
        let after_rerank_count = after_page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_RERANK_TAG))
            .count();
        let after_neighbor_count = after_page_tuples
            .iter()
            .filter(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_NEIGHBOR_TAG))
            .count();

        assert_eq!(
            after_metadata.inserted_since_rebuild,
            before_metadata.inserted_since_rebuild + 1
        );
        assert_eq!(
            after_grouped_hot_tids.len(),
            before_grouped_hot_tids.len() + 1
        );
        assert_eq!(after_rerank_count, before_rerank_count + 1);
        assert_eq!(after_neighbor_count, before_neighbor_count + 1);
        assert!(
            !after_page_tuples
                .iter()
                .any(|(_, tuple)| tuple.first().copied() == Some(am::page::TQ_ELEMENT_TAG)),
            "PqFastScan live insert should keep writing grouped hot tuples, not scalar element tuples",
        );

        let new_hot_tid = after_grouped_hot_tids
            .iter()
            .copied()
            .find(|tid| !before_grouped_hot_tids.contains(tid))
            .expect("live insert should add exactly one grouped hot tuple");
        let layout =
            match am::graph::GraphStorageDescriptor::from_metadata(&after_metadata).unwrap() {
                am::graph::GraphStorageDescriptor::PqFastScan(layout) => layout,
                am::graph::GraphStorageDescriptor::TurboQuant { .. }
                | am::graph::GraphStorageDescriptor::TurboQuantHotCold(_) => {
                    panic!("PqFastScan insert test should still decode as PqFastScan storage")
                }
            };
        let index_relation = unsafe {
            open_valid_ec_hnsw_index(
                index_oid,
                "test_ech_insert_appends_to_built_pq_fastscan_index",
            )
        };
        let new_hot =
            unsafe { am::graph::load_grouped_graph_element(index_relation, new_hot_tid, layout) };
        let rerank = unsafe {
            am::graph::load_grouped_rerank_payload(index_relation, new_hot.reranktid, layout)
        };
        assert!(!new_hot.deleted);
        assert_eq!(new_hot.heaptids.len(), 1);
        assert_eq!(new_hot.search_code.len(), layout.search_code_len);
        assert_eq!(new_hot.binary_words.len(), layout.binary_word_count);
        assert_ne!(new_hot.neighbortid, am::page::ItemPointer::INVALID);
        assert_ne!(new_hot.reranktid, am::page::ItemPointer::INVALID);
        assert_eq!(rerank.code.len(), layout.rerank_code_len);
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

        let ctid_to_id = ctid_id_map("ec_hnsw_insert_pq_fastscan_live");
        let observed_ids = unsafe {
            am::debug_gettuple_scan_heap_tids_with_scores(index_oid, inserted_query.clone())
        }
        .into_iter()
        .map(|(heap_tid, _score)| {
            *ctid_to_id
                .get(&heap_tid)
                .expect("inserted-row query heap tid should map back to a table row")
        })
        .collect::<Vec<_>>();
        assert_eq!(
            observed_ids.first().copied(),
            Some(17),
            "querying the inserted embedding should rank the new PqFastScan row first",
        );
    }

    #[pg_test]
    fn test_ech_insert_coalesces_duplicate_vectors_for_pq_fastscan() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_insert_duplicate_pq_fastscan (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 23 + dim) as f32) * 0.03).sin()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = (0..16)
                .map(|dim| format!("{:.6}", (((id * 13 + dim) as f32) * 0.06).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_duplicate_pq_fastscan VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_duplicate_pq_fastscan_idx ON ec_hnsw_insert_duplicate_pq_fastscan USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_duplicate_pq_fastscan_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_before_block_count, _before_metadata, before_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        let before_tuple_count = before_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();

        Spi::run(
            "INSERT INTO ec_hnsw_insert_duplicate_pq_fastscan
             SELECT 17, source, embedding
               FROM ec_hnsw_insert_duplicate_pq_fastscan
              WHERE id = 1",
        )
        .expect("duplicate insert should succeed on a built PqFastScan index");

        let (_after_block_count, after_metadata, after_pages) =
            unsafe { am::debug_index_pages(index_oid) };
        let after_tuple_count = after_pages
            .iter()
            .map(|page| page.tuples.len())
            .sum::<usize>();
        assert_eq!(
            after_tuple_count, before_tuple_count,
            "duplicate PqFastScan insert should not add new hot/rerank/neighbor tuples",
        );

        let layout =
            match am::graph::GraphStorageDescriptor::from_metadata(&after_metadata).unwrap() {
                am::graph::GraphStorageDescriptor::PqFastScan(layout) => layout,
                am::graph::GraphStorageDescriptor::TurboQuant { .. }
                | am::graph::GraphStorageDescriptor::TurboQuantHotCold(_) => {
                    panic!("PqFastScan duplicate test should still decode as PqFastScan storage")
                }
            };
        let grouped_hot = after_pages
            .iter()
            .flat_map(|page| page.tuples.iter())
            .filter(|tuple| tuple.first().copied() == Some(am::page::TQ_GROUPED_HOT_TAG))
            .map(|tuple| {
                am::page::TqGroupedHotTuple::decode(
                    tuple,
                    layout.binary_word_count,
                    layout.search_code_len,
                )
                .expect("grouped hot tuple should decode")
            })
            .collect::<Vec<_>>();
        let mut heaptid_counts = grouped_hot
            .iter()
            .map(|element| element.heaptids.len())
            .collect::<Vec<_>>();
        heaptid_counts.sort_unstable();
        assert_eq!(heaptid_counts[heaptid_counts.len() - 1], 2);
        assert_eq!(
            heaptid_counts.iter().filter(|count| **count == 1).count(),
            grouped_hot.len() - 1
        );
    }

    #[pg_test]
    fn test_ech_vacuum_stats_accepts_pq_fastscan_index() {
        let _lock = env_var_test_lock();

        let index_oid = create_pq_fastscan_runtime_fixture(
            "ec_hnsw_vacuum_grouped_stats",
            "ec_hnsw_vacuum_grouped_stats_idx",
        );

        let stats = unsafe { am::debug_vacuum_stats(index_oid) };
        let (_metadata, _layout, elements, _neighbors) =
            decode_grouped_index_elements_and_neighbors(index_oid);

        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(
            stats.num_index_tuples,
            elements.len() as f64,
            "vacuum stats should count live grouped hot tuples for PqFastScan indexes",
        );
        assert_eq!(elements.len(), 16);
    }

    #[pg_test]
    fn test_ech_vacuum_pass1_compacts_pq_fastscan_duplicates() {
        let _lock = env_var_test_lock();

        Spi::run(
            "CREATE TABLE ec_hnsw_vacuum_pass1_grouped_duplicates (
                id bigint primary key,
                source real[],
                embedding ecvector
            )",
        )
        .expect("table creation should succeed");
        for id in 1..=16 {
            let source = (0..16)
                .map(|dim| format!("{:.6}", (((id * 31 + dim) as f32) * 0.05).cos()))
                .collect::<Vec<_>>()
                .join(", ");
            let embedding = if id == 2 {
                (0..16)
                    .map(|dim| format!("{:.6}", (((37 + dim) as f32) * 0.04).sin()))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                (0..16)
                    .map(|dim| format!("{:.6}", (((id * 37 + dim) as f32) * 0.04).sin()))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_vacuum_pass1_grouped_duplicates VALUES \
                 ({id}, ARRAY[{source}]::real[], encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42))"
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_pass1_grouped_duplicates_idx ON ec_hnsw_vacuum_pass1_grouped_duplicates USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (build_source_column = 'source', storage_format = 'pq_fastscan')",
        )
        .expect("index creation should succeed");

        let survivor_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_pass1_grouped_duplicates", 1);
        let deleted_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_pass1_grouped_duplicates", 2);
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_vacuum_pass1_grouped_duplicates_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_metadata_before, _layout_before, elements_before, _neighbors_before) =
            decode_grouped_index_elements_and_neighbors(index_oid);
        let (_duplicate_tid_before, duplicate_before) =
            find_grouped_element_for_heap_tid(&elements_before, survivor_heap_tid);
        assert!(
            duplicate_before.heaptids.contains(&deleted_heap_tid),
            "fixture should build a grouped hot tuple with both duplicate heap tids",
        );

        Spi::run("DELETE FROM ec_hnsw_vacuum_pass1_grouped_duplicates WHERE id = 2")
            .expect("delete should succeed");

        let stats = unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
        let (_metadata_after, _layout_after, elements_after, _neighbors_after) =
            decode_grouped_index_elements_and_neighbors(index_oid);
        let (_duplicate_tid_after, duplicate_after) =
            find_grouped_element_for_heap_tid(&elements_after, survivor_heap_tid);

        assert_eq!(
            duplicate_after.heaptids,
            vec![survivor_heap_tid],
            "pass 1 should compact the grouped hot tuple to the surviving heap tid",
        );
        assert!(
            elements_after
                .iter()
                .all(|(_, element)| !element.heaptids.contains(&deleted_heap_tid)),
            "pass 1 should remove the deleted heap tid from every grouped hot tuple",
        );
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(
            stats.num_index_tuples, 15.0,
            "amvacuumcleanup should report the remaining live grouped hot tuple count",
        );
    }

    #[pg_test]
    fn test_ech_vacuum_pass2_unlinks_pq_fastscan_refs() {
        let _lock = env_var_test_lock();

        let table_name = "ec_hnsw_vacuum_grouped_pass2_unlink";
        let index_name = "ec_hnsw_vacuum_grouped_pass2_unlink_idx";
        let index_oid = create_pq_fastscan_runtime_fixture(table_name, index_name);
        let (_metadata_before, _layout_before, elements_before, neighbors_before) =
            decode_grouped_index_elements_and_neighbors(index_oid);
        let (deleted_row_id, deleted_heap_tid, deleted_element_tid) = (1..=16_i64)
            .find_map(|id| {
                let observed_ids =
                    observed_ids_for_query(index_oid, table_name, runtime_fixture_embedding(id));
                if observed_ids.first().copied()
                    != Some(usize::try_from(id).expect("row id should fit in usize"))
                {
                    return None;
                }
                let heap_tid = heap_tid_for_row(table_name, id);
                let (element_tid, _element) =
                    find_grouped_element_for_heap_tid(&elements_before, heap_tid);
                (count_neighbor_refs(&neighbors_before, element_tid) > 0).then_some((
                    id,
                    heap_tid,
                    element_tid,
                ))
            })
            .expect(
                "fixture should expose at least one grouped hot tuple with inbound neighbor refs that also self-ranks before vacuum",
            );
        let deleted_query = (0_i64..16_i64)
            .map(|dim| (((deleted_row_id * 29 + dim) as f32) * 0.02).sin())
            .collect::<Vec<_>>();
        let ctid_to_id = ctid_id_map(table_name);
        let observed_before_ids = unsafe {
            am::debug_gettuple_scan_heap_tids_with_scores(index_oid, deleted_query.clone())
        }
        .into_iter()
        .map(|(heap_tid, _score)| {
            *ctid_to_id
                .get(&heap_tid)
                .expect("pre-vacuum heap tid should map back to a table row")
        })
        .collect::<Vec<_>>();
        assert_eq!(
            observed_before_ids.first().copied(),
            Some(usize::try_from(deleted_row_id).expect("deleted row id should fit in usize")),
            "before vacuum, querying the deleted row's embedding should rank that row first",
        );

        Spi::run(&format!(
            "DELETE FROM {table_name} WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");

        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (_metadata_after, _layout_after, elements_after, neighbors_after) =
            decode_grouped_index_elements_and_neighbors(index_oid);
        let (_, deleted_element_after) = elements_after
            .iter()
            .find(|(tid, _)| *tid == deleted_element_tid)
            .expect("deleted grouped hot tuple should remain on disk after vacuum");

        assert!(
            deleted_element_after.deleted,
            "vacuum should finalize a fully dead grouped hot tuple after repair",
        );
        assert!(
            deleted_element_after.heaptids.is_empty(),
            "vacuum should leave the finalized grouped hot tuple with no surviving heap tids",
        );
        assert_eq!(
            count_neighbor_refs(&neighbors_after, deleted_element_tid),
            0,
            "pass 2 should remove every persisted neighbor ref to the deleted grouped hot tuple",
        );

        let observed_after_ids =
            unsafe { am::debug_gettuple_scan_heap_tids_with_scores(index_oid, deleted_query) }
                .into_iter()
                .map(|(heap_tid, _score)| {
                    *ctid_to_id
                        .get(&heap_tid)
                        .expect("post-vacuum heap tid should map back to a table row")
                })
                .collect::<Vec<_>>();
        assert!(
            !observed_after_ids.is_empty(),
            "vacuumed PqFastScan index should still emit ordered scan results",
        );
        assert!(
            !observed_after_ids.contains(
                &usize::try_from(deleted_row_id).expect("deleted row id should fit in usize")
            ),
            "vacuumed PqFastScan scan results should no longer surface the deleted row",
        );
    }

    #[pg_test]
    fn test_ech_vacuum_pass2_replaces_pq_fastscan_layer0_edges() {
        let _lock = env_var_test_lock();

        let table_name = "ec_hnsw_vacuum_grouped_pass2_replace";
        let index_name = "ec_hnsw_vacuum_grouped_pass2_replace_idx";
        let index_oid = create_pq_fastscan_runtime_fixture_with_m(table_name, index_name, 2);
        let (metadata_before, _layout_before, elements_before, neighbors_before) =
            decode_grouped_index_elements_and_neighbors(index_oid);
        let (deleted_row_id, deleted_heap_tid, deleted_element_tid, affected_before) =
            (1_i64..=16_i64)
                .find_map(|id| {
                    let deleted_heap_tid = heap_tid_for_row(table_name, id);
                    let (deleted_element_tid, _deleted_element) =
                        find_grouped_element_for_heap_tid(&elements_before, deleted_heap_tid);
                    let affected_before = elements_before
                        .iter()
                        .filter_map(|(element_tid, element)| {
                            if *element_tid == deleted_element_tid
                                || element.deleted
                                || element.heaptids.is_empty()
                            {
                                return None;
                            }

                            let neighbor = neighbors_before
                                .get(&element.neighbortid)
                                .expect("live grouped element should have a persisted neighbor tuple");
                            let layer0 = layer_neighbor_slice(
                                &neighbor.tids,
                                usize::from(metadata_before.m),
                                0,
                            );
                            layer0.contains(&deleted_element_tid).then(|| {
                                (
                                    *element_tid,
                                    layer0
                                        .iter()
                                        .copied()
                                        .filter(|tid| {
                                            *tid != am::page::ItemPointer::INVALID
                                                && *tid != deleted_element_tid
                                        })
                                        .collect::<Vec<_>>(),
                                )
                            })
                        })
                        .collect::<Vec<_>>();

                    (!affected_before.is_empty())
                        .then_some((id, deleted_heap_tid, deleted_element_tid, affected_before))
                })
                .expect(
                    "fixture should provide at least one deletable grouped row with a live inbound layer-0 edge",
                );

        Spi::run(&format!(
            "DELETE FROM {table_name} WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (metadata_after, _layout_after, elements_after, neighbors_after) =
            decode_grouped_index_elements_and_neighbors(index_oid);
        let mut replacement_filled = false;

        for (affected_tid, surviving_before) in affected_before {
            let (_, element_after) = elements_after
                .iter()
                .find(|(tid, _)| *tid == affected_tid)
                .expect("affected live grouped element should remain on disk after vacuum");
            let neighbor_after = neighbors_after
                .get(&element_after.neighbortid)
                .expect("affected live grouped element should keep a persisted neighbor tuple");
            let layer0_after =
                layer_neighbor_slice(&neighbor_after.tids, usize::from(metadata_after.m), 0);
            let surviving_after = layer0_after
                .iter()
                .copied()
                .filter(|tid| *tid != am::page::ItemPointer::INVALID)
                .collect::<Vec<_>>();

            if surviving_after
                .iter()
                .any(|tid| *tid != deleted_element_tid && !surviving_before.contains(tid))
            {
                replacement_filled = true;
                break;
            }
        }

        assert_eq!(
            count_neighbor_refs(&neighbors_after, deleted_element_tid),
            0,
            "grouped vacuum replacement should still leave no persisted refs to the deleted element tid",
        );
        assert!(
            replacement_filled,
            "grouped vacuum replacement should fill at least one broken layer-0 edge with a new live candidate",
        );
    }

    #[pg_test]
    fn test_ech_turboquant_reloption_round_trip() {
        let table_name = "ec_hnsw_turboquant_reloption_round_trip";
        let index_name = "ec_hnsw_turboquant_reloption_round_trip_idx";
        let index_oid = create_turboquant_runtime_fixture(table_name, index_name);

        let deleted_row_id = first_self_ranked_runtime_fixture_id(index_oid, table_name, 1..=16);
        let deleted_query = runtime_fixture_embedding(deleted_row_id);
        let observed_before_delete =
            observed_ids_for_query(index_oid, table_name, deleted_query.clone());
        assert_eq!(
            observed_before_delete.first().copied(),
            Some(usize::try_from(deleted_row_id).expect("deleted row id should fit in usize")),
            "explicit turboquant reloption should still rank a row first for its own embedding",
        );

        let _inserted_row_id = (17_i64..=64_i64)
            .find(|candidate_id| {
                let candidate_id = *candidate_id;
                let candidate_embedding = runtime_fixture_embedding(candidate_id);
                let candidate_embedding_sql =
                    format_recall_vector_sql_literal(&candidate_embedding);
                Spi::run(&format!(
                    "INSERT INTO {table_name} VALUES \
                     ({candidate_id}, encode_to_ecvector({candidate_embedding_sql}, 4, 42))"
                ))
                .expect("live insert should succeed on an explicit turboquant index");

                let observed_after_insert =
                    observed_ids_for_query(index_oid, table_name, candidate_embedding.clone());
                observed_after_insert.first().copied()
                    == Some(
                        usize::try_from(candidate_id)
                            .expect("inserted row id should fit in usize"),
                    )
            })
            .expect(
                "fixture should expose at least one live-inserted turboquant row that ranks first for its own embedding",
            );

        let deleted_heap_tid = heap_tid_for_row(table_name, deleted_row_id);
        Spi::run(&format!(
            "DELETE FROM {table_name} WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let observed_after_delete = observed_heap_tids_for_query(index_oid, deleted_query);
        assert!(
            !observed_after_delete.is_empty(),
            "vacuumed turboquant reloption index should still emit ordered scan results",
        );
        assert!(
            !observed_after_delete.contains(&(
                deleted_heap_tid.block_number,
                deleted_heap_tid.offset_number
            )),
            "vacuumed turboquant reloption index should no longer emit the deleted row",
        );
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw index reloption storage_format=pq_fastscan does not match on-disk metadata format=turboquant; REINDEX after switching formats"
    )]
    fn test_ech_storage_format_switch_requires_reindex() {
        let table_name = "ec_hnsw_storage_format_reindex_guard";
        let index_name = "ec_hnsw_storage_format_reindex_guard_idx";
        let _index_oid = create_turboquant_runtime_fixture(table_name, index_name);

        Spi::run(&format!(
            "ALTER INDEX {index_name} SET (storage_format = 'pq_fastscan')"
        ))
        .expect("ALTER INDEX should update the reloption without rewriting the index");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let query = format_recall_vector_sql_literal(&runtime_fixture_embedding(1));
        let _ = Spi::get_one::<i64>(&format!(
            "SELECT id FROM {table_name} \
             ORDER BY embedding <#> {query} \
             LIMIT 1"
        ))
        .expect("ordered scan should reach amrescan before rejecting a storage-format mismatch");
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw index reloption storage_format=pq_fastscan does not match on-disk metadata format=turboquant; REINDEX after switching formats"
    )]
    fn test_ech_storage_format_switch_rejects_insert_until_reindex() {
        let table_name = "ec_hnsw_storage_format_reindex_insert_guard";
        let index_name = "ec_hnsw_storage_format_reindex_insert_guard_idx";
        let _index_oid = create_turboquant_runtime_fixture(table_name, index_name);

        Spi::run(&format!(
            "ALTER INDEX {index_name} SET (storage_format = 'pq_fastscan')"
        ))
        .expect("ALTER INDEX should update the reloption without rewriting the index");

        let inserted_embedding_sql =
            format_recall_vector_sql_literal(&runtime_fixture_embedding(17));
        Spi::run(&format!(
            "INSERT INTO {table_name} VALUES \
             (17, encode_to_ecvector({inserted_embedding_sql}, 4, 42))"
        ))
        .expect("insert should reach aminsert before rejecting a storage-format mismatch");
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw index reloption storage_format=pq_fastscan does not match on-disk metadata format=turboquant; REINDEX after switching formats"
    )]
    fn test_ech_storage_format_switch_rejects_vacuum_until_reindex() {
        let table_name = "ec_hnsw_storage_format_reindex_vacuum_guard";
        let index_name = "ec_hnsw_storage_format_reindex_vacuum_guard_idx";
        let index_oid = create_turboquant_runtime_fixture(table_name, index_name);

        let deleted_row_id = 1_i64;
        let deleted_heap_tid = heap_tid_for_row(table_name, deleted_row_id);
        Spi::run(&format!(
            "DELETE FROM {table_name} WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        Spi::run(&format!(
            "ALTER INDEX {index_name} SET (storage_format = 'pq_fastscan')"
        ))
        .expect("ALTER INDEX should update the reloption without rewriting the index");

        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw index reloption storage_format=turboquant does not match on-disk metadata format=pq_fastscan; REINDEX after switching formats"
    )]
    fn test_ech_storage_format_switch_reverse_requires_reindex() {
        let _lock = env_var_test_lock();

        let table_name = "ec_hnsw_storage_format_reindex_guard_reverse";
        let index_name = "ec_hnsw_storage_format_reindex_guard_reverse_idx";
        let _index_oid = create_pq_fastscan_runtime_fixture(table_name, index_name);

        Spi::run(&format!(
            "ALTER INDEX {index_name} SET (storage_format = 'turboquant')"
        ))
        .expect("ALTER INDEX should update the reloption without rewriting the index");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");

        let query = format_recall_vector_sql_literal(&runtime_fixture_embedding(1));
        let _ = Spi::get_one::<i64>(&format!(
            "SELECT id FROM {table_name} \
             ORDER BY embedding <#> {query} \
             LIMIT 1"
        ))
        .expect("ordered scan should reach amrescan before rejecting the reverse storage-format mismatch");
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw index reloption storage_format=turboquant does not match on-disk metadata format=pq_fastscan; REINDEX after switching formats"
    )]
    fn test_ech_storage_format_switch_reverse_rejects_insert() {
        let _lock = env_var_test_lock();

        let table_name = "ec_hnsw_storage_format_reindex_insert_guard_reverse";
        let index_name = "ec_hnsw_storage_format_reindex_insert_guard_reverse_idx";
        let _index_oid = create_pq_fastscan_runtime_fixture(table_name, index_name);

        Spi::run(&format!(
            "ALTER INDEX {index_name} SET (storage_format = 'turboquant')"
        ))
        .expect("ALTER INDEX should update the reloption without rewriting the index");

        let inserted_source_sql = format_recall_vector_sql_literal(&pq_fastscan_runtime_source(17));
        let inserted_embedding_sql =
            format_recall_vector_sql_literal(&runtime_fixture_embedding(17));
        Spi::run(&format!(
            "INSERT INTO {table_name} VALUES \
             (17, {inserted_source_sql}, encode_to_ecvector({inserted_embedding_sql}, 4, 42))"
        ))
        .expect(
            "insert should reach aminsert before rejecting the reverse storage-format mismatch",
        );
    }

    #[pg_test]
    #[should_panic(
        expected = "ec_hnsw index reloption storage_format=turboquant does not match on-disk metadata format=pq_fastscan; REINDEX after switching formats"
    )]
    fn test_ech_storage_format_switch_reverse_rejects_vacuum() {
        let _lock = env_var_test_lock();

        let table_name = "ec_hnsw_storage_format_reindex_vacuum_guard_reverse";
        let index_name = "ec_hnsw_storage_format_reindex_vacuum_guard_reverse_idx";
        let index_oid = create_pq_fastscan_runtime_fixture(table_name, index_name);

        let deleted_row_id = 1_i64;
        let deleted_heap_tid = heap_tid_for_row(table_name, deleted_row_id);
        Spi::run(&format!(
            "DELETE FROM {table_name} WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        Spi::run(&format!(
            "ALTER INDEX {index_name} SET (storage_format = 'turboquant')"
        ))
        .expect("ALTER INDEX should update the reloption without rewriting the index");

        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
    }

    #[pg_test]
    fn test_ech_storage_format_switch_reindex_restores_runtime() {
        let table_name = "ec_hnsw_storage_format_reindex_restores_runtime_paths";
        let index_name = "ec_hnsw_storage_format_reindex_restores_runtime_paths_idx";
        let index_oid = create_turboquant_runtime_fixture_with_source(table_name, index_name);

        Spi::run(&format!(
            "ALTER INDEX {index_name} SET (storage_format = 'pq_fastscan')"
        ))
        .expect("ALTER INDEX should update the reloption before REINDEX");
        Spi::run(&format!("REINDEX INDEX {index_name}"))
            .expect("REINDEX should rebuild the index to match the new storage format");

        let (_block_count, _m, _ef_construction, metadata) =
            unsafe { am::debug_index_metadata(index_oid) };
        assert_eq!(
            metadata
                .graph_storage_format()
                .expect("reindexed metadata should decode"),
            am::page::GraphStorageFormat::PqFastScan,
            "REINDEX after a storage_format ALTER should rewrite the on-disk metadata format",
        );

        let deleted_row_id = first_self_ranked_runtime_fixture_id(index_oid, table_name, 1..=16);
        let deleted_query = runtime_fixture_embedding(deleted_row_id);
        let observed_before_delete =
            observed_ids_for_query(index_oid, table_name, deleted_query.clone());
        assert_eq!(
            observed_before_delete.first().copied(),
            Some(usize::try_from(deleted_row_id).expect("deleted row id should fit in usize")),
            "reindexed pq_fastscan output should still rank a row first for its own embedding",
        );

        let inserted_embedding_sql =
            format_recall_vector_sql_literal(&runtime_fixture_embedding(17));
        let inserted_source_sql = format_recall_vector_sql_literal(&pq_fastscan_runtime_source(17));
        Spi::run(&format!(
            "INSERT INTO {table_name} VALUES \
             (17, {inserted_source_sql}, encode_to_ecvector({inserted_embedding_sql}, 4, 42))"
        ))
        .expect("matching reloption/metadata pairs should accept insert cleanly after REINDEX");

        let deleted_heap_tid = heap_tid_for_row(table_name, deleted_row_id);
        Spi::run(&format!(
            "DELETE FROM {table_name} WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let observed_after_delete = observed_heap_tids_for_query(index_oid, deleted_query);
        assert!(
            !observed_after_delete.is_empty(),
            "reindexed pq_fastscan output should still emit ordered scan results after insert and vacuum",
        );
        assert!(
            !observed_after_delete.contains(&(
                deleted_heap_tid.block_number,
                deleted_heap_tid.offset_number
            )),
            "vacuum after REINDEX should still remove the deleted heap tid from ordered scan output",
        );
    }

    #[pg_test]
    fn test_ech_pq_fastscan_reloption_round_trip() {
        let _lock = env_var_test_lock();

        let table_name = "ec_hnsw_pq_fastscan_reloption_round_trip";
        let index_name = "ec_hnsw_pq_fastscan_reloption_round_trip_idx";
        let index_oid = create_pq_fastscan_runtime_fixture(table_name, index_name);

        let deleted_row_id = first_self_ranked_runtime_fixture_id(index_oid, table_name, 1..=16);
        let deleted_query = runtime_fixture_embedding(deleted_row_id);
        let observed_before_delete =
            observed_ids_for_query(index_oid, table_name, deleted_query.clone());
        assert_eq!(
            observed_before_delete.first().copied(),
            Some(usize::try_from(deleted_row_id).expect("deleted row id should fit in usize")),
            "explicit pq_fastscan reloption should still rank a row first for its own embedding",
        );

        let _inserted_row_id = (17_i64..=64_i64)
            .find(|candidate_id| {
                let candidate_id = *candidate_id;
                let candidate_source_sql = format_recall_vector_sql_literal(
                    &pq_fastscan_runtime_source(candidate_id),
                );
                let candidate_embedding = runtime_fixture_embedding(candidate_id);
                let candidate_embedding_sql =
                    format_recall_vector_sql_literal(&candidate_embedding);
                Spi::run(&format!(
                    "INSERT INTO {table_name} VALUES \
                     ({candidate_id}, {candidate_source_sql}, encode_to_ecvector({candidate_embedding_sql}, 4, 42))"
                ))
                .expect("live insert should succeed on an explicit pq_fastscan index");

                let observed_after_insert =
                    observed_ids_for_query(index_oid, table_name, candidate_embedding.clone());
                observed_after_insert.first().copied()
                    == Some(
                        usize::try_from(candidate_id)
                            .expect("inserted row id should fit in usize"),
                    )
            })
            .expect(
                "fixture should expose at least one live-inserted pq_fastscan row that ranks first for its own embedding",
            );

        let deleted_heap_tid = heap_tid_for_row(table_name, deleted_row_id);
        Spi::run(&format!(
            "DELETE FROM {table_name} WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let observed_after_delete = observed_heap_tids_for_query(index_oid, deleted_query);
        assert!(
            !observed_after_delete.is_empty(),
            "vacuumed pq_fastscan reloption index should still emit ordered scan results",
        );
        assert!(
            !observed_after_delete.contains(&(
                deleted_heap_tid.block_number,
                deleted_heap_tid.offset_number
            )),
            "vacuumed pq_fastscan reloption index should no longer emit the deleted row",
        );
    }

    #[pg_test]
    fn test_vacuum_source_backed_repair_prefers_source_candidate() {
        let table_name = "ec_hnsw_vacuum_source_metric";
        let index_name = "ec_hnsw_vacuum_source_metric_idx";

        Spi::run(&format!(
            "CREATE TABLE {table_name} (
                id bigint primary key,
                source real[],
                embedding ecvector
            )"
        ))
        .expect("table creation should succeed");

        let mut source_by_id = HashMap::new();
        for id in 1_i64..=8_i64 {
            let theta = 0.18_f32 * id as f32;
            let embedding_rank = match id {
                1 => 1_i64,
                2 => 5,
                3 => 2,
                4 => 7,
                5 => 3,
                6 => 8,
                7 => 4,
                8 => 6,
                _ => unreachable!("fixture only seeds ids 1..=8"),
            };
            let embedding_theta = 0.18_f32 * embedding_rank as f32;
            let source = vec![
                theta.cos(),
                theta.sin(),
                (theta * 0.5).cos(),
                (theta * 0.5).sin(),
            ];
            let embedding = [
                embedding_theta.cos(),
                embedding_theta.sin(),
                (embedding_theta * 0.5).cos(),
                (embedding_theta * 0.5).sin(),
            ];
            source_by_id.insert(id, source.clone());
            Spi::run(&format!(
                "INSERT INTO {table_name} VALUES (
                    {id},
                    ARRAY[{source}]::real[],
                    encode_to_ecvector(ARRAY[{embedding}]::real[], 4, 42)
                )",
                source = source
                    .iter()
                    .map(|value| format!("{value:.6}"))
                    .collect::<Vec<_>>()
                    .join(", "),
                embedding = embedding
                    .iter()
                    .map(|value| format!("{value:.6}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            ))
            .expect("seed insert should succeed");
        }

        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 2, build_source_column = 'source')"
        ))
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (metadata_before, elements_before, neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));

        let heap_tid_to_id = (1_i64..=8_i64)
            .map(|id| (heap_tid_for_row(table_name, id), id))
            .collect::<HashMap<_, _>>();
        let case = (1_i64..=8_i64)
            .find_map(|deleted_row_id| {
                let deleted_heap_tid = heap_tid_for_row(table_name, deleted_row_id);
                let (deleted_element_tid, _) =
                    find_element_for_heap_tid(&elements_before, deleted_heap_tid);

                elements_before.iter().find_map(|(affected_tid, affected_element)| {
                    if *affected_tid == deleted_element_tid
                        || affected_element.deleted
                        || affected_element.heaptids.is_empty()
                    {
                        return None;
                    }

                    let neighbor = neighbors_before.get(&affected_element.neighbortid).expect(
                        "live source-backed element should have a persisted neighbor tuple",
                    );
                    let layer0 =
                        layer_neighbor_slice(&neighbor.tids, usize::from(metadata_before.m), 0);
                    if !layer0.contains(&deleted_element_tid) {
                        return None;
                    }

                    let existing_live = layer0
                        .iter()
                        .copied()
                        .filter(|tid| {
                            *tid != am::page::ItemPointer::INVALID && *tid != deleted_element_tid
                        })
                        .collect::<HashSet<_>>();
                    let affected_row_id = *heap_tid_to_id
                        .get(
                            affected_element
                                .heaptids
                                .first()
                                .expect("affected element should keep one representative heap tid"),
                        )
                        .expect("affected heap tid should map back to a table row");
                    let affected_source = source_by_id
                        .get(&affected_row_id)
                        .expect("affected row should keep its source vector");

                    let mut ranked = elements_before
                        .iter()
                        .filter_map(|(candidate_tid, candidate_element)| {
                            if *candidate_tid == *affected_tid
                                || *candidate_tid == deleted_element_tid
                                || candidate_element.deleted
                                || candidate_element.heaptids.is_empty()
                                || existing_live.contains(candidate_tid)
                            {
                                return None;
                            }

                            let candidate_row_id = *heap_tid_to_id
                                .get(
                                    candidate_element
                                        .heaptids
                                        .first()
                                        .expect("candidate element should keep one representative heap tid"),
                                )
                                .expect("candidate heap tid should map back to a table row");
                            let candidate_source = source_by_id
                                .get(&candidate_row_id)
                                .expect("candidate row should keep its source vector");
                            Some((
                                *candidate_tid,
                                -dot_product(affected_source, candidate_source),
                                -crate::score_code_inner_product(
                                    metadata_before.dimensions as usize,
                                    metadata_before.bits,
                                    metadata_before.seed,
                                    &affected_element.code,
                                    &candidate_element.code,
                                ),
                            ))
                        })
                        .collect::<Vec<_>>();
                    ranked.sort_by(|left, right| {
                        left.1
                            .total_cmp(&right.1)
                            .then_with(|| left.0.block_number.cmp(&right.0.block_number))
                            .then_with(|| left.0.offset_number.cmp(&right.0.offset_number))
                    });
                    let expected_source_best = ranked.first().map(|entry| entry.0)?;
                    ranked.sort_by(|left, right| {
                        left.2
                            .total_cmp(&right.2)
                            .then_with(|| left.0.block_number.cmp(&right.0.block_number))
                            .then_with(|| left.0.offset_number.cmp(&right.0.offset_number))
                    });
                    let expected_code_best = ranked.first().map(|entry| entry.0)?;
                    (expected_source_best != expected_code_best).then_some((
                        deleted_row_id,
                        deleted_heap_tid,
                        *affected_tid,
                        expected_source_best,
                    ))
                })
            })
            .expect("fixture should expose at least one broken layer-0 edge where source-space and code-space replacement rankings differ");

        Spi::run(&format!("DELETE FROM {table_name} WHERE id = {}", case.0))
            .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[case.1]) };

        let (metadata_after, elements_after, neighbors_after) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (_, affected_after) = elements_after
            .iter()
            .find(|(tid, _)| *tid == case.2)
            .expect("affected element should remain on disk after vacuum repair");
        let neighbor_after = neighbors_after
            .get(&affected_after.neighbortid)
            .expect("affected element should keep a persisted neighbor tuple after repair");
        let layer0_after =
            layer_neighbor_slice(&neighbor_after.tids, usize::from(metadata_after.m), 0);

        assert!(
            layer0_after.contains(&case.3),
            "vacuum source-backed repair should choose the best source-space replacement candidate",
        );
    }

