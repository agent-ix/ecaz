//! Published physical-generation reader for Task 179.
//!
//! The logical control index is metadata-only. This module resolves its active
//! pointer to the immutable generation heap relations, pins the exact epoch in
//! the shared scan registry, and adapts those relations to the existing FR-081
//! orchestration seam.

use std::cell::RefCell;
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

#[derive(Debug)]
pub(crate) struct PhysicalOwnerRoute {
    pub(crate) roster_ordinal: usize,
    pub(crate) is_local: bool,
    pub(crate) remote_index_regclass: String,
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
                    "SELECT roster_ordinal, is_local, remote_index_regclass,
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
                let remote_index_regclass = row["remote_index_regclass"]
                    .value::<String>()
                    .map_err(|error| format!("EC_NODE_DESCRIPTOR: locator decode failed: {error}"))?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: locator is NULL".to_owned())?;
                let secret = row["conninfo_secret_name"]
                    .value::<String>()
                    .map_err(|error| format!("EC_NODE_DESCRIPTOR: secret decode failed: {error}"))?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: secret is NULL".to_owned())?;
                Ok(PhysicalOwnerRoute {
                    roster_ordinal: usize::try_from(ordinal).map_err(|_| {
                        "EC_NODE_DESCRIPTOR: binding ordinal is negative".to_owned()
                    })?,
                    is_local,
                    remote_index_regclass,
                    conninfo: if is_local {
                        None
                    } else {
                        Some(super::node_registry::resolve_conninfo_secret(&secret)?)
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
        if query.len() != usize::from(self.descriptor.dimensions) {
            return Err(DistannExpandError::BadInput(format!(
                "query has {} dimensions, retained generation requires {}",
                query.len(),
                self.descriptor.dimensions
            )));
        }
        self.validate_ownership(vec_ids)
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
                        "SELECT graph_record FROM {} ORDER BY vec_id",
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
        expander.expand_nodes(vec_ids, code_threshold, candidate_limit)
    }

    fn resolve_nodes(&self, vec_ids: &[u64]) -> Result<Vec<DistannNodeTuple>, DistannExpandError> {
        self.validate_ownership(vec_ids)?;
        if vec_ids.is_empty() {
            return Ok(Vec::new());
        }
        let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
        let records = lookup_graph_nodes(
            &self.graph_relation,
            &self.directory_relation,
            snapshot,
            vec_ids,
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
    ) -> Result<PhysicalPayloadBatch, DistannExpandError> {
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
        let nodes = self.resolve_nodes(vec_ids)?;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let node_lookup_ns = duration_ns(lookup_started.elapsed());
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
        let sql = super::remote_endpoint::build_payload_sql(&row_name, &columns, &sends)
            .map_err(DistannExpandError::BadInput)?;
        let ctid_texts = nodes
            .iter()
            .map(|node| {
                format!(
                    "({},{})",
                    node.heap_tid.block_number, node.heap_tid.offset_number
                )
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
                    let rows =
                        client.select(&entry.statement, None, &[ctid_refs.as_slice().into()]);
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
                    let rows = client.select(&statement, None, &[ctid_refs.as_slice().into()]);
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
                client.select(&sql, None, &[ctid_refs.as_slice().into()])
            };
            #[cfg(not(feature = "distann-head-attribution-benchmark"))]
            let rows = {
                let _ = use_cached_payload_plan;
                client.select(&sql, None, &[ctid_refs.as_slice().into()])
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
                let values = row["payload_values"]
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
                if nulls.len() != column_count || values.len() != column_count {
                    return Err(DistannExpandError::Internal(
                        "physical payload column count mismatch".to_owned(),
                    ));
                }
                Ok((missing, nulls, values))
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
            .map(|(node, (missing, nulls, values))| {
                (
                    i64::from_le_bytes(node.vec_id.to_le_bytes()),
                    node.tombstoned,
                    missing,
                    nulls,
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
        .materialize_payloads(&[], &[], &expected, false)
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
type PhysicalPayloadRow = (i64, bool, bool, Vec<bool>, Vec<Vec<u8>>);

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
    RetainedGenerationScan::open(index_oid, epoch_fingerprint)?
        .expand(query, query_digest, &ids, code_threshold, candidate_limit)
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
) -> TableIterator<
    'static,
    (
        name!(vec_id, i64),
        name!(exact_dist, Option<f32>),
        name!(is_tombstone, bool),
        name!(neighbor_vec_ids, Vec<i64>),
        name!(neighbor_code_dists, Vec<f32>),
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
        name!(payload_values, Vec<Vec<u8>>),
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
            )
        })
        .map(|batch| batch.rows)
        .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter())
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
) -> TableIterator<
    'static,
    (
        name!(vec_id, i64),
        name!(is_tombstone, bool),
        name!(tuple_payload_missing, bool),
        name!(payload_nulls, Vec<bool>),
        name!(payload_values, Vec<Vec<u8>>),
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
    let batch = store
        .materialize_payloads(
            &ids,
            &projection_attnums,
            &expected_schema_fingerprint,
            use_cached_payload_plan,
        )
        .unwrap_or_else(|error| error.raise());
    let owner_total_ns = duration_ns(total_started.elapsed());
    let owner_open_validate_ns = open_ns.saturating_add(batch.telemetry.validate_ns);
    let owner_node_lookup_ns = batch.telemetry.node_lookup_ns;
    let owner_payload_sql_ns = batch.telemetry.payload_sql_ns;
    let payload_bytes = batch
        .rows
        .iter()
        .map(|(_, _, _, nulls, values)| {
            nulls
                .len()
                .saturating_add(values.iter().map(Vec::len).sum::<usize>())
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
    _scan_token: ScanTokenGuard,
}

pub(crate) struct PhysicalRemotePayload {
    pub(crate) payload_nulls: Vec<bool>,
    pub(crate) payload_values: Vec<Vec<u8>>,
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
        match Self::open_once(index_oid) {
            Err(error) if error.starts_with("EC_EPOCH_MISMATCH:") => Self::open_once(index_oid),
            result => result,
        }
    }

    fn open_once(index_oid: pg_sys::Oid) -> Result<Self, String> {
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
        let first = active_generation_identity(index_oid, logical_index_uuid)?
            .ok_or_else(|| "EC_GENERATION_MISSING: logical index has no active epoch".to_owned())?;
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
        let active =
            active_generation_identity(index_oid, logical_index_uuid)?.ok_or_else(|| {
                "EC_GENERATION_MISSING: active epoch disappeared during registration".to_owned()
            })?;
        if active.build_id != first.build_id || active.fingerprint != first.fingerprint {
            return Err(
                "EC_EPOCH_MISMATCH: active epoch changed during scan registration".to_owned(),
            );
        }

        let (descriptor, descriptor_digest, head_index) = if let Some(cached) =
            cached_physical_epoch(index_oid, logical_index_uuid, &active)
        {
            (
                cached.descriptor,
                cached.descriptor_digest,
                cached.head_index,
            )
        } else {
            let candidate = super::build_coordinator::load_build_candidate(
                index_oid,
                logical_index_uuid,
                active.build_id,
            )?
            .ok_or_else(|| "EC_GENERATION_MISSING: active build candidate is absent".to_owned())?;
            let descriptor = Arc::new(DistannGenerationDescriptor::decode(
                &candidate.generation_descriptor,
            )?);
            let descriptor_digest = descriptor.digest()?;
            if descriptor_digest != candidate.generation_descriptor_digest
                || descriptor.coordinator_logical_index_uuid != *logical_index_uuid.as_bytes()
            {
                return Err(
                    "EC_GENERATION_DESCRIPTOR: active generation descriptor identity mismatch"
                        .to_owned(),
                );
            }
            let head_index = {
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
            };
            cache_physical_epoch(CachedPhysicalEpoch {
                index_oid,
                logical_index_uuid,
                build_id: active.build_id,
                fingerprint: active.fingerprint,
                descriptor: Arc::clone(&descriptor),
                descriptor_digest,
                head_index: head_index.clone(),
            });
            (descriptor, descriptor_digest, head_index)
        };
        let routes = physical_owner_routes(
            index_oid,
            logical_index_uuid,
            active.build_id,
            descriptor.roster.len(),
        )?;
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
            _scan_token: scan_token,
        })
    }

    pub(crate) fn row_relation(&self) -> Option<pg_sys::Relation> {
        self.row_relation.as_ref().map(HeapRelationGuard::as_ptr)
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

        let all_seeds = self.select_seed_candidates(query)?;
        if all_seeds.is_empty() {
            return Ok(DistannHitCollection {
                hits: Vec::new(),
                counters: Default::default(),
                multi_node: self.routes.len() > 1,
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
        let replica = super::traversal_replica::ReadyTraversalReplica::open(
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
        );
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
                        return Ok(DistannHitCollection {
                            hits,
                            counters,
                            multi_node: self.routes.len() > 1,
                        });
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
        Ok(DistannHitCollection {
            hits,
            counters,
            multi_node: self.routes.len() > 1,
        })
    }

    fn select_seed_candidates(&self, query: &[f32]) -> Result<Vec<DistannSeedCandidate>, String> {
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
        let remote_ids = hits
            .iter()
            .filter(|hit| hit.heap_tid == ItemPointer::INVALID)
            .map(|hit| hit.vec_id)
            .collect::<Vec<_>>();
        self.materialize_remote_payload_ids(&remote_ids, projection_attnums)
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
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let prepare_started = Instant::now();
        let schema_fingerprint = self.descriptor.row_schema.fingerprint()?;
        let projection_attnums = projection_attnums
            .iter()
            .map(|attnum| i16::try_from(i32::from(*attnum)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "EC_SCHEMA_MISMATCH: projection attnum exceeds smallint".to_owned())?;
        let buckets = super::placement::group_by_owning_node(
            remote_ids,
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
            remote_work.push((ordinal, ids));
        }
        let requests = remote_work
            .iter()
            .map(|(ordinal, ids)| {
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
        for ((ordinal, ids), response) in remote_work.into_iter().zip(responses) {
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

struct PhysicalMultiOwnerExpander<'a> {
    local: Option<GenerationExpander<'a>>,
    local_ordinal: Option<usize>,
    descriptor: &'a DistannGenerationDescriptor,
    routes: &'a [PhysicalOwnerRoute],
    fingerprint: &'a [u8; 34],
    query: &'a [f32],
    query_digest: &'a [u8; 32],
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
                remote_work.push((ordinal, owned));
            }
        }
        let requests = remote_work
            .iter()
            .map(|(ordinal, owned)| {
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
        for ((ordinal, _), response) in remote_work.into_iter().zip(responses) {
            place_physical_owner_responses(ordinal, &buckets[ordinal], response?, &mut ordered)?;
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

impl DistannNodeExpander for GenerationExpander<'_> {
    fn expand_nodes(
        &mut self,
        vec_ids: &[u64],
        code_threshold: Option<f32>,
        candidate_limit: Option<usize>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
        #[cfg(feature = "distann-head-attribution-benchmark")]
        let graph_started = Instant::now();
        let records = lookup_graph_nodes(
            self.graph_relation,
            self.directory_relation,
            self.snapshot,
            vec_ids,
            self.descriptor.graph_degree,
            self.code_len,
            |vec_id| {
                DistannExpandError::Internal(format!(
                    "EC_RECORD_MISSING: physical generation {} lacks vec_id {vec_id:#018x}",
                    self.generation.epoch
                ))
            },
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
                    DistannExpandError::Internal(format!(
                        "EC_RECORD_MISSING: physical generation {} lacks vec_id {vec_id:#018x}",
                        self.generation.epoch
                    ))
                })?;
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
                let (neighbor_vec_ids, neighbor_code_dists) =
                    super::scan::prune_and_limit_neighbors(
                        &node.neighbor_vec_ids[..count],
                        &neighbor_dists,
                        None,
                        None,
                    )?;
                Ok(DistannExpandedNode {
                    vec_id: node.vec_id,
                    exact_dist: (!node.tombstoned)
                        .then(|| self.exact_distance(node))
                        .transpose()?,
                    is_tombstone: node.tombstoned,
                    heap_tid: node.heap_tid,
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
