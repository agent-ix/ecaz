# Review Request: SPIRE CustomScan read cancellation

## Summary

This checkpoint closes the Phase 12b read-path cancellation coverage row.

Code checkpoint: `86fae0f50fd12749f97d96d0d377dac281f0ebb6`

The slice adds a CustomScan pg_test that builds a loopback remote descriptor,
forces the `EcSpireDistributedScan` read path, sets PostgreSQL query-cancel
flags, and asserts the read query is interrupted at the backend boundary.

It also tightens the receive-layer local-cancel fixture by replacing a
hard-coded endpoint identity with the actual loopback remote index identity, so
the fixture reaches the local-cancel path instead of failing early on endpoint
identity mismatch.

The executor cancellation recommendation now includes the tracked cancellation
category for local query cancel and local statement timeout cases. This keeps
blocked CustomScan/diagnostic text specific without adding another summary
field.

## Scope Guard

This slice does not add to shrink-list files:

- `src/tests/remote_search.rs` remains deleted.
- `src/tests/mod.rs` is unchanged by this checkpoint.

The changed split-file test remains small:

- `src/tests/custom_scan.rs`: 1,063 lines.
- `src/tests/remote_search/receive_faults.rs`: 941 lines.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz test_ec_spire_customscan_read_cancel_releases_transport`
- `cargo test -p ecaz test_ec_spire_prod_receive_local_cancel_remote_cancel`

Raw logs and line counts are in `artifacts/`.

## Reviewer Focus

- Confirm the CustomScan test is the right boundary for PostgreSQL query cancel:
  it proves the read query is interrupted, while the receive-layer fixture
  proves `local_query_cancelled` categorization and governance lock release.
- Confirm including the cancellation category in the recommendation string is
  acceptable for diagnostics and CustomScan blocked-error text.
- Confirm using the actual loopback remote index identity in the receive fixture
  is the correct fix for the endpoint identity precondition.
