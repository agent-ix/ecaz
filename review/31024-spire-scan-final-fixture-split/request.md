# Review Request: SPIRE Scan Final Fixture Split

## Summary

This cleanup slice moves the remaining SPIRE scan fixtures out of
`src/tests/mod.rs` into `src/tests/scan.rs`:

- `test_ec_spire_empty_build_scan_no_rows`
- `test_ec_spire_empty_pq_fastscan_build_scan_no_rows`
- `test_ec_spire_flat_recursive_same_candidate`

The Phase 12b tracker now marks `tests/scan.rs` closed. The change is
intended as fixture relocation only; fixture bodies were moved unchanged.

Code commit: `1312b56f43955c0eb4fbb06cc98c44d83446c018`

## Validation

Packet-local logs are in `artifacts/`.

Passing checks:

- `cargo fmt --check`
- `git diff --check`
- location check confirms the moved scan fixtures now live in
  `src/tests/scan.rs`, while non-scan SPIRE fixtures remain in
  `src/tests/mod.rs`
- PG18 focused tests:
  - `test_ec_spire_empty_build_scan_no_rows`
  - `test_ec_spire_empty_pq_fastscan_build_scan_no_rows`
  - `test_ec_spire_flat_recursive_same_candidate`

## Review Focus

Please check that:

- the scan fixture bodies were moved without semantic edits;
- `src/tests/mod.rs` no longer retains a scan concern block;
- closing `tests/scan.rs` is appropriate.
