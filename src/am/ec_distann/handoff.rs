//! Task 179 participant-side streamed handoff transactions.
//!
//! Invalid input is fully decoded and checked before the first physical write.
//! This is stricter than transaction rollback alone: an aborted heap insert can
//! leave relation-file growth even though no tuple becomes visible.

use std::ffi::CStr;
use std::panic::AssertUnwindSafe;

use pgrx::datum::Uuid;
use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern, pg_sys, FromDatum, PgRelation, Spi};
use sha2::{Digest, Sha256};

use crate::am::common::heap_slot::TupleSlotWriter;
use crate::storage::page::ItemPointer;
use crate::storage::relation::relation_namespace_owner_persistence_handle;
use crate::storage::relation_guard::HeapRelationGuard;
use crate::storage::slot_guard::TupleTableSlotGuard;

use super::canonical_wire::{fixed_digest, is_rfc4122_v4_uuid};
use super::generation_catalog::{self, GenerationBatchCatalogRow, GenerationCatalogRow};
use super::generation_descriptor::{
    roster_digest, DistannGenerationDescriptor, DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
};
use super::generation_store::open_control_index;
use super::handoff_wire::{
    DistannHandoffBatch, DistannHandoffEntry, DistannHandoffShape, DistannOwnerStreamHasher,
};
use super::identity::vec_id_from_source_identity;
use super::lifecycle_state::GenerationState;
use super::manifest_v2::{
    DistannEpochFingerprint, DistannReadyReceipt, DistannReadyReceiptHotCold,
    DistannReadyReceiptPayloadSidecar, DISTANN_READY_RECEIPT_STATE,
};
use super::payload_sidecar::DistannPayloadCoverDescriptorV1;
use super::placement::owning_node;
use super::quote_ident;
use super::row_layout::{DistannRowTierLayoutDescriptorV1, DistannRowTierV1};
use super::row_schema::resolve_relation_schema;
use super::tuple::DistannNodeTuple;

type StageResult = (
    name!(accepted_record_count, i64),
    name!(cumulative_record_count, i64),
    name!(cumulative_owner_digest, Vec<u8>),
);

struct RowAttributeIo {
    receive_flinfo: pg_sys::FmgrInfo,
    send_flinfo: pg_sys::FmgrInfo,
    receive_typioparam: pg_sys::Oid,
    typmod: i32,
}

struct PreparedEntry {
    datums: Vec<Option<pg_sys::Datum>>,
    cold_datums: Option<Vec<Option<pg_sys::Datum>>>,
    payload_sidecar: Option<Vec<u8>>,
    node: DistannNodeTuple,
}

const PERSISTED_GRAPH_DOMAIN: &[u8] = b"ec_distann_persisted_graph_v1\0";
const PERSISTED_ROW_TIER_DOMAIN: &[u8] = b"ec_distann_persisted_row_tier_v1\0";
const HOT_TIER_INITIAL_CONTENT_DOMAIN: &[u8] = b"ec_distann_hot_tier_initial_content_v1\0";
const COLD_TIER_INITIAL_CONTENT_DOMAIN: &[u8] = b"ec_distann_cold_tier_initial_content_v1\0";
const LOCAL_DIRECTORY_DOMAIN: &[u8] = b"ec_distann_local_directory_v1\0";
const OWNED_VEC_IDS_DOMAIN: &[u8] = b"ec_distann_owned_vec_ids_v1\0";
const PAYLOAD_SIDECAR_INITIAL_CONTENT_DOMAIN: &[u8] =
    b"ec_distann_payload_sidecar_initial_content_v1\0";

/// Execute caller-controlled type I/O with the privileges of the control
/// owner, never the SECURITY DEFINER endpoint owner.  PostgreSQL uses the
/// same restricted-operation/GUC pattern while evaluating index expressions.
/// `PgTryBuilder::finally` is required here because a domain CHECK or a type
/// I/O function can raise a PostgreSQL ERROR rather than return normally.
pub(crate) fn with_restricted_type_io_owner<R>(
    owner: pg_sys::Oid,
    operation: impl FnOnce() -> R,
) -> R {
    let mut saved_user = pg_sys::InvalidOid;
    let mut saved_security_context = 0;
    unsafe {
        pg_sys::GetUserIdAndSecContext(&mut saved_user, &mut saved_security_context);
    }
    let guc_nest_level = unsafe { pg_sys::NewGUCNestLevel() };
    pg_sys::PgTryBuilder::new(AssertUnwindSafe(|| {
        unsafe {
            pg_sys::SetUserIdAndSecContext(
                owner,
                saved_security_context | pg_sys::SECURITY_RESTRICTED_OPERATION as i32,
            );
            pg_sys::RestrictSearchPath();
        }
        operation()
    }))
    .finally(|| unsafe {
        pg_sys::AtEOXact_GUC(false, guc_nest_level);
        pg_sys::SetUserIdAndSecContext(saved_user, saved_security_context);
    })
    .execute()
}

fn stage_result(row: &GenerationBatchCatalogRow) -> Result<StageResult, String> {
    Ok((
        i64::try_from(row.accepted_record_count)
            .map_err(|_| "EC_BUILD_INCOMPLETE: accepted record count exceeds bigint".to_owned())?,
        i64::try_from(row.cumulative_record_count).map_err(|_| {
            "EC_BUILD_INCOMPLETE: cumulative record count exceeds bigint".to_owned()
        })?,
        row.cumulative_owner_digest.to_vec(),
    ))
}

fn owned_cstring(pointer: *mut std::ffi::c_char, field: &str) -> Result<String, String> {
    if pointer.is_null() {
        return Err(format!("EC_GENERATION_MISSING: relation {field} is absent"));
    }
    let value = unsafe { CStr::from_ptr(pointer) }
        .to_str()
        .map_err(|_| format!("EC_GENERATION_MISSING: relation {field} is not UTF-8"))?
        .to_owned();
    unsafe { pg_sys::pfree(pointer.cast()) };
    Ok(value)
}

pub(crate) fn qualified_relation_name(relation_oid: pg_sys::Oid) -> Result<String, String> {
    let relation = owned_cstring(unsafe { pg_sys::get_rel_name(relation_oid) }, "name")?;
    let namespace_oid = unsafe { pg_sys::get_rel_namespace(relation_oid) };
    if namespace_oid == pg_sys::InvalidOid {
        return Err("EC_GENERATION_MISSING: relation namespace is absent".to_owned());
    }
    let namespace = owned_cstring(
        unsafe { pg_sys::get_namespace_name(namespace_oid) },
        "namespace",
    )?;
    Ok(format!(
        "{}.{}",
        quote_ident(&namespace),
        quote_ident(&relation)
    ))
}

