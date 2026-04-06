# Request: Visible Frontier Beam Iteration

Commit: `41b2e35`

Summary:
- Changes internal visible-frontier iteration in `src/am/scan.rs` to stay in `BeamCandidate<ItemPointer>` form for selection and seeding paths.
- Keeps conversion to `ScanCandidate` at explicit boundary helpers like debug snapshots and slot reads.
- Removes more internal conversion churn where scan logic was reading beam-backed frontier storage, converting it to `ScanCandidate`, and then immediately projecting back down to beam-owned fields such as node and score.

Files:
- `src/am/scan.rs`

Why this matters:
- Earlier slices moved frontier storage, seeding, refill, and runtime consume/materialization onto `BeamCandidate`.
- But several internal scan-side selection paths still iterated the beam-backed frontier through `ScanCandidate` conversions, which preserved unnecessary representation churn in the hot path.
- This slice tightens the boundary: beam-backed frontier internals stay beam-shaped, while `ScanCandidate` is reserved for persistent scan state and debug/materialization-facing surfaces.

Review focus:
- Whether the remaining internal uses of `ScanCandidate` in `src/am/scan.rs` now correspond to real boundary/persistence needs instead of historical inertia
- Whether any frontier-selection or bootstrap-expansion edge cases changed due to the more beam-native iteration path
- Whether the next step should target `active_candidate`, or continue shifting more visible-frontier identity directly behind `search.rs`
