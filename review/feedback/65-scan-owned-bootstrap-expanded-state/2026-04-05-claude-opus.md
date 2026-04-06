# Feedback: Scan-Owned Bootstrap Expanded State

Request:
- `review/65-scan-owned-bootstrap-expanded-state.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Response to Review Focus

### Scan-state ownership and cleanup for `expanded_source_tids`

**Correct.** The `expanded_source_tids: *mut HashSet<page::ItemPointer>` field (scan.rs:1059) follows the same ownership pattern as `visited_tids` and `candidate_frontier`:
- `reset_scan_expanded_state` (scan.rs:413-419): allocates or clears — called from `reset_scan_position` on rescan
- `free_scan_expanded_set` (scan.rs:421-426): drops and nulls — called from `amendscan`
- `mark_expanded_source` (scan.rs:428-433): guards against null and INVALID
- `expanded_contains_source` (scan.rs:436-441): same guards

The lifecycle mirrors visited-set lifecycle exactly, which is the right consistency.

### Correctness after helper-local vector removal

The transition from a helper-local `Vec<bool>` to scan-owned `HashSet<ItemPointer>` is strictly better: the HashSet doesn't depend on frontier slot ordering (which shifts after consumption), and it persists correctly across helper boundaries. The `next_bootstrap_expand_index` function (scan.rs:510-546) queries `expanded_contains_source` to filter already-expanded candidates, which works correctly with the HashSet.

### Edge cases around resets, empty frontiers, or repeated fill entry

- **Reset on rescan**: `reset_scan_position` → `reset_scan_expanded_state` clears the set. The subsequent `initialize_scan_entry_candidate` → `fill_bootstrap_frontier` calls `reset_scan_expanded_state` again at the top (scan.rs:598). This double-clear is harmless.
- **Empty frontier**: If no candidates are seeded (empty index, deleted entry), `fill_bootstrap_frontier` returns immediately because the frontier is empty and `next_bootstrap_expand_index` returns `None`.
- **Repeated fill entry**: The `fill_bootstrap_frontier` function resets the expanded set before filling (scan.rs:598), so calling it multiple times starts fresh each time. `top_up_bootstrap_frontier` does not reset — it adds to the existing expanded set, which is correct for incremental top-up.

No edge case gap found.

## Additional Findings

No issues found. The ownership move is clean and the lifecycle is consistent with the other scan-owned sets.
