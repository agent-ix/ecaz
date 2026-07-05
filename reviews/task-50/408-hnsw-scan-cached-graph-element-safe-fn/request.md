# Task 50/408: HNSW scan.rs — `cached_graph_element` → safe fn

## Why this slice

Continuation of the slice-406/407 pattern. `cached_graph_element` is the
top-level entry to the scan-side graph-tuple cache. It takes
`*mut TqScanOpaque` to support six callers — the function body uses
only safe operations after slice 406, except for one internal
`unsafe { graph::with_graph_storage_tuple(...) }` block that wraps an
already-bounded closure call on the tuple-storage helper.

Converting `cached_graph_element` to safe `fn` taking `&mut TqScanOpaque`
collapses the six caller-side `unsafe { ... }` wrappers without
introducing anti-pattern B (signature takes a reference, not a raw
pointer-to-reference helper).

## Scope

- `cached_graph_element` in `src/am/ec_hnsw/scan.rs` converted from
  `unsafe fn(pg_sys::Relation, *mut TqScanOpaque, page::ItemPointer)` to
  `fn(pg_sys::Relation, &mut TqScanOpaque, page::ItemPointer)`.
  Internal first-line `scan_opaque_mut(opaque)` removed; the parameter
  is already a borrow. The internal
  `unsafe { graph::with_graph_storage_tuple(...) }` block is preserved
  with the same SAFETY comment.
- 6 caller sites updated:
  - `score_cached_graph_element_dispatch` (line ~3047)
  - `cached_graph_adjacency` (line ~3105)
  - `cached_graph_element_with_prefetch` (line ~3189)
  - `buffer_grouped_graph_result_candidate` (line ~3911)
  - `prefetch_next_grouped_windowed_graph_result` (line ~3960)
  - `refine_grouped_frontier_head_exact` (line ~4576)
- At the last call site (`refine_grouped_frontier_head_exact`) the
  local `opaque_ptr = opaque as *mut TqScanOpaque` binding is moved
  *after* the `cached_graph_element` call so the `&mut` borrow on
  `opaque` for that call doesn't overlap with the later raw-pointer
  use in the `CandidateScoreDispatch::Exact` and grouped-exact branches.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 122 | 116 | -6 |
| **HNSW subsystem subtotal** | **511** | **505** | **-6** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 407 | 511 |
| After 408 | 505 |

Net rotation delta: **-44 in HNSW** (-8.0%).

## Soundness rationale

- `cached_graph_element` body keeps one internal
  `unsafe { graph::with_graph_storage_tuple(...) }` block whose SAFETY
  comment names the lifetime contract (the tuple view is consumed
  inside the closure before the lock guard is dropped). That's the
  same contract the previous `unsafe fn` signature pushed on callers;
  moving it inside the function does not weaken soundness because the
  fn-body retains the explicit `unsafe { }` annotation.
- New signature takes `&mut TqScanOpaque`. Callers acquire the
  borrow either by passing their own `opaque: &mut TqScanOpaque`
  parameter directly (for callers already at that shape) or via the
  existing pre-rotation `scan_opaque_mut(opaque)` helper.
- No new anti-pattern B introduced.

## Validation

Artifacts under
`reviews/task-50/408-hnsw-scan-cached-graph-element-safe-fn/artifacts/`:

- `manifest.md` — head SHA, files touched, validation mapping.
- `per-file-after.log` — post-change HNSW per-file block counts.
- `diff.patch` — exact diff applied (90 lines).
- `cargo-check-pg18.log` — `cargo check --no-default-features --features
  pg18` (lib smoke). Clean, no `unused_unsafe` warnings.

## Performance gate

Scan hot path. No semantic change: every cache lookup, every `Arc::new`
allocation, every record_* call runs identically. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Out of scope

- `cached_graph_element_from_buffer` (pg18 feature variant) — 1 caller,
  same shape, can be lifted in a follow-on slice.
- `exact_score_cached_graph_element` and the rest of the
  cached_graph_* family — bigger lift; queued.
- Storage-side `graph::with_graph_storage_tuple` — still `unsafe fn`,
  irreducible boundary for now (calls into FFI page tuple reader).
