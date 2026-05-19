# Task 35 Review Request: HNSW Shared Page Metadata Safety

## Summary

This packet reviews commit `30c0b220f25f3525b764d69a314545e27b16a76c`, the first layered `src/am/ec_hnsw/shared.rs` slice. It documents the page metadata, live-entry traversal, tuple byte access, and debug page-read unsafe contracts, leaving the admin/cost/debug-vacuum layer for a follow-up packet.

The slice covers:

- metadata page initialization, update, locked mutation, WAL registration, and page special-area writes
- noop vacuum stats allocation and live tuple count setup
- PG18 read-stream live tuple counting and non-PG18 sequential buffer counting
- page line-pointer traversal and live-entry candidate selection
- immutable and mutable tuple byte visitors with tuple bounds checks
- heap TID decoding from PostgreSQL callback pointers
- debug index page/materialization helpers and metadata page reads

## Result

Before this slice, the unsafe baseline had `629` total entries across `37` files, with `73` entries in `src/am/ec_hnsw/shared.rs`.

After this slice, the unsafe baseline has `580` total entries across `37` files, and `src/am/ec_hnsw/shared.rs` has `24` baseline entries remaining for the next layer.

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` exited 0 and wrote `580` entries.
- `bash scripts/check_unsafe_comments.sh` exited 0.
- `bash scripts/unsafe_baseline_report.sh` reported `entries: 580`, `files: 37`.
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
