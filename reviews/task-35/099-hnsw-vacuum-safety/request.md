# Task 35 Review Request: HNSW Vacuum Safety

## Summary

This packet reviews commit `8269424299fbd4ef4607eaca63f975f63fb50633`, which documents the remaining unsafe contracts in `src/am/ec_hnsw/vacuum.rs` and removes that file from `scripts/unsafe_comment_baseline.txt`.

The slice covers:

- vacuum source-vector scoring and relation/attribute resolution
- `ambulkdelete` and `amvacuumcleanup` callback entry points
- pass-1 heap-TID removal planning and WAL-backed tuple rewrites
- metadata entry-point repair after finalization
- graph repair request collection, deleted-neighbor unlinking, replacement search, linear top-up, and repair-plan application
- grouped rerank payload reads during linear repair
- fully-dead element finalization
- debug vacuum helper callbacks

## Result

Before this slice, the unsafe baseline had `897` total entries across `40` files, with `99` entries in `src/am/ec_hnsw/vacuum.rs`.

After this slice, the unsafe baseline has `798` total entries across `39` files, and `src/am/ec_hnsw/vacuum.rs` has `0` baseline entries.

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` exited 0 and wrote `798` entries.
- `bash scripts/check_unsafe_comments.sh` exited 0.
- `bash scripts/unsafe_baseline_report.sh` reported `entries: 798`, `files: 39`.
- `git diff --check` exited 0.
- `cargo check --all-targets --no-default-features --features pg18,bench` exited 0.

`cargo check` still emits the known unrelated unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the full artifact list and key result lines. The main after-state evidence is:

- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/hnsw-vacuum-count-after-format.log`
- `artifacts/unsafe-audit-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
