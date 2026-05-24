# Task 50 Review Request: SPIRE CustomScan Planner Hook Helpers

## Summary

This packet reviews commit `0217ead0082eafd7bd029cb2e2b0f08b842e0020`, which consolidates SPIRE CustomScan planner hook helper boundaries and removes 7 direct unsafe blocks.

The slice advances P2 and P11 from the comprehensive burndown plan by moving planner candidate discovery and path attachment call sites onto checked helper boundaries, while retaining PostgreSQL allocation/callback/catalog unsafe inside named helpers.

## Changed Files

- `src/am/ec_spire/custom_scan/cost_helpers.rs`
- `src/am/ec_spire/custom_scan/planner.rs`

## Count Delta

See `artifacts/count-summary.md`.

The touched SPIRE planner file moves from 19 to 12 direct unsafe blocks. The `src/` total moves from 1970 after packet 085 to 1963 after this packet.

## Validation

- `git diff --check HEAD^ HEAD`
  - log: `artifacts/git-diff-check.log`
  - result: exit code 0
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - result: exit code 0 with the existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`
  - log: `artifacts/src-unsafe-block-count-after.log`
  - result: `1963` current direct unsafe blocks under `src/`
- `make unsafe-ledger`
  - log: `artifacts/unsafe-ledger-generate.log`
  - result: generated `1963` ledger rows
- `make unsafe-ledger-check`
  - log: `artifacts/unsafe-ledger-check.log`
  - result: `ledger covers 1963 current unsafe rows`

## Residual Unsafe

Residual SPIRE CustomScan planner unsafe remains in callback chaining, PostgreSQL CustomPath/CustomScan allocation and `add_path`, DML expression copy, SQL placement index scan setup/execution, root/control object tuple reads, and the DML frontdoor baserel handoff. Those rows remain open in the Task 50 ledger for later deletion or residual registration.
