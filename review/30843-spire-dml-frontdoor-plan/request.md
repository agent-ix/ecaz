# SPIRE DML Front-Door Plan Surface

## Scope

This packet exposes a small SQL planning/status surface for the remaining
transparent ADR-069 DML front doors. The lower-level primitives now exist for
coordinator-routed INSERT, non-embedding UPDATE, DELETE, PK-keyed SELECT, and
embedding-UPDATE rejection, but transparent UPDATE/DELETE/PK SELECT still need
their executor or planner integration point confirmed before implementation.

Changes:

- Adds `ec_spire_coordinator_dml_frontdoor_plan()`, returning one row per v1
  operation with:
  - operation name;
  - intended integration point;
  - narrow supported query shape;
  - backing primitive;
  - current implementation status.
- Marks INSERT and embedding-UPDATE rejection as ready.
- Marks non-embedding UPDATE, DELETE, and PK-keyed SELECT as
  `frontdoor_pending`, with explicit narrow shapes for the upcoming hook work.
- Updates the Phase 11 tracker to record packet `30843` as the planning/status
  surface for the remaining DML hooks.

This packet intentionally does not implement the transparent UPDATE/DELETE/PK
SELECT hooks. The hook boundary question is tracked in
`review/30803-spire-customscan-pivot-adrs/feedback/2026-05-11-002-coder.md`.

## Validation

- `cargo test coordinator_dml_frontdoor_plan --lib`
  - result: pass.
  - key line:
    `test tests::pg_test_ec_spire_coordinator_dml_frontdoor_plan_sql ... ok`
  - summary: `1 passed; 0 failed; 1645 filtered out`
- `cargo fmt --check`
  - result: pass with the repo's existing stable-rustfmt warnings.
- `git diff --check`
  - result: pass.

## Review Focus

- Confirm the exposed operation/status rows are the right operator-facing
  contract while UPDATE/DELETE/PK SELECT hooks are pending.
- Confirm the narrow v1 query-shape descriptions match ADR-069 and the current
  primitives.
- Confirm this is an acceptable checkpoint before implementing transparent
  `ModifyTable` or planner hook integration.

## Artifacts

- `review/30843-spire-dml-frontdoor-plan/artifacts/manifest.md`
- `review/30843-spire-dml-frontdoor-plan/artifacts/cargo-test-coordinator-dml-frontdoor-plan-lib.log`
- `review/30843-spire-dml-frontdoor-plan/artifacts/cargo-fmt-check.log`
- `review/30843-spire-dml-frontdoor-plan/artifacts/git-diff-check.log`
