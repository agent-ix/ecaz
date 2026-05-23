//! Typed borrowed view over the parallel-build shared header.
//!
//! Task 52 §Scope #1: absorb the `SpinLockAcquire + (*shared).mutate +
//! SpinLockRelease + ConditionVariableSignal` compound that ends every
//! parallel worker entry (heap-scan phase and graph-build phase) into a
//! single safe method. Composed from the generic primitives in
//! [`crate::am::common::dsm`].
//!
//! The wrapper is HNSW-specific (it holds the HNSW
//! `EcHnswParallelBuildSharedHeader`) and therefore lives next to its
//! sole owner module `build_parallel.rs`, not under `src/am/common/`.
//! The common module provides the parts; this module is the
//! HNSW-specific composition.
//!
//! Lifetime contract: the view is `Copy` and carries an `'a` lifetime
//! tied to the DSM segment the header lives in. The constructor is
//! `unsafe` so the DSM-lifetime contract is stated at one site per
//! worker entry; all methods on the resulting view are safe.

use std::ptr::NonNull;

use super::build_parallel::EcHnswParallelBuildSharedHeader;
use crate::am::common::dsm::{
    condition_variable_init, spinlock_init, ConditionVariableRef, SpinLockGuard,
};

/// Borrowed view over an [`EcHnswParallelBuildSharedHeader`] living in a
/// DSM segment shared with parallel workers.
///
/// The view is held for the duration of a worker (or leader) frame and
/// exposes safe accessors over the underlying PG SpinLock + Condition
/// Variable + per-phase counters. Both the heap-scan worker entry and
/// the graph-build worker entry consume this single type.
#[derive(Debug, Copy, Clone)]
pub(super) struct EcHnswParallelBuildSharedView<'a> {
    header: NonNull<EcHnswParallelBuildSharedHeader>,
    _marker: std::marker::PhantomData<&'a EcHnswParallelBuildSharedHeader>,
}

// SAFETY: the underlying DSM-laid-out header is engineered to be shared
// across worker boundaries via PG's SpinLock / ConditionVariable. The
// wrapper only exposes operations that route through those primitives.
unsafe impl Send for EcHnswParallelBuildSharedView<'_> {}
unsafe impl Sync for EcHnswParallelBuildSharedView<'_> {}

impl<'a> EcHnswParallelBuildSharedView<'a> {
    /// Construct a view from a leader-supplied or `shm_toc_lookup`-
    /// returned pointer.
    ///
    /// # Safety
    /// `ptr` must point at an initialized
    /// [`EcHnswParallelBuildSharedHeader`] whose backing DSM segment
    /// outlives `'a`. The pointer must be non-null; pgrx-error on a
    /// missing key is the caller's responsibility (see
    /// [`crate::am::common::dsm::ShmTocReader::lookup_required`]).
    pub(super) unsafe fn from_raw(ptr: *mut EcHnswParallelBuildSharedHeader) -> Self {
        // SAFETY: caller asserts the pointer is non-null and points at
        // a live, initialized header (see function-level contract).
        let header = unsafe { NonNull::new_unchecked(ptr) };
        Self {
            header,
            _marker: std::marker::PhantomData,
        }
    }

    /// Borrow the header read-only.
    pub(super) fn header(&self) -> &'a EcHnswParallelBuildSharedHeader {
        // SAFETY: see `from_raw`'s contract — the header is initialized
        // and outlives `'a`.
        unsafe { self.header.as_ref() }
    }

    /// Mirror of the underlying [`EcHnswParallelBuildSharedHeader::validate`]
    /// for callers that hold only the view.
    pub(super) fn validate(&self) {
        self.header().validate();
    }

    /// Initialize the embedded SpinLock and ConditionVariable.
    ///
    /// **Leader-side only.** Must be called exactly once, before any
    /// worker observes the header. Idempotent for leader callers — the
    /// PG primitives both write a fresh sentinel value into their
    /// backing memory.
    ///
    /// # Safety
    /// `self` must reference a header whose `mutex` and `workersdonecv`
    /// fields are uninitialized memory owned exclusively by the leader
    /// at this call. The leader's `ptr::write` of the new header
    /// satisfies the exclusive-ownership clause; calling this method
    /// from inside a worker is undefined behavior.
    pub(super) unsafe fn init_synchronization(&self) {
        let header_ptr = self.header.as_ptr();
        // SAFETY: see function-level contract — leader holds exclusive
        // access to the freshly-written header.
        unsafe {
            condition_variable_init(std::ptr::addr_of_mut!((*header_ptr).workersdonecv));
            spinlock_init(std::ptr::addr_of_mut!((*header_ptr).mutex));
        }
    }

    /// Publish this worker's contribution to the per-phase counters and
    /// wake the leader.
    ///
    /// Absorbs the four-call compound — `SpinLockAcquire` + counter
    /// mutate + `SpinLockRelease` + `ConditionVariableSignal` — into a
    /// single safe method.
    ///
    /// `heap_tuples_delta` and `index_tuples_delta` are this worker's
    /// scanned/encoded tuple counts; the heap-scan phase passes the
    /// heap scan total and the encoded total respectively, while the
    /// graph-build phase passes 0.0 for the heap delta (no rescan) and
    /// the inserted-node count as the index delta. See
    /// [`EcHnswParallelBuildSharedHeader::record_worker_counts`] for
    /// the accumulator semantics.
    pub(super) fn record_workers_done(&self, heap_tuples_delta: f64, index_tuples_delta: f64) {
        let header_ptr = self.header.as_ptr();
        {
            // SAFETY: `mutex` is part of the header whose DSM segment
            // outlives `'a` (constructor contract). `init_synchronization`
            // is the leader's responsibility before workers run.
            let _guard =
                unsafe { SpinLockGuard::acquire(std::ptr::addr_of_mut!((*header_ptr).mutex)) };
            // SAFETY: while `_guard` is alive we hold the spinlock,
            // which serializes worker writes to the header counters.
            // The DSM segment is alive (`'a`).
            unsafe {
                (*header_ptr).record_worker_counts(heap_tuples_delta, index_tuples_delta);
            }
            // `_guard` drops on scope exit → SpinLockRelease.
        }
        // SAFETY: `workersdonecv` lives in the same header whose DSM
        // segment outlives `'a`. Signalling is independent of the
        // spinlock and only requires the CV's backing memory to be
        // live.
        let cv = unsafe {
            ConditionVariableRef::from_raw(std::ptr::addr_of_mut!((*header_ptr).workersdonecv))
        };
        cv.signal();
    }
}
