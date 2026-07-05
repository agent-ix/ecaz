use std::ptr::NonNull;

use pgrx::pg_sys;

#[derive(Debug)]
pub(crate) struct SpiTupleTableGuard {
    tuptable: NonNull<pg_sys::SPITupleTable>,
}

impl SpiTupleTableGuard {
    #[allow(dead_code)]
    pub(crate) unsafe fn from_owned(tuptable: *mut pg_sys::SPITupleTable) -> Option<Self> {
        NonNull::new(tuptable).map(|tuptable| Self { tuptable })
    }

    #[allow(dead_code)]
    pub(crate) fn as_ptr(&self) -> *mut pg_sys::SPITupleTable {
        self.tuptable.as_ptr()
    }
}

impl Drop for SpiTupleTableGuard {
    fn drop(&mut self) {
        // SAFETY: this guard adopts one owned SPI tuptable pointer and releases it exactly once.
        // pgrx ERROR paths must unwind Rust frames so Drop runs; re-audit on pgrx bumps.
        unsafe { pg_sys::SPI_freetuptable(self.tuptable.as_ptr()) };
    }
}
