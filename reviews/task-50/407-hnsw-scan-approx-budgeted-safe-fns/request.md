# Task 50/407: HNSW scan.rs — lift approx + budgeted scoring to safe fn

## Why this slice

Continuation of the slice-406 pattern: convert internal `unsafe fn`s in
`scan.rs` to safe `fn`s taking `&TqScanOpaque` / `&mut TqScanOpaque`
references. The two new conversions in this slice are the leftover
members of the grouped-score family that slice 406 deferred:

- `score_grouped_candidate_context_approx` — body now contains zero
  internal `unsafe { ... }` blocks (slice 406 already made every
  callee safe), and only takes `*mut TqScanOpaque` to call
  `scan_opaque_ref` / `scan_opaque_mut`. Convert to safe `fn(&mut
  TqScanOpaque, ...)`.
- `score_budgeted_grouped_traversal_candidates` — same shape; body
  uses only safe operations after slice 406, conversion is purely
  signature.

Same anti-pattern-B-safe approach as 406: new signatures take
references, not raw pointers.

## Scope

Two functions in `src/am/ec_hnsw/scan.rs` converted:

1. `score_grouped_candidate_context_approx` → safe, takes `&mut TqScanOpaque`.
2. `score_budgeted_grouped_traversal_candidates` → safe, takes `&mut TqScanOpaque`.

Caller-side rewrites:

- 3 caller sites of `score_grouped_candidate_context_approx` drop their
  `unsafe { ... }` wrappers and pass `scan_opaque_mut(opaque)`. One
  caller location (the bottom of `score_grouped_candidate_context`) is
  no longer the final-tail `unsafe { ... }` block, so it also gets the
  wrapper removed cleanly.
- 2 caller sites of `score_budgeted_grouped_traversal_candidates` drop
  their `unsafe { ... }` wrappers.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 127 | 122 | -5 |
| **HNSW subsystem subtotal** | **516** | **511** | **-5** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 406 | 516 |
| After 407 | 511 |

Net rotation delta: **-38 in HNSW** (-6.9%).

## Soundness rationale

Identical to slice 406:

- `score_grouped_candidate_context_approx` body uses only safe
  operations after slice 406 (`score_grouped_search_code_from_scan_state`
  and `record_grouped_traversal_approx_score_elapsed` are both safe).
  Taking `&mut TqScanOpaque` directly is sound.
- `score_budgeted_grouped_traversal_candidates` body reads
  `opaque.scan_graph_storage`, calls `record_grouped_traversal_budget`,
  and calls `score_grouped_candidate_context_exact` — all safe after
  slice 406. Taking `&mut TqScanOpaque` is sound.
- Callers obtain the borrow via `scan_opaque_mut(opaque)` (existing
  pre-rotation `unsafe fn` whose `&'a` return is anti-pattern B but
  *pre-existing*, not newly introduced by this rotation; future slice
  could replace with frame-bounded inline `&mut *opaque`).

## Validation

Artifacts under
`reviews/task-50/407-hnsw-scan-approx-budgeted-safe-fns/artifacts/`:

- `manifest.md` — head SHA, files touched, validation mapping.
- `per-file-after.log` — post-change HNSW per-file block counts.
- `diff.patch` — exact diff applied (163 lines).
- `cargo-check-pg18.log` — `cargo check --no-default-features --features
  pg18` (lib smoke). Clean.

## Performance gate

Scan hot path. Same disposition as 406: no semantic change, no
allocation, no scoring math change. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Out of scope

- `cached_graph_element` family — still `unsafe fn` because of
  `read_page_tuple` / `with_page_tuple_bytes` calls in the chain.
  Bigger lift, queued.
- `prefetch_*` / `produce_next_*` functions — same family pattern.
- `scan_opaque_ref` / `scan_opaque_mut` themselves — pre-existing
  anti-pattern B helpers. Replacing with frame-bounded inline borrows
  across all current call sites is a separate slice.
