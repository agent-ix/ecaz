# Review Request: SPIRE Multi-Store Counter Sums

- agent: coder1
- date: 2026-05-14
- code commit: `c92a40b80c5378bf6d6f8993635d8af8e3febcca`
- task rows: closes remaining `12c.15.a` counter-sum bullet

## Summary

Tightened the existing multi-store scan-width helper so the three-store fixture
asserts the per-store counter rows match overall route/candidate/byte
expectations.

## What Changed

Updated `assert_ec_spire_multistore_scan_width_sql` in `src/tests/scan.rs` to
query `ec_spire_index_scan_local_store_read_overlap_harness` as per-store rows
and assert:

- every expected local store has a row
- every local store has nonzero `route_count`
- every local store has nonzero `candidate_row_count`
- every local store has nonzero `prefetched_object_bytes`
- summed per-store `route_count` matches
  `ec_spire_index_scan_placement_snapshot`
- summed per-store `candidate_row_count` matches
  `ec_spire_index_scan_placement_snapshot`
- summed per-store `prefetched_object_bytes` is positive

The helper is shared by the 3-store and 4-store fixtures, but this specifically
closes the remaining `12c.15.a` bullet.

## Test File Size Discipline

The touched file remains below the 2500-line target:

```text
1355 src/tests/scan.rs
```

No new large fixture file was introduced.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/scan.rs plan/tasks/task30-phase12c-spire-test-coverage.md
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_three_store_scan_width_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

Blocked before test execution:

```text
cargo pgrx test pg18 test_ec_spire_three_store_scan_width_sql
```

Result:

```text
undefined symbol: pg_re_throw
```

The pg_test binary failed at local loader startup before the focused test body
could run.

## Review Focus

- Confirm cross-checking the read-overlap harness against
  `ec_spire_index_scan_placement_snapshot` is the right overall-counter
  comparison for `12c.15.a`.
- Confirm `prefetched_object_bytes > 0` per store is sufficient for the byte
  counter portion of the row.
