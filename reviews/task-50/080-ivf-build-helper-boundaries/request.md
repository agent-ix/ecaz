# Task 50 Packet 080: IVF Build Helper Boundaries

## Summary

This packet continues the IVF/RaBitQ unsafe burndown in build and empty-bootstrap insert paths. It makes helper functions safe to call where they already contain the real PostgreSQL page, WAL, datum, or type-resolution unsafe.

The code checkpoint is:

- `38822699757cde7a571fc483bb98d380cdfaefad` - `Consolidate IVF build helper call boundaries`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/build.rs` | 21 | 18 | -3 |
| `src/am/ec_ivf/insert.rs` | 14 | 13 | -1 |
| `src/` total | 2030 | 2026 | -4 |

The removed direct unsafe blocks were caller wrappers around build-plan data-page flush, detoast conversion, indexed-vector type resolution, and empty-bootstrap flush dispatch.

The real unsafe remains in helper bodies for buffer allocation, WAL registration, page initialization, tuple insertion, detoasting, and PostgreSQL type-name lookup/free.

## Validation

- `git diff --check HEAD^ HEAD`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the known existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `make unsafe-block-count`: `src/am/ec_ivf/build.rs` now reports `18`; `src/am/ec_ivf/insert.rs` reports `13`.
- `make unsafe-ledger`: generated `2026` current `src/` rows.
- `make unsafe-ledger-check`: ledger covers all `2026` current `src/` unsafe rows.

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth.

## Residual Unsafe

IVF build still has direct unsafe around AM callback guards, page/WAL primitives, datum array reads, tuple descriptor access, item pointer decoding, and PostgreSQL type formatting/freeing. IVF insert still has direct unsafe around relation lock internals, metadata/page operations, PQ model loading, and directory/posting scans.

Those are not complete. They remain in the Task 50 ledger for subsequent removal or residual registration.
