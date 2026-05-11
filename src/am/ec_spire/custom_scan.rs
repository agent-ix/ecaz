use pgrx::{pg_guard, pg_sys, FromDatum, PgBox, PgList};

use std::{ffi::CString, ptr};

use super::meta;

const CUSTOM_SCAN_NAME: &core::ffi::CStr = c"EcSpireDistributedScan";
const EC_SPIRE_AM_NAME: &core::ffi::CStr = c"ec_spire";

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
    // currently unavailable placements.
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
    index_oid: pg_sys::Oid,
    top_k: usize,
    query: Vec<f32>,
    tuple_payload_columns: Vec<String>,
    outputs: Vec<super::SpireRemoteProductionScanOutputRow>,
    next_output: usize,
    loaded_outputs: bool,
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
        next_step: "add end-to-end remote CustomScan tuple delivery fixture",
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
    if let Some(index_oid) = unsafe { custom_scan_candidate_index_oid(root, rel, rte) } {
        unsafe { add_custom_scan_path(root, rel, index_oid) };
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
        custom_scan.scan.plan.qual = clauses;
        custom_scan.scan.scanrelid = (*(*best_path).path.parent).relid;
        custom_scan.flags = (*best_path).flags;
        custom_scan.custom_plans = custom_plans;
        custom_scan.custom_exprs = custom_exprs;
        custom_scan.custom_private = pg_sys::lappend_oid(
            pg_sys::lappend_oid(
                std::ptr::null_mut(),
                pg_sys::list_nth_oid((*best_path).custom_private, 0),
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

unsafe fn custom_scan_candidate_index_oid(
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
        if matches!(eligibility, Ok(row) if row.eligible_for_custom_scan) {
            return Some(index_info.indexoid);
        }
    }
    None
}

unsafe fn add_custom_scan_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    index_oid: pg_sys::Oid,
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
    custom_path.path.startup_cost = 0.0;
    custom_path.path.total_cost = rows;
    custom_path.path.pathkeys = root_ref.sort_pathkeys;
    custom_path.flags = pg_sys::CUSTOMPATH_SUPPORT_PROJECTION;
    custom_path.custom_paths = std::ptr::null_mut();
    custom_path.custom_restrictinfo = rel_ref.baserestrictinfo;
    custom_path.custom_private = unsafe { pg_sys::lappend_oid(std::ptr::null_mut(), index_oid) };
    custom_path.methods = &raw const CUSTOM_PATH_METHODS;

    unsafe { pg_sys::add_path(rel, custom_path.into_pg() as *mut pg_sys::Path) };
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
    u32::try_from(var.varno).ok() == Some(relid)
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

unsafe fn custom_scan_index_oid_from_plan(custom_scan: *mut pg_sys::CustomScan) -> pg_sys::Oid {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing private index OID");
        }
        pg_sys::list_nth_oid((*custom_scan).custom_private, 0)
    }
}

