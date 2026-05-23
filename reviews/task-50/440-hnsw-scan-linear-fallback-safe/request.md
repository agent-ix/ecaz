# Task 50/440: HNSW scan.rs — linear-fallback chain safe-fn lifts

## Why this slice

The linear-fallback selection chain
(`select_linear_scan_result_from_buffer` → `select_next_linear_scan_result`
→ `produce_next_linear_fallback_heap_tid`) was previously gated as
`unsafe fn` because of historical assumption that `LockedBufferGuard`
acquisition and buffer-page reads required unsafe contracts. After
prior slices made `score_scan_element_result` and
`with_page_line_tuple_bytes` safe, these functions' bodies are
composed of safe operations on the pg18 path. The pg17 branch
retains its `LockedBufferGuard::read_main` `unsafe { ... }` block as
an internal narrowed scope.

## Scope

Three `unsafe fn` → safe `fn` lifts in `src/am/ec_hnsw/scan.rs`:

1. `select_linear_scan_result_from_buffer` — body composed of safe
   ops only.
2. `select_next_linear_scan_result` — pg18 stream-visit arm has
   zero unsafe blocks; pg17 arm has one narrow internal unsafe
   block for buffer acquisition.
3. `produce_next_linear_fallback_heap_tid` — body composed of safe
   ops only after the chain lifts.

Caller-side `unsafe { ... }` wraps stripped (four):

- `select_next_linear_scan_result` pg18 stream-visit caller of
  `select_linear_scan_result_from_buffer`.
- `select_next_linear_scan_result` pg17 buffer-loop caller of
  `select_linear_scan_result_from_buffer`.
- `produce_next_linear_fallback_heap_tid` call to
  `select_next_linear_scan_result`.
- `produce_next_scan_heap_tid` `LinearFallback` dispatcher arm.

The `GraphTraversal` dispatcher arm in `produce_next_scan_heap_tid`
remains an `unsafe { ... }` block because the
`ensure_prefetched_output → prefetch_next → prefetch_next_graph_result_from_frontier`
chain is still `unsafe fn`. That cascade is the natural next slice
target.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 80 | 76 | -4 |
| **HNSW subsystem subtotal** | **349** | **345** | **-4** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 439 | 349 |
| After 440 | 345 |

**Net rotation delta: -204 in HNSW (-37.2%).**

## Soundness rationale

All three lifted functions have zero internal unsafe blocks on
pg18 after the prior cascade. The pg17 branch of
`select_next_linear_scan_result` retains its
`LockedBufferGuard::read_main` `unsafe { ... }` block as a narrow
internal scope. The lifts are pure signature.

No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/440-hnsw-scan-linear-fallback-safe/artifacts/`:

- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Linear-scan fallback path; signature-only change. Bench evidence
gathered out-of-band per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**-204 (-37.2%)** on HNSW: 549 → 345. The -30% Exit Criteria
target now has a **7.2-point cushion**.
