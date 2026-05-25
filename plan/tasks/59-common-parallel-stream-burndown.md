# Task 59: Common `parallel.rs` + `stream.rs` Unsafe Burndown

Status: **proposed** — post-Task-50 hardening sequence; addresses the
two largest unaddressed-by-prior-tasks files in `src/am/common/`.

## Why

After Tasks 52 (P8 DSM/Atomic), 53 (P6 Datum), 54 (P3 Page/WAL/Buffer),
the post-Task-50 phased plan focused on per-AM consumer migrations
(Tasks 55 DiskANN, 56 SPIRE, 57 IVF) and HNSW stretch lifts (Task
54/006, Task 58). Two `src/am/common/` files remain at high unsafe-block
density without a dedicated task:

| File | Unsafe blocks | Surface |
| --- | ---: | --- |
| `src/am/common/parallel.rs` | 34 | `EcParallelScanState` / `EcParallelCoordinatorState` raw-pointer field accessors; `*self.coordinator` derefs; `worker_slots.cast::<u8>().add(...)` arithmetic; PG FFI calls (`pg_sys::max_parallel_workers_per_gather`, parallel-launch primitives) |
| `src/am/common/stream.rs` | 17 | `pg_sys::read_stream_*` FFI wrappers (`read_stream_next_buffer`, `read_stream_end`, `PrefetchBuffer`); per-buffer-data pointer derefs |

**Combined: 51 unsafe blocks.** Neither was in the original
post-Task-50 phased plan. Both are infrastructure consumed by HNSW
build_parallel (Task 58 partially), DiskANN insert/scan, IVF scan,
and SPIRE scan paths — so a typed-view burndown here cascades
benefits across AMs without per-AM rework.

## Non-Goals

- Do not refactor parallel-build worker orchestration. The state
  machine (claim slot, publish snapshot, release slot) is unchanged.
- Do not change `read_stream` semantics. Prefetch + buffer iteration
  ordering is preserved.
- Do not migrate AM-specific call sites in this task. HNSW
  build_parallel (Task 58 follow-up) and AM scan paths consume the
  new wrappers in their own tasks.
- Do not touch DSM-image layout. On-disk + in-memory shared state
  is invariant.
- Do not extend Phase-1 wrappers (P3 / P6 / P8) — this task adds new
  typed-view surface dedicated to the parallel/stream domain.

## Scope

Add typed-view wrappers in `src/am/common/parallel.rs` and
`src/am/common/stream.rs` (and sibling files if the surface deserves
separation):

1. **`EcParallelCoordinatorView<'state>`** — typed view over
   `*const EcParallelCoordinatorState`. Safe ops:
   `flags() -> u32`, `record_workers_done()`, `wait_for_workers(&self)`,
   etc. Per-method safety contract documented at the function level
   per `feedback_view_operations_not_accessors`.

2. **`EcParallelWorkerSlotsView<'state>`** — typed view over the
   worker-slots array. Safe ops:
   `slot(index: u32) -> Option<EcParallelWorkerSlotSnapshot>`,
   `try_claim() -> Option<EcParallelWorkerSlotGuard<'_>>`,
   `release(guard)`. Encapsulates the
   `cast::<u8>().add(offset).cast()` arithmetic.

3. **`ReadStreamScope<'rel>`** — RAII wrapper around
   `pg_sys::read_stream_begin_relation` + `read_stream_end`. Safe
   `next() -> Option<(PinnedBufferGuard, BlockNumber)>`. Drop calls
   `read_stream_end`. Encapsulates per-buffer-data pointer deref
   into a typed extractor.

4. **`PrefetchScope<'rel>`** — safe wrapper around
   `pg_sys::PrefetchBuffer` for the prefetch-only path (no buffer
   pin returned). One unsafe constructor; safe `prefetch(block)`
   method.

Each wrapper records its PG-callback / parallel-coordinator
lifetime invariant in its constructor doc, same pattern as the
Task 52 P8 wrappers (`PgAtomicU32Ref`, `SpinLockGuard`,
`ConditionVariableRef`) and Task 54 P3 wrappers (`WalTxnScope`,
`RegisteredBufferPage`).

## Migration Targets

This task migrates `src/am/common/parallel.rs` and `src/am/common/stream.rs`
themselves (self-narrowing). Cross-AM consumer migrations are
deferred:

| File | Pre | Target | Min Δ |
| --- | ---: | ---: | ---: |
| `src/am/common/parallel.rs` | 34 | ≤ 22 | -12 (-35%) |
| `src/am/common/stream.rs` | 17 | ≤ 11 | -6 (-35%) |
| **Combined** | **51** | **≤ 33** | **-18 (-35%)** |

Per-file floor is -30%; combined-subsystem target is -35% (slightly
more ambitious than the per-file floor since the two files share
the parallel/stream domain).

