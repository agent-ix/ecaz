# Task 50 Review Request: SPIRE Vacuum Stats Boundary

## Summary

This packet continues the SPIRE vacuum unsafe burndown after packet 071.

The code change consolidates the PostgreSQL vacuum stats finish path so allocation of a missing stats object, relation block-count lookup, and stats-field mutation occur inside one named boundary in `finish_vacuum_stats`.

## Counts

- `src/am/ec_spire/vacuum/mod.rs`: 26 -> 24 direct unsafe blocks
- `src/` total: 2069 -> 2067 direct unsafe blocks

See `artifacts/count-summary.md`.

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`
- unsafe ledger generated and checked:
  - `unsafe-ledger-after.jsonl` contains 2067 current `src/` rows
  - `unsafe-ledger-check.log`: `ledger covers 2067 current unsafe rows`

## Residual / Follow-Up

This is not SPIRE vacuum closeout. The file still has 24 direct unsafe blocks, including PostgreSQL callback entry, publish-lock acquisition, heap-dead callback invocation, debug test callbacks, and centralized vacuum relation view internals.
