//! PostgreSQL type metadata helpers.

use std::ffi::CStr;

use pgrx::pg_sys;

pub(crate) fn base_type_oid(type_oid: pg_sys::Oid) -> pg_sys::Oid {
    // SAFETY: PostgreSQL accepts any type OID and returns either the base type
    // OID or the original OID by value.
    unsafe { pg_sys::getBaseType(type_oid) }
}

pub(crate) fn formatted_base_type_name(type_oid: pg_sys::Oid) -> Option<String> {
    let base_type_oid = base_type_oid(type_oid);
    // SAFETY: PostgreSQL returns a palloc'd NUL-terminated string for known
    // type OIDs. The string is copied into Rust-owned memory before pfree.
    unsafe {
        let formatted = pg_sys::format_type_be(base_type_oid);
        if formatted.is_null() {
            return None;
        }
        let name = CStr::from_ptr(formatted).to_string_lossy().into_owned();
        pg_sys::pfree(formatted.cast());
        Some(name)
    }
}
