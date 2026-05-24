# Task 52: Common P8 Finish — Typed Shared-Header + ShmToc Wrappers

Status: **proposed** — supersedes the §P8 continuation queue named in
`reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md`.
First Phase-1 lane in the post-Task-50 hardening sequence.

## Why

Task 50 closed the HNSW unsafe burndown at **549 → 327 (-40.44%)** but
`build_parallel.rs` plateaued at **112** blocks — the documented
structural ceiling. The 448 closeout names the cause:

> 1. DSM atomics, SpinLocks, ConditionVariables: P8 contract surface.
>    Opening migration landed in slice 447 (`src/am/common/dsm.rs`).
>    The compound `SpinLockAcquire + mutate + Release +
>    ConditionVariableSignal` blocks cannot be split into RAII-scoped
>    fragments without inflating the block count; landing the full P8
>    disposition requires a typed `EcHnswParallelBuildSharedView<'a>`
>    that absorbs the entire compound pattern into a single safe method.
> 2. `shm_toc_allocate` / `shm_toc_insert` / `shm_toc_lookup`
>    (~30 ops batched into wide blocks): typed `ShmTocBuilder<'a>` /
>    `ShmTocReader<'a>` wrappers are open work.
> 4. DSM-laid-out struct field derefs (`(*shared).field` and
>    `(*pcxt).field`): typed views are open work.

This task lands those typed views. Once they exist, slice-level
`build_parallel.rs` migrations can chip the 112-block ceiling without
inflating the count via RAII fragment split.

## Non-Goals

- Do not touch `src/am/ec_ivf/**`, `src/am/ec_spire/**`, or
  `src/am/ec_diskann/**`. SPIRE / IVF parallel-build will consume these
  wrappers later under Tasks 56/57; their consumer migration is out of
  scope here.
- Do not extend `dsm.rs` beyond what `build_parallel.rs` migration
  needs. Wrapper surface is evidence-driven from HNSW residual patterns.
- Do not refactor existing slice-447 `PgAtomicU32Ref` consumers; that
  migration is correct and stable.
- Do not run the bench suite per slice. Per memory
  `feedback_coder_push_smoke_checks`, smoke checks between slices,
  bench window once at task close.

## Scope

Extend `src/am/common/dsm.rs` (and add new modules where the wrapper
deserves its own file) with:

1. **`EcHnswParallelBuildSharedView<'a>`** — typed borrow over the
   `EcHnswParallelBuildShared` DSM-laid-out header. Safe accessors for
   per-field DSM atomics, the embedded `slock_t`, and the embedded
   `ConditionVariable`. Compound `acquire / mutate / signal` pattern
   absorbed into a single safe method (e.g.
   `with_locked_mut(|view, guard| { ... }; signal_workers_done())`).
2. **`EcHnswParallelGraphBuildSharedView<'a>`** — same shape for the
   graph-build phase's distinct shared header.
3. **`ShmTocBuilder<'a>`** — typed wrapper over `shm_toc_estimate` +
   `shm_toc_allocate` + `shm_toc_insert` used in the leader's setup
   path. One `unsafe fn new` constructor; safe `allocate(key, size)`
   and `insert(key, ptr)` methods.
4. **`ShmTocReader<'a>`** — typed wrapper over `shm_toc_lookup_noerror`
   / `shm_toc_lookup` used in worker entrypoints. One `unsafe fn
   attach` constructor; safe `lookup<T>(key) -> &T` / `lookup_mut<T>(key)
   -> &mut T` methods with the key-type contract documented at the
   wrapper level.

Each wrapper records its DSM-segment-lifetime invariant in its
constructor doc, same pattern as `PgAtomicU32Ref::from_raw`.

## Migration Targets

Slice-level migration of `src/am/ec_hnsw/build_parallel.rs` consumers:

| Surface | Expected block delta |
| --- | ---: |
| Leader-side `shm_toc_estimate + allocate + insert` chain (×2: heap-build phase + graph-build phase) | -8 to -12 |
| Worker-side `shm_toc_attach + lookup_noerror` chain (×2) | -6 to -10 |
| `SpinLockAcquire + record_worker_counts + Release + CV signal` compound (×2 worker entries) | -6 to -10 |
| `(*shared).field` typed-view migrations | -8 to -14 |
| `(*pcxt).field` typed-view migrations | -2 to -4 |

**Target**: `build_parallel.rs` 112 → ≤ 80 (-29% or better).

## Techniques

Reuse Task 50 patterns. The new wrappers are an extension of the
slice-447 P8 module; do not invent new abstraction shapes.

## Slice and Packet Rules

Same as Task 50:

- Each packet must report `unsafe { ... }` block count before / after
  for every touched file.
- Wrapper-side `unsafe { ... }` blocks are reported separately from
  consumer-side reductions (HNSW subsystem vs `src/am/common/` total).
- `src/` total snapshot **required** in this task (per Task 50's
  reviewer flag — the closeout did not surface it).
- Helpers introduced by a slice must contain `unsafe` only where
  unavoidable.

## Performance Gate

Parallel-build slot path latency is a candidate hot path. The slice-447
migration was setup-time only; this task touches worker hot loops via
the SpinLock/CV compound migration.

Required evidence per slice that touches a worker loop:

- `ecaz bench latency` on the post-Task-50 M5 baseline corpus
  (`benchmarks/task-50-m5-hnsw-baseline/`) at the same prefixes and
  sweep, with explicit before/after comparison.

Acceptance: regression tolerance is the same as Task 50.

## Validation

Per slice:

- `cargo fmt --all`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  on touched modules
- direct unsafe-block count per touched file
- focused `cargo pgrx test pg18 ec_hnsw::build_parallel` when the slice
  changes worker-loop behavior

## Exit Criteria

Task closes when:

- The four typed views above exist in `src/am/common/dsm.rs` (or
  sibling modules under `src/am/common/`).
- `src/am/ec_hnsw/build_parallel.rs` block count ≤ 80.
- HNSW recall + QPS on the standard M5 corpus shows no regression vs
  the post-Task-50 baseline.
- A closing summary packet records:
  - per-file before/after for `build_parallel.rs` and any other HNSW
    file touched;
  - the full `src/am/common/` wrapper surface added;
  - the `src/` total block count change.

## Coordination

- Phase-1 lane — runs before Tasks 53 (P6) and 54 (P3).
- HNSW-only consumer migration. SPIRE / IVF parallel-build consumers
  are deferred to Tasks 56/57 once those rotations open.
- Coordinate with Task 51 (IVF RaBitQ optimization): no overlap
  expected since IVF parallel-build is not touched here.
- Reviewer scope is automatic from branch name; coder pushes per
  memory `feedback_coder_push_smoke_checks` (smoke checks between
  slices, bench window once at close).

## Cross-References

- Supersedes `reviews/task-50/448-hnsw-burndown-refreshed-closeout`
  §"Next-highest-density modules" P8 continuation queue.
- Builds on `src/am/common/dsm.rs` (slice 447) and
  `reviews/task-50/447-p8-dsm-typed-wrappers/`.
- Bench gate consumes
  `benchmarks/task-50-m5-hnsw-baseline/manifest.md` as the pre-state.
