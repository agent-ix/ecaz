// TC-040 (Task 179 node-registry concurrency).
//
// These fixtures are created through a loopback connection because a pg_test's
// outer transaction is not visible to the independent PostgreSQL backends used
// to prove lock waiting and stronger-isolation behavior.

const DISTANN_REGISTRY_CONCURRENCY_SECRET_NAME: &str = "DISTANN_REGISTRY_CONCURRENCY";
const DISTANN_REGISTRY_CONCURRENCY_SECRET_KEY: &str =
    "EC_SPIRE_REMOTE_CONNINFO_DISTANN_REGISTRY_CONCURRENCY";

struct DistannRegistryConcurrencyFixture {
    schema: &'static str,
    extension_schema: String,
    coordinator_index: String,
    participant_one_index: String,
    participant_two_index: String,
    participant_two_uuid: String,
}

fn distann_registry_concurrency_connect(
    conninfo: &str,
    application_name: &str,
) -> postgres::Client {
    postgres::Client::connect(
        &format!("{conninfo} application_name={application_name}"),
        postgres::NoTls,
    )
    .expect("DistANN registry concurrency client should connect")
}

fn distann_registry_concurrency_install_secret(client: &mut postgres::Client) {
    client
        .execute(
            "SELECT tests.ec_spire_test_set_env_var($1::text, $2::text)",
            &[
                &DISTANN_REGISTRY_CONCURRENCY_SECRET_KEY,
                &"unused-local-conninfo",
            ],
        )
        .expect("loopback backend should receive the DistANN registry secret");
}

fn distann_registry_concurrency_register(
    client: &mut postgres::Client,
    fixture: &DistannRegistryConcurrencyFixture,
    roster_ordinal: i32,
    node_id: i32,
    endpoint_identity: &str,
    participant_index: &str,
) -> Result<u64, postgres::Error> {
    client.execute(
        &format!(
            "SELECT {}.ec_distann_register_node_descriptor(
                 $1::text::regclass, $2::integer, $3::integer, $4::text,
                 $5::text, $6::text, true
             )",
            fixture.extension_schema
        ),
        &[
            &fixture.coordinator_index,
            &roster_ordinal,
            &node_id,
            &endpoint_identity,
            &DISTANN_REGISTRY_CONCURRENCY_SECRET_NAME,
            &participant_index,
        ],
    )
}

fn distann_registry_concurrency_unregister(
    client: &mut postgres::Client,
    fixture: &DistannRegistryConcurrencyFixture,
    roster_ordinal: i32,
) -> Result<u64, postgres::Error> {
    client.execute(
        &format!(
            "SELECT {}.ec_distann_unregister_node_descriptor(
                 $1::text::regclass, $2::integer
             )",
            fixture.extension_schema
        ),
        &[&fixture.coordinator_index, &roster_ordinal],
    )
}

