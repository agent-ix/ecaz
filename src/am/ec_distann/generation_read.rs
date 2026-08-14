//! Published physical-generation reader for Task 179.
//!
//! The logical control index is metadata-only. This module resolves its active
//! pointer to the immutable generation heap relations, pins the exact epoch in
//! the shared scan registry, and adapts those relations to the existing FR-081
//! orchestration seam.

use std::cell::RefCell;
#[cfg(feature = "pg_test")]
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::ptr;
#[cfg(feature = "distann-head-attribution-benchmark")]
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
#[cfg(feature = "distann-head-attribution-benchmark")]
use std::time::Instant;

use pgrx::datum::Uuid;
use pgrx::iter::TableIterator;
#[cfg(feature = "distann-head-attribution-benchmark")]
use pgrx::spi::OwnedPreparedStatement;
use pgrx::{default, name, pg_extern, pg_sys, PgRelation, Spi};
#[cfg(feature = "distann-head-attribution-benchmark")]
use sha2::{Digest, Sha256};

use crate::storage::page::ItemPointer;
use crate::storage::relation_guard::{HeapRelationGuard, IndexRelationGuard};
use crate::storage::scan_guard::IndexScanGuard;
use crate::storage::slot_guard::TupleTableSlotGuard;
use crate::storage::snapshot_guard::RegisteredSnapshotGuard;

use super::expand_error::DistannExpandError;
use super::generation_catalog::{self, GenerationCatalogRow};
use super::generation_descriptor::DistannGenerationDescriptor;
use super::quantizer::{DistannCodecBinding, DistannPreparedQuery};
use super::routine::DistannHitCollection;
use super::scan::{
    distann_orchestrated_search, DistannExpandedNode, DistannNodeExpander,
    DistannOrchestrationParams, DistannScanHit, DistannSeedCandidate,
};
use super::scan_registry::ScanTokenGuard;
use super::tuple::DistannNodeTuple;

pub(crate) struct ActiveGenerationIdentity {
    pub(crate) build_id: Uuid,
    pub(crate) fingerprint: [u8; 34],
}

pub(crate) fn emit_scan_profile_notice(
    counters: &super::scan::DistannScanCounters,
    top_k: usize,
    head_seed_count: usize,
    result_count: usize,
) {
    if !super::options::scan_profile_notice_enabled() {
        return;
    }
    pgrx::notice!(
        "ec_distann_scan_profile beam_width={} hop_rounds={} top_k={} head_seed_count={} rounds_executed={} records_expanded={} neighbors_code_scored={} early_exit={} beam_exhausted={} result_count={}",
        super::options::current_beam_width(),
        super::options::current_hop_rounds(),
        top_k,
        head_seed_count,
        counters.rounds_executed,
        counters.records_expanded,
        counters.neighbors_code_scored,
        counters.early_exit,
        counters.beam_exhausted,
        result_count,
    );
    for round in &counters.rounds {
        // The physical expander reports requested response slots here, not
        // a measured count of records actually expanded. Keep that distinction
        // visible instead of publishing a misleading numeric zero/count.
        let expanded_nodes = "unmeasured";
        let transport_wait_ns = if cfg!(feature = "distann-head-attribution-benchmark") {
            round.transport_wait_ns.to_string()
        } else {
            "absent".to_owned()
        };
        let straggler_spread_ns = if cfg!(feature = "distann-head-attribution-benchmark") {
            round.straggler_spread_ns.to_string()
        } else {
            "absent".to_owned()
        };
        let request_bytes = if cfg!(feature = "distann-head-attribution-benchmark") {
            round.request_bytes.to_string()
        } else {
            "absent".to_owned()
        };
        let response_bytes = if cfg!(feature = "distann-head-attribution-benchmark") {
            round.response_bytes.to_string()
        } else {
            "absent".to_owned()
        };
        pgrx::notice!(
            "ec_distann_scan_round round={} requested_nodes={} expanded_nodes={} transport_wait_ns={} straggler_spread_ns={} request_bytes={} response_bytes={}",
            round.round,
            round.requested_nodes,
            expanded_nodes,
            transport_wait_ns,
            straggler_spread_ns,
            request_bytes,
            response_bytes,
        );
    }
}

fn graph_slot_attr(
    slot: &TupleTableSlotGuard<'_>,
    attnum: i32,
    label: &str,
) -> Result<pg_sys::Datum, DistannExpandError> {
    let mut is_null = false;
    let datum = unsafe { pg_sys::slot_getattr(slot.as_ptr(), attnum, &mut is_null) };
    if is_null {
        return Err(DistannExpandError::GenerationMissing(format!(
            "physical graph {label} is NULL"
        )));
    }
    Ok(datum)
}

fn graph_node_from_slot(
    slot: &TupleTableSlotGuard<'_>,
    graph_degree: u16,
    code_len: usize,
) -> Result<DistannNodeTuple, DistannExpandError> {
    let stored_id = unsafe { pg_sys::DatumGetInt64(graph_slot_attr(slot, 1, "vec_id")?) };
    let record = unsafe {
        crate::am::common::detoast::DetoastedVarlena::packed_from_datum(graph_slot_attr(
            slot, 2, "record",
        )?)
    }
    .ok_or_else(|| {
        DistannExpandError::GenerationMissing(
            "physical graph record could not be detoasted".to_owned(),
        )
    })?;
    let row_tid_datum = graph_slot_attr(slot, 3, "row TID")?;
    let row_tid_ptr = row_tid_datum.cast_mut_ptr::<pg_sys::ItemPointerData>();
    if row_tid_ptr.is_null() {
        return Err(DistannExpandError::GenerationMissing(
            "physical graph row TID pointer is NULL".to_owned(),
        ));
    }
    let row_tid = unsafe { ptr::read_unaligned(row_tid_ptr) };
    let node = DistannNodeTuple::decode_physical_v1(record.as_bytes(), graph_degree, code_len)
        .map_err(DistannExpandError::GenerationMissing)?;
    let (block, offset) = pgrx::itemptr::item_pointer_get_both(row_tid);
    if node.vec_id != u64::from_le_bytes(stored_id.to_le_bytes())
        || node.heap_tid
            != (ItemPointer {
                block_number: block,
                offset_number: offset,
            })
    {
        return Err(DistannExpandError::GenerationMissing(
            "physical graph row identity/locator mismatch".to_owned(),
        ));
    }
    Ok(node)
}

/// Read immutable graph tuples through the generation's unique `vec_id`
/// directory. This keeps hop expansion out of SPI: no per-hop connection,
/// relation-name SQL formatting, parse/plan, or SPI tuple copy is required.
fn lookup_graph_nodes<F>(
    graph_relation: &HeapRelationGuard,
    directory_relation: &IndexRelationGuard,
    snapshot: pg_sys::Snapshot,
    vec_ids: &[u64],
    graph_degree: u16,
    code_len: usize,
    missing: F,
) -> Result<HashMap<u64, DistannNodeTuple>, DistannExpandError>
where
    F: Fn(u64) -> DistannExpandError,
{
    if snapshot.is_null() {
        return Err(DistannExpandError::Internal(
            "physical graph lookup has no active snapshot".to_owned(),
        ));
    }
    let scan = unsafe {
        IndexScanGuard::begin_from_raw(
            graph_relation.as_ptr(),
            directory_relation.as_ptr(),
            snapshot,
            1,
            0,
        )
    }
    .ok_or_else(|| {
        DistannExpandError::Internal("could not begin physical graph index scan".to_owned())
    })?;
    let slot = TupleTableSlotGuard::create_for_heap_guard(graph_relation).ok_or_else(|| {
        DistannExpandError::Internal("could not allocate physical graph scan slot".to_owned())
    })?;
    let mut records = HashMap::with_capacity(vec_ids.len());
    for vec_id in vec_ids {
        if records.contains_key(vec_id) {
            continue;
        }
        let stored_id = i64::from_le_bytes(vec_id.to_le_bytes());
        let mut scan_key =
            unsafe { std::mem::MaybeUninit::<pg_sys::ScanKeyData>::zeroed().assume_init() };
        unsafe {
            pg_sys::ScanKeyInit(
                &mut scan_key,
                1,
                pg_sys::BTEqualStrategyNumber as pg_sys::StrategyNumber,
                pg_sys::Oid::from(pg_sys::F_INT8EQ),
                pg_sys::Datum::from(stored_id),
            );
            pg_sys::index_rescan(scan.as_ptr(), &mut scan_key, 1, ptr::null_mut(), 0);
            pg_sys::ExecClearTuple(slot.as_ptr());
        }
        let found = unsafe {
            pg_sys::index_getnext_slot(
                scan.as_ptr(),
                pg_sys::ScanDirection::ForwardScanDirection,
                slot.as_ptr(),
            )
        };
        if !found {
            return Err(missing(*vec_id));
        }
        let node = graph_node_from_slot(&slot, graph_degree, code_len)?;
        if node.vec_id != *vec_id {
            return Err(DistannExpandError::GenerationMissing(format!(
                "physical graph directory returned vec_id {:#018x} for requested {vec_id:#018x}",
                node.vec_id
            )));
        }
        records.insert(node.vec_id, node);
    }
    Ok(records)
}

/// A remote physical append/backlink is allowed to make a retained edge
/// visible before its owner graph tuple becomes visible to this backend's
/// snapshot. The intent row is the owner-local, transaction-independent fence
/// for that narrow 2PC window. Keep the predicate recent: terminal cleanup
/// deliberately retains rows for audit, and an old unresolved row must not
/// turn every later missing-record error into a retry budget.
fn recent_remote_insert_intent(
    _index_oid: pg_sys::Oid,
    node_ids: &[u32],
    vec_ids: &[u64],
    epoch: u64,
) -> Result<bool, DistannExpandError> {
    let node_ids = node_ids
        .iter()
        .map(|node_id| {
            i32::try_from(*node_id)
                .map(|value| value.to_string())
                .map_err(|_| {
                    DistannExpandError::Internal("physical intent node id exceeds int4".to_owned())
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if node_ids.is_empty() {
        return Ok(false);
    }
    let epoch = i64::try_from(epoch)
        .map_err(|_| DistannExpandError::Internal("physical intent epoch exceeds int8".to_owned()))?;
    let node_ids = node_ids.join(",");
    let vec_ids = vec_ids
        .iter()
        .map(|vec_id| i64::from_le_bytes(vec_id.to_le_bytes()).to_string())
        .collect::<Vec<_>>()
        .join(",");
    Spi::get_one_with_args::<bool>(
        &format!("SELECT EXISTS (\
           SELECT 1 FROM ec_distann_remote_prepared_xact_intent \
            WHERE node_id IN ({node_ids}) \
              AND served_epoch = $1 \
              AND tracked_vec_id IN ({vec_ids}) \
              AND updated_at >= clock_timestamp() - interval '5 seconds' \
              AND intent_state IN ('prepare_requested', 'prepare_acked', 'commit_intended', 'commit_local'))"),
        &[epoch.into()],
    )
    .map_err(|error| {
        DistannExpandError::Internal(format!(
            "physical remote insert intent lookup failed: {error}"
        ))
    })?
    .ok_or_else(|| {
        DistannExpandError::Internal(
            "physical remote insert intent lookup returned NULL".to_owned(),
        )
    })
}

/// Resolve an owner-local graph batch with a bounded 2PC visibility retry.
/// Both traversal expansion and physical materialization use this helper so
/// the retry policy cannot drift between their lookup paths. Five seconds is
/// deliberately finite: it covers a contended insert-planning wave without
/// converting an abandoned audit row into a permanent retry permission.
fn lookup_graph_nodes_with_intent_retry<F>(
    index_oid: pg_sys::Oid,
    generation: &GenerationCatalogRow,
    graph_relation: &HeapRelationGuard,
    directory_relation: &IndexRelationGuard,
    snapshot: pg_sys::Snapshot,
    vec_ids: &[u64],
    intent_node_ids: &[u32],
    graph_degree: u16,
    code_len: usize,
    missing: F,
) -> Result<HashMap<u64, DistannNodeTuple>, DistannExpandError>
where
    F: Fn(u64) -> DistannExpandError + Copy,
{
    let records = {
        #[cfg(feature = "pg_test")]
        {
            let forced = FORCED_FRONTIER_RETRY_USED.with(|used| {
                if super::options::debug_force_frontier_retry() && !used.get() {
                    used.set(true);
                    true
                } else {
                    false
                }
            });
            if forced {
                Err(missing(vec_ids[0]))
            } else {
                lookup_graph_nodes(
                    graph_relation,
                    directory_relation,
                    snapshot,
                    vec_ids,
                    graph_degree,
                    code_len,
                    missing,
                )
            }
        }
        #[cfg(not(feature = "pg_test"))]
        {
            lookup_graph_nodes(
                graph_relation,
                directory_relation,
                snapshot,
                vec_ids,
                graph_degree,
                code_len,
                missing,
            )
        }
    };
    let Err(error @ DistannExpandError::OwnedRecordMissing(_)) = records else {
        return records;
    };
    if !recent_remote_insert_intent(index_oid, intent_node_ids, vec_ids, generation.epoch)? {
        return Err(error);
    }
    super::stage_counters::record_work(
        super::stage_counters::DistannMaterializationWork::TraversalFrontierRetries,
        1,
    );
    // The retry runs in the owner backend that served the RPC. Persist a
    // per-owner attribution marker so an external fixture can observe the
    // retry after that backend exits; process-local stage counters cannot
    // cross the coordinator/owner RPC boundary. The gate above already
    // matched epoch, state, and freshness; retain only the exact owner/id
    // scope here so sampling cannot lose the marker while the wave runs.
    let _ = Spi::run(&format!(
        "UPDATE ec_distann_remote_prepared_xact_intent \
            SET retry_count = retry_count + 1 \
          WHERE node_id IN ({}) \
            AND tracked_vec_id IN ({}) \
            ",
        intent_node_ids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","),
        vec_ids
            .iter()
            .map(|vec_id| i64::from_le_bytes(vec_id.to_le_bytes()).to_string())
            .collect::<Vec<_>>()
            .join(","),
    ));
    let mut last_error = error;
    // Task 167 final fixture provenance: an event-loop yield, bounded to
    // thirty-two attempts, is enough to let
    // the commit callback resolve a prepared owner transaction under the
    // heavier multi-owner benchmark wave without reintroducing the old
    // 40-ms sleep budget under the relation guards.
    for _ in 0..32 {
        let _ = Spi::run("SELECT pg_sleep(0.001)");
        let latest = RegisteredSnapshotGuard::latest().ok_or_else(|| last_error.clone())?;
        match lookup_graph_nodes(
            graph_relation,
            directory_relation,
            latest.as_ptr(),
            vec_ids,
            graph_degree,
            code_len,
            missing,
        ) {
            Ok(found) => return Ok(found),
            Err(next @ DistannExpandError::OwnedRecordMissing(_)) => {
                last_error = next;
                if !recent_remote_insert_intent(index_oid, intent_node_ids, vec_ids, generation.epoch)? {
                    break;
                }
            }
            Err(next) => return Err(next),
        }
    }
    Err(last_error)
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    fn identity(marker: u8) -> ActiveGenerationIdentity {
        let mut build = [marker; 16];
        build[6] = (build[6] & 0x0f) | 0x40;
        build[8] = (build[8] & 0x3f) | 0x80;
        let mut fingerprint = [marker; 34];
        fingerprint[0..2].copy_from_slice(&2_u16.to_le_bytes());
        ActiveGenerationIdentity {
            build_id: Uuid::from_bytes(build),
            fingerprint,
        }
    }

    #[test]
    fn physical_epoch_cache_is_bounded_and_lru() {
        PHYSICAL_EPOCH_CACHE.with(|cache| cache.borrow_mut().clear());
        let index_oid = pg_sys::Oid::from(42_u32);
        let logical_index_uuid = Uuid::from_bytes([0x44; 16]);
        let descriptor =
            Arc::new(super::super::generation_descriptor::sample_generation_descriptor());
        let descriptor_digest = descriptor.digest().unwrap();
        let insert = |active: &ActiveGenerationIdentity| {
            cache_physical_epoch(CachedPhysicalEpoch {
                index_oid,
                logical_index_uuid,
                build_id: active.build_id,
                fingerprint: active.fingerprint,
                descriptor: Arc::clone(&descriptor),
                descriptor_digest,
                head_index: None,
                gateway_copies: None,
                crown: None,
            });
        };
        let first = identity(0x11);
        let second = identity(0x22);
        let third = identity(0x33);
        insert(&first);
        insert(&second);
        let cached = cached_physical_epoch(index_oid, logical_index_uuid, &first).unwrap();
        assert!(Arc::ptr_eq(&cached.descriptor, &descriptor));
        insert(&third);
        assert!(cached_physical_epoch(index_oid, logical_index_uuid, &second).is_none());
        assert!(cached_physical_epoch(index_oid, logical_index_uuid, &first).is_some());
        assert!(cached_physical_epoch(index_oid, logical_index_uuid, &third).is_some());
        PHYSICAL_EPOCH_CACHE.with(|cache| cache.borrow_mut().clear());
    }

    #[test]
    fn physical_query_cache_requires_matching_digest_and_reuses_arc() {
        PHYSICAL_QUERY_CACHE.with(|cache| *cache.borrow_mut() = None);
        let query = vec![1.25, -0.0, f32::from_bits(0x7fc0_0001)];
        let digest = physical_query_digest(&query).unwrap();
        let (installed, installed_digest) =
            resolve_cached_physical_query(query.clone(), digest.to_vec())
                .expect("full query installs cache");
        let (reused, reused_digest) = resolve_cached_physical_query(Vec::new(), digest.to_vec())
            .expect("matching digest reuses cache");
        assert_eq!(installed_digest, digest);
        assert_eq!(reused_digest, digest);
        assert!(Arc::ptr_eq(&installed, &reused));
        assert_eq!(
            reused
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            query
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );

        let mut wrong = digest;
        wrong[0] ^= 0xff;
        assert!(resolve_cached_physical_query(Vec::new(), wrong.to_vec()).is_err());
        assert!(resolve_cached_physical_query(query, wrong.to_vec()).is_err());
        PHYSICAL_QUERY_CACHE.with(|cache| *cache.borrow_mut() = None);
    }
}

const PHYSICAL_EPOCH_CACHE_CAPACITY: usize = 2;

#[derive(Clone)]
struct CachedPhysicalEpoch {
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    fingerprint: [u8; 34],
    descriptor: Arc<DistannGenerationDescriptor>,
    descriptor_digest: [u8; 32],
    head_index: Option<Arc<super::head_sample::DistannPhysicalHeadIndex>>,
    /// TRAV-30 (Task 210 P3): the bounded gateway routing copies for this
    /// epoch. `None` until populated; populated at most once per cached epoch.
    gateway_copies: Option<Arc<super::gateway_copy::DistannGatewayCopySet>>,
    crown: Option<Arc<super::crown_cache::DistannCrownCache>>,
}

thread_local! {
    static PHYSICAL_EPOCH_CACHE: RefCell<VecDeque<CachedPhysicalEpoch>> =
        const { RefCell::new(VecDeque::new()) };
}

static GENERATION_CACHE_INVALIDATION_REGISTERED: AtomicBool = AtomicBool::new(false);

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn invalidate_generation_caches(
    _arg: pg_sys::Datum,
    relation_oid: pg_sys::Oid,
) {
    let mut removed = Vec::new();
    RETAINED_EPOCH_CACHE.with(|cache| {
        let Ok(mut cache) = cache.try_borrow_mut() else {
            return;
        };
        cache.retain(|entry| {
            let generation_relation = entry.generation.row_tier_relid == relation_oid
                || entry.generation.graph_store_relid == relation_oid
                || entry.generation.directory_relid == relation_oid;
            let matches = relation_oid == pg_sys::InvalidOid
                || entry.index_oid == relation_oid
                || generation_relation;
            if matches {
                removed.push((entry.index_oid, entry.fingerprint));
            }
            !matches
        });
    });
    PHYSICAL_PREPARED_QUERY_CACHE.with(|cache| {
        let Ok(mut cache) = cache.try_borrow_mut() else {
            return;
        };
        cache.retain(|entry| {
            relation_oid != pg_sys::InvalidOid
                && entry.index_oid != relation_oid
                && !removed.iter().any(|(index_oid, fingerprint)| {
                    entry.index_oid == *index_oid && entry.fingerprint == *fingerprint
                })
        });
    });
    OWNER_HEAD_SHARD_CACHE.with(|cache| {
        let Ok(mut cache) = cache.try_borrow_mut() else {
            return;
        };
        cache.retain(|entry| {
            relation_oid != pg_sys::InvalidOid
                && entry.index_oid != relation_oid
                && !removed.iter().any(|(index_oid, fingerprint)| {
                    entry.index_oid == *index_oid && entry.fingerprint == *fingerprint
                })
        });
    });
}

pub(crate) unsafe fn register_generation_cache_invalidation() {
    if !GENERATION_CACHE_INVALIDATION_REGISTERED.swap(true, Ordering::Relaxed) {
        unsafe {
            pg_sys::CacheRegisterRelcacheCallback(
                Some(invalidate_generation_caches),
                pg_sys::Datum::from(0_usize),
            );
        }
    }
}

#[cfg(feature = "pg_test")]
#[pg_extern]
fn ec_distann_debug_retained_epoch_cache_len() -> i64 {
    RETAINED_EPOCH_CACHE.with(|cache| cache.borrow().len() as i64)
}

#[cfg(feature = "pg_test")]
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_debug_crown_cache_state() -> TableIterator<
    'static,
    (
        name!(capacity, i64),
        name!(entries, i64),
        name!(epoch_fingerprint, Vec<u8>),
    ),
> {
    let state = PHYSICAL_EPOCH_CACHE.with(|cache| {
        cache.borrow().iter().rev().find_map(|entry| {
            entry.crown.as_ref().map(|crown| {
                (
                    i64::try_from(crown.capacity()).unwrap_or(i64::MAX),
                    i64::try_from(crown.len()).unwrap_or(i64::MAX),
                    crown.epoch_fingerprint().to_vec(),
                )
            })
        })
    });
    TableIterator::new(state.into_iter())
}

fn cached_physical_epoch(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    active: &ActiveGenerationIdentity,
) -> Option<CachedPhysicalEpoch> {
    if !super::options::physical_epoch_cache_enabled() {
        return None;
    }
    PHYSICAL_EPOCH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let position = cache.iter().position(|entry| {
            entry.index_oid == index_oid
                && entry.logical_index_uuid == logical_index_uuid
                && entry.build_id == active.build_id
                && entry.fingerprint == active.fingerprint
        })?;
        let entry = cache.remove(position)?;
        cache.push_back(entry.clone());
        Some(entry)
    })
}

fn cache_physical_epoch(entry: CachedPhysicalEpoch) {
    if !super::options::physical_epoch_cache_enabled() {
        return;
    }
    PHYSICAL_EPOCH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.retain(|candidate| {
            candidate.index_oid != entry.index_oid
                || candidate.logical_index_uuid != entry.logical_index_uuid
                || candidate.build_id != entry.build_id
                || candidate.fingerprint != entry.fingerprint
        });
        while cache.len() >= PHYSICAL_EPOCH_CACHE_CAPACITY {
            cache.pop_front();
        }
        cache.push_back(entry);
    });
}

const RETAINED_EPOCH_CACHE_CAPACITY: usize = 4;

#[derive(Clone)]
struct CachedRetainedEpoch {
    index_oid: pg_sys::Oid,
    fingerprint: [u8; 34],
    descriptor: Arc<DistannGenerationDescriptor>,
    generation: GenerationCatalogRow,
    source_attnum: i32,
    code_len: usize,
    row_schema: Arc<super::row_schema::ResolvedRowSchema>,
    #[cfg(feature = "distann-head-attribution-benchmark")]
    owner_payload_plans: Rc<RefCell<VecDeque<CachedOwnerPayloadPlan>>>,
}

#[cfg(feature = "distann-head-attribution-benchmark")]
const OWNER_PAYLOAD_PLAN_CACHE_CAPACITY: usize = 4;

#[cfg(feature = "distann-head-attribution-benchmark")]
struct CachedOwnerPayloadPlan {
    generation_fingerprint: [u8; 34],
    projection_fingerprint: [u8; 32],
    statement: OwnedPreparedStatement,
}

thread_local! {
    static RETAINED_EPOCH_CACHE: RefCell<VecDeque<CachedRetainedEpoch>> =
        const { RefCell::new(VecDeque::new()) };
    #[cfg(feature = "pg_test")]
    static FORCED_FRONTIER_RETRY_USED: Cell<bool> = const { Cell::new(false) };
}

const PHYSICAL_PREPARED_QUERY_CACHE_CAPACITY: usize = 4;

struct CachedPhysicalPreparedQuery {
    index_oid: pg_sys::Oid,
    fingerprint: [u8; 34],
    query_digest: [u8; 32],
    prepared: Arc<DistannPreparedQuery>,
}

thread_local! {
    static PHYSICAL_PREPARED_QUERY_CACHE: RefCell<VecDeque<CachedPhysicalPreparedQuery>> =
        const { RefCell::new(VecDeque::new()) };
}

