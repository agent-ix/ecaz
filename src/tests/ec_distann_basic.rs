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
             embedding ecvector NOT NULL
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
    let scan_error = expect_pg_error(|| {
        Spi::get_one::<i64>(
            "SELECT 1 FROM ec_distann_control
             ORDER BY embedding <#> ARRAY[1.0,0.0,0.0,0.0]::real[] LIMIT 1",
        )
        .expect("direct control scan must fail before returning");
    });
    assert!(
        scan_error.contains("EC_DISTANN_CONTROL_SCAN"),
        "unexpected direct-scan error: {scan_error}"
    );
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
             embedding ecvector NOT NULL
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
               embedding ecvector NOT NULL
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

struct DistannPhysicalGenerationFixture {
    index_name: String,
    canonical_index_regclass: String,
    index_oid: pg_sys::Oid,
    logical_index_uuid: pgrx::datum::Uuid,
    build_id: pgrx::datum::Uuid,
    descriptor: Vec<u8>,
    descriptor_digest: Vec<u8>,
    roster_digest: Vec<u8>,
    build_spec_digest: Vec<u8>,
    expected_owner_digest: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
struct DistannPhysicalMutationState {
    generation_json: String,
    batch_json: String,
    row_count: i64,
    graph_count: i64,
    row_bytes: i64,
    graph_bytes: i64,
    directory_bytes: i64,
}

fn distann_physical_mutation_state(
    fixture: &DistannPhysicalGenerationFixture,
) -> DistannPhysicalMutationState {
    let (row_oid, graph_oid, directory_oid) = distann_generation_relation_oids(fixture);
    let row_name = canonical_index_locator(row_oid);
    let graph_name = canonical_index_locator(graph_oid);
    Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT
                         (SELECT row_to_json(g)::text FROM {} g
                           WHERE g.index_oid = $1::oid AND g.build_id = $2::uuid)
                             AS generation_json,
                         COALESCE((SELECT jsonb_agg(to_jsonb(b) ORDER BY b.batch_seq)::text
                                     FROM {} b
                                    WHERE b.index_oid = $1::oid AND b.build_id = $2::uuid), '[]')
                             AS batch_json,
                         (SELECT count(*) FROM {row_name})::bigint AS row_count,
                         (SELECT count(*) FROM {graph_name})::bigint AS graph_count,
                         COALESCE(pg_relation_size($3::oid), -1)::bigint AS row_bytes,
                         COALESCE(pg_relation_size($4::oid), -1)::bigint AS graph_bytes,
                         COALESCE(pg_relation_size($5::oid), -1)::bigint AS directory_bytes",
                    distann_generation_catalog_name(),
                    distann_catalog_name("ec_distann_generation_batch"),
                ),
                None,
                &[
                    fixture.index_oid.into(),
                    fixture.build_id.into(),
                    row_oid.into(),
                    graph_oid.into(),
                    directory_oid.into(),
                ],
            )
            .unwrap()
            .map(|row| DistannPhysicalMutationState {
                generation_json: row["generation_json"].value::<String>().unwrap().unwrap(),
                batch_json: row["batch_json"].value::<String>().unwrap().unwrap(),
                row_count: row["row_count"].value::<i64>().unwrap().unwrap(),
                graph_count: row["graph_count"].value::<i64>().unwrap().unwrap(),
                row_bytes: row["row_bytes"].value::<i64>().unwrap().unwrap(),
                graph_bytes: row["graph_bytes"].value::<i64>().unwrap().unwrap(),
                directory_bytes: row["directory_bytes"].value::<i64>().unwrap().unwrap(),
            })
            .next()
            .unwrap()
    })
}

fn distann_generation_catalog_name() -> String {
    distann_catalog_name("ec_distann_generation")
}

fn rehash_distann_handoff_batch(mut encoded: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    use sha2::Digest;

    assert!(encoded.len() >= 32);
    encoded.truncate(encoded.len() - 32);
    let mut hasher = sha2::Sha256::new();
    hasher.update(b"ec_distann_handoff_batch_v1\0");
    hasher.update(&encoded);
    let digest = hasher.finalize().to_vec();
    encoded.extend_from_slice(&digest);
    (digest, encoded)
}

fn add_second_distann_owner(fixture: &mut DistannPhysicalGenerationFixture) {
    let mut descriptor =
        crate::am::ec_distann::DistannGenerationDescriptor::decode(&fixture.descriptor).unwrap();
    let mut second_uuid = [0x7a; 16];
    second_uuid[6] = 0x4a;
    second_uuid[8] = 0xba;
    descriptor
        .roster
        .push(crate::am::ec_distann::DistannRosterEntry {
            node_id: 18,
            logical_index_uuid: second_uuid,
            endpoint_identity: "negative-matrix/node-18".to_owned(),
        });
    fixture.descriptor = descriptor.encode().unwrap();
    fixture.descriptor_digest = descriptor.digest().unwrap().to_vec();
    fixture.roster_digest = crate::am::ec_distann::roster_digest(&descriptor.roster)
        .unwrap()
        .to_vec();
}

fn distann_catalog_name(name: &str) -> String {
    crate::am::ec_distann::catalog_relation_name(name)
        .expect("extension generation catalog should resolve")
}

fn canonical_index_locator(index_oid: pg_sys::Oid) -> String {
    Spi::get_one::<String>(&format!(
        "SELECT pg_catalog.format('%I.%I', n.nspname, c.relname)
           FROM pg_catalog.pg_class c
           JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
          WHERE c.oid = {}",
        u32::from(index_oid)
    ))
    .unwrap()
    .expect("index should have a canonical schema-qualified locator")
}

fn configure_distann_participant_identity(
    fixture: &DistannPhysicalGenerationFixture,
    endpoint_identity: &str,
) {
    configure_distann_participant_identity_at(fixture.index_oid, endpoint_identity);
}

fn configure_distann_participant_identity_at(index_oid: pg_sys::Oid, endpoint_identity: &str) {
    Spi::connect_mut(|client| {
        client
            .update(
                "SELECT ec_distann_configure_participant_identity(
                     $1::oid::regclass, $2::text
                 )",
                None,
                &[index_oid.into(), endpoint_identity.to_owned().into()],
            )
            .expect("participant identity should configure");
    });
}

fn create_distann_physical_generation_fixture(
    stem: &str,
    build_marker: u8,
) -> DistannPhysicalGenerationFixture {
    create_distann_physical_generation_fixture_with_payload_type(stem, build_marker, "text")
}

fn create_distann_physical_generation_fixture_with_payload_type(
    stem: &str,
    build_marker: u8,
    payload_type: &str,
) -> DistannPhysicalGenerationFixture {
    assert!(
        stem.bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_'),
        "test relation stem must be a trusted identifier"
    );
    assert!(
        payload_type.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.')
        }),
        "test payload type must be a trusted qualified identifier"
    );
    let table_name = format!("{stem}_source");
    let index_name = format!("{stem}_idx");
    Spi::run(&format!(
        "CREATE TABLE {table_name} (
             source_id uuid NOT NULL,
             payload {payload_type},
             legacy_payload integer,
             embedding ecvector(4) NOT NULL,
             payload_generated text GENERATED ALWAYS AS (payload || ':generated') STORED
         )"
    ))
    .expect("physical source shell should create");
    Spi::run(&format!(
        "ALTER TABLE {table_name} DROP COLUMN legacy_payload"
    ))
    .expect("source shell dropped-column slot should create");
    Spi::run(&format!(
        "CREATE INDEX {index_name} ON {table_name}
         USING ec_distann (embedding ecvector_distann_ip_ops)
         INCLUDE (source_id)
         WITH (
             distributed_control = true,
             source_identity = 'include',
             graph_degree = 4,
             neighbor_code_format = 'rabitq'
         )"
    ))
    .expect("distributed control fixture should create");

    let index_oid = Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
        .unwrap()
        .unwrap();
    let index_relation = IndexRelationGuard::open(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "physical generation fixture",
    );
    let metadata =
        unsafe { crate::am::ec_distann::read_metadata_from_index(index_relation.as_ptr()) }
            .expect("control metadata should decode");
    let heap_oid = unsafe { pg_sys::IndexGetRelation(index_oid, false) };
    let row_schema = crate::am::ec_distann::resolve_relation_schema(heap_oid)
        .expect("source shell schema should resolve")
        .descriptor;
    drop(index_relation);

    let logical_index_uuid = pgrx::datum::Uuid::from_bytes(metadata.logical_index_uuid);
    let descriptor = crate::am::ec_distann::DistannGenerationDescriptor {
        index_format_version: crate::am::ec_distann::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
        graph_record_version: crate::am::ec_distann::DISTANN_GRAPH_RECORD_VERSION,
        handoff_wire_version: crate::am::ec_distann::DISTANN_HANDOFF_WIRE_VERSION,
        dimensions: 4,
        graph_degree: metadata.graph_degree_r,
        placement_hash_version: crate::am::ec_distann::DISTANN_PLACEMENT_HASH_VERSION,
        roster: vec![crate::am::ec_distann::DistannRosterEntry {
            node_id: 17,
            logical_index_uuid: metadata.logical_index_uuid,
            endpoint_identity: format!("{stem}/node-17"),
        }],
        neighbor_codec_kind: metadata.neighbor_codec_kind,
        codec_artifact: crate::am::ec_distann::DistannCodecArtifact::RaBitQ {
            dimensions: 4,
            seed: metadata.seed,
            bits: 1,
        },
        row_schema,
    };
    let mut build_id = [build_marker; 16];
    build_id[6] = (build_id[6] & 0x0f) | 0x40;
    build_id[8] = (build_id[8] & 0x3f) | 0x80;
    DistannPhysicalGenerationFixture {
        index_name,
        canonical_index_regclass: canonical_index_locator(index_oid),
        index_oid,
        logical_index_uuid,
        build_id: pgrx::datum::Uuid::from_bytes(build_id),
        descriptor: descriptor.encode().expect("descriptor should encode"),
        descriptor_digest: descriptor.digest().expect("descriptor digest").to_vec(),
        roster_digest: crate::am::ec_distann::roster_digest(&descriptor.roster)
            .expect("roster digest")
            .to_vec(),
        build_spec_digest: vec![0x22; 32],
        expected_owner_digest: vec![0x44; 32],
    }
}

fn begin_distann_physical_generation(
    fixture: &DistannPhysicalGenerationFixture,
    expected_owner_digest: &[u8],
) -> (String, i64, i64, Vec<u8>) {
    begin_distann_physical_generation_count(fixture, 2, expected_owner_digest)
}

fn begin_distann_physical_generation_count(
    fixture: &DistannPhysicalGenerationFixture,
    expected_owner_count: i64,
    expected_owner_digest: &[u8],
) -> (String, i64, i64, Vec<u8>) {
    Spi::connect(|client| {
        client
            .select(
                "SELECT state, next_batch_seq, cumulative_record_count,
                        cumulative_owner_digest
                   FROM ec_distann_begin_epoch_handoff(
                       $1::regclass, 7, $2::uuid, $3::bytea, $4::bytea,
                       $5::bytea, $6::bytea, $7::bigint, $8::bytea
                   )",
                None,
                &[
                    fixture.index_oid.into(),
                    fixture.build_id.into(),
                    fixture.build_spec_digest.clone().into(),
                    fixture.roster_digest.clone().into(),
                    fixture.descriptor.clone().into(),
                    fixture.descriptor_digest.clone().into(),
                    expected_owner_count.into(),
                    expected_owner_digest.to_vec().into(),
                ],
            )
            .expect("begin handoff should execute")
            .map(|row| {
                (
                    row["state"].value::<String>().unwrap().unwrap(),
                    row["next_batch_seq"].value::<i64>().unwrap().unwrap(),
                    row["cumulative_record_count"]
                        .value::<i64>()
                        .unwrap()
                        .unwrap(),
                    row["cumulative_owner_digest"]
                        .value::<Vec<u8>>()
                        .unwrap()
                        .unwrap(),
                )
            })
            .next()
            .expect("begin handoff should return one row")
    })
}

