//! Owner-local DML for a published physical generation (Task 167).
//!
//! The legacy M5 experiment writes the index relation directly.  A published
//! v5 generation is different: its graph and row tier are ordinary owner-local
//! relations, and the control index contains no graph records.  This module is
//! deliberately small and transactional.  It resolves the active generation,
//! plans against the published graph, appends the complete row tier tuple and
//! graph record, then amends the affected current graph rows in the same SQL
//! transaction.  An error therefore aborts every physical write together.

use std::collections::{HashMap, HashSet};

use pgrx::{pg_sys, Spi};

use crate::am::common::heap_slot::TupleSlotWriter;
use crate::am::ec_diskann::{decode_heap_tid, ecvector_datum_to_vec};
use crate::am::ec_distann::ambuild;
use crate::am::ec_distann::generation_read::PhysicalGenerationScan;
use crate::am::ec_distann::handoff::qualified_relation_name;
use crate::am::ec_distann::insert::{select_insert_forward_neighbors, DistannForwardCandidate};
use crate::am::ec_distann::options;
use crate::am::ec_distann::stage_counters::{self, DistannInsertWork};
use crate::am::ec_distann::tuple::DistannNodeTuple;
use crate::storage::page::ItemPointer;
use crate::storage::relation_guard::{HeapRelationGuard, IndexRelationGuard};
use crate::storage::slot_guard::TupleTableSlotGuard;
use crate::storage::snapshot_guard::RegisteredSnapshotGuard;

use super::quantizer::DistannCodecBinding;
use super::routine::indexed_ecvector_attnum;
use super::scan::DistannScanHit;

/// Execute the physical owner path for one AM insert.  The caller is inside
/// PostgreSQL's INSERT/UPDATE transaction; this function never commits or
/// opens a second transaction.
pub(crate) unsafe fn insert_from_callback(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: pg_sys::ItemPointer,
    index_info: *mut pg_sys::IndexInfo,
    index_unchanged: bool,
) -> Result<(), String> {
    if index_relation.is_null() || heap_relation.is_null() || index_info.is_null() {
        return Err(
            "EC_GENERATION_MISSING: physical insert received a null callback pointer".to_owned(),
        );
    }
    if values.is_null() || isnull.is_null() || *isnull {
        return Err(
            "EC_SCHEMA_MISMATCH: physical insert received a NULL indexed vector".to_owned(),
        );
    }
    let vector = ecvector_datum_to_vec(*values);
    let options = options::relation_options(index_relation);
    let identity = ambuild::resolve_identity_attribute(heap_relation, index_info, &options)
        .ok_or_else(|| {
            "EC_SOURCE_IDENTITY: physical insert requires source_identity='include'".to_owned()
        })?;
    let identity_payload = ambuild::extract_identity_payload(identity, values, isnull);
    let vec_id = super::identity::vec_id_from_source_identity(&identity_payload);

    // The heap tuple supplied to an index AM callback is the current command's
    // own tuple.  It is not visible through the ordinary active snapshot until
    // the command advances its CID; SnapshotSelf is the callback-safe snapshot
    // used by the other AM insert paths for this exact case.
    let source_snapshot = std::ptr::addr_of_mut!(pg_sys::SnapshotSelfData);
    let heap_guard = HeapRelationGuard::try_open(
        (*(*index_relation).rd_index).indrelid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
    )
    .ok_or_else(|| "EC_GENERATION_MISSING: source heap could not be opened".to_owned())?;
    let source_slot = TupleTableSlotGuard::single_for_heap_guard(&heap_guard).ok_or_else(|| {
        "EC_GENERATION_MISSING: source heap slot could not be allocated".to_owned()
    })?;
    fetch_source_tuple(
        heap_relation,
        heap_tid,
        source_slot.as_ptr(),
        source_snapshot,
    )?;
    // `index_unchanged` is only an executor hint and is false for a
    // vector-changing UPDATE. The heap header is the authoritative local
    // discriminator available to an AM callback; retaining the hint covers
    // the unchanged-key UPDATE case without turning a normal INSERT into a
    // replacement.
    let allow_replacement = index_unchanged || source_slot_is_updated(source_slot.as_ptr());
    let source_tid = decode_heap_tid(heap_tid);
    insert_from_prepared_slot(
        index_relation,
        vector,
        vec_id,
        identity_payload,
        allow_replacement,
        identity,
        source_slot.as_ptr(),
        source_tid,
        None,
        None,
    )
}

