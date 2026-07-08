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

use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;

use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern, pg_sys};
use tokio_postgres::{Client, NoTls};

use crate::storage::page::ItemPointer;
use crate::storage::relation_guard::{HeapRelationGuard, IndexRelationGuard};
use crate::storage::relation::index_heap_relation_oid_handle;
use crate::storage::slot_guard::TupleTableSlotGuard;

use super::ambuild::read_metadata_from_index_handle;
use super::epoch::{compute_epoch_fingerprint, DISTANN_EPOCH_FINGERPRINT_V1};
use super::expand::LocalNodeExpander;
use super::expand_error::DistannExpandError;
use super::head_cache::cached_index_entry;
use super::placement::{group_by_owning_node, DistannPlacementDirectory};
use super::quantizer::{metadata_code_len, DistannPreparedQuery};
use super::roster::{current_epoch, current_placement_directory, current_roster_spec, local_epoch_identity};
use super::routine::indexed_ecvector_attnum;
use super::scan::{
    distann_orchestrated_search, DistannExpandedNode, DistannNodeExpander,
    DistannOrchestrationParams, DistannScanHit, DistannSeedCandidate,
};
use super::tuple::DistannNodeTuple;

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
}

struct PooledConnection {
    client: Client,
    task: tokio::task::JoinHandle<()>,
    /// Reviewer 006-P3: the (roster, local_node_id, epoch) last applied to this
    /// pooled session via `set_config`. The expand hot path skips the setup
    /// round-trip when it is unchanged, and re-applies (epoch-aware) on a
    /// mismatch so a mid-backend epoch change is never served stale.
    applied_identity: Option<(String, String, String)>,
}

struct DistannTransportState {
    runtime: tokio::runtime::Runtime,
    connections: HashMap<String, PooledConnection>,
}

thread_local! {
    static DISTANN_TRANSPORT_STATE: RefCell<Option<DistannTransportState>> =
        const { RefCell::new(None) };
}

impl DistannTransportState {
    fn new() -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|_| "ec_distann remote transport failed to build the pooled runtime".to_owned())?;
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
    DISTANN_TRANSPORT_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.is_none() {
            *state = Some(DistannTransportState::new()?);
        }
        f(state.as_mut().expect("ec_distann transport state initialized"))
    })
}

