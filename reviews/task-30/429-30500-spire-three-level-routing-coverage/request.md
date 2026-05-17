# Review Request: SPIRE Three-Level Routing Coverage

Head SHA: `8f13dee5`

## Summary

Recursive scan coverage now includes a true three-routing-level pure test:
root level 3, internal level 2, internal level 1, then leaves.

The test verifies the conservative recursive policy probes one internal child
at each upper routing level, then applies the configured leaf-level `nprobe`
only once the descent reaches level 1. This closes the depth coverage gap where
previous tests only exercised root-to-internal-to-leaf hierarchies.

The checkpoint also renames the internal policy type to
`SpireConservativeRecursiveNprobePolicy` and adds a TODO at the hardcoded
upper-level one-child default.

## Files

- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test descends_three_levels_conservatively -- --nocapture`
  - 1 passed:
    `route_recursive_routing_objects_to_leaf_pids_descends_three_levels_conservatively`.
- `cargo fmt`
- `git diff --check`

## Review Focus

- Confirm the three-level fixture is non-vacuous and catches upper-level
  policy regressions.
- Confirm the conservative policy naming/TODO makes the temporary scan policy
  clear enough until per-level control lands.
