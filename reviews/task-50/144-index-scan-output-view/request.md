# Task 50 Review Request: Index Scan Output View

## Summary

This packet addresses reviewer soundness finding #2 from
`reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-01-reviewer.md`
and the packet 042 cross-post.

`src/am/common/scan_output.rs` no longer exposes safe helper functions that take
and dereference raw `pg_sys::IndexScanDesc` pointers. It now exposes
`IndexScanOutput<'scan>`, whose constructor is `unsafe fn from_raw(...)` and
whose safe methods perform the heap TID and ORDER BY output writes. HNSW, IVF,
and SPIRE construct the view inside their AM scan callback bodies and pass the
view through local helpers.

## Code

- code commit: `7eb1ec04535333cf0816b5f877023b20eb8842f5`
- changed files:
  - `src/am/common/scan_output.rs`
  - `src/am/ec_hnsw/scan.rs`
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/scan/callbacks.rs`
  - `src/am/ec_spire/scan/relation.rs`

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`: `1550` direct unsafe blocks/functions across `124` files
- `make unsafe-ledger ...`: generated `artifacts/unsafe-ledger-after.jsonl`
- `make unsafe-ledger-check ...`: `ledger covers 1550 current unsafe rows`

## Review Notes

This is a soundness fix rather than a count-reduction slice. The direct unsafe
count remains `1550`, but the safe raw-pointer scan output facade is removed.
The remaining round-2 relation/string facade finding is tracked by
`reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-02-reviewer.md`.
