# 30374 SPIRE Split Replacement Leaf Input — feedback

## What landed

`build_split_replacement_leaf_object_inputs` validates split decision
shape, requires fresh replacement PIDs, checks routed leaf inputs exactly
cover the planned replacement PIDs, orders the returned inputs by
PID-plan order, reuses `validate_replacement_leaf_object_inputs` for row
shape and `vec_id` uniqueness.

## Correctness

- Caller-routed inputs must contain *exactly* the planned replacement
  PIDs (line 528-534 + 552-554) — extras and missing both rejected.
- Output ordering is normalized to `pid_plan.replacement_pids` order so
  downstream object-write order, leaf-placement order, and
  routing-children order all align (the property 30380 verifies).
- Centroid training stays on the live scheduler side — same intentional
  empty-centroid pass-through as 30373.

## Status

Lands cleanly. Together with 30373 this closes the "post-routing
object-input shape" pre-check before live scheduler execution.
