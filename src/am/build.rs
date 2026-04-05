use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, varlena, FromDatum, PgBox, PgTupleDesc};

use super::{
    flush_build_state, initialize_metadata_page, page, tqhnsw_build_callback, BuildState,
    BuildTuple,
};

pub(super) unsafe extern "C-unwind" fn tqhnsw_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut state = BuildState::new(index_relation);

            initialize_metadata_page(index_relation, state.initial_metadata());

            let heap_tuples = if state.options.build_source_column.is_some() {
                tqhnsw_build_scan_with_source(heap_relation, index_info, &mut state)
            } else {
                pg_sys::table_index_build_scan(
                    heap_relation,
                    index_relation,
                    index_info,
                    false,
                    false,
                    Some(tqhnsw_build_callback),
                    (&mut state as *mut BuildState).cast(),
                    ptr::null_mut(),
                )
            };
            let index_tuples = if state.heap_tuples.is_empty() {
                0.0
            } else {
                flush_build_state(index_relation, &state);
                state.heap_tuples.len() as f64
            };

            if heap_tuples != state.scanned_tuples as f64 {
                pgrx::error!(
                    "tqhnsw ambuild scanned {heap_tuples} heap tuples but observed {}",
                    state.scanned_tuples
                );
            }

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = heap_tuples;
            result.index_tuples = index_tuples;
            result.into_pg()
        })
    }
}

pub(super) unsafe extern "C-unwind" fn tqhnsw_ambuildempty(index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = BuildState::new(index_relation);
            initialize_metadata_page(index_relation, state.initial_metadata());
        })
    }
}

pub(super) unsafe fn build_heap_tuple(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: page::ItemPointer,
) -> BuildTuple {
    if values.is_null() || isnull.is_null() {
        pgrx::error!("tqhnsw ambuild received null tuple value arrays");
    }
    if unsafe { *isnull } {
        pgrx::error!("tqhnsw does not support NULL indexed values");
    }

    let datum = unsafe { *values };
    if datum.is_null() {
        pgrx::error!("tqhnsw ambuild received a null tqvector datum");
    }

    let original = datum.cast_mut_ptr::<std::ffi::c_void>().cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    let is_copy = !std::ptr::eq(varlena, original);
    let bytes = unsafe { varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if is_copy {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }

    let (dimensions, bits, seed, gamma, code) = crate::unpack(&bytes)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw ambuild found invalid tqvector: {e}"));

    BuildTuple {
        heap_tids: vec![heap_tid],
        dimensions,
        bits,
        seed,
        gamma,
        code: code.to_vec(),
        source_vector: None,
        source_count: 0,
    }
}

unsafe fn build_heap_tuple_with_source(
    vector_datum: pg_sys::Datum,
    heap_tid: page::ItemPointer,
    source_vector: Vec<f32>,
) -> BuildTuple {
    if vector_datum.is_null() {
        pgrx::error!("tqhnsw ambuild received a null tqvector datum");
    }

    let original = vector_datum
        .cast_mut_ptr::<std::ffi::c_void>()
        .cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    let is_copy = !std::ptr::eq(varlena, original);
    let bytes = unsafe { varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if is_copy {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }

    let (dimensions, bits, seed, gamma, code) = crate::unpack(&bytes)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw ambuild found invalid tqvector: {e}"));

    if source_vector.is_empty() {
        pgrx::error!("tqhnsw build_source_column arrays must not be empty");
    }
    if source_vector.len() != dimensions as usize {
        pgrx::error!(
            "tqhnsw build_source_column dimension mismatch: source dim {} vs tqvector dim {}",
            source_vector.len(),
            dimensions
        );
    }

    BuildTuple {
        heap_tids: vec![heap_tid],
        dimensions,
        bits,
        seed,
        gamma,
        code: code.to_vec(),
        source_vector: Some(source_vector),
        source_count: 1,
    }
}

pub(super) unsafe fn tqhnsw_build_scan_with_source(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    state: &mut BuildState,
) -> f64 {
    let source_column = state
        .options
        .build_source_column
        .clone()
        .expect("source scan should only run when build_source_column is configured");
    let index_attnum = unsafe { source_build_index_attnum(index_info) };
    let source_attnum = unsafe { resolve_source_attnum(heap_relation, &source_column) };
    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(source_attnum as usize - 1)
        .expect("resolved build source attribute should exist");
    if att.attisdropped {
        pgrx::error!("tqhnsw build_source_column \"{source_column}\" references a dropped column");
    }
    if att.atttypid != pg_sys::FLOAT4ARRAYOID {
        pgrx::error!(
            "tqhnsw build_source_column \"{source_column}\" must be real[], got type oid {}",
            u32::from(att.atttypid)
        );
    }

    let slot = unsafe {
        pg_sys::MakeSingleTupleTableSlot(
            (*heap_relation).rd_att,
            pg_sys::table_slot_callbacks(heap_relation),
        )
    };
    if slot.is_null() {
        pgrx::error!("tqhnsw ambuild failed to allocate heap scan slot");
    }

    let snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot()) };
    unsafe { pg_sys::PushActiveSnapshot(snapshot) };
    let scan = unsafe {
        pg_sys::heap_beginscan(
            heap_relation,
            snapshot,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            pg_sys::ScanOptions::SO_TYPE_SEQSCAN
                | pg_sys::ScanOptions::SO_ALLOW_PAGEMODE
                | pg_sys::ScanOptions::SO_ALLOW_STRAT
                | pg_sys::ScanOptions::SO_ALLOW_SYNC,
        )
    };
    if scan.is_null() {
        unsafe {
            pg_sys::UnregisterSnapshot(snapshot);
            pg_sys::ExecDropSingleTupleTableSlot(slot);
        }
        pgrx::error!("tqhnsw ambuild failed to begin heap scan");
    }

    let mut scanned_tuples = 0.0_f64;
    while unsafe {
        pg_sys::heap_getnextslot(scan, pg_sys::ScanDirection::ForwardScanDirection, slot)
    } {
        scanned_tuples += 1.0;
        let heap_tid = unsafe { decode_slot_tid(slot) };
        let vector_datum =
            unsafe { required_slot_datum(slot, index_attnum, "indexed tqvector column") };
        let source_datum =
            unsafe { required_slot_datum(slot, source_attnum, "tqhnsw build_source_column") };
        let source_vector = unsafe {
            Vec::<f32>::from_polymorphic_datum(source_datum, false, pg_sys::FLOAT4ARRAYOID)
        }
        .unwrap_or_else(|| {
            pgrx::error!("tqhnsw build_source_column \"{source_column}\" cannot be NULL")
        });

        let tuple = unsafe { build_heap_tuple_with_source(vector_datum, heap_tid, source_vector) };
        state.push(tuple);
    }

    unsafe {
        pg_sys::heap_endscan(scan);
        pg_sys::PopActiveSnapshot();
        pg_sys::UnregisterSnapshot(snapshot);
        pg_sys::ExecDropSingleTupleTableSlot(slot);
    }
    scanned_tuples
}

