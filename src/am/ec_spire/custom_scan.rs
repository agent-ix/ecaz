use pgrx::{pg_guard, pg_sys, PgBox};

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

static mut PREVIOUS_SET_REL_PATHLIST_HOOK: pg_sys::set_rel_pathlist_hook_type = None;
static mut CUSTOM_SCAN_REGISTERED: bool = false;
static mut REL_PATHLIST_HOOK_INSTALLED: bool = false;

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
}
