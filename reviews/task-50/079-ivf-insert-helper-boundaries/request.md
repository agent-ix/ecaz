# Task 50 Packet 079: IVF Insert Helper Boundaries

## Summary

This packet continues the IVF/RaBitQ unsafe burndown in insert paths. It removes redundant caller-side unsafe from module-private helpers whose bodies already contain the actual PostgreSQL/page unsafe and enforce the relevant metadata and relation contracts.

The code checkpoint is:

- `0a046d51e7c107a253c5c9f1d0e083c8d807225a` - `Consolidate IVF insert helper call boundaries`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/insert.rs` | 20 | 14 | -6 |
| `src/` total | 2036 | 2030 | -6 |

The removed direct unsafe blocks were caller wrappers around empty-bootstrap lock creation, bootstrap flush, trained insert dispatch, centroid model loading, directory lookup, and debug duplicate-heap-TID validation.

The real unsafe remains inside the helper bodies for relation locking, metadata reads, page writes, PQ model loading, and directory/posting scans.

## Validation

- `git diff --check HEAD^ HEAD`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the known existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `make unsafe-block-count`: `src/am/ec_ivf/insert.rs` now reports `14` direct unsafe blocks.
- `make unsafe-ledger`: generated `2030` current `src/` rows.
- `make unsafe-ledger-check`: ledger covers all `2030` current `src/` unsafe rows.

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth.

## Residual Unsafe

`src/am/ec_ivf/insert.rs` still has `14` direct unsafe blocks. The remaining surfaces include relation lock internals, metadata reads/updates, PQ model loading, page append/update calls, build-plan flush, directory/posting scan internals, and debug metadata reads.

Those are not complete. They remain in the Task 50 ledger for subsequent removal or residual registration.
