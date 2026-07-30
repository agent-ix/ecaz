//! M2 async libpq transport for the ec_distann coordinator (Task 164).
//!
//! The coordinator issues one `ec_distann_expand_nodes` call per remote owning
//! node per hop round (FR-079/FR-081) over a per-backend, per-conninfo pooled
//! `tokio-postgres` connection — the same connect/spawn shape as the SPIRE
//! remote transport (`ec_spire/.../tls.rs`), reduced to the M2 essentials
//! (NoTls loopback substrate, ADR-085 D2). Each call first sets the target
//! node's roster/epoch/local_node_id on the session so the endpoint validates
//! ownership for that node — this is what makes the single-instance loopback
//! "two-node" fixture behave like two nodes, and is a redundant no-op against a
//! correctly-configured real node.
//!
//! TLS / connection-secret handling (NFR-014) is deferred to the productionizing
//! pass; M2's gate substrate is loopback multi-instance.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::ptr::NonNull;
use std::time::Duration;

#[cfg(feature = "pg_test")]
use pgrx::iter::TableIterator;
use pgrx::pg_sys;
#[cfg(feature = "pg_test")]
use pgrx::{name, pg_extern};
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, NoTls, Row, Statement};

use crate::storage::page::ItemPointer;
use crate::storage::relation::index_heap_relation_oid_handle;
use crate::storage::relation_guard::{HeapRelationGuard, IndexRelationGuard};
use crate::storage::slot_guard::TupleTableSlotGuard;

use super::ambuild::read_metadata_from_index_handle;
use super::epoch::{compute_epoch_fingerprint, DISTANN_EPOCH_FINGERPRINT_V1};
use super::expand::LocalNodeExpander;
use super::expand_error::DistannExpandError;
use super::head_cache::cached_index_entry;
use super::placement::{group_by_owning_node, DistannPlacementDirectory};
use super::quantizer::{metadata_code_len, DistannPreparedQuery};
use super::roster::{
    current_roster_spec, local_epoch_identity, placement_directory_for_epoch, scan_epoch,
};
use super::routine::indexed_ecvector_attnum;
use super::scan::{
    distann_orchestrated_search, DistannExpandedNode, DistannNodeExpander,
    DistannOrchestrationParams, DistannScanHit, DistannSeedCandidate,
};
use super::tuple::DistannNodeTuple;
use crate::am::ec_diskann::maybe_check_for_interrupts;

enum RemoteAwaitError<E> {
    Remote(E),
    TimedOut,
    Interrupted,
}

const POSTGRES_INTERRUPT_POLL_MS: u64 = 5;
const REMOTE_CANCEL_DELIVERY_TIMEOUT_MS: u64 = 100;

async fn await_remote<T, E>(
    timeout: Duration,
    cancel_token: Option<tokio_postgres::CancelToken>,
    future: impl Future<Output = Result<T, E>>,
) -> Result<T, RemoteAwaitError<E>> {
    let remote = tokio::time::timeout(timeout, future);
    let interrupt = postgres_interrupt_signal();
    match futures_util::future::select(Box::pin(remote), Box::pin(interrupt)).await {
        futures_util::future::Either::Left((result, _)) => match result {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(error)) => Err(RemoteAwaitError::Remote(error)),
            Err(_) => Err(RemoteAwaitError::TimedOut),
        },
        futures_util::future::Either::Right(((), _)) => {
            mark_transport_interrupt_observed();
            if let Some(cancel_token) = cancel_token {
                // Best-effort graceful cancellation is deliberately short. If
                // delivery stalls, with_transport_state drops the pooled
                // clients/driver tasks before CHECK_FOR_INTERRUPTS raises.
                let _ = tokio::time::timeout(
                    Duration::from_millis(REMOTE_CANCEL_DELIVERY_TIMEOUT_MS),
                    cancel_token.cancel_query(NoTls),
                )
                .await;
            }
            Err(RemoteAwaitError::Interrupted)
        }
    }
}

async fn postgres_interrupt_signal() {
    loop {
        if postgres_interrupt_pending() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(POSTGRES_INTERRUPT_POLL_MS)).await;
    }
}

fn interrupt_flags_request_stop(
    interrupt_pending: pg_sys::sig_atomic_t,
    query_cancel_pending: pg_sys::sig_atomic_t,
    proc_die_pending: pg_sys::sig_atomic_t,
) -> bool {
    proc_die_pending != 0 || (interrupt_pending != 0 && query_cancel_pending != 0)
}

fn postgres_interrupt_pending() -> bool {
    // SAFETY: PostgreSQL signal handlers mutate these backend-process globals.
    // Volatile reads preserve the signal-observation semantics without relying
    // on platform-specific dynamic-symbol lookup.
    unsafe {
        interrupt_flags_request_stop(
            std::ptr::read_volatile(&raw const pg_sys::InterruptPending),
            std::ptr::read_volatile(&raw const pg_sys::QueryCancelPending),
            std::ptr::read_volatile(&raw const pg_sys::ProcDiePending),
        )
    }
}

fn connect_timeout() -> Duration {
    Duration::from_millis(super::options::remote_connect_timeout_ms())
}

fn call_timeout() -> Duration {
    // Let PostgreSQL's remote statement_timeout report the primary error; the
    // client deadline is a bounded fallback if cancellation or transport
    // delivery stalls.
    Duration::from_millis(super::options::remote_statement_timeout_ms().saturating_add(5_000))
}

fn parse_remote_config(
    conninfo: &str,
    error_prefix: &str,
) -> Result<tokio_postgres::Config, String> {
    let mut config = conninfo.parse::<tokio_postgres::Config>().map_err(|_| {
        format!("{error_prefix}: could not parse participant connection descriptor")
    })?;
    config.connect_timeout(connect_timeout());
    Ok(config)
}

async fn configure_remote_statement_timeout(
    client: &Client,
    error_prefix: &str,
) -> Result<(), String> {
    let timeout = super::options::remote_statement_timeout_ms().to_string();
    match await_remote(
        call_timeout(),
        Some(client.cancel_token()),
        client.query_one(
            "SELECT set_config('statement_timeout', $1, false)",
            &[&timeout],
        ),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(RemoteAwaitError::Remote(error)) => Err(format!(
            "{error_prefix}: could not configure participant statement timeout: {error}"
        )),
        Err(RemoteAwaitError::TimedOut) => Err(format!(
            "{error_prefix}: participant statement-timeout setup timed out"
        )),
        Err(RemoteAwaitError::Interrupted) => Err(format!(
            "{error_prefix}: participant statement-timeout setup interrupted"
        )),
    }
}

