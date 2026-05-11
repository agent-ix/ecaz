//! ADR-069 DML front-door shape classification.
//!
//! The planner hook maps PostgreSQL query trees into this small input model.
//! Query-tree helpers each extract one fact and let the pure classifier compose
//! the final supported/unsupported result.
//! Keeping the v1 safety rules here makes unsupported distributed DML shapes
//! fail closed before any hook can fall through to the coordinator heap path.
#![allow(dead_code)]

use pgrx::{pg_guard, pg_sys, PgList, Spi};

use std::ffi::CStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpireDmlFrontdoorOperation {
    Update,
    Delete,
    PkSelect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpireDmlFrontdoorValueKind {
    ConstBigint,
    ParamBigint,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SpireDmlFrontdoorShapeInput<'a> {
    pub(crate) operation: SpireDmlFrontdoorOperation,
    pub(crate) ec_spire_distributed_table: bool,
    pub(crate) single_table: bool,
    pub(crate) has_join: bool,
    pub(crate) has_subquery: bool,
    pub(crate) has_returning: bool,
    pub(crate) pk_column: &'a str,
    pub(crate) predicate_column: Option<&'a str>,
    pub(crate) predicate_operator: Option<&'a str>,
    pub(crate) predicate_value_kind: SpireDmlFrontdoorValueKind,
    pub(crate) updated_columns: &'a [&'a str],
    pub(crate) projected_columns: &'a [&'a str],
    pub(crate) embedding_columns: &'a [&'a str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorShapeRow {
    pub(crate) supported: bool,
    pub(crate) operation: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) status: &'static str,
    pub(crate) error: Option<&'static str>,
    pub(crate) hint: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorHookStatusRow {
    pub(crate) hook_name: &'static str,
    pub(crate) planner_hook_installed: bool,
    pub(crate) query_shape_classifier_enabled: bool,
    pub(crate) query_shape_classifier_invoked_by_hook: bool,
    pub(crate) plan_rewrite_enabled: bool,
    pub(crate) last_classification_supported: Option<bool>,
    pub(crate) last_classification_kind: Option<&'static str>,
    pub(crate) last_classification_status: Option<&'static str>,
    pub(crate) status: &'static str,
    pub(crate) next_step: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct SpireDmlFrontdoorRelationContext {
    pub(crate) heap_relation_oid: pg_sys::Oid,
    pub(crate) index_oid: pg_sys::Oid,
    pub(crate) ec_spire_distributed_table: bool,
    pub(crate) pk_column: Option<String>,
    pub(crate) pk_type: Option<String>,
    pub(crate) column_names: Vec<(pg_sys::AttrNumber, String)>,
    pub(crate) embedding_columns: Vec<String>,
    pub(crate) status: &'static str,
    pub(crate) next_step: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct SpireDmlFrontdoorQueryContext<'a> {
    pub(crate) ec_spire_distributed_table: bool,
    pub(crate) pk_column: &'a str,
    pub(crate) column_names: &'a [(pg_sys::AttrNumber, &'a str)],
    pub(crate) embedding_columns: &'a [&'a str],
}

static mut PREVIOUS_PLANNER_HOOK: pg_sys::planner_hook_type = None;
static mut PLANNER_HOOK_INSTALLED: bool = false;
static mut HOOK_CLASSIFICATION_ATTEMPTED: bool = false;
static mut LAST_HOOK_CLASSIFICATION_SUPPORTED: Option<bool> = None;
static mut LAST_HOOK_CLASSIFICATION_KIND: Option<&'static str> = None;
static mut LAST_HOOK_CLASSIFICATION_STATUS: Option<&'static str> = None;

const EC_SPIRE_AM_NAME: &core::ffi::CStr = c"ec_spire";
const ADR_069_HINT: &str = "See ADR-069 for the v1 SPIRE distributed DML shape.";
const DML_FRONTDOOR_MAX_COERCION_WRAPPER_DEPTH: usize = 32;

pub(crate) unsafe fn register_dml_frontdoor_planner_hook() {
    unsafe {
        if !PLANNER_HOOK_INSTALLED {
            PREVIOUS_PLANNER_HOOK = pg_sys::planner_hook;
            pg_sys::planner_hook = Some(ec_spire_dml_frontdoor_planner_hook);
            PLANNER_HOOK_INSTALLED = true;
        }
    }
}

pub(crate) fn dml_frontdoor_hook_status_row() -> SpireDmlFrontdoorHookStatusRow {
    let (installed, classifier_invoked, last_supported, last_kind, last_status) = unsafe {
        (
            PLANNER_HOOK_INSTALLED,
            HOOK_CLASSIFICATION_ATTEMPTED,
            LAST_HOOK_CLASSIFICATION_SUPPORTED,
            LAST_HOOK_CLASSIFICATION_KIND,
            LAST_HOOK_CLASSIFICATION_STATUS,
        )
    };
    SpireDmlFrontdoorHookStatusRow {
        hook_name: "ec_spire_dml_frontdoor_planner_hook",
        planner_hook_installed: installed,
        query_shape_classifier_enabled: true,
        query_shape_classifier_invoked_by_hook: classifier_invoked,
        plan_rewrite_enabled: false,
        last_classification_supported: last_supported,
        last_classification_kind: last_kind,
        last_classification_status: last_status,
        status: if installed && classifier_invoked {
            "pass_through_classifier_observed"
        } else if installed {
            "pass_through_query_classifier_ready"
        } else {
            "not_installed"
        },
        next_step: "replace supported DML front-door plans with CustomScan executor nodes",
    }
}

// The DML front door intentionally keeps two relation-context loaders with the
// same contract. The SPI-backed loader is an operator diagnostic path, while
// the catalog/relcache-backed loader is safe for planner-hook observation and
// later plan replacement because it does not enter SPI recursively.
pub(crate) fn dml_frontdoor_relation_context_row(
    heap_relation_oid: pg_sys::Oid,
) -> Result<SpireDmlFrontdoorRelationContext, String> {
    let index_oid = dml_frontdoor_ec_spire_index_oid(heap_relation_oid)?;
    let column_names = dml_frontdoor_relation_column_names(heap_relation_oid)?;
    let pk = dml_frontdoor_primary_key_column(heap_relation_oid)?;
    let embedding_columns = if index_oid == pg_sys::InvalidOid {
        Vec::new()
    } else {
        dml_frontdoor_index_key_column_names(index_oid)?
    };

    let (status, next_step, ec_spire_distributed_table) = if index_oid == pg_sys::InvalidOid {
        (
            "no_ec_spire_index",
            "create ec_spire index before DML front-door routing",
            false,
        )
    } else if pk.is_none() {
        (
            "unsupported_pk_shape",
            "define one bigint primary-key column for ADR-069 v1 routing",
            false,
        )
    } else {
        (
            "relation_context_ready",
            "wire planner hook query classification to CustomScan executor replacement",
            true,
        )
    };
    let (pk_column, pk_type) = pk
        .map(|pk| (Some(pk.column_name), Some(pk.column_type)))
        .unwrap_or((None, None));

    Ok(SpireDmlFrontdoorRelationContext {
        heap_relation_oid,
        index_oid,
        ec_spire_distributed_table,
        pk_column,
        pk_type,
        column_names,
        embedding_columns,
        status,
        next_step,
    })
}

