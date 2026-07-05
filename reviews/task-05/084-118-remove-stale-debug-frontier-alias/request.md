# Request: Remove Stale Debug Frontier Alias

Commit: `ab1b188`

Summary:
- Removes the old fixed two-slot `DebugCandidateFrontier` alias from `src/am/scan_debug.rs`.
- Switches the remaining bootstrap frontier debug helpers to return the real `Vec`-backed visible frontier snapshot shape.
- Updates the matching pg tests in `src/lib.rs` to assert against `Vec` semantics instead of stale fixed-array slot assumptions.

Files:
- `src/am/scan_debug.rs`
- `src/lib.rs`

Why this matters:
- The runtime frontier has been `Vec`-backed for a long time now, but part of the debug/test surface still implied a permanent two-slot container.
- That stale alias made review and regression output look like the frontier still had structural slot semantics that no longer exist in the scan path.
- This slice keeps the debug boundary aligned with the current runtime container without changing scan behavior.

Review focus:
- Whether the remaining debug/test frontier helpers now reflect the real runtime frontier contract clearly enough
- Whether any other debug-facing APIs still imply fixed-slot frontier structure that no longer exists
- Whether the updated pg tests still cover the intended lifecycle and compaction behavior after the move to `Vec` assertions
