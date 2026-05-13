# Review Request: SPIRE Scan Fixture Split

- Code commit: `475fcb0d` (`Move SPIRE scan fixtures out of test sink`)
- Task: Task 30 Phase 12b.2, `src/lib.rs` PG18 fixture sink split
- Scope: source layout only; no fixture assertions, SQL, scan, routing, or centroid behavior changed

## Summary

This checkpoint starts `src/tests/scan.rs` by moving the scan-placement, scan
pipeline, routing, and centroid-classification fixture block out of
`src/tests/mod.rs`.

`src/tests/mod.rs` includes the file textually:

```rust
include!("scan.rs");
```

That keeps the moved pg_tests in the same `#[pg_schema] mod tests` scope and
preserves existing pgrx-discovered fixture names. This does not close the full
`tests/scan.rs` Phase 12b.2 row yet because later build/scan fixtures still
live in `src/tests/mod.rs`.

## Validation

- Format check:
  `review/31003-spire-scan-fixture-split/artifacts/cargo-fmt-check.log`
- Focused PG18 moved fixture:
  `review/31003-spire-scan-fixture-split/artifacts/cargo-test-scan-fixture.log`
- Fixture location and line-count sanity:
  `review/31003-spire-scan-fixture-split/artifacts/fixture-location-check.log`

See `artifacts/manifest.md` for exact commands and key result lines.

## Reviewer Focus

1. Confirm the extraction range starts at `test_ec_spire_scan_placement_snapshot_sql`.
2. Confirm the range ends before `test_ec_spire_schema_drift_fails_before_dispatch_sql`.
3. Confirm the tracker correctly records this as a partial scan split, not a closed `tests/scan.rs` row.
