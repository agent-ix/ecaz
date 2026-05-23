# Task 50/402: HNSW typed `shm_toc_lookup_required` helper

## Why this slice

After slice 401, `src/am/ec_hnsw/build_parallel.rs` still carries 8 worker-
and leader-side `unsafe { pg_sys::shm_toc_lookup(...) }` blocks that all
follow the same shape:

```rust
let typed = unsafe {
    pg_sys::shm_toc_lookup(toc, KEY, false).cast::<T>()
};
```

`shm_toc_lookup(..., noerror = false)` is documented to make PostgreSQL
ereport when the key is missing, so the returned pointer is non-null on a
successful return. The `noerror = true` path is not used in HNSW — every
HNSW lookup is "required" — so a single typed helper centralizes the lookup
+ cast pattern and removes the call-site `unsafe { ... }` wrapper at every
site.

Per Task 50 §Techniques, this is technique 1 (encapsulate at the FFI
boundary): the FFI call lives inside the helper, and every caller becomes
safe.

## Scope

- Added module-private `shm_toc_lookup_required<T>(toc, key) -> *mut T`
  helper inside `src/am/ec_hnsw/build_parallel.rs`, placed alongside the
  `shared_header_ref` helper from slice 401.
- Converted 8 caller sites in the two worker entrypoints
  (`parallel_build_worker_main` and `parallel_graph_build_worker_main`) to
  call the typed helper. The let-bindings keep an explicit type annotation
  so the generic is inferred at the call site without unsafe.

Out of scope:

- The leader-side `shm_toc_lookup` at `build_parallel.rs:~2613`, which is
  inside a larger `unsafe { ... }` block that also reads `(*pcxt).toc`,
  `(*pcxt).worker.add(...)`, `shm_mq_attach`, `(*worker_info).bgwhandle`,
  and calls `WaitForParallelWorkersToAttach`. That block is one structural
  unit that has to be lifted as a single slice; not the typed-lookup target.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 123 | 116 | -7 |
| **HNSW subsystem subtotal** | **535** | **528** | **-7** |

Breakdown:

- Removed: 8 `unsafe { pg_sys::shm_toc_lookup(...) }` caller blocks across
  the two worker entrypoints.
- Added: 1 `unsafe { pg_sys::shm_toc_lookup(toc, key, false) }` block
  inside `shm_toc_lookup_required`.
- Net: -7.

Per Task 50 §Slice rules:

- Helper call-site count: 8 (well past the ≥2 threshold).
- Documentation-only changes are out of scope: ✓ — structural.

## Validation

Artifacts under `reviews/task-50/402-hnsw-shm-toc-lookup-typed/artifacts/`:

- `manifest.md` — head SHA, lane, command, timestamps, validation mapping.
- `per-file-after.log` — post-change HNSW per-file block counts.
- `build-parallel-unsafe-block-lines-after.log` — line-by-line listing of
  every remaining `unsafe { ... }` block in build_parallel.rs.
- `shm-toc-lookup-sites-after.log` — every remaining `shm_toc_lookup`
  reference. Only the helper definition and one leader-side block lookup
  (out of scope, see above) remain. All eight worker-side call sites use
  `shm_toc_lookup_required` and are safe.
- `diff.patch` — exact diff applied.
- `cargo-check-pg18.log` — `cargo check --no-default-features --features
  pg18` (lib smoke). Clean, no `unused_unsafe` warnings.

## Performance gate

Build hot path. Per the operator's rotation rule
(`feedback_coder_push_smoke_checks`, 2026-05-21), bench evidence is gathered
out-of-band. The change does not alter:

- candidate ordering, scoring, or recall semantics (worker setup only),
- worker launch behavior (same `shm_toc_lookup` arguments, same
  `noerror = false` semantics),
- DSM allocation shape (no Rust heap allocations introduced),
- WAL ordering (no WAL touched).

## Out of scope

- Leader-side TOC lookup at the parallel-leader attach loop — lifted as a
  separate slice if needed.
- DSM atomic field views (`EcHnswParallelBuildSharedAtomicU32`) — queued.
- DiskANN/IVF/SPIRE — HNSW-only rotation per
  `392/2026-05-21-02-reviewer.md`.
