use pgrx::{pg_guard, pg_sys, FromDatum, PgBox, PgList, Spi};

use std::{ffi::CString, ptr};

use crate::am::common::cost::{current_planner_cost_constants, PlannerCostConstants};

use super::meta;

const CUSTOM_SCAN_NAME: &core::ffi::CStr = c"EcSpireDistributedScan";
const EC_SPIRE_AM_NAME: &core::ffi::CStr = c"ec_spire";
const CUSTOM_SCAN_ROUTING_SCORE_BOUND: f64 = 64.0;
const CUSTOM_SCAN_REMOTE_DISPATCH_CPU_UNITS: f64 = 32.0;
const CUSTOM_SCAN_MERGE_CPU_UNITS: f64 = 4.0;
const CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT: u32 = 1;
const CUSTOM_SCAN_PLAN_MODE_DML_PK_SELECT: u32 = 2;
const CUSTOM_SCAN_PLAN_MODE_DML_UPDATE: u32 = 3;
const CUSTOM_SCAN_PLAN_MODE_DML_DELETE: u32 = 4;
const DML_UPDATED_COLUMN_COUNT_OFFSET: i32 = 2;
const EC_SPIRE_PLACEMENT_INDEX_OID_ATTNO: pg_sys::AttrNumber = 1;
const EC_SPIRE_PLACEMENT_RELNAME: &core::ffi::CStr = c"ec_spire_placement";
const EC_SPIRE_PLACEMENT_BY_INDEX_OID_RELNAME: &core::ffi::CStr =
    c"ec_spire_placement_by_index_oid";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpireCustomScanPlanMode {
    VectorOrderLimit,
    DmlPkSelectTuplePayload,
    DmlUpdateTuplePayload,
    DmlDeleteTuplePayload,
}

impl SpireCustomScanPlanMode {
    fn is_dml(self) -> bool {
        matches!(
            self,
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload
                | SpireCustomScanPlanMode::DmlUpdateTuplePayload
                | SpireCustomScanPlanMode::DmlDeleteTuplePayload
        )
    }

