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
use pgrx::{default, name, pg_extern, pg_sys, Spi};

use crate::storage::relation::{
    index_heap_relation_oid_from_index_oid, index_heap_relation_oid_handle,
};
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
use super::reader::{directory_lookup, read_directory_from_relation, read_raw_tuple_bytes_from_relation};
use super::roster::{current_placement_directory, local_epoch_identity};
use super::routine::indexed_ecvector_attnum;
use super::scan::DistannNodeExpander;
use super::tuple::{
    DISTANN_FLAG_TOMBSTONE, DISTANN_NODE_FLAGS_OFFSET, DISTANN_NODE_HEAP_TID_OFFSET,
};
use crate::storage::page::ItemPointer;
use super::tuple::DistannNodeTuple;

/// One wire response row (no `heap_tid` — see module docs).
type ExpandRow = (i64, Option<f32>, bool, Vec<i64>, Vec<f32>);

/// FR-078 ownership surface: the roster index that owns `vec_id` under a given
/// node count and placement hash version. Exposed so operator tooling and the
/// multinode fixture can bucket vec_ids by owner (e.g. to drive a distributed
/// tombstone drill) without reimplementing the placement hash. Pure/deterministic.
#[pg_extern(immutable, parallel_safe)]
fn ec_distann_owning_node(vec_id: i64, node_count: i32, hash_version: i32) -> i32 {
    if node_count <= 0 {
        pgrx::error!("ec_distann_owning_node: node_count must be >= 1, got {node_count}");
    }
    let hash_version = u16::try_from(hash_version)
        .unwrap_or_else(|_| pgrx::error!("ec_distann_owning_node: hash_version out of range"));
    let owner = owning_node(vec_id as u64, node_count as usize, hash_version);
    i32::try_from(owner).unwrap_or_else(|_| pgrx::error!("ec_distann_owning_node: owner overflow"))
}

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
    // tombstone_by_vec_ids now returns a typed error (003-P3): an owned-but-
    // absent record is EC_RECORD_MISSING, structural/storage faults are Internal.
    let removed = unsafe { super::dml::tombstone_by_vec_ids(index_guard.as_ptr(), &ids) }?;
    Ok(i64::try_from(removed).unwrap_or(i64::MAX))
}

/// FR-079/005-P1 row-shipping endpoint: the owning node ships the heap identity
/// (ctid + tombstone flag) for each vec_id it owns, so a coordinator materializes
/// remote-owned hits from the OWNER rather than assuming it holds the full local
/// directory. Validates the caller's epoch (retriable mismatch) and every
/// vec_id's ownership (placement error) before any read — the same FR-079/FR-082
/// preflight as `ec_distann_expand_nodes`.
///
/// This is the data-path half of real multi-node materialization. Returning
/// remote rows through the executor still needs a CustomScan to yield the
/// shipped identity/row directly (a remote ctid is not fetchable from the
/// coordinator's local heap); that integration is the remaining M3 read-path
/// piece. On the co-located/loopback substrate the shipped ctid is locally
/// valid, so the coordinator's `amgettuple` path already completes the fetch.
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_materialize_rows(
    index_regclass: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    vec_ids: Vec<i64>,
) -> TableIterator<
    'static,
    (
        name!(vec_id, i64),
        name!(heap_block, i64),
        name!(heap_offset, i32),
        name!(is_tombstone, bool),
    ),
> {
    let rows = materialize_rows_impl(index_regclass, epoch_fingerprint, &vec_ids)
        .unwrap_or_else(|e| e.raise());
    TableIterator::new(rows)
}

