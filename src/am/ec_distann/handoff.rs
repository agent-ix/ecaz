//! Task 179 participant-side streamed handoff transactions.
//!
//! Invalid input is fully decoded and checked before the first physical write.
//! This is stricter than transaction rollback alone: an aborted heap insert can
//! leave relation-file growth even though no tuple becomes visible.

use std::ffi::CStr;

use pgrx::datum::Uuid;
use pgrx::iter::TableIterator;
use pgrx::{name, pg_extern, pg_sys, PgRelation, Spi};
use sha2::{Digest, Sha256};

use crate::am::common::heap_slot::TupleSlotWriter;
use crate::storage::page::ItemPointer;
use crate::storage::relation_guard::HeapRelationGuard;
use crate::storage::slot_guard::TupleTableSlotGuard;

use super::canonical_wire::is_rfc4122_v4_uuid;
use super::generation_catalog::{self, GenerationBatchCatalogRow, GenerationCatalogRow};
use super::generation_descriptor::{
    roster_digest, DistannGenerationDescriptor, DISTANN_PHYSICAL_INDEX_FORMAT_VERSION,
};
use super::generation_store::open_control_index;
use super::handoff_wire::{
    DistannHandoffBatch, DistannHandoffEntry, DistannHandoffShape, DistannOwnerStreamHasher,
};
use super::identity::vec_id_from_source_identity;
use super::manifest_v2::{
    DistannReadyReceipt, DISTANN_READY_RECEIPT_BYTES, DISTANN_READY_RECEIPT_STATE,
};
use super::placement::owning_node;
use super::quantizer::DistannCodecBinding;
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
    node: DistannNodeTuple,
}

const PERSISTED_GRAPH_DOMAIN: &[u8] = b"ec_distann_persisted_graph_v1\0";
const PERSISTED_ROW_TIER_DOMAIN: &[u8] = b"ec_distann_persisted_row_tier_v1\0";
const LOCAL_DIRECTORY_DOMAIN: &[u8] = b"ec_distann_local_directory_v1\0";