fn distann_stage_batch_fixture(
    fixture: &DistannPhysicalGenerationFixture,
    batch_seq: u64,
    identity_marker: u8,
) -> (Vec<u8>, Vec<u8>, u64) {
    let descriptor =
        crate::am::ec_distann::DistannGenerationDescriptor::decode(&fixture.descriptor)
            .expect("generation descriptor should decode");
    let binding = crate::am::ec_distann::quantizer::DistannCodecBinding::from_artifact(
        &descriptor.codec_artifact,
    )
    .expect("codec binding should restore");
    let shape = crate::am::ec_distann::DistannHandoffShape {
        code_stride: binding
            .code_len(usize::from(descriptor.dimensions))
            .expect("codec length should resolve"),
        graph_degree: usize::from(descriptor.graph_degree),
        non_dropped_attribute_count: descriptor.row_schema.non_dropped_count(),
    };
    let mut identity = [identity_marker; 16];
    identity[6] = (identity[6] & 0x0f) | 0x40;
    identity[8] = (identity[8] & 0x3f) | 0x80;
    let identity_uuid = pgrx::datum::Uuid::from_bytes(identity);
    let (identity_bytes, payload_bytes, embedding_bytes, generated_bytes) =
        Spi::connect(|client| {
            client
                .select(
                    "SELECT pg_catalog.uuid_send($1::uuid) AS identity_bytes,
                            pg_catalog.textsend($2::text) AS payload_bytes,
                            ecvector_send(
                                encode_to_ecvector(ARRAY[1.0,0.0,0.0,0.0], 4, 42)
                            ) AS embedding_bytes,
                            pg_catalog.textsend($3::text) AS generated_bytes",
                    None,
                    &[
                        identity_uuid.into(),
                        "captured payload".to_owned().into(),
                        "captured payload:generated".to_owned().into(),
                    ],
                )
                .expect("binary row payload should encode")
                .map(|row| {
                    (
                        row["identity_bytes"].value::<Vec<u8>>().unwrap().unwrap(),
                        row["payload_bytes"].value::<Vec<u8>>().unwrap().unwrap(),
                        row["embedding_bytes"].value::<Vec<u8>>().unwrap().unwrap(),
                        row["generated_bytes"].value::<Vec<u8>>().unwrap().unwrap(),
                    )
                })
                .next()
                .expect("binary row payload should return one row")
        });
    assert_eq!(identity_bytes, identity);
    let vec_id = crate::am::ec_distann::vec_id_from_source_identity(&identity);
    let entry = crate::am::ec_distann::DistannHandoffEntry {
        vec_id,
        source_identity: identity.to_vec(),
        graph_flags: 0,
        search_code: vec![0x5a; shape.code_stride],
        neighbor_vec_ids: Vec::new(),
        neighbor_codes: Vec::new(),
        row_null_bitmap: vec![0],
        row_values: vec![
            identity_bytes,
            payload_bytes,
            embedding_bytes,
            generated_bytes,
        ],
    };
    let batch = crate::am::ec_distann::DistannHandoffBatch {
        epoch: 7,
        build_id: *fixture.build_id.as_bytes(),
        batch_seq,
        build_spec_digest: fixture
            .build_spec_digest
            .clone()
            .try_into()
            .expect("build spec digest width"),
        row_schema_fingerprint: descriptor
            .row_schema
            .fingerprint()
            .expect("row schema fingerprint"),
        index_format_version: crate::am::ec_distann::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
        neighbor_codec_kind: descriptor.neighbor_codec_kind,
        entries: vec![entry],
    };
    let digest = batch.digest(shape).expect("batch digest").to_vec();
    let encoded = batch.encode(shape).expect("batch encoding");
    (digest, encoded, vec_id)
}

fn distann_empty_stage_batch_fixture(
    fixture: &DistannPhysicalGenerationFixture,
) -> (Vec<u8>, Vec<u8>) {
    let descriptor =
        crate::am::ec_distann::DistannGenerationDescriptor::decode(&fixture.descriptor)
            .expect("generation descriptor should decode");
    let binding = crate::am::ec_distann::quantizer::DistannCodecBinding::from_artifact(
        &descriptor.codec_artifact,
    )
    .expect("codec binding should restore");
    let shape = crate::am::ec_distann::DistannHandoffShape {
        code_stride: binding
            .code_len(usize::from(descriptor.dimensions))
            .expect("codec length should resolve"),
        graph_degree: usize::from(descriptor.graph_degree),
        non_dropped_attribute_count: descriptor.row_schema.non_dropped_count(),
    };
    let batch = crate::am::ec_distann::DistannHandoffBatch {
        epoch: 7,
        build_id: *fixture.build_id.as_bytes(),
        batch_seq: 0,
        build_spec_digest: fixture
            .build_spec_digest
            .clone()
            .try_into()
            .expect("build spec digest width"),
        row_schema_fingerprint: descriptor
            .row_schema
            .fingerprint()
            .expect("row schema fingerprint"),
        index_format_version: crate::am::ec_distann::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
        neighbor_codec_kind: descriptor.neighbor_codec_kind,
        entries: Vec::new(),
    };
    (
        batch.digest(shape).expect("empty batch digest").to_vec(),
        batch.encode(shape).expect("empty batch encoding"),
    )
}

fn stage_distann_physical_batch(
    fixture: &DistannPhysicalGenerationFixture,
    batch_seq: i64,
    batch_digest: &[u8],
    encoded_batch: &[u8],
) -> (i64, i64, Vec<u8>) {
    Spi::connect(|client| {
        client
            .select(
                "SELECT accepted_record_count, cumulative_record_count,
                        cumulative_owner_digest
                   FROM ec_distann_stage_epoch_batch(
                       $1::regclass, $2::uuid, $3::bigint, $4::bytea, $5::bytea
                   )",
                None,
                &[
                    fixture.index_oid.into(),
                    fixture.build_id.into(),
                    batch_seq.into(),
                    batch_digest.to_vec().into(),
                    encoded_batch.to_vec().into(),
                ],
            )
            .expect("stage handoff should execute")
            .map(|row| {
                (
                    row["accepted_record_count"]
                        .value::<i64>()
                        .unwrap()
                        .unwrap(),
                    row["cumulative_record_count"]
                        .value::<i64>()
                        .unwrap()
                        .unwrap(),
                    row["cumulative_owner_digest"]
                        .value::<Vec<u8>>()
                        .unwrap()
                        .unwrap(),
                )
            })
            .next()
            .expect("stage handoff should return one row")
    })
}

fn distann_owner_digest_for_batch(
    fixture: &DistannPhysicalGenerationFixture,
    encoded_batch: &[u8],
) -> Vec<u8> {
    let descriptor =
        crate::am::ec_distann::DistannGenerationDescriptor::decode(&fixture.descriptor)
            .expect("generation descriptor should decode");
    let binding = crate::am::ec_distann::quantizer::DistannCodecBinding::from_artifact(
        &descriptor.codec_artifact,
    )
    .expect("codec binding should restore");
    let shape = crate::am::ec_distann::DistannHandoffShape {
        code_stride: binding
            .code_len(usize::from(descriptor.dimensions))
            .expect("codec length should resolve"),
        graph_degree: usize::from(descriptor.graph_degree),
        non_dropped_attribute_count: descriptor.row_schema.non_dropped_count(),
    };
    let batch = crate::am::ec_distann::DistannHandoffBatch::decode(encoded_batch, shape)
        .expect("batch should decode");
    crate::am::ec_distann::owner_stream_digest(&batch.entries, shape)
        .expect("owner stream should hash")
        .to_vec()
}

fn seal_distann_physical_generation(
    fixture: &DistannPhysicalGenerationFixture,
    expected_owner_count: i64,
    expected_owner_digest: &[u8],
) -> Vec<u8> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT ec_distann_seal_epoch_handoff(
                     $1::regclass, $2::uuid, $3::bigint, $4::bytea
                 ) AS ready_receipt",
                None,
                &[
                    fixture.index_oid.into(),
                    fixture.build_id.into(),
                    expected_owner_count.into(),
                    expected_owner_digest.to_vec().into(),
                ],
            )
            .expect("seal handoff should execute")
            .map(|row| row["ready_receipt"].value::<Vec<u8>>().unwrap().unwrap())
            .next()
            .expect("seal handoff should return one row")
    })
}

fn distann_generation_relation_oids(
    fixture: &DistannPhysicalGenerationFixture,
) -> (pg_sys::Oid, pg_sys::Oid, pg_sys::Oid) {
    let catalog = distann_generation_catalog_name();
    Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT row_tier_relid, graph_store_relid, directory_relid
                       FROM {catalog}
                      WHERE index_oid = $1::oid
                        AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid"
                ),
                None,
                &[
                    fixture.index_oid.into(),
                    fixture.logical_index_uuid.into(),
                    fixture.build_id.into(),
                ],
            )
            .unwrap()
            .map(|row| {
                (
                    row["row_tier_relid"]
                        .value::<pg_sys::Oid>()
                        .unwrap()
                        .unwrap(),
                    row["graph_store_relid"]
                        .value::<pg_sys::Oid>()
                        .unwrap()
                        .unwrap(),
                    row["directory_relid"]
                        .value::<pg_sys::Oid>()
                        .unwrap()
                        .unwrap(),
                )
            })
            .next()
            .expect("generation catalog row should exist")
    })
}

fn abort_distann_physical_generation(fixture: &DistannPhysicalGenerationFixture) {
    Spi::connect_mut(|client| {
        client
            .update(
                "SELECT ec_distann_abort_epoch_handoff($1::regclass, $2::uuid)",
                None,
                &[fixture.index_oid.into(), fixture.build_id.into()],
            )
            .expect("abort handoff should execute");
    });
}

fn register_distann_node(
    coordinator: &DistannPhysicalGenerationFixture,
    roster_ordinal: i32,
    node_id: i32,
    endpoint_identity: &str,
    conninfo_secret_name: &str,
    remote_index_regclass: &str,
    is_local: bool,
) {
    register_distann_node_at(
        coordinator.index_oid,
        roster_ordinal,
        node_id,
        endpoint_identity,
        conninfo_secret_name,
        remote_index_regclass,
        is_local,
    );
}

fn register_distann_node_at(
    coordinator_index_oid: pg_sys::Oid,
    roster_ordinal: i32,
    node_id: i32,
    endpoint_identity: &str,
    conninfo_secret_name: &str,
    remote_index_regclass: &str,
    is_local: bool,
) {
    Spi::connect_mut(|client| {
        client
            .update(
                "SELECT ec_distann_register_node_descriptor(
                     $1::oid::regclass, $2::integer, $3::integer,
                     $4::text, $5::text, $6::text, $7::boolean
                 )",
                None,
                &[
                    coordinator_index_oid.into(),
                    roster_ordinal.into(),
                    node_id.into(),
                    endpoint_identity.to_owned().into(),
                    conninfo_secret_name.to_owned().into(),
                    remote_index_regclass.to_owned().into(),
                    is_local.into(),
                ],
            )
            .expect("node registration should execute");
    });
}

