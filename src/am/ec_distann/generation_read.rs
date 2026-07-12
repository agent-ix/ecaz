//! Published physical-generation reader for Task 179.
//!
//! The logical control index is metadata-only. This module resolves its active
//! pointer to the immutable generation heap relations, pins the exact epoch in
//! the shared scan registry, and adapts those relations to the existing FR-081
//! orchestration seam.

use std::collections::HashMap;

use pgrx::datum::Uuid;
use pgrx::iter::TableIterator;
use pgrx::{default, name, pg_extern, pg_sys, PgRelation, Spi};

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
    DistannOrchestrationParams, DistannScanHit,
};
use super::scan_registry::ScanTokenGuard;
use super::tuple::DistannNodeTuple;

struct ActiveGenerationIdentity {
    build_id: Uuid,
    fingerprint: [u8; 34],
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
        || routes.iter().filter(|route| route.is_local).count() != 1
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
    descriptor: DistannGenerationDescriptor,
    generation: GenerationCatalogRow,
    row_relation: HeapRelationGuard,
    _graph_relation: HeapRelationGuard,
    graph_relation_name: String,
    source_attnum: i32,
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
        let descriptor = DistannGenerationDescriptor::decode(&generation.generation_descriptor)
            .map_err(DistannExpandError::GenerationMissing)?;
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
        let graph_relation_name =
            super::handoff::qualified_relation_name(generation.graph_store_relid)
                .map_err(DistannExpandError::GenerationMissing)?;
        Ok(Self {
            descriptor,
            generation,
            row_relation,
            _graph_relation: graph_relation,
            graph_relation_name,
            source_attnum,
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

    fn expand(
        &self,
        query: &[f32],
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
        let binding = DistannCodecBinding::from_artifact(&self.descriptor.codec_artifact)
            .map_err(DistannExpandError::Internal)?;
        let code_len = binding
            .code_len(usize::from(self.descriptor.dimensions))
            .map_err(DistannExpandError::Internal)?;
        let prepared =
            DistannPreparedQuery::prepare_artifact(&self.descriptor.codec_artifact, query)
                .map_err(DistannExpandError::Internal)?;
        let slot =
            TupleTableSlotGuard::single_for_heap_guard(&self.row_relation).ok_or_else(|| {
                DistannExpandError::Internal("could not allocate retained row-tier slot".to_owned())
            })?;
        let mut expander = GenerationExpander {
            generation: &self.generation,
            descriptor: &self.descriptor,
            graph_relation_name: &self.graph_relation_name,
            row_relation: &self.row_relation,
            slot: &slot,
            snapshot,
            source_attnum: self.source_attnum,
            query,
            prepared: &prepared,
            code_len,
        };
        expander.expand_nodes(vec_ids, code_threshold)
    }

    fn resolve_nodes(&self, vec_ids: &[u64]) -> Result<Vec<DistannNodeTuple>, DistannExpandError> {
        self.validate_ownership(vec_ids)?;
        if vec_ids.is_empty() {
            return Ok(Vec::new());
        }
        let binding = DistannCodecBinding::from_artifact(&self.descriptor.codec_artifact)
            .map_err(DistannExpandError::Internal)?;
        let code_len = binding
            .code_len(usize::from(self.descriptor.dimensions))
            .map_err(DistannExpandError::Internal)?;
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
                .map_err(|error| {
                    DistannExpandError::GenerationMissing(format!(
                        "physical materialize graph lookup failed: {error}"
                    ))
                })?
                .map(|row| {
                    let stored_id = row["vec_id"]
                        .value::<i64>()
                        .map_err(|error| {
                            DistannExpandError::GenerationMissing(format!(
                                "physical graph vec_id decode failed: {error}"
                            ))
                        })?
                        .ok_or_else(|| {
                            DistannExpandError::GenerationMissing(
                                "physical graph vec_id is NULL".to_owned(),
                            )
                        })?;
                    let bytes = row["graph_record"]
                        .value::<Vec<u8>>()
                        .map_err(|error| {
                            DistannExpandError::GenerationMissing(format!(
                                "physical graph record decode failed: {error}"
                            ))
                        })?
                        .ok_or_else(|| {
                            DistannExpandError::GenerationMissing(
                                "physical graph record is NULL".to_owned(),
                            )
                        })?;
                    let row_tid = row["row_tid"]
                        .value::<pg_sys::ItemPointerData>()
                        .map_err(|error| {
                            DistannExpandError::GenerationMissing(format!(
                                "physical graph row TID decode failed: {error}"
                            ))
                        })?
                        .ok_or_else(|| {
                            DistannExpandError::GenerationMissing(
                                "physical graph row TID is NULL".to_owned(),
                            )
                        })?;
                    let node = DistannNodeTuple::decode_physical_v1(
                        &bytes,
                        self.descriptor.graph_degree,
                        code_len,
                    )
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
                    Ok((node.vec_id, node))
                })
                .collect::<Result<HashMap<_, _>, DistannExpandError>>()
        })?;
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
    let ids = vec_ids
        .iter()
        .map(|value| u64::from_le_bytes(value.to_le_bytes()))
        .collect::<Vec<_>>();
    let rows: Vec<PhysicalExpandRow> = (|| {
        RetainedGenerationScan::open(index_regclass.oid(), &epoch_fingerprint)?
            .expand(&query, &ids, code_threshold)
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
    })()
    .unwrap_or_else(|error: DistannExpandError| error.raise());
    TableIterator::new(rows.into_iter())
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
    pub(crate) row_tier_relid: pg_sys::Oid,
    descriptor: DistannGenerationDescriptor,
    generation: GenerationCatalogRow,
    row_relation: HeapRelationGuard,
    _graph_relation: HeapRelationGuard,
    graph_relation_name: String,
    fingerprint: [u8; 34],
    routes: Vec<PhysicalOwnerRoute>,
    head_index: Option<super::head_sample::DistannPhysicalHeadIndex>,
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
        let routes = physical_owner_routes(
            index_oid,
            logical_index_uuid,
            active.build_id,
            descriptor.roster.len(),
        )?;
        let local_route = routes
            .get(generation.owner_ordinal as usize)
            .ok_or_else(|| {
                "EC_NODE_DESCRIPTOR: local generation owner is outside the roster".to_owned()
            })?;
        if !local_route.is_local {
            return Err(
                "EC_NODE_DESCRIPTOR: local generation owner is not the local binding".to_owned(),
            );
        }
        let (head_sample, manifest_build_options) = super::head_sample::load_head_sample(
            index_oid,
            logical_index_uuid,
            active.build_id,
            &active.fingerprint,
        )?;
        let exact_options = manifest_build_options.options;
        let head_index = super::head_sample::DistannPhysicalHeadIndex::build(
            head_sample,
            usize::from(manifest_build_options.graph_degree),
            usize::from(exact_options.build_list_size),
            exact_options.alpha,
            exact_options.seed,
        )?;
        Ok(Self {
            row_tier_relid: generation.row_tier_relid,
            descriptor,
            generation,
            row_relation,
            _graph_relation: graph_relation,
            graph_relation_name,
            fingerprint: active.fingerprint,
            routes,
            head_index,
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

        let seed_limit = (super::options::current_beam_width() * 2).max(32);
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

        let local_expander = GenerationExpander {
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
        let mut expander = PhysicalMultiOwnerExpander {
            local: local_expander,
            local_ordinal: self.generation.owner_ordinal as usize,
            descriptor: &self.descriptor,
            routes: &self.routes,
            fingerprint: &self.fingerprint,
            query,
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
        for (ordinal, bucket) in buckets.iter().enumerate() {
            if bucket.is_empty() {
                continue;
            }
            if ordinal == self.generation.owner_ordinal as usize {
                return Err(
                    "EC_INTERNAL: local owner produced a remote physical hit locator".to_owned(),
                );
            }
            let ids = bucket.iter().map(|(_, vec_id)| *vec_id).collect::<Vec<_>>();
            let route = &self.routes[ordinal];
            let response = super::remote_transport::remote_physical_materialize_payloads(
                route
                    .conninfo
                    .as_deref()
                    .expect("remote route has conninfo"),
                &route.remote_index_regclass,
                &self.fingerprint,
                &ids,
                &projection_attnums,
                &schema_fingerprint,
            )
            .map_err(|error| error.to_string())?;
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
                if payload.is_tombstone || payload.tuple_payload_missing {
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
    local: GenerationExpander<'a>,
    local_ordinal: usize,
    descriptor: &'a DistannGenerationDescriptor,
    routes: &'a [PhysicalOwnerRoute],
    fingerprint: &'a [u8; 34],
    query: &'a [f32],
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
        for (ordinal, bucket) in buckets.iter().enumerate() {
            if bucket.is_empty() {
                continue;
            }
            let owned = bucket.iter().map(|(_, vec_id)| *vec_id).collect::<Vec<_>>();
            let response = if ordinal == self.local_ordinal {
                self.local.expand_nodes(&owned, code_threshold)?
            } else {
                let route = &self.routes[ordinal];
                super::remote_transport::remote_physical_expand_nodes(
                    route
                        .conninfo
                        .as_deref()
                        .expect("remote route has conninfo"),
                    &route.remote_index_regclass,
                    self.fingerprint,
                    self.query,
                    &owned,
                    code_threshold,
                )?
            };
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
