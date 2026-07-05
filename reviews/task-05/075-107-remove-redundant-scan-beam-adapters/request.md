# Request: Remove Redundant Scan-Beam Adapters

Commit: `56c66c3`

Summary:
- Removes the standalone `scan_candidate_to_beam_candidate` and `beam_candidate_to_scan_candidate` helpers from `src/am/scan.rs`.
- Keeps the remaining debug/snapshot conversion points using the existing `From` implementations directly.
- Leaves `ScanCandidate` as a boundary-facing type, but trims one more historical adapter layer from the scan module.

Files:
- `src/am/scan.rs`

Why this matters:
- Recent slices already moved the runtime frontier, active state, removal path, and head selection onto shared `BeamCandidate<ItemPointer>` values.
- The standalone conversion helpers were no longer carrying real boundary logic; they had become thin wrappers around `Into`/`From`.
- Removing them tightens the module surface and makes it clearer where scan-local representation is still intentional versus leftover inertia.

Review focus:
- Whether the remaining `ScanCandidate` conversion points now all correspond to explicit debug/snapshot boundaries
- Whether any other scan-local wrappers around beam-native state are now equally redundant and ready to delete
- Whether the next stronger step should target `ScanCandidate` boundary APIs in `scan_debug.rs` or move more identity/results through `src/am/search.rs`
