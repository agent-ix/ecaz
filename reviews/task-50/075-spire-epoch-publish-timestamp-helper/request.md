# Task 50 Packet 075: SPIRE Epoch Publish Timestamp Helper

## Summary

This packet removes caller-side timestamp unsafe blocks from SPIRE publish paths. `build::current_epoch_publish_times` now owns the PostgreSQL `GetCurrentTimestamp` unsafe boundary internally and exposes a safe result-returning helper to build, maintenance, insert, and vacuum callers.

The code checkpoint is:

- `7a406cdbb827be4d5a7e0c022810c34422dd4030` - `Make SPIRE epoch publish timestamps safe to call`

## Unsafe Burndown

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/build/drafts.rs` | 19 | 17 | -2 |
| `src/am/ec_spire/coordinator/maintenance.rs` | 20 | 19 | -1 |
| `src/am/ec_spire/insert.rs` | 20 | 18 | -2 |
| `src/am/ec_spire/vacuum/mod.rs` | 16 | 15 | -1 |
| `src/` total | 2059 | 2053 | -6 |

The removed direct unsafe blocks were all call-site wrappers around `current_epoch_publish_times`. The helper itself still contains the PostgreSQL timestamp boundary and keeps the checked retention arithmetic unchanged.

## Validation

- `git diff --check HEAD^ HEAD`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the known existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `make unsafe-block-count`: touched files now report `17`, `19`, `18`, and `15` direct unsafe blocks.
- `make unsafe-ledger`: generated `2053` current `src/` rows.
- `make unsafe-ledger-check`: ledger covers all `2053` current `src/` unsafe rows.

Artifacts are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth.

## Residual Unsafe

SPIRE publish paths still contain direct unsafe around relation-backed object stores, page/WAL primitives, PostgreSQL callback boundaries, and relation pointer views. This packet does not declare those complete; they remain in the Task 50 ledger for further removal or residual registration.
