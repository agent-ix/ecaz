# Review Request: SPIRE Recursive Draft Invariant Helper

Head SHA: `f2e9fe42`

## Summary

Recursive draft validation now has a single helper,
`assert_recursive_draft_invariants`, that runs the in-memory routing/centroid
shape checks and returns the derived leaf-parent PID map. The coordinator,
leaf-input epoch path, and placement epoch path now use that helper instead of
reassembling the valid-draft contract from separate calls.

`validate_recursive_epoch_leaf_placements` now receives the already-derived
leaf-parent map from the invariant helper, avoiding another independent
`recursive_routing_leaf_parent_pids` call inside the placement validator.

## Files

- `src/am/ec_spire/build.rs`

## Validation

- `cargo test local_recursive_routing_epoch_from_leaf_inputs -- --nocapture`
  - 2 passed:
    `local_recursive_routing_epoch_from_leaf_inputs_writes_leaf_objects`,
    `local_recursive_routing_epoch_from_leaf_inputs_rejects_parent_drift`.
- `cargo test recursive_build_coordinator_assembles_epoch_input_from_centroid_plan -- --nocapture`
  - 1 passed: `recursive_build_coordinator_assembles_epoch_input_from_centroid_plan`.
- `cargo fmt`
  - Completed with the repo's existing stable-rustfmt warnings about
    unstable import grouping options.
- `git diff --check`

## Review Focus

- Confirm the helper is the right place to define the recursive draft invariant
  bundle.
- Confirm returning the leaf-parent PID map avoids the duplication called out
  in review without hiding the epoch placement checks.
- Confirm keeping the lower-level shape validator separate is still useful for
  targeted unit tests.
