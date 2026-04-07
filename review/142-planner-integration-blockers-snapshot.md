# Review Request: Planner Integration Blockers Snapshot

Scope:
- `src/am/mod.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `src/am/cost.rs`
- `spec/functional/FR-009-hnsw-scan.md`
- `spec/functional/FR-020-cost-estimation.md`
- `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a read-only SQL/admin surface, `tqhnsw_planner_integration_snapshot(regclass)`, that
  consolidates the planner gate, ordered-scan readiness, modeled-cost readiness, live callback
  status, PG18 callback readiness, PG18 diagnostics readiness, and the resolved `ef_search`
  source/value into one row.
- Kept every activation-facing readiness bit honest: the cost model is ready, but ordered scan,
  live planner activation, PG18 callbacks, and PG18 diagnostics all remain false.
- Exposed two explicit blocker strings:
  - runtime blocker: ordered tqhnsw scan semantics plus recall validation are not yet credible
  - PG18 blocker: pgrx PG18 feature support and callback bindings are not yet implemented
- Added pg coverage for both the happy path and non-`tqhnsw` rejection path, and updated
  FR-009 / FR-020 / ADR-011 / test-matrix / Task 11 notes so this remains descriptive
  integration scaffolding rather than planner activation.

Review focus:
- Whether one consolidated planner-integration snapshot is the right cross-agent seam, or whether
  this duplicates the narrower explain/cost/diagnostics snapshots without enough added value
- Whether the blocker strings and readiness bits are explicit enough to help the runtime lane and
  the PG18 lane align on what still blocks planner enablement
- Whether `planner_cost_model_ready = true` while `planner_cost_callback_live = false` is the
  right current contract for productization and review work
- Whether surfacing the resolved `ef_search` in this integration snapshot is useful context rather
  than unnecessary duplication from the existing planner/admin snapshots

Questions to answer:
- Is `tqhnsw_planner_integration_snapshot(regclass)` the right place to surface cross-lane
  integration status for the other agent, or should that remain only in review/spec text?
- Are the current runtime and PG18 blocker strings the right durable phrasing, or should this
  surface use a more structured contract before additional lanes consume it?
- Does this snapshot make it clearer when a future rebase or planner-enablement turn is justified,
  especially once ordered scan semantics or PG18 bindings start moving on other branches?
