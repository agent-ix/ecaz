//! FR-084 coordinator traversal replica identity and canonical content digest.
//!
//! The replica is a derived, coordinator-local performance object. Owner
//! generations remain authoritative and payload materialization remains
//! owner-side.

use sha2::{Digest, Sha256};
#[cfg(feature = "distann-head-attribution-benchmark")]
use std::time::Instant;
#[cfg(feature = "distann-head-attribution-benchmark")]
use std::{path::Path, time::Duration};

#[cfg(feature = "distann-head-attribution-benchmark")]
use pgrx::datum::{TimestampWithTimeZone, Uuid};
#[cfg(feature = "distann-head-attribution-benchmark")]
use pgrx::iter::TableIterator;
#[cfg(feature = "distann-head-attribution-benchmark")]
use pgrx::{name, pg_extern, pg_sys, PgRelation, Spi};

#[cfg(feature = "distann-head-attribution-benchmark")]
use crate::storage::page::ItemPointer;
#[cfg(feature = "distann-head-attribution-benchmark")]
use crate::storage::relation_guard::{HeapRelationGuard, IndexRelationGuard};
#[cfg(feature = "distann-head-attribution-benchmark")]
use crate::storage::scan_guard::IndexScanGuard;
#[cfg(feature = "distann-head-attribution-benchmark")]
use crate::storage::slot_guard::TupleTableSlotGuard;

use super::canonical_wire::is_rfc4122_v4_uuid;
#[cfg(feature = "distann-head-attribution-benchmark")]
use super::expand_error::DistannExpandError;
#[cfg(feature = "distann-head-attribution-benchmark")]
use super::generation_descriptor::DistannGenerationDescriptor;
#[cfg(feature = "distann-head-attribution-benchmark")]
use super::quantizer::{DistannCodecBinding, DistannPreparedQuery};
#[cfg(feature = "distann-head-attribution-benchmark")]
use super::scan::{
    distann_orchestrated_search, DistannExpandedNode, DistannNodeExpander,
    DistannOrchestrationParams, DistannScanCounters, DistannScanHit, DistannSeedCandidate,
};
#[cfg(feature = "distann-head-attribution-benchmark")]
use super::tuple::DistannNodeTuple;

const REPLICA_CONTENT_DOMAIN: &[u8] = b"ec_distann_traversal_replica_v1\0";
pub(crate) const TRAVERSAL_REPLICA_FORMAT_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TraversalReplicaState {
    Building,
    Ready,
    Stale,
    Retiring,
}

impl TraversalReplicaState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Building => "Building",
            Self::Ready => "Ready",
            Self::Stale => "Stale",
            Self::Retiring => "Retiring",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "Building" => Ok(Self::Building),
            "Ready" => Ok(Self::Ready),
            "Stale" => Ok(Self::Stale),
            "Retiring" => Ok(Self::Retiring),
            other => Err(format!(
                "EC_REPLICA_STATE: unknown traversal replica state {other:?}"
            )),
        }
    }

    pub(crate) fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Building, Self::Ready)
                | (Self::Ready, Self::Stale)
                | (Self::Ready | Self::Stale, Self::Retiring)
        ) || self == next
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TraversalReplicaIdentity {
    pub(crate) logical_index_uuid: [u8; 16],
    pub(crate) build_id: [u8; 16],
    pub(crate) epoch_fingerprint: [u8; 34],
    pub(crate) generation_descriptor_digest: [u8; 32],
    pub(crate) dimensions: u16,
    pub(crate) graph_degree: u16,
    pub(crate) neighbor_codec_kind: u8,
    pub(crate) owner_count: u32,
    pub(crate) expected_record_count: u64,
}

impl TraversalReplicaIdentity {
    fn validate(self) -> Result<(), String> {
        if self.logical_index_uuid == [0; 16] {
            return Err("EC_REPLICA_IDENTITY: logical index UUID is zero".to_owned());
        }
        if !is_rfc4122_v4_uuid(&self.build_id) {
            return Err("EC_REPLICA_IDENTITY: build id is not an RFC 4122 v4 UUID".to_owned());
        }
        if self.epoch_fingerprint[..2] != 2_u16.to_le_bytes() {
            return Err(
                "EC_REPLICA_IDENTITY: epoch fingerprint is not canonical version 2".to_owned(),
            );
        }
        if self.dimensions == 0
            || self.graph_degree == 0
            || self.owner_count == 0
            || self.expected_record_count == 0
        {
            return Err("EC_REPLICA_IDENTITY: replica shape contains zero".to_owned());
        }
        Ok(())
    }
}

pub(crate) struct TraversalReplicaContentHasher {
    hasher: Sha256,
    identity: TraversalReplicaIdentity,
    rows: u64,
    last_key: Option<(u32, i64)>,
}

impl TraversalReplicaContentHasher {
    pub(crate) fn new(identity: TraversalReplicaIdentity) -> Result<Self, String> {
        identity.validate()?;
        let mut hasher = Sha256::new();
        hasher.update(REPLICA_CONTENT_DOMAIN);
        hasher.update(TRAVERSAL_REPLICA_FORMAT_VERSION.to_le_bytes());
        hasher.update(identity.logical_index_uuid);
        hasher.update(identity.build_id);
        hasher.update(identity.epoch_fingerprint);
        hasher.update(identity.generation_descriptor_digest);
        hasher.update(identity.dimensions.to_le_bytes());
        hasher.update(identity.graph_degree.to_le_bytes());
        hasher.update([identity.neighbor_codec_kind]);
        hasher.update(identity.owner_count.to_le_bytes());
        hasher.update(identity.expected_record_count.to_le_bytes());
        Ok(Self {
            hasher,
            identity,
            rows: 0,
            last_key: None,
        })
    }

