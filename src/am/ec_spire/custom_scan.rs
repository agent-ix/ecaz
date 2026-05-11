use pgrx::{pg_guard, pg_sys, PgBox, PgList};

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
    pub(crate) remote_node_count: u64,
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
        exec_wiring_enabled: false,
        status: if registered && hook_installed {
            "planner_path_generation_enabled"
        } else {
            "not_registered"
        },
        next_step: "wire CustomScan executor callbacks to SpireRemoteFanoutExecutor",
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
    if root_control.active_epoch == 0 {
        return Err("ec_spire cannot load placement directory for empty active epoch".to_owned());
    }

    // ADR-067 planner eligibility needs only placement availability. Avoid the
    // heavier fanout loader used by executor paths, which also decodes epoch
    // and object manifests.
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
    _root: *mut pg_sys::PlannerInfo,
    _rel: *mut pg_sys::RelOptInfo,
    best_path: *mut pg_sys::CustomPath,
    tlist: *mut pg_sys::List,
    clauses: *mut pg_sys::List,
    custom_plans: *mut pg_sys::List,
) -> *mut pg_sys::Plan {
    unsafe {
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
        custom_scan.custom_exprs = clauses;
        custom_scan.custom_private = (*best_path).custom_private;
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

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_create_custom_scan_state(
    _cscan: *mut pg_sys::CustomScan,
) -> *mut pg_sys::Node {
    unsafe {
        let mut state =
            PgBox::<pg_sys::CustomScanState>::alloc_node(pg_sys::NodeTag::T_CustomScanState);
        state.methods = &raw const CUSTOM_EXEC_METHODS;
        state.into_pg() as *mut pg_sys::Node
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_begin_custom_scan(
    _node: *mut pg_sys::CustomScanState,
    _estate: *mut pg_sys::EState,
    _eflags: core::ffi::c_int,
) {
    // EXPLAIN initializes executor state even without ANALYZE. Keep Begin
    // side-effect-free until Exec is wired to SpireRemoteFanoutExecutor.
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_exec_custom_scan(
    _node: *mut pg_sys::CustomScanState,
) -> *mut pg_sys::TupleTableSlot {
    pgrx::error!(
        "EcSpireDistributedScan executor callbacks are not wired to SpireRemoteFanoutExecutor yet"
    );
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_end_custom_scan(_node: *mut pg_sys::CustomScanState) {}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_rescan_custom_scan(_node: *mut pg_sys::CustomScanState) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_scan_status_reports_provider_name_and_disabled_execution() {
        let row = custom_scan_status_row();

        assert_eq!(row.provider_name, "EcSpireDistributedScan");
        assert!(row.path_generation_enabled);
        assert!(!row.exec_wiring_enabled);
        assert_eq!(
            row.next_step,
            "wire CustomScan executor callbacks to SpireRemoteFanoutExecutor"
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
