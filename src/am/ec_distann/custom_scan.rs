//! FR-081 multi-node CustomScan read path (Task 165 M3).
//!
//! The AM `amgettuple` path returns a heap TID that PostgreSQL fetches from the
//! *coordinator's local heap* — which cannot work for a remote-owned hit, whose
//! row lives on another instance. This CustomScan replaces the index scan for a
//! vector `ORDER BY ... LIMIT k` query when the roster is multi-node: it runs
//! the same FR-080/FR-081 orchestrated search (shared `collect_distann_hits`),
//! then for each hit either fetches the local heap row (local hits carry a valid
//! ctid) or asks the owning node to ship the row's projection columns as
//! PostgreSQL binary (`ec_distann_materialize_row_payloads`) and reconstructs a
//! virtual tuple from that payload. So a real 3-worker scan returns owner-owned
//! SQL rows without a local directory or a local heap fetch.
//!
//! Single-node / empty roster is not eligible here — those queries stay on the
//! local AM `amgettuple` path.

use pgrx::{pg_guard, pg_sys, FromDatum, PgBox, PgList, Spi};
use std::ptr;

use crate::am::common::{
    heap_slot::TupleSlotWriter,
    pg_ptr::{pg_list as cs_pg_list, pg_ref as cs_pg_ref},
};

use super::placement::owning_node;

const CUSTOM_SCAN_NAME: &core::ffi::CStr = c"EcDistannDistributedScan";
const EC_DISTANN_AM_NAME: &core::ffi::CStr = c"ec_distann";
const PLAN_MODE_VECTOR_ORDER_LIMIT: u32 = 1;

static mut PREVIOUS_SET_REL_PATHLIST_HOOK: pg_sys::set_rel_pathlist_hook_type = None;
static mut CUSTOM_SCAN_REGISTERED: bool = false;
static mut REL_PATHLIST_HOOK_INSTALLED: bool = false;

static mut CUSTOM_PATH_METHODS: pg_sys::CustomPathMethods = pg_sys::CustomPathMethods {
    CustomName: CUSTOM_SCAN_NAME.as_ptr(),
    PlanCustomPath: Some(plan_custom_path),
    ReparameterizeCustomPathByChild: None,
};

static mut CUSTOM_SCAN_METHODS: pg_sys::CustomScanMethods = pg_sys::CustomScanMethods {
    CustomName: CUSTOM_SCAN_NAME.as_ptr(),
    CreateCustomScanState: Some(create_custom_scan_state),
};

static mut CUSTOM_EXEC_METHODS: pg_sys::CustomExecMethods = pg_sys::CustomExecMethods {
    CustomName: CUSTOM_SCAN_NAME.as_ptr(),
    BeginCustomScan: Some(begin_custom_scan),
    ExecCustomScan: Some(exec_custom_scan),
    EndCustomScan: Some(end_custom_scan),
    ReScanCustomScan: Some(rescan_custom_scan),
    MarkPosCustomScan: None,
    RestrPosCustomScan: None,
    EstimateDSMCustomScan: None,
    InitializeDSMCustomScan: None,
    ReInitializeDSMCustomScan: None,
    InitializeWorkerCustomScan: None,
    ShutdownCustomScan: None,
    ExplainCustomScan: None,
};

/// Installs the CustomScan provider + planner hook (once per backend, from
/// `_PG_init`). Mirrors the ec_spire registration.
pub(crate) fn register_custom_scan() {
    // SAFETY: `_PG_init` runs this during backend extension load; the writes
    // install process-global PostgreSQL hook/method pointers exactly once.
    unsafe {
        if !CUSTOM_SCAN_REGISTERED {
            pg_sys::RegisterCustomScanMethods(&raw const CUSTOM_SCAN_METHODS);
            CUSTOM_SCAN_REGISTERED = true;
        }
        if !REL_PATHLIST_HOOK_INSTALLED {
            PREVIOUS_SET_REL_PATHLIST_HOOK = pg_sys::set_rel_pathlist_hook;
            pg_sys::set_rel_pathlist_hook = Some(set_rel_pathlist_hook);
            REL_PATHLIST_HOOK_INSTALLED = true;
        }
    }
}

/// True when the session roster is multi-node — the only regime where a scan can
/// see a remote-owned hit and thus needs this CustomScan.
fn distann_multi_node_roster() -> bool {
    super::roster::current_placement_directory()
        .map(|placement| placement.node_count() > 1)
        .unwrap_or(false)
}

fn ec_distann_am_oid() -> Option<pg_sys::Oid> {
    // SAFETY: get_index_am_oid reads the syscache by static C string; a missing
    // AM is InvalidOid because missing_ok is true.
    let am_oid = unsafe { pg_sys::get_index_am_oid(EC_DISTANN_AM_NAME.as_ptr(), true) };
    (am_oid != pg_sys::InvalidOid).then_some(am_oid)
}

// ---------------------------------------------------------------------------
// Planner: vector ORDER BY LIMIT introspection (ported from ec_spire cost path)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct PlannerRel<'a> {
    root_ref: &'a pg_sys::PlannerInfo,
    rel_ref: &'a pg_sys::RelOptInfo,
}

impl<'a> PlannerRel<'a> {
    unsafe fn new(root: *mut pg_sys::PlannerInfo, rel: *mut pg_sys::RelOptInfo) -> Option<Self> {
        Some(Self {
            root_ref: cs_pg_ref(root)?,
            rel_ref: cs_pg_ref(rel)?,
        })
    }

