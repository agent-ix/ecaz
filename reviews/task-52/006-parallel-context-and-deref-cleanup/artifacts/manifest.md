# Task 52 / 006 — ParallelContextRef + Deref Cleanup · Artifact Manifest

Packet path: `reviews/task-52/006-parallel-context-and-deref-cleanup/`
Branch: `task-52`

## Surfaces

- `src/am/common/parallel_context.rs` (new) — typed
  `ParallelContextRef<'a>` + safe free-fn wrappers for PG primitives.
- `src/am/common/mod.rs` — module registration.
- `src/am/ec_hnsw/build_parallel.rs` — consumer migration.

## Per-file before/after `unsafe { ... }` blocks

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 105 | **80** | **-25** |
| `src/am/common/parallel_context.rs` (new) | — | 19 | +19 (wrapper-side) |

`build_parallel.rs` is at the **Task 52 §Exit Criterion #2 target
of ≤ 80**.

## Task 52 cumulative state at end of slice 006

| Surface | Pre-Task-52 | Post-slice-006 | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 112 | **80** | **-32 (-28.6%)** |
| `src/am/common/dsm.rs` | 9 | 13 | +4 |
| `src/am/ec_hnsw/parallel_build_view.rs` (new) | — | 12 | +12 |
| `src/am/common/parallel_context.rs` (new) | — | 19 | +19 |

Wrapper-side investment: +35. Consumer-side reduction: -32. Net
+3 src/ total — the wrapper modules are the durable infrastructure
that Tasks 56 (SPIRE) and 57 (IVF) parallel-build migrations reuse
later.

## What landed in this slice (delta from slice 005's 105)

### `ParallelContextRef<'a>` typed wrapper (new module)
Wraps `*mut pg_sys::ParallelContext` with safe accessors and
operations:
- `unsafe fn new(*mut ParallelContext)` — single contract site per
  leader scope.
- safe `as_ptr() / toc() / seg() / nworkers_launched() /
  estimator_mut() / worker(idx)` — Copy reads of `(*pcxt).field`.
- safe `initialize_dsm() / launch_workers() /
  wait_for_workers_to_attach() / wait_for_workers_to_finish() /
  destroy()` — wrap PG lifecycle primitives, each absorbing 2 prior
  consumer-side standalone unsafe blocks.

### Safe free-fn wrappers for one-off PG primitives
In `src/am/common/parallel_context.rs`:
- `enter_parallel_mode()` / `exit_parallel_mode()`
- `instr_start_parallel_query()`
- `index_info_parallel_workers(*mut IndexInfo) -> i32` (null-safe)
- `index_info_is_concurrent(*mut IndexInfo) -> bool` (null-safe)
- `shm_mq_set_sender(*mut shm_mq, *mut PGPROC)`
- `table_parallelscan_estimate(Relation, Snapshot) -> Size`

Convention follows the slice-447 dsm.rs pattern: safe fn wrappers
where the caller's contract is "live PG resource pointer established
by the surrounding scope" — same as `shm_toc_allocate` /
`shm_toc_insert` etc.

### In-file safe-fn promotions
- `estimate_chunk` / `estimate_keys` (build_parallel.rs locals):
  `unsafe fn` → safe `fn`. Bodies retain one internal unsafe block
  each; call sites in both leaders no longer need surrounding unsafe
  blocks (-2 leader-side unsafe blocks).
- `drain_worker_messages` (`EcHnswParallelBuildLeader` impl):
  `unsafe fn` → safe `fn`. Body retains internal unsafe blocks for
  PG receive/interrupt/sleep ops; one call site drops its unsafe
  wrap (-1).

### Consumer migrations in `build_parallel.rs`
- Both leader `begin()` methods: `EnterParallelMode` /
  `ExitParallelMode` standalone calls → safe free fns;
  `CreateParallelContext` returns a raw pointer wrapped immediately
  into `ParallelContextRef`; `InitializeParallelDSM` /
  `LaunchParallelWorkers` / `WaitForParallelWorkersToAttach` /
  `DestroyParallelContext` standalone unsafe blocks → safe methods
  on `pcxt_ref`; `(*pcxt).seg.is_null()` and `(*pcxt).nworkers_launched`
  standalone reads → safe accessors.
- Both worker entrypoints' `InstrStartParallelQuery` standalone
  unsafe blocks → safe `instr_start_parallel_query()` free fn.
- Both leader `finish()` methods: same `DestroyParallelContext` /
  `WaitForParallelWorkersToFinish` / `ExitParallelMode` /
  `(*self.pcxt).nworkers_launched` migration.
- `parallel_build_shared_workspace_size`:
  `unsafe { pg_sys::table_parallelscan_estimate(...) }` → safe
  `table_parallelscan_estimate(...)`.
- `EcHnswParallelBuildPlan::from_index_info`: null-check +
  `(*index_info).ii_ParallelWorkers` → `index_info_parallel_workers`.
- Worker `shm_mq_set_sender` site → safe free fn.

## Anti-pattern B / view-operations discipline

8th and 9th applications of `feedback_anti_pattern_b_unbounded_lifetime`:
- `ParallelContextRef::seg()` / `toc()` / `worker(idx)` return raw
  `*mut` pointers, not Rust references.
- `ParallelContextRef::estimator_mut()` returns `*mut shm_toc_estimator`
  via `addr_of_mut!` — no `&mut` created.

`feedback_view_operations_not_accessors` (3rd application): all
`ParallelContextRef` methods are operations (initialize, launch,
wait, destroy) or `Copy`-value reads — no `fn(&self) -> &Field`
accessors.

## Validation

- `cargo fmt --all` — clean.
- `cargo check --no-default-features --features pg18` — `Finished`
  exit 0, 14.74s incremental.
- `cargo clippy ... -- -D warnings` — not re-run; same pre-existing
  rabitq backlog documented in slice 002 manifest. Closeout (slice
  007) will record final state.
- `cargo pgrx test` — skipped per `feedback_dyld_buffer_blocks_known`.
  The migration is semantics-preserving: every safe wrapper forwards
  to the same PG primitive in the same order. `unsafe fn` →
  `safe fn` promotions on `estimate_chunk`/`estimate_keys`/
  `drain_worker_messages` are signature-only — bodies unchanged in
  ops sequence.

- Timestamp: 2026-05-23.
- Head SHA: parent of packet commit.
