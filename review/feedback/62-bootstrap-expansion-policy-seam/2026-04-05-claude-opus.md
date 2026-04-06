# Feedback: Bootstrap Expansion Policy Seam

Request:
- `review/62-bootstrap-expansion-policy-seam.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Response to Review Focus

### Is the policy seam at the right level?

**Yes.** The `BootstrapExpandPolicy` enum (scan.rs:14) and `next_bootstrap_expand_index` function (scan.rs:475-492) create a clean selection boundary between "which candidate to expand next" and "how to expand it." The expansion mechanics (`refill_candidate_frontier_from_source`) are policy-agnostic — they just take a source TID and produce new candidates. The policy only controls which source to pick.

This is the right level of abstraction. Moving policy closer to frontier-head state would conflate two concerns: head selection (which candidate to consume for result production) and expand selection (which candidate to expand for frontier growth). These are related but distinct — in full HNSW search, the best candidate to expand may not be the best candidate to return.

### Does `InsertionOrder` as the initial policy keep things clear?

**Note: this has already been superseded by `ScoreOrder` in review 63.** The insertion-order policy was a correct "preserve existing behavior" step that made the policy seam testable without changing semantics. The enum variant structure makes adding future policies (e.g., `LeastExpanded`, `MostNeighbors`) trivial.

### Test sufficiency

The helper-level tests verify that the policy seam correctly delegates to the insertion-order selector and that multi-hop fill still works through the indirection. This is the right test boundary — test the policy contract, not the expansion mechanics (which are tested separately).

## Additional Findings

No issues found. Clean structural refactor that enables score-ordered expansion.
