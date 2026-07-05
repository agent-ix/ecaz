# Review Request: SPIRE DML Baserel UPDATE/DELETE Handoff

## Scope

Code commit: `1f1649bc0dd0725ad1e690d443ce5bc530a23e3d`

This packet generalizes the DML frontdoor baserel primitive-plan expression
handoff so UPDATE, DELETE, and PK SELECT can share one extraction path.

Changes:

- Renames the baserel primitive-plan expression helper to the operation-neutral
  `dml_frontdoor_primitive_plan_expr_from_baserel(...)`.
- Keeps the existing PK SELECT wrapper for the currently wired CustomScan path.
- Adds target-relation filtering for DML baserels: UPDATE/DELETE return `None`
  for non-result relations instead of failing the candidate path.
- Reuses the RestrictInfo-first predicate extraction for UPDATE, DELETE, and
  PK SELECT, falling back to analyzed `Query` quals when needed.
- Carries operation-specific UPDATE target columns and PK SELECT projected
  columns through the shared shape classifier.
- Updates the Phase 11 task file with packet `30876`.

This is still a scaffold packet. It does not wire transparent UPDATE/DELETE
CustomScan or ModifyTable replacement yet.

## Validation

- `cargo test dml_frontdoor --lib`
  - successful rerun: `25 passed; 0 failed; 0 ignored; 1648 filtered out`
  - artifact: `artifacts/cargo-test-dml-frontdoor-lib.rerun.log`
  - first sandboxed attempt failed while cargo-pgrx installed into the local
    PG18 tree; retained as `artifacts/cargo-test-dml-frontdoor-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm UPDATE/DELETE baserel handoff skips non-target baserels instead of
   raising candidate errors for joined DML shapes.
2. Confirm the PK SELECT wrapper still preserves the currently wired CustomScan
   mode boundary.
3. Confirm the shared query-detail extraction carries UPDATE updated columns and
   PK SELECT projected columns without changing DELETE payload semantics.
