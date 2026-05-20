# Task 50 Review Request: SPIRE CustomScan Executor Accessors

Code commit: `4ff977b2333be17474eceb822ad953f011de0a56`

## Scope

This packet advances the broad Task 50 unsafe burndown plan from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

Programs/tranches:

- P10 scan opaque and raw ownership contracts
- P11 planner, node, list, and custom scan views
- Wave 2 SPIRE CustomScan executor fanout

## Change

This slice removes caller-side unsafe from SPIRE CustomScan executor access paths:

- `ec_spire_begin_custom_scan` now uses the shared exec-state accessor and typed state references instead of wrapping the whole callback body in `unsafe`.
- `ec_spire_rescan_custom_scan` now resets state through a checked exec-state reference.
- `ec_spire_custom_scan_access` now dispatches through safe helper functions for tuple slots, processed-row accounting, heap tuple fetch, and DML access paths.
- DML and tuple-payload helpers now accept `&SpireCustomScanExecState` / `&mut SpireCustomScanExecState` where possible instead of raw state pointers.

The remaining unsafe in this area is still PostgreSQL executor/slot/SPI/DML boundary code and remains in the regenerated ledger for follow-up removal or residual registration.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1955 | 1953 | -2 |
| `src/am/ec_spire/custom_scan/begin_exec.rs` | 13 | 10 | -3 |
| `src/am/ec_spire/custom_scan/dml.rs` | 14 | 15 | +1 |
| `src/am/ec_spire/custom_scan/tuple_payload.rs` | 6 | 6 | 0 |
| `src/` unsafe ledger rows | 1955 | 1953 | -2 |

The `dml.rs` count increases by one because the production result-stream FFI boundary is now inside a safe helper instead of hidden behind an unsafe function signature required at the executor callback site.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - result: pass, with the known pre-existing unused SPIRE DML export warning in `src/am/mod.rs`
- `git diff --check`
  - log: `artifacts/git-diff-check.log`
  - result: pass
- `make unsafe-block-count`
  - log: `artifacts/src-unsafe-block-count-after.log`
  - result: current `src/` total is `1953` direct unsafe blocks across `131` files
- `make unsafe-ledger`
  - log: `artifacts/unsafe-ledger-generate.log`
  - result: generated `1953` current `src/` ledger rows
- `make unsafe-ledger-check`
  - log: `artifacts/unsafe-ledger-check.log`
  - result: `ledger covers 1953 current unsafe rows`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/count-summary.md`
- `artifacts/code-diff.patch`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/git-diff-check.log`
- `artifacts/src-unsafe-block-count-after.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`

## Residual Work

Task 50 is not complete. The highest remaining `src/` unsafe surfaces are still HNSW scan/debug/build, DiskANN routine/insert/ambuild, IVF scan/page/build, SPIRE DML/frontdoor/coordinator/page/storage, shared AM/storage guards, quant kernels, tests, hardening, crates, and vendor disposition. Every remaining row must still be removed or residual-registered before closeout.