    fn raw(self) -> u32 {
        match self {
            SpireCustomScanPlanMode::VectorOrderLimit => CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT,
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload => CUSTOM_SCAN_PLAN_MODE_DML_PK_SELECT,
            SpireCustomScanPlanMode::DmlUpdateTuplePayload => CUSTOM_SCAN_PLAN_MODE_DML_UPDATE,
            SpireCustomScanPlanMode::DmlDeleteTuplePayload => CUSTOM_SCAN_PLAN_MODE_DML_DELETE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireCustomScanStatusRow {
    pub(crate) provider_name: &'static str,
    pub(crate) registered: bool,
    pub(crate) rel_pathlist_hook_installed: bool,
    pub(crate) path_generation_enabled: bool,
    pub(crate) exec_wiring_enabled: bool,
    pub(crate) status: &'static str,
    pub(crate) next_step: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SpireCustomScanIndexEligibilityRow {
    pub(crate) active_epoch: u64,
    pub(crate) local_placement_count: u64,
    // Total remote nodes represented by active-epoch placements, including
    // currently unavailable placements; historical epochs are filtered before
    // this count is computed.
    pub(crate) remote_node_count: u64,
    // Planner-relevant subset of remote nodes with at least one available
    // placement.
    pub(crate) remote_available_node_count: u64,
    pub(crate) remote_placement_count: u64,
    pub(crate) remote_available_placement_count: u64,
    pub(crate) remote_unavailable_placement_count: u64,
    pub(crate) all_remote_placements_available: bool,
    pub(crate) eligible_for_custom_scan: bool,
    pub(crate) status: &'static str,
    pub(crate) next_step: &'static str,
}

static mut PREVIOUS_SET_REL_PATHLIST_HOOK: pg_sys::set_rel_pathlist_hook_type = None;
static mut CUSTOM_SCAN_REGISTERED: bool = false;
static mut REL_PATHLIST_HOOK_INSTALLED: bool = false;

// Intentionally inert until the planner path-generation slice builds CustomPath nodes.
static mut CUSTOM_PATH_METHODS: pg_sys::CustomPathMethods = pg_sys::CustomPathMethods {
    CustomName: CUSTOM_SCAN_NAME.as_ptr(),
    PlanCustomPath: Some(ec_spire_plan_custom_path),
    ReparameterizeCustomPathByChild: None,
};

static mut CUSTOM_SCAN_METHODS: pg_sys::CustomScanMethods = pg_sys::CustomScanMethods {
    CustomName: CUSTOM_SCAN_NAME.as_ptr(),
    CreateCustomScanState: Some(ec_spire_create_custom_scan_state),
};

static mut CUSTOM_EXEC_METHODS: pg_sys::CustomExecMethods = pg_sys::CustomExecMethods {
    CustomName: CUSTOM_SCAN_NAME.as_ptr(),
    BeginCustomScan: Some(ec_spire_begin_custom_scan),
    ExecCustomScan: Some(ec_spire_exec_custom_scan),
    EndCustomScan: Some(ec_spire_end_custom_scan),
    ReScanCustomScan: Some(ec_spire_rescan_custom_scan),
    MarkPosCustomScan: None,
    RestrPosCustomScan: None,
    EstimateDSMCustomScan: None,
    InitializeDSMCustomScan: None,
    ReInitializeDSMCustomScan: None,
    InitializeWorkerCustomScan: None,
    ShutdownCustomScan: None,
    ExplainCustomScan: None,
};

#[repr(C)]
struct SpireCustomScanExecState {
    custom_scan_state: pg_sys::CustomScanState,
    mode: SpireCustomScanPlanMode,
    index_oid: pg_sys::Oid,
    top_k: usize,
    query: Vec<f32>,
    dml_pk_column: String,
    dml_pk_value: [u8; 8],
    dml_updated_columns: Vec<String>,
    dml_projected_columns: Vec<String>,
    dml_update_value_exprs: Vec<*mut pg_sys::Expr>,
    tuple_payload_columns: Vec<String>,
    tuple_payload_inputs: Vec<Option<SpireCustomScanPayloadAttrInput>>,
    outputs: Vec<super::SpireRemoteProductionScanOutputRow>,
    next_output: usize,
    loaded_outputs: bool,
    dml_payload_loaded: bool,
    dml_payload_emitted: bool,
    dml_tuple_payload_json: Option<String>,
}

struct SpireCustomScanPayloadAttrInput {
    flinfo: pg_sys::FmgrInfo,
    typioparam: pg_sys::Oid,
    typmod: i32,
}

pub(crate) unsafe fn register_custom_scan() {
    unsafe {
        if !CUSTOM_SCAN_REGISTERED {
            pg_sys::RegisterCustomScanMethods(&raw const CUSTOM_SCAN_METHODS);
            CUSTOM_SCAN_REGISTERED = true;
        }

        if !REL_PATHLIST_HOOK_INSTALLED {
            PREVIOUS_SET_REL_PATHLIST_HOOK = pg_sys::set_rel_pathlist_hook;
            pg_sys::set_rel_pathlist_hook = Some(ec_spire_set_rel_pathlist_hook);
            REL_PATHLIST_HOOK_INSTALLED = true;
        }
    }
}

pub(crate) fn custom_scan_status_row() -> SpireCustomScanStatusRow {
    let (registered, hook_installed) =
        unsafe { (CUSTOM_SCAN_REGISTERED, REL_PATHLIST_HOOK_INSTALLED) };
    SpireCustomScanStatusRow {
        provider_name: "EcSpireDistributedScan",
        registered,
        rel_pathlist_hook_installed: hook_installed,
        path_generation_enabled: true,
        exec_wiring_enabled: true,
        status: if registered && hook_installed {
            "executor_stream_wired_tuple_payload_slots"
        } else {
            "not_registered"
        },
        next_step: "add ADR-069 write path",
    }
}

pub(crate) unsafe fn custom_scan_index_eligibility_row(
    index_relation: pg_sys::Relation,
) -> SpireCustomScanIndexEligibilityRow {
    unsafe {
        custom_scan_index_eligibility_result(index_relation).unwrap_or_else(|e| pgrx::error!("{e}"))
    }
}

unsafe fn custom_scan_index_eligibility_result(
    index_relation: pg_sys::Relation,
) -> Result<SpireCustomScanIndexEligibilityRow, String> {
    let root_control = unsafe { super::page::read_root_control_page(index_relation) };
    if root_control.active_epoch == 0 {
        return Ok(SpireCustomScanIndexEligibilityRow {
            active_epoch: 0,
            local_placement_count: 0,
            remote_node_count: 0,
            remote_available_node_count: 0,
            remote_placement_count: 0,
            remote_available_placement_count: 0,
            remote_unavailable_placement_count: 0,
            all_remote_placements_available: false,
            eligible_for_custom_scan: false,
            status: "no_active_epoch",
            next_step: "keep local-only ec_spire index AM path",
        });
    }

    let placement_directory =
        unsafe { load_custom_scan_placement_directory(index_relation, root_control)? };
    let active_epoch = root_control.active_epoch;
    let mut local_placement_count = 0_u64;
    let mut remote_placement_count = 0_u64;
    let mut remote_available_placement_count = 0_u64;
    let mut remote_unavailable_placement_count = 0_u64;
    let mut remote_node_ids = std::collections::BTreeSet::new();
    let mut remote_available_node_ids = std::collections::BTreeSet::new();

    for placement in placement_directory.entries {
        if placement.node_id == meta::SPIRE_LOCAL_NODE_ID {
            local_placement_count = local_placement_count.saturating_add(1);
        } else {
            remote_node_ids.insert(placement.node_id);
            remote_placement_count = remote_placement_count.saturating_add(1);
            if placement.state == meta::SpirePlacementState::Available {
                remote_available_placement_count =
                    remote_available_placement_count.saturating_add(1);
                remote_available_node_ids.insert(placement.node_id);
            } else {
                remote_unavailable_placement_count =
                    remote_unavailable_placement_count.saturating_add(1);
            }
        }
    }

    let eligible = active_epoch != 0 && remote_available_placement_count > 0;
    let all_remote_placements_available =
        remote_placement_count > 0 && remote_unavailable_placement_count == 0;
    Ok(SpireCustomScanIndexEligibilityRow {
        active_epoch,
        local_placement_count,
        remote_node_count: remote_node_ids.len() as u64,
        remote_available_node_count: remote_available_node_ids.len() as u64,
        remote_placement_count,
        remote_available_placement_count,
        remote_unavailable_placement_count,
        all_remote_placements_available,
        eligible_for_custom_scan: eligible,
        status: if eligible {
            "customscan_candidate"
        } else if active_epoch == 0 {
            "no_active_epoch"
        } else if remote_placement_count == 0 {
            "local_only"
        } else {
            "no_available_remote_placements"
        },
        next_step: if eligible {
            "planner path generation must also verify ORDER BY vector distance LIMIT query shape"
        } else {
            "keep local-only ec_spire index AM path"
        },
    })
}

unsafe fn load_custom_scan_placement_directory(
    index_relation: pg_sys::Relation,
    root_control: meta::SpireRootControlState,
) -> Result<meta::SpirePlacementDirectory, String> {
    // The SQL eligibility wrapper normally returns `no_active_epoch` before
    // this helper is called. Keep the helper fail-closed so future callers
    // cannot accidentally dereference an empty placement-directory TID.
    if root_control.active_epoch == 0 {
        return Err("ec_spire cannot load placement directory for empty active epoch".to_owned());
    }

    // ADR-067 planner eligibility needs only placement availability. Avoid the
    // heavier fanout loader used by executor paths, which also decodes epoch
    // and object manifests; executor paths remain responsible for full
    // identity and manifest validation before result-stream merge.
    let placement_bytes = unsafe {
        super::page::read_object_tuple(index_relation, root_control.placement_directory_tid)?
    };
    let placement_directory = meta::SpirePlacementDirectory::decode(&placement_bytes)?;
    if placement_directory.epoch != root_control.active_epoch {
        return Err(format!(
            "ec_spire root/control active epoch {} does not match placement directory {}",
            root_control.active_epoch, placement_directory.epoch
        ));
    }
    Ok(placement_directory)
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_set_rel_pathlist_hook(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rti: pg_sys::Index,
    rte: *mut pg_sys::RangeTblEntry,
) {
    unsafe {
        if let Some(previous_hook) = PREVIOUS_SET_REL_PATHLIST_HOOK {
            previous_hook(root, rel, rti, rte);
        }
    }
    if let Some((index_oid, eligibility)) =
        unsafe { custom_scan_candidate_index_oid(root, rel, rte) }
    {
        unsafe { add_custom_scan_path(root, rel, index_oid, eligibility) };
    }
    if let Some(index_oid) = unsafe { dml_pk_select_candidate_index_oid(root, rel, rte) } {
        unsafe { add_dml_pk_select_custom_scan_path(root, rel, index_oid) };
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_plan_custom_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    best_path: *mut pg_sys::CustomPath,
    tlist: *mut pg_sys::List,
    clauses: *mut pg_sys::List,
    custom_plans: *mut pg_sys::List,
) -> *mut pg_sys::Plan {
    unsafe {
        let mode = custom_scan_mode_from_path(best_path).unwrap_or_else(|| {
            pgrx::error!("EcSpireDistributedScan CustomPath is missing plan mode")
        });
        if mode.is_dml() {
            return plan_dml_custom_path(root, rel, best_path, tlist, clauses, custom_plans, mode);
        }

        let top_k = custom_scan_top_k(root).unwrap_or(1);
        let query_expr = custom_scan_orderby_query_expr(root, rel).unwrap_or_else(|| {
            pgrx::error!(
                "EcSpireDistributedScan could not extract ORDER BY vector query expression"
            )
        });
        let custom_exprs = pg_sys::lappend(
            std::ptr::null_mut(),
            pg_sys::copyObjectImpl(query_expr.cast()).cast(),
        );

        let mut custom_scan =
            PgBox::<pg_sys::CustomScan>::alloc_node(pg_sys::NodeTag::T_CustomScan);
        custom_scan.scan.plan.type_ = pg_sys::NodeTag::T_CustomScan;
        custom_scan.scan.plan.disabled_nodes = (*best_path).path.disabled_nodes;
        custom_scan.scan.plan.startup_cost = (*best_path).path.startup_cost;
        custom_scan.scan.plan.total_cost = (*best_path).path.total_cost;
        custom_scan.scan.plan.plan_rows = (*best_path).path.rows;
        custom_scan.scan.plan.plan_width = if !(*best_path).path.pathtarget.is_null() {
            (*(*best_path).path.pathtarget).width
        } else {
            0
        };
        custom_scan.scan.plan.parallel_aware = false;
        custom_scan.scan.plan.parallel_safe = false;
        custom_scan.scan.plan.async_capable = false;
        custom_scan.scan.plan.targetlist = tlist;
        custom_scan.scan.plan.qual = pg_sys::extract_actual_clauses(clauses, false);
        custom_scan.scan.scanrelid = (*(*best_path).path.parent).relid;
        custom_scan.flags = (*best_path).flags;
        custom_scan.custom_plans = custom_plans;
        custom_scan.custom_exprs = custom_exprs;
        custom_scan.custom_private = pg_sys::lappend_oid(
            pg_sys::lappend_oid(
                pg_sys::lappend_oid(
                    std::ptr::null_mut(),
                    pg_sys::Oid::from(CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT),
                ),
                custom_scan_index_oid_from_path(best_path),
            ),
            pg_sys::Oid::from(u32::try_from(top_k).unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan LIMIT exceeds CustomScan plan-private range")
            })),
        );
        custom_scan.custom_scan_tlist = std::ptr::null_mut();
        custom_scan.custom_relids = std::ptr::null_mut();
        custom_scan.methods = &raw const CUSTOM_SCAN_METHODS;
        custom_scan.into_pg() as *mut pg_sys::Plan
    }
}

unsafe fn plan_dml_custom_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    best_path: *mut pg_sys::CustomPath,
    tlist: *mut pg_sys::List,
    clauses: *mut pg_sys::List,
    custom_plans: *mut pg_sys::List,
    mode: SpireCustomScanPlanMode,
) -> *mut pg_sys::Plan {
    unsafe {
        let plan_expr = match super::dml_frontdoor_primitive_plan_expr_from_baserel(root, rel)
            .unwrap_or_else(|| {
                pgrx::error!("EcSpireDistributedScan could not build DML expression handoff")
            }) {
            Ok(plan_expr) => plan_expr,
            Err(err) => pgrx::error!("{err}"),
        };
        let plan_mode = custom_scan_plan_mode_for_dml_mode(plan_expr.primitive_plan.mode);
        if plan_mode != mode {
            pgrx::error!(
                "EcSpireDistributedScan DML plan mode {:?} does not match primitive mode {:?}",
                mode,
                plan_expr.primitive_plan.mode
            )
        }
        let custom_exprs = custom_scan_dml_custom_exprs_from_plan_expr(&plan_expr);
        let mut custom_scan =
            PgBox::<pg_sys::CustomScan>::alloc_node(pg_sys::NodeTag::T_CustomScan);
        custom_scan.scan.plan.type_ = pg_sys::NodeTag::T_CustomScan;
        custom_scan.scan.plan.disabled_nodes = (*best_path).path.disabled_nodes;
        custom_scan.scan.plan.startup_cost = (*best_path).path.startup_cost;
        custom_scan.scan.plan.total_cost = (*best_path).path.total_cost;
        custom_scan.scan.plan.plan_rows = (*best_path).path.rows;
        custom_scan.scan.plan.plan_width = if !(*best_path).path.pathtarget.is_null() {
            (*(*best_path).path.pathtarget).width
        } else {
            0
        };
        custom_scan.scan.plan.parallel_aware = false;
        custom_scan.scan.plan.parallel_safe = false;
        custom_scan.scan.plan.async_capable = false;
        custom_scan.scan.plan.targetlist = tlist;
        custom_scan.scan.plan.qual = pg_sys::extract_actual_clauses(clauses, false);
        custom_scan.scan.scanrelid = (*(*best_path).path.parent).relid;
        custom_scan.flags = (*best_path).flags;
        custom_scan.custom_plans = custom_plans;
        custom_scan.custom_exprs = custom_exprs;
        custom_scan.custom_private = custom_scan_dml_plan_private(
            mode,
            custom_scan_index_oid_from_path(best_path),
            &plan_expr.primitive_plan.pk_argument.pk_column,
            &plan_expr.primitive_plan.updated_columns,
            &plan_expr.primitive_plan.projected_columns,
        );
        custom_scan.custom_scan_tlist = std::ptr::null_mut();
        custom_scan.custom_relids = std::ptr::null_mut();
        custom_scan.methods = &raw const CUSTOM_SCAN_METHODS;
        custom_scan.into_pg() as *mut pg_sys::Plan
    }
}

unsafe fn custom_scan_dml_custom_exprs_from_plan_expr(
    plan_expr: &super::dml_frontdoor::SpireDmlFrontdoorPrimitivePlanExpr,
) -> *mut pg_sys::List {
    unsafe {
        let mut custom_exprs = pg_sys::lappend(
            std::ptr::null_mut(),
            pg_sys::copyObjectImpl(plan_expr.pk_value_expr.cast()).cast(),
        );
        for expr in &plan_expr.updated_value_exprs {
            custom_exprs =
                pg_sys::lappend(custom_exprs, pg_sys::copyObjectImpl((*expr).cast()).cast());
        }
        custom_exprs
    }
}

pub(crate) unsafe fn custom_scan_dml_replacement_plan(
    plan_expr: super::dml_frontdoor::SpireDmlFrontdoorPrimitivePlanExpr,
    fallback_plan: *mut pg_sys::Plan,
) -> *mut pg_sys::Plan {
    unsafe {
        let mode = custom_scan_plan_mode_for_dml_mode(plan_expr.primitive_plan.mode);
        let custom_exprs = custom_scan_dml_custom_exprs_from_plan_expr(&plan_expr);
        let mut custom_scan =
            PgBox::<pg_sys::CustomScan>::alloc_node(pg_sys::NodeTag::T_CustomScan);
        custom_scan.scan.plan.type_ = pg_sys::NodeTag::T_CustomScan;
        // This replacement is not competing in path selection; the planner
        // has already produced fallback_plan. Copy its cost fields so EXPLAIN
        // remains roughly comparable until DML-specific costing exists.
        custom_scan.scan.plan.disabled_nodes = if fallback_plan.is_null() {
            0
        } else {
            (*fallback_plan).disabled_nodes
        };
        custom_scan.scan.plan.startup_cost = if fallback_plan.is_null() {
            0.0
        } else {
            (*fallback_plan).startup_cost
        };
        custom_scan.scan.plan.total_cost = if fallback_plan.is_null() {
            0.0
        } else {
            (*fallback_plan).total_cost
        };
        custom_scan.scan.plan.plan_rows = 0.0;
        custom_scan.scan.plan.plan_width = 0;
        custom_scan.scan.plan.parallel_aware = false;
        custom_scan.scan.plan.parallel_safe = false;
        custom_scan.scan.plan.async_capable = false;
        custom_scan.scan.plan.targetlist = std::ptr::null_mut();
        custom_scan.scan.plan.qual = std::ptr::null_mut();
        custom_scan.scan.scanrelid = 0;
        custom_scan.flags = 0;
        custom_scan.custom_plans = std::ptr::null_mut();
        custom_scan.custom_exprs = custom_exprs;
        custom_scan.custom_private = custom_scan_dml_plan_private(
            mode,
            plan_expr.primitive_plan.index_oid,
            &plan_expr.primitive_plan.pk_argument.pk_column,
            &plan_expr.primitive_plan.updated_columns,
            &plan_expr.primitive_plan.projected_columns,
        );
        custom_scan.custom_scan_tlist = std::ptr::null_mut();
        custom_scan.custom_relids = std::ptr::null_mut();
        custom_scan.methods = &raw const CUSTOM_SCAN_METHODS;
        custom_scan.into_pg() as *mut pg_sys::Plan
    }
}

unsafe fn custom_scan_candidate_index_oid(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rte: *mut pg_sys::RangeTblEntry,
) -> Option<(pg_sys::Oid, SpireCustomScanIndexEligibilityRow)> {
    if root.is_null() || rel.is_null() || rte.is_null() {
        return None;
    }
    let rel_ref = unsafe { rel.as_ref()? };
    if rel_ref.reloptkind != pg_sys::RelOptKind::RELOPT_BASEREL {
        return None;
    }
    if rel_ref.rtekind != pg_sys::RTEKind::RTE_RELATION {
        return None;
    }
    let root_ref = unsafe { root.as_ref()? };
    if root_ref.sort_pathkeys.is_null() || root_ref.limit_tuples < 0.0 {
        return None;
    }
    let _ = unsafe { custom_scan_orderby_query_expr(root, rel)? };

    let ec_spire_am_oid = unsafe { pg_sys::get_index_am_oid(EC_SPIRE_AM_NAME.as_ptr(), true) };
    if ec_spire_am_oid == pg_sys::InvalidOid {
        return None;
    }

    let index_list = unsafe { PgList::<pg_sys::IndexOptInfo>::from_pg(rel_ref.indexlist) };
    for index_info in index_list.iter_ptr() {
        let Some(index_info) = (unsafe { index_info.as_ref() }) else {
            continue;
        };
        if index_info.relam != ec_spire_am_oid {
            continue;
        }
        let index_relation = unsafe {
            pg_sys::index_open(
                index_info.indexoid,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )
        };
        let eligibility = unsafe { custom_scan_index_eligibility_result(index_relation) };
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        }
        if let Ok(row) = eligibility {
            if row.eligible_for_custom_scan {
                return Some((index_info.indexoid, row));
            }
        }
    }
    None
}

unsafe fn dml_pk_select_candidate_index_oid(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rte: *mut pg_sys::RangeTblEntry,
) -> Option<pg_sys::Oid> {
    if root.is_null() || rel.is_null() || rte.is_null() {
        return None;
    }
    let rel_ref = unsafe { rel.as_ref()? };
    if rel_ref.reloptkind != pg_sys::RelOptKind::RELOPT_BASEREL {
        return None;
    }
    if rel_ref.rtekind != pg_sys::RTEKind::RTE_RELATION {
        return None;
    }
    let ec_spire_am_oid = unsafe { pg_sys::get_index_am_oid(EC_SPIRE_AM_NAME.as_ptr(), true) };
    if ec_spire_am_oid == pg_sys::InvalidOid {
        return None;
    }

    let index_list = unsafe { PgList::<pg_sys::IndexOptInfo>::from_pg(rel_ref.indexlist) };
    let mut placement_index_oid = None;
    for index_info in index_list.iter_ptr() {
        let Some(index_info) = (unsafe { index_info.as_ref() }) else {
            continue;
        };
        if index_info.relam == ec_spire_am_oid
            && unsafe { custom_scan_index_has_sql_placement(index_info.indexoid) }
        {
            placement_index_oid = Some(index_info.indexoid);
            break;
        }
    }
    let placement_index_oid = placement_index_oid?;
    let plan_expr = match unsafe {
        super::dml_frontdoor_pk_select_primitive_plan_expr_from_baserel(root, rel)?
    } {
        Ok(plan_expr) => plan_expr,
        Err(_err) => return None,
    };
    if plan_expr.primitive_plan.mode
        != super::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload
    {
        return None;
    }
    (plan_expr.primitive_plan.index_oid == placement_index_oid)
        .then_some(plan_expr.primitive_plan.index_oid)
}

unsafe fn custom_scan_index_has_sql_placement(index_oid: pg_sys::Oid) -> bool {
    unsafe {
        let placement_oid = pg_sys::RelnameGetRelid(EC_SPIRE_PLACEMENT_RELNAME.as_ptr());
        let placement_by_index_oid =
            pg_sys::RelnameGetRelid(EC_SPIRE_PLACEMENT_BY_INDEX_OID_RELNAME.as_ptr());
        if placement_oid == pg_sys::InvalidOid {
            return false;
        }
        if placement_by_index_oid == pg_sys::InvalidOid {
            return false;
        }
        let placement_relation =
            pg_sys::table_open(placement_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        if placement_relation.is_null() {
            return false;
        }
        let placement_index = pg_sys::index_open(
            placement_by_index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        );
        if placement_index.is_null() {
            pg_sys::table_close(
                placement_relation,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            );
            return false;
        }

        let mut scan_key = std::mem::MaybeUninit::<pg_sys::ScanKeyData>::zeroed().assume_init();
        pg_sys::ScanKeyInit(
            &mut scan_key,
            EC_SPIRE_PLACEMENT_INDEX_OID_ATTNO,
            pg_sys::BTEqualStrategyNumber as pg_sys::StrategyNumber,
            pg_sys::F_OIDEQ.into(),
            index_oid.into(),
        );
        let snapshot = pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot());
        if snapshot.is_null() {
            pg_sys::index_close(placement_index, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
            pg_sys::table_close(
                placement_relation,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            );
            return false;
        }
        pg_sys::PushActiveSnapshot(snapshot);
        #[cfg(feature = "pg18")]
        let scan = pg_sys::index_beginscan(
            placement_relation,
            placement_index,
            snapshot,
            ptr::null_mut(),
            1,
            0,
        );
        #[cfg(not(feature = "pg18"))]
        let scan = pg_sys::index_beginscan(placement_relation, placement_index, snapshot, 1, 0);
        let slot = pg_sys::table_slot_create(placement_relation, std::ptr::null_mut());
        let found = if scan.is_null() || slot.is_null() {
            false
        } else {
            pg_sys::index_rescan(scan, &mut scan_key, 1, ptr::null_mut(), 0);
            pg_sys::index_getnext_slot(scan, pg_sys::ScanDirection::ForwardScanDirection, slot)
        };
        if !scan.is_null() {
            pg_sys::index_endscan(scan);
        }
        if !slot.is_null() {
            pg_sys::ExecDropSingleTupleTableSlot(slot);
        }
        pg_sys::PopActiveSnapshot();
        pg_sys::UnregisterSnapshot(snapshot);
        pg_sys::index_close(placement_index, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        pg_sys::table_close(
            placement_relation,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        );
        found
    }
}

unsafe fn add_custom_scan_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    index_oid: pg_sys::Oid,
    eligibility: SpireCustomScanIndexEligibilityRow,
) {
    if root.is_null() || rel.is_null() {
        return;
    }
    let root_ref = unsafe { root.as_ref().expect("checked root pointer") };
    let rel_ref = unsafe { rel.as_ref().expect("checked rel pointer") };
    let mut custom_path =
        unsafe { PgBox::<pg_sys::CustomPath>::alloc_node(pg_sys::NodeTag::T_CustomPath) };
    let rows = if root_ref.limit_tuples >= 0.0 {
        root_ref.limit_tuples.max(1.0)
    } else {
        rel_ref.rows.max(1.0)
    };
    let cost = unsafe { estimate_custom_scan_cost(rows, rel_ref.rows.max(1.0), &eligibility) };
    custom_path.path.type_ = pg_sys::NodeTag::T_CustomPath;
    custom_path.path.pathtype = pg_sys::NodeTag::T_CustomScan;
    custom_path.path.parent = rel;
    custom_path.path.pathtarget = rel_ref.reltarget;
    custom_path.path.param_info = std::ptr::null_mut();
    custom_path.path.parallel_aware = false;
    custom_path.path.parallel_safe = false;
    custom_path.path.parallel_workers = 0;
    custom_path.path.rows = rows;
    custom_path.path.disabled_nodes = 0;
    custom_path.path.startup_cost = cost.startup_cost;
    custom_path.path.total_cost = cost.total_cost;
    custom_path.path.pathkeys = root_ref.sort_pathkeys;
    custom_path.flags = pg_sys::CUSTOMPATH_SUPPORT_PROJECTION;
    custom_path.custom_paths = std::ptr::null_mut();
    custom_path.custom_restrictinfo = rel_ref.baserestrictinfo;
    custom_path.custom_private = unsafe {
        pg_sys::lappend_oid(
            pg_sys::lappend_oid(
                std::ptr::null_mut(),
                pg_sys::Oid::from(CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT),
            ),
            index_oid,
        )
    };
    custom_path.methods = &raw const CUSTOM_PATH_METHODS;

    unsafe { pg_sys::add_path(rel, custom_path.into_pg() as *mut pg_sys::Path) };
}

unsafe fn add_dml_pk_select_custom_scan_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    index_oid: pg_sys::Oid,
) {
    if root.is_null() || rel.is_null() {
        return;
    }
    let rel_ref = unsafe { rel.as_ref().expect("checked rel pointer") };
    let mut custom_path =
        unsafe { PgBox::<pg_sys::CustomPath>::alloc_node(pg_sys::NodeTag::T_CustomPath) };
    custom_path.path.type_ = pg_sys::NodeTag::T_CustomPath;
    custom_path.path.pathtype = pg_sys::NodeTag::T_CustomScan;
    custom_path.path.parent = rel;
    custom_path.path.pathtarget = rel_ref.reltarget;
    custom_path.path.param_info = std::ptr::null_mut();
    custom_path.path.parallel_aware = false;
    custom_path.path.parallel_safe = false;
    custom_path.path.parallel_workers = 0;
    custom_path.path.rows = 1.0;
    custom_path.path.disabled_nodes = 0;
    custom_path.path.startup_cost = -1.0;
    custom_path.path.total_cost = -1.0;
    custom_path.path.pathkeys = std::ptr::null_mut();
    custom_path.flags = pg_sys::CUSTOMPATH_SUPPORT_PROJECTION;
    custom_path.custom_paths = std::ptr::null_mut();
    custom_path.custom_restrictinfo = rel_ref.baserestrictinfo;
    custom_path.custom_private = unsafe {
        pg_sys::lappend_oid(
            pg_sys::lappend_oid(
                std::ptr::null_mut(),
                pg_sys::Oid::from(CUSTOM_SCAN_PLAN_MODE_DML_PK_SELECT),
            ),
            index_oid,
        )
    };
    custom_path.methods = &raw const CUSTOM_PATH_METHODS;

    unsafe { pg_sys::add_path(rel, custom_path.into_pg() as *mut pg_sys::Path) };
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpireCustomScanCostEstimate {
    startup_cost: f64,
    total_cost: f64,
}

unsafe fn estimate_custom_scan_cost(
    output_rows: f64,
    rel_rows: f64,
    eligibility: &SpireCustomScanIndexEligibilityRow,
) -> SpireCustomScanCostEstimate {
    let constants = unsafe { current_planner_cost_constants() };
    let cpu_tuple_cost = unsafe { pg_sys::cpu_tuple_cost };
    estimate_custom_scan_cost_with_constants(
        output_rows,
        rel_rows,
        eligibility,
        constants,
        cpu_tuple_cost,
    )
}

fn estimate_custom_scan_cost_with_constants(
    output_rows: f64,
    rel_rows: f64,
    eligibility: &SpireCustomScanIndexEligibilityRow,
    constants: PlannerCostConstants,
    cpu_tuple_cost: f64,
) -> SpireCustomScanCostEstimate {
    let output_rows = output_rows.max(1.0);
    let rel_rows = rel_rows.max(output_rows);
    let fanout = eligibility.remote_available_node_count.max(1) as f64;
    let remote_placements = eligibility.remote_available_placement_count.max(1) as f64;
    let routing_scores = remote_placements.min(CUSTOM_SCAN_ROUTING_SCORE_BOUND);
    let routing_traversal_cost = routing_scores * constants.cpu_operator_cost;
    let remote_dispatch_cost =
        fanout * CUSTOM_SCAN_REMOTE_DISPATCH_CPU_UNITS * constants.cpu_operator_cost;
    let bounded_heap_rows = (output_rows * fanout).min(rel_rows);
    let heap_rerank_cost = bounded_heap_rows * (cpu_tuple_cost + constants.cpu_operator_cost);
    let merge_cost = output_rows
        * fanout.log2().max(1.0)
        * CUSTOM_SCAN_MERGE_CPU_UNITS
        * constants.cpu_operator_cost;
    let tuple_delivery_cost = output_rows * cpu_tuple_cost;
    let startup_cost = routing_traversal_cost + remote_dispatch_cost;
    SpireCustomScanCostEstimate {
        startup_cost,
        total_cost: startup_cost + heap_rerank_cost + merge_cost + tuple_delivery_cost,
    }
}

unsafe fn custom_scan_top_k(root: *mut pg_sys::PlannerInfo) -> Option<usize> {
    let root_ref = unsafe { root.as_ref()? };
    if root_ref.limit_tuples < 0.0 || !root_ref.limit_tuples.is_finite() {
        return None;
    }
    Some(root_ref.limit_tuples.max(0.0).ceil() as usize)
}

unsafe fn custom_scan_orderby_query_expr(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
) -> Option<*mut pg_sys::Expr> {
    if root.is_null() || rel.is_null() {
        return None;
    }
    let root_ref = unsafe { root.as_ref()? };
    let rel_ref = unsafe { rel.as_ref()? };
    let query = unsafe { root_ref.parse.as_ref()? };
    if query.sortClause.is_null() || query.targetList.is_null() {
        return None;
    }
    let sort_clauses = unsafe { PgList::<pg_sys::SortGroupClause>::from_pg(query.sortClause) };
    if sort_clauses.len() != 1 {
        return None;
    }
    let sort_clause = unsafe { sort_clauses.get_ptr(0)?.as_ref()? };
    let target_list = unsafe { PgList::<pg_sys::TargetEntry>::from_pg(query.targetList) };
    for target_entry in target_list.iter_ptr() {
        let Some(target_entry) = (unsafe { target_entry.as_ref() }) else {
            continue;
        };
        if target_entry.ressortgroupref != sort_clause.tleSortGroupRef {
            continue;
        }
        return unsafe { custom_scan_query_expr_from_sort_expr(target_entry.expr, rel_ref.relid) };
    }
    None
}

unsafe fn custom_scan_query_expr_from_sort_expr(
    expr: *mut pg_sys::Expr,
    relid: pg_sys::Index,
) -> Option<*mut pg_sys::Expr> {
    if expr.is_null() {
        return None;
    }
    let node = expr.cast::<pg_sys::Node>();
    if unsafe { (*node).type_ } != pg_sys::NodeTag::T_OpExpr {
        return None;
    }
    let op_expr = unsafe { &*expr.cast::<pg_sys::OpExpr>() };
    let args = unsafe { PgList::<pg_sys::Expr>::from_pg(op_expr.args) };
    if args.len() != 2 {
        return None;
    }
    let left = args.get_ptr(0)?;
    let right = args.get_ptr(1)?;
    if unsafe {
        custom_scan_expr_is_relation_var(left, relid) && custom_scan_expr_is_query_value(right)
    } {
        return Some(right);
    }
    if unsafe {
        custom_scan_expr_is_relation_var(right, relid) && custom_scan_expr_is_query_value(left)
    } {
        return Some(left);
    }
    None
}

unsafe fn custom_scan_expr_is_relation_var(expr: *mut pg_sys::Expr, relid: pg_sys::Index) -> bool {
    if expr.is_null() {
        return false;
    }
    let node = expr.cast::<pg_sys::Node>();
    if unsafe { (*node).type_ } != pg_sys::NodeTag::T_Var {
        return false;
    }
    let var = unsafe { &*expr.cast::<pg_sys::Var>() };
    u32::try_from(var.varno).ok() == Some(relid) && var.varlevelsup == 0
}

unsafe fn custom_scan_expr_is_query_value(expr: *mut pg_sys::Expr) -> bool {
    if expr.is_null() {
        return false;
    }
    let node = expr.cast::<pg_sys::Node>();
    match unsafe { (*node).type_ } {
        pg_sys::NodeTag::T_Const => unsafe {
            custom_scan_query_values_from_const(expr.cast()).is_some()
        },
        pg_sys::NodeTag::T_Param => unsafe {
            let param = &*expr.cast::<pg_sys::Param>();
            param.paramtype == pg_sys::FLOAT4ARRAYOID
        },
        _ => false,
    }
}

unsafe fn custom_scan_query_values_from_const(const_expr: *mut pg_sys::Const) -> Option<Vec<f32>> {
    if const_expr.is_null() {
        return None;
    }
    let const_ref = unsafe { &*const_expr };
    if const_ref.constisnull || const_ref.consttype != pg_sys::FLOAT4ARRAYOID {
        return None;
    }
    let values = unsafe {
        Vec::<f32>::from_polymorphic_datum(const_ref.constvalue, false, pg_sys::FLOAT4ARRAYOID)?
    };
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return None;
    }
    Some(values)
}

unsafe fn custom_scan_plan(node: *mut pg_sys::CustomScanState) -> *mut pg_sys::CustomScan {
    unsafe { (*node).ss.ps.plan.cast::<pg_sys::CustomScan>() }
}

unsafe fn custom_scan_mode_from_path(
    custom_path: *mut pg_sys::CustomPath,
) -> Option<SpireCustomScanPlanMode> {
    unsafe {
        if custom_path.is_null() || (*custom_path).custom_private.is_null() {
            return None;
        }
        custom_scan_mode_from_u32(pg_sys::list_nth_oid((*custom_path).custom_private, 0).to_u32())
    }
}

unsafe fn custom_scan_index_oid_from_path(custom_path: *mut pg_sys::CustomPath) -> pg_sys::Oid {
    unsafe {
        if custom_path.is_null() || (*custom_path).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan CustomPath is missing private index OID");
        }
        pg_sys::list_nth_oid((*custom_path).custom_private, 1)
    }
}

unsafe fn custom_scan_mode_from_plan(
    custom_scan: *mut pg_sys::CustomScan,
) -> SpireCustomScanPlanMode {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing private mode");
        }
        let raw = custom_scan_plan_private_u32((*custom_scan).custom_private, 0, "mode");
        custom_scan_mode_from_u32(raw)
            .unwrap_or_else(|| pgrx::error!("EcSpireDistributedScan plan has unknown mode {raw}"))
    }
}