#[pg_test]
fn test_distann_node_registration_provenance_and_guards() {
    const SECRET_NAME: &str = "DISTANN_LOCAL_REGISTRY";
    const SECRET_KEY: &str = "EC_SPIRE_REMOTE_CONNINFO_DISTANN_LOCAL_REGISTRY";
    let _env_lock = env_var_test_lock();
    assert_eq!(
        crate::am::spire_remote_conninfo_secret_provider_lookup_key(SECRET_NAME).unwrap(),
        SECRET_KEY
    );
    let _conninfo_secret = ScopedEnvVar::set(SECRET_KEY, "host=/unused dbname=unused");

    let coordinator =
        create_distann_physical_generation_fixture("ec_distann_registry_coordinator", 0x71);
    let participant =
        create_distann_physical_generation_fixture("ec_distann_registry_participant", 0x72);
    let participant_two =
        create_distann_physical_generation_fixture("ec_distann_registry_participant_two", 0x73);
    let participant_three =
        create_distann_physical_generation_fixture("ec_distann_registry_participant_three", 0x75);
    configure_distann_participant_identity(&participant, "registry/node-17");
    configure_distann_participant_identity(&participant_two, "registry/node-18");
    configure_distann_participant_identity(&participant_three, "registry/node-17");
    configure_distann_participant_identity(&participant, "registry/node-17");
    let identity_reconfigure_error = expect_pg_error_rolled_back(|| {
        configure_distann_participant_identity(&participant, "registry/changed-node");
    });
    assert!(identity_reconfigure_error.contains("already configured differently"));

    let coordinator_compatibility = Spi::get_one::<Vec<u8>>(&format!(
        "SELECT compatibility_digest
           FROM ec_distann_control_identity('{}'::regclass)",
        coordinator.index_name
    ))
    .unwrap()
    .unwrap();
    let participant_compatibility = Spi::get_one::<Vec<u8>>(&format!(
        "SELECT compatibility_digest
           FROM ec_distann_control_identity('{}'::regclass)",
        participant.index_name
    ))
    .unwrap()
    .unwrap();
    assert_eq!(coordinator_compatibility.len(), 32);
    assert_eq!(coordinator_compatibility, participant_compatibility);

    register_distann_node(
        &coordinator,
        0,
        17,
        "registry/node-17",
        SECRET_NAME,
        &participant.canonical_index_regclass,
        true,
    );
    let node_catalog = distann_catalog_name("ec_distann_node_descriptor");
    let (stored_uuid, stored_secret) = Spi::get_two::<pgrx::datum::Uuid, String>(&format!(
        "SELECT participant_logical_index_uuid, conninfo_secret_name
           FROM {node_catalog}
          WHERE index_oid = {} AND logical_index_uuid = '{}'::uuid
            AND roster_ordinal = 0",
        u32::from(coordinator.index_oid),
        coordinator.logical_index_uuid,
    ))
    .unwrap();
    assert_eq!(stored_uuid, Some(participant.logical_index_uuid));
    assert_eq!(stored_secret.as_deref(), Some(SECRET_NAME));
    let persisted_raw_conninfo = Spi::get_one::<bool>(&format!(
        "SELECT endpoint_identity LIKE '%host=%'
             OR conninfo_secret_name LIKE '%host=%'
             OR remote_index_regclass LIKE '%host=%'
           FROM {node_catalog}
          WHERE index_oid = {} AND roster_ordinal = 0",
        u32::from(coordinator.index_oid)
    ))
    .unwrap()
    .unwrap();
    assert!(!persisted_raw_conninfo);

    let registered_count = || {
        Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM {node_catalog}
              WHERE index_oid = {} AND logical_index_uuid = '{}'::uuid",
            u32::from(coordinator.index_oid),
            coordinator.logical_index_uuid,
        ))
        .unwrap()
        .unwrap()
    };
    for (label, expected, ordinal, node_id, endpoint, target, is_local) in [
        (
            "duplicate ordinal",
            "roster ordinal already exists",
            0,
            18,
            "registry/node-18",
            participant_two.canonical_index_regclass.as_str(),
            true,
        ),
        (
            "duplicate node",
            "node id already exists",
            1,
            17,
            "registry/node-18",
            participant_two.canonical_index_regclass.as_str(),
            true,
        ),
        (
            "duplicate endpoint",
            "endpoint identity already exists",
            1,
            18,
            "registry/node-17",
            participant_three.canonical_index_regclass.as_str(),
            true,
        ),
        (
            "duplicate participant UUID",
            "participant logical UUID already exists",
            1,
            18,
            "registry/node-17",
            participant.canonical_index_regclass.as_str(),
            true,
        ),
        (
            "second local participant",
            "local participant already exists",
            1,
            18,
            "registry/node-18",
            participant_two.canonical_index_regclass.as_str(),
            true,
        ),
    ] {
        let error = expect_pg_error_rolled_back(|| {
            register_distann_node(
                &coordinator,
                ordinal,
                node_id,
                endpoint,
                SECRET_NAME,
                target,
                is_local,
            );
        });
        assert!(
            error.contains("EC_NODE_DESCRIPTOR") && error.contains(expected),
            "{label} returned an unexpected error: {error}"
        );
        assert_eq!(registered_count(), 1, "{label} mutated the registry");
    }

    let raw_endpoint_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "host=secret.example dbname=leak",
            SECRET_NAME,
            &participant_two.canonical_index_regclass,
            true,
        );
    });
    assert!(raw_endpoint_error.contains("EC_NODE_DESCRIPTOR"));
    let raw_secret_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "registry/node-18",
            "host=secret.example",
            &participant_two.canonical_index_regclass,
            true,
        );
    });
    assert!(raw_secret_error.contains("EC_NODE_DESCRIPTOR"));
    let blocklist_counterexample_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "application_name=secret",
            SECRET_NAME,
            &participant_two.canonical_index_regclass,
            true,
        );
    });
    assert!(blocklist_counterexample_error.contains("canonical v1 grammar"));
    let provider_alias_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "registry/node-18",
            "DISTANN-LOCAL-REGISTRY",
            &participant_two.canonical_index_regclass,
            true,
        );
    });
    assert!(provider_alias_error.contains("canonical v1 grammar"));
    let missing_secret_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "registry/node-18",
            "DISTANN_MISSING_REGISTRY_SECRET",
            &participant_two.canonical_index_regclass,
            true,
        );
    });
    assert!(missing_secret_error.contains("conninfo secret is unavailable"));
    let endpoint_mismatch_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "registry/wrong-node",
            SECRET_NAME,
            &participant_two.canonical_index_regclass,
            true,
        );
    });
    assert!(endpoint_mismatch_error.contains("endpoint identity does not match"));
    let unqualified_locator_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "registry/node-18",
            SECRET_NAME,
            &participant_two.index_name,
            true,
        );
    });
    assert!(unqualified_locator_error.contains("schema-qualified canonical"));
    assert_eq!(
        registered_count(),
        1,
        "invalid references mutated the registry"
    );

    let incompatible =
        create_distann_physical_generation_fixture("ec_distann_registry_incompatible", 0x74);
    Spi::run(&format!(
        "ALTER INDEX {} SET (graph_degree = 8); REINDEX INDEX {}",
        incompatible.index_name, incompatible.index_name
    ))
    .unwrap();
    configure_distann_participant_identity(&incompatible, "registry/node-18");
    let incompatible_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "registry/node-18",
            SECRET_NAME,
            &incompatible.canonical_index_regclass,
            true,
        );
    });
    assert!(
        incompatible_error.contains("schema/reloptions are incompatible"),
        "unexpected compatibility error: {incompatible_error}"
    );
    assert_eq!(
        registered_count(),
        1,
        "incompatible control mutated the registry"
    );

    let shape_drift =
        create_distann_physical_generation_fixture("ec_distann_registry_shape_drift", 0x75);
    Spi::run(
        "ALTER TABLE ec_distann_registry_shape_drift_source
             ADD COLUMN participant_only_payload text",
    )
    .unwrap();
    configure_distann_participant_identity(&shape_drift, "registry/node-20");
    let shape_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            20,
            "registry/node-20",
            SECRET_NAME,
            &shape_drift.canonical_index_regclass,
            true,
        );
    });
    assert!(
        shape_error.contains("schema/reloptions are incompatible"),
        "row-schema fingerprint drift must fail registration: {shape_error}"
    );
    assert_eq!(registered_count(), 1, "shape drift mutated the registry");

    let build_catalog = distann_catalog_name("ec_distann_build_registration");
    let binding_catalog = distann_catalog_name("ec_distann_build_participant_binding");
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "INSERT INTO {build_catalog} (
                         index_oid, logical_index_uuid, build_id, epoch, state,
                         registry_revision, roster_snapshot, roster_digest,
                         row_schema_fingerprint, registration_digest
                     ) VALUES (
                         $1::oid, $2::uuid, $3::uuid, 7, 'Registered', 1,
                         '\\x01'::bytea, decode(repeat('11', 32), 'hex'),
                         decode(repeat('22', 32), 'hex'),
                         decode(repeat('33', 32), 'hex')
                     )"
                ),
                None,
                &[
                    coordinator.index_oid.into(),
                    coordinator.logical_index_uuid.into(),
                    coordinator.build_id.into(),
                ],
            )
            .unwrap();
        client
            .update(
                &format!(
                    "INSERT INTO {binding_catalog} (
                         index_oid, logical_index_uuid, build_id, roster_ordinal,
                         node_id, endpoint_identity, conninfo_secret_name,
                         remote_index_regclass, participant_logical_index_uuid,
                         compatibility_digest, is_local
                     ) VALUES (
                         $1::oid, $2::uuid, $3::uuid, 0, 17,
                         'registry/node-17', $4::text, $5::text, $6::uuid,
                         $7::bytea, true
                     )"
                ),
                None,
                &[
                    coordinator.index_oid.into(),
                    coordinator.logical_index_uuid.into(),
                    coordinator.build_id.into(),
                    SECRET_NAME.to_owned().into(),
                    participant.canonical_index_regclass.clone().into(),
                    participant.logical_index_uuid.into(),
                    coordinator_compatibility.clone().into(),
                ],
            )
            .unwrap();
    });
    let guarded_error = expect_pg_error_rolled_back(|| {
        Spi::run(&format!(
            "SELECT ec_distann_unregister_node_descriptor('{}'::regclass, 0)",
            coordinator.index_name
        ))
        .expect("referenced unregister must fail");
    });
    assert!(guarded_error.contains("EC_BUILD_STATE"));
    Spi::run(&format!(
        "UPDATE {build_catalog} SET state = 'Published'
          WHERE index_oid = {} AND logical_index_uuid = '{}'::uuid
            AND build_id = '{}'::uuid",
        u32::from(coordinator.index_oid),
        coordinator.logical_index_uuid,
        coordinator.build_id,
    ))
    .unwrap();
    Spi::run(&format!(
        "SELECT ec_distann_unregister_node_descriptor('{}'::regclass, 0)",
        coordinator.index_name
    ))
    .unwrap();
    assert_eq!(registered_count(), 0);
    assert_eq!(
        Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM {binding_catalog}
              WHERE index_oid = {} AND build_id = '{}'::uuid",
            u32::from(coordinator.index_oid),
            coordinator.build_id,
        ))
        .unwrap(),
        Some(1),
        "Published build binding must outlive desired-roster removal"
    );
    register_distann_node(
        &coordinator,
        0,
        18,
        "registry/node-18",
        SECRET_NAME,
        &participant_two.canonical_index_regclass,
        true,
    );
    assert_eq!(registered_count(), 1);
    Spi::run(&format!(
        "SELECT ec_distann_unregister_node_descriptor('{}'::regclass, 0)",
        coordinator.index_name
    ))
    .unwrap();
    assert_eq!(registered_count(), 0);
    let publish_catalog = distann_catalog_name("ec_distann_publish_decision");
    Spi::run(&format!(
        "INSERT INTO {publish_catalog} (
             index_oid, logical_index_uuid, build_id, epoch,
             epoch_fingerprint, manifest_digest, epoch_manifest, decision_state
         ) VALUES (
             {}, '{}'::uuid, '{}'::uuid, 7,
             decode(repeat('44', 34), 'hex'), decode(repeat('55', 32), 'hex'),
             '\\x01'::bytea, 'Applied'
         )",
        u32::from(coordinator.index_oid),
        coordinator.logical_index_uuid,
        coordinator.build_id,
    ))
    .unwrap();
    let pinned_registration = expect_pg_error_rolled_back(|| {
        Spi::run(&format!(
            "DELETE FROM {build_catalog}
              WHERE index_oid = {} AND logical_index_uuid = '{}'::uuid",
            u32::from(coordinator.index_oid),
            coordinator.logical_index_uuid,
        ))
        .expect("publish decision must pin its build registration");
    });
    assert!(
        pinned_registration.contains("foreign key"),
        "unexpected build-registration retention error: {pinned_registration}"
    );
    Spi::run(&format!(
        "DELETE FROM {publish_catalog}
          WHERE index_oid = {} AND logical_index_uuid = '{}'::uuid;
         DELETE FROM {build_catalog}
          WHERE index_oid = {} AND logical_index_uuid = '{}'::uuid",
        u32::from(coordinator.index_oid),
        coordinator.logical_index_uuid,
        u32::from(coordinator.index_oid),
        coordinator.logical_index_uuid,
    ))
    .unwrap();
    assert_eq!(
        Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM {node_catalog} WHERE index_oid = {}",
            u32::from(coordinator.index_oid)
        ))
        .unwrap(),
        Some(0)
    );

    // Exercise the production libpq provenance path against a separately
    // committed loopback participant. The participant relation is deliberately
    // invisible to the coordinator's local catalog lookup, so this cannot
    // accidentally fall back to the local identity path.
    const REMOTE_SECRET_NAME: &str = "DISTANN_LOOPBACK_REGISTRY";
    const REMOTE_SECRET_KEY: &str = "EC_SPIRE_REMOTE_CONNINFO_DISTANN_LOOPBACK_REGISTRY";
    assert_eq!(
        crate::am::spire_remote_conninfo_secret_provider_lookup_key(REMOTE_SECRET_NAME).unwrap(),
        REMOTE_SECRET_KEY
    );
    const BROKEN_SECRET_NAME: &str = "DISTANN_BROKEN_REGISTRY";
    const BROKEN_SECRET_KEY: &str = "EC_SPIRE_REMOTE_CONNINFO_DISTANN_BROKEN_REGISTRY";
    let _broken_conninfo_secret = ScopedEnvVar::set(
        BROKEN_SECRET_KEY,
        "host=/tmp/ecaz_distann_missing_socket dbname=secret password=do_not_expose connect_timeout=1",
    );
    let connection_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "registry/broken-node",
            BROKEN_SECRET_NAME,
            "public.ec_distann_missing_remote_idx",
            false,
        );
    });
    assert!(connection_error.contains("remote control connection failed"));
    for forbidden in [
        BROKEN_SECRET_NAME,
        "ecaz_distann_missing_socket",
        "do_not_expose",
        "dbname=secret",
    ] {
        assert!(
            !connection_error.contains(forbidden),
            "sanitized connection error exposed {forbidden}: {connection_error}"
        );
    }
    let loopback_conninfo = current_pg_test_loopback_conninfo();
    let _remote_conninfo_secret = ScopedEnvVar::set(REMOTE_SECRET_KEY, &loopback_conninfo);
    let mut loopback_client = postgres::Client::connect(&loopback_conninfo, postgres::NoTls)
        .expect("registration loopback connection should succeed");
    let extension_schema = loopback_client
        .query_one(
            "SELECT pg_catalog.quote_ident(n.nspname)
               FROM pg_catalog.pg_extension e
               JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace
              WHERE e.extname = 'ecaz'",
            &[],
        )
        .expect("remote extension schema should resolve")
        .try_get::<_, String>(0)
        .expect("remote extension schema should decode");
    let remote_table = format!("{extension_schema}.ec_distann_registry_loopback_remote_source");
    let remote_index = format!("{extension_schema}.ec_distann_registry_loopback_remote_idx");
    let missing_remote_index = format!("{extension_schema}.ec_distann_registry_missing_idx");
    let query_error = expect_pg_error_rolled_back(|| {
        register_distann_node(
            &coordinator,
            1,
            18,
            "registry/missing-node",
            REMOTE_SECRET_NAME,
            &missing_remote_index,
            false,
        );
    });
    assert!(query_error.contains("remote control identity query failed"));
    assert!(!query_error.contains(REMOTE_SECRET_NAME));
    assert!(!query_error.contains("ec_distann_registry_missing_idx"));
    loopback_client
        .batch_execute(&format!(
            "SET search_path TO {extension_schema}, pg_catalog;
             DROP TABLE IF EXISTS {remote_table} CASCADE;
             CREATE TABLE {remote_table} (
                 source_id uuid NOT NULL,
                 payload text,
                 legacy_payload integer,
                 embedding ecvector(4) NOT NULL,
                 payload_generated text GENERATED ALWAYS AS (payload || ':generated') STORED
             );
             ALTER TABLE {remote_table} DROP COLUMN legacy_payload;
             CREATE INDEX ec_distann_registry_loopback_remote_idx ON {remote_table}
             USING ec_distann (embedding ecvector_distann_ip_ops)
             INCLUDE (source_id)
             WITH (
                 distributed_control = true,
                 source_identity = 'include',
                 graph_degree = 4,
                 neighbor_code_format = 'rabitq'
             )"
        ))
        .expect("remote registration fixture should create");
    loopback_client
        .execute(
            &format!(
                "SELECT {extension_schema}.ec_distann_configure_participant_identity(
                     $1::text::regclass, $2::text
                 )"
            ),
            &[&remote_index, &"registry/loopback-node-18"],
        )
        .expect("remote participant identity should configure");
    let remote_uuid = loopback_client
        .query_one(
            &format!(
                "SELECT logical_index_uuid::text
                   FROM {extension_schema}.ec_distann_control_identity($1::text::regclass)"
            ),
            &[&remote_index],
        )
        .expect("remote control identity should execute")
        .try_get::<_, String>(0)
        .expect("remote logical UUID should decode");
    register_distann_node(
        &coordinator,
        1,
        18,
        "registry/loopback-node-18",
        REMOTE_SECRET_NAME,
        &remote_index,
        false,
    );
    let stored_remote_uuid = Spi::get_one::<String>(&format!(
        "SELECT participant_logical_index_uuid::text
           FROM {node_catalog}
          WHERE index_oid = {} AND logical_index_uuid = '{}'::uuid
            AND roster_ordinal = 1",
        u32::from(coordinator.index_oid),
        coordinator.logical_index_uuid,
    ))
    .unwrap()
    .expect("remote participant UUID should be stored");
    assert_eq!(stored_remote_uuid, remote_uuid);
    Spi::run(&format!(
        "SELECT ec_distann_unregister_node_descriptor('{}'::regclass, 1)",
        coordinator.index_name
    ))
    .unwrap();
    loopback_client
        .batch_execute(&format!("DROP TABLE {remote_table} CASCADE"))
        .expect("remote registration fixture should clean up");
    let registry_state = distann_catalog_name("ec_distann_registry_state");
    assert_eq!(
        Spi::get_one::<i64>(&format!(
            "SELECT revision FROM {registry_state}
              WHERE index_oid = {} AND logical_index_uuid = '{}'::uuid",
            u32::from(coordinator.index_oid),
            coordinator.logical_index_uuid,
        ))
        .unwrap(),
        Some(6),
        "only committed desired-roster mutations may advance the revision"
    );
}

