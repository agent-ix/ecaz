# Review Request: HNSW Debug Adjacency and Frontier Helper Safe Surface

## Summary

This checkpoint adds safe debug wrappers for graph adjacency loading and
bootstrap frontier prefetch/consume probes in `src/am/ec_hnsw/scan_debug.rs`.

The change moves the remaining debug-probe caller-side unsafe blocks for
`graph::load_exact_graph_adjacency`, `prefetch_next_graph_traversal_result`,
and `consume_and_refill_bootstrap_frontier` into named helper boundaries. Callers
now use the same safe debug surface as the previous graph element loader slice.

## Code Commit

- `0bcc9acd83c8aaf5c32545ac9ce45df86c7204fd` - `Centralize HNSW debug adjacency helpers`

## Unsafe Count

- Previous packet baseline after packet 292: `2091`
- After this checkpoint: `2086`
- Net change: `-5`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- `rg -n unsafe src --count-matches`

The cargo commands pass. The logs include the known pre-existing SPIRE unused-import
warning and Hadamard test-only dead-code warnings.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pg-test-no-run.log`
- `artifacts/unsafe-count-by-file.log`