unsafe fn source_build_index_attnum(index_info: *mut pg_sys::IndexInfo) -> i32 {
    if index_info.is_null() {
        pgrx::error!("tqhnsw ambuild received a null IndexInfo");
    }
    let index_info = unsafe { &*index_info };
    if index_info.ii_NumIndexAttrs != 1 || index_info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("tqhnsw build_source_column currently supports single-column indexes only");
    }
    if !index_info.ii_Expressions.is_null() {
        pgrx::error!("tqhnsw build_source_column does not support expression indexes yet");
    }
    if !index_info.ii_Predicate.is_null() {
        pgrx::error!("tqhnsw build_source_column does not support partial indexes yet");
    }

    let attnum = i32::from(index_info.ii_IndexAttrNumbers[0]);
    if attnum <= 0 {
        pgrx::error!("tqhnsw build_source_column requires a base heap column index key");
    }
    attnum
}

unsafe fn resolve_source_attnum(heap_relation: pg_sys::Relation, source_column: &str) -> i32 {
    let source_column = std::ffi::CString::new(source_column).unwrap_or_else(|_| {
        pgrx::error!("tqhnsw build_source_column contains an invalid NUL byte")
    });
    let attnum = unsafe { pg_sys::get_attnum((*heap_relation).rd_id, source_column.as_ptr()) };
    let attnum = i32::from(attnum);
    if attnum <= 0 {
        pgrx::error!(
            "tqhnsw build_source_column \"{}\" does not name a user column on the heap relation",
            source_column.to_string_lossy()
        );
    }
    attnum
}

unsafe fn decode_slot_tid(slot: *mut pg_sys::TupleTableSlot) -> page::ItemPointer {
    let heap_tid = unsafe { (*slot).tts_tid };
    let tid = pg_sys::ItemPointerData {
        ip_blkid: heap_tid.ip_blkid,
        ip_posid: heap_tid.ip_posid,
    };
    let (block_number, offset_number) = item_pointer_get_both(tid);
    page::ItemPointer {
        block_number,
        offset_number,
    }
}

unsafe fn required_slot_datum(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    label: &str,
) -> pg_sys::Datum {
    if unsafe { (*slot).tts_nvalid } < attnum as i16 {
        unsafe { pg_sys::slot_getsomeattrs_int(slot, attnum) };
    }
    let attr_index = usize::try_from(attnum - 1).expect("attribute number should be positive");
    if unsafe { *(*slot).tts_isnull.add(attr_index) } {
        pgrx::error!("tqhnsw does not support NULL {label}");
    }
    unsafe { *(*slot).tts_values.add(attr_index) }
}
