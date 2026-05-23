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
        // SAFETY: caller guarantees `datum` is a varlena Datum valid in the
        // current PostgreSQL memory context.
        let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
        Self::from_raw(original, varlena)
    }

    pub(crate) unsafe fn plain_from_datum(datum: pg_sys::Datum) -> Option<Self> {
        let original = datum.cast_mut_ptr::<c_void>().cast::<pg_sys::varlena>();
        // SAFETY: caller guarantees `datum` is a varlena Datum valid in the
        // current PostgreSQL memory context.
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
        // SAFETY: `self.varlena` is non-null and remains owned or borrowed by
        // this wrapper for the returned slice lifetime.
        unsafe { pgrx::varlena::varlena_to_byte_slice(self.varlena) }
    }

    pub(crate) fn to_vec(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    /// Reinterpret the detoasted payload as a `&[T]` slice.
    ///
    /// Returns `None` if the byte length is not a multiple of
    /// `size_of::<T>()` or if the underlying buffer is not aligned for `T`.
    /// `T` must be `Copy` (and therefore safely transmuted from a packed
    /// byte payload); callers retain responsibility for asserting that the
    /// varlena bytes were produced by serializing `T` values.
    pub(crate) fn as_typed_slice<T: Copy>(&self) -> Option<&[T]> {
        let bytes = self.as_bytes();
        if std::mem::size_of::<T>() == 0 {
            return None;
        }
        if bytes.len() % std::mem::size_of::<T>() != 0 {
            return None;
        }
        // SAFETY: `align_to` is used only to validate exact `T` alignment;
        // any non-empty prefix/suffix is rejected so the body covers all
        // bytes.
        let (prefix, body, suffix) = unsafe { bytes.align_to::<T>() };
        if !prefix.is_empty() || !suffix.is_empty() {
            return None;
        }
        Some(body)
    }
}

impl Drop for DetoastedVarlena {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: `owned` is set only when PostgreSQL returned a detoasted
            // copy distinct from the original Datum pointer.
            unsafe { pg_sys::pfree(self.varlena.cast()) };
        }
    }
}
