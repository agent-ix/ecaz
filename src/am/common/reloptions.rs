//! Shared helpers for PostgreSQL relation option blobs.

use std::ffi::CStr;

use pgrx::pg_sys;

pub(crate) unsafe fn read_string_reloption(
    rd_options: *mut pg_sys::varlena,
    offset: i32,
    am_name: &str,
    name: &str,
) -> Option<String> {
    if offset == 0 {
        return None;
    }

    // SAFETY: caller guarantees `rd_options` points at the AM's reloptions
    // allocation and `offset` is a string reloption offset produced by
    // PostgreSQL's reloptions parser for that layout.
    let value_ptr = unsafe {
        rd_options
            .cast::<u8>()
            .add(offset as usize)
            .cast::<std::ffi::c_char>()
    };
    // SAFETY: string reloptions are stored as NUL-terminated strings inside
    // the reloptions blob at the validated offset.
    let value = unsafe { CStr::from_ptr(value_ptr) }
        .to_str()
        .unwrap_or_else(|e| pgrx::error!("invalid {am_name} {name} reloption: {e}"));
    if value.is_empty() {
        pgrx::error!("invalid {am_name} {name} reloption: value must not be empty");
    }
    Some(value.to_owned())
}
