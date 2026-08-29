//! Task 179 coordinator/participant catalog access.
//!
//! Every lookup is scoped by both the local index OID and the v5 logical UUID.
//! The UUID comes from control metadata, never from a caller or the relation
//! OID, so a stale row cannot become addressable after OID reuse.

use pgrx::datum::Uuid;
use pgrx::{pg_extern, pg_sys, Spi};

use super::handoff_wire::DISTANN_OWNER_STREAM_HASH_STATE_BYTES;
use super::lifecycle_state::{require_exact_transition_classified, GenerationState};
use super::manifest_v2::{DistannReadyReceipt, DISTANN_READY_RECEIPT_MAX_BYTES};
use super::quote_ident;

const GENERATION_SELECT_COLUMNS: &str = "epoch, owner_ordinal, node_id, state,
    build_spec_digest, roster_digest, generation_descriptor,
    generation_descriptor_digest, expected_owner_count, expected_owner_digest,
    row_tier_relid, cold_tier_relid, graph_store_relid, directory_relid,
    payload_sidecar_relid, payload_sidecar_directory_relid, next_batch_seq,
    cumulative_record_count, cumulative_owner_digest, last_vec_id_le,
    owner_stream_sha256_state, ready_receipt";

pub(crate) fn extension_relation_name(relation_name: &str) -> Result<String, String> {
    let extension_oid = unsafe { pg_sys::get_extension_oid(c"ecaz".as_ptr(), false) };
    if extension_oid == pg_sys::InvalidOid {
        return Err("ec_distann could not resolve the ecaz extension OID".to_owned());
    }
    let namespace_oid = unsafe { pg_sys::get_extension_schema(extension_oid) };
    if namespace_oid == pg_sys::InvalidOid {
        return Err("ec_distann could not resolve the ecaz extension schema".to_owned());
    }
    let namespace_ptr = unsafe { pg_sys::get_namespace_name(namespace_oid) };
    if namespace_ptr.is_null() {
        return Err("ec_distann extension schema has no catalog name".to_owned());
    }
    let namespace = unsafe { std::ffi::CStr::from_ptr(namespace_ptr) }
        .to_str()
        .map_err(|_| "ec_distann extension schema name is not UTF-8".to_owned())?
        .to_owned();
    unsafe { pg_sys::pfree(namespace_ptr.cast()) };
    Ok(format!(
        "{}.{}",
        quote_ident(&namespace),
        quote_ident(relation_name)
    ))
}

struct CatalogRelations {
    participant_identity: String,
    registry_state: String,
    node_descriptor: String,
    generation: String,
    generation_batch: String,
    build_registration: String,
    build_participant_binding: String,
    build_candidate: String,
    generation_head_state: String,
    generation_head_sample: String,
    publish_decision: String,
    predecessor_disposition: String,
    retire_decision: String,
    active_epoch: String,
    generation_reclaim: String,
    cancelled_generation_reclaim: String,
}

impl CatalogRelations {
    fn resolve() -> Result<Self, String> {
        Ok(Self {
            participant_identity: extension_relation_name("ec_distann_participant_identity")?,
            registry_state: extension_relation_name("ec_distann_registry_state")?,
            node_descriptor: extension_relation_name("ec_distann_node_descriptor")?,
            generation: extension_relation_name("ec_distann_generation")?,
            generation_batch: extension_relation_name("ec_distann_generation_batch")?,
            build_registration: extension_relation_name("ec_distann_build_registration")?,
            build_participant_binding: extension_relation_name(
                "ec_distann_build_participant_binding",
            )?,
            build_candidate: extension_relation_name("ec_distann_build_candidate")?,
            generation_head_state: extension_relation_name("ec_distann_generation_head_state")?,
            generation_head_sample: extension_relation_name("ec_distann_generation_head_sample")?,
            publish_decision: extension_relation_name("ec_distann_publish_decision")?,
            predecessor_disposition: extension_relation_name("ec_distann_predecessor_disposition")?,
            retire_decision: extension_relation_name("ec_distann_retire_decision")?,
            active_epoch: extension_relation_name("ec_distann_active_epoch")?,
            generation_reclaim: extension_relation_name("ec_distann_generation_reclaim")?,
            cancelled_generation_reclaim: extension_relation_name(
                "ec_distann_cancelled_generation_reclaim",
            )?,
        })
    }
}