#[pg_test]
fn test_distann_node_registration_binds_indexed_key_attnum() {
    const SECRET_NAME: &str = "DISTANN_KEY_LAYOUT";
    const SECRET_KEY: &str = "EC_SPIRE_REMOTE_CONNINFO_DISTANN_KEY_LAYOUT";
    let _env_lock = env_var_test_lock();
    let _conninfo_secret = ScopedEnvVar::set(SECRET_KEY, "host=/unused dbname=unused");
    Spi::run(
        "CREATE TABLE ec_distann_key_layout_coordinator_source (
             source_id uuid NOT NULL,
             embedding_a ecvector(4) NOT NULL,
             embedding_b ecvector(4) NOT NULL
         );
         CREATE INDEX ec_distann_key_layout_coordinator_idx
           ON ec_distann_key_layout_coordinator_source
           USING ec_distann (embedding_a ecvector_distann_ip_ops)
           INCLUDE (source_id)
           WITH (distributed_control = true, source_identity = 'include', graph_degree = 4);
         CREATE TABLE ec_distann_key_layout_participant_source (
             source_id uuid NOT NULL,
             embedding_a ecvector(4) NOT NULL,
             embedding_b ecvector(4) NOT NULL
         );
         CREATE INDEX ec_distann_key_layout_participant_idx
           ON ec_distann_key_layout_participant_source
           USING ec_distann (embedding_b ecvector_distann_ip_ops)
           INCLUDE (source_id)
           WITH (distributed_control = true, source_identity = 'include', graph_degree = 4);
         CREATE OPERATOR CLASS ec_distann_shadow_ip_ops
           FOR TYPE ecvector USING ec_distann AS
             OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
             FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);
         CREATE TABLE ec_distann_key_layout_shadow_source (
             source_id uuid NOT NULL,
             embedding_a ecvector(4) NOT NULL,
             embedding_b ecvector(4) NOT NULL
         );
         CREATE INDEX ec_distann_key_layout_shadow_idx
           ON ec_distann_key_layout_shadow_source
           USING ec_distann (embedding_a ec_distann_shadow_ip_ops)
           INCLUDE (source_id)
           WITH (distributed_control = true, source_identity = 'include', graph_degree = 4)",
    )
    .unwrap();
    let coordinator_oid = Spi::get_one::<pg_sys::Oid>(
        "SELECT 'ec_distann_key_layout_coordinator_idx'::regclass::oid",
    )
    .unwrap()
    .unwrap();
    let participant_oid = Spi::get_one::<pg_sys::Oid>(
        "SELECT 'ec_distann_key_layout_participant_idx'::regclass::oid",
    )
    .unwrap()
    .unwrap();
    configure_distann_participant_identity_at(participant_oid, "registry/key-layout");
    let participant_locator = canonical_index_locator(participant_oid);
    let error = expect_pg_error_rolled_back(|| {
        register_distann_node_at(
            coordinator_oid,
            0,
            19,
            "registry/key-layout",
            SECRET_NAME,
            &participant_locator,
            true,
        );
    });
    assert!(
        error.contains("schema/reloptions are incompatible"),
        "key-attnum drift must fail registration: {error}"
    );
    let shadow_oid =
        Spi::get_one::<pg_sys::Oid>("SELECT 'ec_distann_key_layout_shadow_idx'::regclass::oid")
            .unwrap()
            .unwrap();
    configure_distann_participant_identity_at(shadow_oid, "registry/shadow-opclass");
    let shadow_error = expect_pg_error_rolled_back(|| {
        Spi::run(
            "SELECT compatibility_digest
               FROM ec_distann_control_identity(
                   'ec_distann_key_layout_shadow_idx'::regclass
               )",
        )
        .expect("shadow opclass identity must fail");
    });
    assert!(
        shadow_error.contains("key/opclass/identity/readiness contract"),
        "custom opclass must not enter compatibility identity: {shadow_error}"
    );
}