fn distann_registry_concurrency_setup(
    client: &mut postgres::Client,
    schema: &'static str,
) -> DistannRegistryConcurrencyFixture {
    assert!(
        schema
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_'),
        "fixture schema must be a trusted canonical identifier"
    );
    let extension_schema = client
        .query_one(
            "SELECT pg_catalog.quote_ident(n.nspname)
               FROM pg_catalog.pg_extension e
               JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace
              WHERE e.extname = 'ecaz'",
            &[],
        )
        .expect("extension schema query should succeed")
        .try_get::<_, String>(0)
        .expect("extension schema should decode");
    client
        .batch_execute(&format!(
            "DROP SCHEMA IF EXISTS {schema} CASCADE;
             CREATE SCHEMA {schema};
             SET search_path TO {extension_schema}, pg_catalog;

             CREATE TABLE {schema}.coordinator_source (
                 source_id uuid NOT NULL,
                 embedding ecvector(4) NOT NULL
             );
             CREATE INDEX coordinator_idx
               ON {schema}.coordinator_source
               USING ec_distann (embedding ecvector_distann_ip_ops)
               INCLUDE (source_id)
               WITH (
                   distributed_control = true,
                   source_identity = 'include',
                   graph_degree = 4,
                   neighbor_code_format = 'rabitq'
               );

             CREATE TABLE {schema}.participant_one_source (
                 source_id uuid NOT NULL,
                 embedding ecvector(4) NOT NULL
             );
             CREATE INDEX participant_one_idx
               ON {schema}.participant_one_source
               USING ec_distann (embedding ecvector_distann_ip_ops)
               INCLUDE (source_id)
               WITH (
                   distributed_control = true,
                   source_identity = 'include',
                   graph_degree = 4,
                   neighbor_code_format = 'rabitq'
               );

             CREATE TABLE {schema}.participant_two_source (
                 source_id uuid NOT NULL,
                 embedding ecvector(4) NOT NULL
             );
             CREATE INDEX participant_two_idx
               ON {schema}.participant_two_source
               USING ec_distann (embedding ecvector_distann_ip_ops)
               INCLUDE (source_id)
               WITH (
                   distributed_control = true,
                   source_identity = 'include',
                   graph_degree = 4,
                   neighbor_code_format = 'rabitq'
               )"
        ))
        .expect("DistANN registry concurrency fixture should create");

    let coordinator_index = format!("{schema}.coordinator_idx");
    let participant_one_index = format!("{schema}.participant_one_idx");
    let participant_two_index = format!("{schema}.participant_two_idx");
    for (index, endpoint) in [
        (participant_one_index.as_str(), "registry-race/p1"),
        (participant_two_index.as_str(), "registry-race/p2"),
    ] {
        client
            .execute(
                &format!(
                    "SELECT {extension_schema}.ec_distann_configure_participant_identity(
                         $1::text::regclass, $2::text
                     )"
                ),
                &[&index, &endpoint],
            )
            .expect("participant endpoint identity should configure");
    }
    let participant_two_uuid = client
        .query_one(
            &format!(
                "SELECT logical_index_uuid::text
                   FROM {extension_schema}.ec_distann_control_identity(
                       $1::text::regclass
                   )"
            ),
            &[&participant_two_index],
        )
        .expect("participant-two identity query should succeed")
        .try_get::<_, String>(0)
        .expect("participant-two UUID should decode");

    let fixture = DistannRegistryConcurrencyFixture {
        schema,
        extension_schema,
        coordinator_index,
        participant_one_index,
        participant_two_index,
        participant_two_uuid,
    };
    distann_registry_concurrency_install_secret(client);
    distann_registry_concurrency_register(
        client,
        &fixture,
        0,
        17,
        "registry-race/p1",
        &fixture.participant_one_index,
    )
    .expect("initial desired participant should register");
    fixture
}

fn distann_registry_concurrency_cleanup(
    client: &mut postgres::Client,
    fixture: &DistannRegistryConcurrencyFixture,
) {
    client
        .batch_execute(&format!("DROP SCHEMA {} CASCADE", fixture.schema))
        .expect("DistANN registry concurrency fixture should clean up");
}

