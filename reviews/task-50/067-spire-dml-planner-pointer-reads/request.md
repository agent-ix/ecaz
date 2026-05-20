# Task 50 Review Request: SPIRE DML Planner Pointer Reads

## Summary

This packet continues the P11 SPIRE DML frontdoor planner/node/list burndown after packet 066.

The code change routes more PostgreSQL planner pointer reads through private DML frontdoor helpers:

- `dml_frontdoor_pg_ref` for immediate-use planner and relcache pointer references
- `dml_frontdoor_c_string` for copied PostgreSQL C-string names

That removes caller-owned unsafe from rtable lookup, baserestrictinfo walking, target-entry walking, catalog attribute name copying, and target-entry fallback name copying.

## Counts

- `src/am/ec_spire/dml_frontdoor/mod.rs`: 39 -> 37 direct unsafe blocks
- `src/` total: 2083 -> 2081 direct unsafe blocks

See `artifacts/count-summary.md`.

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`
- unsafe ledger generated and checked:
  - `unsafe-ledger-after.jsonl` contains 2081 current `src/` rows
  - `unsafe-ledger-check.log`: `ledger covers 2081 current unsafe rows`

## Residual / Follow-Up

This is not DML frontdoor closeout. The file still has 37 direct unsafe blocks, including catalog tuple descriptor reads, relation/index relcache metadata, parameter decoding, top-level hook/cache surfaces, and the centralized planner view helpers introduced by packets 066 and 067.
