# Task 59 / 001 — Execution Plan

**Branch:** `task-59-parallel-stream-burndown`
**Head at audit:** `392432134` (post task-56 merge into main)
**Owner:** Codex (coder)
**Scope-lock:** `src/am/common/{parallel,stream}.rs` only — no AM
consumer call-site migrations (those are Task 58.1 / per-AM
follow-ups).

## Goal

Self-narrow the two largest still-unaddressed `src/am/common/` files
under the post-Task-50 phased plan by introducing dedicated typed-view
wrappers for the parallel-coordinator and read-stream domains.

Per `plan/tasks/59-common-parallel-stream-burndown.md`:

| File | Pre | Target | Min Δ |
| --- | ---: | ---: | ---: |
| `src/am/common/parallel.rs` | **34** | ≤ 22 | -12 (-35%) |
| `src/am/common/stream.rs` | **17** | ≤ 11 | -6 (-35%) |
| **Combined** | **51** | **≤ 33** | **-18 (-35%)** |

Per-file floor is -30%; subsystem combined target is -35%.

## Baseline (HEAD = 392432134)

`scripts/unsafe_block_count.sh src/am/common/parallel.rs src/am/common/stream.rs`:

```
  34 src/am/common/parallel.rs
  17 src/am/common/stream.rs
```

`src/` total: **771** blocks (this number is the closeout-time
diff anchor).

## Per-file Unsafe Inventory

### `src/am/common/parallel.rs` — 34 blocks

Categorized by wrapper-target:

| Category | Blocks | Lines (HEAD) | Wrapper |
| --- | ---: | --- | --- |
| Raw-pointer field accessors on `ParallelScanAttachment` | 2 | 90, 96 | `EcParallelCoordinatorView`, `EcParallelWorkerSlotsView` |
| PG GUC FFI (`max_parallel_workers_per_gather`) | 1 | 148 | small safe helper `parallel_workers_per_gather()` |
| Descriptor-layout pointer arithmetic (`coordinator_ptr`, `worker_slots_ptr`, slot offsets) | 5 | 159, 176, 193, 205, 217 | `EcParallelDescriptorLayout` private helper that owns the offset math; reset/init expressed in terms of typed view writes |
| `parallel_scan_state_ptr` (pg17 + pg18 cfg arms) | 2 | 311, 333 | unchanged — narrow PG-callback offset deref |
| `validate_parallel_scan_state(unsafe { &*state })` call-sites duplicated across release/publish/read/reset | 4 | 354, 411, 433, 449 | `EcParallelStateScope::attach_raw(*mut state)` → single unsafe-fn checker |
| `*mut state` mutability through reset path | 2 | 463, 472 | folded into `EcParallelStateScope::attach_raw_mut` |
| `target.cast::<…>()` init entry | 2 | 367, 378 | `EcParallelDescriptorLayout::init_target` |
| Tests — `unsafe fn` invocations + descriptor ps_offset writes | 15 | 533, 571, 577, 591, 597, 601, 608, 614, 625, 635, 642, 650, 656, 669, 686, 693 | safe ops on `ParallelScanAttachment` shrink several test sites (claim/release/publish/read can become safe view ops); the remaining test sites stay unsafe because they call the still-unsafe-fn extern entry points |

Total production: 19. Total tests: 15. Sum: 34 (matches `unsafe_block_count`).

### `src/am/common/stream.rs` — 17 blocks

| Category | Blocks | Lines (HEAD) | Wrapper |
| --- | ---: | --- | --- |
| `pg_sys::read_stream_begin_relation` call-sites | 4 | 168, 232, 424, 454 | `ReadStreamScope::open` (RAII; one `unsafe fn` constructor) |
| `pg_sys::read_stream_next_buffer` per-iteration deref | 3 | 184, 284, 325 | folded into `ReadStreamScope::next()` (operation, not accessor) |
| `pg_sys::read_stream_reset` | 1 | 346 | `ReadStreamScope::reset()` |
| `pg_sys::read_stream_end` (legacy direct call) | 1 | 199 | `ReadStreamScope::drop` (RAII) — the existing `PgReadStreamGuard::drop` keeps its single `unsafe { }` |
| `pg_sys::PrefetchBuffer` (pg17/non-pg18 path) | 1 | 211 | `PrefetchScope::prefetch(block)` (one `unsafe fn` constructor; safe `prefetch`) |
| `PinnedBufferGuard::from_pinned` / `LockedBufferGuard::lock_pinned` per-buffer typing | 4 | 191, 291, 369, 397 | encapsulated inside `ReadStreamScope::next_pinned()` / `next_locked(lockmode)` operations |
| Per-buffer-data block-number deref | 1 | 265 | typed extractor inside `ReadStreamScope::next` tuple return |
| Per-buffer-data block-number write (callback side) | 1 | 486 | helper `PerBufferDataSlot::write(block)` |
| `PgReadStreamGuard::drop` (`read_stream_end`) | 1 | 252 | retained; folded into the new `ReadStreamScope::drop` (the guard struct is replaced) |

