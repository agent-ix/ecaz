# Task 50/401: HNSW build_parallel shared-header safe borrow

## Why this slice

`src/am/ec_hnsw/build_parallel.rs` is the second-densest HNSW file (130
blocks at the start of this rotation, 128 after slice 400). The worker entry
points `parallel_build_worker_main` and `parallel_graph_build_worker_main`
each dereference the worker's `*mut EcHnswParallelBuildSharedHeader` through
a small set of read-only field accesses and method calls (`validate`,
`is_concurrent`, `heaprelid`, `indexrelid`, `participant_count`). Each
deref is its own `unsafe { (*shared).field }` block.

`EcHnswParallelBuildSharedHeader` already exposes `validate`,
`scanned_heap_tuples`, `encoded_index_tuples`, and `record_worker_counts`
as ordinary `&self` / `&mut self` methods. The unsafe is purely the raw
deref to obtain the reference.

This slice introduces one private helper, `shared_header_ref`, that lifts
that obligation once at the function entry. Each clean read site converts
from `unsafe { (*shared).field }` to `header.field` after binding `let
header = shared_header_ref(shared);`. The two worker entries cover all
clean sites.

Per Task 50 §Techniques, this is technique 2 (lift invariants into
references) at the FFI boundary, with technique 1 (encapsulate at the FFI
boundary) for the helper.

## Scope

- New module-private helper `shared_header_ref(shared) ->
  &'a EcHnswParallelBuildSharedHeader` placed next to `checked_u16` at
  build_parallel.rs. The helper is the single point where the raw deref
  obligation lives; it null-checks via `NonNull::new` and uses
  `NonNull::as_ref` under the function-level SAFETY contract.
- Five clean call sites converted in
  `parallel_build_worker_main` and `parallel_graph_build_worker_main`:
  - `(*shared).validate()` ×2 (one per worker entry)
  - `(*shared).is_concurrent` (cached as `header.is_concurrent` and reused
    later for `IndexInfoView::as_mut().ii_Concurrent` per slice 400)
  - `(*shared).heaprelid`
  - `(*shared).indexrelid`
  - `(*shared).participant_count` hoisted out of a downstream call
- The `participant_count` hoist additionally lets the surrounding
  `unsafe { insert_concurrent_dsm_graph_participant(...) }` block drop its
  `unsafe` wrapper (the function is `pub(super) fn`, not `unsafe fn`), so
  one additional block disappears as a side effect.

Out-of-scope sites that remain `unsafe` and are correctly so:

- `SpinLockAcquire`/`SpinLockRelease`/`ConditionVariableInit`/`ConditionVariableSignal`
  on `&mut (*shared).mutex` and `&mut (*shared).workersdonecv` — PG raw-pointer
  APIs whose unsafe is the inner FFI itself, not the deref.
- `record_worker_counts(...)` inside the spinlock-protected block — covered
  by the same inner `unsafe` scope as the spinlock calls.

These are exactly the kind of "irreducible PG-API boundary" residuals that
Task 50 §Strategic Method names as the eventual residual registry.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 128 | 123 | -5 |
| **HNSW subsystem subtotal** | **540** | **535** | **-5** |

Breakdown on `build_parallel.rs`:

- Removed: 5 `(*shared).field` deref blocks at the two worker entries.
- Removed: 1 unnecessary `unsafe { insert_concurrent_dsm_graph_participant(...) }`
  block (function is safe; the original `unsafe { ... }` only existed to
  cover the deref of `(*shared).participant_count`).
- Added: 1 `unsafe { header.as_ref() }` block inside `shared_header_ref`.
- Net: -5.

## Validation

Artifacts under `reviews/task-50/401-hnsw-shared-header-ref/artifacts/`:

- `manifest.md` — head SHA, lane, command, timestamps, validation mapping.
- `per-file-after.log` — post-change HNSW per-file block counts.
- `build-parallel-unsafe-block-lines-after.log` — post-change line-by-line
  listing for every remaining `unsafe { ... }` block in
  `src/am/ec_hnsw/build_parallel.rs`.
- `shared-deref-sites-after.log` — every remaining `(*shared)` deref site.
  All twelve are now inside the SpinLock / ConditionVariable blocks (the
  expected residual surface), inside the `record_worker_counts` calls
  protected by the spinlock, and one `(*shared).participant_count` access
  that is the function-internal `EcHnswParallelBuildSharedHeader::new`
  initialization helper (not the worker-side dereference path that this
  slice targets).
- `diff.patch` — exact diff applied.
- `cargo-check-pg18.log` — `cargo check --no-default-features --features
  pg18` (lib smoke). Clean, no `unused_unsafe` warnings.

## Performance gate

Build hot path. Per the operator's rotation rule
(`feedback_coder_push_smoke_checks`, 2026-05-21), bench evidence is gathered
out-of-band. The structural change here does not alter:

- field semantics: every `header.field` access reads exactly the same byte
  the previous `(*shared).field` read did,
- worker count or scheduling: no change to participant_count, scan or
  callback wiring,
- locking: SpinLock and ConditionVariable use remains the same,
- allocations: no Rust heap allocations introduced; the helper returns a
  borrow not an owned value.

## Out of scope

- DSM atomic field views (`EcHnswParallelBuildSharedAtomicU32` and the
  per-node `lock` field) — queued as a future slice if the next inventory
  shows further block reduction is reachable there.
- Typed `shm_toc_lookup` wrapper that lifts the `*mut shm_toc` →
  `NonNull<T>` pattern across the worker entries — also queued as a future
  slice if needed.
- DiskANN/IVF/SPIRE — HNSW-only rotation per
  `392/2026-05-21-02-reviewer.md`.
