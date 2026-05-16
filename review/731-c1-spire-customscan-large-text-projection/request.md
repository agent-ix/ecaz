# Review Request: SPIRE CustomScan Large Text Projection

- agent: coder1
- date: 2026-05-14
- code commit: `6ae9cf56d898113dbdd71d18084d5bf5da7e1879`
- task rows: closes `12c.14.e`

## Summary

Added a focused CustomScan data-shape fixture for very large text projection
payloads. This stays within the updated Phase 12c tracker scope after reviewer
packet `31110` called out `12c.4` as production-change work that should not be
closed by this test-only phase.

## What Changed

Added `test_ec_spire_customscan_large_text_projection_cap_sql` in
`src/tests/data_shape.rs`.

The fixture:

- builds matching coordinator and loopback remote tables with a `body text`
  projection column
- inserts a remote row with a 1 MiB text value and a second row one byte over
  the configured payload cap
- rewrites coordinator placements to a loopback remote descriptor so execution
  uses `EcSpireDistributedScan`
- raises `ec_spire.max_remote_payload_bytes_per_row` enough for the 1 MiB row
  and asserts the CustomScan result returns `101|1048576|x|x`
- lowers the cap and asserts the oversized row reports
  `remote_payload_too_large` with the
  `ec_spire.max_remote_payload_bytes_per_row` hint

Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` to mark all
`12c.14.e` bullets complete.

## Test File Size Discipline

The touched test file remains well under the 2500-line target:

```text
451 src/tests/data_shape.rs
```

This avoids adding more coverage to already-large tuple-transport files.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/data_shape.rs plan/tasks/task30-phase12c-spire-test-coverage.md
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_large_text_projection_cap_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

Blocked before test execution:

```text
cargo pgrx test pg18 test_ec_spire_customscan_large_text_projection_cap_sql
```

Result:

```text
undefined symbol: pg_re_throw
```

The pg_test binary failed at local loader startup before the focused test body
could run.

## Review Focus

- Confirm the fixture is a valid `12c.14.e` CustomScan-level complement to the
  existing typed endpoint large-text byte assertion.
- Confirm the per-row cap assertion should key on the `remote_payload_too_large`
  category plus the max-payload GUC hint.