    fn is_plain_base_relation(self) -> bool {
        self.rel_ref.reloptkind == pg_sys::RelOptKind::RELOPT_BASEREL
            && self.rel_ref.rtekind == pg_sys::RTEKind::RTE_RELATION
    }

    fn has_vector_order_limit_shape(self) -> bool {
        !self.root_ref.sort_pathkeys.is_null() && self.root_ref.limit_tuples >= 0.0
    }

    fn top_k(self) -> Option<usize> {
        if self.root_ref.limit_tuples < 0.0 || !self.root_ref.limit_tuples.is_finite() {
            return None;
        }
        Some(self.root_ref.limit_tuples.max(0.0).ceil() as usize)
    }

    fn output_rows(self) -> f64 {
        if self.root_ref.limit_tuples >= 0.0 {
            self.root_ref.limit_tuples.max(1.0)
        } else {
            self.rel_ref.rows.max(1.0)
        }
    }

    fn orderby_query_expr(self) -> Option<*mut pg_sys::Expr> {
        // SAFETY: this view is built from live planner callback pointers; the
        // Query/List nodes are inspected immediately.
        unsafe {
            let query = cs_pg_ref(self.root_ref.parse)?;
            if query.sortClause.is_null() || query.targetList.is_null() {
                return None;
            }
            let sort_clauses = cs_pg_list::<pg_sys::SortGroupClause>(query.sortClause);
            if sort_clauses.len() != 1 {
                return None;
            }
            let sort_clause = cs_pg_ref(sort_clauses.get_ptr(0)?)?;
            let target_list = cs_pg_list::<pg_sys::TargetEntry>(query.targetList);
            for target_entry in target_list.iter_ptr() {
                let Some(target_entry) = cs_pg_ref(target_entry) else {
                    continue;
                };
                if target_entry.ressortgroupref != sort_clause.tleSortGroupRef {
                    continue;
                }
                return self.query_expr_from_sort_expr(target_entry.expr);
            }
            None
        }
    }

    fn query_expr_from_sort_expr(self, expr: *mut pg_sys::Expr) -> Option<*mut pg_sys::Expr> {
        // SAFETY: expr is reached through this live callback view; only inspected
        // for node/list shape.
        unsafe {
            let node = cs_pg_ref(expr.cast::<pg_sys::Node>())?;
            if node.type_ != pg_sys::NodeTag::T_OpExpr {
                return None;
            }
            let op_expr = cs_pg_ref(expr.cast::<pg_sys::OpExpr>())?;
            let args = cs_pg_list::<pg_sys::Expr>(op_expr.args);
            if args.len() != 2 {
                return None;
            }
            let left = args.get_ptr(0)?;
            let right = args.get_ptr(1)?;
            if is_relation_var(left, self.rel_ref.relid) && is_query_value(right) {
                return Some(right);
            }
            if is_relation_var(right, self.rel_ref.relid) && is_query_value(left) {
                return Some(left);
            }
            None
        }
    }
}

unsafe fn is_relation_var(expr: *mut pg_sys::Expr, relid: pg_sys::Index) -> bool {
    let Some(node) = cs_pg_ref(expr.cast::<pg_sys::Node>()) else {
        return false;
    };
    if node.type_ != pg_sys::NodeTag::T_Var {
        return false;
    }
    let Some(var) = cs_pg_ref(expr.cast::<pg_sys::Var>()) else {
        return false;
    };
    u32::try_from(var.varno).ok() == Some(relid) && var.varlevelsup == 0
}

unsafe fn is_query_value(expr: *mut pg_sys::Expr) -> bool {
    let Some(node) = cs_pg_ref(expr.cast::<pg_sys::Node>()) else {
        return false;
    };
    match node.type_ {
        pg_sys::NodeTag::T_Const => query_values_from_const(expr).is_some(),
        pg_sys::NodeTag::T_Param => cs_pg_ref(expr.cast::<pg_sys::Param>())
            .map(|param| param.paramtype == pg_sys::FLOAT4ARRAYOID)
            .unwrap_or(false),
        _ => false,
    }
}

unsafe fn query_values_from_const(expr: *mut pg_sys::Expr) -> Option<Vec<f32>> {
    let const_ref = cs_pg_ref(expr.cast::<pg_sys::Const>())?;
    if const_ref.constisnull || const_ref.consttype != pg_sys::FLOAT4ARRAYOID {
        return None;
    }
    let values =
        Vec::<f32>::from_polymorphic_datum(const_ref.constvalue, false, pg_sys::FLOAT4ARRAYOID)?;
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return None;
    }
    Some(values)
}

/// Is there an ec_distann index on this relation whose scan should be replaced?
unsafe fn custom_scan_candidate_index(planner_rel: PlannerRel<'_>) -> Option<pg_sys::Oid> {
    if !distann_multi_node_roster() {
        return None;
    }
    if !planner_rel.is_plain_base_relation() || !planner_rel.has_vector_order_limit_shape() {
        return None;
    }
    planner_rel.orderby_query_expr()?;
    let am_oid = ec_distann_am_oid()?;
    let index_list = cs_pg_list::<pg_sys::IndexOptInfo>(planner_rel.rel_ref.indexlist);
    for index_info in index_list.iter_ptr() {
        let Some(index_info) = cs_pg_ref(index_info) else {
            continue;
        };
        if index_info.relam == am_oid {
            return Some(index_info.indexoid);
        }
    }
    None
}