fn prepared_physical_query(
    index_oid: pg_sys::Oid,
    fingerprint: [u8; 34],
    query_digest: [u8; 32],
    descriptor: &DistannGenerationDescriptor,
    query: &[f32],
) -> Result<Arc<DistannPreparedQuery>, DistannExpandError> {
    if let Some(prepared) = PHYSICAL_PREPARED_QUERY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let position = cache.iter().position(|entry| {
            entry.index_oid == index_oid
                && entry.fingerprint == fingerprint
                && entry.query_digest == query_digest
        })?;
        let entry = cache.remove(position)?;
        let prepared = Arc::clone(&entry.prepared);
        cache.push_back(entry);
        Some(prepared)
    }) {
        return Ok(prepared);
    }
    let prepared = Arc::new(
        DistannPreparedQuery::prepare_artifact(&descriptor.codec_artifact, query)
            .map_err(DistannExpandError::Internal)?,
    );
    PHYSICAL_PREPARED_QUERY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        while cache.len() >= PHYSICAL_PREPARED_QUERY_CACHE_CAPACITY {
            cache.pop_front();
        }
        cache.push_back(CachedPhysicalPreparedQuery {
            index_oid,
            fingerprint,
            query_digest,
            prepared: Arc::clone(&prepared),
        });
    });
    Ok(prepared)
}

/// Task 210 P2a follow-up: the owner head shard is epoch-immutable and its
/// build is deterministically seeded from the epoch fingerprint, so building
/// it per `head_search` RPC (the first P2 A/B measured ~7 s/query at 100k
/// against a 36 ms owner-path control) is pure waste. Cache the loaded shard
/// per backend, keyed by everything the build reads; an epoch transition
/// changes the fingerprint and therefore the key.
const OWNER_HEAD_SHARD_CACHE_CAPACITY: usize = 4;

const OWNER_HEAD_SHARD_DOMAIN: &[u8] = b"ecaz/ec_distann/owner_head_shard/v1\0";

struct CachedOwnerHeadShard {
    index_oid: pg_sys::Oid,
    fingerprint: [u8; 34],
    shard_key: [u8; 32],
    index: Arc<super::head_sample::DistannPhysicalHeadIndex>,
    /// Whether the shard was materialised from a §4.1 replica copy rather
    /// than owner-held vectors. Carried so the activation counter attributes
    /// every request served from a replica shard, not only the first build —
    /// warmup builds the cache, counters reset, and the measured window would
    /// otherwise read 0 (the 2026-07-31 gate diagnosis).
    from_replica: bool,
}

thread_local! {
    static OWNER_HEAD_SHARD_CACHE: RefCell<VecDeque<CachedOwnerHeadShard>> =
        const { RefCell::new(VecDeque::new()) };
}

#[allow(clippy::too_many_arguments)]
fn owner_head_shard_key(
    owner_ordinal: u32,
    members: &[u64],
    graph_degree: u16,
    build_list_size: usize,
    alpha: f32,
    head_policy: super::generation_descriptor::DistannHeadPolicy,
) -> Result<[u8; 32], String> {
    let mut encoder = super::canonical_wire::CanonicalEncoder::with_capacity(
        24_usize.saturating_add(members.len().saturating_mul(8)),
    );
    encoder.put_u32(owner_ordinal);
    encoder.put_u32(u32::from(graph_degree));
    encoder.put_u32(u32::try_from(build_list_size).unwrap_or(u32::MAX));
    encoder.put_f32(alpha);
    encoder.put_u32(u32::from(head_policy as u8));
    encoder.put_u32(u32::try_from(members.len()).map_err(|_| "head shard exceeds u32".to_owned())?);
    for member in members {
        encoder.put_u64(*member);
    }
    Ok(super::canonical_wire::domain_digest(
        OWNER_HEAD_SHARD_DOMAIN,
        &encoder.finish()?,
    ))
}

fn cached_owner_head_shard(
    index_oid: pg_sys::Oid,
    fingerprint: &[u8; 34],
    shard_key: &[u8; 32],
) -> Option<(Arc<super::head_sample::DistannPhysicalHeadIndex>, bool)> {
    OWNER_HEAD_SHARD_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let position = cache.iter().position(|entry| {
            entry.index_oid == index_oid
                && entry.fingerprint == *fingerprint
                && entry.shard_key == *shard_key
        })?;
        let entry = cache.remove(position)?;
        let hit = (Arc::clone(&entry.index), entry.from_replica);
        cache.push_back(entry);
        Some(hit)
    })
}

fn cache_owner_head_shard(
    index_oid: pg_sys::Oid,
    fingerprint: [u8; 34],
    shard_key: [u8; 32],
    index: Arc<super::head_sample::DistannPhysicalHeadIndex>,
    from_replica: bool,
) {
    OWNER_HEAD_SHARD_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        while cache.len() >= OWNER_HEAD_SHARD_CACHE_CAPACITY {
            cache.pop_front();
        }
        cache.push_back(CachedOwnerHeadShard {
            index_oid,
            fingerprint,
            shard_key,
            index,
            from_replica,
        });
    });
}

fn cached_retained_epoch(
    index_oid: pg_sys::Oid,
    fingerprint: &[u8; 34],
) -> Option<CachedRetainedEpoch> {
    if !super::options::physical_epoch_cache_enabled() {
        return None;
    }
    RETAINED_EPOCH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let position = cache
            .iter()
            .position(|entry| entry.index_oid == index_oid && entry.fingerprint == *fingerprint)?;
        let entry = cache.remove(position)?;
        cache.push_back(entry.clone());
        Some(entry)
    })
}

fn cache_retained_epoch(entry: CachedRetainedEpoch) {
    if !super::options::physical_epoch_cache_enabled() {
        return;
    }
    RETAINED_EPOCH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        // One backend may serve several physical indexes, but observing a new
        // fingerprint for one index is an epoch transition for that endpoint.
        // Discard the predecessor entry immediately instead of retaining two
        // same-index schema snapshots until the global LRU cap is reached.
        cache.retain(|candidate| candidate.index_oid != entry.index_oid);
        while cache.len() >= RETAINED_EPOCH_CACHE_CAPACITY {
            cache.pop_front();
        }
        cache.push_back(entry);
    });
}

const PHYSICAL_QUERY_DIGEST_DOMAIN: &[u8] = b"ecaz/ec_distann/physical_query/v1\0";

thread_local! {
    static PHYSICAL_QUERY_CACHE: RefCell<Option<([u8; 32], Arc<[f32]>)>> =
        const { RefCell::new(None) };
}

pub(crate) fn physical_query_digest(query: &[f32]) -> Result<[u8; 32], String> {
    let dimensions = u32::try_from(query.len())
        .map_err(|_| "physical query dimension exceeds u32".to_owned())?;
    let mut encoder = super::canonical_wire::CanonicalEncoder::with_capacity(
        4_usize.saturating_add(query.len().saturating_mul(4)),
    );
    encoder.put_u32(dimensions);
    for value in query {
        encoder.put_f32(*value);
    }
    Ok(super::canonical_wire::domain_digest(
        PHYSICAL_QUERY_DIGEST_DOMAIN,
        &encoder.finish()?,
    ))
}

fn resolve_cached_physical_query(
    supplied_query: Vec<f32>,
    supplied_digest: Vec<u8>,
) -> Result<(Arc<[f32]>, [u8; 32]), DistannExpandError> {
    let digest = super::canonical_wire::fixed_digest(
        supplied_digest,
        "EC_QUERY_DIGEST",
        "physical query digest",
    )
    .map_err(DistannExpandError::BadInput)?;
    if !supplied_query.is_empty() {
        let query = supplied_query;
        let computed = physical_query_digest(&query).map_err(DistannExpandError::BadInput)?;
        if computed != digest {
            return Err(DistannExpandError::BadInput(
                "physical query digest does not match supplied query".to_owned(),
            ));
        }
        let query = Arc::<[f32]>::from(query);
        PHYSICAL_QUERY_CACHE.with(|cache| {
            *cache.borrow_mut() = Some((digest, Arc::clone(&query)));
        });
        return Ok((query, digest));
    }
    PHYSICAL_QUERY_CACHE.with(|cache| {
        cache
            .borrow()
            .as_ref()
            .filter(|(cached_digest, _)| *cached_digest == digest)
            .map(|(_, query)| (Arc::clone(query), digest))
            .ok_or_else(|| {
                DistannExpandError::BadInput(
                    "physical query cache miss; resend the query vector".to_owned(),
                )
            })
    })
}

#[derive(Debug, Clone)]
pub(crate) struct PhysicalOwnerRoute {
    pub(crate) roster_ordinal: usize,
    pub(crate) node_id: u32,
    pub(crate) is_local: bool,
    pub(crate) remote_index_regclass: String,
    /// Resolved endpoint for roster serialization. This is present for the
    /// local route too; `conninfo` remains `None` locally because callers use
    /// that field as the remote-transport discriminator.
    pub(crate) roster_conninfo: String,
    pub(crate) conninfo: Option<String>,
}

