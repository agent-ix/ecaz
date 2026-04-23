use pgrx::pg_sys;

pub(super) unsafe extern "C-unwind" fn symphony_ambuild(
    _heap_relation: pg_sys::Relation,
    _index_relation: pg_sys::Relation,
    _index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("ambuild")) }
}

pub(super) unsafe extern "C-unwind" fn symphony_ambuildempty(_index_relation: pg_sys::Relation) {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("ambuildempty")) }
}