pub(crate) fn initialize_registry_state(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
) -> Result<(), String> {
    let registry_state = extension_relation_name("ec_distann_registry_state")?;
    let sql = format!(
        "INSERT INTO {registry_state} (
             index_oid, logical_index_uuid, revision
         ) VALUES ($1::oid, $2::uuid, 0)"
    );
    Spi::connect_mut(|client| {
        client
            .update(&sql, None, &[index_oid.into(), logical_index_uuid.into()])
            .map_err(|error| format!("EC_NODE_DESCRIPTOR: registry state init failed: {error}"))?;
        Ok(())
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GenerationCatalogRow {
    pub(crate) epoch: u64,
    pub(crate) owner_ordinal: u32,
    pub(crate) node_id: u32,
    pub(crate) state: GenerationState,
    pub(crate) build_spec_digest: [u8; 32],
    pub(crate) roster_digest: [u8; 32],
    pub(crate) generation_descriptor: Vec<u8>,
    pub(crate) generation_descriptor_digest: [u8; 32],
    pub(crate) expected_owner_count: u64,
    pub(crate) expected_owner_digest: [u8; 32],
    pub(crate) row_tier_relid: pg_sys::Oid,
    pub(crate) cold_tier_relid: Option<pg_sys::Oid>,
    pub(crate) graph_store_relid: pg_sys::Oid,
    pub(crate) directory_relid: pg_sys::Oid,
    pub(crate) payload_sidecar_relid: Option<pg_sys::Oid>,
    pub(crate) payload_sidecar_directory_relid: Option<pg_sys::Oid>,
    pub(crate) next_batch_seq: u64,
    pub(crate) cumulative_record_count: u64,
    pub(crate) cumulative_owner_digest: [u8; 32],
    pub(crate) last_vec_id_le: Option<[u8; 8]>,
    pub(crate) owner_stream_sha256_state: [u8; DISTANN_OWNER_STREAM_HASH_STATE_BYTES],
    pub(crate) ready_receipt: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RetainedGenerationCatalogRow {
    pub(crate) build_id: Uuid,
    pub(crate) generation: GenerationCatalogRow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GenerationBatchCatalogRow {
    pub(crate) batch_seq: u64,
    pub(crate) batch_digest: [u8; 32],
    pub(crate) encoded_bytes: u64,
    pub(crate) accepted_record_count: u64,
    pub(crate) cumulative_record_count: u64,
    pub(crate) cumulative_owner_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GenerationBatchSummary {
    pub(crate) batch_count: u64,
    pub(crate) minimum_sequence: u64,
    pub(crate) maximum_sequence: u64,
    pub(crate) accepted_record_count: u64,
}

fn required_i64(row: &pgrx::spi::SpiHeapTupleData<'_>, name: &str) -> Result<i64, String> {
    row[name]
        .value::<i64>()
        .map_err(|error| format!("ec_distann generation catalog {name} decode failed: {error}"))?
        .ok_or_else(|| format!("ec_distann generation catalog {name} is NULL"))
}

fn required_i32(row: &pgrx::spi::SpiHeapTupleData<'_>, name: &str) -> Result<i32, String> {
    row[name]
        .value::<i32>()
        .map_err(|error| format!("ec_distann generation catalog {name} decode failed: {error}"))?
        .ok_or_else(|| format!("ec_distann generation catalog {name} is NULL"))
}

fn required_oid(row: &pgrx::spi::SpiHeapTupleData<'_>, name: &str) -> Result<pg_sys::Oid, String> {
    row[name]
        .value::<pg_sys::Oid>()
        .map_err(|error| format!("ec_distann generation catalog {name} decode failed: {error}"))?
        .ok_or_else(|| format!("ec_distann generation catalog {name} is NULL"))
}

fn optional_oid(
    row: &pgrx::spi::SpiHeapTupleData<'_>,
    name: &str,
) -> Result<Option<pg_sys::Oid>, String> {
    row[name]
        .value::<pg_sys::Oid>()
        .map_err(|error| format!("ec_distann generation catalog {name} decode failed: {error}"))
}

fn required_string(row: &pgrx::spi::SpiHeapTupleData<'_>, name: &str) -> Result<String, String> {
    row[name]
        .value::<String>()
        .map_err(|error| format!("ec_distann generation catalog {name} decode failed: {error}"))?
        .ok_or_else(|| format!("ec_distann generation catalog {name} is NULL"))
}

fn required_bytes(row: &pgrx::spi::SpiHeapTupleData<'_>, name: &str) -> Result<Vec<u8>, String> {
    row[name]
        .value::<Vec<u8>>()
        .map_err(|error| format!("ec_distann generation catalog {name} decode failed: {error}"))?
        .ok_or_else(|| format!("ec_distann generation catalog {name} is NULL"))
}

fn optional_bytes(
    row: &pgrx::spi::SpiHeapTupleData<'_>,
    name: &str,
) -> Result<Option<Vec<u8>>, String> {
    row[name]
        .value::<Vec<u8>>()
        .map_err(|error| format!("ec_distann generation catalog {name} decode failed: {error}"))
}

fn fixed_bytes<const N: usize>(bytes: Vec<u8>, name: &str) -> Result<[u8; N], String> {
    bytes.try_into().map_err(|bytes: Vec<u8>| {
        format!(
            "ec_distann generation catalog {name} is {} bytes, expected {N}",
            bytes.len()
        )
    })
}

fn optional_fixed_bytes<const N: usize>(
    bytes: Option<Vec<u8>>,
    name: &str,
) -> Result<Option<[u8; N]>, String> {
    bytes.map(|bytes| fixed_bytes(bytes, name)).transpose()
}

fn decode_generation_row(
    row: pgrx::spi::SpiHeapTupleData<'_>,
) -> Result<GenerationCatalogRow, String> {
    let payload_sidecar_relid = optional_oid(&row, "payload_sidecar_relid")?;
    let payload_sidecar_directory_relid = optional_oid(&row, "payload_sidecar_directory_relid")?;
    if payload_sidecar_relid.is_some() != payload_sidecar_directory_relid.is_some() {
        return Err(
            "ec_distann generation catalog payload sidecar relation pair is incomplete".to_owned(),
        );
    }
    let cold_tier_relid = optional_oid(&row, "cold_tier_relid")?;
    if cold_tier_relid.is_some() && payload_sidecar_relid.is_some() {
        return Err(
            "ec_distann generation catalog hot/cold tier conflicts with payload sidecar".to_owned(),
        );
    }
    Ok(GenerationCatalogRow {
        epoch: u64::try_from(required_i64(&row, "epoch")?)
            .map_err(|_| "ec_distann generation catalog epoch is negative".to_owned())?,
        owner_ordinal: u32::try_from(required_i32(&row, "owner_ordinal")?)
            .map_err(|_| "ec_distann generation owner ordinal is negative".to_owned())?,
        node_id: u32::try_from(required_i32(&row, "node_id")?)
            .map_err(|_| "ec_distann generation node id is negative".to_owned())?,
        state: GenerationState::parse(&required_string(&row, "state")?)?,
        build_spec_digest: fixed_bytes::<32>(
            required_bytes(&row, "build_spec_digest")?,
            "build_spec_digest",
        )?,
        roster_digest: fixed_bytes::<32>(required_bytes(&row, "roster_digest")?, "roster_digest")?,
        generation_descriptor: required_bytes(&row, "generation_descriptor")?,
        generation_descriptor_digest: fixed_bytes::<32>(
            required_bytes(&row, "generation_descriptor_digest")?,
            "generation_descriptor_digest",
        )?,
        expected_owner_count: u64::try_from(required_i64(&row, "expected_owner_count")?)
            .map_err(|_| "ec_distann expected owner count is negative".to_owned())?,
        expected_owner_digest: fixed_bytes::<32>(
            required_bytes(&row, "expected_owner_digest")?,
            "expected_owner_digest",
        )?,
        row_tier_relid: required_oid(&row, "row_tier_relid")?,
        cold_tier_relid,
        graph_store_relid: required_oid(&row, "graph_store_relid")?,
        directory_relid: required_oid(&row, "directory_relid")?,
        payload_sidecar_relid,
        payload_sidecar_directory_relid,
        next_batch_seq: u64::try_from(required_i64(&row, "next_batch_seq")?)
            .map_err(|_| "ec_distann next batch sequence is negative".to_owned())?,
        cumulative_record_count: u64::try_from(required_i64(&row, "cumulative_record_count")?)
            .map_err(|_| "ec_distann cumulative record count is negative".to_owned())?,
        cumulative_owner_digest: fixed_bytes::<32>(
            required_bytes(&row, "cumulative_owner_digest")?,
            "cumulative_owner_digest",
        )?,
        last_vec_id_le: optional_fixed_bytes(
            optional_bytes(&row, "last_vec_id_le")?,
            "last_vec_id_le",
        )?,
        owner_stream_sha256_state: fixed_bytes(
            required_bytes(&row, "owner_stream_sha256_state")?,
            "owner_stream_sha256_state",
        )?,
        ready_receipt: optional_bytes(&row, "ready_receipt")?
            .map(|bytes| {
                if bytes.len() > DISTANN_READY_RECEIPT_MAX_BYTES {
                    return Err(format!(
                        "ready_receipt is {} bytes, exceeds {DISTANN_READY_RECEIPT_MAX_BYTES}",
                        bytes.len()
                    ));
                }
                DistannReadyReceipt::decode(&bytes)?;
                Ok(bytes)
            })
            .transpose()?,
    })
}

fn lookup_generation_with_lock(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    for_update: bool,
) -> Result<Option<GenerationCatalogRow>, String> {
    let catalogs = CatalogRelations::resolve()?;
    let lock_clause = if for_update { " FOR UPDATE" } else { "" };
    let sql = format!(
        "SELECT {GENERATION_SELECT_COLUMNS}
           FROM {}
          WHERE index_oid = $1::oid
            AND logical_index_uuid = $2::uuid
            AND build_id = $3::uuid{lock_clause}",
        catalogs.generation
    );
    Spi::connect_mut(|client| {
        client
            .update(
                &sql,
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|error| format!("ec_distann generation lookup failed: {error}"))?
            .map(decode_generation_row)
            .next()
            .transpose()
    })
}

pub(crate) fn lookup_generation(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
) -> Result<Option<GenerationCatalogRow>, String> {
    lookup_generation_with_lock(index_oid, logical_index_uuid, build_id, false)
}

/// Resolve the immutable participant generation named by an FR-079 epoch
/// fingerprint.  Remote read endpoints deliberately use this lookup rather
/// than the participant's active pointer: a coordinator scan may remain pinned
/// to a Retired predecessor until its scan token drains.
pub(crate) fn lookup_retained_generation_by_fingerprint(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    epoch_fingerprint: &[u8; 34],
) -> Result<Option<RetainedGenerationCatalogRow>, String> {
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "SELECT build_id, {GENERATION_SELECT_COLUMNS}
           FROM {}
          WHERE index_oid = $1::oid
            AND logical_index_uuid = $2::uuid
            AND epoch_fingerprint = $3::bytea
            AND state IN ('Published', 'Retired')",
        catalogs.generation
    );
    Spi::connect(|client| {
        client
            .select(
                &sql,
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    epoch_fingerprint.to_vec().into(),
                ],
            )
            .map_err(|error| {
                format!("EC_GENERATION_MISSING: retained generation lookup failed: {error}")
            })?
            .map(|row| {
                let build_id = row["build_id"]
                    .value::<Uuid>()
                    .map_err(|error| {
                        format!("EC_GENERATION_MISSING: build id decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_GENERATION_MISSING: build id is NULL".to_owned())?;
                Ok(RetainedGenerationCatalogRow {
                    build_id,
                    generation: decode_generation_row(row)?,
                })
            })
            .next()
            .transpose()
    })
}

pub(crate) fn lookup_generation_for_update(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
) -> Result<Option<GenerationCatalogRow>, String> {
    lookup_generation_with_lock(index_oid, logical_index_uuid, build_id, true)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn insert_generation(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    row: &GenerationCatalogRow,
) -> Result<(), String> {
    let epoch = i64::try_from(row.epoch)
        .map_err(|_| "ec_distann generation epoch exceeds bigint".to_owned())?;
    let owner_ordinal = i32::try_from(row.owner_ordinal)
        .map_err(|_| "ec_distann generation owner ordinal exceeds integer".to_owned())?;
    let node_id = i32::try_from(row.node_id)
        .map_err(|_| "ec_distann generation node id exceeds integer".to_owned())?;
    let expected_owner_count = i64::try_from(row.expected_owner_count)
        .map_err(|_| "ec_distann expected owner count exceeds bigint".to_owned())?;
    let next_batch_seq = i64::try_from(row.next_batch_seq)
        .map_err(|_| "ec_distann next batch sequence exceeds bigint".to_owned())?;
    let cumulative_record_count = i64::try_from(row.cumulative_record_count)
        .map_err(|_| "ec_distann cumulative record count exceeds bigint".to_owned())?;
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "INSERT INTO {} (
             index_oid, logical_index_uuid, build_id, epoch,
             owner_ordinal, node_id, state, build_spec_digest,
             roster_digest, generation_descriptor,
             generation_descriptor_digest, expected_owner_count,
             expected_owner_digest, row_tier_relid, cold_tier_relid,
             graph_store_relid, directory_relid, payload_sidecar_relid,
             payload_sidecar_directory_relid, next_batch_seq, cumulative_record_count,
             cumulative_owner_digest, last_vec_id_le,
             owner_stream_sha256_state, ready_receipt
         ) VALUES (
             $1::oid, $2::uuid, $3::uuid, $4::bigint,
             $5::integer, $6::integer, $7::text, $8::bytea,
             $9::bytea, $10::bytea, $11::bytea, $12::bigint,
             $13::bytea, $14::oid, $15::oid, $16::oid, $17::oid,
             $18::oid, $19::oid, $20::bigint, $21::bigint,
             $22::bytea, $23::bytea, $24::bytea, $25::bytea
         )",
        catalogs.generation
    );
    Spi::connect_mut(|client| {
        client
            .update(
                &sql,
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    epoch.into(),
                    owner_ordinal.into(),
                    node_id.into(),
                    row.state.as_str().into(),
                    row.build_spec_digest.to_vec().into(),
                    row.roster_digest.to_vec().into(),
                    row.generation_descriptor.clone().into(),
                    row.generation_descriptor_digest.to_vec().into(),
                    expected_owner_count.into(),
                    row.expected_owner_digest.to_vec().into(),
                    row.row_tier_relid.into(),
                    row.cold_tier_relid.into(),
                    row.graph_store_relid.into(),
                    row.directory_relid.into(),
                    row.payload_sidecar_relid.into(),
                    row.payload_sidecar_directory_relid.into(),
                    next_batch_seq.into(),
                    cumulative_record_count.into(),
                    row.cumulative_owner_digest.to_vec().into(),
                    row.last_vec_id_le.map(|bytes| bytes.to_vec()).into(),
                    row.owner_stream_sha256_state.to_vec().into(),
                    row.ready_receipt.clone().into(),
                ],
            )
            .map_err(|error| format!("ec_distann generation catalog insert failed: {error}"))?;
        Ok(())
    })
}

fn decode_generation_batch_row(
    row: pgrx::spi::SpiHeapTupleData<'_>,
) -> Result<GenerationBatchCatalogRow, String> {
    Ok(GenerationBatchCatalogRow {
        batch_seq: u64::try_from(required_i64(&row, "batch_seq")?)
            .map_err(|_| "ec_distann batch sequence is negative".to_owned())?,
        batch_digest: fixed_bytes::<32>(required_bytes(&row, "batch_digest")?, "batch_digest")?,
        encoded_bytes: u64::try_from(required_i64(&row, "encoded_bytes")?)
            .map_err(|_| "ec_distann encoded batch length is negative".to_owned())?,
        accepted_record_count: u64::try_from(required_i64(&row, "accepted_record_count")?)
            .map_err(|_| "ec_distann accepted record count is negative".to_owned())?,
        cumulative_record_count: u64::try_from(required_i64(&row, "cumulative_record_count")?)
            .map_err(|_| "ec_distann cumulative record count is negative".to_owned())?,
        cumulative_owner_digest: fixed_bytes::<32>(
            required_bytes(&row, "cumulative_owner_digest")?,
            "cumulative_owner_digest",
        )?,
    })
}

pub(crate) fn lookup_generation_batch(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    batch_seq: u64,
) -> Result<Option<GenerationBatchCatalogRow>, String> {
    let batch_seq = i64::try_from(batch_seq)
        .map_err(|_| "EC_BATCH_SEQUENCE: batch sequence exceeds bigint".to_owned())?;
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "SELECT batch_seq, batch_digest, encoded_bytes, accepted_record_count,
                cumulative_record_count, cumulative_owner_digest
           FROM {}
          WHERE index_oid = $1::oid
            AND logical_index_uuid = $2::uuid
            AND build_id = $3::uuid
            AND batch_seq = $4::bigint",
        catalogs.generation_batch
    );
    Spi::connect(|client| {
        client
            .select(
                &sql,
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    batch_seq.into(),
                ],
            )
            .map_err(|error| format!("ec_distann generation batch lookup failed: {error}"))?
            .map(decode_generation_batch_row)
            .next()
            .transpose()
    })
}

pub(crate) fn insert_generation_batch(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    row: &GenerationBatchCatalogRow,
) -> Result<(), String> {
    let batch_seq = i64::try_from(row.batch_seq)
        .map_err(|_| "EC_BATCH_SEQUENCE: batch sequence exceeds bigint".to_owned())?;
    let encoded_bytes = i64::try_from(row.encoded_bytes)
        .map_err(|_| "EC_HANDOFF_TOO_LARGE: encoded batch length exceeds bigint".to_owned())?;
    let accepted_record_count = i64::try_from(row.accepted_record_count)
        .map_err(|_| "EC_BUILD_INCOMPLETE: accepted record count exceeds bigint".to_owned())?;
    let cumulative_record_count = i64::try_from(row.cumulative_record_count)
        .map_err(|_| "EC_BUILD_INCOMPLETE: cumulative record count exceeds bigint".to_owned())?;
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "INSERT INTO {} (
             index_oid, logical_index_uuid, build_id, batch_seq, batch_digest,
             encoded_bytes, accepted_record_count, cumulative_record_count,
             cumulative_owner_digest
         ) VALUES (
             $1::oid, $2::uuid, $3::uuid, $4::bigint, $5::bytea,
             $6::bigint, $7::bigint, $8::bigint, $9::bytea
         )",
        catalogs.generation_batch
    );
    Spi::connect_mut(|client| {
        client
            .update(
                &sql,
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    batch_seq.into(),
                    row.batch_digest.to_vec().into(),
                    encoded_bytes.into(),
                    accepted_record_count.into(),
                    cumulative_record_count.into(),
                    row.cumulative_owner_digest.to_vec().into(),
                ],
            )
            .map_err(|error| format!("ec_distann generation batch insert failed: {error}"))?;
        Ok(())
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn advance_generation_after_batch(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    old_next_batch_seq: u64,
    old_cumulative_record_count: u64,
    old_cumulative_owner_digest: [u8; 32],
    new_cumulative_record_count: u64,
    new_cumulative_owner_digest: [u8; 32],
    last_vec_id_le: Option<[u8; 8]>,
    owner_stream_sha256_state: [u8; DISTANN_OWNER_STREAM_HASH_STATE_BYTES],
) -> Result<(), String> {
    let old_next_batch_seq = i64::try_from(old_next_batch_seq)
        .map_err(|_| "EC_BATCH_SEQUENCE: batch sequence exceeds bigint".to_owned())?;
    let new_next_batch_seq = old_next_batch_seq
        .checked_add(1)
        .ok_or_else(|| "EC_BATCH_SEQUENCE: batch sequence exhausted bigint".to_owned())?;
    let old_cumulative_record_count = i64::try_from(old_cumulative_record_count)
        .map_err(|_| "EC_BUILD_INCOMPLETE: cumulative record count exceeds bigint".to_owned())?;
    let new_cumulative_record_count = i64::try_from(new_cumulative_record_count)
        .map_err(|_| "EC_BUILD_INCOMPLETE: cumulative record count exceeds bigint".to_owned())?;
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "UPDATE {}
            SET next_batch_seq = $7::bigint,
                cumulative_record_count = $8::bigint,
                cumulative_owner_digest = $9::bytea,
                last_vec_id_le = $10::bytea,
                owner_stream_sha256_state = $11::bytea,
                updated_at = clock_timestamp()
          WHERE index_oid = $1::oid
            AND logical_index_uuid = $2::uuid
            AND build_id = $3::uuid
            AND state = 'Building'
            AND next_batch_seq = $4::bigint
            AND cumulative_record_count = $5::bigint
            AND cumulative_owner_digest = $6::bytea
          RETURNING 1",
        catalogs.generation
    );
    let updated = Spi::connect_mut(|client| {
        client
            .update(
                &sql,
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    old_next_batch_seq.into(),
                    old_cumulative_record_count.into(),
                    old_cumulative_owner_digest.to_vec().into(),
                    new_next_batch_seq.into(),
                    new_cumulative_record_count.into(),
                    new_cumulative_owner_digest.to_vec().into(),
                    last_vec_id_le.map(|bytes| bytes.to_vec()).into(),
                    owner_stream_sha256_state.to_vec().into(),
                ],
            )
            .map_err(|error| format!("ec_distann generation progress update failed: {error}"))
            .map(|table| table.len())
    })?;
    if updated != 1 {
        return Err(
            "EC_BUILD_STATE: generation progress changed concurrently or is no longer Building"
                .to_owned(),
        );
    }
    Ok(())
}