/// # Safety
/// PostgreSQL invokes set_rel_pathlist hooks with live planner pointers.
#[pg_guard]
unsafe extern "C-unwind" fn set_rel_pathlist_hook(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rti: pg_sys::Index,
    rte: *mut pg_sys::RangeTblEntry,
) {
    if let Some(previous_hook) = PREVIOUS_SET_REL_PATHLIST_HOOK {
        previous_hook(root, rel, rti, rte);
    }
    let Some(planner_rel) = PlannerRel::new(root, rel) else {
        return;
    };
    if cs_pg_ref(rte).is_none() {
        return;
    }
    let Some(index_oid) = custom_scan_candidate_index(planner_rel) else {
        return;
    };

    // SAFETY: pointers are the live planner hook arguments; the CustomPath node
    // and its private OID list are allocated in planner memory and transferred
    // with add_path.
    let mut custom_path = PgBox::<pg_sys::CustomPath>::alloc_node(pg_sys::NodeTag::T_CustomPath);
    let rows = planner_rel.output_rows();
    custom_path.path.type_ = pg_sys::NodeTag::T_CustomPath;
    custom_path.path.pathtype = pg_sys::NodeTag::T_CustomScan;
    custom_path.path.parent = rel;
    custom_path.path.pathtarget = planner_rel.rel_ref.reltarget;
    custom_path.path.param_info = ptr::null_mut();
    custom_path.path.parallel_aware = false;
    custom_path.path.parallel_safe = false;
    custom_path.path.parallel_workers = 0;
    custom_path.path.rows = rows;
    custom_path.path.disabled_nodes = 0;
    // The multi-node index scan cannot return remote rows, so this path must
    // win. A small cost proportional to the LIMIT keeps EXPLAIN sane while
    // beating the local index scan's ORDER BY cost.
    custom_path.path.startup_cost = 0.0;
    custom_path.path.total_cost = rows;
    custom_path.path.pathkeys = planner_rel.root_ref.sort_pathkeys;
    custom_path.flags = pg_sys::CUSTOMPATH_SUPPORT_PROJECTION;
    custom_path.custom_paths = ptr::null_mut();
    custom_path.custom_restrictinfo = planner_rel.rel_ref.baserestrictinfo;
    custom_path.custom_private = pg_sys::lappend_oid(
        pg_sys::lappend_oid(ptr::null_mut(), pg_sys::Oid::from(PLAN_MODE_VECTOR_ORDER_LIMIT)),
        index_oid,
    );
    custom_path.methods = &raw const CUSTOM_PATH_METHODS;
    pg_sys::add_path(rel, custom_path.into_pg() as *mut pg_sys::Path);
}

/// # Safety
/// PostgreSQL calls PlanCustomPath with live planner/path/list pointers.
#[pg_guard]
unsafe extern "C-unwind" fn plan_custom_path(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    best_path: *mut pg_sys::CustomPath,
    tlist: *mut pg_sys::List,
    clauses: *mut pg_sys::List,
    custom_plans: *mut pg_sys::List,
) -> *mut pg_sys::Plan {
    let best_path = cs_pg_ref(best_path)
        .unwrap_or_else(|| pgrx::error!("EcDistannDistributedScan PlanCustomPath missing path"));
    let planner_rel = PlannerRel::new(root, rel).unwrap_or_else(|| {
        pgrx::error!("EcDistannDistributedScan PlanCustomPath missing planner relation")
    });
    if pg_sys::list_length(best_path.custom_private) < 2 {
        pgrx::error!("EcDistannDistributedScan CustomPath missing plan-private oids");
    }
    let index_oid = pg_sys::list_nth_oid(best_path.custom_private, 1);
    let top_k = planner_rel.top_k().unwrap_or(1);
    let query_expr = planner_rel.orderby_query_expr().unwrap_or_else(|| {
        pgrx::error!("EcDistannDistributedScan could not extract ORDER BY vector query expression")
    });
    let custom_exprs = pg_sys::lappend(
        ptr::null_mut(),
        pg_sys::copyObjectImpl(query_expr.cast()).cast(),
    );

    let mut custom_scan = PgBox::<pg_sys::CustomScan>::alloc_node(pg_sys::NodeTag::T_CustomScan);
    custom_scan.scan.plan.type_ = pg_sys::NodeTag::T_CustomScan;
    custom_scan.scan.plan.disabled_nodes = best_path.path.disabled_nodes;
    custom_scan.scan.plan.startup_cost = best_path.path.startup_cost;
    custom_scan.scan.plan.total_cost = best_path.path.total_cost;
    custom_scan.scan.plan.plan_rows = best_path.path.rows;
    custom_scan.scan.plan.plan_width = cs_pg_ref(planner_rel.rel_ref.reltarget)
        .map(|target| target.width)
        .unwrap_or(0);
    custom_scan.scan.plan.parallel_aware = false;
    custom_scan.scan.plan.parallel_safe = false;
    custom_scan.scan.plan.async_capable = false;
    custom_scan.scan.plan.targetlist = tlist;
    custom_scan.scan.plan.qual = pg_sys::extract_actual_clauses(clauses, false);
    custom_scan.scan.scanrelid = planner_rel.rel_ref.relid;
    custom_scan.flags = best_path.flags;
    custom_scan.custom_plans = custom_plans;
    custom_scan.custom_exprs = custom_exprs;
    custom_scan.custom_private = pg_sys::lappend_oid(
        pg_sys::lappend_oid(
            pg_sys::lappend_oid(ptr::null_mut(), pg_sys::Oid::from(PLAN_MODE_VECTOR_ORDER_LIMIT)),
            index_oid,
        ),
        pg_sys::Oid::from(u32::try_from(top_k).unwrap_or_else(|_| {
            pgrx::error!("EcDistannDistributedScan LIMIT exceeds CustomScan plan-private range")
        })),
    );
    custom_scan.custom_scan_tlist = ptr::null_mut();
    custom_scan.custom_relids = ptr::null_mut();
    custom_scan.methods = &raw const CUSTOM_SCAN_METHODS;
    custom_scan.into_pg() as *mut pg_sys::Plan
}

