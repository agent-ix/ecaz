//! PostgreSQL pointer view helpers shared by planner/callback code.

use std::ptr::NonNull;

use pgrx::{pg_sys, PgList};

use crate::storage::page::ItemPointer;

pub(crate) unsafe fn pg_list<T>(list: *mut pg_sys::List) -> PgList<T> {
    // SAFETY: caller guarantees the PostgreSQL-owned list is live for the
    // immediate callback scope in which the returned view is consumed.
    unsafe { PgList::<T>::from_pg(list) }
}

pub(crate) unsafe fn pg_ref<'a, T>(ptr: *mut T) -> Option<&'a T> {
    // SAFETY: caller guarantees the PostgreSQL-owned pointer is live for the
    // immediate callback scope and the referenced fields are copied/inspected
    // without retaining the borrow across callbacks.
    unsafe { ptr.as_ref() }
}

pub(crate) fn index_info<'a>(index_info: NonNull<pg_sys::IndexInfo>) -> &'a pg_sys::IndexInfo {
    // SAFETY: `NonNull` proves the callback-owned IndexInfo exists for this
    // immediate borrow; callers only inspect/copy fields during the callback.
    unsafe { index_info.as_ref() }
}

pub(crate) fn item_pointer(tid: NonNull<pg_sys::ItemPointerData>) -> ItemPointer {
    // SAFETY: `NonNull` proves the callback-owned ItemPointerData exists for
    // this immediate copy; no PostgreSQL-owned pointer is retained.
    let (block_number, offset_number) =
        pgrx::itemptr::item_pointer_get_both(unsafe { *tid.as_ptr() });
    ItemPointer {
        block_number,
        offset_number,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DatumArrayView {
    values: NonNull<pg_sys::Datum>,
    isnull: NonNull<bool>,
}

impl DatumArrayView {
    pub(crate) fn new(values: NonNull<pg_sys::Datum>, isnull: NonNull<bool>) -> Self {
        Self { values, isnull }
    }

    pub(crate) fn non_null_datum(self, offset: usize, context: &str, label: &str) -> pg_sys::Datum {
        // SAFETY: this view is created from PostgreSQL callback value/null
        // arrays, and callers pass offsets resolved from the same callback's
        // index tuple layout.
        let (is_null, datum) = unsafe {
            (
                *self.isnull.as_ptr().add(offset),
                *self.values.as_ptr().add(offset),
            )
        };
        if is_null {
            pgrx::error!("{context} {label} must not be NULL");
        }
        if datum.is_null() {
            pgrx::error!("{context} received a null {label} datum");
        }
        datum
    }
}
