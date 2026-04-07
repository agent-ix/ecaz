# Request: Search-Owned Stale Frontier Cleanup

Commit: `0c7a79f`

Summary:
- Adds `BeamSearch::peek_best_matching` in `src/am/search.rs`.
- Moves stale queued-node cleanup for scheduler-first frontier head selection out of `src/am/scan.rs` and into the shared search module.
- Adds unit coverage that the new search-owned API drops unmatched stale leaders and can fully drain a stale queued frontier.

Files:
- `src/am/search.rs`
- `src/am/scan.rs`

Why this matters:
- The scan executor had still been open-coding one scheduler responsibility: repeatedly peeking the best queued node, checking whether it still mapped to a visible frontier candidate, dropping stale nodes, and retrying.
- That kept stale-node cleanup logic in the pg-facing scan layer even though the underlying queue ownership already lived in `BeamSearch`.
- This slice makes the shared search structure responsible for that stale-best-node cleanup policy, which is a real authority transfer instead of another scan-local wrapper.

Review focus:
- Whether `peek_best_matching` is the right minimal search-facing API for the current dual-structure phase
- Whether mutating the scheduler while probing for the current visible best candidate introduces any scan/executor edge cases
- Whether the next ownership-transfer step should now move more visible-frontier identity or removal semantics behind `search.rs`