// ---------------------------------------------------------------------------
// Executor
// ---------------------------------------------------------------------------

struct PayloadAttrIo {
    receive_flinfo: pg_sys::FmgrInfo,
    receive_typioparam: pg_sys::Oid,
    typmod: i32,
}

enum CustomScanOutputRow {
    /// A local-heap hit: fetch the visible row by ctid from the scan relation.
    Local(crate::storage::page::ItemPointer),
    /// A remote-owned hit: reconstruct a virtual tuple from the owner-shipped
    /// per-column binary payload.
    Remote {
        payload_nulls: Vec<bool>,
        payload_values: Vec<Vec<u8>>,
    },
}

#[repr(C)]
struct DistannCustomScanExecState {
    custom_scan_state: pg_sys::CustomScanState,
    index_oid: pg_sys::Oid,
    top_k: usize,
    /// The ORDER BY query expression, initialized once in BeginCustomScan and
    /// evaluated lazily per (re)scan — a correlated LATERAL Param is only bound
    /// per outer row, so the vector cannot be materialized at Begin time.
    query_expr_state: *mut pg_sys::ExprState,
    /// Heap attnums of the projected payload columns (parallel to the io vecs).
    payload_attnums: Vec<pg_sys::AttrNumber>,
    payload_columns: Vec<String>,
    payload_send_functions: Vec<String>,
    payload_inputs: Vec<PayloadAttrIo>,
    outputs: Vec<CustomScanOutputRow>,
    next_output: usize,
    loaded: bool,
    /// A buffer-heap slot for local-hit `table_tuple_fetch_row_version` (the
    /// CustomScan's own scan slot is virtual, which that heap fetch asserts
    /// against; the fetched row is then copied into the virtual scan slot the
    /// projection is compiled for). Estate-managed.
    local_heap_slot: *mut pg_sys::TupleTableSlot,
}

fn default_exec_state() -> DistannCustomScanExecState {
    DistannCustomScanExecState {
        // SAFETY: CustomScanState is initialized field-by-field by the executor
        // after this wrapper is allocated.
        custom_scan_state: unsafe { std::mem::zeroed() },
        index_oid: pg_sys::InvalidOid,
        top_k: 0,
        query_expr_state: ptr::null_mut(),
        payload_attnums: Vec::new(),
        payload_columns: Vec::new(),
        payload_send_functions: Vec::new(),
        payload_inputs: Vec::new(),
        outputs: Vec::new(),
        next_output: 0,
        loaded: false,
        local_heap_slot: ptr::null_mut(),
    }
}

/// # Safety
/// Called by the executor during custom scan state creation.
#[pg_guard]
unsafe extern "C-unwind" fn create_custom_scan_state(
    _cscan: *mut pg_sys::CustomScan,
) -> *mut pg_sys::Node {
    let state = pg_sys::palloc0(std::mem::size_of::<DistannCustomScanExecState>())
        .cast::<DistannCustomScanExecState>();
    ptr::write(state, default_exec_state());
    (*state).custom_scan_state.ss.ps.type_ = pg_sys::NodeTag::T_CustomScanState;
    (*state).custom_scan_state.methods = &raw const CUSTOM_EXEC_METHODS;
    state.cast::<pg_sys::Node>()
}

unsafe fn exec_state_mut<'a>(
    node: *mut pg_sys::CustomScanState,
) -> &'a mut DistannCustomScanExecState {
    node.cast::<DistannCustomScanExecState>()
        .as_mut()
        .unwrap_or_else(|| pgrx::error!("EcDistannDistributedScan missing exec state"))
}

/// # Safety
/// PostgreSQL invokes BeginCustomScan with a live CustomScanState whose plan is
/// the provider CustomScan built by the planner; all query/payload metadata is
/// copied into Rust-owned state before returning.
#[pg_guard]
unsafe extern "C-unwind" fn begin_custom_scan(
    node: *mut pg_sys::CustomScanState,
    _estate: *mut pg_sys::EState,
    _eflags: core::ffi::c_int,
) {
    let custom_scan = (*node).ss.ps.plan.cast::<pg_sys::CustomScan>();
    if custom_scan.is_null() {
        pgrx::error!("EcDistannDistributedScan BeginCustomScan missing plan");
    }
    if pg_sys::list_length((*custom_scan).custom_private) < 3 {
        pgrx::error!("EcDistannDistributedScan plan missing plan-private oids");
    }
    let index_oid = pg_sys::list_nth_oid((*custom_scan).custom_private, 1);
    let top_k = u32::from(pg_sys::list_nth_oid((*custom_scan).custom_private, 2)) as usize;

    // Initialize (but do not yet evaluate) the ORDER BY query expression from
    // custom_exprs[0]. A Const evaluates to the same vector every scan; a
    // correlated Param is only bound per outer row, so evaluation is deferred to
    // ensure_outputs (after each rescan binds the current param).
    let exprs = cs_pg_list::<pg_sys::Expr>((*custom_scan).custom_exprs);
    let query_expr = exprs
        .get_ptr(0)
        .unwrap_or_else(|| pgrx::error!("EcDistannDistributedScan plan missing ORDER BY query"));
    let query_expr_state = pg_sys::ExecInitExpr(query_expr, &mut (*node).ss.ps);
    if query_expr_state.is_null() {
        pgrx::error!("EcDistannDistributedScan failed to initialize the ORDER BY query expression");
    }

    let (payload_attnums, payload_columns, payload_send_functions, payload_inputs) =
        build_payload_metadata(node, custom_scan);

    let state = exec_state_mut(node);
    state.index_oid = index_oid;
    state.top_k = top_k;
    state.query_expr_state = query_expr_state;
    state.payload_attnums = payload_attnums;
    state.payload_columns = payload_columns;
    state.payload_send_functions = payload_send_functions;
    state.payload_inputs = payload_inputs;
    state.outputs.clear();
    state.next_output = 0;
    state.loaded = false;
}

