# Review Request: SPIRE Post-Build Insert Fixture Split

## Summary

Packet 31019 extends `src/tests/insert.rs` by moving the later
post-build insert fixture block out of `src/tests/mod.rs`:

- `test_ec_spire_insert_after_build_multi_row_epoch_progression`
- `test_ec_spire_insert_after_build_rejects_dimension_mismatch`
- `test_ec_spire_insert_after_build_rejects_null_value`
- `test_ec_spire_insert_bootstraps_empty_index_epoch`

The move keeps the existing textual-include strategy, so the fixtures
remain inside the `#[pg_schema] mod tests` scope. The source-identity
fixtures remain in `src/tests/mod.rs` and the tracker keeps
`tests/insert.rs` open.

Code checkpoint: `107a6d3af3edc49f9d9a07ba78bbc7035eea37f9`

## Review Focus

- Confirm this is a mechanical relocation of the post-build insert block.
- Confirm source-identity fixtures intentionally remain in
  `src/tests/mod.rs`.
- Confirm the tracker correctly keeps `tests/insert.rs` open.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_insert_after_build_multi_row_epoch_progression -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_insert_after_build_rejects_dimension_mismatch -- --nocapture`
- `rg -n 'fn test_ec_spire_insert_after_build_multi_row_epoch_progression|fn test_ec_spire_insert_after_build_rejects_dimension_mismatch|fn test_ec_spire_insert_bootstraps_empty_index_epoch|fn test_ec_spire_srcid_uuid_global_ids' src/tests/insert.rs src/tests/mod.rs`
- `wc -l src/tests/mod.rs src/tests/insert.rs src/lib.rs`
- `git diff --check`

Artifacts and key result lines are recorded in
`review/31019-spire-post-build-insert-fixture-split/artifacts/manifest.md`.