async fn lifecycle_query(
    client: &Client,
    context: &str,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Vec<Row>, String> {
    match await_remote(
        call_timeout(),
        Some(client.cancel_token()),
        client.query(sql, params),
    )
    .await
    {
        Ok(rows) => Ok(rows),
        Err(RemoteAwaitError::Remote(error)) => Err(remote_error(context, error)),
        Err(RemoteAwaitError::TimedOut) => {
            Err(format!("EC_BUILD_INCOMPLETE: remote {context} timed out"))
        }
        Err(RemoteAwaitError::Interrupted) => {
            Err(format!("EC_BUILD_INCOMPLETE: remote {context} interrupted"))
        }
    }
}

async fn lifecycle_query_one(
    client: &Client,
    context: &str,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Row, String> {
    match await_remote(
        call_timeout(),
        Some(client.cancel_token()),
        client.query_one(sql, params),
    )
    .await
    {
        Ok(row) => Ok(row),
        Err(RemoteAwaitError::Remote(error)) => Err(remote_error(context, error)),
        Err(RemoteAwaitError::TimedOut) => {
            Err(format!("EC_BUILD_INCOMPLETE: remote {context} timed out"))
        }
        Err(RemoteAwaitError::Interrupted) => {
            Err(format!("EC_BUILD_INCOMPLETE: remote {context} interrupted"))
        }
    }
}

async fn physical_query(
    client: &Client,
    statement: &Statement,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Vec<Row>, DistannExpandError> {
    match await_remote(
        call_timeout(),
        Some(client.cancel_token()),
        client.query(statement, params),
    )
    .await
    {
        Ok(rows) => Ok(rows),
        Err(RemoteAwaitError::Remote(error)) => Err(classify_physical_read_error(error)),
        Err(RemoteAwaitError::TimedOut) => Err(DistannExpandError::Internal(
            "physical generation RPC timed out".to_owned(),
        )),
        Err(RemoteAwaitError::Interrupted) => Err(DistannExpandError::Internal(
            "physical generation RPC interrupted".to_owned(),
        )),
    }
}

async fn prepare_physical_statement(
    client: &Client,
    sql: &'static str,
) -> Result<Statement, DistannExpandError> {
    match await_remote(
        call_timeout(),
        Some(client.cancel_token()),
        client.prepare(sql),
    )
    .await
    {
        Ok(statement) => Ok(statement),
        Err(RemoteAwaitError::Remote(error)) => Err(classify_physical_read_error(error)),
        Err(RemoteAwaitError::TimedOut) => Err(DistannExpandError::Internal(
            "physical generation statement preparation timed out".to_owned(),
        )),
        Err(RemoteAwaitError::Interrupted) => Err(DistannExpandError::Internal(
            "physical generation statement preparation interrupted".to_owned(),
        )),
    }
}

async fn ensure_physical_statements(
    connections: &mut HashMap<String, PooledConnection>,
    conn_keys: &[String],
    sql: &'static str,
) -> Result<(), DistannExpandError> {
    let mut seen = HashSet::with_capacity(conn_keys.len());
    let stale = conn_keys
        .iter()
        .filter(|key| seen.insert((*key).clone()))
        .filter(|key| !connections[*key].prepared_statements.contains_key(sql))
        .cloned()
        .collect::<Vec<_>>();
    let prepared = join_owner_futures(
        stale
            .iter()
            .map(|key| prepare_physical_statement(&connections[key].client, sql)),
    )
    .await;
    for (key, statement) in stale.into_iter().zip(prepared) {
        connections
            .get_mut(&key)
            .expect("pooled connection disappeared during statement preparation")
            .prepared_statements
            .insert(sql, statement?);
    }
    Ok(())
}

async fn join_owner_futures<I, F>(futures: I) -> Vec<F::Output>
where
    I: IntoIterator<Item = F>,
    F: Future,
{
    futures_util::future::join_all(futures).await
}

async fn scan_query(
    client: &Client,
    context: &str,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Vec<Row>, DistannExpandError> {
    match await_remote(
        call_timeout(),
        Some(client.cancel_token()),
        client.query(sql, params),
    )
    .await
    {
        Ok(rows) => Ok(rows),
        Err(RemoteAwaitError::Remote(error)) => {
            let code = error.code().map(|state| state.code().to_owned());
            let detail = error
                .as_db_error()
                .map(|db| db.message().to_owned())
                .unwrap_or_else(|| error.to_string());
            Err(DistannExpandError::from_wire_sqlstate(
                code.as_deref(),
                format!("ec_distann remote {context} failed: {detail}"),
            ))
        }
        Err(RemoteAwaitError::TimedOut) => Err(DistannExpandError::Internal(format!(
            "ec_distann remote {context} timed out"
        ))),
        Err(RemoteAwaitError::Interrupted) => Err(DistannExpandError::Internal(format!(
            "ec_distann remote {context} interrupted"
        ))),
    }
}

async fn open_remote_connection(
    conninfo: &str,
    error_prefix: &str,
) -> Result<(Client, tokio::task::JoinHandle<()>), String> {
    let config = parse_remote_config(conninfo, error_prefix)?;
    let (client, connection) =
        match await_remote(connect_timeout(), None, config.connect(NoTls)).await {
            Ok(connection) => connection,
            Err(RemoteAwaitError::Remote(error)) => {
                return Err(format!(
                    "{error_prefix}: could not connect to participant: {error}"
                ));
            }
            Err(RemoteAwaitError::TimedOut) => {
                return Err(format!("{error_prefix}: participant connection timed out"));
            }
            Err(RemoteAwaitError::Interrupted) => {
                return Err(format!(
                    "{error_prefix}: participant connection interrupted"
                ));
            }
        };
    let task = tokio::spawn(async move {
        let _ = connection.await;
    });
    if let Err(error) = configure_remote_statement_timeout(&client, error_prefix).await {
        task.abort();
        return Err(error);
    }
    Ok((client, task))
}

async fn configure_scan_identity(
    client: &Client,
    identity: &(String, String, String),
) -> Result<(), String> {
    match await_remote(
        call_timeout(),
        Some(client.cancel_token()),
        client.query(SESSION_SETUP_SQL, &[&identity.0, &identity.1, &identity.2]),
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(RemoteAwaitError::Remote(error)) => {
            let detail = error
                .as_db_error()
                .map(|db| db.message().to_owned())
                .unwrap_or_else(|| error.to_string());
            Err(format!(
                "ec_distann remote transport session setup failed: {detail}"
            ))
        }
        Err(RemoteAwaitError::TimedOut) => {
            Err("ec_distann remote transport session setup timed out".to_owned())
        }
        Err(RemoteAwaitError::Interrupted) => {
            Err("ec_distann remote transport session setup interrupted".to_owned())
        }
    }
}

async fn ensure_pooled_connections(
    connections: &mut HashMap<String, PooledConnection>,
    specs: &[(String, &str)],
    error_prefix: &str,
) -> Result<(), String> {
    let mut seen = HashSet::with_capacity(specs.len());
    let missing = specs
        .iter()
        .filter(|(key, _)| seen.insert(key.clone()))
        .filter(|(key, _)| {
            connections
                .get(key)
                .map(|pooled| pooled.client.is_closed() || pooled.task.is_finished())
                .unwrap_or(true)
        })
        .map(|(key, conninfo)| (key.clone(), *conninfo))
        .collect::<Vec<_>>();
    let opened = join_owner_futures(
        missing
            .iter()
            .map(|(_, conninfo)| open_remote_connection(conninfo, error_prefix)),
    )
    .await;
    let mut ready = Vec::with_capacity(opened.len());
    let mut first_error = None;
    for ((key, _), opened) in missing.into_iter().zip(opened) {
        match opened {
            Ok((client, task)) => ready.push((key, client, task)),
            Err(error) if first_error.is_none() => first_error = Some(error),
            Err(_) => {}
        }
    }
    if let Some(error) = first_error {
        for (_, client, task) in ready {
            task.abort();
            drop(client);
        }
        return Err(error);
    }
    for (key, client, task) in ready {
        connections.insert(
            key,
            PooledConnection {
                client,
                task,
                applied_identity: None,
                applied_statement_timeout_ms: super::options::remote_statement_timeout_ms(),
                prepared_statements: HashMap::new(),
                physical_query_digest: None,
            },
        );
    }

    let desired_timeout = super::options::remote_statement_timeout_ms();
    let mut seen = HashSet::with_capacity(specs.len());
    let stale = specs
        .iter()
        .filter(|(key, _)| {
            seen.insert(key.clone())
                && connections[key].applied_statement_timeout_ms != desired_timeout
        })
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    let refreshed = join_owner_futures(
        stale
            .iter()
            .map(|key| configure_remote_statement_timeout(&connections[key].client, error_prefix)),
    )
    .await;
    for result in refreshed {
        result?;
    }
    for key in stale {
        connections
            .get_mut(&key)
            .expect("pooled connection disappeared during timeout refresh")
            .applied_statement_timeout_ms = desired_timeout;
    }
    Ok(())
}

async fn ensure_scan_sessions(
    connections: &mut HashMap<String, PooledConnection>,
    sessions: &[(String, &str, (String, String, String))],
) -> Result<(), String> {
    let specs = sessions
        .iter()
        .map(|(key, conninfo, _)| (key.clone(), *conninfo))
        .collect::<Vec<_>>();
    ensure_pooled_connections(connections, &specs, "ec_distann remote transport").await?;

    let mut identities = HashMap::with_capacity(sessions.len());
    for (key, _, identity) in sessions {
        if let Some(previous) = identities.insert(key.clone(), identity.clone()) {
            if previous != *identity {
                return Err(
                    "ec_distann remote transport assigned conflicting identities to one session"
                        .to_owned(),
                );
            }
        }
    }
    let stale = identities
        .iter()
        .filter(|(key, identity)| connections[*key].applied_identity.as_ref() != Some(*identity))
        .map(|(key, identity)| (key.clone(), identity.clone()))
        .collect::<Vec<_>>();
    let configured = join_owner_futures(
        stale
            .iter()
            .map(|(key, identity)| configure_scan_identity(&connections[key].client, identity)),
    )
    .await;
    for result in configured {
        result?;
    }
    for (key, identity) in stale {
        connections
            .get_mut(&key)
            .expect("pooled connection disappeared during identity setup")
            .applied_identity = Some(identity);
    }
    Ok(())
}

async fn lifecycle_client<'a>(
    connections: &'a mut HashMap<String, PooledConnection>,
    conninfo: &str,
) -> Result<&'a Client, String> {
    let key = lifecycle_connection_key(conninfo);
    ensure_pooled_connections(
        connections,
        &[(key.clone(), conninfo)],
        "EC_BUILD_INCOMPLETE",
    )
    .await?;
    Ok(&connections[&key].client)
}

fn lifecycle_connection_key(conninfo: &str) -> String {
    format!("lifecycle\u{1}{conninfo}")
}

fn remote_error(context: &str, error: tokio_postgres::Error) -> String {
    let detail = error
        .as_db_error()
        .map(|db| db.message().to_owned())
        .unwrap_or_else(|| error.to_string());
    format!("EC_BUILD_INCOMPLETE: remote {context} failed: {detail}")
}

#[cfg(feature = "distann-head-attribution-benchmark")]
pub(crate) struct DistannPhysicalSeedRequest<'a> {
    pub(crate) conninfo: &'a str,
    pub(crate) index_regclass: &'a str,
    pub(crate) epoch_fingerprint: &'a [u8],
    pub(crate) query: &'a [f32],
    pub(crate) limit: i32,
}

#[cfg(feature = "distann-head-attribution-benchmark")]
const PHYSICAL_SEED_SQL: &str = "SELECT vec_id, code_dist
   FROM ec_distann_physical_seed_candidates_benchmark(
       $1::text::regclass, $2::bytea, $3::real[], $4::integer)";

const TRAVERSAL_REPLICA_CHUNK_SQL: &str = "SELECT owner_ordinal, vec_id, graph_record, exact_vector
       FROM ec_distann_stream_traversal_replica_chunk(
           $1::text::regclass, $2::bytea, $3::bigint, $4::integer)";

#[cfg(not(feature = "distann-head-attribution-benchmark"))]
const PHYSICAL_EXPAND_SQL: &str = "SELECT vec_id, exact_dist, is_tombstone,
        neighbor_vec_ids, neighbor_code_dists
   FROM ec_distann_expand_nodes(
       $1::text::regclass, $2::bytea, $3::real[],
       $4::bytea, $5::bigint[], $6::real, $7::integer)";

#[cfg(feature = "distann-head-attribution-benchmark")]
const PHYSICAL_EXPAND_SQL: &str = "SELECT vec_id, exact_dist, is_tombstone,
        neighbor_vec_ids, neighbor_code_dists, owner_total_ns, owner_open_validate_ns,
        owner_graph_read_ns, owner_score_ns, owner_response_encode_ns, owner_response_bytes
   FROM ec_distann_expand_physical_nodes_profile(
       $1::text::regclass, $2::bytea, $3::real[],
       $4::bytea, $5::bigint[], $6::real, $7::integer)";

#[cfg(not(feature = "distann-head-attribution-benchmark"))]
const PHYSICAL_MATERIALIZE_SQL: &str = "SELECT vec_id, is_tombstone, tuple_payload_missing,
        payload_nulls, payload_values
   FROM ec_distann_materialize_row_payloads(
       $1::text::regclass, $2::bytea, $3::bigint[],
       $4::smallint[], $5::bytea)";

#[cfg(feature = "distann-head-attribution-benchmark")]
const PHYSICAL_MATERIALIZE_SQL: &str = "SELECT vec_id, is_tombstone, tuple_payload_missing,
        payload_nulls, payload_values, owner_total_ns, owner_open_validate_ns,
        owner_node_lookup_ns, owner_payload_sql_ns, payload_bytes
   FROM ec_distann_materialize_physical_row_payloads_profile(
       $1::text::regclass, $2::bytea, $3::bigint[],
       $4::smallint[], $5::bytea, $6::boolean)";

#[cfg(feature = "distann-head-attribution-benchmark")]
pub(crate) fn remote_physical_seed_batch(
    requests: &[DistannPhysicalSeedRequest<'_>],
) -> Vec<Result<Vec<DistannSeedCandidate>, DistannExpandError>> {
    if requests.is_empty() {
        return Vec::new();
    }
    let conn_keys = requests
        .iter()
        .map(|request| lifecycle_connection_key(request.conninfo))
        .collect::<Vec<_>>();
    let outcome = with_transport_state::<_, DistannExpandError>(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let specs = requests
                .iter()
                .enumerate()
                .map(|(index, request)| (conn_keys[index].clone(), request.conninfo))
                .collect::<Vec<_>>();
            ensure_pooled_connections(connections, &specs, "EC_BUILD_INCOMPLETE")
                .await
                .map_err(DistannExpandError::Internal)?;
            ensure_physical_statements(connections, &conn_keys, PHYSICAL_SEED_SQL).await?;
            let futures = requests.iter().enumerate().map(|(index, request)| {
                let pooled = &connections[&conn_keys[index]];
                run_one_physical_seed(
                    &pooled.client,
                    &pooled.prepared_statements[PHYSICAL_SEED_SQL],
                    request,
                )
            });
            Ok(join_owner_futures(futures).await)
        })
    });
    outcome.unwrap_or_else(|error| requests.iter().map(|_| Err(error.clone())).collect())
}

