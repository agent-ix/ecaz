# Review Request: Cost Snapshot For Gated Planner Model

Scope:
- `src/am/cost.rs`
- `src/am/mod.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md`
- `spec/functional/FR-020-cost-estimation.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added `tqhnsw_index_cost_snapshot(regclass)`, a read-only SQL surface that reports the resolved
  tuning inputs, metadata inputs, current PostgreSQL planner cost constants, the modeled FR-020
  startup/total cost outputs, and the still-gated live callback outputs side by side.
- Extended `src/am/shared.rs` to build a planner-cost snapshot from relation options, metadata,
  current `pg_class.reltuples`, and the pure `estimate_planner_cost(...)` helper.
- Kept the live `tqhnsw_amcostestimate` callback hard-gated behind ADR-011, with the snapshot
  explicitly exposing that the callback still returns prohibitive costs and zero selectivity.
- Added pg coverage for both the happy path and non-`tqhnsw` rejection, and updated FR-020 / ADR-011
  / test-matrix / Task 11 tracking to record this as planner scaffolding rather than activation.

Review focus:
- Whether a dedicated cost snapshot is the right seam for planner/admin inspection before
  `amcostestimate` is allowed to consume the real model
- Whether exposing both modeled and gated outputs side by side is clear enough to prevent users or
  future code from mistaking the modeled numbers for live planner behavior
- Whether reading `pg_class.reltuples`, metadata, reloptions, and current cost constants is the
  right current input set for D1 without touching scan runtime
- Whether the SQL result shape is scoped appropriately for planner/productization work and not
  overcommitting later FR-020 activation details

Questions to answer:
- Is `tqhnsw_index_cost_snapshot(regclass)` the right near-term boundary for cost-model inspection,
  or should this remain test-only until `amcostestimate` activation is closer?
- Is showing both modeled and gated outputs in one row the clearest contract, or should the gated
  callback values stay implicit while ADR-011 is active?
- Is using live PostgreSQL cost constants plus `pg_class.reltuples` in the snapshot the right
  current behavior for planner scaffolding, even before PG18 tree-height wiring and real callback
  activation land?
