# Request: Active Candidate Beam State

Commit: `5e5794e`

Summary:
- Changes `active_candidate` in `src/am/scan.rs` from a scan-local `ScanCandidate` payload to `Option<BeamCandidate<ItemPointer>>`.
- Keeps the active bootstrap execution path beam-shaped through consume, linear-scan bridging, and direct result materialization.
- Updates `src/am/scan_debug.rs` so pg-test/debug surfaces still expose the same tuple-oriented active-candidate state while deriving it from the beam-native representation.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`

Why this matters:
- Recent slices already moved visible frontier storage, seeding, refill, consume, and internal frontier iteration onto `BeamCandidate<ItemPointer>`.
- `active_candidate` was the main remaining hot-path state that still converted back into `ScanCandidate` before being turned into a beam candidate again during materialization.
- This slice removes that last conversion loop from the bootstrap execution path and tightens the meaning of `ScanCandidate` toward debug and boundary-facing state.

Review focus:
- Whether the new `Option<BeamCandidate<ItemPointer>>` active-candidate contract is now consistent across consume, materialization, and the linear-scan bridge
- Whether any debug or pg-test surface still assumes the old `ScanCandidate::default()` sentinel model instead of explicit presence/absence
- Whether this creates a clean enough seam to retire more `ScanCandidate` hot-path usage, or whether the next best transfer is now visible-result state rather than frontier state
