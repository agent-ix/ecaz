use std::{cell::RefCell, ffi::c_void, ptr};

use pgrx::{pg_guard, pg_sys, AllocatedByRust, FromDatum, PgBox, PgMemoryContexts};

use crate::{
    quant::grouped_pq::{build_grouped_pq_lut_f32, grouped_pq_score_f32, GROUPED_PQ_CENTROIDS},
    storage::page::ItemPointer,
};

use super::{
    ambuild, insert, options,
    page::VamanaMetadataPage,
    reader::{PersistedGraphReader, VisitedState},
    scan::{self, ScanParams},
    scan_query::{
        build_grouped_pq_lut_from_persisted, encode_query_srht, read_grouped_codebook_chain,
    },
    scan_state::{self, DiskannScanOpaque},
};

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
    amroutine.amcostestimate = Some(ec_diskann_amcostestimate);
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
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("ec_diskann ambulkdelete is not yet implemented (task 17 phase 5)");
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amvacuumcleanup(
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("ec_diskann amvacuumcleanup is not yet implemented (task 17 phase 5)");
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amcostestimate(
    _root: *mut pg_sys::PlannerInfo,
    _path: *mut pg_sys::IndexPath,
    _loop_count: f64,
    index_startup_cost: *mut pg_sys::Cost,
    index_total_cost: *mut pg_sys::Cost,
    index_selectivity: *mut pg_sys::Selectivity,
    index_correlation: *mut f64,
    index_pages: *mut f64,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            // Phase 1A: surface a prohibitive cost so the planner never
            // picks ec_diskann. Phase 9 replaces this with a real cost
            // model once planner opt-in lands.
            *index_startup_cost = pg_sys::disable_cost;
            *index_total_cost = pg_sys::disable_cost;
            *index_selectivity = 1.0;
            *index_correlation = 0.0;
            *index_pages = 0.0;
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
            opaque.visited.clear();
            opaque.result_buf.clear();
            opaque.result_cursor = 0;

            if opaque.metadata.dimensions == 0 {
                opaque.rescan_called = true;
                return;
            }

            let group_count = usize::from(opaque.metadata.search_subvector_count);
            let group_size = usize::from(opaque.metadata.search_subvector_dim);
            if group_count == 0 || group_size == 0 {
                pgrx::error!(
                    "ec_diskann scan metadata is missing grouped-PQ shape: group_count={}, group_size={}",
                    group_count,
                    group_size
                );
            }

            let (helper_lut, helper_group_count) = build_grouped_pq_lut_from_persisted(
                &opaque.chain,
                opaque.metadata.grouped_codebook_head,
                group_count,
                group_size,
                opaque.metadata.dimensions as usize,
                opaque.metadata.seed,
                &raw_query,
            )
            .unwrap_or_else(|e| pgrx::error!("ec_diskann scan query LUT build failed: {e}"));
            opaque.flat_codebooks = read_grouped_codebook_chain(
                &opaque.chain,
                opaque.metadata.grouped_codebook_head,
                group_count,
                GROUPED_PQ_CENTROIDS * group_size,
            )
            .unwrap_or_else(|e| pgrx::error!("ec_diskann scan codebook load failed: {e}"));
            opaque.query_rotated = encode_query_srht(
                &raw_query,
                opaque.metadata.dimensions as usize,
                opaque.metadata.seed,
            );
            opaque.query_lut =
                build_grouped_pq_lut_f32(&opaque.query_rotated, &opaque.flat_codebooks, group_size);
            if helper_group_count != group_count || helper_lut != opaque.query_lut {
                pgrx::error!(
                    "ec_diskann scan LUT reconstruction drifted from the persisted helper path"
                );
            }

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
            let results = scan::vamana_scan_with(
                &reader,
                &mut opaque.visited,
                ScanParams {
                    entry_point,
                    list_size: opaque.list_size,
                    rerank_budget: opaque.rerank_budget,
                    top_k: opaque.top_k,
                },
                |tuple| -grouped_pq_score_f32(&opaque.query_lut, group_count, &tuple.search_code),
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
                opaque.top_k,
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

fn expand_scan_results_with_bound_heap_tids(
    chain: &crate::storage::page::DataPageChain,
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

unsafe fn exact_heap_rerank_distance(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attnum: i32,
    raw_query: &[f32],
    heap_tid: ItemPointer,
) -> Result<f32, String> {
    let source_vector = unsafe {
        fetch_heap_source_vector(
            heap_relation,
            snapshot,
            slot,
            source_attnum,
            heap_tid,
            "heap rerank source vector",
        )?
    };
    if source_vector.len() != raw_query.len() {
        return Err(format!(
            "ec_diskann heap rerank dimension mismatch: query dim {}, heap dim {}",
            raw_query.len(),
            source_vector.len()
        ));
    }
    let distance = -raw_query
        .iter()
        .zip(source_vector.iter())
        .map(|(left, right)| left * right)
        .sum::<f32>();
    Ok(distance)
}

unsafe fn fetch_heap_source_vector(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attnum: i32,
    heap_tid: ItemPointer,
    context: &str,
) -> Result<Vec<f32>, String> {
    unsafe { scan_state::fetch_heap_row_version(heap_relation, heap_tid, snapshot, slot)? };
    let datum = unsafe { scan_state::required_slot_datum(slot, source_attnum, context)? };
    let source_vector = unsafe { ambuild::ecvector_datum_to_vec(datum) };
    unsafe { pg_sys::ExecClearTuple(slot) };
    Ok(source_vector)
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

    fn index_metadata(index_name: &str) -> VamanaMetadataPage {
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let index_relation =
            unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let (metadata, _) = unsafe { scan_state::materialize_chain_from_index(index_relation) }
            .expect("materialize_chain_from_index should succeed");
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        metadata
    }

    fn index_materialized_chain(index_name: &str) -> (VamanaMetadataPage, DataPageChain) {
        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let index_relation =
            unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        let materialized = unsafe { scan_state::materialize_chain_from_index(index_relation) }
            .expect("materialize_chain_from_index should succeed");
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        materialized
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

        let plan = Spi::connect(|client| {
            let rows = client
                .select(
                    "EXPLAIN (COSTS OFF) \
                     SELECT id FROM ec_diskann_sql_ordered_exec \
                     ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[] \
                     LIMIT 2",
                    None,
                    &[],
                )
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
        });

        assert!(
            plan.contains("Index Scan") || plan.contains("Index Only Scan"),
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
}
