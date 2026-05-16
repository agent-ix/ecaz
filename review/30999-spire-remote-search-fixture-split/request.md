# Review Request: SPIRE Remote Search Fixture Split

- Code commit: `a88c92d3` (`Move SPIRE remote search fixtures out of test sink`)
- Task: Task 30 Phase 12b.2, `src/lib.rs` PG18 fixture sink split
- Scope: source layout only; no fixture assertions, SQL, remote-search behavior, or planner behavior changed

## Summary

This checkpoint starts `src/tests/remote_search.rs` by moving the first
contiguous remote-search contract fixture block out of `src/tests/mod.rs`.

`src/tests/mod.rs` includes the file textually:

```rust
include!("remote_search.rs");
```

That keeps the moved pg_tests in the same `#[pg_schema] mod tests` scope and
preserves existing pgrx-discovered fixture names. This does not close the full
`tests/remote_search.rs` Phase 12b.2 row yet: later tuple-payload, libpq,
remote-node, and degraded-mode fixtures still live in `src/tests/mod.rs` and
need separate follow-up moves.

## Validation

- Format check:
  `review/30999-spire-remote-search-fixture-split/artifacts/cargo-fmt-check.log`
- Focused PG18 moved fixture:
  `review/30999-spire-remote-search-fixture-split/artifacts/cargo-test-remote-search-fixture.log`
- Fixture location and line-count sanity:
  `review/30999-spire-remote-search-fixture-split/artifacts/fixture-location-check.log`

See `artifacts/manifest.md` for exact commands and key result lines.

## Reviewer Focus

1. Confirm the extraction range starts at `test_ec_spire_remote_search_sql_scores_selected_leaf_pids`.
2. Confirm the range ends before `test_ec_spire_custom_scan_status_registered_fail_closed`.
3. Confirm the tracker correctly records this as a partial remote-search split, not a closed `tests/remote_search.rs` row.
