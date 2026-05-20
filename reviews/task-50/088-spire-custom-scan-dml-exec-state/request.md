# Task 50 Review Request: SPIRE CustomScan DML Exec-State Access

## Summary

This packet reviews commit `23b6caae13be4366f54862c69e1a1452b3677261`, which consolidates SPIRE CustomScan DML executor-state pointer access and removes 6 direct unsafe blocks.

The slice advances P10 and P11 by making DML executor helpers borrow the provider-owned exec state through one checked boundary instead of broad function-body unsafe blocks.

## Changed Files

- `src/am/ec_spire/custom_scan/dml.rs`

## Count Delta

See `artifacts/count-summary.md`.

The touched SPIRE DML file moves from 20 to 14 direct unsafe blocks. The `src/` total moves from 1961 after packet 087 to 1955 after this packet.

## Validation

- `git diff --check HEAD^ HEAD`
  - log: `artifacts/git-diff-check.log`
  - result: exit code 0
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - result: exit code 0 with the existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`
  - log: `artifacts/src-unsafe-block-count-after.log`
  - result: `1955` current direct unsafe blocks under `src/`
- `make unsafe-ledger`
  - log: `artifacts/unsafe-ledger-generate.log`
  - result: generated `1955` ledger rows
- `make unsafe-ledger-check`
  - log: `artifacts/unsafe-ledger-check.log`
  - result: `ledger covers 1955 current unsafe rows`

## Residual Unsafe

Residual SPIRE CustomScan DML unsafe remains in PostgreSQL expression evaluation, plan expression list reads, tuple descriptor walks, type I/O lookup, datum conversion, and output-function conversion. Those rows remain open in the Task 50 ledger for later deletion or residual registration.
