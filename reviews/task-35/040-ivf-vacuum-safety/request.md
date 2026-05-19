# Task 35 Packet 040: IVF Vacuum Safety

## Code Under Review

- Commit: `e0fc09d5fc0aa9acfe4689d940537939f7a70e28`
- Scope: `src/am/ec_ivf/vacuum.rs` and
  `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in the IVF vacuum callbacks and
vacuum debug helpers. It covers AM callback guard entry, metadata reads,
directory traversal and rewrite, posting tuple rewrite callbacks, heap TID
deadness callbacks, stats allocation/update, and debug vacuum helper lifetimes.

Key safety boundaries documented:

- PostgreSQL callback invariants for `ec_ivf_ambulkdelete` and
  `ec_ivf_amvacuumcleanup`
- live IVF index relation requirements for metadata and directory-chain reads
- stats allocation through `PgBox::alloc0` when PostgreSQL passes null stats
- directory tuple rewrite against the just-read directory TID
- posting-list rewrite visitor scope and bulkdelete callback lifetime
- callback invocation with stack-local `ItemPointerData`
- debug callback state pointer provenance and debug vacuum stats copying

## Baseline Accounting

- Global unsafe-comment baseline: `2720 -> 2694`
- `src/am/ec_ivf/vacuum.rs`: `26 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2720` global entries and `26 src/am/ec_ivf/vacuum.rs`.
- `artifacts/ivf-vacuum-baseline-before.log`: pre-slice IVF vacuum baseline
  entry list.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2694` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2694` global entries and no remaining `src/am/ec_ivf/vacuum.rs` entry.
- `artifacts/ivf-vacuum-baseline-after.log`: after-count output showing
  `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all`.
- `artifacts/cargo-check-pg18-bench.log`:
  `cargo check --all-targets --no-default-features --features pg18,bench`
  completed successfully with the known unrelated warnings in
  `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `artifacts/final-diff.patch`: final review diff for the slice.
