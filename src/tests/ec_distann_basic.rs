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
    Spi::run("REINDEX INDEX ec_distann_empty_lifecycle_idx").expect("reindex should succeed");
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
    let index_oid = Spi::get_one::<pg_sys::Oid>("SELECT 'ec_distann_populated_idx'::regclass::oid")
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
    assert_eq!(
        metadata.format_version,
        crate::am::ec_distann::page::INDEX_FORMAT_V1_DISTANN
    );
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

#[pg_test]
fn test_distann_control_metadata_and_fail_closed() {
    Spi::run(
        "CREATE TABLE ec_distann_control (
             source_id uuid NOT NULL,
             embedding ecvector(4) NOT NULL
         )",
    )
    .expect("control source table should create");
    Spi::run(
        "INSERT INTO ec_distann_control VALUES
         ('00000000-0000-4000-8000-000000000001', encode_to_ecvector(ARRAY[1.0,0.0,0.0,0.0], 4, 42)),
         ('00000000-0000-4000-8000-000000000002', encode_to_ecvector(ARRAY[0.0,1.0,0.0,0.0], 4, 42))",
    )
    .expect("control source rows should insert");
    Spi::run(
        "CREATE INDEX ec_distann_control_idx ON ec_distann_control
         USING ec_distann (embedding ecvector_distann_ip_ops)
         INCLUDE (source_id)
         WITH (distributed_control = true, source_identity = 'include')",
    )
    .expect("metadata-only control index should create");

    let index_oid = Spi::get_one::<pg_sys::Oid>("SELECT 'ec_distann_control_idx'::regclass::oid")
        .unwrap()
        .unwrap();
    let index_relation = IndexRelationGuard::open(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "ec_distann control metadata test",
    );
    let metadata =
        unsafe { crate::am::ec_distann::read_metadata_from_index(index_relation.as_ptr()) }
            .expect("v5 control metadata should decode");
    assert_eq!(
        metadata.format_version,
        crate::am::ec_distann::page::INDEX_FORMAT_V5_DISTANN_CONTROL
    );
    assert!(metadata.is_distributed_control());
    assert_ne!(metadata.logical_index_uuid, [0; 16]);
    assert_eq!(metadata.logical_index_uuid[6] & 0xf0, 0x40);
    assert_eq!(metadata.logical_index_uuid[8] & 0xc0, 0x80);
    assert_eq!(metadata.dimensions, 0);
    assert_eq!(metadata.node_count, 0);
    assert_eq!(metadata.active_epoch, 0);
    let handle = std::ptr::NonNull::new(index_relation.as_ptr()).unwrap();
    assert_eq!(
        crate::storage::relation::main_fork_block_count_handle(handle),
        1,
        "control relation contains only block-0 metadata"
    );
    drop(index_relation);

    // The Task 165 v4 lifecycle helpers must not provide a side door around
    // Task 179's catalog-backed Ready/decision/participant-publish protocol.
    for statement in [
        "SELECT ec_distann_publish_epoch('ec_distann_control_idx'::regclass::oid, 9)",
        "SELECT ec_distann_retire_epoch('ec_distann_control_idx'::regclass::oid)",
        "SELECT ec_distann_force_retire_epoch('ec_distann_control_idx'::regclass::oid)",
        "SELECT ec_distann_debug_set_in_flight('ec_distann_control_idx'::regclass::oid, 1)",
        "SELECT * FROM ec_distann_epoch_status('ec_distann_control_idx'::regclass::oid)",
    ] {
        let error = expect_pg_error_rolled_back(|| {
            Spi::run(statement).expect("legacy lifecycle call must reject v5 control indexes");
        });
        assert!(
            error.contains("EC_EPOCH_STATE")
                && error.contains("legacy metadata-page lifecycle endpoint"),
            "unexpected lifecycle-gate error for {statement}: {error}"
        );
    }

    for statement in [
        "SELECT * FROM ec_distann_list_directory('ec_distann_control_idx'::regclass::oid)",
        "SELECT ec_distann_epoch_fingerprint('ec_distann_control_idx'::regclass::oid)",
        "SELECT ec_distann_fold_delta_into_graph('ec_distann_control_idx'::regclass::oid)",
        "SELECT ec_distann_debug_tombstone('ec_distann_control_idx'::regclass::oid, ARRAY[]::bigint[])",
    ] {
        let error = expect_pg_error(|| {
            Spi::run(statement).expect("legacy local-storage endpoint must reject v5 control");
        });
        assert!(
            error.contains("EC_GENERATION_MISSING"),
            "unexpected local-storage gate error for {statement}: {error}"
        );
    }

    let index_relation = IndexRelationGuard::open(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "ec_distann lifecycle gate metadata check",
    );
    let metadata =
        unsafe { crate::am::ec_distann::read_metadata_from_index(index_relation.as_ptr()) }
            .expect("rejected lifecycle calls must leave control metadata readable");
    assert_eq!(metadata.active_epoch, 0);
    assert_eq!(metadata.in_flight_count, 0);
    drop(index_relation);

    Spi::run("SET enable_seqscan = off").unwrap();
    crate::am::ec_distann::reset_exec_state_context_cleanups_for_test();
    let scan_error = expect_pg_error_rolled_back(|| {
        Spi::get_one::<i64>(
            "SELECT 1 FROM ec_distann_control
             ORDER BY embedding <#> ARRAY[1.0,0.0,0.0,0.0]::real[] LIMIT 1",
        )
        .expect("direct control scan must fail before returning");
    });
    assert!(
        scan_error.contains("EC_GENERATION_MISSING")
            && scan_error.contains("logical index has no active epoch"),
        "unexpected direct-scan error: {scan_error}"
    );
    assert_eq!(
        crate::am::ec_distann::exec_state_context_cleanups_for_test(),
        1,
        "query-context rollback must destroy the CustomScan Rust state",
    );
    Spi::run("SET enable_customscan = off").unwrap();
    let amgettuple_error = expect_pg_error_rolled_back(|| {
        Spi::get_one::<i64>(
            "SELECT 1 FROM ec_distann_control
             ORDER BY embedding <#> ARRAY[1.0,0.0,0.0,0.0]::real[] LIMIT 1",
        )
        .expect("raw index scan of a control index must fail before returning");
    });
    assert!(
        amgettuple_error.contains("EC_DISTANN_CONTROL_SCAN"),
        "unexpected raw-index backstop error: {amgettuple_error}"
    );
    Spi::run("RESET enable_customscan").unwrap();
    Spi::run("RESET enable_seqscan").unwrap();

    let insert_error = expect_pg_error(|| {
        Spi::run(
            "INSERT INTO ec_distann_control VALUES
             ('00000000-0000-4000-8000-000000000003', encode_to_ecvector(ARRAY[0.0,0.0,1.0,0.0], 4, 42))",
        )
        .expect("control insert without a Published generation must fail");
    });
    assert!(
        insert_error.contains("EC_GENERATION_MISSING"),
        "unexpected control insert error: {insert_error}"
    );
}

#[pg_test]
fn test_distann_control_requires_include_identity() {
    Spi::run("CREATE TABLE ec_distann_control_no_identity (embedding ecvector NOT NULL)")
        .expect("source table should create");
    let error = expect_pg_error(|| {
        Spi::run(
            "CREATE INDEX ec_distann_control_no_identity_idx
             ON ec_distann_control_no_identity
             USING ec_distann (embedding ecvector_distann_ip_ops)
             WITH (distributed_control = true)",
        )
        .expect("control index without source identity must fail");
    });
    assert!(
        error.contains("distributed_control=true requires source_identity='include'"),
        "unexpected missing-identity error: {error}"
    );
}

#[pg_test]
fn test_distann_control_requires_not_null_identity() {
    Spi::run(
        "CREATE TABLE ec_distann_control_nullable_identity (
             source_id uuid,
             embedding ecvector NOT NULL
         )",
    )
    .expect("nullable-identity source table should create");
    let error = expect_pg_error_rolled_back(|| {
        Spi::run(
            "CREATE INDEX ec_distann_control_nullable_identity_idx
             ON ec_distann_control_nullable_identity
             USING ec_distann (embedding ecvector_distann_ip_ops)
             INCLUDE (source_id)
             WITH (distributed_control = true, source_identity = 'include')",
        )
        .expect("nullable physical identity must fail");
    });
    assert!(
        error.contains("EC_SOURCE_IDENTITY") && error.contains("NOT NULL"),
        "unexpected nullable-identity error: {error}"
    );
}

#[pg_test]
fn test_distann_control_requires_typed_vector_key() {
    Spi::run(
        "CREATE TABLE ec_distann_control_untyped_key (
             source_id uuid NOT NULL,
             embedding ecvector NOT NULL
         )",
    )
    .unwrap();
    let error = expect_pg_error_rolled_back(|| {
        Spi::run(
            "CREATE INDEX ec_distann_control_untyped_key_idx
             ON ec_distann_control_untyped_key
             USING ec_distann (embedding ecvector_distann_ip_ops)
             INCLUDE (source_id)
             WITH (distributed_control = true, source_identity = 'include')",
        )
        .expect("untyped distributed-control vector key must fail");
    });
    assert!(
        error.contains("EC_SCHEMA_UNSUPPORTED") && error.contains("typmod"),
        "unexpected untyped-key error: {error}"
    );
}

#[pg_test]
fn test_distann_control_requires_permanent_wal_logged_relation() {
    Spi::run(
        "CREATE UNLOGGED TABLE ec_distann_control_unlogged (
             source_id uuid NOT NULL,
             embedding ecvector NOT NULL
         )",
    )
    .expect("unlogged source table should create");
    let error = expect_pg_error(|| {
        Spi::run(
            "CREATE INDEX ec_distann_control_unlogged_idx
             ON ec_distann_control_unlogged
             USING ec_distann (embedding ecvector_distann_ip_ops)
             INCLUDE (source_id)
             WITH (distributed_control = true, source_identity = 'include')",
        )
        .expect("unlogged distributed control must fail");
    });
    assert!(
        error.contains("EC_CONTROL_PERSISTENCE"),
        "unexpected persistence error: {error}"
    );
}

