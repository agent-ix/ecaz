# Review Request: SPIRE CustomScan Fixture Split

- Code commit: `a212e3ad` (`Move SPIRE CustomScan fixtures out of test sink`)
- Task: Task 30 Phase 12b.2, `src/lib.rs` PG18 fixture sink split
- Scope: source layout only; no fixture assertions, SQL, planner, or CustomScan behavior changed

## Summary

This checkpoint moves the contiguous CustomScan pg_test fixture block out of
`src/tests/mod.rs` and into `src/tests/custom_scan.rs`.

`src/tests/mod.rs` now includes that file textually with:

```rust
include!("custom_scan.rs");
```

That keeps the moved functions in the same `#[pg_schema] mod tests` scope and
preserves the existing pgrx-discovered fixture names. The next remote-search
fixture remains in `src/tests/mod.rs`, so this slice only claims the
CustomScan concern file in the Phase 12b.2 module-tree checklist.

## Validation

- Format check:
  `review/30998-spire-customscan-fixture-split/artifacts/cargo-fmt-check.log`
- Focused PG18 moved fixture:
  `review/30998-spire-customscan-fixture-split/artifacts/cargo-test-customscan-fixture.log`
- Fixture location and line-count sanity:
  `review/30998-spire-customscan-fixture-split/artifacts/fixture-location-check.log`

See `artifacts/manifest.md` for exact commands and key result lines.

## Reviewer Focus

1. Confirm the textual include keeps the moved pg_tests in the same module scope.
2. Confirm the extraction range stops before `test_ec_spire_remote_search_local_heap_resolution_plan`.
3. Confirm marking only `tests/custom_scan.rs` complete in Phase 12b.2 is accurate; the other concern files remain open.
