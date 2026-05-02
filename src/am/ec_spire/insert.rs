use pgrx::pg_sys;

pub(super) unsafe extern "C-unwind" fn ec_spire_aminsert(
    _index_relation: pg_sys::Relation,
    _values: *mut pg_sys::Datum,
    _isnull: *mut bool,
    _heap_tid: pg_sys::ItemPointer,
    _heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("aminsert")) }
}