/// Evaluate the ORDER BY query expression for the current (re)scan into a
/// finite `real[]`. Called from `ensure_outputs`, after any correlated Param has
/// been bound for the current outer row.
unsafe fn eval_query_vector(
    scan_state: *mut pg_sys::ScanState,
    expr_state: *mut pg_sys::ExprState,
) -> Vec<f32> {
    if expr_state.is_null() {
        pgrx::error!("EcDistannDistributedScan missing initialized ORDER BY query expression");
    }
    let eval = (*expr_state).evalfunc.unwrap_or_else(|| {
        pgrx::error!("EcDistannDistributedScan ORDER BY query expression has no evaluator")
    });
    let econtext = (*scan_state).ps.ps_ExprContext;
    if econtext.is_null() {
        pgrx::error!("EcDistannDistributedScan missing expression context for the ORDER BY query");
    }
    let mut is_null = false;
    let datum = eval(expr_state, econtext, &mut is_null);
    if is_null {
        pgrx::error!("EcDistannDistributedScan ORDER BY query must not be NULL");
    }
    Vec::<f32>::from_polymorphic_datum(datum, false, pg_sys::FLOAT4ARRAYOID)
        .filter(|values| !values.is_empty() && values.iter().all(|v| v.is_finite()))
        .unwrap_or_else(|| {
            pgrx::error!("EcDistannDistributedScan requires a finite real[] ORDER BY query")
        })
}

/// Build the projected-column payload metadata: the heap attnums + names to ship
/// (narrowed to the target list's Var references when possible), each column's
/// schema-qualified `typsend` function (for the owner-side encode), and each
/// column's binary receive FmgrInfo (for the coordinator-side reconstruct).
unsafe fn build_payload_metadata(
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) -> (
    Vec<pg_sys::AttrNumber>,
    Vec<String>,
    Vec<String>,
    Vec<PayloadAttrIo>,
) {
    let relation = (*node).ss.ss_currentRelation;
    if relation.is_null() {
        pgrx::error!("EcDistannDistributedScan missing scan relation for payload metadata");
    }
    let relation_oid = (*relation).rd_id;
    let tuple_desc = (*relation).rd_att;
    let natts = (*tuple_desc).natts;

    // Narrow to the target-list Var attnums when every target entry is a plain
    // Var; otherwise ship every column.
    let mut narrow: Option<std::collections::BTreeSet<pg_sys::AttrNumber>> = Some(Default::default());
    if !custom_scan.is_null() && !(*custom_scan).scan.plan.targetlist.is_null() {
        let target_list = PgList::<pg_sys::TargetEntry>::from_pg((*custom_scan).scan.plan.targetlist);
        for target_entry in target_list.iter_ptr() {
            let Some(target_entry) = target_entry.as_ref() else {
                continue;
            };
            if target_entry.resjunk || target_entry.expr.is_null() {
                continue;
            }
            let expr = target_entry.expr.cast::<pg_sys::Node>();
            if (*expr).type_ != pg_sys::NodeTag::T_Var {
                narrow = None;
                break;
            }
            let var = &*target_entry.expr.cast::<pg_sys::Var>();
            if var.varattno > 0 {
                if let Some(set) = narrow.as_mut() {
                    set.insert(var.varattno);
                }
            } else {
                narrow = None;
                break;
            }
        }
    } else {
        narrow = None;
    }

    let mut attnums = Vec::new();
    let mut columns = Vec::new();
    let mut inputs = Vec::new();
    for attr_index in 0..natts {
        let attr = pg_sys::TupleDescAttr(tuple_desc, attr_index);
        if attr.is_null() || (*attr).attisdropped {
            continue;
        }
        let attnum = (*attr).attnum;
        if let Some(set) = narrow.as_ref() {
            if !set.contains(&attnum) {
                continue;
            }
        }
        let name = std::ffi::CStr::from_ptr((*attr).attname.data.as_ptr())
            .to_string_lossy()
            .into_owned();
        // Binary receive metadata (in-process; handles array typioparam).
        let mut typreceive = pg_sys::InvalidOid;
        let mut typioparam = pg_sys::InvalidOid;
        pg_sys::getTypeBinaryInputInfo((*attr).atttypid, &mut typreceive, &mut typioparam);
        if typreceive == pg_sys::InvalidOid {
            pgrx::error!(
                "EcDistannDistributedScan column \"{name}\" type has no binary receive function"
            );
        }
        let mut receive_flinfo = std::mem::MaybeUninit::<pg_sys::FmgrInfo>::zeroed().assume_init();
        pg_sys::fmgr_info(typreceive, &mut receive_flinfo);
        attnums.push(attnum);
        columns.push(name);
        inputs.push(PayloadAttrIo {
            receive_flinfo,
            receive_typioparam: typioparam,
            typmod: (*attr).atttypmod,
        });
    }

    let send_functions = resolve_send_functions(relation_oid, &columns);
    (attnums, columns, send_functions, inputs)
}