/// Resolve each requested owned vec_id to its heap ctid + tombstone flag under
/// the FR-082 epoch + FR-078 ownership preflight the expand endpoint uses. This
/// is the shared owner-side resolution the ctid-shipping (`materialize_rows`) and
/// row-payload-shipping (`materialize_row_payloads`) endpoints both build on.
/// Response rows preserve request order and cover every requested vec_id
/// (owned-but-absent → `EC_RECORD_MISSING`, never a silent skip).
fn resolve_owned_rows(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    vec_ids: &[i64],
) -> Result<Vec<(i64, ItemPointer, bool)>, DistannExpandError> {
    let received_fingerprint =
        fingerprint_from_bytes(epoch_fingerprint).map_err(DistannExpandError::BadInput)?;
    let index_guard = IndexRelationGuard::try_access_share(index_oid)
        .ok_or_else(|| "ec_distann materialize could not open the index relation".to_owned())?;
    let handle = NonNull::new(index_guard.as_ptr())
        .ok_or_else(|| "ec_distann materialize got a null index relation".to_owned())?;
    let metadata = read_metadata_from_index_handle(handle)?;
    if metadata.node_count == 0 || metadata.directory_head == ItemPointer::INVALID {
        if vec_ids.is_empty() {
            return Ok(Vec::new());
        }
        return Err(DistannExpandError::OwnedRecordMissing(
            "ec_distann materialize: index is empty (no owned records)".to_owned(),
        ));
    }

    // FR-082 epoch validation (retriable on mismatch), before any read.
    let placement = current_placement_directory()?;
    let identity = local_epoch_identity(&placement, &metadata);
    let local_fingerprint = compute_epoch_fingerprint(&identity, DISTANN_EPOCH_FINGERPRINT_V1);
    if !fingerprints_match(&received_fingerprint, &local_fingerprint) {
        return Err(DistannExpandError::EpochMismatch(
            "ec_distann materialize epoch fingerprint mismatch".to_owned(),
        ));
    }

    let node_count = placement.node_count();
    let local_index = placement
        .nodes
        .iter()
        .position(|node| node.is_local)
        .ok_or_else(|| "ec_distann materialize: no local node in the roster".to_owned())?;
    for &vec_id in vec_ids {
        let owner = owning_node(vec_id as u64, node_count, placement.hash_version);
        if owner != local_index {
            return Err(DistannExpandError::Placement(format!(
                "ec_distann materialize placement error: vec_id {:#018x} owned by roster \
                 index {owner}, not this node ({local_index})",
                vec_id as u64
            )));
        }
    }
    if vec_ids.is_empty() {
        return Ok(Vec::new());
    }

    let directory =
        read_directory_from_relation(handle, metadata.directory_head, metadata.node_count as usize)
            .map_err(DistannExpandError::Internal)?;
    let mut rows = Vec::with_capacity(vec_ids.len());
    for &vec_id in vec_ids {
        // Owned-but-absent is EC_RECORD_MISSING (never a silent skip).
        let record_tid = directory_lookup(&directory, vec_id as u64).ok_or_else(|| {
            DistannExpandError::OwnedRecordMissing(format!(
                "ec_distann materialize: owned vec_id {:#018x} is not in the directory",
                vec_id as u64
            ))
        })?;
        let raw = read_raw_tuple_bytes_from_relation(handle, record_tid, "ec_distann materialize row")
            .map_err(DistannExpandError::Internal)?;
        let heap_tid = ItemPointer::decode(
            &raw[DISTANN_NODE_HEAP_TID_OFFSET..DISTANN_NODE_HEAP_TID_OFFSET + 6],
        )
        .map_err(DistannExpandError::Internal)?;
        let flags = u16::from_le_bytes(
            raw[DISTANN_NODE_FLAGS_OFFSET..DISTANN_NODE_FLAGS_OFFSET + 2]
                .try_into()
                .expect("flags bytes"),
        );
        let is_tombstone = flags & DISTANN_FLAG_TOMBSTONE != 0;
        rows.push((vec_id, heap_tid, is_tombstone));
    }
    Ok(rows)
}

fn materialize_rows_impl(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    vec_ids: &[i64],
) -> Result<Vec<(i64, i64, i32, bool)>, DistannExpandError> {
    Ok(resolve_owned_rows(index_oid, epoch_fingerprint, vec_ids)?
        .into_iter()
        .map(|(vec_id, heap_tid, is_tombstone)| {
            (
                vec_id,
                i64::from(heap_tid.block_number),
                i32::from(heap_tid.offset_number),
                is_tombstone,
            )
        })
        .collect())
}

/// One row-payload wire response row: the owning node's identity + tombstone plus
/// the requested projection columns as PostgreSQL binary (`typsend`) values.
type PayloadRow = (i64, bool, bool, Vec<bool>, Vec<Vec<u8>>);

/// FR-079/005-P1 row-payload shipping endpoint (the CustomScan data path). Where
/// `ec_distann_materialize_rows` ships only the ctid — unusable off the loopback
/// substrate because a remote ctid is not fetchable from the coordinator's local
/// heap — this ships the *row column data itself*: for each owned vec_id, the
/// owner resolves its heap ctid and encodes the requested projection columns to
/// PostgreSQL binary via each column's `typsend` function. The coordinator's
/// CustomScan reconstructs a virtual tuple from `payload_values` (via
/// `ReceiveFunctionCall`) and yields it directly, so a real multi-node scan
/// returns owner-owned SQL rows without a local directory or a local heap fetch.
///
/// `payload_columns` are the heap relation column names to project;
/// `payload_send_functions` are their binary send functions (parallel arrays,
/// supplied by the coordinator from the scan target list). Same FR-082 epoch +
/// FR-078 ownership preflight as the ctid endpoint (validated before any read).
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_materialize_row_payloads(
    index_regclass: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    vec_ids: Vec<i64>,
    payload_columns: Vec<String>,
    payload_send_functions: Vec<String>,
) -> TableIterator<
    'static,
    (
        name!(vec_id, i64),
        name!(is_tombstone, bool),
        name!(tuple_payload_missing, bool),
        name!(payload_nulls, Vec<bool>),
        name!(payload_values, Vec<Vec<u8>>),
    ),
