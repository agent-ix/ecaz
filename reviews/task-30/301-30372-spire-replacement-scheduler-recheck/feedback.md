# 30372 SPIRE Replacement Scheduler Recheck — feedback

## What landed

`recheck_leaf_replacement_schedule_decision` recomputes the selected
decision from freshly loaded leaf-snapshot rows under the publish lock,
fails closed if the decision disappeared or changed before object writes.
Uses `leaf_replacement_schedule_decisions_match` (mode + active_epoch +
replaced_parent_pid + affected_leaf_pids + replacement_leaf_count) for
equivalence — `reason: &'static str` is intentionally not part of the
match, since it's diagnostic.

## Correctness

- Calls `choose_leaf_replacement_schedule` on the fresh rows, then
  compares against the previously-selected decision; if either side
  changed, fail closed. This is exactly the design's "reload under publish
  lock and re-check selected leaf PIDs" requirement.
- Disappearance (`Ok(None)` from the recheck) is treated as a failure
  rather than a no-op — caller cannot accidentally publish based on a
  stale decision.

## Concern (minor)

The recheck assumes the *same selection algorithm* will be invariant.
If `choose_leaf_replacement_schedule` is later tuned (e.g., a different
tie-break), a recheck on equivalent leaf state could spuriously reject a
still-valid decision. Worth a comment on `recheck_leaf_replacement_schedule_decision`
that the selector function and the recheck must move in lockstep.

## Status

Lands cleanly. Concurrency contract is properly anchored at this seam.