#[cfg(feature = "distann-head-attribution-benchmark")]
async fn run_one_physical_seed(
    client: &Client,
    statement: &Statement,
    request: &DistannPhysicalSeedRequest<'_>,
) -> Result<Vec<DistannSeedCandidate>, DistannExpandError> {
    let rows = physical_query(
        client,
        statement,
        &[
            &request.index_regclass,
            &request.epoch_fingerprint,
            &request.query,
            &request.limit,
        ],
    )
    .await?;
    rows.into_iter()
        .map(|row| {
            let vec_id: i64 = row.try_get(0).map_err(row_err)?;
            let dist: f32 = row.try_get(1).map_err(row_err)?;
            Ok(DistannSeedCandidate {
                vec_id: u64::from_le_bytes(vec_id.to_le_bytes()),
                dist,
            })
        })
        .collect()
}

pub(crate) struct DistannTraversalReplicaChunkRequest<'a> {
    pub(crate) conninfo: &'a str,
    pub(crate) index_regclass: &'a str,
    pub(crate) epoch_fingerprint: &'a [u8],
    pub(crate) after_vec_id: Option<i64>,
    pub(crate) limit: i32,
}

pub(crate) fn remote_traversal_replica_chunk(
    request: &DistannTraversalReplicaChunkRequest<'_>,
) -> Result<Vec<super::generation_read::TraversalReplicaChunkRow>, DistannExpandError> {
    let conn_key = lifecycle_connection_key(request.conninfo);
    with_transport_state::<_, DistannExpandError>(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            ensure_pooled_connections(
                connections,
                &[(conn_key.clone(), request.conninfo)],
                "EC_BUILD_INCOMPLETE",
            )
            .await
            .map_err(DistannExpandError::Internal)?;
            ensure_physical_statements(
                connections,
                std::slice::from_ref(&conn_key),
                TRAVERSAL_REPLICA_CHUNK_SQL,
            )
            .await?;
            let pooled = &connections[&conn_key];
            let rows = physical_query(
                &pooled.client,
                &pooled.prepared_statements[TRAVERSAL_REPLICA_CHUNK_SQL],
                &[
                    &request.index_regclass,
                    &request.epoch_fingerprint,
                    &request.after_vec_id,
                    &request.limit,
                ],
            )
            .await?;
            rows.into_iter()
                .map(|row| {
                    let owner_ordinal: i32 = row.try_get(0).map_err(row_err)?;
                    let vec_id: i64 = row.try_get(1).map_err(row_err)?;
                    let graph_record: Vec<u8> = row.try_get(2).map_err(row_err)?;
                    let exact_vector: Vec<u8> = row.try_get(3).map_err(row_err)?;
                    Ok(super::generation_read::TraversalReplicaChunkRow {
                        owner_ordinal: u32::try_from(owner_ordinal).map_err(|_| {
                            DistannExpandError::Internal(
                                "traversal replica owner ordinal is negative".to_owned(),
                            )
                        })?,
                        vec_id: u64::from_le_bytes(vec_id.to_le_bytes()),
                        graph_record,
                        exact_vector,
                    })
                })
                .collect()
        })
    })
}

pub(crate) struct DistannPhysicalExpandRequest<'a> {
    pub(crate) conninfo: &'a str,
    pub(crate) index_regclass: &'a str,
    pub(crate) epoch_fingerprint: &'a [u8],
    pub(crate) query: &'a [f32],
    pub(crate) query_digest: &'a [u8; 32],
    pub(crate) vec_ids: &'a [u64],
    pub(crate) code_threshold: Option<f32>,
    pub(crate) candidate_limit: Option<i32>,
}

pub(crate) fn remote_physical_expand_batch(
    requests: &[DistannPhysicalExpandRequest<'_>],
) -> Vec<Result<Vec<DistannExpandedNode>, DistannExpandError>> {
    if requests.is_empty() {
        return Vec::new();
    }
    #[cfg(feature = "distann-head-attribution-benchmark")]
    let wire_ids_started = std::time::Instant::now();
    let wire_ids = requests
        .iter()
        .map(|request| {
            request
                .vec_ids
                .iter()
                .map(|vec_id| i64::from_le_bytes(vec_id.to_le_bytes()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    #[cfg(feature = "distann-head-attribution-benchmark")]
    let wire_ids_elapsed = wire_ids_started.elapsed();
    let conn_keys = requests
        .iter()
        .map(|request| lifecycle_connection_key(request.conninfo))
        .collect::<Vec<_>>();
    let outcome = with_transport_state::<_, DistannExpandError>(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            // Match the logical transport: connection establishment and budget
            // refresh are outside the hot fan-out, then all owner RPCs are
            // driven together so one hop costs max(remote RTT), not their sum.
            let specs = requests
                .iter()
                .enumerate()
                .map(|(index, request)| (conn_keys[index].clone(), request.conninfo))
                .collect::<Vec<_>>();
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let connections_opened = conn_keys
                .iter()
                .filter(|key| !connections.contains_key(*key))
                .count();
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let connection_started = std::time::Instant::now();
            ensure_pooled_connections(connections, &specs, "EC_BUILD_INCOMPLETE")
                .await
                .map_err(DistannExpandError::Internal)?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let statements_prepared = conn_keys
                .iter()
                .filter(|key| {
                    !connections[*key]
                        .prepared_statements
                        .contains_key(PHYSICAL_EXPAND_SQL)
                })
                .count();
            ensure_physical_statements(connections, &conn_keys, PHYSICAL_EXPAND_SQL).await?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            {
                super::stage_counters::record(
                    super::stage_counters::DistannQueryStage::TraversalConnectionReady,
                    connection_started.elapsed(),
                );
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::TraversalConnectionsOpened,
                    connections_opened,
                );
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::TraversalStatementsPrepared,
                    statements_prepared,
                );
            }
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let encode_started = std::time::Instant::now();
            let wire_queries = requests
                .iter()
                .enumerate()
                .map(|(index, request)| {
                    if connections[&conn_keys[index]].physical_query_digest
                        == Some(*request.query_digest)
                    {
                        &[][..]
                    } else {
                        request.query
                    }
                })
                .collect::<Vec<_>>();
            #[cfg(feature = "distann-head-attribution-benchmark")]
            {
                let query_cache_hits = wire_queries.iter().filter(|query| query.is_empty()).count();
                let request_bytes = requests
                    .iter()
                    .enumerate()
                    .map(|(index, request)| {
                        request
                            .index_regclass
                            .len()
                            .saturating_add(request.epoch_fingerprint.len())
                            .saturating_add(wire_queries[index].len().saturating_mul(4))
                            .saturating_add(request.query_digest.len())
                            .saturating_add(wire_ids[index].len().saturating_mul(8))
                            .saturating_add(5)
                    })
                    .sum();
                super::stage_counters::record(
                    super::stage_counters::DistannQueryStage::TraversalRequestEncode,
                    wire_ids_elapsed.saturating_add(encode_started.elapsed()),
                );
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::TraversalQueryCacheHits,
                    query_cache_hits,
                );
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::TraversalQueryCacheMisses,
                    wire_queries.len().saturating_sub(query_cache_hits),
                );
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::TraversalRequestBytes,
                    request_bytes,
                );
            }
            let futures = requests.iter().enumerate().map(|(index, request)| {
                let pooled = &connections[&conn_keys[index]];
                run_one_physical_expand(
                    &pooled.client,
                    &pooled.prepared_statements[PHYSICAL_EXPAND_SQL],
                    request,
                    wire_queries[index],
                    &wire_ids[index],
                )
            });
            let results = join_owner_futures(futures).await;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            {
                let telemetry = results
                    .iter()
                    .filter_map(|result| result.as_ref().ok())
                    .filter_map(|nodes| nodes.first())
                    .collect::<Vec<_>>();
                if let Some(critical) = telemetry.iter().max_by_key(|node| node.coordinator_rpc_ns) {
                    let rpc_ns = critical.coordinator_rpc_ns.max(0);
                    let owner_ns = critical.owner_total_ns.max(0);
                    let decode_ns = critical.coordinator_decode_ns.max(0);
                    let transport_ns = rpc_ns.saturating_sub(owner_ns).saturating_sub(decode_ns);
                    for (stage, nanos) in [
                        (super::stage_counters::DistannQueryStage::TraversalOwnerOpenValidate, critical.owner_open_validate_ns),
                        (super::stage_counters::DistannQueryStage::TraversalOwnerGraphRead, critical.owner_graph_read_ns),
                        (super::stage_counters::DistannQueryStage::TraversalOwnerScore, critical.owner_score_ns),
                        (super::stage_counters::DistannQueryStage::TraversalOwnerResponseEncode, critical.owner_response_encode_ns),
                        (super::stage_counters::DistannQueryStage::TraversalOwnerService, critical.owner_total_ns),
                        (super::stage_counters::DistannQueryStage::TraversalTransportWait, transport_ns),
                        (super::stage_counters::DistannQueryStage::TraversalCoordinatorReceiveDecode, critical.coordinator_decode_ns),
                    ] {
                        super::stage_counters::record(
                            stage,
                            Duration::from_nanos(u64::try_from(nanos.max(0)).unwrap_or(u64::MAX)),
                        );
                    }
                    let owner_totals = telemetry.iter().map(|node| node.owner_total_ns.max(0));
                    let min = owner_totals.clone().min().unwrap_or(0);
                    let max = owner_totals.max().unwrap_or(0);
                    super::stage_counters::record(
                        super::stage_counters::DistannQueryStage::TraversalStragglerSpread,
                        Duration::from_nanos(u64::try_from((max - min).max(0)).unwrap_or(u64::MAX)),
                    );
                    let response_bytes = telemetry.iter().fold(0_usize, |total, node| {
                        total.saturating_add(usize::try_from(node.owner_response_bytes.max(0)).unwrap_or(usize::MAX))
                    });
                    super::stage_counters::record_work(
                        super::stage_counters::DistannMaterializationWork::TraversalResponseBytes,
                        response_bytes,
                    );
                }
            }
            for (index, result) in results.iter().enumerate() {
                if result.is_ok() {
                    connections
                        .get_mut(&conn_keys[index])
                        .expect("physical owner connection disappeared after expansion")
                        .physical_query_digest = Some(*requests[index].query_digest);
                }
            }
            Ok(results)
        })
    });
    outcome.unwrap_or_else(|error| requests.iter().map(|_| Err(error.clone())).collect())
}