#[pg_test]
fn test_distann_generation_relations_replay_abort_and_privileges() {
    let fixture =
        create_distann_physical_generation_fixture("ec_distann_generation_lifecycle", 0x31);
    let identity = Spi::connect(|client| {
        let table = client
            .select(
                &format!(
                    "SELECT logical_index_uuid, index_format_version
                       FROM ec_distann_control_identity('{}'::regclass)",
                    fixture.index_name
                ),
                None,
                &[],
            )
            .unwrap()
            .first();
        let (uuid, format_version) = table
            .get_two::<pgrx::datum::Uuid, i32>()
            .expect("control identity columns should decode");
        (
            uuid.expect("control identity UUID should exist"),
            format_version.expect("control format version should exist"),
        )
    });
    assert_eq!(identity.0, fixture.logical_index_uuid);
    assert_eq!(identity.1, 5);

    let first = begin_distann_physical_generation(&fixture, &fixture.expected_owner_digest);
    assert_eq!(first.0, "Building");
    assert_eq!((first.1, first.2), (0, 0));
    assert_eq!(first.3.len(), 32);
    let relations = distann_generation_relation_oids(&fixture);

    let dependency_count = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM pg_depend
          WHERE classid = 'pg_class'::regclass
            AND objid IN ({}, {}, {})
            AND refclassid = 'pg_class'::regclass
            AND refobjid = {}
            AND deptype = 'i'",
        u32::from(relations.0),
        u32::from(relations.1),
        u32::from(relations.2),
        u32::from(fixture.index_oid),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(dependency_count, 3);
    let ownership_and_persistence = Spi::get_one::<bool>(&format!(
        "SELECT bool_and(child.relowner = control.relowner
                         AND child.relpersistence = 'p'
                         AND child.relnamespace = control.relnamespace)
           FROM pg_class child
           CROSS JOIN pg_class control
          WHERE control.oid = {}
            AND child.oid IN ({}, {}, {})",
        u32::from(fixture.index_oid),
        u32::from(relations.0),
        u32::from(relations.1),
        u32::from(relations.2),
    ))
    .unwrap()
    .unwrap();
    assert!(
        ownership_and_persistence,
        "physical relations must share the control owner/schema and remain WAL-logged"
    );
    let toast_count = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM pg_class
          WHERE oid IN ({}, {}) AND reltoastrelid <> 0",
        u32::from(relations.0),
        u32::from(relations.1),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(toast_count, 2, "row and graph heaps need TOAST support");
    let generated_is_plain = Spi::get_one::<bool>(&format!(
        "SELECT attgenerated = ''
           FROM pg_attribute
          WHERE attrelid = {} AND attname = 'payload_generated'",
        u32::from(relations.0),
    ))
    .unwrap()
    .unwrap();
    assert!(
        generated_is_plain,
        "captured generated values must not recompute"
    );
    let row_tier_not_null_count = Spi::get_one::<i64>(&format!(
        "SELECT count(*)
           FROM pg_attribute
          WHERE attrelid = {} AND attnum > 0 AND NOT attisdropped AND attnotnull",
        u32::from(relations.0),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(
        row_tier_not_null_count, 0,
        "captured row-tier storage must not copy source NOT-NULL constraints"
    );
    let dropped_slot = Spi::get_one::<bool>(&format!(
        "SELECT attisdropped
           FROM pg_attribute
          WHERE attrelid = {} AND attnum = 3",
        u32::from(relations.0),
    ))
    .unwrap()
    .unwrap();
    assert!(dropped_slot, "row tier preserves physical attnum gaps");
    assert!(Spi::get_one::<bool>(&format!(
        "SELECT indisunique FROM pg_index WHERE indexrelid = {}",
        u32::from(relations.2)
    ))
    .unwrap()
    .unwrap());

    let replay = begin_distann_physical_generation(&fixture, &fixture.expected_owner_digest);
    assert_eq!(replay, first, "exact begin replay returns prior progress");
    assert_eq!(distann_generation_relation_oids(&fixture), relations);
    let conflict = expect_pg_error(|| {
        begin_distann_physical_generation(&fixture, &[0x99; 32]);
    });
    assert!(
        conflict.contains("EC_BUILD_ID_CONFLICT"),
        "unexpected begin conflict: {conflict}"
    );

    let unpublished_count = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM ec_distann_list_unpublished_generations('{}'::regclass)",
        fixture.index_name
    ))
    .unwrap()
    .unwrap();
    assert_eq!(unpublished_count, 1);

    let public_catalog_acl_count = Spi::get_one::<i64>(
        "SELECT count(*)
           FROM pg_class c
           CROSS JOIN LATERAL aclexplode(COALESCE(c.relacl, acldefault('r', c.relowner))) acl
          WHERE c.relname LIKE 'ec_distann_%'
            AND c.relkind = 'r'
            AND acl.grantee = 0",
    )
    .unwrap()
    .unwrap();
    assert_eq!(public_catalog_acl_count, 0);
    let public_endpoint_acl_count = Spi::get_one::<i64>(
        "SELECT count(*)
           FROM pg_proc p
           CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl, acldefault('f', p.proowner))) acl
          WHERE p.proname IN (
              'ec_distann_control_identity',
              'ec_distann_configure_participant_identity',
              'ec_distann_register_node_descriptor',
              'ec_distann_unregister_node_descriptor',
              'ec_distann_begin_epoch_handoff',
              'ec_distann_stage_epoch_batch',
              'ec_distann_seal_epoch_handoff',
              'ec_distann_abort_epoch_handoff',
              'ec_distann_list_unpublished_generations',
              'ec_distann_catalog_index_cleanup'
          )
            AND acl.grantee = 0
            AND acl.privilege_type = 'EXECUTE'",
    )
    .unwrap()
    .unwrap();
    assert_eq!(public_endpoint_acl_count, 0);

    // Simulate a stale catalog row surviving under a reused local OID: the
    // current control UUID must make it unreachable even when build_id/OID and
    // physical relation locators otherwise match.
    let mut stale_uuid_bytes = [0x71; 16];
    stale_uuid_bytes[6] = 0x41;
    stale_uuid_bytes[8] = 0x91;
    let stale_uuid = pgrx::datum::Uuid::from_bytes(stale_uuid_bytes);
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "UPDATE {} SET logical_index_uuid = $1::uuid
                      WHERE index_oid = $2::oid
                        AND logical_index_uuid = $3::uuid
                        AND build_id = $4::uuid",
                    distann_generation_catalog_name()
                ),
                None,
                &[
                    stale_uuid.into(),
                    fixture.index_oid.into(),
                    fixture.logical_index_uuid.into(),
                    fixture.build_id.into(),
                ],
            )
            .unwrap();
    });
    abort_distann_physical_generation(&fixture);
    let stale_relations_still_live = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM pg_class WHERE oid IN ({}, {}, {})",
        u32::from(relations.0),
        u32::from(relations.1),
        u32::from(relations.2),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(stale_relations_still_live, 3);
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "UPDATE {} SET logical_index_uuid = $1::uuid
                      WHERE index_oid = $2::oid
                        AND logical_index_uuid = $3::uuid
                        AND build_id = $4::uuid",
                    distann_generation_catalog_name()
                ),
                None,
                &[
                    fixture.logical_index_uuid.into(),
                    fixture.index_oid.into(),
                    stale_uuid.into(),
                    fixture.build_id.into(),
                ],
            )
            .unwrap();
    });

    abort_distann_physical_generation(&fixture);
    abort_distann_physical_generation(&fixture);
    let live_relation_count = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM pg_class WHERE oid IN ({}, {}, {})",
        u32::from(relations.0),
        u32::from(relations.1),
        u32::from(relations.2),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(live_relation_count, 0);
}

#[pg_test]
fn test_distann_stage_batch_atomic_replay_and_directory() {
    let fixture = create_distann_physical_generation_fixture("ec_distann_stage_batch", 0x39);
    begin_distann_physical_generation(&fixture, &fixture.expected_owner_digest);
    let relations = distann_generation_relation_oids(&fixture);
    let (digest, encoded, vec_id) = distann_stage_batch_fixture(&fixture, 0, 0x73);

    let first = stage_distann_physical_batch(&fixture, 0, &digest, &encoded);
    assert_eq!((first.0, first.1, first.2.len()), (1, 1, 32));
    let replay = stage_distann_physical_batch(&fixture, 0, &digest, &encoded);
    assert_eq!(
        replay, first,
        "exact replay must return the journaled receipt"
    );

    let generation_catalog = distann_generation_catalog_name();
    let batch_catalog = distann_catalog_name("ec_distann_generation_batch");
    let progress = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT next_batch_seq, cumulative_record_count,
                            cumulative_owner_digest, last_vec_id_le,
                            owner_stream_sha256_state, ready_receipt
                       FROM {generation_catalog}
                      WHERE index_oid = $1::oid
                        AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid"
                ),
                None,
                &[
                    fixture.index_oid.into(),
                    fixture.logical_index_uuid.into(),
                    fixture.build_id.into(),
                ],
            )
            .unwrap()
            .map(|row| {
                (
                    row["next_batch_seq"].value::<i64>().unwrap().unwrap(),
                    row["cumulative_record_count"]
                        .value::<i64>()
                        .unwrap()
                        .unwrap(),
                    row["cumulative_owner_digest"]
                        .value::<Vec<u8>>()
                        .unwrap()
                        .unwrap(),
                    row["last_vec_id_le"].value::<Vec<u8>>().unwrap().unwrap(),
                    row["owner_stream_sha256_state"]
                        .value::<Vec<u8>>()
                        .unwrap()
                        .unwrap(),
                    row["ready_receipt"].value::<Vec<u8>>().unwrap(),
                )
            })
            .next()
            .expect("generation progress should exist")
    });
    assert_eq!((progress.0, progress.1), (1, 1));
    assert_eq!(progress.2, first.2);
    assert_eq!(progress.3, vec_id.to_le_bytes());
    assert_eq!(progress.4.len(), 107);
    assert!(progress.5.is_none());
    let journal_count = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT count(*) AS batch_count FROM {batch_catalog}
                      WHERE index_oid = $1::oid AND build_id = $2::uuid"
                ),
                None,
                &[fixture.index_oid.into(), fixture.build_id.into()],
            )
            .unwrap()
            .map(|row| row["batch_count"].value::<i64>().unwrap().unwrap())
            .next()
            .unwrap()
    });
    assert_eq!(
        journal_count, 1,
        "exact replay must not duplicate the journal"
    );

    let row_relation = canonical_index_locator(relations.0);
    let graph_relation = canonical_index_locator(relations.1);
    let captured = Spi::connect(|client| {
        client
            .select(
                &format!("SELECT payload, payload_generated FROM {row_relation}"),
                None,
                &[],
            )
            .unwrap()
            .map(|row| {
                (
                    row["payload"].value::<String>().unwrap().unwrap(),
                    row["payload_generated"].value::<String>().unwrap().unwrap(),
                )
            })
            .next()
            .expect("captured row should exist")
    });
    assert_eq!(captured.0, "captured payload");
    assert_eq!(captured.1, "captured payload:generated");
    let graph = Spi::connect(|client| {
        client
            .select(
                &format!("SELECT vec_id, graph_record, row_tid FROM {graph_relation}"),
                None,
                &[],
            )
            .unwrap()
            .map(|row| {
                (
                    row["vec_id"].value::<i64>().unwrap().unwrap(),
                    row["graph_record"].value::<Vec<u8>>().unwrap().unwrap(),
                    row["row_tid"]
                        .value::<pg_sys::ItemPointerData>()
                        .unwrap()
                        .unwrap(),
                )
            })
            .next()
            .expect("graph row should exist")
    });
    assert_eq!(u64::from_le_bytes(graph.0.to_le_bytes()), vec_id);
    let descriptor =
        crate::am::ec_distann::DistannGenerationDescriptor::decode(&fixture.descriptor).unwrap();
    let binding = crate::am::ec_distann::quantizer::DistannCodecBinding::from_artifact(
        &descriptor.codec_artifact,
    )
    .unwrap();
    let code_len = binding
        .code_len(usize::from(descriptor.dimensions))
        .unwrap();
    let node = crate::am::ec_distann::tuple::DistannNodeTuple::decode_physical_v1(
        &graph.1,
        descriptor.graph_degree,
        code_len,
    )
    .unwrap();
    assert_eq!(node.vec_id, vec_id);
    assert_eq!(
        (node.heap_tid.block_number, node.heap_tid.offset_number),
        pgrx::itemptr::item_pointer_get_both(graph.2)
    );

    let (conflict_digest, conflict_encoded, _) = distann_stage_batch_fixture(&fixture, 0, 0x74);
    let conflict = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&fixture, 0, &conflict_digest, &conflict_encoded);
    });
    assert!(conflict.contains("EC_BATCH_CONFLICT"), "{conflict}");
    let (skip_digest, skip_encoded, _) = distann_stage_batch_fixture(&fixture, 2, 0x75);
    let sequence = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&fixture, 2, &skip_digest, &skip_encoded);
    });
    assert!(sequence.contains("EC_BATCH_SEQUENCE"), "{sequence}");

    assert_eq!(
        Spi::get_one::<i64>(&format!("SELECT count(*) FROM {row_relation}"))
            .unwrap()
            .unwrap(),
        1
    );
    assert_eq!(
        Spi::get_one::<i64>(&format!("SELECT count(*) FROM {graph_relation}"))
            .unwrap()
            .unwrap(),
        1
    );
    abort_distann_physical_generation(&fixture);
}

