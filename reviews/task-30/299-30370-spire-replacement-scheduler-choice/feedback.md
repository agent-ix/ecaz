# 30370 SPIRE Replacement Scheduler Choice — feedback

## What landed

`choose_leaf_replacement_schedule` is a pure selector over leaf-snapshot
diagnostics: rejects rows spanning multiple active epochs, rejects
ambiguous rows (split + merge recommended on the same row), prefers the
largest split candidate, otherwise selects the sparsest same-parent merge
pair.

## Correctness

- Split-over-merge precedence is unconditional, matching the design
  intuition that growth pressure dominates sparsity — split first, then
  reconsider merge candidates next round.
- Merge-pair tie-breaking sorts by `(sum_effective, smaller_effective,
  smaller_pid, larger_pid)` (line 789-800) so the choice is deterministic
  across runs given the same snapshot.
- Rejection of rows spanning multiple `active_epoch` values is the right
  failure mode — caller should always pass a single-epoch snapshot.
- Rejection of `parent_pid == 0` for split/merge candidates is defensive:
  prevents the root from being scheduled as if it were a routed leaf.

## Status

Solid. The recheck step (30372) re-validates this decision under the
publish lock before object writes, so transient-snapshot drift between
selection and execution is caught explicitly rather than silently.
