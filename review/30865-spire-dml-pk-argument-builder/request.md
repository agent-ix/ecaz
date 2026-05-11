# Review Request: SPIRE DML PK Argument Builder

## Scope

This packet adds the typed primary-key argument builder the future DML
CustomScan executor will use to turn a supported replacement decision into a
small executor-ready argument. It does not change planner path generation, plan
rewriting, or remote DML dispatch.

Code commit: `f557d9c35e4c65bd4c2a668824e6f25ada59787e`

Changes:

- Adds `SpireDmlFrontdoorPkArgument` with the selected PK column and typed PK
  value plan.
- Adds `SpireDmlFrontdoorPkValuePlan::{ConstBigint, ParamBigint}`.
- Adds `dml_frontdoor_pk_argument_from_replacement_decision(...)`, which
  accepts only supported replacement decisions and validates const/parameter
  bigint argument shape before executor construction.
- Re-exports the helper and value enum through the `ec_spire` and `am` module
  boundaries for the upcoming DML CustomScan executor path.
- Adds PG18 coverage for a supported PK SELECT decision and fail-closed behavior
  for an unsupported embedding UPDATE decision.
- Updates the Phase 11 task file with the 30865 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 21 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the typed argument shape is sufficient for the upcoming DML
   CustomScan executor.
2. Confirm const/parameter validation matches the replacement-decision contract.
3. Confirm unsupported decisions fail closed before any executor construction
   can consume them.
