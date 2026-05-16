# Review Request: SPIRE Source Identity Insert Fixture Split

## Summary

Packet 31021 closes the `tests/insert.rs` row by moving the remaining
source-identity fixture block out of `src/tests/mod.rs` and into
`src/tests/insert.rs`:

- `test_ec_spire_srcid_uuid_global_ids`
- `test_ec_spire_boundary_replica_identity_snapshot_global_ids`
- `test_ec_spire_srcid_bytea_bootstrap_global`
- `test_ec_spire_srcid_requires_include_column`
- `test_ec_spire_include_requires_srcid_reloption`
- `test_ec_spire_srcid_rejects_bad_type`
- `test_ec_spire_srcid_rejects_null`
- `test_ec_spire_srcid_rejects_bad_bytea_width`

The focused source-identity validation exposed a stale shared helper
constraint: remote heap resolution rejected INCLUDE indexes because
`resolve_single_base_heap_index_attnum` required `ii_NumIndexAttrs == 1`.
SPIRE source identity indexes have one vector key plus one INCLUDE
column, so the helper now requires exactly one key attr
(`ii_NumIndexKeyAttrs == 1`) and allows INCLUDE attrs.

Code checkpoint: `f8825e31ef8dac5da610de02b04965cf23bd79d5`

## Review Focus

- Confirm the source-identity fixture relocation preserves the existing
  `#[pg_schema] mod tests` scope.
- Confirm the shared helper change is appropriately narrow: one key attr
  is still required, but INCLUDE attrs no longer trip heap resolution.
- Confirm the tracker correctly marks `tests/insert.rs` closed.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_srcid_uuid_global_ids -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_srcid_bytea_bootstrap_global -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_srcid_rejects_bad_bytea_width -- --nocapture`
- `rg -n 'fn test_ec_spire_srcid_uuid_global_ids|fn test_ec_spire_srcid_bytea_bootstrap_global|fn test_ec_spire_srcid_rejects_bad_bytea_width|single-key indexes only' src/tests/insert.rs src/tests/mod.rs src/am/ec_hnsw/source.rs`
- `wc -l src/tests/mod.rs src/tests/insert.rs src/lib.rs src/am/ec_hnsw/source.rs`
- `git diff --check`

Artifacts and key result lines are recorded in
`review/31021-spire-source-identity-insert-fixture-split/artifacts/manifest.md`.
