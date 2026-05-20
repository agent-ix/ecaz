#[inline]
pub(crate) fn am_callback<R>(callback: impl FnOnce() -> R) -> R {
    // SAFETY: callers use this only at PostgreSQL AM callback entry points.
    // `pgrx_extern_c_guard` is the required FFI boundary guard that prevents
    // Rust unwinds from crossing back into PostgreSQL C code.
    unsafe { pgrx::pgrx_extern_c_guard(callback) }
}

macro_rules! pg_am_callback {
    ($body:block) => {{
        // SAFETY: call sites use this macro only at PostgreSQL AM callback
        // entry points. PostgreSQL-owned pointers captured by `$body` remain
        // valid for the duration of the guarded callback invocation, and
        // `pgrx_extern_c_guard` prevents Rust unwinds from crossing the C ABI.
        unsafe { pgrx::pgrx_extern_c_guard(|| $body) }
    }};
}

pub(crate) use pg_am_callback;
