# Task 50 Packet 074: SPIRE Vacuum Callback Relation Construction

## Summary

This packet continues the SPIRE production vacuum unsafe burndown. It removes caller-side unsafe blocks from private vacuum relation view construction and from the bulk-delete heap-dead callback call site.

The code checkpoint is:

- `70de562aab4c1452d3ff25869bbbd4b7ca70a013` - `Consolidate SPIRE vacuum callback relation construction`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/vacuum/mod.rs` | 20 | 16 | -4 |
| `src/` total | 2063 | 2059 | -4 |

The removed direct unsafe blocks were:

- three private `SpireVacuumIndexRelation::new` call sites in cleanup, bulkdelete, and live-count orchestration;
- one bulk-delete loop call site for `heap_tid_is_dead`.

The remaining unsafe stays at the actual PostgreSQL boundary methods and callback invocation helper rather than at every orchestration caller.

## Validation

- `git diff --check HEAD^ HEAD`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the known existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `make unsafe-block-count`: `src/am/ec_spire/vacuum/mod.rs` now reports `16` direct unsafe blocks.
- `make unsafe-ledger`: generated `2059` current `src/` rows.
- `make unsafe-ledger-check`: ledger covers all `2059` current `src/` unsafe rows.

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth.

## Residual Unsafe

`src/am/ec_spire/vacuum/mod.rs` still has `16` direct unsafe blocks. The remaining surfaces include relation/page wrapper internals, publish timestamp access, vacuum stats allocation/mutation, the heap-dead callback invocation helper, and test/debug callback adapters.

Those are not complete. They remain in the Task 50 ledger for subsequent removal or residual registration.
