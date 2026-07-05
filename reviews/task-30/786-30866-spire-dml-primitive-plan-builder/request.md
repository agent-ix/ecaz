# Review Request: SPIRE DML Primitive Plan Builder

## Scope

This packet adds the typed primitive plan builder the future DML CustomScan
executor will consume after the planner hook classifies a supported ADR-069
shape. It does not enable plan rewriting or dispatch remote DML from the hook.

Code commit: `7f5759f2db6a7a2b3dd3a3be996f096a9fd91334`

Changes:

- Adds `SpireDmlFrontdoorPrimitivePlan`, carrying the index OID, typed
  CustomScan mode, expected primitive, PK argument, and operation-specific
  column payloads.
- Adds `SpireDmlFrontdoorCustomScanMode` for the three supported v1 DML modes:
  coordinator UPDATE tuple payload, coordinator DELETE tuple payload, and
  coordinator PK SELECT tuple payload.
- Adds `dml_frontdoor_primitive_plan_from_replacement_decision(...)`, which
  validates supported decisions, valid index OID, exact mode/primitive pairing,
  PK argument shape, and the UPDATE/DELETE/PK SELECT column-payload contract.
- Re-exports the helper and mode enum through the `ec_spire` and `am` module
  boundaries for the upcoming executor replacement slice.
- Adds PG18 coverage for UPDATE, DELETE, PK SELECT, mode/primitive mismatch,
  and unsupported embedding UPDATE rejection.
- Updates the Phase 11 task file with the 30866 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 22 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the primitive plan shape is sufficient as the handoff object for the
   DML CustomScan executor replacement.
2. Confirm mode/primitive validation matches the ADR-069 replacement-decision
   contract.
3. Confirm operation-specific column validation prevents UPDATE, DELETE, and PK
   SELECT from consuming the wrong executor payload shape.