/// SECURITY DEFINER bridge for AM initialization. The AM may run as an
/// ordinary table/index owner, while the extension-owned registry remains
/// PUBLIC-revoked. Validate the just-written v5 metadata and exact UUID before
/// crossing that privilege boundary.
#[pg_extern(volatile, strict, parallel_restricted)]
fn ec_distann_initialize_control_registry(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
) -> bool {
    let (_guard, _handle, _metadata, actual_uuid) = super::generation_store::open_control_index(
        index_oid,
        pg_sys::NoLock as pg_sys::LOCKMODE,
        "ec_distann control registry initialization",
    )
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    if actual_uuid != logical_index_uuid {
        pgrx::error!(
            "EC_BUILD_STATE: control registry UUID does not match the initialized metadata"
        );
    }
    initialize_registry_state(index_oid, logical_index_uuid)
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    true
}

pub(crate) fn generation_batch_summary(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
) -> Result<Option<GenerationBatchSummary>, String> {
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "SELECT count(*)::bigint AS batch_count,
                min(batch_seq) AS minimum_sequence,
                max(batch_seq) AS maximum_sequence,
                sum(accepted_record_count)::bigint AS accepted_record_count
           FROM {}
          WHERE index_oid = $1::oid
            AND logical_index_uuid = $2::uuid
            AND build_id = $3::uuid",
        catalogs.generation_batch
    );
    Spi::connect(|client| {
        client
            .select(
                &sql,
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|error| format!("ec_distann batch summary lookup failed: {error}"))?
            .map(|row| {
                let batch_count = u64::try_from(required_i64(&row, "batch_count")?)
                    .map_err(|_| "ec_distann batch count is negative".to_owned())?;
                if batch_count == 0 {
                    return Ok(None);
                }
                Ok(Some(GenerationBatchSummary {
                    batch_count,
                    minimum_sequence: u64::try_from(required_i64(&row, "minimum_sequence")?)
                        .map_err(|_| "ec_distann minimum batch sequence is negative".to_owned())?,
                    maximum_sequence: u64::try_from(required_i64(&row, "maximum_sequence")?)
                        .map_err(|_| "ec_distann maximum batch sequence is negative".to_owned())?,
                    accepted_record_count: u64::try_from(required_i64(
                        &row,
                        "accepted_record_count",
                    )?)
                    .map_err(|_| "ec_distann accepted record total is negative".to_owned())?,
                }))
            })
            .next()
            .transpose()
            .map(Option::flatten)
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn mark_generation_ready(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
    next_batch_seq: u64,
    cumulative_record_count: u64,
    cumulative_owner_digest: [u8; 32],
    ready_receipt: &[u8],
) -> Result<(), String> {
    if ready_receipt.len() > DISTANN_READY_RECEIPT_MAX_BYTES {
        return Err(format!(
            "EC_READY_RECEIPT: receipt is {} bytes, exceeds {DISTANN_READY_RECEIPT_MAX_BYTES}",
            ready_receipt.len()
        ));
    }
    DistannReadyReceipt::decode(ready_receipt)?;
    let next_batch_seq = i64::try_from(next_batch_seq)
        .map_err(|_| "EC_BATCH_SEQUENCE: next batch sequence exceeds bigint".to_owned())?;
    let cumulative_record_count = i64::try_from(cumulative_record_count)
        .map_err(|_| "EC_BUILD_INCOMPLETE: cumulative record count exceeds bigint".to_owned())?;
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "UPDATE {}
            SET state = 'Ready', ready_receipt = $7::bytea,
                updated_at = clock_timestamp()
          WHERE index_oid = $1::oid
            AND logical_index_uuid = $2::uuid
            AND build_id = $3::uuid
            AND state = 'Building' AND ready_receipt IS NULL
            AND next_batch_seq = $4::bigint
            AND cumulative_record_count = $5::bigint
            AND cumulative_owner_digest = $6::bytea
          RETURNING 1",
        catalogs.generation
    );
    let updated = Spi::connect_mut(|client| {
        client
            .update(
                &sql,
                None,
                &[
                    index_oid.into(),
                    logical_index_uuid.into(),
                    build_id.into(),
                    next_batch_seq.into(),
                    cumulative_record_count.into(),
                    cumulative_owner_digest.to_vec().into(),
                    ready_receipt.to_vec().into(),
                ],
            )
            .map_err(|error| format!("ec_distann Ready transition failed: {error}"))
            .map(|table| table.len())
    })?;
    require_exact_transition_classified(
        GenerationState::Building,
        GenerationState::Ready,
        updated,
        "generation",
        "EC_BUILD_STATE",
    )
}

pub(crate) fn generation_relations_for_index(
    index_oid: pg_sys::Oid,
) -> Result<
    Vec<(
        pg_sys::Oid,
        Option<pg_sys::Oid>,
        pg_sys::Oid,
        pg_sys::Oid,
        Option<pg_sys::Oid>,
        Option<pg_sys::Oid>,
    )>,
    String,
> {
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "SELECT row_tier_relid, cold_tier_relid, graph_store_relid, directory_relid,
                payload_sidecar_relid, payload_sidecar_directory_relid
           FROM {}
          WHERE index_oid = $1::oid",
        catalogs.generation
    );
    Spi::connect(|client| {
        client
            .select(&sql, None, &[index_oid.into()])
            .map_err(|error| format!("ec_distann generation relation lookup failed: {error}"))?
            .map(|row| {
                let payload_sidecar_relid = optional_oid(&row, "payload_sidecar_relid")?;
                let payload_sidecar_directory_relid =
                    optional_oid(&row, "payload_sidecar_directory_relid")?;
                if payload_sidecar_relid.is_some() != payload_sidecar_directory_relid.is_some() {
                    return Err(
                        "ec_distann generation catalog payload sidecar relation pair is incomplete"
                            .to_owned(),
                    );
                }
                Ok((
                    required_oid(&row, "row_tier_relid")?,
                    optional_oid(&row, "cold_tier_relid")?,
                    required_oid(&row, "graph_store_relid")?,
                    required_oid(&row, "directory_relid")?,
                    payload_sidecar_relid,
                    payload_sidecar_directory_relid,
                ))
            })
            .collect::<Result<Vec<_>, String>>()
    })
}

