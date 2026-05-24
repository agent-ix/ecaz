# Review Request: HNSW Debug Graph Element Helper Safe Surface

## Summary

This checkpoint centralizes HNSW debug graph element tuple loading behind
`debug_load_graph_element` in `src/am/ec_hnsw/scan_debug.rs`.

The change removes the repeated caller-side `unsafe { graph::load_exact_graph_element(...) }`
blocks from graph debug collectors, top-level/reachable debug helpers, oracle seed
scans, carrydown scans, layer-0 neighbor expansion, and exact-seed scan debug paths.
The one remaining raw call in this file is the helper boundary itself.

## Code Commit

- `8ed8c3206ef6cd454d0e6b5fd9155406c8df9248` - `Centralize HNSW debug graph element loads`

## Unsafe Count

- Previous packet baseline after packet 291: `2106`
- After this checkpoint: `2091`
- Net change: `-15`

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