pub(crate) fn physical_owner_routes(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    roster_len: usize,
) -> Result<Vec<PhysicalOwnerRoute>, String> {
    let bindings =
        generation_catalog::extension_relation_name("ec_distann_build_participant_binding")?;
    let routes = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT roster_ordinal, node_id, is_local, remote_index_regclass,
                            conninfo_secret_name
                       FROM {bindings}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid
                      ORDER BY roster_ordinal"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|error| format!("EC_NODE_DESCRIPTOR: binding lookup failed: {error}"))?
            .map(|row| {
                let ordinal = row["roster_ordinal"]
                    .value::<i32>()
                    .map_err(|error| format!("EC_NODE_DESCRIPTOR: ordinal decode failed: {error}"))?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: ordinal is NULL".to_owned())?;
                let is_local = row["is_local"]
                    .value::<bool>()
                    .map_err(|error| {
                        format!("EC_NODE_DESCRIPTOR: locality decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: locality is NULL".to_owned())?;
                let node_id = row["node_id"]
                    .value::<i32>()
                    .map_err(|error| format!("EC_NODE_DESCRIPTOR: node id decode failed: {error}"))?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: node id is NULL".to_owned())?;
                let remote_index_regclass = row["remote_index_regclass"]
                    .value::<String>()
                    .map_err(|error| format!("EC_NODE_DESCRIPTOR: locator decode failed: {error}"))?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: locator is NULL".to_owned())?;
                let secret = row["conninfo_secret_name"]
                    .value::<String>()
                    .map_err(|error| format!("EC_NODE_DESCRIPTOR: secret decode failed: {error}"))?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: secret is NULL".to_owned())?;
                let roster_conninfo = super::node_registry::resolve_conninfo_secret(&secret)?;
                Ok(PhysicalOwnerRoute {
                    roster_ordinal: usize::try_from(ordinal).map_err(|_| {
                        "EC_NODE_DESCRIPTOR: binding ordinal is negative".to_owned()
                    })?,
                    node_id: u32::try_from(node_id)
                        .map_err(|_| "EC_NODE_DESCRIPTOR: node id is negative".to_owned())?,
                    is_local,
                    remote_index_regclass,
                    roster_conninfo: roster_conninfo.clone(),
                    conninfo: if is_local {
                        None
                    } else {
                        Some(roster_conninfo)
                    },
                })
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    if routes.len() != roster_len
        || routes
            .iter()
            .enumerate()
            .any(|(ordinal, route)| route.roster_ordinal != ordinal)
        || routes.iter().filter(|route| route.is_local).count() > 1
    {
        return Err(
            "EC_NODE_DESCRIPTOR: immutable participant bindings do not cover the roster".to_owned(),
        );
    }
    Ok(routes)
}

/// Owner DML runs on a participant catalog that may retain the published
/// generation without copying the coordinator's private participant-binding
/// rows.  Its insert plan only needs placement and connection endpoints; the
/// immutable descriptor roster plus the already-fenced session roster supply
/// those fields.  Keep the binding table authoritative whenever it exists.
fn physical_owner_routes_for_owner_insert(
    index_oid: pg_sys::Oid,
    descriptor: &DistannGenerationDescriptor,
) -> Result<Vec<PhysicalOwnerRoute>, String> {
    let configured = super::roster::parse_roster(&super::roster::current_roster_spec())?;
    if configured.len() != descriptor.roster.len() {
        return Err(
            "EC_NODE_DESCRIPTOR: owner insert roster does not match the immutable descriptor"
                .to_owned(),
        );
    }
    if configured
        .iter()
        .zip(&descriptor.roster)
        .any(|(configured, descriptor)| configured.node_id != descriptor.node_id)
    {
        return Err(
            "EC_NODE_DESCRIPTOR: owner insert roster ordering differs from the immutable descriptor"
                .to_owned(),
        );
    }
    let local_node_id = super::roster::current_local_node_id();
    let local_count = descriptor
        .roster
        .iter()
        .filter(|entry| entry.node_id == local_node_id)
        .count();
    if local_count != 1 {
        return Err("EC_NODE_DESCRIPTOR: owner insert roster has no unique local node".to_owned());
    }
    let local_locator = super::handoff::qualified_relation_name(index_oid)?;
    let mut routes = Vec::with_capacity(descriptor.roster.len());
    for (ordinal, entry) in descriptor.roster.iter().enumerate() {
        let is_local = entry.node_id == local_node_id;
        routes.push(PhysicalOwnerRoute {
            roster_ordinal: ordinal,
            node_id: entry.node_id,
            is_local,
            remote_index_regclass: local_locator.clone(),
            roster_conninfo: configured[ordinal].conninfo.clone(),
            conninfo: if is_local {
                None
            } else {
                Some(configured[ordinal].conninfo.clone())
            },
        });
    }
    Ok(routes)
}

/// Exact retained participant generation selected by the coordinator's v2
/// manifest fingerprint.  Published and Retired are both readable; retirement
/// only makes the generation unreachable to new coordinator scans, while
/// reclaim waits for registered readers to drain.
struct RetainedGenerationScan {
    index_oid: pg_sys::Oid,
    fingerprint: [u8; 34],
    descriptor: Arc<DistannGenerationDescriptor>,
    generation: GenerationCatalogRow,
    row_relation: HeapRelationGuard,
    graph_relation: HeapRelationGuard,
    directory_relation: IndexRelationGuard,
    #[cfg(feature = "distann-head-attribution-benchmark")]
    graph_relation_name: String,
    source_attnum: i32,
    code_len: usize,
    row_schema: Arc<super::row_schema::ResolvedRowSchema>,
    #[cfg(feature = "distann-head-attribution-benchmark")]
    owner_payload_plans: Rc<RefCell<VecDeque<CachedOwnerPayloadPlan>>>,
}

#[derive(Debug)]
pub(crate) struct TraversalReplicaChunkRow {
    pub(crate) owner_ordinal: u32,
    pub(crate) vec_id: u64,
    pub(crate) graph_record: Vec<u8>,
    pub(crate) exact_vector: Vec<u8>,
}

impl RetainedGenerationScan {
    fn open(index_oid: pg_sys::Oid, fingerprint: &[u8]) -> Result<Self, DistannExpandError> {
        let fingerprint: [u8; 34] = fingerprint.try_into().map_err(|_| {
            DistannExpandError::BadInput(format!(
                "physical epoch fingerprint must be 34 bytes, got {}",
                fingerprint.len()
            ))
        })?;
        if fingerprint[..2] != [2, 0] {
            return Err(DistannExpandError::BadInput(
                "physical epoch fingerprint is not canonical v2".to_owned(),
            ));
        }
        let cached = if let Some(cached) = cached_retained_epoch(index_oid, &fingerprint) {
            cached
        } else {
            let (control, _handle, _metadata, logical_index_uuid) =
                super::generation_store::open_control_index(
                    index_oid,
                    pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                    "physical generation endpoint",
                )
                .map_err(DistannExpandError::GenerationMissing)?;
            let source_attnum = super::routine::indexed_ecvector_attnum(control.as_ptr())
                .map_err(DistannExpandError::BadInput)?;
            let retained = generation_catalog::lookup_retained_generation_by_fingerprint(
                index_oid,
                logical_index_uuid,
                &fingerprint,
            )
            .map_err(DistannExpandError::GenerationMissing)?
            .ok_or_else(|| {
                DistannExpandError::GenerationMissing(
                    "requested physical generation is not retained on this participant".to_owned(),
                )
            })?;
            let generation = retained.generation;
            let descriptor = Arc::new(
                DistannGenerationDescriptor::decode(&generation.generation_descriptor)
                    .map_err(DistannExpandError::GenerationMissing)?,
            );
            let roster_entry = descriptor
                .roster
                .get(generation.owner_ordinal as usize)
                .ok_or_else(|| {
                    DistannExpandError::GenerationMissing(
                        "generation owner ordinal is outside its immutable roster".to_owned(),
                    )
                })?;
            if descriptor
                .digest()
                .map_err(DistannExpandError::GenerationMissing)?
                != generation.generation_descriptor_digest
                || roster_entry.logical_index_uuid != *logical_index_uuid.as_bytes()
                || roster_entry.node_id != generation.node_id
            {
                return Err(DistannExpandError::GenerationMissing(
                    "retained generation descriptor/participant identity mismatch".to_owned(),
                ));
            }
            let binding = DistannCodecBinding::from_artifact(&descriptor.codec_artifact)
                .map_err(DistannExpandError::GenerationMissing)?;
            let code_len = binding
                .code_len(usize::from(descriptor.dimensions))
                .map_err(DistannExpandError::GenerationMissing)?;
            let row_schema = Arc::new(
                super::row_schema::resolve_relation_schema(generation.row_tier_relid)
                    .map_err(DistannExpandError::GenerationMissing)?,
            );
            let cached = CachedRetainedEpoch {
                index_oid,
                fingerprint,
                descriptor,
                generation,
                source_attnum,
                code_len,
                row_schema,
                #[cfg(feature = "distann-head-attribution-benchmark")]
                owner_payload_plans: Rc::new(RefCell::new(VecDeque::new())),
            };
            cache_retained_epoch(cached.clone());
            cached
        };
        let CachedRetainedEpoch {
            descriptor,
            generation,
            source_attnum,
            code_len,
            row_schema,
            #[cfg(feature = "distann-head-attribution-benchmark")]
            owner_payload_plans,
            ..
        } = cached;
        let row_relation = HeapRelationGuard::try_access_share(generation.row_tier_relid)
            .ok_or_else(|| {
                DistannExpandError::GenerationMissing(
                    "retained row-tier relation is absent".to_owned(),
                )
            })?;
        let graph_relation = HeapRelationGuard::try_access_share(generation.graph_store_relid)
            .ok_or_else(|| {
                DistannExpandError::GenerationMissing(
                    "retained graph-store relation is absent".to_owned(),
                )
            })?;
        let Some(directory_relation) =
            IndexRelationGuard::try_access_share(generation.directory_relid)
        else {
            return Err(DistannExpandError::GenerationMissing(
                "retained graph directory is absent".to_owned(),
            ));
        };
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let graph_relation_name =
            super::handoff::qualified_relation_name(generation.graph_store_relid)
                .map_err(DistannExpandError::GenerationMissing)?;
        Ok(Self {
            index_oid,
            fingerprint,
            descriptor,
            generation,
            row_relation,
            graph_relation,
            directory_relation,
            #[cfg(feature = "distann-head-attribution-benchmark")]
            graph_relation_name,
            source_attnum,
            code_len,
            row_schema,
            #[cfg(feature = "distann-head-attribution-benchmark")]
            owner_payload_plans,
        })
    }

    fn validate_request(&self, query: &[f32], vec_ids: &[u64]) -> Result<(), DistannExpandError> {
        self.validate_query(query)?;
        self.validate_ownership(vec_ids)
    }

    /// Dimension validation alone. §4.1 replica serving (Task 210 P2b) needs
    /// this split: a node serving a foreign head shard from its imported copy
    /// is *supposed* to be asked for ids it does not own, so ownership is
    /// enforced only where the answer comes from owner-held vectors
    /// (`resolve_nodes`), not at the endpoint boundary (003a review,
    /// 2026-07-31 finding 2).
    fn validate_query(&self, query: &[f32]) -> Result<(), DistannExpandError> {
        if query.len() != usize::from(self.descriptor.dimensions) {
            return Err(DistannExpandError::BadInput(format!(
                "query has {} dimensions, retained generation requires {}",
                query.len(),
                self.descriptor.dimensions
            )));
        }
        Ok(())
    }

    fn validate_ownership(&self, vec_ids: &[u64]) -> Result<(), DistannExpandError> {
        for vec_id in vec_ids {
            let owner = super::placement::owning_node(
                *vec_id,
                self.descriptor.roster.len(),
                self.descriptor.placement_hash_version,
            );
            if owner != self.generation.owner_ordinal as usize {
                return Err(DistannExpandError::Placement(format!(
                    "vec_id {vec_id:#018x} belongs to roster ordinal {owner}, not {}",
                    self.generation.owner_ordinal
                )));
            }
        }
        Ok(())
    }

    fn traversal_replica_chunk(
        &self,
        after_vec_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<TraversalReplicaChunkRow>, DistannExpandError> {
        if limit == 0 || limit > 4096 {
            return Err(DistannExpandError::BadInput(
                "traversal replica chunk limit must be in 1..=4096".to_owned(),
            ));
        }
        let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
        if snapshot.is_null() {
            return Err(DistannExpandError::Internal(
                "traversal replica stream has no active snapshot".to_owned(),
            ));
        }
        let key_count = usize::from(after_vec_id.is_some());
        let scan = unsafe {
            IndexScanGuard::begin_from_raw(
                self.graph_relation.as_ptr(),
                self.directory_relation.as_ptr(),
                snapshot,
                key_count as i32,
                0,
            )
        }
        .ok_or_else(|| {
            DistannExpandError::Internal("could not begin traversal replica graph scan".to_owned())
        })?;
        let graph_slot = TupleTableSlotGuard::create_for_heap_guard(&self.graph_relation)
            .ok_or_else(|| {
                DistannExpandError::Internal(
                    "could not allocate traversal replica graph slot".to_owned(),
                )
            })?;
        let row_slot =
            TupleTableSlotGuard::single_for_heap_guard(&self.row_relation).ok_or_else(|| {
                DistannExpandError::Internal(
                    "could not allocate traversal replica row slot".to_owned(),
                )
            })?;
        let mut scan_key =
            unsafe { std::mem::MaybeUninit::<pg_sys::ScanKeyData>::zeroed().assume_init() };
        unsafe {
            if let Some(after) = after_vec_id {
                pg_sys::ScanKeyInit(
                    &mut scan_key,
                    1,
                    pg_sys::BTGreaterStrategyNumber as pg_sys::StrategyNumber,
                    pg_sys::Oid::from(pg_sys::F_INT8GT),
                    pg_sys::Datum::from(after),
                );
                pg_sys::index_rescan(scan.as_ptr(), &mut scan_key, 1, ptr::null_mut(), 0);
            } else {
                pg_sys::index_rescan(scan.as_ptr(), ptr::null_mut(), 0, ptr::null_mut(), 0);
            }
        }

        let mut rows = Vec::with_capacity(limit);
        while rows.len() < limit {
            unsafe { pg_sys::ExecClearTuple(graph_slot.as_ptr()) };
            let found = unsafe {
                pg_sys::index_getnext_slot(
                    scan.as_ptr(),
                    pg_sys::ScanDirection::ForwardScanDirection,
                    graph_slot.as_ptr(),
                )
            };
            if !found {
                break;
            }
            let stored_id =
                unsafe { pg_sys::DatumGetInt64(graph_slot_attr(&graph_slot, 1, "vec_id")?) };
            let record =
                unsafe {
                    crate::am::common::detoast::DetoastedVarlena::packed_from_datum(
                        graph_slot_attr(&graph_slot, 2, "record")?,
                    )
                }
                .ok_or_else(|| {
                    DistannExpandError::GenerationMissing(
                        "traversal replica graph record could not be detoasted".to_owned(),
                    )
                })?;
            let graph_record = record.as_bytes().to_vec();
            let node = DistannNodeTuple::decode_physical_v1(
                &graph_record,
                self.descriptor.graph_degree,
                self.code_len,
            )
            .map_err(DistannExpandError::GenerationMissing)?;
            let vec_id = u64::from_le_bytes(stored_id.to_le_bytes());
            if node.vec_id != vec_id {
                return Err(DistannExpandError::GenerationMissing(
                    "traversal replica graph vec_id does not match its directory key".to_owned(),
                ));
            }
            self.validate_ownership(&[vec_id])?;

            let mut row_tid = pg_sys::ItemPointerData::default();
            pgrx::itemptr::item_pointer_set_all(
                &mut row_tid,
                node.heap_tid.block_number,
                node.heap_tid.offset_number,
            );
            unsafe { pg_sys::ExecClearTuple(row_slot.as_ptr()) };
            let row_found = unsafe {
                pg_sys::table_tuple_fetch_row_version(
                    self.row_relation.as_ptr(),
                    &mut row_tid,
                    snapshot,
                    row_slot.as_ptr(),
                )
            };
            if !row_found {
                return Err(DistannExpandError::GenerationMissing(format!(
                    "traversal replica source row is absent for vec_id {vec_id:#018x}"
                )));
            }
            let mut is_null = false;
            let datum = unsafe {
                pg_sys::slot_getattr(row_slot.as_ptr(), self.source_attnum, &mut is_null)
            };
            if is_null {
                return Err(DistannExpandError::GenerationMissing(
                    "traversal replica source vector is NULL".to_owned(),
                ));
            }
            let vector = unsafe { crate::am::ec_diskann::ecvector_datum_to_vec(datum) };
            if vector.len() != usize::from(self.descriptor.dimensions)
                || vector.iter().any(|value| !value.is_finite())
            {
                return Err(DistannExpandError::GenerationMissing(
                    "traversal replica source vector has invalid dimensions or values".to_owned(),
                ));
            }
            let exact_vector = vector
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<_>>();
            rows.push(TraversalReplicaChunkRow {
                owner_ordinal: self.generation.owner_ordinal,
                vec_id,
                graph_record,
                exact_vector,
            });
        }
        Ok(rows)
    }

    #[cfg(feature = "distann-head-attribution-benchmark")]
    fn seed_candidates(
        &self,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<DistannSeedCandidate>, DistannExpandError> {
        self.validate_request(query, &[])?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        let query_digest = physical_query_digest(query).map_err(DistannExpandError::Internal)?;
        let prepared = prepared_physical_query(
            self.index_oid,
            self.fingerprint,
            query_digest,
            &self.descriptor,
            query,
        )?;
        // Do not ask pgrx to convert bytea to Vec<u8>: that conversion
        // detoasts into TopTransactionContext and leaks the copy until commit.
        // Keep the original Datum, detoast it with the repository's owning
        // guard, and let the guard pfree each copy before the next row/call.
        let mut candidates = Spi::connect(|client| {
            let mut rows = client
                .select(
                    &format!(
                        "SELECT graph_record FROM {} WHERE is_current ORDER BY vec_id",
                        self.graph_relation_name
                    ),
                    None,
                    &[],
                )
                .map_err(|error| {
                    DistannExpandError::GenerationMissing(format!(
                        "physical seed scan failed: {error}"
                    ))
                })?;
            let mut candidates = Vec::with_capacity(rows.len());
            while rows.next().is_some() {
                let datum = rows
                    .get_datum_by_name("graph_record")
                    .map_err(|error| {
                        DistannExpandError::GenerationMissing(format!(
                            "physical seed graph record lookup failed: {error}"
                        ))
                    })?
                    .ok_or_else(|| {
                        DistannExpandError::GenerationMissing(
                            "physical seed graph record is NULL".to_owned(),
                        )
                    })?;
                let record = unsafe {
                    crate::am::common::detoast::DetoastedVarlena::packed_from_datum(datum)
                }
                .ok_or_else(|| {
                    DistannExpandError::GenerationMissing(
                        "physical seed graph record is NULL".to_owned(),
                    )
                })?;
                let node = DistannNodeTuple::decode_physical_v1(
                    record.as_bytes(),
                    self.descriptor.graph_degree,
                    self.code_len,
                )
                .map_err(DistannExpandError::GenerationMissing)?;
                let owner = super::placement::owning_node(
                    node.vec_id,
                    self.descriptor.roster.len(),
                    self.descriptor.placement_hash_version,
                );
                if owner != self.generation.owner_ordinal as usize {
                    return Err(DistannExpandError::Placement(format!(
                        "stored vec_id {:#018x} belongs to roster ordinal {owner}, not {}",
                        node.vec_id, self.generation.owner_ordinal
                    )));
                }
                candidates.push(DistannSeedCandidate {
                    vec_id: node.vec_id,
                    dist: prepared.score_dist(&node.search_code),
                });
            }
            Ok(candidates)
        })?;
        candidates.sort_unstable_by(|left, right| {
            left.dist
                .total_cmp(&right.dist)
                .then_with(|| left.vec_id.cmp(&right.vec_id))
        });
        candidates.truncate(limit);
        Ok(candidates)
    }

    fn expand(
        &self,
        query: &[f32],
        query_digest: [u8; 32],
        vec_ids: &[u64],
        code_threshold: Option<f32>,
        candidate_limit: Option<usize>,
        skip_neighbor_vec_ids: &[u64],
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
        self.validate_request(query, vec_ids)?;
        if vec_ids.is_empty() {
            return Ok(Vec::new());
        }
        let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
        if snapshot.is_null() {
            return Err(DistannExpandError::Internal(
                "physical generation endpoint has no active snapshot".to_owned(),
            ));
        }
        let prepared = prepared_physical_query(
            self.index_oid,
            self.fingerprint,
            query_digest,
            &self.descriptor,
            query,
        )?;
        let slot =
            TupleTableSlotGuard::single_for_heap_guard(&self.row_relation).ok_or_else(|| {
                DistannExpandError::Internal("could not allocate retained row-tier slot".to_owned())
            })?;
        let mut expander = GenerationExpander {
            index_oid: self.index_oid,
            generation: &self.generation,
            descriptor: &self.descriptor,
            graph_relation: &self.graph_relation,
            directory_relation: &self.directory_relation,
            row_relation: &self.row_relation,
            slot: &slot,
            snapshot,
            source_attnum: self.source_attnum,
            query,
            prepared: &prepared,
            code_len: self.code_len,
        };
        let skip = skip_neighbor_vec_ids
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        expander.expand_nodes_masked(vec_ids, code_threshold, candidate_limit, &skip)
    }

    /// Search this owner's shard of the FR-080 head (Task 210 P2a).
    ///
    /// `members` are the head landmarks this owner owns under the FR-078
    /// placement hash. Their full-precision vectors are already local (ADR-085
    /// D11), so the shard is materialised from local reads — no landmark vector
    /// crosses the wire and the coordinator holds none. At most `seed_count`
    /// seeds are returned, which is what keeps coordinator state bounded by
    /// `k_head` under NFR-021 clause 2.
    fn head_search(
        &self,
        query: &[f32],
        query_digest: [u8; 32],
        members: &[u64],
        search_width: usize,
        seed_count: usize,
        build_list_size: usize,
        alpha: f32,
        head_policy: super::generation_descriptor::DistannHeadPolicy,
    ) -> Result<Vec<super::scan::DistannSeedCandidate>, DistannExpandError> {
        // Ownership is deliberately NOT validated here: a §4.1 replica serves
        // a shard it does not own from its imported copy. The owner-vector
        // fallback below still enforces ownership through `resolve_nodes`.
        self.validate_query(query)?;
        if members.is_empty() || seed_count == 0 {
            return Ok(Vec::new());
        }
        // The shard ordinal derives from the members (005 review finding 2):
        // on a replica path `self.generation.owner_ordinal` is the SERVING
        // node's ordinal, and folding that into the key and graph seed would
        // give the same shard a different topology per serving node. The
        // members are the authoritative identity — uniform ownership is
        // validated in the derivation.
        let shard_ordinal = super::placement::shard_owner_ordinal(
            members,
            self.descriptor.roster.len(),
            self.descriptor.placement_hash_version,
        )
        .map_err(DistannExpandError::BadInput)?;
        let shard_ordinal = u32::try_from(shard_ordinal)
            .map_err(|_| DistannExpandError::Internal("shard ordinal exceeds u32".to_owned()))?;
        // The shard is epoch-immutable and its build is deterministic in every
        // keyed input, so it is built once per backend per epoch, not per RPC.
        let shard_key = owner_head_shard_key(
            shard_ordinal,
            members,
            self.descriptor.graph_degree,
            build_list_size,
            alpha,
            head_policy,
        )
        .map_err(DistannExpandError::Internal)?;
        if let Some((index, from_replica)) =
            cached_owner_head_shard(self.index_oid, &self.fingerprint, &shard_key)
        {
            if from_replica {
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::HeadReplicaShardsServed,
                    1,
                );
            }
            return Ok(index.search_configured(query, search_width, seed_count));
        }
        let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
        if snapshot.is_null() {
            return Err(DistannExpandError::Internal(
                "physical head endpoint has no active snapshot".to_owned(),
            ));
        }
        let prepared = prepared_physical_query(
            self.index_oid,
            self.fingerprint,
            query_digest,
            &self.descriptor,
            query,
        )?;
        let slot =
            TupleTableSlotGuard::single_for_heap_guard(&self.row_relation).ok_or_else(|| {
                DistannExpandError::Internal("could not allocate retained row-tier slot".to_owned())
            })?;
        let expander = GenerationExpander {
            index_oid: self.index_oid,
            generation: &self.generation,
            descriptor: &self.descriptor,
            graph_relation: &self.graph_relation,
            directory_relation: &self.directory_relation,
            row_relation: &self.row_relation,
            slot: &slot,
            snapshot,
            source_attnum: self.source_attnum,
            query,
            prepared: &prepared,
            code_len: self.code_len,
        };
        // §4.1 (Task 210 P2b): serve from a replica copy when this node was
        // given one, otherwise from the vectors it owns. A node that has
        // neither is asked for ids it does not own, which resolve_nodes
        // correctly rejects.
        let mut from_replica = false;
        let resolved = match self.replica_head_vectors(members, shard_ordinal)? {
            Some(replica) => {
                // Activation counter (003a review finding 2): replica serving
                // must be provable in a run, not inferred from routing. Cache
                // hits carry the same provenance and count too — see
                // CachedOwnerHeadShard::from_replica.
                from_replica = true;
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::HeadReplicaShardsServed,
                    1,
                );
                replica
            }
            None => {
                let nodes = self.resolve_nodes(members)?;
                let mut owned = Vec::with_capacity(nodes.len());
                for node in &nodes {
                    owned.push((node.vec_id, expander.local_source_vector(node)?));
                }
                owned
            }
        };
        let shard = super::head_sample::build_owner_head_shard(
            shard_ordinal,
            self.descriptor.dimensions,
            resolved,
            usize::from(self.descriptor.graph_degree),
            build_list_size,
            alpha,
            // Generation-scoped seed: the shard graph is reproducible for a
            // given epoch without carrying a build-time random seed.
            u64::from_le_bytes(self.fingerprint[2..10].try_into().unwrap_or([0; 8])),
        )
        .map_err(DistannExpandError::Internal)?;
        let index = super::head_sample::DistannPhysicalHeadIndex::load(
            shard.sample,
            shard.graph,
            usize::from(self.descriptor.graph_degree),
            head_policy,
        )
        .map_err(DistannExpandError::Internal)?;
        let Some(index) = index else {
            return Ok(Vec::new());
        };
        let index = Arc::new(index);
        cache_owner_head_shard(
            self.index_oid,
            self.fingerprint,
            shard_key,
            Arc::clone(&index),
            from_replica,
        );
        Ok(index.search_configured(query, search_width, seed_count))
    }

    /// Persist a bounded head-shard copy received from the shard's owner.
    fn import_head_shard(
        &self,
        shard_ordinal: i32,
        vec_ids: &[i64],
        vectors: &[Vec<f32>],
    ) -> Result<i64, DistannExpandError> {
        let table =
            super::generation_catalog::extension_relation_name("ec_distann_head_shard_replica")
                .map_err(DistannExpandError::Internal)?;
        let fingerprint = self.fingerprint.to_vec();
        let mut imported = 0_i64;
        Spi::connect_mut(|client| {
            for (vec_id, vector) in vec_ids.iter().zip(vectors) {
                client
                    .update(
                        &format!(
                            "INSERT INTO {table} (index_oid, epoch_fingerprint,
                                                  shard_ordinal, vec_id, vector)
                             VALUES ($1::oid, $2::bytea, $3::integer, $4::bigint, $5::real[])
                             ON CONFLICT (index_oid, epoch_fingerprint, vec_id)
                             DO UPDATE SET vector = EXCLUDED.vector,
                                           shard_ordinal = EXCLUDED.shard_ordinal"
                        ),
                        None,
                        &[
                            self.index_oid.into(),
                            fingerprint.clone().into(),
                            shard_ordinal.into(),
                            (*vec_id).into(),
                            vector.as_slice().into(),
                        ],
                    )
                    .map_err(|error| {
                        DistannExpandError::Internal(format!(
                            "ec_distann head shard import failed: {error}"
                        ))
                    })?;
                imported += 1;
            }
            Ok::<(), DistannExpandError>(())
        })?;
        Ok(imported)
    }

    /// Vectors for members this node does not own, from a replica copy it was
    /// given (Task 210 P2b). Returns None when no copy is held.
    fn replica_head_vectors(
        &self,
        members: &[u64],
        shard_ordinal: u32,
    ) -> Result<Option<Vec<(u64, Vec<f32>)>>, DistannExpandError> {
        let table =
            super::generation_catalog::extension_relation_name("ec_distann_head_shard_replica")
                .map_err(DistannExpandError::Internal)?;
        let fingerprint = self.fingerprint.to_vec();
        let wire = members
            .iter()
            .map(|vec_id| i64::from_le_bytes(vec_id.to_le_bytes()))
            .collect::<Vec<_>>();
        let rows = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT vec_id, vector FROM {table}
                          WHERE index_oid = $1::oid AND epoch_fingerprint = $2::bytea
                            AND vec_id = ANY($3::bigint[])
                            AND shard_ordinal = $4::integer"
                    ),
                    None,
                    &[
                        self.index_oid.into(),
                        fingerprint.into(),
                        wire.as_slice().into(),
                        i32::try_from(shard_ordinal).unwrap_or(-1).into(),
                    ],
                )
                .map_err(|error| format!("replica head lookup failed: {error}"))?
                .map(|row| {
                    let vec_id = row["vec_id"]
                        .value::<i64>()
                        .map_err(|error| error.to_string())?
                        .ok_or("vec_id NULL")?;
                    let vector = row["vector"]
                        .value::<Vec<f32>>()
                        .map_err(|error| error.to_string())?
                        .ok_or("vector NULL")?;
                    Ok((u64::from_le_bytes(vec_id.to_le_bytes()), vector))
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .map_err(DistannExpandError::Internal)?;
        if rows.len() == members.len() {
            Ok(Some(rows))
        } else {
            Ok(None)
        }
    }

    /// Read this owner's locally held vectors for `members` so another node can
    /// hold the shard as a bounded replica (Task 210 P2b).
    fn export_head_shard(
        &self,
        members: &[u64],
    ) -> Result<Vec<(u64, Vec<f32>)>, DistannExpandError> {
        if members.is_empty() {
            return Ok(Vec::new());
        }
        let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
        if snapshot.is_null() {
            return Err(DistannExpandError::Internal(
                "physical head export has no active snapshot".to_owned(),
            ));
        }
        let slot =
            TupleTableSlotGuard::single_for_heap_guard(&self.row_relation).ok_or_else(|| {
                DistannExpandError::Internal("could not allocate retained row-tier slot".to_owned())
            })?;
        let empty_query = vec![0.0_f32; usize::from(self.descriptor.dimensions)];
        let prepared = prepared_physical_query(
            self.index_oid,
            self.fingerprint,
            physical_query_digest(&empty_query).map_err(DistannExpandError::BadInput)?,
            &self.descriptor,
            &empty_query,
        )?;
        let expander = GenerationExpander {
            index_oid: self.index_oid,
            generation: &self.generation,
            descriptor: &self.descriptor,
            graph_relation: &self.graph_relation,
            directory_relation: &self.directory_relation,
            row_relation: &self.row_relation,
            slot: &slot,
            snapshot,
            source_attnum: self.source_attnum,
            query: &empty_query,
            prepared: &prepared,
            code_len: self.code_len,
        };
        let nodes = self.resolve_nodes(members)?;
        let mut exported = Vec::with_capacity(nodes.len());
        for node in &nodes {
            exported.push((node.vec_id, expander.local_source_vector(node)?));
        }
        Ok(exported)
    }

    fn resolve_nodes(&self, vec_ids: &[u64]) -> Result<Vec<DistannNodeTuple>, DistannExpandError> {
        self.validate_ownership(vec_ids)?;
        if vec_ids.is_empty() {
            return Ok(Vec::new());
        }
        let intent_node_ids = vec_ids
            .iter()
            .map(|vec_id| {
                let ordinal = super::placement::owning_node(
                    *vec_id,
                    self.descriptor.roster.len(),
                    self.descriptor.placement_hash_version,
                );
                self.descriptor.roster[ordinal].node_id
            })
            .collect::<Vec<_>>();
        let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
        let records = lookup_graph_nodes_with_intent_retry(
            self.index_oid,
            &self.generation,
            &self.graph_relation,
            &self.directory_relation,
            snapshot,
            vec_ids,
            &intent_node_ids,
            self.descriptor.graph_degree,
            self.code_len,
            |vec_id| {
                DistannExpandError::OwnedRecordMissing(format!(
                    "retained physical generation lacks owned vec_id {vec_id:#018x}"
                ))
            },
        )?;
        vec_ids
            .iter()
            .map(|vec_id| {
                records.get(vec_id).cloned().ok_or_else(|| {
                    DistannExpandError::OwnedRecordMissing(format!(
                        "retained physical generation lacks owned vec_id {vec_id:#018x}"
                    ))
                })
            })
            .collect()
    }

    fn materialize_payloads(
        &self,
        vec_ids: &[u64],
        projection_attnums: &[i16],
        expected_schema_fingerprint: &[u8],
        use_cached_payload_plan: bool,
        use_typed_locator: bool,
        use_packed_payload: bool,
        owner_heap_tids: Option<&[ItemPointer]>,
    ) -> Result<PhysicalPayloadBatch, DistannExpandError> {
        #[cfg(not(feature = "distann-head-attribution-benchmark"))]
        let _ = owner_heap_tids;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let validate_started = Instant::now();
        let expected: [u8; 32] = expected_schema_fingerprint.try_into().map_err(|_| {
            DistannExpandError::BadInput(format!(
                "expected schema fingerprint must be 32 bytes, got {}",
                expected_schema_fingerprint.len()
            ))
        })?;
        let resolved_schema = self.row_schema.as_ref();
        if self
            .descriptor
            .row_schema
            .fingerprint()
            .map_err(DistannExpandError::GenerationMissing)?
            != expected
            || resolved_schema
                .descriptor
                .fingerprint()
                .map_err(DistannExpandError::GenerationMissing)?
                != expected
            || resolved_schema.descriptor != self.descriptor.row_schema
        {
            return Err(DistannExpandError::BadInput(
                "requested row schema does not match retained generation".to_owned(),
            ));
        }
        let mut columns = Vec::with_capacity(projection_attnums.len());
        let mut sends = Vec::with_capacity(projection_attnums.len());
        let mut seen = std::collections::HashSet::with_capacity(projection_attnums.len());
        for requested in projection_attnums {
            let attnum = u16::try_from(*requested).map_err(|_| {
                DistannExpandError::BadInput(
                    "projection attnums must be positive physical attributes".to_owned(),
                )
            })?;
            if !seen.insert(attnum) {
                return Err(DistannExpandError::BadInput(
                    "projection attnums must not contain duplicates".to_owned(),
                ));
            }
            let attribute = resolved_schema
                .descriptor
                .attributes
                .iter()
                .find(|attribute| attribute.attnum == attnum && !attribute.dropped)
                .ok_or_else(|| {
                    DistannExpandError::BadInput(format!(
                        "projection attnum {attnum} is absent or dropped"
                    ))
                })?;
            columns.push(attribute.name.clone());
            sends.push(attribute.send_function.clone());
        }
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let validate_ns = duration_ns(validate_started.elapsed());
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let lookup_started = Instant::now();
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let nodes = if let Some(owner_heap_tids) = owner_heap_tids {
            if owner_heap_tids.len() != vec_ids.len() {
                return Err(DistannExpandError::BadInput(
                    "expanded owner locator count does not match vec_id count".to_owned(),
                ));
            }
            self.validate_ownership(vec_ids)?;
            vec_ids
                .iter()
                .copied()
                .zip(owner_heap_tids.iter().copied())
                .map(|(vec_id, heap_tid)| {
                    if heap_tid == ItemPointer::INVALID {
                        return Err(DistannExpandError::BadInput(
                            "expanded owner locator contains an invalid TID".to_owned(),
                        ));
                    }
                    let mut node = DistannNodeTuple::empty();
                    node.vec_id = vec_id;
                    node.heap_tid = heap_tid;
                    Ok(node)
                })
                .collect::<Result<Vec<_>, DistannExpandError>>()?
        } else {
            self.resolve_nodes(vec_ids)?
        };
        #[cfg(not(feature = "distann-head-attribution-benchmark"))]
        let nodes = self.resolve_nodes(vec_ids)?;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let node_lookup_ns = if owner_heap_tids.is_some() {
            0
        } else {
            duration_ns(lookup_started.elapsed())
        };
        if nodes.is_empty() {
            return Ok(PhysicalPayloadBatch {
                rows: Vec::new(),
                #[cfg(feature = "distann-head-attribution-benchmark")]
                telemetry: OwnerMaterializationTelemetry {
                    validate_ns,
                    node_lookup_ns,
                    payload_sql_ns: 0,
                },
            });
        }
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let payload_sql_started = Instant::now();
        let row_name = super::handoff::qualified_relation_name(self.generation.row_tier_relid)
            .map_err(DistannExpandError::GenerationMissing)?;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let sql_builder = if use_packed_payload {
            super::remote_endpoint::build_packed_payload_sql
        } else {
            super::remote_endpoint::build_payload_sql
        };
        #[cfg(not(feature = "distann-head-attribution-benchmark"))]
        let sql_builder = super::remote_endpoint::build_payload_sql;
        let sql = sql_builder(&row_name, &columns, &sends, use_typed_locator)
            .map_err(DistannExpandError::BadInput)?;
        let typed_tids = nodes
            .iter()
            .map(|node| {
                let mut tid = pg_sys::ItemPointerData::default();
                pgrx::itemptr::item_pointer_set_all(
                    &mut tid,
                    node.heap_tid.block_number,
                    node.heap_tid.offset_number,
                );
                tid
            })
            .collect::<Vec<_>>();
        let ctid_texts = typed_tids
            .iter()
            .map(|tid| {
                let (block, offset) = pgrx::itemptr::item_pointer_get_both(*tid);
                format!("({block},{offset})")
            })
            .collect::<Vec<_>>();
        let ctid_refs = ctid_texts.iter().map(String::as_str).collect::<Vec<_>>();
        let column_count = columns.len();
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let projection_fingerprint = {
            let mut hasher = Sha256::new();
            hasher.update(b"ecaz/ec_distann/owner_payload_plan/v1\0");
            hasher.update(expected);
            hasher.update(
                u32::try_from(projection_attnums.len())
                    .unwrap_or(u32::MAX)
                    .to_le_bytes(),
            );
            for attnum in projection_attnums {
                hasher.update(attnum.to_le_bytes());
            }
            hasher.update(u64::try_from(sql.len()).unwrap_or(u64::MAX).to_le_bytes());
            hasher.update(sql.as_bytes());
            hasher.finalize().into()
        };
        let payloads = Spi::connect(|client| {
            #[cfg(feature = "distann-head-attribution-benchmark")]
            let rows = if use_cached_payload_plan {
                let mut plans = self.owner_payload_plans.borrow_mut();
                let position = plans.iter().position(|entry| {
                    entry.generation_fingerprint == self.fingerprint
                        && entry.projection_fingerprint == projection_fingerprint
                });
                if let Some(position) = position {
                    let entry = plans
                        .remove(position)
                        .expect("owner payload plan position disappeared");
                    let rows = if use_typed_locator {
                        client.select(&entry.statement, None, &[typed_tids.clone().into()])
                    } else {
                        client.select(&entry.statement, None, &[ctid_refs.as_slice().into()])
                    };
                    plans.push_back(entry);
                    rows
                } else {
                    let statement = client
                        .prepare(sql.as_str(), &[pg_sys::BuiltinOid::TEXTARRAYOID.oid()])
                        .map_err(|error| {
                            DistannExpandError::VectorMissing(format!(
                                "physical row payload plan preparation failed: {error}"
                            ))
                        })?
                        .keep();
                    let rows = if use_typed_locator {
                        client.select(&statement, None, &[typed_tids.clone().into()])
                    } else {
                        client.select(&statement, None, &[ctid_refs.as_slice().into()])
                    };
                    while plans.len() >= OWNER_PAYLOAD_PLAN_CACHE_CAPACITY {
                        plans.pop_front();
                    }
                    plans.push_back(CachedOwnerPayloadPlan {
                        generation_fingerprint: self.fingerprint,
                        projection_fingerprint,
                        statement,
                    });
                    rows
                }
            } else {
                if use_typed_locator {
                    client.select(&sql, None, &[typed_tids.into()])
                } else {
                    client.select(&sql, None, &[ctid_refs.as_slice().into()])
                }
            };
            #[cfg(not(feature = "distann-head-attribution-benchmark"))]
            let rows = {
                let _ = use_cached_payload_plan;
                if use_typed_locator {
                    client.select(&sql, None, &[typed_tids.into()])
                } else {
                    client.select(&sql, None, &[ctid_refs.as_slice().into()])
                }
            };
            rows.map_err(|error| {
                DistannExpandError::VectorMissing(format!(
                    "physical row payload fetch failed: {error}"
                ))
            })?
            .map(|row| {
                let missing = row["tuple_payload_missing"]
                    .value::<bool>()
                    .map_err(|error| {
                        DistannExpandError::Internal(format!(
                            "physical payload missing flag decode failed: {error}"
                        ))
                    })?
                    .ok_or_else(|| {
                        DistannExpandError::Internal(
                            "physical payload missing flag is NULL".to_owned(),
                        )
                    })?;
                let nulls = row["payload_nulls"]
                    .value::<Vec<bool>>()
                    .map_err(|error| {
                        DistannExpandError::Internal(format!(
                            "physical payload null flags decode failed: {error}"
                        ))
                    })?
                    .unwrap_or_default();
                if use_packed_payload {
                    let offsets = row["payload_offsets"]
                        .value::<Vec<i64>>()
                        .map_err(|error| {
                            DistannExpandError::Internal(format!(
                                "physical packed payload offsets decode failed: {error}"
                            ))
                        })?
                        .unwrap_or_default();
                    let values = row["payload_values"]
                        .value::<Vec<u8>>()
                        .map_err(|error| {
                            DistannExpandError::Internal(format!(
                                "physical packed payload values decode failed: {error}"
                            ))
                        })?
                        .unwrap_or_default();
                    let final_offset = offsets.last().copied().unwrap_or(0);
                    let offsets_valid = offsets.windows(2).all(|window| window[0] <= window[1])
                        && final_offset >= 0
                        && usize::try_from(final_offset)
                            .ok()
                            .is_some_and(|end| end == values.len());
                    if nulls.len() != column_count
                        || offsets.len() != column_count
                        || !offsets_valid
                    {
                        return Err(DistannExpandError::Internal(
                            "physical packed payload shape mismatch".to_owned(),
                        ));
                    }
                    return Ok((missing, nulls, offsets, values));
                }
                let arrays = row["payload_values"]
                    .value::<pgrx::datum::Array<&[u8]>>()
                    .map_err(|error| {
                        DistannExpandError::Internal(format!(
                            "physical payload values decode failed: {error}"
                        ))
                    })?
                    .map(|array| {
                        array
                            .iter_deny_null()
                            .map(<[u8]>::to_vec)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                if nulls.len() != column_count || arrays.len() != column_count {
                    return Err(DistannExpandError::Internal(
                        "physical payload column count mismatch".to_owned(),
                    ));
                }
                {
                    let mut offsets = Vec::with_capacity(arrays.len());
                    let mut values = Vec::new();
                    for value in arrays {
                        values.extend_from_slice(&value);
                        offsets.push(i64::try_from(values.len()).unwrap_or(i64::MAX));
                    }
                    Ok((missing, nulls, offsets, values))
                }
            })
            .collect::<Result<Vec<_>, DistannExpandError>>()
        })?;
        if payloads.len() != nodes.len() {
            return Err(DistannExpandError::Internal(
                "physical payload response count mismatch".to_owned(),
            ));
        }
        let rows = nodes
            .into_iter()
            .zip(payloads)
            .map(|(node, payload)| {
                let (missing, nulls, offsets, values) = payload;
                (
                    i64::from_le_bytes(node.vec_id.to_le_bytes()),
                    node.tombstoned,
                    missing,
                    nulls,
                    offsets,
                    values,
                )
            })
            .collect();
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let payload_sql_ns = duration_ns(payload_sql_started.elapsed());
        Ok(PhysicalPayloadBatch {
            rows,
            #[cfg(feature = "distann-head-attribution-benchmark")]
            telemetry: OwnerMaterializationTelemetry {
                validate_ns,
                node_lookup_ns,
                payload_sql_ns,
            },
        })
    }
}

/// PG18 lifecycle-test surface for Tasks 192/195. This takes the exact
/// production retained-generation lookup and cached payload-schema validation
/// path. It is absent from extension builds that do not enable `pg_test`.
#[cfg(feature = "pg_test")]
#[pg_extern]
fn ec_distann_debug_validate_cached_row_schema(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
) -> bool {
    let store = RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)
        .unwrap_or_else(|error| error.raise());
    let expected = store
        .descriptor
        .row_schema
        .fingerprint()
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    store
        .materialize_payloads(&[], &[], &expected, false, false, false, None)
        .unwrap_or_else(|error| error.raise());
    true
}

#[cfg(feature = "pg_test")]
#[pg_extern]
fn ec_distann_debug_retained_epoch_cache_contains(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
) -> bool {
    let Ok(fingerprint) = <[u8; 34]>::try_from(epoch_fingerprint) else {
        return false;
    };
    RETAINED_EPOCH_CACHE.with(|cache| {
        cache.borrow().iter().any(|entry| {
            entry.index_oid == index_regclass.oid() && entry.fingerprint == fingerprint
        })
    })
}

type PhysicalExpandRow = (i64, Option<f32>, bool, Vec<i64>, Vec<f32>);
type PhysicalPayloadRow = (i64, bool, bool, Vec<bool>, Vec<i64>, Vec<u8>);

struct PhysicalPayloadBatch {
    rows: Vec<PhysicalPayloadRow>,
    #[cfg(feature = "distann-head-attribution-benchmark")]
    telemetry: OwnerMaterializationTelemetry,
}

#[cfg(feature = "distann-head-attribution-benchmark")]
#[derive(Debug, Clone, Copy)]
struct OwnerMaterializationTelemetry {
    validate_ns: u64,
    node_lookup_ns: u64,
    payload_sql_ns: u64,
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn duration_ns(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

pub(crate) fn local_traversal_replica_chunk(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    after_vec_id: Option<i64>,
    limit: usize,
) -> Result<Vec<TraversalReplicaChunkRow>, DistannExpandError> {
    RetainedGenerationScan::open(index_oid, epoch_fingerprint)?
        .traversal_replica_chunk(after_vec_id, limit)
}

#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_stream_traversal_replica_chunk(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    after_vec_id: default!(Option<i64>, "NULL"),
    chunk_limit: default!(i32, 256),
) -> TableIterator<
    'static,
    (
        name!(owner_ordinal, i32),
        name!(vec_id, i64),
        name!(graph_record, Vec<u8>),
        name!(exact_vector, Vec<u8>),
    ),
> {
    let limit = usize::try_from(chunk_limit).unwrap_or_else(|_| {
        DistannExpandError::BadInput("traversal replica chunk limit must be in 1..=4096".to_owned())
            .raise()
    });
    let rows = local_traversal_replica_chunk(
        index_regclass.oid(),
        &epoch_fingerprint,
        after_vec_id,
        limit,
    )
    .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter().map(|row| {
        (
            i32::try_from(row.owner_ordinal).expect("owner ordinal fits integer"),
            i64::from_le_bytes(row.vec_id.to_le_bytes()),
            row.graph_record,
            row.exact_vector,
        )
    }))
}

fn expand_physical_nodes_impl(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    query: &[f32],
    query_digest: [u8; 32],
    vec_ids: &[i64],
    code_threshold: Option<f32>,
    candidate_limit: Option<i32>,
    skip_neighbor_vec_ids: &[i64],
) -> Result<Vec<PhysicalExpandRow>, DistannExpandError> {
    let candidate_limit = candidate_limit
        .map(|limit| {
            usize::try_from(limit).map_err(|_| {
                DistannExpandError::BadInput(
                    "ec_distann candidate_limit must be non-negative".to_owned(),
                )
            })
        })
        .transpose()?;
    let ids = vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let skip = skip_neighbor_vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    RetainedGenerationScan::open(index_oid, epoch_fingerprint)?
        .expand(
            query,
            query_digest,
            &ids,
            code_threshold,
            candidate_limit,
            &skip,
        )
        .map(|expanded| {
            expanded
                .into_iter()
                .map(|node| {
                    (
                        i64::from_le_bytes(node.vec_id.to_le_bytes()),
                        node.exact_dist,
                        node.is_tombstone,
                        node.neighbor_vec_ids
                            .into_iter()
                            .map(|id| i64::from_le_bytes(id.to_le_bytes()))
                            .collect(),
                        node.neighbor_code_dists,
                    )
                })
                .collect()
        })
}

/// Export this owner's head-shard landmarks so another node can serve the shard
/// as a replica (Task 210 P2b, `DISTRIBUTEDANN` §4.1).
///
/// The exported set is bounded by head capacity `C` divided across the roster,
/// so a replica holds a *bounded* structure — which NFR-021 permits to be
/// replicated. It is never the O(N) graph or row tier.
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_head_shard_export(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    member_vec_ids: Vec<i64>,
) -> TableIterator<'static, (name!(vec_id, i64), name!(vector, Vec<f32>))> {
    let members = member_vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let exported = RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)
        .and_then(|scan| scan.export_head_shard(&members))
        .unwrap_or_else(|error| error.raise());
    TableIterator::new(
        exported
            .into_iter()
            .map(|(vec_id, vector)| (i64::from_le_bytes(vec_id.to_le_bytes()), vector))
            .collect::<Vec<_>>()
            .into_iter(),
    )
}

/// Export the routing payload (neighbour ids and neighbour codes — the
/// `graph_record` half of the traversal-replica stream, never the co-placed
/// vector) for a bounded id list, so a coordinator can populate its TRAV-30
/// gateway copies (Task 210 P3). Same source as the withdrawn FR-084 replica,
/// bounded destination: that difference is what makes it conforming under
/// NFR-021.
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_gateway_routing_export(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    member_vec_ids: Vec<i64>,
) -> TableIterator<
    'static,
    (
        name!(vec_id, i64),
        name!(is_tombstone, bool),
        name!(neighbor_vec_ids, Vec<i64>),
        name!(neighbor_codes, Vec<u8>),
    ),
> {
    let members = member_vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let rows = (|| {
        let scan = RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)?;
        let code_len = scan.code_len;
        Ok::<_, DistannExpandError>(
            scan.resolve_nodes(&members)?
                .into_iter()
                .map(|node| {
                    let count = usize::from(node.neighbor_count);
                    (
                        i64::from_le_bytes(node.vec_id.to_le_bytes()),
                        node.tombstoned,
                        node.neighbor_vec_ids[..count]
                            .iter()
                            .map(|id| i64::from_le_bytes(id.to_le_bytes()))
                            .collect::<Vec<_>>(),
                        node.neighbor_codes[..count * code_len].to_vec(),
                    )
                })
                .collect::<Vec<_>>(),
        )
    })()
    .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter())
}

