# Request: Remove Dead Queued Beam Lookup

Commit: `60adb1a`

Summary:
- Removes `BeamSearch::queued_candidate(node)` from `src/am/search.rs`.
- Removes the matching unit tests that existed only to cover that unused helper.
- Leaves the shared beam API centered on the scheduler operations that current scan execution actually uses: `peek_best_matching`, `take_best_matching`, `forget_queued`, and snapshot-style test helpers.

Files:
- `src/am/search.rs`

Why this matters:
- The helper was introduced as possible groundwork for future scan/search ownership transfer, but the subsequent cleanup took a different path and never used it in runtime code.
- Keeping an unused read-only scheduler lookup around would make the search boundary look broader than it really is.
- This slice keeps the search API honest and reduces one more dead seam before the next real traversal/result ownership move.

Review focus:
- Whether removing `queued_candidate` leaves any current or near-term scan/search cleanup without a needed read-only scheduler primitive
- Whether the remaining beam API is now the right minimal surface for the current executor
- Whether the next ownership transfer should move more result/frontier identity into `src/am/search.rs`, rather than reintroducing broader introspection helpers
