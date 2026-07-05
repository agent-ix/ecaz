# Task 50 Review Request: EXPLAIN AM Name Boundary

## Summary

This slice advances Task 50 AM common EXPLAIN boundary cleanup.

Code commit: `a58e8d3a2ddf09999086c918504a5b84682c09f5`

Changed `src/am/common/explain.rs` so `explain_access_method_name` performs
the PostgreSQL `get_am_name` call, C-string copy, and `pfree` cleanup inside a
single explicit boundary block.

## Unsafe Counts

- `src/am/common/explain.rs`: `11 -> 9`
- `src/` total direct unsafe blocks: `1180 -> 1178`

See `artifacts/unsafe-counts.log`.

## Plan Coverage

- Program: P1/P7, EXPLAIN callback boundary and PostgreSQL C-string cleanup.
- Wave/tranche: Wave 4, AM common residual minimization.
- Disposition: adjacent AM-name FFI/C-string/free operations were consolidated
  behind the existing EXPLAIN helper boundary.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed; it reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.
- `git diff --check` passed.
- `rustfmt --check src/am/common/explain.rs` passed.
- Raw-boundary guard produced no matches.
- Generated unsafe ledger covers all `1178` current `src` unsafe rows.

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.
