# Review Request: SPIRE Populated Build Fixture Split

## Summary

Code commit: `cd72767fbf1a136ffd3556159b155016ee3bcd0b`

This checkpoint extends the Phase 12b.2 `tests/build.rs` concern file by moving the populated build root-control and logical-store hash-routing fixtures from `src/tests/mod.rs` into `src/tests/build.rs`.

The move keeps the textual include pattern, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Moved:
  - `test_ec_spire_populated_build_publishes_root_control`
  - `test_ec_spire_populated_build_hash_routes_logical_store_set`
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31014`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_populated_build_publishes_root_control -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_populated_build_hash_routes_logical_store_set -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.
- `git diff --check`

Both focused PG18 tests passed. The tests emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/build.rs` remains open because multistore, recursive-fanout, and top-graph build fixtures still live outside the concern file.
