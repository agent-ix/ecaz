# Review Request: SPIRE Per-Level Nprobe

## Summary

Task 30 Phase 8 pull-forward now has durable per-level recursive `nprobe`
configuration before the scale measurement run.

Code checkpoint: `60146ceb` (`Add SPIRE per-level nprobe policy`)

## Scope

- Adds the `nprobe_per_level` reloption as a comma-separated list ordered from
  recursive level 2 upward.
- Keeps level 1 on the existing relation/session `nprobe` resolution path.
- Keeps omitted upper levels on the conservative one-child policy.
- Carries the policy through `SpireSingleLevelScanPlan` so routed snapshot,
  quantized candidate, and top-graph candidate paths use the same recursive
  policy.
- Updates `ec_spire_index_options_snapshot(index_oid)` diagnostics so configured
  upper-level entries report `configured_above_level_1`; conservative omitted
  levels continue to report `one_child_above_level_1`.
- Documents the reloption in `docs/SPIRE_DIAGNOSTICS.md`.
- Marks the Phase 3 carried-forward per-level `nprobe` follow-up complete while
  leaving the Phase 8 scale packet open.

## Validation

- `cargo test --no-default-features --features pg18 nprobe_per_level`
  - `test am::ec_spire::options::tests::nprobe_per_level_reloption_parses_upper_level_values ... ok`
- `cargo test --no-default-features --features pg18 route_recursive_routing_objects_to_leaf_pids_uses_configured_upper_level_nprobe`
  - `test am::ec_spire::scan::tests::routing::route_recursive_routing_objects_to_leaf_pids_uses_configured_upper_level_nprobe ... ok`
- `cargo pgrx test pg18 test_ec_spire_options_snapshot_sql`
  - `test tests::pg_test_ec_spire_options_snapshot_sql ... ok`
- `cargo fmt --check`
  - Exits 0 with the existing stable-rustfmt warnings for unstable import config.
- `git diff --check`

## Notes

This is the one Phase 8 routing-quality pull-forward called out by the reviewer.
The broader routing-quality ladder remains Phase 9 scope; the controlled scale
measurement packet is still open.