/// Export only the quantized search code required by the FR-089 crown. No
/// neighbor payload or full-precision vector crosses the coordinator boundary.
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_crown_code_export(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    member_vec_ids: Vec<i64>,
) -> TableIterator<'static, (name!(vec_id, i64), name!(search_code, Vec<u8>))> {
    let members = member_vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let rows = (|| {
        let scan = RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)?;
        Ok::<_, DistannExpandError>(
            scan.resolve_nodes(&members)?
                .into_iter()
                .map(|node| {
                    (
                        i64::from_le_bytes(node.vec_id.to_le_bytes()),
                        node.search_code,
                    )
                })
                .collect::<Vec<_>>(),
        )
    })()
    .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter())
}

/// Distribute bounded head-shard copies to §4.1 replicas (Task 210 P2b).
///
/// For each shard, pull its landmarks from the owner and push them to the
/// `replica_count` following roster nodes, then record that replicas are
/// populated for this epoch so routing may use them. Returns the number of
/// (shard, replica) copies placed.
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_populate_head_replicas(index_regclass: PgRelation, replica_count: i32) -> i64 {
    let placed = populate_head_replicas_impl(index_regclass.oid(), replica_count)
        .unwrap_or_else(|error| error.raise());
    placed
}

fn populate_head_replicas_impl(
    index_oid: pg_sys::Oid,
    replica_count: i32,
) -> Result<i64, DistannExpandError> {
    let replicas = usize::try_from(replica_count).unwrap_or(0);
    let state = PhysicalGenerationScan::open(index_oid).map_err(DistannExpandError::Internal)?;
    let head = state
        .head_index
        .as_ref()
        .ok_or_else(|| DistannExpandError::Internal("index has no persisted head".to_owned()))?;
    let members = head.members().to_vec();
    let owner_count = state.routes.len();
    let dimensions = i32::from(state.descriptor.dimensions);
    let mut placed = 0_i64;
    let mut expected = 0_i64;
    if replicas > 0 && owner_count > 1 {
        for shard in 0..owner_count {
            let owned = super::head_sample::head_shard_members(
                &members,
                shard,
                owner_count,
                state.descriptor.placement_hash_version,
            );
            if owned.is_empty() {
                continue;
            }
            let owner = &state.routes[shard];
            // The coordinator's own shard is exported through the local path —
            // skipping it while still attesting population is exactly how a
            // valid owner route turned into a deterministic missing-copy
            // failure (003a review, 2026-07-31 finding 3).
            let copy = match owner.conninfo.as_deref() {
                Some(owner_conninfo) => super::remote_transport::remote_head_shard_export(
                    owner_conninfo,
                    &owner.remote_index_regclass,
                    &state.fingerprint,
                    &owned,
                )?,
                None => {
                    if !owner.is_local {
                        return Err(DistannExpandError::Internal(format!(
                            "EC_NODE_DESCRIPTOR: head shard owner {shard} has no connection descriptor"
                        )));
                    }
                    RetainedGenerationScan::open(index_oid, &state.fingerprint)?
                        .export_head_shard(&owned)?
                }
            };
            for offset in 1..=replicas {
                let server = (shard + offset) % owner_count;
                if server == shard {
                    continue;
                }
                expected += 1;
                let route = &state.routes[server];
                match route.conninfo.as_deref() {
                    Some(conninfo) => {
                        super::remote_transport::remote_head_shard_import(
                            conninfo,
                            &route.remote_index_regclass,
                            &state.fingerprint,
                            i32::try_from(shard).unwrap_or(0),
                            &copy,
                            dimensions,
                        )?;
                    }
                    None => {
                        // The coordinator can serve as a replica too: import
                        // into the local replica table rather than silently
                        // leaving this (shard, replica) pair uncovered.
                        if !route.is_local {
                            return Err(DistannExpandError::Internal(format!(
                                "EC_NODE_DESCRIPTOR: head replica server {server} has no connection descriptor"
                            )));
                        }
                        RetainedGenerationScan::open(index_oid, &state.fingerprint)?
                            .import_head_shard(
                                i32::try_from(shard).unwrap_or(0),
                                &copy
                                    .iter()
                                    .map(|(vec_id, _)| i64::from_le_bytes(vec_id.to_le_bytes()))
                                    .collect::<Vec<_>>(),
                                &copy
                                    .iter()
                                    .map(|(_, vector)| vector.clone())
                                    .collect::<Vec<_>>(),
                            )?;
                    }
                }
                placed += 1;
            }
        }
    }
    // The marker is an attestation, not a log line: it records the replica
    // count only when every (shard, replica) pair for that count was actually
    // imported. Routing consults it against the session's current
    // head_replica_count, so an incomplete population can never enable
    // routing to an unbacked replica.
    if placed != expected {
        return Err(DistannExpandError::Internal(format!(
            "head replica population is incomplete: placed {placed} of {expected} shard copies"
        )));
    }
    let table = super::generation_catalog::extension_relation_name("ec_distann_head_replica_state")
        .map_err(DistannExpandError::Internal)?;
    let fingerprint = state.fingerprint.to_vec();
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "INSERT INTO {table} (index_oid, epoch_fingerprint, replica_count)
                     VALUES ($1::oid, $2::bytea, $3::integer)
                     ON CONFLICT (index_oid, epoch_fingerprint)
                     DO UPDATE SET replica_count = EXCLUDED.replica_count"
                ),
                None,
                &[index_oid.into(), fingerprint.into(), replica_count.into()],
            )
            .map_err(|error| {
                DistannExpandError::Internal(format!("recording head replica state: {error}"))
            })?;
        Ok::<(), DistannExpandError>(())
    })?;
    Ok(placed)
}

/// Receive a bounded head-shard copy so this node can serve the shard as a
/// §4.1 replica (Task 210 P2b).
///
/// The payload is head capacity `C` divided across the roster -- a bounded
/// structure, which NFR-021 permits to be replicated -- never the O(N) graph or
/// row tier. Epoch-scoped and rebuildable.
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_head_shard_import(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    shard_ordinal: i32,
    vec_ids: Vec<i64>,
    // Flattened row-major landmark vectors: pgrx cannot unbox a nested array
    // argument, so the wire carries one contiguous real[] plus the dimension.
    flat_vectors: Vec<f32>,
    dimensions: i32,
) -> i64 {
    let dims = usize::try_from(dimensions).unwrap_or(0);
    if dims == 0 || flat_vectors.len() != vec_ids.len().saturating_mul(dims) {
        DistannExpandError::BadInput(
            "ec_distann head shard import id/vector cardinality mismatch".to_owned(),
        )
        .raise();
    }
    let vectors = flat_vectors
        .chunks_exact(dims)
        .map(<[f32]>::to_vec)
        .collect::<Vec<_>>();
    let scan = RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)
        .unwrap_or_else(|error| error.raise());
    scan.import_head_shard(shard_ordinal, &vec_ids, &vectors)
        .unwrap_or_else(|error| error.raise())
}

/// Owner-side FR-080 head-shard search (Task 210 P2a, NFR-021 clause 3).
///
/// The coordinator sends the bounded landmark ids this owner owns under the
/// FR-078 placement hash; the owner reads their co-placed full-precision
/// vectors locally, searches its own shard, and returns at most `seed_count`
/// seeds. No landmark vector crosses the wire and the coordinator retains none.
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::too_many_arguments)]
fn ec_distann_head_search_physical(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    query: Vec<f32>,
    member_vec_ids: Vec<i64>,
    search_width: i32,
    seed_count: i32,
    build_list_size: i32,
    alpha: f32,
    head_policy: i32,
) -> TableIterator<'static, (name!(vec_id, i64), name!(dist, f32))> {
    let query_digest = physical_query_digest(&query)
        .map_err(DistannExpandError::BadInput)
        .unwrap_or_else(|error| error.raise());
    let members = member_vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let non_negative = |value: i32, what: &str| {
        usize::try_from(value).map_err(|_| {
            DistannExpandError::BadInput(format!("ec_distann {what} must be non-negative"))
        })
    };
    let seeds = (|| {
        let policy = super::generation_descriptor::DistannHeadPolicy::decode_wire(
            u8::try_from(head_policy).map_err(|_| {
                DistannExpandError::BadInput("ec_distann head_policy is out of range".to_owned())
            })?,
        )
        .map_err(DistannExpandError::BadInput)?;
        RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)?.head_search(
            &query,
            query_digest,
            &members,
            non_negative(search_width, "search_width")?,
            non_negative(seed_count, "seed_count")?,
            non_negative(build_list_size, "build_list_size")?,
            alpha,
            policy,
        )
    })()
    .unwrap_or_else(|error| error.raise());
    TableIterator::new(
        seeds
            .into_iter()
            .map(|seed| (i64::from_le_bytes(seed.vec_id.to_le_bytes()), seed.dist))
            .collect::<Vec<_>>()
            .into_iter(),
    )
}

/// FR-079 physical-generation overload.  The `regclass` argument separates it
/// from the legacy metadata-page endpoint whose first SQL argument is `oid`.
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_expand_physical_nodes(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    query: Vec<f32>,
    vec_ids: Vec<i64>,
    code_threshold: default!(Option<f32>, "NULL"),
    candidate_limit: default!(Option<i32>, "NULL"),
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
    let query_digest = physical_query_digest(&query)
        .map_err(DistannExpandError::BadInput)
        .unwrap_or_else(|error| error.raise());
    let rows = expand_physical_nodes_impl(
        index_regclass.oid(),
        &epoch_fingerprint,
        &query,
        query_digest,
        &vec_ids,
        code_threshold,
        candidate_limit,
        &[],
    )
    .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter())
}

/// Pooled-transport overload. The first hop on a session supplies `query`;
/// later hops send only its digest and reuse the backend-local immutable
/// vector, avoiding one real[] serialization per owner per hop.
#[pg_extern(
    name = "ec_distann_expand_physical_nodes",
    volatile,
    parallel_restricted
)]
#[allow(clippy::type_complexity)]
fn ec_distann_expand_physical_nodes_cached(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    query: Vec<f32>,
    query_digest: Vec<u8>,
    vec_ids: Vec<i64>,
    code_threshold: default!(Option<f32>, "NULL"),
    candidate_limit: default!(Option<i32>, "NULL"),
    skip_neighbor_vec_ids: default!(Option<Vec<i64>>, "NULL"),
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
    let (query, query_digest) =
        resolve_cached_physical_query(query, query_digest).unwrap_or_else(|error| error.raise());
    let rows = expand_physical_nodes_impl(
        index_regclass.oid(),
        &epoch_fingerprint,
        &query,
        query_digest,
        &vec_ids,
        code_threshold,
        candidate_limit,
        skip_neighbor_vec_ids.as_deref().unwrap_or(&[]),
    )
    .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter())
}

/// Task 194 benchmark-only physical-generation endpoint.  The production
/// transport uses the cached-query overload above; this sibling preserves its
/// row contract while returning owner-side service timing as response
/// sideband.  Keeping the profile endpoint separate avoids changing the
/// production SQL ABI when the attribution feature is disabled.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_expand_physical_nodes_profile(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    query: Vec<f32>,
    query_digest: Vec<u8>,
    vec_ids: Vec<i64>,
    code_threshold: default!(Option<f32>, "NULL"),
    candidate_limit: default!(Option<i32>, "NULL"),
    skip_neighbor_vec_ids: default!(Option<Vec<i64>>, "NULL"),
    expanded_locator: default!(bool, "false"),
) -> TableIterator<
    'static,
    (
        name!(vec_id, i64),
        name!(exact_dist, Option<f32>),
        name!(is_tombstone, bool),
        name!(neighbor_vec_ids, Vec<i64>),
        name!(neighbor_code_dists, Vec<f32>),
        name!(heap_block, i64),
        name!(heap_offset, i32),
        name!(owner_total_ns, i64),
        name!(owner_open_validate_ns, i64),
        name!(owner_graph_read_ns, i64),
        name!(owner_score_ns, i64),
        name!(owner_response_encode_ns, i64),
        name!(owner_response_bytes, i64),
    ),
> {
    let owner_started = Instant::now();
    let (query, query_digest) =
        resolve_cached_physical_query(query, query_digest).unwrap_or_else(|error| error.raise());
    let ids = vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let skip = skip_neighbor_vec_ids
        .unwrap_or_default()
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let open_started = Instant::now();
    let store = RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)
        .unwrap_or_else(|error| error.raise());
    let owner_open_validate_ns = duration_ns(open_started.elapsed());
    let expanded = store
        .expand(
            &query,
            query_digest,
            &ids,
            code_threshold,
            candidate_limit.map(|limit| {
                usize::try_from(limit).unwrap_or_else(|_| {
                    DistannExpandError::BadInput(
                        "ec_distann candidate_limit must be non-negative".to_owned(),
                    )
                    .raise()
                })
            }),
            &skip,
        )
        .unwrap_or_else(|error| error.raise());
    let owner_open_validate_ns = i64::try_from(owner_open_validate_ns).unwrap_or(i64::MAX);
    let owner_graph_read_ns = expanded.first().map_or(0, |node| node.owner_graph_read_ns);
    let owner_score_ns = expanded.first().map_or(0, |node| node.owner_score_ns);
    let response_started = Instant::now();
    let owner_response_bytes = expanded.iter().fold(0_usize, |total, node| {
        total
            .saturating_add(22_usize.saturating_add(node.neighbor_vec_ids.len().saturating_mul(12)))
    });
    let rows = expanded
        .into_iter()
        .map(|node| {
            (
                i64::from_le_bytes(node.vec_id.to_le_bytes()),
                node.exact_dist,
                node.is_tombstone,
                node.neighbor_vec_ids
                    .into_iter()
                    .map(|id| i64::from_le_bytes(id.to_le_bytes()))
                    .collect(),
                node.neighbor_code_dists,
                if expanded_locator {
                    i64::from(node.heap_tid.block_number)
                } else {
                    -1
                },
                if expanded_locator {
                    i32::from(node.heap_tid.offset_number)
                } else {
                    -1
                },
            )
        })
        .collect::<Vec<_>>();
    let owner_response_encode_ns =
        i64::try_from(duration_ns(response_started.elapsed())).unwrap_or(i64::MAX);
    let owner_response_bytes = i64::try_from(owner_response_bytes).unwrap_or(i64::MAX);
    let owner_total_ns = i64::try_from(duration_ns(owner_started.elapsed())).unwrap_or(i64::MAX);
    TableIterator::new(rows.into_iter().map(move |row| {
        (
            row.0,
            row.1,
            row.2,
            row.3,
            row.4,
            row.5,
            row.6,
            owner_total_ns,
            owner_open_validate_ns,
            owner_graph_read_ns,
            owner_score_ns,
            owner_response_encode_ns,
            owner_response_bytes,
        )
    }))
}

/// Task 179 benchmark-only control endpoint. This is absent from normal
/// production builds; the opt-in feature restores the removed owner-wide O(N)
/// seed scan so persisted-head seeding can be measured on otherwise-current
/// code.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_physical_seed_candidates_benchmark(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    query: Vec<f32>,
    limit: i32,
) -> TableIterator<'static, (name!(vec_id, i64), name!(code_dist, f32))> {
    let limit = usize::try_from(limit)
        .ok()
        .filter(|limit| (1..=4096).contains(limit))
        .unwrap_or_else(|| pgrx::error!("physical seed limit must be in 1..=4096"));
    let rows = RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)
        .and_then(|store| store.seed_candidates(&query, limit))
        .map(|seeds| {
            seeds
                .into_iter()
                .map(|seed| (i64::from_le_bytes(seed.vec_id.to_le_bytes()), seed.dist))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter())
}

/// Task 181 compact per-query loss decomposition.  This compares three seed
/// sets on one immutable generation: persisted bounded-head search, exact
/// scoring of the same landmark membership, and the full-owner RaBitQ oracle.
/// It is absent from production builds and never participates in result
/// selection.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_physical_seed_coverage_benchmark(
    index_regclass: PgRelation,
    query: Vec<f32>,
    search_width: i32,
    seed_count: i32,
) -> TableIterator<
    'static,
    (
        name!(query_region, i32),
        name!(owner_seed_count, i32),
        name!(owner_in_head, i32),
        name!(bounded_owner_overlap, i32),
        name!(exact_owner_overlap, i32),
        name!(zero_owner_represented, bool),
        name!(first_owner_head_rank, i32),
        name!(best_score_gap, f32),
        name!(owner_best_dist, f32),
        name!(head_best_dist, f32),
    ),
