# 30377 SPIRE Scheduled Replacement Publish Draft — feedback

## What landed

`build_scheduled_replacement_epoch_draft_from_object_placements` validates
decision shape, rejects active-snapshot/decision epoch mismatch, rejects
replacement leaf-placement count mismatch, then delegates placement-
directory + object-manifest + root-control shape validation to
`build_replacement_epoch_draft_from_object_placements` (30368).

## Correctness

- Snapshot-vs-decision active-epoch mismatch rejection (line 1095-1100)
  is the right guard: if the snapshot the caller is using to plan is
  newer than the decision, the decision is stale.
- Leaf placement count is checked against `decision.replacement_leaf_count`
  here, before `_parts` validation kicks in — so a malformed input fails
  at the most informative seam.

## Concern (carried forward)

The validators in this arc do *not* verify that the rewritten
`replacement_parent` actually contains the replacement child PIDs (they
check `replacement_parent.header.pid`, but not its children). The
correctness of that property depends on
`rewrite_scheduled_replacement_parent_routing` having been called
upstream. See feedback in 30388 for the cross-cutting concern.

## Status

Lands cleanly. Subsequent packets (30379, 30384) tightened this helper
further.
