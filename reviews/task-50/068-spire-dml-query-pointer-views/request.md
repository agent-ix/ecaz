# Task 50 Review Request: SPIRE DML Query Pointer Views

## Summary

This packet continues the P11 SPIRE DML frontdoor planner/node/list burndown after packets 066 and 067.

The code change introduces `dml_frontdoor_query_ref` and uses it for the repeated PostgreSQL `Query` pointer reads in:

- relation-backed query detail extraction
- baserel query detail extraction
- pure query classification
- target relation OID extraction

Those callers now share one private immediate-use query view helper instead of owning separate direct unsafe blocks.

## Counts

- `src/am/ec_spire/dml_frontdoor/mod.rs`: 37 -> 34 direct unsafe blocks
- `src/` total: 2081 -> 2078 direct unsafe blocks

See `artifacts/count-summary.md`.

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`
- unsafe ledger generated and checked:
  - `unsafe-ledger-after.jsonl` contains 2078 current `src/` rows
  - `unsafe-ledger-check.log`: `ledger covers 2078 current unsafe rows`

## Residual / Follow-Up

This is not DML frontdoor closeout. The file still has 34 direct unsafe blocks, including catalog tuple descriptor reads, relation/index relcache metadata, parameter decoding, top-level hook/cache surfaces, and centralized planner-view helper internals.