async fn run_one_physical_expand(
    client: &Client,
    statement: &Statement,
    request: &DistannPhysicalExpandRequest<'_>,
    wire_query: &[f32],
    wire_ids: &[i64],
) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    let rpc_started = std::time::Instant::now();
    let rows = physical_query(
        client,
        statement,
        &[
            &request.index_regclass,
            &request.epoch_fingerprint,
            &wire_query,
            &request.query_digest.as_slice(),
            &wire_ids,
            &request.code_threshold,
            &request.candidate_limit,
        ],
    )
    .await?;
    #[cfg(feature = "distann-head-attribution-benchmark")]
    let decode_started = std::time::Instant::now();
    let nodes = rows
        .into_iter()
        .map(|row| {
            let vec_id: i64 = row.try_get(0).map_err(row_err)?;
            let exact_dist: Option<f32> = row.try_get(1).map_err(row_err)?;
            let is_tombstone: bool = row.try_get(2).map_err(row_err)?;
            let neighbor_vec_ids: Vec<i64> = row.try_get(3).map_err(row_err)?;
            let neighbor_code_dists: Vec<f32> = row.try_get(4).map_err(row_err)?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let owner_total_ns: i64 = row.try_get(5).map_err(row_err)?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let owner_open_validate_ns: i64 = row.try_get(6).map_err(row_err)?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let owner_graph_read_ns: i64 = row.try_get(7).map_err(row_err)?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let owner_score_ns: i64 = row.try_get(8).map_err(row_err)?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let owner_response_encode_ns: i64 = row.try_get(9).map_err(row_err)?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let owner_response_bytes: i64 = row.try_get(10).map_err(row_err)?;
            Ok(DistannExpandedNode {
                vec_id: u64::from_le_bytes(vec_id.to_le_bytes()),
                exact_dist,
                is_tombstone,
                heap_tid: ItemPointer::INVALID,
                neighbor_vec_ids: neighbor_vec_ids
                    .into_iter()
                    .map(|id| u64::from_le_bytes(id.to_le_bytes()))
                    .collect(),
                neighbor_code_dists,
                #[cfg(feature = "distann-head-attribution-benchmark")]
                owner_total_ns,
                #[cfg(not(feature = "distann-head-attribution-benchmark"))]
                owner_total_ns: 0,
                #[cfg(feature = "distann-head-attribution-benchmark")]
                owner_open_validate_ns,
                #[cfg(not(feature = "distann-head-attribution-benchmark"))]
                owner_open_validate_ns: 0,
                #[cfg(feature = "distann-head-attribution-benchmark")]
                owner_graph_read_ns,
                #[cfg(not(feature = "distann-head-attribution-benchmark"))]
                owner_graph_read_ns: 0,
                #[cfg(feature = "distann-head-attribution-benchmark")]
                owner_score_ns,
                #[cfg(not(feature = "distann-head-attribution-benchmark"))]
                owner_score_ns: 0,
                #[cfg(feature = "distann-head-attribution-benchmark")]
                owner_response_encode_ns,
                #[cfg(not(feature = "distann-head-attribution-benchmark"))]
                owner_response_encode_ns: 0,
                #[cfg(feature = "distann-head-attribution-benchmark")]
                owner_response_bytes,
                #[cfg(not(feature = "distann-head-attribution-benchmark"))]
                owner_response_bytes: 0,
                coordinator_rpc_ns: 0,
                coordinator_decode_ns: 0,
            })
        })
        .collect::<Result<Vec<_>, DistannExpandError>>()?;
    #[cfg(feature = "distann-head-attribution-benchmark")]
    let nodes = {
        let mut nodes = nodes;
        let coordinator_decode_ns =
            i64::try_from(decode_started.elapsed().as_nanos()).unwrap_or(i64::MAX);
        let coordinator_rpc_ns =
            i64::try_from(rpc_started.elapsed().as_nanos()).unwrap_or(i64::MAX);
        for node in &mut nodes {
            node.coordinator_decode_ns = coordinator_decode_ns;
            node.coordinator_rpc_ns = coordinator_rpc_ns;
        }
        nodes
    };
    Ok(nodes)
}

pub(crate) struct DistannPhysicalMaterializeRequest<'a> {
    pub(crate) conninfo: &'a str,
    pub(crate) index_regclass: &'a str,
    pub(crate) epoch_fingerprint: &'a [u8],
    pub(crate) vec_ids: &'a [u64],
    pub(crate) projection_attnums: &'a [i16],
    pub(crate) expected_schema_fingerprint: &'a [u8],
    #[cfg(feature = "distann-head-attribution-benchmark")]
    pub(crate) use_cached_payload_plan: bool,
}

pub(crate) fn remote_physical_materialize_batch(
    requests: &[DistannPhysicalMaterializeRequest<'_>],
) -> Vec<Result<DistannPhysicalMaterializeBatch, DistannExpandError>> {
    if requests.is_empty() {
        return Vec::new();
    }
    let wire_ids = requests
        .iter()
        .map(|request| {
            request
                .vec_ids
                .iter()
                .map(|vec_id| i64::from_le_bytes(vec_id.to_le_bytes()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let conn_keys = requests
        .iter()
        .map(|request| lifecycle_connection_key(request.conninfo))
        .collect::<Vec<_>>();
    let outcome = with_transport_state::<_, DistannExpandError>(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let specs = requests
                .iter()
                .enumerate()
                .map(|(index, request)| (conn_keys[index].clone(), request.conninfo))
                .collect::<Vec<_>>();
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let connection_started = std::time::Instant::now();
            ensure_pooled_connections(connections, &specs, "EC_BUILD_INCOMPLETE")
                .await
                .map_err(DistannExpandError::Internal)?;
            ensure_physical_statements(connections, &conn_keys, PHYSICAL_MATERIALIZE_SQL).await?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            super::stage_counters::record(
                super::stage_counters::DistannQueryStage::MaterializeConnectionReady,
                connection_started.elapsed(),
            );
            let futures = requests.iter().enumerate().map(|(index, request)| {
                let pooled = &connections[&conn_keys[index]];
                run_one_physical_materialize_raw(
                    &pooled.client,
                    &pooled.prepared_statements[PHYSICAL_MATERIALIZE_SQL],
                    request,
                    &wire_ids[index],
                )
            });
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let wait_started = std::time::Instant::now();
            let rows = join_owner_futures(futures).await;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            super::stage_counters::record(
                super::stage_counters::DistannQueryStage::MaterializeRequestWait,
                wait_started.elapsed(),
            );
            Ok(rows)
        })
    });
    let raw = outcome.unwrap_or_else(|error| requests.iter().map(|_| Err(error.clone())).collect());
    #[cfg(feature = "distann-head-attribution-benchmark")]
    let decode_started = std::time::Instant::now();
    let decoded = raw
        .into_iter()
        .map(|result| {
            result.and_then(|(rows, roundtrip)| {
                #[cfg(not(feature = "distann-head-attribution-benchmark"))]
                let _ = roundtrip;
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::record(
                    super::stage_counters::DistannQueryStage::MaterializeRequestRoundtripWork,
                    roundtrip,
                );
                decode_physical_materialize_rows(rows)
            })
        })
        .collect::<Vec<_>>();
    #[cfg(feature = "distann-head-attribution-benchmark")]
    {
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::MaterializeCoordinatorDecode,
            decode_started.elapsed(),
        );
        let mut owner_critical_ns = 0;
        for batch in decoded.iter().filter_map(|result| result.as_ref().ok()) {
            owner_critical_ns = owner_critical_ns.max(batch.telemetry.owner_total_ns);
            super::stage_counters::record(
                super::stage_counters::DistannQueryStage::MaterializeOwnerEndpointWork,
                Duration::from_nanos(batch.telemetry.owner_total_ns),
            );
            super::stage_counters::record(
                super::stage_counters::DistannQueryStage::MaterializeOwnerOpenValidateWork,
                Duration::from_nanos(batch.telemetry.owner_open_validate_ns),
            );
            super::stage_counters::record(
                super::stage_counters::DistannQueryStage::MaterializeOwnerNodeLookupWork,
                Duration::from_nanos(batch.telemetry.owner_node_lookup_ns),
            );
            super::stage_counters::record(
                super::stage_counters::DistannQueryStage::MaterializeOwnerPayloadSqlWork,
                Duration::from_nanos(batch.telemetry.owner_payload_sql_ns),
            );
        }
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::MaterializeOwnerEndpointCritical,
            Duration::from_nanos(owner_critical_ns),
        );
    }
    decoded
}

async fn run_one_physical_materialize_raw(
    client: &Client,
    statement: &Statement,
    request: &DistannPhysicalMaterializeRequest<'_>,
    wire_ids: &[i64],
) -> Result<(Vec<Row>, Duration), DistannExpandError> {
    let started = std::time::Instant::now();
    #[cfg(not(feature = "distann-head-attribution-benchmark"))]
    let rows = physical_query(
        client,
        statement,
        &[
            &request.index_regclass,
            &request.epoch_fingerprint,
            &wire_ids,
            &request.projection_attnums,
            &request.expected_schema_fingerprint,
        ],
    )
    .await?;
    #[cfg(feature = "distann-head-attribution-benchmark")]
    let rows = physical_query(
        client,
        statement,
        &[
            &request.index_regclass,
            &request.epoch_fingerprint,
            &wire_ids,
            &request.projection_attnums,
            &request.expected_schema_fingerprint,
            &request.use_cached_payload_plan,
        ],
    )
    .await?;
    Ok((rows, started.elapsed()))
}

