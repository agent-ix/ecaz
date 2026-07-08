//! FR-079 `ec_distann_expand_nodes` remote expansion endpoint (Task 164 M2).
//!
//! Each data node exposes this SQL function; the coordinator calls it once per
//! owning node per hop round (FR-081) over the pooled transport. It is the
//! remote form of the frozen expansion seam: given a batch of locally-owned
//! vec_ids it performs, per id, one index-record read (neighbor scoring from
//! the embedded codes) plus one co-placed heap read for the exact distance
//! (ADR-085 D11) — the same `LocalNodeExpander` the single-node scan uses.
//!
//! The wire contract is independent of the record layout (ADR-085 D1) and does
//! NOT carry `heap_tid` (that is a local-only materialization handle). Response
//! rows preserve request order and cover every requested vec_id (FR-079-AC-1).
//!
//! Four outcomes (FR-079): (a) present → row with `exact_dist` set; (b) not
//! owned by this node under the epoch placement → placement error; (c) record
//! owned but absent → structural fault; (d) record present but its co-placed
//! vector unreadable → distinct structural fault. Errors, never empty results.
//! The caller's `epoch_fingerprint` is validated before any read; a mismatch is
//! a retriable epoch error (FR-082 subset).

use std::ptr::NonNull;

use pgrx::iter::TableIterator;
use pgrx::{default, name, pg_extern, pg_sys};

use crate::storage::relation::index_heap_relation_oid_handle;
use crate::storage::relation_guard::{HeapRelationGuard, IndexRelationGuard};
use crate::storage::slot_guard::TupleTableSlotGuard;

use super::ambuild::read_metadata_from_index_handle;
use super::epoch::{
    compute_epoch_fingerprint, fingerprint_from_bytes, fingerprints_match,
    DISTANN_EPOCH_FINGERPRINT_V1,
};
use super::expand::LocalNodeExpander;
use super::expand_error::DistannExpandError;
use super::head_cache::cached_index_entry;
use super::placement::owning_node;
use super::quantizer::{metadata_code_len, DistannPreparedQuery};
use super::roster::{current_placement_directory, local_epoch_identity};
use super::routine::indexed_ecvector_attnum;
use super::scan::DistannNodeExpander;
use super::tuple::DistannNodeTuple;

/// One wire response row (no `heap_tid` — see module docs).
type ExpandRow = (i64, Option<f32>, bool, Vec<i64>, Vec<f32>);

/// FR-079 endpoint. `index_regclass` is accepted as its `oid` (a `regclass`
/// literal casts implicitly), matching how the coordinator resolves the local
/// index. `epoch_fingerprint` is the coordinator's active-epoch identity
/// (FR-082); `code_threshold` defaults to NULL (no pruning — the only mode with
/// the FR-081 early-exit result guarantee).
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_expand_nodes(
    index_regclass: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    query: Vec<f32>,
    vec_ids: Vec<i64>,
    code_threshold: default!(Option<f32>, "NULL"),
) -> TableIterator<
    'static,
    (
        name!(vec_id, i64),
        name!(exact_dist, Option<f32>),
        name!(is_tombstone, bool),
        name!(neighbor_vec_ids, Vec<i64>),
        name!(neighbor_code_dists, Vec<f32>),
    ),
> {
    let rows = expand_nodes_impl(
        index_regclass,
        epoch_fingerprint,
        &query,
        &vec_ids,
        code_threshold,
    )
    // Raise with the distinct SQLSTATE per FR-079 outcome so the coordinator
    // classifies retriable epoch mismatch vs non-retriable faults by code.
    .unwrap_or_else(|e| e.raise());
    TableIterator::new(rows.into_iter())
}