fn distann_registry_concurrency_wait_for_blocker(
    monitor: &mut postgres::Client,
    blocker_pid: i32,
    waiter_application_name: &str,
) -> bool {
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(5) {
        let blocked = monitor
            .query_one(
                "SELECT COALESCE(
                            bool_or(
                                wait_event_type = 'Lock'
                                AND $1::integer = ANY(
                                    pg_catalog.pg_blocking_pids(pid)
                                )
                            ),
                            false
                        )
                   FROM pg_catalog.pg_stat_activity
                  WHERE application_name = $2::text",
                &[&blocker_pid, &waiter_application_name],
            )
            .expect("registry waiter activity probe should succeed")
            .try_get::<_, bool>(0)
            .expect("registry waiter activity probe should decode");
        if blocked {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    false
}

fn distann_registry_concurrency_state(
    client: &mut postgres::Client,
    fixture: &DistannRegistryConcurrencyFixture,
) -> (i64, i32, String, i64) {
    let row = client
        .query_one(
            &format!(
                "SELECT rs.revision,
                        nd.roster_ordinal,
                        nd.participant_logical_index_uuid::text,
                        count(*) OVER ()::bigint
                   FROM {}.ec_distann_registry_state rs
                   JOIN {}.ec_distann_node_descriptor nd
                     USING (index_oid, logical_index_uuid)
                  WHERE rs.index_oid = $1::text::regclass",
                fixture.extension_schema, fixture.extension_schema
            ),
            &[&fixture.coordinator_index],
        )
        .expect("registry state query should return one desired participant");
    (
        row.try_get::<_, i64>(0)
            .expect("registry revision should decode"),
        row.try_get::<_, i32>(1)
            .expect("roster ordinal should decode"),
        row.try_get::<_, String>(2)
            .expect("participant UUID should decode"),
        row.try_get::<_, i64>(3)
            .expect("desired participant count should decode"),
    )
}

#[pg_test]
fn test_distann_registry_atomic_replacement_no_deadlock() {
    const SCHEMA: &str = "ec_distann_registry_deadlock";
    const WAITER_APPLICATION: &str = "ec_distann_registry_replace_waiter";
    let conninfo = current_pg_test_loopback_conninfo();
    let mut monitor =
        distann_registry_concurrency_connect(&conninfo, "ec_distann_registry_replace_monitor");
    let fixture = distann_registry_concurrency_setup(&mut monitor, SCHEMA);

    let mut winner =
        distann_registry_concurrency_connect(&conninfo, "ec_distann_registry_replace_winner");
    distann_registry_concurrency_install_secret(&mut winner);
    winner
        .batch_execute(
            "SET deadlock_timeout = '100ms';
             SET statement_timeout = '10s';
             BEGIN",
        )
        .expect("replacement winner transaction should begin");
    let winner_pid = winner
        .query_one("SELECT pg_catalog.pg_backend_pid()", &[])
        .expect("replacement winner pid query should succeed")
        .try_get::<_, i32>(0)
        .expect("replacement winner pid should decode");
    distann_registry_concurrency_unregister(&mut winner, &fixture, 0)
        .expect("replacement winner should remove the old desired participant");

    let waiter_conninfo = conninfo.clone();
    let waiter_extension_schema = fixture.extension_schema.clone();
    let waiter_coordinator_index = fixture.coordinator_index.clone();
    let waiter_participant_index = fixture.participant_two_index.clone();
    let waiter = std::thread::spawn(move || {
        let mut client = distann_registry_concurrency_connect(&waiter_conninfo, WAITER_APPLICATION);
        distann_registry_concurrency_install_secret(&mut client);
        client
            .batch_execute(
                "SET deadlock_timeout = '100ms';
                 SET lock_timeout = '5s';
                 SET statement_timeout = '10s'",
            )
            .expect("replacement waiter timeouts should configure");
        let result = client.execute(
            &format!(
                "SELECT {waiter_extension_schema}.ec_distann_register_node_descriptor(
                     $1::text::regclass, 1, 19, 'registry-race/p2',
                     $2::text, $3::text, true
                 )"
            ),
            &[
                &waiter_coordinator_index,
                &DISTANN_REGISTRY_CONCURRENCY_SECRET_NAME,
                &waiter_participant_index,
            ],
        );
        match result {
            Ok(_) => (None, "ok".to_owned()),
            Err(error) => {
                let db_error = error.as_db_error();
                (
                    db_error.map(|error| error.code().code().to_owned()),
                    db_error
                        .map(|error| error.message().to_owned())
                        .unwrap_or_else(|| error.to_string()),
                )
            }
        }
    });

    if !distann_registry_concurrency_wait_for_blocker(&mut monitor, winner_pid, WAITER_APPLICATION)
    {
        let _ = winner.batch_execute("ROLLBACK");
        let waiter_result = waiter.join().expect("replacement waiter should join");
        distann_registry_concurrency_cleanup(&mut monitor, &fixture);
        panic!("replacement waiter did not block behind the winner: {waiter_result:?}");
    }

    if let Err(error) = distann_registry_concurrency_register(
        &mut winner,
        &fixture,
        0,
        18,
        "registry-race/p2",
        &fixture.participant_two_index,
    ) {
        let message = error.to_string();
        let _ = winner.batch_execute("ROLLBACK");
        let waiter_result = waiter.join().expect("replacement waiter should join");
        distann_registry_concurrency_cleanup(&mut monitor, &fixture);
        panic!(
            "replacement register failed while its transaction owned the registry lock: \
             {message}; waiter={waiter_result:?}"
        );
    }
    winner
        .batch_execute("COMMIT")
        .expect("atomic desired-roster replacement should commit");
    let waiter_result = waiter.join().expect("replacement waiter should join");
    let final_state = distann_registry_concurrency_state(&mut monitor, &fixture);
    distann_registry_concurrency_cleanup(&mut monitor, &fixture);

    assert!(
        waiter_result.0.is_some(),
        "waiter unexpectedly registered a second copy of P2"
    );
    assert!(
        waiter_result
            .1
            .contains("participant logical UUID already exists"),
        "waiter did not observe the committed replacement roster: {waiter_result:?}"
    );
    assert!(
        !matches!(
            waiter_result.0.as_deref(),
            Some("40P01") | Some("55P03") | Some("57014")
        ),
        "waiter deadlocked or timed out: {waiter_result:?}"
    );
    assert_eq!(
        final_state,
        (3, 0, fixture.participant_two_uuid.clone(), 1),
        "the committed desired roster must contain exactly P2 at ordinal zero"
    );
}

