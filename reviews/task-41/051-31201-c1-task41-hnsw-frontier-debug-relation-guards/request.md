# Review Request: Task 41 HNSW Frontier Debug Relation Guards

## Summary

This slice migrates the next HNSW pg-test candidate-frontier/lifecycle debug
helper group from manual `index_open` / `index_close` pairs to
`IndexRelationGuard`.

Code commit: `a374f201ac347934244e35912cc8d66d5d9aa399`

## Changes

Updated these helpers in `src/am/ec_hnsw/scan_debug.rs`:

- `debug_materialize_bootstrap_candidate_result`
- `debug_bootstrap_phase_transition`
- `debug_candidate_frontier_head_lifecycle`
- `debug_consume_candidate_frontier_head`
- `debug_consume_candidate_frontier_head_slots`
- `debug_visited_seed_lifecycle`
- `debug_entry_candidate_lifecycle`

Each helper now opens the index relation through `IndexRelationGuard`, borrows
the raw relation pointer only while the guard is live, and keeps explicit AM
scan cleanup before returning.

## Baseline

- Before: `4202`
- After: `4188`

## Review Focus

- Confirm every removed manual close is covered by `IndexRelationGuard`.
- Confirm each `ec_hnsw_amendscan` and `IndexScanEnd` remains before the
  relation guard drops.
- Confirm callbacks that pass `index_relation` into graph traversal helpers
  still hold the guard for the full helper scope.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