fn decode_physical_materialize_rows(
    rows: Vec<Row>,
) -> Result<DistannPhysicalMaterializeBatch, DistannExpandError> {
    #[cfg(feature = "distann-head-attribution-benchmark")]
    let mut telemetry = None;
    let rows = rows
        .into_iter()
        .map(|row| {
            let vec_id: i64 = row.try_get(0).map_err(row_err)?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            {
                let row_telemetry = DistannOwnerMaterializeTelemetry {
                    owner_total_ns: nonnegative_i64_to_u64(row.try_get(5).map_err(row_err)?)?,
                    owner_open_validate_ns: nonnegative_i64_to_u64(
                        row.try_get(6).map_err(row_err)?,
                    )?,
                    owner_node_lookup_ns: nonnegative_i64_to_u64(row.try_get(7).map_err(row_err)?)?,
                    owner_payload_sql_ns: nonnegative_i64_to_u64(row.try_get(8).map_err(row_err)?)?,
                    payload_bytes: nonnegative_i64_to_u64(row.try_get(9).map_err(row_err)?)?,
                };
                if telemetry.is_some_and(|existing| existing != row_telemetry) {
                    return Err(DistannExpandError::Internal(
                        "physical owner returned inconsistent materialization telemetry".to_owned(),
                    ));
                }
                telemetry = Some(row_telemetry);
            }
            Ok(DistannMaterializedRow {
                vec_id: u64::from_le_bytes(vec_id.to_le_bytes()),
                is_tombstone: row.try_get(1).map_err(row_err)?,
                tuple_payload_missing: row.try_get(2).map_err(row_err)?,
                payload_nulls: row.try_get(3).map_err(row_err)?,
                payload_values: row.try_get(4).map_err(row_err)?,
            })
        })
        .collect::<Result<Vec<_>, DistannExpandError>>()?;
    Ok(DistannPhysicalMaterializeBatch {
        rows,
        #[cfg(feature = "distann-head-attribution-benchmark")]
        telemetry: telemetry.unwrap_or_default(),
    })
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn nonnegative_i64_to_u64(value: i64) -> Result<u64, DistannExpandError> {
    u64::try_from(value).map_err(|_| {
        DistannExpandError::Internal(
            "physical owner returned negative materialization telemetry".to_owned(),
        )
    })
}

fn classify_physical_read_error(error: tokio_postgres::Error) -> DistannExpandError {
    let code = error.code().map(|state| state.code().to_owned());
    let detail = error
        .as_db_error()
        .map(|db| db.message().to_owned())
        .unwrap_or_else(|| error.to_string());
    DistannExpandError::from_wire_sqlstate(
        code.as_deref(),
        format!("physical generation RPC failed: {detail}"),
    )
}

pub(crate) struct RemoteHandoffBegin<'a> {
    pub conninfo: &'a str,
    pub index_regclass: &'a str,
    pub epoch: i64,
    pub build_id: &'a str,
    pub build_spec_digest: &'a [u8],
    pub roster_digest: &'a [u8],
    pub descriptor: &'a [u8],
    pub descriptor_digest: &'a [u8],
    pub expected_count: i64,
    pub expected_owner_digest: &'a [u8],
}

pub(crate) fn remote_begin_epoch_handoff(request: RemoteHandoffBegin<'_>) -> Result<(), String> {
    with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let client = lifecycle_client(connections, request.conninfo).await?;
            lifecycle_query(
                client,
                "handoff begin",
                "SELECT state FROM ec_distann_begin_epoch_handoff(
                         $1::text::regclass, $2::bigint, $3::text::uuid,
                         $4::bytea, $5::bytea, $6::bytea, $7::bytea,
                         $8::bigint, $9::bytea)",
                &[
                    &request.index_regclass,
                    &request.epoch,
                    &request.build_id,
                    &request.build_spec_digest,
                    &request.roster_digest,
                    &request.descriptor,
                    &request.descriptor_digest,
                    &request.expected_count,
                    &request.expected_owner_digest,
                ],
            )
            .await?;
            Ok(())
        })
    })
}

pub(crate) fn remote_stage_epoch_batch(
    conninfo: &str,
    index_regclass: &str,
    build_id: &str,
    sequence: i64,
    digest: &[u8],
    encoded: &[u8],
) -> Result<super::handoff_router::DistannStageAck, String> {
    with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let client = lifecycle_client(connections, conninfo).await?;
            let row = lifecycle_query_one(
                client,
                "handoff stage",
                "SELECT accepted_record_count, cumulative_record_count,
                            cumulative_owner_digest
                       FROM ec_distann_stage_epoch_batch(
                           $1::text::regclass, $2::text::uuid, $3::bigint,
                           $4::bytea, $5::bytea)",
                &[&index_regclass, &build_id, &sequence, &digest, &encoded],
            )
            .await?;
            let accepted: i64 = row
                .try_get(0)
                .map_err(|error| remote_error("stage row", error))?;
            let cumulative: i64 = row
                .try_get(1)
                .map_err(|error| remote_error("stage row", error))?;
            let cumulative_digest: Vec<u8> = row
                .try_get(2)
                .map_err(|error| remote_error("stage row", error))?;
            Ok(super::handoff_router::DistannStageAck {
                accepted_record_count: u64::try_from(accepted).map_err(|_| {
                    "EC_BUILD_INCOMPLETE: remote accepted count is negative".to_owned()
                })?,
                cumulative_record_count: u64::try_from(cumulative).map_err(|_| {
                    "EC_BUILD_INCOMPLETE: remote cumulative count is negative".to_owned()
                })?,
                cumulative_owner_digest: super::canonical_wire::fixed_digest(
                    cumulative_digest,
                    "EC_BUILD_INCOMPLETE",
                    "remote cumulative digest",
                )?,
            })
        })
    })
}

pub(crate) fn remote_seal_epoch_handoff(
    conninfo: &str,
    index_regclass: &str,
    build_id: &str,
    expected_count: i64,
    expected_owner_digest: &[u8],
) -> Result<Vec<u8>, String> {
    with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let client = lifecycle_client(connections, conninfo).await?;
            let row = lifecycle_query_one(
                client,
                "handoff seal",
                "SELECT ec_distann_seal_epoch_handoff(
                         $1::text::regclass, $2::text::uuid, $3::bigint,
                         $4::bytea)",
                &[
                    &index_regclass,
                    &build_id,
                    &expected_count,
                    &expected_owner_digest,
                ],
            )
            .await?;
            row.try_get(0)
                .map_err(|error| remote_error("seal row", error))
        })
    })
}

pub(crate) fn remote_abort_epoch_handoff(
    conninfo: &str,
    index_regclass: &str,
    build_id: &str,
) -> Result<(), String> {
    with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let client = lifecycle_client(connections, conninfo).await?;
            lifecycle_query(
                client,
                "handoff abort",
                "SELECT ec_distann_abort_epoch_handoff(
                         $1::text::regclass, $2::text::uuid)",
                &[&index_regclass, &build_id],
            )
            .await?;
            Ok(())
        })
    })
}

pub(crate) fn remote_publish_epoch(
    conninfo: &str,
    index_regclass: &str,
    build_id: &str,
    epoch_manifest: &[u8],
    manifest_digest: &[u8],
) -> Result<Vec<u8>, String> {
    with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let client = lifecycle_client(connections, conninfo).await?;
            let row = lifecycle_query_one(
                client,
                "epoch publish",
                "SELECT ec_distann_publish_epoch(
                         $1::text::regclass, $2::text::uuid, $3::bytea,
                         $4::bytea)",
                &[
                    &index_regclass,
                    &build_id,
                    &epoch_manifest,
                    &manifest_digest,
                ],
            )
            .await?;
            row.try_get(0)
                .map_err(|error| remote_error("publish row", error))
        })
    })
}

pub(crate) fn remote_mark_epoch_retired(
    conninfo: &str,
    index_regclass: &str,
    successor_activation: &[u8],
    activation_digest: &[u8],
) -> Result<(), String> {
    with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let client = lifecycle_client(connections, conninfo).await?;
            lifecycle_query(
                client,
                "predecessor retirement",
                "SELECT ec_distann_mark_epoch_retired(
                         $1::text::regclass, $2::bytea, $3::bytea)",
                &[&index_regclass, &successor_activation, &activation_digest],
            )
            .await?;
            Ok(())
        })
    })
}

pub(crate) fn remote_apply_epoch_retire(
    conninfo: &str,
    index_regclass: &str,
    retire_decision: &[u8],
    retire_decision_digest: &[u8],
) -> Result<(), String> {
    with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let client = lifecycle_client(connections, conninfo).await?;
            lifecycle_query(
                client,
                "epoch retire apply",
                "SELECT ec_distann_apply_epoch_retire(
                         $1::text::regclass, $2::bytea, $3::bytea)",
                &[&index_regclass, &retire_decision, &retire_decision_digest],
            )
            .await?;
            Ok(())
        })
    })
}

pub(crate) fn remote_reclaim_cancelled_generation(
    conninfo: &str,
    index_regclass: &str,
    cancellation_audit: &[u8],
    cancellation_audit_digest: &[u8],
) -> Result<(), String> {
    with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let client = lifecycle_client(connections, conninfo).await?;
            lifecycle_query(
                client,
                "cancelled generation reclaim",
                "SELECT ec_distann_reclaim_cancelled_generation(
                         $1::text::regclass, $2::bytea, $3::bytea)",
                &[
                    &index_regclass,
                    &cancellation_audit,
                    &cancellation_audit_digest,
                ],
            )
            .await?;
            Ok(())
        })
    })
}

/// One remote expansion request over the transport.
pub(super) struct DistannRemoteExpandRequest<'a> {
    /// libpq conninfo of the target node.
    pub(super) conninfo: &'a str,
    /// Roster spec set on the session via **parameterized** `set_config` (never
    /// string-interpolated — a conninfo may contain spaces, `=`, or quotes).
    pub(super) roster_spec: &'a str,
    /// The node id this call targets (its `local_node_id` on the remote
    /// session, so the endpoint validates ownership for that node — loopback).
    pub(super) target_node_id: u32,
    pub(super) epoch: u64,
    /// The target index by NAME (regclass-castable). Resolved per-node by name
    /// — real nodes have different oids for their local index, so the wire
    /// carries the name, not an oid (FR-079 `index_regclass regclass`).
    pub(super) index_regclass: &'a str,
    pub(super) epoch_fingerprint: &'a [u8],
    pub(super) query: &'a [f32],
    /// Owned vec_ids for this node (bit-cast to i64 on the wire).
    pub(super) vec_ids: &'a [u64],
    pub(super) code_threshold: Option<f32>,
    pub(super) candidate_limit: Option<i32>,
}

