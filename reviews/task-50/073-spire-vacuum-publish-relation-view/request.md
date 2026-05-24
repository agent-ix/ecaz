# Task 50 Packet 073: SPIRE Vacuum Publish Relation View

## Summary

This packet continues the broad Task 50 unsafe burndown against SPIRE production vacuum code. It routes the publish lock and replacement-epoch publish helpers through the private `SpireVacuumIndexRelation` view instead of passing raw `pg_sys::Relation` pointers through those call paths.

The code checkpoint is:

- `9edc102f05bd3bdc3e50ee347a89327602b8a999` - `Route SPIRE vacuum publish through relation view`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/vacuum/mod.rs` | 24 | 20 | -4 |
| `src/` total | 2067 | 2063 | -4 |

The removed direct unsafe blocks were caller-side relation pointer accesses for:

- acquiring the SPIRE relation publish lock during vacuum cleanup;
- acquiring the SPIRE relation publish lock during bulkdelete;
- publishing compacted replacement epochs;
- publishing delete-delta replacement epochs.

Those accesses now live behind `SpireVacuumIndexRelation` methods so the callback bodies keep moving toward safe Rust orchestration with smaller named boundary contracts.

## Validation

- `git diff --check HEAD^ HEAD`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the known existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `make unsafe-block-count`: `src/am/ec_spire/vacuum/mod.rs` now reports `20` direct unsafe blocks.
- `make unsafe-ledger`: generated `2063` current `src/` rows.
- `make unsafe-ledger-check`: ledger covers all `2063` current `src/` unsafe rows.

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth.

## Residual Unsafe

`src/am/ec_spire/vacuum/mod.rs` still has `20` direct unsafe blocks. The remaining surfaces include callback entry, construction of typed relation/vacuum views from PostgreSQL pointers, heap-dead callback invocation, stats/debug callback helpers, and owned callback result boundaries.

Those are not being declared complete. They remain in the Task 50 ledger for subsequent removal or residual registration.
