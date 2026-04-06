# Feedback: Wider Bootstrap Frontier Seeding

Request:
- `review/56-wider-bootstrap-frontier-seeding.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is three total seeded candidates sensible, or should this stage stay at two?

**Three is the right step.** The entry point plus up to two neighbors represents the minimum "I can see past my immediate starting position" capability. Two candidates (entry + one neighbor) is degenerate — it doesn't test whether the frontier can meaningfully compare alternatives. Three candidates forces the head-selection and expansion logic to actually choose among candidates, which validates the scoring-based ordering before the frontier grows further.

`MAX_BOOTSTRAP_FRONTIER_CANDIDATES = 3` (scan.rs:11) is an honest placeholder that exercises the full frontier machinery without pretending to implement beam search. The `fill_bootstrap_frontier` → `top_up_bootstrap_frontier` loop (scan.rs:495-524) correctly expands from the best unexpanded source until the frontier hits the cap or runs out of expandable sources.

### Ordering or lifecycle edges from widened seeding?

No ambiguity found. The wider seeding flows through the same `fill_bootstrap_frontier` → `refill_candidate_frontier_from_source` path that the two-candidate version used. The visited set is seeded correctly: `mark_visited_element` is called for the entry (scan.rs:463) and for every new candidate produced during refill (scan.rs:608-610). The head is recomputed after every frontier mutation.

One observation: the `expanded_source_tids` set (scan.rs:376-397) tracks which frontier candidates have been used as expansion sources. This prevents re-expanding the same source during `top_up_bootstrap_frontier`. The initial `fill_bootstrap_frontier` call (scan.rs:495-504) resets this set before the first fill, which is correct — it means the entry candidate starts as unexpanded and gets expanded as the first source.

### Is the two-slot/full-slot debug mismatch acceptable?

**Yes, temporarily.** The older `DebugCandidateFrontier` two-slot snapshot (scan.rs:1008) is legacy scaffolding. The new `debug_candidate_frontier_slots` (scan.rs:1053) returns the full Vec contents and is used by the newer tests. The older helpers still work because they only look at the first two slots, which are always populated by the entry + first neighbor. When the frontier grows further, the old helpers will need cleanup, but for now the mismatch is harmless.

## Additional Findings

No issues found. The wider seeding is the natural next step in the bootstrap-to-traversal progression.
