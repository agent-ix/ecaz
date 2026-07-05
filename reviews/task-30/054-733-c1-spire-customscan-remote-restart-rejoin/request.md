# Review Request: SPIRE CustomScan Remote Restart Rejoin

- agent: coder1
- date: 2026-05-14
- code commit: `c815c26d`
- task rows: closes `12c.8.c`

## Summary

Added a focused CustomScan concurrency fixture for the remote-restart row using
remote backend termination during candidate receive as the restart simulation.
The test then verifies strict failure, degraded skip reporting, and successful
CustomScan execution after the remote descriptor rejoins.

## What Changed

Extended `src/tests/custom_scan_concurrency.rs` with
`test_ec_spire_customscan_remote_backend_termination_rejoin_sql`.

The fixture:

- builds coordinator and loopback remote tables/indexes
- registers a loopback remote descriptor whose conninfo search path overrides
  `ec_spire_remote_search`
- makes the overridden remote function sleep briefly, then terminate its own
  backend during candidate receive
- verifies the query plans as `Custom Scan (EcSpireDistributedScan)`
- asserts strict mode fails closed with a remote termination/category error
- asserts degraded mode returns no rows for the all-remote fixture
- asserts `ec_spire_remote_search_production_scan_handoff_summary` reports one
  degraded skip with `remote_backend_terminated`
- refreshes the same descriptor generation to normal loopback conninfo and
  asserts a subsequent strict CustomScan returns the expected remote row ids

Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` to mark all
`12c.8.c` bullets complete, with an evidence note naming the backend
termination/rejoin simulation.

## Test File Size Discipline

This reuses the small sibling concurrency file rather than adding to
`custom_scan.rs`:

```text
379 src/tests/custom_scan_concurrency.rs
1475 src/tests/custom_scan.rs
452 src/tests/data_shape.rs
107 src/tests/dml_concurrency.rs
```

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/custom_scan_concurrency.rs plan/tasks/task30-phase12c-spire-test-coverage.md
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_remote_backend_termination_rejoin_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

Blocked before test execution:

```text
cargo pgrx test pg18 test_ec_spire_customscan_remote_backend_termination_rejoin_sql
```

Result:

```text
undefined symbol: pg_re_throw
```

The pg_test binary failed at local loader startup before the focused test body
could run.

## Review Focus

- Confirm backend termination during remote candidate receive is an acceptable
  test-only simulation for the `12c.8.c` remote restart row.
- Confirm the degraded-mode coverage should combine the full CustomScan empty
  result assertion with the scan handoff diagnostic summary assertion for the
  named degraded skip category.
- Check whether the rejoin half should keep using the same descriptor secret
  with updated env value, as this test does, or use a distinct secret to model
  operator rotation.
