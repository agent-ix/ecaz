# Request: Remove Dead Bootstrap Expand Helper

Commit: `c77b033`

Summary:
- Removes the dead `next_bootstrap_expand_tid` helper from `src/am/scan.rs`.
- Rewrites the remaining score-order policy unit test to assert directly on the visible frontier's lowest-score unexpanded candidate instead of rebuilding a temporary beam scheduler for the assertion.

Files:
- `src/am/scan.rs`

Why this matters:
- The live bootstrap top-up path no longer uses `next_bootstrap_expand_tid`; scheduler-owned expansion moved on earlier slices.
- Leaving the helper in place kept a stale alternate model of score-order selection alive only for tests.
- Deleting it reduces scan-local duplication and makes the test describe the actual contract more directly: among unexpanded visible frontier candidates, score order prefers the lowest score.

Review focus:
- Whether removing `next_bootstrap_expand_tid` leaves any missing coverage for score-ordered bootstrap expansion behavior
- Whether the rewritten unit test still captures the intended "lowest-score unexpanded frontier candidate wins" contract clearly enough