#[pg_test]
fn test_distann_control_rejects_temporary_relation() {
    Spi::run(
        "CREATE TEMP TABLE ec_distann_control_temp (
             source_id uuid NOT NULL,
             embedding ecvector NOT NULL
         )",
    )
    .expect("temporary source table should create");
    let error = expect_pg_error(|| {
        Spi::run(
            "CREATE INDEX ec_distann_control_temp_idx
             ON ec_distann_control_temp
             USING ec_distann (embedding ecvector_distann_ip_ops)
             INCLUDE (source_id)
             WITH (distributed_control = true, source_identity = 'include')",
        )
        .expect("temporary distributed control must fail");
    });
    assert!(
        error.contains("EC_CONTROL_PERSISTENCE"),
        "unexpected temporary-control error: {error}"
    );
}

#[pg_test]
fn test_distann_control_mode_change_reindex() {
    Spi::run(
        "CREATE TABLE ec_distann_control_mode (
             source_id uuid NOT NULL,
             embedding ecvector(4) NOT NULL
         );
         INSERT INTO ec_distann_control_mode VALUES
           ('00000000-0000-4000-8000-000000000021', encode_to_ecvector(ARRAY[1.0,0.0,0.0,0.0], 4, 42)),
           ('00000000-0000-4000-8000-000000000022', encode_to_ecvector(ARRAY[0.0,1.0,0.0,0.0], 4, 42));
         CREATE INDEX ec_distann_control_mode_idx
           ON ec_distann_control_mode USING ec_distann (embedding ecvector_distann_ip_ops)
           INCLUDE (source_id) WITH (source_identity = 'include', graph_degree = 4)",
    )
    .expect("legacy fixture should create");
    let index_oid =
        Spi::get_one::<pg_sys::Oid>("SELECT 'ec_distann_control_mode_idx'::regclass::oid")
            .unwrap()
            .unwrap();
    let read = || {
        let relation = IndexRelationGuard::open(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "ec_distann mode-change metadata test",
        );
        unsafe { crate::am::ec_distann::read_metadata_from_index(relation.as_ptr()) }
            .expect("metadata should decode")
    };
    let legacy = read();
    assert_eq!(
        legacy.format_version,
        crate::am::ec_distann::page::INDEX_FORMAT_V1_DISTANN
    );
    assert_eq!(legacy.node_count, 2);

    Spi::run("ALTER INDEX ec_distann_control_mode_idx SET (distributed_control = true)")
        .expect("mode reloption ALTER is deferred to REINDEX");
    assert_eq!(
        read().format_version,
        crate::am::ec_distann::page::INDEX_FORMAT_V1_DISTANN
    );
    Spi::run("REINDEX INDEX ec_distann_control_mode_idx")
        .expect("explicit REINDEX should convert to control mode");
    let control = read();
    assert!(control.is_distributed_control());
    assert_ne!(control.logical_index_uuid, [0; 16]);
    assert_eq!(control.node_count, 0);

    Spi::run("ALTER INDEX ec_distann_control_mode_idx SET (distributed_control = false)")
        .expect("reverse mode reloption ALTER is deferred to REINDEX");
    assert!(read().is_distributed_control());
    Spi::run("REINDEX INDEX ec_distann_control_mode_idx")
        .expect("explicit REINDEX should destructively convert to legacy mode");
    let rebuilt_legacy = read();
    assert_eq!(
        rebuilt_legacy.format_version,
        crate::am::ec_distann::page::INDEX_FORMAT_V1_DISTANN
    );
    assert_eq!(rebuilt_legacy.logical_index_uuid, [0; 16]);
    assert_eq!(rebuilt_legacy.node_count, 2);
}

