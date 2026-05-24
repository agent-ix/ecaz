# Task 50 Packet 081: IVF Metadata Page Helper Boundaries

## Summary

This packet continues the IVF/RaBitQ unsafe burndown by making the metadata page helper API safe to call. The helpers already route through `IvfPageRelation`, `WalRegisteredPage`, and page-special storage methods; this removes redundant caller-side unsafe from the modules that only need to read, initialize, or update IVF metadata.

The code checkpoint is:

- `987be0a8732f881f70122226896229bb9a80aba4` - `Make IVF metadata page helpers safe to call`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/admin.rs` | 10 | 7 | -3 |
| `src/am/ec_ivf/build.rs` | 18 | 17 | -1 |
| `src/am/ec_ivf/cost.rs` | 8 | 6 | -2 |
| `src/am/ec_ivf/insert.rs` | 13 | 10 | -3 |
| `src/am/ec_ivf/page.rs` | 35 | 35 | 0 |
| `src/am/ec_ivf/scan.rs` | 41 | 40 | -1 |
| `src/am/ec_ivf/vacuum.rs` | 18 | 15 | -3 |
| `src/` total | 2026 | 2013 | -13 |

The removed direct unsafe blocks were metadata helper wrappers around `read_metadata_page`, `initialize_metadata_page`, and `update_metadata_page`. The real unsafe remains in page helper internals for relation access, buffer/WAL work, and page-special byte access.

## Validation

- `git diff --check HEAD^ HEAD`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the known existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `make unsafe-block-count`: touched IVF files dropped by `13` direct unsafe blocks.
- `make unsafe-ledger`: generated `2013` current `src/` rows.
- `make unsafe-ledger-check`: ledger covers all `2013` current `src/` unsafe rows.

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth.

## Residual Unsafe

IVF still has direct unsafe in scan/page/build/vacuum/insert/admin/cost around scan descriptor access, page and WAL primitives, relation lock internals, tuple/datum reads, page tuple views, and debug wrappers.

Those are not complete. They remain in the Task 50 ledger for subsequent removal or residual registration.
