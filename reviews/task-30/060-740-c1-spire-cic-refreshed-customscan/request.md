# Review Request: SPIRE CIC Refreshed CustomScan

agent: coder1
date: 2026-05-14
code commit: `e1b8154813a85425d09020754bd8f5ff43c4b192`
task rows: closes `12c.3.e` residual bullet, `12c.3.f`

## Summary

This checkpoint fills the lifecycle gap that the updated tracker still
showed for CIC descriptor refresh. Packet `715` covered descriptor
defer/receive behavior, but its request explicitly left the later
full-CustomScan refreshed-descriptor assertion open. This slice adds
that assertion instead of reconciling it away.

## Changes

- Added `src/tests/custom_scan_lifecycle.rs`.
- New test:
  `test_ec_spire_customscan_uses_cic_refreshed_descriptor_sql`.
- The fixture:
  - creates a loopback remote table and old remote index;
  - creates a coordinator table/index and rewrites coordinator
    placements to the remote node;
  - registers the old descriptor;
  - creates a new remote index with `CREATE INDEX CONCURRENTLY`;
  - registers a higher-generation descriptor pointing at the new
    remote index;
  - asserts the descriptor catalog now names the new generation,
    new index, and new identity;
  - drops the old remote index;
  - runs a normal CustomScan and asserts it still returns the
    expected remote row through the refreshed descriptor.
- Updated the tracker for:
  - `12c.3.e` second bullet.
  - `12c.3.f`.

File-size check:

- `src/tests/custom_scan_lifecycle.rs`: 189 lines.
- `src/tests/custom_scan.rs`: 1479 lines.
- `src/tests/custom_scan_fanout.rs`: 257 lines.
- `src/tests/custom_scan_concurrency.rs`: 572 lines.

## Validation

- `cargo fmt --check`
  - Passed, with existing stable-rustfmt warnings about unstable import
    grouping options.
- `git diff --check -- src/tests/custom_scan_lifecycle.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_uses_cic_refreshed_descriptor_sql --no-run`
  - Passed.
- `cargo pgrx test pg18 test_ec_spire_customscan_uses_cic_refreshed_descriptor_sql`
  - Failed before test execution with the existing loader error:
    `undefined symbol: pg_re_throw`.

## Review Focus

- Please check whether dropping the old remote index before the final
  CustomScan is a sufficient guard that the scan is using the refreshed
  descriptor.
- Please check whether this closes the residual `12c.3.e`/`12c.3.f`
  tracker bullets that packet `715` intentionally left open.
- Please check the new file placement against the post-split
  `custom_scan_*` layout.
