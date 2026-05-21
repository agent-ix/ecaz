# Review Request: HNSW Page-Line Debug Helper Safe Boundary

## Summary

This slice continues the HNSW debug unsafe burndown after packet 286.

Code commit: `68e83c7ea2641a058115722d7cb76d412ddc29c8`

Changes:

- Made `debug_with_page_line_tuple_bytes` a safe debug helper. It still contains the validated shared page-line tuple boundary internally, but callers no longer need local unsafe blocks.
- Removed three caller `unsafe` blocks around element-tag checks in the graph element collector helpers.
- Kept the packet 286 oracle scan guard reuse and order-by score-state consolidation in the same finalized code slice.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2137 -> 2133`.
- Replaced repeated caller-local page tuple inspection unsafe blocks with one named debug boundary.
- Preserved the shared lower-level bounds validation in `shared::with_page_line_tuple_bytes`.

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass

Artifact manifest: `reviews/task-50/287-hnsw-page-line-debug-helper-safe/artifacts/manifest.md`