fn identity_attnum(index_oid: pg_sys::Oid) -> Result<u16, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT indkey[1]::int4 AS identity_attnum
                   FROM pg_catalog.pg_index
                  WHERE indexrelid = $1::oid
                    AND indnkeyatts = 1
                    AND indnatts = 2",
                None,
                &[index_oid.into()],
            )
            .map_err(|error| format!("EC_SCHEMA_MISMATCH: identity lookup failed: {error}"))?
            .map(|row| {
                let value = row["identity_attnum"]
                    .value::<i32>()
                    .map_err(|error| {
                        format!("EC_SCHEMA_MISMATCH: identity attnum decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_SCHEMA_MISMATCH: identity attnum is NULL".to_owned())?;
                u16::try_from(value)
                    .map_err(|_| "EC_SCHEMA_MISMATCH: identity attnum is outside u16".to_owned())
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_SCHEMA_MISMATCH: control identity attribute is absent".to_owned())
    })
}

fn indexed_vector_attnum(index_oid: pg_sys::Oid) -> Result<u16, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT indkey[0]::int4 AS vector_attnum
                   FROM pg_catalog.pg_index
                  WHERE indexrelid = $1::oid",
                None,
                &[index_oid.into()],
            )
            .map_err(|error| format!("EC_SCHEMA_MISMATCH: vector lookup failed: {error}"))?
            .map(|row| {
                let value = row["vector_attnum"]
                    .value::<i32>()
                    .map_err(|error| {
                        format!("EC_SCHEMA_MISMATCH: vector attnum decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_SCHEMA_MISMATCH: vector attnum is NULL".to_owned())?;
                u16::try_from(value)
                    .map_err(|_| "EC_SCHEMA_MISMATCH: indexed vector attnum is invalid".to_owned())
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_SCHEMA_MISMATCH: control index catalog row is absent".to_owned())
    })
}

pub(crate) fn compact_tier_schema_matches(
    physical: &super::row_schema::DistannRowSchemaDescriptor,
    logical: &super::row_schema::DistannRowSchemaDescriptor,
    layout: &DistannRowTierLayoutDescriptorV1,
    tier: DistannRowTierV1,
) -> bool {
    let placements = layout
        .placements
        .iter()
        .filter(|placement| placement.tier == tier)
        .collect::<Vec<_>>();
    if physical.attributes.len() != placements.len() + 1 {
        return false;
    }
    let Some(vec_id) = physical.attributes.first() else {
        return false;
    };
    if vec_id.attnum != 1
        || vec_id.name != "vec_id"
        || vec_id.type_namespace != "pg_catalog"
        || vec_id.type_name != "int8"
        || vec_id.typmod != -1
        || !vec_id.collation_namespace.is_empty()
        || !vec_id.collation_name.is_empty()
        || vec_id.dropped
        || vec_id.generated_kind != 0
        || vec_id.send_function != "pg_catalog.int8send"
        || vec_id.receive_function != "pg_catalog.int8recv"
    {
        return false;
    }
    placements.into_iter().all(|placement| {
        let Some(physical_index) = usize::from(placement.physical_ordinal).checked_sub(1) else {
            return false;
        };
        let Some(actual) = physical.attributes.get(physical_index) else {
            return false;
        };
        let Some(source) = logical
            .attributes
            .iter()
            .find(|attribute| attribute.attnum == placement.attnum && !attribute.dropped)
        else {
            return false;
        };
        actual.attnum == placement.physical_ordinal
            && actual.name == format!("a_{}", placement.attnum)
            && actual.type_namespace == source.type_namespace
            && actual.type_name == source.type_name
            && actual.typmod == source.typmod
            && actual.collation_namespace == source.collation_namespace
            && actual.collation_name == source.collation_name
            && !actual.dropped
            && actual.generated_kind == 0
            && actual.send_function == source.send_function
            && actual.receive_function == source.receive_function
    })
}

fn validate_generation_relations(
    row: &GenerationCatalogRow,
    descriptor: &DistannGenerationDescriptor,
    control_owner: pg_sys::Oid,
) -> Result<(), String> {
    let physical_row_schema = resolve_relation_schema(row.row_tier_relid)?;
    let hot_cold = descriptor.row_tier_layout();
    match (hot_cold, row.cold_tier_relid) {
        (None, None) => {
            let mut expected_row_schema = descriptor.row_schema.clone();
            for attribute in &mut expected_row_schema.attributes {
                if !attribute.dropped {
                    attribute.generated_kind = 0;
                }
            }
            if physical_row_schema.descriptor != expected_row_schema {
                return Err(
                    "EC_SCHEMA_MISMATCH: physical row tier differs from the frozen generation schema"
                        .to_owned(),
                );
            }
        }
        (Some(layout), Some(cold_tier_relid)) => {
            let physical_cold_schema = resolve_relation_schema(cold_tier_relid)?;
            if !compact_tier_schema_matches(
                &physical_row_schema.descriptor,
                &descriptor.row_schema,
                layout,
                DistannRowTierV1::Hot,
            ) || !compact_tier_schema_matches(
                &physical_cold_schema.descriptor,
                &descriptor.row_schema,
                layout,
                DistannRowTierV1::Cold,
            ) {
                return Err(
                    "EC_SCHEMA_MISMATCH: compact hot/cold tiers differ from the frozen layout"
                        .to_owned(),
                );
            }
        }
        _ => {
            return Err(
                "EC_GENERATION_MISSING: cold relation disagrees with the generation descriptor"
                    .to_owned(),
            )
        }
    }

    let plain_attribute_names = hot_cold.map(|layout| {
        vec![
            format!("a_{}", layout.indexed_vector_attnum),
            format!("a_{}", layout.source_identity_attnum),
        ]
    });

    let valid = Spi::connect(|client| {
        client
            .select(
                "SELECT
                    EXISTS (
                        SELECT 1 FROM pg_catalog.pg_class
                         WHERE oid = $1::oid AND relkind = 'r' AND relowner = $4::oid
                           AND relpersistence = 'p' AND NOT relrowsecurity
                           AND NOT relforcerowsecurity
                           AND (NOT $5::bool OR
                                COALESCE(reloptions, ARRAY[]::text[]) @> ARRAY['fillfactor=100'])
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_attribute
                         WHERE attrelid = $1::oid AND attnum > 0 AND NOT attisdropped
                           AND (
                               atthasdef OR attidentity <> '' OR attgenerated <> ''
                               OR CASE WHEN $5::bool THEN
                                    (attnum = 1 AND
                                     (NOT attnotnull OR attname <> 'vec_id'
                                      OR atttypid <> 'bigint'::regtype))
                                    OR (attnum > 1 AND attnotnull)
                                  ELSE attnotnull END
                           )
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_index WHERE indrelid = $1::oid
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_constraint
                         WHERE conrelid = $1::oid AND contype <> 'n'
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_trigger
                         WHERE tgrelid = $1::oid AND NOT tgisinternal
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_rewrite WHERE ev_class = $1::oid
                    )
                    AND (NOT $5::bool OR (
                        SELECT count(*) = 2 AND bool_and(attstorage = 'p')
                          FROM pg_catalog.pg_attribute
                         WHERE attrelid = $1::oid AND attnum > 0 AND NOT attisdropped
                           AND attname::text = ANY($7::text[])
                    ))
                    AND (NOT $5::bool OR (
                        EXISTS (
                            SELECT 1 FROM pg_catalog.pg_class
                             WHERE oid = $6::oid AND relkind = 'r' AND relowner = $4::oid
                               AND relpersistence = 'p' AND NOT relrowsecurity
                               AND NOT relforcerowsecurity
                               AND reltablespace = (
                                   SELECT reltablespace FROM pg_catalog.pg_class WHERE oid = $1::oid
                               )
                        )
                        AND NOT EXISTS (
                            SELECT 1 FROM pg_catalog.pg_attribute
                             WHERE attrelid = $6::oid AND attnum > 0 AND NOT attisdropped
                               AND (
                                   atthasdef OR attidentity <> '' OR attgenerated <> ''
                                   OR (attnum = 1 AND
                                       (NOT attnotnull OR attname <> 'vec_id'
                                        OR atttypid <> 'bigint'::regtype))
                                   OR (attnum > 1 AND attnotnull)
                               )
                        )
                        AND NOT EXISTS (
                            SELECT 1 FROM pg_catalog.pg_index WHERE indrelid = $6::oid
                        )
                        AND NOT EXISTS (
                            SELECT 1 FROM pg_catalog.pg_constraint
                             WHERE conrelid = $6::oid AND contype <> 'n'
                        )
                        AND NOT EXISTS (
                            SELECT 1 FROM pg_catalog.pg_trigger
                             WHERE tgrelid = $6::oid AND NOT tgisinternal
                        )
                        AND NOT EXISTS (
                            SELECT 1 FROM pg_catalog.pg_rewrite WHERE ev_class = $6::oid
                        )
                    ))
                    AND EXISTS (
                        SELECT 1 FROM pg_catalog.pg_class
                         WHERE oid = $2::oid AND relkind = 'r' AND relowner = $4::oid
                           AND relpersistence = 'p' AND NOT relrowsecurity
                           AND NOT relforcerowsecurity
                    )
                    AND (
                        SELECT count(*) = 5
                           AND bool_and(
                               attnotnull
                               AND attidentity = '' AND attgenerated = ''
                               AND CASE attnum
                                     WHEN 1 THEN NOT atthasdef AND attname = 'vec_id' AND atttypid = 'bigint'::regtype
                                     WHEN 2 THEN NOT atthasdef AND attname = 'graph_record' AND atttypid = 'bytea'::regtype
                                     WHEN 3 THEN NOT atthasdef AND attname = 'row_tid' AND atttypid = 'tid'::regtype
                                     WHEN 4 THEN attname = 'record_version' AND atttypid = 'int8'::regtype
                                     WHEN 5 THEN attname = 'is_current' AND atttypid = 'bool'::regtype
                                     ELSE false
                                   END
                           )
                          FROM pg_catalog.pg_attribute
                         WHERE attrelid = $2::oid AND attnum > 0 AND NOT attisdropped
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_constraint
                         WHERE conrelid = $2::oid AND contype <> 'n'
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_trigger
                         WHERE tgrelid = $2::oid AND NOT tgisinternal
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_rewrite WHERE ev_class = $2::oid
                    )
                    AND (
                        SELECT count(*) = 1
                           AND bool_and(indexrelid = $3::oid AND indisunique
                                        AND indisvalid AND indisready AND indislive
                                        AND indnkeyatts = 1 AND indnatts = 1
                                        AND indkey[0] = 1 AND indpred IS NOT NULL)
                          FROM pg_catalog.pg_index WHERE indrelid = $2::oid
                    )
                    AND EXISTS (
                        SELECT 1 FROM pg_catalog.pg_class c
                        JOIN pg_catalog.pg_am am ON am.oid = c.relam
                         WHERE c.oid = $3::oid AND c.relkind = 'i'
                           AND c.relowner = $4::oid AND am.amname = 'btree'
                    ) AS valid",
                None,
                &[
                    row.row_tier_relid.into(),
                    row.graph_store_relid.into(),
                    row.directory_relid.into(),
                    control_owner.into(),
                    hot_cold.is_some().into(),
                    row.cold_tier_relid.into(),
                    plain_attribute_names.unwrap_or_default().into(),
                ],
            )
            .map_err(|error| {
                format!("EC_GENERATION_MISSING: physical relation validation failed: {error}")
            })?
            .map(|result| {
                result["valid"]
                    .value::<bool>()
                    .map_err(|error| {
                        format!("EC_GENERATION_MISSING: relation validation decode failed: {error}")
                    })?
                    .ok_or_else(|| {
                        "EC_GENERATION_MISSING: relation validation returned NULL".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or(false))
    })?;
    if !valid {
        return Err(
            "EC_GENERATION_MISSING: generation relations violate their immutable physical shape"
                .to_owned(),
        );
    }
    let sidecar_pair = match (
        row.payload_sidecar_relid,
        row.payload_sidecar_directory_relid,
        descriptor.payload_cover.as_ref(),
    ) {
        (None, None, None) => return Ok(()),
        (Some(sidecar_relid), Some(directory_relid), Some(cover)) => {
            cover.validate_row_schema(&descriptor.row_schema)?;
            (sidecar_relid, directory_relid)
        }
        _ => {
            return Err(
                "EC_GENERATION_MISSING: payload sidecar relation pair disagrees with the generation descriptor"
                    .to_owned(),
            );
        }
    };
    let sidecar_valid = Spi::connect(|client| {
        client
            .select(
                "SELECT
                    EXISTS (
                        SELECT 1 FROM pg_catalog.pg_class c
                         WHERE c.oid = $1::oid AND c.relkind = 'r'
                           AND c.relowner = $3::oid AND c.relpersistence = 'p'
                           AND NOT c.relrowsecurity AND NOT c.relforcerowsecurity
                           AND c.reltablespace = (
                               SELECT reltablespace FROM pg_catalog.pg_class WHERE oid = $4::oid
                           )
                           AND COALESCE(c.reloptions, ARRAY[]::text[]) @> ARRAY['fillfactor=100']
                    )
                    AND (
                        SELECT count(*) = 3
                           AND bool_and(
                               attnotnull AND NOT atthasdef
                               AND attidentity = '' AND attgenerated = ''
                               AND CASE attnum
                                     WHEN 1 THEN attname = 'row_tid' AND atttypid = 'tid'::regtype
                                     WHEN 2 THEN attname = 'vec_id' AND atttypid = 'bigint'::regtype
                                     WHEN 3 THEN attname = 'payload' AND atttypid = 'bytea'::regtype
                                                 AND attstorage = 'p'
                                     ELSE false
                                   END
                           )
                          FROM pg_catalog.pg_attribute
                         WHERE attrelid = $1::oid AND attnum > 0 AND NOT attisdropped
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_constraint
                         WHERE conrelid = $1::oid AND contype <> 'n'
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_trigger
                         WHERE tgrelid = $1::oid AND NOT tgisinternal
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_rewrite WHERE ev_class = $1::oid
                    )
                    AND (
                        SELECT count(*) = 1
                           AND bool_and(indexrelid = $2::oid AND indisunique
                                        AND indisvalid AND indisready AND indislive
                                        AND indnkeyatts = 1 AND indnatts = 1
                                        AND indkey[0] = 1 AND indpred IS NULL)
                          FROM pg_catalog.pg_index WHERE indrelid = $1::oid
                    )
                    AND EXISTS (
                        SELECT 1 FROM pg_catalog.pg_class c
                        JOIN pg_catalog.pg_am am ON am.oid = c.relam
                         WHERE c.oid = $2::oid AND c.relkind = 'i'
                           AND c.relowner = $3::oid AND am.amname = 'btree'
                           AND c.reltablespace = (
                               SELECT reltablespace FROM pg_catalog.pg_class WHERE oid = $4::oid
                           )
                    ) AS valid",
                None,
                &[
                    sidecar_pair.0.into(),
                    sidecar_pair.1.into(),
                    control_owner.into(),
                    row.row_tier_relid.into(),
                ],
            )
            .map_err(|error| {
                format!("EC_GENERATION_MISSING: payload sidecar validation failed: {error}")
            })?
            .map(|result| {
                result["valid"]
                    .value::<bool>()
                    .map_err(|error| {
                        format!("EC_GENERATION_MISSING: payload sidecar validation decode failed: {error}")
                    })?
                    .ok_or_else(|| {
                        "EC_GENERATION_MISSING: payload sidecar validation returned NULL".to_owned()
                    })
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or(false))
    })?;
    if !sidecar_valid {
        return Err(
            "EC_GENERATION_MISSING: payload sidecar relations violate their immutable physical shape"
                .to_owned(),
        );
    }
    Ok(())
}

unsafe fn row_attribute_io(
    row_relation: pg_sys::Relation,
) -> Result<Vec<Option<RowAttributeIo>>, String> {
    let tuple_desc = unsafe { (*row_relation).rd_att };
    if tuple_desc.is_null() {
        return Err("EC_SCHEMA_MISMATCH: row-tier tuple descriptor is NULL".to_owned());
    }
    let natts = usize::try_from(unsafe { (*tuple_desc).natts })
        .map_err(|_| "EC_SCHEMA_MISMATCH: row-tier attribute count is negative".to_owned())?;
    let mut output = Vec::with_capacity(natts);
    for attribute_index in 0..natts {
        let attribute = unsafe { pg_sys::TupleDescAttr(tuple_desc, attribute_index as i32) };
        if attribute.is_null() {
            return Err("EC_SCHEMA_MISMATCH: row-tier attribute descriptor is NULL".to_owned());
        }
        if unsafe { (*attribute).attisdropped } {
            output.push(None);
            continue;
        }
        let type_oid = unsafe { (*attribute).atttypid };
        let mut receive_oid = pg_sys::InvalidOid;
        let mut receive_typioparam = pg_sys::InvalidOid;
        unsafe {
            pg_sys::getTypeBinaryInputInfo(type_oid, &mut receive_oid, &mut receive_typioparam)
        };
        let mut send_oid = pg_sys::InvalidOid;
        let mut is_varlena = false;
        unsafe { pg_sys::getTypeBinaryOutputInfo(type_oid, &mut send_oid, &mut is_varlena) };
        if receive_oid == pg_sys::InvalidOid || send_oid == pg_sys::InvalidOid {
            return Err(format!(
                "EC_SCHEMA_UNSUPPORTED: row-tier attribute {} lacks binary I/O",
                attribute_index + 1
            ));
        }
        let mut receive_flinfo =
            unsafe { std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init() };
        let mut send_flinfo =
            unsafe { std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init() };
        unsafe {
            pg_sys::fmgr_info(receive_oid, &mut receive_flinfo);
            pg_sys::fmgr_info(send_oid, &mut send_flinfo);
        }
        output.push(Some(RowAttributeIo {
            receive_flinfo,
            send_flinfo,
            receive_typioparam,
            typmod: unsafe { (*attribute).atttypmod },
        }));
    }
    Ok(output)
}

unsafe fn receive_and_verify(
    bytes: &[u8],
    attribute: &mut RowAttributeIo,
    attnum: usize,
) -> Result<pg_sys::Datum, String> {
    let len = core::ffi::c_int::try_from(bytes.len()).map_err(|_| {
        format!("EC_HANDOFF_TOO_LARGE: attribute {attnum} binary value exceeds int")
    })?;
    let mut input_bytes = bytes.to_vec();
    let mut input = pg_sys::StringInfoData {
        data: input_bytes.as_mut_ptr().cast(),
        len,
        maxlen: len,
        cursor: 0,
    };
    let datum = unsafe {
        pg_sys::ReceiveFunctionCall(
            &mut attribute.receive_flinfo,
            &mut input,
            attribute.receive_typioparam,
            attribute.typmod,
        )
    };
    if input.cursor != input.len {
        return Err(format!(
            "EC_HANDOFF_FORMAT: attribute {attnum} binary receive left unread bytes"
        ));
    }
    let sent = unsafe { pg_sys::SendFunctionCall(&mut attribute.send_flinfo, datum) };
    if sent.is_null() {
        return Err(format!(
            "EC_HANDOFF_FORMAT: attribute {attnum} binary send returned NULL"
        ));
    }
    let canonical = unsafe { pgrx::varlena::varlena_to_byte_slice(sent.cast()) };
    if canonical != bytes {
        return Err(format!(
            "EC_HANDOFF_FORMAT: attribute {attnum} binary value is non-canonical"
        ));
    }
    unsafe { pg_sys::pfree(sent.cast()) };
    Ok(datum)
}

fn prepare_legacy_entries(
    entries: &[DistannHandoffEntry],
    shape: DistannHandoffShape,
    identity_attnum: u16,
    indexed_vector_attnum: u16,
    row_io: &mut [Option<RowAttributeIo>],
    payload_cover: Option<&DistannPayloadCoverDescriptorV1>,
) -> Result<Vec<PreparedEntry>, String> {
    let mut prepared = Vec::with_capacity(entries.len());
    for entry in entries {
        let mut datums = Vec::with_capacity(row_io.len());
        let mut payload_values = payload_cover.map(|cover| vec![None; cover.attributes.len()]);
        let mut non_dropped_position = 0_usize;
        let mut value_position = 0_usize;
        for (attribute_index, io) in row_io.iter_mut().enumerate() {
            let attnum = attribute_index + 1;
            let Some(io) = io else {
                datums.push(None);
                continue;
            };
            let is_null = entry.row_null_bitmap[non_dropped_position / 8]
                & (1 << (non_dropped_position % 8))
                != 0;
            non_dropped_position += 1;
            if is_null {
                if attnum == usize::from(identity_attnum) {
                    return Err("EC_SOURCE_IDENTITY: source identity is NULL".to_owned());
                }
                if attnum == usize::from(indexed_vector_attnum) {
                    return Err("EC_SCHEMA_MISMATCH: indexed vector is NULL".to_owned());
                }
                datums.push(None);
                continue;
            }
            let bytes = entry.row_values.get(value_position).ok_or_else(|| {
                format!("EC_HANDOFF_FORMAT: attribute {attnum} binary value is absent")
            })?;
            value_position += 1;
            if attnum == usize::from(identity_attnum) && bytes != &entry.source_identity {
                return Err(
                    "EC_SOURCE_IDENTITY: row-tier identity differs from the handoff identity"
                        .to_owned(),
                );
            }
            if let (Some(cover), Some(payload_values)) = (payload_cover, &mut payload_values) {
                if let Ok(position) = cover
                    .attributes
                    .binary_search_by_key(&(attnum as u16), |attribute| attribute.attnum)
                {
                    payload_values[position] = Some(bytes.as_slice());
                }
            }
            let datum = unsafe { receive_and_verify(bytes, io, attnum)? };
            datums.push(Some(datum));
        }
        if non_dropped_position != shape.non_dropped_attribute_count
            || value_position != entry.row_values.len()
        {
            return Err("EC_HANDOFF_FORMAT: row payload width disagrees with schema".to_owned());
        }

        let neighbor_count = u16::try_from(entry.neighbor_vec_ids.len())
            .map_err(|_| "EC_HANDOFF_FORMAT: neighbor count exceeds u16".to_owned())?;
        let mut neighbor_vec_ids = entry.neighbor_vec_ids.clone();
        neighbor_vec_ids.resize(shape.graph_degree, 0);
        let mut neighbor_codes = entry.neighbor_codes.clone();
        neighbor_codes.resize(shape.graph_degree * shape.code_stride, 0);
        let node = DistannNodeTuple {
            tombstoned: false,
            vec_id: entry.vec_id,
            heap_tid: ItemPointer {
                block_number: 0,
                offset_number: 1,
            },
            cold_tid: None,
            neighbor_count,
            search_code: entry.search_code.clone(),
            neighbor_vec_ids,
            neighbor_codes,
        };
        node.encode_physical_v1(
            u16::try_from(shape.graph_degree)
                .map_err(|_| "EC_HANDOFF_FORMAT: graph degree exceeds u16".to_owned())?,
            shape.code_stride,
        )?;
        let payload_sidecar = payload_cover
            .zip(payload_values)
            .map(|(cover, values)| cover.encode_payload(&values))
            .transpose()?;
        prepared.push(PreparedEntry {
            datums,
            cold_datums: None,
            payload_sidecar,
            node,
        });
    }
    Ok(prepared)
}

fn prepare_hot_cold_entries(
    entries: &[DistannHandoffEntry],
    shape: DistannHandoffShape,
    identity_attnum: u16,
    row_schema: &super::row_schema::DistannRowSchemaDescriptor,
    layout: &DistannRowTierLayoutDescriptorV1,
    hot_io: &mut [Option<RowAttributeIo>],
    cold_io: &mut [Option<RowAttributeIo>],
) -> Result<Vec<PreparedEntry>, String> {
    if hot_io.is_empty() || cold_io.is_empty() {
        return Err("EC_SCHEMA_MISMATCH: compact row tier lacks its vec_id column".to_owned());
    }
    let graph_degree = u16::try_from(shape.graph_degree)
        .map_err(|_| "EC_HANDOFF_FORMAT: graph degree exceeds u16".to_owned())?;
    let mut prepared = Vec::with_capacity(entries.len());
    for entry in entries {
        let stored_vec_id = i64::from_le_bytes(entry.vec_id.to_le_bytes());
        let mut hot_datums = vec![None; hot_io.len()];
        let mut cold_datums = vec![None; cold_io.len()];
        hot_datums[0] = Some(pg_sys::Datum::from(stored_vec_id));
        cold_datums[0] = Some(pg_sys::Datum::from(stored_vec_id));
        let mut non_dropped_position = 0_usize;
        let mut value_position = 0_usize;
        for attribute in &row_schema.attributes {
            if attribute.dropped {
                continue;
            }
            let is_null = entry.row_null_bitmap[non_dropped_position / 8]
                & (1 << (non_dropped_position % 8))
                != 0;
            non_dropped_position += 1;
            let placement = layout
                .placements
                .iter()
                .find(|placement| placement.attnum == attribute.attnum)
                .ok_or_else(|| {
                    format!(
                        "EC_SCHEMA_MISMATCH: source attnum {} lacks a row-tier placement",
                        attribute.attnum
                    )
                })?;
            let physical_index = usize::from(placement.physical_ordinal)
                .checked_sub(1)
                .ok_or_else(|| "EC_SCHEMA_MISMATCH: compact row-tier ordinal is zero".to_owned())?;
            let (tier_datums, tier_io) = match placement.tier {
                DistannRowTierV1::Hot => (&mut hot_datums, &mut *hot_io),
                DistannRowTierV1::Cold => (&mut cold_datums, &mut *cold_io),
            };
            let target = tier_datums.get_mut(physical_index).ok_or_else(|| {
                format!(
                    "EC_SCHEMA_MISMATCH: source attnum {} compact ordinal exceeds relation width",
                    attribute.attnum
                )
            })?;
            let io = tier_io
                .get_mut(physical_index)
                .and_then(Option::as_mut)
                .ok_or_else(|| {
                    format!(
                        "EC_SCHEMA_MISMATCH: source attnum {} compact column lacks binary I/O",
                        attribute.attnum
                    )
                })?;
            if is_null {
                if attribute.attnum == identity_attnum {
                    return Err("EC_SOURCE_IDENTITY: source identity is NULL".to_owned());
                }
                if attribute.attnum == layout.indexed_vector_attnum {
                    return Err("EC_SCHEMA_MISMATCH: indexed vector is NULL".to_owned());
                }
                *target = None;
                continue;
            }
            let bytes = entry.row_values.get(value_position).ok_or_else(|| {
                format!(
                    "EC_HANDOFF_FORMAT: attribute {} binary value is absent",
                    attribute.attnum
                )
            })?;
            value_position += 1;
            if attribute.attnum == identity_attnum && bytes != &entry.source_identity {
                return Err(
                    "EC_SOURCE_IDENTITY: hot-tier identity differs from the handoff identity"
                        .to_owned(),
                );
            }
            *target =
                Some(unsafe { receive_and_verify(bytes, io, usize::from(attribute.attnum))? });
        }
        if non_dropped_position != shape.non_dropped_attribute_count
            || value_position != entry.row_values.len()
        {
            return Err("EC_HANDOFF_FORMAT: row payload width disagrees with schema".to_owned());
        }

        let neighbor_count = u16::try_from(entry.neighbor_vec_ids.len())
            .map_err(|_| "EC_HANDOFF_FORMAT: neighbor count exceeds u16".to_owned())?;
        let mut neighbor_vec_ids = entry.neighbor_vec_ids.clone();
        neighbor_vec_ids.resize(shape.graph_degree, 0);
        let mut neighbor_codes = entry.neighbor_codes.clone();
        neighbor_codes.resize(shape.graph_degree * shape.code_stride, 0);
        let node = DistannNodeTuple {
            tombstoned: false,
            vec_id: entry.vec_id,
            heap_tid: ItemPointer {
                block_number: 0,
                offset_number: 1,
            },
            cold_tid: Some(ItemPointer {
                block_number: 0,
                offset_number: 1,
            }),
            neighbor_count,
            search_code: entry.search_code.clone(),
            neighbor_vec_ids,
            neighbor_codes,
        };
        node.encode_physical_v2(graph_degree, shape.code_stride)?;
        prepared.push(PreparedEntry {
            datums: hot_datums,
            cold_datums: Some(cold_datums),
            payload_sidecar: None,
            node,
        });
    }
    Ok(prepared)
}

fn reject_existing_vec_ids(
    graph_relation: &str,
    entries: &[DistannHandoffEntry],
) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }
    let vec_ids = entries
        .iter()
        .map(|entry| i64::from_le_bytes(entry.vec_id.to_le_bytes()))
        .collect::<Vec<_>>();
    let sql = format!(
        "SELECT EXISTS (
             SELECT 1 FROM {graph_relation}
              WHERE vec_id = ANY($1::bigint[])
         ) AS duplicate_exists"
    );
    let duplicate = Spi::connect(|client| {
        client
            .select(&sql, None, &[vec_ids.into()])
            .map_err(|error| format!("EC_GENERATION_MISSING: duplicate lookup failed: {error}"))?
            .map(|row| {
                row["duplicate_exists"]
                    .value::<bool>()
                    .map_err(|error| {
                        format!("EC_DUPLICATE_VEC_ID: duplicate decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_DUPLICATE_VEC_ID: duplicate lookup returned NULL".to_owned())
            })
            .next()
            .transpose()
            .map(|value| value.unwrap_or(false))
    })?;
    if duplicate {
        return Err("EC_DUPLICATE_VEC_ID: batch contains an already stored vec_id".to_owned());
    }
    Ok(())
}

fn insert_prepared_entries(
    row_relation: &HeapRelationGuard,
    cold_relation: Option<&HeapRelationGuard>,
    graph_relation: &str,
    payload_sidecar_relation: Option<&str>,
    entries: &mut [PreparedEntry],
    shape: DistannHandoffShape,
    graph_record_version: u16,
) -> Result<(), String> {
    let slot = TupleTableSlotGuard::create_for_heap_guard(row_relation).ok_or_else(|| {
        "EC_GENERATION_MISSING: could not allocate row-tier tuple slot".to_owned()
    })?;
    let cold_slot = cold_relation
        .map(|relation| {
            TupleTableSlotGuard::create_for_heap_guard(relation).ok_or_else(|| {
                "EC_GENERATION_MISSING: could not allocate cold-tier tuple slot".to_owned()
            })
        })
        .transpose()?;
    let graph_degree = u16::try_from(shape.graph_degree)
        .map_err(|_| "EC_HANDOFF_FORMAT: graph degree exceeds u16".to_owned())?;
    let mut graph_vec_ids = Vec::with_capacity(entries.len());
    let mut graph_records = Vec::with_capacity(entries.len());
    let mut graph_row_tids = Vec::with_capacity(entries.len());
    let mut payloads = payload_sidecar_relation.map(|_| Vec::with_capacity(entries.len()));
    for entry in entries {
        match (
            cold_relation,
            cold_slot.as_ref(),
            entry.cold_datums.as_ref(),
        ) {
            (Some(cold_relation), Some(cold_slot), Some(cold_datums)) => {
                let mut writer = unsafe {
                    TupleSlotWriter::from_raw_slot(
                        cold_slot.as_ptr(),
                        "ec_distann cold-tier handoff",
                    )?
                };
                writer.clear();
                writer.validate_input_width(cold_datums.len())?;
                for (attribute_index, datum) in cold_datums.iter().enumerate() {
                    if let Some(datum) = datum {
                        writer.set_datum(attribute_index as i32, *datum);
                    } else {
                        writer.set_null(attribute_index as i32);
                    }
                }
                let stored_slot = writer.store_virtual_tuple()?;
                unsafe { pg_sys::simple_table_tuple_insert(cold_relation.as_ptr(), stored_slot) };
                let cold_tid = unsafe { (*stored_slot).tts_tid };
                let (block_number, offset_number) = pgrx::itemptr::item_pointer_get_both(cold_tid);
                if block_number == pg_sys::InvalidBlockNumber || offset_number == 0 {
                    return Err(
                        "EC_GENERATION_MISSING: cold-tier insert returned an invalid TID"
                            .to_owned(),
                    );
                }
                entry.node.cold_tid = Some(ItemPointer {
                    block_number,
                    offset_number,
                });
            }
            (None, None, None) => {}
            _ => {
                return Err(
                    "EC_GENERATION_CORRUPT: cold relation and prepared entry disagree".to_owned(),
                )
            }
        }
        let mut writer = unsafe {
            TupleSlotWriter::from_raw_slot(slot.as_ptr(), "ec_distann row-tier handoff")?
        };
        writer.clear();
        writer.validate_input_width(entry.datums.len())?;
        for (attribute_index, datum) in entry.datums.iter().enumerate() {
            if let Some(datum) = datum {
                writer.set_datum(attribute_index as i32, *datum);
            } else {
                writer.set_null(attribute_index as i32);
            }
        }
        let stored_slot = writer.store_virtual_tuple()?;
        unsafe { pg_sys::simple_table_tuple_insert(row_relation.as_ptr(), stored_slot) };
        let row_tid = unsafe { (*stored_slot).tts_tid };
        let (block_number, offset_number) = pgrx::itemptr::item_pointer_get_both(row_tid);
        if block_number == pg_sys::InvalidBlockNumber || offset_number == 0 {
            return Err(
                "EC_GENERATION_MISSING: row-tier insert returned an invalid TID".to_owned(),
            );
        }
        entry.node.heap_tid = ItemPointer {
            block_number,
            offset_number,
        };
        let graph_record = entry.node.encode_physical_version(
            graph_record_version,
            graph_degree,
            shape.code_stride,
        )?;
        graph_vec_ids.push(i64::from_le_bytes(entry.node.vec_id.to_le_bytes()));
        graph_records.push(graph_record);
        graph_row_tids.push(row_tid);
        match (&mut payloads, entry.payload_sidecar.take()) {
            (Some(payloads), Some(payload)) => payloads.push(payload),
            (None, None) => {}
            _ => {
                return Err(
                    "EC_GENERATION_CORRUPT: payload sidecar relation and prepared entry disagree"
                        .to_owned(),
                );
            }
        }
    }
    if graph_vec_ids.is_empty() {
        return Ok(());
    }
    let expected = i64::try_from(graph_vec_ids.len())
        .map_err(|_| "EC_BATCH_CONFLICT: graph insert batch is too large".to_owned())?;
    if let (Some(payload_sidecar_relation), Some(payloads)) = (payload_sidecar_relation, payloads) {
        let inserted = Spi::connect_mut(|client| {
            client
                .update(
                    &format!(
                        "WITH inserted AS (
                             INSERT INTO {payload_sidecar_relation} (row_tid, vec_id, payload)
                             SELECT row_tid, vec_id, payload
                               FROM unnest($1::tid[], $2::bigint[], $3::bytea[])
                                    AS batch(row_tid, vec_id, payload)
                             RETURNING 1
                         ) SELECT count(*)::bigint AS inserted_count FROM inserted"
                    ),
                    None,
                    &[
                        graph_row_tids.clone().into(),
                        graph_vec_ids.clone().into(),
                        payloads.into(),
                    ],
                )
                .map_err(|error| {
                    format!("EC_BATCH_CONFLICT: payload sidecar batch insert failed: {error}")
                })?
                .next()
                .ok_or_else(|| {
                    "EC_BATCH_CONFLICT: payload sidecar batch insert returned no count".to_owned()
                })?["inserted_count"]
                .value::<i64>()
                .map_err(|error| {
                    format!(
                        "EC_BATCH_CONFLICT: payload sidecar insert count decode failed: {error}"
                    )
                })?
                .ok_or_else(|| "EC_BATCH_CONFLICT: payload sidecar insert count is NULL".to_owned())
        })?;
        if inserted != expected {
            return Err(format!(
                "EC_BATCH_CONFLICT: payload sidecar batch inserted {inserted} rows, expected {expected}"
            ));
        }
    }
    let inserted = Spi::connect_mut(|client| {
        client
            .update(
                &format!(
                    "WITH inserted AS (
                         INSERT INTO {graph_relation} (vec_id, graph_record, row_tid)
                         SELECT vec_id, graph_record, row_tid
                           FROM unnest($1::bigint[], $2::bytea[], $3::tid[])
                                AS batch(vec_id, graph_record, row_tid)
                         RETURNING 1
                     ) SELECT count(*)::bigint AS inserted_count FROM inserted"
                ),
                None,
                &[
                    graph_vec_ids.into(),
                    graph_records.into(),
                    graph_row_tids.into(),
                ],
            )
            .map_err(|error| format!("EC_BATCH_CONFLICT: graph batch insert failed: {error}"))?
            .next()
            .ok_or_else(|| "EC_BATCH_CONFLICT: graph batch insert returned no count".to_owned())?
            ["inserted_count"]
            .value::<i64>()
            .map_err(|error| {
                format!("EC_BATCH_CONFLICT: graph insert count decode failed: {error}")
            })?
            .ok_or_else(|| "EC_BATCH_CONFLICT: graph insert count is NULL".to_owned())
    })?;
    if inserted != expected {
        return Err(format!(
            "EC_BATCH_CONFLICT: graph batch inserted {inserted} rows, expected {expected}"
        ));
    }
    Ok(())
}

unsafe fn send_datum(
    datum: pg_sys::Datum,
    attribute: &mut RowAttributeIo,
    attnum: usize,
) -> Result<Vec<u8>, String> {
    let sent = unsafe { pg_sys::SendFunctionCall(&mut attribute.send_flinfo, datum) };
    if sent.is_null() {
        return Err(format!(
            "EC_BUILD_INCOMPLETE: attribute {attnum} binary send returned NULL"
        ));
    }
    let bytes = unsafe { pgrx::varlena::varlena_to_byte_slice(sent.cast()) }.to_vec();
    unsafe { pg_sys::pfree(sent.cast()) };
    Ok(bytes)
}

fn fetch_frozen_row(
    row_relation: &HeapRelationGuard,
    slot: &TupleTableSlotGuard<'_>,
    row_tid: ItemPointer,
    identity_attnum: u16,
    row_io: &mut [Option<RowAttributeIo>],
) -> Result<([u8; 16], Vec<u8>, Vec<Vec<u8>>), String> {
    if row_tid.block_number == pg_sys::InvalidBlockNumber || row_tid.offset_number == 0 {
        return Err("EC_BUILD_INCOMPLETE: graph record has an invalid row-tier TID".to_owned());
    }
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        return Err("EC_BUILD_STATE: seal has no active PostgreSQL snapshot".to_owned());
    }
    let mut tid = pg_sys::ItemPointerData::default();
    pgrx::itemptr::item_pointer_set_all(&mut tid, row_tid.block_number, row_tid.offset_number);
    unsafe { pg_sys::ExecClearTuple(slot.as_ptr()) };
    let found = unsafe {
        pg_sys::table_tuple_fetch_row_version(
            row_relation.as_ptr(),
            &mut tid,
            snapshot,
            slot.as_ptr(),
        )
    };
    if !found {
        return Err("EC_BUILD_INCOMPLETE: graph record row-tier tuple is absent".to_owned());
    }
    let natts = i32::try_from(row_io.len())
        .map_err(|_| "EC_SCHEMA_MISMATCH: row tier is too wide".to_owned())?;
    unsafe { pg_sys::slot_getsomeattrs_int(slot.as_ptr(), natts) };
    let non_dropped_count = row_io.iter().filter(|io| io.is_some()).count();
    let mut null_bitmap = vec![0_u8; non_dropped_count.div_ceil(8)];
    let mut values = Vec::with_capacity(non_dropped_count);
    let mut identity = None;
    let mut non_dropped_position = 0_usize;
    for (attribute_index, io) in row_io.iter_mut().enumerate() {
        let attnum = attribute_index + 1;
        let Some(io) = io else {
            continue;
        };
        let is_null = unsafe { *(*slot.as_ptr()).tts_isnull.add(attribute_index) };
        if is_null {
            null_bitmap[non_dropped_position / 8] |= 1 << (non_dropped_position % 8);
            if attnum == usize::from(identity_attnum) {
                return Err("EC_SOURCE_IDENTITY: frozen source identity is NULL".to_owned());
            }
        } else {
            let datum = unsafe { *(*slot.as_ptr()).tts_values.add(attribute_index) };
            let bytes = unsafe { send_datum(datum, io, attnum)? };
            if attnum == usize::from(identity_attnum) {
                identity = Some(bytes.as_slice().try_into().map_err(|_| {
                    "EC_SOURCE_IDENTITY: frozen identity is not 16 bytes".to_owned()
                })?);
            }
            values.push(bytes);
        }
        non_dropped_position += 1;
    }
    let identity = identity.ok_or_else(|| {
        "EC_SOURCE_IDENTITY: frozen row does not contain its identity attribute".to_owned()
    })?;
    Ok((identity, null_bitmap, values))
}

fn fetch_compact_tier(
    relation: &HeapRelationGuard,
    slot: &TupleTableSlotGuard<'_>,
    row_tid: ItemPointer,
    expected_vec_id: u64,
    row_io: &mut [Option<RowAttributeIo>],
    tier_name: &str,
) -> Result<Vec<Option<Vec<u8>>>, String> {
    if row_tid.block_number == pg_sys::InvalidBlockNumber || row_tid.offset_number == 0 {
        return Err(format!(
            "EC_BUILD_INCOMPLETE: graph record has an invalid {tier_name}-tier TID"
        ));
    }
    if row_io.is_empty() {
        return Err(format!(
            "EC_SCHEMA_MISMATCH: compact {tier_name} tier lacks vec_id"
        ));
    }
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        return Err("EC_BUILD_STATE: seal has no active PostgreSQL snapshot".to_owned());
    }
    let mut tid = pg_sys::ItemPointerData::default();
    pgrx::itemptr::item_pointer_set_all(&mut tid, row_tid.block_number, row_tid.offset_number);
    unsafe { pg_sys::ExecClearTuple(slot.as_ptr()) };
    let found = unsafe {
        pg_sys::table_tuple_fetch_row_version(relation.as_ptr(), &mut tid, snapshot, slot.as_ptr())
    };
    if !found {
        return Err(format!(
            "EC_BUILD_INCOMPLETE: graph record {tier_name}-tier tuple is absent"
        ));
    }
    let natts = i32::try_from(row_io.len())
        .map_err(|_| format!("EC_SCHEMA_MISMATCH: {tier_name} tier is too wide"))?;
    unsafe { pg_sys::slot_getsomeattrs_int(slot.as_ptr(), natts) };
    if unsafe { *(*slot.as_ptr()).tts_isnull } {
        return Err(format!(
            "EC_BUILD_INCOMPLETE: {tier_name}-tier vec_id is NULL"
        ));
    }
    let stored_vec_id = unsafe {
        i64::from_datum(*(*slot.as_ptr()).tts_values, false)
            .ok_or_else(|| format!("EC_BUILD_INCOMPLETE: {tier_name}-tier vec_id is invalid"))?
    };
    if u64::from_le_bytes(stored_vec_id.to_le_bytes()) != expected_vec_id {
        return Err(format!(
            "EC_BUILD_INCOMPLETE: {tier_name}-tier vec_id disagrees with graph"
        ));
    }
    let mut values = vec![None; row_io.len()];
    for (attribute_index, io) in row_io.iter_mut().enumerate().skip(1) {
        let io = io.as_mut().ok_or_else(|| {
            format!(
                "EC_SCHEMA_MISMATCH: compact {tier_name}-tier attribute {} is dropped",
                attribute_index + 1
            )
        })?;
        if !unsafe { *(*slot.as_ptr()).tts_isnull.add(attribute_index) } {
            let datum = unsafe { *(*slot.as_ptr()).tts_values.add(attribute_index) };
            values[attribute_index] = Some(unsafe { send_datum(datum, io, attribute_index + 1)? });
        }
    }
    Ok(values)
}