#[pg_test]
fn test_distann_control_vacuum_and_concurrent_create() {
    let mut client =
        postgres::Client::connect(&current_pg_test_loopback_conninfo(), postgres::NoTls)
            .expect("loopback connection should open");
    client
        .batch_execute(
            "DROP TABLE IF EXISTS ec_distann_control_loopback CASCADE;
             CREATE TABLE ec_distann_control_loopback (
               source_id uuid NOT NULL,
               embedding ecvector(4) NOT NULL
             );
             INSERT INTO ec_distann_control_loopback VALUES
               ('00000000-0000-4000-8000-000000000031', encode_to_ecvector(ARRAY[1.0,0.0,0.0,0.0], 4, 42))",
        )
        .expect("loopback fixture should create");

    let concurrent_error = client
        .batch_execute(
            "CREATE INDEX CONCURRENTLY ec_distann_control_loopback_idx
               ON ec_distann_control_loopback USING ec_distann (embedding ecvector_distann_ip_ops)
               INCLUDE (source_id)
               WITH (distributed_control = true, source_identity = 'include')",
        )
        .expect_err("concurrent control creation must fail during validation");
    let concurrent_message = concurrent_error
        .as_db_error()
        .map(|error| error.message().to_owned())
        .unwrap_or_else(|| concurrent_error.to_string());
    assert!(
        concurrent_message.contains("EC_GENERATION_MISSING"),
        "unexpected concurrent-create error: {concurrent_message}"
    );
    let invalid: bool = client
        .query_one(
            "SELECT NOT i.indisvalid
               FROM pg_index i
               JOIN pg_class c ON c.oid = i.indexrelid
              WHERE c.relname = 'ec_distann_control_loopback_idx'",
            &[],
        )
        .expect("invalid concurrent index should remain inspectable")
        .get(0);
    assert!(invalid);
    client
        .batch_execute(
            "DROP INDEX ec_distann_control_loopback_idx;
             TRUNCATE ec_distann_control_loopback",
        )
        .expect("populated concurrent-build artifact should clean up");

    let empty_concurrent_error = client
        .batch_execute(
            "CREATE INDEX CONCURRENTLY ec_distann_control_loopback_idx
               ON ec_distann_control_loopback USING ec_distann (embedding ecvector_distann_ip_ops)
               INCLUDE (source_id)
               WITH (distributed_control = true, source_identity = 'include')",
        )
        .expect_err("concurrent control creation must also fail for an empty source");
    let empty_concurrent_message = empty_concurrent_error
        .as_db_error()
        .map(|error| error.message().to_owned())
        .unwrap_or_else(|| empty_concurrent_error.to_string());
    assert!(
        empty_concurrent_message.contains("EC_GENERATION_MISSING"),
        "unexpected empty concurrent-create error: {empty_concurrent_message}"
    );
    let empty_invalid: bool = client
        .query_one(
            "SELECT NOT i.indisvalid
               FROM pg_index i
               JOIN pg_class c ON c.oid = i.indexrelid
              WHERE c.relname = 'ec_distann_control_loopback_idx'",
            &[],
        )
        .expect("empty invalid concurrent index should remain inspectable")
        .get(0);
    assert!(empty_invalid);

    client
        .batch_execute(
            "DROP INDEX ec_distann_control_loopback_idx;
             INSERT INTO ec_distann_control_loopback VALUES
               ('00000000-0000-4000-8000-000000000031', encode_to_ecvector(ARRAY[1.0,0.0,0.0,0.0], 4, 42));
             CREATE INDEX ec_distann_control_loopback_idx
               ON ec_distann_control_loopback USING ec_distann (embedding ecvector_distann_ip_ops)
               INCLUDE (source_id)
               WITH (distributed_control = true, source_identity = 'include');
             DELETE FROM ec_distann_control_loopback",
        )
        .expect("regular control index and dead source tuple should be prepared");
    client
        .batch_execute("VACUUM ec_distann_control_loopback")
        .expect("VACUUM must treat the empty logical control root as a no-op index");
    client
        .batch_execute("DROP TABLE ec_distann_control_loopback CASCADE")
        .expect("loopback control fixture should clean up");
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

fn expect_pg_error_rolled_back(run: impl FnOnce() + std::panic::UnwindSafe) -> String {
    // PgTryBuilder catches ereport(ERROR), but it does not itself establish a
    // savepoint. Use a real internal subtransaction so this helper can assert
    // PostgreSQL's transactional DDL/catalog rollback instead of merely
    // continuing after an error with the earlier statements still applied.
    let (old_context, old_owner) = unsafe {
        let old_context = pg_sys::CurrentMemoryContext;
        let old_owner = pg_sys::CurrentResourceOwner;
        // Make the fixture DDL issued by the outer test command visible before
        // the subtransaction starts. PostgreSQL's internal-subtransaction
        // convention then restores the caller's memory context while retaining
        // the child resource owner until release/rollback.
        pg_sys::CommandCounterIncrement();
        pg_sys::BeginInternalSubTransaction(std::ptr::null());
        pg_sys::MemoryContextSwitchTo(old_context);
        (old_context, old_owner)
    };
    // The SQL function invocation already owns an active statement snapshot.
    // Push a fresh post-CCI snapshot so nested SPI catalog reads can see the
    // fixture DDL from the outer transaction while their writes remain owned
    // by this subtransaction.
    let snapshot =
        crate::storage::snapshot_guard::ActiveSnapshotGuard::transaction_after_command_counter()
            .expect("rollback test subtransaction requires an active snapshot");
    let error = expect_pg_error(run);
    drop(snapshot);
    unsafe {
        pg_sys::RollbackAndReleaseCurrentSubTransaction();
        pg_sys::MemoryContextSwitchTo(old_context);
        pg_sys::CurrentResourceOwner = old_owner;
    }
    error
}

fn run_in_committed_subtransaction<T>(run: impl FnOnce() -> T) -> T {
    let (old_context, old_owner) = unsafe {
        let old_context = pg_sys::CurrentMemoryContext;
        let old_owner = pg_sys::CurrentResourceOwner;
        pg_sys::CommandCounterIncrement();
        pg_sys::BeginInternalSubTransaction(std::ptr::null());
        pg_sys::MemoryContextSwitchTo(old_context);
        (old_context, old_owner)
    };
    let snapshot =
        crate::storage::snapshot_guard::ActiveSnapshotGuard::transaction_after_command_counter()
            .expect("committed test subtransaction requires an active snapshot");
    let result = run();
    drop(snapshot);
    unsafe {
        pg_sys::ReleaseCurrentSubTransaction();
        pg_sys::MemoryContextSwitchTo(old_context);
        pg_sys::CurrentResourceOwner = old_owner;
    }
    result
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
    let index_oid = Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
        .expect("SPI query should succeed")
        .expect("index oid should exist");
    let index_relation = IndexRelationGuard::open(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "ec_distann basic test",
    );
    let handle =
        std::ptr::NonNull::new(index_relation.as_ptr()).expect("index relation should be non-null");
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
    let directory =
        crate::am::ec_distann::reader::read_directory_chain(&chain, metadata.directory_head, 8)
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
            if raw.first().copied() != Some(crate::am::ec_distann::tuple::DISTANN_NODE_TAG) {
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
    let (metadata_before, chain_before) = distann_materialized_index("ec_distann_rebuild_ids_idx");
    let directory_before = crate::am::ec_distann::reader::read_directory_chain(
        &chain_before,
        metadata_before.directory_head,
        8,
    )
    .expect("directory should read");

    Spi::run("REINDEX INDEX ec_distann_rebuild_ids_idx").expect("reindex should succeed");

    let (metadata_after, chain_after) = distann_materialized_index("ec_distann_rebuild_ids_idx");
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
fn test_ec_distann_sharded_build_self_recall() {
    // FR-077 M1: the sharded closure-overlap build + stitch returns valid
    // results through the same scan path as the monolithic build. Drive four
    // build shards over the eight-row fixture; self queries must still return
    // self (the stitched graph is navigable end to end).
    create_distann_fixture(
        "ec_distann_sharded_recall",
        "WITH (graph_degree = 4, build_list_size = 16, head_index_cap = 16, \
               build_shards = 4, closure_epsilon = 0.25, neighbor_code_format = 'rabitq')",
    );
    let (metadata, _) = distann_materialized_index("ec_distann_sharded_recall_idx");
    assert_eq!(
        metadata.node_count,
        DISTANN_FIXTURE_ROWS.len() as u64,
        "sharded stitch must emit exactly one record per vec_id"
    );
    assert_ne!(
        metadata.entry_point,
        crate::storage::page::ItemPointer::INVALID,
        "sharded build must set the entry medoid"
    );
    assert_distann_self_queries_return_self("ec_distann_sharded_recall");
}

#[pg_test]
fn test_ec_distann_sharded_build_is_deterministic_across_reindex() {
    // FR-077 determinism: identical corpus + seed + options => identical
    // stitched graph. Two sharded builds must produce the same vec_id
    // directory (the M2 single-vs-multinode result-identity contract).
    create_distann_fixture(
        "ec_distann_sharded_determinism",
        "WITH (graph_degree = 4, build_shards = 3, closure_epsilon = 0.2)",
    );
    let (metadata_before, chain_before) =
        distann_materialized_index("ec_distann_sharded_determinism_idx");
    let directory_before = crate::am::ec_distann::reader::read_directory_chain(
        &chain_before,
        metadata_before.directory_head,
        DISTANN_FIXTURE_ROWS.len(),
    )
    .expect("directory should read");

    Spi::run("REINDEX INDEX ec_distann_sharded_determinism_idx").expect("reindex should succeed");

    let (metadata_after, chain_after) =
        distann_materialized_index("ec_distann_sharded_determinism_idx");
    let directory_after = crate::am::ec_distann::reader::read_directory_chain(
        &chain_after,
        metadata_after.directory_head,
        DISTANN_FIXTURE_ROWS.len(),
    )
    .expect("directory should read after reindex");

    assert_eq!(
        directory_before, directory_after,
        "sharded build must be deterministic across reindex"
    );
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
    Spi::run("SET ec_distann.scan_profile_notice = on").expect("profile notice GUC should set");
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
    assert_eq!(
        before, after,
        "BFS head sample must be rebuild-deterministic"
    );
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
    assert_eq!(
        shallow.len(),
        6,
        "LIMIT 6 must yield 6 rows even at top_k=2"
    );
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
    Spi::run("CREATE TABLE ec_distann_ident_width (id bigint, ident bytea, embedding ecvector)")
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
    Spi::run("CREATE TABLE ec_distann_ident_stray (id bigint, ident uuid, embedding ecvector)")
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

// ── M2 (Task 164): FR-079 ec_distann_expand_nodes remote endpoint ──────────

#[pg_test]
fn test_ec_distann_content_digest_binds_build_content() {
    // Reviewer 2026-07-08-01 P1: two indexes with IDENTICAL shape metadata
    // (node_count, dims, seed, degree, codec) but different build content must
    // have different content digests — and thus different epoch fingerprints —
    // so a stale remote node cannot pass FR-082 validation while serving a
    // different vec_id set / vectors / edges.
    for (table, second_vec) in [
        ("ec_distann_digest_a", "ARRAY[0.0, 1.0, 0.0, 0.0]"),
        ("ec_distann_digest_b", "ARRAY[0.0, 0.0, 1.0, 0.0]"),
    ] {
        Spi::run(&format!(
            "CREATE TABLE {table} (id bigint, embedding ecvector)"
        ))
        .expect("table creation should succeed");
        Spi::run(&format!(
            "INSERT INTO {table} VALUES \
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)), \
             (2, encode_to_ecvector({second_vec}, 4, 42)), \
             (3, encode_to_ecvector(ARRAY[0.0, 0.0, 0.0, 1.0], 4, 42))"
        ))
        .expect("rows should insert");
        Spi::run(&format!(
            "CREATE INDEX {table}_idx ON {table} \
             USING ec_distann (embedding ecvector_distann_ip_ops) WITH (graph_degree = 4)"
        ))
        .expect("index creation should succeed");
    }

    let (meta_a, _) = distann_materialized_index("ec_distann_digest_a_idx");
    let (meta_b, _) = distann_materialized_index("ec_distann_digest_b_idx");
    // Same shape metadata: without the content digest these would collide.
    assert_eq!(meta_a.node_count, meta_b.node_count);
    assert_eq!(meta_a.dimensions, meta_b.dimensions);
    assert_eq!(meta_a.seed, meta_b.seed);
    assert_eq!(meta_a.graph_degree_r, meta_b.graph_degree_r);
    assert_eq!(meta_a.neighbor_codec_kind, meta_b.neighbor_codec_kind);
    // Different content ⇒ different digest ⇒ different fingerprint.
    assert_ne!(
        meta_a.content_digest, meta_b.content_digest,
        "a differing build vector must change the content digest"
    );
    let fp_a = Spi::get_one::<Vec<u8>>(
        "SELECT ec_distann_epoch_fingerprint('ec_distann_digest_a_idx'::regclass::oid)",
    )
    .expect("SPI query should succeed")
    .expect("fingerprint exists");
    let fp_b = Spi::get_one::<Vec<u8>>(
        "SELECT ec_distann_epoch_fingerprint('ec_distann_digest_b_idx'::regclass::oid)",
    )
    .expect("SPI query should succeed")
    .expect("fingerprint exists");
    assert_ne!(
        fp_a, fp_b,
        "differing content must change the epoch fingerprint"
    );
}

#[pg_test]
fn test_ec_distann_expand_nodes_single_node_matches_local() {
    // FR-079-AC-1/AC-5: the endpoint returns one row per requested owned
    // vec_id, and each exact_dist is the full-precision -ip against the node's
    // co-placed heap vector. Single-node (empty roster): this node owns all.
    create_distann_fixture(
        "ec_distann_endpoint",
        "WITH (graph_degree = 4, build_list_size = 16)",
    );
    let vec_ids = distann_directory_vec_ids("ec_distann_endpoint_idx");
    let id_list = vec_ids
        .iter()
        .map(|v| (*v as i64).to_string())
        .collect::<Vec<_>>()
        .join(",");
    let call = format!(
        "ec_distann_expand_nodes(\
           'ec_distann_endpoint_idx'::regclass::oid, \
           ec_distann_epoch_fingerprint('ec_distann_endpoint_idx'::regclass::oid), \
           ARRAY[1,0,0,0]::real[], ARRAY[{id_list}]::bigint[])"
    );

    let row_count = Spi::get_one::<i64>(&format!("SELECT count(*) FROM {call}"))
        .expect("SPI query should succeed")
        .expect("count exists");
    assert_eq!(
        row_count,
        DISTANN_FIXTURE_ROWS.len() as i64,
        "one response row per requested vec_id (FR-079-AC-1)"
    );

    // The [1,0,0,0] fixture row is its own exact nearest: min exact_dist ~ -1.0.
    let min_dist = Spi::get_one::<f64>(&format!("SELECT min(exact_dist)::float8 FROM {call}"))
        .expect("SPI query should succeed")
        .expect("min exists");
    assert!(
        (min_dist + 1.0).abs() < 0.01,
        "nearest exact_dist should be ~ -1.0, got {min_dist}"
    );

    // No tombstones; neighbor arrays are aligned and non-empty for a built graph.
    let bad = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM {call} \
         WHERE is_tombstone OR exact_dist IS NULL \
           OR cardinality(neighbor_vec_ids) IS DISTINCT FROM cardinality(neighbor_code_dists)"
    ))
    .expect("SPI query should succeed")
    .expect("count exists");
    assert_eq!(bad, 0, "no tombstones, aligned neighbor arrays");
}

#[pg_test]
fn test_distann_remote_endpoint_acl_class() {
    let role = format!("ec_distann_unprivileged_{}", unsafe { pg_sys::MyProcPid });
    let mut client =
        postgres::Client::connect(&current_pg_test_loopback_conninfo(), postgres::NoTls)
            .expect("loopback connection should open");
    client
        .batch_execute(&format!(
            "DROP ROLE IF EXISTS {role}; CREATE ROLE {role} NOLOGIN"
        ))
        .expect("test role should create outside the pg_test transaction");

    let endpoints = client
        .query(
            "SELECT proc.oid::regprocedure::text AS identity,
                    namespace.nspname,
                    proc.prosecdef,
                    EXISTS (
                        SELECT 1 FROM unnest(proc.proconfig) setting
                         WHERE setting = format(
                             'search_path=pg_catalog, %s, pg_temp',
                             quote_ident(namespace.nspname)
                         )
                    ) AS safe_search_path,
                    (
                        SELECT count(*)
                          FROM unnest(proc.proargtypes)
                               AS argument(type_oid)
                          JOIN pg_type type ON type.oid = argument.type_oid
                         WHERE type.typelem = 0
                           AND argument.type_oid NOT IN (
                               'oid'::regtype,
                               'regclass'::regtype,
                               'uuid'::regtype,
                               'bytea'::regtype,
                               'text'::regtype,
                               'boolean'::regtype,
                               'smallint'::regtype,
                               'integer'::regtype,
                               'bigint'::regtype,
                               'real'::regtype,
                               'double precision'::regtype
                           )
                    ) AS unsupported_argument_count,
                    format(
                        'SELECT %I.%I(%s)',
                        namespace.nspname,
                        proc.proname,
                        COALESCE((
                            SELECT string_agg(
                                CASE
                                    WHEN type.typelem <> 0 THEN format(
                                        '''{}''::%s', format_type(argument.type_oid, NULL)
                                    )
                                    WHEN argument.type_oid = 'regclass'::regtype THEN
                                        '0::oid::regclass'
                                    WHEN argument.type_oid = 'uuid'::regtype THEN
                                        '''00000000-0000-4000-8000-000000000000''::uuid'
                                    WHEN argument.type_oid = 'bytea'::regtype THEN
                                        '''\\x''::bytea'
                                    WHEN argument.type_oid = 'text'::regtype THEN
                                        '''x''::text'
                                    WHEN argument.type_oid = 'boolean'::regtype THEN
                                        'false'
                                    ELSE format(
                                        '0::%s', format_type(argument.type_oid, NULL)
                                    )
                                END,
                                ', ' ORDER BY argument.ordinality
                            )
                              FROM unnest(proc.proargtypes)
                                   WITH ORDINALITY AS argument(type_oid, ordinality)
                              JOIN pg_type type ON type.oid = argument.type_oid
                        ), '')
                    ) AS call_sql
               FROM pg_proc proc
               JOIN pg_namespace namespace ON namespace.oid = proc.pronamespace
               JOIN pg_depend dependency
                 ON dependency.classid = 'pg_proc'::regclass
                AND dependency.objid = proc.oid
                AND dependency.deptype = 'e'
               JOIN pg_extension extension ON extension.oid = dependency.refobjid
              WHERE extension.extname = 'ecaz'
                AND proc.prokind = 'f'
                AND proc.proname LIKE 'ec_distann\\_%' ESCAPE '\\'
                AND proc.proname NOT IN (
                    'ec_distann_handler',
                    'ec_distann_owning_node',
                    'ec_distann_epoch_status'
                )
              ORDER BY identity",
            &[],
        )
        .expect("installed endpoint inventory should query");
    assert!(
        endpoints.len() >= 40,
        "class audit unexpectedly found only {} protected functions",
        endpoints.len()
    );
    let extension_schema = endpoints
        .first()
        .expect("protected endpoint class must not be empty")
        .get::<_, String>(1);
    client
        .batch_execute(&format!(
            "GRANT USAGE ON SCHEMA {extension_schema} TO {role}; SET ROLE {role}"
        ))
        .expect("unprivileged caller should receive schema usage and assume role");

    for endpoint in &endpoints {
        let identity = endpoint.get::<_, String>(0);
        assert!(
            endpoint.get::<_, bool>(2),
            "{identity} must be SECURITY DEFINER"
        );
        assert!(
            endpoint.get::<_, bool>(3),
            "{identity} must pin its search_path"
        );
        assert_eq!(
            endpoint.get::<_, i64>(4),
            0,
            "{identity} needs a non-NULL inert argument fixture"
        );
        let call_sql = endpoint.get::<_, String>(5);
        let error = client
            .simple_query(&call_sql)
            .expect_err("every protected endpoint call must be denied");
        let db_error = error
            .as_db_error()
            .unwrap_or_else(|| panic!("{identity} returned a non-database error: {error}"));
        assert_eq!(
            db_error.code(),
            &postgres::error::SqlState::INSUFFICIENT_PRIVILEGE,
            "{identity} returned the wrong SQLSTATE: {db_error}"
        );
        assert!(
            db_error
                .message()
                .contains("permission denied for function"),
            "{identity} was denied before the function ACL: {db_error}"
        );
    }

    client
        .batch_execute(&format!(
            "RESET ROLE;
             REVOKE USAGE ON SCHEMA {extension_schema} FROM {role};
             DROP ROLE {role}"
        ))
        .expect("test role should clean up");
}

#[pg_test]
fn test_distann_physical_dml_endpoint_acl_class() {
    let role = format!("ec_distann_physical_unprivileged_{}", unsafe {
        pg_sys::MyProcPid
    });
    let mut client =
        postgres::Client::connect(&current_pg_test_loopback_conninfo(), postgres::NoTls)
            .expect("loopback connection should open");
    client
        .batch_execute(&format!(
            "DROP ROLE IF EXISTS {role}; CREATE ROLE {role} NOLOGIN"
        ))
        .expect("test role should create outside the pg_test transaction");

    let extension_schema = client
        .query_one(
            "SELECT pg_catalog.quote_ident(namespace.nspname)
               FROM pg_catalog.pg_extension extension
               JOIN pg_catalog.pg_namespace namespace
                 ON namespace.oid = extension.extnamespace
              WHERE extension.extname = 'ecaz'",
            &[],
        )
        .expect("extension schema should resolve")
        .get::<_, String>(0);
    client
        .batch_execute(&format!(
            "GRANT USAGE ON SCHEMA {extension_schema} TO {role}; SET ROLE {role}"
        ))
        .expect("unprivileged caller should receive schema usage and assume role");

    let calls = [
        format!(
            "SELECT {extension_schema}.ec_distann_apply_physical_insert(
                 0::oid, '\\x'::bytea, 0::bigint, ARRAY[]::real[], '\\x'::bytea,
                 ARRAY[]::boolean[], ARRAY[]::bigint[], '\\x'::bytea, '\\x'::bytea, false
             )"
        ),
        format!(
            "SELECT {extension_schema}.ec_distann_apply_physical_backlink(
                 0::oid, '\\x'::bytea, 0::bigint, ARRAY[]::real[], 0::bigint,
                 ARRAY[]::real[], '\\x'::bytea
             )"
        ),
        format!(
            "SELECT {extension_schema}.ec_distann_apply_physical_tombstone(
                 0::oid, '\\x'::bytea, 0::bigint
             )"
        ),
    ];
    for call_sql in calls {
        let error = client
            .simple_query(&call_sql)
            .expect_err("physical DML endpoint must be denied to an unprivileged role");
        let db_error = error
            .as_db_error()
            .expect("physical DML ACL denial should be a database error");
        assert_eq!(
            db_error.code(),
            &postgres::error::SqlState::INSUFFICIENT_PRIVILEGE,
            "physical DML endpoint returned the wrong SQLSTATE: {db_error}"
        );
        assert!(
            db_error
                .message()
                .contains("permission denied for function"),
            "physical DML endpoint was not denied by its function ACL: {db_error}"
        );
    }

    client
        .batch_execute(&format!(
            "RESET ROLE;
             REVOKE USAGE ON SCHEMA {extension_schema} FROM {role};
             DROP ROLE {role}"
        ))
        .expect("test role should clean up");
}

