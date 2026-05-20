# Task 50 Review Request: SPIRE DML Planner Node Views

## Summary

This packet advances the comprehensive unsafe burndown plan P11 target for SPIRE DML frontdoor planner/node/list views.

The code change consolidates repeated caller-side unsafe around planner `Query` jointree access, PostgreSQL `List` views, NodeTag-dispatched expression reads, range-table refs, and operator equality lookup into private DML frontdoor helpers:

- `dml_frontdoor_query_jointree`
- `dml_frontdoor_pg_list`
- `dml_frontdoor_expr_node`
- `dml_frontdoor_range_table_ref_node`
- `dml_frontdoor_bigint_equality_operator`

Callers now consume typed, immediate-use views instead of each owning the same direct unsafe block.

## Counts

- `src/am/ec_spire/dml_frontdoor/mod.rs`: 47 -> 39 direct unsafe blocks
- `src/` total: 2091 -> 2083 direct unsafe blocks

See `artifacts/count-summary.md`.

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`
- unsafe ledger generated and checked:
  - `unsafe-ledger-after.jsonl` contains 2083 current `src/` rows
  - `unsafe-ledger-check.log`: `ledger covers 2083 current unsafe rows`

## Residual / Follow-Up

This does not claim DML frontdoor closeout. The file still has 39 direct unsafe blocks, including catalog relation metadata, parameter decoding, remaining query/rtable reads, target entry string handling, and top-level hook/cache surfaces. Those remain in scope for later Task 50 slices.
