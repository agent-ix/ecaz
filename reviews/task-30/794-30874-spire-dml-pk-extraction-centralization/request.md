# Review Request: SPIRE DML PK Extraction Centralization

## Scope

Code commit: `bcb8de5f210c6c6c2e6c53f54bd388b7016dd652`

This packet addresses the remaining 30873 P1 review item about duplicated
PK-predicate extraction between `custom_scan.rs` and `dml_frontdoor.rs`.

Changes:

- Adds `dml_frontdoor_pk_select_primitive_plan_expr_from_baserel(...)`, a
  DML-frontdoor-owned helper for the `set_rel_pathlist_hook` view of a PK SELECT
  baserel.
- Moves baserestrictinfo-aware PK predicate extraction into `dml_frontdoor.rs`.
  The existing analyzed-query extractor now delegates its clause parsing through
  the same clause helper.
- Updates `custom_scan.rs` to consume the typed
  `SpireDmlFrontdoorPrimitivePlanExpr` from the DML frontdoor module instead of
  carrying a parallel PK predicate parser.
- Keeps the 30873 placement gate and verifies the primitive-plan index OID still
  matches the placement-gated index before adding the DML CustomPath.
- Updates the Phase 11 task file with packet `30874`.

This packet is a refactor/hardening slice. It does not add UPDATE or DELETE
CustomScan routing.

## Validation

- `cargo test dml_frontdoor --lib`
  - `24 passed; 0 failed; 0 ignored; 1648 filtered out`
  - artifact: `artifacts/cargo-test-dml-frontdoor-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/dml_frontdoor.rs src/am/ec_spire/mod.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the baserestrictinfo helper belongs in the DML frontdoor module and
   sufficiently reuses the existing classifier/primitive-plan path.
2. Confirm the CustomScan planner no longer has its own PK predicate parser.
3. Confirm the placement-gated index OID check remains correct after the helper
   migration.