// `$1::text::regclass::oid` forces PG to infer $1 as text (the coordinator
// sends the index NAME as a string); `$1::regclass` alone would infer a
// regclass-typed param that tokio-postgres cannot serialize from a &str.
const EXPAND_SQL: &str = "SELECT vec_id, exact_dist, is_tombstone, neighbor_vec_ids, \
    neighbor_code_dists FROM ec_distann_expand_nodes($1::text::regclass::oid, $2, $3::real[], \
    $4::bigint[], $5)";

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
            // Ensure every connection AND its session identity first — sequential
            // and off the per-hop hot path. 006-P3: applying set_config here
            // (once per (roster,node,epoch)) instead of before every expand
            // removes a round-trip per owner per hop and avoids racing session
            // GUCs against concurrent expands on a shared loopback connection.
            for (index, request) in requests.iter().enumerate() {
                let needs_connect = connections
                    .get(request.conninfo)
                    .map(|pooled| pooled.task.is_finished())
                    .unwrap_or(true);
                if needs_connect {
                    let config =
                        request.conninfo.parse::<tokio_postgres::Config>().map_err(|_| {
                            format!(
                                "ec_distann remote transport could not parse conninfo {:?}",
                                request.conninfo
                            )
                        })?;
                    let (client, connection) = config.connect(NoTls).await.map_err(|error| {
                        format!(
                            "ec_distann remote transport could not connect to {:?}: {error}",
                            request.conninfo
                        )
                    })?;
                    let task = tokio::spawn(async move {
                        let _ = connection.await;
                    });
                    connections.insert(
                        request.conninfo.to_owned(),
                        PooledConnection { client, task, applied_identity: None },
                    );
                }

                let identity = (
                    request.roster_spec.to_owned(),
                    node_id_strs[index].clone(),
                    epoch_strs[index].clone(),
                );
                let pooled = connections
                    .get_mut(request.conninfo)
                    .expect("connection just ensured");
                if pooled.applied_identity.as_ref() != Some(&identity) {
                    pooled
                        .client
                        .query(
                            SESSION_SETUP_SQL,
                            &[&identity.0, &identity.1, &identity.2],
                        )
                        .await
                        .map_err(|error| {
                            let detail = error
                                .as_db_error()
                                .map(|db| db.message().to_owned())
                                .unwrap_or_else(|| error.to_string());
                            format!(
                                "ec_distann remote transport session setup failed: {detail}"
                            )
                        })?;
                    pooled.applied_identity = Some(identity);
                }
            }

            // Fire all owners concurrently and await the whole set (expand only —
            // the session identity is already applied above).
            let futures = requests.iter().enumerate().map(|(index, request)| {
                let client = &connections[request.conninfo].client;
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
    let rows = client
        .query(
            EXPAND_SQL,
            &[
                &request.index_regclass,
                &request.epoch_fingerprint,
                &request.query,
                &vec_ids_i64,
                &request.code_threshold,
            ],
        )
        .await
        .map_err(|error| {
            // Classify by the remote endpoint's SQLSTATE so the coordinator can
            // distinguish a retriable epoch mismatch from non-retriable
            // placement/structural faults (FR-082-AC-2). Surface the remote
            // db-error message (tokio_postgres's Display is just "db error").
            let code = error.code().map(|state| state.code().to_owned());
            let detail = error
                .as_db_error()
                .map(|db| db.message().to_owned())
                .unwrap_or_else(|| error.to_string());
            DistannExpandError::from_wire_sqlstate(
                code.as_deref(),
                format!("ec_distann remote expand call failed: {detail}"),
            )
        })?;

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
            })
        })
        .collect::<Result<Vec<_>, DistannExpandError>>()
}

fn row_err(error: tokio_postgres::Error) -> DistannExpandError {
    DistannExpandError::Internal(format!(
        "ec_distann remote transport row decode failed: {error}"
    ))
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
#[pg_extern]
#[allow(clippy::type_complexity)]
fn ec_distann_debug_expand_search(
    index_regclass: pg_sys::Oid,
    index_name: &str,
    query: Vec<f32>,
    beam_width: i32,
    hop_rounds: i32,
    top_k: i32,
) -> TableIterator<
    'static,
    (
        name!(rank, i32),
        name!(vec_id, i64),
        name!(exact_dist, f32),
    ),
> {
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
    let head_list_size = (beam_width * 2).max(32).min(entry.head_vectors.len().max(1));
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

    let directory = current_placement_directory()?;
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
        let epoch = current_epoch();
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
                let responses = self.local.expand_nodes(&ids, code_threshold)?;
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
                (node_index, bucket.iter().map(|(_, vec_id)| *vec_id).collect())
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

    fn node(vec_id: u64) -> DistannExpandedNode {
        DistannExpandedNode {
            vec_id,
            exact_dist: Some(-(vec_id as f32)),
            is_tombstone: false,
            heap_tid: ItemPointer::INVALID,
            neighbor_vec_ids: vec![vec_id.wrapping_add(1)],
            neighbor_code_dists: vec![0.5],
        }
    }

    // FR-079-AC-1: an interleaved request split across two owners, whose
    // per-node responses concatenate in node order, still reassembles into the
    // exact original request order (position-driven, not value-driven).
    #[test]
    fn reassembles_interleaved_request_across_owners() {
        // Pick ids and a 2-node roster; group them, then feed each bucket's
        // responses back in bucket order and check the final order.
        let vec_ids: Vec<u64> =
            (0..12).map(|i| placement_hash(i, DISTANN_PLACEMENT_HASH_V1)).collect();
        let node_count = 2;
        let buckets =
            group_by_owning_node(&vec_ids, node_count, DISTANN_PLACEMENT_HASH_V1);
        // Require a genuine split so the test actually exercises interleaving.
        assert!(buckets.iter().all(|b| !b.is_empty()), "both owners populated");

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