pub(crate) fn has_publish_decision(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
) -> Result<bool, String> {
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "SELECT EXISTS (
             SELECT 1 FROM {}
              WHERE index_oid = $1::oid
                AND logical_index_uuid = $2::uuid
                AND build_id = $3::uuid
         ) AS decision_exists",
        catalogs.publish_decision
    );
    Spi::connect(|client| {
        client
            .select(
                &sql,
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|error| format!("ec_distann publish-decision lookup failed: {error}"))?
            .map(|row| {
                row["decision_exists"]
                    .value::<bool>()
                    .map_err(|error| format!("ec_distann publish-decision decode failed: {error}"))?
                    .ok_or_else(|| "ec_distann publish-decision lookup returned NULL".to_owned())
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or(false))
    })
}

pub(crate) fn delete_generation_if_unpublished(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
) -> Result<(), String> {
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "DELETE FROM {}
          WHERE index_oid = $1::oid
            AND logical_index_uuid = $2::uuid
            AND build_id = $3::uuid
            AND state IN ('Building', 'Ready')
            AND NOT EXISTS (
                SELECT 1 FROM {}
                 WHERE index_oid = $1::oid
                   AND logical_index_uuid = $2::uuid
                   AND build_id = $3::uuid
            )
          RETURNING 1",
        catalogs.generation, catalogs.publish_decision
    );
    let deleted = Spi::connect_mut(|client| {
        client
            .update(
                &sql,
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|error| format!("ec_distann generation delete failed: {error}"))
            .map(|table| table.len())
    })?;
    if deleted != 1 {
        return Err("EC_BUILD_STATE: abort lost its unpublished-generation guard".to_owned());
    }
    Ok(())
}

