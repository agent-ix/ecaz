# Task 50 Review Request: SPIRE CustomScan Plan Accessors

## Summary

This packet reviews commit `cc3d94cc30cc9bd8fad66f364a0a3845b3d66278`, which makes checked SPIRE CustomScan plan accessors safe to call and removes 2 direct unsafe blocks from the explain callback.

The slice advances P10 and P11 by moving CustomScan plan extraction and plan-private index-OID lookup callers onto named helper contracts.

## Changed Files

- `src/am/ec_spire/custom_scan/explain.rs`
- `src/am/ec_spire/custom_scan/plan_private.rs`

## Count Delta

See `artifacts/count-summary.md`.

The touched SPIRE explain file moves from 5 to 3 direct unsafe blocks. The `src/` total moves from 1963 after packet 086 to 1961 after this packet.

## Validation

- `git diff --check HEAD^ HEAD`
  - log: `artifacts/git-diff-check.log`
  - result: exit code 0
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - result: exit code 0 with the existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`
  - log: `artifacts/src-unsafe-block-count-after.log`
  - result: `1961` current direct unsafe blocks under `src/`
- `make unsafe-ledger`
  - log: `artifacts/unsafe-ledger-generate.log`
  - result: generated `1961` ledger rows
- `make unsafe-ledger-check`
  - log: `artifacts/unsafe-ledger-check.log`
  - result: `ledger covers 1961 current unsafe rows`

## Residual Unsafe

Residual SPIRE CustomScan unsafe remains in explain property emission, relation option reads, plan-private list decoding, datum decoding, and executor/planner PostgreSQL callback boundaries. Those rows remain open in the Task 50 ledger for later deletion or residual registration.
