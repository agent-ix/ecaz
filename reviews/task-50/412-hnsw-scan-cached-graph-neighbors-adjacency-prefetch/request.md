# Task 50/412: HNSW scan.rs — cached_graph neighbor/adjacency/prefetch lifts

## Why this slice

Continuation of the safe-fn lift rotation. Three more functions in
the `cached_graph_*` chain that were the next-up `unsafe fn`s after
slice 411:

- `cached_graph_neighbors` — has one internal
  `unsafe { graph::load_graph_neighbors(...) }` block (FFI page reader).
- `cached_graph_adjacency` — composes `cached_graph_element` +
  `cached_graph_neighbors`. After slice 408 and the lift above, the
  body has zero internal unsafe blocks.
- `cached_graph_element_with_prefetch` — composes
  `cached_graph_element_from_buffer` + `cached_graph_element`. Body
  has zero internal unsafe blocks after slice 409.

All three converted to safe `fn(&mut TqScanOpaque, ...)`. Five
caller-side `unsafe { ... }` wrappers removed (1 on neighbors, 1 on
adjacency, 2 on with_prefetch, plus 1 stale wrap inside the lifted
adjacency body).

## Scope

- `cached_graph_neighbors` lifted to safe `fn`. Internal
  `unsafe { graph::load_graph_neighbors(...) }` block preserved with
  its SAFETY comment.
- `cached_graph_adjacency` lifted to safe `fn`. Body uses the borrow
  parameter directly; the previous `unsafe { cached_graph_neighbors(...) }`
  wrap and the `scan_opaque_mut(opaque)` call go away.
- `cached_graph_element_with_prefetch` lifted to safe `fn`. Two
  `scan_opaque_mut(opaque)` calls in the body removed; parameter is
  already a borrow.
- 5 caller-side wraps removed across scan.rs.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 103 | 99 | -4 |
| **HNSW subsystem subtotal** | **492** | **488** | **-4** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 411 | 492 |
| After 412 | 488 |

Net rotation delta: **-61 in HNSW** (-11.1%).

## Soundness rationale

Same pattern as 406-411. `cached_graph_neighbors` retains one
internal unsafe block (FFI page reader); the other two are pure
safe bodies after this rotation's prior lifts. New signatures take
references, not raw pointers. No anti-pattern B.

## Validation

Artifacts under
`reviews/task-50/412-hnsw-scan-cached-graph-neighbors-adjacency-prefetch/artifacts/`.

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

- `cached_scan_successor_candidates_for_layer<KeepFn>` (line ~3141):
  large generic function with a `KeepFn: FnMut(page::ItemPointer) -> bool`
  bound. Still `unsafe fn`; lift requires careful borrow accounting
  around the closure bound. Queued.
- `cached_upper_layer_seed_candidate` (line ~3422): similar shape.
- `prefetch_graph_buffers` (line ~3068): already takes `&mut TqScanOpaque`
  but still `unsafe fn` because of pg18 prefetch FFI. Body has its
  own `unsafe { ... }` block around `pg_sys::PrefetchBuffer`.
