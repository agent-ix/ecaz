# Task 35 Packet 041: IVF Insert Safety

## Code Under Review

- Commit: `c3af01e076cbb303d22bf274d06b032d51f95de3`
- Scope: `src/am/ec_ivf/insert.rs` and
  `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in the IVF insert callback and
insert debug helper. It covers the PostgreSQL AM callback entry, empty-index
bootstrap locking, tuple reencoding for PQ FastScan, trained-index centroid and
directory lookups, posting append and counter updates, duplicate heap TID debug
validation, and single-tuple bootstrap build flushing.

Key safety boundaries documented:

- PostgreSQL `aminsert` callback pointer and array lifetimes
- relation OID lock/unlock pairing in `RelationLockGuard`
- empty-bootstrap lock recheck before initializing an index from one tuple
- PQ FastScan model loading from live index metadata
- trained-index centroid and directory-chain reads
- posting append into the selected list block range
- directory and metadata update callbacks for insert accounting
- duplicate heap TID debug scan over guarded relation storage

## Baseline Accounting

- Global unsafe-comment baseline: `2694 -> 2673`
- `src/am/ec_ivf/insert.rs`: `21 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2694` global entries and `21 src/am/ec_ivf/insert.rs`.
- `artifacts/ivf-insert-baseline-before.log`: pre-slice IVF insert baseline
  entry list.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2673` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2673` global entries and no remaining `src/am/ec_ivf/insert.rs` entry.
- `artifacts/ivf-insert-baseline-after.log`: after-count output showing
  `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all`.
- `artifacts/cargo-check-pg18-bench.log`:
  `cargo check --all-targets --no-default-features --features pg18,bench`
  completed successfully with the known unrelated warnings in
  `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `artifacts/final-diff.patch`: final review diff for the slice.