/// Look up each projected column's schema-qualified `typsend` function name (as
/// the owner must resolve it by name). One SPI query, ordered to match
/// `columns`; a type without a binary send function fails closed.
fn resolve_send_functions(relation_oid: pg_sys::Oid, columns: &[String]) -> Vec<String> {
    if columns.is_empty() {
        return Vec::new();
    }
    let column_refs: Vec<&str> = columns.iter().map(String::as_str).collect();
    Spi::connect(|client| {
        let rows = client
            .select(
                "SELECT quote_ident(n.nspname) || '.' || quote_ident(p.proname) AS send_fn \
                   FROM unnest($1::text[]) WITH ORDINALITY AS req(name, ord) \
                   JOIN pg_attribute a \
                     ON a.attrelid = $2::oid AND a.attname = req.name \
                    AND a.attnum > 0 AND NOT a.attisdropped \
                   JOIN pg_type t ON t.oid = a.atttypid \
                   JOIN pg_proc p ON p.oid = t.typsend \
                   JOIN pg_namespace n ON n.oid = p.pronamespace \
                  ORDER BY req.ord",
                None,
                &[column_refs.as_slice().into(), relation_oid.into()],
            )
            .map_err(|e| format!("ec_distann CustomScan send-function lookup failed: {e}"))?;
        let mut send_functions = Vec::with_capacity(columns.len());
        for row in rows {
            let send_fn = row["send_fn"]
                .value::<String>()
                .map_err(|e| format!("ec_distann CustomScan send-function decode failed: {e}"))?
                .ok_or_else(|| "ec_distann CustomScan send function is null".to_owned())?;
            send_functions.push(send_fn);
        }
        Ok::<_, String>(send_functions)
    })
    .unwrap_or_else(|e: String| pgrx::error!("{e}"))
    .tap_len_check(columns.len())
}

trait LenCheck {
    fn tap_len_check(self, expected: usize) -> Self;
}
impl LenCheck for Vec<String> {
    fn tap_len_check(self, expected: usize) -> Self {
        if self.len() != expected {
            pgrx::error!(
                "EcDistannDistributedScan: {} of {} projected columns have a binary send function \
                 (a projected column's type has no typsend)",
                self.len(),
                expected
            );
        }
        self
    }
}

/// # Safety
/// PostgreSQL invokes ExecCustomScan with the live CustomScanState.
#[pg_guard]
unsafe extern "C-unwind" fn exec_custom_scan(
    node: *mut pg_sys::CustomScanState,
) -> *mut pg_sys::TupleTableSlot {
    pg_sys::ExecScan(
        &mut (*node).ss,
        Some(custom_scan_access),
        Some(custom_scan_recheck),
    )
}

/// # Safety
/// ExecScan invokes this with a live ScanState.
#[pg_guard]
unsafe extern "C-unwind" fn custom_scan_access(
    scan_state: *mut pg_sys::ScanState,
) -> *mut pg_sys::TupleTableSlot {
    let state = exec_state_mut(scan_state.cast());
    ensure_outputs(state, scan_state);
    // The projection is compiled for the (virtual) scan tuple slot, so every row
    // — local or remote — is delivered through it.
    let scan_slot = (*scan_state).ss_ScanTupleSlot;
    loop {
        let output_index = state.next_output;
        if output_index >= state.outputs.len() {
            return pg_sys::ExecClearTuple(scan_slot);
        }
        state.next_output += 1;
        match &state.outputs[output_index] {
            CustomScanOutputRow::Local(tid) => {
                let mut item = pg_sys::ItemPointerData::default();
                pgrx::itemptr::item_pointer_set_all(
                    &mut item,
                    tid.block_number,
                    tid.offset_number,
                );
                let estate = (*scan_state).ps.state;
                if estate.is_null() {
                    pgrx::error!("EcDistannDistributedScan missing executor estate");
                }
                pg_sys::ExecClearTuple(state.local_heap_slot);
                let visible = pg_sys::table_tuple_fetch_row_version(
                    (*scan_state).ss_currentRelation,
                    &mut item,
                    (*estate).es_snapshot,
                    state.local_heap_slot,
                );
                if visible {
                    // Copy the heap row into the virtual scan slot the projection
                    // reads from.
                    return pg_sys::ExecCopySlot(scan_slot, state.local_heap_slot);
                }
                // Row no longer visible under the snapshot — skip to the next.
            }
            CustomScanOutputRow::Remote {
                payload_nulls,
                payload_values,
            } => {
                let nulls = payload_nulls.clone();
                let values = payload_values.clone();
                return store_remote_payload(state, scan_slot, &nulls, &values);
            }
        }
    }
}

