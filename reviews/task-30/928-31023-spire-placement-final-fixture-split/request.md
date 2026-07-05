# Review Request: SPIRE Placement Final Fixture Split

## Summary

This cleanup slice moves the remaining SPIRE placement fixture out of
`src/tests/mod.rs` into `src/tests/placement.rs`:

- `test_pg18_ec_spire_placement_write_contention_distinct_pk_dml`

The Phase 12b tracker now marks `tests/placement.rs` closed. The change is
intended as fixture relocation only; the fixture body was moved unchanged.

Code commit: `b5bb0c3086c94429701026fea37924b3bae58dd4`

## Validation

Packet-local logs are in `artifacts/`.

Passing checks:

- `cargo fmt --check`
- `git diff --check`
- location check confirms the placement contention fixture now lives in
  `src/tests/placement.rs`, while the following relation-storage fixture
  remains in `src/tests/mod.rs`
- PG18 focused test:
  - `test_pg18_ec_spire_placement_write_contention_distinct_pk_dml`

## Review Focus

Please check that:

- the placement contention fixture was moved without semantic edits;
- `src/tests/mod.rs` no longer retains a placement concern block;
- closing `tests/placement.rs` is appropriate now that scan-placement is in
  `src/tests/scan.rs` and placement diagnostics are in
  `src/tests/diagnostics.rs`.
