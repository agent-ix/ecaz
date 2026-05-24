# Task 50 Review Request: HNSW Dead Scalar Adjacency Helper

## Summary

Removed unused scalar `load_graph_adjacency` from
`src/am/ec_hnsw/graph.rs`.

The live exact and grouped adjacency paths remain:

- `load_exact_graph_adjacency`
- `load_grouped_graph_adjacency`

Those are still used by HNSW scan/debug and test coverage. This removes only
the declaration-only scalar helper left behind after the non-storage traversal
wrappers were deleted.

## Unsafe Burndown

- `src/am/ec_hnsw/graph.rs` unsafe grep count: `68 -> 65`
- repository `src` unsafe grep count: `2446 -> 2443`
- exact deleted-symbol search returns no remaining references

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_hnsw/graph.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib ec_hnsw --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify `load_graph_adjacency` was genuinely dead and that this does not
affect the live exact/grouped adjacency helpers.
