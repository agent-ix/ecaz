# SPIRE Planner Cost Snapshot

## Scope

This packet publishes a SQL-visible SPIRE planner cost snapshot so benchmark
and review work can inspect the cost model inputs and modeled outputs directly.

Code checkpoint: `b2dcd56d` (`Publish SPIRE planner cost snapshot`)

## Changes

- Adds `ec_spire_index_cost_snapshot(index_oid)` alongside the existing
  `ec_ivf_index_cost_snapshot(...)` and `ec_hnsw_index_cost_snapshot(...)`
  surfaces.
- Exposes planner readiness, dimensions, configured `nlists`, active leaf
  count, relation/session/effective `nprobe`, local-store count, recursive
  fanout, resolved tree height, routing/leaf estimates, storage format,
  rerank-width resolution, index pages, reltuples, and modeled costs.
- Extends the recursive SPIRE PG18 test to assert the cost snapshot reports:
  - `resolved_tree_height = 2`
  - `tree_height_source = amgettreeheight_callback`
  - `effective_nprobe = 2`
  - finite modeled costs with total cost greater than startup cost.

## Files

- `src/am/ec_spire/cost.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo test ec_spire::cost`
- `cargo pgrx test pg18 test_ec_spire_recursive_fanout_build_hierarchy`
- `git diff --check`

## Notes

The SQL tuple is intentionally kept under pgrx's tuple-width limit while still
showing the cost drivers needed by the Phase 8 benchmark harness.
