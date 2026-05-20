#[inline]
pub(crate) fn am_callback<R>(callback: impl FnOnce() -> R) -> R {
    // SAFETY: callers use this only at PostgreSQL AM callback entry points.
    // `pgrx_extern_c_guard` is the required FFI boundary guard that prevents
    // Rust unwinds from crossing back into PostgreSQL C code.
    unsafe { pgrx::pgrx_extern_c_guard(callback) }
}
