# Task 50 Review Request: Reloptions String Blob

## Summary

This slice continues P7 reloptions cleanup by moving string reloption offset reads into a shared `ReloptionsBlob` wrapper.

Code commit: `aa4bacec4d93c37264b3d1699ed102e163c02e66`

Changes:

- Added `am::common::reloptions::ReloptionsBlob`, a small non-null wrapper around relation-owned reloptions storage.
- Moved string offset pointer arithmetic and `CStr` conversion into `ReloptionsBlob::read_string_reloption`.
- Removed AM-local unsafe wrappers around `read_string_reloption` in DiskANN, HNSW, IVF/RaBitQ, and SPIRE.
- Kept the AM-specific reloptions views responsible for passing offsets copied from their own parsed layout.

Unsafe count:

- Before: `1223`
- After: `1219`
- Delta: `-4`

Targeted scan result:

- No AM-local `unsafe { ... read_string_reloption ... }` wrappers remain in the four AM options modules.

## Validation

Artifacts are under `reviews/task-50/360-reloptions-string-blob/artifacts/`.

- `cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed. It reports the pre-existing SPIRE DML re-export warning in `src/am/mod.rs`.
- `git-diff-check.log`: `git diff --check` passed.
- `unsafe-count.log`: `1219`.
- `raw-boundary-guard.log`: no matches.
- `reloptions-string-scan.log`: no matches.
- `unsafe-ledger-after.jsonl` and `unsafe-ledger-check.log`: ledger regenerated and covers all `1219` current unsafe rows.
