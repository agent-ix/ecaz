# Request: Beam-Native Frontier Debug Boundaries

Commit: `b2b7022`

Summary:
- Changes `src/am/scan_debug.rs` to derive frontier debug state from beam-native frontier helpers instead of scan-shaped snapshot helpers.
- Replaces the remaining `candidate_slot` and `visible_frontier_snapshot` uses with `visible_frontier_slot` and `visible_frontier_candidates`.
- Updates the corresponding scan unit tests in `src/am/scan.rs` to assert against beam-native frontier state instead of scan-shaped aliases.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`

Why this matters:
- The runtime frontier is now beam-native end to end, and recent slices already removed several scan/beam adapter layers.
- The old debug/test helpers were still rebuilding `ScanCandidate` snapshots only to immediately project back into tuple forms for assertions.
- This slice keeps the debug/test boundary explicit while reducing one more set of scan-shaped aliases that no longer reflect the hot-path ownership model.

Review focus:
- Whether the remaining `ScanCandidate` surfaces now correspond to true boundary needs rather than leftover debug convenience
- Whether `scan_debug.rs` now has the right beam-native helper layer, or if more of that tuple projection should be centralized further
- Whether the next cleanup should keep shrinking `ScanCandidate` usage in tests, or shift focus to moving more result identity behind `src/am/search.rs`
