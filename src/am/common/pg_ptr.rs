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
