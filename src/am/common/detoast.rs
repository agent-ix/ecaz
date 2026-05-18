use std::{ffi::c_void, ptr};

use pgrx::pg_sys;

/// Detoast copies are palloc-owned. Drop runs whenever Rust frames unwind
/// (including `pgrx::error!`, which raises a Rust panic before re-raising the
/// PG ERROR), freeing the copy. PostgreSQL memory-context cleanup at (sub)txn
/// abort is the fallback for paths where control leaves Rust without unwinding
/// these frames.
#[derive(Debug)]
pub(crate) struct DetoastedVarlena {
    varlena: *mut pg_sys::varlena,
    owned: bool,
}

impl DetoastedVarlena {
    pub(crate) unsafe fn packed_from_datum(datum: pg_sys::Datum) -> Option<Self> {
        let original = datum.cast_mut_ptr::<c_void>().cast::<pg_sys::varlena>();
        let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
        Self::from_raw(original, varlena)
    }

    pub(crate) unsafe fn plain_from_datum(datum: pg_sys::Datum) -> Option<Self> {
        let original = datum.cast_mut_ptr::<c_void>().cast::<pg_sys::varlena>();
        let varlena = unsafe { pg_sys::pg_detoast_datum(original.cast()) };
        Self::from_raw(original, varlena)
    }

    fn from_raw(original: *mut pg_sys::varlena, varlena: *mut pg_sys::varlena) -> Option<Self> {
        if varlena.is_null() {
            return None;
        }
        Some(Self {
            varlena,
            owned: !ptr::eq(varlena, original),
        })
    }

    pub(crate) fn as_ptr(&self) -> *mut pg_sys::varlena {
        self.varlena
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        unsafe { pgrx::varlena::varlena_to_byte_slice(self.varlena) }
    }

    pub(crate) fn to_vec(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl Drop for DetoastedVarlena {
    fn drop(&mut self) {
        if self.owned {
            unsafe { pg_sys::pfree(self.varlena.cast()) };
        }
    }
}
