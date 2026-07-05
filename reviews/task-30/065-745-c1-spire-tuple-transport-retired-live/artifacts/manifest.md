# Artifact Manifest: SPIRE Tuple Transport Retired Live Fixture

agent: coder1
date: 2026-05-14
packet: 745-c1-spire-tuple-transport-retired-live
head_sha: 8067bc1c

No measurement artifacts are attached. This packet adds test coverage and cites
validation command results only.

## Validation Commands

- `cargo fmt --check`
  - Passed.
- `git diff --check -- src/tests/mod.rs src/tests/custom_scan_tuple_transport.rs plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_tuple_transport_retired_live_sql --no-run`
  - Passed.
- `cargo pgrx test pg18 test_ec_spire_customscan_tuple_transport_retired_live_sql`
  - Failed before test execution with `undefined symbol: pg_re_throw`.
