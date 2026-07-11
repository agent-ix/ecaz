//! Task 179 transactional physical-generation relation lifecycle.
//!
//! V1 deliberately uses ordinary permanent heap/B-tree relations: PostgreSQL
//! supplies WAL, MVCC rollback, TOAST, dependency cleanup, and crash behavior
//! while the handoff protocol is still being built. Local relation OIDs never
//! enter a descriptor or remote response.

use std::collections::HashSet;
use std::ffi::CStr;
use std::ptr::NonNull;

use pgrx::datum::{TimestampWithTimeZone, Uuid};
use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern, pg_sys, PgRelation, Spi};

use crate::storage::relation::{
    index_heap_relation_oid_handle, relation_am_oid_handle,
    relation_namespace_owner_persistence_handle, relation_oid_handle, relation_tablespace_handle,
    RelationHandle,
};
use crate::storage::relation_guard::IndexRelationGuard;

use super::ambuild::read_metadata_from_index_handle;
use super::canonical_wire::{domain_digest, is_rfc4122_v4_uuid, CanonicalEncoder};
use super::generation_catalog::{self, GenerationCatalogRow};
use super::generation_descriptor::DistannGenerationDescriptor;
use super::handoff_wire::{owner_stream_digest, DistannHandoffShape, DistannOwnerStreamHasher};
use super::page::{DistannMetadataPage, INDEX_FORMAT_V5_DISTANN_CONTROL};
use super::quantizer::DistannCodecBinding;
use super::roster_digest as canonical_roster_digest;
use super::row_schema::{resolve_relation_schema, ResolvedRowSchema};

const CONTROL_COMPATIBILITY_VERSION: u16 = 1;
const CONTROL_COMPATIBILITY_DOMAIN: &[u8] = b"ec_distann_control_compatibility_v1\0";