pub(crate) unsafe fn dml_frontdoor_relation_context_catalog_row(
    heap_relation_oid: pg_sys::Oid,
) -> Result<SpireDmlFrontdoorRelationContext, String> {
    if heap_relation_oid == pg_sys::InvalidOid {
        return Err(
            "ec_spire DML frontdoor catalog relation context requires a valid heap relation OID"
                .to_owned(),
        );
    }

    let heap_relation = unsafe {
        pg_sys::table_open(
            heap_relation_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        )
    };
    if heap_relation.is_null() {
        return Err("ec_spire DML frontdoor catalog relation open returned NULL".to_owned());
    }

    let result = unsafe { dml_frontdoor_relation_context_catalog_for_open_heap(heap_relation) };
    unsafe { pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_dml_frontdoor_planner_hook(
    parse: *mut pg_sys::Query,
    query_string: *const core::ffi::c_char,
    cursor_options: core::ffi::c_int,
    bound_params: pg_sys::ParamListInfo,
) -> *mut pg_sys::PlannedStmt {
    unsafe { dml_frontdoor_observe_planner_query(parse) };
    if let Some(previous_hook) = unsafe { PREVIOUS_PLANNER_HOOK } {
        unsafe { previous_hook(parse, query_string, cursor_options, bound_params) }
    } else {
        unsafe { pg_sys::standard_planner(parse, query_string, cursor_options, bound_params) }
    }
}

unsafe fn dml_frontdoor_observe_planner_query(query: *mut pg_sys::Query) {
    let Some(shape) = (unsafe { dml_frontdoor_classify_query_with_catalog_context(query) }) else {
        return;
    };
    unsafe {
        HOOK_CLASSIFICATION_ATTEMPTED = true;
        LAST_HOOK_CLASSIFICATION_SUPPORTED = Some(shape.supported);
        LAST_HOOK_CLASSIFICATION_KIND = Some(shape.kind);
        LAST_HOOK_CLASSIFICATION_STATUS = Some(shape.status);
    }
}

unsafe fn dml_frontdoor_classify_query_with_catalog_context(
    query: *mut pg_sys::Query,
) -> Option<SpireDmlFrontdoorShapeRow> {
    let target_relation_oid = unsafe { dml_frontdoor_target_relation_oid(query)? };
    let relation = match unsafe { dml_frontdoor_relation_context_catalog_row(target_relation_oid) }
    {
        Ok(relation) => relation,
        Err(_err) => {
            return Some(SpireDmlFrontdoorShapeRow {
                supported: false,
                operation: "unsupported",
                kind: "relation_context_error",
                status: "unsupported_shape",
                error: Some("ec_spire_distributed: relation context could not be loaded"),
                hint: Some(ADR_069_HINT),
            });
        }
    };
    let pk_column = relation.pk_column.as_deref().unwrap_or("");
    let column_names = relation
        .column_names
        .iter()
        .map(|(attnum, name)| (*attnum, name.as_str()))
        .collect::<Vec<_>>();
    let embedding_columns = relation
        .embedding_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let query_context = SpireDmlFrontdoorQueryContext {
        ec_spire_distributed_table: relation.ec_spire_distributed_table,
        pk_column,
        column_names: &column_names,
        embedding_columns: &embedding_columns,
    };
    unsafe { classify_dml_frontdoor_query(query, query_context) }
}

struct SpireDmlFrontdoorPrimaryKeyColumn {
    column_name: String,
    column_type: String,
}

unsafe fn dml_frontdoor_relation_context_catalog_for_open_heap(
    heap_relation: pg_sys::Relation,
) -> Result<SpireDmlFrontdoorRelationContext, String> {
    let heap_relation_oid = unsafe { (*heap_relation).rd_id };
    let column_names = unsafe { dml_frontdoor_relation_column_names_from_rel(heap_relation)? };
    let (index_oid, pk) = unsafe { dml_frontdoor_catalog_index_and_pk(heap_relation)? };
    let embedding_columns = if index_oid == pg_sys::InvalidOid {
        Vec::new()
    } else {
        unsafe { dml_frontdoor_index_key_column_names_from_rel(index_oid, heap_relation)? }
    };

    let (status, next_step, ec_spire_distributed_table) = if index_oid == pg_sys::InvalidOid {
        (
            "no_ec_spire_index",
            "create ec_spire index before DML front-door routing",
            false,
        )
    } else if pk.is_none() {
        (
            "unsupported_pk_shape",
            "define one bigint primary-key column for ADR-069 v1 routing",
            false,
        )
    } else {
        (
            "relation_context_ready",
            "wire planner hook query classification to CustomScan executor replacement",
            true,
        )
    };
    let (pk_column, pk_type) = pk
        .map(|pk| (Some(pk.column_name), Some(pk.column_type)))
        .unwrap_or((None, None));

    Ok(SpireDmlFrontdoorRelationContext {
        heap_relation_oid,
        index_oid,
        ec_spire_distributed_table,
        pk_column,
        pk_type,
        column_names,
        embedding_columns,
        status,
        next_step,
    })
}

unsafe fn dml_frontdoor_catalog_index_and_pk(
    heap_relation: pg_sys::Relation,
) -> Result<(pg_sys::Oid, Option<SpireDmlFrontdoorPrimaryKeyColumn>), String> {
    let ec_spire_am_oid = unsafe { pg_sys::get_index_am_oid(EC_SPIRE_AM_NAME.as_ptr(), true) };
    // RelationGetIndexList returns a private OID list, so each index can be
    // opened and closed under AccessShareLock while walking this copy.
    let index_list =
        unsafe { PgList::<pg_sys::Oid>::from_pg(pg_sys::RelationGetIndexList(heap_relation)) };
    let mut ec_spire_index_count = 0_i64;
    let mut ec_spire_index_oid = pg_sys::InvalidOid;
    let mut primary_key = None;

    for index_oid in index_list.iter_oid() {
        let index_relation =
            unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        if index_relation.is_null() {
            continue;
        }
        let index_form = unsafe { (*index_relation).rd_index.as_ref() };
        let class_form = unsafe { (*index_relation).rd_rel.as_ref() };
        if let Some(class_form) = class_form {
            if ec_spire_am_oid != pg_sys::InvalidOid && class_form.relam == ec_spire_am_oid {
                ec_spire_index_count += 1;
                if ec_spire_index_oid == pg_sys::InvalidOid
                    || index_oid.to_u32() < ec_spire_index_oid.to_u32()
                {
                    ec_spire_index_oid = index_oid;
                }
            }
        }
        if primary_key.is_none() {
            if let Some(index_form) = index_form {
                primary_key = unsafe {
                    dml_frontdoor_primary_key_column_from_index(heap_relation, index_form)?
                };
            }
        }
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }

    if ec_spire_index_count > 1 {
        return Err(
            "ec_spire DML frontdoor v1 requires at most one ec_spire index per heap relation"
                .to_owned(),
        );
    }
    Ok((ec_spire_index_oid, primary_key))
}

unsafe fn dml_frontdoor_primary_key_column_from_index(
    heap_relation: pg_sys::Relation,
    index_form: &pg_sys::FormData_pg_index,
) -> Result<Option<SpireDmlFrontdoorPrimaryKeyColumn>, String> {
    if !index_form.indisprimary || index_form.indnkeyatts != 1 {
        return Ok(None);
    }
    let attnum = unsafe { *index_form.indkey.values.as_ptr() };
    if attnum <= 0 {
        return Ok(None);
    }
    let Some((column_name, attr)) =
        (unsafe { dml_frontdoor_relation_attr_name_and_form(heap_relation, attnum)? })
    else {
        return Ok(None);
    };
    if attr.atttypid != pg_sys::INT8OID {
        return Ok(None);
    }
    Ok(Some(SpireDmlFrontdoorPrimaryKeyColumn {
        column_name,
        column_type: unsafe { dml_frontdoor_format_type_name(attr.atttypid)? },
    }))
}

unsafe fn dml_frontdoor_relation_column_names_from_rel(
    heap_relation: pg_sys::Relation,
) -> Result<Vec<(pg_sys::AttrNumber, String)>, String> {
    let tuple_desc = unsafe { (*heap_relation).rd_att };
    if tuple_desc.is_null() {
        return Err("ec_spire DML frontdoor catalog relation tuple descriptor is NULL".to_owned());
    }
    let natts = unsafe { (*tuple_desc).natts };
    let mut columns = Vec::with_capacity(usize::try_from(natts).unwrap_or(0));
    for attr_index in 0..natts {
        let attr = unsafe { pg_sys::TupleDescAttr(tuple_desc, attr_index) };
        if attr.is_null() || unsafe { (*attr).attisdropped } {
            continue;
        }
        let attnum = unsafe { (*attr).attnum };
        if attnum <= 0 {
            continue;
        }
        let name = unsafe { dml_frontdoor_attr_name(attr)? };
        columns.push((attnum, name));
    }
    Ok(columns)
}

unsafe fn dml_frontdoor_index_key_column_names_from_rel(
    index_oid: pg_sys::Oid,
    heap_relation: pg_sys::Relation,
) -> Result<Vec<String>, String> {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    if index_relation.is_null() {
        return Err("ec_spire DML frontdoor catalog index open returned NULL".to_owned());
    }
    let result = unsafe {
        let index_form = (*index_relation)
            .rd_index
            .as_ref()
            .ok_or_else(|| "ec_spire DML frontdoor catalog index metadata is NULL".to_owned())?;
        let mut columns = Vec::new();
        for key_index in 0..index_form.indnkeyatts {
            let attnum = *index_form
                .indkey
                .values
                .as_ptr()
                .add(usize::try_from(key_index).unwrap_or(usize::MAX));
            if attnum <= 0 {
                continue;
            }
            if let Some((column_name, _attr)) =
                dml_frontdoor_relation_attr_name_and_form(heap_relation, attnum)?
            {
                columns.push(column_name);
            }
        }
        Ok(columns)
    };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

unsafe fn dml_frontdoor_relation_attr_name_and_form(
    heap_relation: pg_sys::Relation,
    attnum: pg_sys::AttrNumber,
) -> Result<Option<(String, pg_sys::FormData_pg_attribute)>, String> {
    let tuple_desc = unsafe { (*heap_relation).rd_att };
    if tuple_desc.is_null() || attnum <= 0 || i32::from(attnum) > unsafe { (*tuple_desc).natts } {
        return Ok(None);
    }
    let attr = unsafe { pg_sys::TupleDescAttr(tuple_desc, i32::from(attnum - 1)) };
    if attr.is_null() || unsafe { (*attr).attisdropped } {
        return Ok(None);
    }
    Ok(Some((unsafe { dml_frontdoor_attr_name(attr)? }, unsafe {
        *attr
    })))
}

unsafe fn dml_frontdoor_attr_name(
    attr: *mut pg_sys::FormData_pg_attribute,
) -> Result<String, String> {
    unsafe { CStr::from_ptr((*attr).attname.data.as_ptr()) }
        .to_str()
        .map(str::to_owned)
        .map_err(|e| format!("ec_spire DML frontdoor catalog attribute name is not UTF-8: {e}"))
}

unsafe fn dml_frontdoor_format_type_name(type_oid: pg_sys::Oid) -> Result<String, String> {
    let type_name = unsafe { pg_sys::format_type_be(type_oid) };
    if type_name.is_null() {
        return Err("ec_spire DML frontdoor catalog format_type returned NULL".to_owned());
    }
    let decoded = unsafe { CStr::from_ptr(type_name) }
        .to_str()
        .map(str::to_owned)
        .map_err(|e| format!("ec_spire DML frontdoor catalog type name is not UTF-8: {e}"));
    unsafe { pg_sys::pfree(type_name.cast()) };
    decoded
}

fn dml_frontdoor_ec_spire_index_oid(heap_relation_oid: pg_sys::Oid) -> Result<pg_sys::Oid, String> {
    Spi::connect(|client| {
        let (index_count, index_oid) = client
            .select(
                "SELECT count(*)::bigint AS index_count, \
                        coalesce(min(idx.indexrelid::oid), 0::oid) AS index_oid \
                   FROM pg_index AS idx \
                   JOIN pg_class AS index_class \
                     ON index_class.oid = idx.indexrelid \
                   JOIN pg_am AS access_method \
                     ON access_method.oid = index_class.relam \
                    AND access_method.amname = 'ec_spire' \
                  WHERE idx.indrelid = $1::oid",
                None,
                &[heap_relation_oid.into()],
            )
            .map_err(|e| format!("ec_spire DML frontdoor index lookup failed: {e}"))?
            .map(|row| {
                let index_count = row["index_count"]
                    .value::<i64>()
                    .map_err(|e| format!("ec_spire DML frontdoor index count decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire DML frontdoor index count is null".to_owned())?;
                let index_oid = row["index_oid"]
                    .value::<pg_sys::Oid>()
                    .map_err(|e| format!("ec_spire DML frontdoor index oid decode failed: {e}"))?
                    .ok_or_else(|| "ec_spire DML frontdoor index oid is null".to_owned())?;
                Ok::<(i64, pg_sys::Oid), String>((index_count, index_oid))
            })
            .next()
            .transpose()?
            .ok_or_else(|| "ec_spire DML frontdoor index lookup returned no row".to_owned())?;
        if index_count > 1 {
            return Err(
                "ec_spire DML frontdoor v1 requires at most one ec_spire index per heap relation"
                    .to_owned(),
            );
        }
        Ok(index_oid)
    })
}

fn dml_frontdoor_relation_column_names(
    heap_relation_oid: pg_sys::Oid,
) -> Result<Vec<(pg_sys::AttrNumber, String)>, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT attr.attnum::smallint AS attnum, attr.attname::text AS attname \
                   FROM pg_attribute AS attr \
                  WHERE attr.attrelid = $1::oid \
                    AND attr.attnum > 0 \
                    AND NOT attr.attisdropped \
                  ORDER BY attr.attnum",
                None,
                &[heap_relation_oid.into()],
            )
            .map_err(|e| format!("ec_spire DML frontdoor column lookup failed: {e}"))?
            .map(|row| {
                let attnum = row["attnum"]
                    .value::<i16>()
                    .map_err(|e| {
                        format!("ec_spire DML frontdoor column attnum decode failed: {e}")
                    })?
                    .ok_or_else(|| "ec_spire DML frontdoor column attnum is null".to_owned())?;
                let attname = row["attname"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire DML frontdoor column attname decode failed: {e}")
                    })?
                    .ok_or_else(|| "ec_spire DML frontdoor column attname is null".to_owned())?;
                Ok::<(pg_sys::AttrNumber, String), String>((attnum, attname))
            })
            .collect::<Result<Vec<_>, _>>()
    })
}