#[pg_test]
fn test_distann_registry_repeatable_read_rejects_stale_snapshot() {
    const SCHEMA: &str = "ec_distann_registry_repeatable";
    let conninfo = current_pg_test_loopback_conninfo();
    let mut monitor =
        distann_registry_concurrency_connect(&conninfo, "ec_distann_registry_rr_monitor");
    let fixture = distann_registry_concurrency_setup(&mut monitor, SCHEMA);

    let mut stale = distann_registry_concurrency_connect(&conninfo, "ec_distann_registry_rr_stale");
    stale
        .batch_execute(
            "SET statement_timeout = '10s';
             BEGIN ISOLATION LEVEL REPEATABLE READ",
        )
        .expect("Repeatable Read transaction should begin");
    let initial_revision = stale
        .query_one(
            &format!(
                "SELECT revision
                   FROM {}.ec_distann_registry_state
                  WHERE index_oid = $1::text::regclass",
                fixture.extension_schema
            ),
            &[&fixture.coordinator_index],
        )
        .expect("Repeatable Read transaction should establish its registry snapshot")
        .try_get::<_, i64>(0)
        .expect("initial registry revision should decode");
    assert_eq!(initial_revision, 1);

    let mut writer =
        distann_registry_concurrency_connect(&conninfo, "ec_distann_registry_rr_writer");
    distann_registry_concurrency_install_secret(&mut writer);
    writer
        .batch_execute("SET statement_timeout = '10s'; BEGIN")
        .expect("registry writer transaction should begin");
    distann_registry_concurrency_unregister(&mut writer, &fixture, 0)
        .expect("registry writer should unregister P1");
    distann_registry_concurrency_register(
        &mut writer,
        &fixture,
        0,
        18,
        "registry-race/p2",
        &fixture.participant_two_index,
    )
    .expect("registry writer should register P2");
    writer
        .batch_execute("COMMIT")
        .expect("registry writer replacement should commit");

    let stale_error = distann_registry_concurrency_unregister(&mut stale, &fixture, 0)
        .expect_err("stale Repeatable Read unregister must fail serialization");
    let stale_sqlstate = stale_error
        .as_db_error()
        .map(|error| error.code().code().to_owned());
    let stale_message = stale_error
        .as_db_error()
        .map(|error| error.message().to_owned())
        .unwrap_or_else(|| stale_error.to_string());
    stale
        .batch_execute("ROLLBACK")
        .expect("stale Repeatable Read transaction should roll back");

    let final_state = distann_registry_concurrency_state(&mut monitor, &fixture);
    distann_registry_concurrency_cleanup(&mut monitor, &fixture);
    assert_eq!(
        stale_sqlstate.as_deref(),
        Some("40001"),
        "stale registry edit returned an unexpected error: {stale_message}"
    );
    assert_eq!(
        final_state,
        (3, 0, fixture.participant_two_uuid.clone(), 1),
        "serialization failure must leave the committed replacement untouched"
    );
}
