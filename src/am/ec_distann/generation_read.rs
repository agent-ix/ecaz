//! Published physical-generation reader for Task 179.
//!
//! The logical control index is metadata-only. This module resolves its active
//! pointer to the immutable generation heap relations, pins the exact epoch in
//! the shared scan registry, and adapts those relations to the existing FR-081
//! orchestration seam.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use pgrx::datum::Uuid;
use pgrx::iter::TableIterator;
use pgrx::{default, name, pg_extern, pg_sys, PgRelation, Spi};

use crate::storage::page::ItemPointer;
use crate::storage::relation_guard::{HeapRelationGuard, IndexRelationGuard};
use crate::storage::scan_guard::IndexScanGuard;
use crate::storage::slot_guard::TupleTableSlotGuard;

use super::expand_error::DistannExpandError;
use super::generation_catalog::{self, GenerationCatalogRow};
use super::generation_descriptor::DistannGenerationDescriptor;
use super::quantizer::{DistannCodecBinding, DistannPreparedQuery};
use super::routine::DistannHitCollection;
#[cfg(feature = "distann-legacy-seed-benchmark")]
use super::scan::DistannSeedCandidate;
use super::scan::{
    distann_orchestrated_search, DistannExpandedNode, DistannNodeExpander,
    DistannOrchestrationParams, DistannScanHit,
};
use super::scan_registry::ScanTokenGuard;
use super::tuple::DistannNodeTuple;

struct ActiveGenerationIdentity {
    build_id: Uuid,
    fingerprint: [u8; 34],
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
        cache.retain(|candidate| {
            candidate.index_oid != entry.index_oid || candidate.fingerprint != entry.fingerprint
        });
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
struct PhysicalOwnerRoute {
    roster_ordinal: usize,
    is_local: bool,
    remote_index_regclass: String,
    conninfo: Option<String>,
}

fn physical_owner_routes(
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
    #[cfg(feature = "distann-legacy-seed-benchmark")]
    graph_relation_name: String,
    source_attnum: i32,
    code_len: usize,
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
            let cached = CachedRetainedEpoch {
                index_oid,
                fingerprint,
                descriptor,
                generation,
                source_attnum,
                code_len,
            };
            cache_retained_epoch(cached.clone());
            cached
        };
        let CachedRetainedEpoch {
            descriptor,
            generation,
            source_attnum,
            code_len,
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
        #[cfg(feature = "distann-legacy-seed-benchmark")]
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
            #[cfg(feature = "distann-legacy-seed-benchmark")]
            graph_relation_name,
            source_attnum,
            code_len,
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

    #[cfg(feature = "distann-legacy-seed-benchmark")]
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
        let mut candidates = Spi::connect(|client| {
            client
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
                })?
                .map(|row| {
                    let bytes = row["graph_record"]
                        .value::<Vec<u8>>()
                        .map_err(|error| {
                            DistannExpandError::GenerationMissing(format!(
                                "physical seed graph record decode failed: {error}"
                            ))
                        })?
                        .ok_or_else(|| {
                            DistannExpandError::GenerationMissing(
                                "physical seed graph record is NULL".to_owned(),
                            )
                        })?;
                    let node = DistannNodeTuple::decode_physical_v1(
                        &bytes,
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
                    Ok(DistannSeedCandidate {
                        vec_id: node.vec_id,
                        dist: prepared.score_dist(&node.search_code),
                    })
                })
                .collect::<Result<Vec<_>, DistannExpandError>>()
        })?;
        candidates.sort_unstable_by(|left, right| left.dist.total_cmp(&right.dist));
        candidates.truncate(limit);
        Ok(candidates)
    }

    fn expand(
        &self,
        query: &[f32],
        query_digest: [u8; 32],
        vec_ids: &[u64],
        code_threshold: Option<f32>,
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
        expander.expand_nodes(vec_ids, code_threshold)
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
    ) -> Result<Vec<PhysicalPayloadRow>, DistannExpandError> {
        let expected: [u8; 32] = expected_schema_fingerprint.try_into().map_err(|_| {
            DistannExpandError::BadInput(format!(
                "expected schema fingerprint must be 32 bytes, got {}",
                expected_schema_fingerprint.len()
            ))
        })?;
        let resolved_schema =
            super::row_schema::resolve_relation_schema(self.generation.row_tier_relid)
                .map_err(DistannExpandError::GenerationMissing)?;
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
        let nodes = self.resolve_nodes(vec_ids)?;
        if nodes.is_empty() {
            return Ok(Vec::new());
        }
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
        let payloads = Spi::connect(|client| {
            client
                .select(&sql, None, &[ctid_refs.as_slice().into()])
                .map_err(|error| {
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
        Ok(nodes
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
            .collect())
    }
}

type PhysicalExpandRow = (i64, Option<f32>, bool, Vec<i64>, Vec<f32>);
type PhysicalPayloadRow = (i64, bool, bool, Vec<bool>, Vec<Vec<u8>>);

fn expand_physical_nodes_impl(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: &[u8],
    query: &[f32],
    query_digest: [u8; 32],
    vec_ids: &[i64],
    code_threshold: Option<f32>,
) -> Result<Vec<PhysicalExpandRow>, DistannExpandError> {
    let ids = vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    RetainedGenerationScan::open(index_oid, epoch_fingerprint)?
        .expand(query, query_digest, &ids, code_threshold)
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
    )
    .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter())
}

/// Task 179 benchmark-only control endpoint. This is absent from normal
/// production builds; the opt-in feature restores the removed owner-wide O(N)
/// seed scan so persisted-head seeding can be measured on otherwise-current
/// code.
#[cfg(feature = "distann-legacy-seed-benchmark")]
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

/// Records the compiled physical seed strategy in benchmark provenance.
#[pg_extern(immutable, parallel_safe)]
fn ec_distann_physical_seed_strategy() -> &'static str {
    if cfg!(feature = "distann-legacy-seed-benchmark") {
        "owner_scan"
    } else {
        "persisted_head"
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
            store.materialize_payloads(&ids, &projection_attnums, &expected_schema_fingerprint)
        })
        .unwrap_or_else(|error| error.raise());
    TableIterator::new(rows.into_iter())
}