unsafe fn insert_from_prepared_slot(
    index_relation: pg_sys::Relation,
    vector: Vec<f32>,
    vec_id: u64,
    identity_payload: [u8; 16],
    allow_replacement: bool,
    identity: ambuild::DistannIdentityAttribute,
    source_slot: *mut pg_sys::TupleTableSlot,
    source_tid: ItemPointer,
    scan_fingerprint: Option<[u8; 34]>,
    planned_forward_payload: Option<&[u8]>,
) -> Result<(), String> {
    if index_relation.is_null() || source_slot.is_null() {
        return Err(
            "EC_GENERATION_MISSING: physical insert received a null prepared slot".to_owned(),
        );
    }
    stage_counters::record_insert_work(DistannInsertWork::InsertAttempts, 1);
    let options = options::relation_options(index_relation);

    let index_oid = (*index_relation).rd_id;
    let mut scan = match (scan_fingerprint, planned_forward_payload.is_some()) {
        (Some(fingerprint), true) => {
            PhysicalGenerationScan::open_at_fingerprint_for_owner_insert(index_oid, fingerprint)?
        }
        (Some(fingerprint), false) => {
            PhysicalGenerationScan::open_at_fingerprint(index_oid, fingerprint)?
        }
        (None, _) => PhysicalGenerationScan::open(index_oid)?,
    };
    let (_logical_uuid, _build_id, fingerprint, routed_descriptor, routes) =
        scan.traversal_replica_source();
    let routes = routes.to_owned();
    let owner = super::placement::owning_node(
        vec_id,
        routes.len(),
        routed_descriptor.placement_hash_version,
    );
    let owner_route = routes.get(owner).ok_or_else(|| {
        "EC_NODE_DESCRIPTOR: physical insert owner is outside the active roster".to_owned()
    })?;
    let (generation, descriptor) = scan.local_write_identity()?;
    let source_attnum = indexed_ecvector_attnum(index_relation)?;
    if vector.len() != usize::from(descriptor.dimensions) {
        return Err(format!(
            "EC_SCHEMA_MISMATCH: inserted vector has {} dimensions, generation requires {}",
            vector.len(),
            descriptor.dimensions
        ));
    }

    let local_owner = routes
        .iter()
        .position(|route| route.is_local)
        .ok_or_else(|| "EC_NODE_DESCRIPTOR: active roster has no local owner".to_owned())?;

    let owner_intent_conninfo = if owner_route.is_local {
        owner_route.roster_conninfo.as_str()
    } else {
        owner_route.conninfo.as_deref().ok_or_else(|| {
            "EC_NODE_DESCRIPTOR: remote physical insert has no connection descriptor".to_owned()
        })?
    };
    let coordinator_conninfo = routes[local_owner].roster_conninfo.as_str();
    super::remote_transport::record_physical_insert_intent(
        owner_intent_conninfo,
        coordinator_conninfo,
        index_oid,
        owner_route.node_id,
        generation.epoch,
        vec_id,
    )?;

    // Plan while the read-side generation guard is alive. The physical scan
    // gives us the bounded FR-081 frontier. Remote candidates are materialized
    // through the generation-owned payload contract below; their row TIDs are
    // never treated as coordinator-local.
    let snapshot = pg_sys::GetActiveSnapshot();
    if snapshot.is_null() {
        return Err("EC_BUILD_STATE: physical insert has no active snapshot".to_owned());
    }
    let row_read_relation = row_relation_for_payload(&generation)?;

    let graph_name = qualified_relation_name(generation.graph_store_relid)?;
    let previous_version = current_record_version(&graph_name, vec_id)?;
    if previous_version.is_some() {
        let current_identity =
            current_record_identity(&graph_name, &row_read_relation, vec_id, identity, snapshot)?
                .ok_or_else(|| {
                format!(
                "EC_INSERT_PUBLISH: current vec_id {vec_id:#018x} has no readable row-tier identity"
            )
            })?;
        if current_identity != identity_payload {
            return Err(format!(
                "EC_DUPLICATE_VEC_ID: vec_id {vec_id:#018x} maps to a different source identity"
            ));
        }
    }
    if previous_version.is_some() && !allow_replacement {
        return Err(format!(
            "EC_DUPLICATE_VEC_ID: vec_id {vec_id:#018x} already has a current physical record"
        ));
    }
    let row_relation = HeapRelationGuard::try_open(
        generation.row_tier_relid,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
    )
    .ok_or_else(|| {
        "EC_GENERATION_MISSING: owner row tier could not be opened for write".to_owned()
    })?;
    let graph_relation = HeapRelationGuard::try_open(
        generation.graph_store_relid,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
    )
    .ok_or_else(|| {
        "EC_GENERATION_MISSING: owner graph store could not be opened for write".to_owned()
    })?;

    let code_binding = DistannCodecBinding::from_artifact(&descriptor.codec_artifact)?;
    let code_len = code_binding.code_len(usize::from(descriptor.dimensions))?;
    let new_code = code_binding.encode(&vector);
    if new_code.len() != code_len {
        return Err("EC_INSERT_CODEC: generated search code has the wrong length".to_owned());
    }

    let (candidates, mut remote_vectors, forward_ids) = if let Some(payload) =
        planned_forward_payload
    {
        let planned = decode_planned_forward(payload, vector.len(), descriptor.graph_degree)?;
        let mut candidates = Vec::with_capacity(planned.len());
        let mut remote_vectors = HashMap::new();
        let mut seen = HashSet::new();
        for (planned_vec_id, planned_vector) in planned {
            if planned_vec_id == vec_id || !seen.insert(planned_vec_id) {
                continue;
            }
            let owner = super::placement::owning_node(
                planned_vec_id,
                routes.len(),
                routed_descriptor.placement_hash_version,
            );
            if owner == local_owner {
                let hit = DistannScanHit {
                    vec_id: planned_vec_id,
                    heap_tid: ItemPointer::INVALID,
                    owner_heap_tid: ItemPointer::INVALID,
                    exact_dist: 0.0,
                };
                let Some(candidate) = read_current_candidate(
                    &graph_name,
                    &row_relation,
                    hit,
                    descriptor.graph_degree,
                    code_len,
                    source_attnum,
                    snapshot,
                )?
                else {
                    return Err(format!(
                        "EC_GENERATION_MISSING: planned local candidate {planned_vec_id:#018x} is absent"
                    ));
                };
                if candidate.source_vector != planned_vector {
                    return Err(format!(
                        "EC_EPOCH_MISMATCH: planned candidate {planned_vec_id:#018x} changed during owner insert"
                    ));
                }
                candidates.push(candidate);
            } else {
                remote_vectors.insert(planned_vec_id, planned_vector.clone());
                candidates.push(Candidate {
                    vec_id: planned_vec_id,
                    source_vector: planned_vector,
                    node: DistannNodeTuple::empty(),
                    graph_tid: ItemPointer::INVALID,
                });
            }
        }
        let forward_ids = candidates
            .iter()
            .map(|candidate| candidate.vec_id)
            .collect();
        (candidates, remote_vectors, forward_ids)
    } else {
        let list_size = usize::from(descriptor.graph_degree)
            .saturating_mul(8)
            .max(64);
        let hits = scan
            .search(snapshot, source_attnum, &vector, list_size)
            .map_err(|error| format!("EC_INSERT_SEARCH: {error}"))?;
        stage_counters::record_insert_work(DistannInsertWork::SearchCandidates, hits.hits.len());
        let remote_ids = hits
            .hits
            .iter()
            .filter(|hit| hit.heap_tid == ItemPointer::INVALID && hit.vec_id != vec_id)
            .map(|hit| hit.vec_id)
            .collect::<Vec<_>>();
        let remote_vectors =
            materialize_remote_vectors(&scan, &row_read_relation, &remote_ids, source_attnum)?;
        let mut candidates = Vec::new();
        let mut seen = HashSet::new();
        for hit in hits.hits {
            if hit.vec_id == vec_id || !seen.insert(hit.vec_id) {
                continue;
            }
            if hit.heap_tid == ItemPointer::INVALID {
                let Some(source_vector) = remote_vectors.get(&hit.vec_id) else {
                    return Err(format!(
                        "EC_GENERATION_MISSING: remote candidate payload is absent for vec_id {:#018x}",
                        hit.vec_id
                    ));
                };
                candidates.push(Candidate {
                    vec_id: hit.vec_id,
                    source_vector: source_vector.clone(),
                    node: DistannNodeTuple::empty(),
                    graph_tid: ItemPointer::INVALID,
                });
                continue;
            }
            let Some(candidate) = read_current_candidate(
                &graph_name,
                &row_relation,
                hit,
                descriptor.graph_degree,
                code_len,
                source_attnum,
                snapshot,
            )?
            else {
                continue;
            };
            candidates.push(candidate);
        }
        let forward_ids = select_insert_forward_neighbors(
            &vector,
            &candidates
                .iter()
                .map(|candidate| DistannForwardCandidate {
                    vec_id: candidate.vec_id,
                    source_vector: candidate.source_vector.clone(),
                })
                .collect::<Vec<_>>(),
            options.alpha,
            usize::from(descriptor.graph_degree),
        )?;
        (candidates, remote_vectors, forward_ids)
    };
    let forward = forward_ids
        .iter()
        .filter_map(|id| candidates.iter().find(|candidate| candidate.vec_id == *id))
        .collect::<Vec<_>>();
    stage_counters::record_insert_work(DistannInsertWork::ForwardNeighborsSelected, forward.len());

    let extra_remote_ids = forward
        .iter()
        .filter(|candidate| candidate.graph_tid != ItemPointer::INVALID)
        .flat_map(|candidate| {
            candidate.node.neighbor_vec_ids[..usize::from(candidate.node.neighbor_count)]
                .iter()
                .copied()
                .filter(|neighbor| {
                    super::placement::owning_node(
                        *neighbor,
                        routes.len(),
                        routed_descriptor.placement_hash_version,
                    ) != local_owner
                })
        })
        .filter(|neighbor| !remote_vectors.contains_key(neighbor))
        .collect::<Vec<_>>();
    if !extra_remote_ids.is_empty() {
        remote_vectors.extend(materialize_remote_vectors(
            &scan,
            &row_read_relation,
            &extra_remote_ids,
            source_attnum,
        )?);
    }
    drop(scan);

    if !owner_route.is_local {
        let conninfo = owner_route.conninfo.as_deref().ok_or_else(|| {
            "EC_NODE_DESCRIPTOR: remote physical insert has no connection descriptor".to_owned()
        })?;
        let (payload_nulls, payload_offsets, payload_values) =
            ambuild::freeze_source_slot_packed(source_slot)?;
        let index_name = unsafe { super::routine::distann_index_relname(index_relation) };
        let roster_spec = roster_spec_for_routes(&routes)?;
        let planned_forward = encode_planned_forward(&forward)?;
        stage_counters::record_insert_work(DistannInsertWork::OwnerWrites, 1);
        super::remote_transport::remote_physical_insert(
            &super::remote_transport::DistannRemotePhysicalInsertRequest {
                index_oid,
                conninfo,
                roster_spec: &roster_spec,
                target_node_id: owner_route.node_id,
                epoch: generation.epoch,
                index_regclass: &index_name,
                epoch_fingerprint: &fingerprint,
                vec_id,
                source_vector: &vector,
                source_identity: &identity_payload,
                payload_nulls: &payload_nulls,
                payload_offsets: &payload_offsets,
                payload_values: &payload_values,
                planned_forward: &planned_forward,
                allow_replacement,
            },
        )?;
        for candidate in forward {
            if candidate.graph_tid != ItemPointer::INVALID {
                amend_backlink(
                    &graph_name,
                    candidate,
                    vec_id,
                    &vector,
                    &new_code,
                    descriptor.graph_degree,
                    code_len,
                    &row_relation,
                    source_attnum,
                    options.alpha,
                    &remote_vectors,
                )?;
                continue;
            }
            let target = super::placement::owning_node(
                candidate.vec_id,
                routes.len(),
                routed_descriptor.placement_hash_version,
            );
            let target_route = routes.get(target).ok_or_else(|| {
                "EC_NODE_DESCRIPTOR: backlink target is outside the active roster".to_owned()
            })?;
            if target_route.is_local {
                return Err(
                    "EC_INSERT_BACKLINK: remote candidate resolved to local owner without a ctid"
                        .to_owned(),
                );
            }
            let conninfo = target_route.conninfo.as_deref().ok_or_else(|| {
                "EC_NODE_DESCRIPTOR: remote backlink has no connection descriptor".to_owned()
            })?;
            super::remote_transport::remote_physical_backlink(
                &super::remote_transport::DistannRemotePhysicalBacklinkRequest {
                    index_oid,
                    conninfo,
                    roster_spec: &roster_spec,
                    target_node_id: target_route.node_id,
                    epoch: generation.epoch,
                    index_regclass: &index_name,
                    epoch_fingerprint: &fingerprint,
                    target_vec_id: candidate.vec_id,
                    target_source_vector: &candidate.source_vector,
                    new_vec_id: vec_id,
                    new_source_vector: &vector,
                    new_code: &new_code,
                },
            )?;
            // The owner-side endpoint performs the locked read/modify/write;
            // count the completed RPC here so coordinator evidence includes
            // successful remote amendments as well as local ones.
            stage_counters::record_insert_work(DistannInsertWork::BacklinkAmendments, 1);
        }
        if options::debug_fail_insert() {
            return Err(
                "EC_FAULT_INJECTED: physical insert failed after owner writes before commit"
                    .to_owned(),
            );
        }
        if source_tid != ItemPointer::INVALID {
            update_source_mapping(index_oid, source_tid, vec_id)?;
        }
        return Ok(());
    }

    // Append the complete row-tier tuple before publishing its graph pointer.
    // If any later graph/index operation fails, PostgreSQL aborts this same
    // transaction and the row tuple is not visible to a published scan.
    let row_tid = append_row_tuple(&row_relation, source_slot)?;
    stage_counters::record_insert_work(DistannInsertWork::OwnerWrites, 1);
    let mut neighbor_vec_ids = vec![0_u64; usize::from(descriptor.graph_degree)];
    let mut neighbor_codes = vec![0_u8; usize::from(descriptor.graph_degree) * code_len];
    for (slot, candidate) in forward.iter().enumerate() {
        neighbor_vec_ids[slot] = candidate.vec_id;
        let candidate_code = code_binding.encode(&candidate.source_vector);
        if candidate_code.len() != code_len {
            return Err("EC_INSERT_CODEC: candidate search code has the wrong length".to_owned());
        }
        neighbor_codes[slot * code_len..(slot + 1) * code_len].copy_from_slice(&candidate_code);
    }
    let node = DistannNodeTuple {
        tombstoned: false,
        vec_id,
        heap_tid: row_tid,
        neighbor_count: u16::try_from(forward.len())
            .map_err(|_| "EC_INSERT_GRAPH: forward degree exceeds u16".to_owned())?,
        search_code: new_code.clone(),
        neighbor_vec_ids,
        neighbor_codes,
    };
    let graph_record = node.encode_physical_v1(descriptor.graph_degree, code_len)?;

    // Same vec_id means the stable identity survived an UPDATE.  Retain the
    // old graph tuple and redirect the partial unique directory atomically.
    if previous_version.is_some() {
        Spi::connect_mut(|client| {
            client.update(
                &format!("UPDATE {graph_name} SET is_current = false WHERE vec_id = $1 AND is_current"),
                None,
                &[i64::from_le_bytes(vec_id.to_le_bytes()).into()],
            ).map_err(|error| format!("EC_INSERT_PUBLISH: could not retire prior vec_id: {error}"))?;
            Ok::<(), String>(())
        })?;
    }
    insert_graph_record(
        &graph_name,
        vec_id,
        &graph_record,
        row_tid,
        previous_version.unwrap_or(-1).saturating_add(1),
    )?;
    stage_counters::record_insert_work(DistannInsertWork::GraphRecordsAppended, 1);
    if options::debug_fail_insert() {
        return Err(
            "EC_FAULT_INJECTED: physical insert failed after graph append before backlinks"
                .to_owned(),
        );
    }

    let index_name = unsafe { super::routine::distann_index_relname(index_relation) };
    let roster_spec = roster_spec_for_routes(&routes)?;
    // Amend at most graph_degree forward-neighbor records. Local mutations
    // use their owner-local ctid; remote mutations are independently prepared
    // and resolved by the same top-level transaction callbacks as the owner
    // append, so an abort cannot leave a cross-owner dangling edge.
    if planned_forward_payload.is_none() {
        for candidate in forward {
            if candidate.graph_tid == ItemPointer::INVALID {
                let target = super::placement::owning_node(
                    candidate.vec_id,
                    routes.len(),
                    routed_descriptor.placement_hash_version,
                );
                let target_route = routes.get(target).ok_or_else(|| {
                    "EC_NODE_DESCRIPTOR: backlink target is outside the active roster".to_owned()
                })?;
                if target_route.is_local {
                    return Err(
                    "EC_INSERT_BACKLINK: remote candidate resolved to local owner without a ctid"
                        .to_owned(),
                );
                }
                let conninfo = target_route.conninfo.as_deref().ok_or_else(|| {
                    "EC_NODE_DESCRIPTOR: remote backlink has no connection descriptor".to_owned()
                })?;
                super::remote_transport::remote_physical_backlink(
                    &super::remote_transport::DistannRemotePhysicalBacklinkRequest {
                        index_oid,
                        conninfo,
                        roster_spec: &roster_spec,
                        target_node_id: target_route.node_id,
                        epoch: generation.epoch,
                        index_regclass: &index_name,
                        epoch_fingerprint: &fingerprint,
                        target_vec_id: candidate.vec_id,
                        target_source_vector: &candidate.source_vector,
                        new_vec_id: vec_id,
                        new_source_vector: &vector,
                        new_code: &new_code,
                    },
                )?;
                stage_counters::record_insert_work(DistannInsertWork::BacklinkAmendments, 1);
                continue;
            }
            amend_backlink(
                &graph_name,
                candidate,
                vec_id,
                &vector,
                &new_code,
                descriptor.graph_degree,
                code_len,
                &row_relation,
                source_attnum,
                options.alpha,
                &remote_vectors,
            )?;
        }
    }
    if source_tid != ItemPointer::INVALID {
        update_source_mapping(index_oid, source_tid, vec_id)?;
    }
    drop(graph_relation);
    drop(row_relation);
    Ok(())
}

