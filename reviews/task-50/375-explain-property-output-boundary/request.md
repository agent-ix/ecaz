# Task 50 Review Request: EXPLAIN Property Output Boundary

## Summary

This slice advances Task 50 AM common EXPLAIN boundary cleanup.

Code commit: `76dac3e71ef5bf140f8862101c6fdd38a8f87ee0`

Changed `src/am/common/explain.rs` so `emit_explain_properties` opens the
EXPLAIN output group, emits all properties, and closes the group inside one
explicit PostgreSQL output boundary.

## Unsafe Counts

- `src/am/common/explain.rs`: `9 -> 7`
- `src/` total direct unsafe blocks: `1178 -> 1176`

See `artifacts/unsafe-counts.log`.

## Plan Coverage

- Program: P1, EXPLAIN callback boundary cleanup.
- Wave/tranche: Wave 4, AM common residual minimization.
- Disposition: nested PostgreSQL EXPLAIN output calls were consolidated into
  the existing property-emission boundary.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed; it reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.
- `git diff --check` passed.
- `rustfmt --check src/am/common/explain.rs` passed.
- Raw-boundary guard produced no matches.
- Generated unsafe ledger covers all `1176` current `src` unsafe rows.

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.