struct FrozenHotColdRow {
    identity: [u8; 16],
    logical_null_bitmap: Vec<u8>,
    logical_values: Vec<Vec<u8>>,
    hot_null_bitmap: Vec<u8>,
    hot_values: Vec<Vec<u8>>,
    cold_null_bitmap: Vec<u8>,
    cold_values: Vec<Vec<u8>>,
}

#[allow(clippy::too_many_arguments)]
fn fetch_hot_cold_row(
    descriptor: &DistannGenerationDescriptor,
    layout: &DistannRowTierLayoutDescriptorV1,
    hot_relation: &HeapRelationGuard,
    hot_slot: &TupleTableSlotGuard<'_>,
    hot_tid: ItemPointer,
    hot_io: &mut [Option<RowAttributeIo>],
    cold_relation: &HeapRelationGuard,
    cold_slot: &TupleTableSlotGuard<'_>,
    cold_tid: ItemPointer,
    cold_io: &mut [Option<RowAttributeIo>],
    vec_id: u64,
) -> Result<FrozenHotColdRow, String> {
    let hot = fetch_compact_tier(hot_relation, hot_slot, hot_tid, vec_id, hot_io, "hot")?;
    let cold = fetch_compact_tier(cold_relation, cold_slot, cold_tid, vec_id, cold_io, "cold")?;
    let mut placed = Vec::with_capacity(layout.placements.len());
    for placement in &layout.placements {
        let physical_index = usize::from(placement.physical_ordinal)
            .checked_sub(1)
            .ok_or_else(|| "EC_SCHEMA_MISMATCH: compact row-tier ordinal is zero".to_owned())?;
        let value = match placement.tier {
            DistannRowTierV1::Hot => hot.get(physical_index),
            DistannRowTierV1::Cold => cold.get(physical_index),
        }
        .ok_or_else(|| "EC_SCHEMA_MISMATCH: compact row-tier ordinal exceeds width".to_owned())?
        .clone();
        placed.push((placement.attnum, placement.tier, value));
    }

    let logical_count = descriptor.row_schema.non_dropped_count();
    let mut logical_null_bitmap = vec![0_u8; logical_count.div_ceil(8)];
    let mut logical_values = Vec::with_capacity(logical_count);
    let mut identity = None;
    let mut logical_position = 0_usize;
    for attribute in &descriptor.row_schema.attributes {
        if attribute.dropped {
            continue;
        }
        let value = placed
            .iter()
            .find(|(attnum, _, _)| *attnum == attribute.attnum)
            .ok_or_else(|| {
                format!(
                    "EC_SCHEMA_MISMATCH: source attnum {} lacks a compact value",
                    attribute.attnum
                )
            })?
            .2
            .as_ref();
        match value {
            Some(bytes) => {
                if attribute.attnum == layout.source_identity_attnum {
                    identity = Some(bytes.as_slice().try_into().map_err(|_| {
                        "EC_SOURCE_IDENTITY: frozen identity is not 16 bytes".to_owned()
                    })?);
                }
                logical_values.push(bytes.clone());
            }
            None => {
                logical_null_bitmap[logical_position / 8] |= 1 << (logical_position % 8);
                if attribute.attnum == layout.source_identity_attnum {
                    return Err("EC_SOURCE_IDENTITY: frozen source identity is NULL".to_owned());
                }
                if attribute.attnum == layout.indexed_vector_attnum {
                    return Err("EC_SCHEMA_MISMATCH: frozen indexed vector is NULL".to_owned());
                }
            }
        }
        logical_position += 1;
    }

    let tier_content = |tier| {
        let values = placed
            .iter()
            .filter(|(_, placement_tier, _)| *placement_tier == tier)
            .map(|(_, _, value)| value)
            .collect::<Vec<_>>();
        let mut null_bitmap = vec![0_u8; values.len().div_ceil(8)];
        let mut non_null = Vec::with_capacity(values.len());
        for (position, value) in values.into_iter().enumerate() {
            if let Some(value) = value {
                non_null.push(value.clone());
            } else {
                null_bitmap[position / 8] |= 1 << (position % 8);
            }
        }
        (null_bitmap, non_null)
    };
    let (hot_null_bitmap, hot_values) = tier_content(DistannRowTierV1::Hot);
    let (cold_null_bitmap, cold_values) = tier_content(DistannRowTierV1::Cold);
    Ok(FrozenHotColdRow {
        identity: identity.ok_or_else(|| {
            "EC_SOURCE_IDENTITY: frozen row does not contain its identity attribute".to_owned()
        })?,
        logical_null_bitmap,
        logical_values,
        hot_null_bitmap,
        hot_values,
        cold_null_bitmap,
        cold_values,
    })
}

