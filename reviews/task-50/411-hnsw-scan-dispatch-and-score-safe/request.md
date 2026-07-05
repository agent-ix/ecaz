# Task 50/411: HNSW scan.rs — dispatch + and_score safe-fn lifts

## Why this slice

After slice 410, `score_cached_graph_element_dispatch` and
`cached_graph_element_and_score` are the next-up `unsafe fn`s in the
scan.rs scoring chain. Both have **zero** internal `unsafe { ... }`
blocks after the earlier rotations (every callee is now safe). They
are `unsafe fn` purely because of `*mut TqScanOpaque` parameter shape.

Lifting both to safe `fn(&mut TqScanOpaque)`:
- removes 1 caller-side wrap on `score_cached_graph_element_dispatch`
  (the only caller is `cached_graph_element_and_score`),
- removes 2 caller-side wraps on `cached_graph_element_and_score`
  (both in `initialize_scan_entry_candidate`),
- internal body of `score_cached_graph_element_dispatch` simplifies:
  `scan_opaque_ref(opaque).scan_graph_storage` becomes
  `opaque.scan_graph_storage`, and the two trailing
  `scan_opaque_mut(opaque)` calls (passed to the inner exact and
  grouped scorers) become bare `opaque` passes — the borrow checker
  re-borrows in each branch arm.

## Scope

- `score_cached_graph_element_dispatch` → safe `fn(_, &mut TqScanOpaque, _, _, _)`.
- `cached_graph_element_and_score` → safe `fn(_, &mut TqScanOpaque, _, _)`.
- 3 caller-side `unsafe { ... }` wrappers removed.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 106 | 103 | -3 |
| **HNSW subsystem subtotal** | **495** | **492** | **-3** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 410 | 495 |
| After 411 | 492 |

Net rotation delta: **-57 in HNSW** (-10.4%). Past the 10% mark.

## Soundness rationale

Same pattern as 406-410. Both functions' bodies contain only safe
operations after the prior lifts; the conversion is pure signature
flip. Borrow check holds because each branch in the dispatcher takes
a single `&mut` reborrow at a time.

No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/411-hnsw-scan-dispatch-and-score-safe/artifacts/`.

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Scan hot path. No semantic change. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Out of scope

- `cached_graph_neighbors` (line ~3035): still `unsafe fn`. Body has
  `unsafe { graph::load_graph_neighbors(...) }` — same FFI shape as
  `cached_graph_element_from_buffer`. Next-slice candidate.
- `cached_graph_adjacency`, `prefetch_graph_buffers`,
  `cached_graph_element_with_prefetch`, `cached_scan_successor_candidates_for_layer`,
  `cached_upper_layer_seed_candidate`: similar lift candidates queued.
