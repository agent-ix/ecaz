# Task 52 / 005 — SpinLock+CV Compound Migration · Artifact Manifest

Packet path: `reviews/task-52/005-spinlock-cv-compound-migration/`
Branch: `task-52`

## Surfaces

- `src/am/ec_hnsw/build_parallel.rs` — consumer migration.
- `src/am/ec_hnsw/parallel_build_view.rs` — wrapper extension: 6 new
  `Copy`-field operation methods (participant_count, requested_workers,
  is_concurrent, heaprelid, indexrelid, scanned_heap_tuples,
  encoded_index_tuples).

## Per-file before/after `unsafe { ... }` blocks

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 107 | **105** | **-2** |
| `src/am/ec_hnsw/parallel_build_view.rs` | 6 | 12 | +6 (wrapper-side) |
| `src/am/common/dsm.rs` | 13 | 13 | 0 |

The -2 are the two worker-side SpinLock+CV compounds replaced by
`view.record_workers_done(...)`. Each compound was a 4-call unsafe
block (`SpinLockAcquire` + `record_worker_counts(&mut)` + `SpinLockRelease`
+ `ConditionVariableSignal`); the view's safe method absorbs all four.

The +6 wrapper-side blocks are per-method `unsafe {
(*self.header.as_ptr()).field() }` operations on the view — required
because each accessor must be its own function (Copy field reads
operation-style per memory `feedback_view_operations_not_accessors`).

## Honest accounting — leader-side init compounds not yet migrated

The leader-side `SpinLockInit + ConditionVariableInit` pair at
both leader entry points sits inside an existing larger unsafe block
that also contains `ptr::write(shared, ...)` and
`pg_sys::table_parallelscan_initialize(...)`. Substituting in
`view.init_synchronization()` removes the two PG-FFI ops from inside
the block but the block itself stays (other unsafe ops remain). Net
block count change: 0. Migration deferred to slice 006 where the
`ptr::write` site can be lifted into a typed initializer that
encapsulates init_synchronization + the parallel scan initialize.

## Artifacts

- Head SHA: parent of packet commit.
- Lane / fixture / storage / rerank: N/A (compile-only).
- Isolation: N/A.
- Command (validation):
  - `cargo fmt --all` — clean.
  - `cargo check --no-default-features --features pg18` — `Finished`
    exit 0, 14.75s incremental.
  - `cargo clippy ... -- -D warnings` — not re-run this slice; same
    pre-existing rabitq backlog documented in slice 002 manifest.
  - `cargo pgrx test` — skipped per memory
    `feedback_dyld_buffer_blocks_known`. The worker tail behavior is
    semantics-preserving: `record_workers_done` invokes the same PG
    primitives in the same order (SpinLockAcquire → record_worker_counts
    → SpinLockRelease → ConditionVariableSignal).
- Timestamp: 2026-05-23.
