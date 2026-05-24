# Task 50 Review Request: Planner Relation Stats Unsafe Boundary

## Summary

This packet addresses the `common/cost.rs::relation_reltuples` duplicate/raw
wrapper item from:

- `reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-02-reviewer.md`
- `reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-03-reviewer.md`

The planner relation stats wrappers now require unsafe context, matching the
strict rule that helpers accepting raw PostgreSQL relation pointers must not
hide the live-relation precondition behind a safe signature.

## Code

- code commit: `5ce5415c`
- changed file: `src/am/common/cost.rs`

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`: `1613` direct unsafe blocks/functions across `126` files
- `make unsafe-ledger ...`: generated `artifacts/unsafe-ledger-after.jsonl`
- `make unsafe-ledger-check ...`: `ledger covers 1613 current unsafe rows`

## Review Notes

This is a soundness-signature correction. The direct unsafe count did not change
because these wrappers are already called from unsafe planner callback paths.