#[pg_test]
fn test_ec_distann_fold_delta_requires_read_committed() {
    let mut client =
        postgres::Client::connect(&current_pg_test_loopback_conninfo(), postgres::NoTls)
            .expect("loopback connection should open");
    let extension_schema = client
        .query_one(
            "SELECT quote_ident(namespace.nspname)
               FROM pg_extension extension
               JOIN pg_namespace namespace ON namespace.oid = extension.extnamespace
              WHERE extension.extname = 'ecaz'",
            &[],
        )
        .expect("extension schema should resolve")
        .get::<_, String>(0);
    client
        .batch_execute(&format!(
            "SET search_path = {extension_schema}, public;
             BEGIN ISOLATION LEVEL REPEATABLE READ"
        ))
        .expect("repeatable-read probe should begin");
    let error = client
        .batch_execute("SELECT ec_distann_fold_delta_into_graph(0::oid)")
        .expect_err("fold maintenance must reject stronger isolation before relation access");
    assert!(
        error
            .as_db_error()
            .map(|db_error| db_error.message().contains("EC_TRANSACTION_ISOLATION"))
            .unwrap_or(false),
        "isolation failure must precede relation access: {error}"
    );
    client
        .batch_execute("ROLLBACK")
        .expect("repeatable-read probe should roll back");
}

#[pg_test]
fn test_ec_distann_apply_record_writes_requires_read_committed() {
    let mut client =
        postgres::Client::connect(&current_pg_test_loopback_conninfo(), postgres::NoTls)
            .expect("loopback connection should open");
    let extension_schema = client
        .query_one(
            "SELECT pg_catalog.quote_ident(namespace.nspname)
               FROM pg_catalog.pg_extension extension
               JOIN pg_catalog.pg_namespace namespace
                 ON namespace.oid = extension.extnamespace
              WHERE extension.extname = 'ecaz'",
            &[],
        )
        .expect("extension schema should resolve")
        .get::<_, String>(0);
    client
        .batch_execute(&format!(
            "SET search_path = {extension_schema}, public;
             BEGIN ISOLATION LEVEL REPEATABLE READ"
        ))
        .expect("repeatable-read probe should begin");
    let error = client
        .batch_execute(
            "SELECT ec_distann_apply_record_writes(
                 0::oid, '\\x'::bytea, ARRAY[]::bigint[]
             )",
        )
        .expect_err("mutating endpoint outside READ COMMITTED must fail before relation access");
    assert!(
        error
            .as_db_error()
            .map(|db_error| db_error.message().contains("EC_TRANSACTION_ISOLATION"))
            .unwrap_or(false),
        "isolation failure must be classified before relation access: {error}"
    );
    client
        .batch_execute("ROLLBACK")
        .expect("repeatable-read probe should roll back");
}

