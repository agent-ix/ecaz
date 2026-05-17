# Review Request: SPIRE DML Primitive Invocation Builder

## Scope

Code commit: `ef87b8b075be871c210ab110cfd91344e16f30f6`

This packet adds the executor-ready SPIRE DML primitive invocation boundary. It combines a typed DML primitive plan with runtime `ParamListInfo` PK byte evaluation and produces the exact argument bundle a future DML CustomScan executor will pass to coordinator primitives.

This packet does not enable planner rewrite, executor dispatch, or remote DML forwarding.

## Changes

- Adds `SpireDmlFrontdoorPrimitiveInvocation` with:
  - `index_oid`
  - DML CustomScan mode
  - coordinator primitive function name
  - PK column
  - PK value bytes
  - updated column list
  - projected column list
- Adds `dml_frontdoor_primitive_invocation_from_plan(...)` to build the invocation from a typed primitive plan and runtime params.
- Reuses the existing const/parameter PK byte conversion helper so const and runtime parameter paths share one bytea boundary.
- Re-exports the builder through the SPIRE AM module surfaces.
- Extends PG18 DML frontdoor coverage for:
  - const PK SELECT invocation
  - bound-parameter PK invocation
- Updates the Phase 11 task file with the 30870 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - `23 passed; 0 failed; 0 ignored; 1648 filtered out`
  - artifact: `artifacts/cargo-test-dml-frontdoor-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm `SpireDmlFrontdoorPrimitiveInvocation` is the right handoff object for the upcoming DML CustomScan executor.
2. Confirm const/parameter PK byte evaluation now happens at the correct executor-adjacent boundary.
3. Confirm this packet remains a builder/coverage slice and does not accidentally enable planner rewrite or remote DML dispatch.