fn canonical_control_compatibility_digest(
    metadata: &DistannMetadataPage,
    key_attnum: u16,
    key_kind: u8,
    identity_attnum: u16,
    identity_kind: u8,
    row_schema_fingerprint: &[u8; 32],
) -> Result<[u8; 32], String> {
    let mut encoder = CanonicalEncoder::with_capacity(64);
    encoder.put_u16(CONTROL_COMPATIBILITY_VERSION);
    encoder.put_u16(metadata.graph_degree_r);
    encoder.put_u16(metadata.build_list_size_l);
    encoder.put_f32(metadata.alpha);
    encoder.put_u64(metadata.seed);
    encoder.put_u8(metadata.neighbor_codec_kind);
    encoder.put_u32(metadata.head_index_cap);
    encoder.put_f32(metadata.closure_epsilon);
    encoder.put_u8(1); // source_identity = include
    encoder.put_u16(key_attnum);
    encoder.put_u8(key_kind);
    encoder.put_u16(identity_attnum);
    encoder.put_u8(identity_kind);
    encoder.put_u8(1); // identity attnotnull = true
    encoder.put_fixed(row_schema_fingerprint);
    Ok(domain_digest(
        CONTROL_COMPATIBILITY_DOMAIN,
        &encoder.finish()?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GenerationRelations {
    row_tier_relid: pg_sys::Oid,
    graph_store_relid: pg_sys::Oid,
    directory_relid: pg_sys::Oid,
}

fn fixed_digest(bytes: Vec<u8>, category: &str, field: &str) -> Result<[u8; 32], String> {
    bytes.try_into().map_err(|bytes: Vec<u8>| {
        format!("{category}: {field} is {} bytes, expected 32", bytes.len())
    })
}

fn relation_exists(relation_oid: pg_sys::Oid) -> bool {
    relation_oid != pg_sys::InvalidOid && unsafe { pg_sys::get_rel_relkind(relation_oid) } != 0
}

pub(crate) fn open_control_index(
    index_oid: pg_sys::Oid,
    lockmode: pg_sys::LOCKMODE,
    caller: &'static str,
) -> Result<
    (
        IndexRelationGuard,
        RelationHandle,
        DistannMetadataPage,
        Uuid,
    ),
    String,
> {
    let guard = IndexRelationGuard::try_open(index_oid, lockmode)
        .ok_or_else(|| format!("{caller}: could not open index relation"))?;
    let handle = NonNull::new(guard.as_ptr())
        .ok_or_else(|| format!("{caller}: opened a null index relation"))?;
    let expected_am = unsafe { pg_sys::get_am_oid(c"ec_distann".as_ptr(), false) };
    if relation_am_oid_handle(handle) != expected_am {
        return Err(format!("{caller}: relation is not an ec_distann index"));
    }
    let metadata = read_metadata_from_index_handle(handle)?;
    if metadata.format_version != INDEX_FORMAT_V5_DISTANN_CONTROL
        || !metadata.is_distributed_control()
    {
        return Err(format!(
            "EC_GENERATION_DESCRIPTOR: {caller} requires distributed_control v5 metadata"
        ));
    }
    let logical_index_uuid = Uuid::from_bytes(metadata.logical_index_uuid);
    Ok((guard, handle, metadata, logical_index_uuid))
}

/// Canonical registration-time identity for every control property that must
/// agree before a coordinator can place rows on a participant. The logical
/// UUID is deliberately excluded: distinct participant controls have distinct
/// UUIDs while sharing this compatibility identity.
pub(crate) fn control_compatibility_digest(
    index_handle: RelationHandle,
    metadata: &DistannMetadataPage,
) -> Result<[u8; 32], String> {
    let index_oid = relation_oid_handle(index_handle);
    let heap_oid = index_heap_relation_oid_handle(index_handle);
    if heap_oid == pg_sys::InvalidOid {
        return Err("EC_SCHEMA_MISMATCH: control index has no source relation".to_owned());
    }
    let options = super::options::relation_options(index_handle.as_ptr());
    if !options.distributed_control
        || options.source_identity != super::options::DistannSourceIdentityProvider::Include
    {
        return Err(
            "EC_NODE_DESCRIPTOR: control reloptions require distributed_control=true and source_identity='include'"
                .to_owned(),
        );
    }

    let (key_attnum, key_kind, identity_attnum, identity_type_oid) = Spi::connect(|client| {
        client
            .select(
                "SELECT i.indnkeyatts::int4 AS key_count,
                        i.indnatts::int4 AS total_count,
                        i.indkey[0]::int4 AS key_attnum,
                        i.indkey[1]::int4 AS identity_attnum,
                        a.atttypid AS identity_type_oid,
                        a.attnotnull AS identity_not_null,
                        i.indisvalid AS index_valid,
                        i.indisready AS index_ready,
                        i.indislive AS index_live,
                        CASE
                          WHEN opc.opcnamespace = ext.extnamespace
                           AND opc.opcname = 'ecvector_distann_ip_ops' THEN 1
                          WHEN opc.opcnamespace = ext.extnamespace
                           AND opc.opcname = 'tqvector_distann_ip_ops' THEN 2
                          ELSE 0
                        END::int4 AS key_kind
                   FROM pg_catalog.pg_index i
                   JOIN pg_catalog.pg_attribute a
                     ON a.attrelid = i.indrelid
                    AND a.attnum = i.indkey[1]
                   JOIN pg_catalog.pg_opclass opc ON opc.oid = i.indclass[0]
                   JOIN pg_catalog.pg_extension ext ON ext.extname = 'ecaz'
                  WHERE i.indexrelid = $1::oid",
                None,
                &[index_oid.into()],
            )
            .map_err(|error| format!("EC_NODE_DESCRIPTOR: index identity lookup failed: {error}"))?
            .map(|row| {
                let required_i32 = |name: &str| -> Result<i32, String> {
                    row[name]
                        .value::<i32>()
                        .map_err(|error| {
                            format!("EC_NODE_DESCRIPTOR: {name} decode failed: {error}")
                        })?
                        .ok_or_else(|| format!("EC_NODE_DESCRIPTOR: {name} is NULL"))
                };
                let key_count = required_i32("key_count")?;
                let total_count = required_i32("total_count")?;
                let key_attnum = required_i32("key_attnum")?;
                let identity_attnum = required_i32("identity_attnum")?;
                let key_kind = required_i32("key_kind")?;
                let identity_type_oid = row["identity_type_oid"]
                    .value::<pg_sys::Oid>()
                    .map_err(|error| {
                        format!("EC_NODE_DESCRIPTOR: identity type decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_NODE_DESCRIPTOR: identity type is NULL".to_owned())?;
                let required_bool = |name: &str| -> Result<bool, String> {
                    row[name]
                        .value::<bool>()
                        .map_err(|error| {
                            format!("EC_NODE_DESCRIPTOR: {name} decode failed: {error}")
                        })?
                        .ok_or_else(|| format!("EC_NODE_DESCRIPTOR: {name} is NULL"))
                };
                if key_count != 1
                    || total_count != 2
                    || key_attnum <= 0
                    || identity_attnum <= 0
                    || !matches!(key_kind, 1 | 2)
                    || !required_bool("identity_not_null")?
                    || !required_bool("index_valid")?
                    || !required_bool("index_ready")?
                    || !required_bool("index_live")?
                {
                    return Err(
                        "EC_NODE_DESCRIPTOR: control index key/opclass/identity/readiness contract is incompatible"
                            .to_owned(),
                    );
                }
                Ok((key_attnum, key_kind, identity_attnum, identity_type_oid))
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_NODE_DESCRIPTOR: control index identity row is absent".to_owned())
    })?;
    let key_attnum = u16::try_from(key_attnum)
        .map_err(|_| "EC_NODE_DESCRIPTOR: key attnum exceeds u16".to_owned())?;
    let key_kind =
        u8::try_from(key_kind).map_err(|_| "EC_NODE_DESCRIPTOR: key kind exceeds u8".to_owned())?;
    let identity_attnum = u16::try_from(identity_attnum)
        .map_err(|_| "EC_NODE_DESCRIPTOR: identity attnum exceeds u16".to_owned())?;
    let identity_kind = match crate::storage::type_info::base_type_oid(identity_type_oid) {
        pg_sys::UUIDOID => 1,
        pg_sys::BYTEAOID => 2,
        _ => {
            return Err(
                "EC_NODE_DESCRIPTOR: source identity base type must be uuid or bytea16".to_owned(),
            )
        }
    };
    let row_schema_fingerprint = resolve_relation_schema(heap_oid)?
        .descriptor
        .fingerprint()?;

    canonical_control_compatibility_digest(
        metadata,
        key_attnum,
        key_kind,
        identity_attnum,
        identity_kind,
        &row_schema_fingerprint,
    )
}

fn validate_descriptor_for_control(
    index_handle: RelationHandle,
    metadata: &DistannMetadataPage,
    descriptor_bytes: &[u8],
    expected_descriptor_digest: [u8; 32],
    expected_roster_digest: [u8; 32],
) -> Result<(DistannGenerationDescriptor, ResolvedRowSchema, u32, u32), String> {
    let descriptor = DistannGenerationDescriptor::decode(descriptor_bytes)?;
    if descriptor.digest()? != expected_descriptor_digest {
        return Err("EC_GENERATION_DESCRIPTOR: descriptor digest mismatch".to_owned());
    }
    if canonical_roster_digest(&descriptor.roster)? != expected_roster_digest {
        return Err("EC_NODE_DESCRIPTOR: roster digest mismatch".to_owned());
    }
    if descriptor.graph_degree != metadata.graph_degree_r
        || descriptor.neighbor_codec_kind != metadata.neighbor_codec_kind
        || descriptor.codec_artifact.seed() != metadata.seed
    {
        return Err(
            "EC_GENERATION_DESCRIPTOR: descriptor shape does not match control reloptions"
                .to_owned(),
        );
    }

    let local_uuid = metadata.logical_index_uuid;
    let mut matches = descriptor
        .roster
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.logical_index_uuid == local_uuid);
    let (owner_ordinal, owner) = matches
        .next()
        .ok_or_else(|| "EC_NODE_DESCRIPTOR: local logical UUID is absent from roster".to_owned())?;
    if matches.next().is_some() {
        return Err("EC_NODE_DESCRIPTOR: local logical UUID is ambiguous in roster".to_owned());
    }

    let heap_oid = index_heap_relation_oid_handle(index_handle);
    if heap_oid == pg_sys::InvalidOid {
        return Err("EC_SCHEMA_MISMATCH: control index has no source relation".to_owned());
    }
    let resolved_schema = resolve_relation_schema(heap_oid)?;
    if resolved_schema.descriptor != descriptor.row_schema {
        return Err(
            "EC_SCHEMA_MISMATCH: local source shell differs from generation descriptor".to_owned(),
        );
    }
    let node_id = owner.node_id;
    Ok((
        descriptor,
        resolved_schema,
        u32::try_from(owner_ordinal)
            .map_err(|_| "EC_NODE_DESCRIPTOR: owner ordinal exceeds u32".to_owned())?,
        node_id,
    ))
}

fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn cstring_owned(pointer: *mut std::ffi::c_char, context: &str) -> Result<String, String> {
    if pointer.is_null() {
        return Err(format!("ec_distann could not resolve {context}"));
    }
    let value = unsafe { CStr::from_ptr(pointer) }
        .to_str()
        .map_err(|_| format!("ec_distann {context} is not UTF-8"))?
        .to_owned();
    unsafe { pg_sys::pfree(pointer.cast()) };
    Ok(value)
}

fn generation_relation_names(index_oid: pg_sys::Oid, build_id: Uuid) -> (String, String, String) {
    let suffix = hex::encode(build_id.as_bytes());
    let oid = u32::from(index_oid);
    (
        format!("_ecdz_row_{oid}_{suffix}"),
        format!("_ecdz_graph_{oid}_{suffix}"),
        format!("_ecdz_dir_{oid}_{suffix}"),
    )
}

fn tablespace_clause(index_handle: RelationHandle) -> Result<String, String> {
    let explicit_tablespace_oid = relation_tablespace_handle(index_handle);
    // reltablespace=0 means the database default. Spell the effective
    // tablespace explicitly so a SECURITY DEFINER caller's default_tablespace
    // GUC cannot redirect hidden generation storage away from its control.
    let tablespace_oid = if explicit_tablespace_oid == pg_sys::InvalidOid {
        unsafe { pg_sys::MyDatabaseTableSpace }
    } else {
        explicit_tablespace_oid
    };
    if tablespace_oid == pg_sys::InvalidOid {
        return Err("EC_CONTROL_PERSISTENCE: database tablespace is invalid".to_owned());
    }
    let name = cstring_owned(
        unsafe { pg_sys::get_tablespace_name(tablespace_oid) },
        "index tablespace name",
    )?;
    Ok(format!(" TABLESPACE {}", quote_ident(&name)))
}

fn row_tier_column_sql(
    schema: &ResolvedRowSchema,
    build_id: Uuid,
) -> Result<(String, Vec<String>), String> {
    let mut used_names = schema
        .columns
        .iter()
        .filter(|column| !column.dropped)
        .map(|column| column.name.clone())
        .collect::<HashSet<_>>();
    let mut definitions = Vec::with_capacity(schema.columns.len());
    let mut dropped_names = Vec::new();
    let build_prefix = &hex::encode(build_id.as_bytes())[..8];
    for column in &schema.columns {
        if column.dropped {
            let mut candidate = format!("__ecdz_drop_{}_{}", column.attnum, build_prefix);
            while used_names.contains(&candidate) {
                candidate.push('_');
            }
            used_names.insert(candidate.clone());
            definitions.push(format!("{} bytea", quote_ident(&candidate)));
            dropped_names.push(candidate);
            continue;
        }
        if column.sql_type.is_empty() {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: attribute {} has no local SQL type",
                column.attnum
            ));
        }
        let mut definition = format!("{} {}", quote_ident(&column.name), column.sql_type);
        if let Some(collation) = &column.collation_sql {
            definition.push_str(" COLLATE ");
            definition.push_str(collation);
        }
        definitions.push(definition);
    }
    Ok((definitions.join(", "), dropped_names))
}

fn relation_oid_by_name(namespace_oid: pg_sys::Oid, name: &str) -> Result<pg_sys::Oid, String> {
    let name = std::ffi::CString::new(name)
        .map_err(|_| "ec_distann internal relation name contains NUL".to_owned())?;
    let oid = unsafe { pg_sys::get_relname_relid(name.as_ptr(), namespace_oid) };
    if oid == pg_sys::InvalidOid {
        return Err("ec_distann created relation is absent from pg_class".to_owned());
    }
    Ok(oid)
}

fn record_internal_dependency(dependent_oid: pg_sys::Oid, control_oid: pg_sys::Oid) {
    let dependent = pg_sys::ObjectAddress {
        classId: pg_sys::RelationRelationId,
        objectId: dependent_oid,
        objectSubId: 0,
    };
    let control = pg_sys::ObjectAddress {
        classId: pg_sys::RelationRelationId,
        objectId: control_oid,
        objectSubId: 0,
    };
    unsafe {
        pg_sys::recordDependencyOn(
            &dependent,
            &control,
            pg_sys::DependencyType::DEPENDENCY_INTERNAL,
        )
    };
}

fn create_generation_relations(
    index_handle: RelationHandle,
    build_id: Uuid,
    schema: &ResolvedRowSchema,
) -> Result<GenerationRelations, String> {
    let index_oid = relation_oid_handle(index_handle);
    let (namespace_oid, owner_oid, persistence) =
        relation_namespace_owner_persistence_handle(index_handle);
    if persistence != pg_sys::RELPERSISTENCE_PERMANENT as std::ffi::c_char {
        return Err("EC_CONTROL_PERSISTENCE: generation storage must be permanent".to_owned());
    }
    let namespace = cstring_owned(
        unsafe { pg_sys::get_namespace_name(namespace_oid) },
        "control-index namespace",
    )?;
    let owner = cstring_owned(
        unsafe { pg_sys::GetUserNameFromId(owner_oid, false) },
        "control-index owner",
    )?;
    let tablespace = tablespace_clause(index_handle)?;
    let (row_name, graph_name, directory_name) = generation_relation_names(index_oid, build_id);
    for name in [&row_name, &graph_name, &directory_name] {
        if relation_oid_by_name(namespace_oid, name).is_ok() {
            return Err(
                "EC_GENERATION_MISSING: deterministic generation relation exists without matching catalog identity"
                    .to_owned(),
            );
        }
    }
    let qualified_row = format!("{}.{}", quote_ident(&namespace), quote_ident(&row_name));
    let qualified_graph = format!("{}.{}", quote_ident(&namespace), quote_ident(&graph_name));
    let (row_columns, dropped_columns) = row_tier_column_sql(schema, build_id)?;

    Spi::run(&format!(
        "CREATE TABLE {qualified_row} ({row_columns}){tablespace}"
    ))
    .map_err(|error| format!("ec_distann row-tier relation creation failed: {error}"))?;
    for dropped in dropped_columns {
        Spi::run(&format!(
            "ALTER TABLE {qualified_row} DROP COLUMN {}",
            quote_ident(&dropped)
        ))
        .map_err(|error| format!("ec_distann dropped-column preservation failed: {error}"))?;
    }
    Spi::run(&format!(
        "CREATE TABLE {qualified_graph} (\
             vec_id bigint NOT NULL, \
             graph_record bytea NOT NULL, \
             row_tid tid NOT NULL\
         ){tablespace}"
    ))
    .map_err(|error| format!("ec_distann graph-store relation creation failed: {error}"))?;
    Spi::run(&format!(
        // PostgreSQL requires an unqualified index name here.  The target
        // table's namespace determines the index namespace.
        "CREATE UNIQUE INDEX {} ON {qualified_graph} (vec_id){tablespace}",
        quote_ident(&directory_name)
    ))
    .map_err(|error| format!("ec_distann directory creation failed: {error}"))?;
    Spi::run(&format!(
        "ALTER TABLE {qualified_row} OWNER TO {}",
        quote_ident(&owner)
    ))
    .map_err(|error| format!("ec_distann row-tier ownership change failed: {error}"))?;
    Spi::run(&format!(
        "ALTER TABLE {qualified_graph} OWNER TO {}",
        quote_ident(&owner)
    ))
    .map_err(|error| format!("ec_distann graph-store ownership change failed: {error}"))?;

    let relations = GenerationRelations {
        row_tier_relid: relation_oid_by_name(namespace_oid, &row_name)?,
        graph_store_relid: relation_oid_by_name(namespace_oid, &graph_name)?,
        directory_relid: relation_oid_by_name(namespace_oid, &directory_name)?,
    };
    record_internal_dependency(relations.row_tier_relid, index_oid);
    record_internal_dependency(relations.graph_store_relid, index_oid);
    record_internal_dependency(relations.directory_relid, index_oid);
    unsafe { pg_sys::CommandCounterIncrement() };
    Ok(relations)
}

fn drop_relation_internal(
    relation_oid: pg_sys::Oid,
    control_oid: pg_sys::Oid,
) -> Result<(), String> {
    if !relation_exists(relation_oid) {
        return Ok(());
    }
    let deleted = unsafe {
        pg_sys::deleteDependencyRecordsForSpecific(
            pg_sys::RelationRelationId,
            relation_oid,
            pg_sys::DependencyType::DEPENDENCY_INTERNAL as std::ffi::c_char,
            pg_sys::RelationRelationId,
            control_oid,
        )
    };
    if deleted != 1 {
        return Err(format!(
            "EC_GENERATION_MISSING: physical relation {} has {deleted} internal control dependencies, expected 1",
            u32::from(relation_oid)
        ));
    }
    unsafe { pg_sys::CommandCounterIncrement() };
    let object = pg_sys::ObjectAddress {
        classId: pg_sys::RelationRelationId,
        objectId: relation_oid,
        objectSubId: 0,
    };
    unsafe { pg_sys::performDeletion(&object, pg_sys::DropBehavior::DROP_RESTRICT, 0) };
    Ok(())
}

fn drop_generation_relations(
    control_oid: pg_sys::Oid,
    relations: GenerationRelations,
) -> Result<(), String> {
    drop_relation_internal(relations.directory_relid, control_oid)?;
    drop_relation_internal(relations.graph_store_relid, control_oid)?;
    drop_relation_internal(relations.row_tier_relid, control_oid)?;
    unsafe { pg_sys::CommandCounterIncrement() };
    Ok(())
}

fn validate_replay(
    row: &GenerationCatalogRow,
    epoch: u64,
    owner_ordinal: u32,
    node_id: u32,
    build_spec_digest: [u8; 32],
    roster_digest: [u8; 32],
    descriptor: &[u8],
    descriptor_digest: [u8; 32],
    expected_owner_count: u64,
    expected_owner_digest: [u8; 32],
) -> Result<(), String> {
    if row.epoch != epoch
        || row.owner_ordinal != owner_ordinal
        || row.node_id != node_id
        || row.build_spec_digest != build_spec_digest
        || row.roster_digest != roster_digest
        || row.generation_descriptor != descriptor
        || row.generation_descriptor_digest != descriptor_digest
        || row.expected_owner_count != expected_owner_count
        || row.expected_owner_digest != expected_owner_digest
    {
        return Err(
            "EC_BUILD_ID_CONFLICT: build id was reused with different immutable inputs".to_owned(),
        );
    }
    if !matches!(row.state.as_str(), "Building" | "Ready") {
        return Err(format!(
            "EC_BUILD_STATE: begin cannot replay generation state {}",
            row.state
        ));
    }
    for relation_oid in [
        row.row_tier_relid,
        row.graph_store_relid,
        row.directory_relid,
    ] {
        if !relation_exists(relation_oid) {
            return Err(
                "EC_GENERATION_MISSING: cataloged generation relation is missing".to_owned(),
            );
        }
    }
    Ok(())
}

type BeginResult = (
    name!(state, String),
    name!(next_batch_seq, i64),
    name!(cumulative_record_count, i64),
    name!(cumulative_owner_digest, Vec<u8>),
);

fn begin_result(row: &GenerationCatalogRow) -> BeginResult {
    (
        row.state.clone(),
        i64::try_from(row.next_batch_seq).expect("catalog sequence fits bigint"),
        i64::try_from(row.cumulative_record_count).expect("catalog count fits bigint"),
        row.cumulative_owner_digest.to_vec(),
    )
}

#[pg_extern(stable, strict, parallel_restricted)]
fn ec_distann_control_identity(
    index_regclass: PgRelation,
) -> TableIterator<
    'static,
    (
        name!(logical_index_uuid, Uuid),
        name!(index_format_version, i32),
        name!(distributed_control, bool),
        name!(compatibility_digest, Vec<u8>),
        name!(endpoint_identity, Option<String>),
        name!(canonical_index_regclass, String),
    ),
> {
    let index_oid = index_regclass.oid();
    let (_guard, handle, metadata, logical_index_uuid) = open_control_index(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "ec_distann_control_identity",
    )
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    let compatibility_digest = control_compatibility_digest(handle, &metadata)
        .unwrap_or_else(|error| pgrx::error!("{error}"));
    let participant_identity =
        generation_catalog::extension_relation_name("ec_distann_participant_identity")
            .unwrap_or_else(|error| pgrx::error!("{error}"));
    let (endpoint_identity, canonical_index_regclass) = Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT pi.endpoint_identity,
                            pg_catalog.format('%I.%I', n.nspname, c.relname)
                               AS canonical_index_regclass
                       FROM pg_catalog.pg_class c
                       JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                       LEFT JOIN {participant_identity} pi
                         ON pi.index_oid = c.oid
                        AND pi.logical_index_uuid = $2::uuid
                      WHERE c.oid = $1::oid"
                ),
                None,
                &[index_oid.into(), logical_index_uuid.into()],
            )
            .map_err(|_| "EC_NODE_DESCRIPTOR: control identity catalog lookup failed".to_owned())?
            .map(|row| {
                let endpoint_identity =
                    row["endpoint_identity"].value::<String>().map_err(|_| {
                        "EC_NODE_DESCRIPTOR: configured endpoint identity decode failed".to_owned()
                    })?;
                let canonical_index_regclass = row["canonical_index_regclass"]
                    .value::<String>()
                    .map_err(|_| {
                        "EC_NODE_DESCRIPTOR: canonical index locator decode failed".to_owned()
                    })?
                    .ok_or_else(|| {
                        "EC_NODE_DESCRIPTOR: canonical index locator is NULL".to_owned()
                    })?;
                super::node_registry::validate_canonical_index_locator(&canonical_index_regclass)?;
                Ok::<(Option<String>, String), String>((
                    endpoint_identity,
                    canonical_index_regclass,
                ))
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_NODE_DESCRIPTOR: control identity relation is absent".to_owned())
    })
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::once((
        logical_index_uuid,
        i32::from(metadata.format_version),
        metadata.is_distributed_control(),
        compatibility_digest.to_vec(),
        endpoint_identity,
        canonical_index_regclass,
    ))
}

#[pg_extern(volatile, strict, parallel_restricted)]
#[allow(clippy::too_many_arguments)]
fn ec_distann_begin_epoch_handoff(
    index_regclass: PgRelation,
    epoch: i64,
    build_id: Uuid,
    build_spec_digest: Vec<u8>,
    roster_digest: Vec<u8>,
    generation_descriptor: Vec<u8>,
    generation_descriptor_digest: Vec<u8>,
    expected_owner_count: i64,
    expected_owner_digest: Vec<u8>,
) -> TableIterator<
    'static,
    (
        name!(state, String),
        name!(next_batch_seq, i64),
        name!(cumulative_record_count, i64),
        name!(cumulative_owner_digest, Vec<u8>),
    ),
> {
    let result = (|| -> Result<GenerationCatalogRow, String> {
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let epoch = u64::try_from(epoch)
            .ok()
            .filter(|epoch| *epoch > 0)
            .ok_or_else(|| "EC_BUILD_STATE: epoch must be positive".to_owned())?;
        let expected_owner_count = u64::try_from(expected_owner_count)
            .map_err(|_| "EC_BUILD_INCOMPLETE: expected owner count is negative".to_owned())?;
        let build_spec_digest = fixed_digest(
            build_spec_digest,
            "EC_GENERATION_DESCRIPTOR",
            "build spec digest",
        )?;
        let provided_roster_digest =
            fixed_digest(roster_digest, "EC_NODE_DESCRIPTOR", "roster digest")?;
        let descriptor_digest = fixed_digest(
            generation_descriptor_digest,
            "EC_GENERATION_DESCRIPTOR",
            "generation descriptor digest",
        )?;
        let expected_owner_digest = fixed_digest(
            expected_owner_digest,
            "EC_BUILD_INCOMPLETE",
            "expected owner digest",
        )?;
        let index_oid = index_regclass.oid();
        let (_guard, handle, metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_begin_epoch_handoff",
        )?;
        let (descriptor, resolved_schema, owner_ordinal, node_id) =
            validate_descriptor_for_control(
                handle,
                &metadata,
                &generation_descriptor,
                descriptor_digest,
                provided_roster_digest,
            )?;

        if let Some(existing) =
            generation_catalog::lookup_generation(index_oid, logical_index_uuid, build_id)?
        {
            validate_replay(
                &existing,
                epoch,
                owner_ordinal,
                node_id,
                build_spec_digest,
                provided_roster_digest,
                &generation_descriptor,
                descriptor_digest,
                expected_owner_count,
                expected_owner_digest,
            )?;
            return Ok(existing);
        }

        let binding = DistannCodecBinding::from_artifact(&descriptor.codec_artifact)?;
        let shape = DistannHandoffShape {
            code_stride: binding.code_len(usize::from(descriptor.dimensions))?,
            graph_degree: usize::from(descriptor.graph_degree),
            non_dropped_attribute_count: descriptor.row_schema.non_dropped_count(),
        };
        let initial_owner_hasher = DistannOwnerStreamHasher::new();
        let initial_owner_digest = owner_stream_digest(&[], shape)?;
        if initial_owner_hasher.digest() != initial_owner_digest {
            return Err(
                "EC_HANDOFF_DIGEST: initial owner-stream state disagrees with canonical digest"
                    .to_owned(),
            );
        }
        let relations = create_generation_relations(handle, build_id, &resolved_schema)?;
        let row = GenerationCatalogRow {
            epoch,
            owner_ordinal,
            node_id,
            state: "Building".to_owned(),
            build_spec_digest,
            roster_digest: provided_roster_digest,
            generation_descriptor: generation_descriptor.clone(),
            generation_descriptor_digest: descriptor_digest,
            expected_owner_count,
            expected_owner_digest,
            row_tier_relid: relations.row_tier_relid,
            graph_store_relid: relations.graph_store_relid,
            directory_relid: relations.directory_relid,
            next_batch_seq: 0,
            cumulative_record_count: 0,
            cumulative_owner_digest: initial_owner_digest,
            last_vec_id_le: None,
            owner_stream_sha256_state: initial_owner_hasher.serialize(),
            ready_receipt: None,
        };
        generation_catalog::insert_generation(index_oid, logical_index_uuid, build_id, &row)?;
        Ok(row)
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::once(begin_result(&result))
}

#[pg_extern(volatile, strict, parallel_restricted)]
fn ec_distann_abort_epoch_handoff(index_regclass: PgRelation, build_id: Uuid) {
    let result = (|| -> Result<(), String> {
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();
        let (_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_abort_epoch_handoff",
        )?;
        let Some(row) = generation_catalog::lookup_generation_for_update(
            index_oid,
            logical_index_uuid,
            build_id,
        )?
        else {
            return Ok(());
        };
        // The future publish-decision path must take this same control-index
        // ShareRowExclusiveLock before inserting its decision. Together with
        // the generation-row lock above, that makes this check and the final
        // guarded delete one serialized abort-vs-decision boundary.
        if matches!(row.state.as_str(), "Published" | "Retired")
            || generation_catalog::has_publish_decision(index_oid, logical_index_uuid, build_id)?
        {
            return Err(
                "EC_BUILD_STATE: abort refuses a published/decision-referenced generation"
                    .to_owned(),
            );
        }
        drop_generation_relations(
            index_oid,
            GenerationRelations {
                row_tier_relid: row.row_tier_relid,
                graph_store_relid: row.graph_store_relid,
                directory_relid: row.directory_relid,
            },
        )?;
        generation_catalog::delete_generation_if_unpublished(
            index_oid,
            logical_index_uuid,
            build_id,
        )
    })();
    result.unwrap_or_else(|error| pgrx::error!("{error}"));
}

#[pg_extern(stable, strict, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_list_unpublished_generations(
    index_regclass: PgRelation,
) -> TableIterator<
    'static,
    (
        name!(build_id, Uuid),
        name!(epoch, i64),
        name!(state, String),
        name!(build_spec_digest, Vec<u8>),
        name!(generation_descriptor_digest, Vec<u8>),
        name!(created_at, TimestampWithTimeZone),
    ),
> {
    let result = (|| -> Result<Vec<_>, String> {
        let index_oid = index_regclass.oid();
        let (_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "ec_distann_list_unpublished_generations",
        )?;
        let generation_catalog =
            generation_catalog::extension_relation_name("ec_distann_generation")?;
        let sql = format!(
            "SELECT build_id, epoch, state, build_spec_digest,
                    generation_descriptor_digest, created_at
               FROM {generation_catalog}
              WHERE index_oid = $1::oid
                AND logical_index_uuid = $2::uuid
                AND state IN ('Building', 'Ready')
              ORDER BY epoch, build_id"
        );
        Spi::connect(|client| {
            client
                .select(&sql, None, &[index_oid.into(), logical_index_uuid.into()])
                .map_err(|error| format!("ec_distann unpublished listing failed: {error}"))?
                .map(|row| {
                    let required = |name: &str| -> Result<Vec<u8>, String> {
                        row[name]
                            .value::<Vec<u8>>()
                            .map_err(|error| {
                                format!("ec_distann unpublished {name} decode failed: {error}")
                            })?
                            .ok_or_else(|| format!("ec_distann unpublished {name} is NULL"))
                    };
                    Ok((
                        row["build_id"]
                            .value::<Uuid>()
                            .map_err(|error| {
                                format!("ec_distann unpublished build_id decode failed: {error}")
                            })?
                            .ok_or_else(|| "ec_distann unpublished build_id is NULL".to_owned())?,
                        row["epoch"]
                            .value::<i64>()
                            .map_err(|error| {
                                format!("ec_distann unpublished epoch decode failed: {error}")
                            })?
                            .ok_or_else(|| "ec_distann unpublished epoch is NULL".to_owned())?,
                        row["state"]
                            .value::<String>()
                            .map_err(|error| {
                                format!("ec_distann unpublished state decode failed: {error}")
                            })?
                            .ok_or_else(|| "ec_distann unpublished state is NULL".to_owned())?,
                        required("build_spec_digest")?,
                        required("generation_descriptor_digest")?,
                        row["created_at"]
                            .value::<TimestampWithTimeZone>()
                            .map_err(|error| {
                                format!("ec_distann unpublished timestamp decode failed: {error}")
                            })?
                            .ok_or_else(|| "ec_distann unpublished timestamp is NULL".to_owned())?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()
        })
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::new(result.into_iter())
}

/// Destructive ec_distann rebuild boundary. A control REINDEX receives a fresh
/// v5 logical UUID, while a mode-changing REINDEX may replace the control with a
/// legacy graph. In either case remove every prior physical generation and
/// index-scoped catalog row before new metadata is written so no state aliases
/// the rebuilt relation identity.
pub(crate) fn reset_control_index_for_rebuild(
    index_relation: pg_sys::Relation,
) -> Result<(), String> {
    let index_handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann control rebuild got a null relation".to_owned())?;
    let index_oid = relation_oid_handle(index_handle);
    for (row_tier_relid, graph_store_relid, directory_relid) in
        generation_catalog::generation_relations_for_index(index_oid)?
    {
        drop_generation_relations(
            index_oid,
            GenerationRelations {
                row_tier_relid,
                graph_store_relid,
                directory_relid,
            },
        )?;
    }
    generation_catalog::delete_index_catalog_rows(index_oid)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::canonical_control_compatibility_digest;
    use crate::am::ec_distann::page::{DistannMetadataPage, DISTANN_NEIGHBOR_CODEC_GROUPED_PQ};

    #[test]
    fn control_compatibility_v1_golden_is_frozen() {
        let mut logical_uuid = [0x55; 16];
        logical_uuid[6] = 0x45;
        logical_uuid[8] = 0x85;
        let metadata = DistannMetadataPage::distributed_control(
            4,
            100,
            1.2,
            42,
            DISTANN_NEIGHBOR_CODEC_GROUPED_PQ,
            4096,
            0.3,
            logical_uuid,
        );
        let digest = canonical_control_compatibility_digest(&metadata, 4, 1, 1, 1, &[0x11; 32])
            .expect("compatibility identity should encode");
        assert_eq!(
            hex::encode(digest),
            "3c9e8a0ac974ff8e39587276b0594ebc73d7b352ac867bd5356c09903da475c8"
        );
    }
}