    pub(crate) fn update_row(
        &mut self,
        owner_ordinal: u32,
        vec_id: u64,
        graph_record: &[u8],
        exact_vector: &[u8],
    ) -> Result<(), String> {
        if owner_ordinal >= self.identity.owner_count {
            return Err(format!(
                "EC_REPLICA_CONTENT: owner ordinal {owner_ordinal} is outside {} owners",
                self.identity.owner_count
            ));
        }
        let key = (owner_ordinal, i64::from_le_bytes(vec_id.to_le_bytes()));
        if self.last_key.is_some_and(|last| key <= last) {
            return Err(
                "EC_REPLICA_CONTENT: rows are duplicated or not in canonical order".to_owned(),
            );
        }
        if graph_record.is_empty() {
            return Err("EC_REPLICA_CONTENT: graph record is empty".to_owned());
        }
        let expected_vector_bytes = usize::from(self.identity.dimensions)
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or_else(|| "EC_REPLICA_CONTENT: exact-vector length overflow".to_owned())?;
        if exact_vector.len() != expected_vector_bytes {
            return Err(format!(
                "EC_REPLICA_CONTENT: exact vector is {} bytes, expected {expected_vector_bytes}",
                exact_vector.len()
            ));
        }
        if exact_vector
            .chunks_exact(4)
            .any(|word| !f32::from_le_bytes(word.try_into().expect("four-byte chunk")).is_finite())
        {
            return Err("EC_REPLICA_CONTENT: exact vector contains a non-finite value".to_owned());
        }
        let graph_len = u32::try_from(graph_record.len())
            .map_err(|_| "EC_REPLICA_CONTENT: graph record exceeds u32".to_owned())?;
        let vector_len = u32::try_from(exact_vector.len())
            .map_err(|_| "EC_REPLICA_CONTENT: exact vector exceeds u32".to_owned())?;
        self.hasher.update(owner_ordinal.to_le_bytes());
        self.hasher.update(vec_id.to_le_bytes());
        self.hasher.update(graph_len.to_le_bytes());
        self.hasher.update(graph_record);
        self.hasher.update(vector_len.to_le_bytes());
        self.hasher.update(exact_vector);
        self.last_key = Some(key);
        self.rows = self
            .rows
            .checked_add(1)
            .ok_or_else(|| "EC_REPLICA_CONTENT: record count overflow".to_owned())?;
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<[u8; 32], String> {
        if self.rows != self.identity.expected_record_count {
            return Err(format!(
                "EC_REPLICA_CONTENT: copied {} records, expected {}",
                self.rows, self.identity.expected_record_count
            ));
        }
        Ok(self.hasher.finalize().into())
    }
}

#[cfg(feature = "distann-head-attribution-benchmark")]
const REPLICA_OWNER_CONTENT_DOMAIN: &[u8] = b"ec_distann_traversal_replica_owner_v1\0";
#[cfg(feature = "distann-head-attribution-benchmark")]
const REPLICA_BUILD_CHUNK_ROWS: usize = 256;

#[cfg(feature = "distann-head-attribution-benchmark")]
fn require_index_owner(index_oid: pg_sys::Oid) -> Result<(), String> {
    let permitted = unsafe {
        pg_sys::superuser()
            || pg_sys::object_ownercheck(pg_sys::RelationRelationId, index_oid, pg_sys::GetUserId())
    };
    if !permitted {
        return Err(
            "EC_REPLICA_AUTH: traversal replica operation requires index owner or superuser"
                .to_owned(),
        );
    }
    Ok(())
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn update_content_row(
    hasher: &mut Sha256,
    owner_ordinal: u32,
    vec_id: u64,
    graph_record: &[u8],
    exact_vector: &[u8],
) -> Result<(), String> {
    let graph_len = u32::try_from(graph_record.len())
        .map_err(|_| "EC_REPLICA_CONTENT: graph record exceeds u32".to_owned())?;
    let vector_len = u32::try_from(exact_vector.len())
        .map_err(|_| "EC_REPLICA_CONTENT: exact vector exceeds u32".to_owned())?;
    hasher.update(owner_ordinal.to_le_bytes());
    hasher.update(vec_id.to_le_bytes());
    hasher.update(graph_len.to_le_bytes());
    hasher.update(graph_record);
    hasher.update(vector_len.to_le_bytes());
    hasher.update(exact_vector);
    Ok(())
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn insert_replica_chunk(
    replica_relid: pg_sys::Oid,
    rows: &[super::generation_read::TraversalReplicaChunkRow],
) -> Result<(), String> {
    if rows.is_empty() {
        return Ok(());
    }
    let replica = super::handoff::qualified_relation_name(replica_relid)?;
    let vec_ids = rows
        .iter()
        .map(|row| i64::from_le_bytes(row.vec_id.to_le_bytes()))
        .collect::<Vec<_>>();
    let owner_ordinals = rows
        .iter()
        .map(|row| {
            i32::try_from(row.owner_ordinal)
                .map_err(|_| "EC_REPLICA_CONTENT: owner ordinal exceeds integer".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let graph_records = rows
        .iter()
        .map(|row| row.graph_record.clone())
        .collect::<Vec<_>>();
    let exact_vectors = rows
        .iter()
        .map(|row| row.exact_vector.clone())
        .collect::<Vec<_>>();
    let expected = i64::try_from(rows.len())
        .map_err(|_| "EC_REPLICA_CONTENT: insert batch exceeds bigint".to_owned())?;
    let inserted = Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "WITH inserted AS (
                         INSERT INTO {replica}
                             (vec_id, owner_ordinal, graph_record, exact_vector)
                         SELECT vec_id, owner_ordinal, graph_record, exact_vector
                           FROM unnest(
                               $1::bigint[], $2::integer[], $3::bytea[], $4::bytea[]
                           ) AS batch(
                               vec_id, owner_ordinal, graph_record, exact_vector
                           )
                         RETURNING 1
                     )
                     SELECT count(*)::bigint AS inserted_count FROM inserted"
                ),
                None,
                &[
                    vec_ids.into(),
                    owner_ordinals.into(),
                    graph_records.into(),
                    exact_vectors.into(),
                ],
            )
            .map_err(|error| format!("EC_REPLICA_CONTENT: replica batch insert failed: {error}"))?
            .next()
            .ok_or_else(|| {
                "EC_REPLICA_CONTENT: replica batch insert returned no count".to_owned()
            })?["inserted_count"]
            .value::<i64>()
            .map_err(|error| format!("EC_REPLICA_CONTENT: inserted count decode failed: {error}"))?
            .ok_or_else(|| "EC_REPLICA_CONTENT: inserted count is NULL".to_owned())
    })?;
    if inserted != expected {
        return Err(format!(
            "EC_REPLICA_CONTENT: inserted {inserted} replica rows, expected {expected}"
        ));
    }
    Ok(())
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn current_wal_lsn() -> Result<String, String> {
    Spi::get_one::<String>("SELECT pg_current_wal_insert_lsn()::text")
        .map_err(|error| format!("EC_REPLICA_STATE: WAL position lookup failed: {error}"))?
        .ok_or_else(|| "EC_REPLICA_STATE: WAL position is NULL".to_owned())
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn wal_bytes_since(start: &str) -> Result<i64, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT pg_wal_lsn_diff(
                     pg_current_wal_insert_lsn(), $1::text::pg_lsn
                 )::bigint AS wal_bytes",
                None,
                &[start.into()],
            )
            .map_err(|error| format!("EC_REPLICA_STATE: WAL byte lookup failed: {error}"))?
            .next()
            .ok_or_else(|| "EC_REPLICA_STATE: WAL byte lookup returned no row".to_owned())?
            ["wal_bytes"]
            .value::<i64>()
            .map_err(|error| format!("EC_REPLICA_STATE: WAL byte decode failed: {error}"))?
            .ok_or_else(|| "EC_REPLICA_STATE: WAL byte count is NULL".to_owned())
    })
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn existing_ready_digest(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    identity: &TraversalReplicaIdentity,
) -> Result<Option<Vec<u8>>, String> {
    let catalog =
        super::generation_catalog::extension_relation_name("ec_distann_traversal_replica")?;
    Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT epoch_fingerprint, generation_descriptor_digest,
                            state, content_digest, replica_relid, directory_relid
                       FROM {catalog}
                      WHERE index_oid = $1::oid
                        AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|error| {
                format!("EC_REPLICA_STATE: existing replica lookup failed: {error}")
            })?
            .map(|row| {
                let fingerprint = row["epoch_fingerprint"]
                    .value::<Vec<u8>>()
                    .map_err(|_| {
                        "EC_REPLICA_STATE: stored fingerprint decode failed".to_owned()
                    })?
                    .ok_or_else(|| {
                        "EC_REPLICA_STATE: stored fingerprint is NULL".to_owned()
                    })?;
                let descriptor_digest = row["generation_descriptor_digest"]
                    .value::<Vec<u8>>()
                    .map_err(|_| {
                        "EC_REPLICA_STATE: stored descriptor digest decode failed".to_owned()
                    })?
                    .ok_or_else(|| {
                        "EC_REPLICA_STATE: stored descriptor digest is NULL".to_owned()
                    })?;
                let state = row["state"]
                    .value::<String>()
                    .map_err(|_| "EC_REPLICA_STATE: stored state decode failed".to_owned())?
                    .ok_or_else(|| "EC_REPLICA_STATE: stored state is NULL".to_owned())?;
                let digest = row["content_digest"]
                    .value::<Vec<u8>>()
                    .map_err(|_| "EC_REPLICA_STATE: content digest decode failed".to_owned())?;
                let replica_relid = row["replica_relid"]
                    .value::<pg_sys::Oid>()
                    .map_err(|_| "EC_REPLICA_STATE: replica OID decode failed".to_owned())?
                    .ok_or_else(|| "EC_REPLICA_STATE: replica OID is NULL".to_owned())?;
                let directory_relid = row["directory_relid"]
                    .value::<pg_sys::Oid>()
                    .map_err(|_| "EC_REPLICA_STATE: directory OID decode failed".to_owned())?
                    .ok_or_else(|| "EC_REPLICA_STATE: directory OID is NULL".to_owned())?;
                if fingerprint != identity.epoch_fingerprint
                    || descriptor_digest != identity.generation_descriptor_digest
                {
                    return Err(
                        "EC_REPLICA_STATE: build id is bound to different immutable inputs"
                            .to_owned(),
                    );
                }
                if state != TraversalReplicaState::Ready.as_str()
                    || digest.is_none()
                    || unsafe { pg_sys::get_rel_relkind(replica_relid) } == 0
                    || unsafe { pg_sys::get_rel_relkind(directory_relid) } == 0
                {
                    return Err(format!(
                        "EC_REPLICA_STATE: existing replica is not a complete Ready replay ({state})"
                    ));
                }
                Ok(digest)
            })
            .next()
            .transpose()
            .map(Option::flatten)
    })
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn build_traversal_replica(index_oid: pg_sys::Oid) -> Result<Vec<u8>, String> {
    require_index_owner(index_oid)?;
    let scan = super::generation_read::PhysicalGenerationScan::open(index_oid)?;
    let (logical_index_uuid, build_id, fingerprint, descriptor, routes) =
        scan.traversal_replica_source();
    let candidate =
        super::build_coordinator::load_build_candidate(index_oid, logical_index_uuid, build_id)?
            .ok_or_else(|| "EC_REPLICA_IDENTITY: active build candidate is absent".to_owned())?;
    let build_spec = super::generation_descriptor::DistannBuildSpec::decode(&candidate.build_spec)?;
    if build_spec.build_id != *build_id.as_bytes()
        || build_spec.generation_descriptor_digest != candidate.generation_descriptor_digest
        || candidate.epoch_fingerprint != fingerprint
        || descriptor.digest()? != candidate.generation_descriptor_digest
        || descriptor.roster.len() != routes.len()
        || build_spec.owner_expectations.len() != routes.len()
    {
        return Err(
            "EC_REPLICA_IDENTITY: active candidate, descriptor, build spec, or routes differ"
                .to_owned(),
        );
    }
    let identity = TraversalReplicaIdentity {
        logical_index_uuid: *logical_index_uuid.as_bytes(),
        build_id: *build_id.as_bytes(),
        epoch_fingerprint: fingerprint,
        generation_descriptor_digest: candidate.generation_descriptor_digest,
        dimensions: descriptor.dimensions,
        graph_degree: descriptor.graph_degree,
        neighbor_codec_kind: descriptor.neighbor_codec_kind,
        owner_count: u32::try_from(routes.len())
            .map_err(|_| "EC_REPLICA_IDENTITY: roster exceeds u32".to_owned())?,
        expected_record_count: build_spec.expected_global_count,
    };
    identity.validate()?;
    if let Some(digest) = existing_ready_digest(index_oid, logical_index_uuid, build_id, &identity)?
    {
        return Ok(digest);
    }

    let wal_start = current_wal_lsn()?;
    let relations =
        super::generation_store::create_traversal_replica_relations(index_oid, build_id)?;
    let catalog =
        super::generation_catalog::extension_relation_name("ec_distann_traversal_replica")?;
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "INSERT INTO {catalog} (
                         index_oid, logical_index_uuid, build_id,
                         epoch_fingerprint, generation_descriptor_digest,
                         state, replica_relid, directory_relid, format_version,
                         dimensions, graph_degree, neighbor_codec_kind,
                         owner_count, expected_record_count
                     ) VALUES (
                         $1::oid, $2::uuid, $3::uuid,
                         $4::bytea, $5::bytea,
                         'Building', $6::oid, $7::oid, 1,
                         $8::integer, $9::integer, $10::smallint,
                         $11::integer, $12::bigint
                     )"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    fingerprint.to_vec().into(),
                    candidate.generation_descriptor_digest.to_vec().into(),
                    relations.replica_relid.into(),
                    relations.directory_relid.into(),
                    i32::from(descriptor.dimensions).into(),
                    i32::from(descriptor.graph_degree).into(),
                    i16::from(descriptor.neighbor_codec_kind).into(),
                    i32::try_from(routes.len())
                        .map_err(|_| "EC_REPLICA_IDENTITY: roster exceeds integer".to_owned())?
                        .into(),
                    i64::try_from(build_spec.expected_global_count)
                        .map_err(|_| "EC_REPLICA_IDENTITY: global count exceeds bigint".to_owned())?
                        .into(),
                ],
            )
            .map_err(|error| {
                format!("EC_REPLICA_STATE: Building catalog insert failed: {error}")
            })?;
        Ok::<(), String>(())
    })?;

    let mut content_hasher = TraversalReplicaContentHasher::new(identity)?;
    let owner_catalog =
        super::generation_catalog::extension_relation_name("ec_distann_traversal_replica_owner")?;
    let binding = super::quantizer::DistannCodecBinding::from_artifact(&descriptor.codec_artifact)?;
    let code_len = binding.code_len(usize::from(descriptor.dimensions))?;
    let mut total_count = 0_u64;
    let mut copied_bytes = 0_u64;
    let mut peak_copy_batch_bytes = 0_u64;
    for (ordinal, route) in routes.iter().enumerate() {
        let owner_ordinal = u32::try_from(ordinal)
            .map_err(|_| "EC_REPLICA_CONTENT: owner ordinal exceeds u32".to_owned())?;
        let roster_entry = &descriptor.roster[ordinal];
        let expectation = &build_spec.owner_expectations[ordinal];
        if route.roster_ordinal != ordinal || roster_entry.node_id != expectation.node_id {
            return Err(
                "EC_REPLICA_IDENTITY: route, roster, and owner expectation order differ".to_owned(),
            );
        }
        let mut owner_hasher = Sha256::new();
        owner_hasher.update(REPLICA_OWNER_CONTENT_DOMAIN);
        owner_hasher.update(TRAVERSAL_REPLICA_FORMAT_VERSION.to_le_bytes());
        owner_hasher.update(identity.logical_index_uuid);
        owner_hasher.update(identity.build_id);
        owner_hasher.update(identity.epoch_fingerprint);
        owner_hasher.update(owner_ordinal.to_le_bytes());
        owner_hasher.update(expectation.expected_count.to_le_bytes());
        let mut owner_count = 0_u64;
        let mut owner_bytes = 0_u64;
        let mut after_vec_id = None;
        loop {
            let rows = if route.is_local {
                super::generation_read::local_traversal_replica_chunk(
                    index_oid,
                    &fingerprint,
                    after_vec_id,
                    REPLICA_BUILD_CHUNK_ROWS,
                )
                .map_err(|error| error.to_string())?
            } else {
                let conninfo = route.conninfo.as_deref().ok_or_else(|| {
                    format!(
                        "EC_NODE_DESCRIPTOR: replica owner {ordinal} has no connection descriptor"
                    )
                })?;
                super::remote_transport::remote_traversal_replica_chunk(
                    &super::remote_transport::DistannTraversalReplicaChunkRequest {
                        conninfo,
                        index_regclass: &route.remote_index_regclass,
                        epoch_fingerprint: &fingerprint,
                        after_vec_id,
                        limit: i32::try_from(REPLICA_BUILD_CHUNK_ROWS)
                            .expect("replica chunk size fits integer"),
                    },
                )
                .map_err(|error| error.to_string())?
            };
            if rows.is_empty() {
                break;
            }
            let previous = after_vec_id;
            let mut chunk_bytes = 0_u64;
            for row in &rows {
                if row.owner_ordinal != owner_ordinal
                    || super::placement::owning_node(
                        row.vec_id,
                        routes.len(),
                        descriptor.placement_hash_version,
                    ) != ordinal
                {
                    return Err(format!(
                        "EC_REPLICA_CONTENT: vec_id {:#018x} is not owned by ordinal {ordinal}",
                        row.vec_id
                    ));
                }
                let node = super::tuple::DistannNodeTuple::decode_physical_v1(
                    &row.graph_record,
                    descriptor.graph_degree,
                    code_len,
                )?;
                if node.vec_id != row.vec_id {
                    return Err(
                        "EC_REPLICA_CONTENT: graph record vec_id differs from stream key"
                            .to_owned(),
                    );
                }
                content_hasher.update_row(
                    owner_ordinal,
                    row.vec_id,
                    &row.graph_record,
                    &row.exact_vector,
                )?;
                update_content_row(
                    &mut owner_hasher,
                    owner_ordinal,
                    row.vec_id,
                    &row.graph_record,
                    &row.exact_vector,
                )?;
                owner_count = owner_count
                    .checked_add(1)
                    .ok_or_else(|| "EC_REPLICA_CONTENT: owner count overflow".to_owned())?;
                // Report source payload bytes independently from physical
                // relation overhead so the benchmark can reconcile transfer
                // cost and coordinator storage amplification separately.
                let row_bytes = u64::try_from(
                    row.graph_record
                        .len()
                        .checked_add(row.exact_vector.len())
                        .ok_or_else(|| {
                            "EC_REPLICA_CONTENT: copied byte count overflow".to_owned()
                        })?,
                )
                .map_err(|_| "EC_REPLICA_CONTENT: copied byte count exceeds u64".to_owned())?;
                owner_bytes = owner_bytes
                    .checked_add(row_bytes)
                    .ok_or_else(|| "EC_REPLICA_CONTENT: owner byte count overflow".to_owned())?;
                chunk_bytes = chunk_bytes
                    .checked_add(row_bytes)
                    .ok_or_else(|| "EC_REPLICA_CONTENT: chunk byte count overflow".to_owned())?;
            }
            peak_copy_batch_bytes = peak_copy_batch_bytes.max(chunk_bytes);
            insert_replica_chunk(relations.replica_relid, &rows)?;
            after_vec_id = rows
                .last()
                .map(|row| i64::from_le_bytes(row.vec_id.to_le_bytes()));
            if after_vec_id <= previous {
                return Err("EC_REPLICA_CONTENT: owner stream keyset did not advance".to_owned());
            }
            if rows.len() < REPLICA_BUILD_CHUNK_ROWS {
                break;
            }
        }
        if owner_count != expectation.expected_count {
            return Err(format!(
                "EC_REPLICA_CONTENT: owner {ordinal} copied {owner_count} records, expected {}",
                expectation.expected_count
            ));
        }
        let owner_digest: [u8; 32] = owner_hasher.finalize().into();
        Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "INSERT INTO {owner_catalog} (
                             index_oid, logical_index_uuid, build_id,
                             owner_ordinal, expected_record_count,
                             copied_record_count, content_digest, copied_bytes
                         ) VALUES (
                             $1::oid, $2::uuid, $3::uuid, $4::integer,
                             $5::bigint, $6::bigint, $7::bytea, $8::bigint
                         )"
                    ),
                    None,
                    &[
                        index_oid.into(),
                        logical_index_uuid.into(),
                        build_id.into(),
                        i32::try_from(ordinal)
                            .map_err(|_| {
                                "EC_REPLICA_CONTENT: owner ordinal exceeds integer".to_owned()
                            })?
                            .into(),
                        i64::try_from(expectation.expected_count)
                            .map_err(|_| {
                                "EC_REPLICA_CONTENT: owner count exceeds bigint".to_owned()
                            })?
                            .into(),
                        i64::try_from(owner_count)
                            .map_err(|_| {
                                "EC_REPLICA_CONTENT: copied owner count exceeds bigint".to_owned()
                            })?
                            .into(),
                        owner_digest.to_vec().into(),
                        i64::try_from(owner_bytes)
                            .map_err(|_| {
                                "EC_REPLICA_CONTENT: owner bytes exceed bigint".to_owned()
                            })?
                            .into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_REPLICA_STATE: owner completion insert failed: {error}")
                })?;
            Ok::<(), String>(())
        })?;
        total_count = total_count
            .checked_add(owner_count)
            .ok_or_else(|| "EC_REPLICA_CONTENT: global count overflow".to_owned())?;
        copied_bytes = copied_bytes
            .checked_add(owner_bytes)
            .ok_or_else(|| "EC_REPLICA_CONTENT: global byte count overflow".to_owned())?;
    }
    let content_digest = content_hasher.finish()?;
    if total_count != build_spec.expected_global_count {
        return Err(format!(
            "EC_REPLICA_CONTENT: global copied count {total_count} differs from {}",
            build_spec.expected_global_count
        ));
    }
    let wal_bytes = wal_bytes_since(&wal_start)?;
    let replica_bytes = Spi::connect(|client| {
        client
            .select(
                "SELECT pg_total_relation_size($1::oid)::bigint AS bytes",
                None,
                &[relations.replica_relid.into()],
            )
            .map_err(|error| format!("EC_REPLICA_STATE: relation size lookup failed: {error}"))?
            .next()
            .ok_or_else(|| "EC_REPLICA_STATE: relation size lookup returned no row".to_owned())?
            ["bytes"]
            .value::<i64>()
            .map_err(|error| format!("EC_REPLICA_STATE: relation size decode failed: {error}"))?
            .ok_or_else(|| "EC_REPLICA_STATE: relation size is NULL".to_owned())
    })?;
    Spi::connect_mut(|client| {
        let updated = client
            .update(
                &format!(
                    "UPDATE {catalog}
                        SET state = 'Ready',
                            content_digest = $4::bytea,
                            copied_record_count = $5::bigint,
                            copied_bytes = $6::bigint,
                            peak_copy_batch_bytes = $7::bigint,
                            relation_bytes = $8::bigint,
                            wal_bytes = $9::bigint,
                            ready_at = clock_timestamp()
                      WHERE index_oid = $1::oid
                        AND logical_index_uuid = $2::uuid
                        AND build_id = $3::uuid
                        AND state = 'Building'"
                ),
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    content_digest.to_vec().into(),
                    i64::try_from(total_count)
                        .map_err(|_| "EC_REPLICA_CONTENT: count exceeds bigint".to_owned())?
                        .into(),
                    i64::try_from(copied_bytes)
                        .map_err(|_| "EC_REPLICA_CONTENT: copied bytes exceed bigint".to_owned())?
                        .into(),
                    i64::try_from(peak_copy_batch_bytes)
                        .map_err(|_| {
                            "EC_REPLICA_CONTENT: peak batch bytes exceed bigint".to_owned()
                        })?
                        .into(),
                    replica_bytes.into(),
                    wal_bytes.into(),
                ],
            )
            .map_err(|error| format!("EC_REPLICA_STATE: Ready transition failed: {error}"))?
            .len();
        if updated != 1 {
            return Err("EC_REPLICA_STATE: Building to Ready transition lost".to_owned());
        }
        Ok::<(), String>(())
    })?;
    Ok(content_digest.to_vec())
}