> {
    let search_width = usize::try_from(search_width)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| pgrx::error!("head search width must be in 1..=4096"));
    let seed_count = usize::try_from(seed_count)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| pgrx::error!("head seed count must be in 1..=4096"));
    let scan = PhysicalGenerationScan::open(index_regclass.oid())
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let head = scan
        .head_index
        .as_ref()
        .unwrap_or_else(|| pgrx::error!("EC_HEAD_SAMPLE: active generation has no head"));
    let owner = scan
        .owner_scan_seed_candidates(&query, seed_count)
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let bounded = head.search(&query, search_width, seed_count);
    let exact = head.search_exact(&query, seed_count);
    let owner_ids = owner
        .iter()
        .map(|seed| seed.vec_id)
        .collect::<std::collections::HashSet<_>>();
    let owner_in_head = owner
        .iter()
        .filter(|seed| head.contains_vec_id(seed.vec_id))
        .count();
    let bounded_overlap = bounded
        .iter()
        .filter(|seed| owner_ids.contains(&seed.vec_id))
        .count();
    let exact_overlap = exact
        .iter()
        .filter(|seed| owner_ids.contains(&seed.vec_id))
        .count();
    let first_rank = owner
        .iter()
        .position(|seed| head.contains_vec_id(seed.vec_id))
        .map(|rank| rank + 1)
        .unwrap_or(0);
    let owner_best = owner.first().map(|seed| seed.dist).unwrap_or(f32::INFINITY);
    let head_best = exact.first().map(|seed| seed.dist).unwrap_or(f32::INFINITY);
    let row = (
        i32::from(super::head_sample::benchmark_geometry_region(&query)),
        i32::try_from(owner.len()).unwrap_or(i32::MAX),
        i32::try_from(owner_in_head).unwrap_or(i32::MAX),
        i32::try_from(bounded_overlap).unwrap_or(i32::MAX),
        i32::try_from(exact_overlap).unwrap_or(i32::MAX),
        owner_in_head == 0,
        i32::try_from(first_rank).unwrap_or(i32::MAX),
        head_best - owner_best,
        owner_best,
        head_best,
    );
    TableIterator::once(row)
}

/// Task 185 gateway/basin attribution. This runs the ordinary physical scan
/// with a benchmark-only first-arrival origin mask on the bounded frontier.
/// `hit_origin_masks` lets the harness intersect returned ids with a disjoint
/// truth slice; the aggregate counts expose per-seed expanded-region overlap.
/// It is intentionally absent from production builds and never affects seed
/// selection or result ordering.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_physical_seed_gateway_trace_benchmark(
    index_regclass: PgRelation,
    query: Vec<f32>,
    top_k: i32,
) -> TableIterator<
    'static,
    (
        name!(seed_ids, Vec<i64>),
        name!(seed_expanded_counts, Vec<i32>),
        name!(seed_hit_counts, Vec<i32>),
        name!(hit_ids, Vec<i64>),
        name!(hit_origin_masks, Vec<i64>),
        name!(expanded_unique, i64),
        name!(expanded_overlap, i64),
        name!(records_expanded, i32),
        name!(rounds_executed, i32),
    ),
> {
    let top_k = usize::try_from(top_k)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| pgrx::error!("gateway trace top_k must be in 1..=4096"));
    let index_oid = index_regclass.oid();
    drop(index_regclass);
    let index_guard = IndexRelationGuard::try_access_share(index_oid)
        .unwrap_or_else(|| pgrx::error!("gateway trace could not open index relation"));
    let source_attnum = super::routine::indexed_ecvector_attnum(index_guard.as_ptr())
        .unwrap_or_else(|error| {
            pgrx::error!("gateway trace source column resolution failed: {error}")
        });
    // SAFETY: SQL function execution has an active snapshot; the relation
    // guard remains live for the duration of the physical search.
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        pgrx::error!("gateway trace has no active snapshot");
    }
    let scan =
        PhysicalGenerationScan::open(index_oid).unwrap_or_else(|error| pgrx::error!("{error}"));
    let (result, trace) = super::stage_counters::with_seed_trace(|| {
        scan.search(snapshot, source_attnum, &query, top_k)
    });
    let collection = result.unwrap_or_else(|error| pgrx::error!("{error}"));
    let seed_ids = trace
        .seed_ids
        .into_iter()
        .map(|value| i64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let seed_expanded_counts = trace
        .seed_expanded_counts
        .into_iter()
        .map(|value| i32::try_from(value).unwrap_or(i32::MAX))
        .collect::<Vec<_>>();
    let seed_hit_counts = trace
        .seed_hit_counts
        .into_iter()
        .map(|value| i32::try_from(value).unwrap_or(i32::MAX))
        .collect::<Vec<_>>();
    let hit_ids = trace
        .hit_ids
        .into_iter()
        .map(|value| i64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let hit_origin_masks = trace
        .hit_origin_masks
        .into_iter()
        .map(i64::from)
        .collect::<Vec<_>>();
    TableIterator::once((
        seed_ids,
        seed_expanded_counts,
        seed_hit_counts,
        hit_ids,
        hit_origin_masks,
        i64::try_from(trace.expanded_unique).unwrap_or(i64::MAX),
        i64::try_from(trace.expanded_overlap).unwrap_or(i64::MAX),
        i32::try_from(collection.counters.records_expanded).unwrap_or(i32::MAX),
        i32::try_from(collection.counters.rounds_executed).unwrap_or(i32::MAX),
    ))
}

/// Task 185 candidate-level attribution. This repeats the same physical scan
/// with exactly one member of the control's returned seed list. It isolates a
/// candidate's bounded traversal contribution from the ordering/competition
/// effects of the 32-seed beam. The endpoint is benchmark-only and does not
/// expose or change the production seed-selection path.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_physical_seed_isolated_gateway_trace_benchmark(
    index_regclass: PgRelation,
    query: Vec<f32>,
    top_k: i32,
    seed_position: i32,
) -> TableIterator<
    'static,
    (
        name!(seed_ids, Vec<i64>),
        name!(seed_expanded_counts, Vec<i32>),
        name!(seed_hit_counts, Vec<i32>),
        name!(hit_ids, Vec<i64>),
        name!(hit_origin_masks, Vec<i64>),
        name!(expanded_unique, i64),
        name!(expanded_overlap, i64),
        name!(records_expanded, i32),
        name!(rounds_executed, i32),
    ),
> {
    let top_k = usize::try_from(top_k)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| pgrx::error!("isolated gateway trace top_k must be in 1..=4096"));
    let seed_position = usize::try_from(seed_position)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| {
            pgrx::error!("isolated gateway trace seed_position must be in 1..=4096")
        });
    let index_oid = index_regclass.oid();
    drop(index_regclass);
    let index_guard = IndexRelationGuard::try_access_share(index_oid)
        .unwrap_or_else(|| pgrx::error!("isolated gateway trace could not open index relation"));
    let source_attnum = super::routine::indexed_ecvector_attnum(index_guard.as_ptr())
        .unwrap_or_else(|error| {
            pgrx::error!("isolated gateway trace source column resolution failed: {error}")
        });
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        pgrx::error!("isolated gateway trace has no active snapshot");
    }
    let scan =
        PhysicalGenerationScan::open(index_oid).unwrap_or_else(|error| pgrx::error!("{error}"));
    let seeds = scan
        .select_seed_candidates(&query)
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let seed = seeds.get(seed_position - 1).copied().unwrap_or_else(|| {
        pgrx::error!(
            "isolated gateway trace seed_position {} exceeds returned seed count {}",
            seed_position,
            seeds.len()
        )
    });
    let (result, trace) = super::stage_counters::with_seed_trace(|| {
        scan.search_with_seed_candidates(
            snapshot,
            source_attnum,
            &query,
            top_k,
            Some(std::slice::from_ref(&seed)),
        )
    });
    let collection = result.unwrap_or_else(|error| pgrx::error!("{error}"));
    let seed_ids = trace
        .seed_ids
        .into_iter()
        .map(|value| i64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let seed_expanded_counts = trace
        .seed_expanded_counts
        .into_iter()
        .map(|value| i32::try_from(value).unwrap_or(i32::MAX))
        .collect::<Vec<_>>();
    let seed_hit_counts = trace
        .seed_hit_counts
        .into_iter()
        .map(|value| i32::try_from(value).unwrap_or(i32::MAX))
        .collect::<Vec<_>>();
    let hit_ids = trace
        .hit_ids
        .into_iter()
        .map(|value| i64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let hit_origin_masks = trace
        .hit_origin_masks
        .into_iter()
        .map(i64::from)
        .collect::<Vec<_>>();
    TableIterator::once((
        seed_ids,
        seed_expanded_counts,
        seed_hit_counts,
        hit_ids,
        hit_origin_masks,
        i64::try_from(trace.expanded_unique).unwrap_or(i64::MAX),
        i64::try_from(trace.expanded_overlap).unwrap_or(i64::MAX),
        i32::try_from(collection.counters.records_expanded).unwrap_or(i32::MAX),
        i32::try_from(collection.counters.rounds_executed).unwrap_or(i32::MAX),
    ))
}

/// Task 185 arbitrary-head candidate attribution. This exact-scores the
/// persisted head membership, selects one ranked head member, and reruns the
/// physical scan with only that candidate. It is benchmark-only: it does not
/// alter the production seed selector or persist a policy.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_physical_head_candidate_trace_benchmark(
    index_regclass: PgRelation,
    query: Vec<f32>,
    top_k: i32,
    candidate_position: i32,
) -> TableIterator<
    'static,
    (
        name!(seed_ids, Vec<i64>),
        name!(seed_expanded_counts, Vec<i32>),
        name!(seed_hit_counts, Vec<i32>),
        name!(hit_ids, Vec<i64>),
        name!(hit_origin_masks, Vec<i64>),
        name!(expanded_unique, i64),
        name!(expanded_overlap, i64),
        name!(records_expanded, i32),
        name!(rounds_executed, i32),
    ),
> {
    let top_k = usize::try_from(top_k)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| pgrx::error!("head candidate trace top_k must be in 1..=4096"));
    let candidate_position = usize::try_from(candidate_position)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| {
            pgrx::error!("head candidate trace candidate_position must be in 1..=4096")
        });
    let index_oid = index_regclass.oid();
    drop(index_regclass);
    let index_guard = IndexRelationGuard::try_access_share(index_oid)
        .unwrap_or_else(|| pgrx::error!("head candidate trace could not open index relation"));
    let source_attnum = super::routine::indexed_ecvector_attnum(index_guard.as_ptr())
        .unwrap_or_else(|error| {
            pgrx::error!("head candidate trace source column resolution failed: {error}")
        });
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        pgrx::error!("head candidate trace has no active snapshot");
    }
    let scan =
        PhysicalGenerationScan::open(index_oid).unwrap_or_else(|error| pgrx::error!("{error}"));
    let candidates = scan
        .benchmark_head_candidates(&query)
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let candidate = candidates
        .get(candidate_position - 1)
        .copied()
        .unwrap_or_else(|| {
            pgrx::error!(
                "head candidate trace candidate_position {} exceeds head candidate count {}",
                candidate_position,
                candidates.len()
            )
        });
    let (result, trace) = super::stage_counters::with_seed_trace(|| {
        scan.search_with_seed_candidates(
            snapshot,
            source_attnum,
            &query,
            top_k,
            Some(std::slice::from_ref(&candidate)),
        )
    });
    let collection = result.unwrap_or_else(|error| pgrx::error!("{error}"));
    let seed_ids = trace
        .seed_ids
        .into_iter()
        .map(|value| i64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let seed_expanded_counts = trace
        .seed_expanded_counts
        .into_iter()
        .map(|value| i32::try_from(value).unwrap_or(i32::MAX))
        .collect::<Vec<_>>();
    let seed_hit_counts = trace
        .seed_hit_counts
        .into_iter()
        .map(|value| i32::try_from(value).unwrap_or(i32::MAX))
        .collect::<Vec<_>>();
    let hit_ids = trace
        .hit_ids
        .into_iter()
        .map(|value| i64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let hit_origin_masks = trace
        .hit_origin_masks
        .into_iter()
        .map(i64::from)
        .collect::<Vec<_>>();
    TableIterator::once((
        seed_ids,
        seed_expanded_counts,
        seed_hit_counts,
        hit_ids,
        hit_origin_masks,
        i64::try_from(trace.expanded_unique).unwrap_or(i64::MAX),
        i64::try_from(trace.expanded_overlap).unwrap_or(i64::MAX),
        i32::try_from(collection.counters.records_expanded).unwrap_or(i32::MAX),
        i32::try_from(collection.counters.rounds_executed).unwrap_or(i32::MAX),
    ))
}

/// Task 200 attribution endpoint.  This intentionally repeats only the
/// coordinator-side `PhysicalGenerationScan::open` call and drops each scan;
/// it excludes owner seed scanning and head searches from the measurement.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_physical_scan_open_benchmark(index_regclass: PgRelation, iterations: i32) -> i32 {
    let iterations = usize::try_from(iterations)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| pgrx::error!("scan-open iterations must be in 1..=4096"));
    for _ in 0..iterations {
        PhysicalGenerationScan::open(index_regclass.oid())
            .unwrap_or_else(|error| pgrx::error!("{error}"));
    }
    i32::try_from(iterations).expect("scan-open iterations fit integer")
}

/// Task 200 attribution endpoint.  The physical scan is opened once, then
/// only the owner-side seed path is repeated.  This is the next attribution
/// arm if the coordinator open-only experiment remains bounded.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_physical_owner_seed_scan_benchmark(
    index_regclass: PgRelation,
    query: Vec<f32>,
    iterations: i32,
    seed_count: i32,
) -> i32 {
    let iterations = usize::try_from(iterations)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| pgrx::error!("owner-scan iterations must be in 1..=4096"));
    let seed_count = usize::try_from(seed_count)
        .ok()
        .filter(|value| (1..=4096).contains(value))
        .unwrap_or_else(|| pgrx::error!("owner-scan seed count must be in 1..=4096"));
    let scan = PhysicalGenerationScan::open(index_regclass.oid())
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    for _ in 0..iterations {
        scan.owner_scan_seed_candidates(&query, seed_count)
            .unwrap_or_else(|error| pgrx::error!("{error}"));
    }
    i32::try_from(iterations).expect("owner-scan iterations fit integer")
}

/// Task 183 same-seed attribution helper. The digest covers the ordered seed
/// IDs selected by the exact production seed-selection path under the active
/// benchmark controls. Neighbor scoring is intentionally absent from the
/// digest domain, so the suite can prove that RaBitQ and exact-neighbor arms
/// differ only after seed selection.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, strict, parallel_restricted)]
fn ec_distann_physical_seed_id_digest(index_regclass: PgRelation, query: Vec<f32>) -> Vec<u8> {
    const DOMAIN: &[u8] = b"ec_distann_seed_ids_v1\0";

    let scan = PhysicalGenerationScan::open(index_regclass.oid())
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let seeds = scan
        .select_seed_candidates(&query)
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(
        u32::try_from(seeds.len())
            .expect("benchmark seed count is bounded by 4096")
            .to_le_bytes(),
    );
    for seed in seeds {
        hasher.update(seed.vec_id.to_le_bytes());
    }
    hasher.finalize().to_vec()
}

/// Records the compiled physical seed strategy in benchmark provenance.
#[pg_extern(stable, parallel_safe)]
fn ec_distann_physical_seed_strategy() -> String {
    super::options::current_physical_seed_mode()
        .unwrap_or_else(|error| pgrx::error!("{error}"))
        .as_str()
        .to_owned()
}

/// Records the benchmark-only construction policy used for a Task 181 build.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(stable, parallel_safe)]
fn ec_distann_physical_head_policy() -> String {
    super::options::current_benchmark_head_policy()
        .unwrap_or_else(|error| pgrx::error!("{error}"))
        .as_str()
        .to_owned()
}

/// Attests the immutable production head policy bound to the active generation.
#[pg_extern(stable, strict)]
#[allow(clippy::type_complexity)]
fn ec_distann_active_head_policy(
    index_regclass: PgRelation,
) -> TableIterator<
    'static,
    (
        name!(head_policy, String),
        name!(scoring_mode, String),
        name!(training_query_count, i32),
        name!(training_query_digest, Vec<u8>),
        name!(head_index_cap, i32),
        name!(returned_seed_count, i32),
        name!(sample_count, i32),
        name!(head_sample_digest, Vec<u8>),
    ),
> {
    let index_oid = index_regclass.oid();
    drop(index_regclass);
    let result = (|| -> Result<_, String> {
        let (control, _handle, _metadata, logical_index_uuid) =
            super::generation_store::open_control_index(
                index_oid,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                "ec_distann_active_head_policy",
            )?;
        drop(control);
        let active = generation_catalog::extension_relation_name("ec_distann_active_epoch")?;
        let candidate = generation_catalog::extension_relation_name("ec_distann_build_candidate")?;
        let state =
            generation_catalog::extension_relation_name("ec_distann_generation_head_state")?;
        let (
            manifest_bytes,
            state_policy,
            state_training_count,
            state_training_digest,
            sample_count,
            sample_digest,
        ) = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT candidate.epoch_manifest, state.head_policy,
                                state.training_query_count, state.training_query_digest,
                                state.sample_count,
                                state.head_sample_digest
                           FROM {active} active
                           JOIN {candidate} candidate
                             USING (index_oid, logical_index_uuid, build_id)
                           JOIN {state} state
                             USING (index_oid, logical_index_uuid, build_id)
                          WHERE active.index_oid = $1::oid
                            AND active.logical_index_uuid = $2::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into()],
                )
                .map_err(|error| format!("EC_HEAD_SAMPLE: active policy lookup failed: {error}"))?
                .map(|row| {
                    Ok((
                        row["epoch_manifest"]
                            .value::<Vec<u8>>()?
                            .ok_or("manifest NULL")?,
                        row["head_policy"]
                            .value::<i16>()?
                            .ok_or("head policy NULL")?,
                        row["training_query_count"]
                            .value::<i32>()?
                            .ok_or("training query count NULL")?,
                        row["training_query_digest"]
                            .value::<Vec<u8>>()?
                            .ok_or("training query digest NULL")?,
                        row["sample_count"]
                            .value::<i32>()?
                            .ok_or("sample count NULL")?,
                        row["head_sample_digest"]
                            .value::<Vec<u8>>()?
                            .ok_or("head sample digest NULL")?,
                    ))
                })
                .next()
                .transpose()
                .map_err(|error: Box<dyn std::error::Error + Send + Sync>| {
                    format!("EC_HEAD_SAMPLE: active policy decode failed: {error}")
                })?
                .ok_or_else(|| "EC_GENERATION_MISSING: active head policy is absent".to_owned())
        })?;
        let manifest = super::manifest_v2::DistannEpochManifestV2::decode(&manifest_bytes)?;
        if sample_digest.as_slice() != manifest.head_sample_digest.as_slice() {
            return Err("EC_HEAD_SAMPLE: active sample digest disagrees with manifest".to_owned());
        }
        let options = manifest.build_options.options;
        if state_policy != options.head_policy as i16
            || state_training_count != options.training_query_count as i32
            || state_training_digest.as_slice() != options.training_query_digest.as_slice()
        {
            return Err(
                "EC_HEAD_SAMPLE: active head policy metadata disagrees with manifest".to_owned(),
            );
        }
        let returned_seed_count = super::head_sample::TRAINED_HEAD_SEED_COUNT.min(
            usize::try_from(sample_count)
                .map_err(|_| "EC_HEAD_SAMPLE: active sample count is negative".to_owned())?,
        );
        let scoring_mode = match options.head_policy {
            super::generation_descriptor::DistannHeadPolicy::CurrentSampleGraph => {
                "persisted_head_graph"
            }
            super::generation_descriptor::DistannHeadPolicy::TrainingLandmarksExact => {
                "exact_landmark_scan"
            }
        };
        Ok((
            options.head_policy.as_str().to_owned(),
            scoring_mode.to_owned(),
            i32::try_from(options.training_query_count)
                .map_err(|_| "EC_HEAD_SAMPLE: training count exceeds integer".to_owned())?,
            options.training_query_digest.to_vec(),
            i32::try_from(options.head_index_cap)
                .map_err(|_| "EC_HEAD_SAMPLE: head cap exceeds integer".to_owned())?,
            i32::try_from(returned_seed_count).expect("trained head seed count fits integer"),
            sample_count,
            sample_digest,
        ))
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::once(result)
}

/// Surface the physical head-construction marker persisted with the active
/// generation head state. This is deliberately separate from the existing
/// policy function so its stable return shape remains compatible with older
/// clients.
#[pg_extern(stable, strict)]
fn ec_distann_active_head_construction(
    index_regclass: PgRelation,
) -> TableIterator<
    'static,
    (
        name!(head_construction, String),
        name!(marker_attested, bool),
    ),
> {
    let index_oid = index_regclass.oid();
    drop(index_regclass);
    let result = (|| -> Result<_, String> {
        let (control, _handle, _metadata, logical_index_uuid) =
            super::generation_store::open_control_index(
                index_oid,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                "ec_distann_active_head_construction",
            )?;
        drop(control);
        let active = generation_catalog::extension_relation_name("ec_distann_active_epoch")?;
        let state =
            generation_catalog::extension_relation_name("ec_distann_generation_head_state")?;
        Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT state.head_construction
                           FROM {active} active
                           JOIN {state} state
                             USING (index_oid, logical_index_uuid, build_id)
                          WHERE active.index_oid = $1::oid
                            AND active.logical_index_uuid = $2::uuid"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into()],
                )
                .map_err(|error| {
                    format!("EC_HEAD_SAMPLE: active construction lookup failed: {error}")
                })?
                .map(|row| {
                    let value = row["head_construction"]
                        .value::<i16>()
                        .map_err(|error| {
                            format!("EC_HEAD_SAMPLE: head construction decode failed: {error}")
                        })?
                        .ok_or_else(|| "head construction marker NULL".to_owned())?;
                    let construction = match value {
                        0 => "stitched_bfs",
                        1 => "partition_union",
                        other => {
                            return Err(format!(
                                "EC_HEAD_SAMPLE: invalid head construction marker {other}"
                            ))
                        }
                    };
                    Ok((construction.to_owned(), true))
                })
                .next()
                .transpose()
                .map_err(|error| {
                    format!("EC_HEAD_SAMPLE: active construction decode failed: {error}")
                })?
                .ok_or_else(|| {
                    "EC_GENERATION_MISSING: active head construction is absent".to_owned()
                })
        })
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::once(result)
}

/// Records the active physical neighbor-scoring strategy in benchmark
/// provenance. Production builds can only attest the persisted RaBitQ path.
#[pg_extern(stable, parallel_safe)]
fn ec_distann_physical_neighbor_score_mode() -> &'static str {
    if super::options::benchmark_exact_neighbor() {
        "exact_neighbor"
    } else {
        "rabitq"
    }
}

/// FR-079 physical-generation payload overload.  Projection identity is by
/// attnum and the owner resolves each binary send function from its frozen
/// row-tier schema; callers cannot select SQL function names.
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_materialize_physical_row_payloads(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    vec_ids: Vec<i64>,
    projection_attnums: Vec<i16>,
    expected_schema_fingerprint: Vec<u8>,
) -> TableIterator<
    'static,
    (
        name!(vec_id, i64),
        name!(is_tombstone, bool),
        name!(tuple_payload_missing, bool),
        name!(payload_nulls, Vec<bool>),
        name!(payload_offsets, Vec<i64>),
        name!(payload_values, Vec<u8>),
    ),
> {
    let ids = vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let rows = RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)
        .and_then(|store| {
            store.materialize_payloads(
                &ids,
                &projection_attnums,
                &expected_schema_fingerprint,
                false,
                false,
                false,
                None,
            )
        })
        .map(|batch| batch.rows)
        .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter())
}

/// Task 167 pg_test probe for the owner-side 2PC visibility retry. The
/// fixture installs a recent commit intent, enables the one-shot test fault,
/// and calls the same `resolve_nodes` path used by remote insert planning.
#[cfg(feature = "pg_test")]
#[pg_extern(volatile)]
fn ec_distann_debug_resolve_nodes_retry(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    vec_id: i64,
) -> bool {
    let vec_id = u64::from_le_bytes(vec_id.to_le_bytes());
    RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)
        .and_then(|scan| scan.resolve_nodes(&[vec_id]).map(|_| ()))
        .map(|_| true)
        .unwrap_or_else(|error| error.raise())
}

/// Task 184 benchmark-only physical payload endpoint with owner-side timing
/// metadata. Timing values repeat on each response row so the existing ordered
/// row contract remains intact; the coordinator validates that every row in a
/// request reports identical metadata.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_materialize_physical_row_payloads_profile(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
    vec_ids: Vec<i64>,
    projection_attnums: Vec<i16>,
    expected_schema_fingerprint: Vec<u8>,
    use_cached_payload_plan: bool,
    use_typed_locator: bool,
    use_packed_payload: bool,
    owner_heap_blocks: Vec<i64>,
    owner_heap_offsets: Vec<i32>,
    use_expanded_locator: bool,
) -> TableIterator<
    'static,
    (
        name!(vec_id, i64),
        name!(is_tombstone, bool),
        name!(tuple_payload_missing, bool),
        name!(payload_nulls, Vec<bool>),
        name!(payload_offsets, Vec<i64>),
        name!(payload_values, Vec<u8>),
        name!(owner_total_ns, i64),
        name!(owner_open_validate_ns, i64),
        name!(owner_node_lookup_ns, i64),
        name!(owner_payload_sql_ns, i64),
        name!(payload_bytes, i64),
    ),
