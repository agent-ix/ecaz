//! ADR-069 DML front-door shape classification.
//!
//! The planner hook maps PostgreSQL query trees into this small input model.
//! Query-tree helpers each extract one fact and let the pure classifier compose
//! the final supported/unsupported result.
//! Keeping the v1 safety rules here makes unsupported distributed DML shapes
//! fail closed before any hook can fall through to the coordinator heap path.
#![allow(dead_code)]

use pgrx::{pg_guard, pg_sys, FromDatum, PgList, Spi};

use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

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
    pub(crate) unsupported_shape_fail_closed_enabled: bool,
    pub(crate) plan_rewrite_enabled: bool,
    pub(crate) last_classification_supported: Option<bool>,
    pub(crate) last_classification_kind: Option<&'static str>,
    pub(crate) last_classification_status: Option<&'static str>,
    pub(crate) last_hook_action: Option<&'static str>,
    pub(crate) status: &'static str,
    pub(crate) next_step: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorReplacementDecisionRow {
    pub(crate) target_relation_oid: pg_sys::Oid,
    pub(crate) index_oid: pg_sys::Oid,
    pub(crate) supported: bool,
    pub(crate) operation: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) status: &'static str,
    pub(crate) custom_scan_mode: &'static str,
    pub(crate) primitive: &'static str,
    pub(crate) pk_column: Option<String>,
    pub(crate) pk_value_kind: &'static str,
    pub(crate) pk_value_const: Option<i64>,
    pub(crate) pk_value_param_id: Option<i32>,
    pub(crate) updated_columns: Vec<String>,
    pub(crate) projected_columns: Vec<String>,
    pub(crate) error: Option<&'static str>,
    pub(crate) hint: Option<&'static str>,
    pub(crate) next_step: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorPkArgument {
    pub(crate) pk_column: String,
    pub(crate) value: SpireDmlFrontdoorPkValuePlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpireDmlFrontdoorPkValuePlan {
    ConstBigint(i64),
    ParamBigint(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorPrimitivePlan {
    pub(crate) index_oid: pg_sys::Oid,
    pub(crate) mode: SpireDmlFrontdoorCustomScanMode,
    pub(crate) primitive: &'static str,
    pub(crate) pk_argument: SpireDmlFrontdoorPkArgument,
    pub(crate) updated_columns: Vec<String>,
    pub(crate) projected_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorPrimitiveInvocation {
    pub(crate) index_oid: pg_sys::Oid,
    pub(crate) mode: SpireDmlFrontdoorCustomScanMode,
    pub(crate) primitive: &'static str,
    pub(crate) pk_column: String,
    // ADR-069 v1 DML supports bigint PKs only; widen this when v2 admits UUID
    // or composite PK encodings.
    pub(crate) pk_value: [u8; 8],
    pub(crate) updated_columns: Vec<String>,
    pub(crate) projected_columns: Vec<String>,
}

pub(crate) struct SpireDmlFrontdoorPrimitivePlanExpr {
    pub(crate) primitive_plan: SpireDmlFrontdoorPrimitivePlan,
    pub(crate) pk_value_expr: *mut pg_sys::Expr,
    pub(crate) updated_value_exprs: Vec<*mut pg_sys::Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpireDmlFrontdoorCustomScanMode {
    CoordinatorUpdateTuplePayload,
    CoordinatorDeleteTuplePayload,
    CoordinatorPkSelectTuplePayload,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireDmlFrontdoorRelationContextCacheRow {
    pub(crate) relcache_callback_registered: bool,
    pub(crate) entry_count: i64,
    pub(crate) hit_count: i64,
    pub(crate) miss_count: i64,
    pub(crate) invalidation_count: i64,
    pub(crate) status: &'static str,
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
// These hook diagnostics are intentionally backend-local. They answer "what
// did this session's planner hook last see?" rather than aggregating globally.
static mut HOOK_CLASSIFICATION_ATTEMPTED: bool = false;
static mut LAST_HOOK_CLASSIFICATION_SUPPORTED: Option<bool> = None;
static mut LAST_HOOK_CLASSIFICATION_KIND: Option<&'static str> = None;
static mut LAST_HOOK_CLASSIFICATION_STATUS: Option<&'static str> = None;
static mut LAST_HOOK_ACTION: Option<&'static str> = None;
static mut RELATION_CONTEXT_RELCACHE_CALLBACK_REGISTERED: bool = false;

static RELATION_CONTEXT_CACHE: OnceLock<Mutex<HashMap<u32, CachedRelationContext>>> =
    OnceLock::new();
static RELATION_CONTEXT_CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static RELATION_CONTEXT_CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
static RELATION_CONTEXT_CACHE_INVALIDATIONS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct CachedRelationContext {
    context: SpireDmlFrontdoorRelationContext,
    watched_relation_oids: Vec<pg_sys::Oid>,
}

struct AccessShareIndexRelation {
    relation: pg_sys::Relation,
}

impl AccessShareIndexRelation {
    fn open(index_oid: pg_sys::Oid) -> Option<Self> {
        // SAFETY: PostgreSQL owns the relation cache entry returned by
        // `index_open`; this guard owns the matching AccessShareLock close.
        let relation =
            unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        if relation.is_null() {
            return None;
        }
        Some(Self { relation })
    }

    fn as_ptr(&self) -> pg_sys::Relation {
        self.relation
    }
}

impl Drop for AccessShareIndexRelation {
    fn drop(&mut self) {
        // SAFETY: `relation` was returned by `index_open` in
        // `AccessShareIndexRelation::open`; this guard owns the matching close.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::index_close(self.relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }
}

type RelcacheCallbackFunction =
    Option<unsafe extern "C-unwind" fn(arg: pg_sys::Datum, relid: pg_sys::Oid)>;

unsafe extern "C" {
    fn CacheRegisterRelcacheCallback(function: RelcacheCallbackFunction, arg: pg_sys::Datum);
}

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
        dml_frontdoor_register_relcache_callback();
    }
}

unsafe fn dml_frontdoor_register_relcache_callback() {
    unsafe {
        if !RELATION_CONTEXT_RELCACHE_CALLBACK_REGISTERED {
            CacheRegisterRelcacheCallback(
                Some(dml_frontdoor_relation_context_relcache_callback),
                pg_sys::Datum::from(0),
            );
            RELATION_CONTEXT_RELCACHE_CALLBACK_REGISTERED = true;
        }
    }
}

unsafe extern "C-unwind" fn dml_frontdoor_relation_context_relcache_callback(
    _arg: pg_sys::Datum,
    relid: pg_sys::Oid,
) {
    let Some(cache) = RELATION_CONTEXT_CACHE.get() else {
        return;
    };
    let Ok(mut guard) = cache.lock() else {
        return;
    };
    let before = guard.len();
    if relid == pg_sys::InvalidOid {
        guard.clear();
    } else {
        guard.retain(|_heap_oid, entry| {
            !entry
                .watched_relation_oids
                .iter()
                .any(|watched_oid| *watched_oid == relid)
        });
    }
    if guard.len() != before {
        RELATION_CONTEXT_CACHE_INVALIDATIONS.fetch_add(1, Ordering::Relaxed);
    }
}

pub(crate) fn dml_frontdoor_relation_context_cache_row() -> SpireDmlFrontdoorRelationContextCacheRow
{
    let entry_count = RELATION_CONTEXT_CACHE
        .get()
        .and_then(|cache| cache.lock().ok().map(|guard| guard.len()))
        .unwrap_or(0);
    let callback_registered = unsafe { RELATION_CONTEXT_RELCACHE_CALLBACK_REGISTERED };
    SpireDmlFrontdoorRelationContextCacheRow {
        relcache_callback_registered: callback_registered,
        entry_count: i64::try_from(entry_count).unwrap_or(i64::MAX),
        hit_count: i64::try_from(RELATION_CONTEXT_CACHE_HITS.load(Ordering::Relaxed))
            .unwrap_or(i64::MAX),
        miss_count: i64::try_from(RELATION_CONTEXT_CACHE_MISSES.load(Ordering::Relaxed))
            .unwrap_or(i64::MAX),
        invalidation_count: i64::try_from(
            RELATION_CONTEXT_CACHE_INVALIDATIONS.load(Ordering::Relaxed),
        )
        .unwrap_or(i64::MAX),
        status: if callback_registered {
            "relcache_invalidated_cache_ready"
        } else {
            "relcache_callback_not_registered"
        },
    }
}

pub(crate) fn dml_frontdoor_hook_status_row() -> SpireDmlFrontdoorHookStatusRow {
    let (installed, classifier_invoked, last_supported, last_kind, last_status, last_action) = unsafe {
        (
            PLANNER_HOOK_INSTALLED,
            HOOK_CLASSIFICATION_ATTEMPTED,
            LAST_HOOK_CLASSIFICATION_SUPPORTED,
            LAST_HOOK_CLASSIFICATION_KIND,
            LAST_HOOK_CLASSIFICATION_STATUS,
            LAST_HOOK_ACTION,
        )
    };
    SpireDmlFrontdoorHookStatusRow {
        hook_name: "ec_spire_dml_frontdoor_planner_hook",
        planner_hook_installed: installed,
        query_shape_classifier_enabled: true,
        query_shape_classifier_invoked_by_hook: classifier_invoked,
        unsupported_shape_fail_closed_enabled: true,
        // True means supported UPDATE/DELETE shapes are planned as a
        // CustomScan; per-mode executor dispatch still gates execution.
        plan_rewrite_enabled: true,
        last_classification_supported: last_supported,
        last_classification_kind: last_kind,
        last_classification_status: last_status,
        last_hook_action: last_action,
        status: if installed && classifier_invoked {
            last_action.unwrap_or("pass_through_classifier_observed")
        } else if installed {
            "fail_closed_guard_ready"
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
    unsafe { dml_frontdoor_register_relcache_callback() };
    if heap_relation_oid == pg_sys::InvalidOid {
        return Err(
            "ec_spire DML frontdoor catalog relation context requires a valid heap relation OID"
                .to_owned(),
        );
    }

    let cache_key = heap_relation_oid.to_u32();
    if let Some(context) = RELATION_CONTEXT_CACHE
        .get()
        .and_then(|cache| cache.lock().ok()?.get(&cache_key).cloned())
    {
        RELATION_CONTEXT_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
        return Ok(context.context);
    }
    RELATION_CONTEXT_CACHE_MISSES.fetch_add(1, Ordering::Relaxed);

    let heap_relation = unsafe {
        pg_sys::table_open(
            heap_relation_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        )
    };
    if heap_relation.is_null() {
        return Err("ec_spire DML frontdoor catalog relation open returned NULL".to_owned());
    }

    let result = unsafe {
        dml_frontdoor_relation_context_catalog_for_open_heap(heap_relation).map(
            |(context, watched_relation_oids)| {
                RELATION_CONTEXT_CACHE
                    .get_or_init(|| Mutex::new(HashMap::new()))
                    .lock()
                    .map(|mut guard| {
                        guard.insert(
                            cache_key,
                            CachedRelationContext {
                                context: context.clone(),
                                watched_relation_oids,
                            },
                        );
                    })
                    .ok();
                context
            },
        )
    };
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
    // Run the SPIRE fail-closed guard before chained hooks so unsupported
    // distributed DML cannot be rewritten into a coordinator-heap base plan.
    let decision = unsafe { dml_frontdoor_observe_planner_query(parse) };
    let plan_expr = unsafe { dml_frontdoor_plan_tree_replacement_expr(parse, decision.as_ref()) };
    let planned_stmt = if let Some(previous_hook) = unsafe { PREVIOUS_PLANNER_HOOK } {
        unsafe { previous_hook(parse, query_string, cursor_options, bound_params) }
    } else {
        unsafe { pg_sys::standard_planner(parse, query_string, cursor_options, bound_params) }
    };
    unsafe { dml_frontdoor_maybe_replace_plan_tree(planned_stmt, plan_expr) }
}

unsafe fn dml_frontdoor_plan_tree_replacement_expr(
    query: *mut pg_sys::Query,
    decision: Option<&SpireDmlFrontdoorReplacementDecisionRow>,
) -> Option<SpireDmlFrontdoorPrimitivePlanExpr> {
    unsafe {
        let decision = decision?;
        if !dml_frontdoor_uses_plan_tree_replacement(decision) {
            return None;
        }
        let plan_expr = dml_frontdoor_primitive_plan_expr_catalog_row(query).unwrap_or_else(|| {
            pgrx::error!("ec_spire DML frontdoor plan replacement lost primitive plan")
        });
        Some(match plan_expr {
            Ok(plan_expr) => plan_expr,
            Err(err) => pgrx::error!("{err}"),
        })
    }
}

unsafe fn dml_frontdoor_maybe_replace_plan_tree(
    planned_stmt: *mut pg_sys::PlannedStmt,
    plan_expr: Option<SpireDmlFrontdoorPrimitivePlanExpr>,
) -> *mut pg_sys::PlannedStmt {
    unsafe {
        let Some(plan_expr) = plan_expr else {
            return planned_stmt;
        };
        if planned_stmt.is_null() {
            pgrx::error!("ec_spire DML frontdoor plan replacement got null PlannedStmt");
        }
        (*planned_stmt).planTree = super::custom_scan::custom_scan_dml_replacement_plan(
            plan_expr,
            (*planned_stmt).planTree,
        );
        LAST_HOOK_ACTION = Some("plan_tree_replaced_customscan");
        planned_stmt
    }
}

fn dml_frontdoor_uses_plan_tree_replacement(
    decision: &SpireDmlFrontdoorReplacementDecisionRow,
) -> bool {
    decision.supported
        && matches!(
            decision.custom_scan_mode,
            "coordinator_update_tuple_payload" | "coordinator_delete_tuple_payload"
        )
}

unsafe fn dml_frontdoor_observe_planner_query(
    query: *mut pg_sys::Query,
) -> Option<SpireDmlFrontdoorReplacementDecisionRow> {
    let Some(decision) = (unsafe { dml_frontdoor_replacement_decision_catalog_row(query) }) else {
        return None;
    };
    let action = dml_frontdoor_hook_action(&decision);
    unsafe {
        HOOK_CLASSIFICATION_ATTEMPTED = true;
        LAST_HOOK_CLASSIFICATION_SUPPORTED = Some(decision.supported);
        LAST_HOOK_CLASSIFICATION_KIND = Some(decision.kind);
        LAST_HOOK_CLASSIFICATION_STATUS = Some(decision.status);
        LAST_HOOK_ACTION = Some(action);
    }
    if action == "planner_error_fail_closed" {
        dml_frontdoor_raise_planner_error(&decision);
    }
    Some(decision)
}

fn dml_frontdoor_hook_action(decision: &SpireDmlFrontdoorReplacementDecisionRow) -> &'static str {
    if decision.supported {
        "pass_through_until_rewrite"
    } else if dml_frontdoor_decision_is_spire_frontdoor_candidate(decision) {
        "planner_error_fail_closed"
    } else {
        "pass_through_not_spire_frontdoor"
    }
}

fn dml_frontdoor_decision_is_spire_frontdoor_candidate(
    decision: &SpireDmlFrontdoorReplacementDecisionRow,
) -> bool {
    if decision.kind == "relation_context_error" {
        return true;
    }
    if decision.index_oid == pg_sys::InvalidOid {
        return false;
    }
    if decision.operation == "pk_select" {
        return matches!(
            decision.kind,
            "unsupported_empty_projection" | "unsupported_pk_predicate"
        );
    }
    true
}

fn dml_frontdoor_raise_planner_error(decision: &SpireDmlFrontdoorReplacementDecisionRow) -> ! {
    let message = decision
        .error
        .unwrap_or("ec_spire_distributed: unsupported DML shape for v1 coordinator routing");
    let hint = decision.hint.unwrap_or(ADR_069_HINT);
    pgrx::pg_sys::panic::ErrorReport::new(
        pgrx::PgSqlErrorCode::ERRCODE_FEATURE_NOT_SUPPORTED,
        message,
        pgrx::function_name!(),
    )
    .set_hint(hint)
    .report(pgrx::PgLogLevel::ERROR);
    unreachable!();
}

pub(crate) unsafe fn dml_frontdoor_replacement_decision_catalog_row(
    query: *mut pg_sys::Query,
) -> Option<SpireDmlFrontdoorReplacementDecisionRow> {
    let target_relation_oid = unsafe { dml_frontdoor_target_relation_oid(query)? };
    let relation = match unsafe { dml_frontdoor_relation_context_catalog_row(target_relation_oid) }
    {
        Ok(relation) => relation,
        Err(_err) => {
            return Some(SpireDmlFrontdoorReplacementDecisionRow {
                target_relation_oid,
                index_oid: pg_sys::InvalidOid,
                supported: false,
                operation: "unsupported",
                kind: "relation_context_error",
                status: "unsupported_shape",
                custom_scan_mode: "none",
                primitive: "none",
                pk_column: None,
                pk_value_kind: "other",
                pk_value_const: None,
                pk_value_param_id: None,
                updated_columns: Vec::new(),
                projected_columns: Vec::new(),
                error: Some("ec_spire_distributed: relation context could not be loaded"),
                hint: Some(ADR_069_HINT),
                next_step: "raise ADR-069 planner error instead of using coordinator heap path",
            });
        }
    };
    let detail = unsafe { dml_frontdoor_query_detail_with_relation(query, &relation)? };
    Some(dml_frontdoor_replacement_decision_from_shape(
        target_relation_oid,
        relation.index_oid,
        detail,
    ))
}

pub(crate) unsafe fn dml_frontdoor_primitive_plan_expr_catalog_row(
    query: *mut pg_sys::Query,
) -> Option<Result<SpireDmlFrontdoorPrimitivePlanExpr, String>> {
    let target_relation_oid = unsafe { dml_frontdoor_target_relation_oid(query)? };
    let relation = match unsafe { dml_frontdoor_relation_context_catalog_row(target_relation_oid) }
    {
        Ok(relation) => relation,
        Err(_err) => {
            return Some(Err(
                "ec_spire DML frontdoor CustomScan expression handoff could not load relation context"
                    .to_owned(),
            ));
        }
    };
    let detail = unsafe { dml_frontdoor_query_detail_with_relation(query, &relation)? };
    let updated_value_exprs = detail.updated_value_exprs.clone();
    let pk_value_expr = detail.pk_value_expr.ok_or_else(|| {
        "ec_spire DML frontdoor CustomScan expression handoff requires a PK value expression"
            .to_owned()
    });
    let decision = dml_frontdoor_replacement_decision_from_shape(
        target_relation_oid,
        relation.index_oid,
        detail,
    );
    let primitive_plan = dml_frontdoor_primitive_plan_from_replacement_decision(&decision);
    Some(pk_value_expr.and_then(|pk_value_expr| {
        primitive_plan.map(|primitive_plan| SpireDmlFrontdoorPrimitivePlanExpr {
            primitive_plan,
            pk_value_expr,
            updated_value_exprs,
        })
    }))
}

pub(crate) unsafe fn dml_frontdoor_primitive_plan_expr_from_baserel(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
) -> Option<Result<SpireDmlFrontdoorPrimitivePlanExpr, String>> {
    unsafe {
        if root.is_null() || rel.is_null() {
            return None;
        }
        let root_ref = root.as_ref()?;
        let rel_ref = rel.as_ref()?;
        let query_ref = root_ref.parse.as_ref()?;
        let operation = dml_frontdoor_operation_for_query(query_ref)?;
        let target_rtindex =
            match dml_frontdoor_baserel_target_rtindex(query_ref, rel_ref, operation) {
                Ok(Some(target_rtindex)) => target_rtindex,
                Ok(None) => return None,
                Err(err) => return Some(Err(err)),
            };
        let target_relation_oid =
            dml_frontdoor_relation_oid_from_rtable(query_ref, target_rtindex)?;
        let relation = match dml_frontdoor_relation_context_catalog_row(target_relation_oid) {
            Ok(relation) => relation,
            Err(_err) => {
                return Some(Err(
                    "ec_spire DML frontdoor baserel expression handoff could not load relation context"
                        .to_owned(),
                ));
            }
        };
        let detail = dml_frontdoor_query_detail_from_baserel(
            root_ref.parse,
            rel_ref,
            operation,
            target_rtindex,
            &relation,
        )?;
        let pk_value_expr = detail.pk_value_expr;
        let updated_value_exprs = detail.updated_value_exprs.clone();
        let decision = dml_frontdoor_replacement_decision_from_shape(
            target_relation_oid,
            relation.index_oid,
            detail,
        );
        if !decision.supported {
            return None;
        }
        let pk_value_expr = pk_value_expr.ok_or_else(|| {
            "ec_spire DML frontdoor baserel expression handoff requires a PK value expression"
                .to_owned()
        });
        let primitive_plan = dml_frontdoor_primitive_plan_from_replacement_decision(&decision);
        Some(pk_value_expr.and_then(|pk_value_expr| {
            primitive_plan.map(|primitive_plan| SpireDmlFrontdoorPrimitivePlanExpr {
                primitive_plan,
                pk_value_expr,
                updated_value_exprs,
            })
        }))
    }
}

pub(crate) unsafe fn dml_frontdoor_pk_select_primitive_plan_expr_from_baserel(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
) -> Option<Result<SpireDmlFrontdoorPrimitivePlanExpr, String>> {
    unsafe {
        let plan_expr = dml_frontdoor_primitive_plan_expr_from_baserel(root, rel)?;
        Some(plan_expr.and_then(|plan_expr| {
            dml_frontdoor_primitive_plan_expr_require_mode(
                plan_expr,
                SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload,
                "PK SELECT",
            )
        }))
    }
}

pub(crate) unsafe fn dml_frontdoor_update_primitive_plan_expr_from_baserel(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
) -> Option<Result<SpireDmlFrontdoorPrimitivePlanExpr, String>> {
    unsafe {
        let plan_expr = dml_frontdoor_primitive_plan_expr_from_baserel(root, rel)?;
        Some(plan_expr.and_then(|plan_expr| {
            dml_frontdoor_primitive_plan_expr_require_mode(
                plan_expr,
                SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload,
                "UPDATE",
            )
        }))
    }
}

pub(crate) unsafe fn dml_frontdoor_delete_primitive_plan_expr_from_baserel(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
) -> Option<Result<SpireDmlFrontdoorPrimitivePlanExpr, String>> {
    unsafe {
        let plan_expr = dml_frontdoor_primitive_plan_expr_from_baserel(root, rel)?;
        Some(plan_expr.and_then(|plan_expr| {
            dml_frontdoor_primitive_plan_expr_require_mode(
                plan_expr,
                SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload,
                "DELETE",
            )
        }))
    }
}

fn dml_frontdoor_primitive_plan_expr_require_mode(
    plan_expr: SpireDmlFrontdoorPrimitivePlanExpr,
    expected_mode: SpireDmlFrontdoorCustomScanMode,
    operation_name: &'static str,
) -> Result<SpireDmlFrontdoorPrimitivePlanExpr, String> {
    if plan_expr.primitive_plan.mode == expected_mode {
        Ok(plan_expr)
    } else {
        Err(format!(
            "ec_spire DML frontdoor baserel expression handoff expected {operation_name} primitive plan"
        ))
    }
}

pub(crate) fn dml_frontdoor_bigint_pk_value_bytes(value: i64) -> [u8; 8] {
    value.to_be_bytes()
}

pub(crate) fn dml_frontdoor_pk_argument_from_replacement_decision(
    decision: &SpireDmlFrontdoorReplacementDecisionRow,
) -> Result<SpireDmlFrontdoorPkArgument, String> {
    if !decision.supported {
        return Err(format!(
            "ec_spire DML frontdoor PK argument requires a supported decision, got {}",
            decision.kind
        ));
    }
    let pk_column = decision
        .pk_column
        .as_deref()
        .filter(|column| !column.is_empty())
        .ok_or_else(|| "ec_spire DML frontdoor PK argument is missing pk_column".to_owned())?
        .to_owned();
    let value = match decision.pk_value_kind {
        "const_bigint" => {
            SpireDmlFrontdoorPkValuePlan::ConstBigint(decision.pk_value_const.ok_or_else(|| {
                "ec_spire DML frontdoor const_bigint PK argument is missing a value".to_owned()
            })?)
        }
        "param_bigint" => {
            let param_id = decision.pk_value_param_id.ok_or_else(|| {
                "ec_spire DML frontdoor param_bigint PK argument is missing a parameter id"
                    .to_owned()
            })?;
            if param_id <= 0 {
                return Err(format!(
                    "ec_spire DML frontdoor param_bigint PK argument has invalid parameter id {param_id}"
                ));
            }
            SpireDmlFrontdoorPkValuePlan::ParamBigint(param_id)
        }
        other => {
            return Err(format!(
                "ec_spire DML frontdoor PK argument has unsupported value kind {other}"
            ));
        }
    };
    Ok(SpireDmlFrontdoorPkArgument { pk_column, value })
}

pub(crate) fn dml_frontdoor_primitive_plan_from_replacement_decision(
    decision: &SpireDmlFrontdoorReplacementDecisionRow,
) -> Result<SpireDmlFrontdoorPrimitivePlan, String> {
    if decision.index_oid == pg_sys::InvalidOid {
        return Err("ec_spire DML frontdoor primitive plan requires a valid index OID".to_owned());
    }
    let pk_argument = dml_frontdoor_pk_argument_from_replacement_decision(decision)?;
    let mode = dml_frontdoor_custom_scan_mode_from_decision(decision)?;
    let expected_primitive = dml_frontdoor_primitive_for_mode(mode);
    if decision.primitive != expected_primitive {
        return Err(format!(
            "ec_spire DML frontdoor primitive plan mode {} requires primitive {}, got {}",
            decision.custom_scan_mode, expected_primitive, decision.primitive
        ));
    }
    match mode {
        SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload => {
            if decision.updated_columns.is_empty() {
                return Err(
                    "ec_spire DML frontdoor UPDATE primitive plan requires updated columns"
                        .to_owned(),
                );
            }
            if !decision.projected_columns.is_empty() {
                return Err(
                    "ec_spire DML frontdoor UPDATE primitive plan must not project columns"
                        .to_owned(),
                );
            }
        }
        SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload => {
            if !decision.updated_columns.is_empty() || !decision.projected_columns.is_empty() {
                return Err(
                    "ec_spire DML frontdoor DELETE primitive plan must not carry column payloads"
                        .to_owned(),
                );
            }
        }
        SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload => {
            if decision.projected_columns.is_empty() {
                return Err(
                    "ec_spire DML frontdoor PK SELECT primitive plan requires projected columns"
                        .to_owned(),
                );
            }
            if !decision.updated_columns.is_empty() {
                return Err(
                    "ec_spire DML frontdoor PK SELECT primitive plan must not update columns"
                        .to_owned(),
                );
            }
        }
    }
    Ok(SpireDmlFrontdoorPrimitivePlan {
        index_oid: decision.index_oid,
        mode,
        primitive: expected_primitive,
        pk_argument,
        updated_columns: decision.updated_columns.clone(),
        projected_columns: decision.projected_columns.clone(),
    })
}

pub(crate) fn dml_frontdoor_primitive_plan_const_pk_value_bytes(
    plan: &SpireDmlFrontdoorPrimitivePlan,
) -> Result<[u8; 8], String> {
    match plan.pk_argument.value {
        SpireDmlFrontdoorPkValuePlan::ConstBigint(value) => {
            Ok(dml_frontdoor_bigint_pk_value_bytes(value))
        }
        SpireDmlFrontdoorPkValuePlan::ParamBigint(param_id) => Err(format!(
            "ec_spire DML frontdoor primitive plan requires executor parameter evaluation for PK parameter ${param_id}"
        )),
    }
}

pub(crate) unsafe fn dml_frontdoor_primitive_plan_pk_value_bytes(
    plan: &SpireDmlFrontdoorPrimitivePlan,
    params: pg_sys::ParamListInfo,
) -> Result<[u8; 8], String> {
    match plan.pk_argument.value {
        SpireDmlFrontdoorPkValuePlan::ConstBigint(value) => {
            Ok(dml_frontdoor_bigint_pk_value_bytes(value))
        }
        SpireDmlFrontdoorPkValuePlan::ParamBigint(param_id) => {
            let value = unsafe { dml_frontdoor_bound_param_bigint_value(params, param_id)? };
            Ok(dml_frontdoor_bigint_pk_value_bytes(value))
        }
    }
}

pub(crate) unsafe fn dml_frontdoor_primitive_invocation_from_plan(
    plan: &SpireDmlFrontdoorPrimitivePlan,
    params: pg_sys::ParamListInfo,
) -> Result<SpireDmlFrontdoorPrimitiveInvocation, String> {
    let pk_value = unsafe { dml_frontdoor_primitive_plan_pk_value_bytes(plan, params)? };
    if plan.pk_argument.pk_column.is_empty() {
        return Err("ec_spire DML frontdoor primitive invocation requires pk_column".to_owned());
    }
    Ok(SpireDmlFrontdoorPrimitiveInvocation {
        index_oid: plan.index_oid,
        mode: plan.mode,
        primitive: plan.primitive,
        pk_column: plan.pk_argument.pk_column.clone(),
        pk_value,
        updated_columns: plan.updated_columns.clone(),
        projected_columns: plan.projected_columns.clone(),
    })
}

unsafe fn dml_frontdoor_bound_param_bigint_value(
    params: pg_sys::ParamListInfo,
    param_id: i32,
) -> Result<i64, String> {
    if param_id <= 0 {
        return Err(format!(
            "ec_spire DML frontdoor PK parameter id {param_id} is invalid"
        ));
    }
    if params.is_null() {
        return Err(format!(
            "ec_spire DML frontdoor PK parameter ${param_id} has no bound parameter list"
        ));
    }
    let params_ref = unsafe { &*params };
    if param_id > params_ref.numParams {
        return Err(format!(
            "ec_spire DML frontdoor PK parameter ${param_id} exceeds bound parameter count {}",
            params_ref.numParams
        ));
    }
    let param = if let Some(fetch) = params_ref.paramFetch {
        let mut workspace = pg_sys::ParamExternData::default();
        // paramFetch may return a pointer into workspace; consume it before
        // this function returns rather than handing it to a caller.
        unsafe { fetch(params, param_id, false, &mut workspace) }
    } else {
        unsafe { params_ref.params.as_ptr().add((param_id - 1) as usize) }
    };
    if param.is_null() {
        return Err(format!(
            "ec_spire DML frontdoor PK parameter ${param_id} fetch returned NULL"
        ));
    }
    let param_ref = unsafe { &*param };
    if param_ref.isnull {
        return Err(format!(
            "ec_spire DML frontdoor PK parameter ${param_id} must not be NULL"
        ));
    }
    unsafe { dml_frontdoor_param_datum_to_bigint(param_id, param_ref.value, param_ref.ptype) }
}

unsafe fn dml_frontdoor_param_datum_to_bigint(
    param_id: i32,
    datum: pg_sys::Datum,
    typoid: pg_sys::Oid,
) -> Result<i64, String> {
    match typoid {
        pg_sys::INT2OID => Ok(i64::from(unsafe { pg_sys::DatumGetInt16(datum) })),
        pg_sys::INT4OID => Ok(i64::from(unsafe { pg_sys::DatumGetInt32(datum) })),
        pg_sys::INT8OID => Ok(unsafe { pg_sys::DatumGetInt64(datum) }),
        other => Err(format!(
            "ec_spire DML frontdoor PK parameter ${param_id} has unsupported type OID {}",
            other.to_u32()
        )),
    }
}

fn dml_frontdoor_custom_scan_mode_from_decision(
    decision: &SpireDmlFrontdoorReplacementDecisionRow,
) -> Result<SpireDmlFrontdoorCustomScanMode, String> {
    match decision.custom_scan_mode {
        "coordinator_update_tuple_payload" => {
            Ok(SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload)
        }
        "coordinator_delete_tuple_payload" => {
            Ok(SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload)
        }
        "coordinator_pk_select_tuple_payload" => {
            Ok(SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload)
        }
        other => Err(format!(
            "ec_spire DML frontdoor primitive plan has unsupported CustomScan mode {other}"
        )),
    }
}

fn dml_frontdoor_primitive_for_mode(mode: SpireDmlFrontdoorCustomScanMode) -> &'static str {
    match mode {
        SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload => {
            "ec_spire_forward_coordinator_update_tuple_payload"
        }
        SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload => {
            "ec_spire_prepare_coordinator_delete_tuple_payload"
        }
        SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload => {
            "ec_spire_forward_coordinator_select_tuple_payload"
        }
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
    unsafe { dml_frontdoor_classify_query_with_relation(query, &relation) }
}

unsafe fn dml_frontdoor_classify_query_with_relation(
    query: *mut pg_sys::Query,
    relation: &SpireDmlFrontdoorRelationContext,
) -> Option<SpireDmlFrontdoorShapeRow> {
    let detail = unsafe { dml_frontdoor_query_detail_with_relation(query, relation)? };
    Some(detail.shape)
}

struct SpireDmlFrontdoorQueryDetail {
    shape: SpireDmlFrontdoorShapeRow,
    pk_column: Option<String>,
    pk_value: SpireDmlFrontdoorPredicateValue,
    pk_value_expr: Option<*mut pg_sys::Expr>,
    updated_columns: Vec<String>,
    updated_value_exprs: Vec<*mut pg_sys::Expr>,
    projected_columns: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpireDmlFrontdoorPredicateValue {
    kind: SpireDmlFrontdoorValueKind,
    const_bigint: Option<i64>,
    param_id: Option<i32>,
}

unsafe fn dml_frontdoor_query_detail_with_relation(
    query: *mut pg_sys::Query,
    relation: &SpireDmlFrontdoorRelationContext,
) -> Option<SpireDmlFrontdoorQueryDetail> {
    let query_ref = unsafe { query.as_ref()? };
    let operation = dml_frontdoor_operation_for_query(query_ref)?;
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
    let shape = unsafe { classify_dml_frontdoor_query(query, query_context)? };
    let target_rtindex = match operation {
        SpireDmlFrontdoorOperation::Update | SpireDmlFrontdoorOperation::Delete => {
            query_ref.resultRelation
        }
        SpireDmlFrontdoorOperation::PkSelect => unsafe {
            single_range_table_ref(query_ref).unwrap_or_default()
        },
    };
    let predicate =
        unsafe { dml_frontdoor_pk_predicate(query_ref, target_rtindex, &query_context) };
    let updated_targets = if operation == SpireDmlFrontdoorOperation::Update {
        unsafe { dml_frontdoor_target_column_exprs(query_ref.targetList, &query_context) }
    } else {
        Vec::new()
    };
    let updated_columns = updated_targets
        .iter()
        .map(|(column, _expr)| column.clone())
        .collect::<Vec<_>>();
    let updated_value_exprs = updated_targets
        .iter()
        .map(|(_column, expr)| *expr)
        .collect::<Vec<_>>();
    let projected_columns = if operation == SpireDmlFrontdoorOperation::PkSelect {
        unsafe { dml_frontdoor_target_columns(query_ref.targetList, &query_context) }
    } else {
        Vec::new()
    };
    Some(SpireDmlFrontdoorQueryDetail {
        shape,
        pk_column: relation.pk_column.clone(),
        pk_value: predicate.value,
        pk_value_expr: predicate.value_expr,
        updated_columns,
        updated_value_exprs,
        projected_columns,
    })
}

fn dml_frontdoor_baserel_target_rtindex(
    query: &pg_sys::Query,
    rel: &pg_sys::RelOptInfo,
    operation: SpireDmlFrontdoorOperation,
) -> Result<Option<i32>, String> {
    let rel_rtindex = i32::try_from(rel.relid).map_err(|_| {
        "ec_spire DML frontdoor baserel expression handoff relid exceeds planner rtindex range"
            .to_owned()
    })?;
    match operation {
        SpireDmlFrontdoorOperation::PkSelect => Ok(Some(rel_rtindex)),
        SpireDmlFrontdoorOperation::Update | SpireDmlFrontdoorOperation::Delete => {
            if query.resultRelation == rel_rtindex {
                Ok(Some(rel_rtindex))
            } else {
                Ok(None)
            }
        }
    }
}

unsafe fn dml_frontdoor_query_detail_from_baserel(
    query: *mut pg_sys::Query,
    rel: &pg_sys::RelOptInfo,
    operation: SpireDmlFrontdoorOperation,
    target_rtindex: i32,
    relation: &SpireDmlFrontdoorRelationContext,
) -> Option<SpireDmlFrontdoorQueryDetail> {
    let query_ref = unsafe { query.as_ref()? };
    if dml_frontdoor_operation_for_query(query_ref) != Some(operation) {
        return None;
    }
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
    // set_rel_pathlist_hook sees planner-normalized RestrictInfo clauses;
    // diagnostic SQL helpers operate on a freshly analyzed Query jointree.
    let predicate = unsafe {
        dml_frontdoor_pk_predicate_from_baserestrictinfo(
            rel.baserestrictinfo,
            target_rtindex,
            &query_context,
        )
        .unwrap_or_else(|| dml_frontdoor_pk_predicate(query_ref, target_rtindex, &query_context))
    };
    let updated_targets = if operation == SpireDmlFrontdoorOperation::Update {
        unsafe { dml_frontdoor_target_column_exprs(query_ref.targetList, &query_context) }
    } else {
        Vec::new()
    };
    let updated_columns = updated_targets
        .iter()
        .map(|(column, _expr)| column.clone())
        .collect::<Vec<_>>();
    let updated_value_exprs = updated_targets
        .iter()
        .map(|(_column, expr)| *expr)
        .collect::<Vec<_>>();
    let projected_columns = if operation == SpireDmlFrontdoorOperation::PkSelect {
        unsafe { dml_frontdoor_target_columns(query_ref.targetList, &query_context) }
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
    let has_join = unsafe { dml_frontdoor_query_has_join_shape(query_ref, operation) };
    let shape = classify_dml_frontdoor_shape(SpireDmlFrontdoorShapeInput {
        operation,
        ec_spire_distributed_table: query_context.ec_spire_distributed_table,
        single_table: target_rtindex > 0 && !has_join,
        has_join,
        has_subquery: dml_frontdoor_query_has_subquery_shape(query_ref),
        has_returning: !query_ref.returningList.is_null(),
        pk_column: query_context.pk_column,
        predicate_column: predicate.column.as_deref(),
        predicate_operator: predicate.operator,
        predicate_value_kind: predicate.value.kind,
        updated_columns: &updated_column_refs,
        projected_columns: &projected_column_refs,
        embedding_columns: query_context.embedding_columns,
    });
    Some(SpireDmlFrontdoorQueryDetail {
        shape,
        pk_column: relation.pk_column.clone(),
        pk_value: predicate.value,
        pk_value_expr: predicate.value_expr,
        updated_columns,
        updated_value_exprs,
        projected_columns,
    })
}

fn dml_frontdoor_replacement_decision_from_shape(
    target_relation_oid: pg_sys::Oid,
    index_oid: pg_sys::Oid,
    detail: SpireDmlFrontdoorQueryDetail,
) -> SpireDmlFrontdoorReplacementDecisionRow {
    let shape = detail.shape;
    let (custom_scan_mode, primitive, next_step) = if shape.supported {
        match shape.operation {
            "update_non_embedding" => (
                "coordinator_update_tuple_payload",
                "ec_spire_forward_coordinator_update_tuple_payload",
                "replace base UPDATE plan with DML CustomScan executor node",
            ),
            "delete" => (
                "coordinator_delete_tuple_payload",
                "ec_spire_prepare_coordinator_delete_tuple_payload",
                "replace base DELETE plan with DML CustomScan executor node",
            ),
            "pk_select" => (
                "coordinator_pk_select_tuple_payload",
                "ec_spire_forward_coordinator_select_tuple_payload",
                "replace base PK SELECT plan with DML CustomScan executor node",
            ),
            _ => (
                "none",
                "none",
                "raise ADR-069 planner error instead of using coordinator heap path",
            ),
        }
    } else {
        (
            "none",
            "none",
            "raise ADR-069 planner error instead of using coordinator heap path",
        )
    };
    SpireDmlFrontdoorReplacementDecisionRow {
        target_relation_oid,
        index_oid,
        supported: shape.supported,
        operation: shape.operation,
        kind: shape.kind,
        status: shape.status,
        custom_scan_mode,
        primitive,
        pk_column: detail.pk_column,
        pk_value_kind: dml_frontdoor_value_kind_name(detail.pk_value.kind),
        pk_value_const: detail.pk_value.const_bigint,
        pk_value_param_id: detail.pk_value.param_id,
        updated_columns: detail.updated_columns,
        projected_columns: detail.projected_columns,
        error: shape.error,
        hint: shape.hint,
        next_step,
    }
}

struct SpireDmlFrontdoorPrimaryKeyColumn {
    column_name: String,
    column_type: String,
}

unsafe fn dml_frontdoor_relation_context_catalog_for_open_heap(
    heap_relation: pg_sys::Relation,
) -> Result<(SpireDmlFrontdoorRelationContext, Vec<pg_sys::Oid>), String> {
    let heap_relation_oid = unsafe { (*heap_relation).rd_id };
    let column_names = unsafe { dml_frontdoor_relation_column_names_from_rel(heap_relation)? };
    let (index_oid, pk, mut watched_relation_oids) =
        unsafe { dml_frontdoor_catalog_index_and_pk(heap_relation)? };
    watched_relation_oids.push(heap_relation_oid);
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

    Ok((
        SpireDmlFrontdoorRelationContext {
            heap_relation_oid,
            index_oid,
            ec_spire_distributed_table,
            pk_column,
            pk_type,
            column_names,
            embedding_columns,
            status,
            next_step,
        },
        watched_relation_oids,
    ))
}

unsafe fn dml_frontdoor_catalog_index_and_pk(
    heap_relation: pg_sys::Relation,
) -> Result<
    (
        pg_sys::Oid,
        Option<SpireDmlFrontdoorPrimaryKeyColumn>,
        Vec<pg_sys::Oid>,
    ),
    String,
> {
    let ec_spire_am_oid = unsafe { pg_sys::get_index_am_oid(EC_SPIRE_AM_NAME.as_ptr(), true) };
    // RelationGetIndexList returns a private OID list, so each index can be
    // opened and closed under AccessShareLock while walking this copy.
    let index_list =
        unsafe { PgList::<pg_sys::Oid>::from_pg(pg_sys::RelationGetIndexList(heap_relation)) };
    let mut ec_spire_index_count = 0_i64;
    let mut ec_spire_index_oid = pg_sys::InvalidOid;
    let mut primary_key = None;
    let mut watched_index_oids = Vec::new();

    for index_oid in index_list.iter_oid() {
        watched_index_oids.push(index_oid);
        let Some(index_relation) = AccessShareIndexRelation::open(index_oid) else {
            continue;
        };
        let index_form = unsafe { (*index_relation.as_ptr()).rd_index.as_ref() };
        let class_form = unsafe { (*index_relation.as_ptr()).rd_rel.as_ref() };
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
    }

    if ec_spire_index_count > 1 {
        return Err(
            "ec_spire DML frontdoor v1 requires at most one ec_spire index per heap relation"
                .to_owned(),
        );
    }
    Ok((ec_spire_index_oid, primary_key, watched_index_oids))
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
    let Some(index_relation) = AccessShareIndexRelation::open(index_oid) else {
        return Err("ec_spire DML frontdoor catalog index open returned NULL".to_owned());
    };
    let result = unsafe {
        let index_form = (*index_relation.as_ptr())
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
    if input.operation == SpireDmlFrontdoorOperation::PkSelect
        && input.predicate_column != Some(input.pk_column)
    {
        return unsupported(
            operation,
            "non_pk_select_pass_through",
            "ec_spire_distributed: non-PK SELECT is not routed by the DML frontdoor",
            None,
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
    let target_rtindex = match operation {
        SpireDmlFrontdoorOperation::Update | SpireDmlFrontdoorOperation::Delete => {
            query_ref.resultRelation
        }
        SpireDmlFrontdoorOperation::PkSelect => {
            // Keep unsupported SELECT shapes flowing into the shared classifier
            // so diagnostics report the same fail-closed status/kind matrix.
            unsafe { single_range_table_ref(query_ref) }.unwrap_or_default()
        }
    };
    let has_join = unsafe { dml_frontdoor_query_has_join_shape(query_ref, operation) };
    let predicate = unsafe { dml_frontdoor_pk_predicate(query_ref, target_rtindex, &context) };
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
        single_table: target_rtindex > 0 && !has_join,
        has_join,
        has_subquery: dml_frontdoor_query_has_subquery_shape(query_ref),
        has_returning: !query_ref.returningList.is_null(),
        pk_column: context.pk_column,
        predicate_column: predicate.column.as_deref(),
        predicate_operator: predicate.operator,
        predicate_value_kind: predicate.value.kind,
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

unsafe fn dml_frontdoor_query_has_join_shape(
    query: &pg_sys::Query,
    operation: SpireDmlFrontdoorOperation,
) -> bool {
    // SELECT v1 must have exactly one range-table ref. UPDATE/DELETE v1 allow
    // only the result relation in the jointree; FROM/USING relations make the
    // shape a join even though baserel handoff skips those non-target rels.
    match operation {
        SpireDmlFrontdoorOperation::PkSelect => unsafe { single_range_table_ref(query).is_none() },
        SpireDmlFrontdoorOperation::Update | SpireDmlFrontdoorOperation::Delete => unsafe {
            dml_frontdoor_dml_has_extra_from_shape(query)
        },
    }
}

unsafe fn dml_frontdoor_dml_has_extra_from_shape(query: &pg_sys::Query) -> bool {
    let Some(jointree) = (unsafe { query.jointree.as_ref() }) else {
        return false;
    };
    if jointree.fromlist.is_null() {
        return false;
    }
    let fromlist = unsafe { PgList::<pg_sys::Node>::from_pg(jointree.fromlist) };
    if fromlist.is_empty() {
        return false;
    }
    if fromlist.len() != 1 {
        return true;
    }
    let Some(from_node) = fromlist.get_ptr(0) else {
        return false;
    };
    if from_node.is_null() || unsafe { (*from_node).type_ } != pg_sys::NodeTag::T_RangeTblRef {
        return true;
    }
    let range_table_ref = from_node.cast::<pg_sys::RangeTblRef>();
    unsafe { (*range_table_ref).rtindex != query.resultRelation }
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
) -> SpireDmlFrontdoorPkPredicate {
    let Some(jointree) = (unsafe { query.jointree.as_ref() }) else {
        return dml_frontdoor_empty_pk_predicate();
    };
    let qual = jointree.quals;
    unsafe { dml_frontdoor_pk_predicate_from_clause(qual.cast(), target_rtindex, context) }
        .unwrap_or_else(dml_frontdoor_empty_pk_predicate)
}

unsafe fn dml_frontdoor_pk_predicate_from_baserestrictinfo(
    baserestrictinfo: *mut pg_sys::List,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> Option<SpireDmlFrontdoorPkPredicate> {
    if baserestrictinfo.is_null() {
        return None;
    }
    let restrict_infos = unsafe { PgList::<pg_sys::RestrictInfo>::from_pg(baserestrictinfo) };
    for restrict_info in restrict_infos.iter_ptr() {
        let Some(restrict_info) = (unsafe { restrict_info.as_ref() }) else {
            continue;
        };
        if let Some(predicate) = unsafe {
            dml_frontdoor_pk_predicate_from_clause(restrict_info.clause, target_rtindex, context)
        } {
            return Some(predicate);
        }
    }
    None
}

unsafe fn dml_frontdoor_pk_predicate_from_clause(
    clause: *mut pg_sys::Expr,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> Option<SpireDmlFrontdoorPkPredicate> {
    if clause.is_null() {
        return None;
    }
    if unsafe { (*clause.cast::<pg_sys::Node>()).type_ } != pg_sys::NodeTag::T_OpExpr {
        if unsafe {
            dml_frontdoor_expr_references_column(clause, target_rtindex, context, context.pk_column)
        } {
            return Some(SpireDmlFrontdoorPkPredicate {
                column: Some(context.pk_column.to_owned()),
                operator: Some("other"),
                value: dml_frontdoor_other_predicate_value(),
                value_expr: None,
            });
        }
        return None;
    }
    let op_expr = clause.cast::<pg_sys::OpExpr>();
    let operator =
        if dml_frontdoor_bigint_equality_opcode(unsafe { pg_sys::get_opcode((*op_expr).opno) }) {
            Some("=")
        } else {
            Some("other")
        };
    let args = unsafe { PgList::<pg_sys::Expr>::from_pg((*op_expr).args) };
    if args.len() != 2 {
        return Some(SpireDmlFrontdoorPkPredicate {
            column: None,
            operator,
            value: dml_frontdoor_other_predicate_value(),
            value_expr: None,
        });
    }
    let left = args.get_ptr(0);
    let right = args.get_ptr(1);
    match (left, right) {
        (Some(left), Some(right)) => {
            if let Some(column) =
                unsafe { dml_frontdoor_predicate_var_column(left, target_rtindex, context) }
            {
                return Some(SpireDmlFrontdoorPkPredicate {
                    column: Some(column),
                    operator,
                    value: unsafe { dml_frontdoor_predicate_value(right) },
                    value_expr: Some(right),
                });
            }
            if let Some(column) =
                unsafe { dml_frontdoor_predicate_var_column(right, target_rtindex, context) }
            {
                return Some(SpireDmlFrontdoorPkPredicate {
                    column: Some(column),
                    operator,
                    value: unsafe { dml_frontdoor_predicate_value(left) },
                    value_expr: Some(left),
                });
            }
            Some(SpireDmlFrontdoorPkPredicate {
                column: None,
                operator,
                value: dml_frontdoor_other_predicate_value(),
                value_expr: None,
            })
        }
        _ => Some(SpireDmlFrontdoorPkPredicate {
            column: None,
            operator,
            value: dml_frontdoor_other_predicate_value(),
            value_expr: None,
        }),
    }
}

unsafe fn dml_frontdoor_predicate_var_column(
    expr: *mut pg_sys::Expr,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> Option<String> {
    if let Some(column) = unsafe { dml_frontdoor_var_column(expr, target_rtindex, context) } {
        return Some(column);
    }
    if expr.is_null() {
        return None;
    }
    match unsafe { (*expr.cast::<pg_sys::Node>()).type_ } {
        pg_sys::NodeTag::T_RelabelType => {
            let relabel = expr.cast::<pg_sys::RelabelType>();
            unsafe { dml_frontdoor_predicate_var_column((*relabel).arg, target_rtindex, context) }
        }
        pg_sys::NodeTag::T_CoerceViaIO => {
            let coerce = expr.cast::<pg_sys::CoerceViaIO>();
            unsafe { dml_frontdoor_predicate_var_column((*coerce).arg, target_rtindex, context) }
        }
        pg_sys::NodeTag::T_FuncExpr => {
            let func_expr = expr.cast::<pg_sys::FuncExpr>();
            unsafe {
                dml_frontdoor_single_predicate_var_column(
                    (*func_expr).args,
                    target_rtindex,
                    context,
                )
            }
        }
        _ => None,
    }
}

unsafe fn dml_frontdoor_single_predicate_var_column(
    exprs: *mut pg_sys::List,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> Option<String> {
    if exprs.is_null() {
        return None;
    }
    let exprs = unsafe { PgList::<pg_sys::Expr>::from_pg(exprs) };
    if exprs.len() != 1 {
        return None;
    }
    let expr = exprs.get_ptr(0)?;
    unsafe { dml_frontdoor_predicate_var_column(expr, target_rtindex, context) }
}

unsafe fn dml_frontdoor_expr_references_column(
    expr: *mut pg_sys::Expr,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
    column_name: &str,
) -> bool {
    if expr.is_null() {
        return false;
    }
    match unsafe { (*expr.cast::<pg_sys::Node>()).type_ } {
        pg_sys::NodeTag::T_Var => unsafe {
            dml_frontdoor_var_column(expr, target_rtindex, context).as_deref() == Some(column_name)
        },
        pg_sys::NodeTag::T_OpExpr => {
            let op_expr = expr.cast::<pg_sys::OpExpr>();
            unsafe {
                dml_frontdoor_expr_list_references_column(
                    (*op_expr).args,
                    target_rtindex,
                    context,
                    column_name,
                )
            }
        }
        pg_sys::NodeTag::T_ScalarArrayOpExpr => {
            let array_expr = expr.cast::<pg_sys::ScalarArrayOpExpr>();
            unsafe {
                dml_frontdoor_expr_list_references_column(
                    (*array_expr).args,
                    target_rtindex,
                    context,
                    column_name,
                )
            }
        }
        pg_sys::NodeTag::T_BoolExpr => {
            let bool_expr = expr.cast::<pg_sys::BoolExpr>();
            unsafe {
                dml_frontdoor_expr_list_references_column(
                    (*bool_expr).args,
                    target_rtindex,
                    context,
                    column_name,
                )
            }
        }
        pg_sys::NodeTag::T_FuncExpr => {
            let func_expr = expr.cast::<pg_sys::FuncExpr>();
            unsafe {
                dml_frontdoor_expr_list_references_column(
                    (*func_expr).args,
                    target_rtindex,
                    context,
                    column_name,
                )
            }
        }
        pg_sys::NodeTag::T_RelabelType => {
            let relabel = expr.cast::<pg_sys::RelabelType>();
            unsafe {
                dml_frontdoor_expr_references_column(
                    (*relabel).arg,
                    target_rtindex,
                    context,
                    column_name,
                )
            }
        }
        pg_sys::NodeTag::T_CoerceViaIO => {
            let coerce = expr.cast::<pg_sys::CoerceViaIO>();
            unsafe {
                dml_frontdoor_expr_references_column(
                    (*coerce).arg,
                    target_rtindex,
                    context,
                    column_name,
                )
            }
        }
        _ => false,
    }
}

unsafe fn dml_frontdoor_expr_list_references_column(
    exprs: *mut pg_sys::List,
    target_rtindex: i32,
    context: &SpireDmlFrontdoorQueryContext<'_>,
    column_name: &str,
) -> bool {
    if exprs.is_null() {
        return false;
    }
    let exprs = unsafe { PgList::<pg_sys::Expr>::from_pg(exprs) };
    let references_column = exprs.iter_ptr().any(|expr| unsafe {
        dml_frontdoor_expr_references_column(expr, target_rtindex, context, column_name)
    });
    references_column
}

struct SpireDmlFrontdoorPkPredicate {
    column: Option<String>,
    operator: Option<&'static str>,
    value: SpireDmlFrontdoorPredicateValue,
    value_expr: Option<*mut pg_sys::Expr>,
}

fn dml_frontdoor_empty_pk_predicate() -> SpireDmlFrontdoorPkPredicate {
    SpireDmlFrontdoorPkPredicate {
        column: None,
        operator: None,
        value: dml_frontdoor_other_predicate_value(),
        value_expr: None,
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
    unsafe { dml_frontdoor_predicate_value(expr).kind }
}

unsafe fn dml_frontdoor_predicate_value(
    expr: *mut pg_sys::Expr,
) -> SpireDmlFrontdoorPredicateValue {
    unsafe { dml_frontdoor_predicate_value_inner(expr, 0) }
}

unsafe fn dml_frontdoor_predicate_value_inner(
    expr: *mut pg_sys::Expr,
    wrapper_depth: usize,
) -> SpireDmlFrontdoorPredicateValue {
    if expr.is_null() {
        return dml_frontdoor_other_predicate_value();
    }
    if wrapper_depth > DML_FRONTDOOR_MAX_COERCION_WRAPPER_DEPTH {
        return dml_frontdoor_other_predicate_value();
    }
    match unsafe { (*expr.cast::<pg_sys::Node>()).type_ } {
        pg_sys::NodeTag::T_Const => {
            let const_expr = unsafe { &*expr.cast::<pg_sys::Const>() };
            if !const_expr.constisnull
                && dml_frontdoor_integer_oid_can_coerce_to_bigint(const_expr.consttype)
            {
                SpireDmlFrontdoorPredicateValue {
                    kind: SpireDmlFrontdoorValueKind::ConstBigint,
                    const_bigint: unsafe { dml_frontdoor_const_bigint_value(const_expr) },
                    param_id: None,
                }
            } else {
                dml_frontdoor_other_predicate_value()
            }
        }
        pg_sys::NodeTag::T_Param => {
            let param = unsafe { &*expr.cast::<pg_sys::Param>() };
            if dml_frontdoor_integer_oid_can_coerce_to_bigint(param.paramtype) {
                SpireDmlFrontdoorPredicateValue {
                    kind: SpireDmlFrontdoorValueKind::ParamBigint,
                    const_bigint: None,
                    param_id: Some(param.paramid as i32),
                }
            } else {
                dml_frontdoor_other_predicate_value()
            }
        }
        pg_sys::NodeTag::T_FuncExpr => {
            let func_expr = unsafe { &*expr.cast::<pg_sys::FuncExpr>() };
            if func_expr.funcresulttype != pg_sys::INT8OID {
                return dml_frontdoor_other_predicate_value();
            }
            unsafe { dml_frontdoor_single_coerced_arg_value(func_expr.args, wrapper_depth) }
        }
        pg_sys::NodeTag::T_RelabelType => {
            let relabel = unsafe { &*expr.cast::<pg_sys::RelabelType>() };
            if relabel.resulttype != pg_sys::INT8OID {
                return dml_frontdoor_other_predicate_value();
            }
            unsafe { dml_frontdoor_coercible_integer_value(relabel.arg, wrapper_depth) }
        }
        pg_sys::NodeTag::T_CoerceViaIO => {
            let coerce = unsafe { &*expr.cast::<pg_sys::CoerceViaIO>() };
            if coerce.resulttype != pg_sys::INT8OID {
                return dml_frontdoor_other_predicate_value();
            }
            unsafe { dml_frontdoor_coercible_integer_value(coerce.arg, wrapper_depth) }
        }
        _ => dml_frontdoor_other_predicate_value(),
    }
}

unsafe fn dml_frontdoor_single_coerced_arg_value(
    args: *mut pg_sys::List,
    wrapper_depth: usize,
) -> SpireDmlFrontdoorPredicateValue {
    let Some(arg) = (unsafe { dml_frontdoor_single_list_expr_arg(args) }) else {
        return dml_frontdoor_other_predicate_value();
    };
    unsafe { dml_frontdoor_coercible_integer_value(arg, wrapper_depth) }
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

unsafe fn dml_frontdoor_coercible_integer_value(
    expr: *mut pg_sys::Expr,
    wrapper_depth: usize,
) -> SpireDmlFrontdoorPredicateValue {
    if expr.is_null() {
        return dml_frontdoor_other_predicate_value();
    }
    match unsafe { (*expr.cast::<pg_sys::Node>()).type_ } {
        pg_sys::NodeTag::T_Const => {
            let const_expr = unsafe { &*expr.cast::<pg_sys::Const>() };
            if !const_expr.constisnull
                && dml_frontdoor_integer_oid_can_coerce_to_bigint(const_expr.consttype)
            {
                SpireDmlFrontdoorPredicateValue {
                    kind: SpireDmlFrontdoorValueKind::ConstBigint,
                    const_bigint: unsafe { dml_frontdoor_const_bigint_value(const_expr) },
                    param_id: None,
                }
            } else {
                dml_frontdoor_other_predicate_value()
            }
        }
        pg_sys::NodeTag::T_Param => {
            let param = unsafe { &*expr.cast::<pg_sys::Param>() };
            if dml_frontdoor_integer_oid_can_coerce_to_bigint(param.paramtype) {
                SpireDmlFrontdoorPredicateValue {
                    kind: SpireDmlFrontdoorValueKind::ParamBigint,
                    const_bigint: None,
                    param_id: Some(param.paramid as i32),
                }
            } else {
                dml_frontdoor_other_predicate_value()
            }
        }
        _ => unsafe { dml_frontdoor_predicate_value_inner(expr, wrapper_depth + 1) },
    }
}

fn dml_frontdoor_other_predicate_value() -> SpireDmlFrontdoorPredicateValue {
    SpireDmlFrontdoorPredicateValue {
        kind: SpireDmlFrontdoorValueKind::Other,
        const_bigint: None,
        param_id: None,
    }
}

unsafe fn dml_frontdoor_const_bigint_value(const_expr: &pg_sys::Const) -> Option<i64> {
    unsafe {
        if const_expr.constisnull {
            return None;
        }
        if const_expr.consttype == pg_sys::INT8OID {
            i64::from_datum(const_expr.constvalue, false)
        } else if const_expr.consttype == pg_sys::INT4OID {
            i32::from_datum(const_expr.constvalue, false).map(i64::from)
        } else if const_expr.consttype == pg_sys::INT2OID {
            i16::from_datum(const_expr.constvalue, false).map(i64::from)
        } else {
            None
        }
    }
}

fn dml_frontdoor_value_kind_name(kind: SpireDmlFrontdoorValueKind) -> &'static str {
    match kind {
        SpireDmlFrontdoorValueKind::ConstBigint => "const_bigint",
        SpireDmlFrontdoorValueKind::ParamBigint => "param_bigint",
        SpireDmlFrontdoorValueKind::Other => "other",
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
    unsafe {
        dml_frontdoor_target_column_exprs(target_list, context)
            .into_iter()
            .map(|(column, _expr)| column)
            .collect()
    }
}

unsafe fn dml_frontdoor_target_column_exprs(
    target_list: *mut pg_sys::List,
    context: &SpireDmlFrontdoorQueryContext<'_>,
) -> Vec<(String, *mut pg_sys::Expr)> {
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
            columns.push((column, target_entry.expr));
            continue;
        }
        if !target_entry.resname.is_null() {
            if let Ok(column) = unsafe { CStr::from_ptr(target_entry.resname) }.to_str() {
                columns.push((column.to_owned(), target_entry.expr));
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

include!("tests.rs");
