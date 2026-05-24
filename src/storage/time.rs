//! PostgreSQL backend time helpers.

use pgrx::pg_sys;

pub(crate) fn current_timestamp_micros() -> i64 {
    // SAFETY: reads PostgreSQL backend-local current timestamp state and
    // returns it by value; no pointer or backend-owned memory is retained.
    unsafe { pg_sys::GetCurrentTimestamp() }
}