fn custom_scan_mode_from_u32(raw: u32) -> Option<SpireCustomScanPlanMode> {
    match raw {
        CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT => Some(SpireCustomScanPlanMode::VectorOrderLimit),
        CUSTOM_SCAN_PLAN_MODE_DML_PK_SELECT => {
            Some(SpireCustomScanPlanMode::DmlPkSelectTuplePayload)
        }
        CUSTOM_SCAN_PLAN_MODE_DML_UPDATE => Some(SpireCustomScanPlanMode::DmlUpdateTuplePayload),
        CUSTOM_SCAN_PLAN_MODE_DML_DELETE => Some(SpireCustomScanPlanMode::DmlDeleteTuplePayload),
        _ => None,
    }
}

fn custom_scan_plan_mode_for_dml_mode(
    mode: super::SpireDmlFrontdoorCustomScanMode,
) -> SpireCustomScanPlanMode {
    match mode {
        super::SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload => {
            SpireCustomScanPlanMode::DmlUpdateTuplePayload
        }
        super::SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload => {
            SpireCustomScanPlanMode::DmlDeleteTuplePayload
        }
        super::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload => {
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload
        }
    }
}

unsafe fn custom_scan_index_oid_from_plan(custom_scan: *mut pg_sys::CustomScan) -> pg_sys::Oid {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing private index OID");
        }
        pg_sys::Oid::from(custom_scan_plan_private_u32(
            (*custom_scan).custom_private,
            1,
            "index OID",
        ))
    }
}

