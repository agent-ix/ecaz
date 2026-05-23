# Task 52 / 005 — SpinLock+CV Compound Migration (Worker Side)

Branch: `task-52`

## Summary

Migrate the two worker-side `SpinLockAcquire + record_worker_counts(&mut)
+ SpinLockRelease + ConditionVariableSignal` four-call compounds in
`src/am/ec_hnsw/build_parallel.rs` over to the slice-003 view method
`view.record_workers_done(scan_delta, encoded_delta)`. Worker entry
points now construct the view via `unsafe { EcHnswParallelBuildSharedView::from_raw(shared) }`
(after a null check) and consume `view.validate()`, `view.is_concurrent()`,
`view.heaprelid()`, `view.indexrelid()`, and `view.participant_count()`
as operation-style accessors.

The view wrapper extension adds 6 `Copy`-field accessor methods
(operations, not `&T` accessors — memory rule
`feedback_view_operations_not_accessors`).

## Per-file `unsafe { ... }` block delta

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 107 | **105** | **-2** |
| `src/am/ec_hnsw/parallel_build_view.rs` | 6 | 12 | +6 (wrapper-side) |

Per-file inventory available in `artifacts/manifest.md`.

## What changed

### Worker entry points

Both `parallel_build_worker_main` and `parallel_graph_build_worker_main`
now follow this shape:
```rust
let reader = unsafe { ShmTocReader::attach(toc) };
let shared: *mut EcHnswParallelBuildSharedHeader =
    reader.lookup_required(PARALLEL_KEY_EC_HNSW_BUILD_SHARED);
if shared.is_null() {
    pgrx::error!("...");
}
// SAFETY: leader installed `shared` before launching workers; DSM
// segment outlives this worker frame.
let view = unsafe { EcHnswParallelBuildSharedView::from_raw(shared) };
view.validate();
// ... worker work using view.is_concurrent(), view.heaprelid() etc ...
view.record_workers_done(scan_delta, encoded_delta);
```

The prior shape used `ptr::NonNull::new(shared).unwrap_or_else(...).as_ref()`
to obtain `&Header`, followed by direct field reads
(`header.is_concurrent`, `header.heaprelid`, `header.indexrelid`,
`header.participant_count`). Those field reads sat in the same
module so they were syntactically safe but they relied on `&Header`
escaping into the worker body — anti-pattern B's underlying concern.
The view methods route every read through an inlined unsafe deref
that doesn't escape.

The trailing 4-call SpinLock+CV unsafe block:
```rust
unsafe {
    pg_sys::SpinLockAcquire(&mut (*shared).mutex);
    (*shared).record_worker_counts(scanned_tuples, encoded as f64);
    pg_sys::SpinLockRelease(&mut (*shared).mutex);
    pg_sys::ConditionVariableSignal(&mut (*shared).workersdonecv);
}
```
becomes:
```rust
view.record_workers_done(scanned_tuples, encoded as f64);
```
Behavior-preserving: `record_workers_done` acquires the spinlock via
the slice-447 `SpinLockGuard` (Drop releases), invokes the same
`record_worker_counts(&mut self, ...)` under the guard, and signals
the CV after the guard drops.

### Header impl additions

Added 5 `Copy`-field getter methods on `EcHnswParallelBuildSharedHeader`:
`participant_count(&self) -> u16`, `requested_workers(&self) -> u16`,
`is_concurrent(&self) -> bool`, `heaprelid(&self) -> pg_sys::Oid`,
`indexrelid(&self) -> pg_sys::Oid`. They mirror the existing
`scanned_heap_tuples` and `encoded_index_tuples` accessors.

### View impl additions

Added 6 wrapper methods (operation-style, each `unsafe {
(*self.header.as_ptr()).field() }`): participant_count, is_concurrent,
heaprelid, indexrelid, scanned_heap_tuples, encoded_index_tuples.
Each adds 1 wrapper-side unsafe block. Net wrapper-side: +6.

## Anti-pattern B / view-operations discipline

5 prior applications:
1. `feedback_anti_pattern_b_unbounded_lifetime` blocked safe
   `*mut T -> &'a T` on `ShmTocReader::lookup_*`.
2. `feedback_view_operations_not_accessors` blocked the
   `EcHnswParallelBuildSharedView::header() -> &'a Header` accessor
   in slice 003 refactor.

This slice's 6th and 7th applications:
3. The view's new `Copy`-field methods return values, not references.
   No `fn(&self) -> &'a Field` accessors authored.
4. Worker entries call `view.is_concurrent()`, `view.heaprelid()`,
   etc. — operation-style — instead of `header.is_concurrent`,
   `header.heaprelid` field reads on an escaped `&Header`. The
   `&Header` value no longer escapes the view; it's confined to the
   view's method bodies.

## What's not in this slice

### Leader-side init compounds (deferred to slice 006)
Both leader entries have a `unsafe { ptr::write + ConditionVariableInit
+ SpinLockInit + table_parallelscan_initialize }` block. Substituting
`view.init_synchronization()` only removes the two PG-FFI ops from
inside the block; the block stays because `ptr::write` and
`table_parallelscan_initialize` are still unsafe ops. Net block count
change: 0. Slice 006 will tackle this via a typed
`initialize_in_place(...)` constructor on the view that absorbs the
whole sequence into a single unsafe call site at the leader scope.

### (*pcxt).field standalone reads (slice 006)
`unsafe { (*pcxt).seg.is_null() }` ×2, `unsafe { (*pcxt).nworkers_launched }`
×2 are 4 single-op standalone unsafe blocks. A small `ParallelContextRef<'a>`
wrapper can replace them; slice 006 will add it.

## Toward ≤ 80

Current: 105. Target: ≤ 80. Remaining: -25 blocks. Slice 006 plan:
- ParallelContextRef wrapper: -4 standalone + N for accessor methods,
  net -2 to -3.
- View `initialize_in_place` constructor: -2 (one per leader).
- DSM graph-image accessors (`EcHnswConcurrentDsmGraphParts` impl
  has ~6 small unsafe blocks per accessor) — these are P8-adjacent
  but mostly graph-image not shared-header. Possible -3 to -5 via
  consolidation.
- `(*shared).field` deref residuals in leader init: -2 to -4 if the
  initialize_in_place path is taken.
- `parallel_table_scan_from_shared` consolidation, queue-rebind
  factoring: -2 to -4.

Estimated slice 006 ceiling: -15 to -20. Closing summary slice 007:
final residuals enumerated, bench gate vs `benchmarks/task-50-m5-hnsw-baseline/`.

If slice 006 lands ~-15, ending state is ~90 — short of ≤ 80. May
need a slice 006b for further migration before closeout, or revised
discussion of the structural ceiling.

## Validation

- `cargo fmt --all` — clean.
- `cargo check --no-default-features --features pg18` — `Finished`
  exit 0, 14.75s incremental.
- `cargo clippy ... -- -D warnings` — not re-run; pre-existing rabitq
  backlog unchanged.
- `cargo pgrx test` — deferred per `feedback_dyld_buffer_blocks_known`.

## Cross-references

- View wrapper: `src/am/ec_hnsw/parallel_build_view.rs` (slice 003).
- Slice 002 ShmToc wrappers: `src/am/common/dsm.rs`.
- Reviewer's premature-close rebuttal:
  `reviews/task-52/004-build-parallel-shm-toc-migration/feedback/2026-05-23-03-coder.md`.
- Memory rules applied: `feedback_anti_pattern_b_unbounded_lifetime`,
  `feedback_view_operations_not_accessors`,
  `feedback_no_premature_task_close`.
