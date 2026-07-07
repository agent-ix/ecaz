use std::{ffi::c_void, ptr};

use pgrx::{pg_guard, pg_sys, FromDatum, PgBox, PgMemoryContexts};

use crate::am::common::{
    callback::pg_am_callback,
    routine::{alloc_index_am_routine, IndexAmRoutineBox},
    vacuum::{alloc_index_bulk_delete_result, set_index_bulk_delete_summary},
};
use crate::am::ec_diskann::DiskannScanDescView;

use super::{
    ambuild, cost, expand::LocalNodeExpander, head_cache, options,
    quantizer::{self, DistannPreparedQuery},
    scan::{
        distann_orchestrated_search, DistannOrchestrationParams, DistannScanHit,
        DistannSeedCandidate,
    },
    tuple::DistannNodeTuple,
};

fn build_ec_distann_routine() -> IndexAmRoutineBox {
    let mut amroutine = alloc_index_am_routine();

    amroutine.amstrategies = 1;
    amroutine.amsupport = 1;
    amroutine.amoptsprocnum = 0;

    amroutine.amcanorder = false;
    amroutine.amcanorderbyop = true;
    amroutine.amcanbackward = false;
    amroutine.amcanunique = false;
    amroutine.amcanmulticol = false;
    amroutine.amoptionalkey = true;
    amroutine.amsearcharray = false;
    amroutine.amsearchnulls = false;
    amroutine.amstorage = false;
    amroutine.amclusterable = false;
    amroutine.ampredlocks = false;
    amroutine.amcanparallel = false;
    // The FR-077 sharded build (M1) owns build parallelism; the monolithic
    // M0 build is single-process.
    amroutine.amcanbuildparallel = false;
    // ADR-063 include provider: `source_identity = 'include'` carries the
    // 16-byte identity through one INCLUDE column (uuid or bytea16).
    amroutine.amcaninclude = true;
    amroutine.amusemaintenanceworkmem = true;
    amroutine.amsummarizing = false;
    amroutine.amparallelvacuumoptions = 0;
    amroutine.amkeytype = pg_sys::InvalidOid;

    amroutine.ambuild = Some(ambuild::ec_distann_ambuild);
    amroutine.ambuildempty = Some(ambuild::ec_distann_ambuildempty);
    amroutine.aminsert = Some(ec_distann_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(ec_distann_ambulkdelete);
    amroutine.amvacuumcleanup = Some(ec_distann_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(cost::ec_distann_amcostestimate);
    amroutine.amoptions = Some(options::ec_distann_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(ec_distann_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(ec_distann_ambeginscan);
    amroutine.amrescan = Some(ec_distann_amrescan);
    amroutine.amgettuple = Some(ec_distann_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(ec_distann_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = None;
    amroutine.aminitparallelscan = None;
    amroutine.amparallelrescan = None;

    amroutine
}

unsafe extern "C-unwind" fn ec_distann_aminsert(
    _index_relation: pg_sys::Relation,
    _values: *mut pg_sys::Datum,
    _isnull: *mut bool,
    _heap_tid: pg_sys::ItemPointer,
    _heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    pg_am_callback!({
        pgrx::error!(
            "ec_distann aminsert is not implemented yet: the FR-083 delta-buffer DML slice lands later in Task 162"
        );
    })
}

unsafe extern "C-unwind" fn ec_distann_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    pg_am_callback!({
        // The D10 tombstone path arrives with the FR-083 DML slice; within
        // a published epoch nothing is physically reclaimed regardless.
        ec_distann_noop_vacuum_stats((*info).index, stats)
            .unwrap_or_else(|e| pgrx::error!("ec_distann ambulkdelete failed: {e}"))
    })
}

unsafe extern "C-unwind" fn ec_distann_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    pg_am_callback!({
        ec_distann_noop_vacuum_stats((*info).index, stats)
            .unwrap_or_else(|e| pgrx::error!("ec_distann amvacuumcleanup failed: {e}"))
    })
}

unsafe fn ec_distann_noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> Result<*mut pg_sys::IndexBulkDeleteResult, String> {
    let stats = if stats.is_null() {
        alloc_index_bulk_delete_result().into()
    } else {
        stats
    };
    let stats_handle =
        ptr::NonNull::new(stats).ok_or_else(|| "ec_distann vacuum stats is null".to_owned())?;
    let index_relation_handle = ptr::NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann vacuum stats needs a valid index relation".to_owned())?;
    let block_count = crate::storage::relation::main_fork_block_count_handle(index_relation_handle);
    let live_tuples = ambuild::read_metadata_from_index_handle(index_relation_handle)
        .map(|metadata| metadata.node_count)
        .unwrap_or(0);
    set_index_bulk_delete_summary(stats_handle, block_count, live_tuples);

    Ok(stats)
}

/// Eager scan state (ADR-056 pattern): amrescan runs the FR-081 loop to
/// completion; amgettuple is a cursor over the finished results.
///
/// Only the first `proven_k` entries of `result_buf` are ordering-proven
/// when `early_exit` fired (the D9 exit shows the beam cannot improve the
/// kth exact distance, nothing deeper). When a consumer reads past that
/// prefix, `amgettuple` transparently re-runs the orchestration with a
/// doubled exit bar (iterative deepening) instead of serving unproven
/// rows — so `LIMIT > ec_distann.top_k` stays correct without manual GUC
/// tuning. Deepening terminates: once k exceeds the BW×H expansion cap the
/// early-exit can no longer fire and the buffer is the complete FR-081
/// answer.
struct DistannScanOpaque {
    raw_query: Vec<f32>,
    result_buf: Vec<DistannScanHit>,
    result_cursor: usize,
    proven_k: usize,
    early_exit: bool,
    rescan_called: bool,
}

unsafe extern "C-unwind" fn ec_distann_ambeginscan(
    index_relation: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    pg_am_callback!({
        let scan = pg_sys::RelationGetIndexScan(index_relation, nkeys, norderbys);
        if scan.is_null() {
            pgrx::error!("ec_distann failed to allocate scan descriptor");
        }
        let opaque =
            PgBox::<DistannScanOpaque>::alloc_in_context(PgMemoryContexts::CurrentMemoryContext);
        ptr::write(
            opaque.as_ptr(),
            DistannScanOpaque {
                raw_query: Vec::new(),
                result_buf: Vec::new(),
                result_cursor: 0,
                proven_k: 0,
                early_exit: false,
                rescan_called: false,
            },
        );
        (*scan).opaque = opaque.into_pg().cast();
        scan
    })
}

unsafe extern "C-unwind" fn ec_distann_amrescan(
    scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    orderbys: pg_sys::ScanKey,
    norderbys: std::ffi::c_int,
) {
    pg_am_callback!({
        if scan.is_null() {
            pgrx::error!("ec_distann amrescan received a null scan descriptor");
        }
        if nkeys != 0 {
            pgrx::error!("ec_distann scan does not support index quals");
        }
        if norderbys != 1 {
            pgrx::error!("ec_distann scan currently requires exactly one ORDER BY query");
        }
        if orderbys.is_null() {
            pgrx::error!("ec_distann amrescan received null order-by scan keys");
        }

        let opaque_ptr = (*scan).opaque.cast::<DistannScanOpaque>();
        if opaque_ptr.is_null() {
            pgrx::error!("ec_distann amrescan missing scan opaque state");
        }
        let opaque = &mut *opaque_ptr;
        opaque.result_buf.clear();
        opaque.result_cursor = 0;
        opaque.proven_k = 0;
        opaque.early_exit = false;
        opaque.rescan_called = true;

        let orderby = &*orderbys;
        if (orderby.sk_flags as u32) & pg_sys::SK_ISNULL != 0 {
            pgrx::error!("ec_distann scan query must not be NULL");
        }
        let raw_query =
            Vec::<f32>::from_polymorphic_datum(orderby.sk_argument, false, pg_sys::FLOAT4ARRAYOID)
                .unwrap_or_else(|| {
                    pgrx::error!("ec_distann scan requires a real[] ORDER BY query")
                });
        if raw_query.is_empty() {
            pgrx::error!("ec_distann scan query must not be empty");
        }
        opaque.raw_query = raw_query;

        (*scan).xs_recheck = false;
        (*scan).xs_recheckorderby = false;
        (*scan).xs_orderbyvals = ptr::null_mut();
        (*scan).xs_orderbynulls = ptr::null_mut();

        execute_distann_scan(scan, opaque, options::current_top_k());
    })
}

/// Run (or re-run, for iterative deepening) the FR-081 orchestration into
/// the scan opaque. `effective_top_k` is the D9 exit bar for this attempt.
unsafe fn execute_distann_scan(
    scan: pg_sys::IndexScanDesc,
    opaque: &mut DistannScanOpaque,
    effective_top_k: usize,
) {
    {
        // Cloned so the opaque can be mutated at the end of the attempt
        // (1536 floats; negligible next to the scan itself).
        let raw_query = opaque.raw_query.clone();
        let raw_query = raw_query.as_slice();
        let index_relation = (*scan).indexRelation;
        let handle = ptr::NonNull::new(index_relation)
            .unwrap_or_else(|| pgrx::error!("ec_distann scan received a null index relation"));
        let metadata = ambuild::read_metadata_from_index_handle(handle)
            .unwrap_or_else(|e| pgrx::error!("ec_distann scan metadata read failed: {e}"));
        if metadata.dimensions == 0 || metadata.node_count == 0 {
            // Empty index -> zero rows (FR-081); nothing to deepen either.
            opaque.result_buf = Vec::new();
            opaque.proven_k = effective_top_k;
            opaque.early_exit = false;
            return;
        }
        if raw_query.len() != usize::from(metadata.dimensions) {
            pgrx::error!(
                "ec_distann scan query dimension mismatch: index dim {}, query dim {}",
                metadata.dimensions,
                raw_query.len()
            );
        }

        let index_oid = (*index_relation).rd_id;
        let entry = head_cache::cached_index_entry(index_oid.into(), handle, &metadata)
            .unwrap_or_else(|e| pgrx::error!("ec_distann scan head-cache setup failed: {e}"));
        let prepared_query =
            DistannPreparedQuery::prepare(&metadata, entry.flat_codebooks.as_deref(), raw_query)
                .unwrap_or_else(|e| pgrx::error!("ec_distann scan query preparation failed: {e}"));
        let code_len = quantizer::metadata_code_len(&metadata)
            .unwrap_or_else(|e| pgrx::error!("ec_distann scan code length failed: {e}"));

        let beam_width = options::current_beam_width();
        let hop_rounds = options::current_hop_rounds();
        let top_k = effective_top_k;

        // FR-080 head-index descent: exact -ip over the sample vectors,
        // zero remote calls; the frontier seeds the hop rounds.
        let head_list_size = (beam_width * 2).max(32).min(entry.head_vectors.len());
        let head_result = crate::am::greedy_search(
            &entry.head_graph,
            entry.head_entry,
            head_list_size,
            |node: u32| {
                -crate::am::ec_diskann::source_inner_product(
                    raw_query,
                    &entry.head_vectors[node as usize],
                )
            },
        );
        let seeds: Vec<DistannSeedCandidate> = head_result
            .frontier
            .iter()
            .map(|candidate| DistannSeedCandidate {
                vec_id: entry.head_vec_ids[candidate.node as usize],
                dist: candidate.distance,
            })
            .collect();

        let scan_desc = DiskannScanDescView::from_raw(scan, "ec_distann scan");
        let heap_relation_state = scan_desc
            .resolve_heap_relation()
            .unwrap_or_else(|e| pgrx::error!("ec_distann scan heap relation setup failed: {e}"));
        let snapshot_state = scan_desc
            .resolve_snapshot()
            .unwrap_or_else(|e| pgrx::error!("ec_distann scan snapshot setup failed: {e}"));
        let source_attnum = indexed_ecvector_attnum(index_relation).unwrap_or_else(|e| {
            pgrx::error!("ec_distann scan source-column resolution failed: {e}")
        });
        let heap_relation = heap_relation_state.as_ptr();
        let slot = crate::storage::slot_guard::TupleTableSlotGuard::single_for_heap(heap_relation)
            .unwrap_or_else(|| pgrx::error!("ec_distann scan heap slot setup failed"));

        let mut expander = LocalNodeExpander {
            index_handle: handle,
            directory: &entry.directory,
            graph_degree_r: metadata.graph_degree_r,
            code_len,
            prepared_query: &prepared_query,
            heap_relation,
            snapshot: snapshot_state.as_ptr(),
            slot: slot.as_ptr(),
            source_attnum,
            raw_query,
            pooled_node: DistannNodeTuple::placeholder(metadata.graph_degree_r, code_len),
        };
        let (hits, counters) = distann_orchestrated_search(
            &seeds,
            &mut expander,
            DistannOrchestrationParams {
                beam_width,
                hop_rounds,
                top_k,
            },
        )
        .unwrap_or_else(|e| pgrx::error!("ec_distann scan orchestration failed: {e}"));

        if options::scan_profile_notice_enabled() {
            pgrx::notice!(
                "ec_distann_scan_profile beam_width={} hop_rounds={} top_k={} rounds_executed={} records_expanded={} neighbors_code_scored={} early_exit={} beam_exhausted={} result_count={}",
                beam_width,
                hop_rounds,
                top_k,
                counters.rounds_executed,
                counters.records_expanded,
                counters.neighbors_code_scored,
                counters.early_exit,
                counters.beam_exhausted,
                hits.len(),
            );
        }
        opaque.result_buf = hits;
        opaque.proven_k = top_k;
        opaque.early_exit = counters.early_exit;
    }
}

unsafe extern "C-unwind" fn ec_distann_amgettuple(
    scan: pg_sys::IndexScanDesc,
    direction: pg_sys::ScanDirection::Type,
) -> bool {
    pg_am_callback!({
        if scan.is_null() {
            pgrx::error!("ec_distann amgettuple received a null scan descriptor");
        }
        if direction != pg_sys::ScanDirection::ForwardScanDirection {
            pgrx::error!("ec_distann amgettuple only supports forward scan direction");
        }
        let opaque_ptr = (*scan).opaque.cast::<DistannScanOpaque>();
        if opaque_ptr.is_null() {
            pgrx::error!("ec_distann amgettuple missing scan opaque state");
        }
        let opaque = &mut *opaque_ptr;
        if !opaque.rescan_called {
            pgrx::error!("ec_distann amgettuple requires amrescan before scan execution");
        }
        // Iterative deepening: never serve a row past the proven prefix of
        // an early-exited scan; re-run with a doubled exit bar instead.
        while opaque.early_exit && opaque.result_cursor >= opaque.proven_k {
            let deeper_k = opaque
                .proven_k
                .saturating_mul(2)
                .max(opaque.result_cursor + 1);
            execute_distann_scan(scan, opaque, deeper_k);
        }
        if opaque.result_cursor >= opaque.result_buf.len() {
            return false;
        }
        let hit = opaque.result_buf[opaque.result_cursor];
        opaque.result_cursor += 1;
        crate::am::ec_diskann::set_scan_heap_tid(scan, hit.heap_tid);
        (*scan).xs_recheckorderby = false;
        true
    })
}

unsafe extern "C-unwind" fn ec_distann_amendscan(scan: pg_sys::IndexScanDesc) {
    pg_am_callback!({
        if scan.is_null() {
            return;
        }
        let opaque_ptr = (*scan).opaque.cast::<DistannScanOpaque>();
        if !opaque_ptr.is_null() {
            ptr::drop_in_place(opaque_ptr);
            pg_sys::pfree(opaque_ptr.cast());
            (*scan).opaque = ptr::null_mut();
        }
    })
}

fn indexed_ecvector_attnum(index_relation: pg_sys::Relation) -> Result<i32, String> {
    // SAFETY: The index relation is live; BuildIndexInfo returns palloc'd
    // metadata that remains valid until it is released at the end of this block.
    unsafe {
        let index_info = pg_sys::BuildIndexInfo(index_relation);
        if index_info.is_null() {
            return Err("ec_distann scan could not build index metadata".into());
        }
        let info = &*index_info;
        // One key column; an optional second attr is the ADR-063
        // source_identity INCLUDE column.
        let result = if info.ii_NumIndexKeyAttrs != 1 || info.ii_NumIndexAttrs > 2 {
            Err("ec_distann scan currently supports single-key-column indexes only".into())
        } else {
            let attnum = i32::from(info.ii_IndexAttrNumbers[0]);
            if attnum <= 0 {
                Err("ec_distann scan requires a base heap column index key".into())
            } else {
                Ok(attnum)
            }
        };
        pg_sys::pfree(index_info.cast());
        result
    }
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_distann_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    true
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn ec_distann_handler(
    _fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    // `#[pg_guard]` is the pgrx boundary guard for this PostgreSQL callback.
    pg_sys::Datum::from(build_ec_distann_routine().into_pg())
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_ec_distann_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}
