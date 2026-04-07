# Review Request: EXPLAIN Output Group Contract

Scope:
- `src/am/explain.rs`
- `spec/functional/FR-024-custom-explain.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a pure `ExplainOutputGroup` contract in `src/am/explain.rs` for the EXPLAIN section label
  and its expected `ExplainOpenGroup` / `ExplainCloseGroup` bracketing.
- Added `explain_output_group()` so the eventual PG18 `explain_per_node_hook` has explicit section
  metadata alongside the existing gating and property-emission helpers.
- Added unit coverage for that group contract and updated FR-024, the test matrix, and Task 11
  notes to record that the group shape is now modeled in pure planner-owned code.

Review focus:
- Whether the `"TQVector Stats"` group contract belongs in `am/explain.rs` as part of the D1 hook
  seam
- Whether making `ExplainOpenGroup` / `ExplainCloseGroup` explicit in pure code is the right final
  EXPLAIN-scaffolding step before actual PG18 hook registration
- Whether this keeps the work in the “real D1 seam” category rather than slipping back into low-
  value static scaffolding

Questions to answer:
- Is `ExplainOutputGroup` the right abstraction, or should the group label and open/close metadata
  stay implicit until real hook code exists?
- Does this make the remaining EXPLAIN D1 work effectively complete, with only actual PG18 hook
  registration/binding and scan-lane counter wiring left?
- Is any other pure EXPLAIN contract still obviously missing before we should stop touching
  `am/explain.rs`?
