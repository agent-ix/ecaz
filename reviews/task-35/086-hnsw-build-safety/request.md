# Task 35 Packet 086: HNSW Build Safety

## Summary

This slice documents the remaining unsafe boundaries in `src/am/ec_hnsw/build.rs` and removes the file from `scripts/unsafe_comment_baseline.txt`.

The comments cover:

- PostgreSQL `ambuild`, `ambuildempty`, and build callback C-boundary guards
- build-state relation option and indexed/source attribute resolution
- indexed datum and heap scan slot extraction
- build flush dispatch into page writes and metadata initialization
- page allocation, generic WAL registration, page initialization, tuple insertion, and WAL finish

## Code Under Review

- Code commit: `45385acb929b690ee7619861979129828057a1df`
- Files changed:
  - `src/am/ec_hnsw/build.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1725 -> 1692`
- Baseline files: `49 -> 48`
- `src/am/ec_hnsw/build.rs`: `33 -> 0`

Evidence:

- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/hnsw-build-baseline-before.log`
- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/hnsw-build-baseline-after.log`

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` passed and wrote `1692` entries after formatting.
- `bash scripts/check_unsafe_comments.sh` passed.
- `bash scripts/unsafe_baseline_report.sh` reported `1692` entries across `48` files.
- `git diff --check` passed.
- `cargo fmt --all` completed. It emitted the existing stable-rustfmt warnings about unstable import grouping options.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

Artifacts:

- `artifacts/unsafe-audit-before.log`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/diff-before-format.patch`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/unsafe-audit-after.log`
- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/hnsw-build-baseline-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/final-diff.patch`

## Reviewer Notes

This is comment-only in Rust source plus baseline removal. No build logic or data layout behavior changed.