> {
    let total_started = Instant::now();
    let ids = vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let open_started = Instant::now();
    let store = RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)
        .unwrap_or_else(|error| error.raise());
    let open_ns = duration_ns(open_started.elapsed());
    let owner_heap_tids = if use_expanded_locator {
        if owner_heap_blocks.len() != ids.len() || owner_heap_offsets.len() != ids.len() {
            pgrx::error!("expanded owner locator arrays must match vec_ids");
        }
        Some(
            owner_heap_blocks
                .into_iter()
                .zip(owner_heap_offsets)
                .map(|(block, offset)| {
                    let block = u32::try_from(block).unwrap_or_else(|_| {
                        pgrx::error!("expanded owner heap block is out of range")
                    });
                    let offset = u16::try_from(offset).unwrap_or_else(|_| {
                        pgrx::error!("expanded owner heap offset is out of range")
                    });
                    let tid = ItemPointer {
                        block_number: block,
                        offset_number: offset,
                    };
                    if tid == ItemPointer::INVALID {
                        pgrx::error!("expanded owner heap locator is invalid");
                    }
                    tid
                })
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };
    let batch = store
        .materialize_payloads(
            &ids,
            &projection_attnums,
            &expected_schema_fingerprint,
            use_cached_payload_plan,
            use_typed_locator,
            use_packed_payload,
            owner_heap_tids.as_deref(),
        )
        .unwrap_or_else(|error| error.raise());
    let owner_total_ns = duration_ns(total_started.elapsed());
    let owner_open_validate_ns = open_ns.saturating_add(batch.telemetry.validate_ns);
    let owner_node_lookup_ns = batch.telemetry.node_lookup_ns;
    let owner_payload_sql_ns = batch.telemetry.payload_sql_ns;
    let payload_bytes = batch
        .rows
        .iter()
        .map(|(_, _, _, nulls, offsets, values)| {
            nulls
                .len()
                .saturating_add(offsets.len())
                .saturating_add(values.len())
        })
        .sum::<usize>();
    let owner_total_ns = i64::try_from(owner_total_ns).unwrap_or(i64::MAX);
    let owner_open_validate_ns = i64::try_from(owner_open_validate_ns).unwrap_or(i64::MAX);
    let owner_node_lookup_ns = i64::try_from(owner_node_lookup_ns).unwrap_or(i64::MAX);
    let owner_payload_sql_ns = i64::try_from(owner_payload_sql_ns).unwrap_or(i64::MAX);
    let payload_bytes = i64::try_from(payload_bytes).unwrap_or(i64::MAX);
    TableIterator::new(batch.rows.into_iter().map(move |row| {
        (
            row.0,
            row.1,
            row.2,
            row.3,
            row.4,
            row.5,
            owner_total_ns,
            owner_open_validate_ns,
            owner_node_lookup_ns,
            owner_payload_sql_ns,
            payload_bytes,
        )
    }))
}

pub(crate) struct PhysicalGenerationScan {
    index_oid: pg_sys::Oid,
    descriptor: Arc<DistannGenerationDescriptor>,
    generation: Option<GenerationCatalogRow>,
    row_relation: Option<HeapRelationGuard>,
    graph_relation: Option<HeapRelationGuard>,
    directory_relation: Option<IndexRelationGuard>,
    build_id: Uuid,
    fingerprint: [u8; 34],
    descriptor_digest: [u8; 32],
    routes: Vec<PhysicalOwnerRoute>,
    head_index: Option<Arc<super::head_sample::DistannPhysicalHeadIndex>>,
    /// TRAV-30 bounded gateway routing copies (Task 210 P3); `None` when the
    /// capacity GUC is 0, the roster is single-node, or population failed.
    gateway_copies: Option<Arc<super::gateway_copy::DistannGatewayCopySet>>,
    crown: Option<Arc<super::crown_cache::DistannCrownCache>>,
    _scan_token: ScanTokenGuard,
}

pub(crate) struct PhysicalRemotePayload {
    pub(crate) payload_nulls: Vec<bool>,
    pub(crate) payload_offsets: Vec<i64>,
    pub(crate) payload_values: Vec<u8>,
}

fn bounded_replica_failure_reason(kind: &str, error: &str) -> String {
    const MAX_ERROR_CHARS: usize = 768;
    let bounded = error.chars().take(MAX_ERROR_CHARS).collect::<String>();
    if error.chars().count() > MAX_ERROR_CHARS {
        format!("{kind}: {bounded}…")
    } else {
        format!("{kind}: {bounded}")
    }
}

pub(crate) fn active_generation_identity(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
) -> Result<Option<ActiveGenerationIdentity>, String> {
    let active = generation_catalog::extension_relation_name("ec_distann_active_epoch")?;
    Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT build_id, epoch_fingerprint FROM {active}
                      WHERE index_oid = $1::oid AND logical_index_uuid = $2::uuid"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into()],
            )
            .map_err(|error| {
                format!("EC_GENERATION_MISSING: active pointer lookup failed: {error}")
            })?
            .map(|row| {
                let build_id = row["build_id"]
                    .value::<Uuid>()
                    .map_err(|_| "EC_GENERATION_MISSING: active build id decode failed".to_owned())?
                    .ok_or_else(|| "EC_GENERATION_MISSING: active build id is NULL".to_owned())?;
                let fingerprint = row["epoch_fingerprint"]
                    .value::<Vec<u8>>()
                    .map_err(|_| {
                        "EC_GENERATION_MISSING: active fingerprint decode failed".to_owned()
                    })?
                    .ok_or_else(|| "EC_GENERATION_MISSING: active fingerprint is NULL".to_owned())?
                    .try_into()
                    .map_err(|_| {
                        "EC_GENERATION_MISSING: active fingerprint is not 34 bytes".to_owned()
                    })?;
                Ok(ActiveGenerationIdentity {
                    build_id,
                    fingerprint,
                })
            })
            .next()
            .transpose()
    })
}

fn published_generation_identity(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    fingerprint: &[u8; 34],
) -> Result<Option<ActiveGenerationIdentity>, String> {
    let Some(retained) = generation_catalog::lookup_retained_generation_by_fingerprint(
        index_oid,
        logical_index_uuid,
        fingerprint,
    )?
    else {
        return Ok(None);
    };
    if retained.generation.state != super::lifecycle_state::GenerationState::Published {
        return Ok(None);
    }
    Ok(Some(ActiveGenerationIdentity {
        build_id: retained.build_id,
        fingerprint: *fingerprint,
    }))
}

impl PhysicalGenerationScan {
    /// ResourceOwner and xact callbacks run before the executor query context
    /// reset on ERROR. PostgreSQL already closed these relations and released
    /// the tracked scan token, so the later memory-context callback must only
    /// forget the Rust guards rather than invoke their Drop implementations.
    pub(crate) fn disarm_after_resource_owner_release(&mut self) {
        self._scan_token.disarm_after_transaction_cleanup();
        if let Some(relation) = self.row_relation.take() {
            std::mem::forget(relation);
        }
        if let Some(relation) = self.graph_relation.take() {
            std::mem::forget(relation);
        }
        if let Some(relation) = self.directory_relation.take() {
            std::mem::forget(relation);
        }
    }

    pub(crate) fn open(index_oid: pg_sys::Oid) -> Result<Self, String> {
        match Self::open_once_with_fingerprint(index_oid, None, true) {
            Err(error) if error.starts_with("EC_EPOCH_MISMATCH:") => {
                Self::open_once_with_fingerprint(index_oid, None, true)
            }
            result => result,
        }
    }

    /// Open a participant generation named by the coordinator's immutable
    /// fingerprint. Participant catalogs retain a Published generation but do
    /// not install a local active pointer for the coordinator-owned epoch.
    pub(crate) fn open_at_fingerprint(
        index_oid: pg_sys::Oid,
        fingerprint: [u8; 34],
    ) -> Result<Self, String> {
        Self::open_once_with_fingerprint(index_oid, Some(fingerprint), true)
    }

    /// Open an owner generation for a coordinator-planned physical insert.
    /// The owner has the immutable generation descriptor and its local graph,
    /// but the persisted head sample is coordinator-resident (FR-080).  The
    /// caller therefore supplies the bounded forward plan and must not cause
    /// this participant to reload or search the coordinator head.
    pub(crate) fn open_at_fingerprint_for_owner_insert(
        index_oid: pg_sys::Oid,
        fingerprint: [u8; 34],
    ) -> Result<Self, String> {
        Self::open_once_with_fingerprint(index_oid, Some(fingerprint), false)
    }

    fn open_once_with_fingerprint(
        index_oid: pg_sys::Oid,
        requested_fingerprint: Option<[u8; 34]>,
        load_head: bool,
    ) -> Result<Self, String> {
        let (control, _handle, _metadata, logical_index_uuid) =
            super::generation_store::open_control_index(
                index_oid,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
                "physical generation scan",
            )?;
        drop(control);

        // Resolve, pin, then revalidate. If activation changes between the
        // first read and registration, the exact old fingerprint is pinned
        // before its relations can retire; the second read rejects the stale
        // attempt and the bounded open() retry above resolves the successor.
        let first = match requested_fingerprint {
            Some(fingerprint) => {
                published_generation_identity(index_oid, logical_index_uuid, &fingerprint)?
                    .ok_or_else(|| {
                        "EC_GENERATION_MISSING: logical index has no published epoch".to_owned()
                    })?
            }
            None => {
                active_generation_identity(index_oid, logical_index_uuid)?.ok_or_else(|| {
                    "EC_GENERATION_MISSING: logical index has no active epoch".to_owned()
                })?
            }
        };
        let scan_token =
            ScanTokenGuard::register_checked(logical_index_uuid, first.fingerprint, || {
                super::coordinator_retirement::ensure_fingerprint_not_retiring(
                    index_oid,
                    logical_index_uuid,
                    &first.fingerprint,
                )
            })
            .map_err(|(error, detail)| {
                detail.unwrap_or_else(|| error.stable_message().to_owned())
            })?;
        let active = match requested_fingerprint {
            Some(fingerprint) => {
                published_generation_identity(index_oid, logical_index_uuid, &fingerprint)?
                    .ok_or_else(|| {
                        "EC_GENERATION_MISSING: published epoch disappeared during registration"
                            .to_owned()
                    })?
            }
            None => {
                active_generation_identity(index_oid, logical_index_uuid)?.ok_or_else(|| {
                    "EC_GENERATION_MISSING: active epoch disappeared during registration".to_owned()
                })?
            }
        };
        if active.build_id != first.build_id || active.fingerprint != first.fingerprint {
            return Err(
                "EC_EPOCH_MISMATCH: active epoch changed during scan registration".to_owned(),
            );
        }

        let (descriptor, descriptor_digest, head_index, mut gateway_copies, mut crown) =
            if let Some(cached) = cached_physical_epoch(index_oid, logical_index_uuid, &active) {
                (
                    cached.descriptor,
                    cached.descriptor_digest,
                    cached.head_index,
                    cached.gateway_copies,
                    cached.crown,
                )
            } else {
                let (generation_descriptor, generation_descriptor_digest) =
                    if requested_fingerprint.is_some() {
                        let generation = generation_catalog::lookup_generation(
                            index_oid,
                            logical_index_uuid,
                            active.build_id,
                        )?
                        .ok_or_else(|| {
                            "EC_GENERATION_MISSING: published generation is absent".to_owned()
                        })?;
                        (
                            generation.generation_descriptor,
                            generation.generation_descriptor_digest,
                        )
                    } else {
                        let candidate = super::build_coordinator::load_build_candidate(
                            index_oid,
                            logical_index_uuid,
                            active.build_id,
                        )?
                        .ok_or_else(|| {
                            "EC_GENERATION_MISSING: active build candidate is absent".to_owned()
                        })?;
                        (
                            candidate.generation_descriptor,
                            candidate.generation_descriptor_digest,
                        )
                    };
                let descriptor =
                    Arc::new(DistannGenerationDescriptor::decode(&generation_descriptor)?);
                let descriptor_digest = descriptor.digest()?;
                let identity_matches = if requested_fingerprint.is_some() {
                    let generation = generation_catalog::lookup_generation(
                        index_oid,
                        logical_index_uuid,
                        active.build_id,
                    )?
                    .ok_or_else(|| {
                        "EC_GENERATION_MISSING: published generation disappeared".to_owned()
                    })?;
                    let roster_entry = descriptor
                        .roster
                        .get(generation.owner_ordinal as usize)
                        .ok_or_else(|| {
                            "EC_NODE_DESCRIPTOR: participant owner ordinal is outside the roster"
                                .to_owned()
                        })?;
                    roster_entry.logical_index_uuid == *logical_index_uuid.as_bytes()
                        && roster_entry.node_id == generation.node_id
                } else {
                    descriptor.coordinator_logical_index_uuid == *logical_index_uuid.as_bytes()
                };
                if descriptor_digest != generation_descriptor_digest || !identity_matches {
                    return Err(
                        "EC_GENERATION_DESCRIPTOR: active generation descriptor identity mismatch"
                            .to_owned(),
                    );
                }
                let head_index = if load_head {
                    let (head_sample, head_graph, manifest_build_options) =
                        super::head_sample::load_head_sample(
                            index_oid,
                            logical_index_uuid,
                            active.build_id,
                            &active.fingerprint,
                        )?;
                    super::head_sample::DistannPhysicalHeadIndex::load(
                        head_sample,
                        head_graph,
                        usize::from(manifest_build_options.graph_degree),
                        manifest_build_options.options.head_policy,
                    )?
                    .map(Arc::new)
                } else {
                    None
                };
                cache_physical_epoch(CachedPhysicalEpoch {
                    index_oid,
                    logical_index_uuid,
                    build_id: active.build_id,
                    fingerprint: active.fingerprint,
                    descriptor: Arc::clone(&descriptor),
                    descriptor_digest,
                    head_index: head_index.clone(),
                    gateway_copies: None,
                    crown: None,
                });
                (descriptor, descriptor_digest, head_index, None, None)
            };
        let routes = if !load_head && requested_fingerprint.is_some() {
            physical_owner_routes_for_owner_insert(index_oid, &descriptor)?
        } else {
            physical_owner_routes(
                index_oid,
                logical_index_uuid,
                active.build_id,
                descriptor.roster.len(),
            )?
        };
        // TRAV-30 (Task 210 P3): populate the bounded gateway copies once per
        // cached epoch. The gateway set is the FR-080 head membership — already
        // bounded and coordinator-resident — and only routing payload moves.
        //
        // The GUC is part of the cache validity, not just a populate-time
        // input (004 review, 2026-07-31): a set built under one capacity is
        // discarded the moment the same backend runs with a different one, so
        // `SET ec_distann.gateway_copy_capacity = 0` genuinely disables an
        // already-populated set and capacity changes cannot leak a stale size
        // into an A/B arm. The observability counters reset with the discard.
        let gateway_capacity = super::options::gateway_copy_capacity();
        if gateway_copies
            .as_ref()
            .is_some_and(|set| set.capacity() != gateway_capacity)
        {
            gateway_copies = None;
            super::gateway_copy::record_cleared();
            cache_physical_epoch(CachedPhysicalEpoch {
                index_oid,
                logical_index_uuid,
                build_id: active.build_id,
                fingerprint: active.fingerprint,
                descriptor: Arc::clone(&descriptor),
                descriptor_digest,
                head_index: head_index.clone(),
                gateway_copies: None,
                crown: crown.clone(),
            });
        }
        if gateway_copies.is_none() && gateway_capacity > 0 {
            if let Some(head) = head_index.as_ref() {
                if let Some(populated) = populate_gateway_copies(
                    index_oid,
                    &active.fingerprint,
                    &descriptor,
                    &routes,
                    head.members(),
                ) {
                    let populated = Arc::new(populated);
                    gateway_copies = Some(Arc::clone(&populated));
                    cache_physical_epoch(CachedPhysicalEpoch {
                        index_oid,
                        logical_index_uuid,
                        build_id: active.build_id,
                        fingerprint: active.fingerprint,
                        descriptor: Arc::clone(&descriptor),
                        descriptor_digest,
                        head_index: head_index.clone(),
                        gateway_copies: Some(populated),
                        crown: crown.clone(),
                    });
                }
            }
        }
        let crown_capacity = super::options::crown_capacity();
        if crown
            .as_ref()
            .is_some_and(|cache| cache.capacity() != crown_capacity)
        {
            crown = None;
            super::crown_cache::record_cleared();
            cache_physical_epoch(CachedPhysicalEpoch {
                index_oid,
                logical_index_uuid,
                build_id: active.build_id,
                fingerprint: active.fingerprint,
                descriptor: Arc::clone(&descriptor),
                descriptor_digest,
                head_index: head_index.clone(),
                gateway_copies: gateway_copies.clone(),
                crown: None,
            });
        }
        if crown_capacity == 0 {
            super::crown_cache::record_cleared();
        }
        if crown.is_none() && crown_capacity > 0 {
            if let Some(head) = head_index.as_ref() {
                if let Some(populated) = populate_crown_cache(
                    index_oid,
                    &active.fingerprint,
                    &descriptor,
                    &routes,
                    head.members(),
                ) {
                    let populated = Arc::new(populated);
                    crown = Some(Arc::clone(&populated));
                    super::crown_cache::record_population(&populated);
                    cache_physical_epoch(CachedPhysicalEpoch {
                        index_oid,
                        logical_index_uuid,
                        build_id: active.build_id,
                        fingerprint: active.fingerprint,
                        descriptor: Arc::clone(&descriptor),
                        descriptor_digest,
                        head_index: head_index.clone(),
                        gateway_copies: gateway_copies.clone(),
                        crown: Some(populated),
                    });
                }
            }
        }
        if let Some(crown) = crown.as_ref() {
            super::crown_cache::record_population(crown);
        }
        let generation =
            generation_catalog::lookup_generation(index_oid, logical_index_uuid, active.build_id)?;
        let (row_relation, graph_relation, directory_relation) = match generation.as_ref() {
            Some(generation) => {
                if generation.state != super::lifecycle_state::GenerationState::Published {
                    return Err(format!(
                        "EC_GENERATION_MISSING: active generation is {} rather than Published",
                        generation.state
                    ));
                }
                if load_head {
                    let local_route =
                        routes
                            .get(generation.owner_ordinal as usize)
                            .ok_or_else(|| {
                                "EC_NODE_DESCRIPTOR: local generation owner is outside the roster"
                                    .to_owned()
                            })?;
                    if !local_route.is_local {
                        return Err(
                            "EC_NODE_DESCRIPTOR: local generation owner is not the local binding"
                                .to_owned(),
                        );
                    }
                }
                if generation.generation_descriptor_digest != descriptor_digest {
                    return Err("EC_GENERATION_DESCRIPTOR: local generation descriptor differs from candidate".to_owned());
                }
                let row = HeapRelationGuard::try_access_share(generation.row_tier_relid)
                    .ok_or_else(|| {
                        "EC_GENERATION_MISSING: row-tier relation is absent".to_owned()
                    })?;
                let graph = HeapRelationGuard::try_access_share(generation.graph_store_relid)
                    .ok_or_else(|| {
                        "EC_GENERATION_MISSING: graph-store relation is absent".to_owned()
                    })?;
                let Some(directory) =
                    IndexRelationGuard::try_access_share(generation.directory_relid)
                else {
                    return Err("EC_GENERATION_MISSING: graph directory is absent".to_owned());
                };
                (Some(row), Some(graph), Some(directory))
            }
            None => {
                if routes.iter().any(|route| route.is_local) {
                    return Err(
                        "EC_GENERATION_MISSING: local binding has no active generation".to_owned(),
                    );
                }
                (None, None, None)
            }
        };
        Ok(Self {
            index_oid,
            descriptor,
            generation,
            row_relation,
            graph_relation,
            directory_relation,
            build_id: active.build_id,
            fingerprint: active.fingerprint,
            descriptor_digest,
            routes,
            head_index,
            gateway_copies,
            crown,
            _scan_token: scan_token,
        })
    }

    pub(crate) fn row_relation(&self) -> Option<pg_sys::Relation> {
        self.row_relation.as_ref().map(HeapRelationGuard::as_ptr)
    }

    /// Return the immutable descriptor and catalog row that identify the
    /// owner-local physical relations. DML takes a fresh RowExclusive lock
    /// after this snapshot; the catalog identity is copied so the read-side
    /// scan guard can be dropped before the write lock is acquired.
    pub(crate) fn local_write_identity(
        &self,
    ) -> Result<(GenerationCatalogRow, Arc<DistannGenerationDescriptor>), String> {
        let generation = self
            .generation
            .clone()
            .ok_or_else(|| "EC_GENERATION_MISSING: local owner has no generation".to_owned())?;
        if generation.state != super::lifecycle_state::GenerationState::Published {
            return Err(format!(
                "EC_GENERATION_MISSING: local generation is {} rather than Published",
                generation.state
            ));
        }
        Ok((generation, Arc::clone(&self.descriptor)))
    }

    pub(crate) fn row_schema_attributes(&self) -> Vec<u16> {
        self.descriptor
            .row_schema
            .attributes
            .iter()
            .filter(|attribute| !attribute.dropped)
            .map(|attribute| attribute.attnum)
            .collect()
    }

    pub(crate) fn traversal_replica_source(
        &self,
    ) -> (
        Uuid,
        Uuid,
        [u8; 34],
        Arc<DistannGenerationDescriptor>,
        &[PhysicalOwnerRoute],
    ) {
        (
            Uuid::from_bytes(self.descriptor.coordinator_logical_index_uuid),
            self.build_id,
            self.fingerprint,
            Arc::clone(&self.descriptor),
            &self.routes,
        )
    }

    pub(crate) fn search(
        &self,
        snapshot: pg_sys::Snapshot,
        source_attnum: i32,
        query: &[f32],
        effective_top_k: usize,
    ) -> Result<DistannHitCollection, String> {
        self.search_with_seed_candidates(snapshot, source_attnum, query, effective_top_k, None)
    }

    fn search_with_seed_candidates(
        &self,
        snapshot: pg_sys::Snapshot,
        source_attnum: i32,
        query: &[f32],
        effective_top_k: usize,
        seed_override: Option<&[DistannSeedCandidate]>,
    ) -> Result<DistannHitCollection, String> {
        if query.len() != usize::from(self.descriptor.dimensions) {
            return Err(format!(
                "EC_SCHEMA_MISMATCH: query has {} dimensions, generation requires {}",
                query.len(),
                self.descriptor.dimensions
            ));
        }
        #[cfg(feature = "distann-head-attribution-benchmark")]
        super::stage_counters::record_scan();
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let prep_started = Instant::now();
        let binding = DistannCodecBinding::from_artifact(&self.descriptor.codec_artifact)?;
        let code_len = binding.code_len(usize::from(self.descriptor.dimensions))?;
        let prepared =
            DistannPreparedQuery::prepare_artifact(&self.descriptor.codec_artifact, query)?;
        let query_digest = physical_query_digest(query)?;
        let slot = self
            .row_relation
            .as_ref()
            .map(|relation| {
                TupleTableSlotGuard::single_for_heap_guard(relation).ok_or_else(|| {
                    "EC_GENERATION_MISSING: could not allocate row-tier scan slot".to_owned()
                })
            })
            .transpose()?;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::QueryPrep,
            prep_started.elapsed(),
        );

        let all_seeds = match seed_override {
            Some(seeds) => seeds.to_vec(),
            None => self.select_seed_candidates(query)?,
        };
        if all_seeds.is_empty() {
            return Ok(DistannHitCollection {
                hits: Vec::new(),
                counters: Default::default(),
                multi_node: self.routes.len() > 1,
                head_seed_count: 0,
            });
        }

