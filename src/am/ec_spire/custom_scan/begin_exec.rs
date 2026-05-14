#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_create_custom_scan_state(
    _cscan: *mut pg_sys::CustomScan,
) -> *mut pg_sys::Node {
    unsafe {
        let state = pg_sys::palloc0(std::mem::size_of::<SpireCustomScanExecState>())
            .cast::<SpireCustomScanExecState>();
        ptr::write(state, custom_scan_default_exec_state());
        (*state).custom_scan_state.ss.ps.type_ = pg_sys::NodeTag::T_CustomScanState;
        (*state).custom_scan_state.methods = &raw const CUSTOM_EXEC_METHODS;
        state.cast::<pg_sys::Node>()
    }
}

fn custom_scan_default_exec_state() -> SpireCustomScanExecState {
    SpireCustomScanExecState {
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
            custom_scan_payload_attr_io((*(*node).ss.ss_currentRelation).rd_att);
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
        custom_scan_release_exec_state_for_end(&mut *state);
        ptr::drop_in_place(state);
        pg_sys::pfree(state.cast());
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_spire_rescan_custom_scan(node: *mut pg_sys::CustomScanState) {
    unsafe {
        let state = node.cast::<SpireCustomScanExecState>();
        custom_scan_reset_exec_state_for_rescan(&mut *state);
    }
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
            let Some(output_index) = custom_scan_next_output_index(&mut *state) else {
                return pg_sys::ExecClearTuple((*scan_state).ss_ScanTupleSlot);
            };
            let output = &(&(*state).outputs)[output_index];
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
