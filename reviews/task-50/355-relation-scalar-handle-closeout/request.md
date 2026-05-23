# Task 50 Review Request: Relation Scalar Handle Closeout

## Summary

This slice continues the comprehensive unsafe burndown by removing remaining caller-side unsafe relation scalar reads in the common, DiskANN, HNSW, and SPIRE paths.

Code commit: `6c13902b12784d9d370cd15e7be03ec3f51ed3c6`

Changes:

- Added `relation_am_oid_handle(RelationHandle)` and removed the now-unused raw `relation_reltuples` wrapper.
- Converted common EXPLAIN/cost relation scalar reads to `NonNull` relation handles.
- Converted DiskANN scan/build block-count reads to `main_fork_block_count_handle`.
- Converted SPIRE page relation wrapper to hold a checked `RelationHandle`, removing repeated unsafe constructor calls and the raw block-count wrapper call.
- Added a local HNSW insert block-count helper so append and duplicate-scan paths no longer call the raw relation wrapper directly.

Unsafe count:

- Before: `1265`
- After: `1246`
- Delta: `-19`

Targeted scan result:

- No remaining matches for direct `unsafe { crate::storage::relation::{main_fork_block_count,relation_reltuples,relation_am_oid}... }` in `src/am/common`, `src/am/ec_diskann`, `src/am/ec_hnsw`, or `src/am/ec_spire`.

## Validation

Artifacts are under `reviews/task-50/355-relation-scalar-handle-closeout/artifacts/`.

- `cargo-check-pg18-bench-final.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed. It reports the pre-existing SPIRE DML re-export warning in `src/am/mod.rs`.
- `git-diff-check-final.log`: `git diff --check` passed.
- `unsafe-count-final.log`: `1246`.
- `raw-boundary-guard-final.log`: no matches.
- `relation-scalar-targeted-scan-final.log`: no matches.
- `unsafe-ledger-after.jsonl` and `unsafe-ledger-check-final.log`: ledger regenerated and covers all `1246` current unsafe rows.

The initial cargo run caught a DiskANN scan-state shadowing bug introduced during this slice; that was fixed before the final validation run.
