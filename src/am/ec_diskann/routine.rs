#[cfg(any(test, feature = "pg_test"))]
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex, OnceLock,
};
use std::{cell::RefCell, collections::HashSet, ffi::c_void, ptr, slice};

use pgrx::{pg_guard, pg_sys, AllocatedByRust, FromDatum, PgBox, PgMemoryContexts};

use crate::{
    quant::grouped_pq::{build_grouped_pq_lut_f32, grouped_pq_score_f32, GROUPED_PQ_CENTROIDS},
    storage::{
        page::{DataPageChain, ItemPointer},
        wal,
    },
};

use super::{
    ambuild, cost, insert, maybe_check_for_interrupts, options,
    page::{VamanaMetadataPage, PAYLOAD_FLAG_BINARY_SIDECAR},
    reader::{PersistedGraphReader, VisitedState},
    scan::{self, ScanParams},
    scan_query::{
        build_grouped_pq_lut_from_persisted, encode_query_srht, hamming_xor_popcount,
        pack_query_sign_bits, read_grouped_codebook_chain,
    },
    scan_state::{self, DiskannScanOpaque},
    tuple::VamanaNodeTuple,
    vacuum, warn_on_non_unit_source_vector,
};

type BulkDeleteCallback = unsafe extern "C-unwind" fn(pg_sys::ItemPointer, *mut c_void) -> bool;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TupleRewrite {
    tid: ItemPointer,
    expected_raw: Vec<u8>,
    replacement_raw: Vec<u8>,
}

const MAX_REPAIR_REPLAN_PASSES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VacuumRewriteApplyOutcome {
    Applied,
    RetryReplan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VacuumBulkDeletePassResult {
    rewrite_outcome: VacuumRewriteApplyOutcome,
    block_count: pg_sys::BlockNumber,
    live_tuple_count: usize,
    removed_heap_tids: usize,
    entry_point_needs_medoid_refresh: bool,
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone)]
struct VacuumRewriteTestInjection {
    target_tid: ItemPointer,
    replacement_raw: Vec<u8>,
}

#[cfg(any(test, feature = "pg_test"))]
fn vacuum_rewrite_test_injection_cell() -> &'static Mutex<Option<VacuumRewriteTestInjection>> {
    static CELL: OnceLock<Mutex<Option<VacuumRewriteTestInjection>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

#[cfg(any(test, feature = "pg_test"))]
fn vacuum_replan_event_counter() -> &'static AtomicUsize {
    static COUNTER: OnceLock<AtomicUsize> = OnceLock::new();
    COUNTER.get_or_init(|| AtomicUsize::new(0))
}

#[cfg(any(test, feature = "pg_test"))]
fn record_vacuum_replan_event() {
    vacuum_replan_event_counter().fetch_add(1, Ordering::SeqCst);
}

#[cfg(not(any(test, feature = "pg_test")))]
fn record_vacuum_replan_event() {}

#[cfg(any(test, feature = "pg_test"))]
fn set_vacuum_rewrite_test_injection(injection: VacuumRewriteTestInjection) {
    *vacuum_rewrite_test_injection_cell()
        .lock()
        .expect("vacuum rewrite injection lock should not be poisoned") = Some(injection);
}

#[cfg(any(test, feature = "pg_test"))]
fn clear_vacuum_rewrite_test_state() {
    *vacuum_rewrite_test_injection_cell()
        .lock()
        .expect("vacuum rewrite injection lock should not be poisoned") = None;
    vacuum_replan_event_counter().store(0, Ordering::SeqCst);
}

#[cfg(any(test, feature = "pg_test"))]
fn vacuum_replan_event_count() -> usize {
    vacuum_replan_event_counter().load(Ordering::SeqCst)
}

#[cfg(any(test, feature = "pg_test"))]
fn take_vacuum_rewrite_test_injection() -> Option<VacuumRewriteTestInjection> {
    vacuum_rewrite_test_injection_cell()
        .lock()
        .expect("vacuum rewrite injection lock should not be poisoned")
        .take()
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn maybe_apply_vacuum_rewrite_test_injection(
    index_relation: pg_sys::Relation,
) -> Result<(), String> {
    let Some(injection) = take_vacuum_rewrite_test_injection() else {
        return Ok(());
    };
    unsafe {
        write_raw_tuple_bytes(
            index_relation,
            injection.target_tid,
            &injection.replacement_raw,
        )
    }
}

#[cfg(not(any(test, feature = "pg_test")))]
unsafe fn maybe_apply_vacuum_rewrite_test_injection(
    _index_relation: pg_sys::Relation,
) -> Result<(), String> {
    Ok(())
}

fn build_ec_diskann_routine() -> PgBox<pg_sys::IndexAmRoutine, AllocatedByRust> {
    // SAFETY: `IndexAmRoutine` is a PostgreSQL Node type and must be allocated
    // with the corresponding node tag.
    let mut amroutine =
        unsafe { PgBox::<pg_sys::IndexAmRoutine>::alloc_node(pg_sys::NodeTag::T_IndexAmRoutine) };

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
    amroutine.amcanbuildparallel = false;
    amroutine.amcaninclude = false;
    amroutine.amusemaintenanceworkmem = true;
    amroutine.amsummarizing = false;
    amroutine.amparallelvacuumoptions = 0;
    amroutine.amkeytype = pg_sys::InvalidOid;

    amroutine.ambuild = Some(ambuild::ec_diskann_ambuild);
    amroutine.ambuildempty = Some(ambuild::ec_diskann_ambuildempty);
    amroutine.aminsert = Some(ec_diskann_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(ec_diskann_ambulkdelete);
    amroutine.amvacuumcleanup = Some(ec_diskann_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(cost::ec_diskann_amcostestimate);
    amroutine.amoptions = Some(options::ec_diskann_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(ec_diskann_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(ec_diskann_ambeginscan);
    amroutine.amrescan = Some(ec_diskann_amrescan);
    amroutine.amgettuple = Some(ec_diskann_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(ec_diskann_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = None;
    amroutine.aminitparallelscan = None;
    amroutine.amparallelrescan = None;

    amroutine
}

unsafe extern "C-unwind" fn ec_diskann_aminsert(
    index_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: pg_sys::ItemPointer,
    heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if values.is_null() || isnull.is_null() {
                pgrx::error!("ec_diskann aminsert received null datum arrays");
            }
            if *isnull {
                pgrx::error!("ec_diskann does not support NULL indexed values");
            }
            let datum = *values;
            if datum.is_null() {
                pgrx::error!("ec_diskann aminsert received a null indexed datum");
            }

            let source_vector = ambuild::ecvector_datum_to_vec(datum);
            warn_on_non_unit_source_vector(&source_vector, "aminsert");
            let heap_tid = ambuild::decode_heap_tid(heap_tid);
            let metadata = insert::read_metadata_page(index_relation).unwrap_or_else(|e| {
                pgrx::error!("ec_diskann aminsert failed to read metadata: {e}")
            });

            if metadata.dimensions == 0 && metadata.entry_point == ItemPointer::INVALID {
                let bootstrapped = insert::with_locked_metadata_page(index_relation, |metadata| {
                    if metadata.dimensions != 0 || metadata.entry_point != ItemPointer::INVALID {
                        return Ok(false);
                    }
                    let output = insert::bootstrap_empty_insert_output(
                        index_relation,
                        heap_tid,
                        &source_vector,
                    )?;
                    ambuild::write_data_pages(index_relation, &output.chain);
                    *metadata = output.metadata;
                    Ok(true)
                })
                .unwrap_or_else(|e| {
                    pgrx::error!("ec_diskann empty-index bootstrap insert failed: {e}")
                });
                if bootstrapped {
                    return false;
                }
            }

            let refreshed = insert::read_metadata_page(index_relation).unwrap_or_else(|e| {
                pgrx::error!("ec_diskann aminsert failed to refresh metadata: {e}")
            });
            if refreshed.dimensions != 0 && source_vector.len() != refreshed.dimensions as usize {
                pgrx::error!(
                    "ec_diskann insert source dimension mismatch: source dim {}, index dim {}",
                    source_vector.len(),
                    refreshed.dimensions
                );
            }

            let (materialized_metadata, chain) =
                scan_state::materialize_chain_from_index(index_relation).unwrap_or_else(|e| {
                    pgrx::error!("ec_diskann aminsert failed to materialize persisted chain: {e}")
                });
            let payload = insert::derive_insert_payload_from_persisted(
                &materialized_metadata,
                &chain,
                &source_vector,
            )
            .unwrap_or_else(|e| {
                pgrx::error!("ec_diskann aminsert failed to derive insert payload: {e}")
            });
            let reader = PersistedGraphReader::new(
                &chain,
                materialized_metadata.graph_degree_r,
                scan_state::metadata_binary_word_count(&materialized_metadata),
                scan_state::metadata_search_code_len(&materialized_metadata),
            );
            let duplicate_candidates =
                insert::duplicate_candidate_tids_by_payload(&reader, &payload).unwrap_or_else(
                    |e| pgrx::error!("ec_diskann aminsert failed to probe duplicate payloads: {e}"),
                );
            let source_attnum = indexed_ecvector_attnum(index_relation).unwrap_or_else(|e| {
                pgrx::error!("ec_diskann aminsert could not resolve indexed ecvector column: {e}")
            });
            if !duplicate_candidates.is_empty() {
                let slot = scan_state::allocate_heap_slot(heap_relation).unwrap_or_else(|e| {
                    pgrx::error!("ec_diskann aminsert could not allocate duplicate-probe slot: {e}")
                });
                let snapshot = std::ptr::addr_of_mut!(pg_sys::SnapshotSelfData);
                let duplicate_tid = duplicate_candidates.into_iter().find(|candidate_tid| {
                    let Ok(candidate_tuple) = reader.read_node(*candidate_tid) else {
                        return false;
                    };
                    if candidate_tuple.primary_heaptid == ItemPointer::INVALID {
                        return false;
                    }
                    let Ok(existing_vector) = fetch_heap_source_vector(
                        heap_relation,
                        snapshot,
                        slot,
                        source_attnum,
                        candidate_tuple.primary_heaptid,
                        "duplicate probe source vector",
                    ) else {
                        return false;
                    };
                    existing_vector == source_vector
                });
                pg_sys::ExecDropSingleTupleTableSlot(slot);
                if let Some(existing_tid) = duplicate_tid {
                    insert::bind_duplicate_heap_tid(index_relation, existing_tid, heap_tid)
                        .unwrap_or_else(|e| pgrx::error!("ec_diskann duplicate bind failed: {e}"));
                    return false;
                }
            }

            let entry_point = scan::resolve_entry_point(&reader, materialized_metadata.entry_point)
                .unwrap_or_else(|e| {
                    pgrx::error!(
                        "ec_diskann unique insert planning could not resolve entry point: {e}"
                    )
                });
            let Some(entry_point) = entry_point else {
                pgrx::error!("ec_diskann unique insert planning found no live entry point");
            };

            let group_count = usize::from(materialized_metadata.search_subvector_count);
            let group_size = usize::from(materialized_metadata.search_subvector_dim);
            if group_count == 0 || group_size == 0 {
                pgrx::error!(
                    "ec_diskann unique insert planning requires grouped-PQ metadata: group_count={}, group_size={}",
                    group_count,
                    group_size
                );
            }
            let build_list_size = usize::from(materialized_metadata.build_list_size_l);
            if build_list_size == 0 {
                pgrx::error!("ec_diskann unique insert planning requires build_list_size_l > 0");
            }
            let (query_lut, helper_group_count) = build_grouped_pq_lut_from_persisted(
                &chain,
                materialized_metadata.grouped_codebook_head,
                group_count,
                group_size,
                materialized_metadata.dimensions as usize,
                materialized_metadata.seed,
                &source_vector,
            )
            .unwrap_or_else(|e| {
                pgrx::error!(
                    "ec_diskann unique insert planning failed to build grouped-PQ LUT: {e}"
                )
            });
            if helper_group_count != group_count {
                pgrx::error!(
                    "ec_diskann unique insert planning grouped-PQ helper returned group_count {}, expected {}",
                    helper_group_count,
                    group_count
                );
            }

            let slot = scan_state::allocate_heap_slot(heap_relation).unwrap_or_else(|e| {
                pgrx::error!("ec_diskann unique insert planning could not allocate heap slot: {e}")
            });
            let snapshot = std::ptr::addr_of_mut!(pg_sys::SnapshotSelfData);
            let rerank_error = RefCell::new(None::<String>);
            let mut visited = VisitedState::new();
            let exact_candidates = scan::vamana_scan_with(
                &reader,
                &mut visited,
                ScanParams {
                    entry_point,
                    list_size: build_list_size,
                    rerank_budget: build_list_size,
                    top_k: build_list_size,
                },
                |tuple| -grouped_pq_score_f32(&query_lut, group_count, &tuple.search_code),
                |_: &[ItemPointer]| {},
                |heap_tid| match exact_heap_rerank_distance(
                    heap_relation,
                    snapshot,
                    slot,
                    source_attnum,
                    &source_vector,
                    heap_tid,
                ) {
                    Ok(distance) => distance,
                    Err(error) => {
                        if rerank_error.borrow().is_none() {
                            *rerank_error.borrow_mut() = Some(error);
                        }
                        f32::INFINITY
                    }
                },
            )
            .unwrap_or_else(|e| pgrx::error!("ec_diskann unique insert planning scan failed: {e}"));
            if let Some(error) = rerank_error.into_inner() {
                pg_sys::ExecDropSingleTupleTableSlot(slot);
                pgrx::error!("ec_diskann unique insert planning exact rerank failed: {error}");
            }
            let planning_candidates = exact_candidates
                .into_iter()
                .map(|candidate| {
                    let source_vector = fetch_heap_source_vector(
                        heap_relation,
                        snapshot,
                        slot,
                        source_attnum,
                        candidate.primary_heaptid,
                        "forward-neighbor planning source vector",
                    )
                    .unwrap_or_else(|e| {
                        pgrx::error!(
                            "ec_diskann unique insert planning could not materialize candidate heap vector: {e}"
                        )
                    });
                    insert::ForwardNeighborCandidate {
                        tid: candidate.tid,
                        source_vector,
                    }
                })
                .collect::<Vec<_>>();
            pg_sys::ExecDropSingleTupleTableSlot(slot);

            let forward_neighbors = insert::select_insert_forward_neighbors(
                &source_vector,
                &planning_candidates,
                materialized_metadata.alpha,
                materialized_metadata.graph_degree_r as usize,
            )
            .unwrap_or_else(|e| {
                pgrx::error!("ec_diskann unique insert forward-neighbor selection failed: {e}")
            });
            let new_tid = insert::append_live_node(
                index_relation,
                &materialized_metadata,
                heap_tid,
                &payload,
                &forward_neighbors,
            )
            .unwrap_or_else(|e| pgrx::error!("ec_diskann unique insert append failed: {e}"));
            install_backlinks_with_replan(
                index_relation,
                heap_relation,
                source_attnum,
                &forward_neighbors,
                new_tid,
                &source_vector,
            )
            .unwrap_or_else(|e| {
                pgrx::error!("ec_diskann unique insert backlink update failed: {e}")
            });
            insert::increment_inserted_since_rebuild(index_relation).unwrap_or_else(|e| {
                pgrx::error!("ec_diskann unique insert metadata update failed: {e}")
            });
            false
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: pg_sys::IndexBulkDeleteCallback,
    callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let Some(callback) = callback else {
                return ec_diskann_noop_vacuum_stats((*info).index, stats)
                    .unwrap_or_else(|e| pgrx::error!("ec_diskann ambulkdelete failed: {e}"));
            };
            run_diskann_bulkdelete(
                (*info).index,
                (*info).heaprel,
                stats,
                callback,
                callback_state,
            )
            .unwrap_or_else(|e| pgrx::error!("ec_diskann ambulkdelete failed: {e}"))
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            ec_diskann_noop_vacuum_stats((*info).index, stats)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann amvacuumcleanup failed: {e}"))
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_ambeginscan(
    index_relation: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let scan = pg_sys::RelationGetIndexScan(index_relation, nkeys, norderbys);
            if scan.is_null() {
                pgrx::error!("ec_diskann failed to allocate scan descriptor");
            }

            let (metadata, chain) = scan_state::materialize_chain_from_index(index_relation)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann ambeginscan failed: {e}"));
            let options = options::relation_options(index_relation);
            let opaque_state = DiskannScanOpaque::new(metadata, chain, options)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann ambeginscan failed: {e}"));

            let opaque = PgBox::<DiskannScanOpaque>::alloc_in_context(
                PgMemoryContexts::CurrentMemoryContext,
            );
            ptr::write(opaque.as_ptr(), opaque_state);
            (*scan).opaque = opaque.into_pg().cast();
            scan
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amrescan(
    scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    orderbys: pg_sys::ScanKey,
    norderbys: std::ffi::c_int,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_diskann amrescan received a null scan descriptor");
            }
            if nkeys != 0 {
                pgrx::error!("ec_diskann scan does not support index quals");
            }
            if norderbys != 1 {
                pgrx::error!("ec_diskann scan currently requires exactly one ORDER BY query");
            }
            if orderbys.is_null() {
                pgrx::error!("ec_diskann amrescan received null order-by scan keys");
            }

            let opaque_ptr = (*scan).opaque.cast::<DiskannScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_diskann amrescan missing scan opaque state");
            }
            let opaque = &mut *opaque_ptr;
            let orderby = &*orderbys;
            if (orderby.sk_flags as u32) & pg_sys::SK_ISNULL != 0 {
                pgrx::error!("ec_diskann scan query must not be NULL");
            }

            let raw_query = Vec::<f32>::from_polymorphic_datum(
                orderby.sk_argument,
                false,
                pg_sys::FLOAT4ARRAYOID,
            )
            .unwrap_or_else(|| pgrx::error!("ec_diskann scan requires a real[] ORDER BY query"));
            if raw_query.is_empty() {
                pgrx::error!("ec_diskann scan query must not be empty");
            }
            if opaque.metadata.dimensions != 0
                && raw_query.len() != opaque.metadata.dimensions as usize
            {
                pgrx::error!(
                    "ec_diskann scan query dimension mismatch: index dim {}, query dim {}",
                    opaque.metadata.dimensions,
                    raw_query.len()
                );
            }

            (*scan).xs_recheck = false;
            (*scan).xs_recheckorderby = false;
            (*scan).xs_orderbyvals = ptr::null_mut();
            (*scan).xs_orderbynulls = ptr::null_mut();

            opaque.flat_codebooks.clear();
            opaque.query_rotated.clear();
            opaque.query_lut.clear();
            opaque.query_binary_words.clear();
            opaque.visited.clear();
            opaque.result_buf.clear();
            opaque.result_cursor = 0;

            if opaque.metadata.dimensions == 0 {
                opaque.rescan_called = true;
                return;
            }

            let prefilter = prepare_prefilter(
                &opaque.chain,
                &opaque.metadata,
                &raw_query,
                options::current_prefilter_kind(),
                "scan",
            )
            .unwrap_or_else(|e| pgrx::error!("ec_diskann scan prefilter setup failed: {e}"));

            let reader = PersistedGraphReader::new(
                &opaque.chain,
                opaque.metadata.graph_degree_r,
                opaque.binary_word_count(),
                opaque.search_code_len(),
            );
            let entry_point = scan::resolve_entry_point(&reader, opaque.metadata.entry_point)
                .unwrap_or_else(|e| {
                    pgrx::error!("ec_diskann scan entry-point resolution failed: {e}")
                });
            let Some(entry_point) = entry_point else {
                opaque.rescan_called = true;
                return;
            };

            let heap_relation_state =
                scan_state::resolve_scan_heap_relation(scan).unwrap_or_else(|e| {
                    pgrx::error!("ec_diskann scan heap relation setup failed: {e}")
                });
            let snapshot_state = scan_state::resolve_scan_snapshot(scan)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann scan snapshot setup failed: {e}"));
            let slot = scan_state::allocate_heap_slot(heap_relation_state.0)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann scan heap slot setup failed: {e}"));
            let source_attnum =
                indexed_ecvector_attnum((*scan).indexRelation).unwrap_or_else(|e| {
                    pgrx::error!("ec_diskann scan source-column resolution failed: {e}")
                });
            let rerank_error = RefCell::new(None::<String>);
            let sql_result_cap = sql_scan_result_cap(opaque.top_k, opaque.rerank_budget);
            let scan_params = ScanParams {
                entry_point,
                list_size: opaque.list_size,
                rerank_budget: opaque.rerank_budget,
                top_k: sql_result_cap,
            };
            let results = scan::vamana_scan_with(
                &reader,
                &mut opaque.visited,
                scan_params,
                |tuple| prefilter.score(tuple),
                |heap_tids: &[ItemPointer]| {
                    prefetch_heap_rerank_blocks(heap_relation_state.0, heap_tids)
                },
                |heap_tid| match exact_heap_rerank_distance(
                    heap_relation_state.0,
                    snapshot_state.0,
                    slot,
                    source_attnum,
                    &raw_query,
                    heap_tid,
                ) {
                    Ok(distance) => distance,
                    Err(error) => {
                        if rerank_error.borrow().is_none() {
                            *rerank_error.borrow_mut() = Some(error);
                        }
                        f32::INFINITY
                    }
                },
            );
            prefilter.load_into_scan_opaque(opaque);
            scan_state::release_owned_scan_heap_state(
                heap_relation_state.0,
                heap_relation_state.1,
                snapshot_state.0,
                snapshot_state.1,
                slot,
            );

            if let Some(error) = rerank_error.into_inner() {
                pgrx::error!("ec_diskann scan heap rerank failed: {error}");
            }
            let node_results =
                results.unwrap_or_else(|e| pgrx::error!("ec_diskann scan execution failed: {e}"));
            opaque.result_buf = expand_scan_results_with_bound_heap_tids(
                &opaque.chain,
                &node_results,
                sql_result_cap,
            )
            .unwrap_or_else(|e| pgrx::error!("ec_diskann duplicate expansion failed: {e}"));
            opaque.result_cursor = 0;
            opaque.rescan_called = true;
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amgettuple(
    scan: pg_sys::IndexScanDesc,
    direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("ec_diskann amgettuple received a null scan descriptor");
            }
            if direction != pg_sys::ScanDirection::ForwardScanDirection {
                pgrx::error!("ec_diskann amgettuple only supports forward scan direction");
            }

            let opaque_ptr = (*scan).opaque.cast::<DiskannScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("ec_diskann amgettuple missing scan opaque state");
            }
            let opaque = &mut *opaque_ptr;
            if !opaque.rescan_called {
                pgrx::error!("ec_diskann amgettuple requires amrescan before scan execution");
            }
            if opaque.result_cursor >= opaque.result_buf.len() {
                return false;
            }

            let hit = opaque.result_buf[opaque.result_cursor];
            opaque.result_cursor += 1;
            scan_state::set_scan_heap_tid(scan, hit.primary_heaptid);
            (*scan).xs_recheckorderby = false;
            true
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amendscan(scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                return;
            }

            let opaque_ptr = (*scan).opaque.cast::<DiskannScanOpaque>();
            if !opaque_ptr.is_null() {
                ptr::drop_in_place(opaque_ptr);
                pg_sys::pfree(opaque_ptr.cast());
                (*scan).opaque = ptr::null_mut();
            }
        })
    }
}

unsafe fn indexed_ecvector_attnum(index_relation: pg_sys::Relation) -> Result<i32, String> {
    let index_info = unsafe { pg_sys::BuildIndexInfo(index_relation) };
    if index_info.is_null() {
        return Err("ec_diskann scan could not build index metadata".into());
    }
    let info = unsafe { &*index_info };
    let result = if info.ii_NumIndexAttrs != 1 || info.ii_NumIndexKeyAttrs != 1 {
        Err("ec_diskann scan currently supports single-column indexes only".into())
    } else {
        let attnum = i32::from(info.ii_IndexAttrNumbers[0]);
        if attnum <= 0 {
            Err("ec_diskann scan requires a base heap column index key".into())
        } else {
            Ok(attnum)
        }
    };
    unsafe { pg_sys::pfree(index_info.cast()) };
    result
}

unsafe fn install_backlinks_with_replan(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    source_attnum: i32,
    backlink_targets: &[ItemPointer],
    new_tid: ItemPointer,
    new_source_vector: &[f32],
) -> Result<(), String> {
    let mut pending = backlink_targets.to_vec();
    sort_and_dedup_item_pointers(&mut pending);

    for _ in 0..insert::MAX_BACKLINK_REPLAN_PASSES {
        if pending.is_empty() {
            return Ok(());
        }

        let (metadata, mutations) = unsafe {
            plan_backlink_mutations(
                index_relation,
                heap_relation,
                source_attnum,
                &pending,
                new_tid,
                new_source_vector,
            )?
        };
        if mutations.is_empty() {
            return Ok(());
        }

        pending = unsafe {
            insert::apply_backlink_mutations(index_relation, &metadata, &mutations, new_tid)?
        };
    }

    if pending.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "ec_diskann backlink rewrite exceeded {} replan passes for {} target(s)",
            insert::MAX_BACKLINK_REPLAN_PASSES,
            pending.len()
        ))
    }
}

