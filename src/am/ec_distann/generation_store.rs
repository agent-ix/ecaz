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
use super::canonical_wire::is_rfc4122_v4_uuid;
use super::generation_catalog::{self, GenerationCatalogRow};
use super::generation_descriptor::DistannGenerationDescriptor;
use super::handoff_wire::{owner_stream_digest, DistannHandoffShape};
use super::page::{DistannMetadataPage, INDEX_FORMAT_V5_DISTANN_CONTROL};
use super::quantizer::DistannCodecBinding;
use super::roster_digest as canonical_roster_digest;
use super::row_schema::{resolve_relation_schema, ResolvedRowSchema};

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

fn open_control_index(
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
    let tablespace_oid = relation_tablespace_handle(index_handle);
    if tablespace_oid == pg_sys::InvalidOid {
        return Ok(String::new());
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
        if column.not_null {
            definition.push_str(" NOT NULL");
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
    ),
> {
    let index_oid = index_regclass.oid();
    let (_guard, _handle, metadata, logical_index_uuid) = open_control_index(
        index_oid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        "ec_distann_control_identity",
    )
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::once((
        logical_index_uuid,
        i32::from(metadata.format_version),
        metadata.is_distributed_control(),
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
            cumulative_owner_digest: owner_stream_digest(&[], shape)?,
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
        let Some(row) =
            generation_catalog::lookup_generation(index_oid, logical_index_uuid, build_id)?
        else {
            return Ok(());
        };
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
        generation_catalog::delete_generation(index_oid, logical_index_uuid, build_id)
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