fn update_source_mapping(
    index_oid: pg_sys::Oid,
    source_tid: ItemPointer,
    vec_id: u64,
) -> Result<(), String> {
    let signed_id = i64::from_le_bytes(vec_id.to_le_bytes());
    let source_map =
        super::generation_catalog::extension_relation_name("ec_distann_physical_source_map")?;
    let mut tid = pg_sys::ItemPointerData::default();
    pgrx::itemptr::item_pointer_set_all(
        &mut tid,
        source_tid.block_number,
        source_tid.offset_number,
    );
    Spi::connect_mut(|client| {
        client
            .update(
                &format!("DELETE FROM {source_map} WHERE index_oid = $1::oid AND vec_id = $2"),
                None,
                &[index_oid.into(), signed_id.into()],
            )
            .map_err(|error| {
                format!("EC_INSERT_PUBLISH: source mapping cleanup failed: {error}")
            })?;
        client
            .update(
                &format!(
                    "INSERT INTO {source_map} (index_oid, source_tid, vec_id) \
                 VALUES ($1::oid, $2::tid, $3) \
                 ON CONFLICT (index_oid, source_tid) DO UPDATE SET vec_id = EXCLUDED.vec_id, \
                     created_at = clock_timestamp()"
                ),
                None,
                &[index_oid.into(), tid.into(), signed_id.into()],
            )
            .map_err(|error| format!("EC_INSERT_PUBLISH: source mapping append failed: {error}"))?;
        Ok::<(), String>(())
    })
}

fn delete_source_mapping(
    index_oid: pg_sys::Oid,
    source_tid: ItemPointer,
    vec_id: u64,
) -> Result<(), String> {
    let signed_id = i64::from_le_bytes(vec_id.to_le_bytes());
    let source_map =
        super::generation_catalog::extension_relation_name("ec_distann_physical_source_map")?;
    let mut tid = pg_sys::ItemPointerData::default();
    pgrx::itemptr::item_pointer_set_all(
        &mut tid,
        source_tid.block_number,
        source_tid.offset_number,
    );
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "DELETE FROM {source_map} \
                     WHERE index_oid = $1::oid AND source_tid = $2::tid AND vec_id = $3"
                ),
                None,
                &[index_oid.into(), tid.into(), signed_id.into()],
            )
            .map_err(|error| format!("EC_DELETE_ROUTE: source mapping cleanup failed: {error}"))?;
        Ok::<(), String>(())
    })
}

/// Rebuild the source-TID directory after a physical generation becomes
/// active. Published generations are immutable, while VACUUM's callback still
/// enumerates current source-table TIDs; refreshing this directory on publish
/// keeps delete routing correct for rows that predate incremental DML.
pub(crate) fn refresh_source_mapping(index_oid: pg_sys::Oid) -> Result<(), String> {
    let source_oid = Spi::get_one::<pg_sys::Oid>(&format!(
        "SELECT indrelid FROM pg_catalog.pg_index WHERE indexrelid = {index_oid}::oid"
    ))
    .map_err(|error| format!("EC_SCHEMA_MISMATCH: source relation lookup failed: {error}"))?
    .ok_or_else(|| "EC_SCHEMA_MISMATCH: source relation is absent".to_owned())?;
    let (identity_attnum, identity_name, identity_type) = Spi::connect(|client| {
        client
            .select(
                "SELECT i.indkey[1]::int4 AS attnum,
                        a.attname::text AS attname,
                        t.typname::text AS typname
                   FROM pg_catalog.pg_index i
                     JOIN pg_catalog.pg_attribute a
                     ON a.attrelid = i.indrelid AND a.attnum = i.indkey[1]
                   JOIN pg_catalog.pg_type t ON t.oid = a.atttypid
                  WHERE i.indexrelid = $1::oid AND i.indnatts = 2",
                None,
                &[index_oid.into()],
            )
            .map_err(|error| format!("EC_SCHEMA_MISMATCH: source identity lookup failed: {error}"))?
            .map(|row| {
                let attnum = row["attnum"]
                    .value::<i32>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "identity attnum is NULL".to_owned())?;
                let name = row["attname"]
                    .value::<String>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "identity name is NULL".to_owned())?;
                let typname = row["typname"]
                    .value::<String>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "identity type is NULL".to_owned())?;
                Ok::<_, String>((attnum, name, typname))
            })
            .next()
            .transpose()
    })?
    .ok_or_else(|| "EC_SCHEMA_MISMATCH: source identity INCLUDE column is absent".to_owned())?;
    if identity_attnum <= 0 || !matches!(identity_type.as_str(), "uuid" | "bytea") {
        return Err("EC_SCHEMA_MISMATCH: source identity must be uuid or bytea".to_owned());
    }
    let source_relation = qualified_relation_name(source_oid)?;
    let identity_expr = if identity_type == "uuid" {
        format!("uuid_send({})", super::quote_ident(&identity_name))
    } else {
        super::quote_ident(&identity_name)
    };
    let rows = Spi::connect(|client| {
        client
            .select(
                &format!("SELECT ctid, {identity_expr} AS identity_payload FROM {source_relation}"),
                None,
                &[],
            )
            .map_err(|error| {
                format!("EC_DELETE_ROUTE: source mapping refresh scan failed: {error}")
            })?
            .map(|row| {
                let tid = row["ctid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "source ctid is NULL".to_owned())?;
                let payload = row["identity_payload"]
                    .value::<Vec<u8>>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "source identity is NULL".to_owned())?;
                let identity: [u8; 16] = payload.try_into().map_err(|_| {
                    "EC_SOURCE_IDENTITY: source identity is not 16 bytes".to_owned()
                })?;
                Ok::<_, String>((
                    tid,
                    i64::from_le_bytes(
                        super::identity::vec_id_from_source_identity(&identity).to_le_bytes(),
                    ),
                ))
            })
            .collect::<Result<Vec<_>, _>>()
    })?;
    let table =
        super::generation_catalog::extension_relation_name("ec_distann_physical_source_map")?;
    let tids = rows.iter().map(|(tid, _)| *tid).collect::<Vec<_>>();
    let vec_ids = rows.iter().map(|(_, vec_id)| *vec_id).collect::<Vec<_>>();
    Spi::connect_mut(|client| {
        client
            .update(
                &format!("DELETE FROM {table} WHERE index_oid = $1::oid"),
                None,
                &[index_oid.into()],
            )
            .map_err(|error| {
                format!("EC_DELETE_ROUTE: source mapping refresh cleanup failed: {error}")
            })?;
        if tids.is_empty() {
            return Ok::<(), String>(());
        }
        client
            .update(
                &format!(
                    "INSERT INTO {table} (index_oid, source_tid, vec_id) \
                     SELECT $1::oid, source_tid, vec_id \
                       FROM unnest($2::tid[], $3::bigint[]) AS rows(source_tid, vec_id) \
                     ON CONFLICT (index_oid, source_tid) DO UPDATE SET vec_id = EXCLUDED.vec_id, \
                         created_at = clock_timestamp()"
                ),
                None,
                &[index_oid.into(), tids.into(), vec_ids.into()],
            )
            .map_err(|error| {
                format!("EC_DELETE_ROUTE: source mapping refresh append failed: {error}")
            })?;
        Ok::<(), String>(())
    })
}

