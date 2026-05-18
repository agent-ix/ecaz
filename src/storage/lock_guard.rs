//! PostgreSQL lightweight-lock ownership guards.

use pgrx::pg_sys;

pub(crate) struct LwLockGuard {
    lock: *mut pg_sys::LWLock,
    release: unsafe fn(*mut pg_sys::LWLock),
}

impl LwLockGuard {
    pub(crate) unsafe fn acquire_shared(lock: *mut pg_sys::LWLock) -> Self {
        // SAFETY: caller supplies a live PostgreSQL LWLock pointer.
        unsafe { pg_sys::LWLockAcquire(lock, pg_sys::LWLockMode::LW_SHARED) };
        Self::from_acquired(lock)
    }

    pub(crate) unsafe fn acquire_exclusive(lock: *mut pg_sys::LWLock) -> Self {
        // SAFETY: caller supplies a live PostgreSQL LWLock pointer.
        unsafe { pg_sys::LWLockAcquire(lock, pg_sys::LWLockMode::LW_EXCLUSIVE) };
        Self::from_acquired(lock)
    }

    pub(crate) unsafe fn from_acquired(lock: *mut pg_sys::LWLock) -> Self {
        // SAFETY: caller has already acquired `lock`; this guard owns the
        // matching `LWLockRelease`.
        Self {
            lock,
            release: lwlock_release,
        }
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn from_acquired_with_release(
        lock: *mut pg_sys::LWLock,
        release: unsafe fn(*mut pg_sys::LWLock),
    ) -> Self {
        // SAFETY: caller has already acquired `lock`; this guard owns the
        // supplied release function. This supports callback surfaces that
        // inject no-op test locks while using the same RAII shape.
        Self { lock, release }
    }
}

impl Drop for LwLockGuard {
    fn drop(&mut self) {
        // SAFETY: this guard owns a previously acquired LWLock and the matching
        // release function.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { (self.release)(self.lock) };
    }
}

unsafe fn lwlock_release(lock: *mut pg_sys::LWLock) {
    // SAFETY: caller supplies a lock that was acquired by an `LwLockGuard`
    // constructor or adopted into an `LwLockGuard`.
    unsafe { pg_sys::LWLockRelease(lock) };
}