#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_build_traversal_replica(index_regclass: PgRelation) -> Vec<u8> {
    build_traversal_replica(index_regclass.oid()).unwrap_or_else(|error| pgrx::error!("{error}"))
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn mark_ready_replicas_stale(index_oid: pg_sys::Oid) -> Result<bool, String> {
    require_index_owner(index_oid)?;
    super::lifecycle_guard::require_read_committed("ec_distann_mark_traversal_replica_stale")?;
    let catalog =
        super::generation_catalog::extension_relation_name("ec_distann_traversal_replica")?;
    let updated = Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "UPDATE {catalog}
                        SET state = 'Stale', stale_at = clock_timestamp()
                      WHERE index_oid = $1::oid
                        AND state = 'Ready'"
                ),
                None,
                &[index_oid.into()],
            )
            .map(|rows| rows.len())
            .map_err(|error| format!("EC_REPLICA_STATE: Ready to Stale transition failed: {error}"))
    })?;
    Ok(updated > 0)
}

/// Side-transaction target. This is intentionally separate from the guard:
/// the loopback connection commits this transition before the caller raises
/// the retryable first-attempt error.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_mark_traversal_replica_stale(index_regclass: PgRelation) -> bool {
    mark_ready_replicas_stale(index_regclass.oid()).unwrap_or_else(|error| pgrx::error!("{error}"))
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn loopback_connection_config() -> Result<postgres::Config, String> {
    let (database, user, socket_directories, port) = Spi::connect(|client| {
        client
            .select(
                "SELECT current_database()::text AS database,
                        current_user::text AS user_name,
                        current_setting('unix_socket_directories') AS socket_directories,
                        current_setting('port')::integer AS port",
                None,
                &[],
            )
            .map_err(|error| {
                format!("EC_REPLICA_INVALIDATION: loopback configuration failed: {error}")
            })?
            .next()
            .ok_or_else(|| {
                "EC_REPLICA_INVALIDATION: loopback configuration returned no row".to_owned()
            })
            .and_then(|row| {
                let string = |name: &str| {
                    row[name]
                        .value::<String>()
                        .map_err(|_| {
                            format!("EC_REPLICA_INVALIDATION: loopback {name} decode failed")
                        })?
                        .ok_or_else(|| format!("EC_REPLICA_INVALIDATION: loopback {name} is NULL"))
                };
                let port = row["port"]
                    .value::<i32>()
                    .map_err(|_| "EC_REPLICA_INVALIDATION: loopback port decode failed".to_owned())?
                    .ok_or_else(|| "EC_REPLICA_INVALIDATION: loopback port is NULL".to_owned())?;
                Ok((
                    string("database")?,
                    string("user_name")?,
                    string("socket_directories")?,
                    port,
                ))
            })
    })?;
    let socket = socket_directories
        .split(',')
        .map(str::trim)
        .find(|entry| entry.starts_with('/'));
    let mut config = postgres::Config::new();
    config
        .port(
            u16::try_from(port)
                .map_err(|_| "EC_REPLICA_INVALIDATION: port is outside u16".to_owned())?,
        )
        .dbname(&database)
        .user(&user)
        .application_name("ec_distann_replica_invalidation")
        .connect_timeout(Duration::from_secs(5));
    if let Some(socket) = socket {
        config.host_path(Path::new(socket));
    } else {
        config.host("127.0.0.1");
    }
    Ok(config)
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn invalidate_in_side_transaction(index_oid: pg_sys::Oid) -> Result<bool, String> {
    let stale_function = super::generation_catalog::extension_relation_name(
        "ec_distann_mark_traversal_replica_stale",
    )?;
    let mut client = loopback_connection_config()?
        .connect(postgres::NoTls)
        .map_err(|error| {
            format!("EC_REPLICA_INVALIDATION: dedicated coordinator connection failed: {error}")
        })?;
    let row = client
        .query_one(
            &format!("SELECT {stale_function}($1::oid::regclass)"),
            &[&u32::from(index_oid)],
        )
        .map_err(|error| {
            format!("EC_REPLICA_INVALIDATION: dedicated coordinator transaction failed: {error}")
        })?;
    Ok(row.get::<_, bool>(0))
}

/// Mutation front-door guard. It returns normally when no Ready replica is
/// eligible. The first caller that wins Ready -> Stale commits that change in
/// a dedicated transaction and then receives SQLSTATE 40001. Its retry sees
/// Stale and may dispatch the unchanged owner mutation.
#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_guard_traversal_replica_mutation(index_regclass: PgRelation) -> bool {
    require_index_owner(index_regclass.oid()).unwrap_or_else(|error| pgrx::error!("{error}"));
    let catalog =
        super::generation_catalog::extension_relation_name("ec_distann_traversal_replica")
            .unwrap_or_else(|error| pgrx::error!("{error}"));
    let ready = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT EXISTS (
                         SELECT 1 FROM {catalog}
                          WHERE index_oid = $1::oid AND state = 'Ready'
                     ) AS ready"
                ),
                None,
                &[index_regclass.oid().into()],
            )
            .unwrap_or_else(|error| {
                pgrx::error!("EC_REPLICA_INVALIDATION: Ready lookup failed: {error}")
            })
            .next()
            .and_then(|row| row["ready"].value::<bool>().ok().flatten())
            .unwrap_or(false)
    });
    if !ready {
        return false;
    }
    let transitioned = invalidate_in_side_transaction(index_regclass.oid())
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    if transitioned {
        DistannExpandError::EpochMismatch(
            "EC_REPLICA_INVALIDATED_RETRY: Ready coordinator traversal replica was durably marked Stale; retry the mutation on the owner path"
                .to_owned(),
        )
        .raise();
    }
    false
}

