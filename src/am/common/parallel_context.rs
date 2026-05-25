//! Typed wrappers for PostgreSQL's `ParallelContext` lifecycle.
//!
//! The PG parallel-build leader scope opens with `EnterParallelMode`,
//! creates a `ParallelContext`, runs through `InitializeParallelDSM`,
//! `LaunchParallelWorkers`, `WaitForParallelWorkersToAttach`, ... waits
//! for finish, destroys the context, and exits parallel mode. Each of
//! these PG primitives is a single-op FFI call that historically lived
//! in its own `unsafe { ... }` block at the consumer site.
//!
//! This module exposes safe wrappers that collapse N consumer-side
//! `unsafe { ... }` blocks into one wrapper-side `unsafe { ... }` per
//! primitive. The consumer states the DSM-segment / parallel-mode
//! lifetime contract once at `ParallelContextRef::new` instead of at
//! every call site.
//!
//! Theme: P1 callback-shell-adjacent — same separation-of-concerns
//! pattern as `crate::am::common::callback` macros for the AM
//! callback boundary.

use pgrx::pg_sys;

/// A borrowed handle over a PostgreSQL `ParallelContext`.
///
/// The handle is `Copy + Clone` and carries an `'a` lifetime tied to
/// the parallel-mode scope. The constructor is `unsafe` so the
/// "PG ParallelContext outlives `'a`" contract is stated once per
/// leader scope; the operations on the handle are safe and route
/// through PG's parallel-context API.
#[derive(Debug, Copy, Clone)]
pub(crate) struct ParallelContextRef<'a> {
    pcxt: *mut pg_sys::ParallelContext,
    _marker: std::marker::PhantomData<&'a mut pg_sys::ParallelContext>,
}

impl<'a> ParallelContextRef<'a> {
    /// Construct a borrowed handle over the leader's parallel context.
    ///
    /// # Safety
    /// `pcxt` must point at a `ParallelContext` returned by
    /// `pg_sys::CreateParallelContext` and not yet destroyed; its
    /// backing memory must outlive `'a`.
    pub(crate) unsafe fn new(pcxt: *mut pg_sys::ParallelContext) -> Self {
        Self {
            pcxt,
            _marker: std::marker::PhantomData,
        }
    }

    /// Raw pointer for callers that still need it (e.g., passing into
    /// PG primitives this wrapper hasn't typed yet).
    pub(crate) fn as_ptr(&self) -> *mut pg_sys::ParallelContext {
        self.pcxt
    }

    /// Read `(*pcxt).toc` as a Copy raw pointer value.
    pub(crate) fn toc(&self) -> *mut pg_sys::shm_toc {
        // SAFETY: see `new`. Field read returns a `Copy` raw-pointer
        // value; no Rust reference to ParallelContext escapes.
        unsafe { (*self.pcxt).toc }
    }

    /// Read `(*pcxt).seg` as a Copy raw pointer value.
    pub(crate) fn seg(&self) -> *mut pg_sys::dsm_segment {
        // SAFETY: see `new`. Copy raw-pointer read; no reference escapes.
        unsafe { (*self.pcxt).seg }
    }

    /// Read `(*pcxt).nworkers_launched` (set by `LaunchParallelWorkers`).
    pub(crate) fn nworkers_launched(&self) -> i32 {
        // SAFETY: see `new`. Copy value read; no reference escapes.
        unsafe { (*self.pcxt).nworkers_launched }
    }

    /// Mutable raw pointer to `(*pcxt).estimator` for code paths that
    /// thread the estimator through `pg_sys::shm_toc_estimate_chunk` /
    /// `shm_toc_estimate_keys` macros and need a `*mut` directly.
    ///
    /// Returns a Copy raw pointer; no Rust reference escapes.
    pub(crate) fn estimator_mut(&self) -> *mut pg_sys::shm_toc_estimator {
        // SAFETY: see `new`. `addr_of_mut!` on a live struct field.
        unsafe { std::ptr::addr_of_mut!((*self.pcxt).estimator) }
    }

    /// Pointer into the parallel-worker metadata array. The caller is
    /// responsible for indexing within `[0, nworkers_launched)`.
    pub(crate) fn worker(&self, index: usize) -> *mut pg_sys::ParallelWorkerInfo {
        // SAFETY: see `new`. Caller ensures `index < nworkers_launched`.
        unsafe { (*self.pcxt).worker.add(index) }
    }

    /// Initialize the DSM segment + TOC. Wraps `pg_sys::InitializeParallelDSM`.
    pub(crate) fn initialize_dsm(&self) {
        // SAFETY: see `new` — `pcxt` is a live parallel context. PG
        // populates `(*pcxt).seg` and `(*pcxt).toc`.
        unsafe { pg_sys::InitializeParallelDSM(self.pcxt) }
    }