        let params = DistannOrchestrationParams {
            beam_width: super::options::current_beam_width(),
            candidate_heap_limit: super::options::current_candidate_heap_limit()
                .max(super::options::current_beam_width())
                .max(effective_top_k),
            hop_rounds: super::options::current_hop_rounds(),
            top_k: effective_top_k,
            debug_fail_hop_round: super::options::debug_fail_hop_round(),
        };
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let replica_open_started = Instant::now();
        let logical_index_uuid = Uuid::from_bytes(self.descriptor.coordinator_logical_index_uuid);
        let local_ordinal = self
            .generation
            .as_ref()
            .map(|generation| generation.owner_ordinal as usize);
        // NFR-021 clause 4/5: the FR-084 traversal replica holds every owner's
        // graph record and full-precision vector on one node, so a scan served
        // from it is not distributed. It is reachable only through an explicit
        // opt-in and is never the default path. Task 210 P1.
        let replica = if super::options::allow_nonconforming_replica() {
            super::traversal_replica::ReadyTraversalReplica::open(
                self.index_oid,
                logical_index_uuid,
                self.build_id,
                &self.fingerprint,
                &self.descriptor,
                self.descriptor_digest,
                local_ordinal,
                snapshot,
                query,
                &prepared,
            )
        } else {
            Ok(None)
        };
        #[cfg(feature = "distann-head-attribution-benchmark")]
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::ReplicaOpenValidate,
            replica_open_started.elapsed(),
        );
        match replica {
            Ok(Some(mut replica)) => {
                #[cfg(feature = "distann-head-attribution-benchmark")]
                if super::options::benchmark_exact_neighbor() {
                    return Err(
                        "EC_REPLICA_ALGORITHM_MISMATCH: exact-neighbor scoring has no traversal-replica implementation"
                            .to_owned(),
                    );
                }
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::ReplicaScans,
                    1,
                );
                #[cfg(feature = "distann-head-attribution-benchmark")]
                let traversal_started = Instant::now();
                #[cfg(feature = "distann-head-attribution-benchmark")]
                let traversal = super::stage_counters::with_successful_attribution(|| {
                    replica.search(&all_seeds, params)
                });
                #[cfg(not(feature = "distann-head-attribution-benchmark"))]
                let traversal = replica.search(&all_seeds, params);
                match traversal {
                    Ok((hits, counters)) => {
                        #[cfg(feature = "distann-head-attribution-benchmark")]
                        super::stage_counters::record(
                            super::stage_counters::DistannQueryStage::TraversalTotal,
                            traversal_started.elapsed(),
                        );
                        let collection = DistannHitCollection {
                            hits,
                            counters,
                            multi_node: self.routes.len() > 1,
                            head_seed_count: all_seeds.len(),
                        };
                        emit_scan_profile_notice(
                            &collection.counters,
                            effective_top_k,
                            collection.head_seed_count,
                            collection.hits.len(),
                        );
                        return Ok(collection);
                    }
                    Err(error) => {
                        let reason = bounded_replica_failure_reason(
                            "replica traversal failed",
                            &error.to_string(),
                        );
                        super::traversal_replica::handle_ready_replica_failure(
                            self.index_oid,
                            logical_index_uuid,
                            self.build_id,
                            &self.fingerprint,
                            self.descriptor_digest,
                            &reason,
                        )?;
                        pgrx::warning!("{reason}; restarting through owners");
                        #[cfg(feature = "distann-head-attribution-benchmark")]
                        super::stage_counters::record_work(
                            super::stage_counters::DistannMaterializationWork::ReplicaFallbacks,
                            1,
                        );
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                let reason = bounded_replica_failure_reason("replica validation failed", &error);
                super::traversal_replica::handle_ready_replica_failure(
                    self.index_oid,
                    logical_index_uuid,
                    self.build_id,
                    &self.fingerprint,
                    self.descriptor_digest,
                    &reason,
                )?;
                pgrx::warning!("{reason}; restarting through owners");
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::ReplicaFallbacks,
                    1,
                );
            }
        }

        let local_expander = match (
            self.generation.as_ref(),
            self.row_relation.as_ref(),
            self.graph_relation.as_ref(),
            self.directory_relation.as_ref(),
            slot.as_ref(),
        ) {
            (
                Some(generation),
                Some(row_relation),
                Some(graph_relation),
                Some(directory_relation),
                Some(slot),
            ) => Some(GenerationExpander {
                index_oid: self.index_oid,
                generation,
                descriptor: &self.descriptor,
                graph_relation,
                directory_relation,
                row_relation,
                slot,
                snapshot,
                source_attnum,
                query,
                prepared: &prepared,
                code_len,
            }),
            (None, None, None, None, None) => None,
            _ => return Err("EC_INTERNAL: incomplete local generation reader".to_owned()),
        };
        let mut expander = PhysicalMultiOwnerExpander {
            local: local_expander,
            local_ordinal: self
                .generation
                .as_ref()
                .map(|generation| generation.owner_ordinal as usize),
            descriptor: &self.descriptor,
            routes: &self.routes,
            fingerprint: &self.fingerprint,
            query,
            query_digest: &query_digest,
            prepared: &prepared,
            code_len,
            gateway: self.gateway_copies.as_deref(),
        };
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let traversal_started = Instant::now();
        let (hits, counters) = distann_orchestrated_search(&all_seeds, &mut expander, params)
            .map_err(|error| error.to_string())?;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::TraversalTotal,
            traversal_started.elapsed(),
        );
        let collection = DistannHitCollection {
            hits,
            counters,
            multi_node: self.routes.len() > 1,
            head_seed_count: all_seeds.len(),
        };
        emit_scan_profile_notice(
            &collection.counters,
            effective_top_k,
            collection.head_seed_count,
            collection.hits.len(),
        );
        Ok(collection)
    }

    /// Preserve the retriable epoch category after the physical search path's
    /// legacy string-shaped setup errors have been collapsed.  Remote owner
    /// errors retain the stable category token in their Display form.
    pub(crate) fn classify_search_error(message: String) -> DistannExpandError {
        if message.starts_with("[EC_EPOCH_MISMATCH]") {
            DistannExpandError::EpochMismatch(message)
        } else {
            DistannExpandError::Internal(message)
        }
    }

    /// Whether head-shard replicas were populated for this epoch (Task 210 P2b).
    /// Whether this epoch's replica population attests at least
    /// `requested_count` replicas per shard. Population records the count only
    /// after every (shard, replica) pair imported, so `attested >= requested`
    /// means each server the routing hash can pick under the current GUC
    /// actually holds its copy (003a review, 2026-07-31 finding 3).
    fn head_replicas_populated(&self, requested_count: usize) -> Result<bool, String> {
        let table =
            super::generation_catalog::extension_relation_name("ec_distann_head_replica_state")?;
        let fingerprint = self.fingerprint.to_vec();
        let requested =
            i32::try_from(requested_count).map_err(|_| "replica count out of range".to_owned())?;
        Spi::connect(|client| {
            let found = client
                .select(
                    &format!(
                        "SELECT replica_count >= $3::integer AND replica_count > 0
                           FROM {table}
                          WHERE index_oid = $1::oid AND epoch_fingerprint = $2::bytea"
                    ),
                    None,
                    &[self.index_oid.into(), fingerprint.into(), requested.into()],
                )
                .map_err(|error| format!("head replica state lookup failed: {error}"))?
                .first()
                .get::<bool>(1)
                .unwrap_or(None)
                .unwrap_or(false);
            Ok::<bool, String>(found)
        })
    }

    /// Fan the FR-080 head search out across the roster (Task 210 P2a).
    ///
    /// Each owner searches the landmarks it owns under the FR-078 placement
    /// hash, reading their co-placed vectors locally, and returns at most
    /// `seed_count` seeds. The coordinator merges to `seed_count` — bounded
    /// state under NFR-021 clause 2 — and holds no landmark vectors.
    fn sharded_head_seeds(
        &self,
        query: &[f32],
        members: &[u64],
        search_width: usize,
        seed_count: usize,
        head_policy: super::generation_descriptor::DistannHeadPolicy,
    ) -> Result<Vec<DistannSeedCandidate>, String> {
        let owner_count = self.routes.len();
        if owner_count == 0 {
            return Err("EC_NODE_DESCRIPTOR: physical scan has no owner routes".to_owned());
        }
        let per_owner_members = (0..owner_count)
            .map(|owner_ordinal| {
                super::head_sample::head_shard_members(
                    members,
                    owner_ordinal,
                    owner_count,
                    self.descriptor.placement_hash_version,
                )
            })
            .collect::<Vec<_>>();
        let search_width_usize = search_width;
        let search_width = i32::try_from(search_width).unwrap_or(i32::MAX);
        let seed_count_wire = i32::try_from(seed_count).unwrap_or(i32::MAX);
        // A shard whose server is the coordinator itself is answered in
        // process; only genuinely remote shards become RPCs.
        let mut local_seeds: Vec<Vec<DistannSeedCandidate>> = Vec::new();
        // §4.1 (Task 210 P2b): a shard may be served by its owner or by one of
        // `head_replica_count` further roster nodes, chosen deterministically
        // from the query digest so head CPU is not bound to one machine.
        let query_digest = physical_query_digest(query)?;
        let replica_count = super::options::head_replica_count();
        // Routing may use a replica only for an epoch whose shard copies were
        // actually distributed by ec_distann_populate_head_replicas (P2b).
        let replicas_populated =
            replica_count > 0 && self.head_replicas_populated(replica_count).unwrap_or(false);
        let mut requests = Vec::with_capacity(owner_count);
        for (ordinal, owned) in per_owner_members.iter().enumerate() {
            if owned.is_empty() {
                continue;
            }
            // §4.1 replica routing is only admissible to a node that actually
            // holds the shard. Head shards are materialised from vectors the
            // owner already has, so until a publish-time step distributes a
            // bounded copy to replicas, the only node that can serve shard `i`
            // is its owner. Routing elsewhere makes the serving node reject
            // ids it does not own (EC_PLACEMENT), so the selection is clamped
            // rather than allowed to mis-route.
            let requested_server = super::head_sample::head_shard_server(
                ordinal,
                owner_count,
                replica_count,
                &query_digest,
            );
            let server = if requested_server == ordinal {
                ordinal
            } else if replicas_populated {
                requested_server
            } else {
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::HeadReplicaFallbacks,
                    1,
                );
                ordinal
            };
            let route = &self.routes[server];
            // The coordinator is normally itself an owner, and its own route
            // carries no conninfo. Serve that shard in-process rather than
            // dialling ourselves — same shape as the traversal expander's
            // local-ordinal branch.
            let Some(conninfo) = route.conninfo.as_deref() else {
                if !route.is_local {
                    return Err(format!(
                        "EC_NODE_DESCRIPTOR: physical owner {server} route has no connection"
                    ));
                }
                let local = RetainedGenerationScan::open(self.index_oid, &self.fingerprint)
                    .map_err(|error| error.to_string())?;
                let query_digest_local =
                    physical_query_digest(query).map_err(|error| error.to_string())?;
                local_seeds.push(
                    local
                        .head_search(
                            query,
                            query_digest_local,
                            owned,
                            search_width_usize,
                            seed_count,
                            super::ECDISTANN_DEFAULT_BUILD_LIST_SIZE as usize,
                            super::ECDISTANN_DEFAULT_ALPHA,
                            head_policy,
                        )
                        .map_err(|error| error.to_string())?,
                );
                continue;
            };
            requests.push(super::remote_transport::DistannPhysicalHeadRequest {
                conninfo,
                index_regclass: &route.remote_index_regclass,
                epoch_fingerprint: &self.fingerprint,
                query,
                member_vec_ids: owned,
                search_width,
                seed_count: seed_count_wire,
                build_list_size: super::ECDISTANN_DEFAULT_BUILD_LIST_SIZE,
                alpha: super::ECDISTANN_DEFAULT_ALPHA,
                head_policy: head_policy as i32,
            });
        }
        let responses = super::remote_transport::remote_physical_head_search_batch(&requests);
        let mut per_owner = local_seeds;
        for response in responses {
            per_owner.push(response.map_err(|error| error.to_string())?);
        }
        Ok(super::head_sample::merge_head_seeds(per_owner, seed_count))
    }

    #[cfg(feature = "distann-head-attribution-benchmark")]
    fn benchmark_head_candidates(
        &self,
        query: &[f32],
    ) -> Result<Vec<DistannSeedCandidate>, String> {
        let Some(head) = self.head_index.as_ref() else {
            return Err("EC_HEAD_SAMPLE: no persisted head is available".to_owned());
        };
        let members = head.members();
        if members.is_empty() {
            return Ok(Vec::new());
        }
        // The selector screen is defined over exact scoring of the persisted
        // 4,096-member head. Force that policy for the diagnostic even when a
        // legacy/current-sample fixture is used; this does not affect the
        // normal scan selector.
        let exact_policy = super::generation_descriptor::DistannHeadPolicy::TrainingLandmarksExact;
        if self.routes.len() > 1 || head.is_membership_only() {
            return self.sharded_head_seeds(
                query,
                members,
                members.len(),
                members.len(),
                exact_policy,
            );
        }
        Ok(head.search_exact(query, members.len()))
    }

    fn select_seed_candidates(&self, query: &[f32]) -> Result<Vec<DistannSeedCandidate>, String> {
        let fused = super::options::fused_head_hop();
        let width_pruning = super::options::crown_width_pruning();
        // The crown is a candidate cache, not a replacement for the FR-080
        // head fan-out.  Only the explicit FR-090 fused arm may use ranked
        // crown ids as traversal seeds; width pruning may use crown scores to
        // narrow complete shards.  A plain crown-on arm must therefore remain
        // result-neutral and execute the ordinary full sharded head search.
        if super::options::crown_capacity() > 0 {
            let production_seed_count = (super::options::current_beam_width() * 2).max(32);
            let search_width = super::options::current_head_search_width(production_seed_count);
            let seed_count = super::options::current_head_seed_count(production_seed_count);
            let Some(crown) = self.crown.as_ref() else {
                if fused || width_pruning {
                    super::crown_cache::record_fallback();
                }
                return self.select_seed_candidates_without_crown(query);
            };
            // A plain crown is an identity-preserving control arm.  Do not
            // spend a query scoring crown entries when neither consumer is
            // enabled: the ranked candidates would only be discarded before
            // the authoritative full-head search.
            if !fused && !width_pruning {
                return self.select_seed_candidates_without_crown(query);
            }
            let binding = DistannCodecBinding::from_artifact(&self.descriptor.codec_artifact)?;
            let code_len = binding.code_len(usize::from(self.descriptor.dimensions))?;
            let prepared =
                DistannPreparedQuery::prepare_artifact(&self.descriptor.codec_artifact, query)?;
            let seeds = crown.rank(seed_count, |code| {
                let mut distance = [0.0_f32; 1];
                prepared
                    .score_dists_batch(code, code_len, 1, &mut distance)
                    .map(|_| distance[0])
            })?;
            if seeds.is_empty() {
                super::crown_cache::record_fallback();
                return self.select_seed_candidates_without_crown(query);
            }
            if width_pruning {
                // This counter attests that the candidate arm was entered;
                // crown_width_pruned_shards separately reports actual shard
                // removals when the complete-shard rule permits them.
                super::crown_cache::record_width_pruning_activation();
            }
            if fused {
                super::crown_cache::record_seeds_served(seeds.len());
                super::crown_cache::record_fused_head_hop();
                super::crown_cache::record_fused_first_round_requested_ids(seeds.len());
                return Ok(seeds);
            }
            if width_pruning
                && super::options::current_physical_seed_mode()?
                    == super::options::PhysicalSeedMode::PersistedHead
                && self.routes.len() > 1
            {
                if let Some(head) = self.head_index.as_ref() {
                    // A width arm is intentionally more selective than the
                    // ordinary head seed count.  This leaves a measurable
                    // candidate-shard decision while retaining the complete
                    // crown-held shard safety rule below.
                    let promising = seeds
                        .iter()
                        .take((seed_count / self.routes.len().max(1)).max(1))
                        .map(|seed| seed.vec_id)
                        .collect::<std::collections::HashSet<_>>();
                    let crown_ids = crown.entry_ids().collect::<std::collections::HashSet<_>>();
                    let mut filtered_members = Vec::new();
                    let mut pruned_shards = 0;
                    for ordinal in 0..self.routes.len() {
                        let shard = super::head_sample::head_shard_members(
                            head.members(),
                            ordinal,
                            self.routes.len(),
                            self.descriptor.placement_hash_version,
                        );
                        let complete = shard.iter().all(|vec_id| crown_ids.contains(vec_id));
                        let keep =
                            !complete || shard.iter().any(|vec_id| promising.contains(vec_id));
                        if keep {
                            filtered_members.extend(shard);
                        } else {
                            pruned_shards += 1;
                        }
                    }
                    if pruned_shards > 0
                        && !filtered_members.is_empty()
                        && filtered_members.len() < head.members().len()
                    {
                        super::crown_cache::record_width_pruned_shards(pruned_shards);
                        return self.sharded_head_seeds(
                            query,
                            &filtered_members,
                            search_width,
                            seed_count,
                            head.policy(),
                        );
                    }
                }
            }
            return self.select_seed_candidates_without_crown(query);
        }
        self.select_seed_candidates_without_crown(query)
    }

    fn select_seed_candidates_without_crown(
        &self,
        query: &[f32],
    ) -> Result<Vec<DistannSeedCandidate>, String> {
        if query.len() != usize::from(self.descriptor.dimensions) {
            return Err(format!(
                "EC_SCHEMA_MISMATCH: query has {} dimensions, generation requires {}",
                query.len(),
                self.descriptor.dimensions
            ));
        }
        let production_seed_count = (super::options::current_beam_width() * 2).max(32);
        let search_width = super::options::current_head_search_width(production_seed_count);
        let seed_count = super::options::current_head_seed_count(production_seed_count);
        let seed_mode = super::options::current_physical_seed_mode()?;
        // NFR-021 clause 3 (Task 210 P2a): when the head is sharded, the
        // coordinator keeps only the bounded membership list and every owner
        // searches the landmarks it already holds.
        // A membership-only head has no coordinator-resident vectors, so the
        // sharded path is the only correct one — the persisted shape decides,
        // not the session GUC.
        let membership_only_head = self
            .head_index
            .as_ref()
            .is_some_and(|head| head.is_membership_only());
        if membership_only_head && self.routes.len() <= 1 {
            return Err(
                "EC_HEAD_SAMPLE: membership-only head requires a multi-owner roster".to_owned(),
            );
        }
        if (super::options::sharded_head_search() || membership_only_head) && self.routes.len() > 1
        {
            if let Some(head) = self.head_index.as_ref() {
                if seed_mode == super::options::PhysicalSeedMode::PersistedHead {
                    return self.sharded_head_seeds(
                        query,
                        head.members(),
                        search_width,
                        seed_count,
                        head.policy(),
                    );
                }
            }
        }
        let seeds = match seed_mode {
            super::options::PhysicalSeedMode::PersistedHead => self
                .head_index
                .as_ref()
                .map(|head| head.search_configured(query, search_width, seed_count))
                .unwrap_or_default(),
            super::options::PhysicalSeedMode::HeadSampleExact => self
                .head_index
                .as_ref()
                .map(|head| head.search_exact(query, seed_count))
                .unwrap_or_default(),
            super::options::PhysicalSeedMode::HeadHierarchy => self
                .head_index
                .as_ref()
                .map(|head| head.search_hierarchy(query, seed_count))
                .unwrap_or_default(),
            super::options::PhysicalSeedMode::OwnerScan => {
                #[cfg(feature = "distann-head-attribution-benchmark")]
                {
                    self.owner_scan_seed_candidates(query, seed_count)?
                }
                #[cfg(not(feature = "distann-head-attribution-benchmark"))]
                {
                    return Err(
                        "EC_BAD_INPUT: owner_scan is unavailable in production builds".to_owned(),
                    );
                }
            }
        };
        #[cfg(feature = "distann-head-attribution-benchmark")]
        if seeds.len() < seed_count {
            return Err(format!(
                "EC_INVARIANT: benchmark seed mode {} requested {} seeds but returned only {}",
                seed_mode.as_str(),
                seed_count,
                seeds.len()
            ));
        }
        Ok(seeds)
    }

    #[cfg(feature = "distann-head-attribution-benchmark")]
    fn owner_scan_seed_candidates(
        &self,
        query: &[f32],
        seed_limit: usize,
    ) -> Result<Vec<DistannSeedCandidate>, String> {
        let mut seeds = if self.generation.is_some() {
            RetainedGenerationScan::open(self.index_oid, &self.fingerprint)
                .and_then(|store| store.seed_candidates(query, seed_limit))
                .map_err(|error| error.to_string())?
        } else {
            Vec::new()
        };
        let seed_limit_i32 = i32::try_from(seed_limit)
            .map_err(|_| "EC_BAD_INPUT: seed limit exceeds integer".to_owned())?;
        let requests = self
            .routes
            .iter()
            .filter(|route| !route.is_local)
            .map(|route| {
                let conninfo = route.conninfo.as_deref().ok_or_else(|| {
                    format!(
                        "EC_NODE_DESCRIPTOR: physical owner {} route has no connection descriptor",
                        route.roster_ordinal
                    )
                })?;
                Ok(super::remote_transport::DistannPhysicalSeedRequest {
                    conninfo,
                    index_regclass: &route.remote_index_regclass,
                    epoch_fingerprint: &self.fingerprint,
                    query,
                    limit: seed_limit_i32,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        for response in super::remote_transport::remote_physical_seed_batch(&requests) {
            seeds.extend(response.map_err(|error| error.to_string())?);
        }
        seeds.sort_unstable_by(|left, right| {
            left.dist
                .total_cmp(&right.dist)
                .then_with(|| left.vec_id.cmp(&right.vec_id))
        });
        seeds.truncate(seed_limit);
        Ok(seeds)
    }

    pub(crate) fn materialize_remote_payloads(
        &self,
        hits: &[DistannScanHit],
        projection_attnums: &[pg_sys::AttrNumber],
    ) -> Result<HashMap<u64, PhysicalRemotePayload>, String> {
        let remote_pairs = hits
            .iter()
            .filter(|hit| hit.heap_tid == ItemPointer::INVALID)
            .map(|hit| (hit.vec_id, hit.owner_heap_tid))
            .collect::<Vec<_>>();
        self.materialize_remote_payload_pairs(&remote_pairs, projection_attnums)
    }

    /// Materialize an already-ranked subset of remote physical identities.
    /// Task 184's opt-in candidate uses this at executor demand boundaries;
    /// eager production materialization reaches the same implementation after
    /// filtering all remote hits above.
    pub(crate) fn materialize_remote_payload_ids(
        &self,
        remote_ids: &[u64],
        projection_attnums: &[pg_sys::AttrNumber],
    ) -> Result<HashMap<u64, PhysicalRemotePayload>, String> {
        let remote_pairs = remote_ids
            .iter()
            .copied()
            .map(|vec_id| (vec_id, ItemPointer::INVALID))
            .collect::<Vec<_>>();
        self.materialize_remote_payload_pairs(&remote_pairs, projection_attnums)
    }

    pub(crate) fn materialize_remote_payload_pairs(
        &self,
        remote_pairs: &[(u64, ItemPointer)],
        projection_attnums: &[pg_sys::AttrNumber],
    ) -> Result<HashMap<u64, PhysicalRemotePayload>, String> {
        let remote_ids = remote_pairs
            .iter()
            .map(|(vec_id, _)| *vec_id)
            .collect::<Vec<_>>();
        let candidate = super::options::benchmark_expanded_locator();
        let remote_locators = remote_pairs.iter().map(|(_, tid)| *tid).collect::<Vec<_>>();
        if candidate
            && remote_locators
                .iter()
                .any(|tid| *tid == ItemPointer::INVALID)
        {
            return Err(
                "EC_INTERNAL: expanded locator arm received a remote hit without owner TID"
                    .to_owned(),
            );
        }
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let prepare_started = Instant::now();
        let schema_fingerprint = self.descriptor.row_schema.fingerprint()?;
        let projection_attnums = projection_attnums
            .iter()
            .map(|attnum| i16::try_from(i32::from(*attnum)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "EC_SCHEMA_MISMATCH: projection attnum exceeds smallint".to_owned())?;
        let buckets = super::placement::group_by_owning_node(
            &remote_ids,
            self.routes.len(),
            self.descriptor.placement_hash_version,
        );
        let mut payloads = HashMap::with_capacity(remote_ids.len());
        let mut remote_work = Vec::new();
        for (ordinal, bucket) in buckets.iter().enumerate() {
            if bucket.is_empty() {
                continue;
            }
            if self
                .generation
                .as_ref()
                .is_some_and(|generation| ordinal == generation.owner_ordinal as usize)
            {
                return Err(
                    "EC_INTERNAL: local owner produced a remote physical hit locator".to_owned(),
                );
            }
            let ids = bucket.iter().map(|(_, vec_id)| *vec_id).collect::<Vec<_>>();
            let locators = ids
                .iter()
                .map(|vec_id| {
                    remote_pairs
                        .iter()
                        .find(|(candidate, _)| candidate == vec_id)
                        .map(|(_, tid)| *tid)
                        .unwrap_or(ItemPointer::INVALID)
                })
                .collect::<Vec<_>>();
            remote_work.push((ordinal, ids, locators));
        }
        let requests = remote_work
            .iter()
            .map(|(ordinal, ids, _locators)| {
                let route = &self.routes[*ordinal];
                let conninfo = route.conninfo.as_deref().ok_or_else(|| {
                    format!(
                        "EC_NODE_DESCRIPTOR: physical owner {ordinal} route has no connection descriptor"
                    )
                })?;
                Ok(super::remote_transport::DistannPhysicalMaterializeRequest {
                    conninfo,
                    index_regclass: &route.remote_index_regclass,
                    epoch_fingerprint: &self.fingerprint,
                    vec_ids: ids,
                    projection_attnums: &projection_attnums,
                    expected_schema_fingerprint: &schema_fingerprint,
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    use_cached_payload_plan:
                        super::options::benchmark_owner_payload_plan_cache(),
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    use_typed_locator: super::options::benchmark_typed_locator(),
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    use_packed_payload: super::options::benchmark_packed_payload(),
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    owner_heap_tids: _locators,
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    use_expanded_locator: candidate,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        {
            super::stage_counters::record(
                super::stage_counters::DistannQueryStage::MaterializePrepare,
                prepare_started.elapsed(),
            );
            super::stage_counters::record_work(
                super::stage_counters::DistannMaterializationWork::RemoteCandidatesRequested,
                remote_ids.len(),
            );
            super::stage_counters::record_work(
                super::stage_counters::DistannMaterializationWork::RemoteOwnersRequested,
                remote_work.len(),
            );
            super::stage_counters::record_work(
                super::stage_counters::DistannMaterializationWork::PayloadColumnsRequested,
                remote_ids.len().saturating_mul(projection_attnums.len()),
            );
        }
        let responses = super::remote_transport::remote_physical_materialize_batch(&requests);
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let map_started = Instant::now();
        for ((ordinal, ids, _), response) in remote_work.into_iter().zip(responses) {
            let response = response.map_err(|error| error.to_string())?;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            {
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::RemoteRowsReturned,
                    response.rows.len(),
                );
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::PayloadBytesReturned,
                    usize::try_from(response.telemetry.payload_bytes).unwrap_or(usize::MAX),
                );
            }
            if response.rows.len() != ids.len() {
                return Err(format!(
                    "EC_INTERNAL: physical owner {ordinal} returned {} payloads for {} rows",
                    response.rows.len(),
                    ids.len()
                ));
            }
            for (requested, payload) in ids.into_iter().zip(response.rows) {
                if payload.vec_id != requested {
                    return Err(format!(
                        "EC_INTERNAL: physical owner {ordinal} did not preserve payload order"
                    ));
                }
                if payload.tuple_payload_missing {
                    return Err(format!(
                        "EC_GENERATION_MISSING: physical owner {ordinal} has no row-tier payload for vec_id {requested}"
                    ));
                }
                if payload.is_tombstone {
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    super::stage_counters::record_work(
                        super::stage_counters::DistannMaterializationWork::RemoteTombstones,
                        1,
                    );
                    continue;
                }
                payloads.insert(
                    requested,
                    PhysicalRemotePayload {
                        payload_nulls: payload.payload_nulls,
                        payload_offsets: payload.payload_offsets,
                        payload_values: payload.payload_values,
                    },
                );
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::RemotePayloadsInstalled,
                    1,
                );
            }
        }
        #[cfg(feature = "distann-head-attribution-benchmark")]
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::MaterializeMapInsert,
            map_started.elapsed(),
        );
        Ok(payloads)
    }
}

/// Populate the TRAV-30 gateway copy set from the FR-080 head membership
/// (Task 210 P3). The gateway nodes are the head landmarks: bounded by head
/// capacity, and exactly the nodes every scan expands first. Each owner's
/// landmarks are fetched as routing payload only (neighbour ids + codes) via
/// `ec_distann_gateway_routing_export`; local landmarks are read in process.
/// Any failure degrades to `None` — the copy is an accelerator, never a
/// correctness dependency — but degrades loudly (warning), because a silently
/// absent cache is the inert-mechanism failure mode this program keeps hitting.
fn populate_gateway_copies(
    index_oid: pg_sys::Oid,
    fingerprint: &[u8; 34],
    descriptor: &DistannGenerationDescriptor,
    routes: &[PhysicalOwnerRoute],
    head_members: &[u64],
) -> Option<super::gateway_copy::DistannGatewayCopySet> {
    let capacity = super::options::gateway_copy_capacity();
    if capacity == 0 || routes.len() < 2 || head_members.is_empty() {
        return None;
    }
    // The bound is the GUC capacity, never the head size: a larger head cannot
    // grow the copy set (refusal, not eviction — same invariant as insert()).
    let members = head_members
        .iter()
        .copied()
        .take(capacity)
        .collect::<Vec<_>>();
    let buckets = super::placement::group_by_owning_node(
        &members,
        routes.len(),
        descriptor.placement_hash_version,
    );
    let mut set = super::gateway_copy::DistannGatewayCopySet::with_capacity(capacity);
    let mut remote_work: Vec<(usize, Vec<u64>)> = Vec::new();
    for (ordinal, bucket) in buckets.iter().enumerate() {
        if bucket.is_empty() {
            continue;
        }
        let owned = bucket.iter().map(|(_, vec_id)| *vec_id).collect::<Vec<_>>();
        let route = &routes[ordinal];
        if route.conninfo.is_some() {
            remote_work.push((ordinal, owned));
            continue;
        }
        if !route.is_local {
            pgrx::warning!(
                "ec_distann gateway copies disabled: owner {ordinal} has no connection descriptor"
            );
            return None;
        }
        let local = match RetainedGenerationScan::open(index_oid, fingerprint) {
            Ok(local) => local,
            Err(error) => {
                pgrx::warning!("ec_distann gateway copies disabled: local open failed: {error}");
                return None;
            }
        };
        let code_len = local.code_len;
        let nodes = match local.resolve_nodes(&owned) {
            Ok(nodes) => nodes,
            Err(error) => {
                pgrx::warning!("ec_distann gateway copies disabled: local resolve failed: {error}");
                return None;
            }
        };
        for node in nodes {
            let count = usize::from(node.neighbor_count);
            set.insert(super::gateway_copy::DistannGatewayCopy {
                vec_id: node.vec_id,
                is_tombstone: node.tombstoned,
                neighbor_vec_ids: node.neighbor_vec_ids[..count].to_vec(),
                neighbor_codes: node.neighbor_codes[..count * code_len].to_vec(),
            });
        }
    }
    let requests = remote_work
        .iter()
        .map(
            |(ordinal, owned)| super::remote_transport::DistannGatewayRoutingRequest {
                conninfo: routes[*ordinal]
                    .conninfo
                    .as_deref()
                    .expect("remote gateway work requires a connection descriptor"),
                index_regclass: &routes[*ordinal].remote_index_regclass,
                epoch_fingerprint: fingerprint,
                member_vec_ids: owned,
            },
        )
        .collect::<Vec<_>>();
    for response in super::remote_transport::remote_gateway_routing_batch(&requests) {
        match response {
            Ok(copies) => {
                for copy in copies {
                    set.insert(copy);
                }
            }
            Err(error) => {
                pgrx::warning!("ec_distann gateway copies disabled: remote export failed: {error}");
                return None;
            }
        }
    }
    super::gateway_copy::record_population(&set);
    Some(set)
}

/// Populate the FR-089 crown lazily from owner-held search codes. The
/// selection is deterministic and capacity-bounded before any RPC is issued;
/// a failed population simply leaves the scan on the ordinary head path.
fn populate_crown_cache(
    index_oid: pg_sys::Oid,
    fingerprint: &[u8; 34],
    descriptor: &DistannGenerationDescriptor,
    routes: &[PhysicalOwnerRoute],
    head_members: &[u64],
) -> Option<super::crown_cache::DistannCrownCache> {
    if super::options::debug_fail_crown_population() {
        pgrx::warning!("ec_distann crown population forced to fail by pg_test fault");
        return None;
    }
    let capacity = super::options::crown_capacity();
    if capacity == 0 || routes.is_empty() || head_members.is_empty() {
        return None;
    }
    let selected = super::crown_cache::DistannCrownCache::select_member_ids_for_roster(
        head_members,
        capacity,
        routes.len(),
        descriptor.placement_hash_version,
    );
    let buckets = super::placement::group_by_owning_node(
        &selected,
        routes.len(),
        descriptor.placement_hash_version,
    );
    let mut entries = Vec::with_capacity(selected.len());
    let mut remote_work = Vec::new();
    for (ordinal, bucket) in buckets.iter().enumerate() {
        if bucket.is_empty() {
            continue;
        }
        let ids = bucket.iter().map(|(_, vec_id)| *vec_id).collect::<Vec<_>>();
        let route = &routes[ordinal];
        if let Some(conninfo) = route.conninfo.as_deref() {
            remote_work.push((ordinal, ids, conninfo));
            continue;
        }
        if !route.is_local {
            pgrx::warning!("ec_distann crown population disabled: owner {ordinal} has no connection descriptor");
            return None;
        }
        let local = match RetainedGenerationScan::open(index_oid, fingerprint) {
            Ok(local) => local,
            Err(error) => {
                pgrx::warning!("ec_distann crown population disabled: local open failed: {error}");
                return None;
            }
        };
        let nodes = match local.resolve_nodes(&ids) {
            Ok(nodes) => nodes,
            Err(error) => {
                pgrx::warning!(
                    "ec_distann crown population disabled: local resolve failed: {error}"
                );
                return None;
            }
        };
        entries.extend(
            nodes
                .into_iter()
                .map(|node| super::crown_cache::DistannCrownEntry {
                    vec_id: node.vec_id,
                    search_code: node.search_code,
                }),
        );
    }
    let requests = remote_work
        .iter()
        .map(
            |(ordinal, ids, conninfo)| super::remote_transport::DistannCrownCodeRequest {
                conninfo,
                index_regclass: &routes[*ordinal].remote_index_regclass,
                epoch_fingerprint: fingerprint,
                member_vec_ids: ids,
            },
        )
        .collect::<Vec<_>>();
    for response in super::remote_transport::remote_crown_code_batch(&requests) {
        match response {
            Ok(mut remote_entries) => entries.append(&mut remote_entries),
            Err(error) => {
                pgrx::warning!(
                    "ec_distann crown population disabled: remote export failed: {error}"
                );
                return None;
            }
        }
    }
    match super::crown_cache::DistannCrownCache::from_entries(
        capacity,
        *fingerprint,
        &selected,
        entries,
    ) {
        Ok(cache) => Some(cache),
        Err(error) => {
            pgrx::warning!("ec_distann crown population disabled: {error}");
            None
        }
    }
}

struct PhysicalMultiOwnerExpander<'a> {
    local: Option<GenerationExpander<'a>>,
    local_ordinal: Option<usize>,
    descriptor: &'a DistannGenerationDescriptor,
    routes: &'a [PhysicalOwnerRoute],
    fingerprint: &'a [u8; 34],
    query: &'a [f32],
    query_digest: &'a [u8; 32],
    prepared: &'a DistannPreparedQuery,
    code_len: usize,
    gateway: Option<&'a super::gateway_copy::DistannGatewayCopySet>,
}

impl PhysicalMultiOwnerExpander<'_> {
    fn expand_nodes_raw(
        &mut self,
        vec_ids: &[u64],
        code_threshold: Option<f32>,
        candidate_limit: Option<usize>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let partition_started = Instant::now();
        let buckets = super::placement::group_by_owning_node(
            vec_ids,
            self.routes.len(),
            self.descriptor.placement_hash_version,
        );
        #[cfg(feature = "distann-head-attribution-benchmark")]
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::TraversalCoordinatorPartition,
            partition_started.elapsed(),
        );
        let mut ordered = (0..vec_ids.len()).map(|_| None).collect::<Vec<_>>();
        let mut remote_work = Vec::new();
        for (ordinal, bucket) in buckets.iter().enumerate() {
            if bucket.is_empty() {
                continue;
            }
            let owned = bucket.iter().map(|(_, vec_id)| *vec_id).collect::<Vec<_>>();
            if Some(ordinal) == self.local_ordinal {
                #[cfg(feature = "distann-head-attribution-benchmark")]
                let local_started = Instant::now();
                let response = self
                    .local
                    .as_mut()
                    .ok_or_else(|| {
                        DistannExpandError::Internal(
                            "local owner route has no generation reader".to_owned(),
                        )
                    })?
                    .expand_nodes(&owned, code_threshold, candidate_limit)?;
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::record(
                    super::stage_counters::DistannQueryStage::LocalExpand,
                    local_started.elapsed(),
                );
                place_physical_owner_responses(ordinal, bucket, response, &mut ordered)?;
            } else {
                // TRAV-30 (Task 210 P3): ids the coordinator holds a gateway
                // copy for still go to their owner — `exact_dist` needs the
                // owner's co-placed vector (the result half of Algorithm 1's
                // split; holding those here is the FR-084 trap) — but the
                // owner is told to omit their neighbour payload, which the
                // coordinator reconstructs locally below.
                let cached_mask = owned
                    .iter()
                    .map(|vec_id| {
                        self.gateway
                            .is_some_and(|gateway| gateway.get(*vec_id).is_some())
                    })
                    .collect::<Vec<_>>();
                let skip_ids = owned
                    .iter()
                    .zip(&cached_mask)
                    .filter_map(|(vec_id, cached)| cached.then_some(*vec_id))
                    .collect::<Vec<_>>();
                remote_work.push((ordinal, owned, cached_mask, skip_ids));
            }
        }
        let requests = remote_work
            .iter()
            .map(|(ordinal, owned, _, skip_ids)| {
                let route = &self.routes[*ordinal];
                let conninfo = route.conninfo.as_deref().ok_or_else(|| {
                    DistannExpandError::Internal(format!(
                        "EC_NODE_DESCRIPTOR: physical owner {ordinal} route has no connection descriptor"
                    ))
                })?;
                Ok(super::remote_transport::DistannPhysicalExpandRequest {
                    conninfo,
                    index_regclass: &route.remote_index_regclass,
                    epoch_fingerprint: self.fingerprint,
                    query: self.query,
                    query_digest: self.query_digest,
                    vec_ids: owned,
                    code_threshold,
                    candidate_limit: candidate_limit.map(|limit| {
                        i32::try_from(limit).unwrap_or(i32::MAX)
                    }),
                    skip_neighbor_vec_ids: skip_ids,
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    expanded_locator: super::options::benchmark_expanded_locator(),
                })
            })
            .collect::<Result<Vec<_>, DistannExpandError>>()?;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let remote_started = Instant::now();
        let responses = super::remote_transport::remote_physical_expand_batch(&requests);
        #[cfg(feature = "distann-head-attribution-benchmark")]
        if !requests.is_empty() {
            super::stage_counters::record(
                super::stage_counters::DistannQueryStage::RemoteExpand,
                remote_started.elapsed(),
            );
        }
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let decode_started = Instant::now();
        for ((ordinal, _, cached_mask, _), response) in remote_work.into_iter().zip(responses) {
            let mut response = response?;
            if cached_mask.iter().any(|cached| *cached) {
                let gateway = self.gateway.ok_or_else(|| {
                    DistannExpandError::Internal(
                        "gateway-cached expansion rows without a gateway copy set".to_owned(),
                    )
                })?;
                let filled = super::gateway_copy::fill_gateway_rows(
                    &mut response,
                    &cached_mask,
                    gateway,
                    code_threshold,
                    |copy| {
                        let count = copy.neighbor_vec_ids.len();
                        if copy.neighbor_codes.len() != count * self.code_len {
                            return Err(format!(
                                "gateway copy for vec_id {:#018x} has a malformed code payload",
                                copy.vec_id
                            ));
                        }
                        let mut dists = vec![0.0; count];
                        self.prepared.score_dists_batch(
                            &copy.neighbor_codes,
                            self.code_len,
                            count,
                            &mut dists,
                        )?;
                        Ok(dists)
                    },
                )?;
                // Re-apply the batch L limit over the whole owner batch now
                // that cached rows carry their candidates again; the owner
                // applied it to the uncached subset only, and top-L of
                // (top-L(subset) ∪ cached) equals top-L of the full batch, so
                // the Task 205 semantics are preserved exactly.
                super::scan::prune_and_limit_neighbor_batch(&mut response, None, candidate_limit)?;
                super::gateway_copy::record_served(filled);
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::record_work(
                    super::stage_counters::DistannMaterializationWork::GatewayCopiesServed,
                    filled,
                );
            }
            place_physical_owner_responses(ordinal, &buckets[ordinal], response, &mut ordered)?;
        }
        #[cfg(feature = "distann-head-attribution-benchmark")]
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::TraversalCoordinatorDecode,
            decode_started.elapsed(),
        );
        ordered
            .into_iter()
            .map(|node| {
                node.ok_or_else(|| {
                    DistannExpandError::Internal(
                        "physical expansion response has an unfilled request slot".to_owned(),
                    )
                })
            })
            .collect()
    }
}

