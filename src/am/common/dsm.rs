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

/// Leader-side typed wrapper over a PostgreSQL `shm_toc`.
///
/// The leader allocates DSM-backed structures and registers them under
/// integer keys; workers attach and look those keys up later. The wrapper
/// carries an explicit `'a` lifetime tied to the toc's backing DSM
/// segment, so the leader cannot inadvertently emit references to
/// memory it has already released.
///
/// The compound pattern this wrapper absorbs from `build_parallel.rs` is:
///
/// ```text
/// shm_toc_estimate(&mut (*pcxt).estimator);
/// shm_toc_allocate((*pcxt).toc, size)       // -> *mut c_void
/// shm_toc_insert((*pcxt).toc, KEY, ptr)
/// ```
///
/// becoming
///
/// ```text
/// builder.allocate_bytes(size) -> *mut c_void
/// builder.insert(KEY, ptr)
/// ```
///
/// with the unsafe DSM-segment-lifetime contract stated once at the
/// `unsafe fn new` call site, not at every allocate / insert pair.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ShmTocBuilder<'a> {
    toc: *mut pg_sys::shm_toc,
    _marker: std::marker::PhantomData<&'a mut pg_sys::shm_toc>,
}

impl<'a> ShmTocBuilder<'a> {
    /// Construct a builder over a leader-owned toc.
    ///
    /// # Safety
    /// `toc` must point at a `shm_toc` created by `shm_toc_create` (or
    /// supplied by `(*ParallelContext).toc`) whose backing DSM segment
    /// outlives `'a`.
    pub(crate) unsafe fn new(toc: *mut pg_sys::shm_toc) -> Self {
        Self {
            toc,
            _marker: std::marker::PhantomData,
        }
    }

    /// Allocate `nbytes` of typeless storage in the toc's segment.
    ///
    /// Returns a raw pointer; callers cast and initialize. PG ereports on
    /// allocation failure so the returned pointer is non-null.
    pub(crate) fn allocate_bytes(&self, nbytes: pg_sys::Size) -> *mut std::ffi::c_void {
        // SAFETY: `self.toc` is a live toc per `new`'s contract.
        unsafe { pg_sys::shm_toc_allocate(self.toc, nbytes) }
    }

    /// Allocate `nbytes` of typeless storage and re-type the result.
    ///
    /// Convenience over [`allocate_bytes`] for the common
    /// `.cast::<T>()` site. Caller is still responsible for the cast's
    /// validity (size, alignment, initialization of the returned
    /// region).
    pub(crate) fn allocate_typed<T>(&self, nbytes: pg_sys::Size) -> *mut T {
        self.allocate_bytes(nbytes).cast::<T>()
    }

    /// Register `address` under integer `key` in this toc.
    pub(crate) fn insert<T>(&self, key: u64, address: *mut T) {
        // SAFETY: `self.toc` is live; `address` is caller-supplied. PG
        // copies the (key, address) pair into the toc's index.
        unsafe { pg_sys::shm_toc_insert(self.toc, key, address.cast::<std::ffi::c_void>()) }
    }
}

/// Worker-side typed wrapper over an attached `shm_toc`.
///
/// Workers attach to the leader's DSM segment and look up the leader-
/// registered structures by key. The wrapper carries the segment's
/// lifetime so a returned `&'a T` cannot outlive the worker's view of
/// the DSM region.
///
/// `lookup_required` is the safe replacement for the previous local
/// `shm_toc_lookup_required<T>` helper in
/// `src/am/ec_hnsw/build_parallel.rs`: PG ereports on a missing key, so
/// the returned pointer is non-null on successful return.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ShmTocReader<'a> {
    toc: *mut pg_sys::shm_toc,
    _marker: std::marker::PhantomData<&'a pg_sys::shm_toc>,
}

impl<'a> ShmTocReader<'a> {
    /// Construct a reader over an attached toc.
    ///
    /// # Safety
    /// `toc` must point at a `shm_toc` that the caller has either
    /// received from `(*ParallelContext).toc` or attached via
    /// `shm_toc_attach`, and the backing DSM segment must outlive `'a`.
    pub(crate) unsafe fn attach(toc: *mut pg_sys::shm_toc) -> Self {
        Self {
            toc,
            _marker: std::marker::PhantomData,
        }
    }

    /// Look up `key`, returning the raw pointer. Returns null when the
    /// key is absent.
    pub(crate) fn lookup_raw<T>(&self, key: u64) -> *mut T {
        // SAFETY: `self.toc` is live per `attach`'s contract. PG performs
        // the index walk in-process; passing `noerror = true` lets the
        // caller decide what to do with a missing key.
        unsafe { pg_sys::shm_toc_lookup(self.toc, key, true) }.cast::<T>()
    }

    /// Look up `key`. PG ereports on a missing key, so the returned
    /// pointer is non-null on successful return.
    pub(crate) fn lookup_required<T>(&self, key: u64) -> *mut T {
        // SAFETY: `self.toc` is live; `noerror = false` makes PG handle
        // the missing-key case via ereport.
        unsafe { pg_sys::shm_toc_lookup(self.toc, key, false) }.cast::<T>()
    }
}