Total: 17 (matches).

## Wrappers To Add

Per `plan/tasks/59`, §Scope:

### 1. `EcParallelCoordinatorView<'state>`

- Constructor: `unsafe fn from_raw(coord: *const EcParallelCoordinatorState) -> Self` — caller asserts the
  pointer aims at a validated AM-private coordinator slot.
- Ops (value-returning or in-place atomic, never `&'a T`):
  - `flags() -> u32`
  - `claimed_worker_slots() -> u32`
  - `record_worker_slot_claimed()` → `fetch_add(1, AcqRel)`
  - `record_worker_slot_released()` → `fetch_sub(1, AcqRel)`
  - (test-side only, behind `#[cfg(test)]`): `store_claimed_worker_slots(u32)`.

Per `feedback_view_operations_not_accessors`, no safe
`fn(&self) -> &'a AtomicU32`.

### 2. `EcParallelWorkerSlotsView<'state>`

- Constructor: `unsafe fn from_raw(base: *mut EcParallelWorkerSlot,
  count: u32, stride: Size) -> Self`.
- Ops:
  - `count() -> u32`
  - `with_slot<R>(index: u32, f: impl FnOnce(&EcParallelWorkerSlot) -> R) -> Result<R, &'static str>`
    — encapsulates the bounds-checked stride deref. Callers operate on
    the slot only inside the closure, never receive `&'a slot`.

Closure form is used because `EcParallelWorkerSlot` is a leaf
atomic-only struct already; the closure isolates the unsafe deref
without leaking a reference.

### 3. `EcParallelStateScope<'state>`

- Constructor: `unsafe fn attach_raw(state: *mut EcParallelScanState) -> Result<Self, &'static str>` —
  performs the validate-and-fan-out once, replacing the four call-site
  `validate_parallel_scan_state(unsafe { &*state })?` lines.
- Exposes `coordinator() -> EcParallelCoordinatorView<'_>`,
  `worker_slots() -> EcParallelWorkerSlotsView<'_>`, `rescan_epoch() -> u32`,
  `worker_slot_count() -> u32`.

### 4. `ReadStreamScope<'rel>`

- Constructor: `unsafe fn open<S>(mode: i32, rel: pg_sys::Relation, cb: ReadStreamCallback, state: &mut S) -> Self`
  — single `unsafe fn`, replaces the four `read_stream_begin_relation`
  call-sites.
- Ops:
  - `next_pinned() -> Option<(PinnedBufferGuard, Option<BlockNumber>)>` (operation; encapsulates
    `read_stream_next_buffer` + `PinnedBufferGuard::from_pinned` +
    per-buffer-data extraction)
  - `next_locked(lockmode: i32) -> Result<Option<(LockedBufferGuard, Option<BlockNumber>)>, &'static str>`
  - `reset()` (wraps `read_stream_reset`)
- `Drop` calls `read_stream_end`.

### 5. `PrefetchScope<'rel>`

- Constructor: `unsafe fn for_relation(rel: pg_sys::Relation, fork: pg_sys::ForkNumber) -> Self`.
- Op: `prefetch(block: pg_sys::BlockNumber)` — safe; encapsulates the
  `PrefetchBuffer` FFI call. pg17-only consumer (the pg18 path
  goes through ReadStreamScope).

Each wrapper records its PG-callback / parallel-coordinator lifetime
invariant in its constructor doc, same pattern as Task 52 P8 wrappers
(`PgAtomicU32Ref`, `SpinLockGuard`, `ConditionVariableRef`) and Task 54
P3 wrappers (`WalTxnScope`, `RegisteredBufferPage`).

## Block-Reduction Accounting (planned)

### parallel.rs: 34 → expected ≤ 21

- `EcParallelCoordinatorView` consumes L96 deref. Net **-1**.
- `EcParallelWorkerSlotsView::with_slot` consumes L90, L205, L217. Net **-3**.
- `EcParallelStateScope::attach_raw` consumes L354, L411, L433, L449. Net **-4**.
- `attach_raw_mut` (or sibling) consumes L463, L472. Net **-2**.
- `parallel_workers_per_gather()` safe helper consumes L148. Net **-1**.
- `EcParallelDescriptorLayout::init_target` consumes L367, L378. Net **-2**.

Production: **-13** (19 → 6 expected).

Test sites: claim/release/publish/read can be expressed as safe
methods on `EcParallelStateScope` exposed to tests, clearing **L625,
L635, L642, L650, L656, L669, L686** (-7) while leaving the still
genuinely-unsafe descriptor-storage manipulations (`ps_offset` writes,
target pointer arithmetic). The `unsafe fn` external entry points
themselves remain (`unsafe fn` declarations don't count toward block
count; only `unsafe { … }` blocks do — those at the test boundary may
collapse if call sites move through safe `EcParallelStateScope`).

Conservative test-side delta: **-5** (15 → 10). Final
parallel.rs estimate: **6 + 10 = 16**, comfortably under the 22 ceiling.

