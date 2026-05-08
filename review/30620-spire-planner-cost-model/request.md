# SPIRE Planner Cost Model

## Scope

This packet replaces the SPIRE planner cost gate with an active-snapshot-aware
cost estimate and wires PG18 tree height to the actual SPIRE hierarchy depth.

Code checkpoint: `82c1fd36` (`Model SPIRE planner cost from active hierarchy`)

## Changes

- Replaces `cost::gated_planner_cost_estimate(block_count)` in
  `ec_spire_amcostestimate` with a SPIRE-specific estimator.
- Factors planner cost through relation `nlists`, effective `nprobe`,
  `local_store_count`, storage format, rerank width, active hierarchy depth,
  routing child distribution, leaf assignment count, and routing/leaf object
  bytes from the active snapshot diagnostics.
- Models routing work as startup cost and selected leaf/candidate work as run
  cost.
- Implements `ec_spire_amgettreeheight` from
  `ec_spire_index_hierarchy_snapshot(...).hierarchy_depth` instead of the
  previous hardcoded zero.
- Adds pure Rust estimator tests for probe count, recursive depth, and
  local-store fanout cost effects.
- Marks the Phase 8 planner-cost task complete.

## Files

- `src/am/ec_spire/cost.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

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

This makes planner-visible SPIRE paths finite and hierarchy-sensitive. It does
not claim benchmark calibration; the Phase 8 benchmark harness and scale packet
remain separate work.
