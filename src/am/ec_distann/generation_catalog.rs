//! Task 179 coordinator/participant catalog access.
//!
//! Every lookup is scoped by both the local index OID and the v5 logical UUID.
//! The UUID comes from control metadata, never from a caller or the relation
//! OID, so a stale row cannot become addressable after OID reuse.

use pgrx::datum::Uuid;
use pgrx::{pg_extern, pg_sys, Spi};

fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

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
    publish_decision: String,
    retire_decision: String,
    active_epoch: String,
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
            publish_decision: extension_relation_name("ec_distann_publish_decision")?,
            retire_decision: extension_relation_name("ec_distann_retire_decision")?,
            active_epoch: extension_relation_name("ec_distann_active_epoch")?,
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
    pub(crate) state: String,
    pub(crate) build_spec_digest: [u8; 32],
    pub(crate) roster_digest: [u8; 32],
    pub(crate) generation_descriptor: Vec<u8>,
    pub(crate) generation_descriptor_digest: [u8; 32],
    pub(crate) expected_owner_count: u64,
    pub(crate) expected_owner_digest: [u8; 32],
    pub(crate) row_tier_relid: pg_sys::Oid,
    pub(crate) graph_store_relid: pg_sys::Oid,
    pub(crate) directory_relid: pg_sys::Oid,
    pub(crate) next_batch_seq: u64,
    pub(crate) cumulative_record_count: u64,
    pub(crate) cumulative_owner_digest: [u8; 32],
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

fn fixed_digest(bytes: Vec<u8>, name: &str) -> Result<[u8; 32], String> {
    bytes.try_into().map_err(|bytes: Vec<u8>| {
        format!(
            "ec_distann generation catalog {name} is {} bytes, expected 32",
            bytes.len()
        )
    })
}

fn decode_generation_row(
    row: pgrx::spi::SpiHeapTupleData<'_>,
) -> Result<GenerationCatalogRow, String> {
    Ok(GenerationCatalogRow {
        epoch: u64::try_from(required_i64(&row, "epoch")?)
            .map_err(|_| "ec_distann generation catalog epoch is negative".to_owned())?,
        owner_ordinal: u32::try_from(required_i32(&row, "owner_ordinal")?)
            .map_err(|_| "ec_distann generation owner ordinal is negative".to_owned())?,
        node_id: u32::try_from(required_i32(&row, "node_id")?)
            .map_err(|_| "ec_distann generation node id is negative".to_owned())?,
        state: required_string(&row, "state")?,
        build_spec_digest: fixed_digest(
            required_bytes(&row, "build_spec_digest")?,
            "build_spec_digest",
        )?,
        roster_digest: fixed_digest(required_bytes(&row, "roster_digest")?, "roster_digest")?,
        generation_descriptor: required_bytes(&row, "generation_descriptor")?,
        generation_descriptor_digest: fixed_digest(
            required_bytes(&row, "generation_descriptor_digest")?,
            "generation_descriptor_digest",
        )?,
        expected_owner_count: u64::try_from(required_i64(&row, "expected_owner_count")?)
            .map_err(|_| "ec_distann expected owner count is negative".to_owned())?,
        expected_owner_digest: fixed_digest(
            required_bytes(&row, "expected_owner_digest")?,
            "expected_owner_digest",
        )?,
        row_tier_relid: required_oid(&row, "row_tier_relid")?,
        graph_store_relid: required_oid(&row, "graph_store_relid")?,
        directory_relid: required_oid(&row, "directory_relid")?,
        next_batch_seq: u64::try_from(required_i64(&row, "next_batch_seq")?)
            .map_err(|_| "ec_distann next batch sequence is negative".to_owned())?,
        cumulative_record_count: u64::try_from(required_i64(&row, "cumulative_record_count")?)
            .map_err(|_| "ec_distann cumulative record count is negative".to_owned())?,
        cumulative_owner_digest: fixed_digest(
            required_bytes(&row, "cumulative_owner_digest")?,
            "cumulative_owner_digest",
        )?,
    })
}

