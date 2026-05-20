use pgrx::{pg_sys, IntoDatum};

use crate::storage::page::ItemPointer;

pub(crate) fn set_scan_heap_tid(scan: pg_sys::IndexScanDesc, heap_tid: ItemPointer) {
    if scan.is_null() {
        pgrx::error!("index scan descriptor is null while setting heap TID");
    }
    // SAFETY: caller passes a live IndexScanDesc from an AM callback;
    // xs_heaptid is PostgreSQL-owned output storage for the current tuple.
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut (*scan).xs_heaptid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
    }
}

pub(crate) fn set_scan_orderby_score(
    scan: pg_sys::IndexScanDesc,
    score: f32,
    values_context: &str,
    nulls_context: &str,
) {
    if scan.is_null() {
        pgrx::error!("index scan descriptor is null while setting order-by score");
    }
    // SAFETY: caller passes a live IndexScanDesc with one order-by output slot.
    // The value/null arrays are allocated in PostgreSQL memory on first write.
    unsafe {
        if (*scan).xs_orderbyvals.is_null() {
            crate::fault::maybe_fail_palloc(values_context);
            (*scan).xs_orderbyvals =
                pg_sys::palloc0(std::mem::size_of::<pg_sys::Datum>()).cast::<pg_sys::Datum>();
        }
        if (*scan).xs_orderbynulls.is_null() {
            crate::fault::maybe_fail_palloc(nulls_context);
            (*scan).xs_orderbynulls = pg_sys::palloc0(std::mem::size_of::<bool>()).cast::<bool>();
        }

        *(*scan).xs_orderbyvals = score.into_datum().expect("score should convert to datum");
        *(*scan).xs_orderbynulls = false;
    }
}

pub(crate) fn clear_scan_orderby_output(scan: pg_sys::IndexScanDesc) {
    if scan.is_null() {
        pgrx::error!("index scan descriptor is null while clearing order-by output");
    }
    // SAFETY: caller passes a live IndexScanDesc; if the nulls array has been
    // allocated, setting the first flag clears the single ORDER BY output.
    unsafe {
        if !(*scan).xs_orderbynulls.is_null() {
            *(*scan).xs_orderbynulls = true;
        }
    }
}