#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_retire_traversal_replica(index_regclass: PgRelation) -> bool {
    require_index_owner(index_regclass.oid()).unwrap_or_else(|error| pgrx::error!("{error}"));
    let catalog =
        super::generation_catalog::extension_relation_name("ec_distann_traversal_replica")
            .unwrap_or_else(|error| pgrx::error!("{error}"));
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "UPDATE {catalog}
                        SET state = 'Retiring', retiring_at = clock_timestamp()
                      WHERE index_oid = $1::oid AND state = 'Stale'"
                ),
                None,
                &[index_regclass.oid().into()],
            )
            .unwrap_or_else(|error| {
                pgrx::error!("EC_REPLICA_STATE: Stale to Retiring failed: {error}")
            })
            .len()
            > 0
    })
}

#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(volatile, parallel_restricted)]
fn ec_distann_reclaim_traversal_replica(index_regclass: PgRelation) -> bool {
    require_index_owner(index_regclass.oid()).unwrap_or_else(|error| pgrx::error!("{error}"));
    let catalog =
        super::generation_catalog::extension_relation_name("ec_distann_traversal_replica")
            .unwrap_or_else(|error| pgrx::error!("{error}"));
    let rows = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT replica_relid, directory_relid
                       FROM {catalog}
                      WHERE index_oid = $1::oid AND state = 'Retiring'"
                ),
                None,
                &[index_regclass.oid().into()],
            )
            .unwrap_or_else(|error| {
                pgrx::error!("EC_REPLICA_STATE: Retiring lookup failed: {error}")
            })
            .map(|row| {
                (
                    row["replica_relid"]
                        .value::<pg_sys::Oid>()
                        .ok()
                        .flatten()
                        .unwrap_or(pg_sys::InvalidOid),
                    row["directory_relid"]
                        .value::<pg_sys::Oid>()
                        .ok()
                        .flatten()
                        .unwrap_or(pg_sys::InvalidOid),
                )
            })
            .collect::<Vec<_>>()
    });
    if rows.is_empty() {
        return false;
    }
    for (replica_relid, directory_relid) in &rows {
        if *replica_relid != pg_sys::InvalidOid && *directory_relid != pg_sys::InvalidOid {
            super::generation_store::drop_traversal_replica_relations(
                index_regclass.oid(),
                super::generation_store::TraversalReplicaRelations {
                    replica_relid: *replica_relid,
                    directory_relid: *directory_relid,
                },
            )
            .unwrap_or_else(|error| pgrx::error!("{error}"));
        }
    }
    Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "DELETE FROM {catalog}
                      WHERE index_oid = $1::oid AND state = 'Retiring'"
                ),
                None,
                &[index_regclass.oid().into()],
            )
            .unwrap_or_else(|error| {
                pgrx::error!("EC_REPLICA_STATE: Retiring reclaim delete failed: {error}")
            });
    });
    true
}

