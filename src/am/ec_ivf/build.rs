use std::ffi::c_void;
use std::ptr;

use pgrx::{pg_sys, PgBox};

use super::{options, page};

struct BuildState {
    scanned_tuples: usize,
}

unsafe extern "C-unwind" fn ec_ivf_build_callback(
    _index: pg_sys::Relation,
    _tid: pg_sys::ItemPointer,
    _values: *mut pg_sys::Datum,
    _isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = &mut *state.cast::<BuildState>();
            state.scanned_tuples += 1;
            pgrx::error!("ec_ivf populated builds are not implemented yet");
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let options = options::relation_options(index_relation);
            let metadata = page::MetadataPage::empty(options);
            page::initialize_metadata_page(index_relation, metadata);

            let mut state = BuildState { scanned_tuples: 0 };
            let heap_tuples = pg_sys::table_index_build_scan(
                heap_relation,
                index_relation,
                index_info,
                false,
                false,
                Some(ec_ivf_build_callback),
                (&mut state as *mut BuildState).cast(),
                ptr::null_mut(),
            );
            if state.scanned_tuples != 0 {
                pgrx::error!(
                    "ec_ivf populated builds are not implemented yet; saw {} heap tuples",
                    state.scanned_tuples
                );
            }

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = heap_tuples;
            result.index_tuples = 0.0;
            result.into_pg()
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambuildempty(index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let options = options::relation_options(index_relation);
            let metadata = page::MetadataPage::empty(options);
            page::initialize_metadata_page(index_relation, metadata);
        })
    }
}
