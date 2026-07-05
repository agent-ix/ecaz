# Review Request: Planner Cost Model Scaffolding

Scope:
- `src/am/cost.rs`
- `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md`
- `spec/functional/FR-020-cost-estimation.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a pure `estimate_planner_cost(...)` helper in `src/am/cost.rs` that models the FR-020
  startup and total cost formula from `(index_pages, reltuples, m, ef_search, dimensions,
  max_level)` plus PostgreSQL cost constants.
- Added unit coverage for the planned large-table crossover, small-table seqscan preference,
  empty-index `f64::MAX`, and missing-`reltuples` heuristic cases.
- Kept the live `tqhnsw_amcostestimate` callback unchanged behind ADR-011, so planner-visible
  `tqhnsw` scans remain disabled while the cost model matures in isolation.
- Updated ADR-011, FR-020, the test matrix, and Task 11 tracking to record this as D1 scaffolding
  rather than planner activation.

Review focus:
- Whether the pure helper is the right seam for FR-020 D1 without prematurely wiring planner
  behavior
- Whether the modeled edge cases and crossover fixtures are the right initial unit-test envelope
- Whether the staged docs clearly separate “cost model exists” from “planner costing is live”
- Whether `selectivity = 1.0` in the pure helper is the right final-model contract while the live
  callback still returns ADR-011 gate values

Questions to answer:
- Is keeping the pure FR-020 helper inside `src/am/cost.rs` the right long-lived shape, or should
  the final implementation split a pure math module from the callback glue once metadata reads and
  PG18 tree-height wiring arrive?
- Are the current unit-test fixtures a sensible first approximation of the intended crossover
  behavior, or do they need a different representative envelope before planner activation work?
- Is it acceptable that the pure helper already reflects the intended final selectivity contract
  while the live callback still reports non-competitive gate values under ADR-011?