unsafe fn plan_backlink_mutations(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    source_attnum: i32,
    target_tids: &[ItemPointer],
    new_tid: ItemPointer,
    new_source_vector: &[f32],
) -> Result<(VamanaMetadataPage, Vec<insert::BacklinkMutation>), String> {
    let (metadata, chain) = unsafe { scan_state::materialize_chain_from_index(index_relation)? };
    let reader = PersistedGraphReader::new(
        &chain,
        metadata.graph_degree_r,
        scan_state::metadata_binary_word_count(&metadata),
        scan_state::metadata_search_code_len(&metadata),
    );
    let slot = unsafe { scan_state::allocate_heap_slot(heap_relation)? };
    let snapshot = std::ptr::addr_of_mut!(pg_sys::SnapshotSelfData);
    let planned = (|| -> Result<Vec<insert::BacklinkMutation>, String> {
        let mut mutations = Vec::new();
        for &target_tid in target_tids {
            let target_tuple = match reader.read_node(target_tid) {
                Ok(tuple) => tuple,
                Err(_) => continue,
            };
            if !target_tuple.is_live() || target_tuple.primary_heaptid == ItemPointer::INVALID {
                continue;
            }

            let target_source_vector = unsafe {
                fetch_heap_source_vector(
                    heap_relation,
                    snapshot,
                    slot,
                    source_attnum,
                    target_tuple.primary_heaptid,
                    "backlink planning target source vector",
                )?
            };
            let mut existing_candidates = Vec::new();
            for neighbor_tid in target_tuple
                .neighbors
                .iter()
                .take(target_tuple.neighbor_count as usize)
                .copied()
            {
                if neighbor_tid == ItemPointer::INVALID {
                    continue;
                }
                let neighbor_tuple = match reader.read_node(neighbor_tid) {
                    Ok(tuple) => tuple,
                    Err(_) => continue,
                };
                if !neighbor_tuple.is_live()
                    || neighbor_tuple.primary_heaptid == ItemPointer::INVALID
                {
                    continue;
                }
                let neighbor_source_vector = unsafe {
                    fetch_heap_source_vector(
                        heap_relation,
                        snapshot,
                        slot,
                        source_attnum,
                        neighbor_tuple.primary_heaptid,
                        "backlink planning neighbor source vector",
                    )?
                };
                existing_candidates.push(insert::ForwardNeighborCandidate {
                    tid: neighbor_tid,
                    source_vector: neighbor_source_vector,
                });
            }

            if let Some(mutation) = insert::plan_backlink_mutation(
                target_tid,
                &target_tuple,
                &target_source_vector,
                &existing_candidates,
                new_tid,
                new_source_vector,
                metadata.alpha,
                metadata.graph_degree_r as usize,
            )? {
                mutations.push(mutation);
            }
        }
        Ok(mutations)
    })();
    unsafe { pg_sys::ExecDropSingleTupleTableSlot(slot) };
    Ok((metadata, planned?))
}

fn sort_and_dedup_item_pointers(tids: &mut Vec<ItemPointer>) {
    tids.sort_unstable_by(insert::cmp_item_pointer_physical);
    tids.dedup();
}

unsafe fn ec_diskann_noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> Result<*mut pg_sys::IndexBulkDeleteResult, String> {
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };

    unsafe {
        (*stats).num_pages = pg_sys::RelationGetNumberOfBlocksInFork(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
        );
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = count_live_node_tuples(index_relation)? as f64;
    }

    Ok(stats)
}

unsafe fn run_diskann_bulkdelete(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> Result<*mut pg_sys::IndexBulkDeleteResult, String> {
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };
    let mut max_removed_heap_tids = 0usize;
    for _ in 0..MAX_REPAIR_REPLAN_PASSES {
        let pass = unsafe {
            run_diskann_bulkdelete_pass(index_relation, heap_relation, callback, callback_state)?
        };
        max_removed_heap_tids = max_removed_heap_tids.max(pass.removed_heap_tids);
        match pass.rewrite_outcome {
            VacuumRewriteApplyOutcome::Applied => {
                if pass.entry_point_needs_medoid_refresh {
                    unsafe {
                        insert::with_locked_metadata_page(index_relation, |metadata| {
                            metadata.needs_medoid_refresh = true;
                            Ok(())
                        })?
                    };
                }

                unsafe {
                    (*stats).num_pages = pass.block_count;
                    (*stats).estimated_count = false;
                    (*stats).num_index_tuples = pass.live_tuple_count as f64;
                    (*stats).tuples_removed += max_removed_heap_tids as f64;
                }
                return Ok(stats);
            }
            VacuumRewriteApplyOutcome::RetryReplan => record_vacuum_replan_event(),
        }
    }
    Err(format!(
        "ec_diskann vacuum repair exceeded {} replan passes",
        MAX_REPAIR_REPLAN_PASSES
    ))
}