> {
    let rows = materialize_row_payloads_impl(
        index_regclass,
        epoch_fingerprint,
        &vec_ids,
        &payload_columns,
        &payload_send_functions,
    )
    .unwrap_or_else(|e| e.raise());
    TableIterator::new(rows)
}

fn materialize_row_payloads_impl(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    vec_ids: &[i64],
    payload_columns: &[String],
    payload_send_functions: &[String],
) -> Result<Vec<PayloadRow>, DistannExpandError> {
    if payload_columns.len() != payload_send_functions.len() {
        return Err(DistannExpandError::BadInput(format!(
            "ec_distann_materialize_row_payloads: {} payload columns but {} send functions",
            payload_columns.len(),
            payload_send_functions.len()
        )));
    }
    // FR-082/FR-078 preflight + heap ctid resolution (shared with the ctid path).
    let resolved = resolve_owned_rows(index_oid, epoch_fingerprint, vec_ids)?;
    if resolved.is_empty() {
        return Ok(Vec::new());
    }

    let heap_oid = index_heap_relation_oid_from_index_oid(index_oid);
    // The heap relation name (schema-qualified, quoted) — a regclass value cannot
    // be a FROM-clause target, so the LATERAL join needs the resolved name.
    let heap_name = unsafe { heap_relation_qualified_name(heap_oid) }?;
    let sql = build_payload_sql(&heap_name, payload_columns, payload_send_functions)
        .map_err(DistannExpandError::BadInput)?;
    // Candidate ctids in resolved (request) order; the SQL preserves that order
    // via WITH ORDINALITY so responses zip 1:1 back onto `resolved`.
    let ctid_texts: Vec<String> = resolved
        .iter()
        .map(|(_, tid, _)| format!("({},{})", tid.block_number, tid.offset_number))
        .collect();
    let column_count = payload_columns.len();
    let ctid_refs: Vec<&str> = ctid_texts.iter().map(String::as_str).collect();

    let payloads: Vec<(bool, Vec<bool>, Vec<Vec<u8>>)> = Spi::connect(|client| {
        let rows = client
            .select(sql.as_str(), None, &[ctid_refs.as_slice().into()])
            .map_err(|e| format!("ec_distann materialize row payload heap fetch failed: {e}"))?;
        let mut out = Vec::with_capacity(ctid_texts.len());
        for row in rows {
            let tuple_payload_missing = row["tuple_payload_missing"]
                .value::<bool>()
                .map_err(|e| format!("ec_distann row payload missing-flag decode failed: {e}"))?
                .ok_or_else(|| "ec_distann row payload missing flag is null".to_owned())?;
            let payload_nulls = row["payload_nulls"]
                .value::<Vec<bool>>()
                .map_err(|e| format!("ec_distann row payload nulls decode failed: {e}"))?
                .unwrap_or_default();
            let payload_values = row["payload_values"]
                .value::<pgrx::datum::Array<&[u8]>>()
                .map_err(|e| format!("ec_distann row payload values decode failed: {e}"))?
                .map(|array| array.iter_deny_null().map(<[u8]>::to_vec).collect::<Vec<_>>())
                .unwrap_or_default();
            if payload_nulls.len() != column_count || payload_values.len() != column_count {
                return Err(format!(
                    "ec_distann row payload returned {} null flags and {} values for {} columns",
                    payload_nulls.len(),
                    payload_values.len(),
                    column_count
                ));
            }
            out.push((tuple_payload_missing, payload_nulls, payload_values));
        }
        Ok(out)
    })
    .map_err(DistannExpandError::Internal)?;

    if payloads.len() != resolved.len() {
        return Err(DistannExpandError::Internal(format!(
            "ec_distann row payload returned {} rows for {} requested ctids",
            payloads.len(),
            resolved.len()
        )));
    }

    Ok(resolved
        .into_iter()
        .zip(payloads)
        .map(
            |((vec_id, _tid, is_tombstone), (missing, nulls, values))| {
                (vec_id, is_tombstone, missing, nulls, values)
            },
        )
        .collect())
}

