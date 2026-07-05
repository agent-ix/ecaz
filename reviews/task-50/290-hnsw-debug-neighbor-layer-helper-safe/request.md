# Review Request: HNSW Debug Neighbor Layer Helper Safe API

## Summary

This slice continues the HNSW debug unsafe burndown after packet 289.

Code commit: `78c43bd7c846c0e3e46b7510ca8d5463c2081bfe`

Changes:

- Made private `debug_load_neighbor_tids_for_layer` safe.
- Kept the `load_exact_graph_adjacency` unsafe boundary inside the named helper, where graph tuple validation already happens.
- Removed three caller-local unsafe blocks around layer-neighbor collection.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2120 -> 2116`.
- Deleted one `unsafe fn` debug helper API plus three caller unsafe wrappers.

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass

Artifact manifest: `reviews/task-50/290-hnsw-debug-neighbor-layer-helper-safe/artifacts/manifest.md`