#[pg_test]
fn test_distann_stage_seal_zero_mutation_matrix() {
    let fixture = create_distann_physical_generation_fixture("ec_distann_negative_matrix", 0x68);
    begin_distann_physical_generation_count(&fixture, 2, &fixture.expected_owner_digest);
    let (valid_digest, valid_encoded, _) = distann_stage_batch_fixture(&fixture, 0, 0x69);
    let descriptor =
        crate::am::ec_distann::DistannGenerationDescriptor::decode(&fixture.descriptor).unwrap();
    let binding = crate::am::ec_distann::quantizer::DistannCodecBinding::from_artifact(
        &descriptor.codec_artifact,
    )
    .unwrap();
    let shape = crate::am::ec_distann::DistannHandoffShape {
        code_stride: binding
            .code_len(usize::from(descriptor.dimensions))
            .unwrap(),
        graph_degree: usize::from(descriptor.graph_degree),
        non_dropped_attribute_count: descriptor.row_schema.non_dropped_count(),
    };
    let baseline = distann_physical_mutation_state(&fixture);

    let mut malformed = valid_encoded.clone();
    malformed[113] ^= 0xff;
    let (malformed_digest, malformed) = rehash_distann_handoff_batch(malformed);
    let malformed_error = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&fixture, 0, &malformed_digest, &malformed);
    });
    assert!(
        malformed_error.contains("EC_HANDOFF_FORMAT"),
        "{malformed_error}"
    );
    assert_eq!(distann_physical_mutation_state(&fixture), baseline);

    let mut schema_batch =
        crate::am::ec_distann::DistannHandoffBatch::decode(&valid_encoded, shape).unwrap();
    schema_batch.row_schema_fingerprint = [0x91; 32];
    let schema_encoded = schema_batch.encode(shape).unwrap();
    let schema_digest = schema_batch.digest(shape).unwrap().to_vec();
    let schema_error = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&fixture, 0, &schema_digest, &schema_encoded);
    });
    assert!(schema_error.contains("EC_HANDOFF_FORMAT"), "{schema_error}");
    assert_eq!(distann_physical_mutation_state(&fixture), baseline);

    let mut codec_batch =
        crate::am::ec_distann::DistannHandoffBatch::decode(&valid_encoded, shape).unwrap();
    codec_batch.neighbor_codec_kind = if descriptor.neighbor_codec_kind == 1 {
        2
    } else {
        1
    };
    let codec_encoded = codec_batch.encode(shape).unwrap();
    let codec_digest = codec_batch.digest(shape).unwrap().to_vec();
    let codec_error = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&fixture, 0, &codec_digest, &codec_encoded);
    });
    assert!(codec_error.contains("EC_HANDOFF_FORMAT"), "{codec_error}");
    assert_eq!(distann_physical_mutation_state(&fixture), baseline);

    let mut noncanonical_batch =
        crate::am::ec_distann::DistannHandoffBatch::decode(&valid_encoded, shape).unwrap();
    noncanonical_batch.entries[0].row_values[2].push(0);
    let noncanonical_encoded = noncanonical_batch.encode(shape).unwrap();
    let noncanonical_digest = noncanonical_batch.digest(shape).unwrap().to_vec();
    let noncanonical_error = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&fixture, 0, &noncanonical_digest, &noncanonical_encoded);
    });
    assert!(
        noncanonical_error.contains("EC_HANDOFF_FORMAT")
            || noncanonical_error.contains("invalid")
            || noncanonical_error.contains("incorrect"),
        "{noncanonical_error}"
    );
    assert_eq!(distann_physical_mutation_state(&fixture), baseline);

    let oversized = vec![0_u8; crate::am::ec_distann::DISTANN_HANDOFF_MAX_BYTES + 1];
    let oversize_error = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&fixture, 0, &[0; 32], &oversized);
    });
    assert!(
        oversize_error.contains("EC_HANDOFF_TOO_LARGE"),
        "{oversize_error}"
    );
    assert_eq!(distann_physical_mutation_state(&fixture), baseline);

    let prepared_error = expect_pg_error_rolled_back(|| {
        Spi::run("SET LOCAL ec_distann.debug_fail_handoff_after_prepare = on").unwrap();
        stage_distann_physical_batch(&fixture, 0, &valid_digest, &valid_encoded);
    });
    assert!(
        prepared_error.contains("EC_FAULT_INJECTED"),
        "{prepared_error}"
    );
    assert_eq!(distann_physical_mutation_state(&fixture), baseline);

    let mut wrong_owner_fixture =
        create_distann_physical_generation_fixture("ec_distann_wrong_owner", 0x70);
    add_second_distann_owner(&mut wrong_owner_fixture);
    let (wrong_digest, wrong_encoded, wrong_vec_id) = (1_u8..=u8::MAX)
        .map(|marker| distann_stage_batch_fixture(&wrong_owner_fixture, 0, marker))
        .find(|(_, _, vec_id)| {
            crate::am::ec_distann::placement::owning_node(
                *vec_id,
                2,
                crate::am::ec_distann::DISTANN_PLACEMENT_HASH_VERSION,
            ) == 1
        })
        .unwrap();
    assert_ne!(wrong_vec_id, 0);
    begin_distann_physical_generation_count(
        &wrong_owner_fixture,
        1,
        &wrong_owner_fixture.expected_owner_digest,
    );
    let wrong_baseline = distann_physical_mutation_state(&wrong_owner_fixture);
    let wrong_error = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&wrong_owner_fixture, 0, &wrong_digest, &wrong_encoded);
    });
    assert!(wrong_error.contains("EC_WRONG_OWNER"), "{wrong_error}");
    assert_eq!(
        distann_physical_mutation_state(&wrong_owner_fixture),
        wrong_baseline
    );

    let duplicate_fixture =
        create_distann_physical_generation_fixture("ec_distann_duplicate_existing", 0x71);
    begin_distann_physical_generation_count(
        &duplicate_fixture,
        2,
        &duplicate_fixture.expected_owner_digest,
    );
    let (duplicate_digest, duplicate_encoded, duplicate_vec_id) =
        distann_stage_batch_fixture(&duplicate_fixture, 0, 0x72);
    let duplicate_relations = distann_generation_relation_oids(&duplicate_fixture);
    Spi::run(&format!(
        "INSERT INTO {} (vec_id, graph_record, row_tid)
         VALUES ({}, decode('00', 'hex'), '(0,1)'::tid)",
        canonical_index_locator(duplicate_relations.1),
        i64::from_le_bytes(duplicate_vec_id.to_le_bytes()),
    ))
    .unwrap();
    let duplicate_baseline = distann_physical_mutation_state(&duplicate_fixture);
    let duplicate_error = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&duplicate_fixture, 0, &duplicate_digest, &duplicate_encoded);
    });
    assert!(
        duplicate_error.contains("EC_DUPLICATE_VEC_ID"),
        "{duplicate_error}"
    );
    assert_eq!(
        distann_physical_mutation_state(&duplicate_fixture),
        duplicate_baseline
    );

    let corrupt_fixture =
        create_distann_physical_generation_fixture("ec_distann_corrupt_hash", 0x73);
    begin_distann_physical_generation_count(
        &corrupt_fixture,
        1,
        &corrupt_fixture.expected_owner_digest,
    );
    Spi::run(&format!(
        "UPDATE {} SET owner_stream_sha256_state =
             set_byte(
                 owner_stream_sha256_state,
                 3,
                 (get_byte(owner_stream_sha256_state, 3) + 1) % 256
             )
          WHERE index_oid = {} AND build_id = '{}'::uuid",
        distann_generation_catalog_name(),
        u32::from(corrupt_fixture.index_oid),
        corrupt_fixture.build_id,
    ))
    .unwrap();
    let corrupt_baseline = distann_physical_mutation_state(&corrupt_fixture);
    let (corrupt_digest, corrupt_encoded, _) =
        distann_stage_batch_fixture(&corrupt_fixture, 0, 0x74);
    let corrupt_error = expect_pg_error_rolled_back(|| {
        stage_distann_physical_batch(&corrupt_fixture, 0, &corrupt_digest, &corrupt_encoded);
    });
    assert!(
        corrupt_error.contains("state disagrees with cumulative digest"),
        "{corrupt_error}"
    );
    assert_eq!(
        distann_physical_mutation_state(&corrupt_fixture),
        corrupt_baseline
    );

    for (stem, remove_directory) in [
        ("ec_distann_missing_row", false),
        ("ec_distann_missing_directory", true),
    ] {
        let seal_fixture = create_distann_physical_generation_fixture(stem, 0x75);
        let (stage_digest, stage_encoded, _) = distann_stage_batch_fixture(&seal_fixture, 0, 0x76);
        let owner_digest = distann_owner_digest_for_batch(&seal_fixture, &stage_encoded);
        begin_distann_physical_generation_count(&seal_fixture, 1, &owner_digest);
        stage_distann_physical_batch(&seal_fixture, 0, &stage_digest, &stage_encoded);
        let relations = distann_generation_relation_oids(&seal_fixture);
        if remove_directory {
            let dummy_table = format!("{stem}_dummy_directory_source");
            let dummy_index = format!("{stem}_dummy_directory_idx");
            Spi::run(&format!(
                "CREATE TABLE {dummy_table}(vec_id bigint NOT NULL);
                 CREATE UNIQUE INDEX {dummy_index} ON {dummy_table}(vec_id);
                 UPDATE {} SET directory_relid = '{dummy_index}'::regclass::oid
                  WHERE index_oid = {} AND build_id = '{}'::uuid",
                distann_generation_catalog_name(),
                u32::from(seal_fixture.index_oid),
                seal_fixture.build_id,
            ))
            .unwrap();
        } else {
            Spi::run(&format!(
                "DELETE FROM {}",
                canonical_index_locator(relations.0)
            ))
            .unwrap();
        }
        let seal_baseline = distann_physical_mutation_state(&seal_fixture);
        let seal_error = expect_pg_error_rolled_back(|| {
            seal_distann_physical_generation(&seal_fixture, 1, &owner_digest);
        });
        assert!(
            seal_error.contains("EC_GENERATION_MISSING")
                || seal_error.contains("EC_BUILD_INCOMPLETE"),
            "{seal_error}"
        );
        assert_eq!(
            distann_physical_mutation_state(&seal_fixture),
            seal_baseline
        );
    }
}

#[pg_test]
fn test_distann_stage_type_io_runs_as_restricted_control_owner() {
    const CONTROL_OWNER: &str = "ec_distann_typeio_control_owner";
    const TRANSPORT_ROLE: &str = "ec_distann_typeio_transport";
    let test_schema = Spi::get_one::<String>("SELECT current_schema()::text")
        .unwrap()
        .unwrap();
    let quoted_test_schema = format!("\"{}\"", test_schema.replace('"', "\"\""));
    Spi::run(&format!(
        "CREATE ROLE {CONTROL_OWNER} NOLOGIN;
         CREATE ROLE {TRANSPORT_ROLE} NOLOGIN;
         CREATE TABLE ec_distann_typeio_canary(marker integer);
         REVOKE ALL ON ec_distann_typeio_canary FROM PUBLIC, {CONTROL_OWNER}, {TRANSPORT_ROLE};
         CREATE FUNCTION ec_distann_typeio_check(value text) RETURNS boolean
         LANGUAGE plpgsql SECURITY INVOKER AS $$
         BEGIN
             INSERT INTO {quoted_test_schema}.ec_distann_typeio_canary VALUES (1);
             RETURN true;
         EXCEPTION WHEN insufficient_privilege THEN
             RAISE EXCEPTION 'TYPE_IO_USER=%', current_user USING ERRCODE = '42501';
         END
         $$;
         ALTER FUNCTION ec_distann_typeio_check(text) OWNER TO {CONTROL_OWNER};
         CREATE DOMAIN ec_distann_typeio_payload AS text
             CHECK (ec_distann_typeio_check(VALUE));
         ALTER DOMAIN ec_distann_typeio_payload OWNER TO {CONTROL_OWNER};"
    ))
    .expect("hostile domain roles and canary should create");

    let fixture = create_distann_physical_generation_fixture_with_payload_type(
        "ec_distann_typeio",
        0x6d,
        "ec_distann_typeio_payload",
    );
    Spi::run(&format!(
        "ALTER TABLE ec_distann_typeio_source OWNER TO {CONTROL_OWNER};
         GRANT EXECUTE ON FUNCTION
             ec_distann_stage_epoch_batch(regclass, uuid, bigint, bytea, bytea)
             TO {TRANSPORT_ROLE};"
    ))
    .expect("control and transport privileges should configure");

    let control_owner_oid =
        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{CONTROL_OWNER}'::regrole::oid"))
            .unwrap()
            .unwrap();
    let endpoint_owner = Spi::get_one::<pg_sys::Oid>(
        "SELECT proowner FROM pg_proc
          WHERE oid = 'ec_distann_stage_epoch_batch(regclass,uuid,bigint,bytea,bytea)'::regprocedure",
    )
    .unwrap()
    .unwrap();
    assert_ne!(control_owner_oid, endpoint_owner);
    assert_eq!(
        Spi::get_one::<pg_sys::Oid>(&format!(
            "SELECT relowner FROM pg_class WHERE oid = {}",
            u32::from(fixture.index_oid)
        ))
        .unwrap(),
        Some(control_owner_oid)
    );
    assert_eq!(
        Spi::get_one::<bool>(&format!(
            "SELECT has_table_privilege('{CONTROL_OWNER}', 'ec_distann_typeio_canary', 'INSERT')
                 OR has_table_privilege('{TRANSPORT_ROLE}', 'ec_distann_typeio_canary', 'INSERT')"
        ))
        .unwrap(),
        Some(false)
    );

    let mut saved_user = pg_sys::InvalidOid;
    let mut saved_context = 0;
    unsafe { pg_sys::GetUserIdAndSecContext(&mut saved_user, &mut saved_context) };
    let saved_path = Spi::get_one::<String>("SELECT current_setting('search_path')")
        .unwrap()
        .unwrap();
    crate::am::ec_distann::with_restricted_type_io_owner(control_owner_oid, || {
        let mut user = pg_sys::InvalidOid;
        let mut context = 0;
        unsafe { pg_sys::GetUserIdAndSecContext(&mut user, &mut context) };
        assert_eq!(user, control_owner_oid);
        assert_ne!(context & pg_sys::SECURITY_RESTRICTED_OPERATION as i32, 0);
    });
    let ordinary_error: Result<(), &'static str> =
        crate::am::ec_distann::with_restricted_type_io_owner(control_owner_oid, || Err("stop"));
    assert_eq!(ordinary_error, Err("stop"));
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::am::ec_distann::with_restricted_type_io_owner(control_owner_oid, || {
            panic!("type I/O panic restoration")
        })
    }));
    assert!(panic.is_err());
    let nested_error = expect_pg_error(|| {
        crate::am::ec_distann::with_restricted_type_io_owner(control_owner_oid, || {
            pgrx::error!("type I/O PostgreSQL error restoration")
        })
    });
    assert!(nested_error.contains("type I/O PostgreSQL error restoration"));
    let mut restored_user = pg_sys::InvalidOid;
    let mut restored_context = 0;
    unsafe { pg_sys::GetUserIdAndSecContext(&mut restored_user, &mut restored_context) };
    assert_eq!(
        (restored_user, restored_context),
        (saved_user, saved_context)
    );
    assert_eq!(
        Spi::get_one::<String>("SELECT current_setting('search_path')")
            .unwrap()
            .unwrap(),
        saved_path
    );

    let (batch_digest, encoded_batch, _) = distann_stage_batch_fixture(&fixture, 0, 0x6e);
    let owner_digest = distann_owner_digest_for_batch(&fixture, &encoded_batch);
    begin_distann_physical_generation_count(&fixture, 1, &owner_digest);
    Spi::run(&format!("SET LOCAL ROLE {TRANSPORT_ROLE}")).expect("transport role should activate");
    let error = expect_pg_error_rolled_back(|| {
        Spi::connect(|client| {
            client
                .select(
                    "SELECT * FROM ec_distann_stage_epoch_batch(
                         $1::regclass, $2::uuid, 0, $3::bytea, $4::bytea
                     )",
                    None,
                    &[
                        fixture.index_oid.into(),
                        fixture.build_id.into(),
                        batch_digest.clone().into(),
                        encoded_batch.clone().into(),
                    ],
                )
                .expect("hostile domain receive must error");
        });
    });
    Spi::run("RESET ROLE").expect("transport role should reset");
    assert!(
        error.contains(&format!("TYPE_IO_USER={CONTROL_OWNER}")),
        "type I/O ran under the wrong identity: {error}"
    );
    assert_eq!(
        Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM {quoted_test_schema}.ec_distann_typeio_canary"
        ))
        .unwrap(),
        Some(0)
    );
    assert_eq!(
        Spi::get_one::<bool>(&format!(
            "SELECT next_batch_seq = 0 AND cumulative_record_count = 0
               AND NOT EXISTS (
                   SELECT 1 FROM {} b
                    WHERE b.index_oid = g.index_oid
                      AND b.logical_index_uuid = g.logical_index_uuid
                      AND b.build_id = g.build_id
               )
              FROM {} g
             WHERE g.index_oid = {} AND g.build_id = '{}'::uuid",
            distann_catalog_name("ec_distann_generation_batch"),
            distann_generation_catalog_name(),
            u32::from(fixture.index_oid),
            fixture.build_id,
        ))
        .unwrap(),
        Some(true)
    );
}