pub(crate) fn delete_index_catalog_rows(index_oid: pg_sys::Oid) -> Result<i64, String> {
    // DROP/REINDEX may run in the same backend that owns a build's retained
    // relation locks. Defer removal until commit so an aborted destructive
    // operation restores both the catalog gate and its session ownership.
    super::build_coordinator::schedule_session_lock_release_for_control(index_oid);
    let catalogs = CatalogRelations::resolve()?;
    let count_sql = format!(
        "SELECT
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid) +
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid)
            AS removed_count",
        catalogs.participant_identity,
        catalogs.registry_state,
        catalogs.node_descriptor,
        catalogs.generation,
        catalogs.generation_batch,
        catalogs.build_registration,
        catalogs.build_participant_binding,
        catalogs.build_candidate,
        catalogs.generation_head_state,
        catalogs.generation_head_sample,
        catalogs.publish_decision,
        catalogs.predecessor_disposition,
        catalogs.retire_decision,
        catalogs.active_epoch,
        catalogs.generation_reclaim,
        catalogs.cancelled_generation_reclaim,
    );
    let delete_before_publish = [
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.active_epoch
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.generation_reclaim
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.cancelled_generation_reclaim
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.retire_decision
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.predecessor_disposition
        ),
    ];
    let delete_after_publish = [
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.build_registration
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.node_descriptor
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.participant_identity
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.registry_state
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.generation
        ),
    ];
    Spi::connect_mut(|client| {
        let count = client
            .select(&count_sql, None, &[index_oid.into()])
            .map_err(|error| format!("ec_distann catalog cleanup count failed: {error}"))?
            .map(|row| required_i64(&row, "removed_count"))
            .next()
            .transpose()?
            .unwrap_or(0);

        for statement in &delete_before_publish {
            client
                .update(statement, None, &[index_oid.into()])
                .map_err(|error| format!("ec_distann catalog cleanup failed: {error}"))?;
        }
        let delete_publish_leaves = format!(
            "DELETE FROM {decision} AS candidate
              WHERE candidate.index_oid = $1::oid
                AND NOT EXISTS (
                    SELECT 1 FROM {decision} AS successor
                     WHERE successor.index_oid = candidate.index_oid
                       AND successor.logical_index_uuid = candidate.logical_index_uuid
                       AND successor.predecessor_build_id = candidate.build_id
                       AND successor.predecessor_epoch = candidate.epoch
                       AND successor.predecessor_epoch_fingerprint = candidate.epoch_fingerprint
                       AND successor.predecessor_manifest_digest = candidate.manifest_digest
                )",
            decision = catalogs.publish_decision,
        );
        loop {
            let deleted = client
                .update(&delete_publish_leaves, None, &[index_oid.into()])
                .map_err(|error| format!("ec_distann publish-chain cleanup failed: {error}"))?
                .len();
            if deleted == 0 {
                break;
            }
        }
        let remaining_decisions = client
            .select(
                &format!(
                    "SELECT count(*) AS remaining FROM {} WHERE index_oid = $1::oid",
                    catalogs.publish_decision
                ),
                None,
                &[index_oid.into()],
            )
            .map_err(|error| format!("ec_distann publish-chain verification failed: {error}"))?
            .map(|row| required_i64(&row, "remaining"))
            .next()
            .transpose()?
            .unwrap_or(0);
        if remaining_decisions != 0 {
            return Err(
                "EC_BUILD_STATE: publish-decision predecessor chain is cyclic or corrupt"
                    .to_owned(),
            );
        }
        for statement in &delete_after_publish {
            client
                .update(statement, None, &[index_oid.into()])
                .map_err(|error| format!("ec_distann catalog cleanup failed: {error}"))?;
        }
        Ok(count)
    })
}

/// Event-trigger target. Physical relations are internal dependents and have
/// already been removed by PostgreSQL when this function runs; only catalog
/// rows remain to be cleaned.
#[pg_extern(volatile, strict, parallel_restricted)]
fn ec_distann_catalog_index_cleanup(index_oid: pg_sys::Oid) -> i64 {
    let relation_still_exists = unsafe { pg_sys::get_rel_relkind(index_oid) } != 0;
    if relation_still_exists {
        pgrx::error!(
            "EC_BUILD_STATE: ec_distann catalog cleanup refuses a live relation; use the generation abort/rebuild operation"
        );
    }
    delete_index_catalog_rows(index_oid).unwrap_or_else(|error| pgrx::error!("{error}"))
}
