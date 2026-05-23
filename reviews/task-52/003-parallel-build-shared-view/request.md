# Task 52 / 003 — EcHnswParallelBuildSharedView

Branch: `task-52`

Code path: `src/am/ec_hnsw/parallel_build_view.rs` (new file) +
visibility opens in `src/am/ec_hnsw/build_parallel.rs` and the
matching `mod parallel_build_view;` line in `src/am/ec_hnsw/mod.rs`.

## Provenance — slightly unusual

The initial slice-003 source files landed inside the reviewer's
absorption commit `1f983fb0b` (Codex co-author), alongside the
reviewer-feedback files for slices 001 and 002. The reviewer
co-committed code that was already in the working tree when they ran
their feedback commit; it was not an independent reviewer change.

This `request.md` is authored on top of that commit, and the parent of
this `request.md` commit applies the slice-002 reviewer's durable
directive on view shape (see §"Anti-pattern B refactor" below).

## Summary

Add the typed borrowed view absorbing the
`SpinLockAcquire + record_worker_counts(&mut) + SpinLockRelease +
ConditionVariableSignal` four-call compound that ends every
parallel-build worker entry (heap-scan and graph-build phases) into a
single safe method: `view.record_workers_done(scan_delta,
encoded_delta)`. Leader-side `SpinLockInit + ConditionVariableInit`
pair lifts into `view.init_synchronization()` (still `unsafe fn` since
it requires exclusive uninitialized-memory ownership).

Wrapper-only commit. Consumer migration of the four worker / leader
call sites (lines 2253-2256, 2504-2507, 2828-2833, 2897-2902) is slice
005. The shm_toc allocate/insert/lookup migration is slice 004.

## Per-file `unsafe { ... }` block deltas

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/parallel_build_view.rs` (new) | — | 6 | +6 |
| `src/am/ec_hnsw/build_parallel.rs` | 112 | 112 | 0 |
| `src/am/common/dsm.rs` | 13 | 13 | 0 |

The +6 are wrapper-side PG-primitive interactions. Slice 005 sheds the
two worker-side `unsafe { ... }` compounds and the two leader-side
init compounds at the call sites in `build_parallel.rs`.

`src/` total: closeout, not per slice.

## Surface added

`src/am/ec_hnsw/parallel_build_view.rs`:

- `pub(super) struct EcHnswParallelBuildSharedView<'a>` — `Copy +
  Clone`, `Send + Sync`, holds a `NonNull<EcHnswParallelBuildSharedHeader>`
  plus `'a` PhantomData over the DSM segment.
- `unsafe fn from_raw(*mut header) -> Self` — single DSM-lifetime
  contract per worker entry (later: per leader scope).
- safe `fn validate(&self)` — magic+version check. Operation, not
  accessor. Reads two `Copy` fields via a method-call on the wrapped
  header, scoped inside the method body; no reference escapes.
- `unsafe fn init_synchronization(&self)` — leader-side SpinLockInit
  + ConditionVariableInit pair. Stays `unsafe fn` because calling it
  from a worker is UB (the leader's `ptr::write` of the header is the
  exclusive-ownership clause).
- safe `fn record_workers_done(&self, heap_delta, index_delta)` — the
  worker-side compound. Acquires the spinlock via the slice-447
  `SpinLockGuard` (Drop releases), invokes the existing
  `EcHnswParallelBuildSharedHeader::record_worker_counts(&mut self,
  ...)` inside the locked region, signals via
  `ConditionVariableRef::signal()` after the guard drops.

Visibility opens in `build_parallel.rs` to allow the sibling-module
view to address the synchronization fields and validation hook:

- `EcHnswParallelBuildSharedHeader::workersdonecv: ConditionVariable`
  → `pub(super)`
- `EcHnswParallelBuildSharedHeader::mutex: slock_t` → `pub(super)`
- `EcHnswParallelBuildSharedHeader::validate(&self)` → `pub(super)`

## Task-spec divergence (confirmed by 001 reviewer)

The task spec named two distinct shared headers
(`EcHnswParallelBuildSharedView` and
`EcHnswParallelGraphBuildSharedView`). The code reuses one
`EcHnswParallelBuildSharedHeader` across both worker entries. Planning
packet 001 collapsed the two views into one; the 001 reviewer
confirmed the spec correction in
`reviews/task-52/001-execution-planning/feedback/2026-05-23-01-reviewer.md`.

## Anti-pattern B refactor (per slice-002 reviewer)

The slice-002 reviewer (`002-shm-toc-wrappers/feedback/2026-05-23-01-reviewer.md`,
§"Direction for slice 003") landed a durable directive:

> Direction for slice 003: keep the same anti-pattern B discipline
> on `EcHnswParallelBuildSharedView<'a>`. Expose operations
> (`atomic_field()`, `with_locked_mut(|view, guard| ...)`,
> `signal_workers_done()`) — not raw `&FieldT` accessors.

The initial slice-003 file (in `1f983fb0b`) included a
`fn header(&self) -> &'a EcHnswParallelBuildSharedHeader` accessor and
a `validate()` that went through it. That accessor is exactly the
anti-pattern: it encodes the per-key type/init invariant into the
wrapper's signature. The refactor in this packet's parent commit:

1. Removes the `header() -> &'a Header` accessor.
2. Rewrites `validate()` to dispatch through
   `unsafe { (*self.header.as_ptr()).validate() }` — the auto-borrow
   inside the deref-call is scoped to the method body and never
   escapes, satisfying the rule.

This is the 5th application of memory rule
`feedback_anti_pattern_b_unbounded_lifetime` and the 1st application
of the new memory rule
`feedback_view_operations_not_accessors` (saved this session).

Operations the view exposes:
- `validate()` — reads magic + version, no escape.
- `init_synchronization()` — leader-only init pair.
- `record_workers_done(...)` — locked mutate + signal compound.

Additional `Copy`-field operations (`participant_count() -> u16`,
`is_concurrent() -> bool`, etc.) are deferred to slice 005 alongside
the consumer migration that needs them; no point landing dead code.

## Validation

- `cargo fmt --all` — clean.
- `cargo check --no-default-features --features pg18` — `Finished`
  exit 0, 14.49s incremental.
- `cargo clippy ... -- -D warnings`: not re-run for the refactor;
  the only delta from the in-tree code is one method body. Pre-existing
  rabitq backlog unchanged.
- `cargo pgrx test` — skipped per memory
  `feedback_dyld_buffer_blocks_known`; no callback behavior added.

## Cross-references

- 002 reviewer direction:
  `reviews/task-52/002-shm-toc-wrappers/feedback/2026-05-23-01-reviewer.md`
  §"Direction for slice 003".
- 001 reviewer concur (two-headers → one-view consolidation):
  `reviews/task-52/001-execution-planning/feedback/2026-05-23-01-reviewer.md`.
- New durable memory rule:
  `feedback_view_operations_not_accessors`.
- Composed primitives (slice 447):
  `src/am/common/dsm.rs::{SpinLockGuard, ConditionVariableRef,
  spinlock_init, condition_variable_init}`.