unsafe fn run_diskann_bulkdelete_pass(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
) -> Result<VacuumBulkDeletePassResult, String> {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let (metadata, original_chain) =
        unsafe { scan_state::materialize_chain_from_index(index_relation)? };
    let graph_degree_r = metadata.graph_degree_r;
    let binary_word_count = scan_state::metadata_binary_word_count(&metadata);
    let search_code_len = scan_state::metadata_search_code_len(&metadata);
    let node_tids = collect_node_tids(
        &original_chain,
        graph_degree_r,
        binary_word_count,
        search_code_len,
    )?;
    let mut mutated_chain = original_chain.clone();

    let mut removed_heap_tids = 0usize;
    let mut finalize_tids = Vec::new();
    let mut repair_target_tids = Vec::new();
    for &tid in &node_tids {
        maybe_check_for_interrupts();

        let mut tuple = read_chain_node(
            &mutated_chain,
            graph_degree_r,
            binary_word_count,
            search_code_len,
            tid,
        )?;
        let original_tuple = tuple.clone();
        removed_heap_tids += insert::vacuum_bound_heap_rows(
            &mut mutated_chain,
            tid,
            &mut tuple,
            |heap_tid| unsafe { callback_marks_heap_tid_dead(callback, callback_state, heap_tid) },
        )?;
        if tuple != original_tuple {
            write_chain_node(
                &mut mutated_chain,
                graph_degree_r,
                binary_word_count,
                search_code_len,
                tid,
                &tuple,
            )?;
        }
        if vacuum::is_fully_dead(&tuple) {
            finalize_tids.push(tid);
        }
    }

    let dead_set: HashSet<_> = finalize_tids.iter().copied().collect();
    if !dead_set.is_empty() {
        for &tid in &node_tids {
            maybe_check_for_interrupts();

            let mut tuple = read_chain_node(
                &mutated_chain,
                graph_degree_r,
                binary_word_count,
                search_code_len,
                tid,
            )?;
            if !tuple.is_live() {
                continue;
            }
            if vacuum::repair_neighbors(&mut tuple, &dead_set) != 0 {
                repair_target_tids.push(tid);
                write_chain_node(
                    &mut mutated_chain,
                    graph_degree_r,
                    binary_word_count,
                    search_code_len,
                    tid,
                    &tuple,
                )?;
            }
        }
        let (heap_relation, heap_relation_owned) =
            unsafe { resolve_vacuum_heap_relation(index_relation, heap_relation)? };
        let fill_result = unsafe {
            fill_vacuum_neighbor_slots(
                index_relation,
                heap_relation,
                &metadata,
                &mut mutated_chain,
                &repair_target_tids,
                &dead_set,
            )
        };
        unsafe { release_owned_vacuum_heap_relation(heap_relation, heap_relation_owned) };
        fill_result?;
    }

    for &tid in &finalize_tids {
        let mut tuple = read_chain_node(
            &mutated_chain,
            graph_degree_r,
            binary_word_count,
            search_code_len,
            tid,
        )?;
        if tuple.deleted || !vacuum::is_fully_dead(&tuple) {
            continue;
        }
        vacuum::mark_deleted(&mut tuple);
        write_chain_node(
            &mut mutated_chain,
            graph_degree_r,
            binary_word_count,
            search_code_len,
            tid,
            &tuple,
        )?;
    }

    let rewrites = collect_tuple_rewrites(&original_chain, &mutated_chain)?;
    let rewrite_outcome = unsafe { apply_tuple_rewrites(index_relation, &rewrites)? };
    Ok(VacuumBulkDeletePassResult {
        rewrite_outcome,
        block_count,
        live_tuple_count: count_live_tuples_in_chain(
            &mutated_chain,
            graph_degree_r,
            binary_word_count,
            search_code_len,
        )?,
        removed_heap_tids,
        entry_point_needs_medoid_refresh: chain_entry_point_needs_medoid_refresh(
            &mutated_chain,
            &metadata,
        )?,
    })
}

unsafe fn resolve_vacuum_heap_relation(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
) -> Result<(pg_sys::Relation, bool), String> {
    if !heap_relation.is_null() {
        return Ok((heap_relation, false));
    }

    let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        return Err("ec_diskann vacuum could not resolve heap relation".into());
    }
    Ok((
        unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) },
        true,
    ))
}

unsafe fn release_owned_vacuum_heap_relation(
    heap_relation: pg_sys::Relation,
    heap_relation_owned: bool,
) {
    if heap_relation_owned && !heap_relation.is_null() {
        unsafe { pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }
}

unsafe fn fill_vacuum_neighbor_slots(
    index_relation: pg_sys::Relation,
    heap_relation: pg_sys::Relation,
    metadata: &VamanaMetadataPage,
    chain: &mut DataPageChain,
    repair_target_tids: &[ItemPointer],
    dead_set: &HashSet<ItemPointer>,
) -> Result<(), String> {
    if repair_target_tids.is_empty() {
        return Ok(());
    }
    let source_attnum = unsafe { indexed_ecvector_attnum(index_relation)? };
    let slot = unsafe { scan_state::allocate_heap_slot(heap_relation)? };
    let snapshot = std::ptr::addr_of_mut!(pg_sys::SnapshotSelfData);
    let mut visited = VisitedState::new();

    let repair_result = (|| -> Result<(), String> {
        for &target_tid in repair_target_tids {
            maybe_check_for_interrupts();

            let planner = VacuumFillPlanner {
                heap_relation,
                snapshot,
                slot,
                source_attnum,
                metadata,
                chain,
                dead_set,
            };
            let fill_candidates = unsafe {
                plan_vacuum_fill_candidates_for_target(&planner, target_tid, &mut visited)?
            };
            if fill_candidates.is_empty() {
                continue;
            }

            let mut tuple = read_chain_node(
                chain,
                metadata.graph_degree_r,
                scan_state::metadata_binary_word_count(metadata),
                scan_state::metadata_search_code_len(metadata),
                target_tid,
            )?;
            let original_tuple = tuple.clone();
            for candidate_tid in fill_candidates {
                insert::insert_backlink_if_free(&mut tuple, candidate_tid);
            }
            if tuple != original_tuple {
                write_chain_node(
                    chain,
                    metadata.graph_degree_r,
                    scan_state::metadata_binary_word_count(metadata),
                    scan_state::metadata_search_code_len(metadata),
                    target_tid,
                    &tuple,
                )?;
            }
        }
        Ok(())
    })();

    unsafe { pg_sys::ExecDropSingleTupleTableSlot(slot) };
    repair_result
}

struct VacuumFillPlanner<'a> {
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attnum: i32,
    metadata: &'a VamanaMetadataPage,
    chain: &'a DataPageChain,
    dead_set: &'a HashSet<ItemPointer>,
}

#[derive(Debug, Clone, PartialEq)]
enum PreparedPrefilter {
    BinarySidecar {
        rotated_query: Vec<f32>,
        query_words: Vec<u64>,
    },
    GroupedPq {
        rotated_query: Vec<f32>,
        flat_codebooks: Vec<f32>,
        query_lut: Vec<f32>,
        group_count: usize,
    },
}

impl PreparedPrefilter {
    fn score(&self, tuple: &VamanaNodeTuple) -> f32 {
        match self {
            Self::BinarySidecar { query_words, .. } => {
                hamming_xor_popcount(query_words, &tuple.binary_words) as f32
            }
            Self::GroupedPq {
                query_lut,
                group_count,
                ..
            } => -grouped_pq_score_f32(query_lut, *group_count, &tuple.search_code),
        }
    }

    fn load_into_scan_opaque(self, opaque: &mut DiskannScanOpaque) {
        match self {
            Self::BinarySidecar {
                rotated_query,
                query_words,
            } => {
                opaque.query_rotated = rotated_query;
                opaque.query_binary_words = query_words;
            }
            Self::GroupedPq {
                rotated_query,
                flat_codebooks,
                query_lut,
                ..
            } => {
                opaque.query_rotated = rotated_query;
                opaque.flat_codebooks = flat_codebooks;
                opaque.query_lut = query_lut;
            }
        }
    }
}

fn prepare_prefilter(
    chain: &DataPageChain,
    metadata: &VamanaMetadataPage,
    raw_query: &[f32],
    prefilter_kind: options::PrefilterKind,
    context: &str,
) -> Result<PreparedPrefilter, String> {
    let has_binary_sidecar = metadata.payload_flags & PAYLOAD_FLAG_BINARY_SIDECAR != 0;
    let use_binary_sidecar = match prefilter_kind {
        options::PrefilterKind::Auto => has_binary_sidecar,
        options::PrefilterKind::BinarySidecar => {
            if !has_binary_sidecar {
                return Err(format!(
                    "ec_diskann.prefilter_kind=binary_sidecar requested but {context} has no binary sidecar"
                ));
            }
            true
        }
        options::PrefilterKind::GroupedPq => false,
    };

    let dimensions = metadata.dimensions as usize;
    let rotated_query = encode_query_srht(raw_query, dimensions, metadata.seed);
    if use_binary_sidecar {
        return Ok(PreparedPrefilter::BinarySidecar {
            query_words: pack_query_sign_bits(&rotated_query, dimensions),
            rotated_query,
        });
    }

    let group_count = usize::from(metadata.search_subvector_count);
    let group_size = usize::from(metadata.search_subvector_dim);
    if group_count == 0 || group_size == 0 {
        return Err(format!(
            "ec_diskann {context} requires grouped-PQ metadata: group_count={}, group_size={}",
            group_count, group_size
        ));
    }
    if rotated_query.len() != group_count * group_size {
        return Err(format!(
            "ec_diskann {context} rotated query length {} does not match group_count {group_count} * group_size {group_size}",
            rotated_query.len()
        ));
    }
    let flat_codebooks = read_grouped_codebook_chain(
        chain,
        metadata.grouped_codebook_head,
        group_count,
        GROUPED_PQ_CENTROIDS * group_size,
    )?;
    let query_lut = build_grouped_pq_lut_f32(&rotated_query, &flat_codebooks, group_size);
    Ok(PreparedPrefilter::GroupedPq {
        rotated_query,
        flat_codebooks,
        query_lut,
        group_count,
    })
}

unsafe fn plan_vacuum_fill_candidates_for_target(
    planner: &VacuumFillPlanner<'_>,
    target_tid: ItemPointer,
    visited: &mut VisitedState,
) -> Result<Vec<ItemPointer>, String> {
    let binary_word_count = scan_state::metadata_binary_word_count(planner.metadata);
    let search_code_len = scan_state::metadata_search_code_len(planner.metadata);
    let target_tuple = read_chain_node(
        planner.chain,
        planner.metadata.graph_degree_r,
        binary_word_count,
        search_code_len,
        target_tid,
    )?;
    if !target_tuple.is_live() || target_tuple.primary_heaptid == ItemPointer::INVALID {
        return Ok(Vec::new());
    }

    let free_slots = target_tuple.neighbors.len() - usize::from(target_tuple.neighbor_count);
    if free_slots == 0 {
        return Ok(Vec::new());
    }

    let target_source_vector = unsafe {
        fetch_heap_source_vector(
            planner.heap_relation,
            planner.snapshot,
            planner.slot,
            planner.source_attnum,
            target_tuple.primary_heaptid,
            "vacuum repair target source vector",
        )?
    };
    let existing_neighbor_tids = target_tuple
        .neighbors
        .iter()
        .take(target_tuple.neighbor_count as usize)
        .copied()
        .filter(|tid| *tid != ItemPointer::INVALID && !planner.dead_set.contains(tid))
        .collect::<Vec<_>>();
    let mut existing_neighbor_set = existing_neighbor_tids
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let mut planning_candidates = Vec::with_capacity(existing_neighbor_tids.len());
    for neighbor_tid in &existing_neighbor_tids {
        let neighbor_tuple = read_chain_node(
            planner.chain,
            planner.metadata.graph_degree_r,
            binary_word_count,
            search_code_len,
            *neighbor_tid,
        )?;
        if !neighbor_tuple.is_live() || neighbor_tuple.primary_heaptid == ItemPointer::INVALID {
            continue;
        }
        let neighbor_source_vector = unsafe {
            fetch_heap_source_vector(
                planner.heap_relation,
                planner.snapshot,
                planner.slot,
                planner.source_attnum,
                neighbor_tuple.primary_heaptid,
                "vacuum repair neighbor source vector",
            )?
        };
        planning_candidates.push(insert::ForwardNeighborCandidate {
            tid: *neighbor_tid,
            source_vector: neighbor_source_vector,
        });
    }

    let build_list_size = usize::from(planner.metadata.build_list_size_l);
    if build_list_size == 0 {
        return Err("ec_diskann vacuum repair requires build_list_size_l > 0".into());
    }
    let repair_scan_budget =
        vacuum_repair_scan_budget(build_list_size, planner.metadata.graph_degree_r as usize);
    let prefilter = prepare_prefilter(
        planner.chain,
        planner.metadata,
        &target_source_vector,
        options::current_prefilter_kind(),
        "vacuum repair",
    )?;

    let frontier_candidates = {
        let reader = PersistedGraphReader::new(
            planner.chain,
            planner.metadata.graph_degree_r,
            binary_word_count,
            search_code_len,
        );
        let entry_point = scan::resolve_entry_point(&reader, planner.metadata.entry_point)?;
        let Some(entry_point) = entry_point else {
            return Ok(Vec::new());
        };
        scan::greedy_descent_with(
            &reader,
            visited,
            entry_point,
            repair_scan_budget,
            &|tuple: &VamanaNodeTuple| prefilter.score(tuple),
        )?
    };

    existing_neighbor_set.insert(target_tid);
    for candidate in frontier_candidates {
        maybe_check_for_interrupts();

        if planner.dead_set.contains(&candidate.tid) || !existing_neighbor_set.insert(candidate.tid)
        {
            continue;
        }

        let candidate_tuple = read_chain_node(
            planner.chain,
            planner.metadata.graph_degree_r,
            binary_word_count,
            search_code_len,
            candidate.tid,
        )?;
        if !candidate_tuple.is_live() || candidate_tuple.primary_heaptid == ItemPointer::INVALID {
            continue;
        }
        let candidate_source_vector = unsafe {
            fetch_heap_source_vector(
                planner.heap_relation,
                planner.snapshot,
                planner.slot,
                planner.source_attnum,
                candidate_tuple.primary_heaptid,
                "vacuum repair candidate source vector",
            )?
        };
        planning_candidates.push(insert::ForwardNeighborCandidate {
            tid: candidate.tid,
            source_vector: candidate_source_vector,
        });
    }

    if planning_candidates.len() <= existing_neighbor_tids.len() {
        return Ok(Vec::new());
    }

    let selected = insert::select_insert_forward_neighbors(
        &target_source_vector,
        &planning_candidates,
        planner.metadata.alpha,
        planner.metadata.graph_degree_r as usize,
    )?;
    let prior_neighbor_set = existing_neighbor_tids
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    Ok(selected
        .into_iter()
        .filter(|tid| !prior_neighbor_set.contains(tid))
        .take(free_slots)
        .collect())
}

