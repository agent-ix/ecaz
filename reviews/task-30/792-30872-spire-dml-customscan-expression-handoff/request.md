# Review Request: SPIRE DML CustomScan Expression Handoff

## Scope

Code commit: `e68fa9667c8816b55044a91f2e4548a590c2c2ce`

This packet adds the planner-side expression handoff needed before the DML frontdoor can build CustomPath/CustomScan nodes for ADR-069 UPDATE, DELETE, and PK SELECT shapes.

Changes:

- Adds `SpireDmlFrontdoorPrimitivePlanExpr`, bundling the typed primitive plan with the raw PK value expression pointer.
- Adds `dml_frontdoor_primitive_plan_expr_catalog_row(...)`.
- Refactors the private PK predicate extraction to retain the matched PK value expression while preserving the existing classifier and replacement-decision outputs.
- Re-exports the helper through the SPIRE AM surfaces for the upcoming CustomScan path builder.
- Extends PG18 DML frontdoor coverage to assert UPDATE, DELETE, and PK SELECT all produce a typed primitive plan plus non-null const PK expression handoff.
- Updates the Phase 11 task file with packet `30872`.

This packet does not enable planner rewrite or DML dispatch. The helper exists so the next CustomPath slice can copy the PK expression into `custom_exprs`.

## Validation

- `cargo test dml_frontdoor --lib`
  - `23 passed; 0 failed; 0 ignored; 1648 filtered out`
  - artifact: `artifacts/cargo-test-dml-frontdoor-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs src/am/ec_spire/mod.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm `SpireDmlFrontdoorPrimitivePlanExpr` is the right planner handoff object for the upcoming DML CustomPath builder.
2. Confirm retaining the raw matched PK value expression does not change classifier behavior.
3. Confirm this stays a non-dispatch scaffold packet.
