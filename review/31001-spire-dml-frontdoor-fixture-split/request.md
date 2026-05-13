# Review Request: SPIRE DML Frontdoor Fixture Split

- Code commit: `438c0fbf` (`Move SPIRE DML frontdoor fixtures out of test sink`)
- Task: Task 30 Phase 12b.2, `src/lib.rs` PG18 fixture sink split
- Scope: source layout only; no fixture assertions, SQL, DML hook, or CustomScan behavior changed

## Summary

This checkpoint starts `src/tests/dml_frontdoor.rs` by moving the main
DML hook, plan, and remote-CustomScan fixture block out of `src/tests/mod.rs`.

`src/tests/mod.rs` includes the file textually:

```rust
include!("dml_frontdoor.rs");
```

That keeps the moved pg_tests in the same `#[pg_schema] mod tests` scope and
preserves existing pgrx-discovered fixture names. This does not close the full
`tests/dml_frontdoor.rs` Phase 12b.2 row yet because earlier select-plan and
later primitive-plan fixtures still live in `src/tests/mod.rs`.

## Validation

- Format check:
  `review/31001-spire-dml-frontdoor-fixture-split/artifacts/cargo-fmt-check.log`
- Focused PG18 moved fixture:
  `review/31001-spire-dml-frontdoor-fixture-split/artifacts/cargo-test-dml-frontdoor-fixture.log`
- Fixture location and line-count sanity:
  `review/31001-spire-dml-frontdoor-fixture-split/artifacts/fixture-location-check.log`

See `artifacts/manifest.md` for exact commands and key result lines.

## Reviewer Focus

1. Confirm the extraction range starts at `test_ec_spire_dml_frontdoor_hook_status_installed_pass_through`.
2. Confirm the range ends before `test_ec_spire_reaper_resolves_lost_prepare_ack_fixture`.
3. Confirm the tracker correctly records this as a partial DML-frontdoor split, not a closed `tests/dml_frontdoor.rs` row.
