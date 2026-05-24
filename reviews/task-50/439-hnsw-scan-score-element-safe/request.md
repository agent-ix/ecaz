# Task 50/439: HNSW scan.rs — `live_loaded_state_from_exact_payload` + `score_scan_element_result` safe-fn lifts

## Why this slice

After slice 410 (`score_and_cache_scan_element`) and the broader
scan-side cascade through 415, both functions' bodies are composed
entirely of safe operations. Lifting each to a safe `fn` strips
three caller-side `unsafe { ... }` wraps and crosses the **-200**
rotation milestone.

## Scope

Two `unsafe fn` → safe `fn` lifts in `src/am/ec_hnsw/scan.rs`:

1. `live_loaded_state_from_exact_payload` — body delegates to
   `score_and_cache_scan_element` (safe since 410); no internal
   unsafe blocks.
2. `score_scan_element_result` — body uses `cached_quantizer_ref`,
   `scan_box_ref`, and quantizer score helpers; no internal unsafe
   blocks.

Caller-side `unsafe { ... }` wraps stripped:

- `score_and_cache_scan_element` internal call to
  `score_scan_element_result` (was `unsafe { ... }`).
- `build_cached_graph_element` dispatcher arm calling
  `live_loaded_state_from_exact_payload` (was `unsafe { ... }`).
- `miri_score_scan_element_result_via_raw_opaque_ptr_updates_stats_delta`
  test — the wrap is now tightened to scope just the
  `&mut *opaque_ptr` deref rather than the whole call.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 82 | 80 | -2 |
| **HNSW subsystem subtotal** | **351** | **349** | **-2** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 437 | 351 |
| After 439 | 349 |

**Net rotation delta: -200 in HNSW (-36.4%).**

## Soundness rationale

Both functions' bodies have zero internal `unsafe { ... }` blocks
after the prior cascade. The lift is pure signature.

`score_scan_element_result` test caller previously dereferenced
`opaque_ptr` through the function's `unsafe fn` contract; now the
deref is its own narrow `unsafe { &mut *opaque_ptr }` expression,
which more accurately scopes the raw-pointer safety obligation to
its source.

## Validation

Artifacts under `reviews/task-50/439-hnsw-scan-score-element-safe/artifacts/`:

- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Inner-loop scoring path; signature-only change. Bench evidence
gathered out-of-band per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**-200 (-36.4%)** on HNSW: 549 → 349. The -30% Exit Criteria
target now has a **6.4-point cushion**.
