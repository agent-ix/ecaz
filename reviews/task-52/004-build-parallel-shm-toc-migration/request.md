# Task 52 / 004 — shm_toc Consumer Migration in build_parallel.rs

Branch: `task-52`
Code path: `src/am/ec_hnsw/build_parallel.rs`

## Summary

Migrate the four leader/worker `shm_toc_*` chains in
`src/am/ec_hnsw/build_parallel.rs` over to the `ShmTocBuilder<'a>` /
`ShmTocReader<'a>` wrappers landed in slice 002. Retire the local
`shm_toc_lookup_required<T>` helper. Wrapper-side counts unchanged;
consumer-side count drops by 5.

This is a pure consumer migration. No new abstractions; no behavior
change; no semantic difference from the prior FFI chain.

## Per-file `unsafe { ... }` block delta

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 112 | **107** | **-5** |

Diff stat: `+64 / -139` (75 lines net deletion). Other slice-52 files
unchanged.

See `artifacts/manifest.md` for the per-site delta accounting that
adds to -5.

## What changed

### Helper retirement
`fn shm_toc_lookup_required<T>(toc, key) -> *mut T` (private safe
helper at lines 1989-2001) deleted. Its 9 call sites now go through
`ShmTocReader::lookup_required<T>` which has identical
`noerror = false` PG-ereport semantics (slice 002 mirrors the helper
contract exactly).

### Worker functions (heap-scan + graph-build)
Each worker entry now opens with a single Reader attachment:
```rust
// SAFETY: `toc` is the worker's attached TOC handed in by PostgreSQL;
// the backing DSM segment outlives this worker frame.
let reader = unsafe { ShmTocReader::attach(toc) };
let shared: *mut EcHnswParallelBuildSharedHeader =
    reader.lookup_required(PARALLEL_KEY_EC_HNSW_BUILD_SHARED);
```
All subsequent `shm_toc_lookup_required(toc, KEY)` calls become safe
`reader.lookup_required(KEY)`. Anti-pattern B respected: `lookup_required`
returns `*mut T`; the worker stamps the type/init contract inline via
the existing `ptr::NonNull::new(shared).unwrap_or_else(...).as_ref()`
pattern (unchanged).

### Leader-side heap-scan setup
Single Builder construction near the top of the leader scope:
```rust
// SAFETY: After InitializeParallelDSM, `(*pcxt).toc` is the leader's
// live TOC; the backing DSM segment outlives this leader scope.
let builder = unsafe { ShmTocBuilder::new((*pcxt).toc) };
```
Subsequently:
- `shm_toc_allocate(...).cast::<T>()` → `builder.allocate_typed<T>(size)`
  (safe).
- `shm_toc_allocate(...)` (untyped) → `builder.allocate_bytes(size)`
  (safe).
- `shm_toc_insert(toc, KEY, ptr.cast())` → `builder.insert(KEY, ptr)`
  (safe; `insert<T>` does the cast internally).

The 4 standalone allocate blocks (shared / queues / walusage /
bufferusage) and the 1 standalone 2-insert block all become safe
expressions. The big "init shared header" unsafe block keeps its
`ptr::write` + `LWLockInitialize` + `ConditionVariableInit` +
`SpinLockInit` + `table_parallelscan_initialize` calls (slice 005
absorbs the SpinLockInit + ConditionVariableInit pair via
`EcHnswParallelBuildSharedView::init_synchronization`).

### Leader-side graph-build setup
Same shape as heap-scan. Notable: the original 3-call block at lines
2287-2294 (`2x shm_toc_insert + LaunchParallelWorkers`) splits — the
2 inserts go safe via builder, and `LaunchParallelWorkers` survives as
its own one-call `unsafe { }` block. Net 0 for that block (collapse
not possible without consolidating Launch into a typed wrapper, which
is out of scope).

## Anti-pattern B / view-operations discipline

The 5th application of `feedback_anti_pattern_b_unbounded_lifetime`
(builder/reader return `*mut T` raw pointers) and the 2nd application
of `feedback_view_operations_not_accessors` (no `&Header` accessors
introduced — workers continue to stamp the type/init contract inline
at the same NonNull check point they already used).

## Out-of-scope (deferred)

- **Leader queue-rebind** at line ~2517 (`pg_sys::shm_toc_lookup`):
  not migrated. It sits inside an existing large unsafe block that
  retains its other ops (`shm_mq_attach`, `(*pcxt).worker.add`,
  `WaitForParallelWorkersToAttach`); routing it through a Reader
  would require adding a fresh `unsafe { ShmTocReader::attach }`
  block on top with no compensating reduction. Net would be +1 unsafe
  block — declined for honest accounting.
- **SpinLock+CV compounds** and the leader-side `SpinLockInit +
  ConditionVariableInit` pair are slice 005.
- **Residual `(*shared).field` / `(*pcxt).field` derefs** are slice 006.

## Validation

- `cargo fmt --all` — clean (touched only `build_parallel.rs`).
- `cargo check --no-default-features --features pg18` — `Finished`
  exit 0, 14.80s incremental.
- `cargo clippy ... -- -D warnings` — not re-run; same crate-wide
  rabitq backlog from main-merge, documented in slice 002's manifest.
- `cargo pgrx test` — skipped per memory
  `feedback_dyld_buffer_blocks_known`. No PG callback behavior touched;
  worker entrypoints unchanged in shape — they just route their TOC
  reads through a Reader instead of a safe helper, with identical
  `noerror = false` semantics.

## Cross-references

- Wrappers consumed: `src/am/common/dsm.rs::{ShmTocBuilder,
  ShmTocReader}` (slice 002).
- Slice 005 (SpinLock+CV compound migration) follows.
- Pre-state baseline:
  `reviews/task-52/001-execution-planning/artifacts/baseline-unsafe-density.txt`.
