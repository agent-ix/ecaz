# Task 50 Review Request: HNSW Frontier And Top-Level Debug Safe Helpers

## Summary

This slice makes a small HNSW frontier/top-level graph debug cluster safe at
the exported test API surface:

- `debug_rescan_entry_candidate_state`
- `debug_rescan_successor_candidate_state`
- `debug_rescan_candidate_frontier`
- `debug_all_top_level_heap_tids`
- `debug_top_level_reachable_heap_tids`

The helpers own their scan or relation guard setup. Raw scan opaque borrows and
graph/page reads remain explicit internal unsafe boundaries. Test callers now
invoke these helpers directly, and the now-unused `hnsw_graph_debug!` macro was
removed.

## Files Changed

- `src/am/ec_hnsw/scan_debug.rs`
- `src/tests/ec_hnsw_graph_lifecycle.rs`
- `src/tests/ec_hnsw_recall_helpers.rs`
- `src/tests/ec_hnsw_scan_gettuple.rs`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2251 -> 2249`.
- Removed HNSW scan/recall/graph debug macro wrappers around the newly-safe
  frontier/top-level helpers.
- Removed the unused `hnsw_graph_debug!` macro.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_hnsw/scan_debug.rs`
- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`

Known pre-existing warnings are unchanged:

- normal `cargo check`: SPIRE DML test re-export unused-import warning in
  `src/am/mod.rs`
- `pg_test` no-run: Hadamard test-only helper dead-code warnings

## Artifacts

- `artifacts/manifest.md`
- `artifacts/unsafe-count.log`
- `artifacts/hnsw-frontier-top-level-wrapper-grep.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pgtest-no-run.log`
