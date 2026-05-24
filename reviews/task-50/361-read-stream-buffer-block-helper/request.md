# Task 50 Review Request: Read Stream Buffer Block Helper

## Summary

This slice advances P9 read-stream cleanup by centralizing duplicate per-buffer `BlockNumber` metadata reads in `am::common::stream`.

Code commit: `9c2407a69b67f421a06c1da265f39ef5c2da1a8f`

Changes:

- Added private `read_stream_per_buffer_block_number`.
- Reused it in both guarded relation stream visits and scan-owned read stream buffer handling.
- Removed one duplicate direct unsafe read of callback-provided per-buffer block metadata.

Unsafe count:

- Before: `1219`
- After: `1218`
- Delta: `-1`

Targeted scan result:

- The `per_buffer_data.cast::<pg_sys::BlockNumber>()` read now exists only in `read_stream_per_buffer_block_number`.

## Validation

Artifacts are under `reviews/task-50/361-read-stream-buffer-block-helper/artifacts/`.

- `cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed. It reports the pre-existing SPIRE DML re-export warning in `src/am/mod.rs`.
- `git-diff-check.log`: `git diff --check` passed.
- `unsafe-count.log`: `1218`.
- `raw-boundary-guard.log`: no matches.
- `read-stream-buffer-block-scan.log`: one helper-owned per-buffer block read and both helper call sites.
- `unsafe-ledger-after.jsonl` and `unsafe-ledger-check.log`: ledger regenerated and covers all `1218` current unsafe rows.
