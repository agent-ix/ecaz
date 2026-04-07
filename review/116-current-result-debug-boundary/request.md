# Request: Add Current-Result Debug Boundary

Commit: `528f570`

Summary:
- Makes `CurrentScanResult` fields private in `src/am/scan.rs`.
- Adds small accessor methods on `CurrentScanResult` for element TID, heap TID, score, score-valid, and presence.
- Updates `src/am/scan_debug.rs` to use that boundary instead of reaching into raw `current_result` fields directly.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`

Why this matters:
- The recent scan/search cleanup left `current_result` as the main remaining scan-owned result boundary.
- `scan_debug.rs` was still reaching through that boundary as raw struct fields, which made the boundary implicit rather than real.
- This slice makes `current_result` a clearer contract: runtime state stays scan-owned, while debug/test consumers read it through explicit accessors.

Review focus:
- Whether `CurrentScanResult` is now the right stable boundary for scan-owned result state
- Whether any remaining scan/debug access should be promoted into similar explicit methods instead of raw field reach-through
- Whether the new accessors are the right minimal surface, or if some should remain private to `scan.rs`
