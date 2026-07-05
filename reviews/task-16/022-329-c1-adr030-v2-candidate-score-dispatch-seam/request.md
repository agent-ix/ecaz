# Review Request: C1 ADR-030 V2 Candidate Score Dispatch Seam

## Context

Packet `328` added `GroupedScoreInput<'a>` so grouped-v2 hot payloads can be presented to a future
grouped scorer through one typed cache-level input seam.

The next missing boundary was the score-dispatch site itself. Candidate scoring still called the
scalar exact-score path directly, and grouped-v2 unsupported behavior was expressed only as local
error branches inside that exact-score path.

## Problem

Before this slice, the score boundary still looked scalar-first:

1. `cached_graph_element_and_score(...)` called `exact_score_cached_graph_element(...)`
2. binary-prefilter survivor reranking also called `exact_score_cached_graph_element(...)`
3. grouped-v2 score attempts only failed because the scalar exact path eventually discovered there
   was no scalar payload

That makes the future grouped scorer cutover wider than it needs to be. We need one dispatch seam
that decides:

1. exact scalar path
2. grouped path

while still keeping grouped runtime unsupported for now.

## Planned Slice

Add an explicit candidate-score dispatch seam:

1. scalar loaded states keep routing to the existing exact-score path
2. `ExactUnavailable` grouped entries route through grouped dispatch
3. grouped dispatch still returns the existing unsupported-runtime error

This still excludes:

- no grouped-v2 traversal enablement
- no grouped approximate scorer
- no rerank fetch path
- no change to the existing grouped-v2 runtime rejection behavior

## Implementation

Updated `src/am/scan.rs`:

1. added `CandidateScoreDispatch<'a>`:
   - `Exact(LoadedElementState)`
   - `Grouped(GroupedScoreInput<'a>)`
2. added `candidate_score_dispatch(...)`:
   - `LoadedElementState::ExactUnavailable` now maps to grouped dispatch using the cached grouped
     score input
   - all other loaded states preserve the existing exact path
3. added `score_cached_graph_element_dispatch(...)` as the single score boundary:
   - exact dispatch forwards to `exact_score_cached_graph_element(...)`
   - grouped dispatch still errors with `ADR030_GROUPED_V2_SCAN_UNSUPPORTED`
4. changed both score call sites to use the dispatch seam:
   - `cached_graph_element_and_score(...)`
   - binary-prefilter survivor rescoring in `cached_scan_successor_candidates_for_layer(...)`

New tests:

- `candidate_score_dispatch_uses_grouped_input_for_exact_unavailable`
- `candidate_score_dispatch_keeps_scalar_loaded_state_exact`

## Measurements

This packet is still a runtime seam packet, so there are no new latency or recall measurements.

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

ADR-030 v2 now has an explicit score-dispatch boundary that separates scalar exact scoring from the
future grouped path.

What this de-risks:

1. the grouped scorer can now cut in at one dispatch point instead of replacing scattered exact
   score calls
2. grouped-v2 unsupported behavior is now an explicit grouped-dispatch outcome rather than a
   side-effect of the scalar exact path
3. the next scorer packet can stay local to grouped dispatch logic without reshaping cache or scan
   call sites again

## Next Slice

The next narrow slice should add the first no-op grouped scorer stub behind this dispatch seam:

1. extract grouped dispatch into a dedicated helper
2. keep the helper returning the existing unsupported-runtime error
3. shape that helper around `GroupedScoreInput` so the later LUT scorer cutover is local