/// Tombstone one current owner-local graph record. This is idempotent: a retry
/// after a lost routed response observes the already-set flag and succeeds.
pub(crate) unsafe fn tombstone_owner_record(
    index_oid: pg_sys::Oid,
    vec_id: u64,
    epoch_fingerprint: Option<[u8; 34]>,
) -> Result<bool, String> {
    let scan = match epoch_fingerprint {
        Some(fingerprint) => {
            PhysicalGenerationScan::open_at_fingerprint_for_owner_insert(index_oid, fingerprint)?
        }
        None => PhysicalGenerationScan::open(index_oid)?,
    };
    let (generation, descriptor) = scan.local_write_identity()?;
    let graph_name = qualified_relation_name(generation.graph_store_relid)?;
    let code_binding = DistannCodecBinding::from_artifact(&descriptor.codec_artifact)?;
    let code_len = code_binding.code_len(usize::from(descriptor.dimensions))?;
    let signed_id = i64::from_le_bytes(vec_id.to_le_bytes());
    let row = Spi::connect_mut(|client| {
        client
            .select(
                &format!(
                    "SELECT graph_record, ctid FROM {graph_name} \
                     WHERE vec_id = $1 AND is_current FOR UPDATE"
                ),
                None,
                &[signed_id.into()],
            )
            .map_err(|error| format!("EC_DELETE_ROUTE: owner tombstone lookup failed: {error}"))?
            .map(|row| {
                let record = row["graph_record"]
                    .value::<Vec<u8>>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "graph_record is NULL".to_owned())?;
                let graph_tid = row["ctid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "ctid is NULL".to_owned())?;
                Ok::<_, String>((record, graph_tid))
            })
            .next()
            .transpose()
    })?;
    let Some((record, graph_tid)) = row else {
        return Err(format!(
            "EC_RECORD_MISSING: physical owner has no current vec_id {vec_id:#018x}"
        ));
    };
    let mut node =
        DistannNodeTuple::decode_physical_v1(&record, descriptor.graph_degree, code_len)?;
    if node.tombstoned {
        return Ok(false);
    }
    node.tombstoned = true;
    let encoded = node.encode_physical_v1(descriptor.graph_degree, code_len)?;
    let (block, offset) = pgrx::itemptr::item_pointer_get_both(graph_tid);
    let updated = Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "UPDATE {graph_name} SET graph_record = $1 \
                     WHERE ctid = '({block},{offset})'::tid AND is_current"
                ),
                None,
                &[encoded.into()],
            )
            .map_err(|error| format!("EC_DELETE_ROUTE: owner tombstone update failed: {error}"))
            .map(|table| table.len())
    })?;
    if updated != 1 {
        return Err(format!(
            "EC_DELETE_ROUTE: owner tombstone affected {updated} rows, expected one"
        ));
    }
    Ok(true)
}

/// Resolve VACUUM's callback TIDs through the coordinator-side physical source
/// map and route each dead stable vec_id to its hash owner. The control index
/// itself intentionally contains no graph tuples, so this directory is the
/// durable enumeration surface for distributed-control maintenance.
pub(crate) unsafe fn tombstone_dead_records(
    index_relation: pg_sys::Relation,
    callback: pg_sys::IndexBulkDeleteCallback,
    callback_state: *mut std::ffi::c_void,
) -> Result<u64, String> {
    let Some(callback) = callback else {
        return Ok(0);
    };
    let index_oid = (*index_relation).rd_id;
    let scan = PhysicalGenerationScan::open(index_oid)?;
    let (_logical_uuid, _build_id, fingerprint_ref, routed_descriptor, routes_ref) =
        scan.traversal_replica_source();
    let fingerprint = fingerprint_ref;
    let placement_hash_version = routed_descriptor.placement_hash_version;
    let routes = routes_ref.to_owned();
    let roster_spec = roster_spec_for_routes(&routes)?;
    let metadata = ambuild::read_metadata_from_index(index_relation)?;
    let epoch = super::roster::scan_epoch(&metadata);
    let source_map =
        super::generation_catalog::extension_relation_name("ec_distann_physical_source_map")?;
    let local_owner = routes
        .iter()
        .position(|route| route.is_local)
        .ok_or_else(|| "EC_NODE_DESCRIPTOR: local owner is absent from roster".to_owned())?;
    let mappings = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT source_tid, vec_id FROM {source_map} \
                     WHERE index_oid = $1::oid ORDER BY source_tid"
                ),
                None,
                &[index_oid.into()],
            )
            .map_err(|error| format!("EC_DELETE_ROUTE: source map scan failed: {error}"))?
            .map(|row| {
                let source_tid = row["source_tid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "source_tid is NULL".to_owned())?;
                let vec_id = row["vec_id"]
                    .value::<i64>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "vec_id is NULL".to_owned())?;
                Ok::<_, String>((source_tid, u64::from_le_bytes(vec_id.to_le_bytes())))
            })
            .collect::<Result<Vec<_>, _>>()
    })?;
    drop(scan);
    let mut removed = 0_u64;
    for (source_tid, vec_id) in mappings {
        let mut raw_tid = source_tid;
        if !callback(&mut raw_tid, callback_state) {
            continue;
        }
        let owner = super::placement::owning_node(vec_id, routes.len(), placement_hash_version);
        let route = routes
            .get(owner)
            .ok_or_else(|| "EC_NODE_DESCRIPTOR: delete owner is outside roster".to_owned())?;
        if owner == local_owner {
            // Do not reclaim the source tuple when the owner update fails. The
            // source-map row is the only durable retry token for this routed
            // delete, so VACUUM must fail and leave the callback retryable
            // rather than silently orphaning the map.
            if tombstone_owner_record(index_oid, vec_id, None)? {
                removed += 1;
            }
            let (block, offset) = pgrx::itemptr::item_pointer_get_both(source_tid);
            delete_source_mapping(
                index_oid,
                ItemPointer {
                    block_number: block,
                    offset_number: offset,
                },
                vec_id,
            )?;
        } else {
            let conninfo = route.conninfo.as_deref().ok_or_else(|| {
                "EC_NODE_DESCRIPTOR: remote delete route has no conninfo".to_owned()
            })?;
            let index_name = super::routine::distann_index_relname(index_relation);
            super::remote_transport::remote_physical_tombstone(
                &super::remote_transport::DistannRemotePhysicalTombstoneRequest {
                    conninfo,
                    roster_spec: &roster_spec,
                    target_node_id: route.node_id,
                    epoch,
                    index_regclass: &index_name,
                    epoch_fingerprint: &fingerprint,
                    vec_id,
                },
            )?;
            removed += 1;
            let (block, offset) = pgrx::itemptr::item_pointer_get_both(source_tid);
            delete_source_mapping(
                index_oid,
                ItemPointer {
                    block_number: block,
                    offset_number: offset,
                },
                vec_id,
            )?;
        }
    }
    Ok(removed)
}

/// The payload decoder needs a row-tier descriptor. This helper is kept
/// separate from the write lock acquisition so remote planning cannot hold a
/// write lock while it performs owner RPCs.
fn row_relation_for_payload(
    generation: &super::generation_catalog::GenerationCatalogRow,
) -> Result<HeapRelationGuard, String> {
    HeapRelationGuard::try_access_share(generation.row_tier_relid).ok_or_else(|| {
        "EC_GENERATION_MISSING: row tier could not be opened for remote payload planning".to_owned()
    })
}

/// Remote DML must carry the immutable scan roster even when the coordinator
/// session did not have the operator roster GUC installed.  The coordinator
/// scan has already resolved every route from the published participant
/// bindings, so serialize that resolved topology for the owner session.
fn roster_spec_for_routes(
    routes: &[super::generation_read::PhysicalOwnerRoute],
) -> Result<String, String> {
    let configured = super::roster::current_roster_spec();
    if !configured.trim().is_empty() {
        return Ok(configured);
    }
    routes
        .iter()
        .map(|route| Ok(format!("{}@{}", route.node_id, route.roster_conninfo)))
        .collect::<Result<Vec<_>, String>>()
        .map(|entries| entries.join(";"))
}

