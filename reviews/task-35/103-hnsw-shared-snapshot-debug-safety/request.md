# Task 35 Review Request: HNSW Shared Snapshot Debug Safety

## Summary

This packet reviews commit `b5c9335a8a51844e59acc3a930550629b3c0630b`, the second layered `src/am/ec_hnsw/shared.rs` slice. It documents the remaining admin/cost/planner snapshot and debug-vacuum unsafe contracts, removing `shared.rs` from `scripts/unsafe_comment_baseline.txt`.

The slice covers:

- admin snapshot relation options, metadata, live tuple count, and block-count reads
- explain, cost, and planner integration snapshot delegation over a live index relation
- PostgreSQL relation catalog tuple access for `reltuples`
- planner cost constant reads from backend GUC state
- debug planner tuning snapshots
- debug data-page materialization tuple traversal
- debug metadata reads/updates under relation guards
- debug vacuum stats callback entry and cleanup invocation

## Result

Before this slice, the unsafe baseline had `580` total entries across `37` files, with `24` entries remaining in `src/am/ec_hnsw/shared.rs`.

After this slice, the unsafe baseline has `556` total entries across `36` files, and `src/am/ec_hnsw/shared.rs` has `0` baseline entries.

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` exited 0 and wrote `556` entries.
- `bash scripts/check_unsafe_comments.sh` exited 0.
- `bash scripts/unsafe_baseline_report.sh` reported `entries: 556`, `files: 36`.
- `git diff --check` exited 0.
- `cargo check --all-targets --no-default-features --features pg18,bench` exited 0.

`cargo check` still emits the known unrelated unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

## Artifacts

See `artifacts/manifest.md` for the full artifact list and key result lines. The main after-state evidence is:

- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/hnsw-shared-count-after-format.log`
- `artifacts/unsafe-audit-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
