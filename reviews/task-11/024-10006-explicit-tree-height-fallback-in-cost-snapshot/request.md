# Review Request: Explicit Tree-Height Fallback In Cost Snapshot

Scope:
- `src/am/cost.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md`
- `spec/functional/FR-020-cost-estimation.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Refactored the pure planner cost helper so it consumes an explicit `tree_height` input instead of
  implicitly binding that concept to metadata `max_level`.
- Added a narrow `metadata_fallback_tree_height(...)` seam that makes current planner scaffolding
  honest: tree height still comes from metadata because PG18 `amgettreeheight` wiring and toolchain
  support do not exist yet in this repository.
- Extended `tqhnsw_index_cost_snapshot(regclass)` to report `resolved_tree_height`,
  `tree_height_source`, and `pg18_tree_height_callback_ready` so planner/admin tooling can inspect
  the current staging boundary directly.
- Added unit and pg coverage for the explicit fallback contract, and updated ADR-011 / FR-020 /
  test matrix / Task 11 notes to record this as D1 scaffolding rather than hidden PG18 progress.

Review focus:
- Whether making tree-height sourcing explicit is the right preparatory seam before any PG18
  feature flag or `IndexAmRoutine` callback registration exists
- Whether the new SQL snapshot fields are scoped appropriately for planner/productization work
  without implying that PG18 planner callbacks are already supported
- Whether the refactor from `max_level` to explicit `tree_height` keeps the pure FR-020 helper on
  the right long-term path toward eventual planner callback activation

Questions to answer:
- Is `metadata_fallback_tree_height(...)` the right near-term boundary, or should the tree-height
  source remain implicit until a real PG18 callback can be wired?
- Is exposing `pg18_tree_height_callback_ready = false` explicit and useful, or does that surface
  too much implementation staging detail for a long-lived SQL snapshot?
- Does the cost snapshot now make the FR-020 current-vs-target state clearer, especially for later
  `amgettreeheight` activation work?
