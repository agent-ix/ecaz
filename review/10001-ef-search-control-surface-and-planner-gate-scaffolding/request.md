# Review Request: ef_search Control Surface And Planner Gate Scaffolding

Scope:
- `src/am/mod.rs`
- `src/am/options.rs`
- `src/am/shared.rs`
- `src/am/cost.rs`
- `src/lib.rs`
- `spec/functional/FR-009-hnsw-scan.md`
- `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md`
- `spec/adr/ADR-014-traversal-state-memory-budget.md`
- `spec/adr/ADR-015-bootstrap-to-ordered-traversal-transition.md`
- `spec/adr/ADR-016-ef-search-control-surface.md`
- `spec/adr/ADR-017-pg18-module-identity-and-upgrade-direction.md`
- `spec/usecase/US-004-install-manage-extension.md`
- `plan/plan.md`
- `plan/tasks/archive/05-build-and-scan.md`
- `plan/tasks/05-graph-scan.md`

What changed:
- Added a session-level `tqhnsw.ef_search` GUC in `_PG_init` and resolved it against the existing
  per-index `ef_search` reloption with explicit precedence rules.
- Kept planner-visible `tqhnsw` scan selection disabled, but made that gate explicit in code and
  exposed a debug snapshot for planner-facing tuning state.
- Added unit and pg tests for default/fallback/override behavior and for the planner-disabled
  tuning snapshot.
- Reconciled the spec/ADR/plan surface around the real staged state: planner gate still on,
  traversal memory ADRs now decided, layer-0-first transition direction, and PG18 support keeping
  the existing `tqvector` extension identity.

Review focus:
- Whether the relation-versus-session `ef_search` precedence rules are coherent and safe for later
  planner costing and ordered traversal wiring
- Whether the planner gate remains explicit enough that this slice cannot accidentally enable
  planner-visible scans early
- Whether the new planner/config debug snapshot is a sensible scaffolding seam rather than leaking
  runtime-only details into the wrong layer
- Whether the ADR/spec/plan updates now match implementation reality without overcommitting the
  next ordered-scan stages

Questions to answer:
- Is treating the default GUC value as "no session override" the right compromise until/unless a
  separate sentinel policy is needed?
- Is the explicit `TQHNSW_PLANNER_SCAN_ENABLED = false` gate a good enough architectural seam for
  later planner enablement and EXPLAIN/statistics work?
- Do ADR-016 and ADR-017 capture the real long-lived decisions clearly enough to guide the next
  planner/productization slices?
