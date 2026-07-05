# Task 50 Packet 082: IVF Tuple-Chain Reader Boundaries

## Summary

This packet continues the IVF/RaBitQ unsafe burndown by making tuple-chain reader helpers safe to call. The centroid, list-directory, and PQ-codebook readers delegate to the shared page tuple reader that owns the actual buffer/page unsafe, so callers no longer need their own unsafe wrappers for chain traversal.

The code checkpoint is:

- `9b460c2bb78b1b8d447dd699ee6b2850ae88de96` - `Make IVF tuple chain readers safe to call`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/admin.rs` | 7 | 6 | -1 |
| `src/am/ec_ivf/insert.rs` | 10 | 7 | -3 |
| `src/am/ec_ivf/page.rs` | 35 | 35 | 0 |
| `src/am/ec_ivf/quantizer.rs` | 1 | 0 | -1 |
| `src/am/ec_ivf/scan.rs` | 40 | 37 | -3 |
| `src/am/ec_ivf/vacuum.rs` | 15 | 14 | -1 |
| `src/` total | 2013 | 2004 | -9 |

The removed direct unsafe blocks were tuple-chain reader wrappers in directory summary, insert duplicate checks and centroid/directory loads, PQ-fastscan codebook loading, scan centroid/directory loads, and vacuum directory traversal.

The real unsafe remains in the shared page tuple reader and lower page primitives.

## Validation

- `git diff --check HEAD^ HEAD`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the known existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `make unsafe-block-count`: touched IVF files dropped by `9` direct unsafe blocks.
- `make unsafe-ledger`: generated `2004` current `src/` rows.
- `make unsafe-ledger-check`: ledger covers all `2004` current `src/` unsafe rows.

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth.

## Residual Unsafe

IVF still has direct unsafe in scan/page/build/vacuum/insert/admin/cost around scan descriptor access, page and WAL primitives, relation lock internals, tuple/datum reads, page tuple views, and debug wrappers.

Those are not complete. They remain in the Task 50 ledger for subsequent removal or residual registration.
