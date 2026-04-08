# Review Request: Scan Test-Only Gating And Plan Status

Commit: `40d4db1`

Scope:
- `src/am/scan.rs`
- `plan/plan.md`
- `plan/tasks/11-planner.md`
- `spec/adr/ADR-015-bootstrap-to-ordered-traversal-transition.md`

Summary:
- gate the known test-only bootstrap helper enum/functions in `scan.rs` behind
  `#[cfg(any(test, feature = "pg_test"))]` so production builds stop carrying dead scaffolding
- update the top-level plan and Task 11 status to reflect that planner D1 scaffolding is now
  substantially complete on `main`, both planner branches are merged, and D2 remains blocked on A4
- record in ADR-015 that the current merged traversal scaffolding is following the decided
  layer-0-only boundary rather than leaving that decision framed as still-open

Please review:
- whether the `scan.rs` gating is narrow enough to avoid changing runtime behavior while removing
  only test-only production surface
- whether the plan/task status now matches the real merged state of planner work
- whether the ADR wording accurately captures the now-resolved layer-0 traversal boundary
