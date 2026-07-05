# Task 50 Review Request: HNSW Dead Graph Traversal Wrappers

## Summary

Removed unused HNSW graph/scan helper surfaces that had no exact references
under `src/am/ec_hnsw`:

- non-storage graph traversal wrappers superseded by the live
  `*_with_storage` APIs,
- unused grouped rerank tuple callback wrapper,
- unused layer-neighbor TID loaders,
- unused scan-state grouped rerank scorer.

The live HNSW insert, vacuum, scan, and debug paths continue to use the
storage-descriptor traversal helpers such as
`greedy_descend_from_entry_with_storage`,
`search_layer*_result_candidates_with_storage`,
`load_layer0_refill_successors_with_storage`, and
`expand_layer0_visible_seeds_with_storage`.

## Unsafe Burndown

- `src/am/ec_hnsw/graph.rs` unsafe grep count: `94 -> 68`
- `src/am/ec_hnsw/scan.rs` unsafe grep count: `203 -> 202`
- repository `src` unsafe grep count: `2473 -> 2446`
- exact deleted-symbol search returns no remaining references

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_hnsw/graph.rs src/am/ec_hnsw/scan.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib ec_hnsw --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Completion Audit Note

Task 50 is not complete after this packet. The live `src` unsafe count remains
`2446`, and the comprehensive plan requires every direct unsafe to be removed,
centralized, or residual-registered with ownership and invariants.

## Review Focus

Please verify the deleted non-storage wrappers were genuinely dead and that the
remaining storage-descriptor traversal APIs still cover all live HNSW graph
paths.
