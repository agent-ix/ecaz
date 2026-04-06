# Request: Localized Frontier Node Lookup

Commit: `ef01ba7`

Summary:
- Removes the shared `find_candidate_frontier_index` helper from `src/am/scan.rs`.
- Makes scheduler visibility checks use direct node containment instead of shared node-to-index mapping.
- Keeps the remaining node-to-index search local to visible-frontier removal, and updates the last helper test to reason in TID terms instead of index terms.

Files:
- `src/am/scan.rs`
- `src/lib.rs`

Why this matters:
- The previous slices already moved cached head state, consume semantics, and debug reporting away from Vec-slot-first thinking.
- `find_candidate_frontier_index` had become the last reusable helper that normalized scheduler-owned node identity back into Vec-index semantics across multiple paths.
- Removing that helper confines index lookup to the one place where the Vec still genuinely needs it: removing a visible candidate from the current container.

Review focus:
- Whether removing the shared node-to-index helper is the right cleanup before deeper frontier/container changes
- Whether any remaining scan paths still promote scheduler-owned node identity back into reusable Vec-index semantics unnecessarily
- Whether the next slice should now target the visible frontier container itself rather than more mapping cleanup
