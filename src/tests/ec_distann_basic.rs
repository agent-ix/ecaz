// TC-037 (ec_distann basic AM lifecycle) — Task 162 M0.
//
// This file starts with the scaffold-slice coverage: access-method
// registration, index create/drop/reindex on empty and populated tables,
// reloption validation (FR-075-AC-2), GUC defaults, metadata-page
// round-trip through a real relation, and the interim not-implemented
// postures (aminsert / scans). Recall-parity and traversal assertions
// join this file as the later Task 162 slices land.

#[pg_test]
fn test_ec_distann_access_method_is_registered() {
    let am_count = Spi::get_one::<i64>(
        "SELECT count(*) FROM pg_am WHERE amname = 'ec_distann' AND amtype = 'i'",
    )
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(am_count, 1, "ec_distann access method should be registered");

    let opclass_count = Spi::get_one::<i64>(
        "SELECT count(*) FROM pg_opclass c JOIN pg_am a ON a.oid = c.opcmethod \
         WHERE a.amname = 'ec_distann'",
    )
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(
        opclass_count, 2,
        "ec_distann should register ecvector and tqvector operator classes"
    );
}

#[pg_test]
fn test_ec_distann_create_drop_reindex_on_empty_table() {
    Spi::run("CREATE TABLE ec_distann_empty_lifecycle (id bigint, embedding ecvector)")
        .expect("table creation should succeed");
    Spi::run(
        "CREATE INDEX ec_distann_empty_lifecycle_idx ON ec_distann_empty_lifecycle \
         USING ec_distann (embedding ecvector_distann_ip_ops)",
    )
    .expect("empty index creation should succeed");
    Spi::run("REINDEX INDEX ec_distann_empty_lifecycle_idx")
        .expect("reindex should succeed");
    Spi::run("DROP INDEX ec_distann_empty_lifecycle_idx").expect("index drop should succeed");
}

#[pg_test]
fn test_ec_distann_create_on_populated_table_records_metadata() {
    Spi::run("CREATE TABLE ec_distann_populated (id bigint, embedding ecvector)")
        .expect("table creation should succeed");
    Spi::run(
        "INSERT INTO ec_distann_populated VALUES \
         (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)), \
         (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.0, 0.0], 4, 42)), \
         (3, encode_to_ecvector(ARRAY[0.0, 0.0, 1.0, 0.0], 4, 42))",
    )
    .expect("fixture rows should insert");
    Spi::run(
        "CREATE INDEX ec_distann_populated_idx ON ec_distann_populated \
         USING ec_distann (embedding ecvector_distann_ip_ops) \
         WITH (graph_degree = 16, build_list_size = 32, head_index_cap = 64, \
               neighbor_code_format = 'rabitq')",
    )
    .expect("populated index creation should succeed");

    // Block 0 metadata page exists and decodes with the reloption values.
    let index_oid = Spi::get_one::<pg_sys::Oid>(
        "SELECT 'ec_distann_populated_idx'::regclass::oid",
    )
    .expect("SPI query should succeed")
    .expect("index oid should exist");
    let index_relation = IndexRelationGuard::open(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "ec_distann basic test",
    );
    // SAFETY: the guard holds the index relation open for the read.
    let metadata =
        unsafe { crate::am::ec_distann::read_metadata_from_index(index_relation.as_ptr()) }
            .expect("metadata should decode");
    assert_eq!(metadata.format_version, 1);
    assert_eq!(metadata.graph_degree_r, 16);
    assert_eq!(metadata.build_list_size_l, 32);
    assert_eq!(metadata.head_index_cap, 64);
    assert_eq!(
        metadata.neighbor_codec_kind,
        crate::am::ec_distann::page::DISTANN_NEIGHBOR_CODEC_RABITQ
    );
    assert_eq!(metadata.dimensions, 4);
    assert_eq!(metadata.node_count, 3);
    assert_ne!(
        metadata.entry_point,
        crate::storage::page::ItemPointer::INVALID,
        "populated build must record the medoid entry point"
    );
    assert_ne!(
        metadata.directory_head,
        crate::storage::page::ItemPointer::INVALID
    );
    assert_ne!(
        metadata.head_sample_head,
        crate::storage::page::ItemPointer::INVALID
    );
    drop(index_relation);

    Spi::run("DROP INDEX ec_distann_populated_idx").expect("index drop should succeed");
}

