# Feedback: Bootstrap Expanded-State Groundwork

Request:
- `review/64-bootstrap-expanded-state-groundwork.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Response to Review Focus

### Is score-ordered expansion the right bootstrap policy?

**Yes — covered in review 63 response.** Score-ordered expansion is the HNSW greedy-descent heuristic expressed through the bootstrap framework. It does not overstate traversal semantics because it's doing exactly what real traversal will do, just at a smaller scale.

### Should expanded-state bookkeeping move into scan-owned state?

**This is addressed in review 65.** The current slice establishes the score-ordered policy; the next slice (65) moves expanded-state tracking from helper-local storage into `TqScanOpaque`. The sequencing is correct: establish the policy behavior first, then formalize the state ownership.

### Risk of overstating traversal semantics while tuple production is linear?

**Low risk.** The bootstrap frontier is explicitly decoration during linear scan — it doesn't affect which tuples are produced or in what order. The frontier exercises the scoring, expansion, and visited-set machinery in parallel with linear production, which is valuable for validating correctness before the traversal switchover. The separation between "frontier state" and "tuple production" is clean.

## Additional Findings

This review request appears to cover the same commit as review 63 (both reference `65fa7d2`). The review focus is slightly different — 63 focuses on the policy behavior, 64 focuses on the expanded-state implications. My responses to both cover the relevant concerns.
