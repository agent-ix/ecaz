# Review Request: HNSW Debug Graph Collectors Safe API

## Summary

This slice continues the HNSW debug unsafe burndown after packet 287.

Code commit: `ecf0427cfd875798331360a605856ee10911e54d`

Changes:

- Made the private graph collector helpers safe:
  - `debug_collect_element_tids_at_level`
  - `debug_collect_element_tids_at_or_above_level`
  - `debug_collect_element_tid_by_heap_tid`
- Removed caller-local unsafe blocks for these collectors, including stale wrappers surfaced by validation after packet 287’s safe page-line helper.
- Kept raw page reads, tuple loads, and relation block-count access centralized inside the collector implementation.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2133 -> 2124`.
- Removed three `unsafe fn` helper APIs and six caller unsafe wrappers around collector calls.

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass

Artifact manifest: `reviews/task-50/288-hnsw-debug-graph-collectors-safe/artifacts/manifest.md`

