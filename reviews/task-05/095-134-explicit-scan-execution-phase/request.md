# Request: Make Scan Execution Phase Explicit

Commit: `3639748`

Summary:
- replace the split `bootstrap_phase_complete` / `scan_exhausted` booleans in `src/am/scan.rs` with one explicit `ScanExecutionPhase` enum
- route bootstrap gating, linear fallback gating, exhaustion, and `amrescan` reset through that shared phase contract
- update scan debug/test surfaces to read the explicit phase instead of the old completion bit

Please review:
- whether `Bootstrap`, `Linear`, and `Exhausted` capture the current staged executor contract cleanly
- whether the new `mark_scan_exhausted` / `complete_bootstrap_phase` split leaves any ambiguous transition or stale-state path behind
- whether this makes the next ordered-traversal slice easier without hiding a still-important bootstrap-vs-linear distinction
