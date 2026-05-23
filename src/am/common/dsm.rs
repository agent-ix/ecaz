//! Typed wrappers for PostgreSQL DSM atomics, SpinLocks, and condition
//! variables consumed from HNSW (and future AM) parallel-build code.
//!
//! Program P8 of `reviews/task-50/030-comprehensive-unsafe-burndown-plan`:
//! "DSM, Atomics, Shared Memory, And Lock Contracts" — typed shared-memory
//! layouts where each field wrapper names its memory-ordering and lock
//! invariant.
//!
//! The wrappers carry an explicit `'a` lifetime tied to the DSM segment the
//! caller knows is live. The construction site is `unsafe` (it asserts the
//! pointer is part of a live segment held for that lifetime); the methods
//! on the wrapper are safe and encapsulate the underlying PG primitive.

use pgrx::pg_sys;

/// A borrowed view over a PostgreSQL `pg_atomic_uint32` field.
///
/// The wrapper is a borrow tied to lifetime `'a`. Callers obtain it through
/// `unsafe { PgAtomicU32Ref::from_raw(ptr) }` after asserting that `ptr`
/// points at a live atomic field whose backing DSM segment outlives `'a`.
/// All read/write/CAS operations on the resulting view are safe and use
/// PostgreSQL's acquire/release semantics.
#[derive(Debug, Copy, Clone)]
pub(crate) struct PgAtomicU32Ref<'a> {
    ptr: *mut pg_sys::pg_atomic_uint32,
    _marker: std::marker::PhantomData<&'a pg_sys::pg_atomic_uint32>,
}

// SAFETY: PG atomics are shareable across threads by definition; the wrapper
// only exposes operations that go through PostgreSQL's atomic primitives.
unsafe impl Send for PgAtomicU32Ref<'_> {}
unsafe impl Sync for PgAtomicU32Ref<'_> {}

impl<'a> PgAtomicU32Ref<'a> {
    /// Construct a borrowed view over a live PG atomic field.
    ///
    /// # Safety
    /// `ptr` must point at a `pg_atomic_uint32` whose backing DSM segment
    /// outlives `'a` and has been initialized by the segment owner.
    pub(crate) unsafe fn from_raw(ptr: *mut pg_sys::pg_atomic_uint32) -> Self {
        Self {
            ptr,
            _marker: std::marker::PhantomData,
        }
    }

    /// Acquire-load the current value.
    pub(crate) fn load_acquire(&self) -> u32 {
        // SAFETY: `self.ptr` was registered by the wrapper's `unsafe`
        // constructor as part of a live DSM segment held for `'a`.
        unsafe { pg_sys::pg_atomic_read_u32(self.ptr) }
    }

    /// Release-store a new value, publishing through PG's membarrier.
    pub(crate) fn store_release(&self, value: u32) {
        // SAFETY: see `from_raw` — the backing field is live.
        unsafe { pg_sys::pg_atomic_write_membarrier_u32(self.ptr, value) }
    }

    /// Compare-exchange with acquire/release semantics. Returns true if the
    /// value was successfully swapped from `current` to `new`.
    pub(crate) fn compare_exchange_acqrel_acquire(&self, current: u32, new: u32) -> bool {
        let mut expected = current;
        // SAFETY: see `from_raw`.
        unsafe { pg_sys::pg_atomic_compare_exchange_u32(self.ptr, &mut expected, new) }
    }
}

/// RAII guard for a PostgreSQL `slock_t` (SpinLock).
///
/// Construction acquires the spinlock; Drop releases it. The lifetime ties
/// the guard to a borrow of the protected DSM field so the spinlock cannot
/// outlive its backing memory.
pub(crate) struct SpinLockGuard<'a> {
    mutex: *mut pg_sys::slock_t,
    _marker: std::marker::PhantomData<&'a mut pg_sys::slock_t>,
}

impl<'a> SpinLockGuard<'a> {
    /// Acquire the spinlock at `mutex`.
    ///
    /// # Safety
    /// `mutex` must point at a `slock_t` that has been initialized via
    /// `SpinLockInit` (typically inside [`spinlock_init`]) and whose
    /// backing memory outlives `'a`.
    pub(crate) unsafe fn acquire(mutex: *mut pg_sys::slock_t) -> Self {
        // SAFETY: caller asserts the spinlock is initialized and live.
        unsafe { pg_sys::SpinLockAcquire(mutex) };
        Self {
            mutex,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Drop for SpinLockGuard<'_> {
    fn drop(&mut self) {
        // SAFETY: `self.mutex` was acquired through `SpinLockAcquire` in
        // `acquire`; it remains live for the borrow lifetime by the
        // constructor's safety contract.
        unsafe { pg_sys::SpinLockRelease(self.mutex) }
    }
}

/// Initialize an embedded `slock_t` field in place.
///
/// # Safety
/// `mutex` must point at uninitialized memory of size/alignment equal to
/// `slock_t`, exclusively owned by the caller for the duration of this call.
pub(crate) unsafe fn spinlock_init(mutex: *mut pg_sys::slock_t) {
    // SAFETY: caller asserts exclusive ownership of uninitialized memory.
    unsafe { pg_sys::SpinLockInit(mutex) }
}

/// Initialize an embedded `ConditionVariable` field in place.
///
/// # Safety
/// `cv` must point at uninitialized memory of size/alignment equal to
/// `ConditionVariable`, exclusively owned by the caller for the duration
/// of this call.
pub(crate) unsafe fn condition_variable_init(cv: *mut pg_sys::ConditionVariable) {
    // SAFETY: caller asserts exclusive ownership of uninitialized memory.
    unsafe { pg_sys::ConditionVariableInit(cv) }
}

/// A borrowed view over a PostgreSQL `ConditionVariable` field.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ConditionVariableRef<'a> {
    ptr: *mut pg_sys::ConditionVariable,
    _marker: std::marker::PhantomData<&'a pg_sys::ConditionVariable>,
}

// SAFETY: PG condition variables are shareable across worker boundaries.
unsafe impl Send for ConditionVariableRef<'_> {}
unsafe impl Sync for ConditionVariableRef<'_> {}

impl<'a> ConditionVariableRef<'a> {
    /// Construct a borrowed view over a live PG condition variable.
    ///
    /// # Safety
    /// `ptr` must point at a `ConditionVariable` whose backing memory
    /// outlives `'a` and has been initialized via
    /// [`condition_variable_init`].
    pub(crate) unsafe fn from_raw(ptr: *mut pg_sys::ConditionVariable) -> Self {
        Self {
            ptr,
            _marker: std::marker::PhantomData,
        }
    }

    /// Signal one waiter on this condition variable.
    pub(crate) fn signal(&self) {
        // SAFETY: see `from_raw`.
        unsafe { pg_sys::ConditionVariableSignal(self.ptr) }
    }
}
