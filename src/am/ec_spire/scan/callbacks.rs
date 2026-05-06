pub(super) unsafe extern "C-unwind" fn ec_spire_ambeginscan(
    index_relation: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let scan = pg_sys::RelationGetIndexScan(index_relation, nkeys, norderbys);
            if scan.is_null() {
                pgrx::error!("ec_spire failed to allocate scan descriptor");
            }

            let opaque = PgBox::<SpireScanOpaque>::alloc_in_context(PgMemoryContexts::For(
                pg_sys::CurrentMemoryContext,
            ));
            ptr::write(opaque.as_ptr(), SpireScanOpaque::default());
            (*scan).parallel_scan = ptr::null_mut();
            (*scan).opaque = opaque.into_pg().cast();
            scan
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amrescan(
    scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    orderbys: pg_sys::ScanKey,
    norderbys: std::ffi::c_int,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_spire amrescan received a null scan descriptor");
            }
            if nkeys != 0 {
                pgrx::error!("ec_spire scan does not support index quals yet");
            }
            if norderbys != 1 {
                pgrx::error!("ec_spire scan currently requires exactly one ORDER BY query");
            }

            let opaque_ptr = (*scan).opaque.cast::<SpireScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_spire amrescan missing scan opaque state");
            }
            let opaque = &mut *opaque_ptr;
            opaque.clear_scan_work();
            let query = decode_scan_orderby_query(orderbys).unwrap_or_else(|e| pgrx::error!("{e}"));
            (*scan).xs_recheck = false;
            (*scan).xs_recheckorderby = false;
            (*scan).xs_orderbyvals = ptr::null_mut();
            (*scan).xs_orderbynulls = ptr::null_mut();

            let root_control = opaque.root_control_for_rescan((*scan).indexRelation);
            if root_control.active_epoch == 0 {
                let scan_plan =
                    resolve_single_level_scan_plan(0, relation_options((*scan).indexRelation))
                        .unwrap_or_else(|e| pgrx::error!("{e}"));
                opaque.reset_for_candidates(query, scan_plan, Vec::new());
                return;
            }

            let (epoch_manifest, object_manifest, placement_directory) =
                load_relation_epoch_manifests((*scan).indexRelation, root_control)
                    .unwrap_or_else(|e| pgrx::error!("{e}"));
            let snapshot = SpirePublishedEpochSnapshot::new(
                &epoch_manifest,
                &object_manifest,
                &placement_directory,
            )
            .unwrap_or_else(|e| pgrx::error!("{e}"));
            let object_store = SpireRelationObjectStore::for_index_relation((*scan).indexRelation)
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            let prepared = prepare_single_level_relation_snapshot_scan_candidates(
                scan,
                &snapshot,
                &object_store,
                &query,
                relation_options((*scan).indexRelation),
            )
            .unwrap_or_else(|e| pgrx::error!("{e}"));
            opaque.reset_for_candidates(query, prepared.scan_plan, prepared.candidates);
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amgettuple(
    scan: pg_sys::IndexScanDesc,
    direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_spire amgettuple received a null scan descriptor");
            }
            if direction != pg_sys::ScanDirection::ForwardScanDirection {
                pgrx::error!("ec_spire amgettuple only supports forward scan direction");
            }
            let opaque_ptr = (*scan).opaque.cast::<SpireScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_spire amgettuple missing scan opaque state");
            }
            let opaque = &mut *opaque_ptr;
            if !opaque.rescan_called {
                pgrx::error!("ec_spire amgettuple requires amrescan before scan execution");
            }

            match opaque.next_output() {
                Some(output) => {
                    set_scan_heap_tid(scan, output.heap_tid);
                    set_scan_orderby_score(scan, output.orderby_score);
                    (*scan).xs_recheck = false;
                    (*scan).xs_recheckorderby = false;
                    true
                }
                None => {
                    clear_scan_orderby_output(scan);
                    false
                }
            }
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amendscan(scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                return;
            }

            let opaque_ptr = (*scan).opaque.cast::<SpireScanOpaque>();
            if !opaque_ptr.is_null() {
                ptr::drop_in_place(opaque_ptr);
                pg_sys::pfree(opaque_ptr.cast());
                (*scan).opaque = ptr::null_mut();
            }
        })
    }
}