#[cfg(feature = "distann-head-attribution-benchmark")]
#[pg_extern(stable, parallel_restricted)]
fn ec_distann_traversal_replica_status(
    index_regclass: PgRelation,
) -> TableIterator<
    'static,
    (
        name!(build_id, Uuid),
        name!(epoch_fingerprint, Vec<u8>),
        name!(generation_descriptor_digest, Vec<u8>),
        name!(state, String),
        name!(content_digest, Option<Vec<u8>>),
        name!(owner_count, i32),
        name!(expected_record_count, i64),
        name!(copied_record_count, i64),
        name!(copied_bytes, i64),
        name!(peak_copy_batch_bytes, i64),
        name!(relation_bytes, Option<i64>),
        name!(wal_bytes, Option<i64>),
        name!(replica_relid, pg_sys::Oid),
        name!(directory_relid, pg_sys::Oid),
        name!(build_started_at, TimestampWithTimeZone),
        name!(ready_at, Option<TimestampWithTimeZone>),
        name!(stale_at, Option<TimestampWithTimeZone>),
        name!(retiring_at, Option<TimestampWithTimeZone>),
    ),
> {
    require_index_owner(index_regclass.oid()).unwrap_or_else(|error| pgrx::error!("{error}"));
    let (_control, _handle, _metadata, logical_index_uuid) =
        super::generation_store::open_control_index(
            index_regclass.oid(),
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "traversal replica status",
        )
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let catalog =
        super::generation_catalog::extension_relation_name("ec_distann_traversal_replica")
            .unwrap_or_else(|error| pgrx::error!("{error}"));
    let rows = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT build_id, epoch_fingerprint,
                            generation_descriptor_digest, state, content_digest,
                            owner_count, expected_record_count,
                            copied_record_count, copied_bytes,
                            peak_copy_batch_bytes, relation_bytes,
                            wal_bytes,
                            replica_relid, directory_relid,
                            build_started_at, ready_at, stale_at, retiring_at
                       FROM {catalog}
                      WHERE index_oid = $1::oid
                        AND logical_index_uuid = $2::uuid
                      ORDER BY build_started_at DESC"
                ),
                None,
                &[index_regclass.oid().into(), logical_index_uuid.into()],
            )
            .unwrap_or_else(|error| pgrx::error!("EC_REPLICA_STATE: status failed: {error}"))
            .map(|row| {
                macro_rules! required {
                    ($name:literal, $ty:ty) => {
                        row[$name]
                            .value::<$ty>()
                            .unwrap_or_else(|error| {
                                pgrx::error!(
                                    "EC_REPLICA_STATE: status {} decode failed: {error}",
                                    $name
                                )
                            })
                            .unwrap_or_else(|| {
                                pgrx::error!("EC_REPLICA_STATE: status {} is NULL", $name)
                            })
                    };
                }
                (
                    required!("build_id", Uuid),
                    required!("epoch_fingerprint", Vec<u8>),
                    required!("generation_descriptor_digest", Vec<u8>),
                    required!("state", String),
                    row["content_digest"]
                        .value::<Vec<u8>>()
                        .unwrap_or_else(|error| {
                            pgrx::error!(
                                "EC_REPLICA_STATE: status content digest decode failed: {error}"
                            )
                        }),
                    required!("owner_count", i32),
                    required!("expected_record_count", i64),
                    required!("copied_record_count", i64),
                    required!("copied_bytes", i64),
                    required!("peak_copy_batch_bytes", i64),
                    row["relation_bytes"]
                        .value::<i64>()
                        .unwrap_or_else(|error| {
                            pgrx::error!(
                                "EC_REPLICA_STATE: status relation_bytes decode failed: {error}"
                            )
                        }),
                    row["wal_bytes"].value::<i64>().unwrap_or_else(|error| {
                        pgrx::error!("EC_REPLICA_STATE: status WAL bytes decode failed: {error}")
                    }),
                    required!("replica_relid", pg_sys::Oid),
                    required!("directory_relid", pg_sys::Oid),
                    required!("build_started_at", TimestampWithTimeZone),
                    row["ready_at"]
                        .value::<TimestampWithTimeZone>()
                        .unwrap_or_else(|error| {
                            pgrx::error!(
                                "EC_REPLICA_STATE: status ready timestamp decode failed: {error}"
                            )
                        }),
                    row["stale_at"]
                        .value::<TimestampWithTimeZone>()
                        .unwrap_or_else(|error| {
                            pgrx::error!(
                                "EC_REPLICA_STATE: status stale timestamp decode failed: {error}"
                            )
                        }),
                    row["retiring_at"]
                        .value::<TimestampWithTimeZone>()
                        .unwrap_or_else(|error| {
                            pgrx::error!(
                                "EC_REPLICA_STATE: status retiring timestamp decode failed: {error}"
                            )
                        }),
                )
            })
            .collect::<Vec<_>>()
    });
    TableIterator::new(rows.into_iter())
}

