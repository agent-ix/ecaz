use std::marker::PhantomData;

use pgrx::{pg_sys, IntoDatum};

use crate::storage::page::ItemPointer;

pub(crate) struct IndexScanOutput<'scan> {
    scan: pg_sys::IndexScanDesc,
    _marker: PhantomData<&'scan mut pg_sys::IndexScanDescData>,
}

impl<'scan> IndexScanOutput<'scan> {
    /// Create an output writer for the live descriptor passed to an AM scan callback.
    ///
    /// # Safety
    ///
    /// `scan` must be the live `IndexScanDesc` for the current callback, and
    /// this view must not outlive that callback.
    pub(crate) unsafe fn from_raw(scan: pg_sys::IndexScanDesc, context: &str) -> Self {
        if scan.is_null() {
            pgrx::error!("{context} requires a non-null index scan descriptor");
        }
        Self {
            scan,
            _marker: PhantomData,
        }
    }

    pub(crate) fn set_heap_tid(&mut self, heap_tid: ItemPointer) {
        // SAFETY: `IndexScanOutput` is constructed only for a live callback
        // descriptor; xs_heaptid is PostgreSQL-owned output storage for the
        // current tuple.
        unsafe {
            pgrx::itemptr::item_pointer_set_all(
                &mut (*self.scan).xs_heaptid,
                heap_tid.block_number,
                heap_tid.offset_number,
            );
        }
    }

    pub(crate) fn set_orderby_score(
        &mut self,
        score: f32,
        values_context: &str,
        nulls_context: &str,
    ) {
        // SAFETY: `IndexScanOutput` is constructed only for a live callback
        // descriptor with one order-by output slot. The value/null arrays are
        // allocated in PostgreSQL memory on first write.
        unsafe {
            if (*self.scan).xs_orderbyvals.is_null() {
                crate::fault::maybe_fail_palloc(values_context);
                (*self.scan).xs_orderbyvals =
                    pg_sys::palloc0(std::mem::size_of::<pg_sys::Datum>()).cast::<pg_sys::Datum>();
            }
            if (*self.scan).xs_orderbynulls.is_null() {
                crate::fault::maybe_fail_palloc(nulls_context);
                (*self.scan).xs_orderbynulls =
                    pg_sys::palloc0(std::mem::size_of::<bool>()).cast::<bool>();
            }

            *(*self.scan).xs_orderbyvals =
                score.into_datum().expect("score should convert to datum");
            *(*self.scan).xs_orderbynulls = false;
        }
    }

    pub(crate) fn clear_orderby_output(&mut self) {
        // SAFETY: `IndexScanOutput` is constructed only for a live callback
        // descriptor; if the nulls array has been allocated, setting the first
        // flag clears the single ORDER BY output.
        unsafe {
            if !(*self.scan).xs_orderbynulls.is_null() {
                *(*self.scan).xs_orderbynulls = true;
            }
        }
    }
}
