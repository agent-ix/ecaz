# Task 50 Review Request: HNSW IndexInfo Guard

## Summary

This slice advances P5 heap/source metadata cleanup by replacing manual `BuildIndexInfo`/`pfree` pairing in HNSW source metadata resolution with an RAII guard.

Code commit: `c5469136ddff78cf7c5d46c9b73ebce2e3674388`

Changes:

- Added `IndexInfoGuard` in `src/am/ec_hnsw/source.rs`.
- Centralized `BuildIndexInfo` allocation and `pfree` cleanup in the guard.
- Removed the two manual `pfree(index_info)` unsafe blocks from indexed source/vector attribute resolution.
- Kept the existing metadata validation and typed attribute resolution behavior unchanged.

Unsafe count:

- Before: `1228`
- After: `1226`
- Delta: `-2`

Targeted scan result:

- `BuildIndexInfo` is now called only inside `IndexInfoGuard::build`.
- No manual `pfree(index_info...)` remains in `src/am/ec_hnsw/source.rs`.

## Validation

Artifacts are under `reviews/task-50/358-hnsw-index-info-guard/artifacts/`.

- `cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed. It reports the pre-existing SPIRE DML re-export warning in `src/am/mod.rs`.
- `git-diff-check.log`: `git diff --check` passed.
- `unsafe-count.log`: `1226`.
- `raw-boundary-guard.log`: no matches.
- `hnsw-index-info-scan.log`: shows the single remaining `BuildIndexInfo` boundary inside the guard and no manual `pfree(index_info...)` matches.
- `unsafe-ledger-after.jsonl` and `unsafe-ledger-check.log`: ledger regenerated and covers all `1226` current unsafe rows.
