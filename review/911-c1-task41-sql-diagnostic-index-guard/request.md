# Review Request: Task 41 SQL diagnostic index guard

Code commit: `626edae157a957b964d67c5a5e78246cdb4a5e21`

## Summary

This packet starts Task 41 by wrapping SQL diagnostic index relations in
`src/lib.rs`.

- Added `AccessShareIndexRelation`, an RAII guard for
  `index_open(... AccessShareLock)` / `index_close(... AccessShareLock)`.
- Centralized AM validation through `open_valid_ec_index_guard`.
- Kept legacy raw-pointer helpers for existing callers by converting the guard
  with `into_raw`, so this packet does not churn every SQL diagnostic at once.
- Switched `ec_hnsw_index_admin_snapshot` and
  `ec_diskann_index_graph_summary` to keep the guard directly.
- Removed manual error-path closes from validation and DiskANN graph summary.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: 4725 entries.
- After: 4700 entries.
- Net change: 25 fewer grandfathered unsafe-comment baseline entries.

## Reviewer Focus

- Confirm `AccessShareIndexRelation` owns exactly one AccessShare index
  relation close.
- Confirm the validation helper keeps the relation alive while reading
  `rd_rel` and formatting `relname`.
- Confirm `into_raw` preserves the old close contract for existing callers
  until later Task 41 slices migrate them to guards.

## Validation

- `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-911.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
