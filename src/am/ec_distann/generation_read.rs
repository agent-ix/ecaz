//! Published physical-generation reader for Task 179.
//!
//! The logical control index is metadata-only. This module resolves its active
//! pointer to the immutable generation heap relations, pins the exact epoch in
//! the shared scan registry, and adapts those relations to the existing FR-081
//! orchestration seam.

use std::collections::HashMap;

use pgrx::datum::Uuid;
use pgrx::{pg_sys, Spi};

use crate::storage::page::ItemPointer;
use crate::storage::relation_guard::HeapRelationGuard;
use crate::storage::slot_guard::TupleTableSlotGuard;

use super::expand_error::DistannExpandError;
use super::generation_catalog::{self, GenerationCatalogRow};
use super::generation_descriptor::DistannGenerationDescriptor;
use super::quantizer::{DistannCodecBinding, DistannPreparedQuery};
use super::routine::DistannHitCollection;
use super::scan::{
    distann_orchestrated_search, DistannExpandedNode, DistannNodeExpander,
    DistannOrchestrationParams, DistannSeedCandidate,
};
use super::scan_registry::ScanTokenGuard;
use super::tuple::DistannNodeTuple;

struct ActiveGenerationIdentity {
    build_id: Uuid,
    fingerprint: [u8; 34],
}

pub(crate) struct PhysicalGenerationScan {
    pub(crate) row_tier_relid: pg_sys::Oid,
    descriptor: DistannGenerationDescriptor,
    generation: GenerationCatalogRow,
    row_relation: HeapRelationGuard,
    _graph_relation: HeapRelationGuard,
    graph_relation_name: String,
    _scan_token: ScanTokenGuard,
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
    pub(crate) fn open(index_oid: pg_sys::Oid) -> Result<Self, String> {
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
        // attempt and executor-level restart can resolve the successor.
        let first = active_generation_identity(index_oid, logical_index_uuid)?
            .ok_or_else(|| "EC_GENERATION_MISSING: logical index has no active epoch".to_owned())?;
        let scan_token = ScanTokenGuard::register_checked(
            logical_index_uuid,
            first.fingerprint,
            || {
                super::coordinator_retirement::ensure_fingerprint_not_retiring(
                    index_oid,
                    logical_index_uuid,
                    &first.fingerprint,
                )
            },
        )
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

        let generation =
            generation_catalog::lookup_generation(index_oid, logical_index_uuid, active.build_id)?
                .ok_or_else(|| {
                    "EC_GENERATION_MISSING: active generation catalog row is absent".to_owned()
                })?;
        if generation.state != "Published" {
            return Err(format!(
                "EC_GENERATION_MISSING: active generation is {} rather than Published",
                generation.state
            ));
        }
        let descriptor = DistannGenerationDescriptor::decode(&generation.generation_descriptor)?;
        if descriptor.digest()? != generation.generation_descriptor_digest
            || descriptor.coordinator_logical_index_uuid != *logical_index_uuid.as_bytes()
        {
            return Err(
                "EC_GENERATION_DESCRIPTOR: active generation descriptor identity mismatch"
                    .to_owned(),
            );
        }
        let row_relation = HeapRelationGuard::try_access_share(generation.row_tier_relid)
            .ok_or_else(|| "EC_GENERATION_MISSING: row-tier relation is absent".to_owned())?;
        let graph_relation = HeapRelationGuard::try_access_share(generation.graph_store_relid)
            .ok_or_else(|| "EC_GENERATION_MISSING: graph-store relation is absent".to_owned())?;
        let graph_relation_name =
            super::handoff::qualified_relation_name(generation.graph_store_relid)?;
        Ok(Self {
            row_tier_relid: generation.row_tier_relid,
            descriptor,
            generation,
            row_relation,
            _graph_relation: graph_relation,
            graph_relation_name,
            _scan_token: scan_token,
        })
    }