/// Build the owner-side per-ctid projection SQL. Each requested column becomes a
/// `payload_nulls` boolean and a `payload_values` bytea (its `typsend` binary, or
/// `''::bytea` when the row/column is NULL or the ctid resolves to no live tuple).
/// Column names are double-quote escaped; send functions are validated to plain
/// (optionally schema-qualified) identifiers so the projection cannot inject SQL.
/// Resolve a heap relation's **schema-qualified, quoted** name from its oid, for
/// safe interpolation as a FROM-clause target. Mirrors `distann_index_relname`
/// but works from the oid via the syscache (no open relation needed).
unsafe fn heap_relation_qualified_name(
    heap_oid: pg_sys::Oid,
) -> Result<String, DistannExpandError> {
    let name_ptr = pg_sys::get_rel_name(heap_oid);
    if name_ptr.is_null() {
        return Err(DistannExpandError::Internal(format!(
            "ec_distann materialize: heap relation oid {} has no name (dropped?)",
            u32::from(heap_oid)
        )));
    }
    let relname = std::ffi::CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
    let nsp_oid = pg_sys::get_rel_namespace(heap_oid);
    let nsp_ptr = pg_sys::get_namespace_name(nsp_oid);
    if nsp_ptr.is_null() {
        return Err(DistannExpandError::Internal(format!(
            "ec_distann materialize: heap relation oid {} has no namespace",
            u32::from(heap_oid)
        )));
    }
    let nspname = std::ffi::CStr::from_ptr(nsp_ptr).to_string_lossy().into_owned();
    Ok(format!("{}.{}", quote_ident(&nspname), quote_ident(&relname)))
}

fn build_payload_sql(
    heap_relation: &str,
    payload_columns: &[String],
    payload_send_functions: &[String],
) -> Result<String, String> {
    let mut null_exprs = Vec::with_capacity(payload_columns.len());
    let mut value_exprs = Vec::with_capacity(payload_columns.len());
    let mut projected = Vec::with_capacity(payload_columns.len());
    for (column, send_function) in payload_columns.iter().zip(payload_send_functions) {
        let ident = quote_ident(column);
        let send = validate_send_function(send_function)?;
        projected.push(format!("heap_row.{ident} AS {ident}"));
        null_exprs.push(format!("(heap.__ec_distann_found IS NULL OR heap.{ident} IS NULL)"));
        value_exprs.push(format!(
            "CASE WHEN heap.__ec_distann_found IS NULL OR heap.{ident} IS NULL \
                  THEN ''::bytea ELSE {send}(heap.{ident}) END"
        ));
    }
    let found_projection = if projected.is_empty() {
        "true AS __ec_distann_found".to_owned()
    } else {
        format!("true AS __ec_distann_found, {}", projected.join(", "))
    };
    let null_array = if null_exprs.is_empty() {
        "ARRAY[]::boolean[]".to_owned()
    } else {
        format!("ARRAY[{}]::boolean[]", null_exprs.join(", "))
    };
    let value_array = if value_exprs.is_empty() {
        "ARRAY[]::bytea[]".to_owned()
    } else {
        format!("ARRAY[{}]::bytea[]", value_exprs.join(", "))
    };
    Ok(format!(
        "SELECT heap.__ec_distann_found IS NULL AS tuple_payload_missing, \
                {null_array} AS payload_nulls, \
                {value_array} AS payload_values \
           FROM unnest($1::text[]) WITH ORDINALITY AS candidate(ctid_text, ordinality) \
           LEFT JOIN LATERAL ( \
             SELECT {found_projection} \
               FROM {heap_relation} AS heap_row \
              WHERE heap_row.ctid = candidate.ctid_text::tid \
           ) AS heap ON true \
          ORDER BY candidate.ordinality"
    ))
}

/// Double-quote-escape a heap column identifier for safe interpolation.
fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

/// A binary send function name supplied by the coordinator must be a plain SQL
/// identifier, optionally schema-qualified (`pg_catalog.int8send`). Reject
/// anything else so the projection SQL cannot be used as an injection vector.
fn validate_send_function(name: &str) -> Result<String, String> {
    let is_ident = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !part.chars().next().unwrap().is_ascii_digit()
    };
    let ok = match name.split_once('.') {
        Some((schema, func)) => is_ident(schema) && is_ident(func),
        None => is_ident(name),
    };
    if ok {
        Ok(name.to_owned())
    } else {
        Err(format!(
            "ec_distann_materialize_row_payloads: invalid send function name {name:?}"
        ))
    }
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
