# Task 50 Review Request: SPIRE DML Integer Datum Views

## Summary

This packet continues the SPIRE DML frontdoor unsafe burndown.

The code change consolidates two recurring helper boundaries:

- by-value integer Datum reads now flow through `dml_frontdoor_integer_datum_value`, shared by constant predicate decoding and bound parameter decoding;
- one-element coerced expression argument reads now use the existing DML `PgList` view helper instead of manually reading PG18 `ListCell` storage.

This keeps caller logic in safe Rust while preserving the same fail-closed type checks for int2/int4/int8 and one-argument wrapper expressions.

## Counts

- `src/am/ec_spire/dml_frontdoor/mod.rs`: 34 -> 32 direct unsafe blocks
- `src/` total: 2078 -> 2076 direct unsafe blocks

See `artifacts/count-summary.md`.

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`
- unsafe ledger generated and checked:
  - `unsafe-ledger-after.jsonl` contains 2076 current `src/` rows
  - `unsafe-ledger-check.log`: `ledger covers 2076 current unsafe rows`

## Residual / Follow-Up

This is not DML frontdoor closeout. The file still has 32 direct unsafe blocks, including catalog tuple descriptor reads, relation/index relcache metadata, bound parameter list fetch internals, top-level hook/cache surfaces, and centralized planner-view helper internals.