fn expect_pg_error(run: impl FnOnce() + std::panic::UnwindSafe) -> String {
    pg_sys::PgTryBuilder::new(|| {
        run();
        "no_error".to_owned()
    })
    .catch_others(|cause| match cause {
        pg_sys::panic::CaughtError::ErrorReport(report)
        | pg_sys::panic::CaughtError::PostgresError(report) => report.message().to_owned(),
        pg_sys::panic::CaughtError::RustPanic { ereport, .. } => ereport.message().to_owned(),
    })
    .execute()
}

#[pg_test]
fn test_ec_distann_rejects_invalid_neighbor_code_format() {
    Spi::run("CREATE TABLE ec_distann_bad_codec (id bigint, embedding ecvector)")
        .expect("table creation should succeed");
    let error = expect_pg_error(|| {
        Spi::run(
            "CREATE INDEX ec_distann_bad_codec_idx ON ec_distann_bad_codec \
             USING ec_distann (embedding ecvector_distann_ip_ops) \
             WITH (neighbor_code_format = 'opq')",
        )
        .expect("this create must error before succeeding");
    });
    assert!(
        error.contains("invalid ec_distann neighbor_code_format reloption"),
        "unexpected error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_rejects_out_of_range_graph_degree() {
    Spi::run("CREATE TABLE ec_distann_bad_degree (id bigint, embedding ecvector)")
        .expect("table creation should succeed");
    let error = expect_pg_error(|| {
        Spi::run(
            "CREATE INDEX ec_distann_bad_degree_idx ON ec_distann_bad_degree \
             USING ec_distann (embedding ecvector_distann_ip_ops) \
             WITH (graph_degree = 1024)",
        )
        .expect("this create must error before succeeding");
    });
    assert!(
        error.contains("out of bounds") && error.contains("graph_degree"),
        "unexpected error: {error}"
    );
}

fn distann_materialized_index(
    index_name: &str,
) -> (
    crate::am::ec_distann::page::DistannMetadataPage,
    crate::storage::page::DataPageChain,
) {
    let index_oid =
        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist");
    let index_relation = IndexRelationGuard::open(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "ec_distann basic test",
    );
    let handle = std::ptr::NonNull::new(index_relation.as_ptr())
        .expect("index relation should be non-null");
    crate::am::ec_distann::reader::materialize_chain_from_index_handle(handle)
        .expect("chain should materialize")
}

// Eight unit-norm fixture vectors (dim 4).
const DISTANN_HALF_SQRT2: f32 = std::f32::consts::FRAC_1_SQRT_2;
const DISTANN_FIXTURE_ROWS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
    [DISTANN_HALF_SQRT2, DISTANN_HALF_SQRT2, 0.0, 0.0],
    [0.0, DISTANN_HALF_SQRT2, DISTANN_HALF_SQRT2, 0.0],
    [0.0, 0.0, DISTANN_HALF_SQRT2, DISTANN_HALF_SQRT2],
    [DISTANN_HALF_SQRT2, 0.0, 0.0, DISTANN_HALF_SQRT2],
];

