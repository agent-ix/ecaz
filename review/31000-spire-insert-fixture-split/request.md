# Review Request: SPIRE Insert Fixture Split

- Code commit: `6f682516` (`Move SPIRE insert fixtures out of test sink`)
- Task: Task 30 Phase 12b.2, `src/lib.rs` PG18 fixture sink split
- Scope: source layout only; no fixture assertions, SQL, insert dispatch, or trigger behavior changed

## Summary

This checkpoint starts `src/tests/insert.rs` by moving the contiguous
coordinator-insert and insert-trigger fixture block out of `src/tests/mod.rs`.

`src/tests/mod.rs` includes the file textually:

```rust
include!("insert.rs");
```

That keeps the moved pg_tests in the same `#[pg_schema] mod tests` scope and
preserves existing pgrx-discovered fixture names. This does not close the full
`tests/insert.rs` Phase 12b.2 row yet because the later insert-after-build
fixtures still live in `src/tests/mod.rs`.

## Validation

- Format check:
  `review/31000-spire-insert-fixture-split/artifacts/cargo-fmt-check.log`
- Focused PG18 moved fixture:
  `review/31000-spire-insert-fixture-split/artifacts/cargo-test-insert-fixture.log`
- Fixture location and line-count sanity:
  `review/31000-spire-insert-fixture-split/artifacts/fixture-location-check.log`

See `artifacts/manifest.md` for exact commands and key result lines.

## Reviewer Focus

1. Confirm the extraction range starts at `test_ec_spire_plan_coordinator_insert_sql`.
2. Confirm the range ends before `test_ec_spire_schema_drift_fails_before_dispatch_sql`.
3. Confirm the tracker correctly records this as a partial insert split, not a closed `tests/insert.rs` row.
