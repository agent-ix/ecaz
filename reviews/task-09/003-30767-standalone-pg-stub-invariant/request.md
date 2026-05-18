# 30767 - Standalone PostgreSQL Stub Invariant

## Summary

This packet reviews commit `a388370f6c67cfe2976f154e43d2e71e127758b0`
(`Document standalone PG stub invariant`).

The slice addresses the `30764` reviewer P3 note by documenting the contract
for `csrc/standalone_pg_backend_stubs.c`: newly required PostgreSQL symbols
must be classified deliberately as either inert helper stubs or panicking
backend-execution stubs. Anything that would fake SPI, heap, catalog, or
executor behavior belongs in the pgrx/pg_test lane instead.

No behavior changes.

## Key Files

- `csrc/standalone_pg_backend_stubs.c`

## Validation

- `git diff --check -- csrc/standalone_pg_backend_stubs.c`
- `cargo test row_materialization_contract --no-default-features --features pg18`

No PostgreSQL distributed fixture or performance run was started for this
packet.

## Review Focus

- Does the invariant make the standalone-vs-backend test boundary explicit
  enough for future symbol additions?
- Is the wording strict enough to prevent fake backend behavior in direct
  cargo tests?
