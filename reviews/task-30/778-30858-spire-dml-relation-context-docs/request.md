# Review Request: SPIRE DML Relation Context Docs

## Scope

This packet addresses the remaining P2 from the 30855 review by documenting
the deliberate split between the SPI-backed diagnostic relation-context loader
and the catalog/relcache-backed planner-hook loader.

Code commit: `52d9cca2d05c63c34c98925d9960feeb4fb7ebfd`

Changes:

- Adds a module-local comment that the SPI loader remains the operator
  diagnostic path while the catalog/relcache loader is the hook-safe production
  path because it avoids recursive SPI.
- Adds a comment beside `RelationGetIndexList` documenting that the returned
  index OID list is a private copy, so the loop can open and close each index
  under `AccessShareLock`.
- Updates the Phase 11 task tracker with the packet milestone.

No behavior changes are intended in this packet.

## Validation

- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the comments capture the 30855 review intent without overpromising
   a future migration to a single loader.
2. Confirm the `RelationGetIndexList` comment matches PostgreSQL ownership and
   lock-ordering expectations.
