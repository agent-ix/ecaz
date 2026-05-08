# SPIRE Manifest Publication Blocked Result Coverage

## Scope

This packet tightens coverage for
`ec_spire_remote_epoch_manifest_publication_result_summary(...)` by asserting
the blocked persistence path.

Code checkpoint: `8d048cd2` (`Cover SPIRE manifest publication blocked result`)

## Changes

- Extends the missing-persistence manifest catalog PG18 test to query the new
  publication result summary.
- Asserts that missing manifest persistence reports:
  - `result_source = 'blocked'`
  - `libpq_receive_count = 0`
  - `status = 'requires_remote_epoch_manifest_persistence'`
  - `next_blocker = 'remote_epoch_manifest_persistence'`

## Files

- `src/lib.rs`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_catalog_summary_missing`
- `git diff --check`

## Notes

This is a coverage-only checkpoint for the result-summary surface added in the
previous packet.
