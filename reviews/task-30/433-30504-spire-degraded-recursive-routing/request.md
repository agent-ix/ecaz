# Review Request: SPIRE Degraded Recursive Routing

Head SHA: `3d58a09a`

## Summary

Recursive scan coverage now includes degraded placement behavior for an
unavailable internal routing object. The test builds a two-level recursive
hierarchy with two internal children, marks the non-selected internal routing
object unavailable under degraded consistency, and verifies recursive routing
still descends through the available internal child.

This checkpoint also documents two deferred/intentional boundaries from the
Phase 3 review:

- `load_snapshot_routing_hierarchy` applies visibility and kind filtering only;
  level and parent/child coherence remains validated during descent by
  `require_recursive_internal_child`, where the expected parent context exists.
- `SpireRecursiveRoutingEpochDraft::centroid_records` are not persisted as
  standalone records yet; diagnostics rebuild them from routing objects until
  durable centroid objects land.

## Files

- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/scan.rs`

## Validation

- `cargo test recursive_routed_leaf_rows_skip_degraded_unavailable_unselected_internal -- --nocapture`
  - 1 passed:
    `recursive_routed_leaf_rows_skip_degraded_unavailable_unselected_internal`.
- `cargo fmt`
  - Completed with the repo's existing stable-rustfmt warnings about
    unstable import grouping options.
- `git diff --check`

## Review Focus

- Confirm this degraded recursive test covers the intended unavailable-internal
  placement path without over-specifying future degraded routing policy.
- Confirm the loader/descent validation comment captures the current boundary.
- Confirm the centroid-record TODO is placed at the right draft field.