#[pg_test]
fn test_distann_seal_ready_replay_and_receipt() {
    let fixture = create_distann_physical_generation_fixture("ec_distann_seal_ready", 0x3a);
    let (batch_digest, encoded_batch, vec_id) = distann_stage_batch_fixture(&fixture, 0, 0x76);
    let owner_digest = distann_owner_digest_for_batch(&fixture, &encoded_batch);
    begin_distann_physical_generation_count(&fixture, 1, &owner_digest);
    let staged = stage_distann_physical_batch(&fixture, 0, &batch_digest, &encoded_batch);
    assert_eq!((staged.0, staged.1), (1, 1));
    assert_eq!(staged.2, owner_digest);

    let wrong_expectation = expect_pg_error_rolled_back(|| {
        seal_distann_physical_generation(&fixture, 2, &owner_digest);
    });
    assert!(
        wrong_expectation.contains("EC_BUILD_INCOMPLETE"),
        "unexpected seal expectation error: {wrong_expectation}"
    );
    let generation_catalog = distann_generation_catalog_name();
    assert_eq!(
        Spi::get_one::<String>(&format!(
            "SELECT state FROM {generation_catalog}
              WHERE index_oid = {} AND logical_index_uuid = '{}'::uuid
                AND build_id = '{}'::uuid",
            u32::from(fixture.index_oid),
            fixture.logical_index_uuid,
            fixture.build_id,
        ))
        .unwrap()
        .as_deref(),
        Some("Building"),
        "failed seal must leave the generation Building"
    );

    let encoded_receipt = seal_distann_physical_generation(&fixture, 1, &owner_digest);
    assert_eq!(
        encoded_receipt.len(),
        crate::am::ec_distann::DISTANN_READY_RECEIPT_BYTES
    );
    let receipt = crate::am::ec_distann::DistannReadyReceipt::decode(&encoded_receipt)
        .expect("Ready receipt should decode");
    assert_eq!(receipt.node_id, 17);
    assert_eq!(receipt.epoch, 7);
    assert_eq!(receipt.build_id, *fixture.build_id.as_bytes());
    assert_eq!(
        receipt.build_spec_digest.to_vec(),
        fixture.build_spec_digest
    );
    assert_eq!(
        receipt.generation_descriptor_digest.to_vec(),
        fixture.descriptor_digest
    );
    assert_eq!(receipt.last_acknowledged_batch_sequence, 0);
    assert_eq!((receipt.owned_record_count, receipt.row_count), (1, 1));
    assert_eq!(receipt.owner_stream_digest.to_vec(), owner_digest);
    assert_eq!(
        receipt.state,
        crate::am::ec_distann::DISTANN_READY_RECEIPT_STATE
    );
    assert!(receipt.graph_bytes > 0);
    assert!(receipt.row_tier_bytes > 0);
    assert!(receipt.directory_bytes > 0);
    assert_ne!(receipt.persisted_graph_digest, [0; 32]);
    assert_ne!(receipt.persisted_row_tier_digest, [0; 32]);
    assert_ne!(receipt.local_directory_digest, [0; 32]);

    let persisted = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT state, ready_receipt FROM {generation_catalog}
                      WHERE index_oid = $1::oid
                        AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid"
                ),
                None,
                &[
                    fixture.index_oid.into(),
                    fixture.logical_index_uuid.into(),
                    fixture.build_id.into(),
                ],
            )
            .unwrap()
            .map(|row| {
                (
                    row["state"].value::<String>().unwrap().unwrap(),
                    row["ready_receipt"].value::<Vec<u8>>().unwrap().unwrap(),
                )
            })
            .next()
            .expect("Ready generation should exist")
    });
    assert_eq!(persisted.0, "Ready");
    assert_eq!(persisted.1, encoded_receipt);

    let seal_replay = seal_distann_physical_generation(&fixture, 1, &owner_digest);
    assert_eq!(seal_replay, encoded_receipt, "seal replay must be exact");
    let stage_replay = stage_distann_physical_batch(&fixture, 0, &batch_digest, &encoded_batch);
    assert_eq!(
        stage_replay, staged,
        "acknowledged stage replay survives Ready"
    );

    let relations = distann_generation_relation_oids(&fixture);
    let graph_relation = canonical_index_locator(relations.1);
    assert_eq!(
        Spi::get_one::<i64>(&format!(
            "SELECT count(*) FROM {graph_relation} WHERE vec_id = {}",
            i64::from_le_bytes(vec_id.to_le_bytes())
        ))
        .unwrap(),
        Some(1),
        "Ready replays must not duplicate physical records"
    );

    let empty_fixture = create_distann_physical_generation_fixture("ec_distann_seal_empty", 0x3b);
    let (empty_batch_digest, empty_batch) = distann_empty_stage_batch_fixture(&empty_fixture);
    let empty_owner_digest = distann_owner_digest_for_batch(&empty_fixture, &empty_batch);
    begin_distann_physical_generation_count(&empty_fixture, 0, &empty_owner_digest);
    let empty_stage =
        stage_distann_physical_batch(&empty_fixture, 0, &empty_batch_digest, &empty_batch);
    assert_eq!((empty_stage.0, empty_stage.1), (0, 0));
    assert_eq!(empty_stage.2, empty_owner_digest);
    let empty_receipt = crate::am::ec_distann::DistannReadyReceipt::decode(
        &seal_distann_physical_generation(&empty_fixture, 0, &empty_owner_digest),
    )
    .expect("empty Ready receipt should decode");
    assert_eq!(
        (empty_receipt.owned_record_count, empty_receipt.row_count),
        (0, 0)
    );
    assert_eq!(
        empty_receipt.owner_stream_digest.to_vec(),
        empty_owner_digest
    );
}