If structural realities (e.g., descriptor-layout offset math cannot
collapse without exposing safe `&'a T` accessors per
`feedback_view_operations_not_accessors`) push us above 16, the
ceiling target stays 22. Falling short of 22 is a HARD BLOCK and
triggers a re-plan rather than a structural-ceiling claim — per
`feedback_no_premature_task_close`, this is task-start, not closeout.

### stream.rs: 17 → expected ≤ 9

- `ReadStreamScope::open` consumes L168, L232, L424, L454 (four
  `read_stream_begin_relation` sites). Net **-4** at the source; one
  unsafe fn constructor (the implementation block) re-introduces **+1**
  internally → Net **-3**.
- `ReadStreamScope::next_pinned` / `next_locked` consume L184, L284, L325
  (three `read_stream_next_buffer`) plus L191, L369, L397
  (PinnedBufferGuard / LockedBufferGuard typing) plus L265
  (per-buffer-data deref). Implementation re-uses ~2 unsafe blocks
  internally. Net **-5**.
- `ReadStreamScope::reset` consumes L346. One internal block. Net **0**.
- `ReadStreamScope::drop` consumes L199 (legacy direct) and
  retains L252 in the replacement Drop impl. Net **-1**.
- `PrefetchScope::prefetch` consumes L211. One internal block in
  constructor or method. Net **0**.
- `PerBufferDataSlot::write` consumes L486. One internal block. Net **0**.

Stream final estimate: **9**, under the 11 ceiling.

## Slice Plan

Per `plan/tasks/59-common-parallel-stream-burndown.md` §Slice Plan:

1. **001 (this packet)** — execution plan.
2. **002 — parallel.rs typed views**: add the five
   parallel-side helpers above. Self-narrow parallel.rs to ≤22.
   Smoke checks (cargo fmt + cargo check + cargo clippy pg18) between
   slice and commit, per `feedback_coder_push_smoke_checks`. Tests
   are run on parallel.rs/parallel_slot.rs scope (cargo test
   parallel::tests, parallel_slot::tests).
3. **003 — stream.rs typed views**: add ReadStreamScope + PrefetchScope
   + PerBufferDataSlot. Self-narrow stream.rs to ≤11. Smoke checks.
   Stream-side tests are static (cargo check) — runtime stream
   exercise is the bench gate at 004.
4. **004 — closeout**: per-file deltas, src/ total change, bench
   gate via `ecaz bench suite` against
   `benchmarks/task-50-m5-hnsw-baseline/` since parallel.rs is on the
   HNSW build_parallel path. Cross-AM consumer-site handoff list per
   §Exit Criteria.

## Validation

Per `plan/tasks/59` §Validation:

- `cargo fmt --all` (each slice)
- `cargo check --all-targets --no-default-features --features pg18,bench` (each slice)
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` (each slice)
- `cargo test -p ecaz-vector --lib --no-default-features --features pg18 -- am::common::parallel::tests am::common::stream::tests am::common::parallel_slot::tests` at 002 and 003.
- Per-file `unsafe { … }` block count grep at HEAD and after each
  slice; src/ total snapshot at 004.
- Bench gate at 004 only: full 8-step `ecaz bench suite` against
  `benchmarks/task-50-m5-hnsw-baseline/`.

## Non-Goals (re-state from task plan)

- No refactor of worker orchestration semantics (claim/publish/release
  ordering is invariant).
- No change to `read_stream` semantics (prefetch + buffer iteration
  ordering invariant).
- No AM consumer call-site migrations (HNSW build_parallel, IVF
  scan, DiskANN scan, SPIRE custom_scan stay out — Task 58.1 / per-AM
  follow-ups).
- No DSM-image layout change.
- No Phase-1 (P3 / P6 / P8) wrapper extension.

## Cross-References

- Precedents: Tasks 52 (P8), 53 (P6), 54 (P3) — Phase-1 typed-view
  patterns.
- View-op discipline: `feedback_view_operations_not_accessors`,
  `feedback_anti_pattern_b_unbounded_lifetime`.
- Don't-defer rule: `feedback_dont_defer_safety_fixes`,
  `feedback_no_premature_task_close`.
- Bench provenance: `spec/non-functional/NFR-007-benchmark-provenance.md`.
- Baseline anchor: `benchmarks/task-50-m5-hnsw-baseline/`.

## Open Questions

None. Targets are set by the task plan; per-block accounting above
gives credible headroom over both per-file ceilings. If the structural
realities push parallel.rs higher than 22, the surface-by-surface
trade is presented as a HARD BLOCK in the 002 packet rather than as a
structural-ceiling deferral. Reviewer decides next-step disposition
there.

## Artifacts

- `artifacts/parallel_blocks.txt` — `grep -n "unsafe {" src/am/common/parallel.rs` at HEAD.
- `artifacts/stream_blocks.txt` — `grep -n "unsafe {" src/am/common/stream.rs` at HEAD.
- `artifacts/baseline_counts.txt` — `scripts/unsafe_block_count.sh` output for both files plus src/ total.
- `artifacts/manifest.md` — packet-local source of truth.