unsafe fn custom_scan_dml_plan_private(
    mode: SpireCustomScanPlanMode,
    index_oid: pg_sys::Oid,
    pk_column: &str,
    updated_columns: &[String],
    projected_columns: &[String],
) -> *mut pg_sys::List {
    unsafe {
        let mut custom_private =
            custom_scan_lappend_string(std::ptr::null_mut(), &mode.raw().to_string());
        custom_private =
            custom_scan_lappend_string(custom_private, &index_oid.to_u32().to_string());
        custom_private = custom_scan_lappend_counted_column_list(custom_private, updated_columns);
        custom_private = custom_scan_lappend_counted_column_list(custom_private, projected_columns);
        custom_scan_lappend_string(custom_private, pk_column)
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn custom_scan_dml_plan_private_copy_roundtrip_for_test() -> String {
    unsafe {
        let updated_columns = vec!["title".to_owned(), "status".to_owned()];
        let projected_columns = Vec::<String>::new();
        let custom_private = custom_scan_dml_plan_private(
            SpireCustomScanPlanMode::DmlUpdateTuplePayload,
            pg_sys::Oid::from(12_345),
            "id",
            &updated_columns,
            &projected_columns,
        );
        let copied = pg_sys::copyObjectImpl(custom_private.cast()).cast::<pg_sys::List>();
        let mode = custom_scan_plan_private_u32(copied, 0, "mode");
        let index_oid = custom_scan_plan_private_u32(copied, 1, "index OID");
        let updated = custom_scan_dml_column_list_from_plan_private(copied, 2, "updated columns");
        let projected =
            custom_scan_dml_column_list_from_plan_private(copied, 3, "projected columns");
        let pk_column = custom_scan_dml_pk_column_from_plan_private(copied);
        format!(
            "{}|{}|{}|{}|{}",
            mode,
            index_oid,
            updated.join(","),
            projected.join(","),
            pk_column
        )
    }
}

unsafe fn custom_scan_plan_private_u32(
    custom_private: *mut pg_sys::List,
    offset: i32,
    label: &str,
) -> u32 {
    unsafe {
        if custom_private.is_null() || (*custom_private).length <= offset {
            pgrx::error!("EcSpireDistributedScan plan is missing private {label}");
        }
        match (*custom_private).type_ {
            pg_sys::NodeTag::T_OidList => pg_sys::list_nth_oid(custom_private, offset).to_u32(),
            pg_sys::NodeTag::T_List => {
                let node = pg_sys::list_nth(custom_private, offset).cast::<pg_sys::Node>();
                if node.is_null() {
                    pgrx::error!("EcSpireDistributedScan plan has null private {label}");
                }
                match (*node).type_ {
                    pg_sys::NodeTag::T_Integer => {
                        let value = (*node.cast::<pg_sys::Integer>()).ival;
                        u32::try_from(value).unwrap_or_else(|_| {
                            pgrx::error!("EcSpireDistributedScan plan private {label} is negative")
                        })
                    }
                    pg_sys::NodeTag::T_String => custom_scan_string_node_value(node, label)
                        .parse::<u32>()
                        .unwrap_or_else(|e| {
                            pgrx::error!(
                                "EcSpireDistributedScan plan private {label} is not u32: {e}"
                            )
                        }),
                    _ => {
                        pgrx::error!("EcSpireDistributedScan plan has invalid private {label}")
                    }
                }
            }
            _ => pgrx::error!("EcSpireDistributedScan plan has invalid private metadata list"),
        }
    }
}

unsafe fn custom_scan_lappend_string(list: *mut pg_sys::List, value: &str) -> *mut pg_sys::List {
    unsafe {
        let c_value = CString::new(value).unwrap_or_else(|_| {
            pgrx::error!("EcSpireDistributedScan plan-private string contains NUL")
        });
        let copied = pg_sys::pstrdup(c_value.as_ptr());
        pg_sys::lappend(list, pg_sys::makeString(copied).cast())
    }
}

unsafe fn custom_scan_lappend_counted_column_list(
    list: *mut pg_sys::List,
    columns: &[String],
) -> *mut pg_sys::List {
    unsafe {
        let mut list = custom_scan_lappend_string(list, &columns.len().to_string());
        for column in columns {
            list = custom_scan_lappend_string(list, column);
        }
        list
    }
}

unsafe fn custom_scan_string_node_value(node: *mut pg_sys::Node, label: &str) -> String {
    unsafe {
        if node.is_null() || (*node).type_ != pg_sys::NodeTag::T_String {
            pgrx::error!("EcSpireDistributedScan DML plan has invalid {label} metadata");
        }
        let value_node = node.cast::<pg_sys::String>();
        if (*value_node).sval.is_null() {
            pgrx::error!("EcSpireDistributedScan DML plan has null {label} metadata");
        }
        std::ffi::CStr::from_ptr((*value_node).sval)
            .to_str()
            .unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan DML plan {label} metadata is not UTF-8")
            })
            .to_owned()
    }
}

unsafe fn custom_scan_dml_column_list_from_plan(
    custom_scan: *mut pg_sys::CustomScan,
    offset: i32,
    label: &str,
) -> Vec<String> {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan DML plan is missing {label} metadata");
        }
        custom_scan_dml_column_list_from_plan_private((*custom_scan).custom_private, offset, label)
    }
}

unsafe fn custom_scan_dml_column_list_from_plan_private(
    custom_private: *mut pg_sys::List,
    offset: i32,
    label: &str,
) -> Vec<String> {
    unsafe {
        if custom_private.is_null() || (*custom_private).length <= offset {
            pgrx::error!("EcSpireDistributedScan DML plan is missing {label} metadata");
        }
        let count_offset = match offset {
            DML_UPDATED_COLUMN_COUNT_OFFSET => DML_UPDATED_COLUMN_COUNT_OFFSET,
            3 => custom_scan_dml_projected_column_count_offset(custom_private),
            _ => pgrx::error!("EcSpireDistributedScan DML plan has invalid {label} offset"),
        };
        custom_scan_dml_counted_column_list_from_plan_private(custom_private, count_offset, label)
            .unwrap_or_else(|e| pgrx::error!("{e}"))
    }
}

unsafe fn custom_scan_dml_counted_column_list_from_plan_private(
    custom_private: *mut pg_sys::List,
    count_offset: i32,
    label: &str,
) -> Result<Vec<String>, String> {
    unsafe {
        if custom_private.is_null() || (*custom_private).length <= count_offset {
            return Err(format!(
                "EcSpireDistributedScan DML plan is missing {label} count metadata"
            ));
        }
        let count = usize::try_from(custom_scan_plan_private_u32(
            custom_private,
            count_offset,
            &format!("{label} count"),
        ))
        .map_err(|_| format!("EcSpireDistributedScan DML plan {label} count is too large"))?;
        let required_len = count_offset
            .checked_add(1)
            .and_then(|offset| offset.checked_add(i32::try_from(count).ok()?))
            .ok_or_else(|| {
                format!("EcSpireDistributedScan DML plan {label} count overflows metadata list")
            })?;
        if (*custom_private).length < required_len {
            return Err(format!(
                "EcSpireDistributedScan DML plan {label} metadata is truncated"
            ));
        }
        let mut columns = Vec::with_capacity(count);
        for index in 0..count {
            let offset = count_offset
                + 1
                + i32::try_from(index).unwrap_or_else(|_| {
                    pgrx::error!("EcSpireDistributedScan DML plan {label} index is too large")
                });
            let column = custom_scan_string_node_value(
                pg_sys::list_nth(custom_private, offset).cast(),
                label,
            );
            if column.is_empty() {
                return Err(format!(
                    "EcSpireDistributedScan DML plan {label} metadata contains an empty column name"
                ));
            }
            columns.push(column);
        }
        Ok(columns)
    }
}

unsafe fn custom_scan_dml_projected_column_count_offset(custom_private: *mut pg_sys::List) -> i32 {
    unsafe {
        let updated_count = custom_scan_plan_private_u32(
            custom_private,
            DML_UPDATED_COLUMN_COUNT_OFFSET,
            "updated column count",
        );
        DML_UPDATED_COLUMN_COUNT_OFFSET
            + 1
            + i32::try_from(updated_count).unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan DML plan updated column count is too large")
            })
    }
}

unsafe fn custom_scan_dml_pk_column_offset(custom_private: *mut pg_sys::List) -> i32 {
    unsafe {
        let projected_offset = custom_scan_dml_projected_column_count_offset(custom_private);
        let projected_count = custom_scan_plan_private_u32(
            custom_private,
            projected_offset,
            "projected column count",
        );
        projected_offset
            + 1
            + i32::try_from(projected_count).unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan DML plan projected column count is too large")
            })
    }
}

unsafe fn custom_scan_dml_pk_column_from_plan(custom_scan: *mut pg_sys::CustomScan) -> String {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan DML plan is missing PK column metadata");
        }
        custom_scan_dml_pk_column_from_plan_private((*custom_scan).custom_private)
    }
}