fn dml_frontdoor_primary_key_column(
    heap_relation_oid: pg_sys::Oid,
) -> Result<Option<SpireDmlFrontdoorPrimaryKeyColumn>, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT attr.attname::text AS column_name, \
                        format_type(attr.atttypid, attr.atttypmod)::text AS column_type \
                   FROM pg_index AS idx \
                   JOIN unnest(idx.indkey) WITH ORDINALITY AS key_column(attnum, ord) \
                     ON key_column.attnum > 0 \
                    AND key_column.ord <= idx.indnkeyatts \
                   JOIN pg_attribute AS attr \
                     ON attr.attrelid = idx.indrelid \
                    AND attr.attnum = key_column.attnum \
                  WHERE idx.indrelid = $1::oid \
                    AND idx.indisprimary \
                    AND idx.indnkeyatts = 1 \
                    AND attr.atttypid = 'int8'::regtype::oid",
                None,
                &[heap_relation_oid.into()],
            )
            .map_err(|e| format!("ec_spire DML frontdoor primary-key lookup failed: {e}"))?
            .map(|row| {
                let column_name = row["column_name"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire DML frontdoor primary-key name decode failed: {e}")
                    })?
                    .ok_or_else(|| "ec_spire DML frontdoor primary-key name is null".to_owned())?;
                let column_type = row["column_type"]
                    .value::<String>()
                    .map_err(|e| {
                        format!("ec_spire DML frontdoor primary-key type decode failed: {e}")
                    })?
                    .ok_or_else(|| "ec_spire DML frontdoor primary-key type is null".to_owned())?;
                Ok::<SpireDmlFrontdoorPrimaryKeyColumn, String>(SpireDmlFrontdoorPrimaryKeyColumn {
                    column_name,
                    column_type,
                })
            })
            .next()
            .transpose()
    })
}