#[cfg(feature = "distann-head-attribution-benchmark")]
struct ReplicaStoredNode {
    node: DistannNodeTuple,
    owner_ordinal: usize,
    exact_vector: Vec<f32>,
}

#[cfg(feature = "distann-head-attribution-benchmark")]
pub(crate) struct ReadyTraversalReplica<'a> {
    descriptor: &'a DistannGenerationDescriptor,
    relation: HeapRelationGuard,
    directory: IndexRelationGuard,
    snapshot: pg_sys::Snapshot,
    query: &'a [f32],
    prepared: &'a DistannPreparedQuery,
    code_len: usize,
    local_ordinal: Option<usize>,
    expansion_batches: usize,
}

#[cfg(feature = "distann-head-attribution-benchmark")]
fn replica_slot_attr(
    slot: &TupleTableSlotGuard<'_>,
    attnum: i32,
    label: &str,
) -> Result<pg_sys::Datum, DistannExpandError> {
    let mut is_null = false;
    let datum = unsafe { pg_sys::slot_getattr(slot.as_ptr(), attnum, &mut is_null) };
    if is_null {
        return Err(DistannExpandError::GenerationMissing(format!(
            "traversal replica {label} is NULL"
        )));
    }
    Ok(datum)
}

#[cfg(feature = "distann-head-attribution-benchmark")]
impl<'a> ReadyTraversalReplica<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn open(
        index_oid: pg_sys::Oid,
        logical_index_uuid: Uuid,
        build_id: Uuid,
        fingerprint: &[u8; 34],
        descriptor: &'a DistannGenerationDescriptor,
        descriptor_digest: [u8; 32],
        local_ordinal: Option<usize>,
        snapshot: pg_sys::Snapshot,
        query: &'a [f32],
        prepared: &'a DistannPreparedQuery,
    ) -> Result<Option<Self>, String> {
        if snapshot.is_null() {
            return Err("EC_REPLICA_STATE: scan has no active snapshot".to_owned());
        }
        let catalog =
            super::generation_catalog::extension_relation_name("ec_distann_traversal_replica")?;
        let owner_catalog = super::generation_catalog::extension_relation_name(
            "ec_distann_traversal_replica_owner",
        )?;
        let row = Spi::connect(|client| {
            client
                .select(
                    &format!(
                        "SELECT r.state, r.epoch_fingerprint,
                                r.generation_descriptor_digest,
                                r.content_digest, r.replica_relid,
                                r.directory_relid,
                                r.format_version::integer AS format_version,
                                r.dimensions, r.graph_degree,
                                r.neighbor_codec_kind::integer
                                    AS neighbor_codec_kind,
                                r.owner_count,
                                r.expected_record_count,
                                r.copied_record_count,
                                count(o.owner_ordinal)::bigint
                                    AS completed_owners,
                                coalesce(sum(o.copied_record_count), 0)::bigint
                                    AS owner_record_count
                           FROM {catalog} r
                           LEFT JOIN {owner_catalog} o
                             ON o.index_oid = r.index_oid
                            AND o.logical_index_uuid = r.logical_index_uuid
                            AND o.build_id = r.build_id
                          WHERE r.index_oid = $1::oid
                            AND r.logical_index_uuid = $2::uuid
                            AND r.build_id = $3::uuid
                          GROUP BY r.index_oid, r.logical_index_uuid, r.build_id"
                    ),
                    None,
                    &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
                )
                .map_err(|error| format!("EC_REPLICA_STATE: Ready replica lookup failed: {error}"))?
                .map(|row| {
                    let required_i32 = |name: &str| -> Result<i32, String> {
                        row[name]
                            .value::<i32>()
                            .map_err(|_| format!("EC_REPLICA_STATE: {name} decode failed"))?
                            .ok_or_else(|| format!("EC_REPLICA_STATE: {name} is NULL"))
                    };
                    let required_i64 = |name: &str| -> Result<i64, String> {
                        row[name]
                            .value::<i64>()
                            .map_err(|_| format!("EC_REPLICA_STATE: {name} decode failed"))?
                            .ok_or_else(|| format!("EC_REPLICA_STATE: {name} is NULL"))
                    };
                    let required_bytes = |name: &str| -> Result<Vec<u8>, String> {
                        row[name]
                            .value::<Vec<u8>>()
                            .map_err(|_| format!("EC_REPLICA_STATE: {name} decode failed"))?
                            .ok_or_else(|| format!("EC_REPLICA_STATE: {name} is NULL"))
                    };
                    Ok::<_, String>((
                        row["state"]
                            .value::<String>()
                            .map_err(|_| "EC_REPLICA_STATE: state decode failed".to_owned())?
                            .ok_or_else(|| "EC_REPLICA_STATE: state is NULL".to_owned())?,
                        required_bytes("epoch_fingerprint")?,
                        required_bytes("generation_descriptor_digest")?,
                        row["content_digest"].value::<Vec<u8>>().map_err(|_| {
                            "EC_REPLICA_STATE: content digest decode failed".to_owned()
                        })?,
                        row["replica_relid"]
                            .value::<pg_sys::Oid>()
                            .map_err(|_| "EC_REPLICA_STATE: replica OID decode failed".to_owned())?
                            .ok_or_else(|| "EC_REPLICA_STATE: replica OID is NULL".to_owned())?,
                        row["directory_relid"]
                            .value::<pg_sys::Oid>()
                            .map_err(|_| {
                                "EC_REPLICA_STATE: directory OID decode failed".to_owned()
                            })?
                            .ok_or_else(|| "EC_REPLICA_STATE: directory OID is NULL".to_owned())?,
                        required_i32("format_version")?,
                        required_i32("dimensions")?,
                        required_i32("graph_degree")?,
                        required_i32("neighbor_codec_kind")?,
                        required_i32("owner_count")?,
                        required_i64("expected_record_count")?,
                        required_i64("copied_record_count")?,
                        required_i64("completed_owners")?,
                        required_i64("owner_record_count")?,
                    ))
                })
                .next()
                .transpose()
        })?;
        let Some((
            state,
            stored_fingerprint,
            stored_descriptor_digest,
            content_digest,
            replica_relid,
            directory_relid,
            format_version,
            dimensions,
            graph_degree,
            neighbor_codec_kind,
            owner_count,
            expected_record_count,
            copied_record_count,
            completed_owners,
            owner_record_count,
        )) = row
        else {
            return Ok(None);
        };
        let eligible = state == TraversalReplicaState::Ready.as_str()
            && stored_fingerprint.as_slice() == fingerprint
            && stored_descriptor_digest == descriptor_digest
            && content_digest
                .as_ref()
                .is_some_and(|digest| digest.len() == 32)
            && format_version == i32::from(TRAVERSAL_REPLICA_FORMAT_VERSION)
            && dimensions == i32::from(descriptor.dimensions)
            && graph_degree == i32::from(descriptor.graph_degree)
            && neighbor_codec_kind == i32::from(descriptor.neighbor_codec_kind)
            && owner_count == i32::try_from(descriptor.roster.len()).unwrap_or(-1)
            && completed_owners == i64::from(owner_count)
            && expected_record_count == copied_record_count
            && owner_record_count == copied_record_count
            && copied_record_count > 0;
        if !eligible {
            return Ok(None);
        }
        let Some(relation) = HeapRelationGuard::try_access_share(replica_relid) else {
            return Ok(None);
        };
        let Some(directory) = IndexRelationGuard::try_access_share(directory_relid) else {
            return Ok(None);
        };
        let binding = DistannCodecBinding::from_artifact(&descriptor.codec_artifact)?;
        let code_len = binding.code_len(usize::from(descriptor.dimensions))?;
        Ok(Some(Self {
            descriptor,
            relation,
            directory,
            snapshot,
            query,
            prepared,
            code_len,
            local_ordinal,
            expansion_batches: 0,
        }))
    }

    fn lookup_nodes(&self, vec_ids: &[u64]) -> Result<Vec<ReplicaStoredNode>, DistannExpandError> {
        let scan = unsafe {
            IndexScanGuard::begin_from_raw(
                self.relation.as_ptr(),
                self.directory.as_ptr(),
                self.snapshot,
                1,
                0,
            )
        }
        .ok_or_else(|| {
            DistannExpandError::Internal(
                "could not begin traversal replica directory scan".to_owned(),
            )
        })?;
        let slot = TupleTableSlotGuard::create_for_heap_guard(&self.relation).ok_or_else(|| {
            DistannExpandError::Internal(
                "could not allocate traversal replica tuple slot".to_owned(),
            )
        })?;
        let mut nodes = Vec::with_capacity(vec_ids.len());
        for vec_id in vec_ids {
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
                pg_sys::index_rescan(scan.as_ptr(), &mut scan_key, 1, std::ptr::null_mut(), 0);
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
                return Err(DistannExpandError::GenerationMissing(format!(
                    "EC_RECORD_MISSING: traversal replica lacks vec_id {vec_id:#018x}"
                )));
            }
            let returned_id =
                unsafe { pg_sys::DatumGetInt64(replica_slot_attr(&slot, 1, "vec_id")?) };
            if returned_id != stored_id {
                return Err(DistannExpandError::GenerationMissing(
                    "traversal replica directory returned a different vec_id".to_owned(),
                ));
            }
            let owner_ordinal =
                unsafe { pg_sys::DatumGetInt32(replica_slot_attr(&slot, 2, "owner ordinal")?) };
            let owner_ordinal = usize::try_from(owner_ordinal).map_err(|_| {
                DistannExpandError::GenerationMissing(
                    "traversal replica owner ordinal is negative".to_owned(),
                )
            })?;
            if owner_ordinal >= self.descriptor.roster.len()
                || super::placement::owning_node(
                    *vec_id,
                    self.descriptor.roster.len(),
                    self.descriptor.placement_hash_version,
                ) != owner_ordinal
            {
                return Err(DistannExpandError::Placement(format!(
                    "traversal replica vec_id {vec_id:#018x} has wrong owner {owner_ordinal}"
                )));
            }
            let graph_record =
                unsafe {
                    crate::am::common::detoast::DetoastedVarlena::packed_from_datum(
                        replica_slot_attr(&slot, 3, "graph record")?,
                    )
                }
                .ok_or_else(|| {
                    DistannExpandError::GenerationMissing(
                        "traversal replica graph record could not be detoasted".to_owned(),
                    )
                })?;
            let node = DistannNodeTuple::decode_physical_v1(
                graph_record.as_bytes(),
                self.descriptor.graph_degree,
                self.code_len,
            )
            .map_err(DistannExpandError::GenerationMissing)?;
            if node.vec_id != *vec_id {
                return Err(DistannExpandError::GenerationMissing(
                    "traversal replica graph record identity mismatch".to_owned(),
                ));
            }
            let exact_bytes =
                unsafe {
                    crate::am::common::detoast::DetoastedVarlena::packed_from_datum(
                        replica_slot_attr(&slot, 4, "exact vector")?,
                    )
                }
                .ok_or_else(|| {
                    DistannExpandError::GenerationMissing(
                        "traversal replica exact vector could not be detoasted".to_owned(),
                    )
                })?;
            if exact_bytes.as_bytes().len() != usize::from(self.descriptor.dimensions) * 4 {
                return Err(DistannExpandError::GenerationMissing(
                    "traversal replica exact vector has wrong length".to_owned(),
                ));
            }
            let exact_vector = exact_bytes
                .as_bytes()
                .chunks_exact(4)
                .map(|word| {
                    let value = f32::from_le_bytes(word.try_into().expect("four-byte exact word"));
                    if value.is_finite() {
                        Ok(value)
                    } else {
                        Err(DistannExpandError::GenerationMissing(
                            "traversal replica exact vector contains non-finite value".to_owned(),
                        ))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            nodes.push(ReplicaStoredNode {
                node,
                owner_ordinal,
                exact_vector,
            });
        }
        Ok(nodes)
    }

    pub(crate) fn search(
        &mut self,
        seeds: &[DistannSeedCandidate],
        params: DistannOrchestrationParams,
    ) -> Result<(Vec<DistannScanHit>, DistannScanCounters), DistannExpandError> {
        distann_orchestrated_search(seeds, self, params)
    }
}

#[cfg(feature = "distann-head-attribution-benchmark")]
impl DistannNodeExpander for ReadyTraversalReplica<'_> {
    fn expand_nodes(
        &mut self,
        vec_ids: &[u64],
        _code_threshold: Option<f32>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
        if super::options::benchmark_traversal_replica_fail_batch() == Some(self.expansion_batches)
        {
            return Err(DistannExpandError::Internal(format!(
                "injected traversal replica failure before batch {}",
                self.expansion_batches
            )));
        }
        self.expansion_batches = self.expansion_batches.saturating_add(1);
        let read_started = Instant::now();
        let stored = self.lookup_nodes(vec_ids)?;
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::ReplicaGraphVectorRead,
            read_started.elapsed(),
        );
        let score_started = Instant::now();
        let expanded = stored
            .into_iter()
            .map(|stored| {
                let node = stored.node;
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
                    exact_dist: (!node.tombstoned).then(|| {
                        -crate::am::ec_diskann::source_inner_product(
                            self.query,
                            &stored.exact_vector,
                        )
                    }),
                    is_tombstone: node.tombstoned,
                    heap_tid: if self.local_ordinal == Some(stored.owner_ordinal) {
                        node.heap_tid
                    } else {
                        ItemPointer::INVALID
                    },
                    neighbor_vec_ids: node.neighbor_vec_ids[..count].to_vec(),
                    neighbor_code_dists: neighbor_dists,
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
            .collect();
        super::stage_counters::record(
            super::stage_counters::DistannQueryStage::ReplicaScore,
            score_started.elapsed(),
        );
        expanded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(records: u64) -> TraversalReplicaIdentity {
        let mut build_id = [0x22; 16];
        build_id[6] = 0x42;
        build_id[8] = 0x82;
        let mut fingerprint = [0x33; 34];
        fingerprint[..2].copy_from_slice(&2_u16.to_le_bytes());
        TraversalReplicaIdentity {
            logical_index_uuid: [0x11; 16],
            build_id,
            epoch_fingerprint: fingerprint,
            generation_descriptor_digest: [0x44; 32],
            dimensions: 2,
            graph_degree: 32,
            neighbor_codec_kind: 3,
            owner_count: 2,
            expected_record_count: records,
        }
    }

    fn vector(left: f32, right: f32) -> Vec<u8> {
        [left.to_le_bytes(), right.to_le_bytes()].concat()
    }

    #[test]
    fn content_digest_is_deterministic_and_identity_bound() {
        let build = || {
            let mut hasher = TraversalReplicaContentHasher::new(identity(2)).unwrap();
            hasher
                .update_row(0, 7, b"graph-a", &vector(1.0, -2.0))
                .unwrap();
            hasher
                .update_row(1, 9, b"graph-b", &vector(3.0, 4.0))
                .unwrap();
            hasher.finish().unwrap()
        };
        assert_eq!(build(), build());
        let mut changed = identity(2);
        changed.generation_descriptor_digest[0] ^= 1;
        let mut hasher = TraversalReplicaContentHasher::new(changed).unwrap();
        hasher
            .update_row(0, 7, b"graph-a", &vector(1.0, -2.0))
            .unwrap();
        hasher
            .update_row(1, 9, b"graph-b", &vector(3.0, 4.0))
            .unwrap();
        assert_ne!(build(), hasher.finish().unwrap());
    }

    #[test]
    fn content_digest_rejects_duplicate_order_shape_and_cardinality() {
        let mut hasher = TraversalReplicaContentHasher::new(identity(2)).unwrap();
        hasher
            .update_row(0, 7, b"graph-a", &vector(1.0, 2.0))
            .unwrap();
        assert!(hasher
            .update_row(0, 7, b"graph-a", &vector(1.0, 2.0))
            .is_err());
        assert!(hasher.finish().is_err());

        let mut hasher = TraversalReplicaContentHasher::new(identity(1)).unwrap();
        assert!(hasher
            .update_row(2, 7, b"graph-a", &vector(1.0, 2.0))
            .is_err());
        assert!(hasher
            .update_row(0, 7, b"graph-a", &vector(f32::NAN, 2.0))
            .is_err());
        assert!(hasher.update_row(0, 7, b"graph-a", &[0; 4]).is_err());
    }

    #[test]
    fn state_machine_allows_only_contract_transitions() {
        assert!(TraversalReplicaState::Building.can_transition_to(TraversalReplicaState::Ready));
        assert!(TraversalReplicaState::Ready.can_transition_to(TraversalReplicaState::Stale));
        assert!(TraversalReplicaState::Ready.can_transition_to(TraversalReplicaState::Retiring));
        assert!(TraversalReplicaState::Stale.can_transition_to(TraversalReplicaState::Retiring));
        assert!(!TraversalReplicaState::Building.can_transition_to(TraversalReplicaState::Stale));
        assert!(!TraversalReplicaState::Stale.can_transition_to(TraversalReplicaState::Ready));
    }
}
