# Request: Search-Owned Scheduler Consume

Commit: `5e082a5`

Summary:
- Adds `BeamSearch::take_best_matching` in `src/am/search.rs`.
- Moves scheduler-first consume in `src/am/scan.rs` from a two-step `peek_best_matching` plus `forget_queued` flow to one search-owned consume API.
- Adds unit coverage that the search-owned consume path drops stale leaders and removes the first live candidate from scheduler discovery/frontier state.

Files:
- `src/am/search.rs`
- `src/am/scan.rs`

Why this matters:
- The previous slice moved stale queued-node cleanup into `search.rs`, but visible consume in `scan.rs` still had to coordinate two scheduler operations manually: identify the best visible node, then explicitly forget it after visible-frontier removal.
- This slice moves that consume-side scheduler bookkeeping behind the shared search seam too.
- It keeps the visible frontier as the materialization container for now, but narrows `scan.rs` further toward “remove visible node by identity” instead of “run scheduler policy by hand.”

Review focus:
- Whether `take_best_matching` is the right consume-side companion to `peek_best_matching` during the current dual-structure phase
- Whether removing the matched candidate from scheduler discovery state before visible-frontier removal creates any subtle drift or recovery hazards
- Whether the next ownership-transfer step should now target fuller candidate storage in `search.rs` or keep shrinking visible-frontier-only responsibilities in `scan.rs`
