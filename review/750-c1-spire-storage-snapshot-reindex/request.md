# Review Request: SPIRE Storage Snapshot During REINDEX

## Summary

This checkpoint closes Phase 12c.13.c from the updated SPIRE test-coverage tracker.

Changes:

- Added `src/tests/diagnostics_reindex.rs` as a focused diagnostics concurrency test file to avoid growing `diagnostics.rs`.
- Added `test_ec_spire_relation_storage_snapshot_during_reindex_sql`.
- The test starts a `REINDEX INDEX` from a separate session, waits until it is active and blocked on a table lock, then calls `ec_spire_index_relation_storage_snapshot` from the test session.
- The assertion pins sane behavior during concurrent REINDEX: snapshot returns without panic, reports an active epoch, reports object tuples, and keeps `physical_cleanup_supported = true`.
- Updated `plan/tasks/task30-phase12c-spire-test-coverage.md` 12c.13.c rows with the new evidence.

## Review Focus

Please review whether the lock-waiting REINDEX window is an acceptable deterministic proxy for the “mid-REINDEX” coverage row. It avoids a large slow corpus while still proving the snapshot surface tolerates an in-progress REINDEX operation in another session.

## Validation

- `cargo fmt --check` passed.
- `git diff --check -- src/tests/diagnostics_reindex.rs src/tests/mod.rs plan/tasks/task30-phase12c-spire-test-coverage.md` passed.
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_relation_storage_snapshot_during_reindex_sql --no-run` passed.
- `cargo pgrx test pg18 test_ec_spire_relation_storage_snapshot_during_reindex_sql` failed before running the test with the existing loader error: `undefined symbol: pg_re_throw`.

## Files

- `src/tests/diagnostics_reindex.rs`
- `src/tests/mod.rs`
- `plan/tasks/task30-phase12c-spire-test-coverage.md`