fn dml_frontdoor_index_key_column_names(index_oid: pg_sys::Oid) -> Result<Vec<String>, String> {
    Spi::connect(|client| {
        client
            .select(
                "SELECT coalesce(array_agg(attr.attname::text ORDER BY key_column.ord), ARRAY[]::text[]) \
                   AS key_columns \
                   FROM pg_index AS idx \
                   JOIN unnest(idx.indkey) WITH ORDINALITY AS key_column(attnum, ord) \
                     ON key_column.attnum > 0 \
                    AND key_column.ord <= idx.indnkeyatts \
                   JOIN pg_attribute AS attr \
                     ON attr.attrelid = idx.indrelid \
                    AND attr.attnum = key_column.attnum \
                  WHERE idx.indexrelid = $1::oid",
                None,
                &[index_oid.into()],
            )
            .map_err(|e| format!("ec_spire DML frontdoor index key column lookup failed: {e}"))?
            .first()
            .get_one::<Vec<String>>()
            .map_err(|e| format!("ec_spire DML frontdoor index key column decode failed: {e}"))?
            .ok_or_else(|| "ec_spire DML frontdoor index key column list is null".to_owned())
    })
}

pub(crate) fn classify_dml_frontdoor_shape(
    input: SpireDmlFrontdoorShapeInput<'_>,
) -> SpireDmlFrontdoorShapeRow {
    let operation = operation_name(input.operation);
    if !input.ec_spire_distributed_table {
        return unsupported(
            operation,
            "not_distributed_table",
            "not an ec_spire distributed coordinator table",
            None,
        );
    }
    if !input.single_table || input.has_join {
        return unsupported_v1(
            operation,
            "unsupported_join_shape",
            "ec_spire_distributed: joined DML is not yet supported in v1",
        );
    }
    if input.has_subquery {
        return unsupported_v1(
            operation,
            "unsupported_subquery_shape",
            "ec_spire_distributed: subquery DML is not yet supported in v1",
        );
    }
    if input.has_returning {
        return unsupported_v1(
            operation,
            "unsupported_returning_shape",
            "ec_spire_distributed: RETURNING is not yet supported in v1",
        );
    }
    if input.pk_column.is_empty()
        || input.predicate_column != Some(input.pk_column)
        || input.predicate_operator != Some("=")
        || !matches!(
            input.predicate_value_kind,
            SpireDmlFrontdoorValueKind::ConstBigint | SpireDmlFrontdoorValueKind::ParamBigint
        )
    {
        return unsupported_v1(
            operation,
            "unsupported_pk_predicate",
            "ec_spire_distributed: DML requires a bigint primary-key equality predicate in v1",
        );
    }

    match input.operation {
        SpireDmlFrontdoorOperation::Update => classify_update(input),
        SpireDmlFrontdoorOperation::Delete => supported(operation, "delete_by_pk"),
        SpireDmlFrontdoorOperation::PkSelect => classify_pk_select(input),
    }
}