fn vacuum_repair_scan_budget(build_list_size: usize, graph_degree_r: usize) -> usize {
    build_list_size.min(graph_degree_r.max(1))
}

fn count_live_node_tuples(index_relation: pg_sys::Relation) -> Result<usize, String> {
    let (metadata, chain) = unsafe { scan_state::materialize_chain_from_index(index_relation)? };
    count_live_tuples_in_chain(
        &chain,
        metadata.graph_degree_r,
        scan_state::metadata_binary_word_count(&metadata),
        scan_state::metadata_search_code_len(&metadata),
    )
}

fn count_live_tuples_in_chain(
    chain: &DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
) -> Result<usize, String> {
    let reader =
        PersistedGraphReader::new(chain, graph_degree_r, binary_word_count, search_code_len);
    let live_count = reader
        .iter_live_tids()
        .try_fold(0usize, |count, item| item.map(|_| count + 1))?;
    Ok(live_count)
}

fn chain_entry_point_needs_medoid_refresh(
    chain: &DataPageChain,
    metadata: &VamanaMetadataPage,
) -> Result<bool, String> {
    if metadata.entry_point == ItemPointer::INVALID {
        return Ok(false);
    }
    let tuple = read_chain_node(
        chain,
        metadata.graph_degree_r,
        scan_state::metadata_binary_word_count(metadata),
        scan_state::metadata_search_code_len(metadata),
        metadata.entry_point,
    )?;
    Ok(tuple.deleted || vacuum::is_fully_dead(&tuple))
}

fn collect_node_tids(
    chain: &DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
) -> Result<Vec<ItemPointer>, String> {
    let reader =
        PersistedGraphReader::new(chain, graph_degree_r, binary_word_count, search_code_len);
    reader.iter_node_tids().collect()
}

fn read_chain_node(
    chain: &DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
    tid: ItemPointer,
) -> Result<VamanaNodeTuple, String> {
    let reader =
        PersistedGraphReader::new(chain, graph_degree_r, binary_word_count, search_code_len);
    reader.read_node(tid)
}

fn write_chain_node(
    chain: &mut DataPageChain,
    graph_degree_r: u16,
    binary_word_count: usize,
    search_code_len: usize,
    tid: ItemPointer,
    tuple: &VamanaNodeTuple,
) -> Result<(), String> {
    let encoded = tuple.encode(graph_degree_r, binary_word_count, search_code_len)?;
    let page = chain.get_page_mut(tid.block_number).ok_or_else(|| {
        format!(
            "ec_diskann vacuum rewrite could not find page {} for ({},{})",
            tid.block_number, tid.block_number, tid.offset_number
        )
    })?;
    page.update_raw_tuple(tid, encoded)
}

fn collect_tuple_rewrites(
    original_chain: &DataPageChain,
    mutated_chain: &DataPageChain,
) -> Result<Vec<TupleRewrite>, String> {
    if original_chain.pages().len() != mutated_chain.pages().len() {
        return Err(format!(
            "ec_diskann vacuum rewrite page-count mismatch: original {}, mutated {}",
            original_chain.pages().len(),
            mutated_chain.pages().len()
        ));
    }

    let mut rewrites = Vec::new();
    for (original_page, mutated_page) in original_chain.pages().iter().zip(mutated_chain.pages()) {
        if original_page.block_number() != mutated_page.block_number() {
            return Err(format!(
                "ec_diskann vacuum rewrite block mismatch: original {}, mutated {}",
                original_page.block_number(),
                mutated_page.block_number()
            ));
        }
        if original_page.tuple_count() != mutated_page.tuple_count() {
            return Err(format!(
                "ec_diskann vacuum rewrite tuple-count mismatch on block {}: original {}, mutated {}",
                original_page.block_number(),
                original_page.tuple_count(),
                mutated_page.tuple_count()
            ));
        }

        for offset in 1..=original_page.tuple_count() {
            let tid = ItemPointer {
                block_number: original_page.block_number(),
                offset_number: offset as u16,
            };
            let expected_raw = original_page.raw_tuple(tid)?.to_vec();
            let replacement_raw = mutated_page.raw_tuple(tid)?.to_vec();
            if expected_raw != replacement_raw {
                rewrites.push(TupleRewrite {
                    tid,
                    expected_raw,
                    replacement_raw,
                });
            }
        }
    }
    Ok(rewrites)
}

unsafe fn apply_tuple_rewrites(
    index_relation: pg_sys::Relation,
    rewrites: &[TupleRewrite],
) -> Result<VacuumRewriteApplyOutcome, String> {
    if rewrites.is_empty() {
        return Ok(VacuumRewriteApplyOutcome::Applied);
    }

    unsafe { maybe_apply_vacuum_rewrite_test_injection(index_relation)? };
    let mut cursor = 0usize;
    while cursor < rewrites.len() {
        let block_number = rewrites[cursor].tid.block_number;
        let block_start = cursor;
        while cursor < rewrites.len() && rewrites[cursor].tid.block_number == block_number {
            cursor += 1;
        }
        let block_rewrites = &rewrites[block_start..cursor];
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            return Err(format!(
                "ec_diskann vacuum rewrite could not open data block {block_number}"
            ));
        }
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
        let page_result = (|| -> Result<VacuumRewriteApplyOutcome, String> {
            let page = unsafe { pg_sys::BufferGetPage(buffer) };
            let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
            for rewrite in block_rewrites {
                let (tuple_ptr, tuple_len) =
                    unsafe { vacuum_page_tuple_location(page, page_size, rewrite.tid)? };
                let current_raw =
                    unsafe { slice::from_raw_parts(tuple_ptr.cast_const(), tuple_len) };
                if current_raw != rewrite.expected_raw.as_slice() {
                    return Ok(VacuumRewriteApplyOutcome::RetryReplan);
                }
            }

            let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
            let writable_page =
                unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
            for rewrite in block_rewrites {
                let (tuple_ptr, tuple_len) =
                    unsafe { vacuum_page_tuple_location(writable_page, page_size, rewrite.tid)? };
                if tuple_len != rewrite.replacement_raw.len() {
                    return Err(format!(
                        "ec_diskann vacuum rewrite length mismatch at ({},{}): got {}, expected {}",
                        rewrite.tid.block_number,
                        rewrite.tid.offset_number,
                        rewrite.replacement_raw.len(),
                        tuple_len
                    ));
                }
                unsafe {
                    ptr::copy_nonoverlapping(
                        rewrite.replacement_raw.as_ptr(),
                        tuple_ptr,
                        rewrite.replacement_raw.len(),
                    );
                }
            }
            unsafe { wal_txn.finish() };
            Ok(VacuumRewriteApplyOutcome::Applied)
        })();
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        match page_result? {
            VacuumRewriteApplyOutcome::Applied => {}
            VacuumRewriteApplyOutcome::RetryReplan => {
                return Ok(VacuumRewriteApplyOutcome::RetryReplan);
            }
        }
    }
    Ok(VacuumRewriteApplyOutcome::Applied)
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn write_raw_tuple_bytes(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
    replacement_raw: &[u8],
) -> Result<(), String> {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err(format!(
            "ec_diskann vacuum test rewrite could not open data block {}",
            tid.block_number
        ));
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let page_result = (|| -> Result<(), String> {
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
        let writable_page =
            unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
        let (tuple_ptr, tuple_len) =
            unsafe { vacuum_page_tuple_location(writable_page, page_size, tid)? };
        if tuple_len != replacement_raw.len() {
            return Err(format!(
                "ec_diskann vacuum test rewrite length mismatch at ({},{}): got {}, expected {}",
                tid.block_number,
                tid.offset_number,
                replacement_raw.len(),
                tuple_len
            ));
        }
        unsafe {
            ptr::copy_nonoverlapping(replacement_raw.as_ptr(), tuple_ptr, replacement_raw.len())
        };
        unsafe { wal_txn.finish() };
        Ok(())
    })();
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page_result
}

unsafe fn callback_marks_heap_tid_dead(
    callback: BulkDeleteCallback,
    callback_state: *mut c_void,
    heap_tid: ItemPointer,
) -> bool {
    let mut raw_tid = pg_sys::ItemPointerData::default();
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut raw_tid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
        callback(&mut raw_tid, callback_state)
    }
}

unsafe fn vacuum_page_tuple_location(
    page: pg_sys::Page,
    page_size: usize,
    tid: ItemPointer,
) -> Result<(*mut u8, usize), String> {
    let max_offset = unsafe { pg_sys::PageGetMaxOffsetNumber(page) };
    if tid.offset_number == pg_sys::InvalidOffsetNumber || tid.offset_number > max_offset {
        return Err(format!(
            "ec_diskann vacuum target ({},{}) has invalid offset {} (max {})",
            tid.block_number, tid.offset_number, tid.offset_number, max_offset
        ));
    }

    let item_id = unsafe { pg_sys::PageGetItemId(page, tid.offset_number) };
    if item_id.is_null() {
        return Err(format!(
            "ec_diskann vacuum target ({},{}) returned a null item id",
            tid.block_number, tid.offset_number
        ));
    }
    let item_id_ref = unsafe { &*item_id };
    if item_id_ref.lp_flags() == 0 {
        return Err(format!(
            "ec_diskann vacuum target ({},{}) points at an unused slot",
            tid.block_number, tid.offset_number
        ));
    }

    let tuple_offset = item_id_ref.lp_off() as usize;
    let tuple_len = item_id_ref.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        return Err(format!(
            "ec_diskann vacuum target ({},{}) has invalid tuple bounds",
            tid.block_number, tid.offset_number
        ));
    }

    let tuple_ptr = unsafe { (page as *mut u8).add(tuple_offset) };
    Ok((tuple_ptr, tuple_len))
}

fn expand_scan_results_with_bound_heap_tids(
    chain: &DataPageChain,
    node_results: &[scan::ScanResult],
    top_k: usize,
) -> Result<Vec<scan::ScanResult>, String> {
    let mut expanded = Vec::with_capacity(top_k.min(node_results.len()));
    for result in node_results {
        let bound_heap_tids =
            insert::bound_heap_tids_for_owner(chain, result.tid, result.primary_heaptid)?;
        for heap_tid in bound_heap_tids {
            expanded.push(scan::ScanResult {
                tid: result.tid,
                primary_heaptid: heap_tid,
                distance: result.distance,
            });
            if expanded.len() >= top_k {
                return Ok(expanded);
            }
        }
    }
    Ok(expanded)
}

fn sql_scan_result_cap(reloption_top_k: usize, rerank_budget: usize) -> usize {
    // `LIMIT` is not visible to `amrescan`, so the SQL scan path must
    // materialize the full rerank window and let the executor truncate.
    // The reloption `top_k` remains a pure scan-shell knob rather than a
    // hard SQL result cap.
    let _ = reloption_top_k;
    rerank_budget
}

#[cfg(feature = "pg18")]
unsafe fn prefetch_heap_rerank_blocks(heap_relation: pg_sys::Relation, heap_tids: &[ItemPointer]) {
    if heap_tids.is_empty() {
        return;
    }
    let block_numbers = heap_tids.iter().map(|tid| tid.block_number).collect();
    let mut state = crate::am::stream::BlockSequencePrefetchState::new(block_numbers);
    let stream = unsafe {
        pg_sys::read_stream_begin_relation(
            pg_sys::READ_STREAM_DEFAULT as i32,
            ptr::null_mut(),
            heap_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            Some(crate::am::stream::block_sequence_prefetch_cb),
            (&mut state as *mut crate::am::stream::BlockSequencePrefetchState).cast(),
            std::mem::size_of::<pg_sys::BlockNumber>(),
        )
    };
    loop {
        let mut per_buffer_data = ptr::null_mut();
        let buffer = unsafe { pg_sys::read_stream_next_buffer(stream, &mut per_buffer_data) };
        if buffer == pg_sys::InvalidBuffer as pg_sys::Buffer {
            break;
        }
        unsafe { pg_sys::ReleaseBuffer(buffer) };
    }
    unsafe { pg_sys::read_stream_end(stream) };
}

#[cfg(not(feature = "pg18"))]
unsafe fn prefetch_heap_rerank_blocks(heap_relation: pg_sys::Relation, heap_tids: &[ItemPointer]) {
    for heap_tid in heap_tids {
        unsafe {
            pg_sys::PrefetchBuffer(
                heap_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                heap_tid.block_number,
            )
        };
    }
}

unsafe fn exact_heap_rerank_distance(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attnum: i32,
    raw_query: &[f32],
    heap_tid: ItemPointer,
) -> Result<f32, String> {
    unsafe {
        with_heap_source_vector(
            heap_relation,
            snapshot,
            slot,
            source_attnum,
            heap_tid,
            "heap rerank source vector",
            |source_vector| {
                if source_vector.len() != raw_query.len() {
                    return Err(format!(
                        "ec_diskann heap rerank dimension mismatch: query dim {}, heap dim {}",
                        raw_query.len(),
                        source_vector.len()
                    ));
                }
                Ok(-ambuild::source_inner_product(raw_query, source_vector))
            },
        )
    }
}

unsafe fn with_heap_source_vector<T>(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attnum: i32,
    heap_tid: ItemPointer,
    context: &str,
    f: impl FnOnce(&[f32]) -> Result<T, String>,
) -> Result<T, String> {
    unsafe { scan_state::fetch_heap_row_version(heap_relation, heap_tid, snapshot, slot)? };
    let datum = unsafe { scan_state::required_slot_datum(slot, source_attnum, context)? };
    let result = unsafe { ambuild::with_ecvector_datum_slice(datum, f) };
    unsafe { pg_sys::ExecClearTuple(slot) };
    result
}

unsafe fn fetch_heap_source_vector(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attnum: i32,
    heap_tid: ItemPointer,
    context: &str,
) -> Result<Vec<f32>, String> {
    unsafe {
        with_heap_source_vector(
            heap_relation,
            snapshot,
            slot,
            source_attnum,
            heap_tid,
            context,
            |source_vector| Ok(source_vector.to_vec()),
        )
    }
}

unsafe extern "C-unwind" fn ec_diskann_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| true) }
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn ec_diskann_handler(
    _fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| pg_sys::Datum::from(build_ec_diskann_routine().into_pg()))
    }
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_ec_diskann_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}

#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
mod tests {
    use super::{insert, scan_state, PersistedGraphReader};
    use crate::am::ec_diskann::page::VamanaMetadataPage;
    use crate::storage::page::{DataPageChain, ItemPointer};
    use pgrx::{pg_sys, pg_test, Spi};
    use std::{
        collections::HashSet,
        ffi::c_void,
        ptr,
        sync::{Mutex, OnceLock},
    };

    fn index_oid(index_name: &str) -> pg_sys::Oid {
        Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
            .expect("SPI query should succeed")
            .expect("index oid should exist")
    }

    fn parse_ctid(ctid: &str) -> ItemPointer {
        let inner = ctid
            .trim()
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .expect("ctid should have (block,offset) format");
        let (block_number, offset_number) = inner
            .split_once(',')
            .expect("ctid should contain a comma separator");
        ItemPointer {
            block_number: block_number
                .trim()
                .parse()
                .expect("ctid block number should parse"),
            offset_number: offset_number
                .trim()
                .parse()
                .expect("ctid offset number should parse"),
        }
    }