struct PooledConnection {
    client: Client,
    task: tokio::task::JoinHandle<()>,
    /// Reviewer 006-P3: the (roster, local_node_id, epoch) last applied to this
    /// pooled session via `set_config`. The expand hot path skips the setup
    /// round-trip when it is unchanged, and re-applies (epoch-aware) on a
    /// mismatch so a mid-backend epoch change is never served stale.
    applied_identity: Option<(String, String, String)>,
    /// Remote `statement_timeout` last applied to this pooled session. Userset
    /// changes are refreshed before the next RPC instead of leaving a warm
    /// session under its old budget.
    applied_statement_timeout_ms: u64,
    /// Server-prepared physical RPC statements, retained with the pooled
    /// session so hot hop rounds do not repeat parse/describe work.
    prepared_statements: HashMap<&'static str, Statement>,
    /// Last physical query vector installed in this owner backend. Matching
    /// hop rounds send only the digest.
    physical_query_digest: Option<[u8; 32]>,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        self.task.abort();
    }
}

struct DistannTransportState {
    runtime: tokio::runtime::Runtime,
    connections: HashMap<String, PooledConnection>,
}

thread_local! {
    static DISTANN_TRANSPORT_STATE: RefCell<Option<DistannTransportState>> =
        const { RefCell::new(None) };
    static DISTANN_TRANSPORT_INTERRUPT_OBSERVED: Cell<bool> = const { Cell::new(false) };
}

fn mark_transport_interrupt_observed() {
    DISTANN_TRANSPORT_INTERRUPT_OBSERVED.with(|observed| observed.set(true));
}

fn take_transport_interrupt_observed() -> bool {
    DISTANN_TRANSPORT_INTERRUPT_OBSERVED.with(|observed| observed.replace(false))
}

impl DistannTransportState {
    fn new() -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| {
                "ec_distann remote transport failed to build the pooled runtime".to_owned()
            })?;
        Ok(Self {
            runtime,
            connections: HashMap::new(),
        })
    }
}

fn with_transport_state<T, E>(
    f: impl FnOnce(&mut DistannTransportState) -> Result<T, E>,
) -> Result<T, E>
where
    E: From<String>,
{
    // CHECK_FOR_INTERRUPTS may ereport(ERROR), whose PostgreSQL longjmp skips
    // Rust destructors. Keep both interrupt boundaries entirely outside the
    // RefCell borrow: otherwise a cancellation can permanently leak RefMut and
    // poison every later transport call in this backend.
    maybe_check_for_interrupts();
    let _ = take_transport_interrupt_observed();
    let result = DISTANN_TRANSPORT_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.is_none() {
            *state = Some(DistannTransportState::new()?);
        }
        let state = state
            .as_mut()
            .expect("ec_distann transport state initialized");
        let result = f(state);
        if take_transport_interrupt_observed() {
            // A query cancel can arrive while the current-thread runtime is
            // parked. The async poll returns normally so this RefMut unwinds;
            // discard every pooled connection before the outer interrupt
            // boundary raises, guaranteeing no in-flight protocol state is
            // reused by the next command in this backend.
            state.connections.clear();
        }
        result
    });
    maybe_check_for_interrupts();
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn remote_timeout_probe_for_test(
    conninfo: &str,
    sleep_seconds: f64,
) -> Result<(), String> {
    with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let client = lifecycle_client(connections, conninfo).await?;
            lifecycle_query(
                client,
                "timeout probe",
                "SELECT pg_sleep($1::double precision)",
                &[&sleep_seconds],
            )
            .await?;
            Ok(())
        })
    })
}

// `$1::text::regclass::oid` forces PG to infer $1 as text (the coordinator
// sends the index NAME as a string); `$1::regclass` alone would infer a
// regclass-typed param that tokio-postgres cannot serialize from a &str.
const EXPAND_SQL: &str = "SELECT vec_id, exact_dist, is_tombstone, neighbor_vec_ids, \
    neighbor_code_dists FROM ec_distann_expand_nodes($1::text::regclass::oid, $2, $3::real[], \
    $4::bigint[], $5, $6)";

/// Parameterized session setup — sets the target node identity without any
/// string interpolation of the (possibly quote/space-bearing) roster spec.
const SESSION_SETUP_SQL: &str = "SELECT set_config('ec_distann.roster', $1, false), \
    set_config('ec_distann.local_node_id', $2, false), \
    set_config('ec_distann.epoch', $3, false)";

/// Issue a batch of remote `ec_distann_expand_nodes` calls — one per owning
/// node for this hop round — and drive them **concurrently** on the runtime
/// (FR-081: per-node calls in parallel, so a hop costs ~max remote RTT, not the
/// sum). Returns per-request results in request order; a connect/parse failure
/// fails every request uniformly. `heap_tid` stays INVALID for remote
/// responses (local-only handle).
pub(super) fn remote_expand_batch(
    requests: &[DistannRemoteExpandRequest<'_>],
) -> Vec<Result<Vec<DistannExpandedNode>, DistannExpandError>> {
    if requests.is_empty() {
        return Vec::new();
    }
    // Owned scratch borrowed by the async futures.
    let vec_ids_i64: Vec<Vec<i64>> = requests
        .iter()
        .map(|request| request.vec_ids.iter().map(|&v| v as i64).collect())
        .collect();
    let node_id_strs: Vec<String> = requests
        .iter()
        .map(|request| request.target_node_id.to_string())
        .collect();
    // 007-P1/021-P2: pool by (conninfo, target_node_id), not conninfo alone. Two
    // logical owners that share one physical conninfo (the loopback/logical-shard
    // topology) MUST get separate pooled sessions — otherwise the second owner's
    // `local_node_id` set_config overwrites the first, and the concurrent expands
    // launched below run under the wrong node identity. The real 3-node fixture
    // gives each node a distinct port (distinct conninfo) so it was masked there.
    let conn_keys: Vec<String> = requests
        .iter()
        .map(|request| format!("{}\u{1}{}", request.conninfo, request.target_node_id))
        .collect();
    let epoch_strs: Vec<String> = requests
        .iter()
        .map(|request| request.epoch.to_string())
        .collect();

    let outcome = with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            // Establish cold owner sessions and apply their distinct identities
            // concurrently. Warm sessions skip both operations.
            let sessions = requests
                .iter()
                .enumerate()
                .map(|(index, request)| {
                    (
                        conn_keys[index].clone(),
                        request.conninfo,
                        (
                            request.roster_spec.to_owned(),
                            node_id_strs[index].clone(),
                            epoch_strs[index].clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>();
            ensure_scan_sessions(connections, &sessions).await?;

            // Fire all owners concurrently and await the whole set (expand only —
            // the session identity is already applied above).
            let futures = requests.iter().enumerate().map(|(index, request)| {
                let client = &connections[&conn_keys[index]].client;
                run_one_remote(client, request, &vec_ids_i64[index])
            });
            Ok::<_, String>(futures_util::future::join_all(futures).await)
        })
    });

    match outcome {
        Ok(results) => results,
        // A connect/parse failure (or runtime init failure) fails all uniformly.
        Err(message) => requests
            .iter()
            .map(|_| Err(DistannExpandError::Internal(message.clone())))
            .collect(),
    }
}

/// One remote call: the expand query only. The session identity
/// (roster/local_node_id/epoch) is applied once per pooled connection by the
/// caller (006-P3), so the per-hop hot path is a single round trip.
async fn run_one_remote(
    client: &Client,
    request: &DistannRemoteExpandRequest<'_>,
    vec_ids_i64: &[i64],
) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
    let rows = scan_query(
        client,
        "expand call",
        EXPAND_SQL,
        &[
            &request.index_regclass,
            &request.epoch_fingerprint,
            &request.query,
            &vec_ids_i64,
            &request.code_threshold,
            &request.candidate_limit,
        ],
    )
    .await?;

    rows.into_iter()
        .map(|row| {
            let vec_id: i64 = row.try_get(0).map_err(row_err)?;
            let exact_dist: Option<f32> = row.try_get(1).map_err(row_err)?;
            let is_tombstone: bool = row.try_get(2).map_err(row_err)?;
            let neighbor_vec_ids: Vec<i64> = row.try_get(3).map_err(row_err)?;
            let neighbor_code_dists: Vec<f32> = row.try_get(4).map_err(row_err)?;
            Ok(DistannExpandedNode {
                vec_id: vec_id as u64,
                exact_dist,
                is_tombstone,
                heap_tid: ItemPointer::INVALID,
                neighbor_vec_ids: neighbor_vec_ids.into_iter().map(|v| v as u64).collect(),
                neighbor_code_dists,
                owner_total_ns: 0,
                owner_open_validate_ns: 0,
                owner_graph_read_ns: 0,
                owner_score_ns: 0,
                owner_response_encode_ns: 0,
                owner_response_bytes: 0,
                coordinator_rpc_ns: 0,
                coordinator_decode_ns: 0,
            })
        })
        .collect::<Result<Vec<_>, DistannExpandError>>()
}

fn row_err(error: tokio_postgres::Error) -> DistannExpandError {
    DistannExpandError::Internal(format!(
        "ec_distann remote transport row decode failed: {error}"
    ))
}

// Same `$1::text::regclass::oid` name→oid trick as EXPAND_SQL: the coordinator
// ships the index NAME and the requested projection columns + their typsend
// functions; the owner returns each owned row's projection as PG binary.
const MATERIALIZE_ROW_PAYLOADS_SQL: &str =
    "SELECT vec_id, is_tombstone, tuple_payload_missing, payload_nulls, payload_values \
     FROM ec_distann_materialize_row_payloads($1::text::regclass::oid, $2, $3::bigint[], \
     $4::text[], $5::text[])";

/// One remote row-payload materialization request over the transport (the
/// CustomScan read path). The requested projection columns and their typsend
/// functions are shared across owners, so they are passed to the batch call
/// once rather than per request.
pub(super) struct DistannRemoteMaterializeRequest<'a> {
    pub(super) conninfo: &'a str,
    pub(super) roster_spec: &'a str,
    pub(super) target_node_id: u32,
    pub(super) epoch: u64,
    pub(super) index_regclass: &'a str,
    pub(super) epoch_fingerprint: &'a [u8],
    /// Owned vec_ids for this node (bit-cast to i64 on the wire).
    pub(super) vec_ids: &'a [u64],
}

