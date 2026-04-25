use pgrx::pg_sys;

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambeginscan(
    _index_relation: pg_sys::Relation,
    _nkeys: std::ffi::c_int,
    _norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("ambeginscan")) }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amrescan(
    _scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    _nkeys: std::ffi::c_int,
    _orderbys: pg_sys::ScanKey,
    _norderbys: std::ffi::c_int,
) {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amrescan")) }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amgettuple(
    _scan: pg_sys::IndexScanDesc,
    _direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amgettuple")) }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amendscan(_scan: pg_sys::IndexScanDesc) {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amendscan")) }
}