unsafe fn custom_scan_top_k_from_plan(custom_scan: *mut pg_sys::CustomScan) -> usize {
    unsafe {
        if custom_scan.is_null() || (*custom_scan).custom_private.is_null() {
            pgrx::error!("EcSpireDistributedScan plan is missing private LIMIT");
        }
        let raw = pg_sys::list_nth_oid((*custom_scan).custom_private, 1);
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

unsafe fn custom_scan_tuple_payload_columns(node: *mut pg_sys::CustomScanState) -> Vec<String> {
    unsafe {
        let relation = (*node).ss.ss_currentRelation;
        if relation.is_null() {
            pgrx::error!("EcSpireDistributedScan missing scan relation for tuple payload columns");
        }
        let tuple_desc = (*relation).rd_att;
        if tuple_desc.is_null() {
            pgrx::error!("EcSpireDistributedScan missing scan relation tuple descriptor");
        }
        let natts = (*tuple_desc).natts;
        let mut columns = Vec::with_capacity(usize::try_from(natts).unwrap_or(0));
        for attr_index in 0..natts {
            let attr = pg_sys::TupleDescAttr(tuple_desc, attr_index);
            if attr.is_null() || (*attr).attisdropped {
                continue;
            }
            let name = std::ffi::CStr::from_ptr((*attr).attname.data.as_ptr())
                .to_str()
                .unwrap_or_else(|_| {
                    pgrx::error!("EcSpireDistributedScan relation attribute name is not UTF-8")
                })
                .to_owned();
            columns.push(name);
        }
        columns
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
                index_oid: pg_sys::InvalidOid,
                top_k: 0,
                query: Vec::new(),
                tuple_payload_columns: Vec::new(),
                outputs: Vec::new(),
                next_output: 0,
                loaded_outputs: false,
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
        (*state).index_oid = custom_scan_index_oid_from_plan(custom_scan);
        (*state).top_k = custom_scan_top_k_from_plan(custom_scan);
        (*state).query = custom_scan_query_from_plan(node, custom_scan);
        (*state).tuple_payload_columns = custom_scan_tuple_payload_columns(node);
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
        custom_scan_ensure_outputs(state);
        loop {
            let Some(output) = (&(*state).outputs).get((*state).next_output) else {
                return pg_sys::ExecClearTuple((*scan_state).ss_ScanTupleSlot);
            };
            (*state).next_output = (*state).next_output.saturating_add(1);
            if !matches!(
                output.heap_lookup_owner,
                super::SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION
                    | super::SPIRE_REMOTE_MATERIALIZED_HEAP_RESOLUTION
            ) {
                return custom_scan_store_remote_tuple_payload(scan_state, output);
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

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_custom_scan_recheck(
    _scan_state: *mut pg_sys::ScanState,
    _slot: *mut pg_sys::TupleTableSlot,
) -> bool {
    true
}

unsafe fn custom_scan_store_remote_tuple_payload(
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
        custom_scan_store_tuple_payload_json((*scan_state).ss_ScanTupleSlot, payload_json)
    }
}

unsafe fn custom_scan_store_tuple_payload_json(
    slot: *mut pg_sys::TupleTableSlot,
    payload_json: &str,
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
                    *(*slot).tts_values.add(attr_index as usize) =
                        custom_scan_json_value_to_datum(value, attr);
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
    unsafe { custom_scan_store_tuple_payload_json(slot, payload_json) }
}

unsafe fn custom_scan_json_value_to_datum(
    value: &serde_json::Value,
    attr: pg_sys::Form_pg_attribute,
) -> pg_sys::Datum {
    unsafe {
        let input_text = match value {
            serde_json::Value::String(value) => value.clone(),
            serde_json::Value::Bool(value) => value.to_string(),
            serde_json::Value::Number(value) => value.to_string(),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => value.to_string(),
            serde_json::Value::Null => {
                pgrx::error!("EcSpireDistributedScan cannot convert JSON null to non-null datum")
            }
        };
        let input = CString::new(input_text)
            .unwrap_or_else(|_| pgrx::error!("EcSpireDistributedScan tuple payload contains NUL"));
        let mut typinput = pg_sys::InvalidOid;
        let mut typioparam = pg_sys::InvalidOid;
        pg_sys::getTypeInputInfo((*attr).atttypid, &mut typinput, &mut typioparam);
        let mut flinfo = std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init();
        pg_sys::fmgr_info(typinput, &mut flinfo);
        pg_sys::InputFunctionCall(
            &mut flinfo,
            input.as_ptr().cast_mut(),
            typioparam,
            (*attr).atttypmod,
        )
    }
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

    #[test]
    fn custom_scan_status_reports_executor_stream_tuple_payload_slots() {
        let row = custom_scan_status_row();

        assert_eq!(row.provider_name, "EcSpireDistributedScan");
        assert!(row.path_generation_enabled);
        assert!(row.exec_wiring_enabled);
        assert_eq!(
            row.next_step,
            "add end-to-end remote CustomScan tuple delivery fixture"
        );
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
}
