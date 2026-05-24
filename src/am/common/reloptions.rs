//! Shared helpers for PostgreSQL relation option blobs.

use std::ffi::CStr;
use std::ptr::NonNull;

use pgrx::pg_sys;

#[derive(Clone, Copy)]
pub(crate) struct ReloptionsBlob {
    rd_options: NonNull<pg_sys::varlena>,
}

impl ReloptionsBlob {
    pub(crate) fn new(rd_options: NonNull<pg_sys::varlena>) -> Self {
        Self { rd_options }
    }

    pub(crate) fn handle(&self) -> &NonNull<pg_sys::varlena> {
        &self.rd_options
    }

    pub(crate) fn read_string_reloption(
        &self,
        offset: i32,
        am_name: &str,
        name: &str,
    ) -> Option<String> {
        if offset == 0 {
            return None;
        }

        // SAFETY: this blob is relation-owned reloptions storage for one AM,
        // and callers pass a string reloption offset copied from that same
        // AM's parsed reloptions layout.
        let value_ptr = unsafe {
            self.rd_options
                .as_ptr()
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
}
