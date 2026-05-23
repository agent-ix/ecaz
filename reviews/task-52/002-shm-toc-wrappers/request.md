# Task 52 / 002 — ShmTocBuilder + ShmTocReader

Branch: `task-52`
Code commit: `e2bade4e9 Task 52/002: ShmTocBuilder + ShmTocReader wrappers`

## Summary

Extend the P8 wrapper module `src/am/common/dsm.rs` (slice-447 base)
with the leader-side and worker-side typed wrappers over PostgreSQL's
`shm_toc` API named in `plan/tasks/52-...md` §Scope items 3 and 4.

This is the first of two wrapper-only slices. No consumer migration in
`build_parallel.rs` yet — that lands in slice 004.

## Per-file `unsafe { ... }` block deltas

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/common/dsm.rs` | 9 | 13 | +4 |
| `src/am/ec_hnsw/build_parallel.rs` | 112 | 112 | 0 |

The +4 wrapper-side blocks are the PG FFI calls inside the wrapper
methods. Each one substitutes for an N-many consumer-side
`unsafe { ... }` block; slice 004 will collapse the corresponding
consumer-side surface.

`src/` total: tracked in the closeout, not per slice.

## Surface added

In `src/am/common/dsm.rs`:

- `ShmTocBuilder<'a>` — leader-side wrapper.
  - `unsafe fn new(*mut shm_toc) -> Self` — single DSM-lifetime contract site.
  - safe `allocate_bytes(Size) -> *mut c_void`
  - safe `allocate_typed<T>(Size) -> *mut T` — convenience for the
    common `.cast::<T>()` chain.
  - safe `insert<T>(key: u64, *mut T)`
- `ShmTocReader<'a>` — worker-side wrapper.
  - `unsafe fn attach(*mut shm_toc) -> Self` — DSM-lifetime contract.
  - safe `lookup_raw<T>(key) -> *mut T` (returns null on missing).
  - safe `lookup_required<T>(key) -> *mut T` (PG ereports on missing —
    same `noerror=false` semantics as the local
    `shm_toc_lookup_required` helper in `build_parallel.rs:1997`, which
    slice 004 retires).

## Divergence from task-spec wording — flagged

The task spec §Scope #4 names `lookup<T>(key) -> &T` /
`lookup_mut<T>(key) -> &mut T`. The wrapper instead exposes raw-pointer
returns. Reason: memory rule
`feedback_anti_pattern_b_unbounded_lifetime` (2026-05-22) blocks safe
`*mut T -> &'a T` conversions at the wrapper layer — the wrapper's
constructor contract only covers "the toc and DSM segment are live",
not "key X has been inserted as type T and is initialized". Those
additional per-key invariants belong at the call site, where they are
visible. Slice 004 consumer migration therefore takes the form

```rust
let header_ptr: *mut EcHnswParallelBuildSharedHeader =
    reader.lookup_required(PARALLEL_KEY_EC_HNSW_BUILD_SHARED);
// SAFETY: the leader installed this key as
// EcHnswParallelBuildSharedHeader and the segment outlives the worker
// body.
let header = unsafe { NonNull::new_unchecked(header_ptr).as_ref() };
```

— stating the type/initialization contract inline at the boundary
between PG bookkeeping and Rust references.

## Validation

- `cargo fmt --all` — clean.
- `cargo check --no-default-features --features pg18` — `Finished` exit
  0 (background id `bnfbmkhm5`, 11m 27s from-scratch debug rebuild).
- Clippy on the touched module — to be delegated as a parallel
  validation, recorded in this packet's `artifacts/` if any warnings
  surface.
- No `cargo pgrx test` in this slice; no callbacks touched, dyld
  `_BufferBlocks` blocker per memory `feedback_dyld_buffer_blocks_known`
  remains active on macOS regardless.

## Out of scope

- No `build_parallel.rs` consumer migration. Slice 004.
- No `EcHnswParallelBuildSharedView`. Slice 003.
- No bench. Bench window opens at task close (slice 007).

## Cross-references

- Base module: `src/am/common/dsm.rs` opened in
  `reviews/task-50/447-p8-dsm-typed-wrappers/`.
- Pre-state: `reviews/task-52/001-execution-planning/artifacts/baseline-unsafe-density.txt`.
