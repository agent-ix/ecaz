# Review Request: HNSW Debug Graph Search Helper Safe Surface

## Summary

This checkpoint centralizes HNSW debug graph search calls in
`src/am/ec_hnsw/scan_debug.rs`.

The change adds safe debug wrappers for layer-0 and upper-layer graph result
candidate search helpers, then rolls them through oracle seed/carrydown/exact
seed debug probes. It also reuses `debug_graph_storage` for grouped-storage
classification instead of re-resolving the graph storage descriptor at the call
site.

## Code Commit

- `741b4aa1ac650987017db862a10596ec4d7b4096` - `Centralize HNSW debug graph search helpers`

## Unsafe Count

- Previous packet baseline after packet 295: `2076`
- After this checkpoint: `2073`
- Net change: `-3`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1399 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- `rg -n unsafe src --count-matches`
- `make UNSAFE_LEDGER=reviews/task-50/296-hnsw-debug-graph-search-helper-safe/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/296-hnsw-debug-graph-search-helper-safe unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/296-hnsw-debug-graph-search-helper-safe/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

The cargo commands pass. The logs include the known pre-existing SPIRE unused-import
warning and Hadamard test-only dead-code warnings.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pg-test-no-run.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
