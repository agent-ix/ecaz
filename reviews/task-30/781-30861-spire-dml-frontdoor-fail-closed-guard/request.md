# Review Request: SPIRE DML Frontdoor Fail-Closed Guard

## Scope

This packet adds the interim fail-closed planner-hook guard for ADR-069 DML
front-door shapes. Plan rewriting remains disabled; supported UPDATE, DELETE,
and PK SELECT shapes still pass through until the DML CustomScan executor
replacement lands.

Code commit: `51af7cbd4aa6416aa4554029ba5b864025c792fc`

Changes:

- Adds hook action tracking to `ec_spire_dml_frontdoor_hook_status()`.
- Exposes `unsupported_shape_fail_closed_enabled` and `last_hook_action` in the
  hook-status SQL surface.
- Raises the classifier's ADR-069 planner error for unsupported shapes that
  target an `ec_spire` DML front-door candidate, instead of falling through to
  the coordinator heap path.
- Keeps non-`ec_spire` relations on the pass-through path.
- Adds PG18 coverage proving an embedding-column UPDATE on an indexed
  distributed table fails closed with the ADR-069 error/hint, while the same
  UPDATE shape on a plain table still passes through.
- Updates the Phase 11 task file with the 30861 milestone.

## Validation

- `cargo test dml_frontdoor --lib`
  - 18 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the guard only errors for `ec_spire` front-door candidates and does
   not block ordinary heap tables.
2. Confirm supported ADR-069 DML shapes still pass through while plan rewriting
   remains disabled.
3. Confirm the hook-status fields give enough operator/reviewer visibility into
   the interim guard behavior before CustomScan executor replacement lands.