unsafe fn custom_scan_dml_pk_column_from_plan_private(custom_private: *mut pg_sys::List) -> String {
    unsafe {
        if custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan DML plan is missing PK column metadata");
        }
        let offset = custom_scan_dml_pk_column_offset(custom_private);
        if (*custom_private).length <= offset {
            pgrx::error!("EcSpireDistributedScan DML plan is missing PK column metadata");
        }
        let pk_column = custom_scan_string_node_value(
            pg_sys::list_nth(custom_private, offset).cast(),
            "PK column",
        );
        if pk_column.is_empty() {
            pgrx::error!("EcSpireDistributedScan DML plan PK column metadata is empty");
        }
        pk_column
    }
}

fn custom_scan_validate_dml_column_metadata(
    mode: SpireCustomScanPlanMode,
    updated_columns: &[String],
    projected_columns: &[String],
) -> Result<(), String> {
    match mode {
        SpireCustomScanPlanMode::DmlPkSelectTuplePayload => {
            if projected_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML PK SELECT plan requires projected columns"
                        .to_owned(),
                );
            }
            if !updated_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML PK SELECT plan must not update columns".to_owned(),
                );
            }
        }
        SpireCustomScanPlanMode::DmlUpdateTuplePayload => {
            if updated_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML UPDATE plan requires updated columns".to_owned(),
                );
            }
            if !projected_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML UPDATE plan must not project columns".to_owned(),
                );
            }
        }
        SpireCustomScanPlanMode::DmlDeleteTuplePayload => {
            if !updated_columns.is_empty() || !projected_columns.is_empty() {
                return Err(
                    "EcSpireDistributedScan DML DELETE plan must not carry column payload metadata"
                        .to_owned(),
                );
            }
        }
        SpireCustomScanPlanMode::VectorOrderLimit => {}
    }
    Ok(())
}

fn custom_scan_dml_frontdoor_mode_for_plan_mode(
    mode: SpireCustomScanPlanMode,
) -> Result<super::SpireDmlFrontdoorCustomScanMode, String> {
    match mode {
        SpireCustomScanPlanMode::DmlPkSelectTuplePayload => {
            Ok(super::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload)
        }
        SpireCustomScanPlanMode::DmlUpdateTuplePayload => {
            Ok(super::SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload)
        }
        SpireCustomScanPlanMode::DmlDeleteTuplePayload => {
            Ok(super::SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload)
        }
        SpireCustomScanPlanMode::VectorOrderLimit => {
            Err("EcSpireDistributedScan vector plan mode has no DML primitive".to_owned())
        }
    }
}

fn custom_scan_dml_primitive_name(mode: SpireCustomScanPlanMode) -> Result<&'static str, String> {
    match mode {
        SpireCustomScanPlanMode::DmlPkSelectTuplePayload => {
            Ok("ec_spire_forward_coordinator_select_tuple_payload")
        }
        SpireCustomScanPlanMode::DmlUpdateTuplePayload => {
            Ok("ec_spire_forward_coordinator_update_tuple_payload")
        }
        SpireCustomScanPlanMode::DmlDeleteTuplePayload => {
            Ok("ec_spire_prepare_coordinator_delete_tuple_payload")
        }
        SpireCustomScanPlanMode::VectorOrderLimit => {
            Err("EcSpireDistributedScan vector plan mode has no DML primitive".to_owned())
        }
    }
}

fn custom_scan_dml_primitive_invocation_from_parts(
    index_oid: pg_sys::Oid,
    mode: SpireCustomScanPlanMode,
    pk_column: &str,
    pk_value: [u8; 8],
    updated_columns: &[String],
    projected_columns: &[String],
) -> Result<super::dml_frontdoor::SpireDmlFrontdoorPrimitiveInvocation, String> {
    if index_oid == pg_sys::InvalidOid {
        return Err(
            "EcSpireDistributedScan DML primitive invocation requires index OID".to_owned(),
        );
    }
    if pk_column.is_empty() {
        return Err(
            "EcSpireDistributedScan DML primitive invocation requires PK column".to_owned(),
        );
    }
    custom_scan_validate_dml_column_metadata(mode, updated_columns, projected_columns)?;
    Ok(super::dml_frontdoor::SpireDmlFrontdoorPrimitiveInvocation {
        index_oid,
        mode: custom_scan_dml_frontdoor_mode_for_plan_mode(mode)?,
        primitive: custom_scan_dml_primitive_name(mode)?,
        pk_column: pk_column.to_owned(),
        pk_value,
        updated_columns: updated_columns.to_vec(),
        projected_columns: projected_columns.to_vec(),
    })
}

fn custom_scan_dml_primitive_invocation(
    state: &SpireCustomScanExecState,
) -> Result<super::dml_frontdoor::SpireDmlFrontdoorPrimitiveInvocation, String> {
    custom_scan_dml_primitive_invocation_from_parts(
        state.index_oid,
        state.mode,
        &state.dml_pk_column,
        state.dml_pk_value,
        &state.dml_updated_columns,
        &state.dml_projected_columns,
    )
}

unsafe fn custom_scan_top_k_from_plan(custom_scan: *mut pg_sys::CustomScan) -> usize {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing private LIMIT");
        }
        let raw = pg_sys::list_nth_oid((*custom_scan).custom_private, 2);
        usize::try_from(raw.to_u32())
            .unwrap_or_else(|_| pgrx::error!("EcSpireDistributedScan plan LIMIT is out of range"))
    }
}

unsafe fn custom_scan_query_from_plan(
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) -> Vec<f32> {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_exprs.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing ORDER BY query expression");
        }
        let expr = pg_sys::list_nth((*custom_scan).custom_exprs, 0).cast::<pg_sys::Expr>();
        if expr.is_null() {
            pgrx::error!("EcSpireDistributedScan plan has a null ORDER BY query expression");
        }
        let datum = match (*expr.cast::<pg_sys::Node>()).type_ {
            pg_sys::NodeTag::T_Const => {
                return custom_scan_query_values_from_const(expr.cast()).unwrap_or_else(|| {
                    pgrx::error!(
                        "EcSpireDistributedScan requires a non-null finite real[] ORDER BY query"
                    )
                });
            }
            pg_sys::NodeTag::T_Param => {
                let expr_state = pg_sys::ExecInitExpr(expr, &mut (*node).ss.ps);
                if expr_state.is_null() {
                    pgrx::error!("EcSpireDistributedScan failed to initialize ORDER BY query parameter expression");
                }
                let eval = (*expr_state).evalfunc.unwrap_or_else(|| {
                    pgrx::error!(
                        "EcSpireDistributedScan ORDER BY query expression has no evaluator"
                    )
                });
                let mut is_null = false;
                let datum = eval(expr_state, (*node).ss.ps.ps_ExprContext, &mut is_null);
                if is_null {
                    pgrx::error!(
                        "EcSpireDistributedScan ORDER BY query parameter must not be NULL"
                    );
                }
                datum
            }
            _ => pgrx::error!(
                "EcSpireDistributedScan requires a constant or parameter real[] ORDER BY query"
            ),
        };
        custom_scan_query_values_from_datum(datum).unwrap_or_else(|| {
            pgrx::error!("EcSpireDistributedScan requires a finite real[] ORDER BY query parameter")
        })
    }
}

unsafe fn custom_scan_query_values_from_datum(datum: pg_sys::Datum) -> Option<Vec<f32>> {
    let values =
        unsafe { Vec::<f32>::from_polymorphic_datum(datum, false, pg_sys::FLOAT4ARRAYOID)? };
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return None;
    }
    Some(values)
}

unsafe fn custom_scan_tuple_payload_columns(
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) -> Vec<String> {
    unsafe {
        let relation = (*node).ss.ss_currentRelation;
        if relation.is_null() {
            pgrx::error!("EcSpireDistributedScan missing scan relation for tuple payload columns");
        }
        let tuple_desc = (*relation).rd_att;
        if tuple_desc.is_null() {
            pgrx::error!("EcSpireDistributedScan missing scan relation tuple descriptor");
        }
        let mut attr_numbers = std::collections::BTreeSet::new();
        let mut can_narrow_projection = false;
        if !custom_scan.is_null() && !(*custom_scan).scan.plan.targetlist.is_null() {
            let target_list =
                PgList::<pg_sys::TargetEntry>::from_pg((*custom_scan).scan.plan.targetlist);
            can_narrow_projection = true;
            for target_entry in target_list.iter_ptr() {
                let Some(target_entry) = target_entry.as_ref() else {
                    continue;
                };
                if target_entry.resjunk || target_entry.expr.is_null() {
                    continue;
                }
                let expr = target_entry.expr.cast::<pg_sys::Node>();
                if (*expr).type_ != pg_sys::NodeTag::T_Var {
                    can_narrow_projection = false;
                    break;
                }
                let var = &*target_entry.expr.cast::<pg_sys::Var>();
                if var.varattno > 0 {
                    attr_numbers.insert(var.varattno);
                } else {
                    can_narrow_projection = false;
                    break;
                }
            }
        }
        if !can_narrow_projection {
            attr_numbers.clear();
        }
        let natts = (*tuple_desc).natts;
        let mut columns = Vec::with_capacity(usize::try_from(natts).unwrap_or(0));
        for attr_index in 0..natts {
            let attr = pg_sys::TupleDescAttr(tuple_desc, attr_index);
            if attr.is_null() || (*attr).attisdropped {
                continue;
            }
            if !attr_numbers.is_empty() && !attr_numbers.contains(&(*attr).attnum) {
                continue;
            }
            let name = std::ffi::CStr::from_ptr((*attr).attname.data.as_ptr())
                .to_str()
                .unwrap_or_else(|_| {
                    pgrx::error!("EcSpireDistributedScan relation attribute name is not UTF-8")
                })
                .to_owned();
            custom_scan_validate_tuple_payload_attr(attr, &name);
            columns.push(name);
        }
        columns
    }
}

unsafe fn custom_scan_validate_tuple_payload_attr(attr: pg_sys::Form_pg_attribute, name: &str) {
    unsafe {
        // Reject PG arrays and row composites while allowing scalar base/domain
        // types such as ecvector, json/jsonb, enum, and range through type input.
        if pg_sys::get_element_type((*attr).atttypid) != pg_sys::InvalidOid {
            pgrx::error!(
                "EcSpireDistributedScan tuple payload column \"{name}\" has unsupported array type"
            );
        }
        if pg_sys::type_is_rowtype((*attr).atttypid) {
            pgrx::error!(
                "EcSpireDistributedScan tuple payload column \"{name}\" has unsupported composite type"
            );
        }
    }
}