impl DistannNodeExpander for PhysicalMultiOwnerExpander<'_> {
    fn expand_nodes(
        &mut self,
        vec_ids: &[u64],
        code_threshold: Option<f32>,
        candidate_limit: Option<usize>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
        let mut expanded = self.expand_nodes_raw(vec_ids, code_threshold, candidate_limit)?;
        if !super::options::benchmark_exact_neighbor() {
            return Ok(expanded);
        }

        // Benchmark-only fixed-seed traversal oracle: fetch the exact source
        // distance already returned for every referenced neighbor, then
        // substitute only the traversal scores. The seed set, BW/H budget,
        // graph, and adjacency remain identical to the RaBitQ arm. Work stays
        // bounded by this expansion's requested nodes times graph degree.
        let neighbor_ids = expanded
            .iter()
            .flat_map(|node| node.neighbor_vec_ids.iter().copied())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let exact_neighbors = self.expand_nodes_raw(&neighbor_ids, None, None)?;
        let exact_distances = exact_neighbors
            .into_iter()
            .map(|node| {
                node.exact_dist
                    .map(|distance| (node.vec_id, distance))
                    .ok_or_else(|| {
                        DistannExpandError::Internal(format!(
                            "exact-neighbor oracle found tombstoned vec_id {:#018x}",
                            node.vec_id
                        ))
                    })
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        for node in &mut expanded {
            node.neighbor_code_dists = node
                .neighbor_vec_ids
                .iter()
                .map(|vec_id| {
                    exact_distances.get(vec_id).copied().ok_or_else(|| {
                        DistannExpandError::Internal(format!(
                            "exact-neighbor oracle did not resolve vec_id {vec_id:#018x}"
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
        }
        Ok(expanded)
    }
}

fn place_physical_owner_responses(
    ordinal: usize,
    bucket: &[(usize, u64)],
    response: Vec<DistannExpandedNode>,
    ordered: &mut [Option<DistannExpandedNode>],
) -> Result<(), DistannExpandError> {
    if response.len() != bucket.len() {
        return Err(DistannExpandError::Internal(format!(
            "physical owner {ordinal} returned {} rows for {} requests",
            response.len(),
            bucket.len()
        )));
    }
    for ((request_index, requested_id), node) in bucket.iter().zip(response) {
        if node.vec_id != *requested_id {
            return Err(DistannExpandError::Internal(format!(
                "physical owner {ordinal} did not preserve request order"
            )));
        }
        ordered[*request_index] = Some(node);
    }
    Ok(())
}

struct GenerationExpander<'a> {
    index_oid: pg_sys::Oid,
    generation: &'a GenerationCatalogRow,
    descriptor: &'a DistannGenerationDescriptor,
    graph_relation: &'a HeapRelationGuard,
    directory_relation: &'a IndexRelationGuard,
    row_relation: &'a HeapRelationGuard,
    slot: &'a TupleTableSlotGuard<'a>,
    snapshot: pg_sys::Snapshot,
    source_attnum: i32,
    query: &'a [f32],
    prepared: &'a DistannPreparedQuery,
    code_len: usize,
}

impl GenerationExpander<'_> {
    /// Read this owner's locally held full-precision vector for `node`.
    ///
    /// Task 210 P2: an FR-080 head landmark is co-placed with its row-tier
    /// vector under the same FR-078 hash (ADR-085 D11), so an owner can
    /// materialise its own head shard without any vector crossing the wire.
    fn local_source_vector(&self, node: &DistannNodeTuple) -> Result<Vec<f32>, DistannExpandError> {
        let mut tid = pg_sys::ItemPointerData::default();
        pgrx::itemptr::item_pointer_set_all(
            &mut tid,
            node.heap_tid.block_number,
            node.heap_tid.offset_number,
        );
        unsafe { pg_sys::ExecClearTuple(self.slot.as_ptr()) };
        let found = unsafe {
            pg_sys::table_tuple_fetch_row_version(
                self.row_relation.as_ptr(),
                &mut tid,
                self.snapshot,
                self.slot.as_ptr(),
            )
        };
        if !found {
            return Err(DistannExpandError::Internal(format!(
                "EC_GENERATION_MISSING: row tier missing vec_id {:#018x}",
                node.vec_id
            )));
        }
        let mut is_null = false;
        let datum =
            unsafe { pg_sys::slot_getattr(self.slot.as_ptr(), self.source_attnum, &mut is_null) };
        if is_null {
            return Err(DistannExpandError::Internal(
                "EC_SCHEMA_MISMATCH: frozen source vector is NULL".to_owned(),
            ));
        }
        let vector = unsafe { crate::am::ec_diskann::ecvector_datum_to_vec(datum) };
        if vector.len() != usize::from(self.descriptor.dimensions) {
            return Err(DistannExpandError::Internal(
                "EC_SCHEMA_MISMATCH: frozen source vector dimension mismatch".to_owned(),
            ));
        }
        Ok(vector)
    }

    fn exact_distance(&self, node: &DistannNodeTuple) -> Result<f32, DistannExpandError> {
        let mut tid = pg_sys::ItemPointerData::default();
        pgrx::itemptr::item_pointer_set_all(
            &mut tid,
            node.heap_tid.block_number,
            node.heap_tid.offset_number,
        );
        unsafe { pg_sys::ExecClearTuple(self.slot.as_ptr()) };
        let found = unsafe {
            pg_sys::table_tuple_fetch_row_version(
                self.row_relation.as_ptr(),
                &mut tid,
                self.snapshot,
                self.slot.as_ptr(),
            )
        };
        if !found {
            return Err(DistannExpandError::Internal(format!(
                "EC_GENERATION_MISSING: row tier missing vec_id {:#018x}",
                node.vec_id
            )));
        }
        let mut is_null = false;
        let datum =
            unsafe { pg_sys::slot_getattr(self.slot.as_ptr(), self.source_attnum, &mut is_null) };
        if is_null {
            return Err(DistannExpandError::Internal(
                "EC_SCHEMA_MISMATCH: frozen source vector is NULL".to_owned(),
            ));
        }
        let vector = unsafe { crate::am::ec_diskann::ecvector_datum_to_vec(datum) };
        if vector.len() != usize::from(self.descriptor.dimensions) {
            return Err(DistannExpandError::Internal(
                "EC_SCHEMA_MISMATCH: frozen source vector dimension mismatch".to_owned(),
            ));
        }
        Ok(-crate::am::ec_diskann::source_inner_product_deterministic(
            self.query, &vector,
        ))
    }
}

impl GenerationExpander<'_> {
    /// Expand with a gateway-copy skip mask (TRAV-30, Task 210 P3). Ids in
    /// `skip_neighbors` still get their record read and exact distance — the
    /// result half stays owner-authoritative — but their neighbour payload is
    /// omitted (empty arrays) because the coordinator reconstructs it from its
    /// bounded gateway copy, so those bytes never cross the wire and this
    /// owner skips their scoring work. The batch L limit then covers only the
    /// rows that carry neighbours; the coordinator re-applies it over the full
    /// batch after filling the cached rows, which preserves the Task 205
    /// batch-threshold semantics exactly.
    fn expand_nodes_masked(
        &mut self,
        vec_ids: &[u64],
        code_threshold: Option<f32>,
        candidate_limit: Option<usize>,
        skip_neighbors: &std::collections::HashSet<u64>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let graph_started = Instant::now();
        let missing = |vec_id| {
            DistannExpandError::OwnedRecordMissing(format!(
                "physical generation {} lacks vec_id {vec_id:#018x}",
                self.generation.epoch
            ))
        };
        let records = lookup_graph_nodes_with_intent_retry(
            self.index_oid,
            self.generation,
            self.graph_relation,
            self.directory_relation,
            self.snapshot,
            vec_ids,
            &vec_ids
                .iter()
                .map(|vec_id| {
                    let ordinal = super::placement::owning_node(
                        *vec_id,
                        self.descriptor.roster.len(),
                        self.descriptor.placement_hash_version,
                    );
                    self.descriptor.roster[ordinal].node_id
                })
                .collect::<Vec<_>>(),
            self.descriptor.graph_degree,
            self.code_len,
            missing,
        )?;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let owner_graph_read_ns =
            i64::try_from(duration_ns(graph_started.elapsed())).unwrap_or(i64::MAX);
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let score_started = Instant::now();
        let mut responses = vec_ids
            .iter()
            .map(|vec_id| {
                let node = records.get(vec_id).ok_or_else(|| {
                    DistannExpandError::OwnedRecordMissing(format!(
                        "physical generation {} lacks vec_id {vec_id:#018x}",
                        self.generation.epoch
                    ))
                })?;
                let (neighbor_vec_ids, neighbor_code_dists) = if skip_neighbors.contains(vec_id) {
                    (Vec::new(), Vec::new())
                } else {
                    let count = usize::from(node.neighbor_count);
                    let mut neighbor_dists = vec![0.0; count];
                    self.prepared
                        .score_dists_batch(
                            &node.neighbor_codes[..count * self.code_len],
                            self.code_len,
                            count,
                            &mut neighbor_dists,
                        )
                        .map_err(DistannExpandError::Internal)?;
                    super::scan::prune_and_limit_neighbors(
                        &node.neighbor_vec_ids[..count],
                        &neighbor_dists,
                        None,
                        None,
                    )?
                };
                Ok(DistannExpandedNode {
                    vec_id: node.vec_id,
                    exact_dist: (!node.tombstoned)
                        .then(|| self.exact_distance(node))
                        .transpose()?,
                    is_tombstone: node.tombstoned,
                    heap_tid: node.heap_tid,
                    owner_heap_tid: ItemPointer::INVALID,
                    neighbor_vec_ids,
                    neighbor_code_dists,
                    neighbors_pruned: 0,
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
            .collect::<Result<Vec<_>, DistannExpandError>>()?;
        super::scan::prune_and_limit_neighbor_batch(
            &mut responses,
            code_threshold,
            candidate_limit,
        )?;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let responses = {
            let mut responses = responses;
            let owner_score_ns =
                i64::try_from(duration_ns(score_started.elapsed())).unwrap_or(i64::MAX);
            for response in &mut responses {
                response.owner_graph_read_ns = owner_graph_read_ns;
                response.owner_score_ns = owner_score_ns;
            }
            responses
        };
        Ok(responses)
    }
}

impl DistannNodeExpander for GenerationExpander<'_> {
    fn expand_nodes(
        &mut self,
        vec_ids: &[u64],
        code_threshold: Option<f32>,
        candidate_limit: Option<usize>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
        self.expand_nodes_masked(
            vec_ids,
            code_threshold,
            candidate_limit,
            &std::collections::HashSet::new(),
        )
    }
}
