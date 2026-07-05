# Task 35 Review Request: DiskANN Routine Safety

## Summary

This packet reviews commit `3d02b3fb87eb86d6e40f5e63b38c32d8108ae416`, which documents the remaining unsafe contracts in `src/am/ec_diskann/routine.rs` and removes that file from `scripts/unsafe_comment_baseline.txt`.

The slice covers:

- DiskANN AM callback entry points for insert, vacuum, scan begin/rescan/gettuple/end
- index metadata reads and relation/attribute resolution
- backlink planning and source-vector fetches
- vacuum bulkdelete passes, stats allocation, medoid refresh flagging, and tuple rewrite application
- vacuum neighbor repair planning and heap source-vector fetches
- WAL-backed raw tuple byte reads/writes and page tuple bounds checks
- heap rerank prefetch and exact rerank source-vector access
- DiskANN routine test helpers that invoke AM callbacks or page rewrite helpers directly

## Result

Before this slice, the unsafe baseline had `798` total entries across `39` files, with `91` entries in `src/am/ec_diskann/routine.rs`.

After this slice, the unsafe baseline has `707` total entries across `38` files, and `src/am/ec_diskann/routine.rs` has `0` baseline entries.

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` exited 0 and wrote `707` entries.
- `bash scripts/check_unsafe_comments.sh` exited 0.
- `bash scripts/unsafe_baseline_report.sh` reported `entries: 707`, `files: 38`.
- `git diff --check` exited 0.
- `cargo check --all-targets --no-default-features --features pg18,bench` exited 0.

`cargo check` still emits the known unrelated unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the full artifact list and key result lines. The main after-state evidence is:

- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/diskann-routine-count-after-format.log`
- `artifacts/unsafe-audit-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