#[pg_test]
fn test_ec_distann_expand_nodes_rejects_epoch_mismatch() {
    // FR-079-AC-2: a stale/wrong epoch fingerprint yields the retriable
    // epoch-mismatch error, never data.
    create_distann_fixture("ec_distann_ep_mismatch", "WITH (graph_degree = 4)");
    let vec_ids = distann_directory_vec_ids("ec_distann_ep_mismatch_idx");
    let error = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT * FROM ec_distann_expand_nodes(\
               'ec_distann_ep_mismatch_idx'::regclass::oid, \
               '\\x000102030405060708090a0b0c0d0e0f'::bytea, \
               ARRAY[1,0,0,0]::real[], ARRAY[{}]::bigint[])",
            vec_ids[0] as i64
        ))
        .expect("this call must error before returning data");
    });
    assert!(
        error.contains("epoch fingerprint mismatch"),
        "unexpected error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_remote_transport_statement_timeout() {
    let conninfo = format!(
        "{} application_name=ecaz_distann_timeout_{}",
        current_pg_test_loopback_conninfo(),
        unsafe { pg_sys::MyProcPid }
    );
    Spi::run("SET ec_distann.remote_connect_timeout_ms = 1000")
        .expect("connect timeout should set");
    Spi::run("SET ec_distann.remote_statement_timeout_ms = 10000")
        .expect("initial statement timeout should set");
    crate::am::ec_distann::remote_timeout_probe_for_test(&conninfo, 0.0)
        .expect("initial probe should establish the pooled session");
    Spi::run("SET ec_distann.remote_statement_timeout_ms = 10")
        .expect("updated statement timeout should set");

    let started = std::time::Instant::now();
    let error = crate::am::ec_distann::remote_timeout_probe_for_test(&conninfo, 1.0)
        .expect_err("one-second remote sleep must exceed the 10ms budget");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(2),
        "remote timeout probe exceeded its bounded client deadline"
    );
    assert!(
        error.contains("statement timeout") || error.contains("timed out"),
        "unexpected timeout error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_remote_transport_cancel_then_reuse() {
    let conninfo = format!(
        "{} application_name=ecaz_distann_cancel_reuse_{}",
        current_pg_test_loopback_conninfo(),
        unsafe { pg_sys::MyProcPid }
    );
    Spi::run("SET ec_distann.remote_connect_timeout_ms = 1000")
        .expect("connect timeout should set");
    Spi::run("SET ec_distann.remote_statement_timeout_ms = 10000")
        .expect("statement timeout should set");
    crate::am::ec_distann::remote_timeout_probe_for_test(&conninfo, 0.0)
        .expect("initial probe should establish the pooled session");

    let backend_pid = unsafe { pg_sys::MyProcPid };
    let cancel_conninfo = current_pg_test_loopback_conninfo();
    let canceller = std::thread::spawn(move || {
        let mut client = postgres::Client::connect(&cancel_conninfo, postgres::NoTls)
            .expect("canceller should connect to the pg_test instance");
        std::thread::sleep(std::time::Duration::from_millis(100));
        let cancelled = client
            .query_one("SELECT pg_cancel_backend($1)", &[&backend_pid])
            .expect("pg_cancel_backend should execute")
            .get::<_, bool>(0);
        assert!(cancelled, "pg_test backend should accept cancellation");
    });

    let started = std::time::Instant::now();
    let error = expect_pg_error_rolled_back(|| {
        crate::am::ec_distann::remote_timeout_probe_for_test(&conninfo, 5.0)
            .expect("local cancellation must escape the remote await");
    });
    let elapsed = started.elapsed();
    canceller.join().expect("canceller thread should finish");
    assert!(
        error.contains("canceling statement due to user request"),
        "unexpected cancellation error: {error}"
    );
    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "local cancellation was not prompt: {elapsed:?}"
    );

    crate::am::ec_distann::remote_timeout_probe_for_test(&conninfo, 0.0)
        .expect("same backend must reuse transport state after cancellation");
}

#[pg_test]
fn test_ec_distann_remote_transport_cancel_mid_connect_then_reuse() {
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .expect("blackhole listener should bind");
    let port = listener
        .local_addr()
        .expect("blackhole listener address")
        .port();
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel::<()>(0);
    let acceptor = std::thread::spawn(move || {
        let (_socket, _) = listener.accept().expect("blackhole should accept");
        let _ = release_rx.recv_timeout(std::time::Duration::from_secs(2));
    });

    Spi::run("SET ec_distann.remote_connect_timeout_ms = 10000")
        .expect("connect timeout should set");
    Spi::run("SET ec_distann.remote_statement_timeout_ms = 10000")
        .expect("statement timeout should set");
    let backend_pid = unsafe { pg_sys::MyProcPid };
    let cancel_conninfo = current_pg_test_loopback_conninfo();
    let canceller = std::thread::spawn(move || {
        let mut client = postgres::Client::connect(&cancel_conninfo, postgres::NoTls)
            .expect("canceller should connect to the pg_test instance");
        std::thread::sleep(std::time::Duration::from_millis(100));
        let cancelled = client
            .query_one("SELECT pg_cancel_backend($1)", &[&backend_pid])
            .expect("pg_cancel_backend should execute")
            .get::<_, bool>(0);
        assert!(cancelled, "pg_test backend should accept cancellation");
    });
    let blackhole = format!("host=127.0.0.1 port={port} dbname=postgres");

    let started = std::time::Instant::now();
    let error = expect_pg_error_rolled_back(|| {
        crate::am::ec_distann::remote_timeout_probe_for_test(&blackhole, 0.0)
            .expect("local cancellation must escape connection establishment");
    });
    let elapsed = started.elapsed();
    canceller.join().expect("canceller thread should finish");
    release_tx.send(()).ok();
    acceptor.join().expect("blackhole acceptor should finish");
    assert!(
        error.contains("canceling statement due to user request"),
        "unexpected cancellation error: {error}"
    );
    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "mid-connect cancellation was not prompt: {elapsed:?}"
    );

    crate::am::ec_distann::remote_timeout_probe_for_test(&current_pg_test_loopback_conninfo(), 0.0)
        .expect("same backend must reconnect after mid-connect cancellation");
}

#[pg_test]
fn test_ec_distann_expand_nodes_rejects_nonowned_placement() {
    // FR-079-AC-3 case (b): a vec_id not owned by this node under the epoch
    // placement is a placement error, never a silent miss. Configure a 2-node
    // roster with this instance as node 0 and request a node-1-owned id.
    create_distann_fixture("ec_distann_place", "WITH (graph_degree = 4)");
    Spi::run("SET ec_distann.roster = '0@local;1@host=/nonexistent port=1 dbname=x'")
        .expect("roster set should succeed");
    Spi::run("SET ec_distann.local_node_id = 0").expect("local_node_id set should succeed");

    let vec_ids = distann_directory_vec_ids("ec_distann_place_idx");
    // Find one owned by roster index 1 (the non-local node).
    let node1_id = vec_ids.iter().copied().find(|&id| {
        crate::am::ec_distann::placement::owning_node(
            id,
            2,
            crate::am::ec_distann::placement::DISTANN_PLACEMENT_HASH_V1,
        ) == 1
    });
    let node1_id = node1_id.expect("with 8 hashed ids across 2 nodes, node 1 owns >=1") as i64;

    let error = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT * FROM ec_distann_expand_nodes(\
               'ec_distann_place_idx'::regclass::oid, \
               ec_distann_epoch_fingerprint('ec_distann_place_idx'::regclass::oid), \
               ARRAY[1,0,0,0]::real[], ARRAY[{node1_id}]::bigint[])"
        ))
        .expect("this call must error on the non-owned id");
    });
    assert!(
        error.contains("placement error"),
        "unexpected error: {error}"
    );

    Spi::run("RESET ec_distann.roster").expect("roster reset should succeed");
    Spi::run("RESET ec_distann.local_node_id").expect("local_node_id reset should succeed");
}

#[pg_test]
fn test_ec_distann_fold_delta_into_graph() {
    // FR-083 M5: an inserted row (delta buffer) is folded into the persisted
    // graph — node_count grows, the buffer drains, and the row is found via
    // graph traversal (empty delta buffer), not the exact-scan tail.
    create_distann_fixture("ec_distann_fold", "WITH (graph_degree = 4)");
    Spi::run(
        "INSERT INTO ec_distann_fold VALUES \
         (99, encode_to_ecvector(ARRAY[0.5, 0.5, 0.5, 0.5], 4, 42))",
    )
    .expect("delta insert should succeed");

    let (before, _) = distann_materialized_index("ec_distann_fold_idx");
    assert_ne!(
        before.delta_buffer_head,
        crate::storage::page::ItemPointer::INVALID,
        "row is in the delta buffer before folding"
    );

    let folded = Spi::get_one::<i64>(
        "SELECT ec_distann_fold_delta_into_graph('ec_distann_fold_idx'::regclass::oid)",
    )
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(folded, 1, "one delta entry folded into the graph");

    let (after, _) = distann_materialized_index("ec_distann_fold_idx");
    assert_eq!(
        after.node_count,
        before.node_count + 1,
        "folded node joined the graph"
    );
    assert_eq!(
        after.delta_buffer_head,
        crate::storage::page::ItemPointer::INVALID,
        "delta buffer drained after fold"
    );

    // The folded row is found via graph traversal (delta buffer is empty now).
    // The scan reads the rebuilt directory + node record by real on-disk TIDs
    // (read_*_from_relation), so this exercises the actual incremental layout.
    Spi::run("SET enable_seqscan = off").expect("disabling seqscan should succeed");
    let top = Spi::get_one::<i64>(
        "SELECT id FROM ec_distann_fold \
         ORDER BY embedding <#> ARRAY[0.5, 0.5, 0.5, 0.5]::real[] LIMIT 1",
    )
    .expect("SPI query should succeed")
    .expect("row should exist");
    assert_eq!(top, 99, "folded row is found via the graph");
    Spi::run("RESET enable_seqscan").expect("seqscan reset should succeed");
}