/// Run the shared FR-080/FR-081 search once, then split hits into local-heap
/// fetches and owner-shipped remote payloads.
unsafe fn ensure_outputs(
    state: &mut DistannCustomScanExecState,
    scan_state: *mut pg_sys::ScanState,
) {
    if state.loaded {
        return;
    }
    state.loaded = true;

    let heap_relation = (*scan_state).ss_currentRelation;
    if heap_relation.is_null() {
        pgrx::error!("EcDistannDistributedScan missing scan relation");
    }
    let estate = (*scan_state).ps.state;
    if estate.is_null() {
        pgrx::error!("EcDistannDistributedScan missing executor estate");
    }
    let snapshot = (*estate).es_snapshot;

    // A private buffer-heap slot for local ctid fetches (once, estate-managed).
    if state.local_heap_slot.is_null() {
        state.local_heap_slot = pg_sys::ExecInitExtraTupleSlot(
            estate,
            (*heap_relation).rd_att,
            pg_sys::table_slot_callbacks(heap_relation),
        );
    }

    let index_guard = crate::storage::relation_guard::IndexRelationGuard::try_access_share(
        state.index_oid,
    )
    .unwrap_or_else(|| pgrx::error!("EcDistannDistributedScan could not open the index relation"));
    let index_relation = index_guard.as_ptr();
    let handle = ptr::NonNull::new(index_relation)
        .unwrap_or_else(|| pgrx::error!("EcDistannDistributedScan got a null index relation"));
    let source_attnum = super::routine::indexed_ecvector_attnum(index_relation)
        .unwrap_or_else(|e| pgrx::error!("EcDistannDistributedScan source column: {e}"));
    let rerank_slot =
        crate::storage::slot_guard::TupleTableSlotGuard::single_for_heap(heap_relation)
            .unwrap_or_else(|| pgrx::error!("EcDistannDistributedScan rerank slot setup failed"));

    // Evaluate the ORDER BY query for this scan (binds any correlated Param).
    let query = eval_query_vector(scan_state, state.query_expr_state);

    // Shared search core: local hits get a resolved ctid; remote hits carry
    // INVALID (we ship their row payloads below).
    //
    // Exploration bar = `ec_distann.top_k` (the D9 exit bar / ef-search knob),
    // NOT the plan LIMIT — the AM `amgettuple` path explores to
    // `options::current_top_k()` (routine.rs) and serves LIMIT rows from it. Using
    // the LIMIT as the bar made the CustomScan under-explore (ef=LIMIT vs ef=GUC),
    // trailing single-node recall at larger `ec_distann.top_k` (caught by the
    // suite recall gate). Explore to at least the GUC and at least the LIMIT, then
    // iteratively deepen on early-exit for parity, and truncate to the LIMIT.
    let mut effective = super::options::current_top_k().max(state.top_k).max(1);
    let deepen_cap = effective.saturating_mul(64).max(1024);
    let mut collection = super::routine::collect_distann_hits(
        handle,
        index_relation,
        heap_relation,
        snapshot,
        rerank_slot.as_ptr(),
        source_attnum,
        &query,
        effective,
        false,
    );
    while collection.counters.early_exit
        && collection.hits.len() < state.top_k
        && effective < deepen_cap
    {
        effective = effective.saturating_mul(2);
        collection = super::routine::collect_distann_hits(
            handle,
            index_relation,
            heap_relation,
            snapshot,
            rerank_slot.as_ptr(),
            source_attnum,
            &query,
            effective,
            false,
        );
    }
    let mut hits = collection.hits;
    hits.truncate(state.top_k);

    // Group remote hits (INVALID ctid) by owning node and fetch their row
    // payloads from the owner.
    let payloads = fetch_remote_payloads(state, index_relation, &hits);

    state.outputs = hits
        .iter()
        .filter_map(|hit| {
            if hit.heap_tid != crate::storage::page::ItemPointer::INVALID {
                return Some(CustomScanOutputRow::Local(hit.heap_tid));
            }
            match payloads.get(&hit.vec_id) {
                Some(payload) if !payload.tuple_payload_missing => Some(CustomScanOutputRow::Remote {
                    payload_nulls: payload.payload_nulls.clone(),
                    payload_values: payload.payload_values.clone(),
                }),
                // A missing payload means the owner no longer holds a live row
                // for this vec_id (deleted between search and materialize); drop
                // it rather than emit a wrong row.
                _ => None,
            }
        })
        .collect();
}

struct RemotePayload {
    tuple_payload_missing: bool,
    payload_nulls: Vec<bool>,
    payload_values: Vec<Vec<u8>>,
}

