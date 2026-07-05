# Task 50 Review Request: HNSW Graph Neighbor Debug Safe Helpers

## Summary

This slice makes two HNSW graph-inspection debug helpers safe at their exported
test API surface:

- `debug_entry_point_neighbor_tids`
- `debug_layer0_reachable_live_element_tids`

Both helpers own the index relation guard and already keep metadata/page/graph
tuple reads inside explicit internal unsafe boundaries. Scan and SQL-visible
debug callers now invoke these helpers directly.

## Files Changed

- `src/am/ec_hnsw/scan_debug.rs`
- `src/tests/ec_hnsw_scan_gettuple.rs`
- `src/tests/ec_hnsw_recall_debug_exports.rs`

## Unsafe Burndown

- Broad `src` unsafe grep hits: `2254 -> 2251`.
- Removed HNSW scan/recall-export debug macro wrappers around the newly-safe
  graph-neighbor helpers.

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
- `artifacts/hnsw-graph-neighbor-wrapper-grep.log`
- `artifacts/rustfmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pgtest-no-run.log`