## Techniques

- Same patterns as Tasks 52 / 53 / 54: single `unsafe fn` constructor
  per wrapper, safe ops on the resulting view, lifetime-bound
  borrows.
- View types expose **operations**, not safe accessors returning
  `&'a T`. Strictly enforce `feedback_view_operations_not_accessors`.
- For `EcParallelCoordinatorView`: `flags()` returns a `u32`
  (value), not `&u32`; `record_workers_done()` performs the atomic
  op internally.
- For `ReadStreamScope`: `next()` returns `Option<(PinnedBufferGuard,
  BlockNumber)>` as owned values — no exposed `*mut` or `&'a` to
  per-buffer-data.
- RAII Drop: `ReadStreamScope::drop` calls `read_stream_end` unless
  explicitly finished.

## Cross-AM consumer absorption (deferred)

The new wrappers enable downstream reduction in:

- HNSW `build_parallel.rs` (currently 84; Task 58 partial close)
  — consumer migration to `EcParallelCoordinatorView` /
  `EcParallelWorkerSlotsView` is the path to clear the Task 58
  structural-ceiling concern.
- IVF / DiskANN scan paths — `read_stream` call sites consume
  `ReadStreamScope` directly.
- SPIRE custom_scan — same prefetch pattern.

These cross-AM migrations are out of Task 59 scope per §Non-Goals;
they belong to a future Task 58.1 (HNSW build_parallel followup)
and per-AM AM-specific tasks.

## Slice Plan

1. **001 — execution plan**: per-file unsafe surface audit, wrapper
   inventory enumeration, target reaffirmation.
2. **002 — `parallel.rs` typed views**: add `EcParallelCoordinatorView`,
   `EcParallelWorkerSlotsView`, `EcParallelWorkerSlotGuard`. Self-narrow
   `parallel.rs` to ≤ 22. No cross-AM call-site moves.
3. **003 — `stream.rs` typed views**: add `ReadStreamScope`,
   `PrefetchScope`. Self-narrow `stream.rs` to ≤ 11.
4. **004 — closeout**: per-file deltas, src/ total change, bench
   gate (HNSW build_parallel path is the indirect bench coverage —
   8-step suite vs Task 50 M5 baseline since `parallel.rs` is
   bench-exercised by parallel-build steps).

Stretch consumer migrations (HNSW build_parallel, AM scan paths)
are deferred to follow-on tasks.

## Validation

- `cargo fmt --all`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Per-file `unsafe { ... }` block count grep at HEAD
- `src/` total snapshot
- Bench gate: full 8-step `ecaz bench suite` against
  `benchmarks/task-50-m5-hnsw-baseline/` — `parallel.rs` is on the
  HNSW build_parallel path so the bench exercises it indirectly

## Exit Criteria

Task closes when:

- The four typed wrappers above (or their structural equivalents)
  exist in `src/am/common/parallel.rs` and `src/am/common/stream.rs`.
- `src/am/common/parallel.rs` ≤ 22 (-35%) OR documented structural
  ceiling per Task 50/448 precedent.
- `src/am/common/stream.rs` ≤ 11 (-35%) OR documented structural
  ceiling.
- Combined subsystem ≤ 33 (-35%) — task-level target.
- HNSW build_parallel + scan recall + QPS + per-row storage show
  no regression vs the post-Task-50 baseline.
- A closing summary packet records:
  - per-file before/after for `parallel.rs` and `stream.rs`;
  - the wrapper surface added;
  - the `src/` total block count change;
  - cross-AM consumer-site handoff list naming where the new
    wrappers will be absorbed under follow-on tasks (HNSW
    build_parallel, IVF scan, DiskANN scan, SPIRE custom_scan).

## Coordination

- Sequence after Task 57 (IVF) merge and Task 56 (SPIRE) progress
  — both will need the new wrappers eventually but neither blocks
  Task 59 start.
- Task 58 (HNSW build_parallel) is structurally adjacent — Task 59
  delivers the wrappers Task 58 needs for its remaining lift.
  Consider sequencing: Task 59 → Task 58.1 follow-up.
- Reviewer scope-lock: `src/am/common/{parallel,stream}.rs` only
  for the wrapper additions. AM consumer call sites stay HNSW /
  DiskANN / IVF / SPIRE task scope.

## Cross-References

- Phase-1 wrapper precedent: Tasks 52 (P8), 53 (P6), 54 (P3).
- Cross-AM consumer migration precedent: Tasks 55 (DiskANN), 57
  (IVF).
- Structural-ceiling rationale precedent: `reviews/task-50/448-hnsw-burndown-refreshed-closeout/`.
- Task 58 build_parallel disposition (blocked): `reviews/task-58/003-closeout/feedback/2026-05-23-01-reviewer.md`.
- Highest-standards rule: [[feedback_dont_defer_safety_fixes]].
