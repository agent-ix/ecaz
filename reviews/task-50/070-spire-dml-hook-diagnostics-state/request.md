# Task 50 Review Request: SPIRE DML Hook Diagnostics State

## Summary

This packet continues the SPIRE DML frontdoor unsafe burndown by removing reducible `static mut` usage from backend-local hook diagnostics.

The code change introduces a safe `Mutex<DmlFrontdoorBackendHookState>` for:

- planner-hook installed status exposed by diagnostics;
- relcache callback registration status exposed by diagnostics;
- last hook classification result;
- last hook action.

The actual PostgreSQL hook install and relcache callback registration remain explicit unsafe FFI boundaries. This packet only removes unnecessary direct unsafe around diagnostic state reads and writes.

## Counts

- `src/am/ec_spire/dml_frontdoor/mod.rs`: 32 -> 30 direct unsafe blocks
- `src/` total: 2076 -> 2074 direct unsafe blocks

See `artifacts/count-summary.md`.

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`
- unsafe ledger generated and checked:
  - `unsafe-ledger-after.jsonl` contains 2074 current `src/` rows
  - `unsafe-ledger-check.log`: `ledger covers 2074 current unsafe rows`

## Residual / Follow-Up

This is not DML frontdoor closeout. The file still has 30 direct unsafe blocks, including PostgreSQL hook installation, relcache callback FFI, catalog tuple descriptor reads, relation/index relcache metadata, bound parameter list fetch internals, and centralized planner-view helper internals.