fn classify_update(input: SpireDmlFrontdoorShapeInput<'_>) -> SpireDmlFrontdoorShapeRow {
    let operation = operation_name(input.operation);
    if input.updated_columns.is_empty() {
        return unsupported_v1(
            operation,
            "unsupported_empty_update",
            "ec_spire_distributed: UPDATE requires at least one target column in v1",
        );
    }
    if input
        .updated_columns
        .iter()
        .any(|column| *column == input.pk_column)
    {
        return unsupported_v1(
            operation,
            "unsupported_pk_update",
            "ec_spire_distributed: UPDATE of the primary-key column is not supported in v1",
        );
    }
    if input.updated_columns.iter().any(|column| {
        input
            .embedding_columns
            .iter()
            .any(|embedding_column| embedding_column == column)
    }) {
        return unsupported(
            operation,
            "embedding_update_rejected",
            "ec_spire_distributed: UPDATE of indexed embedding column is not supported on a distributed ec_spire table. Use DELETE + INSERT.",
            Some("Cross-shard atomic moves will be available in a future release."),
        );
    }
    supported(operation, "update_non_embedding_by_pk")
}

fn classify_pk_select(input: SpireDmlFrontdoorShapeInput<'_>) -> SpireDmlFrontdoorShapeRow {
    let operation = operation_name(input.operation);
    if input.projected_columns.is_empty() {
        return unsupported_v1(
            operation,
            "unsupported_empty_projection",
            "ec_spire_distributed: PK SELECT requires at least one projected column in v1",
        );
    }
    supported(operation, "pk_select_by_pk")
}

pub(crate) unsafe fn classify_dml_frontdoor_query(
    query: *mut pg_sys::Query,
    context: SpireDmlFrontdoorQueryContext<'_>,
) -> Option<SpireDmlFrontdoorShapeRow> {
    if query.is_null() {
        return None;
    }
    let query_ref = unsafe { query.as_ref()? };
    let operation = dml_frontdoor_operation_for_query(query_ref)?;
    let range_table_ref = unsafe { single_range_table_ref(query_ref) };
    let target_rtindex = match operation {
        SpireDmlFrontdoorOperation::Update | SpireDmlFrontdoorOperation::Delete => {
            query_ref.resultRelation
        }
        SpireDmlFrontdoorOperation::PkSelect => {
            // Keep unsupported SELECT shapes flowing into the shared classifier
            // so diagnostics report the same fail-closed status/kind matrix.
            range_table_ref.unwrap_or_default()
        }
    };
    let (predicate_column, predicate_operator, predicate_value_kind) =
        unsafe { dml_frontdoor_pk_predicate(query_ref, target_rtindex, &context) };
    let updated_columns = if operation == SpireDmlFrontdoorOperation::Update {
        unsafe { dml_frontdoor_target_columns(query_ref.targetList, &context) }
    } else {
        Vec::new()
    };
    let projected_columns = if operation == SpireDmlFrontdoorOperation::PkSelect {
        unsafe { dml_frontdoor_target_columns(query_ref.targetList, &context) }
    } else {
        Vec::new()
    };
    let updated_column_refs = updated_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let projected_column_refs = projected_columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    Some(classify_dml_frontdoor_shape(SpireDmlFrontdoorShapeInput {
        operation,
        ec_spire_distributed_table: context.ec_spire_distributed_table,
        single_table: range_table_ref.is_some(),
        has_join: range_table_ref.is_none(),
        has_subquery: dml_frontdoor_query_has_subquery_shape(query_ref),
        has_returning: !query_ref.returningList.is_null(),
        pk_column: context.pk_column,
        predicate_column: predicate_column.as_deref(),
        predicate_operator,
        predicate_value_kind,
        updated_columns: &updated_column_refs,
        projected_columns: &projected_column_refs,
        embedding_columns: context.embedding_columns,
    }))
}

pub(crate) unsafe fn dml_frontdoor_target_relation_oid(
    query: *mut pg_sys::Query,
) -> Option<pg_sys::Oid> {
    if query.is_null() {
        return None;
    }
    let query_ref = unsafe { query.as_ref()? };
    let operation = dml_frontdoor_operation_for_query(query_ref)?;
    let target_rtindex = match operation {
        SpireDmlFrontdoorOperation::Update | SpireDmlFrontdoorOperation::Delete => {
            query_ref.resultRelation
        }
        SpireDmlFrontdoorOperation::PkSelect => unsafe { single_range_table_ref(query_ref)? },
    };
    unsafe { dml_frontdoor_relation_oid_from_rtable(query_ref, target_rtindex) }
}

fn dml_frontdoor_operation_for_query(query: &pg_sys::Query) -> Option<SpireDmlFrontdoorOperation> {
    match query.commandType {
        pg_sys::CmdType::CMD_UPDATE => Some(SpireDmlFrontdoorOperation::Update),
        pg_sys::CmdType::CMD_DELETE => Some(SpireDmlFrontdoorOperation::Delete),
        pg_sys::CmdType::CMD_SELECT => Some(SpireDmlFrontdoorOperation::PkSelect),
        _ => None,
    }
}

fn dml_frontdoor_query_has_subquery_shape(query: &pg_sys::Query) -> bool {
    query.hasSubLinks
        || query.hasModifyingCTE
        || query.hasRecursive
        || !query.cteList.is_null()
        || !query.setOperations.is_null()
}

unsafe fn single_range_table_ref(query: &pg_sys::Query) -> Option<i32> {
    let jointree = unsafe { query.jointree.as_ref()? };
    if jointree.fromlist.is_null() {
        return None;
    }
    let fromlist = unsafe { PgList::<pg_sys::Node>::from_pg(jointree.fromlist) };
    if fromlist.len() != 1 {
        return None;
    }
    let from_node = fromlist.get_ptr(0)?;
    if from_node.is_null() || unsafe { (*from_node).type_ } != pg_sys::NodeTag::T_RangeTblRef {
        return None;
    }
    let range_table_ref = from_node.cast::<pg_sys::RangeTblRef>();
    Some(unsafe { (*range_table_ref).rtindex })
}

