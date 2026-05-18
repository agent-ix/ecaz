# Review Request: SPIRE CustomScan Multi-Remote Fanout

- agent: coder1
- date: 2026-05-14
- code commit: `aaf76df5`
- task rows: closes `12c.7.a`

## Summary

Added focused CustomScan fanout coverage for three remote nodes, plus the
optional fanout=8 widening variant from the tracker.

## What Changed

Added `src/tests/custom_scan_fanout.rs` and included it from
`src/tests/mod.rs`.

New fixtures:

- `test_ec_spire_customscan_three_remote_fanout_sql`
- `test_ec_spire_customscan_eight_remote_fanout_sql`

The shared helper builds a coordinator SPIRE index and one loopback remote
index per remote node. It rewrites selected coordinator PIDs across disjoint
remote node IDs, registers one descriptor per remote index, probes each remote
endpoint for the expected origin-coded payload, then asserts CustomScan returns
that exact union. The JSON EXPLAIN assertion also pins `remote_fanout` to the
remote node count.

Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` to mark all
`12c.7.a` bullets complete.

## Test File Size Discipline

This uses a new sibling file rather than growing `custom_scan.rs`:

```text
254 src/tests/custom_scan_fanout.rs
1475 src/tests/custom_scan.rs
572 src/tests/custom_scan_concurrency.rs
452 src/tests/data_shape.rs
2317 src/tests/insert.rs
525 src/tests/insert_remote_trigger.rs
```

`src/tests/mod.rs` remains over target from existing root include structure;
this slice adds only a one-line include there.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/custom_scan_fanout.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_three_remote_fanout_sql --no-run
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_eight_remote_fanout_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

Blocked before test execution:

```text
cargo pgrx test pg18 test_ec_spire_customscan_three_remote_fanout_sql
```

Result:

```text
undefined symbol: pg_re_throw
```

The pg_test binary failed at local loader startup before the focused test body
could run.

## Review Focus

- Confirm origin-coded remote payloads are sufficient “origin-remote metadata”
  for `12c.7.a`.
- Confirm comparing the CustomScan result set to per-node endpoint probes is
  the right expected-union assertion.
- Confirm the fanout=8 widening variant closes the optional P3 bullet in
  `12c.7.a`.
