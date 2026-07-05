# Review Request: SPIRE Level Parameter Diagnostics

Head SHA: `c50252b1`

## Summary

SPIRE now exposes per-routing-level build and scan parameters through
`ec_spire_index_level_parameter_snapshot(index_oid)`.

The new snapshot emits one row per active routing level with:

- routing object and child counts;
- target fanout;
- relation/session/effective `nprobe`;
- the active per-level `nprobe` policy;
- training sample rows and training iterations;
- centroid dimensions;
- distance semantics;
- assignment payload format.

For current recursive indexes, level 1 reports the relation/session leaf-level
`nprobe` resolution and upper levels report the conservative one-child policy.
Valid recursive hierarchies now report
`per_level_nprobe_supported = true` in
`ec_spire_index_hierarchy_snapshot(index_oid)` because the active level
metadata and policy are SQL-visible.

## Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test recursive_fanout_build_hierarchy -- --nocapture`
  - 1 passed, including PG18 pg-test
    `pg_test_ec_spire_recursive_fanout_build_hierarchy`.
- `cargo fmt`
- `git diff --check`

## Review Focus

- Confirm the level-parameter snapshot covers the Phase 3 hierarchy metadata
  requirement without introducing a new reloption surface yet.
- Confirm `per_level_nprobe_supported = true` is appropriate for valid
  recursive hierarchies now that each level's active policy is visible.
- Confirm the current upper-level policy should remain conservative one-child
  routing until a richer control surface lands.
