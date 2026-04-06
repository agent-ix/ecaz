# Request: Frontier Remove-Node Beam Return

Commit: `3541e71`

Summary:
- Changes `VisibleCandidateFrontierState::remove_node` in `src/am/scan.rs` to return the stored `BeamCandidate<ItemPointer>` directly.
- Removes the beam-to-scan-to-beam conversion loop from `take_candidate_frontier_node`.
- Keeps visible-frontier removal aligned with the beam-backed storage and the beam-native runtime consume/materialization path.

Files:
- `src/am/scan.rs`

Why this matters:
- The owned visible frontier already stores shared `BeamCandidate<ItemPointer>` values internally.
- Recent slices moved seeding, refill, iteration, active state, and consume/materialization onto beam candidates, but node removal still converted through `ScanCandidate` before returning to the hot path.
- This is a narrow authority-transfer cleanup that trims one more remaining dual-representation loop from the execution path.

Review focus:
- Whether any remaining visible-frontier runtime helpers still bounce through `ScanCandidate` only to immediately return to beam-native flow
- Whether this simplified `remove_node` contract makes the next ownership transfer clearer inside `src/am/scan.rs` or points more strongly at moving frontier identity into `src/am/search.rs`
- Whether the current debug and snapshot boundaries remain the only justified places where `ScanCandidate` is still the better surface
