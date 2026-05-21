#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_create_custom_scan_state(
    _cscan: *mut pg_sys::CustomScan,
) -> *mut pg_sys::Node {
    // SAFETY: PostgreSQL calls this provider callback during custom scan state
    // creation; palloc0 allocates executor-lifetime memory for the state.
    unsafe {
        let allocation_context = pg_sys::CurrentMemoryContext;
        custom_scan_note_memory_baseline_for_test(allocation_context);
        let state = pg_sys::palloc0(std::mem::size_of::<SpireCustomScanExecState>())
            .cast::<SpireCustomScanExecState>();
        ptr::write(state, custom_scan_default_exec_state());
        #[cfg(any(test, feature = "pg_test"))]
        {
            (*state).allocation_context = allocation_context;
        }
        (*state).custom_scan_state.ss.ps.type_ = pg_sys::NodeTag::T_CustomScanState;
        (*state).custom_scan_state.methods = &raw const CUSTOM_EXEC_METHODS;
        state.cast::<pg_sys::Node>()
    }
}

fn custom_scan_default_exec_state() -> SpireCustomScanExecState {
    SpireCustomScanExecState {
        // SAFETY: CustomScanState is a C struct initialized field-by-field by
        // PostgreSQL/executor setup after this Rust wrapper is allocated.
        custom_scan_state: unsafe { std::mem::zeroed() },
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
        #[cfg(any(test, feature = "pg_test"))]
        allocation_context: std::ptr::null_mut(),
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_begin_custom_scan(
    node: *mut pg_sys::CustomScanState,
    _estate: *mut pg_sys::EState,
    _eflags: core::ffi::c_int,
) {
    let state = custom_scan_exec_state_mut(node.cast(), "BeginCustomScan");
    // SAFETY: PostgreSQL invokes BeginCustomScan with a live CustomScanState
    // whose plan pointer is the provider-owned CustomScan built by the planner.
    let custom_scan = unsafe { custom_scan_plan(node) };
    let mode = unsafe { custom_scan_mode_from_plan(custom_scan) };
    let index_oid = unsafe { custom_scan_index_oid_from_plan(custom_scan) };
    match mode {
        SpireCustomScanPlanMode::VectorOrderLimit => {
            let (tuple_payload_columns, tuple_payload_inputs) =
                unsafe { custom_scan_tuple_payload_state_from_plan(node, custom_scan) };
            custom_scan_init_vector_order_limit_exec_state(
                state,
                index_oid,
                unsafe { custom_scan_top_k_from_plan(custom_scan) },
                unsafe { custom_scan_query_from_plan(node, custom_scan) },
                tuple_payload_columns,
                tuple_payload_inputs,
            );
        }
        SpireCustomScanPlanMode::DmlPkSelectTuplePayload => {
            // SAFETY: PostgreSQL invokes BeginCustomScan with the live
            // provider-owned CustomScanState and matching CustomScan plan.
            unsafe { custom_scan_init_dml_exec_state(state, node, custom_scan, mode, index_oid) };
        }
        SpireCustomScanPlanMode::DmlUpdateTuplePayload
        | SpireCustomScanPlanMode::DmlDeleteTuplePayload => {
            // SAFETY: PostgreSQL invokes BeginCustomScan with the live
            // provider-owned CustomScanState and matching CustomScan plan.
            unsafe { custom_scan_init_dml_exec_state(state, node, custom_scan, mode, index_oid) };
        }
    }
}

fn custom_scan_init_vector_order_limit_exec_state(
    state: &mut SpireCustomScanExecState,
    index_oid: pg_sys::Oid,
    top_k: usize,
    query: Vec<f32>,
    tuple_payload_columns: Vec<String>,
    tuple_payload_inputs: Vec<Option<SpireCustomScanPayloadAttrIo>>,
) {
    state.mode = SpireCustomScanPlanMode::VectorOrderLimit;
    state.index_oid = index_oid;
    state.top_k = top_k;
    state.query = query;
    state.tuple_payload_columns = tuple_payload_columns;
    state.tuple_payload_inputs = tuple_payload_inputs;
    state.outputs.clear();
    state.next_output = 0;
    state.loaded_outputs = false;
    state.dml_payload_loaded = false;
    state.dml_payload_emitted = false;
    state.dml_tuple_payload_json = None;
}

unsafe fn custom_scan_tuple_payload_state_from_plan(
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) -> (Vec<String>, Vec<Option<SpireCustomScanPayloadAttrIo>>) {
    // SAFETY: node/custom_scan are the live executor state and matching plan;
    // the current relation tuple descriptor remains valid during BeginCustomScan.
    unsafe {
        let tuple_desc = crate::storage::relation::relation_tuple_desc_copy(
            custom_scan_current_relation(node, "tuple payload input descriptor"),
        );
        (
            custom_scan_tuple_payload_columns(node, custom_scan),
            custom_scan_payload_attr_io(tuple_desc.as_ptr()),
        )
    }
}

unsafe fn custom_scan_init_tuple_payload_state(
    state: &mut SpireCustomScanExecState,
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
) {
    let (tuple_payload_columns, tuple_payload_inputs) =
        unsafe { custom_scan_tuple_payload_state_from_plan(node, custom_scan) };
    state.tuple_payload_columns = tuple_payload_columns;
    state.tuple_payload_inputs = tuple_payload_inputs;
}

unsafe fn custom_scan_init_dml_exec_state(
    state: &mut SpireCustomScanExecState,
    node: *mut pg_sys::CustomScanState,
    custom_scan: *mut pg_sys::CustomScan,
    mode: SpireCustomScanPlanMode,
    index_oid: pg_sys::Oid,
) {
    state.mode = mode;
    state.index_oid = index_oid;
    // SAFETY: caller passes the live BeginCustomScan state and matching
    // provider-owned CustomScan plan. All plan-private metadata reads are
    // copied into Rust-owned state before the callback returns.
    unsafe {
        if mode == SpireCustomScanPlanMode::DmlPkSelectTuplePayload {
            custom_scan_init_tuple_payload_state(state, node, custom_scan);
            state.dml_pk_column = custom_scan_dml_pk_column(node);
        } else {
            state.dml_pk_column = custom_scan_dml_pk_column_from_plan(custom_scan);
        }
        state.dml_pk_value = custom_scan_dml_pk_value_from_plan(node, custom_scan);
        state.dml_updated_columns =
            custom_scan_dml_column_list_from_plan(custom_scan, 2, "updated columns");
        state.dml_projected_columns =
            custom_scan_dml_column_list_from_plan(custom_scan, 3, "projected columns");
        if mode == SpireCustomScanPlanMode::DmlUpdateTuplePayload {
            state.dml_update_value_exprs = custom_scan_dml_update_value_exprs_from_plan(
                custom_scan,
                state.dml_updated_columns.len(),
            );
        }
    }
    custom_scan_validate_dml_column_metadata(
        state.mode,
        &state.dml_updated_columns,
        &state.dml_projected_columns,
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_exec_custom_scan(
    node: *mut pg_sys::CustomScanState,
) -> *mut pg_sys::TupleTableSlot {
    // SAFETY: PostgreSQL invokes ExecCustomScan with the live CustomScanState;
    // ExecScan owns callback invocation and scan tuple slot handling.
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
    // SAFETY: PostgreSQL invokes EndCustomScan at most once for the live state
    // allocated by create_custom_scan_state; this path drops Rust fields then pfree's it.
    unsafe {
        if node.is_null() {
            return;
        }
        custom_scan_note_end_custom_scan_for_test();
        let state = node.cast::<SpireCustomScanExecState>();
        #[cfg(any(test, feature = "pg_test"))]
        let allocation_context = (*state).allocation_context;
        custom_scan_release_exec_state_for_end(&mut *state);
        ptr::drop_in_place(state);
        custom_scan_note_pfree_for_test();
        pg_sys::pfree(state.cast());
        #[cfg(any(test, feature = "pg_test"))]
        custom_scan_note_memory_after_end_for_test(allocation_context);
    }
}

#[cfg(any(test, feature = "pg_test"))]
static CUSTOM_SCAN_END_CUSTOM_SCAN_COUNT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
#[cfg(any(test, feature = "pg_test"))]
static CUSTOM_SCAN_PFREE_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
#[cfg(any(test, feature = "pg_test"))]
static CUSTOM_SCAN_MEMORY_BASELINE_USED_BYTES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(u64::MAX);
#[cfg(any(test, feature = "pg_test"))]
static CUSTOM_SCAN_MEMORY_AFTER_END_USED_BYTES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(u64::MAX);
#[cfg(any(test, feature = "pg_test"))]
static CUSTOM_SCAN_RESCAN_COUNT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
#[cfg(any(test, feature = "pg_test"))]
static CUSTOM_SCAN_RESCAN_OUTPUTS_LEN_AFTER_RESET: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(any(test, feature = "pg_test"))]
static CUSTOM_SCAN_RESCAN_NEXT_OUTPUT_AFTER_RESET: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(any(test, feature = "pg_test"))]
static CUSTOM_SCAN_RESCAN_LOADED_OUTPUTS_AFTER_RESET: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(any(test, feature = "pg_test"))]
fn custom_scan_note_end_custom_scan_for_test() {
    CUSTOM_SCAN_END_CUSTOM_SCAN_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(not(any(test, feature = "pg_test")))]
fn custom_scan_note_end_custom_scan_for_test() {}

#[cfg(any(test, feature = "pg_test"))]
fn custom_scan_note_pfree_for_test() {
    CUSTOM_SCAN_PFREE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(not(any(test, feature = "pg_test")))]
fn custom_scan_note_pfree_for_test() {}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn custom_scan_memory_context_used_bytes_for_test(context: pg_sys::MemoryContext) -> u64 {
    if context.is_null() {
        return u64::MAX;
    }
    let mut counters = pg_sys::MemoryContextCounters::default();
    // SAFETY: context is a live PostgreSQL MemoryContext captured from the
    // executor allocation path for test-only accounting.
    unsafe {
        pg_sys::MemoryContextMemConsumed(context, &mut counters);
    }
    u64::try_from(counters.totalspace.saturating_sub(counters.freespace)).unwrap_or(u64::MAX)
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn custom_scan_note_memory_baseline_for_test(context: pg_sys::MemoryContext) {
    CUSTOM_SCAN_MEMORY_BASELINE_USED_BYTES.store(
        unsafe { custom_scan_memory_context_used_bytes_for_test(context) },
        std::sync::atomic::Ordering::Relaxed,
    );
}

#[cfg(not(any(test, feature = "pg_test")))]
unsafe fn custom_scan_note_memory_baseline_for_test(_context: pg_sys::MemoryContext) {}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn custom_scan_note_memory_after_end_for_test(context: pg_sys::MemoryContext) {
    CUSTOM_SCAN_MEMORY_AFTER_END_USED_BYTES.store(
        unsafe { custom_scan_memory_context_used_bytes_for_test(context) },
        std::sync::atomic::Ordering::Relaxed,
    );
}

#[cfg(not(any(test, feature = "pg_test")))]
unsafe fn custom_scan_note_memory_after_end_for_test(_context: pg_sys::MemoryContext) {}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn custom_scan_reset_cleanup_counters_for_test() {
    CUSTOM_SCAN_END_CUSTOM_SCAN_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
    CUSTOM_SCAN_PFREE_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
    CUSTOM_SCAN_MEMORY_BASELINE_USED_BYTES.store(u64::MAX, std::sync::atomic::Ordering::Relaxed);
    CUSTOM_SCAN_MEMORY_AFTER_END_USED_BYTES.store(u64::MAX, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn custom_scan_cleanup_counters_for_test() -> SpireCustomScanCleanupCounters {
    SpireCustomScanCleanupCounters {
        end_custom_scan_count: CUSTOM_SCAN_END_CUSTOM_SCAN_COUNT
            .load(std::sync::atomic::Ordering::Relaxed),
        pfree_count: CUSTOM_SCAN_PFREE_COUNT.load(std::sync::atomic::Ordering::Relaxed),
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn custom_scan_memory_context_snapshot_for_test() -> SpireCustomScanMemoryContextSnapshot
{
    let baseline_used_bytes =
        CUSTOM_SCAN_MEMORY_BASELINE_USED_BYTES.load(std::sync::atomic::Ordering::Relaxed);
    let after_end_used_bytes =
        CUSTOM_SCAN_MEMORY_AFTER_END_USED_BYTES.load(std::sync::atomic::Ordering::Relaxed);
    SpireCustomScanMemoryContextSnapshot {
        baseline_captured: baseline_used_bytes != u64::MAX,
        after_end_captured: after_end_used_bytes != u64::MAX,
        baseline_used_bytes,
        after_end_used_bytes,
    }
}

#[cfg(any(test, feature = "pg_test"))]
fn custom_scan_note_rescan_for_test(state: &SpireCustomScanExecState) {
    CUSTOM_SCAN_RESCAN_OUTPUTS_LEN_AFTER_RESET
        .store(state.outputs.len(), std::sync::atomic::Ordering::Relaxed);
    CUSTOM_SCAN_RESCAN_NEXT_OUTPUT_AFTER_RESET
        .store(state.next_output, std::sync::atomic::Ordering::Relaxed);
    CUSTOM_SCAN_RESCAN_LOADED_OUTPUTS_AFTER_RESET
        .store(state.loaded_outputs, std::sync::atomic::Ordering::Relaxed);
    CUSTOM_SCAN_RESCAN_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(not(any(test, feature = "pg_test")))]
fn custom_scan_note_rescan_for_test(_state: &SpireCustomScanExecState) {}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn custom_scan_reset_rescan_snapshot_for_test() {
    CUSTOM_SCAN_RESCAN_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
    CUSTOM_SCAN_RESCAN_OUTPUTS_LEN_AFTER_RESET
        .store(usize::MAX, std::sync::atomic::Ordering::Relaxed);
    CUSTOM_SCAN_RESCAN_NEXT_OUTPUT_AFTER_RESET
        .store(usize::MAX, std::sync::atomic::Ordering::Relaxed);
    CUSTOM_SCAN_RESCAN_LOADED_OUTPUTS_AFTER_RESET.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn custom_scan_rescan_snapshot_for_test() -> SpireCustomScanRescanSnapshot {
    SpireCustomScanRescanSnapshot {
        rescan_count: CUSTOM_SCAN_RESCAN_COUNT.load(std::sync::atomic::Ordering::Relaxed),
        outputs_len_after_reset: CUSTOM_SCAN_RESCAN_OUTPUTS_LEN_AFTER_RESET
            .load(std::sync::atomic::Ordering::Relaxed),
        next_output_after_reset: CUSTOM_SCAN_RESCAN_NEXT_OUTPUT_AFTER_RESET
            .load(std::sync::atomic::Ordering::Relaxed),
        loaded_outputs_after_reset: CUSTOM_SCAN_RESCAN_LOADED_OUTPUTS_AFTER_RESET
            .load(std::sync::atomic::Ordering::Relaxed),
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_rescan_custom_scan(node: *mut pg_sys::CustomScanState) {
    let state = custom_scan_exec_state_mut(node.cast(), "ReScanCustomScan");
    custom_scan_reset_exec_state_for_rescan(state);
    custom_scan_note_rescan_for_test(state);
}

fn custom_scan_release_exec_state_for_end(state: &mut SpireCustomScanExecState) {
    state.index_oid = pg_sys::InvalidOid;
    state.top_k = 0;
    state.query = Vec::new();
    state.dml_pk_column = String::new();
    state.dml_pk_value = [0; 8];
    state.dml_updated_columns = Vec::new();
    state.dml_projected_columns = Vec::new();
    state.dml_update_value_exprs = Vec::new();
    state.tuple_payload_columns = Vec::new();
    state.tuple_payload_inputs = Vec::new();
    state.outputs = Vec::new();
    state.next_output = 0;
    state.loaded_outputs = false;
    state.dml_payload_loaded = false;
    state.dml_payload_emitted = false;
    state.dml_tuple_payload_json = None;
}

fn custom_scan_reset_exec_state_for_rescan(state: &mut SpireCustomScanExecState) {
    state.outputs.clear();
    state.next_output = 0;
    state.loaded_outputs = false;
    state.dml_payload_loaded = false;
    state.dml_payload_emitted = false;
    state.dml_tuple_payload_json = None;
}

fn custom_scan_next_output_index(state: &mut SpireCustomScanExecState) -> Option<usize> {
    let output_index = state.next_output;
    state.outputs.get(output_index)?;
    state.next_output = state.next_output.saturating_add(1);
    Some(output_index)
}

#[derive(Clone, Copy)]
struct CustomScanAccessState<'a> {
    scan_state: *mut pg_sys::ScanState,
    _scan_state: std::marker::PhantomData<&'a pg_sys::ScanState>,
}

impl<'a> CustomScanAccessState<'a> {
    unsafe fn new(scan_state: *mut pg_sys::ScanState) -> Self {
        if scan_state.is_null() {
            pgrx::error!("EcSpireDistributedScan access method received null scan state");
        }
        Self {
            scan_state,
            _scan_state: std::marker::PhantomData,
        }
    }

    fn as_ptr(self) -> *mut pg_sys::ScanState {
        self.scan_state
    }

    fn tuple_slot(self) -> *mut pg_sys::TupleTableSlot {
        // SAFETY: ExecScan invokes provider access callbacks with a live
        // ScanState whose scan tuple slot is owned by the active executor node.
        unsafe { (*self.scan_state).ss_ScanTupleSlot }
    }

    fn clear_tuple_slot(self) -> *mut pg_sys::TupleTableSlot {
        // SAFETY: the slot belongs to the live scan state for this callback.
        unsafe { pg_sys::ExecClearTuple(self.tuple_slot()) }
    }

    fn add_processed_count(self, processed: u64) {
        // SAFETY: the executor estate belongs to the live scan callback;
        // saturating arithmetic preserves PostgreSQL's processed-row counter.
        unsafe {
            let estate = (*self.scan_state).ps.state;
            if !estate.is_null() {
                (*estate).es_processed = (*estate).es_processed.saturating_add(processed);
            }
        }
    }

    fn fetch_row_version_into_scan_slot(self, tid: &mut pg_sys::ItemPointerData) -> bool {
        // SAFETY: relation and slot are borrowed from the live scan state for
        // the current callback; PostgreSQL writes the visible heap tuple into
        // the scan tuple slot using the active estate snapshot.
        unsafe {
            let estate = (*self.scan_state).ps.state;
            if estate.is_null() {
                pgrx::error!("EcSpireDistributedScan missing executor estate");
            }
            pg_sys::table_tuple_fetch_row_version(
                (*self.scan_state).ss_currentRelation,
                tid,
                (*estate).es_snapshot,
                (*self.scan_state).ss_ScanTupleSlot,
            )
        }
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_custom_scan_access(
    scan_state: *mut pg_sys::ScanState,
) -> *mut pg_sys::TupleTableSlot {
    let state = custom_scan_exec_state_mut(scan_state.cast(), "access method");
    // SAFETY: ExecScan invokes this provider callback with its live ScanState.
    let access_state = unsafe { CustomScanAccessState::new(scan_state) };
    if state.mode == SpireCustomScanPlanMode::DmlPkSelectTuplePayload {
        return custom_scan_dml_pk_select_access(state, access_state);
    }
    if state.mode == SpireCustomScanPlanMode::DmlUpdateTuplePayload {
        return custom_scan_dml_update_access(state, access_state);
    }
    if state.mode == SpireCustomScanPlanMode::DmlDeleteTuplePayload {
        return custom_scan_dml_delete_access(state, access_state);
    }
    custom_scan_ensure_outputs(state);
    loop {
        let Some(output_index) = custom_scan_next_output_index(state) else {
            return access_state.clear_tuple_slot();
        };
        let output = state.outputs[output_index].clone();
        if !matches!(
            output.heap_lookup_owner,
            super::SPIRE_REMOTE_LOCAL_HEAP_RESOLUTION
        ) {
            return custom_scan_store_remote_tuple_payload(state, access_state, &output);
        }

        let mut tid = pg_sys::ItemPointerData::default();
        pgrx::itemptr::item_pointer_set_all(&mut tid, output.heap_block, output.heap_offset);
        access_state.clear_tuple_slot();
        let visible = access_state.fetch_row_version_into_scan_slot(&mut tid);
        if visible {
            return access_state.tuple_slot();
        }
    }
}

fn custom_scan_dml_pk_select_access(
    state: &mut SpireCustomScanExecState,
    access_state: CustomScanAccessState<'_>,
) -> *mut pg_sys::TupleTableSlot {
    custom_scan_ensure_dml_pk_select_payload(state);
    if state.dml_payload_emitted {
        return access_state.clear_tuple_slot();
    }
    state.dml_payload_emitted = true;
    let Some(payload_json) = state.dml_tuple_payload_json.as_deref() else {
        return access_state.clear_tuple_slot();
    };
    // SAFETY: access_state holds the live scan tuple slot for this callback and
    // state owns the matching tuple payload input cache.
    unsafe {
        custom_scan_store_tuple_payload_json(
            access_state.tuple_slot(),
            payload_json,
            &mut state.tuple_payload_inputs,
        )
    }
}

fn custom_scan_dml_update_access(
    state: &mut SpireCustomScanExecState,
    access_state: CustomScanAccessState<'_>,
) -> *mut pg_sys::TupleTableSlot {
    if !state.dml_payload_emitted {
        // SAFETY: access_state was built from the live scan callback state.
        let updated_count = unsafe { custom_scan_execute_dml_update(state, access_state.as_ptr()) };
        access_state.add_processed_count(updated_count);
        state.dml_payload_emitted = true;
    }
    access_state.clear_tuple_slot()
}

fn custom_scan_dml_delete_access(
    state: &mut SpireCustomScanExecState,
    access_state: CustomScanAccessState<'_>,
) -> *mut pg_sys::TupleTableSlot {
    if !state.dml_payload_emitted {
        let deleted_count = custom_scan_execute_dml_delete(state);
        access_state.add_processed_count(deleted_count);
        state.dml_payload_emitted = true;
    }
    access_state.clear_tuple_slot()
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_custom_scan_recheck(
    _scan_state: *mut pg_sys::ScanState,
    _slot: *mut pg_sys::TupleTableSlot,
) -> bool {
    custom_scan_recheck_allows_epq_stale_row()
}

fn custom_scan_recheck_allows_epq_stale_row() -> bool {
    // V1 remote tuples are virtual payloads without coordinator heap row
    // identity, so EvalPlanQual cannot re-fetch the origin row here.
    true
}
