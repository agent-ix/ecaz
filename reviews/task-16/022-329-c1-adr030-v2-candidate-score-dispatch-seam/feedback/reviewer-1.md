## Feedback: ADR-030 v2 Candidate Score Dispatch Seam

Read `CandidateScoreDispatch<'a>`, `candidate_score_dispatch(...)`, and
`score_cached_graph_element_dispatch(...)` in `src/am/scan.rs`, plus both callsite
updates (`cached_graph_element_and_score` and the binary-prefilter survivor rescoring
path).

### What's right

- Two callsites, one dispatch seam. Previously each callsite called
  `exact_score_cached_graph_element` directly; now both go through the single dispatch
  function. Future grouped scoring is a local change to one function, not a scatter-
  gather edit across two callsites.
- `CandidateScoreDispatch::Exact(LoadedElementState)` preserves the scalar state
  directly. The exact arm does not re-derive state, just forwards it. That's the
  right minimum.
- Grouped dispatch still ends in `ADR030_GROUPED_V2_SCAN_UNSUPPORTED`. Runtime
  semantics unchanged. The gate from packet 323 is still honored.

### Concerns

1. **Dispatch is state-driven, not format-driven.** `candidate_score_dispatch` branches
   on `LoadedElementState::ExactUnavailable` to pick the grouped arm, not on the
   `GraphStorageDescriptor`. That works today because `ExactUnavailable` is set only
   for grouped-v2 live tuples. But it couples two invariants:

   - "scalar-v1 never produces `ExactUnavailable`"
   - "grouped-v2 always produces `ExactUnavailable` when live"

   If either ever drifts, dispatch silently picks the wrong arm. Packet 332's
   `grouped_score_context_from_scan_state` adds a second guard (returns `None` when
   scan storage is ScalarV1), but packet 329's dispatch still panics if that guard
   fails — see packet 332 feedback.

   A cleaner long-term shape: dispatch branches on scan descriptor, not loaded state.
   State becomes an input to the exact arm only.

2. **Tests are shape-only.** The two new dispatch tests check the enum variant, not
   the end-to-end scoring behavior. That's appropriate for a seam packet, but as soon
   as a real scorer lands, there must be an end-to-end test that the binary-
   prefilter survivor path and `cached_graph_element_and_score` both get the correct
   variant on a real built index.

### Observation

Pulling grouped-unsupported behavior out of the exact-score path into explicit grouped
dispatch removes a class of "scalar path silently becoming grouped path" drift. That's
worth the packet even though runtime behavior is unchanged.
