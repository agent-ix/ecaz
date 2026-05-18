# Review Request: Task 41 DiskANN materialized-chain helper reuse

## Summary

Task 41 follow-up for the last test-only raw `index_open` sites in
`src/am/ec_diskann/routine.rs`.

Packet `31214` made `index_materialized_chain` guard-backed. This slice reuses
that helper at three callsites that still opened an index relation manually just
to call `scan_state::materialize_chain_from_index`.

Code commit: `3e17a136`

## Safety Effect

- Removes three manual `index_open` / `index_close` pairs from DiskANN PG tests.
- Centralizes those reads behind the guard-backed `index_materialized_chain`
  helper.
- Leaves `routine.rs` with only the production vacuum heap resolver in this
  specific raw relation-open pattern.
- Updates the unsafe comment baseline from `4114` to `4105`.

## Review Focus

- Confirm the helper reuse preserves the same index names and access-share
  materialization behavior.
- Confirm no removed `index_oid` local was used for anything besides the manual
  open/materialize/close sequence.
- Confirm the changed tests still materialize after the same inserts and before
  the same assertions.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
