# Task 50 Review Request: SPIRE CustomScan Plan-Private Helpers

## Summary

This packet reviews commit `9865b278258228e90ea8f08701443c6e7cc331ae`, which consolidates SPIRE CustomScan plan-private construction and read helpers and removes 9 direct unsafe blocks.

The slice advances P10 and P11 in the comprehensive burndown plan by making plan-private metadata access go through checked helper boundaries instead of repeating raw `CustomScan` and list access at each caller.

## Changed Files

- `src/am/ec_spire/custom_scan/plan_private.rs`

## Count Delta

See `artifacts/count-summary.md`.

The touched SPIRE file moves from 19 to 10 direct unsafe blocks. The `src/` total moves from 1979 after packet 084 to 1970 after this packet.

## Validation

- `git diff --check HEAD^ HEAD`
  - log: `artifacts/git-diff-check.log`
  - result: exit code 0
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - result: exit code 0 with the existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`
  - log: `artifacts/src-unsafe-block-count-after.log`
  - result: `1970` current direct unsafe blocks under `src/`
- `make unsafe-ledger`
  - log: `artifacts/unsafe-ledger-generate.log`
  - result: generated `1970` ledger rows
- `make unsafe-ledger-check`
  - log: `artifacts/unsafe-ledger-check.log`
  - result: `ledger covers 1970 current unsafe rows`

## Residual Unsafe

Residual SPIRE CustomScan unsafe remains in low-level PostgreSQL list/node decoding, string-node C string decoding, datum decoding, path-private reads, and a test-only deep-copy roundtrip. Those rows remain open in the Task 50 ledger for later deletion or residual registration.