unsafe fn materialize_remote_vectors(
    scan: &PhysicalGenerationScan,
    row_relation: &HeapRelationGuard,
    remote_ids: &[u64],
    source_attnum: i32,
) -> Result<HashMap<u64, Vec<f32>>, String> {
    if remote_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let projection = scan
        .row_schema_attributes()
        .into_iter()
        .map(|attnum| {
            i16::try_from(attnum).map_err(|_| {
                "EC_SCHEMA_MISMATCH: row attribute number exceeds AttrNumber".to_owned()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let payloads = scan.materialize_remote_payload_ids(remote_ids, &projection)?;
    let slot = TupleTableSlotGuard::create_for_heap_guard(row_relation).ok_or_else(|| {
        "EC_GENERATION_MISSING: remote payload slot could not be allocated".to_owned()
    })?;
    let mut vectors = HashMap::with_capacity(remote_ids.len());
    for vec_id in remote_ids {
        let payload = payloads.get(vec_id).ok_or_else(|| {
            format!("EC_GENERATION_MISSING: remote payload missing for vec_id {vec_id:#018x}")
        })?;
        decode_packed_row(
            slot.as_ptr(),
            &payload.payload_nulls,
            &payload.payload_offsets,
            &payload.payload_values,
        )?;
        let mut is_null = false;
        let datum = pg_sys::slot_getattr(slot.as_ptr(), source_attnum, &mut is_null);
        if is_null {
            return Err(format!(
                "EC_SCHEMA_MISMATCH: remote candidate vector is NULL for vec_id {vec_id:#018x}"
            ));
        }
        vectors.insert(*vec_id, ecvector_datum_to_vec(datum));
    }
    Ok(vectors)
}

/// Legacy owner endpoint adapter retained for fixture compatibility. The
/// production transport uses `insert_from_owner_payload`, which never assumes
/// that the coordinator's source heap is mirrored on the owner.
pub(crate) unsafe fn insert_from_owner_source(
    index_oid: pg_sys::Oid,
    expected_vec_id: u64,
    source_tid: ItemPointer,
) -> Result<(), String> {
    let index_guard = IndexRelationGuard::open(
        index_oid,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        "ec_distann physical owner insert",
    );
    let index_relation = index_guard.as_ptr();
    let heap_oid = index_guard.heap_relation_oid();
    let heap_guard =
        HeapRelationGuard::try_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
            .ok_or_else(|| {
                "EC_GENERATION_MISSING: owner source heap could not be opened".to_owned()
            })?;
    let slot = TupleTableSlotGuard::single_for_heap_guard(&heap_guard).ok_or_else(|| {
        "EC_GENERATION_MISSING: owner source slot could not be allocated".to_owned()
    })?;
    let snapshot = pg_sys::GetActiveSnapshot();
    if snapshot.is_null() {
        return Err("EC_BUILD_STATE: physical owner insert has no active snapshot".to_owned());
    }
    let mut raw_tid = pg_sys::ItemPointerData::default();
    pgrx::itemptr::item_pointer_set_all(
        &mut raw_tid,
        source_tid.block_number,
        source_tid.offset_number,
    );
    if !pg_sys::table_tuple_fetch_row_version(
        heap_guard.as_ptr(),
        &mut raw_tid,
        snapshot,
        slot.as_ptr(),
    ) {
        return Err("EC_SOURCE_SNAPSHOT: owner source ctid is not visible".to_owned());
    }
    pg_sys::slot_getallattrs(slot.as_ptr());
    let index_info = pg_sys::BuildIndexInfo(index_relation);
    if index_info.is_null() {
        return Err("EC_SCHEMA_MISMATCH: owner insert could not build index metadata".to_owned());
    }
    let attr_count = usize::try_from((*index_info).ii_NumIndexAttrs)
        .map_err(|_| "EC_SCHEMA_MISMATCH: owner index attribute count is negative".to_owned())?;
    let mut values = Vec::with_capacity(attr_count);
    let mut nulls = Vec::with_capacity(attr_count);
    for offset in 0..attr_count {
        let attnum = *(*index_info)
            .ii_IndexAttrNumbers
            .get(offset)
            .ok_or_else(|| {
                "EC_SCHEMA_MISMATCH: owner index attribute offset is out of bounds".to_owned()
            })?;
        if attnum <= 0 {
            return Err(
                "EC_SCHEMA_MISMATCH: owner index attribute is not a heap column".to_owned(),
            );
        }
        let mut is_null = false;
        let datum = pg_sys::slot_getattr(slot.as_ptr(), i32::from(attnum), &mut is_null);
        values.push(datum);
        nulls.push(is_null);
    }
    let identity = ambuild::resolve_identity_attribute(
        heap_guard.as_ptr(),
        index_info,
        &options::relation_options(index_relation),
    )
    .ok_or_else(|| {
        "EC_SOURCE_IDENTITY: owner insert requires source_identity='include'".to_owned()
    })?;
    let identity_payload =
        ambuild::extract_identity_payload(identity, values.as_mut_ptr(), nulls.as_mut_ptr());
    let actual_vec_id = super::identity::vec_id_from_source_identity(&identity_payload);
    if actual_vec_id != expected_vec_id {
        pg_sys::pfree(index_info.cast());
        return Err(format!(
            "EC_SOURCE_IDENTITY: owner source identity hashes to {actual_vec_id:#018x}, expected {expected_vec_id:#018x}"
        ));
    }
    let mut callback_tid = raw_tid;
    let result = insert_from_callback(
        index_relation,
        heap_guard.as_ptr(),
        values.as_mut_ptr(),
        nulls.as_mut_ptr(),
        &mut callback_tid,
        index_info,
        false,
    );
    pg_sys::pfree(index_info.cast());
    result
}

/// Owner endpoint path for a complete frozen row payload.  The packed payload
/// is decoded against the owner generation's immutable row schema before any
/// row or graph relation is touched; malformed or non-canonical input fails
/// closed.
pub(crate) unsafe fn insert_from_owner_payload(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: [u8; 34],
    expected_vec_id: u64,
    source_vector: Vec<f32>,
    source_identity: &[u8],
    payload_nulls: &[bool],
    payload_offsets: &[i64],
    payload_values: &[u8],
    planned_forward_payload: &[u8],
    allow_replacement: bool,
) -> Result<(), String> {
    let identity: [u8; 16] = source_identity
        .try_into()
        .map_err(|_| "EC_SOURCE_IDENTITY: owner payload identity must be 16 bytes".to_owned())?;
    let actual_vec_id = super::identity::vec_id_from_source_identity(&identity);
    if actual_vec_id != expected_vec_id {
        return Err("EC_SOURCE_IDENTITY: owner payload identity does not match vec_id".to_owned());
    }
    let index_guard = IndexRelationGuard::open(
        index_oid,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        "ec_distann physical owner payload insert",
    );
    let scan =
        PhysicalGenerationScan::open_at_fingerprint_for_owner_insert(index_oid, epoch_fingerprint)?;
    let (generation, _descriptor) = scan.local_write_identity()?;
    drop(scan);
    let row_relation = HeapRelationGuard::try_open(
        generation.row_tier_relid,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
    )
    .ok_or_else(|| "EC_GENERATION_MISSING: owner row tier could not be opened".to_owned())?;
    let slot = TupleTableSlotGuard::create_for_heap_guard(&row_relation)
        .ok_or_else(|| "EC_GENERATION_MISSING: owner row slot could not be allocated".to_owned())?;
    decode_packed_row(
        slot.as_ptr(),
        payload_nulls,
        payload_offsets,
        payload_values,
    )?;
    let vector_attnum = indexed_ecvector_attnum(index_guard.as_ptr())?;
    let mut is_null = false;
    let datum = pg_sys::slot_getattr(slot.as_ptr(), vector_attnum, &mut is_null);
    if is_null || ecvector_datum_to_vec(datum) != source_vector {
        return Err(
            "EC_SCHEMA_MISMATCH: owner payload vector differs from graph insert vector".to_owned(),
        );
    }
    let source_heap = HeapRelationGuard::try_open(
        index_guard.heap_relation_oid(),
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
    )
    .ok_or_else(|| "EC_GENERATION_MISSING: source heap could not be opened".to_owned())?;
    let index_info = pg_sys::BuildIndexInfo(index_guard.as_ptr());
    if index_info.is_null() {
        return Err("EC_SCHEMA_MISMATCH: owner payload identity metadata is missing".to_owned());
    }
    let identity_attribute = ambuild::resolve_identity_attribute(
        source_heap.as_ptr(),
        index_info,
        &options::relation_options(index_guard.as_ptr()),
    )
    .ok_or_else(|| {
        pg_sys::pfree(index_info.cast());
        "EC_SOURCE_IDENTITY: owner payload requires source_identity='include'".to_owned()
    })?;
    pg_sys::pfree(index_info.cast());
    insert_from_prepared_slot(
        index_guard.as_ptr(),
        source_vector,
        expected_vec_id,
        identity,
        allow_replacement,
        identity_attribute,
        slot.as_ptr(),
        ItemPointer::INVALID,
        Some(epoch_fingerprint),
        Some(planned_forward_payload),
    )
}

unsafe fn decode_packed_row(
    slot: *mut pg_sys::TupleTableSlot,
    nulls: &[bool],
    offsets: &[i64],
    packed: &[u8],
) -> Result<(), String> {
    if slot.is_null() || nulls.len() != offsets.len() {
        return Err("EC_HANDOFF_FORMAT: packed source row shape mismatch".to_owned());
    }
    let tuple_desc = (*slot).tts_tupleDescriptor;
    let natts = usize::try_from((*tuple_desc).natts)
        .map_err(|_| "EC_SCHEMA_MISMATCH: row-tier attribute count is negative".to_owned())?;
    let non_dropped = (0..natts)
        .filter(|index| !(*pg_sys::TupleDescAttr(tuple_desc, *index as i32)).attisdropped)
        .count();
    if nulls.len() != non_dropped {
        return Err("EC_HANDOFF_FORMAT: packed source row attribute count mismatch".to_owned());
    }
    let mut writer = TupleSlotWriter::from_raw_slot(slot, "ec_distann physical payload")?;
    writer.clear();
    let mut position = 0_usize;
    let mut previous_end = 0_usize;
    for attribute_index in 0..natts {
        let attribute = pg_sys::TupleDescAttr(tuple_desc, attribute_index as i32);
        if (*attribute).attisdropped {
            writer.set_null(attribute_index as i32);
            continue;
        }
        if nulls[position] {
            writer.set_null(attribute_index as i32);
            position += 1;
            continue;
        }
        let end = usize::try_from(offsets[position])
            .map_err(|_| "EC_HANDOFF_FORMAT: packed source row offset is negative".to_owned())?;
        if end < previous_end || end > packed.len() {
            return Err("EC_HANDOFF_FORMAT: packed source row offsets are invalid".to_owned());
        }
        let bytes = &packed[previous_end..end];
        let mut receive_oid = pg_sys::InvalidOid;
        let mut typioparam = pg_sys::InvalidOid;
        pg_sys::getTypeBinaryInputInfo((*attribute).atttypid, &mut receive_oid, &mut typioparam);
        if receive_oid == pg_sys::InvalidOid {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: row-tier attribute {} lacks binary input",
                attribute_index + 1
            ));
        }
        let mut flinfo = std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init();
        pg_sys::fmgr_info(receive_oid, &mut flinfo);
        let len = core::ffi::c_int::try_from(bytes.len())
            .map_err(|_| "EC_HANDOFF_TOO_LARGE: row attribute exceeds int".to_owned())?;
        let mut input_bytes = bytes.to_vec();
        let mut input = pg_sys::StringInfoData {
            data: input_bytes.as_mut_ptr().cast(),
            len,
            maxlen: len,
            cursor: 0,
        };
        let datum = pg_sys::ReceiveFunctionCall(
            &mut flinfo,
            &mut input,
            typioparam,
            (*attribute).atttypmod,
        );
        // PostgreSQL's receive functions return a palloc-owned datum.  The
        // input buffer is intentionally iteration-local; no receive function
        // may retain a pointer into it after ReceiveFunctionCall returns.
        if input.cursor != input.len {
            return Err(format!(
                "EC_HANDOFF_FORMAT: row attribute {} binary input left bytes",
                attribute_index + 1
            ));
        }
        writer.set_datum(attribute_index as i32, datum);
        previous_end = end;
        position += 1;
    }
    if previous_end != packed.len() {
        return Err("EC_HANDOFF_FORMAT: packed source row has trailing bytes".to_owned());
    }
    writer.store_virtual_tuple()?;
    Ok(())
}

struct Candidate {
    vec_id: u64,
    source_vector: Vec<f32>,
    node: DistannNodeTuple,
    graph_tid: ItemPointer,
}

const PLANNED_FORWARD_MAGIC: &[u8; 4] = b"EPI1";

/// Serialize the coordinator's selected forward plan for a participant
/// owner.  The participant cannot search the coordinator-resident head, so
/// it receives only the bounded, already-pruned `(vec_id, source_vector)`
/// set.  Owner-local graph records are re-read by vec_id before mutation.
fn encode_planned_forward(forward: &[&Candidate]) -> Result<Vec<u8>, String> {
    let count = u32::try_from(forward.len())
        .map_err(|_| "EC_HANDOFF_FORMAT: planned forward count exceeds u32".to_owned())?;
    let dimensions = forward
        .first()
        .map(|candidate| candidate.source_vector.len())
        .unwrap_or(0);
    for candidate in forward {
        if candidate.source_vector.len() != dimensions {
            return Err("EC_HANDOFF_FORMAT: planned forward dimensions disagree".to_owned());
        }
    }
    let dimensions = u32::try_from(dimensions)
        .map_err(|_| "EC_HANDOFF_FORMAT: planned forward dimensions exceed u32".to_owned())?;
    let mut payload = Vec::with_capacity(12 + forward.len() * (8 + 4 + dimensions as usize * 4));
    payload.extend_from_slice(PLANNED_FORWARD_MAGIC);
    payload.extend_from_slice(&count.to_le_bytes());
    payload.extend_from_slice(&dimensions.to_le_bytes());
    for candidate in forward {
        payload.extend_from_slice(&candidate.vec_id.to_le_bytes());
        for value in &candidate.source_vector {
            if !value.is_finite() {
                return Err("EC_SCHEMA_MISMATCH: planned forward vector is not finite".to_owned());
            }
            payload.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok(payload)
}

fn decode_planned_forward(
    payload: &[u8],
    expected_dimensions: usize,
    graph_degree: u16,
) -> Result<Vec<(u64, Vec<f32>)>, String> {
    if payload.len() < 12 || &payload[..4] != PLANNED_FORWARD_MAGIC {
        return Err("EC_HANDOFF_FORMAT: planned forward payload has an invalid header".to_owned());
    }
    let count = u32::from_le_bytes(payload[4..8].try_into().unwrap()) as usize;
    let dimensions = u32::from_le_bytes(payload[8..12].try_into().unwrap()) as usize;
    if count > usize::from(graph_degree) {
        return Err("EC_HANDOFF_FORMAT: planned forward exceeds graph degree".to_owned());
    }
    if dimensions != expected_dimensions {
        return Err(format!(
            "EC_SCHEMA_MISMATCH: planned forward has {dimensions} dimensions, expected {expected_dimensions}"
        ));
    }
    let entry_bytes =
        8usize
            .checked_add(dimensions.checked_mul(4).ok_or_else(|| {
                "EC_HANDOFF_FORMAT: planned forward byte length overflow".to_owned()
            })?)
            .ok_or_else(|| "EC_HANDOFF_FORMAT: planned forward byte length overflow".to_owned())?;
    let expected_len =
        12usize
            .checked_add(count.checked_mul(entry_bytes).ok_or_else(|| {
                "EC_HANDOFF_FORMAT: planned forward byte length overflow".to_owned()
            })?)
            .ok_or_else(|| "EC_HANDOFF_FORMAT: planned forward byte length overflow".to_owned())?;
    if payload.len() != expected_len {
        return Err("EC_HANDOFF_FORMAT: planned forward payload length mismatch".to_owned());
    }
    let mut position = 12;
    let mut decoded = Vec::with_capacity(count);
    for _ in 0..count {
        let vec_id = u64::from_le_bytes(payload[position..position + 8].try_into().unwrap());
        position += 8;
        let mut vector = Vec::with_capacity(dimensions);
        for _ in 0..dimensions {
            let value = f32::from_le_bytes(payload[position..position + 4].try_into().unwrap());
            if !value.is_finite() {
                return Err("EC_SCHEMA_MISMATCH: planned forward vector is not finite".to_owned());
            }
            vector.push(value);
            position += 4;
        }
        decoded.push((vec_id, vector));
    }
    Ok(decoded)
}

unsafe fn fetch_source_tuple(
    heap_relation: pg_sys::Relation,
    heap_tid: pg_sys::ItemPointer,
    slot: *mut pg_sys::TupleTableSlot,
    snapshot: pg_sys::Snapshot,
) -> Result<(), String> {
    let mut tid = pg_sys::ItemPointerData::default();
    let decoded = decode_heap_tid(heap_tid);
    pgrx::itemptr::item_pointer_set_all(&mut tid, decoded.block_number, decoded.offset_number);
    pg_sys::ExecClearTuple(slot);
    if !pg_sys::table_tuple_fetch_row_version(heap_relation, &mut tid, snapshot, slot) {
        return Err("EC_SOURCE_SNAPSHOT: inserted source row is not visible".to_owned());
    }
    Ok(())
}

unsafe fn source_slot_is_updated(slot: *mut pg_sys::TupleTableSlot) -> bool {
    if slot.is_null() {
        return false;
    }
    let mut should_free = false;
    let tuple = pg_sys::ExecFetchSlotHeapTuple(slot, false, &mut should_free);
    if tuple.is_null() || (*tuple).t_data.is_null() {
        return false;
    }
    let updated = ((*(*tuple).t_data).t_infomask as u32 & pg_sys::HEAP_UPDATED) != 0;
    if should_free {
        pg_sys::heap_freetuple(tuple);
    }
    updated
}

fn read_current_candidate(
    graph_name: &str,
    row_relation: &HeapRelationGuard,
    hit: DistannScanHit,
    graph_degree: u16,
    code_len: usize,
    source_attnum: i32,
    snapshot: pg_sys::Snapshot,
) -> Result<Option<Candidate>, String> {
    let signed_id = i64::from_le_bytes(hit.vec_id.to_le_bytes());
    let row = Spi::connect(|client| {
        client.select(
            &format!("SELECT graph_record, row_tid, ctid FROM {graph_name} WHERE vec_id = $1 AND is_current"),
            None,
            &[signed_id.into()],
        ).map_err(|error| format!("EC_INSERT_SEARCH: graph candidate lookup failed: {error}"))?
        .map(|row| {
            let record = row["graph_record"].value::<Vec<u8>>().map_err(|error| error.to_string())?.ok_or_else(|| "graph_record is NULL".to_owned())?;
            let row_tid = row["row_tid"].value::<pg_sys::ItemPointerData>().map_err(|error| error.to_string())?.ok_or_else(|| "row_tid is NULL".to_owned())?;
            let graph_tid = row["ctid"].value::<pg_sys::ItemPointerData>().map_err(|error| error.to_string())?.ok_or_else(|| "ctid is NULL".to_owned())?;
            Ok::<_, String>((record, row_tid, graph_tid))
        }).next().transpose()
    })?;
    let Some((record, row_tid, graph_tid)) = row else {
        return Ok(None);
    };
    let node = DistannNodeTuple::decode_physical_v1(&record, graph_degree, code_len)?;
    let vector_slot =
        TupleTableSlotGuard::single_for_heap_guard(row_relation).ok_or_else(|| {
            "EC_GENERATION_MISSING: row-tier vector slot could not be allocated".to_owned()
        })?;
    unsafe { pg_sys::ExecClearTuple(vector_slot.as_ptr()) };
    let mut raw_tid = pg_sys::ItemPointerData::default();
    let (row_block, row_offset) = pgrx::itemptr::item_pointer_get_both(row_tid);
    pgrx::itemptr::item_pointer_set_all(&mut raw_tid, row_block, row_offset);
    let mut visible = unsafe {
        pg_sys::table_tuple_fetch_row_version(
            row_relation.as_ptr(),
            &mut raw_tid,
            snapshot,
            vector_slot.as_ptr(),
        )
    };
    if !visible {
        if let Some(latest) = RegisteredSnapshotGuard::latest() {
            let (row_block, row_offset) = pgrx::itemptr::item_pointer_get_both(row_tid);
            pgrx::itemptr::item_pointer_set_all(&mut raw_tid, row_block, row_offset);
            unsafe { pg_sys::ExecClearTuple(vector_slot.as_ptr()) };
            visible = unsafe {
                pg_sys::table_tuple_fetch_row_version(
                    row_relation.as_ptr(),
                    &mut raw_tid,
                    latest.as_ptr(),
                    vector_slot.as_ptr(),
                )
            };
        }
    }
    if !visible {
        return Err(format!(
            "EC_GENERATION_MISSING: candidate row tier is absent for vec_id {:#018x}",
            hit.vec_id
        ));
    }
    let mut is_null = false;
    let datum = unsafe { pg_sys::slot_getattr(vector_slot.as_ptr(), source_attnum, &mut is_null) };
    if is_null {
        return Err("EC_GENERATION_MISSING: candidate source vector is NULL".to_owned());
    }
    let source_vector = unsafe { ecvector_datum_to_vec(datum) };
    let (graph_block, graph_offset) = pgrx::itemptr::item_pointer_get_both(graph_tid);
    Ok(Some(Candidate {
        vec_id: hit.vec_id,
        source_vector,
        node,
        graph_tid: ItemPointer {
            block_number: graph_block,
            offset_number: graph_offset,
        },
    }))
}

fn append_row_tuple(
    row_relation: &HeapRelationGuard,
    source_slot: *mut pg_sys::TupleTableSlot,
) -> Result<ItemPointer, String> {
    let slot = TupleTableSlotGuard::create_for_heap_guard(row_relation).ok_or_else(|| {
        "EC_GENERATION_MISSING: row-tier insert slot could not be allocated".to_owned()
    })?;
    let mut writer =
        unsafe { TupleSlotWriter::from_raw_slot(slot.as_ptr(), "ec_distann physical insert")? };
    writer.clear();
    let natts = writer.natts();
    writer.validate_input_width(
        usize::try_from(natts).map_err(|_| "row tier width is negative".to_owned())?,
    )?;
    unsafe { pg_sys::slot_getallattrs(source_slot) };
    for attr in 0..natts {
        let index =
            usize::try_from(attr).map_err(|_| "row tier attribute index overflow".to_owned())?;
        if unsafe { *(*source_slot).tts_isnull.add(index) } {
            writer.set_null(attr);
        } else {
            writer.set_datum(attr, unsafe { *(*source_slot).tts_values.add(index) });
        }
    }
    let stored = writer.store_virtual_tuple()?;
    unsafe { pg_sys::simple_table_tuple_insert(row_relation.as_ptr(), stored) };
    let tid = unsafe { (*stored).tts_tid };
    let (block_number, offset_number) = pgrx::itemptr::item_pointer_get_both(tid);
    Ok(ItemPointer {
        block_number,
        offset_number,
    })
}

fn current_record_version(graph_name: &str, vec_id: u64) -> Result<Option<i64>, String> {
    let signed_id = i64::from_le_bytes(vec_id.to_le_bytes());
    Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT record_version FROM {graph_name} WHERE vec_id = $1 AND is_current"
                ),
                None,
                &[signed_id.into()],
            )
            .map_err(|error| {
                format!("EC_INSERT_PUBLISH: current directory lookup failed: {error}")
            })?
            .map(|row| {
                row["record_version"]
                    .value::<i64>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "record_version is NULL".to_owned())
            })
            .next()
            .transpose()
    })
}

