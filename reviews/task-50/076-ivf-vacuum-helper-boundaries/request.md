# Task 50 Packet 076: IVF Vacuum Helper Boundaries

## Summary

This packet continues the Task 50 unsafe burndown in the IVF/RaBitQ priority lane. It removes caller-side unsafe blocks from IVF vacuum orchestration by making module-private helpers safe where they already own the live vacuum relation, stats pointer, and PostgreSQL callback contracts.

The code checkpoint is:

- `6c48e2842816265e6ba50b035283255e71e4ae18` - `Consolidate IVF vacuum helper call boundaries`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/vacuum.rs` | 23 | 18 | -5 |
| `src/` total | 2053 | 2048 | -5 |

The removed direct unsafe blocks were:

- stats finalization call wrappers from noop and bulkdelete paths;
- the posting-list bulkdelete helper call wrapper;
- the heap-dead retain callback call wrapper.

The real PostgreSQL unsafe remains in the helper bodies that allocate stats, count relation blocks, invoke the callback, and rewrite IVF pages.

## Validation

- `git diff --check HEAD^ HEAD`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the known existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `make unsafe-block-count`: `src/am/ec_ivf/vacuum.rs` now reports `18` direct unsafe blocks.
- `make unsafe-ledger`: generated `2048` current `src/` rows.
- `make unsafe-ledger-check`: ledger covers all `2048` current `src/` unsafe rows.

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth.

## Residual Unsafe

`src/am/ec_ivf/vacuum.rs` still has `18` direct unsafe blocks. The remaining surfaces include metadata page access, directory page reads/rewrites, stats allocation/mutation, relation block counting, the PostgreSQL heap-dead callback invocation helper, and pg_test debug callback wrappers.

Those are not complete. They remain in the Task 50 ledger for subsequent removal or residual registration.