#[pg_test]
fn test_ec_distann_fold_multi_row_clustered_delta() {
    // 167-006-P1 regression: fold SEVERAL delta rows in one call. The candidate
    // search (`collect_distann_hits`) merges the still-live delta chain into its
    // hits, so while folding row A the search sees not-yet-folded rows B/C. Those
    // rows have no directory entry; before the fix, robust-prune could pick one as
    // a forward neighbor and the fold would error at the mandatory directory lookup
    // AFTER earlier rows already mutated the graph. The rows are clustered near a
    // common point so they are genuinely each other's nearest neighbors — the exact
    // shape that surfaces the bug. Every row must fold and be found via traversal.
    create_distann_fixture("ec_distann_multifold", "WITH (graph_degree = 4)");
    for (row_id, coord) in [(90, 0.51_f32), (91, 0.52), (92, 0.53)] {
        Spi::run(&format!(
            "INSERT INTO ec_distann_multifold VALUES \
             ({row_id}, encode_to_ecvector(ARRAY[{coord}, {coord}, {coord}, {coord}], 4, 42))"
        ))
        .expect("clustered delta insert should succeed");
    }

    let (before, _) = distann_materialized_index("ec_distann_multifold_idx");
    assert_ne!(
        before.delta_buffer_head,
        crate::storage::page::ItemPointer::INVALID,
        "clustered rows are in the delta buffer before folding"
    );

    let folded = Spi::get_one::<i64>(
        "SELECT ec_distann_fold_delta_into_graph('ec_distann_multifold_idx'::regclass::oid)",
    )
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(
        folded, 3,
        "all three clustered delta entries fold in one call"
    );

    let (after, _) = distann_materialized_index("ec_distann_multifold_idx");
    assert_eq!(
        after.node_count,
        before.node_count + 3,
        "all three folded nodes joined the graph"
    );
    assert_eq!(
        after.delta_buffer_head,
        crate::storage::page::ItemPointer::INVALID,
        "delta buffer drained after multi-row fold"
    );

    // All three folded rows are reachable via graph traversal (delta buffer is
    // empty now). The three cluster coords (0.51/0.52/0.53) are near-identical, so
    // the quantized `<#>` order cannot reliably distinguish top-1 among them; the
    // regression assertion is that the whole folded cluster is graph-reachable, so
    // query the cluster centre and require all three to appear in the top-3 (they
    // were connected into the graph by the multi-row fold, not lost).
    Spi::run("SET enable_seqscan = off").expect("disabling seqscan should succeed");
    let mut found: Vec<i64> = Vec::new();
    Spi::connect(|client| {
        let rows = client
            .select(
                "SELECT id FROM ec_distann_multifold \
                 ORDER BY embedding <#> ARRAY[0.52, 0.52, 0.52, 0.52]::real[] LIMIT 3",
                None,
                &[],
            )
            .expect("cluster query should succeed");
        for row in rows {
            found.push(row.get::<i64>(1).expect("id column").expect("id present"));
        }
    });
    for row_id in [90_i64, 91, 92] {
        assert!(
            found.contains(&row_id),
            "folded clustered row {row_id} must be graph-reachable in the top-3 (got {found:?})"
        );
    }
    Spi::run("RESET enable_seqscan").expect("seqscan reset should succeed");
}

#[pg_test]
fn test_ec_distann_reindex_drains_delta_buffer() {
    // FR-083-AC-2: the epoch build (REINDEX) drains the interim delta buffer —
    // the inserted row joins the graph and delta_buffer_head resets to INVALID
    // (a fresh build indexes all live heap rows, including delta-inserted ones).
    create_distann_fixture("ec_distann_drain", "WITH (graph_degree = 4)");
    Spi::run(
        "INSERT INTO ec_distann_drain VALUES \
         (99, encode_to_ecvector(ARRAY[0.5, 0.5, 0.5, 0.5], 4, 42))",
    )
    .expect("delta insert should succeed");

    let (before, _) = distann_materialized_index("ec_distann_drain_idx");
    assert_ne!(
        before.delta_buffer_head,
        crate::storage::page::ItemPointer::INVALID,
        "the inserted row is in the delta buffer before the epoch build"
    );
    assert_eq!(
        before.node_count,
        DISTANN_FIXTURE_ROWS.len() as u64,
        "the graph still holds only the built rows before REINDEX"
    );

    Spi::run("REINDEX INDEX ec_distann_drain_idx").expect("reindex should succeed");

    let (after, _) = distann_materialized_index("ec_distann_drain_idx");
    assert_eq!(
        after.delta_buffer_head,
        crate::storage::page::ItemPointer::INVALID,
        "the delta buffer is drained by the epoch build"
    );
    assert_eq!(
        after.node_count,
        DISTANN_FIXTURE_ROWS.len() as u64 + 1,
        "the delta-inserted row is now part of the rebuilt graph"
    );
}

#[pg_test]
fn test_ec_distann_fault_drills_distinct_classes() {
    // TC-042 / NFR-020: each fault is an ERROR carrying a distinct
    // machine-readable class ([EC_*]) so the coordinator can decide retry vs
    // fail-fast — never a wrong or silent result. Single-node endpoint drills.
    create_distann_fixture("ec_distann_drill", "WITH (graph_degree = 4)");
    let vec_ids = distann_directory_vec_ids("ec_distann_drill_idx");
    let good_fp = "ec_distann_epoch_fingerprint('ec_distann_drill_idx'::regclass::oid)";

    // Drill: epoch_mismatch (retriable class).
    let epoch = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT * FROM ec_distann_expand_nodes('ec_distann_drill_idx'::regclass::oid, \
             '\\x000102030405060708090a0b0c0d0e0f'::bytea, ARRAY[1,0,0,0]::real[], ARRAY[{}]::bigint[])",
            vec_ids[0] as i64
        ))
        .expect("must error");
    });
    assert!(
        epoch.contains("[EC_EPOCH_MISMATCH]"),
        "epoch drill class: {epoch}"
    );

    // Drill: missing_node_record (owned-but-absent structural fault) — a vec_id
    // that hashes to this node (single-node owns all) but is not in the
    // directory.
    let absent = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT * FROM ec_distann_expand_nodes('ec_distann_drill_idx'::regclass::oid, \
             {good_fp}, ARRAY[1,0,0,0]::real[], ARRAY[{}]::bigint[])",
            1_i64
        ))
        .expect("must error on a vec_id absent from the directory");
    });
    assert!(
        absent.contains("[EC_RECORD_MISSING]"),
        "absent-record drill class: {absent}"
    );

    // Drill: bad input (malformed fingerprint width).
    let bad = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT * FROM ec_distann_expand_nodes('ec_distann_drill_idx'::regclass::oid, \
             '\\x0011'::bytea, ARRAY[1,0,0,0]::real[], ARRAY[{}]::bigint[])",
            vec_ids[0] as i64
        ))
        .expect("must error on a malformed fingerprint");
    });
    assert!(
        bad.contains("[EC_BAD_INPUT]"),
        "bad-input drill class: {bad}"
    );

    // Drill: placement_drift (non-owned id under a 2-node roster).
    Spi::run("SET ec_distann.roster = '0@local;1@host=/x port=1 dbname=y'").expect("roster set");
    Spi::run("SET ec_distann.local_node_id = 0").expect("id set");
    let node1 = vec_ids.iter().copied().find(|&id| {
        crate::am::ec_distann::placement::owning_node(
            id,
            2,
            crate::am::ec_distann::placement::DISTANN_PLACEMENT_HASH_V1,
        ) == 1
    });
    if let Some(node1_id) = node1 {
        let placement = expect_pg_error(|| {
            Spi::run(&format!(
                "SELECT * FROM ec_distann_expand_nodes('ec_distann_drill_idx'::regclass::oid, \
                 {good_fp}, ARRAY[1,0,0,0]::real[], ARRAY[{}]::bigint[])",
                node1_id as i64
            ))
            .expect("must error on a non-owned id");
        });
        assert!(
            placement.contains("[EC_PLACEMENT]"),
            "placement drill class: {placement}"
        );
    }
    Spi::run("RESET ec_distann.roster").expect("roster reset");
    Spi::run("RESET ec_distann.local_node_id").expect("id reset");
}

