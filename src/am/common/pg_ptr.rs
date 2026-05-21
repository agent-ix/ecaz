//! PostgreSQL pointer view helpers shared by planner/callback code.

use pgrx::{pg_sys, PgList};

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
