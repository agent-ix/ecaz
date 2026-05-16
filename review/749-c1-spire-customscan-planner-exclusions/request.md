# Review Request: SPIRE CustomScan Planner Exclusions

## Summary

This checkpoint closes Phase 12c.1.d in the updated SPIRE test-coverage tracker.

Changes:

- Added `src/tests/custom_scan_planner.rs` as a new focused test concern file to avoid growing `custom_scan.rs`.
- Added JSON EXPLAIN coverage that asserts a partitioned `ORDER BY ... LIMIT` fixture exercises `Merge Append` without planning SPIRE `Custom Scan` below it.
- Added JSON EXPLAIN coverage that asserts a correlated LATERAL fixture exercises `Nested Loop` without planning SPIRE `Custom Scan` under the rescan side.
- Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` 12c.1.d rows with the new evidence names.

## Review Focus

Please review whether the two planner fixtures are strict enough for the `MarkPos` / `RestrPos` exclusion contract:

- `test_ec_spire_customscan_not_below_mergeappend_sql`
- `test_ec_spire_customscan_not_inner_rescan_nested_loop_sql`

The intent is to pin the planner-side behavior corresponding to the callback contract already covered by `custom_scan_exec_methods_do_not_advertise_mark_restore_callbacks`.

## Validation

- `cargo fmt --check` passed.
- `git diff --check -- src/tests/custom_scan_planner.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md` passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_customscan_not --no-run` passed.
- `cargo pgrx test pg18 test_ec_spire_customscan_not` failed before running the tests with the existing loader error: `undefined symbol: pg_re_throw`.

## Files

- `src/tests/custom_scan_planner.rs`
- `src/tests/mod.rs`
- `plan/tasks/task30-phase12c-spire-test-coverage.md`