    fn heap_tid_for_row(table_name: &str, id: i64) -> ItemPointer {
        let ctid = Spi::get_one::<String>(&format!(
            "SELECT ctid::text FROM {table_name} WHERE id = {id}"
        ))
        .expect("SPI query should succeed")
        .expect("table row should exist");
        parse_ctid(&ctid)
    }

    fn row_id_for_heap_tid(table_name: &str, heap_tid: ItemPointer) -> i64 {
        Spi::get_one::<i64>(&format!(
            "SELECT id FROM {table_name} WHERE ctid = '({},{})'::tid",
            heap_tid.block_number, heap_tid.offset_number
        ))
        .expect("SPI query should succeed")
        .expect("table row should exist")
    }

    fn index_metadata(index_name: &str) -> VamanaMetadataPage {
        let index_relation = unsafe {
            pg_sys::index_open(
                index_oid(index_name),
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )
        };
        let (metadata, _) = unsafe { scan_state::materialize_chain_from_index(index_relation) }
            .expect("materialize_chain_from_index should succeed");
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        metadata
    }

    fn index_materialized_chain(index_name: &str) -> (VamanaMetadataPage, DataPageChain) {
        let index_relation = unsafe {
            pg_sys::index_open(
                index_oid(index_name),
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )
        };
        let materialized = unsafe { scan_state::materialize_chain_from_index(index_relation) }
            .expect("materialize_chain_from_index should succeed");
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        materialized
    }

    #[pg_test]
    fn test_ec_diskann_session_list_size_override_changes_scan_width() {
        Spi::run(
            "CREATE TABLE ec_diskann_session_list_size_override (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_session_list_size_override VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.9, 0.1, 0.0, 0.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.8, 0.2, 0.0, 0.0], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.7, 0.3, 0.0, 0.0], 4, 42))",
        )
        .expect("fixture rows should insert");
        Spi::run(
            "CREATE INDEX ec_diskann_session_list_size_override_idx ON ec_diskann_session_list_size_override USING ec_diskann \
             (embedding ecvector_diskann_ip_ops) WITH (list_size = 111)",
        )
        .expect("index creation should succeed");

        let index_relation = unsafe {
            pg_sys::index_open(
                index_oid("ec_diskann_session_list_size_override_idx"),
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )
        };

        let relation_options = unsafe { super::options::relation_options(index_relation) };
        let (metadata, chain) = unsafe { scan_state::materialize_chain_from_index(index_relation) }
            .expect("materialize_chain_from_index should succeed");
        let relation_opaque =
            scan_state::DiskannScanOpaque::new(metadata, chain, relation_options.clone())
                .expect("relation scan state should build");
        assert_eq!(
            relation_opaque.list_size, 111,
            "without a session override, DiskANN scan state should use the reloption width",
        );

        Spi::run("SET ec_diskann.list_size = 7").expect("session override should succeed");
        let (metadata, chain) = unsafe { scan_state::materialize_chain_from_index(index_relation) }
            .expect("materialize_chain_from_index should succeed");
        let session_opaque = scan_state::DiskannScanOpaque::new(metadata, chain, relation_options)
            .expect("session scan state should build");
        assert_eq!(
            session_opaque.list_size, 7,
            "session ec_diskann.list_size should override the reloption during scan setup",
        );
        Spi::run("RESET ec_diskann.list_size").expect("reset should succeed");

        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }

    #[pg_test]
    fn test_ec_diskann_prefilter_kind_override_switches_prefilter() {
        Spi::run(
            "CREATE TABLE ec_diskann_prefilter_kind_override (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_prefilter_kind_override
             SELECT fixture.id,
                    encode_to_ecvector(
                        ARRAY(
                            SELECT CASE WHEN dim = fixture.hot_dim THEN 1.0::real ELSE 0.0::real END
                            FROM generate_series(1, 1536) dim
                        ),
                        4,
                        42
                    )
             FROM (VALUES (1, 1), (2, 2), (3, 3), (4, 4)) AS fixture(id, hot_dim)",
        )
        .expect("fixture rows should insert");
        Spi::run(
            "CREATE INDEX ec_diskann_prefilter_kind_override_idx ON ec_diskann_prefilter_kind_override USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");

        let (metadata, chain) = index_materialized_chain("ec_diskann_prefilter_kind_override_idx");
        let mut query = vec![0.0_f32; 1536];
        query[0] = 1.0;

        Spi::run("RESET ec_diskann.prefilter_kind").expect("reset should succeed");
        let auto_prefilter = super::prepare_prefilter(
            &chain,
            &metadata,
            &query,
            super::options::current_prefilter_kind(),
            "test",
        )
        .expect("auto prefilter should prepare");
        assert!(
            matches!(
                auto_prefilter,
                super::PreparedPrefilter::BinarySidecar { .. }
            ),
            "auto should use the persisted binary sidecar when present",
        );

        Spi::run("SET ec_diskann.prefilter_kind = 'grouped_pq'")
            .expect("grouped_pq override should succeed");
        let grouped_prefilter = super::prepare_prefilter(
            &chain,
            &metadata,
            &query,
            super::options::current_prefilter_kind(),
            "test",
        )
        .expect("grouped-PQ prefilter should prepare");
        assert!(
            matches!(
                grouped_prefilter,
                super::PreparedPrefilter::GroupedPq { .. }
            ),
            "grouped_pq should force the legacy grouped-PQ prefilter",
        );

        Spi::run("SET ec_diskann.prefilter_kind = 'binary_sidecar'")
            .expect("binary_sidecar override should succeed");
        let binary_prefilter = super::prepare_prefilter(
            &chain,
            &metadata,
            &query,
            super::options::current_prefilter_kind(),
            "test",
        )
        .expect("binary sidecar prefilter should prepare");
        assert!(
            matches!(
                binary_prefilter,
                super::PreparedPrefilter::BinarySidecar { .. }
            ),
            "binary_sidecar should force the sidecar prefilter when persisted",
        );
        Spi::run("RESET ec_diskann.prefilter_kind").expect("reset should succeed");
    }

    fn explain_text(sql: &str) -> String {
        Spi::connect(|client| {
            let rows = client
                .select(sql, None, &[])
                .expect("EXPLAIN should succeed")
                .first();
            let mut lines = Vec::new();
            for row in rows {
                lines.push(
                    row.get::<String>(1)
                        .expect("plan row should decode")
                        .expect("plan row should not be NULL"),
                );
            }
            lines.join("\n")
        })
    }

    fn explain_ordered_diskann_ids(table_name: &str, query_array: &str, limit: usize) -> String {
        explain_text(&format!(
            "EXPLAIN (COSTS OFF) SELECT id FROM {table_name} \
             ORDER BY embedding <#> {query_array} LIMIT {limit}"
        ))
    }

    fn explain_plan_uses_index(plan: &str) -> bool {
        plan.contains("Index Scan") || plan.contains("Index Only Scan")
    }

    #[test]
    fn sql_scan_result_cap_defaults_to_rerank_budget() {
        assert_eq!(
            super::sql_scan_result_cap(10, 64),
            64,
            "the SQL path must materialize the full rerank window when LIMIT is not visible",
        );
        assert_eq!(
            super::sql_scan_result_cap(128, 64),
            64,
            "reloption top_k must not exceed the rerank window in the SQL scan path",
        );
    }

    #[test]
    fn vacuum_repair_scan_budget_caps_at_graph_degree() {
        assert_eq!(super::vacuum_repair_scan_budget(100, 32), 32);
        assert_eq!(super::vacuum_repair_scan_budget(24, 32), 24);
        assert_eq!(super::vacuum_repair_scan_budget(100, 0), 1);
    }