unsafe fn dml_frontdoor_relation_oid_from_rtable(
    query: &pg_sys::Query,
    rtindex: i32,
) -> Option<pg_sys::Oid> {
    if rtindex <= 0 || query.rtable.is_null() {
        return None;
    }
    let rtable = unsafe { PgList::<pg_sys::RangeTblEntry>::from_pg(query.rtable) };
    let rte = rtable.get_ptr(usize::try_from(rtindex - 1).ok()?)?;
    let rte = unsafe { rte.as_ref()? };
    if rte.rtekind != pg_sys::RTEKind::RTE_RELATION || rte.relid == pg_sys::InvalidOid {
        return None;
    }
    Some(rte.relid)
}

unsafe fn dml_frontdoor_pk_predicate(
    query: &pg_sys::Query,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> (
    Option<String>,
    Option<&'static str>,
    SpireDmlFrontdoorValueKind,
) {
    let Some(jointree) = (unsafe { query.jointree.as_ref() }) else {
        return (None, None, SpireDmlFrontdoorValueKind::Other);
    };
    let qual = jointree.quals;
    if qual.is_null() || unsafe { (*qual).type_ } != pg_sys::NodeTag::T_OpExpr {
        return (None, None, SpireDmlFrontdoorValueKind::Other);
    }
    let op_expr = qual.cast::<pg_sys::OpExpr>();
    let operator =
        if dml_frontdoor_bigint_equality_opcode(unsafe { pg_sys::get_opcode((*op_expr).opno) }) {
            Some("=")
        } else {
            Some("other")
        };
    let args = unsafe { PgList::<pg_sys::Expr>::from_pg((*op_expr).args) };
    if args.len() != 2 {
        return (None, operator, SpireDmlFrontdoorValueKind::Other);
    }
    let left = args.get_ptr(0);
    let right = args.get_ptr(1);
    match (left, right) {
        (Some(left), Some(right)) => {
            if let Some(column) = unsafe { dml_frontdoor_var_column(left, target_rtindex, context) }
            {
                return (Some(column), operator, unsafe {
                    dml_frontdoor_value_kind(right)
                });
            }
            if let Some(column) =
                unsafe { dml_frontdoor_var_column(right, target_rtindex, context) }
            {
                return (Some(column), operator, unsafe {
                    dml_frontdoor_value_kind(left)
                });
            }
            (None, operator, SpireDmlFrontdoorValueKind::Other)
        }
        _ => (None, operator, SpireDmlFrontdoorValueKind::Other),
    }
}

unsafe fn dml_frontdoor_var_column(
    expr: *mut pg_sys::Expr,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> Option<String> {
    if expr.is_null() || unsafe { (*expr.cast::<pg_sys::Node>()).type_ } != pg_sys::NodeTag::T_Var {
        return None;
    }
    let var = unsafe { &*expr.cast::<pg_sys::Var>() };
    if var.varno != target_rtindex || var.varlevelsup != 0 || var.varattno <= 0 {
        return None;
    }
    context
        .column_names
        .iter()
        .find_map(|(attno, name)| (*attno == var.varattno).then(|| (*name).to_owned()))
}

unsafe fn dml_frontdoor_value_kind(expr: *mut pg_sys::Expr) -> SpireDmlFrontdoorValueKind {
    unsafe { dml_frontdoor_value_kind_inner(expr, 0) }
}

unsafe fn dml_frontdoor_value_kind_inner(
    expr: *mut pg_sys::Expr,
    wrapper_depth: usize,
) -> SpireDmlFrontdoorValueKind {
    if expr.is_null() {
        return SpireDmlFrontdoorValueKind::Other;
    }
    if wrapper_depth > DML_FRONTDOOR_MAX_COERCION_WRAPPER_DEPTH {
        return SpireDmlFrontdoorValueKind::Other;
    }
    match unsafe { (*expr.cast::<pg_sys::Node>()).type_ } {
        pg_sys::NodeTag::T_Const => {
            let const_expr = unsafe { &*expr.cast::<pg_sys::Const>() };
            if !const_expr.constisnull
                && dml_frontdoor_integer_oid_can_coerce_to_bigint(const_expr.consttype)
            {
                SpireDmlFrontdoorValueKind::ConstBigint
            } else {
                SpireDmlFrontdoorValueKind::Other
            }
        }
        pg_sys::NodeTag::T_Param => {
            let param = unsafe { &*expr.cast::<pg_sys::Param>() };
            if dml_frontdoor_integer_oid_can_coerce_to_bigint(param.paramtype) {
                SpireDmlFrontdoorValueKind::ParamBigint
            } else {
                SpireDmlFrontdoorValueKind::Other
            }
        }
        pg_sys::NodeTag::T_FuncExpr => {
            let func_expr = unsafe { &*expr.cast::<pg_sys::FuncExpr>() };
            if func_expr.funcresulttype != pg_sys::INT8OID {
                return SpireDmlFrontdoorValueKind::Other;
            }
            unsafe { dml_frontdoor_single_coerced_arg_value_kind(func_expr.args, wrapper_depth) }
        }
        pg_sys::NodeTag::T_RelabelType => {
            let relabel = unsafe { &*expr.cast::<pg_sys::RelabelType>() };
            if relabel.resulttype != pg_sys::INT8OID {
                return SpireDmlFrontdoorValueKind::Other;
            }
            unsafe { dml_frontdoor_coercible_integer_value_kind(relabel.arg, wrapper_depth) }
        }
        pg_sys::NodeTag::T_CoerceViaIO => {
            let coerce = unsafe { &*expr.cast::<pg_sys::CoerceViaIO>() };
            if coerce.resulttype != pg_sys::INT8OID {
                return SpireDmlFrontdoorValueKind::Other;
            }
            unsafe { dml_frontdoor_coercible_integer_value_kind(coerce.arg, wrapper_depth) }
        }
        _ => SpireDmlFrontdoorValueKind::Other,
    }
}

unsafe fn dml_frontdoor_single_coerced_arg_value_kind(
    args: *mut pg_sys::List,
    wrapper_depth: usize,
) -> SpireDmlFrontdoorValueKind {
    let Some(arg) = (unsafe { dml_frontdoor_single_list_expr_arg(args) }) else {
        return SpireDmlFrontdoorValueKind::Other;
    };
    unsafe { dml_frontdoor_coercible_integer_value_kind(arg, wrapper_depth) }
}

unsafe fn dml_frontdoor_single_list_expr_arg(args: *mut pg_sys::List) -> Option<*mut pg_sys::Expr> {
    let args = unsafe { args.as_ref()? };
    if args.type_ != pg_sys::NodeTag::T_List || args.length != 1 || args.elements.is_null() {
        return None;
    }
    // PG18 exposes List cells through `elements`; this remains the stable
    // single-argument check for implicit-cast FuncExpr wrappers.
    Some(unsafe { (*args.elements).ptr_value }.cast::<pg_sys::Expr>())
}

unsafe fn dml_frontdoor_coercible_integer_value_kind(
    expr: *mut pg_sys::Expr,
    wrapper_depth: usize,
) -> SpireDmlFrontdoorValueKind {
    if expr.is_null() {
        return SpireDmlFrontdoorValueKind::Other;
    }
    match unsafe { (*expr.cast::<pg_sys::Node>()).type_ } {
        pg_sys::NodeTag::T_Const => {
            let const_expr = unsafe { &*expr.cast::<pg_sys::Const>() };
            if !const_expr.constisnull
                && dml_frontdoor_integer_oid_can_coerce_to_bigint(const_expr.consttype)
            {
                SpireDmlFrontdoorValueKind::ConstBigint
            } else {
                SpireDmlFrontdoorValueKind::Other
            }
        }
        pg_sys::NodeTag::T_Param => {
            let param = unsafe { &*expr.cast::<pg_sys::Param>() };
            if dml_frontdoor_integer_oid_can_coerce_to_bigint(param.paramtype) {
                SpireDmlFrontdoorValueKind::ParamBigint
            } else {
                SpireDmlFrontdoorValueKind::Other
            }
        }
        _ => unsafe { dml_frontdoor_value_kind_inner(expr, wrapper_depth + 1) },
    }
}

fn dml_frontdoor_integer_oid_can_coerce_to_bigint(oid: pg_sys::Oid) -> bool {
    oid == pg_sys::INT8OID || oid == pg_sys::INT4OID || oid == pg_sys::INT2OID
}

fn dml_frontdoor_bigint_equality_opcode(opcode: pg_sys::Oid) -> bool {
    opcode == pg_sys::Oid::from(pg_sys::F_INT8EQ)
        || opcode == pg_sys::Oid::from(pg_sys::F_INT84EQ)
        || opcode == pg_sys::Oid::from(pg_sys::F_INT82EQ)
        || opcode == pg_sys::Oid::from(pg_sys::F_INT48EQ)
        || opcode == pg_sys::Oid::from(pg_sys::F_INT28EQ)
}

unsafe fn dml_frontdoor_target_columns(
    target_list: *mut pg_sys::List,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> Vec<String> {
    if target_list.is_null() {
        return Vec::new();
    }
    let targets = unsafe { PgList::<pg_sys::TargetEntry>::from_pg(target_list) };
    let mut columns = Vec::new();
    for target_entry in targets.iter_ptr() {
        let Some(target_entry) = (unsafe { target_entry.as_ref() }) else {
            continue;
        };
        if target_entry.resjunk {
            continue;
        }
        if let Some(column) = context
            .column_names
            .iter()
            .find_map(|(attno, name)| (*attno == target_entry.resno).then(|| (*name).to_owned()))
        {
            columns.push(column);
            continue;
        }
        if !target_entry.resname.is_null() {
            if let Ok(column) = unsafe { CStr::from_ptr(target_entry.resname) }.to_str() {
                columns.push(column.to_owned());
            }
        }
    }
    columns
}

fn operation_name(operation: SpireDmlFrontdoorOperation) -> &'static str {
    match operation {
        SpireDmlFrontdoorOperation::Update => "update_non_embedding",
        SpireDmlFrontdoorOperation::Delete => "delete",
        SpireDmlFrontdoorOperation::PkSelect => "pk_select",
    }
}

fn supported(operation: &'static str, kind: &'static str) -> SpireDmlFrontdoorShapeRow {
    SpireDmlFrontdoorShapeRow {
        supported: true,
        operation,
        kind,
        status: "supported_v1_shape",
        error: None,
        hint: None,
    }
}

fn unsupported_v1(
    operation: &'static str,
    kind: &'static str,
    error: &'static str,
) -> SpireDmlFrontdoorShapeRow {
    unsupported(operation, kind, error, Some(ADR_069_HINT))
}

fn unsupported(
    operation: &'static str,
    kind: &'static str,
    error: &'static str,
    hint: Option<&'static str>,
) -> SpireDmlFrontdoorShapeRow {
    SpireDmlFrontdoorShapeRow {
        supported: false,
        operation,
        kind,
        status: "unsupported_shape",
        error: Some(error),
        hint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_accepts_update_delete_and_pk_select_v1_shapes() {
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&["title"], &["embedding"])).kind,
            "update_non_embedding_by_pk"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(delete_input()).kind,
            "delete_by_pk"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(select_input(&["id", "title"])).kind,
            "pk_select_by_pk"
        );
    }

    #[test]
    fn classifier_rejects_joins_subqueries_and_returning() {
        let mut joined = update_input(&["title"], &["embedding"]);
        joined.has_join = true;
        assert_eq!(
            classify_dml_frontdoor_shape(joined).kind,
            "unsupported_join_shape"
        );

        let mut subquery = delete_input();
        subquery.has_subquery = true;
        assert_eq!(
            classify_dml_frontdoor_shape(subquery).kind,
            "unsupported_subquery_shape"
        );

        let mut returning = delete_input();
        returning.has_returning = true;
        assert_eq!(
            classify_dml_frontdoor_shape(returning).kind,
            "unsupported_returning_shape"
        );
    }

    #[test]
    fn classifier_requires_bigint_pk_equality_predicate() {
        let mut wrong_column = select_input(&["id"]);
        wrong_column.predicate_column = Some("title");
        assert_eq!(
            classify_dml_frontdoor_shape(wrong_column).kind,
            "unsupported_pk_predicate"
        );

        let mut wrong_operator = select_input(&["id"]);
        wrong_operator.predicate_operator = Some(">");
        assert_eq!(
            classify_dml_frontdoor_shape(wrong_operator).kind,
            "unsupported_pk_predicate"
        );

        let mut wrong_value = select_input(&["id"]);
        wrong_value.predicate_value_kind = SpireDmlFrontdoorValueKind::Other;
        assert_eq!(
            classify_dml_frontdoor_shape(wrong_value).kind,
            "unsupported_pk_predicate"
        );
    }

    #[test]
    fn classifier_rejects_embedding_and_pk_updates() {
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&["embedding"], &["embedding"])).kind,
            "embedding_update_rejected"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&["id"], &["embedding"])).kind,
            "unsupported_pk_update"
        );
    }

    #[test]
    fn classifier_rejects_empty_update_or_projection() {
        assert_eq!(
            classify_dml_frontdoor_shape(update_input(&[], &["embedding"])).kind,
            "unsupported_empty_update"
        );
        assert_eq!(
            classify_dml_frontdoor_shape(select_input(&[])).kind,
            "unsupported_empty_projection"
        );
    }

    #[test]
    fn query_layer_maps_command_and_subquery_flags() {
        let mut update_query = pg_sys::Query::default();
        update_query.commandType = pg_sys::CmdType::CMD_UPDATE;
        assert_eq!(
            dml_frontdoor_operation_for_query(&update_query),
            Some(SpireDmlFrontdoorOperation::Update)
        );

        let mut delete_query = pg_sys::Query::default();
        delete_query.commandType = pg_sys::CmdType::CMD_DELETE;
        assert_eq!(
            dml_frontdoor_operation_for_query(&delete_query),
            Some(SpireDmlFrontdoorOperation::Delete)
        );

        let mut select_query = pg_sys::Query::default();
        select_query.commandType = pg_sys::CmdType::CMD_SELECT;
        assert_eq!(
            dml_frontdoor_operation_for_query(&select_query),
            Some(SpireDmlFrontdoorOperation::PkSelect)
        );
        assert!(!dml_frontdoor_query_has_subquery_shape(&select_query));

        select_query.hasSubLinks = true;
        assert!(dml_frontdoor_query_has_subquery_shape(&select_query));
    }

    #[test]
    fn query_layer_recognizes_bigint_const_and_param_values() {
        let mut bigint_const = pg_sys::Const::default();
        bigint_const.xpr.type_ = pg_sys::NodeTag::T_Const;
        bigint_const.consttype = pg_sys::INT8OID;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut bigint_const as *mut pg_sys::Const).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::ConstBigint
        );

        bigint_const.constisnull = true;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut bigint_const as *mut pg_sys::Const).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::Other
        );

        let mut bigint_param = pg_sys::Param::default();
        bigint_param.xpr.type_ = pg_sys::NodeTag::T_Param;
        bigint_param.paramtype = pg_sys::INT8OID;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut bigint_param as *mut pg_sys::Param).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::ParamBigint
        );
    }

    #[test]
    fn query_layer_walks_nested_integer_coercion_wrappers() {
        let mut int_param = pg_sys::Param::default();
        int_param.xpr.type_ = pg_sys::NodeTag::T_Param;
        int_param.paramtype = pg_sys::INT4OID;

        let mut coerce = pg_sys::CoerceViaIO::default();
        coerce.xpr.type_ = pg_sys::NodeTag::T_CoerceViaIO;
        coerce.resulttype = pg_sys::INT8OID;
        coerce.arg = (&mut int_param as *mut pg_sys::Param).cast::<pg_sys::Expr>();

        let mut relabel = pg_sys::RelabelType::default();
        relabel.xpr.type_ = pg_sys::NodeTag::T_RelabelType;
        relabel.resulttype = pg_sys::INT8OID;
        relabel.arg = (&mut coerce as *mut pg_sys::CoerceViaIO).cast::<pg_sys::Expr>();

        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut relabel as *mut pg_sys::RelabelType).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::ParamBigint
        );

        relabel.resulttype = pg_sys::INT4OID;
        assert_eq!(
            unsafe {
                dml_frontdoor_value_kind(
                    (&mut relabel as *mut pg_sys::RelabelType).cast::<pg_sys::Expr>(),
                )
            },
            SpireDmlFrontdoorValueKind::Other
        );
    }

    #[test]
    fn query_layer_recognizes_bigint_integer_equality_variants() {
        for opcode in [
            pg_sys::F_INT8EQ,
            pg_sys::F_INT84EQ,
            pg_sys::F_INT82EQ,
            pg_sys::F_INT48EQ,
            pg_sys::F_INT28EQ,
        ] {
            assert!(dml_frontdoor_bigint_equality_opcode(pg_sys::Oid::from(
                opcode
            )));
        }
        assert!(!dml_frontdoor_bigint_equality_opcode(pg_sys::Oid::from(
            pg_sys::F_INT4EQ
        )));
    }

    #[test]
    fn query_layer_binds_target_relation_var_to_column_name() {
        let context = SpireDmlFrontdoorQueryContext {
            ec_spire_distributed_table: true,
            pk_column: "id",
            column_names: &[(1, "id"), (2, "title"), (3, "embedding")],
            embedding_columns: &["embedding"],
        };
        let mut var = pg_sys::Var::default();
        var.xpr.type_ = pg_sys::NodeTag::T_Var;
        var.varno = 1;
        var.varattno = 1;

        assert_eq!(
            unsafe {
                dml_frontdoor_var_column(
                    (&mut var as *mut pg_sys::Var).cast::<pg_sys::Expr>(),
                    1,
                    &context,
                )
            },
            Some("id".to_owned())
        );

        var.varno = 2;
        assert_eq!(
            unsafe {
                dml_frontdoor_var_column(
                    (&mut var as *mut pg_sys::Var).cast::<pg_sys::Expr>(),
                    1,
                    &context,
                )
            },
            None
        );
    }

    fn update_input<'a>(
        updated_columns: &'a [&'a str],
        embedding_columns: &'a [&'a str],
    ) -> SpireDmlFrontdoorShapeInput<'a> {
        base_input(
            SpireDmlFrontdoorOperation::Update,
            updated_columns,
            &[],
            embedding_columns,
        )
    }

    fn delete_input<'a>() -> SpireDmlFrontdoorShapeInput<'a> {
        base_input(SpireDmlFrontdoorOperation::Delete, &[], &[], &[])
    }

    fn select_input<'a>(projected_columns: &'a [&'a str]) -> SpireDmlFrontdoorShapeInput<'a> {
        base_input(
            SpireDmlFrontdoorOperation::PkSelect,
            &[],
            projected_columns,
            &[],
        )
    }

    fn base_input<'a>(
        operation: SpireDmlFrontdoorOperation,
        updated_columns: &'a [&'a str],
        projected_columns: &'a [&'a str],
        embedding_columns: &'a [&'a str],
    ) -> SpireDmlFrontdoorShapeInput<'a> {
        SpireDmlFrontdoorShapeInput {
            operation,
            ec_spire_distributed_table: true,
            single_table: true,
            has_join: false,
            has_subquery: false,
            has_returning: false,
            pk_column: "id",
            predicate_column: Some("id"),
            predicate_operator: Some("="),
            predicate_value_kind: SpireDmlFrontdoorValueKind::ConstBigint,
            updated_columns,
            projected_columns,
            embedding_columns,
        }
    }
}
