# Review Request: SPIRE Insert Delta Fixture Split

## Summary

Packet 31020 extends `src/tests/insert.rs` by moving the earlier
post-build insert delta fixture block out of `src/tests/mod.rs`:

- `test_ec_spire_insert_after_build_delta_epoch`
- `test_ec_spire_insert_after_build_multiple_same_leaf_deltas`
- `test_pg18_ec_spire_concurrent_same_leaf_inserts`

The move keeps the existing textual-include strategy, so the fixtures
remain inside the `#[pg_schema] mod tests` scope. The PG18 placement
contention fixture and the source-identity fixtures remain in
`src/tests/mod.rs`, so `tests/insert.rs` stays open in the tracker.

Code checkpoint: `a75c8479970788c55899ad8a5aae70b8f964a3ac`

## Review Focus

- Confirm this is a mechanical relocation of the insert delta block.
- Confirm the placement contention fixture intentionally remains in
  `src/tests/mod.rs`.
- Confirm the tracker correctly keeps `tests/insert.rs` open because
  source-identity fixtures still remain.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_insert_after_build_multiple_same_leaf_deltas -- --nocapture`
- `cargo test --no-default-features --features pg18 test_pg18_ec_spire_concurrent_same_leaf_inserts -- --nocapture`
- `rg -n 'fn test_ec_spire_insert_after_build_delta_epoch|fn test_ec_spire_insert_after_build_multiple_same_leaf_deltas|fn test_pg18_ec_spire_concurrent_same_leaf_inserts|fn test_pg18_ec_spire_placement_write_contention_distinct_pk_dml' src/tests/insert.rs src/tests/mod.rs`
- `wc -l src/tests/mod.rs src/tests/insert.rs src/lib.rs`
- `git diff --check`

Artifacts and key result lines are recorded in
`review/31020-spire-insert-delta-fixture-split/artifacts/manifest.md`.