fn create_distann_fixture(table: &str, with_clause: &str) {
    Spi::run(&format!(
        "CREATE TABLE {table} (id bigint, embedding ecvector)"
    ))
    .expect("table creation should succeed");
    for (row_id, vector) in DISTANN_FIXTURE_ROWS.iter().enumerate() {
        Spi::run(&format!(
            "INSERT INTO {table} VALUES ({}, encode_to_ecvector(ARRAY[{}, {}, {}, {}], 4, 42))",
            row_id + 1,
            vector[0],
            vector[1],
            vector[2],
            vector[3]
        ))
        .expect("fixture row should insert");
    }
    Spi::run(&format!(
        "CREATE INDEX {table}_idx ON {table} \
         USING ec_distann (embedding ecvector_distann_ip_ops) {with_clause}"
    ))
    .expect("index creation should succeed");
}

#[pg_test]
fn test_ec_distann_build_persists_graph_structures() {
    create_distann_fixture(
        "ec_distann_build_shapes",
        "WITH (graph_degree = 4, build_list_size = 16, head_index_cap = 16, \
               neighbor_code_format = 'rabitq')",
    );
    let (metadata, chain) = distann_materialized_index("ec_distann_build_shapes_idx");
    assert_eq!(metadata.node_count, 8);
    let code_len = crate::am::ec_distann::quantizer::metadata_code_len(&metadata)
        .expect("code_len should derive");

    // Directory: eight strictly-ascending entries, each resolving to a
    // node record carrying that vec_id.
    let directory = crate::am::ec_distann::reader::read_directory_chain(
        &chain,
        metadata.directory_head,
        8,
    )
    .expect("directory should read");
    for (vec_id, tid) in &directory {
        let node = crate::am::ec_distann::reader::read_node(
            &chain,
            *tid,
            metadata.graph_degree_r,
            code_len,
        )
        .expect("directory tid should resolve to a node record");
        assert_eq!(node.vec_id, *vec_id);
        assert!(node.is_live());
    }

    // Entry point decodes as a node record (the build medoid).
    let entry = crate::am::ec_distann::reader::read_node(
        &chain,
        metadata.entry_point,
        metadata.graph_degree_r,
        code_len,
    )
    .expect("entry point should resolve");
    assert!(entry.is_live());

    // FR-076-AC-3 groundwork: every embedded neighbor code equals the
    // neighbor record's own search code, so one record read scores
    // neighbors identically to reading each neighbor.
    let mut checked_neighbors = 0;
    for (_, tid) in &directory {
        let node = crate::am::ec_distann::reader::read_node(
            &chain,
            *tid,
            metadata.graph_degree_r,
            code_len,
        )
        .expect("node should decode");
        for slot in 0..usize::from(node.neighbor_count) {
            let neighbor_vec_id = node.neighbor_vec_ids[slot];
            let neighbor_tid =
                crate::am::ec_distann::reader::directory_lookup(&directory, neighbor_vec_id)
                    .expect("neighbor vec_id should resolve through the directory");
            let neighbor = crate::am::ec_distann::reader::read_node(
                &chain,
                neighbor_tid,
                metadata.graph_degree_r,
                code_len,
            )
            .expect("neighbor should decode");
            assert_eq!(
                &node.neighbor_codes[slot * code_len..(slot + 1) * code_len],
                neighbor.search_code.as_slice(),
                "embedded neighbor code must equal the neighbor's search code"
            );
            checked_neighbors += 1;
        }
    }
    assert!(checked_neighbors > 0, "graph should have edges");

    // FR-080: BFS head sample present, within cap, seeded at the medoid.
    let samples = crate::am::ec_distann::reader::read_head_sample_chain(
        &chain,
        metadata.head_sample_head,
        usize::from(metadata.dimensions),
        metadata.head_index_cap as usize,
    )
    .expect("head sample should read");
    assert_eq!(samples.len(), 8, "cap 16 > 8 nodes: sample covers all");
    assert_eq!(samples[0].vec_id, entry.vec_id, "BFS starts at the medoid");
}

