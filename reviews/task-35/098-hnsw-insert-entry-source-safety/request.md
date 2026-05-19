# Task 35 Review Request: HNSW Insert Entry/Source Safety

## Summary

This packet reviews commit `d552b865cdf8f94c1bb4ebd702aabc4f49e1636f`, which documents the remaining unsafe contracts in `src/am/ec_hnsw/insert.rs` and removes that file from `scripts/unsafe_comment_baseline.txt`.

The slice covers:

- live insert source-vector lookup/scoring and source-backed backlink scoring
- format adapter dispatch for duplicate detection, append, coalesce, and backlink update paths
- forward-neighbor discovery and backlink mutation planning/application
- physical append paths for TurboQuant, TurboQuant hot/cold, and PqFastScan
- duplicate scans and duplicate heap-TID coalescing for each insert storage format

## Result

Before this slice, the unsafe baseline had `1030` total entries across `41` files, with `133` entries in `src/am/ec_hnsw/insert.rs`.

After this slice, the unsafe baseline has `897` total entries across `40` files, and `src/am/ec_hnsw/insert.rs` has `0` baseline entries.

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` exited 0 and wrote `897` entries.
- `bash scripts/check_unsafe_comments.sh` exited 0.
- `bash scripts/unsafe_baseline_report.sh` reported `entries: 897`, `files: 40`.
- `git diff --check` exited 0.
- `cargo check --all-targets --no-default-features --features pg18,bench` exited 0.

`cargo check` still emits the known unrelated unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the full artifact list and key result lines. The main after-state evidence is:

- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/hnsw-insert-entry-count-after-format.log`
- `artifacts/unsafe-audit-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