#[pg_test]
fn test_distann_source_capture_spools_complete_frozen_rows() {
    unsafe { crate::am::ec_distann::test_physical_capture_dead_callback_does_not_access_datums() };
    Spi::run(
        "CREATE TABLE ec_distann_source_capture_source (
             source_id uuid NOT NULL,
             payload text,
             legacy_payload integer,
             embedding ecvector(4) NOT NULL,
             payload_generated text GENERATED ALWAYS AS (payload || ':generated') STORED
         );
         ALTER TABLE ec_distann_source_capture_source DROP COLUMN legacy_payload;
         INSERT INTO ec_distann_source_capture_source
             (source_id, payload, embedding)
         VALUES
             ('11111111-1111-4111-8111-111111111111', repeat('x', 20000),
              encode_to_ecvector(ARRAY[1.0,0.0,0.0,0.0], 4, 42)),
             ('22222222-2222-4222-8222-222222222222', NULL,
              encode_to_ecvector(ARRAY[0.0,1.0,0.0,0.0], 4, 42)),
             ('33333333-3333-4333-8333-333333333333', 'deleted before snapshot',
              encode_to_ecvector(ARRAY[0.0,0.0,1.0,0.0], 4, 42)),
             ('44444444-4444-4444-8444-444444444444', 'hot-before',
              encode_to_ecvector(ARRAY[0.0,0.0,0.0,1.0], 4, 42));
         UPDATE ec_distann_source_capture_source
            SET payload = 'hot-after'
          WHERE source_id = '44444444-4444-4444-8444-444444444444';
         DELETE FROM ec_distann_source_capture_source
          WHERE source_id = '33333333-3333-4333-8333-333333333333';
         CREATE INDEX ec_distann_source_capture_idx
           ON ec_distann_source_capture_source
           USING ec_distann (embedding ecvector_distann_ip_ops)
           INCLUDE (source_id)
           WITH (
               distributed_control = true,
               source_identity = 'include',
               graph_degree = 4,
               neighbor_code_format = 'rabitq'
           )",
    )
    .unwrap();
    let index_oid =
        Spi::get_one::<pg_sys::Oid>("SELECT 'ec_distann_source_capture_idx'::regclass::oid")
            .unwrap()
            .unwrap();

    let index = IndexRelationGuard::open(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "source capture test",
    );
    let heap = crate::storage::relation_guard::HeapRelationGuard::try_access_share(
        index.heap_relation_oid(),
    )
    .expect("source heap should open");
    let index_info =
        crate::am::common::index_info::IndexInfoGuard::build(index.as_ptr(), "source capture test");
    let mut capture = unsafe {
        crate::am::ec_distann::capture_physical_source_rows(
            heap.as_ptr(),
            index.as_ptr(),
            index_info.as_ptr(),
        )
    }
    .expect("physical source capture should succeed");
    assert_eq!(capture.len(), 3);
    assert_eq!(capture.dimensions(), 4);

    capture
        .preflight_handoff_entries(crate::am::ec_distann::DistannHandoffShape {
            code_stride: 1,
            graph_degree: 4,
            non_dropped_attribute_count: 4,
        })
        .expect("every complete eventual entry should fit the handoff bound");

    let mut seen_toasted = false;
    let mut seen_nulls = false;
    let mut seen_hot = false;
    for node in 0..capture.len() {
        let heap_tid = capture.rows()[node].heap_tid();
        let vector = capture.rows()[node].source_vector().to_vec();
        let identity = capture.rows()[node].identity_payload();
        assert_ne!(heap_tid.offset_number, 0);
        let payload = capture
            .payload_for_node(node)
            .expect("spooled source payload should read back");
        assert_eq!(payload.source_identity, identity);
        assert_eq!(
            payload.vec_id,
            crate::am::ec_distann::vec_id_from_source_identity(&identity)
        );
        if identity[0] == 0x11 {
            assert_eq!(vector, vec![1.0, 0.0, 0.0, 0.0]);
            assert_eq!(payload.row_null_bitmap, vec![0]);
            assert_eq!(payload.row_values.len(), 4);
            assert_eq!(payload.row_values[1].len(), 20_000);
            assert!(payload.row_values[1].iter().all(|byte| *byte == b'x'));
            assert_eq!(payload.row_values[3].len(), 20_010);
            assert!(payload.row_values[3].ends_with(b":generated"));
            seen_toasted = true;
        } else if identity[0] == 0x22 {
            assert_eq!(identity[0], 0x22);
            assert_eq!(vector, vec![0.0, 1.0, 0.0, 0.0]);
            assert_eq!(payload.row_null_bitmap, vec![0b0000_1010]);
            assert_eq!(payload.row_values.len(), 2);
            seen_nulls = true;
        } else {
            assert_eq!(identity[0], 0x44);
            assert_eq!(vector, vec![0.0, 0.0, 0.0, 1.0]);
            assert_eq!(payload.row_null_bitmap, vec![0]);
            assert_eq!(payload.row_values[1], b"hot-after");
            assert_eq!(payload.row_values[3], b"hot-after:generated");
            seen_hot = true;
        }
    }
    assert!(seen_toasted && seen_nulls && seen_hot);

    let mut workspace =
        crate::am::ec_distann::build_physical_graph_workspace(index.as_ptr(), capture)
            .expect("physical graph workspace should build from the frozen capture");
    assert_eq!(workspace.record_count(), 3);
    assert_eq!(workspace.shape().non_dropped_attribute_count, 4);
    assert_eq!(workspace.codec_artifact().dimensions(), 4);
    let metadata = unsafe { crate::am::ec_distann::read_metadata_from_index(index.as_ptr()) }
        .expect("control metadata should decode");
    let roster = vec![crate::am::ec_distann::DistannRosterEntry {
        node_id: 17,
        logical_index_uuid: metadata.logical_index_uuid,
        endpoint_identity: "capture/node-17".to_owned(),
    }];
    let expectations = workspace
        .owner_expectations(
            &roster,
            crate::am::ec_distann::DISTANN_PLACEMENT_HASH_VERSION,
        )
        .expect("owner expectations should stream from the workspace");
    assert_eq!(expectations[0].expected_count, 3);
    let mut build_id = [0x3c; 16];
    build_id[6] = 0x4c;
    build_id[8] = 0xbc;
    let schema_fingerprint =
        crate::am::ec_distann::resolve_relation_schema(index.heap_relation_oid())
            .unwrap()
            .descriptor
            .fingerprint()
            .unwrap();
    let route_identity = crate::am::ec_distann::DistannHandoffRouteIdentity {
        epoch: 7,
        build_id,
        build_spec_digest: [0x22; 32],
        row_schema_fingerprint: schema_fingerprint,
        index_format_version: crate::am::ec_distann::DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
        neighbor_codec_kind: metadata.neighbor_codec_kind,
        placement_hash_version: crate::am::ec_distann::DISTANN_PLACEMENT_HASH_VERSION,
    };
    let mut routed_entries = Vec::new();
    let route_shape = workspace.shape();
    let summaries = workspace
        .route(
            route_identity,
            1,
            &mut |owner, sequence, digest, encoded| {
                assert_eq!(owner, 0);
                assert_eq!(sequence, 0);
                let batch =
                    crate::am::ec_distann::DistannHandoffBatch::decode(encoded, route_shape)
                        .unwrap();
                assert_eq!(batch.digest(route_shape).unwrap(), *digest);
                routed_entries.extend(batch.entries.clone());
                Ok(crate::am::ec_distann::DistannStageAck {
                    accepted_record_count: batch.entries.len() as u64,
                    cumulative_record_count: routed_entries.len() as u64,
                    cumulative_owner_digest: crate::am::ec_distann::owner_stream_digest(
                        &routed_entries,
                        route_shape,
                    )
                    .unwrap(),
                })
            },
        )
        .expect("workspace entries should route through one bounded owner batch");
    assert_eq!(summaries[0].record_count, 3);
    assert_eq!(
        summaries[0].owner_stream_digest,
        expectations[0].expected_owner_digest
    );
}

#[pg_test]
fn test_distann_source_capture_mismatch_faults() {
    Spi::run(
        "CREATE TABLE ec_distann_capture_fault_source (
             source_id uuid NOT NULL,
             embedding ecvector(4) NOT NULL
         );
         INSERT INTO ec_distann_capture_fault_source VALUES (
             '55555555-5555-4555-8555-555555555555',
             encode_to_ecvector(ARRAY[1.0,0.0,0.0,0.0], 4, 42)
         );
         CREATE INDEX ec_distann_capture_fault_idx
           ON ec_distann_capture_fault_source
           USING ec_distann (embedding ecvector_distann_ip_ops)
           INCLUDE (source_id)
           WITH (distributed_control = true, source_identity = 'include')",
    )
    .unwrap();
    let index_oid =
        Spi::get_one::<pg_sys::Oid>("SELECT 'ec_distann_capture_fault_idx'::regclass::oid")
            .unwrap()
            .unwrap();
    for (fault, expected) in [
        (1, "no visible row"),
        (2, "vector differs"),
        (3, "identity differs"),
    ] {
        let error = expect_pg_error_rolled_back(|| {
            Spi::run(&format!(
                "SET LOCAL ec_distann.debug_source_capture_fault = {fault}"
            ))
            .unwrap();
            let index = IndexRelationGuard::open(
                index_oid,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                "source capture fault test",
            );
            let heap = crate::storage::relation_guard::HeapRelationGuard::try_access_share(
                index.heap_relation_oid(),
            )
            .unwrap();
            let index_info = crate::am::common::index_info::IndexInfoGuard::build(
                index.as_ptr(),
                "source capture fault test",
            );
            unsafe {
                crate::am::ec_distann::capture_physical_source_rows(
                    heap.as_ptr(),
                    index.as_ptr(),
                    index_info.as_ptr(),
                )
            }
            .expect("faulted capture must error");
        });
        assert!(error.contains(expected), "fault {fault}: {error}");
        assert_eq!(
            Spi::get_one::<i64>("SELECT count(*) FROM ec_distann_capture_fault_source").unwrap(),
            Some(1)
        );
    }
}

#[pg_test]
fn test_distann_legacy_build_as_unprivileged_table_owner() {
    const ROLE: &str = "ec_distann_legacy_owner";
    const SCHEMA: &str = "ec_distann_legacy_owner_schema";
    let extension_schema = Spi::get_one::<String>(
        "SELECT pg_catalog.quote_ident(n.nspname)
           FROM pg_catalog.pg_extension e
           JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace
          WHERE e.extname = 'ecaz'",
    )
    .unwrap()
    .unwrap();
    Spi::run(&format!(
        "CREATE ROLE {ROLE} NOLOGIN;
         CREATE SCHEMA {SCHEMA} AUTHORIZATION {ROLE};
         GRANT USAGE ON SCHEMA {extension_schema} TO {ROLE};
         SET LOCAL ROLE {ROLE};
         CREATE TABLE {SCHEMA}.source (
             id bigint, embedding {extension_schema}.ecvector(4)
         );
         CREATE INDEX source_idx ON {SCHEMA}.source
             USING ec_distann (embedding {extension_schema}.ecvector_distann_ip_ops);
         RESET ROLE"
    ))
    .expect("legacy index creation must not touch revoked generation catalogs");
    let endpoint_denial = expect_pg_error_rolled_back(|| {
        Spi::run(&format!(
            "SET LOCAL ROLE {ROLE};
             SELECT {extension_schema}.ec_distann_list_unpublished_generations(
                 '{SCHEMA}.source_idx'::regclass
             )"
        ))
        .expect("unprivileged internal endpoint call must fail");
    });
    assert!(
        endpoint_denial.contains("permission denied")
            && endpoint_denial.contains("ec_distann_list_unpublished_generations"),
        "unexpected internal endpoint ACL error: {endpoint_denial}"
    );
    Spi::run(&format!("DROP SCHEMA {SCHEMA} CASCADE; DROP ROLE {ROLE}")).unwrap();
}

#[pg_test]
fn test_distann_generation_creation_rolls_back_whole_transaction() {
    let fixture =
        create_distann_physical_generation_fixture("ec_distann_generation_rollback", 0x41);
    let error = expect_pg_error_rolled_back(|| {
        begin_distann_physical_generation(&fixture, &fixture.expected_owner_digest);
        let relations = distann_generation_relation_oids(&fixture);
        assert!(relations.0 != pg_sys::InvalidOid);
        assert!(relations.1 != pg_sys::InvalidOid);
        assert!(relations.2 != pg_sys::InvalidOid);
        pgrx::error!("EC_TEST_ROLLBACK: deliberate generation transaction abort");
    });
    assert!(
        error.contains("EC_TEST_ROLLBACK"),
        "unexpected rollback trigger error: {error}"
    );
    let catalog_count = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM {} WHERE index_oid = {}",
        distann_generation_catalog_name(),
        u32::from(fixture.index_oid)
    ))
    .unwrap()
    .unwrap();
    assert_eq!(catalog_count, 0);
    let name_prefix_count = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM pg_class
          WHERE relname LIKE '_ecdz_%_{}_{}'",
        u32::from(fixture.index_oid),
        hex::encode(fixture.build_id.as_bytes()),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(name_prefix_count, 0, "DDL must roll back with the batch");
}

#[pg_test]
fn test_distann_generation_drop_and_reindex_clean_dependencies() {
    let drop_fixture =
        create_distann_physical_generation_fixture("ec_distann_generation_drop", 0x51);
    begin_distann_physical_generation(&drop_fixture, &drop_fixture.expected_owner_digest);
    let dropped_relations = distann_generation_relation_oids(&drop_fixture);
    Spi::run(&format!("DROP INDEX {}", drop_fixture.index_name)).unwrap();
    let after_drop_relations = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM pg_class WHERE oid IN ({}, {}, {})",
        u32::from(dropped_relations.0),
        u32::from(dropped_relations.1),
        u32::from(dropped_relations.2),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(after_drop_relations, 0);
    let after_drop_catalog = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM {} WHERE index_oid = {}",
        distann_generation_catalog_name(),
        u32::from(drop_fixture.index_oid)
    ))
    .unwrap()
    .unwrap();
    assert_eq!(after_drop_catalog, 0);

    let reindex_fixture =
        create_distann_physical_generation_fixture("ec_distann_generation_reindex", 0x61);
    begin_distann_physical_generation(&reindex_fixture, &reindex_fixture.expected_owner_digest);
    let old_relations = distann_generation_relation_oids(&reindex_fixture);
    let old_uuid = reindex_fixture.logical_index_uuid;
    Spi::run(&format!("REINDEX INDEX {}", reindex_fixture.index_name)).unwrap();
    let new_uuid = Spi::get_one::<pgrx::datum::Uuid>(&format!(
        "SELECT logical_index_uuid
           FROM ec_distann_control_identity('{}'::regclass)",
        reindex_fixture.index_name
    ))
    .unwrap()
    .unwrap();
    assert_ne!(new_uuid, old_uuid, "control REINDEX must mint a fresh UUID");
    let old_relation_count = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM pg_class WHERE oid IN ({}, {}, {})",
        u32::from(old_relations.0),
        u32::from(old_relations.1),
        u32::from(old_relations.2),
    ))
    .unwrap()
    .unwrap();
    assert_eq!(old_relation_count, 0);
    let old_catalog_count = Spi::get_one::<i64>(&format!(
        "SELECT count(*) FROM {} WHERE index_oid = {}",
        distann_generation_catalog_name(),
        u32::from(reindex_fixture.index_oid)
    ))
    .unwrap()
    .unwrap();
    assert_eq!(old_catalog_count, 0);
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
