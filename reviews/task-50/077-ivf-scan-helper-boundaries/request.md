# Task 50 Packet 077: IVF Scan Helper Boundaries

## Summary

This packet continues the IVF/RaBitQ unsafe burndown in scan orchestration. It removes caller-side unsafe blocks by making module-private helpers safe where they already own the relevant scan descriptor, heap relation, snapshot, selected-probe, and directory-chain contracts.

The code checkpoint is:

- `623d3411843c5d1931b51594bfe9180bc006d9a7` - `Consolidate IVF scan helper call boundaries`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/scan.rs` | 46 | 41 | -5 |
| `src/` total | 2048 | 2043 | -5 |

The removed direct unsafe blocks were:

- heap-rerank heap relation resolver call;
- heap-rerank snapshot resolver call;
- selected-probe plan builder call;
- directory loader call inside selected-probe planning;
- debug directory-entry loader call.

The real PostgreSQL unsafe remains inside the helpers that dereference the scan descriptor, resolve active snapshots, or read IVF directory pages.

## Validation

- `git diff --check HEAD^ HEAD`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the known existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `make unsafe-block-count`: `src/am/ec_ivf/scan.rs` now reports `41` direct unsafe blocks.
- `make unsafe-ledger`: generated `2043` current `src/` rows.
- `make unsafe-ledger-check`: ledger covers all `2043` current `src/` unsafe rows.

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth.

## Residual Unsafe

`src/am/ec_ivf/scan.rs` still has `41` direct unsafe blocks. The remaining surfaces include scan-owned pointer arrays, scan descriptor field access, PQ/RaBitQ model loading, metadata/directory/page visitors, heap slot reader construction, debug callback wrappers, and datum/order-by extraction.

Those are not complete. They remain in the Task 50 ledger for subsequent removal or residual registration.
