# Review Request: SPIRE DML Baserel Mode Wrappers

## Scope

Code commit: `e23850136112c2b35049d2bb89a31d6a1ef8d336`

This packet adds mode-specific DML baserel primitive-plan wrapper surfaces for
the upcoming transparent UPDATE/DELETE rewrite slices.

Changes:

- Keeps the existing PK SELECT wrapper but routes its mode assertion through a
  shared guard.
- Adds UPDATE and DELETE wrappers around the generic baserel primitive-plan
  expression helper.
- Adds a shared mode guard that returns operation-specific fail-closed errors if
  the extracted primitive plan mode does not match the requested wrapper.
- Adds unit coverage for the guard's success and mismatch paths.
- Updates the Phase 11 task file with packet `30879`.

No CustomScan path generation or executor behavior changes are included.

## Validation

- `cargo test dml_frontdoor --lib`
  - `26 passed; 0 failed; 0 ignored; 1649 filtered out`
  - artifact: `artifacts/cargo-test-dml-frontdoor-lib.log`
- `cargo fmt --check`
  - passed
  - emits the known stable-rustfmt warnings for unstable import grouping options
  - artifact: `artifacts/cargo-fmt-check.log`
- `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs`
  - passed
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

1. Confirm the UPDATE/DELETE wrappers preserve the same `None` vs `Some(Err)`
   semantics as the generic baserel helper.
2. Confirm the shared mode guard keeps the existing PK SELECT fail-closed
   boundary intact.
3. Confirm this wrapper-only slice does not enable any new planner path before
   executor support exists.