    pub(crate) fn row_relation(&self) -> pg_sys::Relation {
        self.row_relation.as_ptr()
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
        let slot =
            TupleTableSlotGuard::single_for_heap_guard(&self.row_relation).ok_or_else(|| {
                "EC_GENERATION_MISSING: could not allocate row-tier scan slot".to_owned()
            })?;

        let mut all_seeds = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT graph_record FROM {} ORDER BY vec_id",
                        self.graph_relation_name
                    ),
                    None,
                    &[],
                )
                .map_err(|error| format!("EC_GENERATION_MISSING: seed scan failed: {error}"))?
                .map(|row| {
                    let bytes = row["graph_record"]
                        .value::<Vec<u8>>()
                        .map_err(|_| {
                            "EC_GENERATION_MISSING: seed graph record decode failed".to_owned()
                        })?
                        .ok_or_else(|| {
                            "EC_GENERATION_MISSING: seed graph record is NULL".to_owned()
                        })?;
                    let node = DistannNodeTuple::decode_physical_v1(
                        &bytes,
                        self.descriptor.graph_degree,
                        code_len,
                    )?;
                    Ok(DistannSeedCandidate {
                        vec_id: node.vec_id,
                        dist: prepared.score_dist(&node.search_code),
                    })
                })
                .collect::<Result<Vec<_>, String>>()
        })?;
        if all_seeds.is_empty() {
            return Ok(DistannHitCollection {
                hits: Vec::new(),
                counters: Default::default(),
                multi_node: false,
            });
        }
        all_seeds.sort_unstable_by(|left, right| left.dist.total_cmp(&right.dist));
        all_seeds.truncate(
            (super::options::current_beam_width() * 2)
                .max(32)
                .min(all_seeds.len()),
        );

        let mut expander = GenerationExpander {
            generation: &self.generation,
            descriptor: &self.descriptor,
            graph_relation_name: &self.graph_relation_name,
            row_relation: &self.row_relation,
            slot: &slot,
            snapshot,
            source_attnum,
            query,
            prepared: &prepared,
            code_len,
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
            multi_node: false,
        })
    }
}

struct GenerationExpander<'a> {
    generation: &'a GenerationCatalogRow,
    descriptor: &'a DistannGenerationDescriptor,
    graph_relation_name: &'a str,
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
        let requested = vec_ids
            .iter()
            .map(|vec_id| i64::from_le_bytes(vec_id.to_le_bytes()))
            .collect::<Vec<_>>();
        let records = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT vec_id, graph_record, row_tid FROM {}
                          WHERE vec_id = ANY($1::bigint[])",
                        self.graph_relation_name
                    ),
                    None,
                    &[requested.as_slice().into()],
                )
                .map_err(|error| format!("EC_GENERATION_MISSING: graph lookup failed: {error}"))?
                .map(|row| {
                    let stored_id = row["vec_id"]
                        .value::<i64>()
                        .map_err(|_| {
                            "EC_GENERATION_MISSING: graph vec_id decode failed".to_owned()
                        })?
                        .ok_or_else(|| "EC_GENERATION_MISSING: graph vec_id is NULL".to_owned())?;
                    let graph_record = row["graph_record"]
                        .value::<Vec<u8>>()
                        .map_err(|_| {
                            "EC_GENERATION_MISSING: graph record decode failed".to_owned()
                        })?
                        .ok_or_else(|| "EC_GENERATION_MISSING: graph record is NULL".to_owned())?;
                    let row_tid = row["row_tid"]
                        .value::<pg_sys::ItemPointerData>()
                        .map_err(|_| {
                            "EC_GENERATION_MISSING: graph row TID decode failed".to_owned()
                        })?
                        .ok_or_else(|| "EC_GENERATION_MISSING: graph row TID is NULL".to_owned())?;
                    let node = DistannNodeTuple::decode_physical_v1(
                        &graph_record,
                        self.descriptor.graph_degree,
                        self.code_len,
                    )?;
                    let (block, offset) = pgrx::itemptr::item_pointer_get_both(row_tid);
                    if node.vec_id != u64::from_le_bytes(stored_id.to_le_bytes())
                        || node.heap_tid
                            != (ItemPointer {
                                block_number: block,
                                offset_number: offset,
                            })
                    {
                        return Err(
                            "EC_GENERATION_MISSING: graph row identity/locator mismatch".to_owned()
                        );
                    }
                    Ok((node.vec_id, node))
                })
                .collect::<Result<HashMap<_, _>, String>>()
        })
        .map_err(DistannExpandError::Internal)?;

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