/// Group INVALID-ctid (remote-owned) hits by owning node, then issue one
/// `ec_distann_materialize_row_payloads` call per owner over the pooled
/// transport, returning a vec_id -> payload map.
unsafe fn fetch_remote_payloads(
    state: &DistannCustomScanExecState,
    index_relation: pg_sys::Relation,
    hits: &[super::scan::DistannScanHit],
) -> std::collections::HashMap<u64, RemotePayload> {
    let mut result = std::collections::HashMap::new();
    let remote_vec_ids: Vec<u64> = hits
        .iter()
        .filter(|hit| hit.heap_tid == crate::storage::page::ItemPointer::INVALID)
        .map(|hit| hit.vec_id)
        .collect();
    if remote_vec_ids.is_empty() {
        return result;
    }

    let placement = super::roster::current_placement_directory()
        .unwrap_or_else(|e| pgrx::error!("EcDistannDistributedScan roster resolution failed: {e}"));
    let node_count = placement.node_count();
    let metadata = super::ambuild::read_metadata_from_index_handle(
        ptr::NonNull::new(index_relation).expect("index relation non-null"),
    )
    .unwrap_or_else(|e| pgrx::error!("EcDistannDistributedScan metadata read failed: {e}"));
    let identity = super::roster::local_epoch_identity(&placement, &metadata);
    let fingerprint = super::epoch::compute_epoch_fingerprint(
        &identity,
        super::epoch::DISTANN_EPOCH_FINGERPRINT_V1,
    )
    .to_vec();
    let roster_spec = super::roster::current_roster_spec();
    let epoch = super::roster::current_epoch();
    let index_name = super::routine::distann_index_relname(index_relation);

    // Bucket vec_ids by owning-node index.
    let mut buckets: std::collections::BTreeMap<usize, Vec<u64>> = std::collections::BTreeMap::new();
    for &vec_id in &remote_vec_ids {
        let owner = owning_node(vec_id, node_count, placement.hash_version);
        buckets.entry(owner).or_default().push(vec_id);
    }

    let mut requests = Vec::new();
    let mut request_vec_ids = Vec::new();
    for (owner, vec_ids) in &buckets {
        let node = &placement.nodes[*owner];
        if node.is_local {
            // A local-owned hit should have carried a valid ctid; INVALID here is
            // a structural fault rather than something to ship remotely.
            pgrx::error!(
                "EcDistannDistributedScan: locally-owned hit has no heap ctid (node {})",
                node.node_id
            );
        }
        request_vec_ids.push(vec_ids.clone());
        requests.push(super::remote_transport::DistannRemoteMaterializeRequest {
            conninfo: node.conninfo.as_str(),
            roster_spec: roster_spec.as_str(),
            target_node_id: node.node_id,
            epoch,
            index_regclass: index_name.as_str(),
            epoch_fingerprint: fingerprint.as_slice(),
            vec_ids: vec_ids.as_slice(),
        });
    }

    let responses = super::remote_transport::remote_materialize_row_payloads_batch(
        &requests,
        &state.payload_columns,
        &state.payload_send_functions,
    );
    for response in responses {
        let rows = response
            .unwrap_or_else(|e| pgrx::error!("EcDistannDistributedScan remote materialize: {e}"));
        for row in rows {
            result.insert(
                row.vec_id,
                RemotePayload {
                    tuple_payload_missing: row.tuple_payload_missing,
                    payload_nulls: row.payload_nulls,
                    payload_values: row.payload_values,
                },
            );
        }
    }
    result
}

/// Reconstruct a virtual tuple in the scan slot from an owner-shipped payload:
/// each projected heap attnum's binary value is decoded via `ReceiveFunctionCall`
/// and set on the matching slot attribute; all other attributes are NULL.
unsafe fn store_remote_payload(
    state: &mut DistannCustomScanExecState,
    slot: *mut pg_sys::TupleTableSlot,
    payload_nulls: &[bool],
    payload_values: &[Vec<u8>],
) -> *mut pg_sys::TupleTableSlot {
    let mut writer = TupleSlotWriter::from_raw_slot(slot, "EcDistannDistributedScan")
        .unwrap_or_else(|e| pgrx::error!("{e}"));
    writer.clear();
    let natts = writer.natts();
    for attr_index in 0..natts {
        let Some(attr) = writer
            .attribute(attr_index)
            .unwrap_or_else(|e| pgrx::error!("{e}"))
        else {
            writer.set_null(attr_index);
            continue;
        };
        let Some(pos) = state.payload_attnums.iter().position(|a| *a == attr.attnum) else {
            writer.set_null(attr_index);
            continue;
        };
        if payload_nulls.get(pos).copied().unwrap_or(true) {
            writer.set_null(attr_index);
            continue;
        }
        let value = payload_values
            .get(pos)
            .unwrap_or_else(|| pgrx::error!("EcDistannDistributedScan payload value missing"));
        let datum = binary_value_to_datum(value, &mut state.payload_inputs[pos], &attr.name);
        writer.set_datum(attr_index, datum);
    }
    writer
        .store_virtual_tuple()
        .unwrap_or_else(|e| pgrx::error!("{e}"))
}

unsafe fn binary_value_to_datum(
    value: &[u8],
    attr_input: &mut PayloadAttrIo,
    attr_name: &str,
) -> pg_sys::Datum {
    let len = core::ffi::c_int::try_from(value.len()).unwrap_or_else(|_| {
        pgrx::error!("EcDistannDistributedScan payload column \"{attr_name}\" is too large")
    });
    let mut bytes = value.to_vec();
    let mut input = pg_sys::StringInfoData {
        data: bytes.as_mut_ptr().cast(),
        len,
        maxlen: len,
        cursor: 0,
    };
    let datum = pg_sys::ReceiveFunctionCall(
        &mut attr_input.receive_flinfo,
        &mut input,
        attr_input.receive_typioparam,
        attr_input.typmod,
    );
    if input.cursor != input.len {
        pgrx::error!(
            "EcDistannDistributedScan payload column \"{attr_name}\" binary receive left unread bytes"
        );
    }
    datum
}

/// # Safety
/// Called by the executor for EvalPlanQual rechecks.
#[pg_guard]
unsafe extern "C-unwind" fn custom_scan_recheck(
    _scan_state: *mut pg_sys::ScanState,
    _slot: *mut pg_sys::TupleTableSlot,
) -> bool {
    // Remote virtual tuples have no coordinator heap identity to re-fetch.
    true
}

/// # Safety
/// Called by the executor to restart the scan.
#[pg_guard]
unsafe extern "C-unwind" fn rescan_custom_scan(node: *mut pg_sys::CustomScanState) {
    let state = exec_state_mut(node);
    state.outputs.clear();
    state.next_output = 0;
    state.loaded = false;
}

/// # Safety
/// Called at most once for the state allocated by create_custom_scan_state.
#[pg_guard]
unsafe extern "C-unwind" fn end_custom_scan(node: *mut pg_sys::CustomScanState) {
    if node.is_null() {
        return;
    }
    let state = node.cast::<DistannCustomScanExecState>();
    ptr::drop_in_place(state);
    pg_sys::pfree(state.cast());
}