#[pg_test]
fn test_ec_distann_materialize_rows_ships_heap_identity() {
    // 005-P1: the owning node ships the heap identity (ctid + tombstone) for the
    // vec_ids it owns, under epoch + ownership validation — so a coordinator
    // materializes remote hits from the OWNER rather than a local directory.
    // Single-node: this node owns all.
    create_distann_fixture("ec_distann_mat", "WITH (graph_degree = 4)");
    let vec_ids = distann_directory_vec_ids("ec_distann_mat_idx");
    let fp = "ec_distann_epoch_fingerprint('ec_distann_mat_idx'::regclass::oid)";

    let n = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM ec_distann_materialize_rows(\
           'ec_distann_mat_idx'::regclass::oid, {fp}, ARRAY[{}, {}]::bigint[])",
        vec_ids[0] as i64, vec_ids[1] as i64
    ))
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(n, 2, "two owned rows materialized");

    // Every shipped row carries a valid heap ctid and is not tombstoned.
    let bad = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM ec_distann_materialize_rows(\
           'ec_distann_mat_idx'::regclass::oid, {fp}, ARRAY[{}, {}]::bigint[]) \
         WHERE heap_block < 0 OR heap_offset <= 0 OR is_tombstone",
        vec_ids[0] as i64, vec_ids[1] as i64
    ))
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(bad, 0, "shipped ctids are valid and rows are live");

    // Wrong epoch fingerprint → retriable mismatch error (fail closed).
    let error = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT * FROM ec_distann_materialize_rows(\
               'ec_distann_mat_idx'::regclass::oid, \
               '\\x000102030405060708090a0b0c0d0e0f'::bytea, ARRAY[{}]::bigint[])",
            vec_ids[0] as i64
        ))
        .expect("must error on the wrong epoch");
    });
    assert!(
        error.contains("epoch fingerprint mismatch"),
        "unexpected error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_materialize_row_payloads_ships_binary_columns() {
    // 005-P1 / CustomScan data path: the owning node ships the requested heap
    // column data itself (not just a ctid) as PostgreSQL binary (`typsend`), so a
    // coordinator can reconstruct a real SQL row for a remote-owned hit without a
    // local directory or a local heap fetch. Single-node: this node owns all.
    create_distann_fixture("ec_distann_payload", "WITH (graph_degree = 4)");
    let vec_ids = distann_directory_vec_ids("ec_distann_payload_idx");
    let fp = "ec_distann_epoch_fingerprint('ec_distann_payload_idx'::regclass::oid)";
    let call = format!(
        "ec_distann_materialize_row_payloads(\
           'ec_distann_payload_idx'::regclass::oid, {fp}, \
           ARRAY[{}, {}]::bigint[], ARRAY['id']::text[], ARRAY['int8send']::text[])",
        vec_ids[0] as i64, vec_ids[1] as i64
    );

    // Every requested owned vec_id yields exactly one live, present row carrying a
    // single 8-byte (int8send) non-null column value.
    let n = Spi::get_one::<i64>(&format!("SELECT count(*) FROM {call}"))
        .expect("SPI query should succeed")
        .expect("count should exist");
    assert_eq!(n, 2, "two owned rows materialized with payloads");

    let bad = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM {call} \
         WHERE is_tombstone OR tuple_payload_missing \
            OR array_length(payload_values, 1) <> 1 OR payload_nulls[1] \
            OR octet_length(payload_values[1]) <> 8"
    ))
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(
        bad, 0,
        "shipped columns are live, present, and 8-byte int8send"
    );

    // The shipped binary decodes back to the row's id: int8send is big-endian, and
    // the fixture ids are 1..=8 so only the low byte is set. This proves the owner
    // ships the actual column value, byte-exact, not a placeholder.
    let all_decode = Spi::get_one::<bool>(&format!(
        "SELECT bool_and(\
             get_byte(payload_values[1], 0) = 0 AND get_byte(payload_values[1], 1) = 0 AND \
             get_byte(payload_values[1], 2) = 0 AND get_byte(payload_values[1], 3) = 0 AND \
             get_byte(payload_values[1], 4) = 0 AND get_byte(payload_values[1], 5) = 0 AND \
             get_byte(payload_values[1], 6) = 0 AND \
             get_byte(payload_values[1], 7) BETWEEN 1 AND 8) \
           FROM {call}"
    ))
    .expect("SPI query should succeed")
    .expect("bool should exist");
    assert!(all_decode, "shipped int8send payload decodes to the row id");

    // Wrong epoch fingerprint → retriable mismatch error (fail closed).
    let error = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT * FROM ec_distann_materialize_row_payloads(\
               'ec_distann_payload_idx'::regclass::oid, \
               '\\x000102030405060708090a0b0c0d0e0f'::bytea, ARRAY[{}]::bigint[], \
               ARRAY['id']::text[], ARRAY['int8send']::text[])",
            vec_ids[0] as i64
        ))
        .expect("must error on the wrong epoch");
    });
    assert!(
        error.contains("epoch fingerprint mismatch"),
        "unexpected error: {error}"
    );

    // An injection-shaped send function name is rejected before any SQL runs.
    let error = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT * FROM ec_distann_materialize_row_payloads(\
               'ec_distann_payload_idx'::regclass::oid, {fp}, ARRAY[{}]::bigint[], \
               ARRAY['id']::text[], ARRAY['int8send(id); DROP TABLE ec_distann_payload; --']::text[])",
            vec_ids[0] as i64
        ))
        .expect("must reject a non-identifier send function");
    });
    assert!(
        error.contains("invalid send function"),
        "unexpected error: {error}"
    );

    // Column/send-function count mismatch is rejected.
    let error = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT * FROM ec_distann_materialize_row_payloads(\
               'ec_distann_payload_idx'::regclass::oid, {fp}, ARRAY[{}]::bigint[], \
               ARRAY['id']::text[], ARRAY[]::text[])",
            vec_ids[0] as i64
        ))
        .expect("must reject a column/send-function count mismatch");
    });
    assert!(
        error.contains("payload columns but") && error.contains("send functions"),
        "unexpected error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_tombstone_excludes_and_preserves_live_vectors() {
    // FR-082-AC-4 (tombstones honored at expansion) + AC-5 (a live record's
    // exact-rerank vector is unaffected by another record's tombstone — the D10
    // "nothing physically reclaimed within a Published epoch" model means the
    // frozen vec_id→vector correspondence holds without a base-table race, so
    // long as deletion is a tombstone-flag set (FR-083), NOT a raw base DELETE
    // (which strips the co-placed vector — the EC_VECTOR_MISSING hazard).
    Spi::run("SET enable_seqscan = off").expect("seqscan off");
    create_distann_fixture("ec_distann_ac45", "WITH (graph_degree = 4)");
    let vec_ids = distann_directory_vec_ids("ec_distann_ac45_idx");

    // Self-query each fixture row: its own vector's nearest neighbor is itself
    // (id = row_index + 1), at exact_dist ≈ -1 (unit vectors, ip self = 1).
    let self_top1 = |row: &[f32; 4]| -> (i64, f64) {
        Spi::get_two::<i64, f64>(&format!(
            "SELECT id, (embedding <#> ARRAY[{},{},{},{}]::real[])::float8 \
               FROM ec_distann_ac45 \
              ORDER BY embedding <#> ARRAY[{},{},{},{}]::real[] LIMIT 1",
            row[0], row[1], row[2], row[3], row[0], row[1], row[2], row[3]
        ))
        .map(|(id, d)| (id.expect("id"), d.expect("dist")))
        .expect("SPI")
    };

    let baseline: Vec<(i64, f64)> = DISTANN_FIXTURE_ROWS.iter().map(self_top1).collect();
    for (i, (id, _)) in baseline.iter().enumerate() {
        assert_eq!(*id, i as i64 + 1, "baseline: row {i} self-returns itself");
    }

    // Tombstone three records via the FR-083 write endpoint.
    let removed = Spi::get_one::<i64>(&format!(
        "SELECT ec_distann_apply_record_writes(\
           'ec_distann_ac45_idx'::regclass::oid, \
           ec_distann_epoch_fingerprint('ec_distann_ac45_idx'::regclass::oid), \
           ARRAY[{}, {}, {}]::bigint[])",
        vec_ids[0] as i64, vec_ids[1] as i64, vec_ids[2] as i64
    ))
    .expect("SPI")
    .expect("count");
    assert_eq!(removed, 3, "three records tombstoned");

    // After: exactly 3 rows no longer self-return (their record is tombstoned →
    // excluded at expansion, AC-4); the other 5 still self-return AND at the
    // byte-identical exact distance (their vector is intact, AC-5).
    let mut still_self = 0;
    for (i, row) in DISTANN_FIXTURE_ROWS.iter().enumerate() {
        let (id, dist) = self_top1(row);
        if id == i as i64 + 1 {
            still_self += 1;
            assert_eq!(
                dist, baseline[i].1,
                "AC-5: surviving row {i} rerank distance is unchanged by others' tombstones"
            );
        }
    }
    assert_eq!(
        still_self, 5,
        "AC-4: exactly the 3 tombstoned records are excluded; 5 live records remain"
    );
}

#[pg_test]
fn test_ec_distann_owning_node_surface() {
    // FR-078 ownership surface: deterministic, in-range, and load-distributing,
    // so operator tooling / the multinode fixture can bucket vec_ids by owner.
    // hash_version 1 == DISTANN_PLACEMENT_HASH_V1.
    let out_of_range = Spi::get_one::<i64>(
        "SELECT count(*) FROM generate_series(1, 500) g \
         WHERE ec_distann_owning_node(g, 3, 1) < 0 OR ec_distann_owning_node(g, 3, 1) >= 3",
    )
    .expect("SPI")
    .expect("count");
    assert_eq!(out_of_range, 0, "every owner is in [0, node_count)");

    // Deterministic: same inputs -> same owner.
    let stable = Spi::get_one::<bool>(
        "SELECT ec_distann_owning_node(12345, 3, 1) = ec_distann_owning_node(12345, 3, 1)",
    )
    .expect("SPI")
    .expect("bool");
    assert!(stable, "ownership is deterministic");

    // Load-distributing: 500 ids across 3 nodes touch every node.
    let distinct_owners = Spi::get_one::<i64>(
        "SELECT count(DISTINCT ec_distann_owning_node(g, 3, 1)) FROM generate_series(1, 500) g",
    )
    .expect("SPI")
    .expect("count");
    assert_eq!(distinct_owners, 3, "hash placement uses all 3 nodes");

    // Single-node roster: everything owned by node 0.
    let all_local = Spi::get_one::<i64>(
        "SELECT count(*) FROM generate_series(1, 100) g WHERE ec_distann_owning_node(g, 1, 1) <> 0",
    )
    .expect("SPI")
    .expect("count");
    assert_eq!(all_local, 0, "single-node owns every vec_id");
}

