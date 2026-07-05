# Request: Remove Stale Bootstrap Candidate Staging

Commit: `a60b707`

Summary:
- Removes the stale `active_candidate` staging path from `src/am/scan.rs`.
- Updates the debug/test surface so bootstrap-candidate materialization is exercised directly through frontier consumption plus pending heap-TID drain state instead of a separate active-candidate slot.
- Collapses the remaining frontier/scheduler coordination behind one explicit local helper in `src/am/scan.rs`, so head selection and consume no longer open-code the same raw-pointer borrow workaround in two places.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Why this matters:
- Recent slices already made bootstrap frontier storage, consume, and head selection beam-native.
- The old `active_candidate` field had become stale scaffolding: `amgettuple` materialized frontier candidates directly, while the extra staging slot only remained for debug-only helpers and a dead linear-scan branch.
- Removing that slot simplifies scan-owned state, makes the remaining direct materialization path more honest, and narrows the next ownership question to `current_result` rather than a second bootstrap-result carrier.

Review focus:
- Whether removing `active_candidate` leaves any scan/debug/test path still expecting an intermediate bootstrap-result state
- Whether the new direct bootstrap-materialization debug helper covers the old regression intent adequately
- Whether the new `with_visible_frontier_and_bootstrap_expansion` seam is the right local ownership boundary, or if the next cleanup should move more of that coordination into `src/am/search.rs`
