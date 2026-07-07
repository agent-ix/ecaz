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
    assert_eq!(
        metadata.entry_point,
        crate::storage::page::ItemPointer::INVALID,
        "scaffold slice writes no graph yet"
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

#[pg_test]
fn test_ec_distann_guc_defaults() {
    let beam_width = Spi::get_one::<String>("SHOW ec_distann.beam_width")
        .expect("SPI query should succeed")
        .expect("GUC should exist");
    assert_eq!(beam_width, "4");
    let hop_rounds = Spi::get_one::<String>("SHOW ec_distann.hop_rounds")
        .expect("SPI query should succeed")
        .expect("GUC should exist");
    assert_eq!(hop_rounds, "8");
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