/// Compute this node's active-epoch fingerprint for `index_regclass` under the
/// current roster/epoch GUCs. The coordinator calls this to obtain the
/// fingerprint it then passes to every `ec_distann_expand_nodes` call so all
/// participants agree on the epoch (FR-082 subset). Also the operator/test
/// surface for inspecting epoch identity.
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_epoch_fingerprint(index_regclass: pg_sys::Oid) -> Vec<u8> {
    epoch_fingerprint_impl(index_regclass).unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn epoch_fingerprint_impl(index_oid: pg_sys::Oid) -> Result<Vec<u8>, String> {
    let index_guard = IndexRelationGuard::try_access_share(index_oid).ok_or_else(|| {
        "ec_distann_epoch_fingerprint could not open the index relation".to_owned()
    })?;
    let handle = NonNull::new(index_guard.as_ptr())
        .ok_or_else(|| "ec_distann_epoch_fingerprint got a null index relation".to_owned())?;
    let metadata = read_metadata_from_index_handle(handle)?;
    let directory = current_placement_directory()?;
    let identity = local_epoch_identity(&directory, &metadata);
    Ok(compute_epoch_fingerprint(&identity, DISTANN_EPOCH_FINGERPRINT_V1).to_vec())
}

/// FR-083 remote write endpoint (write counterpart to FR-079): apply
/// record-level writes on the hash-owning node under epoch-fingerprint
/// validation. This slice implements the **tombstone-set** operation (the
/// coordinator's delete routes the tombstone to the owning node); new-record
/// append + back-edge amendment (M5 incremental insert) are later. Validates
/// the caller's epoch (retriable mismatch) and every vec_id's ownership
/// (placement error) before any write, exactly like FR-079. Returns the count
/// newly tombstoned.
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_apply_record_writes(
    index_regclass: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    tombstone_vec_ids: Vec<i64>,
) -> i64 {
    apply_record_writes_impl(index_regclass, epoch_fingerprint, &tombstone_vec_ids)
        .unwrap_or_else(|e| e.raise())
}

fn apply_record_writes_impl(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    tombstone_vec_ids: &[i64],
) -> Result<i64, DistannExpandError> {
    let received_fingerprint =
        fingerprint_from_bytes(epoch_fingerprint).map_err(DistannExpandError::BadInput)?;
    // Writes need a RowExclusive lock (tombstone flips are exclusive-buffer).
    let index_guard = IndexRelationGuard::open(
        index_oid,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        "ec_distann_apply_record_writes",
    );
    let handle = NonNull::new(index_guard.as_ptr())
        .ok_or_else(|| "ec_distann_apply_record_writes got a null index relation".to_owned())?;
    let metadata = read_metadata_from_index_handle(handle)?;

    // FR-082 epoch validation (retriable on mismatch), before any write.
    let directory = current_placement_directory()?;
    let identity = local_epoch_identity(&directory, &metadata);
    let local_fingerprint = compute_epoch_fingerprint(&identity, DISTANN_EPOCH_FINGERPRINT_V1);
    if !fingerprints_match(&received_fingerprint, &local_fingerprint) {
        return Err(DistannExpandError::EpochMismatch(
            "ec_distann_apply_record_writes epoch fingerprint mismatch".to_owned(),
        ));
    }

    // FR-078 ownership: every write target must be owned by this node.
    let node_count = directory.node_count();
    let local_index = directory
        .nodes
        .iter()
        .position(|node| node.is_local)
        .ok_or_else(|| {
            "ec_distann_apply_record_writes: no local node in the active roster".to_owned()
        })?;
    let ids: Vec<u64> = tombstone_vec_ids.iter().map(|&v| v as u64).collect();
    for &vec_id in &ids {
        if owning_node(vec_id, node_count, directory.hash_version) != local_index {
            return Err(DistannExpandError::Placement(format!(
                "ec_distann_apply_record_writes placement error: vec_id {vec_id:#018x} not owned \
                 by this node (roster index {local_index})"
            )));
        }
    }

    // SAFETY: the guard holds the index open for write for the call.
    let removed = unsafe { super::dml::tombstone_by_vec_ids(index_guard.as_ptr(), &ids) }
        .map_err(DistannExpandError::OwnedRecordMissing)?;
    Ok(i64::try_from(removed).unwrap_or(i64::MAX))
}

fn expand_nodes_impl(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    query: &[f32],
    vec_ids: &[i64],
    code_threshold: Option<f32>,
) -> Result<Vec<ExpandRow>, DistannExpandError> {
    let received_fingerprint =
        fingerprint_from_bytes(epoch_fingerprint).map_err(DistannExpandError::BadInput)?;

    let index_guard = IndexRelationGuard::try_access_share(index_oid)
        .ok_or_else(|| "ec_distann_expand_nodes could not open the index relation".to_owned())?;
    let handle = NonNull::new(index_guard.as_ptr())
        .ok_or_else(|| "ec_distann_expand_nodes got a null index relation".to_owned())?;
    let metadata = read_metadata_from_index_handle(handle)?;

    if metadata.dimensions == 0 || metadata.node_count == 0 {
        // Empty index owns nothing; any requested id is a structural fault
        // (owned-but-absent). If none requested, an empty response is correct.
        if vec_ids.is_empty() {
            return Ok(Vec::new());
        }
        return Err(DistannExpandError::OwnedRecordMissing(
            "ec_distann_expand_nodes: index is empty (no owned records)".to_owned(),
        ));
    }
    if query.len() != usize::from(metadata.dimensions) {
        return Err(DistannExpandError::BadInput(format!(
            "ec_distann_expand_nodes query dimension {} != index dimension {}",
            query.len(),
            metadata.dimensions
        )));
    }

    // FR-082 (subset): validate the caller's epoch fingerprint against this
    // node's active-epoch identity before any read. Mismatch is retriable.
    let directory = current_placement_directory()?;
    let identity = local_epoch_identity(&directory, &metadata);
    let local_fingerprint = compute_epoch_fingerprint(&identity, DISTANN_EPOCH_FINGERPRINT_V1);
    if !fingerprints_match(&received_fingerprint, &local_fingerprint) {
        return Err(DistannExpandError::EpochMismatch(
            "ec_distann_expand_nodes epoch fingerprint mismatch: caller epoch differs from this \
             node's active epoch"
                .to_owned(),
        ));
    }

    // FR-079 case (b): every requested vec_id must be owned by THIS node under
    // the epoch placement; a non-owned id is a placement error, never a miss.
    let node_count = directory.node_count();
    let local_index = directory
        .nodes
        .iter()
        .position(|node| node.is_local)
        .ok_or_else(|| "ec_distann_expand_nodes: no local node in the active roster".to_owned())?;
    for &vec_id in vec_ids {
        let owner = owning_node(vec_id as u64, node_count, directory.hash_version);
        if owner != local_index {
            return Err(DistannExpandError::Placement(format!(
                "ec_distann_expand_nodes placement error: vec_id {:#018x} is owned by roster \
                 index {owner}, not this node (roster index {local_index})",
                vec_id as u64
            )));
        }
    }

    if vec_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Query-independent scan state (directory + codebooks). Reuses the head
    // cache; the head graph it also builds is unused by the endpoint (the
    // remote node never does head descent) — a known M2 first-call cost.
    let entry = cached_index_entry(index_oid.into(), handle, &metadata)?;
    let prepared_query =
        DistannPreparedQuery::prepare(&metadata, entry.flat_codebooks.as_deref(), query)?;
    let code_len = metadata_code_len(&metadata)?;

    let heap_oid = index_heap_relation_oid_handle(handle);
    let heap_guard = HeapRelationGuard::try_access_share(heap_oid).ok_or_else(|| {
        "ec_distann_expand_nodes could not open the co-placed heap relation".to_owned()
    })?;
    let heap_relation = heap_guard.as_ptr();
    let source_attnum = indexed_ecvector_attnum(index_guard.as_ptr())?;
    // SAFETY: called inside a SQL function invocation, which always runs under
    // an active snapshot; the pointer is valid for this call.
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        return Err(DistannExpandError::Internal(
            "ec_distann_expand_nodes has no active snapshot".to_owned(),
        ));
    }
    // SAFETY: `heap_relation` is a live relation from `heap_guard`, held open
    // for the duration of this call.
    let slot = unsafe { TupleTableSlotGuard::single_for_heap(heap_relation) }
        .ok_or_else(|| "ec_distann_expand_nodes could not build a heap tuple slot".to_owned())?;

    let vec_ids_u64: Vec<u64> = vec_ids.iter().map(|&v| v as u64).collect();
    let mut expander = LocalNodeExpander {
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

    let responses = expander.expand_nodes(&vec_ids_u64, code_threshold)?;
    Ok(responses
        .into_iter()
        .map(|response| {
            (
                response.vec_id as i64,
                response.exact_dist,
                response.is_tombstone,
                response
                    .neighbor_vec_ids
                    .iter()
                    .map(|&v| v as i64)
                    .collect::<Vec<i64>>(),
                response.neighbor_code_dists,
            )
        })
        .collect())
}
