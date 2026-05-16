# Review Request: SPIRE DML Frontdoor Primitive Fixture Split

- Code commit: `04bf4917` (`Move SPIRE DML primitive fixtures into concern file`)
- Task: Task 30 Phase 12b.2, `src/lib.rs` PG18 fixture sink split
- Scope: source layout only; no fixture assertions, SQL, DML primitive planning, or hook behavior changed

## Summary

This checkpoint extends `src/tests/dml_frontdoor.rs` by moving the later
DML-frontdoor primitive helper fixture block out of `src/tests/mod.rs`.

The file remains included textually from `src/tests/mod.rs`:

```rust
include!("dml_frontdoor.rs");
```

That keeps the moved pg_tests in the same `#[pg_schema] mod tests` scope and
preserves existing pgrx-discovered fixture names. This still does not close the
full `tests/dml_frontdoor.rs` Phase 12b.2 row because the earlier select-plan
fixture and the replacement-decision SQL fixture remain in `src/tests/mod.rs`.

## Validation

- Format check:
  `review/31004-spire-dml-frontdoor-primitive-fixture-split/artifacts/cargo-fmt-check.log`
- Focused PG18 moved fixture:
  `review/31004-spire-dml-frontdoor-primitive-fixture-split/artifacts/cargo-test-dml-frontdoor-primitive-fixture.log`
- Fixture location and line-count sanity:
  `review/31004-spire-dml-frontdoor-primitive-fixture-split/artifacts/fixture-location-check.log`

See `artifacts/manifest.md` for exact commands and key result lines.

## Reviewer Focus

1. Confirm the appended range starts at `test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send`.
2. Confirm the range ends before `test_ec_spire_dml_frontdoor_replacement_decision_sql`.
3. Confirm the tracker correctly records this as still partial, not a closed `tests/dml_frontdoor.rs` row.