#[pg_test]
fn test_ec_distann_epoch_lifecycle_publish_retire_override() {
    // FR-082 AC-1/AC-3/AC-6: the epoch lifecycle state machine persists in the v4
    // metadata page. A built index is Published; republish swaps the active
    // epoch; retire is gated on the in-flight count; the operator override
    // force-retires a wedged count.
    create_distann_fixture("ec_distann_epoch", "WITH (graph_degree = 4)");
    let idx = "'ec_distann_epoch_idx'::regclass::oid";

    // A freshly built index is Published at epoch 1, nothing in flight.
    let state = Spi::get_one::<String>(&format!(
        "SELECT epoch_state FROM ec_distann_epoch_status({idx})"
    ))
    .expect("SPI")
    .expect("state");
    assert_eq!(state, "published", "built index is published");

    // AC-1: republish swaps the active epoch atomically.
    Spi::run(&format!("SELECT ec_distann_publish_epoch({idx}, 5)")).expect("publish");
    let epoch = Spi::get_one::<i64>(&format!(
        "SELECT active_epoch FROM ec_distann_epoch_status({idx})"
    ))
    .expect("SPI")
    .expect("epoch");
    assert_eq!(epoch, 5, "republish set the active epoch");

    // AC-3: a non-zero in-flight count blocks retire (retention gate).
    Spi::run(&format!("SELECT ec_distann_debug_set_in_flight({idx}, 3)")).expect("set in-flight");
    let error = expect_pg_error(|| {
        Spi::run(&format!("SELECT ec_distann_retire_epoch({idx})")).expect("must gate");
    });
    assert!(
        error.contains("retention gate"),
        "unexpected error: {error}"
    );
    // Still published, count intact.
    let (state, in_flight) = Spi::get_two::<String, i64>(&format!(
        "SELECT epoch_state, in_flight_count FROM ec_distann_epoch_status({idx})"
    ))
    .expect("SPI");
    assert_eq!(state.as_deref(), Some("published"), "retire was blocked");
    assert_eq!(in_flight, Some(3), "in-flight count retained");

    // AC-6: the operator override force-retires the wedged count and clears it.
    Spi::run(&format!("SELECT ec_distann_force_retire_epoch({idx})")).expect("force retire");
    let (state, in_flight) = Spi::get_two::<String, i64>(&format!(
        "SELECT epoch_state, in_flight_count FROM ec_distann_epoch_status({idx})"
    ))
    .expect("SPI");
    assert_eq!(
        state.as_deref(),
        Some("retired"),
        "override retired the epoch"
    );
    assert_eq!(in_flight, Some(0), "override cleared the wedged count");

    // Gate opens once in-flight is zero: a clean retire is idempotent.
    Spi::run(&format!("SELECT ec_distann_retire_epoch({idx})")).expect("retire when drained");
    let state = Spi::get_one::<String>(&format!(
        "SELECT epoch_state FROM ec_distann_epoch_status({idx})"
    ))
    .expect("SPI")
    .expect("state");
    assert_eq!(state, "retired");
}

#[pg_test]
fn test_ec_distann_apply_record_writes_tombstones() {
    // FR-083 write endpoint (M3): the tombstone-set operation applies on the
    // hash-owning node under epoch validation. Single-node: this node owns all.
    create_distann_fixture("ec_distann_write", "WITH (graph_degree = 4)");
    let vec_ids = distann_directory_vec_ids("ec_distann_write_idx");

    let removed = Spi::get_one::<i64>(&format!(
        "SELECT ec_distann_apply_record_writes(\
           'ec_distann_write_idx'::regclass::oid, \
           ec_distann_epoch_fingerprint('ec_distann_write_idx'::regclass::oid), \
           ARRAY[{}, {}]::bigint[])",
        vec_ids[0] as i64, vec_ids[1] as i64
    ))
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(removed, 2, "two records tombstoned via the write endpoint");

    // FR-082-AC-2: a wrong epoch fingerprint is a retriable mismatch error.
    let error = expect_pg_error(|| {
        Spi::run(&format!(
            "SELECT ec_distann_apply_record_writes(\
               'ec_distann_write_idx'::regclass::oid, \
               '\\x000102030405060708090a0b0c0d0e0f'::bytea, ARRAY[{}]::bigint[])",
            vec_ids[2] as i64
        ))
        .expect("this call must error on the wrong epoch");
    });
    assert!(
        error.contains("epoch fingerprint mismatch"),
        "unexpected error: {error}"
    );
}

#[pg_test]
fn test_ec_distann_delete_tombstones_record() {
    // FR-083-AC-1 (M3 D10 tombstone slice): tombstoning records sets the FR-076
    // flag monotonically in place, and the FR-081 scan excludes them while
    // remaining rows are unaffected. Uses the write-endpoint tombstone-by-vec_id
    // primitive (VACUUM/ambulkdelete can't run inside pg_test's txn; the
    // ambulkdelete callback path is integration-tested against a committed DB).
    // Asserts the flag directly — proves the tombstone mechanism, not MVCC.
    create_distann_fixture("ec_distann_del", "WITH (graph_degree = 4)");

    // Tombstone the first two records by vec_id.
    let vec_ids = distann_directory_vec_ids("ec_distann_del_idx");
    let victims = [vec_ids[0] as i64, vec_ids[1] as i64];
    let removed = Spi::get_one::<i64>(&format!(
        "SELECT ec_distann_debug_tombstone('ec_distann_del_idx'::regclass::oid, ARRAY[{}, {}]::bigint[])",
        victims[0], victims[1]
    ))
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(removed, 2, "two records should be newly tombstoned");
    // Idempotent: re-tombstoning the same vec_ids is a no-op.
    let removed_again = Spi::get_one::<i64>(&format!(
        "SELECT ec_distann_debug_tombstone('ec_distann_del_idx'::regclass::oid, ARRAY[{}, {}]::bigint[])",
        victims[0], victims[1]
    ))
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(removed_again, 0, "re-tombstoning is a monotone no-op");

    // The flags are set on-disk.
    let (metadata, chain) = distann_materialized_index("ec_distann_del_idx");
    let code_len = crate::am::ec_distann::quantizer::metadata_code_len(&metadata)
        .expect("code length should resolve");
    let directory = crate::am::ec_distann::reader::read_directory_chain(
        &chain,
        metadata.directory_head,
        metadata.node_count as usize,
    )
    .expect("directory should read");
    let tombstoned = directory
        .iter()
        .filter(|(_, tid)| {
            crate::am::ec_distann::reader::read_node(
                &chain,
                *tid,
                metadata.graph_degree_r,
                code_len,
            )
            .map(|node| node.tombstoned)
            .unwrap_or(false)
        })
        .count();
    assert_eq!(tombstoned, 2, "exactly two records tombstoned on disk");

    // The scan excludes tombstoned rows (FR-081 is_tombstone). The victims'
    // ids are whichever rows hash to vec_ids[0]/[1]; assert the tombstoned
    // COUNT shrinks the returned set from 8 to 6.
    Spi::run("SET enable_seqscan = off").expect("disabling seqscan should succeed");
    let returned = Spi::get_one::<i64>(
        "SELECT count(*) FROM (SELECT id FROM ec_distann_del \
         ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::real[] LIMIT 8) q",
    )
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(
        returned, 6,
        "two tombstoned rows are excluded from the scan (8 - 2)"
    );
    Spi::run("RESET enable_seqscan").expect("seqscan reset should succeed");
}

#[pg_test]
fn test_ec_distann_delta_insert_visible_same_statement() {
    // FR-083-AC-3 (M3 D5 delta buffer): aminsert spools the new row into the
    // bounded exact-scan delta buffer; it is visible in the same statement,
    // exact-scanned and merged into results at its true rank.
    create_distann_fixture("ec_distann_ins", "WITH (graph_degree = 4)");
    // Insert a new row whose vector no existing fixture row is closest to.
    Spi::run(
        "INSERT INTO ec_distann_ins VALUES \
         (99, encode_to_ecvector(ARRAY[0.5, 0.5, 0.5, 0.5], 4, 42))",
    )
    .expect("insert should succeed");

    // The delta buffer now has one entry (head set).
    let (metadata, _) = distann_materialized_index("ec_distann_ins_idx");
    assert_ne!(
        metadata.delta_buffer_head,
        crate::storage::page::ItemPointer::INVALID,
        "delta buffer head should be set after an insert"
    );

    // Same-statement visibility: querying the inserted vector returns it first.
    Spi::run("SET enable_seqscan = off").expect("disabling seqscan should succeed");
    let top = Spi::get_one::<i64>(
        "SELECT id FROM ec_distann_ins \
         ORDER BY embedding <#> ARRAY[0.5, 0.5, 0.5, 0.5]::real[] LIMIT 1",
    )
    .expect("SPI query should succeed")
    .expect("row should exist");
    assert_eq!(
        top, 99,
        "inserted row is visible same-statement and ranked nearest"
    );

    // It also appears within a larger LIMIT alongside the graph rows.
    let inserted_in_topk = Spi::get_one::<i64>(
        "SELECT count(*) FROM (SELECT id FROM ec_distann_ins \
         ORDER BY embedding <#> ARRAY[0.5, 0.5, 0.5, 0.5]::real[] LIMIT 5) q WHERE id = 99",
    )
    .expect("SPI query should succeed")
    .expect("count should exist");
    assert_eq!(
        inserted_in_topk, 1,
        "inserted row merged into the ranked result"
    );
    Spi::run("RESET enable_seqscan").expect("seqscan reset should succeed");
}

#[pg_test]
fn test_ec_distann_insert_into_empty_index_is_rejected() {
    // The FR-081 scan early-returns for an empty graph, so a delta-only buffer
    // would be invisible; delta insert into an empty index is rejected (not
    // silently dropped) until the delta-only scan path lands in a later slice.
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
        error.contains("empty index is not supported yet"),
        "unexpected error: {error}"
    );
}
