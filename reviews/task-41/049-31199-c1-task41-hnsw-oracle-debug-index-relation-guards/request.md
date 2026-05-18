# Review Request: Task 41 HNSW Oracle Debug Index Relation Guards

## Summary

This slice migrates the first HNSW oracle/debug helper group from manual
`index_open` / `index_close` pairs to the shared `IndexRelationGuard`.

Code commit: `c61f470c3aa31c2bc06e93a2e3f03e35b3bd28d5`

## Changes

Updated these pg-test helpers in `src/am/ec_hnsw/scan_debug.rs`:

- `debug_top_level_oracle_k_seed_heap_tids`
- `debug_top_level_oracle_k_seed_scan_heap_tids`
- `debug_layer_oracle_k_carrydown_scan_heap_tids`
- `debug_layer_oracle_k_seed_layer0_neighbor_heap_tids`
- `debug_exact_seed_scan_heap_tids`

Each helper now opens the index relation through `IndexRelationGuard` and
keeps a local raw pointer derived from `as_ptr()` for the existing graph/scan
calls. Early-return paths now drop through the guard instead of requiring an
explicit manual close.

## Baseline

- Before: `4235`
- After: `4220`

## Review Focus

- Confirm every removed manual `index_close` is covered by
  `IndexRelationGuard` drop, including early-return paths.
- Confirm the raw `index_relation` pointer remains valid for the same lexical
  scope because the guard is held until the helper returns.
- Confirm scan lifetime still ends before the relation guard drops:
  each migrated helper still calls `ec_hnsw_amendscan` and `IndexScanEnd`
  before returning.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
