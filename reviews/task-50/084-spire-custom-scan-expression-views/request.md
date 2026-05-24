# Task 50 Review Request: SPIRE CustomScan Expression Views

## Summary

This packet reviews commit `b470166c1ccd74ce96525c652d580a8fbfdd62e0`, which centralizes SPIRE CustomScan planner expression reads behind local typed helpers and removes 17 direct unsafe blocks.

The slice advances the comprehensive burndown plan programs P2 and P11 by replacing repeated raw planner pointer/list/node reads with named helper boundaries in the SPIRE CustomScan planner path.

## Changed Files

- `src/am/ec_spire/custom_scan/cost_helpers.rs`
- `src/am/ec_spire/custom_scan/plan_private.rs`

## Count Delta

See `artifacts/count-summary.md`.

The touched SPIRE files remove 17 direct unsafe blocks. The `src/` total moves from 1996 after packet 083 to 1979 after this packet.

## Validation

- `git diff --check HEAD^ HEAD`
  - log: `artifacts/git-diff-check.log`
  - result: exit code 0
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - result: exit code 0 with the existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`
  - log: `artifacts/src-unsafe-block-count-after.log`
  - result: `1979` current direct unsafe blocks under `src/`
- `make unsafe-ledger`
  - log: `artifacts/unsafe-ledger-generate.log`
  - result: generated `1979` ledger rows
- `make unsafe-ledger-check`
  - log: `artifacts/unsafe-ledger-check.log`
  - result: `ledger covers 1979 current unsafe rows`

## Residual Unsafe

Residual SPIRE CustomScan unsafe remains in plan-private/list/executor-state boundaries, planner callback allocation, DML executor wiring, and datum decoding. Those rows remain open in the Task 50 ledger for later deletion or residual registration.
