# 30375 SPIRE Scheduled Routing Replacement Children — feedback

## What landed

`build_scheduled_routing_replacement_children` turns a checked scheduler
decision + PID plan + caller-supplied centroids into a list of
`SpireRoutingReplacementChild` ordered by PID-plan order. Validates
decision shape, fresh PIDs, PID/centroid count match, no duplicate or
zero PIDs, non-empty + finite centroids.

## Correctness

- This is the only helper in the scheduled arc that *does* check centroid
  shape (non-empty, all-finite) — appropriate, because routing-children
  validation is the canonical place for centroid-vector constraints.
  Dimension is enforced later by `validate_replacement_routing_children`
  inside the rewrite helper, against `parent.dimensions`.
- Order is preserved: replacement_pids[i] ↔ centroids[i] zip, no
  reordering. The PID-plan order propagates from here to the rewrite, the
  object writer, the placement directory, and the publish-plan-output
  validator.
- Reuse of `pid_plan.reuses_existing_pid` rejection (line 634) is correct
  — rebalance never reaches the scheduler.

## Status

Solid. Centroid-finiteness is the only place this constraint is checked
in the scheduled arc; the rewrite helper does check it again at line
2045, so the live scheduler is double-protected.
