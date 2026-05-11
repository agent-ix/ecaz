use pgrx::{pg_guard, pg_sys, PgBox};

use super::meta;

const CUSTOM_SCAN_NAME: &core::ffi::CStr = c"EcSpireDistributedScan";

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
        path_generation_enabled: false,
        exec_wiring_enabled: false,
        status: if registered && hook_installed {
            "registered"
        } else {
            "not_registered"
        },
        next_step: "add planner path generation for ec_spire indexes with active remote placements",
    }
}

pub(crate) unsafe fn custom_scan_index_eligibility_row(
    index_relation: pg_sys::Relation,
) -> SpireCustomScanIndexEligibilityRow {
    let root_control = unsafe { super::page::read_root_control_page(index_relation) };
    if root_control.active_epoch == 0 {
        return SpireCustomScanIndexEligibilityRow {
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
        };
    }

    let placement_directory = unsafe {
        load_custom_scan_placement_directory(index_relation, root_control)
            .unwrap_or_else(|e| pgrx::error!("{e}"))
    };
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
    SpireCustomScanIndexEligibilityRow {
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
    }
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
    // CustomPath generation lands in the next planner slice.
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_plan_custom_path(
    _root: *mut pg_sys::PlannerInfo,
    _rel: *mut pg_sys::RelOptInfo,
    _best_path: *mut pg_sys::CustomPath,
    _tlist: *mut pg_sys::List,
    _clauses: *mut pg_sys::List,
    _custom_plans: *mut pg_sys::List,
) -> *mut pg_sys::Plan {
    pgrx::error!("EcSpireDistributedScan planner path generation is registered but not enabled");
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
    pgrx::error!(
        "EcSpireDistributedScan executor callbacks are not wired to SpireRemoteFanoutExecutor yet"
    );
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
        assert!(!row.path_generation_enabled);
        assert!(!row.exec_wiring_enabled);
        assert_eq!(
            row.next_step,
            "add planner path generation for ec_spire indexes with active remote placements"
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
