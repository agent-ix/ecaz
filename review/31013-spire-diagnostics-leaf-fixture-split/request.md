# Review Request: SPIRE Diagnostics Leaf Fixture Split

## Summary

Code commit: `56657029b7f0784c5e89f0eaf8660a6c1db5382f`

This checkpoint moves the leaf snapshot fixture from `src/tests/mod.rs` into `src/tests/diagnostics.rs`, completing the Phase 12b.2 diagnostics concern file.

The move keeps the textual include pattern, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Moved `test_ec_spire_leaf_snapshot_sql`.
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31013` as the diagnostics closeout.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_leaf_snapshot_sql -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.
- `git diff --check`

The focused PG18 test passed. The test emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/diagnostics.rs` is closed. Other concern files remain open under the tracker.
