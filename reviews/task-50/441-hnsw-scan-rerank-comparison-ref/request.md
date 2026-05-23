# Task 50/441: HNSW scan.rs — `grouped_candidate_rerank_comparison_score` reference-pass lift

## Why this slice

`grouped_candidate_rerank_comparison_score` was an `unsafe fn`
taking `opaque: *mut TqScanOpaque` for legacy reasons: it used the
`scan_opaque_ref` / `scan_opaque_mut` anti-pattern B helpers
internally. Both callers (`buffer_grouped_graph_result_candidate`,
`materialize_graph_result_candidate`) already had
`&mut TqScanOpaque` in scope and were casting it to `*mut` purely
to satisfy the `unsafe fn` contract. The cast and the helpers are
unnecessary indirection — direct `&mut TqScanOpaque` works fine.

## Scope

One signature change in `src/am/ec_hnsw/scan.rs`:

- `grouped_candidate_rerank_comparison_score(*mut TqScanOpaque)` →
  `grouped_candidate_rerank_comparison_score(&mut TqScanOpaque)`,
  lifted to safe `fn`.

All five internal `scan_opaque_ref(opaque)` and
`scan_opaque_mut(opaque)` sites collapse to direct borrows of
`opaque`.

Caller-side `unsafe { ... }` wraps stripped (two):

- `buffer_grouped_graph_result_candidate`
- `materialize_graph_result_candidate`

The `scan_opaque_ref` / `scan_opaque_mut` helpers themselves
remain in the file (used by the other dispatcher sites in scan.rs
that still take `*mut TqScanOpaque`); they are not removed by this
slice.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 76 | 74 | -2 |
| **HNSW subsystem subtotal** | **345** | **343** | **-2** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 440 | 345 |
| After 441 | 343 |

**Net rotation delta: -206 in HNSW (-37.5%).**

## Soundness rationale

The function body no longer uses any raw-pointer operations. All
field reads/writes go through the `&mut TqScanOpaque` borrow. No
anti-pattern B — both callers pass a real `&mut TqScanOpaque`
borrow rather than re-deriving one from a raw pointer.

## Validation

Artifacts under `reviews/task-50/441-hnsw-scan-rerank-comparison-ref/artifacts/`:

- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Inner-loop scoring path; signature change is reference-pass
rather than raw-pointer-pass. Bench evidence gathered out-of-band
per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**-206 (-37.5%)** on HNSW: 549 → 343. The -30% Exit Criteria
target now has a **7.5-point cushion**.
