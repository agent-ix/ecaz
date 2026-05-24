# Task 50 Review Request: GenericXLog Register Return Check

## Summary

This packet addresses reviewer soundness finding #6 from
`reviews/task-50/132-helper-soundness-audit/feedback/2026-05-20-01-reviewer.md`
and the re-review finding in
`reviews/task-50/050-generic-xlog-full-image-registration/feedback/2026-05-20-02-reviewer.md`.

`GenericXLogTxn::register_locked_buffer_full_image` now checks the page pointer
returned by `pg_sys::GenericXLogRegisterBuffer` before returning it to AM
callers. The helper already centralizes full-image GenericXLog registration, so
the check covers DiskANN, IVF, HNSW, and SPIRE callers without changing each
call site.

## Code

- code commit: `19ae8d77583fbcac86e1595854bc51666e1bbf0d`
- changed file: `src/storage/wal.rs`

## Validation

- `git diff --check HEAD^ HEAD`: passed
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with the pre-existing `src/am/mod.rs` unused SPIRE DML import warning
- `make unsafe-block-count`: `1550` direct unsafe blocks/functions across `124` files
- `make unsafe-ledger ...`: generated `artifacts/unsafe-ledger-after.jsonl`
- `make unsafe-ledger-check ...`: `ledger covers 1550 current unsafe rows`

## Review Notes

This is a soundness fix rather than a count-reduction slice. The direct unsafe
count remains `1550`, but callers no longer receive an unchecked page pointer
from the shared WAL registration boundary.