    /// Launch the parallel workers. Wraps `pg_sys::LaunchParallelWorkers`.
    /// PG fills `(*pcxt).nworkers_launched` with the actual count.
    pub(crate) fn launch_workers(&self) {
        // SAFETY: see `new`.
        unsafe { pg_sys::LaunchParallelWorkers(self.pcxt) }
    }

    /// Block until all launched workers have attached to the DSM
    /// segment. Wraps `pg_sys::WaitForParallelWorkersToAttach`.
    pub(crate) fn wait_for_workers_to_attach(&self) {
        // SAFETY: see `new`.
        unsafe { pg_sys::WaitForParallelWorkersToAttach(self.pcxt) }
    }

    /// Block until all workers exit. Wraps
    /// `pg_sys::WaitForParallelWorkersToFinish`.
    pub(crate) fn wait_for_workers_to_finish(&self) {
        // SAFETY: see `new`.
        unsafe { pg_sys::WaitForParallelWorkersToFinish(self.pcxt) }
    }

    /// Destroy the parallel context (releases DSM segment, worker
    /// records). Wraps `pg_sys::DestroyParallelContext`. The
    /// `ParallelContextRef` is unusable afterwards — caller is
    /// responsible for not reusing it.
    pub(crate) fn destroy(self) {
        // SAFETY: see `new`. After this call the `pcxt` pointer is
        // dead; the wrapper is consumed by-value here so accidental
        // reuse is prevented at the type-system level.
        unsafe { pg_sys::DestroyParallelContext(self.pcxt) }
    }
}

/// Enter parallel mode. Wraps `pg_sys::EnterParallelMode`.
pub(crate) fn enter_parallel_mode() {
    // SAFETY: `EnterParallelMode` is a no-arg PG global toggle. The
    // caller is responsible for pairing with `exit_parallel_mode`.
    unsafe { pg_sys::EnterParallelMode() }
}

/// Exit parallel mode. Wraps `pg_sys::ExitParallelMode`.
pub(crate) fn exit_parallel_mode() {
    // SAFETY: pairs with `enter_parallel_mode`.
    unsafe { pg_sys::ExitParallelMode() }
}

/// Begin parallel-query instrumentation for this worker. Wraps
/// `pg_sys::InstrStartParallelQuery`.
pub(crate) fn instr_start_parallel_query() {
    // SAFETY: PG requires this be called inside a parallel worker
    // scope. The caller is the worker entrypoint that's running under
    // PostgreSQL's parallel-query instrumentation regime.
    unsafe { pg_sys::InstrStartParallelQuery() }
}

/// Read `(*index_info).ii_ParallelWorkers`, treating null as 0.
pub(crate) fn index_info_parallel_workers(index_info: *mut pg_sys::IndexInfo) -> i32 {
    if index_info.is_null() {
        return 0;
    }
    // SAFETY: Non-null pointer is supplied by PostgreSQL build setup
    // and remains live while planning the build. Copy field read; no
    // reference escapes.
    unsafe { (*index_info).ii_ParallelWorkers }
}

/// Read `(*index_info).ii_Concurrent`, treating null as `false`.
pub(crate) fn index_info_is_concurrent(index_info: *mut pg_sys::IndexInfo) -> bool {
    if index_info.is_null() {
        return false;
    }
    // SAFETY: Non-null pointer is supplied by PostgreSQL build setup.
    // Copy field read; no reference escapes.
    unsafe { (*index_info).ii_Concurrent }
}

/// Designate `proc` as the sender end of `mq`. Wraps
/// `pg_sys::shm_mq_set_sender`.
///
/// Convention matches the safe FFI wrappers in
/// [`crate::am::common::dsm`]: the caller is expected to pass live PG
/// resource pointers (established by the surrounding parallel-build
/// scope); the wrapper only forwards to PG's primitive.
pub(crate) fn shm_mq_set_sender(mq: *mut pg_sys::shm_mq, proc: *mut pg_sys::PGPROC) {
    // SAFETY: callers pass `mq` produced by `shm_mq_create` and `proc`
    // is `MyProc` (set by PostgreSQL on backend init).
    unsafe { pg_sys::shm_mq_set_sender(mq, proc) }
}

/// Estimate the workspace bytes for a parallel heap scan. Wraps
/// `pg_sys::table_parallelscan_estimate`.
pub(crate) fn table_parallelscan_estimate(
    heap_relation: pg_sys::Relation,
    snapshot: pg_sys::Snapshot,
) -> pg_sys::Size {
    // SAFETY: callers pass an open heap relation and a valid snapshot
    // pointer; the AM build coordinator establishes both.
    unsafe { pg_sys::table_parallelscan_estimate(heap_relation, snapshot) }
}
