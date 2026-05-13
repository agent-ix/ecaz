# Review Request: SPIRE Lib Fixture Body Split

Branch: `task-30-spire`
Task row: Phase 12b.2 fixture sink split
Checkpoint scope: move `#[pg_schema] mod tests` body out of `src/lib.rs`

## Summary

This checkpoint moves the large PG18 fixture module body from `src/lib.rs` to
`src/tests/mod.rs`. `src/lib.rs` keeps the `#[pg_schema] mod tests` wrapper and
includes the moved file so pgrx still sees the same schema/test module.

Two relative `include_str!` paths were adjusted for the new file location:

- `../../sql/bootstrap.sql`
- `../../ecaz--0.1.0--0.1.1.sql`

## Tracker State

This is not the full Phase 12b.2 finish. It closes the specific row that
`test_ec_spire_*` fixture bodies no longer live directly in `src/lib.rs`.

Still open:

- split `src/tests/mod.rs` into concern-specific files;
- reduce `src/lib.rs` below 2,000 lines;
- run the broader PG18 fixture validation and tracker spot-checks.

## Validation

Artifacts are in `review/30995-spire-lib-fixture-body-split/artifacts/`.

- `cargo check --no-default-features --features pg18`: pass.
- `cargo fmt --check`: pass, with existing stable-rustfmt config warnings.
- `git diff --check -- ...`: pass.
- `cargo test --no-default-features --features pg18 test_ec_spire_custom_scan_status_registered_fail_closed`:
  pass, 1 passed / 0 failed / 1711 filtered out.

## Review Focus

- Confirm pgrx fixture discovery remains intact through the include.
- Confirm the tracker is honest that the concern-specific split remains open.
- Confirm the adjusted `include_str!` paths are the only behavior-relevant
  textual changes.

