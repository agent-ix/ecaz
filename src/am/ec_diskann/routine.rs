use std::{cell::RefCell, ffi::c_void, ptr};

use pgrx::{pg_guard, pg_sys, AllocatedByRust, FromDatum, PgBox, PgMemoryContexts};

use crate::{
    quant::grouped_pq::{build_grouped_pq_lut_f32, grouped_pq_score_f32, GROUPED_PQ_CENTROIDS},
    storage::page::ItemPointer,
};

use super::{
    ambuild, insert, options,
    reader::PersistedGraphReader,
    scan::{self, ScanParams},
    scan_query::{build_grouped_pq_lut_from_persisted, encode_query_srht, read_grouped_codebook_chain},
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
    _heap_relation: pg_sys::Relation,
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
            let metadata = insert::read_metadata_page(index_relation)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann aminsert failed to read metadata: {e}"));

            if metadata.dimensions == 0 && metadata.entry_point == ItemPointer::INVALID {
                let bootstrapped = insert::with_locked_metadata_page(index_relation, |metadata| {
                    if metadata.dimensions != 0 || metadata.entry_point != ItemPointer::INVALID {
                        return Ok(false);
                    }
                    let output =
                        insert::bootstrap_empty_insert_output(index_relation, heap_tid, &source_vector)?;
                    ambuild::write_data_pages(index_relation, &output.chain);
                    *metadata = output.metadata;
                    Ok(true)
                })
                .unwrap_or_else(|e| pgrx::error!("ec_diskann empty-index bootstrap insert failed: {e}"));
                if bootstrapped {
                    return false;
                }
            }

            let refreshed = insert::read_metadata_page(index_relation)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann aminsert failed to refresh metadata: {e}"));
            if refreshed.dimensions != 0 && source_vector.len() != refreshed.dimensions as usize {
                pgrx::error!(
                    "ec_diskann insert source dimension mismatch: source dim {}, index dim {}",
                    source_vector.len(),
                    refreshed.dimensions
                );
            }

            pgrx::error!("ec_diskann non-empty aminsert is not yet implemented (task 17 phase 7)");
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

            let opaque =
                PgBox::<DiskannScanOpaque>::alloc_in_context(PgMemoryContexts::CurrentMemoryContext);
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
            if opaque.metadata.dimensions != 0 && raw_query.len() != opaque.metadata.dimensions as usize {
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
                pgrx::error!("ec_diskann scan LUT reconstruction drifted from the persisted helper path");
            }

            let reader = PersistedGraphReader::new(
                &opaque.chain,
                opaque.metadata.graph_degree_r,
                opaque.binary_word_count(),
                opaque.search_code_len(),
            );
            let entry_point = scan::resolve_entry_point(&reader, opaque.metadata.entry_point)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann scan entry-point resolution failed: {e}"));
            let Some(entry_point) = entry_point else {
                opaque.rescan_called = true;
                return;
            };

            let heap_relation_state = scan_state::resolve_scan_heap_relation(scan)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann scan heap relation setup failed: {e}"));
            let snapshot_state = scan_state::resolve_scan_snapshot(scan)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann scan snapshot setup failed: {e}"));
            let slot = scan_state::allocate_heap_slot(heap_relation_state.0)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann scan heap slot setup failed: {e}"));
            let source_attnum = indexed_ecvector_attnum((*scan).indexRelation)
                .unwrap_or_else(|e| pgrx::error!("ec_diskann scan source-column resolution failed: {e}"));
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
            opaque.result_buf =
                results.unwrap_or_else(|e| pgrx::error!("ec_diskann scan execution failed: {e}"));
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

unsafe fn exact_heap_rerank_distance(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
    slot: *mut pg_sys::TupleTableSlot,
    source_attnum: i32,
    raw_query: &[f32],
    heap_tid: ItemPointer,
) -> Result<f32, String> {
    unsafe { scan_state::fetch_heap_row_version(heap_relation, heap_tid, snapshot, slot)? };
    let datum = unsafe {
        scan_state::required_slot_datum(slot, source_attnum, "heap rerank source vector")?
    };
    let source_vector = unsafe { ambuild::ecvector_datum_to_vec(datum) };
    if source_vector.len() != raw_query.len() {
        unsafe { pg_sys::ExecClearTuple(slot) };
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
    unsafe { pg_sys::ExecClearTuple(slot) };
    Ok(distance)
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
    use pgrx::{pg_test, Spi};

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

        assert_eq!(ordered_ids.len(), 2, "query should return the requested LIMIT");
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
    fn test_ec_diskann_second_insert_still_errors() {
        Spi::run(
            "CREATE TABLE ec_diskann_second_insert_boundary (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_diskann_second_insert_boundary_idx ON ec_diskann_second_insert_boundary USING ec_diskann \
             (embedding ecvector_diskann_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_diskann_second_insert_boundary VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("first insert should bootstrap");

        Spi::run(
            "DO $$
             BEGIN
               BEGIN
                 INSERT INTO ec_diskann_second_insert_boundary VALUES
                   (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42));
                 RAISE EXCEPTION 'expected non-empty ec_diskann insert to fail in this slice';
               EXCEPTION
                 WHEN OTHERS THEN
                   IF SQLERRM NOT LIKE '%ec_diskann non-empty aminsert is not yet implemented%' THEN
                     RAISE;
                   END IF;
               END;
             END
             $$",
        )
        .expect("boundary insert should fail with the expected message");
    }
}
