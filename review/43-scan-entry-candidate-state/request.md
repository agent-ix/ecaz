# Review Request: Scan Entry Candidate State

Scope:
- `src/am/mod.rs`
- `src/am/scan.rs`
- `src/lib.rs`

What changed:
- `amrescan` now seeds one explicit entry candidate in scan-owned state from the persisted metadata entry point for non-empty indexes.
- The entry candidate currently carries only the minimum traversal-start payload: element TID plus precomputed score.
- The bootstrap linear scan still drives tuple production exactly as before; this slice does not turn on graph traversal or planner-visible ordered search.
- Added regression coverage that `amrescan` seeds the entry candidate at the metadata entry point and that the candidate clears once the current bootstrap scan fully exhausts.

Review focus:
- Whether the seeded entry candidate is the right minimal bridge between current scan state and later traversal work
- Whether clearing the entry candidate on scan exhaustion matches a sane future candidate-queue lifecycle
- Whether the current test captures the right invariant without overcommitting to graph-search semantics that do not exist yet

Questions to answer:
- Is `element_tid + score` the right minimal shape for the first candidate slot, or is another field obviously needed before real traversal starts?
- Does seeding from the metadata entry point during `amrescan` establish the right ownership/lifecycle boundary for future candidate heaps?
- Should exhaustion clear the entry candidate now, or is there a stronger argument for keeping that seed around until the next rescan?
