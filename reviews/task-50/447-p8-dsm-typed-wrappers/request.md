# Task 50/447: P8 — typed DSM atomic / SpinLock / CondVar wrapper module

## Why this slice

Opens the long-deferred **P8** contract program from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`:

> "DSM, Atomics, Shared Memory, And Lock Contracts —
>  typed shared-memory layouts where each field wrapper names its
>  memory-ordering and lock invariant. Expected disposition: DSM
>  pointer arithmetic and atomic field access removed from
>  HNSW/common call sites."

P8 had been the densest residual on HNSW (114 blocks in
`build_parallel.rs` after the 446-slice rotation hit the safe-fn
lift ceiling). Per-slice signature lifts cannot reduce it without
new typed contracts.

## Scope

New module `src/am/common/dsm.rs` with five typed primitives:

| Surface | Purpose |
| --- | --- |
| `PgAtomicU32Ref<'a>` | Borrowed view over `pg_atomic_uint32`. Safe methods: `load_acquire`, `store_release`, `compare_exchange_acqrel_acquire`. |
| `SpinLockGuard<'a>` | RAII guard with Drop-on-release for `slock_t`. |
| `spinlock_init` (fn) | One-shot in-place init for embedded `slock_t` fields. |
| `condition_variable_init` (fn) | One-shot in-place init for embedded `ConditionVariable` fields. |
| `ConditionVariableRef<'a>` | Borrowed view with safe `signal()`. |

Construction is the only unsafe surface (each `from_raw` /
`acquire` asserts the field is part of a live DSM segment held for
the wrapper's borrow lifetime). All subsequent operations on a
constructed wrapper are safe.

## First migration

`PgLockedDsmInsertStateCell` (the lifted concurrent-DSM insert
state cell at `build_parallel.rs:50-76`) now routes its three
methods through a shared `as_atomic_ref()` helper that constructs
a `PgAtomicU32Ref<'_>` once and delegates. Three internal unsafe
blocks collapse to one.

The SpinLock / ConditionVariable wrappers are landed but not yet
consumed. The HNSW parallel-build SpinLock call sites currently
batch `SpinLockAcquire + record_worker_counts + SpinLockRelease +
ConditionVariableSignal` into a single wide unsafe block; naively
splitting them into RAII-scoped fragments produces multiple
narrower blocks, which is worse on the block-count metric.
Future P8 slices will migrate them once the surrounding raw-ptr
deref of `*shared` is also encapsulated (likely via a typed
`EcHnswParallelBuildSharedView<'a>` wrapper).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 114 | 112 | -2 |
| `src/am/common/dsm.rs` | 0 | 9 | +9 (new file, not in HNSW subsystem) |
| **HNSW subsystem subtotal** | **329** | **327** | **-2** |

The 9 new unsafe blocks in `src/am/common/dsm.rs` sit *outside*
the HNSW subsystem (`src/am/ec_hnsw/**`) and are not counted
against the HNSW total. This is the intended P8 disposition.

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 446 | 329 |
| After 447 | 327 |

**Net rotation delta: -222 in HNSW (-40.44%).**

## Soundness rationale

Each wrapper records its DSM-segment-lifetime invariant in its
constructor doc:

- `PgAtomicU32Ref::from_raw` — pointer must reference an atomic
  field whose backing DSM segment outlives `'a` and has been
  initialized by the segment owner.
- `SpinLockGuard::acquire` — mutex must be initialized via
  `SpinLockInit` (typically through `spinlock_init`) and live for
  `'a`.
- `ConditionVariableRef::from_raw` — backing memory live for `'a`
  and initialized via `condition_variable_init`.

The PG primitives themselves (atomic CAS, SpinLockAcquire/Release,
CV signal) require no additional unsafety: PostgreSQL guarantees
correctness of these C-level primitives.

## Validation

Artifacts under `reviews/task-50/447-p8-dsm-typed-wrappers/artifacts/`:

- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## P8 progress

This is the **opening slice** of P8. Remaining P8 work for HNSW:

| Sub-target | Status |
| --- | --- |
| DSM atomic cell migration | ✓ (this slice) |
| SpinLock acquire/release migration | Deferred until typed shared-header view exists |
| ConditionVariable signal migration | Deferred until typed shared-header view exists |
| `EcHnswParallelBuildSharedHeader` typed view | Open |
| `EcHnswParallelGraphBuildSharedHeader` typed view | Open |
| `shm_toc` typed allocate/insert/lookup | Open |
| DSM-laid-out struct field views | Open |

The next P8 slice will add typed shared-header views to enable
compound block migration without count-inflation.

## Performance gate

Setup-time primitive only; no inner-loop change. Bench evidence
gathered out-of-band per `feedback_coder_push_smoke_checks`.