unsafe fn custom_scan_payload_attr_inputs(
    tuple_desc: pg_sys::TupleDesc,
) -> Vec<Option<SpireCustomScanPayloadAttrInput>> {
    unsafe {
        if tuple_desc.is_null() {
            pgrx::error!("EcSpireDistributedScan tuple payload input descriptor is null");
        }
        let natts = (*tuple_desc).natts;
        let mut inputs = Vec::with_capacity(usize::try_from(natts).unwrap_or(0));
        for attr_index in 0..natts {
            let attr = pg_sys::TupleDescAttr(tuple_desc, attr_index);
            if attr.is_null() || (*attr).attisdropped {
                inputs.push(None);
                continue;
            }
            let mut typinput = pg_sys::InvalidOid;
            let mut typioparam = pg_sys::InvalidOid;
            pg_sys::getTypeInputInfo((*attr).atttypid, &mut typinput, &mut typioparam);
            let mut flinfo = std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init();
            pg_sys::fmgr_info(typinput, &mut flinfo);
            inputs.push(Some(SpireCustomScanPayloadAttrInput {
                flinfo,
                typioparam,
                typmod: (*attr).atttypmod,
            }));
        }
        inputs
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_create_custom_scan_state(
    _cscan: *mut pg_sys::CustomScan,
) -> *mut pg_sys::Node {
    unsafe {
        let state = pg_sys::palloc0(std::mem::size_of::<SpireCustomScanExecState>())
            .cast::<SpireCustomScanExecState>();
        ptr::write(
            state,
            SpireCustomScanExecState {
                custom_scan_state: std::mem::zeroed(),
                mode: SpireCustomScanPlanMode::VectorOrderLimit,
                index_oid: pg_sys::InvalidOid,
                top_k: 0,
                query: Vec::new(),
                dml_pk_column: String::new(),
                dml_pk_value: [0; 8],
                dml_updated_columns: Vec::new(),
                dml_projected_columns: Vec::new(),
                dml_update_value_exprs: Vec::new(),
                tuple_payload_columns: Vec::new(),
                tuple_payload_inputs: Vec::new(),
                outputs: Vec::new(),
                next_output: 0,
                loaded_outputs: false,
                dml_payload_loaded: false,
                dml_payload_emitted: false,
                dml_tuple_payload_json: None,
            },
        );
        (*state).custom_scan_state.ss.ps.type_ = pg_sys::NodeTag::T_CustomScanState;
        (*state).custom_scan_state.methods = &raw const CUSTOM_EXEC_METHODS;
        state.cast::<pg_sys::Node>()
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_begin_custom_scan(
    node: *mut pg_sys::CustomScanState,
    _estate: *mut pg_sys::EState,
    _eflags: core::ffi::c_int,
) {
    unsafe {
        let state = node.cast::<SpireCustomScanExecState>();
        let custom_scan = custom_scan_plan(node);
        (*state).mode = custom_scan_mode_from_plan(custom_scan);
        (*state).index_oid = custom_scan_index_oid_from_plan(custom_scan);
        match (*state).mode {
            SpireCustomScanPlanMode::VectorOrderLimit => {
                custom_scan_init_tuple_payload_state(state, node, custom_scan);
                (*state).top_k = custom_scan_top_k_from_plan(custom_scan);
                (*state).query = custom_scan_query_from_plan(node, custom_scan);
            }
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload => {
                custom_scan_init_tuple_payload_state(state, node, custom_scan);
                (*state).dml_pk_column = custom_scan_dml_pk_column(node);
                (*state).dml_pk_value = custom_scan_dml_pk_value_from_plan(node, custom_scan);
                (*state).dml_updated_columns =
                    custom_scan_dml_column_list_from_plan(custom_scan, 2, "updated columns");
                (*state).dml_projected_columns =
                    custom_scan_dml_column_list_from_plan(custom_scan, 3, "projected columns");
                custom_scan_validate_dml_column_metadata(
                    (*state).mode,
                    &(*state).dml_updated_columns,
                    &(*state).dml_projected_columns,
                )
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            }
            SpireCustomScanPlanMode::DmlUpdateTuplePayload
            | SpireCustomScanPlanMode::DmlDeleteTuplePayload => {
                (*state).dml_pk_column = custom_scan_dml_pk_column_from_plan(custom_scan);
                (*state).dml_pk_value = custom_scan_dml_pk_value_from_plan(node, custom_scan);
                (*state).dml_updated_columns =
                    custom_scan_dml_column_list_from_plan(custom_scan, 2, "updated columns");
                (*state).dml_projected_columns =
                    custom_scan_dml_column_list_from_plan(custom_scan, 3, "projected columns");
                if (*state).mode == SpireCustomScanPlanMode::DmlUpdateTuplePayload {
                    (*state).dml_update_value_exprs = custom_scan_dml_update_value_exprs_from_plan(
                        custom_scan,
                        (*state).dml_updated_columns.len(),
                    );
                }
                custom_scan_validate_dml_column_metadata(
                    (*state).mode,
                    &(*state).dml_updated_columns,
                    &(*state).dml_projected_columns,
                )
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            }
        }
    }
}

unsafe fn custom_scan_init_tuple_payload_state(
    state: *mut SpireCustomScanExecState,
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) {
    unsafe {
        (*state).tuple_payload_columns = custom_scan_tuple_payload_columns(node, custom_scan);
        (*state).tuple_payload_inputs =
            custom_scan_payload_attr_inputs((*(*node).ss.ss_currentRelation).rd_att);
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_exec_custom_scan(
    node: *mut pg_sys::CustomScanState,
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        pg_sys::ExecScan(
            &mut (*node).ss,
            Some(ec_spire_custom_scan_access),
            Some(ec_spire_custom_scan_recheck),
        )
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_end_custom_scan(node: *mut pg_sys::CustomScanState) {
    unsafe {
        if node.is_null() {
            return;
        }
        let state = node.cast::<SpireCustomScanExecState>();
        ptr::drop_in_place(state);
        pg_sys::pfree(state.cast());
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_rescan_custom_scan(node: *mut pg_sys::CustomScanState) {
    unsafe {
        let state = node.cast::<SpireCustomScanExecState>();
        (*state).outputs.clear();
        (*state).next_output = 0;
        (*state).loaded_outputs = false;
        (*state).dml_payload_loaded = false;
        (*state).dml_payload_emitted = false;
        (*state).dml_tuple_payload_json = None;
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_custom_scan_access(
    scan_state: *mut pg_sys::ScanState,
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if scan_state.is_null() {
            pgrx::error!("EcSpireDistributedScan access method received null scan state");
        }
        let state = scan_state.cast::<SpireCustomScanExecState>();
        if (*state).mode == SpireCustomScanPlanMode::DmlPkSelectTuplePayload {
            return custom_scan_dml_pk_select_access(state, scan_state);
        }
        if (*state).mode == SpireCustomScanPlanMode::DmlUpdateTuplePayload {
            return custom_scan_dml_update_access(state, scan_state);
        }
        if (*state).mode == SpireCustomScanPlanMode::DmlDeleteTuplePayload {
            return custom_scan_dml_delete_access(state, scan_state);
        }
        custom_scan_ensure_outputs(state);
        loop {
            let Some(output) = (&(*state).outputs).get((*state).next_output) else {
                return pg_sys::ExecClearTuple((*scan_state).ss_ScanTupleSlot);
            };
            (*state).next_output = (*state).next_output.saturating_add(1);
            if !matches!(
                output.heap_lookup_owner,
                super::SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION
            ) {
                return custom_scan_store_remote_tuple_payload(state, scan_state, output);
            }

            let mut tid = pg_sys::ItemPointerData::default();
            pgrx::itemptr::item_pointer_set_all(&mut tid, output.heap_block, output.heap_offset);
            pg_sys::ExecClearTuple((*scan_state).ss_ScanTupleSlot);
            let estate = (*scan_state).ps.state;
            if estate.is_null() {
                pgrx::error!("EcSpireDistributedScan missing executor estate");
            }
            let visible = pg_sys::table_tuple_fetch_row_version(
                (*scan_state).ss_currentRelation,
                &mut tid,
                (*estate).es_snapshot,
                (*scan_state).ss_ScanTupleSlot,
            );
            if visible {
                return (*scan_state).ss_ScanTupleSlot;
            }
        }
    }
}

unsafe fn custom_scan_dml_pk_select_access(
    state: *mut SpireCustomScanExecState,
    scan_state: *mut pg_sys::ScanState,
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        custom_scan_ensure_dml_pk_select_payload(state);
        if (*state).dml_payload_emitted {
            return pg_sys::ExecClearTuple((*scan_state).ss_ScanTupleSlot);
        }
        (*state).dml_payload_emitted = true;
        let Some(payload_json) = (*state).dml_tuple_payload_json.as_deref() else {
            return pg_sys::ExecClearTuple((*scan_state).ss_ScanTupleSlot);
        };
        custom_scan_store_tuple_payload_json(
            (*scan_state).ss_ScanTupleSlot,
            payload_json,
            &mut (*state).tuple_payload_inputs,
        )
    }
}

unsafe fn custom_scan_dml_update_access(
    state: *mut SpireCustomScanExecState,
    scan_state: *mut pg_sys::ScanState,
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if !(*state).dml_payload_emitted {
            let updated_count = custom_scan_execute_dml_update(state, scan_state);
            let estate = (*scan_state).ps.state;
            if !estate.is_null() {
                (*estate).es_processed = (*estate).es_processed.saturating_add(updated_count);
            }
            (*state).dml_payload_emitted = true;
        }
        pg_sys::ExecClearTuple((*scan_state).ss_ScanTupleSlot)
    }
}

unsafe fn custom_scan_dml_delete_access(
    state: *mut SpireCustomScanExecState,
    scan_state: *mut pg_sys::ScanState,
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if !(*state).dml_payload_emitted {
            let deleted_count = custom_scan_execute_dml_delete(state);
            let estate = (*scan_state).ps.state;
            if !estate.is_null() {
                (*estate).es_processed = (*estate).es_processed.saturating_add(deleted_count);
            }
            (*state).dml_payload_emitted = true;
        }
        pg_sys::ExecClearTuple((*scan_state).ss_ScanTupleSlot)
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_custom_scan_recheck(
    _scan_state: *mut pg_sys::ScanState,
    _slot: *mut pg_sys::TupleTableSlot,
) -> bool {
    // V1 remote tuples are virtual payloads without coordinator heap row
    // identity, so EvalPlanQual cannot re-fetch the origin row here.
    true
}

unsafe fn custom_scan_store_remote_tuple_payload(
    state: *mut SpireCustomScanExecState,
    scan_state: *mut pg_sys::ScanState,
    output: &super::SpireRemoteProductionScanOutputRow,
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if output.tuple_payload_missing {
            pgrx::error!(
                "EcSpireDistributedScan remote tuple payload is missing for node_id {} output",
                output.node_id
            );
        }
        let Some(payload_json) = output.tuple_payload_json.as_deref() else {
            pgrx::error!(
                "EcSpireDistributedScan tuple payload delivery requires remote payload for node_id {} output; heap_lookup_owner {}",
                output.node_id,
                output.heap_lookup_owner
            );
        };
        custom_scan_store_tuple_payload_json(
            (*scan_state).ss_ScanTupleSlot,
            payload_json,
            &mut (*state).tuple_payload_inputs,
        )
    }
}

unsafe fn custom_scan_store_tuple_payload_json(
    slot: *mut pg_sys::TupleTableSlot,
    payload_json: &str,
    attr_inputs: &mut [Option<SpireCustomScanPayloadAttrInput>],
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if slot.is_null() {
            pgrx::error!("EcSpireDistributedScan tuple payload slot is null");
        }
        let tuple_desc = (*slot).tts_tupleDescriptor;
        if tuple_desc.is_null() {
            pgrx::error!("EcSpireDistributedScan tuple payload slot has no tuple descriptor");
        }
        let payload = serde_json::from_str::<serde_json::Value>(payload_json).unwrap_or_else(|e| {
            pgrx::error!("EcSpireDistributedScan remote tuple payload JSON decode failed: {e}")
        });
        let payload_object = payload.as_object().unwrap_or_else(|| {
            pgrx::error!("EcSpireDistributedScan remote tuple payload must be a JSON object")
        });

        if attr_inputs.len() != usize::try_from((*tuple_desc).natts).unwrap_or(usize::MAX) {
            pgrx::error!("EcSpireDistributedScan tuple payload input cache width mismatch");
        }

        pg_sys::ExecClearTuple(slot);
        let natts = (*tuple_desc).natts;
        for attr_index in 0..natts {
            let attr = pg_sys::TupleDescAttr(tuple_desc, attr_index);
            if attr.is_null() || (*attr).attisdropped {
                *(*slot).tts_isnull.add(attr_index as usize) = true;
                *(*slot).tts_values.add(attr_index as usize) = pg_sys::Datum::from(0);
                continue;
            }
            let attr_name = std::ffi::CStr::from_ptr((*attr).attname.data.as_ptr())
                .to_str()
                .unwrap_or_else(|_| {
                    pgrx::error!("EcSpireDistributedScan relation attribute name is not UTF-8")
                });
            match payload_object.get(attr_name) {
                None | Some(serde_json::Value::Null) => {
                    *(*slot).tts_isnull.add(attr_index as usize) = true;
                    *(*slot).tts_values.add(attr_index as usize) = pg_sys::Datum::from(0);
                }
                Some(value) => {
                    *(*slot).tts_isnull.add(attr_index as usize) = false;
                    let Some(attr_input) = attr_inputs
                        .get_mut(attr_index as usize)
                        .and_then(Option::as_mut)
                    else {
                        pgrx::error!(
                            "EcSpireDistributedScan tuple payload input cache missing attribute {}",
                            attr_index + 1
                        );
                    };
                    *(*slot).tts_values.add(attr_index as usize) =
                        custom_scan_json_value_to_datum(value, attr_name, attr_input);
                }
            }
        }
        (*slot).tts_nvalid = i16::try_from(natts)
            .unwrap_or_else(|_| pgrx::error!("EcSpireDistributedScan tuple descriptor too wide"));
        pg_sys::ExecStoreVirtualTuple(slot)
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn custom_scan_store_tuple_payload_json_for_test(
    slot: *mut pg_sys::TupleTableSlot,
    payload_json: &str,
) -> *mut pg_sys::TupleTableSlot {
    unsafe {
        if slot.is_null() {
            pgrx::error!("EcSpireDistributedScan tuple payload slot is null");
        }
        let mut attr_inputs = custom_scan_payload_attr_inputs((*slot).tts_tupleDescriptor);
        custom_scan_store_tuple_payload_json(slot, payload_json, &mut attr_inputs)
    }
}

unsafe fn custom_scan_json_value_to_datum(
    value: &serde_json::Value,
    attr_name: &str,
    attr_input: &mut SpireCustomScanPayloadAttrInput,
) -> pg_sys::Datum {
    unsafe {
        let input_text = match value {
            serde_json::Value::String(value) => value.clone(),
            serde_json::Value::Bool(value) => value.to_string(),
            serde_json::Value::Number(value) => value.to_string(),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                pgrx::error!(
                    "EcSpireDistributedScan tuple payload column \"{attr_name}\" has unsupported non-scalar JSON value"
                )
            }
            serde_json::Value::Null => {
                pgrx::error!("EcSpireDistributedScan cannot convert JSON null to non-null datum")
            }
        };
        let input = CString::new(input_text)
            .unwrap_or_else(|_| pgrx::error!("EcSpireDistributedScan tuple payload contains NUL"));
        pg_sys::InputFunctionCall(
            &mut attr_input.flinfo,
            input.as_ptr().cast_mut(),
            attr_input.typioparam,
            attr_input.typmod,
        )
    }
}

unsafe fn custom_scan_dml_pk_column(node: *mut pg_sys::CustomScanState) -> String {
    unsafe {
        let relation = (*node).ss.ss_currentRelation;
        if relation.is_null() {
            pgrx::error!("EcSpireDistributedScan DML path missing scan relation");
        }
        let context = super::dml_frontdoor_relation_context_catalog_row((*relation).rd_id)
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        context.pk_column.unwrap_or_else(|| {
            pgrx::error!("EcSpireDistributedScan DML path relation has no PK column")
        })
    }
}

unsafe fn custom_scan_dml_pk_value_from_plan(
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) -> [u8; 8] {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_exprs.is_null() {
            pgrx::error!("EcSpireDistributedScan DML plan is missing PK expression");
        }
        let expr = pg_sys::list_nth((*custom_scan).custom_exprs, 0).cast::<pg_sys::Expr>();
        let value = custom_scan_bigint_expr_value(node, expr);
        super::dml_frontdoor_bigint_pk_value_bytes(value)
    }
}

unsafe fn custom_scan_dml_update_value_exprs_from_plan(
    custom_scan: *mut pg_sys::CustomScan,
    expected_count: usize,
) -> Vec<*mut pg_sys::Expr> {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_exprs.is_null() {
            pgrx::error!("EcSpireDistributedScan DML UPDATE plan is missing value expressions");
        }
        let custom_exprs = (*custom_scan).custom_exprs;
        let expected_len = i32::try_from(expected_count.saturating_add(1)).unwrap_or_else(|_| {
            pgrx::error!("EcSpireDistributedScan DML UPDATE expression list is too wide")
        });
        if (*custom_exprs).length != expected_len {
            pgrx::error!(
                "EcSpireDistributedScan DML UPDATE plan expression count does not match updated columns"
            );
        }
        let mut exprs = Vec::with_capacity(expected_count);
        for offset in 1..expected_len {
            let expr = pg_sys::list_nth(custom_exprs, offset).cast::<pg_sys::Expr>();
            if expr.is_null() {
                pgrx::error!("EcSpireDistributedScan DML UPDATE plan has null value expression");
            }
            exprs.push(expr);
        }
        exprs
    }
}

unsafe fn custom_scan_bigint_expr_value(
    node: *mut pg_sys::CustomScanState,
    expr: *mut pg_sys::Expr,
) -> i64 {
    unsafe {
        if expr.is_null() {
            pgrx::error!("EcSpireDistributedScan DML PK expression is null");
        }
        match (*expr.cast::<pg_sys::Node>()).type_ {
            pg_sys::NodeTag::T_Const => {
                let const_expr = &*expr.cast::<pg_sys::Const>();
                if const_expr.constisnull {
                    pgrx::error!("EcSpireDistributedScan DML constant PK is NULL");
                }
                custom_scan_bigint_datum_value(const_expr.constvalue, const_expr.consttype)
            }
            pg_sys::NodeTag::T_Param => {
                let param = &*expr.cast::<pg_sys::Param>();
                let expr_state = pg_sys::ExecInitExpr(expr, &mut (*node).ss.ps);
                if expr_state.is_null() {
                    pgrx::error!("EcSpireDistributedScan failed to initialize DML PK parameter");
                }
                let eval = (*expr_state).evalfunc.unwrap_or_else(|| {
                    pgrx::error!("EcSpireDistributedScan DML PK parameter has no evaluator")
                });
                let mut is_null = false;
                let datum = eval(expr_state, (*node).ss.ps.ps_ExprContext, &mut is_null);
                if is_null {
                    pgrx::error!("EcSpireDistributedScan DML PK parameter must not be NULL");
                }
                custom_scan_bigint_datum_value(datum, param.paramtype)
            }
            _ => pgrx::error!(
                "EcSpireDistributedScan DML path requires a constant or parameter bigint PK"
            ),
        }
    }
}

unsafe fn custom_scan_bigint_datum_value(datum: pg_sys::Datum, typoid: pg_sys::Oid) -> i64 {
    unsafe {
        match typoid {
            pg_sys::INT2OID => i64::from(pg_sys::DatumGetInt16(datum)),
            pg_sys::INT4OID => i64::from(pg_sys::DatumGetInt32(datum)),
            pg_sys::INT8OID => pg_sys::DatumGetInt64(datum),
            other => pgrx::error!(
                "EcSpireDistributedScan DML path unsupported PK type OID {}",
                other.to_u32()
            ),
        }
    }
}

unsafe fn custom_scan_execute_dml_delete(state: *mut SpireCustomScanExecState) -> u64 {
    unsafe {
        let state_ref = &mut *state;
        let invocation =
            custom_scan_dml_primitive_invocation(state_ref).unwrap_or_else(|e| pgrx::error!("{e}"));
        if invocation.mode != super::SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload
        {
            pgrx::error!("EcSpireDistributedScan DML DELETE got non-delete primitive mode");
        }
        let deleted_count = Spi::connect(|client| {
            client
                .select(
                    "SELECT remote_deleted_count \
                       FROM ec_spire_prepare_coordinator_delete_tuple_payload(\
                            $1::oid, $2::text, $3::bytea)",
                    None,
                    &[
                        invocation.index_oid.into(),
                        invocation.pk_column.as_str().into(),
                        invocation.pk_value.to_vec().into(),
                    ],
                )
                .map_err(|e| format!("EcSpireDistributedScan DML DELETE primitive failed: {e}"))?
                .map(|row| {
                    row["remote_deleted_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "EcSpireDistributedScan DML DELETE deleted_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "EcSpireDistributedScan DML DELETE deleted_count is null".to_owned()
                        })
                })
                .next()
                .transpose()
                .map(|value| {
                    value.ok_or_else(|| {
                        "EcSpireDistributedScan DML DELETE primitive returned no rows".to_owned()
                    })
                })?
        })
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        u64::try_from(deleted_count).unwrap_or_else(|_| {
            pgrx::error!("EcSpireDistributedScan DML DELETE returned a negative deleted_count")
        })
    }
}

unsafe fn custom_scan_execute_dml_update(
    state: *mut SpireCustomScanExecState,
    scan_state: *mut pg_sys::ScanState,
) -> u64 {
    unsafe {
        let state_ref = &mut *state;
        let invocation =
            custom_scan_dml_primitive_invocation(state_ref).unwrap_or_else(|e| pgrx::error!("{e}"));
        if invocation.mode != super::SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload
        {
            pgrx::error!("EcSpireDistributedScan DML UPDATE got non-update primitive mode");
        }
        if state_ref.dml_update_value_exprs.len() != invocation.updated_columns.len() {
            pgrx::error!(
                "EcSpireDistributedScan DML UPDATE value expression count does not match updated columns"
            );
        }
        let row_payload_json = custom_scan_dml_update_row_payload_json(
            scan_state.cast::<pg_sys::CustomScanState>(),
            &invocation.updated_columns,
            &state_ref.dml_update_value_exprs,
        );
        let updated_column_refs = invocation
            .updated_columns
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let updated_count = Spi::connect(|client| {
            client
                .select(
                    "SELECT remote_updated_count \
                       FROM ec_spire_forward_coordinator_update_tuple_payload(\
                            $1::oid, $2::text, $3::bytea, $4::jsonb, $5::text[])",
                    None,
                    &[
                        invocation.index_oid.into(),
                        invocation.pk_column.as_str().into(),
                        invocation.pk_value.to_vec().into(),
                        row_payload_json.as_str().into(),
                        updated_column_refs.as_slice().into(),
                    ],
                )
                .map_err(|e| format!("EcSpireDistributedScan DML UPDATE primitive failed: {e}"))?
                .map(|row| {
                    row["remote_updated_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "EcSpireDistributedScan DML UPDATE updated_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "EcSpireDistributedScan DML UPDATE updated_count is null".to_owned()
                        })
                })
                .next()
                .transpose()
                .map(|value| {
                    value.ok_or_else(|| {
                        "EcSpireDistributedScan DML UPDATE primitive returned no rows".to_owned()
                    })
                })?
        })
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        u64::try_from(updated_count).unwrap_or_else(|_| {
            pgrx::error!("EcSpireDistributedScan DML UPDATE returned a negative updated_count")
        })
    }
}

unsafe fn custom_scan_dml_update_row_payload_json(
    node: *mut pg_sys::CustomScanState,
    updated_columns: &[String],
    value_exprs: &[*mut pg_sys::Expr],
) -> String {
    unsafe {
        if updated_columns.len() != value_exprs.len() {
            pgrx::error!("EcSpireDistributedScan DML UPDATE payload column/value width mismatch");
        }
        let mut payload = serde_json::Map::with_capacity(updated_columns.len());
        for (column, expr) in updated_columns.iter().zip(value_exprs.iter().copied()) {
            let value = custom_scan_dml_update_expr_json_value(node, expr);
            payload.insert(column.clone(), value);
        }
        serde_json::Value::Object(payload).to_string()
    }
}

unsafe fn custom_scan_dml_update_expr_json_value(
    node: *mut pg_sys::CustomScanState,
    expr: *mut pg_sys::Expr,
) -> serde_json::Value {
    unsafe {
        if expr.is_null() {
            pgrx::error!("EcSpireDistributedScan DML UPDATE value expression is null");
        }
        match (*expr.cast::<pg_sys::Node>()).type_ {
            pg_sys::NodeTag::T_Const => {
                let const_expr = &*expr.cast::<pg_sys::Const>();
                if const_expr.constisnull {
                    serde_json::Value::Null
                } else {
                    custom_scan_dml_update_datum_json_value(
                        const_expr.constvalue,
                        const_expr.consttype,
                    )
                }
            }
            pg_sys::NodeTag::T_Param => {
                let expr_state = pg_sys::ExecInitExpr(expr, &mut (*node).ss.ps);
                if expr_state.is_null() {
                    pgrx::error!(
                        "EcSpireDistributedScan failed to initialize DML UPDATE parameter"
                    );
                }
                let eval = (*expr_state).evalfunc.unwrap_or_else(|| {
                    pgrx::error!("EcSpireDistributedScan DML UPDATE parameter has no evaluator")
                });
                let mut is_null = false;
                let datum = eval(expr_state, (*node).ss.ps.ps_ExprContext, &mut is_null);
                if is_null {
                    serde_json::Value::Null
                } else {
                    let typoid = pg_sys::exprType(expr.cast());
                    custom_scan_dml_update_datum_json_value(datum, typoid)
                }
            }
            _ => pgrx::error!(
                "EcSpireDistributedScan DML UPDATE supports only constant or parameter SET values in v1"
            ),
        }
    }
}

unsafe fn custom_scan_dml_update_datum_json_value(
    datum: pg_sys::Datum,
    typoid: pg_sys::Oid,
) -> serde_json::Value {
    unsafe {
        if typoid == pg_sys::InvalidOid {
            pgrx::error!("EcSpireDistributedScan DML UPDATE value has invalid type OID");
        }
        let mut typoutput = pg_sys::InvalidOid;
        let mut typisvarlena = false;
        pg_sys::getTypeOutputInfo(typoid, &mut typoutput, &mut typisvarlena);
        let mut flinfo = std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init();
        pg_sys::fmgr_info(typoutput, &mut flinfo);
        let output = pg_sys::OutputFunctionCall(&mut flinfo, datum);
        if output.is_null() {
            pgrx::error!("EcSpireDistributedScan DML UPDATE type output returned NULL");
        }
        let value = std::ffi::CStr::from_ptr(output)
            .to_str()
            .unwrap_or_else(|_| {
                pgrx::error!("EcSpireDistributedScan DML UPDATE output value is not UTF-8")
            })
            .to_owned();
        serde_json::Value::String(value)
    }
}

unsafe fn custom_scan_ensure_dml_pk_select_payload(state: *mut SpireCustomScanExecState) {
    unsafe {
        let state_ref = &mut *state;
        if state_ref.dml_payload_loaded {
            return;
        }
        let invocation =
            custom_scan_dml_primitive_invocation(state_ref).unwrap_or_else(|e| pgrx::error!("{e}"));
        if invocation.mode
            != super::SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload
        {
            pgrx::error!("EcSpireDistributedScan DML PK SELECT got non-select primitive mode");
        }
        let requested_columns = custom_scan_dml_pk_select_requested_columns(
            &invocation,
            &state_ref.tuple_payload_columns,
        );
        let requested_column_refs = requested_columns
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let tuple_payload_json = Spi::connect(|client| {
            client
                .select(
                    "SELECT selected_count, tuple_payload_json \
                       FROM ec_spire_forward_coordinator_select_tuple_payload(\
                            $1::oid, $2::text, $3::bytea, $4::text[])",
                    None,
                    &[
                        invocation.index_oid.into(),
                        invocation.pk_column.as_str().into(),
                        invocation.pk_value.to_vec().into(),
                        requested_column_refs.as_slice().into(),
                    ],
                )
                .map_err(|e| format!("EcSpireDistributedScan DML PK SELECT primitive failed: {e}"))?
                .map(|row| {
                    let selected_count = row["selected_count"]
                        .value::<i64>()
                        .map_err(|e| {
                            format!(
                                "EcSpireDistributedScan DML PK SELECT selected_count decode failed: {e}"
                            )
                        })?
                        .ok_or_else(|| {
                            "EcSpireDistributedScan DML PK SELECT selected_count is null"
                                .to_owned()
                        })?;
                    let payload = row["tuple_payload_json"].value::<String>().map_err(|e| {
                        format!("EcSpireDistributedScan DML PK SELECT payload decode failed: {e}")
                    })?;
                    Ok::<Option<String>, String>((selected_count == 1).then_some(payload).flatten())
                })
                .next()
                .transpose()
                .map(|value| {
                    value.ok_or_else(|| {
                        "EcSpireDistributedScan DML PK SELECT primitive returned no rows".to_owned()
                    })
                })?
        })
        .unwrap_or_else(|e| pgrx::error!("{e}"));
        state_ref.dml_tuple_payload_json = tuple_payload_json;
        state_ref.dml_payload_loaded = true;
    }
}

fn custom_scan_dml_pk_select_requested_columns(
    invocation: &super::dml_frontdoor::SpireDmlFrontdoorPrimitiveInvocation,
    tuple_payload_columns: &[String],
) -> Vec<String> {
    let mut columns = if tuple_payload_columns.is_empty() {
        invocation.projected_columns.clone()
    } else {
        tuple_payload_columns.to_vec()
    };
    if !columns.iter().any(|column| column == &invocation.pk_column) {
        columns.insert(0, invocation.pk_column.clone());
    }
    columns
}

unsafe fn custom_scan_ensure_outputs(state: *mut SpireCustomScanExecState) {
    unsafe {
        if (*state).loaded_outputs {
            return;
        }
        let index_relation = pg_sys::index_open(
            (*state).index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
        );
        let stream = super::remote_search_production_scan_tuple_payload_result_stream(
            index_relation,
            (*state).query.clone(),
            (*state).top_k,
            &(*state).tuple_payload_columns,
        );
        pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
        if stream.summary.next_blocker != super::SPIRE_REMOTE_NONE {
            pgrx::error!(
                "EcSpireDistributedScan production executor blocked: status {}, next_blocker {}, recommendation {}",
                stream.summary.status,
                stream.summary.next_blocker,
                stream.summary.recommendation
            );
        }
        (*state).outputs = stream.outputs;
        (*state).loaded_outputs = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_spire::SpireDmlFrontdoorCustomScanMode;

    #[test]
    fn custom_scan_status_reports_executor_stream_tuple_payload_slots() {
        let row = custom_scan_status_row();

        assert_eq!(row.provider_name, "EcSpireDistributedScan");
        assert!(row.path_generation_enabled);
        assert!(row.exec_wiring_enabled);
        assert_eq!(row.next_step, "add ADR-069 write path");
    }

    #[test]
    fn custom_scan_eligibility_counts_remote_available_placements() {
        let row = SpireCustomScanIndexEligibilityRow {
            active_epoch: 7,
            local_placement_count: 1,
            remote_node_count: 1,
            remote_available_node_count: 1,
            remote_placement_count: 2,
            remote_available_placement_count: 2,
            remote_unavailable_placement_count: 0,
            all_remote_placements_available: true,
            eligible_for_custom_scan: true,
            status: "customscan_candidate",
            next_step:
                "planner path generation must also verify ORDER BY vector distance LIMIT query shape",
        };

        assert!(row.eligible_for_custom_scan);
        assert_eq!(row.status, "customscan_candidate");
        assert_eq!(row.remote_node_count, 1);
    }

    #[test]
    fn custom_scan_dml_modes_map_to_plan_private_values() {
        let cases = [
            (
                SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
                CUSTOM_SCAN_PLAN_MODE_DML_PK_SELECT,
                SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload,
            ),
            (
                SpireCustomScanPlanMode::DmlUpdateTuplePayload,
                CUSTOM_SCAN_PLAN_MODE_DML_UPDATE,
                SpireDmlFrontdoorCustomScanMode::CoordinatorUpdateTuplePayload,
            ),
            (
                SpireCustomScanPlanMode::DmlDeleteTuplePayload,
                CUSTOM_SCAN_PLAN_MODE_DML_DELETE,
                SpireDmlFrontdoorCustomScanMode::CoordinatorDeleteTuplePayload,
            ),
        ];

        for (plan_mode, raw, dml_mode) in cases {
            assert!(plan_mode.is_dml());
            assert_eq!(plan_mode.raw(), raw);
            assert_eq!(custom_scan_mode_from_u32(raw), Some(plan_mode));
            assert_eq!(custom_scan_plan_mode_for_dml_mode(dml_mode), plan_mode);
        }
        assert!(!SpireCustomScanPlanMode::VectorOrderLimit.is_dml());
        assert_eq!(
            custom_scan_mode_from_u32(CUSTOM_SCAN_PLAN_MODE_VECTOR_ORDER_LIMIT),
            Some(SpireCustomScanPlanMode::VectorOrderLimit)
        );
    }

    #[test]
    fn custom_scan_dml_column_metadata_validates_by_mode() {
        let updated = vec!["title".to_owned()];
        let projected = vec!["id".to_owned(), "title".to_owned()];
        let empty = Vec::<String>::new();

        custom_scan_validate_dml_column_metadata(
            SpireCustomScanPlanMode::DmlUpdateTuplePayload,
            &updated,
            &empty,
        )
        .expect("UPDATE metadata should validate");
        custom_scan_validate_dml_column_metadata(
            SpireCustomScanPlanMode::DmlDeleteTuplePayload,
            &empty,
            &empty,
        )
        .expect("DELETE metadata should validate");
        custom_scan_validate_dml_column_metadata(
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
            &empty,
            &projected,
        )
        .expect("PK SELECT metadata should validate");

        assert_eq!(
            custom_scan_validate_dml_column_metadata(
                SpireCustomScanPlanMode::DmlUpdateTuplePayload,
                &empty,
                &empty,
            )
            .expect_err("UPDATE without updated columns should fail"),
            "EcSpireDistributedScan DML UPDATE plan requires updated columns"
        );
        assert_eq!(
            custom_scan_validate_dml_column_metadata(
                SpireCustomScanPlanMode::DmlDeleteTuplePayload,
                &updated,
                &empty,
            )
            .expect_err("DELETE with updated columns should fail"),
            "EcSpireDistributedScan DML DELETE plan must not carry column payload metadata"
        );
    }

    #[test]
    fn custom_scan_dml_primitive_invocation_uses_plan_metadata() {
        let pk_value = [0, 0, 0, 0, 0, 0, 0, 5];
        let projected = vec!["id".to_owned(), "title".to_owned()];
        let invocation = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::Oid::from(42),
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
            "id",
            pk_value,
            &[],
            &projected,
        )
        .expect("PK SELECT invocation should build");

        assert_eq!(invocation.index_oid, pg_sys::Oid::from(42));
        assert_eq!(
            invocation.mode,
            SpireDmlFrontdoorCustomScanMode::CoordinatorPkSelectTuplePayload
        );
        assert_eq!(
            invocation.primitive,
            "ec_spire_forward_coordinator_select_tuple_payload"
        );
        assert_eq!(invocation.pk_column, "id");
        assert_eq!(invocation.pk_value, pk_value);
        assert!(invocation.updated_columns.is_empty());
        assert_eq!(invocation.projected_columns, projected);
    }

    #[test]
    fn custom_scan_dml_primitive_invocation_rejects_incomplete_state() {
        let error = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::InvalidOid,
            SpireCustomScanPlanMode::DmlUpdateTuplePayload,
            "id",
            [0, 0, 0, 0, 0, 0, 0, 5],
            &["title".to_owned()],
            &[],
        )
        .expect_err("missing index OID should fail");
        assert_eq!(
            error,
            "EcSpireDistributedScan DML primitive invocation requires index OID"
        );

        let error = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::Oid::from(42),
            SpireCustomScanPlanMode::DmlUpdateTuplePayload,
            "id",
            [0, 0, 0, 0, 0, 0, 0, 5],
            &[],
            &[],
        )
        .expect_err("UPDATE without column metadata should fail");
        assert_eq!(
            error,
            "EcSpireDistributedScan DML UPDATE plan requires updated columns"
        );
    }

    #[test]
    fn custom_scan_dml_pk_select_requested_columns_include_pk_for_quals() {
        let invocation = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::Oid::from(42),
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
            "id",
            [0, 0, 0, 0, 0, 0, 0, 5],
            &[],
            &["title".to_owned()],
        )
        .expect("PK SELECT invocation should build");

        assert_eq!(
            custom_scan_dml_pk_select_requested_columns(&invocation, &[]),
            vec!["id".to_owned(), "title".to_owned()]
        );
        assert_eq!(
            custom_scan_dml_pk_select_requested_columns(
                &invocation,
                &[
                    "id".to_owned(),
                    "title".to_owned(),
                    "source_identity".to_owned()
                ]
            ),
            vec![
                "id".to_owned(),
                "title".to_owned(),
                "source_identity".to_owned()
            ]
        );

        let invocation = custom_scan_dml_primitive_invocation_from_parts(
            pg_sys::Oid::from(42),
            SpireCustomScanPlanMode::DmlPkSelectTuplePayload,
            "id",
            [0, 0, 0, 0, 0, 0, 0, 5],
            &[],
            &["id".to_owned(), "title".to_owned()],
        )
        .expect("PK SELECT invocation should build");
        assert_eq!(
            custom_scan_dml_pk_select_requested_columns(&invocation, &[]),
            vec!["id".to_owned(), "title".to_owned()]
        );
    }

    #[test]
    fn custom_scan_cost_increases_with_remote_fanout() {
        let mut low_fanout = eligible_cost_row();
        low_fanout.remote_available_node_count = 1;
        low_fanout.remote_available_placement_count = 4;
        let mut high_fanout = low_fanout;
        high_fanout.remote_available_node_count = 4;
        high_fanout.remote_available_placement_count = 16;

        let low = estimate_custom_scan_cost_with_constants(
            10.0,
            1_000.0,
            &low_fanout,
            default_cost_constants(),
            0.01,
        );
        let high = estimate_custom_scan_cost_with_constants(
            10.0,
            1_000.0,
            &high_fanout,
            default_cost_constants(),
            0.01,
        );

        assert!(low.total_cost.is_finite());
        assert!(high.startup_cost > low.startup_cost);
        assert!(high.total_cost > low.total_cost);
    }

    #[test]
    fn custom_scan_cost_increases_with_output_rows() {
        let eligibility = eligible_cost_row();
        let small = estimate_custom_scan_cost_with_constants(
            1.0,
            1_000.0,
            &eligibility,
            default_cost_constants(),
            0.01,
        );
        let large = estimate_custom_scan_cost_with_constants(
            100.0,
            1_000.0,
            &eligibility,
            default_cost_constants(),
            0.01,
        );

        assert!(large.total_cost > small.total_cost);
        assert_eq!(large.startup_cost, small.startup_cost);
    }

    fn eligible_cost_row() -> SpireCustomScanIndexEligibilityRow {
        SpireCustomScanIndexEligibilityRow {
            active_epoch: 7,
            local_placement_count: 0,
            remote_node_count: 2,
            remote_available_node_count: 2,
            remote_placement_count: 8,
            remote_available_placement_count: 8,
            remote_unavailable_placement_count: 0,
            all_remote_placements_available: true,
            eligible_for_custom_scan: true,
            status: "customscan_candidate",
            next_step:
                "planner path generation must also verify ORDER BY vector distance LIMIT query shape",
        }
    }

    fn default_cost_constants() -> PlannerCostConstants {
        PlannerCostConstants {
            random_page_cost: 4.0,
            seq_page_cost: 1.0,
            cpu_operator_cost: 0.0025,
        }
    }
}
