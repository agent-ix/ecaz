# Task 50 Review Request: StringInfo Reader View

## Summary

This packet addresses the `storage/string_info.rs` portion of
`reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-02-reviewer.md`.

`storage/string_info.rs` no longer exposes safe helper functions that accept a
raw `pg_sys::StringInfo`. It now exposes `StringInfoReader<'msg>`, whose
constructor is `unsafe fn from_raw(...)` and whose safe methods read remaining
length, copy message bytes, and finish the receive buffer. The receive paths in
`src/lib.rs` construct the reader at the PostgreSQL type-receive boundary.

## Code

- code commit: `61d149fe6ddca844eb8ecc487b59393c45af88f0`
- changed files:
  - `src/storage/string_info.rs`
  - `src/lib.rs`

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`: `1550` direct unsafe blocks/functions across `124` files
- `make unsafe-ledger ...`: generated `artifacts/unsafe-ledger-after.jsonl`
- `make unsafe-ledger-check ...`: `ledger covers 1550 current unsafe rows`

## Review Notes

This is a soundness fix rather than a count-reduction slice. The direct unsafe
count remains `1550`, but the safe raw-pointer `StringInfo` facade is removed.
The larger `storage/relation.rs` facade finding remains open for the next
structural relation-boundary slice.
