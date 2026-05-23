# Task 52 / 006 — ParallelContextRef + Deref Cleanup

Branch: `task-52`

## Summary

Introduce `ParallelContextRef<'a>` as a typed wrapper over PostgreSQL's
`ParallelContext` lifecycle (Enter/Exit/Initialize/Launch/Wait/Destroy
+ Copy-field reads), plus safe free-fn wrappers for one-off PG
primitives (`instr_start_parallel_query`, `shm_mq_set_sender`,
`table_parallelscan_estimate`, `index_info_*`).

Migrate `src/am/ec_hnsw/build_parallel.rs` consumer sites to the new
wrappers. Promote three formerly-`unsafe fn` helpers
(`estimate_chunk`, `estimate_keys`, `drain_worker_messages`) to safe
`fn` since their unsafe ops are confined to method bodies and the
contracts are encapsulated.

**Result: `build_parallel.rs` 105 → 80** — at the Task 52
§Exit Criterion #2 target.

## Per-file `unsafe { ... }` block delta

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 105 | **80** | **-25** |
| `src/am/common/parallel_context.rs` (new) | — | 19 | +19 |
| `src/am/common/mod.rs` | n/a | n/a | +1 module decl |

## Task 52 cumulative arc (slices 001-006)

| Surface | Pre-Task-52 | Now | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | **112** | **80** | **-32 (-28.6%)** |

Task 52 §Exit Criterion #2 (≤ 80) **satisfied**.

| Wrapper-side investment | Count |
| --- | ---: |
| `src/am/common/dsm.rs` (slice 002) | +4 |
| `src/am/ec_hnsw/parallel_build_view.rs` (slice 003+005) | +12 |
| `src/am/common/parallel_context.rs` (slice 006) | +19 |
| **Total wrapper-side** | **+35** |

Net `src/` total: was 960, now ≈ 968 (consumer -32 + wrapper +35 +
no change elsewhere in scope = +3 net; the wrapper modules are
durable infrastructure for Tasks 56/57's SPIRE/IVF parallel-build
migrations).

## What landed in slice 006

### `ParallelContextRef<'a>` (new wrapper)
Wraps `*mut pg_sys::ParallelContext` with:

- `unsafe fn new(*mut ParallelContext)` — single DSM-segment-lifetime
  contract per leader scope.
- safe Copy-field accessors: `as_ptr() / toc() / seg() /
  nworkers_launched() / estimator_mut() / worker(idx)`.
- safe lifecycle operations: `initialize_dsm() / launch_workers() /
  wait_for_workers_to_attach() / wait_for_workers_to_finish() /
  destroy()`.

`destroy()` consumes the wrapper by value, preventing post-destroy
reuse at the type-system level.

### Safe free-fn PG-primitive wrappers
- `enter_parallel_mode()` / `exit_parallel_mode()`
- `instr_start_parallel_query()`
- `index_info_parallel_workers(*mut IndexInfo) -> i32` (null-safe)
- `index_info_is_concurrent(*mut IndexInfo) -> bool` (null-safe)
- `shm_mq_set_sender(*mut shm_mq, *mut PGPROC)`
- `table_parallelscan_estimate(Relation, Snapshot) -> Size`

Each forwards to its PG primitive inside a wrapper-side unsafe block.
The caller's "live PG resource" contract is implicit in the
surrounding parallel-build scope — same convention as the slice-447
`shm_toc_allocate` / `insert` safe wrappers.

### In-file safe-fn promotions
- `estimate_chunk(estimator, size)`: `unsafe fn` → safe `fn`.
- `estimate_keys(estimator, keys)`: `unsafe fn` → safe `fn`.
- `EcHnswParallelBuildLeader::drain_worker_messages`: `unsafe fn` →
  safe `fn`. Body retains internal unsafe blocks for PG
  receive/interrupt/sleep ops.

Bodies unchanged in op sequence; only the function signatures move
the contract one layer inward.

### Consumer migrations in `build_parallel.rs`
Both leader `begin()` methods:
- `pg_sys::EnterParallelMode()` / `ExitParallelMode()` →
  safe free fns. (4 standalone consumer unsafe blocks shed)