fn update_length_prefixed(hasher: &mut Sha256, bytes: &[u8]) -> Result<(), String> {
    let length = u32::try_from(bytes.len())
        .map_err(|_| "EC_BUILD_INCOMPLETE: persisted digest field exceeds u32".to_owned())?;
    hasher.update(length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn update_tier_content_digest(
    hasher: &mut Sha256,
    vec_id: u64,
    null_bitmap: &[u8],
    values: &[Vec<u8>],
) -> Result<(), String> {
    hasher.update(vec_id.to_le_bytes());
    update_length_prefixed(hasher, null_bitmap)?;
    hasher.update(
        u32::try_from(values.len())
            .map_err(|_| "EC_BUILD_INCOMPLETE: tier value count exceeds u32".to_owned())?
            .to_le_bytes(),
    );
    for value in values {
        update_length_prefixed(hasher, value)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct PhysicalPayloadSidecarSummary {
    row_count: u64,
    content_digest: [u8; 32],
}

fn scan_payload_sidecar(
    payload_sidecar_relation: &str,
    graph_relation: &str,
    cover: &DistannPayloadCoverDescriptorV1,
    require_current_graph_match: bool,
) -> Result<PhysicalPayloadSidecarSummary, String> {
    let mut hasher = Sha256::new();
    hasher.update(PAYLOAD_SIDECAR_INITIAL_CONTENT_DOMAIN);
    let mut row_count = 0_u64;
    let mut previous_identity = None;
    let sql = format!(
        "SELECT s.row_tid, s.vec_id, s.payload,
                EXISTS (
                    SELECT 1 FROM {graph_relation} g
                     WHERE g.is_current AND g.vec_id = s.vec_id
                       AND g.row_tid = s.row_tid
                ) AS graph_match
           FROM {payload_sidecar_relation} s
          ORDER BY (s.vec_id < 0), s.vec_id, s.row_tid"
    );
    Spi::connect(|client| -> Result<(), String> {
        let mut cursor = client.try_open_cursor(&sql, &[]).map_err(|error| {
            format!("EC_GENERATION_MISSING: payload sidecar cursor failed: {error}")
        })?;
        loop {
            let rows = cursor.fetch(256).map_err(|error| {
                format!("EC_GENERATION_MISSING: payload sidecar fetch failed: {error}")
            })?;
            if rows.is_empty() {
                break;
            }
            for row in rows {
                let row_tid = row["row_tid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| {
                        format!(
                            "EC_BUILD_INCOMPLETE: payload sidecar row TID decode failed: {error}"
                        )
                    })?
                    .ok_or_else(|| {
                        "EC_BUILD_INCOMPLETE: payload sidecar row TID is NULL".to_owned()
                    })?;
                let signed_vec_id = row["vec_id"]
                    .value::<i64>()
                    .map_err(|error| {
                        format!(
                            "EC_BUILD_INCOMPLETE: payload sidecar vec_id decode failed: {error}"
                        )
                    })?
                    .ok_or_else(|| {
                        "EC_BUILD_INCOMPLETE: payload sidecar vec_id is NULL".to_owned()
                    })?;
                let vec_id = u64::from_le_bytes(signed_vec_id.to_le_bytes());
                let payload = row["payload"]
                    .value::<Vec<u8>>()
                    .map_err(|error| {
                        format!("EC_BUILD_INCOMPLETE: payload sidecar value decode failed: {error}")
                    })?
                    .ok_or_else(|| {
                        "EC_BUILD_INCOMPLETE: payload sidecar value is NULL".to_owned()
                    })?;
                let graph_match = row["graph_match"]
                    .value::<bool>()
                    .map_err(|error| {
                        format!("EC_BUILD_INCOMPLETE: payload sidecar graph match decode failed: {error}")
                    })?
                    .ok_or_else(|| {
                        "EC_BUILD_INCOMPLETE: payload sidecar graph match is NULL".to_owned()
                    })?;
                let (block_number, offset_number) = pgrx::itemptr::item_pointer_get_both(row_tid);
                let canonical_tid = ItemPointer {
                    block_number,
                    offset_number,
                };
                let identity = (vec_id, block_number, offset_number);
                if previous_identity.is_some_and(|previous| identity <= previous)
                    || (require_current_graph_match
                        && previous_identity.is_some_and(|previous| vec_id == previous.0))
                {
                    return Err(
                        "EC_BUILD_INCOMPLETE: payload sidecar identity order is not canonical"
                            .to_owned(),
                    );
                }
                if (require_current_graph_match && !graph_match)
                    || block_number == pg_sys::InvalidBlockNumber
                    || offset_number == pg_sys::InvalidOffsetNumber
                {
                    return Err(
                        "EC_BUILD_INCOMPLETE: payload sidecar identity has no current graph match"
                            .to_owned(),
                    );
                }
                cover.decode_row(canonical_tid, vec_id, canonical_tid, vec_id, &payload)?;
                hasher.update(vec_id.to_le_bytes());
                hasher.update(block_number.to_le_bytes());
                hasher.update(offset_number.to_le_bytes());
                update_length_prefixed(&mut hasher, &payload)?;
                row_count = row_count.checked_add(1).ok_or_else(|| {
                    "EC_BUILD_INCOMPLETE: payload sidecar row count overflow".to_owned()
                })?;
                previous_identity = Some(identity);
            }
        }
        Ok(())
    })?;
    Ok(PhysicalPayloadSidecarSummary {
        row_count,
        content_digest: hasher.finalize().into(),
    })
}

#[derive(Debug)]
struct PhysicalSealSummary {
    record_count: u64,
    owner_stream_digest: [u8; 32],
    graph_digest: [u8; 32],
    row_tier_digest: [u8; 32],
    hot_tier_initial_content_digest: Option<[u8; 32]>,
    cold_tier_initial_content_digest: Option<[u8; 32]>,
    directory_digest: [u8; 32],
}

#[allow(clippy::too_many_arguments)]
fn scan_physical_generation(
    generation: &GenerationCatalogRow,
    descriptor: &DistannGenerationDescriptor,
    shape: DistannHandoffShape,
    identity_attnum: u16,
    row_relation: &HeapRelationGuard,
    cold_relation: Option<&HeapRelationGuard>,
    graph_relation: &str,
) -> Result<PhysicalSealSummary, String> {
    let slot = TupleTableSlotGuard::create_for_heap_guard(row_relation)
        .ok_or_else(|| "EC_GENERATION_MISSING: could not allocate seal row slot".to_owned())?;
    let mut row_io = unsafe { row_attribute_io(row_relation.as_ptr())? };
    let cold_slot = cold_relation
        .map(|relation| {
            TupleTableSlotGuard::create_for_heap_guard(relation).ok_or_else(|| {
                "EC_GENERATION_MISSING: could not allocate seal cold-tier slot".to_owned()
            })
        })
        .transpose()?;
    let mut cold_io = cold_relation
        .map(|relation| unsafe { row_attribute_io(relation.as_ptr()) })
        .transpose()?;
    match (descriptor.row_tier_layout(), cold_relation) {
        (None, None) | (Some(_), Some(_)) => {}
        _ => {
            return Err(
                "EC_GENERATION_MISSING: cold relation disagrees with generation descriptor"
                    .to_owned(),
            )
        }
    }
    let mut owner_hasher = DistannOwnerStreamHasher::new();
    let mut graph_hasher = Sha256::new();
    graph_hasher.update(PERSISTED_GRAPH_DOMAIN);
    let mut row_hasher = Sha256::new();
    row_hasher.update(PERSISTED_ROW_TIER_DOMAIN);
    let mut hot_hasher = descriptor.row_tier_layout().map(|_| {
        let mut hasher = Sha256::new();
        hasher.update(HOT_TIER_INITIAL_CONTENT_DOMAIN);
        hasher
    });
    let mut cold_hasher = descriptor.row_tier_layout().map(|_| {
        let mut hasher = Sha256::new();
        hasher.update(COLD_TIER_INITIAL_CONTENT_DOMAIN);
        hasher
    });
    let mut directory_hasher = Sha256::new();
    directory_hasher.update(LOCAL_DIRECTORY_DOMAIN);
    let mut record_count = 0_u64;
    let mut previous_vec_id = None;
    let sql = format!(
        "SELECT vec_id, graph_record, row_tid, ctid AS graph_tid
           FROM {graph_relation}
          WHERE is_current
          ORDER BY (vec_id < 0), vec_id"
    );
    Spi::connect(|client| -> Result<(), String> {
        let mut cursor = client
            .try_open_cursor(&sql, &[])
            .map_err(|error| format!("EC_GENERATION_MISSING: graph cursor failed: {error}"))?;
        loop {
            let rows = cursor
                .fetch(256)
                .map_err(|error| format!("EC_GENERATION_MISSING: graph fetch failed: {error}"))?;
            if rows.is_empty() {
                break;
            }
            for row in rows {
                let signed_vec_id = row["vec_id"]
                    .value::<i64>()
                    .map_err(|error| format!("EC_BUILD_INCOMPLETE: vec_id decode failed: {error}"))?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: graph vec_id is NULL".to_owned())?;
                let vec_id = u64::from_le_bytes(signed_vec_id.to_le_bytes());
                if previous_vec_id.is_some_and(|previous| vec_id <= previous) {
                    return Err(
                        "EC_DUPLICATE_VEC_ID: physical graph order is not strictly increasing"
                            .to_owned(),
                    );
                }
                let graph_record = row["graph_record"]
                    .value::<Vec<u8>>()
                    .map_err(|error| {
                        format!("EC_BUILD_INCOMPLETE: graph record decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: graph record is NULL".to_owned())?;
                let row_tid = row["row_tid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| {
                        format!("EC_BUILD_INCOMPLETE: row TID decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: row TID is NULL".to_owned())?;
                let graph_tid = row["graph_tid"]
                    .value::<pg_sys::ItemPointerData>()
                    .map_err(|error| {
                        format!("EC_BUILD_INCOMPLETE: graph TID decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: graph TID is NULL".to_owned())?;
                let node = DistannNodeTuple::decode_physical_version(
                    &graph_record,
                    descriptor.graph_record_version,
                    descriptor.graph_degree,
                    shape.code_stride,
                )?;
                let (row_block, row_offset) = pgrx::itemptr::item_pointer_get_both(row_tid);
                if node.tombstoned
                    || node.vec_id != vec_id
                    || node.heap_tid.block_number != row_block
                    || node.heap_tid.offset_number != row_offset
                    || owning_node(
                        vec_id,
                        descriptor.roster.len(),
                        descriptor.placement_hash_version,
                    ) != generation.owner_ordinal as usize
                {
                    return Err(
                        "EC_BUILD_INCOMPLETE: physical graph identity, locator, or owner mismatch"
                            .to_owned(),
                    );
                }
                let (source_identity, row_null_bitmap, row_values) = match (
                    descriptor.row_tier_layout(),
                    cold_relation,
                    cold_slot.as_ref(),
                    cold_io.as_deref_mut(),
                    node.cold_tid,
                ) {
                    (None, None, None, None, None) => fetch_frozen_row(
                        row_relation,
                        &slot,
                        node.heap_tid,
                        identity_attnum,
                        &mut row_io,
                    )?,
                    (
                        Some(layout),
                        Some(cold_relation),
                        Some(cold_slot),
                        Some(cold_io),
                        Some(cold_tid),
                    ) => {
                        let frozen = fetch_hot_cold_row(
                            descriptor,
                            layout,
                            row_relation,
                            &slot,
                            node.heap_tid,
                            &mut row_io,
                            cold_relation,
                            cold_slot,
                            cold_tid,
                            cold_io,
                            vec_id,
                        )?;
                        update_tier_content_digest(
                            hot_hasher.as_mut().expect("hot/cold descriptor"),
                            vec_id,
                            &frozen.hot_null_bitmap,
                            &frozen.hot_values,
                        )?;
                        update_tier_content_digest(
                            cold_hasher.as_mut().expect("hot/cold descriptor"),
                            vec_id,
                            &frozen.cold_null_bitmap,
                            &frozen.cold_values,
                        )?;
                        (
                            frozen.identity,
                            frozen.logical_null_bitmap,
                            frozen.logical_values,
                        )
                    }
                    _ => return Err(
                        "EC_BUILD_INCOMPLETE: graph cold locator disagrees with generation layout"
                            .to_owned(),
                    ),
                };
                if vec_id_from_source_identity(&source_identity) != vec_id {
                    return Err(
                        "EC_SOURCE_IDENTITY: frozen row identity differs from graph vec_id"
                            .to_owned(),
                    );
                }
                let live_neighbors = usize::from(node.neighbor_count);
                let entry = DistannHandoffEntry {
                    vec_id,
                    source_identity: source_identity.to_vec(),
                    graph_flags: 0,
                    search_code: node.search_code.clone(),
                    neighbor_vec_ids: node.neighbor_vec_ids[..live_neighbors].to_vec(),
                    neighbor_codes: node.neighbor_codes[..live_neighbors * shape.code_stride]
                        .to_vec(),
                    row_null_bitmap: row_null_bitmap.clone(),
                    row_values: row_values.clone(),
                };
                owner_hasher.update_entry(&entry, shape)?;

                graph_hasher.update(vec_id.to_le_bytes());
                update_length_prefixed(&mut graph_hasher, &graph_record)?;
                row_hasher.update(vec_id.to_le_bytes());
                update_length_prefixed(&mut row_hasher, &row_null_bitmap)?;
                let value_count = u32::try_from(row_values.len())
                    .map_err(|_| "EC_BUILD_INCOMPLETE: row value count exceeds u32".to_owned())?;
                row_hasher.update(value_count.to_le_bytes());
                for value in &row_values {
                    update_length_prefixed(&mut row_hasher, value)?;
                }
                let (graph_block, graph_offset) = pgrx::itemptr::item_pointer_get_both(graph_tid);
                if graph_block == pg_sys::InvalidBlockNumber || graph_offset == 0 {
                    return Err("EC_BUILD_INCOMPLETE: graph heap TID is invalid".to_owned());
                }
                directory_hasher.update(vec_id.to_le_bytes());
                directory_hasher.update(graph_block.to_le_bytes());
                directory_hasher.update(graph_offset.to_le_bytes());

                record_count = record_count.checked_add(1).ok_or_else(|| {
                    "EC_BUILD_INCOMPLETE: physical record count overflow".to_owned()
                })?;
                previous_vec_id = Some(vec_id);
            }
        }
        Ok(())
    })?;
    Ok(PhysicalSealSummary {
        record_count,
        owner_stream_digest: owner_hasher.digest(),
        graph_digest: graph_hasher.finalize().into(),
        row_tier_digest: row_hasher.finalize().into(),
        hot_tier_initial_content_digest: hot_hasher.map(|hasher| hasher.finalize().into()),
        cold_tier_initial_content_digest: cold_hasher.map(|hasher| hasher.finalize().into()),
        directory_digest: directory_hasher.finalize().into(),
    })
}

#[derive(Debug, Clone, Copy)]
struct GenerationSizes {
    graph_bytes: u64,
    row_tier_bytes: u64,
    cold_tier_bytes: Option<u64>,
    directory_bytes: u64,
    payload_sidecar_heap_bytes: Option<u64>,
    payload_sidecar_index_bytes: Option<u64>,
}

fn generation_sizes(generation: &GenerationCatalogRow) -> Result<GenerationSizes, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT pg_catalog.pg_table_size($1::oid::regclass)::bigint AS graph_bytes,
                        pg_catalog.pg_table_size($2::oid::regclass)::bigint AS row_tier_bytes,
                        pg_catalog.pg_total_relation_size($3::oid::regclass)::bigint
                            AS directory_bytes,
                        CASE WHEN $4::oid IS NULL THEN NULL
                             ELSE pg_catalog.pg_table_size($4::oid::regclass)::bigint
                         END AS cold_tier_bytes,
                        CASE WHEN $5::oid IS NULL THEN NULL
                             ELSE pg_catalog.pg_table_size($5::oid::regclass)::bigint
                         END AS payload_sidecar_heap_bytes,
                        CASE WHEN $6::oid IS NULL THEN NULL
                             ELSE pg_catalog.pg_total_relation_size($6::oid::regclass)::bigint
                         END AS payload_sidecar_index_bytes",
                None,
                &[
                    generation.graph_store_relid.into(),
                    generation.row_tier_relid.into(),
                    generation.directory_relid.into(),
                    generation.cold_tier_relid.into(),
                    generation.payload_sidecar_relid.into(),
                    generation.payload_sidecar_directory_relid.into(),
                ],
            )
            .map_err(|error| format!("EC_BUILD_INCOMPLETE: size lookup failed: {error}"))?
            .map(|row| {
                let required = |name: &str| -> Result<u64, String> {
                    let value = row[name]
                        .value::<i64>()
                        .map_err(|error| {
                            format!("EC_BUILD_INCOMPLETE: {name} decode failed: {error}")
                        })?
                        .ok_or_else(|| format!("EC_BUILD_INCOMPLETE: {name} is NULL"))?;
                    u64::try_from(value)
                        .map_err(|_| format!("EC_BUILD_INCOMPLETE: {name} is negative"))
                };
                let optional = |name: &str| -> Result<Option<u64>, String> {
                    row[name]
                        .value::<i64>()
                        .map_err(|error| {
                            format!("EC_BUILD_INCOMPLETE: {name} decode failed: {error}")
                        })?
                        .map(|value| {
                            u64::try_from(value)
                                .map_err(|_| format!("EC_BUILD_INCOMPLETE: {name} is negative"))
                        })
                        .transpose()
                };
                Ok::<GenerationSizes, String>(GenerationSizes {
                    graph_bytes: required("graph_bytes")?,
                    row_tier_bytes: required("row_tier_bytes")?,
                    cold_tier_bytes: optional("cold_tier_bytes")?,
                    directory_bytes: required("directory_bytes")?,
                    payload_sidecar_heap_bytes: optional("payload_sidecar_heap_bytes")?,
                    payload_sidecar_index_bytes: optional("payload_sidecar_index_bytes")?,
                })
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_BUILD_INCOMPLETE: size lookup returned no row".to_owned())
    })
}

fn relation_row_count(relation_name: &str) -> Result<u64, String> {
    let sql = format!("SELECT count(*)::bigint AS row_count FROM {relation_name}");
    Spi::connect(|client| {
        client
            .select(&sql, None, &[])
            .map_err(|error| format!("EC_BUILD_INCOMPLETE: row count failed: {error}"))?
            .map(|row| {
                let count = row["row_count"]
                    .value::<i64>()
                    .map_err(|error| {
                        format!("EC_BUILD_INCOMPLETE: row count decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: row count is NULL".to_owned())?;
                u64::try_from(count)
                    .map_err(|_| "EC_BUILD_INCOMPLETE: row count is negative".to_owned())
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_BUILD_INCOMPLETE: row count returned no row".to_owned())
    })
}

fn control_index_total_bytes(index_oid: pg_sys::Oid) -> Result<u64, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT pg_catalog.pg_total_relation_size($1::oid::regclass)::bigint AS bytes",
                None,
                &[index_oid.into()],
            )
            .map_err(|error| format!("EC_BUILD_INCOMPLETE: control size lookup failed: {error}"))?
            .map(|row| {
                let bytes = row["bytes"]
                    .value::<i64>()
                    .map_err(|error| {
                        format!("EC_BUILD_INCOMPLETE: control size decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: control size is NULL".to_owned())?;
                u64::try_from(bytes)
                    .map_err(|_| "EC_BUILD_INCOMPLETE: control size is negative".to_owned())
            })
            .next()
            .transpose()?
            .ok_or_else(|| "EC_BUILD_INCOMPLETE: control size returned no row".to_owned())
    })
}

#[derive(Debug)]
struct PhysicalTopologySummary {
    record_count: u64,
    owned_vec_id_digest: [u8; 32],
    graph_digest: [u8; 32],
    row_tier_digest: [u8; 32],
    non_owned_live_count: u64,
    non_owned_tombstone_count: u64,
    orphan_record_count: u64,
    orphan_row_count: u64,
}

/// Read-only diagnostic scan of a physical generation. Unlike the strict seal
/// validator (`scan_physical_generation`), this never errors on a non-owned,
/// tombstoned, or orphaned record — it classifies and counts them so the
/// coordinator and operators can audit a Building/Ready generation. Every
/// digest is recomputed from the physical relations and equals the Ready
/// receipt exactly when the generation is clean (all records owned, live, and
/// co-located with a single row-tier tuple).
fn diagnose_physical_generation(
    generation: &GenerationCatalogRow,
    descriptor: &DistannGenerationDescriptor,
    shape: DistannHandoffShape,
    identity_attnum: u16,
    row_relation: &HeapRelationGuard,
    graph_relation: &str,
    row_count: u64,
) -> Result<PhysicalTopologySummary, String> {
    let slot = TupleTableSlotGuard::create_for_heap_guard(row_relation)
        .ok_or_else(|| "EC_GENERATION_MISSING: could not allocate topology row slot".to_owned())?;
    let mut row_io = unsafe { row_attribute_io(row_relation.as_ptr())? };
    let mut owned_vec_id_hasher = Sha256::new();
    owned_vec_id_hasher.update(OWNED_VEC_IDS_DOMAIN);
    let mut graph_hasher = Sha256::new();
    graph_hasher.update(PERSISTED_GRAPH_DOMAIN);
    let mut row_hasher = Sha256::new();
    row_hasher.update(PERSISTED_ROW_TIER_DOMAIN);
    let mut record_count = 0_u64;
    let mut non_owned_live_count = 0_u64;
    let mut non_owned_tombstone_count = 0_u64;
    let mut orphan_record_count = 0_u64;
    let mut colocated_row_count = 0_u64;
    let roster_len = descriptor.roster.len();
    let owner_ordinal = generation.owner_ordinal as usize;
    let sql = format!(
        "SELECT vec_id, graph_record, row_tid, ctid AS graph_tid
           FROM {graph_relation}
          WHERE is_current
          ORDER BY (vec_id < 0), vec_id"
    );
    Spi::connect(|client| -> Result<(), String> {
        let mut cursor = client
            .try_open_cursor(&sql, &[])
            .map_err(|error| format!("EC_GENERATION_MISSING: topology cursor failed: {error}"))?;
        loop {
            let rows = cursor.fetch(256).map_err(|error| {
                format!("EC_GENERATION_MISSING: topology fetch failed: {error}")
            })?;
            if rows.is_empty() {
                break;
            }
            for row in rows {
                let signed_vec_id = row["vec_id"]
                    .value::<i64>()
                    .map_err(|error| format!("EC_BUILD_INCOMPLETE: vec_id decode failed: {error}"))?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: graph vec_id is NULL".to_owned())?;
                let vec_id = u64::from_le_bytes(signed_vec_id.to_le_bytes());
                let graph_record = row["graph_record"]
                    .value::<Vec<u8>>()
                    .map_err(|error| {
                        format!("EC_BUILD_INCOMPLETE: graph record decode failed: {error}")
                    })?
                    .ok_or_else(|| "EC_BUILD_INCOMPLETE: graph record is NULL".to_owned())?;
                let node = DistannNodeTuple::decode_physical_v1(
                    &graph_record,
                    descriptor.graph_degree,
                    shape.code_stride,
                )?;

                // Every physical graph record contributes to the graph digest in
                // vec_id order, matching the seal computation exactly.
                graph_hasher.update(vec_id.to_le_bytes());
                update_length_prefixed(&mut graph_hasher, &graph_record)?;
                record_count = record_count.checked_add(1).ok_or_else(|| {
                    "EC_BUILD_INCOMPLETE: topology record count overflow".to_owned()
                })?;

                let owned = owning_node(vec_id, roster_len, descriptor.placement_hash_version)
                    == owner_ordinal;
                if !owned {
                    if node.tombstoned {
                        non_owned_tombstone_count += 1;
                    } else {
                        non_owned_live_count += 1;
                    }
                    continue;
                }
                if node.tombstoned {
                    // Owned tombstones (post-FR-083 DML) carry no live row tier;
                    // they are neither orphans nor part of the owned-live digest.
                    continue;
                }
                // Owned and live: the frozen row must be co-located and share the
                // record's vec_id identity. Any failure is a co-location defect
                // (orphaned record), not a hard error in the diagnostic path.
                match fetch_frozen_row(
                    row_relation,
                    &slot,
                    node.heap_tid,
                    identity_attnum,
                    &mut row_io,
                ) {
                    Ok((source_identity, row_null_bitmap, row_values)) => {
                        if vec_id_from_source_identity(&source_identity) != vec_id {
                            orphan_record_count += 1;
                            continue;
                        }
                        owned_vec_id_hasher.update(vec_id.to_le_bytes());
                        row_hasher.update(vec_id.to_le_bytes());
                        update_length_prefixed(&mut row_hasher, &row_null_bitmap)?;
                        let value_count = u32::try_from(row_values.len()).map_err(|_| {
                            "EC_BUILD_INCOMPLETE: row value count exceeds u32".to_owned()
                        })?;
                        row_hasher.update(value_count.to_le_bytes());
                        for value in &row_values {
                            update_length_prefixed(&mut row_hasher, value)?;
                        }
                        colocated_row_count += 1;
                    }
                    Err(_) => {
                        orphan_record_count += 1;
                    }
                }
            }
        }
        Ok(())
    })?;
    // Row-tier tuples with no co-located owned-live record are orphaned rows.
    let orphan_row_count = row_count.saturating_sub(colocated_row_count);
    Ok(PhysicalTopologySummary {
        record_count,
        owned_vec_id_digest: owned_vec_id_hasher.finalize().into(),
        graph_digest: graph_hasher.finalize().into(),
        row_tier_digest: row_hasher.finalize().into(),
        non_owned_live_count,
        non_owned_tombstone_count,
        orphan_record_count,
        orphan_row_count,
    })
}

/// The 19-column physical topology row shared by the by-build-id and
/// by-fingerprint inspection endpoints.
type DistannTopologyRow = (
    i32,
    String,
    i64,
    i64,
    Vec<u8>,
    Vec<u8>,
    Vec<u8>,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    Option<i64>,
    Option<Vec<u8>>,
    Option<i64>,
    Option<i64>,
);

/// Diagnose one already-resolved physical generation and emit its 19-column
/// topology row. Decodes and identity-checks the descriptor, locks the physical
/// relations against concurrent reclaim, recomputes the counts/digests from
/// storage, and reads the exact relation sizes.
fn build_topology_row(
    index_oid: pg_sys::Oid,
    generation: &GenerationCatalogRow,
    control_owner: pg_sys::Oid,
) -> Result<DistannTopologyRow, String> {
    let descriptor = DistannGenerationDescriptor::decode(&generation.generation_descriptor)?;
    if descriptor.digest()? != generation.generation_descriptor_digest
        || roster_digest(&descriptor.roster)? != generation.roster_digest
    {
        return Err(
            "EC_GENERATION_DESCRIPTOR: cataloged generation descriptor identity is corrupt"
                .to_owned(),
        );
    }
    let shape = DistannHandoffShape::from_descriptor(&descriptor)?;
    // Hold AccessShareLock on each physical relation so a concurrent retirement
    // reclaim (which drops them under AccessExclusiveLock) cannot delete storage
    // mid-inspection.
    let row_relation = HeapRelationGuard::try_open(
        generation.row_tier_relid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
    )
    .ok_or_else(|| "EC_GENERATION_MISSING: row-tier relation is absent".to_owned())?;
    let _graph_relation_guard = HeapRelationGuard::try_open(
        generation.graph_store_relid,
        pg_sys::AccessShareLock as pg_sys::LOCKMODE,
    )
    .ok_or_else(|| "EC_GENERATION_MISSING: graph-store relation is absent".to_owned())?;
    // The local directory is a unique index on the graph relation, not a heap,
    // so it is lock-held by OID rather than opened as a table.
    unsafe {
        pg_sys::LockRelationOid(
            generation.directory_relid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        )
    };
    let _payload_sidecar_guard = generation
        .payload_sidecar_relid
        .map(|relation_oid| {
            HeapRelationGuard::try_open(relation_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
                .ok_or_else(|| {
                    "EC_GENERATION_MISSING: payload-sidecar relation is absent".to_owned()
                })
        })
        .transpose()?;
    if let Some(directory_relid) = generation.payload_sidecar_directory_relid {
        unsafe {
            pg_sys::LockRelationOid(directory_relid, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
        };
    }
    validate_generation_relations(generation, &descriptor, control_owner)?;
    let graph_relation = qualified_relation_name(generation.graph_store_relid)?;
    let row_relation_name = qualified_relation_name(generation.row_tier_relid)?;
    let identity_attnum = identity_attnum(index_oid)?;
    let row_count = relation_row_count(&row_relation_name)?;
    let payload_sidecar = match (
        generation.payload_sidecar_relid,
        descriptor.payload_cover.as_ref(),
    ) {
        (Some(relation_oid), Some(cover)) => Some(scan_payload_sidecar(
            &qualified_relation_name(relation_oid)?,
            &graph_relation,
            cover,
            false,
        )?),
        (None, None) => None,
        _ => {
            return Err(
                "EC_GENERATION_MISSING: payload sidecar relation disagrees with descriptor"
                    .to_owned(),
            );
        }
    };
    let summary = with_restricted_type_io_owner(control_owner, || {
        diagnose_physical_generation(
            generation,
            &descriptor,
            shape,
            identity_attnum,
            &row_relation,
            &graph_relation,
            row_count,
        )
    })?;
    let sizes = generation_sizes(generation)?;
    let control_index_bytes = control_index_total_bytes(index_oid)?;
    let to_i64 = |value: u64, field: &str| -> Result<i64, String> {
        i64::try_from(value).map_err(|_| format!("EC_BUILD_INCOMPLETE: {field} exceeds bigint"))
    };
    Ok((
        i32::try_from(generation.node_id)
            .map_err(|_| "EC_BUILD_INCOMPLETE: node id exceeds integer".to_owned())?,
        generation.state.to_string(),
        to_i64(summary.record_count, "record_count")?,
        to_i64(row_count, "row_count")?,
        summary.owned_vec_id_digest.to_vec(),
        summary.graph_digest.to_vec(),
        summary.row_tier_digest.to_vec(),
        to_i64(summary.non_owned_live_count, "non_owned_live_count")?,
        to_i64(
            summary.non_owned_tombstone_count,
            "non_owned_tombstone_count",
        )?,
        to_i64(summary.orphan_record_count, "orphan_record_count")?,
        to_i64(summary.orphan_row_count, "orphan_row_count")?,
        to_i64(sizes.graph_bytes, "graph_bytes")?,
        to_i64(sizes.row_tier_bytes, "row_tier_bytes")?,
        to_i64(sizes.directory_bytes, "directory_bytes")?,
        to_i64(control_index_bytes, "control_index_bytes")?,
        payload_sidecar
            .map(|sidecar| to_i64(sidecar.row_count, "payload_sidecar_row_count"))
            .transpose()?,
        payload_sidecar.map(|sidecar| sidecar.content_digest.to_vec()),
        sizes
            .payload_sidecar_heap_bytes
            .map(|bytes| to_i64(bytes, "payload_sidecar_heap_bytes"))
            .transpose()?,
        sizes
            .payload_sidecar_index_bytes
            .map(|bytes| to_i64(bytes, "payload_sidecar_index_bytes"))
            .transpose()?,
    ))
}

/// Report the physical topology of a build-id-selected generation. Only
/// in-progress `Building`/`Ready` generations are reported by build id;
/// Published and retained Retired generations are inspected by fingerprint
/// through `ec_distann_epoch_topology`, and Reclaimed/Aborted/absent build ids
/// carry no physical generation and yield no rows. All counts and digests are
/// recomputed from the live physical relations, never from expected manifest
/// fields. The sidecar digest therefore diverges from the immutable Ready
/// receipt's initial-content digest after any valid post-Ready DML.
#[pg_extern(stable, strict, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_generation_topology(
    index_regclass: PgRelation,
    build_id: Uuid,
) -> TableIterator<
    'static,
    (
        name!(node_id, i32),
        name!(state, String),
        name!(record_count, i64),
        name!(row_count, i64),
        name!(owned_vec_id_digest, Vec<u8>),
        name!(graph_digest, Vec<u8>),
        name!(row_tier_digest, Vec<u8>),
        name!(non_owned_live_count, i64),
        name!(non_owned_tombstone_count, i64),
        name!(orphan_record_count, i64),
        name!(orphan_row_count, i64),
        name!(graph_bytes, i64),
        name!(row_tier_bytes, i64),
        name!(directory_bytes, i64),
        name!(control_index_bytes, i64),
        name!(payload_sidecar_row_count, Option<i64>),
        name!(payload_sidecar_live_content_digest, Option<Vec<u8>>),
        name!(payload_sidecar_heap_bytes, Option<i64>),
        name!(payload_sidecar_index_bytes, Option<i64>),
    ),
> {
    let rows = (|| -> Result<Vec<DistannTopologyRow>, String> {
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let index_oid = index_regclass.oid();
        let (_control_guard, control_handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "ec_distann_generation_topology",
        )?;
        let (_, control_owner, _) = relation_namespace_owner_persistence_handle(control_handle);
        if control_owner == pg_sys::InvalidOid {
            return Err("EC_SCHEMA_MISMATCH: control relation owner is invalid".to_owned());
        }
        let Some(generation) =
            generation_catalog::lookup_generation(index_oid, logical_index_uuid, build_id)?
        else {
            return Ok(Vec::new());
        };
        if !matches!(
            generation.state,
            GenerationState::Building | GenerationState::Ready
        ) {
            return Ok(Vec::new());
        }
        Ok(vec![build_topology_row(
            index_oid,
            &generation,
            control_owner,
        )?])
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::new(rows.into_iter())
}

/// Report the physical topology of a Published or retained Retired generation
/// selected by its 34-byte versioned epoch fingerprint.
/// Building/Ready generations are selectable only by build id; an unknown
/// fingerprint version fails `EC_EPOCH_FINGERPRINT_VERSION`, and a fingerprint
/// that resolves to no retained generation (unknown, in-progress, or Reclaimed)
/// fails `EC_GENERATION_MISSING`. All counts and digests are recomputed from the
/// physical relations.
#[pg_extern(stable, strict, parallel_restricted)]
#[allow(clippy::type_complexity)]
fn ec_distann_epoch_topology(
    index_regclass: PgRelation,
    epoch_fingerprint: Vec<u8>,
) -> TableIterator<
    'static,
    (
        name!(node_id, i32),
        name!(state, String),
        name!(record_count, i64),
        name!(row_count, i64),
        name!(owned_vec_id_digest, Vec<u8>),
        name!(graph_digest, Vec<u8>),
        name!(row_tier_digest, Vec<u8>),
        name!(non_owned_live_count, i64),
        name!(non_owned_tombstone_count, i64),
        name!(orphan_record_count, i64),
        name!(orphan_row_count, i64),
        name!(graph_bytes, i64),
        name!(row_tier_bytes, i64),
        name!(directory_bytes, i64),
        name!(control_index_bytes, i64),
        name!(payload_sidecar_row_count, Option<i64>),
        name!(payload_sidecar_live_content_digest, Option<Vec<u8>>),
        name!(payload_sidecar_heap_bytes, Option<i64>),
        name!(payload_sidecar_index_bytes, Option<i64>),
    ),
> {
    let rows = (|| -> Result<Vec<DistannTopologyRow>, String> {
        DistannEpochFingerprint::decode(&epoch_fingerprint)?;
        let index_oid = index_regclass.oid();
        let (_control_guard, control_handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            "ec_distann_epoch_topology",
        )?;
        let (_, control_owner, _) = relation_namespace_owner_persistence_handle(control_handle);
        if control_owner == pg_sys::InvalidOid {
            return Err("EC_SCHEMA_MISMATCH: control relation owner is invalid".to_owned());
        }
        // A participant does not persist the coordinator's publish-decision
        // row. Resolve directly through its unique retained generation
        // fingerprint so the same endpoint works on every real instance.
        let fingerprint: [u8; 34] = epoch_fingerprint
            .as_slice()
            .try_into()
            .expect("length checked above");
        let Some(retained) = generation_catalog::lookup_retained_generation_by_fingerprint(
            index_oid,
            logical_index_uuid,
            &fingerprint,
        )?
        else {
            return Err(
                "EC_GENERATION_MISSING: the epoch generation has been reclaimed".to_owned(),
            );
        };
        let generation = retained.generation;
        if !matches!(
            generation.state,
            GenerationState::Published | GenerationState::Retired
        ) {
            return Err(
                "EC_GENERATION_MISSING: epoch generation is not Published or retained Retired"
                    .to_owned(),
            );
        }
        Ok(vec![build_topology_row(
            index_oid,
            &generation,
            control_owner,
        )?])
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::new(rows.into_iter())
}

#[pg_extern(volatile, strict, parallel_restricted)]
fn ec_distann_stage_epoch_batch(
    index_regclass: PgRelation,
    build_id: Uuid,
    batch_seq: i64,
    batch_digest: Vec<u8>,
    encoded_batch: Vec<u8>,
) -> TableIterator<
    'static,
    (
        name!(accepted_record_count, i64),
        name!(cumulative_record_count, i64),
        name!(cumulative_owner_digest, Vec<u8>),
    ),
> {
    let result = (|| -> Result<StageResult, String> {
        super::lifecycle_guard::require_read_committed("ec_distann_stage_epoch_batch")?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let batch_seq = u64::try_from(batch_seq)
            .map_err(|_| "EC_BATCH_SEQUENCE: batch sequence is negative".to_owned())?;
        let supplied_digest = fixed_digest(batch_digest, "EC_HANDOFF_DIGEST", "batch digest")?;
        let verified_digest = DistannHandoffBatch::verified_digest(&encoded_batch)?;
        if supplied_digest != verified_digest {
            return Err(
                "EC_HANDOFF_DIGEST: supplied batch digest differs from encoded batch".to_owned(),
            );
        }

        let index_oid = index_regclass.oid();
        let (_control_guard, control_handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_stage_epoch_batch",
        )?;
        let (_, control_owner, _) = relation_namespace_owner_persistence_handle(control_handle);
        if control_owner == pg_sys::InvalidOid {
            return Err("EC_SCHEMA_MISMATCH: control relation owner is invalid".to_owned());
        }
        let generation = generation_catalog::lookup_generation_for_update(
            index_oid,
            logical_index_uuid,
            build_id,
        )?
        .ok_or_else(|| "EC_GENERATION_MISSING: generation is absent".to_owned())?;

        if let Some(prior) = generation_catalog::lookup_generation_batch(
            index_oid,
            logical_index_uuid,
            build_id,
            batch_seq,
        )? {
            if prior.batch_digest == supplied_digest
                && prior.encoded_bytes == encoded_batch.len() as u64
            {
                return stage_result(&prior);
            }
            return Err(
                "EC_BATCH_CONFLICT: acknowledged batch identity differs from replay".to_owned(),
            );
        }
        if generation.state != GenerationState::Building {
            return Err(format!(
                "EC_BUILD_STATE: cannot stage an unacknowledged batch in state {}",
                generation.state
            ));
        }
        if batch_seq != generation.next_batch_seq {
            return Err(format!(
                "EC_BATCH_SEQUENCE: got sequence {batch_seq}, expected {}",
                generation.next_batch_seq
            ));
        }

        let descriptor = DistannGenerationDescriptor::decode(&generation.generation_descriptor)?;
        if descriptor.digest()? != generation.generation_descriptor_digest
            || roster_digest(&descriptor.roster)? != generation.roster_digest
        {
            return Err(
                "EC_GENERATION_DESCRIPTOR: cataloged generation descriptor identity is corrupt"
                    .to_owned(),
            );
        }
        let shape = DistannHandoffShape::from_descriptor(&descriptor)?;
        let batch = DistannHandoffBatch::decode(&encoded_batch, shape)?;
        if batch.digest(shape)? != supplied_digest
            || batch.build_id != *build_id.as_bytes()
            || batch.epoch != generation.epoch
            || batch.batch_seq != batch_seq
            || batch.build_spec_digest != generation.build_spec_digest
            || batch.row_schema_fingerprint != descriptor.row_schema.fingerprint()?
            || batch.index_format_version != DISTANN_PHYSICAL_INDEX_FORMAT_VERSION
            || batch.neighbor_codec_kind != descriptor.neighbor_codec_kind
        {
            return Err("EC_HANDOFF_FORMAT: batch envelope differs from its generation".to_owned());
        }
        if batch.entries.is_empty() && batch_seq != 0 {
            return Err("EC_BATCH_SEQUENCE: only sequence zero may be empty".to_owned());
        }
        let accepted_record_count = u64::try_from(batch.entries.len())
            .map_err(|_| "EC_BUILD_INCOMPLETE: batch record count exceeds u64".to_owned())?;
        let new_cumulative_record_count = generation
            .cumulative_record_count
            .checked_add(accepted_record_count)
            .filter(|count| *count <= generation.expected_owner_count)
            .ok_or_else(|| {
                "EC_BUILD_INCOMPLETE: batch exceeds the expected owner record count".to_owned()
            })?;
        let previous_vec_id = generation.last_vec_id_le.map(u64::from_le_bytes);
        if let Some(first) = batch.entries.first() {
            if previous_vec_id.is_some_and(|previous| first.vec_id <= previous) {
                return Err(
                    "EC_HANDOFF_FORMAT: owner stream vec_ids are not strictly increasing"
                        .to_owned(),
                );
            }
        }
        let node_count = descriptor.roster.len();
        for entry in &batch.entries {
            let identity: [u8; 16] =
                entry.source_identity.as_slice().try_into().map_err(|_| {
                    "EC_SOURCE_IDENTITY: source identity is not 16 bytes".to_owned()
                })?;
            if vec_id_from_source_identity(&identity) != entry.vec_id {
                return Err(
                    "EC_SOURCE_IDENTITY: vec_id differs from its source identity".to_owned(),
                );
            }
            if owning_node(entry.vec_id, node_count, descriptor.placement_hash_version)
                != generation.owner_ordinal as usize
            {
                return Err("EC_WRONG_OWNER: participant does not own batch vec_id".to_owned());
            }
        }

        let mut owner_hasher = DistannOwnerStreamHasher::restore(
            &generation.owner_stream_sha256_state,
            generation.cumulative_owner_digest,
        )?;
        for entry in &batch.entries {
            owner_hasher.update_entry(entry, shape)?;
        }
        let new_cumulative_owner_digest = owner_hasher.digest();
        let new_owner_hash_state = owner_hasher.serialize();
        let last_vec_id_le = batch
            .entries
            .last()
            .map(|entry| entry.vec_id.to_le_bytes())
            .or(generation.last_vec_id_le);

        let row_relation = HeapRelationGuard::try_open(
            generation.row_tier_relid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
        )
        .ok_or_else(|| "EC_GENERATION_MISSING: row-tier relation is absent".to_owned())?;
        let cold_relation = generation
            .cold_tier_relid
            .map(|relation_oid| {
                HeapRelationGuard::try_open(
                    relation_oid,
                    pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
                )
                .ok_or_else(|| "EC_GENERATION_MISSING: cold-tier relation is absent".to_owned())
            })
            .transpose()?;
        let _graph_relation_guard = HeapRelationGuard::try_open(
            generation.graph_store_relid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
        )
        .ok_or_else(|| "EC_GENERATION_MISSING: graph-store relation is absent".to_owned())?;
        let _payload_sidecar_guard = generation
            .payload_sidecar_relid
            .map(|relation_oid| {
                HeapRelationGuard::try_open(
                    relation_oid,
                    pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
                )
                .ok_or_else(|| {
                    "EC_GENERATION_MISSING: payload-sidecar relation is absent".to_owned()
                })
            })
            .transpose()?;
        validate_generation_relations(&generation, &descriptor, control_owner)?;
        let graph_relation = qualified_relation_name(generation.graph_store_relid)?;
        let payload_sidecar_relation = generation
            .payload_sidecar_relid
            .map(qualified_relation_name)
            .transpose()?;
        reject_existing_vec_ids(&graph_relation, &batch.entries)?;
        let identity_attnum = identity_attnum(index_oid)?;
        let vector_attnum = indexed_vector_attnum(index_oid)?;
        let mut row_io = unsafe { row_attribute_io(row_relation.as_ptr())? };
        let mut cold_io = cold_relation
            .as_ref()
            .map(|relation| unsafe { row_attribute_io(relation.as_ptr()) })
            .transpose()?;
        let mut prepared = with_restricted_type_io_owner(control_owner, || {
            match (descriptor.row_tier_layout(), cold_io.as_deref_mut()) {
                (Some(layout), Some(cold_io)) => prepare_hot_cold_entries(
                    &batch.entries,
                    shape,
                    identity_attnum,
                    &descriptor.row_schema,
                    layout,
                    &mut row_io,
                    cold_io,
                ),
                (None, None) => prepare_legacy_entries(
                    &batch.entries,
                    shape,
                    identity_attnum,
                    vector_attnum,
                    &mut row_io,
                    descriptor.payload_cover.as_ref(),
                ),
                _ => Err(
                    "EC_GENERATION_MISSING: cold relation disagrees with the generation descriptor"
                        .to_owned(),
                ),
            }
        })?;
        if super::options::debug_fail_handoff_after_prepare() {
            return Err(
                "EC_FAULT_INJECTED: handoff failed after preparation before insertion".to_owned(),
            );
        }

        insert_prepared_entries(
            &row_relation,
            cold_relation.as_ref(),
            &graph_relation,
            payload_sidecar_relation.as_deref(),
            &mut prepared,
            shape,
            descriptor.graph_record_version,
        )?;
        let journal = GenerationBatchCatalogRow {
            batch_seq,
            batch_digest: supplied_digest,
            encoded_bytes: encoded_batch.len() as u64,
            accepted_record_count,
            cumulative_record_count: new_cumulative_record_count,
            cumulative_owner_digest: new_cumulative_owner_digest,
        };
        generation_catalog::insert_generation_batch(
            index_oid,
            logical_index_uuid,
            build_id,
            &journal,
        )?;
        generation_catalog::advance_generation_after_batch(
            index_oid,
            logical_index_uuid,
            build_id,
            generation.next_batch_seq,
            generation.cumulative_record_count,
            generation.cumulative_owner_digest,
            new_cumulative_record_count,
            new_cumulative_owner_digest,
            last_vec_id_le,
            new_owner_hash_state,
        )?;
        stage_result(&journal)
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"));
    TableIterator::once(result)
}

#[pg_extern(volatile, strict, parallel_restricted)]
fn ec_distann_seal_epoch_handoff(
    index_regclass: PgRelation,
    build_id: Uuid,
    expected_owner_count: i64,
    expected_owner_digest: Vec<u8>,
) -> Vec<u8> {
    (|| -> Result<Vec<u8>, String> {
        super::lifecycle_guard::require_read_committed("ec_distann_seal_epoch_handoff")?;
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let expected_owner_count = u64::try_from(expected_owner_count)
            .map_err(|_| "EC_BUILD_INCOMPLETE: expected owner count is negative".to_owned())?;
        let expected_owner_digest = fixed_digest(
            expected_owner_digest,
            "EC_HANDOFF_DIGEST",
            "expected owner digest",
        )?;
        let index_oid = index_regclass.oid();
        let (_control_guard, control_handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_seal_epoch_handoff",
        )?;
        let (_, control_owner, _) = relation_namespace_owner_persistence_handle(control_handle);
        if control_owner == pg_sys::InvalidOid {
            return Err("EC_SCHEMA_MISMATCH: control relation owner is invalid".to_owned());
        }
        let generation = generation_catalog::lookup_generation_for_update(
            index_oid,
            logical_index_uuid,
            build_id,
        )?
        .ok_or_else(|| "EC_GENERATION_MISSING: generation is absent".to_owned())?;
        if generation.expected_owner_count != expected_owner_count
            || generation.expected_owner_digest != expected_owner_digest
        {
            return Err("EC_BUILD_INCOMPLETE: seal expectation differs from begin".to_owned());
        }
        if matches!(
            generation.state,
            GenerationState::Ready | GenerationState::Published | GenerationState::Retired
        ) {
            return generation.ready_receipt.clone().ok_or_else(|| {
                "EC_BUILD_STATE: non-Building generation has no Ready receipt".to_owned()
            });
        }
        if generation.state != GenerationState::Building {
            return Err(format!(
                "EC_BUILD_STATE: cannot seal generation state {}",
                generation.state
            ));
        }
        if generation.next_batch_seq == 0 {
            return Err("EC_BUILD_INCOMPLETE: sequence zero was not acknowledged".to_owned());
        }
        if generation.cumulative_record_count != expected_owner_count
            || generation.cumulative_owner_digest != expected_owner_digest
        {
            return Err(
                "EC_BUILD_INCOMPLETE: cumulative owner count or digest differs from expectation"
                    .to_owned(),
            );
        }
        let journal =
            generation_catalog::generation_batch_summary(index_oid, logical_index_uuid, build_id)?
                .ok_or_else(|| "EC_BUILD_INCOMPLETE: generation has no batch journal".to_owned())?;
        if journal.batch_count != generation.next_batch_seq
            || journal.minimum_sequence != 0
            || journal.maximum_sequence != generation.next_batch_seq - 1
            || journal.accepted_record_count != generation.cumulative_record_count
        {
            return Err(
                "EC_BUILD_INCOMPLETE: batch journal is missing or non-contiguous".to_owned(),
            );
        }

        let descriptor = DistannGenerationDescriptor::decode(&generation.generation_descriptor)?;
        if descriptor.digest()? != generation.generation_descriptor_digest
            || roster_digest(&descriptor.roster)? != generation.roster_digest
        {
            return Err(
                "EC_GENERATION_DESCRIPTOR: cataloged generation descriptor identity is corrupt"
                    .to_owned(),
            );
        }
        let shape = DistannHandoffShape::from_descriptor(&descriptor)?;
        let row_relation = HeapRelationGuard::try_open(
            generation.row_tier_relid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
        )
        .ok_or_else(|| "EC_GENERATION_MISSING: row-tier relation is absent".to_owned())?;
        let cold_relation = generation
            .cold_tier_relid
            .map(|relation_oid| {
                HeapRelationGuard::try_open(
                    relation_oid,
                    pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
                )
                .ok_or_else(|| "EC_GENERATION_MISSING: cold-tier relation is absent".to_owned())
            })
            .transpose()?;
        let _graph_relation_guard = HeapRelationGuard::try_open(
            generation.graph_store_relid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
        )
        .ok_or_else(|| "EC_GENERATION_MISSING: graph-store relation is absent".to_owned())?;
        unsafe {
            pg_sys::LockRelationOid(
                generation.directory_relid,
                pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            )
        };
        let _payload_sidecar_guard = generation
            .payload_sidecar_relid
            .map(|relation_oid| {
                HeapRelationGuard::try_open(
                    relation_oid,
                    pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
                )
                .ok_or_else(|| {
                    "EC_GENERATION_MISSING: payload-sidecar relation is absent".to_owned()
                })
            })
            .transpose()?;
        if let Some(directory_relid) = generation.payload_sidecar_directory_relid {
            unsafe {
                pg_sys::LockRelationOid(
                    directory_relid,
                    pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
                )
            };
        }
        validate_generation_relations(&generation, &descriptor, control_owner)?;
        let graph_relation = qualified_relation_name(generation.graph_store_relid)?;
        let row_relation_name = qualified_relation_name(generation.row_tier_relid)?;
        let payload_sidecar_relation = generation
            .payload_sidecar_relid
            .map(qualified_relation_name)
            .transpose()?;
        let identity_attnum = identity_attnum(index_oid)?;
        let physical = with_restricted_type_io_owner(control_owner, || {
            scan_physical_generation(
                &generation,
                &descriptor,
                shape,
                identity_attnum,
                &row_relation,
                cold_relation.as_ref(),
                &graph_relation,
            )
        })?;
        let row_count = relation_row_count(&row_relation_name)?;
        let cold_count = generation
            .cold_tier_relid
            .map(|relation_oid| {
                qualified_relation_name(relation_oid).and_then(|name| relation_row_count(&name))
            })
            .transpose()?;
        let payload_sidecar = match (
            payload_sidecar_relation.as_deref(),
            descriptor.payload_cover.as_ref(),
        ) {
            (Some(relation), Some(cover)) => Some(scan_payload_sidecar(
                relation,
                &graph_relation,
                cover,
                true,
            )?),
            (None, None) => None,
            _ => {
                return Err(
                    "EC_GENERATION_MISSING: payload sidecar relation disagrees with descriptor"
                        .to_owned(),
                );
            }
        };
        if physical.record_count != expected_owner_count
            || row_count != expected_owner_count
            || cold_count.is_some_and(|count| count != expected_owner_count)
            || physical.owner_stream_digest != expected_owner_digest
            || payload_sidecar.is_some_and(|sidecar| sidecar.row_count != expected_owner_count)
        {
            return Err(
                "EC_BUILD_INCOMPLETE: physical row/record count or owner digest mismatch"
                    .to_owned(),
            );
        }
        let sizes = generation_sizes(&generation)?;
        let payload_sidecar = match (
            payload_sidecar,
            sizes.payload_sidecar_heap_bytes,
            sizes.payload_sidecar_index_bytes,
        ) {
            (Some(sidecar), Some(heap_bytes), Some(index_bytes)) => {
                Some(DistannReadyReceiptPayloadSidecar {
                    initial_content_digest: sidecar.content_digest,
                    heap_bytes,
                    index_bytes,
                })
            }
            (None, None, None) => None,
            _ => {
                return Err(
                    "EC_BUILD_INCOMPLETE: payload sidecar relation sizes are incomplete".to_owned(),
                );
            }
        };
        let hot_cold = match (
            physical.hot_tier_initial_content_digest,
            physical.cold_tier_initial_content_digest,
            sizes.cold_tier_bytes,
            descriptor.row_tier_layout(),
        ) {
            (Some(hot_digest), Some(cold_digest), Some(cold_heap_bytes), Some(_)) => {
                Some(DistannReadyReceiptHotCold {
                    hot_initial_content_digest: hot_digest,
                    cold_initial_content_digest: cold_digest,
                    hot_heap_bytes: sizes.row_tier_bytes,
                    cold_heap_bytes,
                })
            }
            (None, None, None, None) => None,
            _ => {
                return Err(
                    "EC_BUILD_INCOMPLETE: hot/cold digests or relation sizes are incomplete"
                        .to_owned(),
                )
            }
        };
        let row_tier_bytes = sizes
            .row_tier_bytes
            .checked_add(sizes.cold_tier_bytes.unwrap_or(0))
            .ok_or_else(|| "EC_BUILD_INCOMPLETE: row-tier byte count overflow".to_owned())?;
        let receipt = DistannReadyReceipt {
            node_id: generation.node_id,
            epoch: generation.epoch,
            build_id: *build_id.as_bytes(),
            build_spec_digest: generation.build_spec_digest,
            generation_descriptor_digest: generation.generation_descriptor_digest,
            last_acknowledged_batch_sequence: generation.next_batch_seq - 1,
            owned_record_count: physical.record_count,
            row_count,
            owner_stream_digest: physical.owner_stream_digest,
            persisted_graph_digest: physical.graph_digest,
            persisted_row_tier_digest: physical.row_tier_digest,
            local_directory_digest: physical.directory_digest,
            graph_bytes: sizes.graph_bytes,
            row_tier_bytes,
            directory_bytes: sizes.directory_bytes,
            state: DISTANN_READY_RECEIPT_STATE,
            payload_sidecar,
            hot_cold,
        };
        let encoded = receipt.encode()?;
        generation_catalog::mark_generation_ready(
            index_oid,
            logical_index_uuid,
            build_id,
            generation.next_batch_seq,
            generation.cumulative_record_count,
            generation.cumulative_owner_digest,
            &encoded,
        )?;
        Ok(encoded)
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}
