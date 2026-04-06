# Request: Remove Test-Only Frontier Alias

Commit: `ed07802`

Summary:
- Removes `candidate_frontier_ref` from `src/am/scan.rs`.
- Updates the remaining scan tests to use the explicit `visible_frontier_snapshot` debug boundary instead of a second test-only alias over the same conversion path.
- Keeps `ScanCandidate` conversions confined to named boundary helpers rather than duplicated helper layers.

Files:
- `src/am/scan.rs`

Why this matters:
- The runtime frontier is beam-native now, and the remaining `ScanCandidate` uses are supposed to live at snapshot/debug boundaries.
- `candidate_frontier_ref` was just a second name for that same boundary and encouraged more duplicate test-side access patterns.
- Removing it tightens the scan module surface and makes the remaining boundary helper names more intentional.

Review focus:
- Whether the surviving snapshot/slot helpers are now the right explicit boundary for test and debug code
- Whether any further `ScanCandidate` cleanup in `src/am/scan.rs` would still be meaningful, or if the next valuable move is elsewhere
- Whether the next best simplification should target `scan_debug.rs` surfaces or move more traversal/result identity behind `src/am/search.rs`
