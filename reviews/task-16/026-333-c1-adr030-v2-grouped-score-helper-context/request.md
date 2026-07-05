# Review Request: C1 ADR-030 V2 Grouped Score Helper Context

## Context

Packet `332` extracted a dedicated grouped scorer context builder:

- `grouped_score_context_from_scan_state(...)`

That centralized how grouped dispatch assembles:

1. metadata-derived grouped search shape
2. cached grouped hot payloads
3. element identity

But `CandidateScoreDispatch::Grouped(...)` still threw away that outer context and only passed the
inner `GroupedScoreCall<'a>` into the grouped helper stub.

## Problem

The future grouped scorer will need one stable handoff point that carries the full grouped scorer
context, not just the inner call payload. If dispatch strips `element_tid` and context framing now,
the first real grouped scorer packet would still need to widen the helper boundary again.

## Planned Slice

Move grouped helper dispatch from raw grouped calls to full grouped score context:

1. make grouped candidate dispatch carry `GroupedScoreContext<'a>`
2. make the grouped helper stub accept `GroupedScoreContext<'_>`
3. keep runtime behavior unchanged

This still excludes:

- no grouped-v2 traversal enablement
- no grouped approximate scorer
- no rerank fetch path
- no behavior change for grouped-v2 scans

## Implementation

Updated `src/am/scan.rs`:

1. changed `CandidateScoreDispatch::Grouped(...)` to carry `GroupedScoreContext<'a>`
2. changed `candidate_score_dispatch(...)` to forward the full grouped context instead of only
   `GroupedScoreCall<'a>`
3. replaced `score_grouped_candidate_input(...)` with `score_grouped_candidate_context(...)`
4. changed grouped dispatch call sites to route through the new grouped helper boundary
5. updated grouped dispatch assertions to check the dispatched `element_tid` as well as grouped
   shape and cached payload bytes

Validation updates:

- existing grouped score context tests still pass
- grouped dispatch test now verifies that helper-boundary context preserves `element_tid`

## Measurements

This packet is still scorer-seam work, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test candidate_score_dispatch_uses_grouped_input_for_exact_unavailable --lib`: passed
  - `cargo test grouped_score_context_uses_scan_shape_and_cached_payloads --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: one wide run hit the existing flaky
    `pg_test_tqhnsw_debug_reachable_live_count_matches_admin_snapshot`
  - isolated rerun of that flaky test: invalid due stale `postmaster.pid` from the failed run
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed on rerun
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

Grouped scan scoring now has one direct helper boundary that carries the full grouped score
context, rather than rebuilding or widening that context at the helper call site.

What this de-risks:

1. the first real grouped scorer can replace one helper instead of widening helper inputs again
2. helper-boundary grouped scoring now preserves element identity explicitly
3. grouped dispatch and grouped helper inputs are now aligned around one typed context object

## Next Slice

The next narrow slice should add a grouped scorer payload view derived from `GroupedScoreContext`:

1. derive borrowed grouped search-code and binary-sidecar slices from one helper-local view
2. keep runtime behavior unchanged
3. prepare the first real LUT-scoring implementation without reworking dispatch
