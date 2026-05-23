# Task 50 Review Request: SPIRE Custom Scan List Boundary

## Summary

Code commit: `3c48f99ac531f7bfe995e6475fb8f3bde8ae0484`

This slice advances the P11 planner/list/custom-scan view program for SPIRE:

- added shared custom-scan `List` helpers for length, tag, nth-node, and nth-OID
  access in `custom_scan/cost_helpers.rs`
- rewired DML plan-private metadata decoding in `custom_scan/plan_private.rs`
  through those helpers
- removed direct raw `List` length and `list_nth` access from the higher-level
  plan-private decoding paths

Direct unsafe count moved from packet 137's `1548` to `1544`.

## Validation

- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - passed with the known pre-existing `src/am/mod.rs` unused import warning
- `make unsafe-block-count`
  - `unsafe_blocks 1544`
  - `files 123`
- `make unsafe-ledger`
- `make unsafe-ledger-check`
  - `ledger covers 1544 current unsafe rows`

## Artifacts

- `artifacts/code-stat.log`
- `artifacts/code-diff.patch`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/src-unsafe-block-count-after.log`
- `artifacts/count-summary.md`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