    fn diskann_large_query_array() -> String {
        let mut out = String::from("ARRAY[");
        for i in 0..64 {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!("{:.6}", i as f32 * 0.05 - 1.5));
        }
        out.push_str("]::real[]");
        out
    }

    #[derive(Debug, Clone)]
    struct VacuumRefillFixture {
        prefill_metadata: VamanaMetadataPage,
        prefill_chain: DataPageChain,
        target_tid: ItemPointer,
        target_neighbors_before: Vec<ItemPointer>,
        replacement_tid: ItemPointer,
        deleted_tid: ItemPointer,
    }

    fn vacuum_retry_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("vacuum retry test lock should not be poisoned")
    }

    struct ScopedVacuumRetryState {
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl ScopedVacuumRetryState {
        fn new() -> Self {
            let lock = vacuum_retry_test_lock();
            super::clear_vacuum_rewrite_test_state();
            Self { _lock: lock }
        }
    }

    impl Drop for ScopedVacuumRetryState {
        fn drop(&mut self) {
            super::clear_vacuum_rewrite_test_state();
        }
    }

    fn find_vacuum_refill_fixture(index_name: &str) -> VacuumRefillFixture {
        let (prefill_metadata, prefill_chain) = index_materialized_chain(index_name);
        let prefill_reader = PersistedGraphReader::new(
            &prefill_chain,
            prefill_metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&prefill_metadata),
            scan_state::metadata_search_code_len(&prefill_metadata),
        );
        let live_node_tids = prefill_reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node iteration should succeed");
        let graph_degree = prefill_metadata.graph_degree_r as usize;
        let binary_word_count = scan_state::metadata_binary_word_count(&prefill_metadata);
        let search_code_len = scan_state::metadata_search_code_len(&prefill_metadata);
        assert_eq!(
            live_node_tids.len(),
            graph_degree + 2,
            "fixture should expose exactly two more live nodes than the graph degree",
        );

        let search_index_relation = unsafe {
            pg_sys::index_open(
                index_oid(index_name),
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )
        };
        let heap_oid = unsafe { pg_sys::IndexGetRelation((*search_index_relation).rd_id, false) };
        assert_ne!(heap_oid, pg_sys::InvalidOid, "heap relation should resolve");
        let heap_relation =
            unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let source_attnum = unsafe { super::indexed_ecvector_attnum(search_index_relation) }
            .expect("indexed source attnum should resolve");
        let slot = unsafe { scan_state::allocate_heap_slot(heap_relation) }
            .expect("heap slot allocation should succeed");
        let snapshot = std::ptr::addr_of_mut!(pg_sys::SnapshotSelfData);
        let mut visited = super::VisitedState::new();

        let fixture_plan = live_node_tids.iter().copied().find_map(|target_tid| {
            let candidate_tids = live_node_tids
                .iter()
                .copied()
                .filter(|tid| *tid != target_tid)
                .collect::<Vec<_>>();
            candidate_tids.iter().copied().find_map(|replacement_tid| {
                let target_neighbors_before = candidate_tids
                    .iter()
                    .copied()
                    .filter(|tid| *tid != replacement_tid)
                    .collect::<Vec<_>>();
                target_neighbors_before
                    .iter()
                    .copied()
                    .find_map(|deleted_tid| {
                        let deleted_heap_tid = prefill_reader
                            .read_node(deleted_tid)
                            .expect("deleted tuple should decode during fixture search")
                            .primary_heaptid;
                        let mut working_chain = prefill_chain.clone();
                        let mut target_tuple_before = super::read_chain_node(
                            &working_chain,
                            prefill_metadata.graph_degree_r,
                            binary_word_count,
                            search_code_len,
                            target_tid,
                        )
                        .expect("target tuple should decode during fixture search");
                        for slot in &mut target_tuple_before.neighbors {
                            *slot = ItemPointer::INVALID;
                        }
                        for (slot, neighbor_tid) in
                            target_neighbors_before.iter().copied().enumerate()
                        {
                            target_tuple_before.neighbors[slot] = neighbor_tid;
                        }
                        target_tuple_before.neighbor_count = prefill_metadata.graph_degree_r;
                        super::write_chain_node(
                            &mut working_chain,
                            prefill_metadata.graph_degree_r,
                            binary_word_count,
                            search_code_len,
                            target_tid,
                            &target_tuple_before,
                        )
                        .expect("target rewrite should encode during fixture search");

                        let node_tids = super::collect_node_tids(
                            &working_chain,
                            prefill_metadata.graph_degree_r,
                            binary_word_count,
                            search_code_len,
                        )
                        .expect("node tids should collect during fixture search");
                        let mut finalize_tids = Vec::new();
                        for &tid in &node_tids {
                            let mut tuple = super::read_chain_node(
                                &working_chain,
                                prefill_metadata.graph_degree_r,
                                binary_word_count,
                                search_code_len,
                                tid,
                            )
                            .expect("tuple should decode during pass-1 search");
                            let original_tuple = tuple.clone();
                            insert::vacuum_bound_heap_rows(
                                &mut working_chain,
                                tid,
                                &mut tuple,
                                |heap_tid| heap_tid == deleted_heap_tid,
                            )
                            .expect("pass-1 strip should succeed during fixture search");
                            if tuple != original_tuple {
                                super::write_chain_node(
                                    &mut working_chain,
                                    prefill_metadata.graph_degree_r,
                                    binary_word_count,
                                    search_code_len,
                                    tid,
                                    &tuple,
                                )
                                .expect("pass-1 rewrite should succeed during fixture search");
                            }
                            if super::vacuum::is_fully_dead(&tuple) {
                                finalize_tids.push(tid);
                            }
                        }

                        let dead_set = finalize_tids.iter().copied().collect::<HashSet<_>>();
                        if !dead_set.contains(&deleted_tid) {
                            return None;
                        }
                        for &tid in &node_tids {
                            let mut tuple = super::read_chain_node(
                                &working_chain,
                                prefill_metadata.graph_degree_r,
                                binary_word_count,
                                search_code_len,
                                tid,
                            )
                            .expect("tuple should decode during pass-2 search");
                            if !tuple.is_live() {
                                continue;
                            }
                            if super::vacuum::repair_neighbors(&mut tuple, &dead_set) != 0 {
                                super::write_chain_node(
                                    &mut working_chain,
                                    prefill_metadata.graph_degree_r,
                                    binary_word_count,
                                    search_code_len,
                                    tid,
                                    &tuple,
                                )
                                .expect("pass-2 rewrite should succeed during fixture search");
                            }
                        }

                        let planner = super::VacuumFillPlanner {
                            heap_relation,
                            snapshot,
                            slot,
                            source_attnum,
                            metadata: &prefill_metadata,
                            chain: &working_chain,
                            dead_set: &dead_set,
                        };
                        let fill_candidates = unsafe {
                            super::plan_vacuum_fill_candidates_for_target(
                                &planner,
                                target_tid,
                                &mut visited,
                            )
                        }
                        .expect("fixture search should plan vacuum fill candidates");
                        if fill_candidates.contains(&replacement_tid) {
                            Some(VacuumRefillFixture {
                                prefill_metadata: prefill_metadata.clone(),
                                prefill_chain: prefill_chain.clone(),
                                target_tid,
                                target_neighbors_before: target_neighbors_before.clone(),
                                replacement_tid,
                                deleted_tid,
                            })
                        } else {
                            None
                        }
                    })
            })
        });

        unsafe { pg_sys::ExecDropSingleTupleTableSlot(slot) };
        unsafe { pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        unsafe {
            pg_sys::index_close(
                search_index_relation,
                pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            )
        };

        fixture_plan.expect("fixture search should find a reachable vacuum refill candidate")
    }

    #[derive(Debug, Default)]
    struct DebugVacuumCallbackState {
        dead_tids: HashSet<ItemPointer>,
    }

    unsafe extern "C-unwind" fn debug_vacuum_dead_tid_callback(
        itemptr: pg_sys::ItemPointer,
        state: *mut c_void,
    ) -> bool {
        let state = unsafe { &*(state.cast::<DebugVacuumCallbackState>()) };
        let (block_number, offset_number) =
            pgrx::itemptr::item_pointer_get_both(unsafe { *itemptr });
        state.dead_tids.contains(&ItemPointer {
            block_number,
            offset_number,
        })
    }

    unsafe fn debug_vacuum_stats(index_oid: pg_sys::Oid) -> pg_sys::IndexBulkDeleteResult {
        let index_relation =
            unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let mut info = pgrx::PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
        info.index = index_relation;
        let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;

        let stats = unsafe {
            super::ec_diskann_ambulkdelete(info_ptr, ptr::null_mut(), None, ptr::null_mut())
        };
        let stats = unsafe { super::ec_diskann_amvacuumcleanup(info_ptr, stats) };
        let result = unsafe { *stats };

        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        result
    }

    unsafe fn debug_vacuum_remove_heap_tids(
        index_oid: pg_sys::Oid,
        dead_tids: &[ItemPointer],
    ) -> pg_sys::IndexBulkDeleteResult {
        let index_relation = unsafe {
            pg_sys::index_open(
                index_oid,
                pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
            )
        };
        let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
        let heap_relation = if heap_oid == pg_sys::InvalidOid {
            ptr::null_mut()
        } else {
            unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) }
        };
        let mut info = pgrx::PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
        info.index = index_relation;
        info.heaprel = heap_relation;
        let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;
        let mut callback_state = DebugVacuumCallbackState {
            dead_tids: dead_tids.iter().copied().collect(),
        };

        let stats = unsafe {
            super::ec_diskann_ambulkdelete(
                info_ptr,
                ptr::null_mut(),
                Some(debug_vacuum_dead_tid_callback),
                (&mut callback_state as *mut DebugVacuumCallbackState).cast(),
            )
        };
        let stats = unsafe { super::ec_diskann_amvacuumcleanup(info_ptr, stats) };
        let result = unsafe { *stats };

        unsafe {
            if !heap_relation.is_null() {
                pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE);
            }
            pg_sys::index_close(
                index_relation,
                pg_sys::ShareUpdateExclusiveLock as pg_sys::LOCKMODE,
            );
        }
        result
    }

    #[pg_test]
    fn test_ec_diskann_sql_ordered_index_scan_executes() {
        Spi::run(
            "CREATE TABLE ec_diskann_sql_ordered_exec (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_sql_ordered_exec VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.0, 1.0], 4, 42))",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_sql_ordered_exec_idx ON ec_diskann_sql_ordered_exec USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");

        let plan = explain_ordered_diskann_ids(
            "ec_diskann_sql_ordered_exec",
            "ARRAY[1.0, 0.0, 0.5, -1.0]::real[]",
            2,
        );

        assert!(
            explain_plan_uses_index(&plan),
            "ordered execution test should route through ec_diskann at runtime: {plan}"
        );

        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_sql_ordered_exec \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 2",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(
            ordered_ids.len(),
            2,
            "query should return the requested LIMIT"
        );
        assert_eq!(
            ordered_ids[0], 1,
            "runtime ordered ec_diskann scan should return the nearest vector first"
        );
    }

    #[pg_test]
    fn test_ec_diskann_sql_limit_can_exceed_reloption_top_k() {
        Spi::run(
            "CREATE TABLE ec_diskann_sql_limit_over_top_k (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_sql_limit_over_top_k_idx ON ec_diskann_sql_limit_over_top_k USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        for id in 1..=12_i64 {
            Spi::run(&format!(
                "INSERT INTO ec_diskann_sql_limit_over_top_k VALUES \
                 ({id}, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))"
            ))
            .expect("duplicate insert should succeed");
        }
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");

        let plan = explain_ordered_diskann_ids(
            "ec_diskann_sql_limit_over_top_k",
            "ARRAY[1.0, 0.0, 0.5, -1.0]::real[]",
            12,
        );

        assert!(
            explain_plan_uses_index(&plan),
            "ordered LIMIT-over-top_k test should route through ec_diskann at runtime: {plan}"
        );

        let mut ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_sql_limit_over_top_k \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 12",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        ordered_ids.sort_unstable();
        assert_eq!(
            ordered_ids.len(),
            12,
            "runtime ordered ec_diskann scan should not be capped by the reloption top_k default",
        );
        assert_eq!(ordered_ids, (1..=12_i64).collect::<Vec<_>>());
    }

    #[pg_test]
    fn test_ec_diskann_build_coalesces_duplicate_vectors() {
        Spi::run(
            "CREATE TABLE ec_diskann_duplicate_build (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        for id in 1..=12_i64 {
            Spi::run(&format!(
                "INSERT INTO ec_diskann_duplicate_build VALUES \
                 ({id}, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))"
            ))
            .expect("duplicate insert should succeed");
        }
        Spi::run(
            "CREATE INDEX ec_diskann_duplicate_build_idx ON ec_diskann_duplicate_build USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");

        let (metadata, chain) = index_materialized_chain("ec_diskann_duplicate_build_idx");
        let reader = PersistedGraphReader::new(
            &chain,
            metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&metadata),
            scan_state::metadata_search_code_len(&metadata),
        );
        let node_tids = reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node tid iteration should succeed");
        assert_eq!(
            node_tids.len(),
            1,
            "duplicate build rows should share one DiskANN graph node",
        );

        let node_tid = node_tids[0];
        let node_tuple = reader
            .read_node(node_tid)
            .expect("node tuple should decode");
        assert!(node_tuple.has_overflow_heaptids);
        let bound_heap_tids =
            insert::bound_heap_tids_for_owner(&chain, node_tid, node_tuple.primary_heaptid)
                .expect("bound heap tids should decode");
        assert_eq!(bound_heap_tids.len(), 12);

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");

        let plan = explain_ordered_diskann_ids(
            "ec_diskann_duplicate_build",
            "ARRAY[1.0, 0.0, 0.5, -1.0]::real[]",
            12,
        );
        assert!(
            explain_plan_uses_index(&plan),
            "duplicate-build test should route through ec_diskann at runtime: {plan}"
        );

        let mut ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_duplicate_build \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 12",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        ordered_ids.sort_unstable();
        assert_eq!(ordered_ids, (1..=12_i64).collect::<Vec<_>>());
    }

    #[pg_test]
    fn test_ec_diskann_empty_index_remains_planner_gated() {
        Spi::run("CREATE TABLE ec_diskann_empty_cost (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_empty_cost_idx ON ec_diskann_empty_cost USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("empty-index creation should succeed");

        let plan = explain_ordered_diskann_ids(
            "ec_diskann_empty_cost",
            "ARRAY[1.0, 0.0, 0.5, -1.0]::real[]",
            1,
        );

        assert!(
            !explain_plan_uses_index(&plan),
            "planner must not pick an empty ec_diskann index after Phase 9 activation: {plan}"
        );
    }

    #[pg_test]
    fn test_ec_diskann_planner_prefers_seqscan_for_small_tables() {
        Spi::run(
            "CREATE TABLE ec_diskann_small_seqscan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_small_seqscan \
             SELECT g, encode_to_ecvector(ARRAY[g::real, (g * 0.25)::real, (g * -0.5)::real, 1.0::real], 4, 42) \
             FROM generate_series(1, 50) AS g",
        )
        .expect("insert should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_small_seqscan_idx ON ec_diskann_small_seqscan USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("ANALYZE ec_diskann_small_seqscan").expect("analyze should succeed");

        let plan = explain_ordered_diskann_ids(
            "ec_diskann_small_seqscan",
            "ARRAY[1.0, 0.0, 0.5, -1.0]::real[]",
            1,
        );

        assert!(
            !explain_plan_uses_index(&plan),
            "planner should prefer seqscan on a 50-row table even with ec_diskann Phase 9 activation: {plan}"
        );
    }

    #[pg_test]
    fn test_ec_diskann_planner_chooses_index_scan_for_large_table() {
        Spi::run("CREATE TABLE ec_diskann_large_plan (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_large_plan \
             SELECT g, encode_to_ecvector( \
                 ARRAY( \
                     SELECT ((get_byte( \
                              decode(md5(g::text) \
                                     || md5((g + 999983)::text) \
                                     || md5((g + 1999993)::text) \
                                     || md5((g + 2999999)::text), 'hex'), \
                              i)::real - 128.0) / 128.0)::real \
                     FROM generate_series(0, 63) AS i), \
                 4, 42) \
             FROM generate_series(1, 10000) AS g",
        )
        .expect("10k-row insert should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_large_plan_idx ON ec_diskann_large_plan USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("ANALYZE ec_diskann_large_plan").expect("analyze should succeed");

        let query_array = diskann_large_query_array();
        let plan = explain_ordered_diskann_ids("ec_diskann_large_plan", &query_array, 10);

        assert!(
            plan.contains("Index Scan") && plan.contains("ec_diskann_large_plan_idx"),
            "planner must naturally pick the ec_diskann index on a 10K-row table after Phase 9 activation: {plan}"
        );
    }

    #[pg_test]
    fn test_ec_diskann_empty_index_bootstrap_insert_executes() {
        Spi::run(
            "CREATE TABLE ec_diskann_bootstrap_insert_exec (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_bootstrap_insert_exec_idx ON ec_diskann_bootstrap_insert_exec USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_bootstrap_insert_exec VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("first insert should bootstrap the empty ec_diskann index");
        let metadata = index_metadata("ec_diskann_bootstrap_insert_exec_idx");
        assert_eq!(
            metadata.inserted_since_rebuild, 1,
            "bootstrap insert should initialize inserted_since_rebuild to one",
        );
        assert!(
            !metadata.needs_medoid_refresh,
            "bootstrap insert should not set needs_medoid_refresh",
        );
        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");

        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_bootstrap_insert_exec \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed after bootstrap")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(ordered_ids, vec![1]);
    }

    #[pg_test]
    fn test_ec_diskann_unique_insert_is_scan_reachable() {
        Spi::run(
            "CREATE TABLE ec_diskann_unique_insert_append (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_unique_insert_append_idx ON ec_diskann_unique_insert_append USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_unique_insert_append VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("first insert should bootstrap");

        Spi::run(
            "INSERT INTO ec_diskann_unique_insert_append VALUES
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("second distinct insert should append a live node and backfill free backlinks");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_diskann_unique_insert_append_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let index_relation =
            unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let (metadata, chain) = unsafe { scan_state::materialize_chain_from_index(index_relation) }
            .expect("materialize_chain_from_index should succeed");
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        assert_eq!(
            metadata.inserted_since_rebuild, 2,
            "true new-node inserts should increment inserted_since_rebuild",
        );
        assert!(
            !metadata.needs_medoid_refresh,
            "insert should leave needs_medoid_refresh ownership to maintenance paths",
        );
        let reader = PersistedGraphReader::new(
            &chain,
            metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&metadata),
            scan_state::metadata_search_code_len(&metadata),
        );
        let row1_heap_tid = heap_tid_for_row("ec_diskann_unique_insert_append", 1);
        let row2_heap_tid = heap_tid_for_row("ec_diskann_unique_insert_append", 2);
        let mut row1_tid = ItemPointer::INVALID;
        let mut row2_tid = ItemPointer::INVALID;
        for tid in reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node tid iteration should succeed")
        {
            let tuple = reader.read_node(tid).expect("node decode should succeed");
            if tuple.primary_heaptid == row1_heap_tid {
                row1_tid = tid;
            }
            if tuple.primary_heaptid == row2_heap_tid {
                row2_tid = tid;
            }
        }
        assert_ne!(
            row1_tid,
            ItemPointer::INVALID,
            "seed row should have a node tid"
        );
        assert_ne!(
            row2_tid,
            ItemPointer::INVALID,
            "inserted row should have a node tid"
        );
        let row2_tuple = reader
            .read_node(row2_tid)
            .expect("inserted node should decode");
        let row2_neighbors = row2_tuple
            .neighbors
            .iter()
            .take(row2_tuple.neighbor_count as usize)
            .copied()
            .collect::<Vec<_>>();
        assert!(
            row2_neighbors.contains(&row1_tid),
            "the inserted node should retain the seed node as a forward neighbor in the free-backlink slice: got {:?}",
            row2_neighbors,
        );
        let row1_tuple = reader.read_node(row1_tid).expect("seed node should decode");
        let row1_neighbors = row1_tuple
            .neighbors
            .iter()
            .take(row1_tuple.neighbor_count as usize)
            .copied()
            .collect::<Vec<_>>();
        assert!(
            row1_neighbors.contains(&row2_tid),
            "the seed node should receive a backlink to the inserted node before scan reachability is checked: got {:?}",
            row1_neighbors,
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_diskann_unique_insert_append \
                     ORDER BY embedding <#> ARRAY[0.0, 1.0, 0.25, -0.5]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("EXPLAIN should succeed")
                .map(|row| {
                    row["QUERY PLAN"]
                        .value::<String>()
                        .expect("plan row should decode")
                        .expect("plan row should be non-null")
                })
                .collect::<Vec<_>>();
            rows.join("\n")
        });
        assert!(
            plan.contains("Index Scan using ec_diskann_unique_insert_append_idx"),
            "reachability test should run through ec_diskann at runtime: {plan}"
        );

        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_unique_insert_append \
                     ORDER BY embedding <#> ARRAY[0.0, 1.0, 0.25, -0.5]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(
            ordered_ids,
            vec![2],
            "the inserted row should become reachable through the runtime ec_diskann graph scan",
        );
    }

    #[pg_test]
    fn test_ec_diskann_full_backlink_rewrite_keeps_insert_reachable() {
        Spi::run(
            "CREATE TABLE ec_diskann_full_backlink_rewrite (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_full_backlink_rewrite_idx ON ec_diskann_full_backlink_rewrite USING ec_diskann \
             (embedding ecvector_diskann_ip_ops) WITH (graph_degree = 4, build_list_size = 10)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_full_backlink_rewrite VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("first insert should bootstrap");
        Spi::run(
            "INSERT INTO ec_diskann_full_backlink_rewrite VALUES
             (2, encode_to_ecvector(ARRAY[0.1, -1.0, 0.0, 0.0, 0.0], 4, 42))",
        )
        .expect("second insert should backlink to the seed node");
        Spi::run(
            "INSERT INTO ec_diskann_full_backlink_rewrite VALUES
             (3, encode_to_ecvector(ARRAY[0.1, 0.0, -1.0, 0.0, 0.0], 4, 42))",
        )
        .expect("third insert should backlink to the seed node");
        Spi::run(
            "INSERT INTO ec_diskann_full_backlink_rewrite VALUES
             (4, encode_to_ecvector(ARRAY[0.1, 0.0, 0.0, -1.0, 0.0], 4, 42))",
        )
        .expect("fourth insert should backlink to the seed node");
        Spi::run(
            "INSERT INTO ec_diskann_full_backlink_rewrite VALUES
             (5, encode_to_ecvector(ARRAY[0.1, 0.0, 0.0, 0.0, -1.0], 4, 42))",
        )
        .expect("fifth insert should fill the seed node's backlink slice");

        let row1_heap_tid = heap_tid_for_row("ec_diskann_full_backlink_rewrite", 1);
        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_diskann_full_backlink_rewrite_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let index_relation =
            unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let (prefill_metadata, prefill_chain) =
            unsafe { scan_state::materialize_chain_from_index(index_relation) }
                .expect("prefill materialize_chain_from_index should succeed");
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let prefill_reader = PersistedGraphReader::new(
            &prefill_chain,
            prefill_metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&prefill_metadata),
            scan_state::metadata_search_code_len(&prefill_metadata),
        );
        let row1_tid = prefill_reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("prefill node tid iteration should succeed")
            .into_iter()
            .find(|&tid| {
                prefill_reader
                    .read_node(tid)
                    .map(|tuple| tuple.primary_heaptid == row1_heap_tid)
                    .unwrap_or(false)
            })
            .expect("seed row should have a node tid before rewrite");
        let row1_neighbors_before = prefill_reader
            .read_node(row1_tid)
            .expect("seed node should decode before rewrite")
            .neighbors
            .iter()
            .take(prefill_metadata.graph_degree_r as usize)
            .copied()
            .filter(|tid| *tid != ItemPointer::INVALID)
            .collect::<Vec<_>>();
        assert_eq!(
            row1_neighbors_before.len(),
            prefill_metadata.graph_degree_r as usize,
            "the rewrite test must start from a full backlink slice",
        );

        Spi::run(
            "INSERT INTO ec_diskann_full_backlink_rewrite VALUES
             (6, encode_to_ecvector(ARRAY[1.0, 1.0, 1.0, 1.0, 1.0], 4, 42))",
        )
        .expect("sixth insert should rewrite the full backlink slice");

        let metadata = index_metadata("ec_diskann_full_backlink_rewrite_idx");
        assert_eq!(
            metadata.inserted_since_rebuild, 6,
            "six true inserts should advance inserted_since_rebuild six times",
        );
        assert!(
            !metadata.needs_medoid_refresh,
            "insert-side full-slice rewrite should not set needs_medoid_refresh",
        );

        let index_relation =
            unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let (metadata, chain) = unsafe { scan_state::materialize_chain_from_index(index_relation) }
            .expect("materialize_chain_from_index should succeed");
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let reader = PersistedGraphReader::new(
            &chain,
            metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&metadata),
            scan_state::metadata_search_code_len(&metadata),
        );

        let row6_heap_tid = heap_tid_for_row("ec_diskann_full_backlink_rewrite", 6);
        let mut row6_tid = ItemPointer::INVALID;
        for tid in reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node tid iteration should succeed")
        {
            let tuple = reader.read_node(tid).expect("node decode should succeed");
            if tuple.primary_heaptid == row6_heap_tid {
                row6_tid = tid;
            }
        }
        assert_ne!(
            row6_tid,
            ItemPointer::INVALID,
            "rewritten row should have a node tid"
        );

        let row1_tuple = reader.read_node(row1_tid).expect("seed node should decode");
        let row1_neighbors = row1_tuple
            .neighbors
            .iter()
            .take(row1_tuple.neighbor_count as usize)
            .copied()
            .collect::<Vec<_>>();
        assert!(
            row1_neighbors.contains(&row6_tid),
            "full-slice rewrite should install the new node into the target's backlink slice; before={row1_neighbors_before:?} after={row1_neighbors:?} row6_tid={row6_tid:?}",
        );
        assert_ne!(
            row1_neighbors, row1_neighbors_before,
            "rewriting a full backlink slice should change at least one neighbor slot",
        );

        let row6_tuple = reader
            .read_node(row6_tid)
            .expect("rewritten node should decode");
        let row6_neighbors = row6_tuple
            .neighbors
            .iter()
            .take(row6_tuple.neighbor_count as usize)
            .copied()
            .collect::<Vec<_>>();
        assert!(
            row6_neighbors.contains(&row1_tid),
            "the inserted node should still keep the seed node as its forward neighbor",
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");

        let plan = Spi::connect(|client| {
            client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_diskann_full_backlink_rewrite \
                     ORDER BY embedding <#> ARRAY[1.0, 1.0, 1.0, 1.0, 1.0]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("EXPLAIN should succeed")
                .map(|row| {
                    row["QUERY PLAN"]
                        .value::<String>()
                        .expect("plan row should decode")
                        .expect("plan row should be non-null")
                })
                .collect::<Vec<_>>()
                .join("\n")
        });
        assert!(
            plan.contains("Index Scan using ec_diskann_full_backlink_rewrite_idx"),
            "rewrite reachability test should route through ec_diskann at runtime: {plan}"
        );

        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_full_backlink_rewrite \
                     ORDER BY embedding <#> ARRAY[1.0, 1.0, 1.0, 1.0, 1.0]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(
            ordered_ids,
            vec![6],
            "rewritten backlinks should keep the newest node reachable to runtime scan",
        );
    }

    #[pg_test]
    fn test_ec_diskann_duplicate_after_append_binds_existing_node() {
        Spi::run(
            "CREATE TABLE ec_diskann_duplicate_after_append (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_duplicate_after_append_idx ON ec_diskann_duplicate_after_append USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_duplicate_after_append VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("bootstrap and append inserts should succeed");

        Spi::run(
            "INSERT INTO ec_diskann_duplicate_after_append VALUES
             (3, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("duplicate insert should bind to the appended node");

        let metadata = index_metadata("ec_diskann_duplicate_after_append_idx");
        assert_eq!(
            metadata.inserted_since_rebuild, 2,
            "duplicate inserts must not advance inserted_since_rebuild",
        );
        assert!(
            !metadata.needs_medoid_refresh,
            "duplicate inserts should not set needs_medoid_refresh",
        );

        let row2_heap_tid = heap_tid_for_row("ec_diskann_duplicate_after_append", 2);
        let row3_heap_tid = heap_tid_for_row("ec_diskann_duplicate_after_append", 3);
        let (materialized_metadata, chain) =
            index_materialized_chain("ec_diskann_duplicate_after_append_idx");
        let reader = PersistedGraphReader::new(
            &chain,
            materialized_metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&materialized_metadata),
            scan_state::metadata_search_code_len(&materialized_metadata),
        );
        let duplicate_node_tid = reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node tid iteration should succeed")
            .into_iter()
            .find(|tid| {
                reader
                    .read_node(*tid)
                    .expect("node decode should succeed")
                    .primary_heaptid
                    == row2_heap_tid
            })
            .expect("duplicate target node should exist");
        let duplicate_tuple = reader
            .read_node(duplicate_node_tid)
            .expect("duplicate target node should decode");
        assert!(
            duplicate_tuple.has_overflow_heaptids,
            "duplicate target should advertise an overflow chain after bind",
        );
        assert_eq!(
            insert::bound_heap_tids_for_owner(
                &chain,
                duplicate_node_tid,
                duplicate_tuple.primary_heaptid
            )
            .expect("bound heap tids should decode"),
            vec![row2_heap_tid, row3_heap_tid],
            "duplicate bind should preserve primary-first heap tid order",
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");
        let mut ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_duplicate_after_append \
                     ORDER BY embedding <#> ARRAY[0.0, 1.0, 0.25, -0.5]::real[] \
                     LIMIT 2",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        ordered_ids.sort_unstable();
        assert_eq!(
            ordered_ids,
            vec![2, 3],
            "runtime scan should expand duplicate-bound heap tids for the appended node",
        );
    }

    #[pg_test]
    fn test_ec_diskann_duplicate_insert_binds_first_overflow_tuple() {
        Spi::run(
            "CREATE TABLE ec_diskann_duplicate_insert_boundary (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_duplicate_insert_boundary_idx ON ec_diskann_duplicate_insert_boundary USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_duplicate_insert_boundary VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("first insert should bootstrap");

        Spi::run(
            "INSERT INTO ec_diskann_duplicate_insert_boundary VALUES
             (2, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("duplicate insert should bind to the seed node");

        let metadata = index_metadata("ec_diskann_duplicate_insert_boundary_idx");
        assert_eq!(
            metadata.inserted_since_rebuild, 1,
            "duplicate bind must not advance inserted_since_rebuild",
        );
        assert!(
            !metadata.needs_medoid_refresh,
            "duplicate bind should not set needs_medoid_refresh",
        );

        let row1_heap_tid = heap_tid_for_row("ec_diskann_duplicate_insert_boundary", 1);
        let row2_heap_tid = heap_tid_for_row("ec_diskann_duplicate_insert_boundary", 2);
        let (materialized_metadata, chain) =
            index_materialized_chain("ec_diskann_duplicate_insert_boundary_idx");
        let reader = PersistedGraphReader::new(
            &chain,
            materialized_metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&materialized_metadata),
            scan_state::metadata_search_code_len(&materialized_metadata),
        );
        let node_tids = reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node tid iteration should succeed");
        assert_eq!(
            node_tids.len(),
            1,
            "duplicate bind should not create a new graph node"
        );
        let node_tid = node_tids[0];
        let node_tuple = reader.read_node(node_tid).expect("seed node should decode");
        assert!(
            node_tuple.has_overflow_heaptids,
            "duplicate bind should set the overflow flag on the seed node",
        );
        assert_eq!(
            insert::bound_heap_tids_for_owner(&chain, node_tid, node_tuple.primary_heaptid)
                .expect("bound heap tids should decode"),
            vec![row1_heap_tid, row2_heap_tid],
            "duplicate bind should preserve primary row first and overflow row second",
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");
        let mut ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_duplicate_insert_boundary \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 2",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        ordered_ids.sort_unstable();
        assert_eq!(
            ordered_ids,
            vec![1, 2],
            "runtime scan should expand the duplicate-bound overflow heap tid",
        );
    }

    #[pg_test]
    fn test_ec_diskann_duplicate_bind_grows_second_overflow_tuple() {
        Spi::run(
            "CREATE TABLE ec_diskann_duplicate_overflow_growth (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_duplicate_overflow_growth_idx ON ec_diskann_duplicate_overflow_growth USING ec_diskann \
             (embedding ecvector_diskann_ip_ops) WITH (list_size = 12, rerank_budget = 12, top_k = 12)",
        )
        .expect("index creation should succeed");
        for id in 1..=12_i64 {
            Spi::run(&format!(
                "INSERT INTO ec_diskann_duplicate_overflow_growth VALUES \
                 ({id}, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))"
            ))
            .expect("duplicate insert should succeed");
        }

        let metadata = index_metadata("ec_diskann_duplicate_overflow_growth_idx");
        assert_eq!(
            metadata.inserted_since_rebuild, 1,
            "duplicate binds must keep inserted_since_rebuild pinned to the seed insert",
        );

        let (materialized_metadata, chain) =
            index_materialized_chain("ec_diskann_duplicate_overflow_growth_idx");
        let reader = PersistedGraphReader::new(
            &chain,
            materialized_metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&materialized_metadata),
            scan_state::metadata_search_code_len(&materialized_metadata),
        );
        let node_tids = reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node tid iteration should succeed");
        assert_eq!(
            node_tids.len(),
            1,
            "twelve identical rows should still bind to a single graph node",
        );
        let node_tid = node_tids[0];
        let node_tuple = reader.read_node(node_tid).expect("seed node should decode");
        assert!(node_tuple.has_overflow_heaptids);
        let bound_heap_tids =
            insert::bound_heap_tids_for_owner(&chain, node_tid, node_tuple.primary_heaptid)
                .expect("bound heap tids should decode");
        assert_eq!(
            bound_heap_tids.len(),
            12,
            "overflow expansion should surface every duplicate heap tid across multiple overflow tuples",
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");
        let mut ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_duplicate_overflow_growth \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 12",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        ordered_ids.sort_unstable();
        assert_eq!(
            ordered_ids,
            (1..=12_i64).collect::<Vec<_>>(),
            "runtime scan should expand multi-tuple duplicate overflow chains",
        );
    }

    #[pg_test]
    fn test_ec_diskann_vacuum_noop_stats_on_empty_index() {
        Spi::run(
            "CREATE TABLE ec_diskann_vacuum_noop_empty (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_vacuum_noop_empty_idx ON ec_diskann_vacuum_noop_empty USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");

        let stats = unsafe { debug_vacuum_stats(index_oid("ec_diskann_vacuum_noop_empty_idx")) };
        assert_eq!(stats.num_index_tuples, 0.0);
        assert_eq!(stats.tuples_removed, 0.0);
        assert!(!stats.estimated_count);
        assert!(
            stats.num_pages >= 1,
            "vacuum stats should at least report the metadata page",
        );
    }

    #[pg_test]
    fn test_ec_diskann_vacuum_promotes_duplicate_overflow_primary() {
        Spi::run(
            "CREATE TABLE ec_diskann_vacuum_duplicate_promote (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_vacuum_duplicate_promote_idx ON ec_diskann_vacuum_duplicate_promote USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_vacuum_duplicate_promote VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("duplicate seed rows should insert");

        let row1_heap_tid = heap_tid_for_row("ec_diskann_vacuum_duplicate_promote", 1);
        let row2_heap_tid = heap_tid_for_row("ec_diskann_vacuum_duplicate_promote", 2);
        Spi::run("DELETE FROM ec_diskann_vacuum_duplicate_promote WHERE id = 1")
            .expect("delete should succeed");

        let stats = unsafe {
            debug_vacuum_remove_heap_tids(
                index_oid("ec_diskann_vacuum_duplicate_promote_idx"),
                &[row1_heap_tid],
            )
        };
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 1.0);

        let (metadata, chain) = index_materialized_chain("ec_diskann_vacuum_duplicate_promote_idx");
        let reader = PersistedGraphReader::new(
            &chain,
            metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&metadata),
            scan_state::metadata_search_code_len(&metadata),
        );
        let node_tids = reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node iteration should succeed");
        assert_eq!(node_tids.len(), 1);
        let node_tid = node_tids[0];
        let node_tuple = reader.read_node(node_tid).expect("node should decode");
        assert_eq!(
            node_tuple.primary_heaptid, row2_heap_tid,
            "vacuum should promote the surviving overflow heap tid into the primary slot",
        );
        assert!(
            !node_tuple.has_overflow_heaptids,
            "a lone surviving duplicate should clear the overflow flag after promotion",
        );
        assert_eq!(
            insert::bound_heap_tids_for_owner(&chain, node_tid, node_tuple.primary_heaptid)
                .expect("bound heap tids should decode"),
            vec![row2_heap_tid],
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");
        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_vacuum_duplicate_promote \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(ordered_ids, vec![2]);
    }

    #[pg_test]
    fn test_ec_diskann_vacuum_unlinks_and_tombstones_dead_node() {
        Spi::run(
            "CREATE TABLE ec_diskann_vacuum_unlink_dead (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_vacuum_unlink_dead_idx ON ec_diskann_vacuum_unlink_dead USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_vacuum_unlink_dead VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("seed and appended row should insert");

        let row1_heap_tid = heap_tid_for_row("ec_diskann_vacuum_unlink_dead", 1);
        let row2_heap_tid = heap_tid_for_row("ec_diskann_vacuum_unlink_dead", 2);
        let (prefill_metadata, prefill_chain) =
            index_materialized_chain("ec_diskann_vacuum_unlink_dead_idx");
        let prefill_reader = PersistedGraphReader::new(
            &prefill_chain,
            prefill_metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&prefill_metadata),
            scan_state::metadata_search_code_len(&prefill_metadata),
        );
        let row1_tid = prefill_reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node iteration should succeed")
            .into_iter()
            .find(|&tid| {
                prefill_reader
                    .read_node(tid)
                    .expect("node should decode")
                    .primary_heaptid
                    == row1_heap_tid
            })
            .expect("row 1 node tid should exist");
        let row2_tid = prefill_reader
            .iter_node_tids()
            .collect::<Result<Vec<_>, _>>()
            .expect("node iteration should succeed")
            .into_iter()
            .find(|&tid| {
                prefill_reader
                    .read_node(tid)
                    .expect("node should decode")
                    .primary_heaptid
                    == row2_heap_tid
            })
            .expect("row 2 node tid should exist");

        Spi::run("DELETE FROM ec_diskann_vacuum_unlink_dead WHERE id = 2")
            .expect("delete should succeed");
        let stats = unsafe {
            debug_vacuum_remove_heap_tids(
                index_oid("ec_diskann_vacuum_unlink_dead_idx"),
                &[row2_heap_tid],
            )
        };
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 1.0);

        let (metadata, chain) = index_materialized_chain("ec_diskann_vacuum_unlink_dead_idx");
        let reader = PersistedGraphReader::new(
            &chain,
            metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&metadata),
            scan_state::metadata_search_code_len(&metadata),
        );
        let row1_tuple = reader
            .read_node(row1_tid)
            .expect("row 1 node should decode");
        let row1_neighbors = row1_tuple
            .neighbors
            .iter()
            .take(row1_tuple.neighbor_count as usize)
            .copied()
            .collect::<Vec<_>>();
        assert!(
            !row1_neighbors.contains(&row2_tid),
            "vacuum pass 2 should unlink dead neighbor references from live nodes",
        );

        let row2_tuple = reader
            .read_node(row2_tid)
            .expect("row 2 node should decode");
        assert_eq!(row2_tuple.primary_heaptid, ItemPointer::INVALID);
        assert!(
            row2_tuple.deleted,
            "vacuum pass 3 should tombstone fully dead tuples",
        );

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_bitmapscan = off").expect("SET LOCAL should succeed");
        Spi::run("SET LOCAL enable_sort = off").expect("SET LOCAL should succeed");
        let ordered_ids = Spi::connect(|client| {
            client
                .select(
                    "SELECT id FROM ec_diskann_vacuum_unlink_dead \
                     ORDER BY embedding <#> ARRAY[0.0, 1.0, 0.25, -0.5]::real[] \
                     LIMIT 1",
                    None,
                    &[],
                )
                .expect("ordered SELECT should succeed")
                .map(|row| {
                    row["id"]
                        .value::<i64>()
                        .expect("id should decode")
                        .expect("id should be non-null")
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(ordered_ids, vec![1]);
    }

    #[pg_test]
    fn test_ec_diskann_vacuum_refills_broken_neighbor_slot() {
        Spi::run(
            "CREATE TABLE ec_diskann_vacuum_refill_slot (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_vacuum_refill_slot_idx ON ec_diskann_vacuum_refill_slot USING ec_diskann \
             (embedding ecvector_diskann_ip_ops) WITH (graph_degree = 4, build_list_size = 10)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_vacuum_refill_slot VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.1, -1.0, 0.0, 0.0, 0.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.1, 0.0, -1.0, 0.0, 0.0], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.1, 0.0, 0.0, -1.0, 0.0], 4, 42)),
             (5, encode_to_ecvector(ARRAY[0.1, 0.0, 0.0, 0.0, -1.0], 4, 42)),
             (6, encode_to_ecvector(ARRAY[1.0, 1.0, 1.0, 1.0, 1.0], 4, 42))",
        )
        .expect("fixture rows should insert");

        let VacuumRefillFixture {
            prefill_metadata,
            prefill_chain,
            target_tid,
            target_neighbors_before,
            replacement_tid,
            deleted_tid,
        } = find_vacuum_refill_fixture("ec_diskann_vacuum_refill_slot_idx");
        let prefill_reader = PersistedGraphReader::new(
            &prefill_chain,
            prefill_metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&prefill_metadata),
            scan_state::metadata_search_code_len(&prefill_metadata),
        );
        let binary_word_count = scan_state::metadata_binary_word_count(&prefill_metadata);
        let search_code_len = scan_state::metadata_search_code_len(&prefill_metadata);

        let mut mutated_chain = prefill_chain.clone();
        let mut target_tuple_before = super::read_chain_node(
            &mutated_chain,
            prefill_metadata.graph_degree_r,
            binary_word_count,
            search_code_len,
            target_tid,
        )
        .expect("target tuple should decode before rewrite");
        for slot in &mut target_tuple_before.neighbors {
            *slot = ItemPointer::INVALID;
        }
        for (slot, neighbor_tid) in target_neighbors_before.iter().copied().enumerate() {
            target_tuple_before.neighbors[slot] = neighbor_tid;
        }
        target_tuple_before.neighbor_count = prefill_metadata.graph_degree_r;
        super::write_chain_node(
            &mut mutated_chain,
            prefill_metadata.graph_degree_r,
            binary_word_count,
            search_code_len,
            target_tid,
            &target_tuple_before,
        )
        .expect("target rewrite should encode");
        let rewrites = super::collect_tuple_rewrites(&prefill_chain, &mutated_chain)
            .expect("target rewrite diff should collect");
        assert_eq!(
            rewrites.len(),
            1,
            "fixture rewrite should only touch the target tuple"
        );
        let index_relation = unsafe {
            pg_sys::index_open(
                index_oid("ec_diskann_vacuum_refill_slot_idx"),
                pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
            )
        };
        assert_eq!(
            unsafe { super::apply_tuple_rewrites(index_relation, &rewrites) }
                .expect("fixture rewrite should apply"),
            super::VacuumRewriteApplyOutcome::Applied,
        );
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::RowExclusiveLock as pg_sys::LOCKMODE)
        };

        let deleted_heap_tid = prefill_reader
            .read_node(deleted_tid)
            .expect("deleted tuple should decode before vacuum")
            .primary_heaptid;
        let deleted_row_id = row_id_for_heap_tid("ec_diskann_vacuum_refill_slot", deleted_heap_tid);

        Spi::run(&format!(
            "DELETE FROM ec_diskann_vacuum_refill_slot WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        let stats = unsafe {
            debug_vacuum_remove_heap_tids(
                index_oid("ec_diskann_vacuum_refill_slot_idx"),
                &[deleted_heap_tid],
            )
        };
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 5.0);

        let (metadata, chain) = index_materialized_chain("ec_diskann_vacuum_refill_slot_idx");
        let reader = PersistedGraphReader::new(
            &chain,
            metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&metadata),
            scan_state::metadata_search_code_len(&metadata),
        );
        let target_tuple_after = reader
            .read_node(target_tid)
            .expect("target tuple should decode after vacuum");
        let target_neighbors_after = target_tuple_after
            .neighbors
            .iter()
            .take(target_tuple_after.neighbor_count as usize)
            .copied()
            .collect::<Vec<_>>();
        assert!(
            !target_neighbors_after.contains(&deleted_tid),
            "vacuum refill should still remove the deleted neighbor tid",
        );
        assert!(
            target_neighbors_after.contains(&replacement_tid),
            "vacuum refill should install the previously-missing live candidate into the freed slot; before={target_neighbors_before:?} after={target_neighbors_after:?} replacement={replacement_tid:?}",
        );
        assert_eq!(
            target_neighbors_after.len(),
            target_neighbors_before.len(),
            "refill should restore the live node's neighbor count after the delete",
        );
    }

    #[pg_test]
    fn test_ec_diskann_vacuum_replans_on_stale_repair_tuple() {
        let _state = ScopedVacuumRetryState::new();

        Spi::run(
            "CREATE TABLE ec_diskann_vacuum_retry_replan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_vacuum_retry_replan_idx ON ec_diskann_vacuum_retry_replan USING ec_diskann \
             (embedding ecvector_diskann_ip_ops) WITH (graph_degree = 4, build_list_size = 10)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_vacuum_retry_replan VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.1, -1.0, 0.0, 0.0, 0.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.1, 0.0, -1.0, 0.0, 0.0], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.1, 0.0, 0.0, -1.0, 0.0], 4, 42)),
             (5, encode_to_ecvector(ARRAY[0.1, 0.0, 0.0, 0.0, -1.0], 4, 42)),
             (6, encode_to_ecvector(ARRAY[1.0, 1.0, 1.0, 1.0, 1.0], 4, 42))",
        )
        .expect("fixture rows should insert");

        let VacuumRefillFixture {
            prefill_metadata,
            prefill_chain,
            target_tid,
            target_neighbors_before,
            replacement_tid,
            deleted_tid,
        } = find_vacuum_refill_fixture("ec_diskann_vacuum_retry_replan_idx");
        let prefill_reader = PersistedGraphReader::new(
            &prefill_chain,
            prefill_metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&prefill_metadata),
            scan_state::metadata_search_code_len(&prefill_metadata),
        );
        let binary_word_count = scan_state::metadata_binary_word_count(&prefill_metadata);
        let search_code_len = scan_state::metadata_search_code_len(&prefill_metadata);

        let mut mutated_chain = prefill_chain.clone();
        let mut target_tuple_before = super::read_chain_node(
            &mutated_chain,
            prefill_metadata.graph_degree_r,
            binary_word_count,
            search_code_len,
            target_tid,
        )
        .expect("target tuple should decode before rewrite");
        for slot in &mut target_tuple_before.neighbors {
            *slot = ItemPointer::INVALID;
        }
        for (slot, neighbor_tid) in target_neighbors_before.iter().copied().enumerate() {
            target_tuple_before.neighbors[slot] = neighbor_tid;
        }
        target_tuple_before.neighbor_count = prefill_metadata.graph_degree_r;
        super::write_chain_node(
            &mut mutated_chain,
            prefill_metadata.graph_degree_r,
            binary_word_count,
            search_code_len,
            target_tid,
            &target_tuple_before,
        )
        .expect("target rewrite should encode");
        let rewrites = super::collect_tuple_rewrites(&prefill_chain, &mutated_chain)
            .expect("target rewrite diff should collect");
        assert_eq!(
            rewrites.len(),
            1,
            "fixture rewrite should only touch the target tuple"
        );
        let index_relation = unsafe {
            pg_sys::index_open(
                index_oid("ec_diskann_vacuum_retry_replan_idx"),
                pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
            )
        };
        assert_eq!(
            unsafe { super::apply_tuple_rewrites(index_relation, &rewrites) }
                .expect("fixture rewrite should apply"),
            super::VacuumRewriteApplyOutcome::Applied,
        );
        unsafe {
            pg_sys::index_close(index_relation, pg_sys::RowExclusiveLock as pg_sys::LOCKMODE)
        };

        let mut drifted_target_tuple = target_tuple_before.clone();
        let replaced_deleted_neighbor = drifted_target_tuple.neighbors.iter_mut().any(|neighbor| {
            if *neighbor == deleted_tid {
                *neighbor = replacement_tid;
                true
            } else {
                false
            }
        });
        assert!(
            replaced_deleted_neighbor,
            "fixture drift should replace the soon-to-be-deleted neighbor",
        );
        super::set_vacuum_rewrite_test_injection(super::VacuumRewriteTestInjection {
            target_tid,
            replacement_raw: drifted_target_tuple
                .encode(
                    prefill_metadata.graph_degree_r,
                    binary_word_count,
                    search_code_len,
                )
                .expect("drifted target tuple should encode"),
        });

        let deleted_heap_tid = prefill_reader
            .read_node(deleted_tid)
            .expect("deleted tuple should decode before vacuum")
            .primary_heaptid;
        let deleted_row_id =
            row_id_for_heap_tid("ec_diskann_vacuum_retry_replan", deleted_heap_tid);

        Spi::run(&format!(
            "DELETE FROM ec_diskann_vacuum_retry_replan WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        let stats = unsafe {
            debug_vacuum_remove_heap_tids(
                index_oid("ec_diskann_vacuum_retry_replan_idx"),
                &[deleted_heap_tid],
            )
        };
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 5.0);
        assert_eq!(
            super::vacuum_replan_event_count(),
            1,
            "stale rewrite injection should force exactly one replan pass",
        );

        let (metadata, chain) = index_materialized_chain("ec_diskann_vacuum_retry_replan_idx");
        let reader = PersistedGraphReader::new(
            &chain,
            metadata.graph_degree_r,
            scan_state::metadata_binary_word_count(&metadata),
            scan_state::metadata_search_code_len(&metadata),
        );
        let target_tuple_after = reader
            .read_node(target_tid)
            .expect("target tuple should decode after vacuum");
        let target_neighbors_after = target_tuple_after
            .neighbors
            .iter()
            .take(target_tuple_after.neighbor_count as usize)
            .copied()
            .collect::<Vec<_>>();
        assert!(
            !target_neighbors_after.contains(&deleted_tid),
            "vacuum retry should not resurrect the deleted neighbor after replanning",
        );
        assert!(
            target_neighbors_after.contains(&replacement_tid),
            "vacuum retry should preserve the drifted replacement after replanning",
        );
    }

    #[pg_test]
    fn test_ec_diskann_vacuum_sets_medoid_refresh_flag() {
        Spi::run(
            "CREATE TABLE ec_diskann_vacuum_medoid_refresh (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_vacuum_medoid_refresh_idx ON ec_diskann_vacuum_medoid_refresh USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_vacuum_medoid_refresh VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed row should insert");

        let row1_heap_tid = heap_tid_for_row("ec_diskann_vacuum_medoid_refresh", 1);
        Spi::run("DELETE FROM ec_diskann_vacuum_medoid_refresh WHERE id = 1")
            .expect("delete should succeed");
        let stats = unsafe {
            debug_vacuum_remove_heap_tids(
                index_oid("ec_diskann_vacuum_medoid_refresh_idx"),
                &[row1_heap_tid],
            )
        };
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 0.0);

        let metadata = index_metadata("ec_diskann_vacuum_medoid_refresh_idx");
        assert!(
            metadata.needs_medoid_refresh,
            "vacuum should own the monotonic medoid-refresh flag when the entry point dies",
        );
    }
}
