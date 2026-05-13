use pgrx::{pg_guard, pg_sys, FromDatum, PgBox, PgList, Spi};

use std::{ffi::CString, ptr};

use crate::am::common::cost::{current_planner_cost_constants, PlannerCostConstants};

use super::meta;

const CUSTOM_SCAN_NAME: &core::ffi::CStr = c"EcSpireDistributedScan";
const EC_SPIRE_AM_NAME: &core::ffi::CStr = c"ec_spire";
const CUSTOM_SCAN_ROUTING_SCORE_BOUND: f64 = 64.0;
const CUSTOM_SCAN_REMOTE_DISPATCH_CPU_UNITS: f64 = 1024.0;
const CUSTOM_SCAN_MERGE_CPU_UNITS: f64 = 0.5;
const CUSTOM_SCAN_TUPLE_BYTE_CPU_UNITS: f64 = 0.001;
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
    ExplainCustomScan: Some(ec_spire_explain_custom_scan),
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
    tuple_payload_inputs: Vec<Option<SpireCustomScanPayloadAttrIo>>,
    outputs: Vec<super::SpireRemoteProductionScanOutputRow>,
    next_output: usize,
    loaded_outputs: bool,
    dml_payload_loaded: bool,
    dml_payload_emitted: bool,
    dml_tuple_payload_json: Option<String>,
}

struct SpireCustomScanPayloadAttrIo {
    input_flinfo: pg_sys::FmgrInfo,
    input_typioparam: pg_sys::Oid,
    receive_flinfo: pg_sys::FmgrInfo,
    receive_typioparam: pg_sys::Oid,
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

include!("planner.rs");
include!("cost_helpers.rs");
include!("plan_private.rs");
include!("dml.rs");
include!("explain.rs");
include!("begin_exec.rs");
include!("tuple_payload.rs");
include!("tests.rs");