pub(crate) fn lookup_generation(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
) -> Result<Option<GenerationCatalogRow>, String> {
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "SELECT epoch, owner_ordinal, node_id, state,
                build_spec_digest, roster_digest, generation_descriptor,
                generation_descriptor_digest, expected_owner_count,
                expected_owner_digest, row_tier_relid, graph_store_relid,
                directory_relid, next_batch_seq, cumulative_record_count,
                cumulative_owner_digest
           FROM {}
          WHERE index_oid = $1::oid
            AND logical_index_uuid = $2::uuid
            AND build_id = $3::uuid",
        catalogs.generation
    );
    Spi::connect(|client| {
        client
            .select(
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
             expected_owner_digest, row_tier_relid, graph_store_relid,
             directory_relid, next_batch_seq, cumulative_record_count,
             cumulative_owner_digest
         ) VALUES (
             $1::oid, $2::uuid, $3::uuid, $4::bigint,
             $5::integer, $6::integer, $7::text, $8::bytea,
             $9::bytea, $10::bytea, $11::bytea, $12::bigint,
             $13::bytea, $14::oid, $15::oid, $16::oid,
             $17::bigint, $18::bigint, $19::bytea
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
                    row.state.clone().into(),
                    row.build_spec_digest.to_vec().into(),
                    row.roster_digest.to_vec().into(),
                    row.generation_descriptor.clone().into(),
                    row.generation_descriptor_digest.to_vec().into(),
                    expected_owner_count.into(),
                    row.expected_owner_digest.to_vec().into(),
                    row.row_tier_relid.into(),
                    row.graph_store_relid.into(),
                    row.directory_relid.into(),
                    next_batch_seq.into(),
                    cumulative_record_count.into(),
                    row.cumulative_owner_digest.to_vec().into(),
                ],
            )
            .map_err(|error| format!("ec_distann generation catalog insert failed: {error}"))?;
        Ok(())
    })
}

pub(crate) fn generation_relations_for_index(
    index_oid: pg_sys::Oid,
) -> Result<Vec<(pg_sys::Oid, pg_sys::Oid, pg_sys::Oid)>, String> {
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "SELECT row_tier_relid, graph_store_relid, directory_relid
           FROM {}
          WHERE index_oid = $1::oid",
        catalogs.generation
    );
    Spi::connect(|client| {
        client
            .select(&sql, None, &[index_oid.into()])
            .map_err(|error| format!("ec_distann generation relation lookup failed: {error}"))?
            .map(|row| {
                Ok((
                    required_oid(&row, "row_tier_relid")?,
                    required_oid(&row, "graph_store_relid")?,
                    required_oid(&row, "directory_relid")?,
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

pub(crate) fn delete_generation(
    index_oid: pg_sys::Oid,
    logical_index_uuid: Uuid,
    build_id: Uuid,
) -> Result<(), String> {
    let catalogs = CatalogRelations::resolve()?;
    let sql = format!(
        "DELETE FROM {}
          WHERE index_oid = $1::oid
            AND logical_index_uuid = $2::uuid
            AND build_id = $3::uuid",
        catalogs.generation
    );
    Spi::connect_mut(|client| {
        client
            .update(
                &sql,
                None,
                &[index_oid.into(), logical_index_uuid.into(), build_id.into()],
            )
            .map_err(|error| format!("ec_distann generation delete failed: {error}"))?;
        Ok(())
    })
}

pub(crate) fn delete_index_catalog_rows(index_oid: pg_sys::Oid) -> Result<i64, String> {
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
            (SELECT count(*) FROM {} WHERE index_oid = $1::oid)
            AS removed_count",
        catalogs.participant_identity,
        catalogs.registry_state,
        catalogs.node_descriptor,
        catalogs.generation,
        catalogs.generation_batch,
        catalogs.build_registration,
        catalogs.build_participant_binding,
        catalogs.publish_decision,
        catalogs.retire_decision,
        catalogs.active_epoch,
    );
    let delete_statements = [
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.active_epoch
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.retire_decision
        ),
        format!(
            "DELETE FROM {} WHERE index_oid = $1::oid",
            catalogs.publish_decision
        ),
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

        for statement in &delete_statements {
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
