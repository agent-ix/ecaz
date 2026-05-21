pub(super) unsafe extern "C-unwind" fn ec_spire_ambeginscan(
    index_relation: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    pg_am_callback!({
        let scan = pg_sys::RelationGetIndexScan(index_relation, nkeys, norderbys);
        if scan.is_null() {
            pgrx::error!("ec_spire failed to allocate scan descriptor");
        }

        crate::fault::maybe_fail_palloc("ec_spire ambeginscan opaque");
        let opaque = PgBox::<SpireScanOpaque>::alloc_in_context(PgMemoryContexts::For(
            pg_sys::CurrentMemoryContext,
        ));
        ptr::write(opaque.as_ptr(), SpireScanOpaque::default());
        (*scan).parallel_scan = ptr::null_mut();
        (*scan).opaque = opaque.into_pg().cast();
        scan
    })
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amrescan(
    scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    orderbys: pg_sys::ScanKey,
    norderbys: std::ffi::c_int,
) {
    pg_am_callback!({
        let mut scan_view = SpireIndexScanView::from_raw(scan, "amrescan");
        if nkeys != 0 {
            pgrx::error!("ec_spire scan does not support index quals yet");
        }
        if norderbys != 1 {
            pgrx::error!("ec_spire scan currently requires exactly one ORDER BY query");
        }

        scan_view.opaque_mut("amrescan").clear_scan_work();
        let query = decode_scan_orderby_query(orderbys).unwrap_or_else(|e| pgrx::error!("{e}"));
        scan_view.clear_recheck_flags_and_orderby_outputs();

        let index_relation = scan_view.index_relation();
        let root_control = scan_view
            .opaque_mut("amrescan")
            .root_control_for_rescan(index_relation);
        if root_control.active_epoch == 0 {
            let scan_plan = resolve_single_level_scan_plan(0, relation_options(index_relation))
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            scan_view
                .opaque_mut("amrescan")
                .reset_for_candidates(query, scan_plan, Vec::new());
            return;
        }

        let heap_relation = scan_view.heap_relation();
        let snapshot = scan_view.snapshot();
        let stream = super::remote_search_production_scan_heap_resolution_am_result_stream(
            index_relation,
            heap_relation.as_ptr(),
            snapshot,
            query.values().to_vec(),
        );
        let outputs = production_scan_result_stream_am_outputs(&stream)
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        scan_view
            .opaque_mut("amrescan")
            .reset_for_outputs(query, None, outputs);
    })
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amgettuple(
    scan: pg_sys::IndexScanDesc,
    direction: pg_sys::ScanDirection::Type,
) -> bool {
    pg_am_callback!({
        let mut scan_view = SpireIndexScanView::from_raw(scan, "amgettuple");
        if direction != pg_sys::ScanDirection::ForwardScanDirection {
            pgrx::error!("ec_spire amgettuple only supports forward scan direction");
        }
        if !scan_view.opaque_mut("amgettuple").rescan_called {
            pgrx::error!("ec_spire amgettuple requires amrescan before scan execution");
        }

        let mut scan_output = crate::am::common::scan_output::IndexScanOutput::from_raw(
            scan,
            "ec_spire amgettuple scan output",
        );
        match scan_view.opaque_mut("amgettuple").next_output() {
            Some(output) => {
                set_scan_heap_tid(&mut scan_output, output.heap_tid);
                set_scan_orderby_score(&mut scan_output, output.orderby_score);
                scan_view.mark_tuple_output_current();
                true
            }
            None => {
                clear_scan_orderby_output(&mut scan_output);
                false
            }
        }
    })
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amendscan(scan: pg_sys::IndexScanDesc) {
    pg_am_callback!({
        if scan.is_null() {
            return;
        }

        let mut scan_view = SpireIndexScanView::from_raw(scan, "amendscan");
        if let Some(opaque_ptr) = scan_view.take_opaque_for_end_scan() {
            ptr::drop_in_place(opaque_ptr);
            pg_sys::pfree(opaque_ptr.cast());
        }
    })
}