/// One owner-shipped row payload: the row's identity + tombstone plus the
/// requested projection columns as PostgreSQL binary (`typsend`) values.
pub(super) struct DistannMaterializedRow {
    pub(super) vec_id: u64,
    pub(super) is_tombstone: bool,
    pub(super) tuple_payload_missing: bool,
    pub(super) payload_nulls: Vec<bool>,
    pub(super) payload_values: Vec<Vec<u8>>,
}

pub(crate) struct DistannPhysicalMaterializeBatch {
    pub(crate) rows: Vec<DistannMaterializedRow>,
    #[cfg(feature = "distann-head-attribution-benchmark")]
    pub(crate) telemetry: DistannOwnerMaterializeTelemetry,
}

#[cfg(feature = "distann-head-attribution-benchmark")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DistannOwnerMaterializeTelemetry {
    pub(crate) owner_total_ns: u64,
    pub(crate) owner_open_validate_ns: u64,
    pub(crate) owner_node_lookup_ns: u64,
    pub(crate) owner_payload_sql_ns: u64,
    pub(crate) payload_bytes: u64,
}

/// Issue a batch of remote `ec_distann_materialize_row_payloads` calls — one per
/// owning node — concurrently over the pooled transport (reusing the warm
/// connections + session identity the hop-round expands already established).
/// Returns per-request results in request order; a connect/parse failure fails
/// every request uniformly.
pub(super) fn remote_materialize_row_payloads_batch(
    requests: &[DistannRemoteMaterializeRequest<'_>],
    payload_columns: &[String],
    send_functions: &[String],
) -> Vec<Result<Vec<DistannMaterializedRow>, DistannExpandError>> {
    if requests.is_empty() {
        return Vec::new();
    }
    let vec_ids_i64: Vec<Vec<i64>> = requests
        .iter()
        .map(|request| request.vec_ids.iter().map(|&v| v as i64).collect())
        .collect();
    let node_id_strs: Vec<String> = requests
        .iter()
        .map(|request| request.target_node_id.to_string())
        .collect();
    // 007-P1/021-P2: pool by (conninfo, target_node_id) — see the expand batch.
    let conn_keys: Vec<String> = requests
        .iter()
        .map(|request| format!("{}\u{1}{}", request.conninfo, request.target_node_id))
        .collect();
    let epoch_strs: Vec<String> = requests
        .iter()
        .map(|request| request.epoch.to_string())
        .collect();
    let columns = payload_columns.to_vec();
    let sends = send_functions.to_vec();

    let outcome = with_transport_state(|state| {
        let DistannTransportState {
            runtime,
            connections,
        } = state;
        runtime.block_on(async {
            let sessions = requests
                .iter()
                .enumerate()
                .map(|(index, request)| {
                    (
                        conn_keys[index].clone(),
                        request.conninfo,
                        (
                            request.roster_spec.to_owned(),
                            node_id_strs[index].clone(),
                            epoch_strs[index].clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>();
            ensure_scan_sessions(connections, &sessions).await?;

            let futures = requests.iter().enumerate().map(|(index, request)| {
                let client = &connections[&conn_keys[index]].client;
                run_one_materialize(client, request, &vec_ids_i64[index], &columns, &sends)
            });
            Ok::<_, String>(futures_util::future::join_all(futures).await)
        })
    });

    match outcome {
        Ok(results) => results,
        Err(message) => requests
            .iter()
            .map(|_| Err(DistannExpandError::Internal(message.clone())))
            .collect(),
    }
}

async fn run_one_materialize(
    client: &Client,
    request: &DistannRemoteMaterializeRequest<'_>,
    vec_ids_i64: &[i64],
    payload_columns: &[String],
    send_functions: &[String],
) -> Result<Vec<DistannMaterializedRow>, DistannExpandError> {
    let rows = scan_query(
        client,
        "row-payload call",
        MATERIALIZE_ROW_PAYLOADS_SQL,
        &[
            &request.index_regclass,
            &request.epoch_fingerprint,
            &vec_ids_i64,
            &payload_columns,
            &send_functions,
        ],
    )
    .await?;

    rows.into_iter()
        .map(|row| {
            let vec_id: i64 = row.try_get(0).map_err(row_err)?;
            let is_tombstone: bool = row.try_get(1).map_err(row_err)?;
            let tuple_payload_missing: bool = row.try_get(2).map_err(row_err)?;
            let payload_nulls: Vec<bool> = row.try_get(3).map_err(row_err)?;
            let payload_values: Vec<Vec<u8>> = row.try_get(4).map_err(row_err)?;
            Ok(DistannMaterializedRow {
                vec_id: vec_id as u64,
                is_tombstone,
                tuple_payload_missing,
                payload_nulls,
                payload_values,
            })
        })
        .collect::<Result<Vec<_>, DistannExpandError>>()
}

/// Debug/test surface: run the FR-081 orchestration for `query` against
/// `index_regclass`, selecting the local or remote expander from the active
/// roster (`ec_distann.roster`), and return the ranked hits. With an empty
/// roster this is the single-node path; with a multi-node roster it drives the
/// full group → transport → endpoint → reassemble remote path over loopback.
/// `index_name` is the regclass-castable name the remote nodes resolve (real
/// nodes have different oids, so the name — not the oid — is the wire handle).
///
/// This exists so TC-040/041 can assert 2-node top-k is identical to the
/// single-node build without the CustomScan row-materialization layer (which
/// returns remote heap rows to SQL and is a separable integration).
#[cfg(feature = "pg_test")]
#[pg_extern]
#[allow(clippy::type_complexity)]
fn ec_distann_debug_expand_search(
    index_regclass: pg_sys::Oid,
    index_name: &str,
    query: Vec<f32>,
    beam_width: i32,
    hop_rounds: i32,
    top_k: i32,
) -> TableIterator<'static, (name!(rank, i32), name!(vec_id, i64), name!(exact_dist, f32))> {
    let hits = debug_expand_search_impl(
        index_regclass,
        index_name,
        &query,
        beam_width.max(1) as usize,
        hop_rounds.max(1) as usize,
        top_k.max(1) as usize,
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let rows: Vec<(i32, i64, f32)> = hits
        .into_iter()
        .enumerate()
        .map(|(rank, hit)| (rank as i32, hit.vec_id as i64, hit.exact_dist))
        .collect();
    TableIterator::new(rows.into_iter())
}

fn debug_expand_search_impl(
    index_oid: pg_sys::Oid,
    index_name: &str,
    query: &[f32],
    beam_width: usize,
    hop_rounds: usize,
    top_k: usize,
) -> Result<Vec<DistannScanHit>, String> {
    let index_guard = IndexRelationGuard::try_access_share(index_oid)
        .ok_or_else(|| "ec_distann_debug_expand_search could not open the index".to_owned())?;
    let handle = NonNull::new(index_guard.as_ptr())
        .ok_or_else(|| "ec_distann_debug_expand_search got a null index".to_owned())?;
    let metadata = read_metadata_from_index_handle(handle)?;
    if metadata.dimensions == 0 || metadata.node_count == 0 {
        return Ok(Vec::new());
    }
    if query.len() != usize::from(metadata.dimensions) {
        return Err(format!(
            "ec_distann_debug_expand_search query dim {} != index dim {}",
            query.len(),
            metadata.dimensions
        ));
    }

    let entry = cached_index_entry(index_oid.into(), handle, &metadata)?;
    let prepared_query =
        DistannPreparedQuery::prepare(&metadata, entry.flat_codebooks.as_deref(), query)?;
    let code_len = metadata_code_len(&metadata)?;

    // FR-080 head-index descent → hop-round seeds (same rule as the scan).
    let head_list_size = (beam_width * 2)
        .max(32)
        .min(entry.head_vectors.len().max(1));
    let head_result = crate::am::greedy_search(
        &entry.head_graph,
        entry.head_entry,
        head_list_size,
        |node: u32| {
            -crate::am::ec_diskann::source_inner_product(query, &entry.head_vectors[node as usize])
        },
    );
    let seeds: Vec<DistannSeedCandidate> = head_result
        .frontier
        .iter()
        .map(|candidate| DistannSeedCandidate {
            vec_id: entry.head_vec_ids[candidate.node as usize],
            dist: candidate.distance,
        })
        .collect();

    let heap_oid = index_heap_relation_oid_handle(handle);
    let heap_guard = HeapRelationGuard::try_access_share(heap_oid)
        .ok_or_else(|| "ec_distann_debug_expand_search could not open the heap".to_owned())?;
    let heap_relation = heap_guard.as_ptr();
    let source_attnum = indexed_ecvector_attnum(index_guard.as_ptr())?;
    // SAFETY: SQL function invocation always runs under an active snapshot.
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        return Err("ec_distann_debug_expand_search has no active snapshot".to_owned());
    }
    // SAFETY: `heap_relation` is live for the duration of this call.
    let slot = unsafe { TupleTableSlotGuard::single_for_heap(heap_relation) }
        .ok_or_else(|| "ec_distann_debug_expand_search could not build a heap slot".to_owned())?;

    let params = DistannOrchestrationParams {
        beam_width,
        hop_rounds,
        top_k,
        debug_fail_hop_round: None,
    };
    let make_local = || LocalNodeExpander {
        index_handle: handle,
        directory: &entry.directory,
        graph_degree_r: metadata.graph_degree_r,
        code_len,
        prepared_query: &prepared_query,
        heap_relation,
        snapshot,
        slot: slot.as_ptr(),
        source_attnum,
        raw_query: query,
        pooled_node: DistannNodeTuple::placeholder(metadata.graph_degree_r, code_len),
    };

    // FR-082: debug expander agrees with owners on the PUBLISHED epoch.
    let scan_epoch_val = scan_epoch(&metadata);
    let directory = placement_directory_for_epoch(scan_epoch_val)?;
    let (hits, _counters) = if directory.node_count() > 1 {
        let local_index = directory
            .nodes
            .iter()
            .position(|node| node.is_local)
            .ok_or_else(|| "ec_distann_debug_expand_search: no local node in roster".to_owned())?;
        let identity = local_epoch_identity(&directory, &metadata);
        let fingerprint =
            compute_epoch_fingerprint(&identity, DISTANN_EPOCH_FINGERPRINT_V1).to_vec();
        let roster_spec = current_roster_spec();
        let epoch = scan_epoch_val;
        let mut expander = RemoteNodeExpander {
            local: make_local(),
            placement: &directory,
            local_index,
            index_regclass: index_name,
            epoch_fingerprint: &fingerprint,
            roster_spec: &roster_spec,
            epoch,
        };
        distann_orchestrated_search(&seeds, &mut expander, params).map_err(|e| e.to_string())?
    } else {
        let mut expander = make_local();
        distann_orchestrated_search(&seeds, &mut expander, params).map_err(|e| e.to_string())?
    };
    Ok(hits)
}

/// Coordinator-side implementation of the frozen expansion seam over the
/// roster (FR-078/FR-079/FR-081): group the beam batch by owning node, expand
/// this node's ids in-process (`LocalNodeExpander`) and every other node's ids
/// via one pooled `ec_distann_expand_nodes` call, then reassemble responses in
/// request order (FR-079-AC-1). The FR-081 orchestration loop is unchanged — it
/// sees a `DistannNodeExpander` and never learns the batch was split.
pub(crate) struct RemoteNodeExpander<'a> {
    /// In-process expander for this node's owned ids.
    pub(crate) local: LocalNodeExpander<'a>,
    /// Active roster + placement (FR-078).
    pub(crate) placement: &'a DistannPlacementDirectory,
    /// This node's roster index (owning-node index of the local node).
    pub(crate) local_index: usize,
    /// The index name every node resolves (regclass-castable).
    pub(crate) index_regclass: &'a str,
    /// The coordinator's active epoch fingerprint (sent to every node).
    pub(crate) epoch_fingerprint: &'a [u8],
    /// The roster spec + epoch echoed onto each remote session (loopback).
    pub(crate) roster_spec: &'a str,
    pub(crate) epoch: u64,
}

impl DistannNodeExpander for RemoteNodeExpander<'_> {
    fn expand_nodes(
        &mut self,
        vec_ids: &[u64],
        code_threshold: Option<f32>,
        candidate_limit: Option<usize>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
        let node_count = self.placement.node_count();
        // Position-carrying buckets: reassembly is driven by the original
        // request index, robust to a request that repeats a vec_id.
        let buckets = group_by_owning_node(vec_ids, node_count, self.placement.hash_version);

        let mut ordered: Vec<Option<DistannExpandedNode>> =
            (0..vec_ids.len()).map(|_| None).collect();

        // Expand this node's ids in-process first.
        if let Some(local_bucket) = buckets.get(self.local_index) {
            if !local_bucket.is_empty() {
                let ids: Vec<u64> = local_bucket.iter().map(|(_, vec_id)| *vec_id).collect();
                let responses = self
                    .local
                    .expand_nodes(&ids, code_threshold, candidate_limit)?;
                place_bucket_responses(self.local_index, local_bucket, responses, &mut ordered)?;
            }
        }

        // Build one request per remote owner and drive them concurrently.
        let query = self.local.raw_query;
        let remote_ids: Vec<(usize, Vec<u64>)> = buckets
            .iter()
            .enumerate()
            .filter(|(node_index, bucket)| *node_index != self.local_index && !bucket.is_empty())
            .map(|(node_index, bucket)| {
                (
                    node_index,
                    bucket.iter().map(|(_, vec_id)| *vec_id).collect(),
                )
            })
            .collect();
        if !remote_ids.is_empty() {
            let requests: Vec<DistannRemoteExpandRequest<'_>> = remote_ids
                .iter()
                .map(|(node_index, ids)| {
                    let node = &self.placement.nodes[*node_index];
                    DistannRemoteExpandRequest {
                        conninfo: &node.conninfo,
                        roster_spec: self.roster_spec,
                        target_node_id: node.node_id,
                        epoch: self.epoch,
                        index_regclass: self.index_regclass,
                        epoch_fingerprint: self.epoch_fingerprint,
                        query,
                        vec_ids: ids,
                        code_threshold,
                        candidate_limit: candidate_limit
                            .map(|limit| i32::try_from(limit).unwrap_or(i32::MAX)),
                    }
                })
                .collect();
            let results = remote_expand_batch(&requests);
            for ((node_index, _), responses) in remote_ids.iter().zip(results.into_iter()) {
                place_bucket_responses(
                    *node_index,
                    &buckets[*node_index],
                    responses?,
                    &mut ordered,
                )?;
            }
        }

        finalize_request_order(ordered)
    }
}

/// Place a node's responses (returned in the bucket's request order) back at
/// their original request positions. Endpoint responses preserve request order
/// (FR-079-AC-1), so `responses[k]` is `bucket[k]`'s answer.
fn place_bucket_responses(
    node_index: usize,
    bucket: &[(usize, u64)],
    responses: Vec<DistannExpandedNode>,
    ordered: &mut [Option<DistannExpandedNode>],
) -> Result<(), DistannExpandError> {
    if responses.len() != bucket.len() {
        return Err(DistannExpandError::Internal(format!(
            "ec_distann node {node_index} returned {} responses for {} requested ids \
             (FR-079-AC-1 coverage)",
            responses.len(),
            bucket.len()
        )));
    }
    for (&(orig_index, _), response) in bucket.iter().zip(responses.into_iter()) {
        ordered[orig_index] = Some(response);
    }
    Ok(())
}

/// Finalize the position-indexed responses into request order, erroring on any
/// gap (a requested id whose node dropped it — never a silent miss, FR-079).
fn finalize_request_order(
    ordered: Vec<Option<DistannExpandedNode>>,
) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
    ordered
        .into_iter()
        .enumerate()
        .map(|(index, response)| {
            response.ok_or_else(|| {
                DistannExpandError::Internal(format!(
                    "ec_distann remote expansion did not cover request position {index}"
                ))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_distann::placement::{placement_hash, DISTANN_PLACEMENT_HASH_V1};

    #[test]
    fn interrupt_poll_accepts_cancel_and_backend_termination() {
        assert!(!interrupt_flags_request_stop(0, 0, 0));
        assert!(!interrupt_flags_request_stop(1, 0, 0));
        assert!(!interrupt_flags_request_stop(0, 1, 0));
        assert!(interrupt_flags_request_stop(1, 1, 0));
        assert!(interrupt_flags_request_stop(0, 0, 1));
    }

    #[test]
    fn remote_await_enforces_client_deadline() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("test runtime");
        let outcome = runtime.block_on(await_remote(
            Duration::from_millis(1),
            None,
            std::future::pending::<Result<(), ()>>(),
        ));
        assert!(matches!(outcome, Err(RemoteAwaitError::TimedOut)));
    }

    #[test]
    fn remote_await_preserves_remote_error() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("test runtime");
        let outcome = runtime.block_on(await_remote(
            Duration::from_secs(1),
            None,
            std::future::ready(Err::<(), _>("remote")),
        ));
        assert!(matches!(outcome, Err(RemoteAwaitError::Remote("remote"))));
    }

    #[test]
    fn owner_futures_are_driven_concurrently() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("test runtime");
        let started = std::time::Instant::now();
        runtime.block_on(join_owner_futures((0..3).map(|_| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
        })));
        assert!(
            started.elapsed() < Duration::from_millis(230),
            "three owner futures ran serially instead of concurrently"
        );
    }

    #[test]
    fn parsed_remote_config_has_nonzero_connect_timeout() {
        let config =
            parse_remote_config("host=127.0.0.1 port=5432 dbname=postgres", "test transport")
                .expect("conninfo should parse");
        assert_eq!(
            config.get_connect_timeout().copied(),
            Some(connect_timeout())
        );
        assert!(!connect_timeout().is_zero());
    }

    #[test]
    fn conninfo_parse_error_is_redacted() {
        let error = parse_remote_config(
            "host=/secret dbname=private password=do_not_expose port=invalid",
            "test transport",
        )
        .expect_err("invalid port must fail parsing");
        for forbidden in ["/secret", "private", "do_not_expose"] {
            assert!(
                !error.contains(forbidden),
                "error leaked {forbidden}: {error}"
            );
        }
    }

    fn node(vec_id: u64) -> DistannExpandedNode {
        DistannExpandedNode {
            vec_id,
            exact_dist: Some(-(vec_id as f32)),
            is_tombstone: false,
            heap_tid: ItemPointer::INVALID,
            neighbor_vec_ids: vec![vec_id.wrapping_add(1)],
            neighbor_code_dists: vec![0.5],
            owner_total_ns: 0,
            owner_open_validate_ns: 0,
            owner_graph_read_ns: 0,
            owner_score_ns: 0,
            owner_response_encode_ns: 0,
            owner_response_bytes: 0,
            coordinator_rpc_ns: 0,
            coordinator_decode_ns: 0,
        }
    }

    // FR-079-AC-1: an interleaved request split across two owners, whose
    // per-node responses concatenate in node order, still reassembles into the
    // exact original request order (position-driven, not value-driven).
    #[test]
    fn reassembles_interleaved_request_across_owners() {
        // Pick ids and a 2-node roster; group them, then feed each bucket's
        // responses back in bucket order and check the final order.
        let vec_ids: Vec<u64> = (0..12)
            .map(|i| placement_hash(i, DISTANN_PLACEMENT_HASH_V1))
            .collect();
        let node_count = 2;
        let buckets = group_by_owning_node(&vec_ids, node_count, DISTANN_PLACEMENT_HASH_V1);
        // Require a genuine split so the test actually exercises interleaving.
        assert!(
            buckets.iter().all(|b| !b.is_empty()),
            "both owners populated"
        );

        let mut ordered: Vec<Option<DistannExpandedNode>> =
            (0..vec_ids.len()).map(|_| None).collect();
        for (node_index, bucket) in buckets.iter().enumerate() {
            let responses: Vec<DistannExpandedNode> =
                bucket.iter().map(|(_, id)| node(*id)).collect();
            place_bucket_responses(node_index, bucket, responses, &mut ordered).unwrap();
        }
        let final_order = finalize_request_order(ordered).expect("full coverage");
        assert_eq!(
            final_order.iter().map(|n| n.vec_id).collect::<Vec<_>>(),
            vec_ids,
            "final order equals original request order"
        );
    }

    // A missing response (a node dropped a requested id) is a fault, not a
    // silent gap.
    #[test]
    fn missing_coverage_is_an_error() {
        let ordered = vec![Some(node(1)), None, Some(node(3))];
        let error = finalize_request_order(ordered)
            .expect_err("must error on gap")
            .to_string();
        assert!(error.contains("did not cover"), "unexpected: {error}");
    }

    // A node returning the wrong count is rejected (coverage guard).
    #[test]
    fn wrong_response_count_is_an_error() {
        let bucket = vec![(0_usize, 10_u64), (1, 20)];
        let mut ordered = vec![None, None];
        let error = place_bucket_responses(0, &bucket, vec![node(10)], &mut ordered)
            .expect_err("count mismatch must error")
            .to_string();
        assert!(error.contains("coverage"), "unexpected: {error}");
    }
}
