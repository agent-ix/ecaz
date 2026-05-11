# Review Request: SPIRE DML Primitive Plan Diagnostic

## Scope

This packet exposes the typed DML primitive plan as a SQL diagnostic surface.
It lets reviewers verify that analyzed SQL produces the executor handoff object
and, for constant PK predicates, the ADR-069 `pk_value bytea` argument before
planner-hook replacement is enabled.

Code commit: `a9d8df2c944303589117f72a38f7c512eebd37d0`

Changes:

- Adds `ec_spire_dml_frontdoor_primitive_plan_sql(sql text)`.
- For supported constant-PK shapes, returns the CustomScan mode, coordinator
  primitive name, PK argument metadata, operation-specific column payloads, and
  `pk_value_bytes`.
- For unsupported shapes, returns `primitive_plan_not_ready` with the typed
  builder error instead of constructing an executor handoff object.
- Adds PG18 coverage for a ready PK SELECT primitive plan and an unsupported
  embedding UPDATE diagnostic path.
- Updates the Phase 11 task file with the 30869 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 23 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the diagnostic exposes the right executor handoff fields without
   implying planner replacement is enabled.
2. Confirm constant `pk_value_bytes` gives a useful review signal for the
   coordinator primitive argument.
3. Confirm unsupported shapes fail through the typed primitive-plan builder
   rather than bypassing ADR-069 validation.
