# Review Request: C1 ADR-030 V2 Grouped Score Helper Stub

## Context

Packet `329` added an explicit candidate-score dispatch seam:

1. scalar exact path
2. grouped path

But grouped dispatch still errored inline inside `score_cached_graph_element_dispatch(...)`.

That means the future grouped scorer still had no dedicated implementation boundary of its own.

## Problem

The next grouped-runtime packet should be able to replace one dedicated helper instead of editing the
dispatch match directly.

Without that helper, grouped score cutover would still be coupled to the dispatch function body.

We need one small slice that:

1. extracts grouped score handling into its own helper
2. keeps behavior unchanged
3. preserves the existing unsupported-runtime error for grouped inputs

## Planned Slice

Introduce a grouped scorer stub helper:

1. grouped dispatch forwards into the helper
2. the helper still returns `ADR030_GROUPED_V2_SCAN_UNSUPPORTED`
3. tests stay focused on dispatch shape rather than invoking the stub directly through the pgrx
   error path

This still excludes:

- no grouped-v2 traversal enablement
- no grouped approximate scorer
- no rerank fetch path
- no behavior change for grouped-v2 scans

## Implementation

Updated `src/am/scan.rs`:

1. added `score_grouped_candidate_input(...)`
2. changed grouped arms in `score_cached_graph_element_dispatch(...)` to call that helper
3. kept helper behavior unchanged: it still errors with
   `tqhnsw scan runtime does not support ADR-030 grouped-v2 indexes yet`

Testing note:

- I briefly added a direct unit test that called the helper and asserted on the panic payload
- that recreated the repo’s known pgrx/libtest linker edge, because direct invocation of the helper
  pulled the PostgreSQL error path into a plain libtest binary
- I removed that direct helper test and kept the dispatch-shape tests instead

The retained tests still validate the seam:

- grouped `ExactUnavailable` entries dispatch through grouped input
- scalar entries keep the exact path

## Measurements

This packet is still a seam packet, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test candidate_score_dispatch_uses_grouped_input_for_exact_unavailable --lib`: passed
  - `cargo test candidate_score_dispatch_keeps_scalar_loaded_state_exact --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 v2 now has a dedicated grouped scorer helper boundary, even though that helper is still a
stub.

What this de-risks:

1. the first real grouped scorer can replace one helper instead of editing dispatch logic and call
   sites together
2. grouped runtime behavior is now isolated behind a dedicated grouped score boundary
3. the next packet can focus entirely on grouped score-helper shape and inputs

## Next Slice

The next narrow slice should start shaping the helper around the eventual grouped LUT scorer:

1. define the grouped helper inputs more explicitly around `GroupedScoreInput`
2. add any preconditions or metadata-derived shape needed by a later LUT scorer
3. still keep the helper returning the existing unsupported-runtime error until the actual scorer is
   ready