#[pg_test]
fn test_ec_distann_search_codes_match_direct_codec_encoding() {
    create_distann_fixture(
        "ec_distann_codec_parity",
        "WITH (graph_degree = 4, neighbor_code_format = 'rabitq')",
    );
    let (metadata, chain) = distann_materialized_index("ec_distann_codec_parity_idx");
    let code_len = crate::am::ec_distann::quantizer::metadata_code_len(&metadata)
        .expect("code_len should derive");

    // heap ctid -> fixture vector, via the repo's ctid::text convention.
    let mut tid_to_row: std::collections::HashMap<(u32, u16), usize> =
        std::collections::HashMap::new();
    for row_id in 1..=DISTANN_FIXTURE_ROWS.len() {
        let ctid = Spi::get_one::<String>(&format!(
            "SELECT ctid::text FROM ec_distann_codec_parity WHERE id = {row_id}"
        ))
        .expect("SPI query should succeed")
        .expect("ctid should exist");
        let trimmed = ctid
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .to_owned();
        let (block, offset) = trimmed
            .split_once(',')
            .expect("ctid should contain block and offset");
        tid_to_row.insert(
            (
                block.trim().parse::<u32>().expect("ctid block"),
                offset.trim().parse::<u16>().expect("ctid offset"),
            ),
            row_id - 1,
        );
    }

    use crate::quant::Quantizer as _;
    let quantizer = crate::quant::rabitq::RaBitQQuantizer::cached_seeded_srht_bits(
        usize::from(metadata.dimensions),
        metadata.seed,
        u8::try_from(metadata.codec_subvector_dim).expect("bits fit u8"),
    )
    .expect("quantizer should build");

    let mut checked = 0;
    for page in chain.pages() {
        for raw in page.tuples() {
            if raw.first().copied()
                != Some(crate::am::ec_distann::tuple::DISTANN_NODE_TAG)
            {
                continue;
            }
            let node = crate::am::ec_distann::tuple::DistannNodeTuple::decode(
                raw,
                metadata.graph_degree_r,
                code_len,
            )
            .expect("node should decode");
            let row = tid_to_row
                .get(&(node.heap_tid.block_number, node.heap_tid.offset_number))
                .expect("record heap_tid should map to a fixture row");
            let expected = quantizer
                .encode_code(&DISTANN_FIXTURE_ROWS[*row])
                .into_vec();
            assert_eq!(
                node.search_code, expected,
                "persisted search code must equal direct codec encoding (FR-076-AC-3)"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 8);
}

#[pg_test]
fn test_ec_distann_rebuild_assigns_identical_vec_ids() {
    // FR-076-AC-2: two builds of the same corpus assign identical vec_ids.
    create_distann_fixture("ec_distann_rebuild_ids", "WITH (graph_degree = 4)");
    let (metadata_before, chain_before) =
        distann_materialized_index("ec_distann_rebuild_ids_idx");
    let directory_before = crate::am::ec_distann::reader::read_directory_chain(
        &chain_before,
        metadata_before.directory_head,
        8,
    )
    .expect("directory should read");

    Spi::run("REINDEX INDEX ec_distann_rebuild_ids_idx").expect("reindex should succeed");

    let (metadata_after, chain_after) =
        distann_materialized_index("ec_distann_rebuild_ids_idx");
    let directory_after = crate::am::ec_distann::reader::read_directory_chain(
        &chain_after,
        metadata_after.directory_head,
        8,
    )
    .expect("directory should read after reindex");

    let ids_before: Vec<u64> = directory_before.iter().map(|(id, _)| *id).collect();
    let ids_after: Vec<u64> = directory_after.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids_before, ids_after);
    assert_eq!(metadata_before.seed, metadata_after.seed);
}

fn assert_distann_self_queries_return_self(table: &str) {
    // Drive scans through the planner/executor: the ec_hnsw debug scan
    // driver is HNSW-specific and cannot exercise this AM.
    Spi::run("SET enable_seqscan = off").expect("disabling seqscan should succeed");
    for (row, vector) in DISTANN_FIXTURE_ROWS.iter().enumerate() {
        let top_id = Spi::get_one::<i64>(&format!(
            "SELECT id FROM {table} \
             ORDER BY embedding <#> ARRAY[{}, {}, {}, {}]::real[] LIMIT 1",
            vector[0], vector[1], vector[2], vector[3]
        ))
        .expect("SPI query should succeed")
        .expect("top row should exist");
        assert_eq!(
            top_id,
            (row + 1) as i64,
            "query vector {row}'s nearest neighbor must be itself"
        );
    }
    Spi::run("RESET enable_seqscan").expect("seqscan reset should succeed");
}

#[pg_test]
fn test_ec_distann_ordered_scan_self_recall_rabitq() {
    create_distann_fixture(
        "ec_distann_scan_rabitq",
        "WITH (graph_degree = 4, build_list_size = 16, head_index_cap = 16, \
               neighbor_code_format = 'rabitq')",
    );
    assert_distann_self_queries_return_self("ec_distann_scan_rabitq");
}

#[pg_test]
fn test_ec_distann_ordered_scan_self_recall_grouped_pq() {
    // Explicit GroupedPq: exercises codebook persistence, the codebook
    // chain reader, and the LUT scoring path end to end.
    create_distann_fixture(
        "ec_distann_scan_grouped",
        "WITH (graph_degree = 4, build_list_size = 16, \
               neighbor_code_format = 'grouped_pq')",
    );
    assert_distann_self_queries_return_self("ec_distann_scan_grouped");
}

#[pg_test]
fn test_ec_distann_default_codec_is_rabitq() {
    // D7 default measured at M0 (task-162 packet 002): rabitq.
    create_distann_fixture("ec_distann_default_codec", "");
    let (metadata, _) = distann_materialized_index("ec_distann_default_codec_idx");
    assert_eq!(
        metadata.neighbor_codec_kind,
        crate::am::ec_distann::page::DISTANN_NEIGHBOR_CODEC_RABITQ
    );
    assert_distann_self_queries_return_self("ec_distann_default_codec");
}

#[pg_test]
fn test_ec_distann_sql_ordered_scan_through_planner() {
    create_distann_fixture(
        "ec_distann_sql_scan",
        "WITH (graph_degree = 4, neighbor_code_format = 'rabitq')",
    );
    Spi::run("SET enable_seqscan = off").expect("disabling seqscan should succeed");
    let (top_id, plan_uses_index) = (
        Spi::get_one::<i64>(
            "SELECT id FROM ec_distann_sql_scan \
             ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::real[] LIMIT 1",
        )
        .expect("SPI query should succeed")
        .expect("top row should exist"),
        Spi::get_one::<String>(
            "EXPLAIN (FORMAT text) SELECT id FROM ec_distann_sql_scan \
             ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::real[] LIMIT 1",
        )
        .expect("EXPLAIN should succeed")
        .expect("plan should exist"),
    );
    assert_eq!(top_id, 1, "unit query e1 must return row 1 first");
    assert!(
        plan_uses_index.contains("Limit"),
        "plan head should be a Limit node: {plan_uses_index}"
    );
    Spi::run("SET ec_distann.scan_profile_notice = on")
        .expect("profile notice GUC should set");
    let probed = Spi::get_one::<i64>(
        "SELECT id FROM ec_distann_sql_scan \
         ORDER BY embedding <#> ARRAY[0.0, 1.0, 0.0, 0.0]::real[] LIMIT 1",
    )
    .expect("SPI query should succeed")
    .expect("top row should exist");
    assert_eq!(probed, 2);
    Spi::run("RESET ec_distann.scan_profile_notice").expect("GUC reset should succeed");
    Spi::run("RESET enable_seqscan").expect("seqscan reset should succeed");
}

#[pg_test]
fn test_ec_distann_ordered_scan_scores_are_monotone() {
    // FR-075-AC-3: ordered top-k scans return results in non-increasing
    // score order (non-decreasing <#> operator values).
    create_distann_fixture(
        "ec_distann_scan_order",
        "WITH (graph_degree = 4, neighbor_code_format = 'rabitq')",
    );
    Spi::run("SET enable_seqscan = off").expect("disabling seqscan should succeed");
    Spi::connect(|client| {
        let rows = client
            .select(
                "SELECT embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::real[] AS score \
                 FROM ec_distann_scan_order \
                 ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::real[] LIMIT 5",
                None,
                &[],
            )
            .expect("ordered select should succeed");
        let mut previous = f32::NEG_INFINITY;
        let mut count = 0;
        for row in rows {
            let score = row
                .get_datum_by_ordinal(1)
                .expect("score datum")
                .value::<f32>()
                .expect("score should convert")
                .expect("score should be non-null");
            assert!(
                score >= previous,
                "scores must be non-decreasing: {score} after {previous}"
            );
            previous = score;
            count += 1;
        }
        assert_eq!(count, 5);
    });
    Spi::run("RESET enable_seqscan").expect("seqscan reset should succeed");
}

#[pg_test]
fn test_ec_distann_head_sample_is_deterministic_across_reindex() {
    // FR-080-AC-2: head construction is deterministic for a fixed seed;
    // the persisted BFS sample must be identical across rebuilds of the
    // same corpus.
    create_distann_fixture("ec_distann_head_determinism", "WITH (graph_degree = 4)");
    let read_sample = || {
        let (metadata, chain) = distann_materialized_index("ec_distann_head_determinism_idx");
        crate::am::ec_distann::reader::read_head_sample_chain(
            &chain,
            metadata.head_sample_head,
            usize::from(metadata.dimensions),
            metadata.head_index_cap as usize,
        )
        .expect("head sample should read")
        .into_iter()
        .map(|sample| (sample.vec_id, sample.vector))
        .collect::<Vec<_>>()
    };
    let before = read_sample();
    Spi::run("REINDEX INDEX ec_distann_head_determinism_idx").expect("reindex should succeed");
    let after = read_sample();
    assert_eq!(before, after, "BFS head sample must be rebuild-deterministic");
}

#[pg_test]
fn test_ec_distann_limit_beyond_top_k_deepens_correctly() {
    // F4 regression (packet 003 feedback): a LIMIT above ec_distann.top_k
    // must return the same ordering as a scan whose exit bar covers the
    // LIMIT — the proven-prefix guard re-runs with a deeper bar instead of
    // serving unproven rows.
    create_distann_fixture(
        "ec_distann_deepening",
        "WITH (graph_degree = 4, neighbor_code_format = 'rabitq')",
    );
    Spi::run("SET enable_seqscan = off").expect("disabling seqscan should succeed");
    let ordered_ids = |top_k: i32| -> Vec<i64> {
        Spi::run(&format!("SET ec_distann.top_k = {top_k}")).expect("GUC set should succeed");
        Spi::get_one::<Vec<i64>>(
            "SELECT array_agg(id) FROM (SELECT id FROM ec_distann_deepening \
             ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::real[] LIMIT 6) q",
        )
        .expect("SPI query should succeed")
        .expect("ids should exist")
    };
    let shallow = ordered_ids(2);
    let deep = ordered_ids(200);
    assert_eq!(shallow.len(), 6, "LIMIT 6 must yield 6 rows even at top_k=2");
    assert_eq!(
        shallow, deep,
        "deepened scan must match a scan whose exit bar covers the LIMIT"
    );
    Spi::run("RESET ec_distann.top_k").expect("GUC reset should succeed");
    Spi::run("RESET enable_seqscan").expect("seqscan reset should succeed");
}

#[pg_test]
fn test_ec_distann_head_cache_invalidates_across_reindex() {
    // Repeated scans in one backend across a REINDEX: the fingerprint-keyed
    // head cache must refresh (stale chain heads would break directory or
    // head-sample reads).
    create_distann_fixture(
        "ec_distann_cache_reindex",
        "WITH (graph_degree = 4, neighbor_code_format = 'rabitq')",
    );
    assert_distann_self_queries_return_self("ec_distann_cache_reindex");
    Spi::run("REINDEX INDEX ec_distann_cache_reindex_idx").expect("reindex should succeed");
    assert_distann_self_queries_return_self("ec_distann_cache_reindex");
}

fn create_distann_identity_fixture(table: &str, tid_shift: bool) {
    // ADR-063 include-provider fixture: stable uuid identities per logical
    // row. `tid_shift` reverses the insertion order so two tables carry the
    // same identities at different physical heap addresses.
    Spi::run(&format!(
        "CREATE TABLE {table} (id bigint, ident uuid, embedding ecvector)"
    ))
    .expect("table creation should succeed");
    let order: Vec<usize> = if tid_shift {
        (0..DISTANN_FIXTURE_ROWS.len()).rev().collect()
    } else {
        (0..DISTANN_FIXTURE_ROWS.len()).collect()
    };
    for row_id in order {
        let vector = &DISTANN_FIXTURE_ROWS[row_id];
        Spi::run(&format!(
            "INSERT INTO {table} VALUES ({}, '00000000-0000-0000-0000-{:012}', \
             encode_to_ecvector(ARRAY[{}, {}, {}, {}], 4, 42))",
            row_id + 1,
            row_id + 1,
            vector[0],
            vector[1],
            vector[2],
            vector[3]
        ))
        .expect("fixture row should insert");
    }
    Spi::run(&format!(
        "CREATE INDEX {table}_idx ON {table} \
         USING ec_distann (embedding ecvector_distann_ip_ops) INCLUDE (ident) \
         WITH (source_identity = 'include', graph_degree = 4, \
               neighbor_code_format = 'rabitq')"
    ))
    .expect("include-mode index creation should succeed");
}

fn distann_directory_vec_ids(index_name: &str) -> Vec<u64> {
    let (metadata, chain) = distann_materialized_index(index_name);
    crate::am::ec_distann::reader::read_directory_chain(
        &chain,
        metadata.directory_head,
        metadata.node_count as usize,
    )
    .expect("directory should read")
    .into_iter()
    .map(|(vec_id, _)| vec_id)
    .collect()
}

#[pg_test]
fn test_ec_distann_include_identity_is_tid_independent() {
    // FR-076-AC-2 in global-identity mode: the same logical rows (same
    // uuids) get identical vec_ids regardless of heap TID layout — the
    // property local mode cannot provide across table rewrites.
    create_distann_identity_fixture("ec_distann_ident_a", false);
    create_distann_identity_fixture("ec_distann_ident_b", true);
    let ids_a = distann_directory_vec_ids("ec_distann_ident_a_idx");
    let ids_b = distann_directory_vec_ids("ec_distann_ident_b_idx");
    assert_eq!(ids_a.len(), 8);
    assert_eq!(
        ids_a, ids_b,
        "identity-derived vec_ids must not depend on heap TIDs"
    );

    // And the include-mode index scans correctly through the planner.
    Spi::run("SET enable_seqscan = off").expect("disabling seqscan should succeed");
    let top_id = Spi::get_one::<i64>(
        "SELECT id FROM ec_distann_ident_a \
         ORDER BY embedding <#> ARRAY[0.0, 1.0, 0.0, 0.0]::real[] LIMIT 1",
    )
    .expect("SPI query should succeed")
    .expect("top row should exist");
    assert_eq!(top_id, 2);
    Spi::run("RESET enable_seqscan").expect("seqscan reset should succeed");
}

#[pg_test]
fn test_ec_distann_include_mode_requires_include_column() {
    Spi::run("CREATE TABLE ec_distann_ident_noinc (id bigint, embedding ecvector)")
        .expect("table creation should succeed");
    let error = expect_pg_error(|| {
        Spi::run(
            "CREATE INDEX ec_distann_ident_noinc_idx ON ec_distann_ident_noinc \
             USING ec_distann (embedding ecvector_distann_ip_ops) \
             WITH (source_identity = 'include')",
        )
        .expect("this create must error before succeeding");
    });
    assert!(
        error.contains("requires exactly one INCLUDE column"),
        "unexpected error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_include_mode_rejects_null_identity() {
    Spi::run("CREATE TABLE ec_distann_ident_null (id bigint, ident uuid, embedding ecvector)")
        .expect("table creation should succeed");
    Spi::run(
        "INSERT INTO ec_distann_ident_null VALUES \
         (1, NULL, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
    )
    .expect("row should insert");
    let error = expect_pg_error(|| {
        Spi::run(
            "CREATE INDEX ec_distann_ident_null_idx ON ec_distann_ident_null \
             USING ec_distann (embedding ecvector_distann_ip_ops) INCLUDE (ident) \
             WITH (source_identity = 'include')",
        )
        .expect("this create must error before succeeding");
    });
    assert!(
        error.contains("source_identity INCLUDE column"),
        "unexpected error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_include_mode_rejects_short_bytea_identity() {
    Spi::run(
        "CREATE TABLE ec_distann_ident_width (id bigint, ident bytea, embedding ecvector)",
    )
    .expect("table creation should succeed");
    Spi::run(
        "INSERT INTO ec_distann_ident_width VALUES \
         (1, '\\x0102030405'::bytea, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
    )
    .expect("row should insert");
    let error = expect_pg_error(|| {
        Spi::run(
            "CREATE INDEX ec_distann_ident_width_idx ON ec_distann_ident_width \
             USING ec_distann (embedding ecvector_distann_ip_ops) INCLUDE (ident) \
             WITH (source_identity = 'include')",
        )
        .expect("this create must error before succeeding");
    });
    assert!(
        error.contains("must be 16 bytes"),
        "unexpected error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_rejects_stray_include_column() {
    Spi::run(
        "CREATE TABLE ec_distann_ident_stray (id bigint, ident uuid, embedding ecvector)",
    )
    .expect("table creation should succeed");
    let error = expect_pg_error(|| {
        Spi::run(
            "CREATE INDEX ec_distann_ident_stray_idx ON ec_distann_ident_stray \
             USING ec_distann (embedding ecvector_distann_ip_ops) INCLUDE (ident)",
        )
        .expect("this create must error before succeeding");
    });
    assert!(
        error.contains("takes no INCLUDE columns"),
        "unexpected error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_guc_defaults() {
    let beam_width = Spi::get_one::<String>("SHOW ec_distann.beam_width")
        .expect("SPI query should succeed")
        .expect("GUC should exist");
    assert_eq!(beam_width, "4");
    let hop_rounds = Spi::get_one::<String>("SHOW ec_distann.hop_rounds")
        .expect("SPI query should succeed")
        .expect("GUC should exist");
    assert_eq!(hop_rounds, "100");
    let top_k = Spi::get_one::<String>("SHOW ec_distann.top_k")
        .expect("SPI query should succeed")
        .expect("GUC should exist");
    assert_eq!(top_k, "10");
}

#[pg_test]
fn test_ec_distann_insert_reports_unimplemented_dml_posture() {
    Spi::run("CREATE TABLE ec_distann_insert_posture (id bigint, embedding ecvector)")
        .expect("table creation should succeed");
    Spi::run(
        "CREATE INDEX ec_distann_insert_posture_idx ON ec_distann_insert_posture \
         USING ec_distann (embedding ecvector_distann_ip_ops)",
    )
    .expect("index creation should succeed");
    let error = expect_pg_error(|| {
        Spi::run(
            "INSERT INTO ec_distann_insert_posture VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("this insert must error before succeeding");
    });
    assert!(
        error.contains("ec_distann aminsert is not implemented yet"),
        "unexpected error: {error}"
    );
}
