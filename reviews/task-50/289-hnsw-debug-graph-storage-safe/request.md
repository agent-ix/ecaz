# Review Request: HNSW Debug Graph Storage Safe Boundary

## Summary

This slice continues the HNSW debug unsafe burndown after packet 288.

Code commit: `3db0d8907fa817dbd0967651b0348ac30655000e`

Changes:

- Made private `debug_graph_storage` safe.
- Centralized the raw `GraphStorageDescriptor::from_index_relation` constructor inside that helper.
- Removed four caller-local unsafe blocks that only resolved graph storage from metadata already read from the open index relation.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2124 -> 2120`.
- Deleted the `unsafe fn` debug graph-storage API and all caller unsafe wrappers around it.

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass

Artifact manifest: `reviews/task-50/289-hnsw-debug-graph-storage-safe/artifacts/manifest.md`