fn fixed_digest(bytes: Vec<u8>, field: &str) -> Result<[u8; 32], String> {
    bytes.try_into().map_err(|bytes: Vec<u8>| {
        format!(
            "EC_HANDOFF_DIGEST: {field} is {} bytes, expected 32",
            bytes.len()
        )
    })
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

fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
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

fn qualified_relation_name(relation_oid: pg_sys::Oid) -> Result<String, String> {
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

fn validate_generation_relations(
    row: &GenerationCatalogRow,
    descriptor: &DistannGenerationDescriptor,
) -> Result<(), String> {
    let mut expected_row_schema = descriptor.row_schema.clone();
    for attribute in &mut expected_row_schema.attributes {
        if !attribute.dropped {
            attribute.generated_kind = 0;
        }
    }
    let physical_row_schema = resolve_relation_schema(row.row_tier_relid)?;
    if physical_row_schema.descriptor != expected_row_schema {
        return Err(
            "EC_SCHEMA_MISMATCH: physical row tier differs from the frozen generation schema"
                .to_owned(),
        );
    }

    let valid = Spi::connect(|client| {
        client
            .select(
                "SELECT
                    EXISTS (
                        SELECT 1 FROM pg_catalog.pg_class
                         WHERE oid = $1::oid AND relkind = 'r'
                           AND relpersistence = 'p' AND NOT relrowsecurity
                           AND NOT relforcerowsecurity
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_attribute
                         WHERE attrelid = $1::oid AND attnum > 0 AND NOT attisdropped
                           AND (attnotnull OR atthasdef OR attidentity <> '' OR attgenerated <> '')
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_index WHERE indrelid = $1::oid
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid = $1::oid
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_trigger
                         WHERE tgrelid = $1::oid AND NOT tgisinternal
                    )
                    AND NOT EXISTS (
                        SELECT 1 FROM pg_catalog.pg_rewrite WHERE ev_class = $1::oid
                    )
                    AND EXISTS (
                        SELECT 1 FROM pg_catalog.pg_class
                         WHERE oid = $2::oid AND relkind = 'r'
                           AND relpersistence = 'p' AND NOT relrowsecurity
                           AND NOT relforcerowsecurity
                    )
                    AND (
                        SELECT count(*) = 3
                           AND bool_and(
                               attnotnull AND NOT atthasdef
                               AND attidentity = '' AND attgenerated = ''
                               AND CASE attnum
                                     WHEN 1 THEN attname = 'vec_id' AND atttypid = 'bigint'::regtype
                                     WHEN 2 THEN attname = 'graph_record' AND atttypid = 'bytea'::regtype
                                     WHEN 3 THEN attname = 'row_tid' AND atttypid = 'tid'::regtype
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
                                        AND indkey[0] = 1)
                          FROM pg_catalog.pg_index WHERE indrelid = $2::oid
                    )
                    AND EXISTS (
                        SELECT 1 FROM pg_catalog.pg_class c
                        JOIN pg_catalog.pg_am am ON am.oid = c.relam
                         WHERE c.oid = $3::oid AND c.relkind = 'i' AND am.amname = 'btree'
                    ) AS valid",
                None,
                &[
                    row.row_tier_relid.into(),
                    row.graph_store_relid.into(),
                    row.directory_relid.into(),
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

fn prepare_entries(
    entries: &[DistannHandoffEntry],
    shape: DistannHandoffShape,
    identity_attnum: u16,
    row_io: &mut [Option<RowAttributeIo>],
) -> Result<Vec<PreparedEntry>, String> {
    let mut prepared = Vec::with_capacity(entries.len());
    for entry in entries {
        let mut datums = Vec::with_capacity(row_io.len());
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
        prepared.push(PreparedEntry { datums, node });
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
    graph_relation: &str,
    entries: &mut [PreparedEntry],
    shape: DistannHandoffShape,
) -> Result<(), String> {
    let slot = TupleTableSlotGuard::create_for_heap_guard(row_relation).ok_or_else(|| {
        "EC_GENERATION_MISSING: could not allocate row-tier tuple slot".to_owned()
    })?;
    let graph_degree = u16::try_from(shape.graph_degree)
        .map_err(|_| "EC_HANDOFF_FORMAT: graph degree exceeds u16".to_owned())?;
    let graph_sql = format!(
        "INSERT INTO {graph_relation} (vec_id, graph_record, row_tid)
         VALUES ($1::bigint, $2::bytea, $3::tid)"
    );
    for entry in entries {
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
        let graph_record = entry
            .node
            .encode_physical_v1(graph_degree, shape.code_stride)?;
        let vec_id = i64::from_le_bytes(entry.node.vec_id.to_le_bytes());
        Spi::connect_mut(|client| {
            client
                .update(
                    &graph_sql,
                    None,
                    &[vec_id.into(), graph_record.into(), row_tid.into()],
                )
                .map_err(|error| format!("EC_BATCH_CONFLICT: graph insert failed: {error}"))?;
            Ok::<(), String>(())
        })?;
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

fn update_length_prefixed(hasher: &mut Sha256, bytes: &[u8]) -> Result<(), String> {
    let length = u32::try_from(bytes.len())
        .map_err(|_| "EC_BUILD_INCOMPLETE: persisted digest field exceeds u32".to_owned())?;
    hasher.update(length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

#[derive(Debug)]
struct PhysicalSealSummary {
    record_count: u64,
    owner_stream_digest: [u8; 32],
    graph_digest: [u8; 32],
    row_tier_digest: [u8; 32],
    directory_digest: [u8; 32],
}

#[allow(clippy::too_many_arguments)]
fn scan_physical_generation(
    generation: &GenerationCatalogRow,
    descriptor: &DistannGenerationDescriptor,
    shape: DistannHandoffShape,
    identity_attnum: u16,
    row_relation: &HeapRelationGuard,
    graph_relation: &str,
) -> Result<PhysicalSealSummary, String> {
    let slot = TupleTableSlotGuard::create_for_heap_guard(row_relation)
        .ok_or_else(|| "EC_GENERATION_MISSING: could not allocate seal row slot".to_owned())?;
    let mut row_io = unsafe { row_attribute_io(row_relation.as_ptr())? };
    let mut owner_hasher = DistannOwnerStreamHasher::new();
    let mut graph_hasher = Sha256::new();
    graph_hasher.update(PERSISTED_GRAPH_DOMAIN);
    let mut row_hasher = Sha256::new();
    row_hasher.update(PERSISTED_ROW_TIER_DOMAIN);
    let mut directory_hasher = Sha256::new();
    directory_hasher.update(LOCAL_DIRECTORY_DOMAIN);
    let mut record_count = 0_u64;
    let mut previous_vec_id = None;
    let sql = format!(
        "SELECT vec_id, graph_record, row_tid, ctid AS graph_tid
           FROM {graph_relation}
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
                let node = DistannNodeTuple::decode_physical_v1(
                    &graph_record,
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
                let (source_identity, row_null_bitmap, row_values) = fetch_frozen_row(
                    row_relation,
                    &slot,
                    node.heap_tid,
                    identity_attnum,
                    &mut row_io,
                )?;
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
        directory_digest: directory_hasher.finalize().into(),
    })
}

fn generation_sizes(generation: &GenerationCatalogRow) -> Result<(u64, u64, u64), String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT pg_catalog.pg_table_size($1::oid::regclass)::bigint AS graph_bytes,
                        pg_catalog.pg_table_size($2::oid::regclass)::bigint AS row_tier_bytes,
                        pg_catalog.pg_total_relation_size($3::oid::regclass)::bigint
                            AS directory_bytes",
                None,
                &[
                    generation.graph_store_relid.into(),
                    generation.row_tier_relid.into(),
                    generation.directory_relid.into(),
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
                Ok::<(u64, u64, u64), String>((
                    required("graph_bytes")?,
                    required("row_tier_bytes")?,
                    required("directory_bytes")?,
                ))
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
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let batch_seq = u64::try_from(batch_seq)
            .map_err(|_| "EC_BATCH_SEQUENCE: batch sequence is negative".to_owned())?;
        let supplied_digest = fixed_digest(batch_digest, "batch digest")?;
        let verified_digest = DistannHandoffBatch::verified_digest(&encoded_batch)?;
        if supplied_digest != verified_digest {
            return Err(
                "EC_HANDOFF_DIGEST: supplied batch digest differs from encoded batch".to_owned(),
            );
        }

        let index_oid = index_regclass.oid();
        let (_control_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_stage_epoch_batch",
        )?;
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
        if generation.state != "Building" {
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
        let binding = DistannCodecBinding::from_artifact(&descriptor.codec_artifact)?;
        let shape = DistannHandoffShape {
            code_stride: binding.code_len(usize::from(descriptor.dimensions))?,
            graph_degree: usize::from(descriptor.graph_degree),
            non_dropped_attribute_count: descriptor.row_schema.non_dropped_count(),
        };
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
        let _graph_relation_guard = HeapRelationGuard::try_open(
            generation.graph_store_relid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
        )
        .ok_or_else(|| "EC_GENERATION_MISSING: graph-store relation is absent".to_owned())?;
        validate_generation_relations(&generation, &descriptor)?;
        let graph_relation = qualified_relation_name(generation.graph_store_relid)?;
        reject_existing_vec_ids(&graph_relation, &batch.entries)?;
        let identity_attnum = identity_attnum(index_oid)?;
        let mut row_io = unsafe { row_attribute_io(row_relation.as_ptr())? };
        let mut prepared = prepare_entries(&batch.entries, shape, identity_attnum, &mut row_io)?;

        insert_prepared_entries(&row_relation, &graph_relation, &mut prepared, shape)?;
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
        if !is_rfc4122_v4_uuid(build_id.as_bytes()) {
            return Err("EC_BUILD_ID_CONFLICT: build id must be an RFC 4122 v4 UUID".to_owned());
        }
        let expected_owner_count = u64::try_from(expected_owner_count)
            .map_err(|_| "EC_BUILD_INCOMPLETE: expected owner count is negative".to_owned())?;
        let expected_owner_digest = fixed_digest(expected_owner_digest, "expected owner digest")?;
        let index_oid = index_regclass.oid();
        let (_control_guard, _handle, _metadata, logical_index_uuid) = open_control_index(
            index_oid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
            "ec_distann_seal_epoch_handoff",
        )?;
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
        if matches!(generation.state.as_str(), "Ready" | "Published" | "Retired") {
            return generation
                .ready_receipt
                .map(|receipt| receipt.to_vec())
                .ok_or_else(|| {
                    "EC_BUILD_STATE: non-Building generation has no Ready receipt".to_owned()
                });
        }
        if generation.state != "Building" {
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
        let binding = DistannCodecBinding::from_artifact(&descriptor.codec_artifact)?;
        let shape = DistannHandoffShape {
            code_stride: binding.code_len(usize::from(descriptor.dimensions))?,
            graph_degree: usize::from(descriptor.graph_degree),
            non_dropped_attribute_count: descriptor.row_schema.non_dropped_count(),
        };
        let row_relation = HeapRelationGuard::try_open(
            generation.row_tier_relid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
        )
        .ok_or_else(|| "EC_GENERATION_MISSING: row-tier relation is absent".to_owned())?;
        let _graph_relation_guard = HeapRelationGuard::try_open(
            generation.graph_store_relid,
            pg_sys::ShareRowExclusiveLock as pg_sys::LOCKMODE,
        )
        .ok_or_else(|| "EC_GENERATION_MISSING: graph-store relation is absent".to_owned())?;
        validate_generation_relations(&generation, &descriptor)?;
        let graph_relation = qualified_relation_name(generation.graph_store_relid)?;
        let row_relation_name = qualified_relation_name(generation.row_tier_relid)?;
        let identity_attnum = identity_attnum(index_oid)?;
        let physical = scan_physical_generation(
            &generation,
            &descriptor,
            shape,
            identity_attnum,
            &row_relation,
            &graph_relation,
        )?;
        let row_count = relation_row_count(&row_relation_name)?;
        if physical.record_count != expected_owner_count
            || row_count != expected_owner_count
            || physical.owner_stream_digest != expected_owner_digest
        {
            return Err(
                "EC_BUILD_INCOMPLETE: physical row/record count or owner digest mismatch"
                    .to_owned(),
            );
        }
        let (graph_bytes, row_tier_bytes, directory_bytes) = generation_sizes(&generation)?;
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
            graph_bytes,
            row_tier_bytes,
            directory_bytes,
            state: DISTANN_READY_RECEIPT_STATE,
        };
        let encoded = receipt.encode()?;
        let fixed: [u8; DISTANN_READY_RECEIPT_BYTES] =
            encoded.as_slice().try_into().map_err(|_| {
                "EC_READY_RECEIPT: encoded receipt has the wrong fixed length".to_owned()
            })?;
        generation_catalog::mark_generation_ready(
            index_oid,
            logical_index_uuid,
            build_id,
            generation.next_batch_seq,
            generation.cumulative_record_count,
            generation.cumulative_owner_digest,
            fixed,
        )?;
        Ok(encoded)
    })()
    .unwrap_or_else(|error| pgrx::error!("{error}"))
}
