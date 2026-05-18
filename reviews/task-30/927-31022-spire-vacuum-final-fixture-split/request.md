# Review Request: SPIRE Vacuum Final Fixture Split

## Summary

This cleanup slice moves the remaining SPIRE VACUUM fixtures out of
`src/tests/mod.rs` into `src/tests/vacuum.rs`:

- delete-delta visible-row suppression;
- no-delta cleanup;
- insert-delta cleanup compaction;
- mixed-delta leaf cleanup compaction;
- SQL VACUUM mixed-delta;
- multistore SQL VACUUM local-store routing;
- concurrent insert/VACUUM/scan.

The Phase 12b tracker now marks `tests/vacuum.rs` closed. The change is
intended as a fixture relocation only; it does not alter production code or
fixture bodies.

Code commit: `1feba980f6f7c3234179bf5cb94d044ac221aa4a`

## Validation

Packet-local logs are in `artifacts/`.

Passing checks:

- `cargo fmt --check`
- `git diff --check`
- location check confirms the moved vacuum fixtures now live in
  `src/tests/vacuum.rs`, while the following relation-storage fixture remains
  in `src/tests/mod.rs`
- PG18 focused tests:
  - `test_ec_spire_vacuum_cleanup_compacts_insert_delta`
  - `test_pg18_ec_spire_sql_vacuum_mixed_delta`
  - `test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores`

Known failing validation observed during this slice:

- `test_pg18_ec_spire_concurrent_insert_vacuum_scan` failed twice with the
  scan worker error
  `ec_spire remote search target plan requested epoch 3 does not match active epoch 4`.

The failed fixture was moved unchanged in this slice. I did not change the
fixture semantics in this cleanup commit; the failure is recorded in the
packet logs for reviewer visibility.

## Review Focus

Please check that:

- the vacuum fixture block was moved without semantic edits;
- `src/tests/mod.rs` no longer retains a vacuum concern block;
- closing `tests/vacuum.rs` in the tracker is appropriate despite the
  separately visible concurrent scan/VACUUM fixture failure.