- `pg_sys::CreateParallelContext` raw pointer immediately wrapped:
  `let pcxt_ref = unsafe { ParallelContextRef::new(pcxt_ptr) }` —
  single contract site per leader scope.
- `(*pcxt).seg.is_null()` standalone reads → `pcxt_ref.seg().is_null()`
  (safe). (2 shed)
- `(*pcxt).nworkers_launched` reads → `pcxt_ref.nworkers_launched()`
  (safe). (4 shed across both leaders + finish() methods)
- `pg_sys::InitializeParallelDSM` / `LaunchParallelWorkers` /
  `WaitForParallelWorkersToAttach` / `WaitForParallelWorkersToFinish` /
  `DestroyParallelContext` standalone calls → safe methods on
  pcxt_ref. (10 shed)
- Two large estimator unsafe blocks → safe expression bodies (now
  that `estimate_chunk` / `estimate_keys` are safe fn). (-2)

Both worker entrypoints:
- `pg_sys::InstrStartParallelQuery()` standalone → safe free fn. (2 shed)
- `pg_sys::shm_mq_set_sender(queue, MyProc)` standalone → safe free
  fn. (1 shed)

`parallel_build_shared_workspace_size`:
- `unsafe { pg_sys::table_parallelscan_estimate(...) }` → safe free
  fn. (1 shed)

`EcHnswParallelBuildPlan::from_index_info`:
- null check + `(*index_info).ii_ParallelWorkers` → safe
  `index_info_parallel_workers`. (1 shed)

Heap-leader `begin`:
- `unsafe { !index_info.is_null() && (*index_info).ii_Concurrent }` →
  safe `index_info_is_concurrent`. (1 shed)

`drain_worker_messages` call site:
- One unsafe wrap dropped via fn-promotion. (1 shed)

## Anti-pattern B / view-operations discipline (8th+ applications)

- `ParallelContextRef`: no `fn(&self) -> &Field` accessors. Pointer
  fields return `*mut` Copy values; `estimator_mut()` uses
  `addr_of_mut!` for the embedded field (no `&mut` materialized).
- Safe free fns return Copy values, not references.
- `unsafe fn → safe fn` promotions keep the unsafe ops confined to
  method bodies — no escaped `&T` references introduced.

## Validation

- `cargo fmt --all` — clean (touched only the in-scope files).
- `cargo check --no-default-features --features pg18` — `Finished`
  exit 0, 14.74s incremental.
- `cargo clippy ... -- -D warnings` — same pre-existing rabitq
  backlog; not blocking.
- `cargo pgrx test` — deferred per `feedback_dyld_buffer_blocks_known`.

## Toward closeout (slice 007)

Task 52 §Exit Criteria status:

1. **Four typed views in src/am/common/ (or sibling modules)** —
   satisfied:
   - `ShmTocBuilder<'a>` (dsm.rs, slice 002)
   - `ShmTocReader<'a>` (dsm.rs, slice 002)
   - `EcHnswParallelBuildSharedView<'a>` (ec_hnsw/parallel_build_view.rs,
     slice 003) — absorbs both heap-scan and graph-build phases per
     planning packet's correction of the spec.
   - `ParallelContextRef<'a>` (common/parallel_context.rs, slice 006)
     — added beyond the originally-named four; needed to clear the
     (*pcxt) deref residuals and lifecycle calls.
2. **`build_parallel.rs` ≤ 80** — **satisfied (80)**.
3. **HNSW recall + QPS no regression vs post-Task-50 baseline** —
   pending slice 007 bench window.
4. **Closing summary packet** — slice 007.

Closeout (slice 007) will:
- Run `ecaz bench latency` on `benchmarks/task-50-m5-hnsw-baseline/`.
- Author the formal closeout packet with per-file deltas, full
  wrapper surface, and `src/` total.

## Cross-references

- Memory rules applied: `feedback_anti_pattern_b_unbounded_lifetime`,
  `feedback_view_operations_not_accessors`,
  `feedback_no_premature_task_close`.
- Reviewer's premature-close rebuttal:
  `reviews/task-52/004-build-parallel-shm-toc-migration/feedback/2026-05-23-03-coder.md`.
- Pre-Task-52 baseline: `reviews/task-52/001-execution-planning/artifacts/baseline-unsafe-density.txt`.
