# Review Request: SPIRE CustomScan empty remote result fixture

## Summary

This checkpoint closes the Phase 12b empty-remote-result CustomScan fixture
gap.

The new fixture is
`test_ec_spire_customscan_empty_remote_result_returns_no_rows` in
`src/tests/custom_scan.rs`. It sets up a loopback remote node, registers a
remote placement for a primary-key select, verifies the remote helper sends the
request and returns `selected_count = 0`, then runs the CustomScan query and
asserts:

- the EXPLAIN surface is still `EcSpireDistributedScan`
- the query returns zero rows
- `EXPLAIN (FORMAT JSON, ANALYZE)` reports tuple transport `ready`
- the JSON plan does not leak `not_applicable`

Code checkpoint: `6fbdec7d1612b63786c497aeaf5dae07539187f7`

## Validation

- `cargo fmt --check`
- `git diff --check`
- location/diff-stat checks
- focused PG18:
  - `cargo pgrx test pg18 test_ec_spire_customscan_empty_remote_result_returns_no_rows`: passed

The packet also includes failed intermediate logs. Those runs document two
discarded fixture shapes:

- post-build vector deletes still returned a remote tuple, so they did not
  create a clean empty remote result
- the DML helper status for a zero-row remote response is
  `remote_select_ready` with `selected_count = 0`, not a distinct
  `remote_select_empty` status

Raw logs and command metadata are in `artifacts/manifest.md`.

## Reviewer Focus

- Confirm the DML PK-select CustomScan path is acceptable coverage for the
  empty-remote-result gap.
- Confirm the zero-row remote response is asserted before the CustomScan query
  and that the query itself returns zero rows.
- Confirm the tracker note is precise enough about this being the DML
  CustomScan empty-result path, not the vector remote-search tuple stream.
