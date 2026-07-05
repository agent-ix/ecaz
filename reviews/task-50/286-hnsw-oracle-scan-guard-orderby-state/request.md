# Review Request: HNSW Oracle Scan Guard And Order-By State

## Summary

This slice continues the HNSW debug unsafe burndown after packet 285.

Code commit: `12d38ce1872e833fd58191ba97912194c5180a44`

Changes:

- Added `DebugAmScan::with_oracle_score_parts` so oracle debug probes can prepare query state through the descriptor-owning guard.
- Converted five oracle-score debug probes from raw AM scan descriptors and explicit cleanup to `DebugAmScan`:
  - `debug_top_level_oracle_k_seed_heap_tids`
  - `debug_top_level_oracle_k_seed_scan_heap_tids`
  - `debug_layer_oracle_k_carrydown_scan_heap_tids`
  - `debug_layer_oracle_k_seed_layer0_neighbor_heap_tids`
  - `debug_exact_seed_scan_heap_tids`
- Consolidated duplicate raw `xs_orderbyvals` / `xs_orderbynulls` reads into one `debug_scan_orderby_score_state` boundary shared by both order-by score helpers.

## Unsafe Burned Down

- Broad `rg -n "unsafe" src | wc -l`: `2138 -> 2137`.
- Removed repeated manual AM scan cleanup from the converted oracle probes.
- Reduced duplicated order-by descriptor dereference helpers from two unsafe blocks to one shared helper.

## Validation

- `git diff --check`: pass
- `cargo check --all-targets --no-default-features --features pg18,bench`: pass
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`: pass

Artifact manifest: `reviews/task-50/286-hnsw-oracle-scan-guard-orderby-state/artifacts/manifest.md`

