# Review Request: SPIRE Placement Fixture Split

- Code commit: `7a665828` (`Move SPIRE placement fixtures out of test sink`)
- Task: Task 30 Phase 12b.2, `src/lib.rs` PG18 fixture sink split
- Scope: source layout only; no fixture assertions, SQL, placement behavior, or scan behavior changed

## Summary

This checkpoint starts `src/tests/placement.rs` by moving the placement catalog
and placement-snapshot fixture block out of `src/tests/mod.rs`.

`src/tests/mod.rs` includes the file textually:

```rust
include!("placement.rs");
```

That keeps the moved pg_tests in the same `#[pg_schema] mod tests` scope and
preserves existing pgrx-discovered fixture names. This does not close the full
`tests/placement.rs` Phase 12b.2 row yet because the scan-placement and later
placement contention/diagnostic fixtures still live in `src/tests/mod.rs`.

## Validation

- Format check:
  `review/31002-spire-placement-fixture-split/artifacts/cargo-fmt-check.log`
- Focused PG18 moved fixture:
  `review/31002-spire-placement-fixture-split/artifacts/cargo-test-placement-fixture.log`
- Fixture location and line-count sanity:
  `review/31002-spire-placement-fixture-split/artifacts/fixture-location-check.log`

See `artifacts/manifest.md` for exact commands and key result lines.

## Reviewer Focus

1. Confirm the extraction range starts at `test_ec_spire_placement_directory_catalog_sql`.
2. Confirm the range ends before `test_ec_spire_scan_placement_snapshot_sql`.
3. Confirm the tracker correctly records this as a partial placement split, not a closed `tests/placement.rs` row.