unsafe fn current_record_identity(
    graph_name: &str,
    row_relation: &HeapRelationGuard,
    vec_id: u64,
    identity: ambuild::DistannIdentityAttribute,
    snapshot: pg_sys::Snapshot,
) -> Result<Option<[u8; 16]>, String> {
    let signed_id = i64::from_le_bytes(vec_id.to_le_bytes());
    let row_tid = Spi::connect(|client| {
        client
            .select(
                &format!("SELECT row_tid FROM {graph_name} WHERE vec_id = $1 AND is_current"),
                None,
                &[signed_id.into()],
            )
            .map_err(|error| format!("EC_INSERT_PUBLISH: identity lookup failed: {error}"))?
            .map(|row| {
                row["row_tid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "row_tid is NULL".to_owned())
            })
            .next()
            .transpose()
    })?;
    let Some(row_tid) = row_tid else {
        return Ok(None);
    };
    let slot = TupleTableSlotGuard::create_for_heap_guard(row_relation).ok_or_else(|| {
        "EC_GENERATION_MISSING: identity lookup row slot could not be allocated".to_owned()
    })?;
    let mut raw_tid = row_tid;
    if !pg_sys::table_tuple_fetch_row_version(
        row_relation.as_ptr(),
        &mut raw_tid,
        snapshot,
        slot.as_ptr(),
    ) {
        return Ok(None);
    }
    let mut is_null = false;
    let datum = pg_sys::slot_getattr(slot.as_ptr(), identity.heap_attnum, &mut is_null);
    if is_null {
        return Err("EC_SOURCE_IDENTITY: current row identity is NULL".to_owned());
    }
    Ok(Some(ambuild::identity_payload_from_datum(
        identity.datum_kind,
        datum,
    )))
}

fn insert_graph_record(
    graph_name: &str,
    vec_id: u64,
    record: &[u8],
    row_tid: ItemPointer,
    version: i64,
) -> Result<(), String> {
    let signed_id = i64::from_le_bytes(vec_id.to_le_bytes());
    let mut row_tid_data = pg_sys::ItemPointerData::default();
    pgrx::itemptr::item_pointer_set_all(
        &mut row_tid_data,
        row_tid.block_number,
        row_tid.offset_number,
    );
    let row_tids = vec![row_tid_data];
    let inserted = Spi::connect_mut(|client| {
        client
            .update(
                &format!("INSERT INTO {graph_name} (vec_id, graph_record, row_tid, record_version, is_current) SELECT * FROM unnest($1::bigint[], $2::bytea[], $3::tid[], $4::bigint[], $5::bool[])"),
                None,
                &[vec![signed_id].into(), vec![record.to_vec()].into(), row_tids.into(), vec![version].into(), vec![true].into()],
            )
            .map_err(|error| format!("EC_INSERT_PUBLISH: graph record append failed: {error}"))
            .map(|table| table.len())
    })?;
    if inserted != 1 {
        return Err(format!(
            "EC_INSERT_PUBLISH: graph record append affected {inserted} rows"
        ));
    }
    Ok(())
}

unsafe fn amend_backlink(
    graph_name: &str,
    candidate: &Candidate,
    new_vec_id: u64,
    new_source_vector: &[f32],
    new_code: &[u8],
    graph_degree: u16,
    code_len: usize,
    row_relation: &HeapRelationGuard,
    source_attnum: i32,
    alpha: f32,
    remote_vectors: &HashMap<u64, Vec<f32>>,
) -> Result<(), String> {
    // Lock and re-read the current target before computing the replacement.
    // A concurrent backlink amendment or tombstone may have replaced the
    // tuple since the scan planned this candidate; updating from the stale
    // candidate would lose the other writer or resurrect a tombstone.
    let (current_record, graph_tid) =
        current_graph_record_for_update(graph_name, candidate.vec_id)?.ok_or_else(|| {
            format!(
                "EC_INSERT_BACKLINK: current target vec_id {:#018x} is missing",
                candidate.vec_id
            )
        })?;
    let mut node = DistannNodeTuple::decode_physical_v1(&current_record, graph_degree, code_len)?;
    // FOR UPDATE may wait for a concurrent backlink transaction and then
    // return its newer graph version. Refresh the snapshot before fetching
    // the row-tier vectors referenced by that version; the original insert
    // snapshot can predate the transaction whose graph edge we just locked.
    let latest_snapshot = RegisteredSnapshotGuard::latest()
        .ok_or_else(|| "EC_BUILD_STATE: backlink could not acquire a latest snapshot".to_owned())?;
    let count = usize::from(node.neighbor_count);
    if node.neighbor_vec_ids[..count].contains(&new_vec_id) {
        stage_counters::record_insert_work(DistannInsertWork::BacklinkAlreadyPresent, 1);
        return Ok(());
    }
    // This is a normal userset diagnostic GUC, so the production build must
    // honor it too.  Keeping the branch behind `pg_test` would make the
    // production A/B control a no-op while still advertising the setting.
    let append_when_room = !options::debug_disable_append_when_room();
    if count < usize::from(graph_degree) && append_when_room {
        if new_code.len() != code_len {
            return Err("EC_INSERT_CODEC: backlink code has the wrong length".to_owned());
        }
        node.neighbor_vec_ids[count] = new_vec_id;
        node.neighbor_codes[count * code_len..(count + 1) * code_len].copy_from_slice(new_code);
        node.neighbor_count = node
            .neighbor_count
            .checked_add(1)
            .ok_or_else(|| "EC_INSERT_BACKLINK: backlink degree overflow".to_owned())?;
        let record = node.encode_physical_v1(graph_degree, code_len)?;
        let block = graph_tid.block_number;
        let offset = graph_tid.offset_number;
        let updated = Spi::connect_mut(|client| {
            client
                .update(
                    &format!("UPDATE {graph_name} SET graph_record = $1 WHERE ctid = '({block},{offset})'::tid AND is_current"),
                    None,
                    &[record.into()],
                )
                .map_err(|error| format!("EC_INSERT_BACKLINK: graph amendment failed: {error}"))
                .map(|table| table.len())
        })?;
        if updated != 1 {
            return Err(format!(
                "EC_INSERT_BACKLINK: graph amendment affected {updated} rows, expected one current target"
            ));
        }
        stage_counters::record_insert_work(DistannInsertWork::BacklinkAmendments, 1);
        return Ok(());
    }
    let original_node = node.clone();
    let mut candidates = Vec::with_capacity(count + 1);
    candidates.push(DistannForwardCandidate {
        vec_id: new_vec_id,
        source_vector: new_source_vector.to_vec(),
    });
    for neighbor in &node.neighbor_vec_ids[..count] {
        let source_vector = if let Some(source_vector) = remote_vectors.get(neighbor) {
            source_vector.clone()
        } else {
            let source_vector = read_current_vector(
                graph_name,
                row_relation,
                *neighbor,
                source_attnum,
                latest_snapshot.as_ptr(),
            );
            match source_vector {
                Ok(source_vector) => source_vector,
                Err(error) if is_current_vector_missing(&error) => {
                    // A tombstone or a concurrent replacement can leave a
                    // retained target pointing at a neighbor with no current
                    // row-tier tuple. Drop that stale edge while rebuilding
                    // the target; malformed vectors and other lookup errors
                    // remain hard failures below.
                    continue;
                }
                Err(error) => {
                    return Err(format!(
                        "{error}; backlink_target={:#018x}; missing_neighbor={neighbor:#018x}",
                        candidate.vec_id
                    ));
                }
            }
        };
        candidates.push(DistannForwardCandidate {
            vec_id: *neighbor,
            source_vector,
        });
    }
    // The planner is vantage-point agnostic: reorder the union from the
    // target's source vector using the same exact robust-prune metric as
    // insertion. The shipped strategy uses this for free and full targets;
    // append-when-room remains an explicit diagnostic candidate.
    let kept = select_insert_forward_neighbors(
        &candidate.source_vector,
        &candidates,
        alpha,
        usize::from(graph_degree),
    )?;
    // The local ec_diskann incremental planner treats a pruned backlink as a
    // no-op. Preserve the same contract here: rewriting without the proposed
    // reverse edge can only perturb the established traversal graph. If a
    // concurrent tombstone left a stale neighbor, retain the existing cleanup
    // behavior rather than preserving the malformed slice.
    if count == usize::from(graph_degree)
        && candidates.len() == count + 1
        && !kept.contains(&new_vec_id)
    {
        stage_counters::record_insert_work(DistannInsertWork::BacklinkPruneRejected, 1);
        return Ok(());
    }
    node.neighbor_vec_ids.fill(0);
    node.neighbor_codes.fill(0);
    node.neighbor_count = u16::try_from(kept.len())
        .map_err(|_| "EC_INSERT_BACKLINK: pruned degree exceeds u16".to_owned())?;
    for (slot, vec_id) in kept.iter().enumerate() {
        node.neighbor_vec_ids[slot] = *vec_id;
        let code = if *vec_id == new_vec_id {
            new_code
        } else {
            let old_slot = original_node.neighbor_vec_ids[..count]
                .iter()
                .position(|id| id == vec_id)
                .ok_or_else(|| {
                    "EC_INSERT_BACKLINK: pruned neighbor was absent from current node".to_owned()
                })?;
            &original_node.neighbor_codes[old_slot * code_len..(old_slot + 1) * code_len]
        };
        if code.len() != code_len {
            return Err("EC_INSERT_CODEC: backlink code has the wrong length".to_owned());
        }
        node.neighbor_codes[slot * code_len..(slot + 1) * code_len].copy_from_slice(code);
    }
    let record = node.encode_physical_v1(graph_degree, code_len)?;
    let block = graph_tid.block_number;
    let offset = graph_tid.offset_number;
    let updated = Spi::connect_mut(|client| {
        client
            .update(
                &format!("UPDATE {graph_name} SET graph_record = $1 WHERE ctid = '({block},{offset})'::tid AND is_current"),
                None,
                &[record.into()],
            )
            .map_err(|error| format!("EC_INSERT_BACKLINK: graph amendment failed: {error}"))
            .map(|table| table.len())
    })?;
    if updated != 1 {
        return Err(format!(
            "EC_INSERT_BACKLINK: graph amendment affected {updated} rows, expected one current target"
        ));
    }
    stage_counters::record_insert_work(DistannInsertWork::BacklinkAmendments, 1);
    Ok(())
}

fn current_graph_record_for_update(
    graph_name: &str,
    vec_id: u64,
) -> Result<Option<(Vec<u8>, ItemPointer)>, String> {
    let signed_id = i64::from_le_bytes(vec_id.to_le_bytes());
    Spi::connect_mut(|client| {
        client
            .select(
                &format!(
                    "SELECT graph_record, ctid FROM {graph_name} \
                      WHERE vec_id = $1 AND is_current FOR UPDATE"
                ),
                None,
                &[signed_id.into()],
            )
            .map_err(|error| format!("EC_INSERT_BACKLINK: current target lock failed: {error}"))?
            .map(|row| {
                let record = row["graph_record"]
                    .value::<Vec<u8>>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "graph_record is NULL".to_owned())?;
                let graph_tid = row["ctid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "ctid is NULL".to_owned())?;
                let (block, offset) = pgrx::itemptr::item_pointer_get_both(graph_tid);
                Ok::<_, String>((
                    record,
                    ItemPointer {
                        block_number: block,
                        offset_number: offset,
                    },
                ))
            })
            .next()
            .transpose()
    })
}

/// Conservative owner fallback for a retained graph record that references a
/// source row no longer present in the owner row tier.  The new record's
/// forward edge is already durable; append the reverse edge only when the
/// target has spare degree, otherwise preserve the existing graph exactly.
unsafe fn append_backlink_if_room(
    graph_name: &str,
    target_vec_id: u64,
    new_vec_id: u64,
    new_code: &[u8],
    graph_degree: u16,
    code_len: usize,
) -> Result<(), String> {
    let (current_record, graph_tid) = current_graph_record_for_update(graph_name, target_vec_id)?
        .ok_or_else(|| {
        format!("EC_INSERT_BACKLINK: current target vec_id {target_vec_id:#018x} is missing")
    })?;
    let mut node = DistannNodeTuple::decode_physical_v1(&current_record, graph_degree, code_len)?;
    let count = usize::from(node.neighbor_count);
    if node.neighbor_vec_ids[..count].contains(&new_vec_id) {
        stage_counters::record_insert_work(DistannInsertWork::BacklinkAlreadyPresent, 1);
        return Ok(());
    }
    if count >= usize::from(graph_degree) {
        stage_counters::record_insert_work(DistannInsertWork::BacklinkNoRoom, 1);
        return Ok(());
    }
    if new_code.len() != code_len {
        return Err("EC_INSERT_CODEC: backlink code has the wrong length".to_owned());
    }
    node.neighbor_vec_ids[count] = new_vec_id;
    node.neighbor_codes[count * code_len..(count + 1) * code_len].copy_from_slice(new_code);
    node.neighbor_count = node
        .neighbor_count
        .checked_add(1)
        .ok_or_else(|| "EC_INSERT_BACKLINK: backlink degree overflow".to_owned())?;
    let record = node.encode_physical_v1(graph_degree, code_len)?;
    let block = graph_tid.block_number;
    let offset = graph_tid.offset_number;
    let updated = Spi::connect_mut(|client| {
        client
            .update(
                &format!("UPDATE {graph_name} SET graph_record = $1 WHERE ctid = '({block},{offset})'::tid AND is_current"),
                None,
                &[record.into()],
            )
            .map_err(|error| format!("EC_INSERT_BACKLINK: graph amendment failed: {error}"))
            .map(|table| table.len())
    })?;
    if updated != 1 {
        return Err(format!(
            "EC_INSERT_BACKLINK: graph amendment affected {updated} rows, expected one current target"
        ));
    }
    stage_counters::record_insert_work(DistannInsertWork::BacklinkAmendments, 1);
    Ok(())
}

unsafe fn read_current_vector(
    graph_name: &str,
    row_relation: &HeapRelationGuard,
    vec_id: u64,
    source_attnum: i32,
    snapshot: pg_sys::Snapshot,
) -> Result<Vec<f32>, String> {
    let signed_id = i64::from_le_bytes(vec_id.to_le_bytes());
    let row_tid = Spi::connect(|client| {
        client
            .select(
                &format!("SELECT row_tid FROM {graph_name} WHERE vec_id = $1 AND is_current"),
                None,
                &[signed_id.into()],
            )
            .map_err(|error| format!("EC_INSERT_BACKLINK: neighbor lookup failed: {error}"))?
            .map(|row| {
                row["row_tid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "row_tid is NULL".to_owned())
            })
            .next()
            .transpose()
    })?
    .ok_or_else(|| {
        format!("EC_INSERT_BACKLINK: current row for vec_id {vec_id:#018x} is missing")
    })?;
    let slot = TupleTableSlotGuard::create_for_heap_guard(row_relation).ok_or_else(|| {
        "EC_GENERATION_MISSING: backlink vector slot could not be allocated".to_owned()
    })?;
    let mut raw_tid = row_tid;
    if !pg_sys::table_tuple_fetch_row_version(
        row_relation.as_ptr(),
        &mut raw_tid,
        snapshot,
        slot.as_ptr(),
    ) {
        return Err(format!(
            "EC_INSERT_BACKLINK: current row for vec_id {vec_id:#018x} is not visible"
        ));
    }
    let mut is_null = false;
    let datum = pg_sys::slot_getattr(slot.as_ptr(), source_attnum, &mut is_null);
    if is_null {
        return Err(format!(
            "EC_INSERT_BACKLINK: source vector for vec_id {vec_id:#018x} is NULL"
        ));
    }
    Ok(ecvector_datum_to_vec(datum))
}

fn is_current_vector_missing(error: &str) -> bool {
    error.starts_with("EC_INSERT_BACKLINK: current row for vec_id")
        && error.ends_with(" is missing")
}

/// Apply one owner-local backlink from a coordinator-prepared remote write.
/// The target is looked up by stable vec_id; the caller never supplies a
/// coordinator-local ctid for a remote relation.
pub(crate) unsafe fn apply_owner_backlink(
    index_oid: pg_sys::Oid,
    epoch_fingerprint: [u8; 34],
    target_vec_id: u64,
    target_source_vector: Vec<f32>,
    new_vec_id: u64,
    new_source_vector: Vec<f32>,
    new_code: &[u8],
) -> Result<(), String> {
    let index_guard = IndexRelationGuard::open(
        index_oid,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        "ec_distann physical owner backlink",
    );
    let scan =
        PhysicalGenerationScan::open_at_fingerprint_for_owner_insert(index_oid, epoch_fingerprint)?;
    let (generation, descriptor) = scan.local_write_identity()?;
    let code_binding = DistannCodecBinding::from_artifact(&descriptor.codec_artifact)?;
    if target_source_vector.len() != usize::from(descriptor.dimensions)
        || target_source_vector.iter().any(|value| !value.is_finite())
    {
        return Err(
            "EC_SCHEMA_MISMATCH: backlink target vector has invalid dimensions or values"
                .to_owned(),
        );
    }
    let code_len = code_binding.code_len(usize::from(descriptor.dimensions))?;
    if new_code.len() != code_len {
        return Err("EC_INSERT_CODEC: backlink code has the wrong length".to_owned());
    }
    let graph_name = qualified_relation_name(generation.graph_store_relid)?;
    let row_relation = HeapRelationGuard::try_open(
        generation.row_tier_relid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
    )
    .ok_or_else(|| "EC_GENERATION_MISSING: backlink row tier could not be opened".to_owned())?;
    let source_attnum = indexed_ecvector_attnum(index_guard.as_ptr())?;
    let snapshot = pg_sys::GetActiveSnapshot();
    if snapshot.is_null() {
        return Err("EC_BUILD_STATE: backlink has no active snapshot".to_owned());
    }
    let latest_snapshot = RegisteredSnapshotGuard::latest()
        .ok_or_else(|| "EC_BUILD_STATE: backlink could not acquire a latest snapshot".to_owned())?;
    let signed_id = i64::from_le_bytes(target_vec_id.to_le_bytes());
    let row = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT graph_record, ctid FROM {graph_name} \
                     WHERE vec_id = $1 AND is_current"
                ),
                None,
                &[signed_id.into()],
            )
            .map_err(|error| format!("EC_INSERT_BACKLINK: target lookup failed: {error}"))?
            .map(|row| {
                let record = row["graph_record"]
                    .value::<Vec<u8>>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "graph_record is NULL".to_owned())?;
                let graph_tid = row["ctid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "ctid is NULL".to_owned())?;
                Ok::<_, String>((record, graph_tid))
            })
            .next()
            .transpose()
    })?;
    let Some((record, graph_tid)) = row else {
        return Err(format!(
            "EC_INSERT_BACKLINK: current target vec_id {target_vec_id:#018x} is missing"
        ));
    };
    let node = DistannNodeTuple::decode_physical_v1(&record, descriptor.graph_degree, code_len)?;
    let (_, _, _, routed_descriptor, routes) = scan.traversal_replica_source();
    let local_owner = routes
        .iter()
        .position(|route| route.is_local)
        .ok_or_else(|| "EC_NODE_DESCRIPTOR: active roster has no local owner".to_owned())?;
    let remote_ids = node.neighbor_vec_ids[..usize::from(node.neighbor_count)]
        .iter()
        .copied()
        .filter(|neighbor| {
            super::placement::owning_node(
                *neighbor,
                routes.len(),
                routed_descriptor.placement_hash_version,
            ) != local_owner
        })
        .collect::<Vec<_>>();
    let remote_vectors =
        materialize_remote_vectors(&scan, &row_relation, &remote_ids, source_attnum)?;
    let mut missing_local_neighbor = false;
    for neighbor in &node.neighbor_vec_ids[..usize::from(node.neighbor_count)] {
        let neighbor_owner = super::placement::owning_node(
            *neighbor,
            routes.len(),
            routed_descriptor.placement_hash_version,
        );
        if neighbor_owner == local_owner {
            if let Err(error) = read_current_vector(
                &graph_name,
                &row_relation,
                *neighbor,
                source_attnum,
                latest_snapshot.as_ptr(),
            ) {
                if is_current_vector_missing(&error) {
                    missing_local_neighbor = true;
                    break;
                }
                return Err(error);
            }
        }
    }
    if missing_local_neighbor {
        return append_backlink_if_room(
            &graph_name,
            target_vec_id,
            new_vec_id,
            &new_code,
            descriptor.graph_degree,
            code_len,
        );
    }
    drop(scan);
    let (graph_block, graph_offset) = pgrx::itemptr::item_pointer_get_both(graph_tid);
    amend_backlink(
        &graph_name,
        &Candidate {
            vec_id: target_vec_id,
            source_vector: target_source_vector,
            node,
            graph_tid: ItemPointer {
                block_number: graph_block,
                offset_number: graph_offset,
            },
        },
        new_vec_id,
        &new_source_vector,
        new_code,
        descriptor.graph_degree,
        code_len,
        &row_relation,
        source_attnum,
        options::relation_options(index_guard.as_ptr()).alpha,
        &remote_vectors,
    )
}