pub(crate) struct PhysicalGenerationScan {
    #[cfg(feature = "distann-legacy-seed-benchmark")]
    index_oid: pg_sys::Oid,
    descriptor: Arc<DistannGenerationDescriptor>,
    generation: Option<GenerationCatalogRow>,
    row_relation: Option<HeapRelationGuard>,
    graph_relation: Option<HeapRelationGuard>,
    directory_relation: Option<IndexRelationGuard>,
    fingerprint: [u8; 34],
    routes: Vec<PhysicalOwnerRoute>,
    #[cfg(not(feature = "distann-legacy-seed-benchmark"))]
    head_index: Option<Arc<super::head_sample::DistannPhysicalHeadIndex>>,
    _scan_token: ScanTokenGuard,
}

pub(crate) struct PhysicalRemotePayload {
    pub(crate) payload_nulls: Vec<bool>,
    pub(crate) payload_values: Vec<Vec<u8>>,
}

fn active_generation_identity(
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
            #[cfg(feature = "distann-legacy-seed-benchmark")]
            let head_index = None;
            #[cfg(not(feature = "distann-legacy-seed-benchmark"))]
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
        #[cfg(feature = "distann-legacy-seed-benchmark")]
        let _ = &head_index;
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
            #[cfg(feature = "distann-legacy-seed-benchmark")]
            index_oid,
            descriptor,
            generation,
            row_relation,
            graph_relation,
            directory_relation,
            fingerprint: active.fingerprint,
            routes,
            #[cfg(not(feature = "distann-legacy-seed-benchmark"))]
            head_index,
            _scan_token: scan_token,
        })
    }

    pub(crate) fn row_relation(&self) -> Option<pg_sys::Relation> {
        self.row_relation.as_ref().map(HeapRelationGuard::as_ptr)
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

        let seed_limit = (super::options::current_beam_width() * 2).max(32);
        #[cfg(feature = "distann-legacy-seed-benchmark")]
        let all_seeds = self.legacy_seed_candidates(query, seed_limit)?;
        #[cfg(not(feature = "distann-legacy-seed-benchmark"))]
        let all_seeds = self
            .head_index
            .as_ref()
            .map(|head| head.search(query, seed_limit))
            .unwrap_or_default();
        if all_seeds.is_empty() {
            return Ok(DistannHitCollection {
                hits: Vec::new(),
                counters: Default::default(),
                multi_node: self.routes.len() > 1,
            });
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
        let params = DistannOrchestrationParams {
            beam_width: super::options::current_beam_width(),
            hop_rounds: super::options::current_hop_rounds(),
            top_k: effective_top_k,
            debug_fail_hop_round: super::options::debug_fail_hop_round(),
        };
        let (hits, counters) = distann_orchestrated_search(&all_seeds, &mut expander, params)
            .map_err(|error| error.to_string())?;
        Ok(DistannHitCollection {
            hits,
            counters,
            multi_node: self.routes.len() > 1,
        })
    }

    #[cfg(feature = "distann-legacy-seed-benchmark")]
    fn legacy_seed_candidates(
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
        seeds.sort_unstable_by(|left, right| left.dist.total_cmp(&right.dist));
        seeds.truncate(seed_limit);
        Ok(seeds)
    }

    pub(crate) fn materialize_remote_payloads(
        &self,
        hits: &[DistannScanHit],
        projection_attnums: &[pg_sys::AttrNumber],
    ) -> Result<HashMap<u64, PhysicalRemotePayload>, String> {
        let schema_fingerprint = self.descriptor.row_schema.fingerprint()?;
        let projection_attnums = projection_attnums
            .iter()
            .map(|attnum| i16::try_from(i32::from(*attnum)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "EC_SCHEMA_MISMATCH: projection attnum exceeds smallint".to_owned())?;
        let remote_ids = hits
            .iter()
            .filter(|hit| hit.heap_tid == ItemPointer::INVALID)
            .map(|hit| hit.vec_id)
            .collect::<Vec<_>>();
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
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let responses = super::remote_transport::remote_physical_materialize_batch(&requests);
        for ((ordinal, ids), response) in remote_work.into_iter().zip(responses) {
            let response = response.map_err(|error| error.to_string())?;
            if response.len() != ids.len() {
                return Err(format!(
                    "EC_INTERNAL: physical owner {ordinal} returned {} payloads for {} rows",
                    response.len(),
                    ids.len()
                ));
            }
            for (requested, payload) in ids.into_iter().zip(response) {
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
                    continue;
                }
                payloads.insert(
                    requested,
                    PhysicalRemotePayload {
                        payload_nulls: payload.payload_nulls,
                        payload_values: payload.payload_values,
                    },
                );
            }
        }
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

impl DistannNodeExpander for PhysicalMultiOwnerExpander<'_> {
    fn expand_nodes(
        &mut self,
        vec_ids: &[u64],
        code_threshold: Option<f32>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
        let buckets = super::placement::group_by_owning_node(
            vec_ids,
            self.routes.len(),
            self.descriptor.placement_hash_version,
        );
        let mut ordered = (0..vec_ids.len()).map(|_| None).collect::<Vec<_>>();
        let mut remote_work = Vec::new();
        for (ordinal, bucket) in buckets.iter().enumerate() {
            if bucket.is_empty() {
                continue;
            }
            let owned = bucket.iter().map(|(_, vec_id)| *vec_id).collect::<Vec<_>>();
            if Some(ordinal) == self.local_ordinal {
                let response = self
                    .local
                    .as_mut()
                    .ok_or_else(|| {
                        DistannExpandError::Internal(
                            "local owner route has no generation reader".to_owned(),
                        )
                    })?
                    .expand_nodes(&owned, code_threshold)?;
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
                })
            })
            .collect::<Result<Vec<_>, DistannExpandError>>()?;
        let responses = super::remote_transport::remote_physical_expand_batch(&requests);
        for ((ordinal, _), response) in remote_work.into_iter().zip(responses) {
            place_physical_owner_responses(ordinal, &buckets[ordinal], response?, &mut ordered)?;
        }
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
        Ok(-crate::am::ec_diskann::source_inner_product(
            self.query, &vector,
        ))
    }
}

impl DistannNodeExpander for GenerationExpander<'_> {
    fn expand_nodes(
        &mut self,
        vec_ids: &[u64],
        _code_threshold: Option<f32>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
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

        vec_ids
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
                Ok(DistannExpandedNode {
                    vec_id: node.vec_id,
                    exact_dist: (!node.tombstoned)
                        .then(|| self.exact_distance(node))
                        .transpose()?,
                    is_tombstone: node.tombstoned,
                    heap_tid: node.heap_tid,
                    neighbor_vec_ids: node.neighbor_vec_ids[..count].to_vec(),
                    neighbor_code_dists: neighbor_dists,
                })
            })
            .collect()
    }
}
